#!/usr/bin/env python3
"""Sieve daemon smoke test。

自动启动 sieve daemon -> 跑透传断言 -> 结束 daemon。

依赖:Python 3.10+,仅 stdlib(无需 pip)。

用法:
    scripts/smoke_test.py --mock-only                     # hermetic：本地 mock 上游，无需真 key/网络（CI 用）
    scripts/smoke_test.py                                 # 仅无 key 透传测试（打真 api.anthropic.com）
    ANTHROPIC_API_KEY=sk-ant-... scripts/smoke_test.py    # 加 200/SSE/tool_use（真 key + 真网络）
    scripts/smoke_test.py --port 12000                    # 指定端口
    scripts/smoke_test.py --debug                         # daemon stderr 打到屏幕

--mock-only 下启动一个本地 mock Anthropic 上游（http），daemon 经 tls_verify_upstream=false
转发到它。fake key → 401（注入 cloudflare 头，模拟真上游）；valid key → 200/SSE/tool_use。
全套断言（含 200/SSE/tool_use/benign 透传）无真 key 无网络即可跑。出站拦截(426)由 daemon
自身完成，与上游无关。
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import http.client
import http.server
import json
import os
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Iterator
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DAEMON_BIN_CANDIDATES = [
    REPO_ROOT / "target" / "release" / "sieve",
    REPO_ROOT / "target" / "debug" / "sieve",
]


def red(s: str) -> str:
    return f"\033[31m{s}\033[0m"


def green(s: str) -> str:
    return f"\033[32m{s}\033[0m"


def bold(s: str) -> str:
    return f"\033[1m{s}\033[0m"


def dim(s: str) -> str:
    return f"\033[2m{s}\033[0m"


@dataclasses.dataclass
class Stats:
    passed: int = 0
    failed: int = 0

    def add(self, ok: bool) -> None:
        if ok:
            self.passed += 1
        else:
            self.failed += 1

    @property
    def total(self) -> int:
        return self.passed + self.failed


# ──────────── daemon 生命周期 ────────────


def find_daemon_binary() -> Path:
    for p in DAEMON_BIN_CANDIDATES:
        if p.exists():
            return p
    raise FileNotFoundError(
        f"sieve binary not found in {[str(p) for p in DAEMON_BIN_CANDIDATES]}\n"
        f"先跑:cargo build --release -p sieve-cli"
    )


def find_free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def write_config(
    port: int,
    upstream: str = "https://api.anthropic.com",
    tls_verify: bool = True,
) -> Path:
    rules_path = REPO_ROOT / "crates" / "sieve-rules" / "rules" / "outbound.toml"
    inbound_rules_path = REPO_ROOT / "crates" / "sieve-rules" / "rules" / "inbound.toml"
    f = tempfile.NamedTemporaryFile(
        mode="w", suffix=".toml", prefix="sieve-smoke-", delete=False
    )
    f.write(
        f'upstream_url = "{upstream}"\n'
        f"port = {port}\n"
        f'bind_addr = "127.0.0.1"\n'
        f'rules_path = "{rules_path}"\n'
        f'inbound_rules_path = "{inbound_rules_path}"\n'
    )
    if not tls_verify:
        # mock 上游是 plain HTTP，关掉上游 TLS 校验（与 Rust 集成测试一致）。
        f.write("tls_verify_upstream = false\n")
    f.close()
    return Path(f.name)


@contextlib.contextmanager
def sieve_daemon(
    port: int,
    debug: bool = False,
    upstream: str = "https://api.anthropic.com",
    tls_verify: bool = True,
) -> Iterator[subprocess.Popen[bytes]]:
    binary = find_daemon_binary()
    config = write_config(port, upstream, tls_verify)
    print(dim(f"  binary:   {binary}"))
    print(dim(f"  config:   {config}"))
    print(dim(f"  upstream: {upstream}"))
    print(dim(f"  port:     {port}"))

    stderr_dst = None if debug else subprocess.DEVNULL
    stdout_dst = None if debug else subprocess.DEVNULL
    proc = subprocess.Popen(
        [str(binary), "start", "--config", str(config)],
        stdout=stdout_dst,
        stderr=stderr_dst,
        # smoke test 绝不联网做更新/遥测，保持 hermetic。
        env={
            **os.environ,
            "SIEVE_LOG": "warn",
            "SIEVE_NO_UPDATE": "1",
            "SIEVE_NO_TELEMETRY": "1",
        },
    )
    try:
        wait_for_listen(port, timeout=10.0)
        print(green(f"  ✓ daemon up (pid={proc.pid})"))
        yield proc
    finally:
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()
        config.unlink(missing_ok=True)
        print(dim(f"  daemon stopped (exit={proc.returncode})"))


def wait_for_listen(port: int, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.1)
    raise TimeoutError(f"daemon did not listen on :{port} within {timeout}s")


# ──────────── mock 上游(--mock-only,去真 API 依赖)────────────

# mock 模式下 real 测试用的「有效 key」(任何不含 "fake" 的 key 都被 mock 视为有效)。
MOCK_VALID_KEY = "sk-ant-mock-valid-key"

# mock 上游收到的请求体(供出站脱敏断言：验证 key 在转发前已被 daemon 脱敏)。
# CPython list.append/clear 是原子的，线程间共享安全。
_MOCK_RECEIVED: list[bytes] = []


def _compact(obj: object) -> str:
    """紧凑 JSON(无空格),匹配真 Anthropic wire 格式(断言找 '"type":"message"')。"""
    return json.dumps(obj, separators=(",", ":"))


def _sse(events: list[tuple[str, dict[str, object]]]) -> bytes:
    """把 (event_name, data) 列表编码为 Anthropic SSE wire 格式。"""
    return "".join(
        f"event: {name}\ndata: {_compact(data)}\n\n" for name, data in events
    ).encode()


def _mock_json_message() -> bytes:
    return _compact(
        {
            "id": "msg_mock_0001",
            "type": "message",
            "role": "assistant",
            "model": "claude-haiku-4-5",
            "content": [{"type": "text", "text": "ok"}],
            "stop_reason": "end_turn",
            "stop_sequence": None,
            "usage": {"input_tokens": 5, "output_tokens": 1},
        }
    ).encode()


def _mock_text_sse() -> bytes:
    msg = {
        "id": "msg_mock_0002",
        "type": "message",
        "role": "assistant",
        "model": "claude-haiku-4-5",
        "content": [],
        "stop_reason": None,
        "usage": {"input_tokens": 5, "output_tokens": 0},
    }
    return _sse(
        [
            ("message_start", {"type": "message_start", "message": msg}),
            (
                "content_block_start",
                {"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}},
            ),
            ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "1"}}),
            ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "2"}}),
            ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "3"}}),
            ("content_block_stop", {"type": "content_block_stop", "index": 0}),
            ("message_delta", {"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 3}}),
            ("message_stop", {"type": "message_stop"}),
        ]
    )


def _mock_tool_use_sse() -> bytes:
    msg = {
        "id": "msg_mock_0003",
        "type": "message",
        "role": "assistant",
        "model": "claude-haiku-4-5",
        "content": [],
        "stop_reason": None,
        "usage": {"input_tokens": 8, "output_tokens": 0},
    }
    return _sse(
        [
            ("message_start", {"type": "message_start", "message": msg}),
            (
                "content_block_start",
                {
                    "type": "content_block_start",
                    "index": 0,
                    "content_block": {"type": "tool_use", "id": "toolu_mock_1", "name": "get_weather", "input": {}},
                },
            ),
            ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": {"type": "input_json_delta", "partial_json": '{"city":'}}),
            ("content_block_delta", {"type": "content_block_delta", "index": 0, "delta": {"type": "input_json_delta", "partial_json": '"Beijing"}'}}),
            ("content_block_stop", {"type": "content_block_stop", "index": 0}),
            ("message_delta", {"type": "message_delta", "delta": {"stop_reason": "tool_use"}, "usage": {"output_tokens": 12}}),
            ("message_stop", {"type": "message_stop"}),
        ]
    )


class _MockUpstreamHandler(http.server.BaseHTTPRequestHandler):
    """本地 mock Anthropic 上游：按 x-api-key 与请求体形状返回确定性响应。"""

    def log_message(self, *_args: object) -> None:  # 静音访问日志
        return

    def do_POST(self) -> None:  # noqa: N802 (http.server 接口约定)
        length = int(self.headers.get("content-length", "0") or "0")
        raw = self.rfile.read(length) if length else b""
        _MOCK_RECEIVED.append(raw)
        api_key = self.headers.get("x-api-key", "")
        try:
            body = json.loads(raw)
        except (ValueError, UnicodeDecodeError):
            self._send(
                400,
                b'{"type":"error","error":{"type":"invalid_request_error","message":"mock: bad json"},"request_id":"req_mock_400"}',
            )
            return
        # 空/含 "fake" 的 key → 401（模拟真 Anthropic 拒假 key），注入 cloudflare 头以保留原断言。
        if (not api_key) or ("fake" in api_key):
            self._send(
                401,
                b'{"type":"error","error":{"type":"authentication_error","message":"mock: invalid x-api-key"},"request_id":"req_mock_401"}',
                extra_headers={"server": "cloudflare", "cf-ray": "0000000000000000-SJC"},
            )
            return
        stream = bool(body.get("stream"))
        tools = bool(body.get("tools"))
        if stream and tools:
            self._send(200, _mock_tool_use_sse(), content_type="text/event-stream")
        elif stream:
            self._send(200, _mock_text_sse(), content_type="text/event-stream")
        else:
            self._send(200, _mock_json_message())

    def _send(
        self,
        status: int,
        body: bytes,
        content_type: str = "application/json",
        extra_headers: dict[str, str] | None = None,
    ) -> None:
        self.send_response(status)
        self.send_header("content-type", content_type)
        self.send_header("content-length", str(len(body)))
        for k, v in (extra_headers or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(body)


def start_mock_upstream() -> tuple[http.server.ThreadingHTTPServer, int]:
    """在随机端口起 mock 上游，返回 (server, port)。调用方负责 shutdown()。"""
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), _MockUpstreamHandler)
    port = srv.server_address[1]
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv, port


# ──────────── HTTP helpers(stdlib only)────────────


def http_request(
    base_url: str,
    method: str,
    path: str,
    headers: dict[str, str] | None = None,
    body: bytes | None = None,
    timeout: float = 30.0,
) -> tuple[int, dict[str, str], bytes]:
    """同步 HTTP 请求(buffered)。"""
    url = base_url.rstrip("/") + path
    req = urllib.request.Request(
        url, method=method, headers=headers or {}, data=body
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, {k.lower(): v for k, v in resp.headers.items()}, resp.read()
    except urllib.error.HTTPError as e:
        return e.code, {k.lower(): v for k, v in e.headers.items()}, e.read()


def http_stream_request(
    base_url: str,
    path: str,
    headers: dict[str, str],
    body: bytes,
    timeout: float = 30.0,
) -> tuple[int, dict[str, str], bytes]:
    """流式读响应,collect 完整 body(用于 SSE)。"""
    parsed = urllib.parse.urlparse(base_url)
    assert parsed.hostname and parsed.port, f"invalid base_url: {base_url}"
    conn = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=timeout)
    try:
        conn.request("POST", path, body=body, headers=headers)
        resp = conn.getresponse()
        status = resp.status
        resp_headers = {k.lower(): v for k, v in resp.getheaders()}
        chunks: list[bytes] = []
        while True:
            chunk = resp.read(4096)
            if not chunk:
                break
            chunks.append(chunk)
        return status, resp_headers, b"".join(chunks)
    finally:
        conn.close()


# ──────────── 断言 ────────────


def _format_snippet(hay: object, limit: int = 200) -> str:
    if isinstance(hay, bytes):
        return repr(hay[:limit]) + ("..." if len(hay) > limit else "")
    s = str(hay)
    return s[:limit] + ("..." if len(s) > limit else "")


def assert_eq(name: str, expected: object, actual: object, stats: Stats) -> None:
    ok = expected == actual
    print(green(f"  ✓ {name}") if ok else red(f"  ✗ {name}"))
    if not ok:
        print(red(f"    expected: {expected!r}"))
        print(red(f"    actual:   {actual!r}"))
    stats.add(ok)


def assert_contains(name: str, needle: str | bytes, hay: str | bytes, stats: Stats) -> None:
    if isinstance(hay, bytes) and isinstance(needle, str):
        needle_b: bytes = needle.encode()
        ok = needle_b in hay
    elif isinstance(hay, str) and isinstance(needle, bytes):
        ok = needle.decode("utf-8", errors="replace") in hay
    else:
        ok = needle in hay  # type: ignore[operator]
    print(green(f"  ✓ {name}") if ok else red(f"  ✗ {name}"))
    if not ok:
        print(red(f"    looking for: {needle!r}"))
        print(red(f"    in:          {_format_snippet(hay)}"))
    stats.add(ok)


# ──────────── 测试用例 ────────────


FAKE_KEY = "sk-ant-fake-smoke-test-key"


def test_no_key_passthrough(base_url: str, stats: Stats) -> None:
    print(bold("\n[2] 无 API key:401 透传(字节级)"))
    status, headers, body = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": FAKE_KEY,
        },
        body=json.dumps(
            {
                "model": "claude-sonnet-4-5",
                "max_tokens": 16,
                "messages": [{"role": "user", "content": "hi"}],
            }
        ).encode(),
    )
    assert_eq("HTTP 401", 401, status, stats)
    assert_eq("server header = cloudflare", "cloudflare", headers.get("server", ""), stats)
    assert_contains("body 含 request_id", "request_id", body, stats)
    assert_contains("body 是 Anthropic 错误格式", b'"type":"error"', body, stats)
    assert_contains("error.type = authentication_error", b"authentication_error", body, stats)
    assert_eq("cf-ray header 存在", True, "cf-ray" in headers, stats)


def test_bad_request(base_url: str, stats: Stats) -> None:
    print(bold("\n[3] 非法请求体:4xx 透传"))
    status, _, _ = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": FAKE_KEY,
        },
        body=b"not even json",
    )
    assert_eq("status 是 4xx", True, 400 <= status < 500, stats)
    print(dim(f"    实际 status: {status}"))


def test_large_body(base_url: str, stats: Stats) -> None:
    print(bold("\n[4] 大 body 透传(~8KB)"))
    big_body = json.dumps(
        {
            "model": "claude-sonnet-4-5",
            "max_tokens": 16,
            "messages": [{"role": "user", "content": "x" * 8000}],
        }
    ).encode()
    status, _, _ = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": FAKE_KEY,
        },
        body=big_body,
    )
    assert_eq("8KB body 上游响应 4xx/5xx 而非 connection error", True, 400 <= status < 600, stats)
    print(dim(f"    实际 status: {status}, body size: {len(big_body)}B"))


def test_concurrent(base_url: str, stats: Stats, n: int = 20) -> None:
    print(bold(f"\n[5] 并发 {n} 路 daemon 不崩溃"))
    statuses: list[int] = []
    lock = threading.Lock()

    def one() -> None:
        try:
            s, _, _ = http_request(
                base_url,
                "POST",
                "/v1/messages",
                headers={
                    "content-type": "application/json",
                    "anthropic-version": "2023-06-01",
                    "x-api-key": FAKE_KEY,
                },
                body=b'{"model":"claude-sonnet-4-5","max_tokens":1,"messages":[{"role":"user","content":"x"}]}',
                timeout=15.0,
            )
        except (urllib.error.URLError, OSError, TimeoutError):
            s = 0
        with lock:
            statuses.append(s)

    threads = [threading.Thread(target=one) for _ in range(n)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    ok = sum(1 for s in statuses if s == 401)
    threshold = n - 2
    assert_eq(f"≥ {threshold}/{n} 路返回 401", True, ok >= threshold, stats)
    print(dim(f"    实际:{ok}/{n}"))


def test_real_non_streaming(base_url: str, stats: Stats, api_key: str) -> None:
    print(bold("\n[6] 真 key:非流式 200"))
    status, _, body = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": api_key,
        },
        body=json.dumps(
            {
                "model": "claude-haiku-4-5",
                "max_tokens": 16,
                "messages": [{"role": "user", "content": "reply with the single word: ok"}],
            }
        ).encode(),
    )
    assert_eq("HTTP 200", 200, status, stats)
    assert_contains("body 含 message 类型", b'"type":"message"', body, stats)
    assert_contains("body 含 assistant role", b'"role":"assistant"', body, stats)
    assert_contains("body 含 stop_reason", b"stop_reason", body, stats)


def test_real_streaming(base_url: str, stats: Stats, api_key: str) -> None:
    print(bold("\n[7] 真 key:流式 SSE"))
    status, _, body = http_stream_request(
        base_url,
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": api_key,
        },
        body=json.dumps(
            {
                "model": "claude-haiku-4-5",
                "max_tokens": 32,
                "stream": True,
                "messages": [{"role": "user", "content": "count to 3"}],
            }
        ).encode(),
    )
    assert_eq("HTTP 200", 200, status, stats)
    assert_contains("event: message_start", b"event: message_start", body, stats)
    assert_contains("event: content_block_delta", b"event: content_block_delta", body, stats)
    assert_contains("event: message_stop", b"event: message_stop", body, stats)
    assert_contains("SSE 事件以 \\n\\n 分隔", b"\n\n", body, stats)


def test_real_tool_use(base_url: str, stats: Stats, api_key: str) -> None:
    print(bold("\n[8] 真 key:tool_use 流式"))
    status, _, body = http_stream_request(
        base_url,
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": api_key,
        },
        body=json.dumps(
            {
                "model": "claude-haiku-4-5",
                "max_tokens": 256,
                "stream": True,
                "tools": [
                    {
                        "name": "get_weather",
                        "description": "get current weather",
                        "input_schema": {
                            "type": "object",
                            "properties": {"city": {"type": "string"}},
                            "required": ["city"],
                        },
                    }
                ],
                "messages": [{"role": "user", "content": "weather in Beijing"}],
            }
        ).encode(),
    )
    assert_eq("HTTP 200", 200, status, stats)
    assert_contains("tool_use content_block", b'"type":"tool_use"', body, stats)
    assert_contains("partial_json 流式增量", b"input_json_delta", body, stats)


def test_outbound_redact_fake_key(
    base_url: str, stats: Stats, mock_only: bool
) -> None:
    print(bold("\n[9] 出站脱敏:fake Anthropic key → OUT-01 auto_redact 转发"))
    # OUT-01 处置 = auto_redact（severity critical，但 disposition 优先于 action=block，
    # 见 ADR-016 二维处置矩阵 + PRD v1.4 §6.1）：daemon 脱敏后转发上游，**不返 426**。
    # 构造符合 OUT-01 pattern 的 fake key：sk-ant-api03- + 93 chars + AA
    suffix_93 = ("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-" * 2)[:93]
    fake_key = f"sk-ant-api03-{suffix_93}AA"
    body = json.dumps({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{"role": "user", "content": f"leaked: {fake_key}"}],
    }).encode()
    # mock 模式用 valid key（走干净 200 转发路径）；真模式保留 FAKE_KEY。
    request_key = MOCK_VALID_KEY if mock_only else FAKE_KEY
    if mock_only:
        _MOCK_RECEIVED.clear()
    status, _, body_resp = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": request_key,
        },
        body=body,
    )
    # auto_redact ≠ block：不应是 426 / sieve_blocked。
    assert_eq("非 426（OUT-01 是 auto_redact 不 block）", True, status != 426, stats)
    assert_eq("非 sieve_blocked", False, b"sieve_blocked" in body_resp, stats)
    if mock_only:
        # 关键安全断言：转发到上游的 body 里原始 key 已被脱敏（不再出现）。
        forwarded = b"".join(_MOCK_RECEIVED)
        assert_eq("上游收到已转发请求", True, len(_MOCK_RECEIVED) >= 1, stats)
        assert_eq(
            "转发 body 已脱敏（原始 key 不出现）",
            False,
            fake_key.encode() in forwarded,
            stats,
        )
    else:
        print(dim(f"    实际 status: {status}（真模式无法窥探上游 body，脱敏由 mock 模式断言）"))


def test_benign_passes_through(base_url: str, stats: Stats, api_key: str) -> None:
    print(bold("\n[10] benign 消息透传(真 key)"))
    body = json.dumps({
        "model": "claude-haiku-4-5",
        "max_tokens": 16,
        "messages": [{"role": "user", "content": "say ok"}],
    }).encode()
    status, _, _ = http_request(
        base_url,
        "POST",
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": api_key,
        },
        body=body,
    )
    assert_eq("HTTP 200(benign 不被拦截)", 200, status, stats)


def test_inbound_rules_loaded_benign_passes(base_url: str, stats: Stats, api_key: str) -> None:
    print(bold("\n[11] 入站规则集成后 benign 流式仍正常透传"))
    body = json.dumps({
        "model": "claude-haiku-4-5",
        "max_tokens": 32,
        "stream": True,
        "messages": [{"role": "user", "content": "say hello in one short sentence"}],
    }).encode()
    status, _, body_resp = http_stream_request(
        base_url,
        "/v1/messages",
        headers={
            "content-type": "application/json",
            "anthropic-version": "2023-06-01",
            "x-api-key": api_key,
        },
        body=body,
    )
    assert_eq("HTTP 200", 200, status, stats)
    # 关键：body 不含 sieve_blocked event（benign 响应不被入站规则误判）
    assert_eq("无 sieve_blocked event", False, b"sieve_blocked" in body_resp, stats)
    assert_contains("含 message_stop event", b"event: message_stop", body_resp, stats)


# ──────────── main ────────────


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Sieve daemon smoke test(自动启停 daemon)"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=0,
        help="daemon listen port(默认让 OS 分配空闲端口)",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="把 daemon stderr 打到屏幕,便于排查",
    )
    parser.add_argument(
        "--mock-only",
        action="store_true",
        help="用本地 mock 上游(无需真 ANTHROPIC_API_KEY/网络),CI hermetic 用",
    )
    args = parser.parse_args()

    port = args.port or find_free_port()
    base_url = f"http://127.0.0.1:{port}"
    api_key = os.environ.get("ANTHROPIC_API_KEY")

    upstream = "https://api.anthropic.com"
    tls_verify = True
    mock_srv: http.server.ThreadingHTTPServer | None = None
    if args.mock_only:
        mock_srv, mock_port = start_mock_upstream()
        upstream = f"http://127.0.0.1:{mock_port}"
        tls_verify = False
        # mock 模式下 real 测试也跑——用「有效」mock key 打到 mock 上游。
        api_key = MOCK_VALID_KEY
        print(bold(f"[mock] 本地 mock 上游 @ {upstream}"))

    stats = Stats()
    print(bold(f"[1] 启动 daemon @ {base_url}"))

    try:
        with sieve_daemon(
            port, debug=args.debug, upstream=upstream, tls_verify=tls_verify
        ):
            test_no_key_passthrough(base_url, stats)
            test_bad_request(base_url, stats)
            test_large_body(base_url, stats)
            test_concurrent(base_url, stats)
            if api_key:
                test_real_non_streaming(base_url, stats, api_key)
                test_real_streaming(base_url, stats, api_key)
                test_real_tool_use(base_url, stats, api_key)
            else:
                print(bold("\n[6-8] 真 API key 测试"))
                print(dim("  ⊘ 跳过(未设 ANTHROPIC_API_KEY 且非 --mock-only)"))
            # Week 2 新增：出站脱敏验证（OUT-01 auto_redact，不依赖真 key/上游）
            test_outbound_redact_fake_key(base_url, stats, args.mock_only)
            if api_key:
                test_benign_passes_through(base_url, stats, api_key)
                # Week 3 新增：入站规则集成后 benign 流式仍正常透传
                test_inbound_rules_loaded_benign_passes(base_url, stats, api_key)
    except FileNotFoundError as e:
        print(red(f"\n  ✗ {e}"))
        return 2
    except TimeoutError as e:
        print(red(f"\n  ✗ {e}"))
        return 2
    finally:
        if mock_srv is not None:
            mock_srv.shutdown()

    print(bold("\n结果"))
    if stats.failed == 0:
        print(green(f"  {stats.passed}/{stats.total} 通过"))
        return 0
    print(red(f"  {stats.passed}/{stats.total} 通过,{stats.failed} 失败"))
    return 1


if __name__ == "__main__":
    sys.exit(main())
