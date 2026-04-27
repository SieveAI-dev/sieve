#!/usr/bin/env python3
"""Sieve daemon smoke test。

自动启动 sieve daemon -> 跑透传断言 -> 结束 daemon。

依赖:Python 3.10+,仅 stdlib(无需 pip)。

用法:
    scripts/smoke_test.py                                 # 仅无 key 透传测试
    ANTHROPIC_API_KEY=sk-ant-... scripts/smoke_test.py    # 加 200/SSE/tool_use
    scripts/smoke_test.py --port 12000                    # 指定端口
    scripts/smoke_test.py --debug                         # daemon stderr 打到屏幕
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import http.client
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


def write_config(port: int, upstream: str = "https://api.anthropic.com") -> Path:
    f = tempfile.NamedTemporaryFile(
        mode="w", suffix=".toml", prefix="sieve-smoke-", delete=False
    )
    f.write(
        f'upstream_url = "{upstream}"\n'
        f"port = {port}\n"
        f'bind_addr = "127.0.0.1"\n'
    )
    f.close()
    return Path(f.name)


@contextlib.contextmanager
def sieve_daemon(port: int, debug: bool = False) -> Iterator[subprocess.Popen[bytes]]:
    binary = find_daemon_binary()
    config = write_config(port)
    print(dim(f"  binary:  {binary}"))
    print(dim(f"  config:  {config}"))
    print(dim(f"  port:    {port}"))

    stderr_dst = None if debug else subprocess.DEVNULL
    stdout_dst = None if debug else subprocess.DEVNULL
    proc = subprocess.Popen(
        [str(binary), "start", "--config", str(config)],
        stdout=stdout_dst,
        stderr=stderr_dst,
        env={**os.environ, "SIEVE_LOG": "warn"},
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
    args = parser.parse_args()

    port = args.port or find_free_port()
    base_url = f"http://127.0.0.1:{port}"
    api_key = os.environ.get("ANTHROPIC_API_KEY")

    stats = Stats()
    print(bold(f"[1] 启动 daemon @ {base_url}"))

    try:
        with sieve_daemon(port, debug=args.debug):
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
                print(dim("  ⊘ 跳过(未设 ANTHROPIC_API_KEY 环境变量)"))
    except FileNotFoundError as e:
        print(red(f"\n  ✗ {e}"))
        return 2
    except TimeoutError as e:
        print(red(f"\n  ✗ {e}"))
        return 2

    print(bold("\n结果"))
    if stats.failed == 0:
        print(green(f"  {stats.passed}/{stats.total} 通过"))
        return 0
    print(red(f"  {stats.passed}/{stats.total} 通过,{stats.failed} 失败"))
    return 1


if __name__ == "__main__":
    sys.exit(main())
