//! Headless dogfood 端到端集成测试（P1.2 Phase A–D，关联 tasks/PROGRESS.md 当前 Epic）。
//!
//! 用 `sieve-testing` harness 串起完整 dogfood 验证：全程本地 mock 上游 + headless CLI
//! 当决策客户端，**不依赖真 API / 真网络 / 真 GUI**。每个测试独立 SIEVE_HOME（harness
//! 默认隔离）+ 独立端口（`find_free_port`）。
//!
//! 覆盖（命名 `phase_a_*` ~ `phase_d_*` / `phase_c2_*`）：
//! - Phase A：出站 OUT-01 AutoRedact（脱敏后转发，上游收不到原始 key）
//! - Phase B：入站 Critical 拦截（IN-CR-05-EVM / IN-CR-01）跨 Anthropic SSE + JSON + OpenAI SSE，
//!   外加 benign 反例
//! - Phase C：`--no-client-policy` 三策略（auto-block / auto-warn / hold-and-fail-closed）的
//!   HTTP 行为分流 + Critical（OUT-07）不受 policy 影响
//! - Phase C2：完整决策流——mock GUI 直连 IPC 回 Deny → 被 hold 的出站请求返回 426
//! - Phase D：审计基础设施闭环（audit.db schema + `sieve audit` CLI 行为）
//!
//! ## 实测确认的 daemon 真实行为（写测试时用探针核实，详见交付报告）
//!
//! 1. **`DaemonConfig::with_no_client_policy()`**：daemon 的 `no_client_policy` 只从 CLI flag
//!    `sieve start --no-client-policy` 读（harness 已改为传 CLI flag，2026-06-18 修）。Phase C
//!    用本测试内的 [`spawn_daemon_with_policy`]（直接 spawn 二进制 + CLI flag）等价。
//! 2. **`wait_for_ipc` 会污染 `connected_clients`**：IPC accept loop 对每个新连接立即
//!    `gui_writers.push(tx)` 且断开后 lazy 清理，探测连接残留 → `connected_clients()>0` →
//!    `gated_request_decision` 跳过 no-client 分支。Phase C 测「无 client」路径时**不调
//!    `wait_for_ipc`**（daemon 侧 eager 清理为待办，不阻塞）。
//! 3. **detection 审计已接线（2026-06-18 修）**：`gated_request_decision` 写 `DecisionMade`
//!    （所有 gui_popup 决策 + no-client-policy）、出站脱敏写 `OutboundRedacted`。Phase D-1 正向
//!    断言 OUT-01 脱敏可经 SQLite + `sieve audit query` 查到。
//! 4. **`sieve audit` / `sieve decisions` CLI nested-runtime panic（2026-06-18 修）**：`run()`
//!    改 async 委托 `run_async`、由 `main` 直接 `.await`，不再 `block_on`。Phase D-2 正向断言 CLI
//!    干净运行。
//!
//! .cursorrules §3.2：测试代码允许使用 `.unwrap()`。

use sieve_testing::daemon::{spawn_daemon, DaemonConfig};
use sieve_testing::http::http_post;
use sieve_testing::paths::{find_free_port, inbound_rules_path, outbound_rules_path, sieve_binary};
use sieve_testing::upstream::{responses, spawn_mock_streaming_upstream, spawn_mock_upstream};
use std::collections::HashMap;
use std::io::Write as _;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// ───────────────────────── 共享 payload / 工具 ─────────────────────────────────

/// 构造含 fake Anthropic key 的请求 body（触发 OUT-01，disposition=auto_redact）。
///
/// key 格式：`sk-ant-api03-` + 93 个 `[a-zA-Z0-9_-]` + `AA`（符合 OUT-01 pattern）。
fn out01_key_body() -> String {
    let suffix_93: String = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-"
        .chars()
        .cycle()
        .take(93)
        .collect();
    let api_key = format!("sk-ant-api03-{suffix_93}AA");
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": format!("leaked: {api_key}") }],
    })
    .to_string()
}

/// 含 JWT 的请求 body（触发 OUT-06，severity=high，disposition=gui_popup，
/// default_on_timeout=redact，**非** fail-closed → 受 no_client_policy 影响）。
fn out06_jwt_body() -> String {
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXN1YmplY3QifQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": format!("my token: {jwt}") }],
    })
    .to_string()
}

/// 含 PEM 私钥的请求 body（触发 OUT-07，severity=critical，disposition=gui_popup，
/// default_on_timeout=block，**在** fail-closed 名单 → 不受 no_client_policy 影响）。
fn out07_pem_body() -> String {
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{
            "role": "user",
            "content": "私钥：-----BEGIN EC PRIVATE KEY-----\nMHQCAQEEINsamplekey\n-----END EC PRIVATE KEY-----",
        }],
    })
    .to_string()
}

/// 阻塞 POST JSON，返回 `(status, headers, body)`；包 spawn_blocking 避免 current_thread 死锁。
async fn post_json(
    base_url: &str,
    path: &str,
    body: String,
) -> (u16, HashMap<String, String>, Vec<u8>) {
    let base = base_url.to_owned();
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        http_post(
            &base,
            &path,
            &[("content-type", "application/json")],
            body.as_bytes(),
        )
    })
    .await
    .unwrap()
}

/// 直接读 audit.db 的 `audit_events` 行（绕开 `sieve audit` CLI 的 runtime panic bug）。
///
/// 返回 `(rule_id, severity, direction, disposition, decision, provider_id)` 列表。
/// DB 不存在或表为空时返回空 Vec。
fn read_audit_rows(db: &Path) -> Vec<(String, String, String, String, Option<String>, String)> {
    if !db.exists() {
        return Vec::new();
    }
    let conn = rusqlite::Connection::open_with_flags(
        db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT rule_id, severity, direction, disposition, decision, provider_id \
             FROM audit_events ORDER BY id",
        )
        .unwrap();
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
            ))
        })
        .unwrap();
    rows.map(Result::unwrap).collect()
}

/// 用 CLI flag `--no-client-policy <policy>` 直接 spawn daemon（绕开 harness 坏掉的
/// `with_no_client_policy`，见文件头注释 §1）。返回 `(base_url, sieve_home, child)`。
///
/// **刻意不轮询 IPC**——Phase C 测「无 client」路径，调 `wait_for_ipc` 会污染
/// `connected_clients`（见文件头注释 §2）。只等 TCP listener 就绪。
struct PolicyDaemon {
    base_url: String,
    _home: tempfile::TempDir,
    proc: Child,
}

impl Drop for PolicyDaemon {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

fn spawn_daemon_with_policy(upstream_url: &str, policy: &str) -> Option<PolicyDaemon> {
    let rules = outbound_rules_path();
    if !rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过 ({})",
            rules.display()
        );
        return None;
    }
    let inbound_rules = inbound_rules_path();
    if !inbound_rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过 ({})",
            inbound_rules.display()
        );
        return None;
    }

    let port = find_free_port();
    let home = tempfile::tempdir().unwrap();

    let mut cfg = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        cfg,
        "upstream_url = \"{}\"\n\
         port = {}\n\
         bind_addr = \"127.0.0.1\"\n\
         rules_path = \"{}\"\n\
         inbound_rules_path = \"{}\"\n\
         tls_verify_upstream = false\n\
         dry_run = false\n",
        upstream_url,
        port,
        rules.display(),
        inbound_rules.display(),
    )
    .unwrap();
    cfg.flush().unwrap();
    // 进程运行期内 config 文件需存活；leak NamedTempFile 让路径不被回收。
    let cfg_path = cfg.path().to_owned();
    std::mem::forget(cfg);

    let proc = Command::new(sieve_binary())
        .arg("start")
        .arg("--config")
        .arg(&cfg_path)
        .arg("--no-client-policy")
        .arg(policy)
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sieve daemon with policy");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(300),
        )
        .is_ok()
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "daemon (policy={policy}) did not listen on :{port}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    Some(PolicyDaemon {
        base_url: format!("http://127.0.0.1:{port}"),
        _home: home,
        proc,
    })
}

// ════════════════════════════ Phase A：出站脱敏 ════════════════════════════════

/// Phase A：OUT-01 AutoRedact —— mock plain 上游记录收到的 body，daemon（默认 auto_redact）
/// 收到含 fake key 的请求 → 脱敏后转发上游 → 200，且上游 body 里原始 key 已被替换。
///
/// 关联 AutoRedact 路径 / OUT-01 / 硬约束 #13。
#[tokio::test]
async fn phase_a_outbound_out01_auto_redact_forwards_redacted() {
    let upstream_body: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = upstream_body.clone();

    let mock = spawn_mock_upstream(move |req| {
        let recorder = recorder.clone();
        async move {
            *recorder.lock().await = req.body().to_vec();
            responses::anthropic_json_response("ok-from-upstream")
        }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };

    let (status, _h, _body) = post_json(&guard.base_url(), "/v1/messages", out01_key_body()).await;
    assert_eq!(status, 200, "OUT-01 AutoRedact 应脱敏后转发上游 → 200");

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);
    assert!(
        !received_str.contains("sk-ant-api03-"),
        "脱敏后上游不应收到原始 key:\n{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "脱敏后上游 body 应含 REDACTED 占位符:\n{received_str}"
    );
}

/// Phase A · full 档：daemon 开启加密审计归档（write-only logging）后，含 OUT-01
/// 密钥的出站请求 → 脱敏后转发 + **加密归档段落盘**。红线断言：归档段文件**只含密文**，
/// 绝不出现原始 `sk-ant-api03-` 明文；用 `sieve audit decrypt`（口令解锁私钥）解开后，
/// 内容是脱敏后的 `[REDACTED:OUT-01]` 占位符——证明 daemon 喂给归档的是脱敏后内容。
#[tokio::test]
async fn phase_a_full_archive_stores_ciphertext_only_no_plaintext() {
    if !outbound_rules_path().exists() || !inbound_rules_path().exists() {
        eprintln!("SKIP phase_a_full_archive_stores_ciphertext_only_no_plaintext: 规则文件不存在（需安装签名规则包），跳过");
        return;
    }
    const PASS: &str = "e2e-archive-passphrase";
    let home = tempfile::tempdir().unwrap();

    // 1. 驱动真实 `sieve audit keygen` 生成密钥对（私钥写 home/audit-identity.age，公钥打印）。
    let keygen = Command::new(sieve_binary())
        .args(["audit", "keygen"])
        .env("SIEVE_HOME", home.path())
        .env("SIEVE_AUDIT_PASSPHRASE", PASS)
        .output()
        .expect("run sieve audit keygen");
    assert!(
        keygen.status.success(),
        "keygen 应成功:\n{}",
        String::from_utf8_lossy(&keygen.stderr)
    );
    let keygen_out = String::from_utf8_lossy(&keygen.stdout);
    let recipient = keygen_out
        .lines()
        .find_map(|l| {
            l.split_once("recipient = \"")
                .and_then(|(_, r)| r.strip_suffix('"'))
        })
        .map(|s| s.trim().to_owned())
        .expect("keygen 输出应含 recipient = \"age1...\"");
    assert!(
        recipient.starts_with("age1"),
        "recipient 应为 age1 公钥: {recipient}"
    );
    let identity_path = home.path().join("audit-identity.age");
    assert!(identity_path.exists(), "keygen 应写出口令保护的私钥文件");

    // 2. 起 mock 上游 + full 档 daemon（[audit] level=full + recipient）。
    let mock = spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let port = find_free_port();
    let mut cfg = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        cfg,
        "upstream_url = \"{}\"\n\
         port = {}\n\
         bind_addr = \"127.0.0.1\"\n\
         rules_path = \"{}\"\n\
         inbound_rules_path = \"{}\"\n\
         tls_verify_upstream = false\n\
         dry_run = false\n\
         [audit]\n\
         level = \"full\"\n\
         recipient = \"{}\"\n",
        mock.url(),
        port,
        outbound_rules_path().display(),
        inbound_rules_path().display(),
        recipient,
    )
    .unwrap();
    cfg.flush().unwrap();
    let cfg_path = cfg.path().to_owned();
    std::mem::forget(cfg);

    let mut proc = Command::new(sieve_binary())
        .arg("start")
        .arg("--config")
        .arg(&cfg_path)
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn full-档 daemon");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(300),
        )
        .is_ok()
        {
            break;
        }
        assert!(Instant::now() < deadline, "full-档 daemon 未监听 :{port}");
        std::thread::sleep(Duration::from_millis(100));
    }

    // 3. 发含 OUT-01 密钥的出站请求 → 脱敏后转发 + 归档。
    let base = format!("http://127.0.0.1:{port}");
    let (status, _h, _b) = post_json(&base, "/v1/messages", out01_key_body()).await;
    assert_eq!(status, 200, "full 档下 OUT-01 仍应脱敏后转发 200");
    // 等 fire-and-forget 归档（spawn_blocking）落盘。
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // 4. 红线断言：归档段文件存在且**只含密文**，无原始密钥明文。
    let archive_dir = home.path().join("audit-archive");
    let seg = std::fs::read_dir(&archive_dir)
        .expect("归档目录应存在")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("archive-") && n.ends_with(".jsonl"))
                .unwrap_or(false)
        })
        .expect("应有 archive-*.jsonl 归档段");
    let raw = std::fs::read(&seg).expect("读归档段");
    assert!(!raw.is_empty(), "归档段不应为空（出站脱敏应已归档一条）");
    let raw_str = String::from_utf8_lossy(&raw);
    assert!(
        !raw_str.contains("sk-ant-api03-"),
        "红线违反：归档段含原始密钥明文！必须只存密文"
    );

    // 5. 用 `sieve audit decrypt` 解开，断言是脱敏后内容（含 REDACTED，无明文密钥）。
    let decrypt = Command::new(sieve_binary())
        .args(["audit", "decrypt"])
        .arg("--identity")
        .arg(&identity_path)
        .arg(&seg)
        .env("SIEVE_HOME", home.path())
        .env("SIEVE_AUDIT_PASSPHRASE", PASS)
        .output()
        .expect("run sieve audit decrypt");
    assert!(
        decrypt.status.success(),
        "decrypt 应成功（哈希链校验 + age 解密）:\n{}",
        String::from_utf8_lossy(&decrypt.stderr)
    );
    let plain = String::from_utf8_lossy(&decrypt.stdout);
    assert!(
        plain.contains("REDACTED"),
        "解密内容应是脱敏后（含 REDACTED 占位符）:\n{plain}"
    );
    assert!(
        !plain.contains("sk-ant-api03-"),
        "解密内容不应含原始密钥明文（脱敏先于落盘）:\n{plain}"
    );

    let _ = proc.kill();
    let _ = proc.wait();
}

/// Phase A · billing daemon（relay 上游）收到 relay **虚高 usage** 的 JSON 响应 →
/// 独立 token 核算检出超额 → `usage.db` 落 `overbilled` 记录（严格本地）。这是「relay 虚报
/// 被检出」回归的 daemon 级端到端验证。
#[tokio::test]
async fn phase_a_billing_detects_relay_usage_inflation() {
    if !outbound_rules_path().exists() || !inbound_rules_path().exists() {
        eprintln!("SKIP phase_a_billing_detects_relay_usage_inflation: 规则文件不存在（需安装签名规则包），跳过");
        return;
    }
    // mock relay 对极短内容（"hi"）声明 16 万 token（远超实际）→ 必判 overbilled。
    let mock = spawn_mock_upstream(|_req| async {
        responses::anthropic_json_response_with_usage("hi", 80_000, 80_000)
    })
    .await;
    let home = tempfile::tempdir().unwrap();
    let port = find_free_port();
    let mut cfg = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        cfg,
        "upstream_url = \"{}\"\n\
         port = {}\n\
         bind_addr = \"127.0.0.1\"\n\
         rules_path = \"{}\"\n\
         inbound_rules_path = \"{}\"\n\
         tls_verify_upstream = false\n\
         dry_run = false\n\
         [billing_check]\n\
         enabled = true\n\
         tolerance_pct = 15.0\n",
        mock.url(),
        port,
        outbound_rules_path().display(),
        inbound_rules_path().display(),
    )
    .unwrap();
    cfg.flush().unwrap();
    let cfg_path = cfg.path().to_owned();
    std::mem::forget(cfg);

    let mut proc = Command::new(sieve_binary())
        .arg("start")
        .arg("--config")
        .arg(&cfg_path)
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn billing daemon");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(300),
        )
        .is_ok()
        {
            break;
        }
        assert!(Instant::now() < deadline, "billing daemon 未监听 :{port}");
        std::thread::sleep(Duration::from_millis(100));
    }

    let base = format!("http://127.0.0.1:{port}");
    let benign = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hello world" }],
    })
    .to_string();
    let (status, _h, _b) = post_json(&base, "/v1/messages", benign).await;
    assert_eq!(status, 200, "benign 请求应 200 透传");
    // 等 fire-and-forget usage 观测落库。
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let usage_db = home.path().join("usage.db");
    assert!(usage_db.exists(), "billing 启用应建 usage.db");
    let conn = rusqlite::Connection::open_with_flags(
        &usage_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap();
    let (verdict, trust, dev): (String, String, Option<f64>) = conn
        .query_row(
            "SELECT verdict, trust, deviation_pct FROM usage_records ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("usage_records 应有记录");
    assert_eq!(
        trust, "relay",
        "127.0.0.1 mock 上游应判 relay（非官方 host）"
    );
    assert_eq!(
        verdict, "overbilled",
        "relay 虚高 usage 应被检出 overbilled"
    );
    assert!(
        dev.unwrap_or(0.0) > 15.0,
        "偏差应远超容差 15%，实际 {dev:?}"
    );

    let _ = proc.kill();
    let _ = proc.wait();
}

// ── 超额计费检测【四路径全覆盖】（同触发=relay usage 虚报，渲染 4 种 wire）──
//
// phase_a_billing_detects_relay_usage_inflation（上）= Route Anthropic JSON。
// 下面补齐 Anthropic SSE / OpenAI JSON / OpenAI SSE，确保 billing 观测器不是 JSON-only
// （v1.5.4 P0 教训：每个入站特性必须四路径证明）。

/// 起一个开启超额计费检测的 daemon（relay 上游=mock；127.0.0.1 非官方 host → relay）。
struct BillingDaemon {
    base_url: String,
    home: tempfile::TempDir,
    proc: Child,
}
impl Drop for BillingDaemon {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

fn spawn_billing_daemon(upstream_url: &str) -> Option<BillingDaemon> {
    if !outbound_rules_path().exists() || !inbound_rules_path().exists() {
        eprintln!("SKIP: 规则文件不存在（需安装签名规则包），跳过");
        return None;
    }
    let port = find_free_port();
    let home = tempfile::tempdir().unwrap();
    let mut cfg = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        cfg,
        "upstream_url = \"{}\"\n\
         port = {}\n\
         bind_addr = \"127.0.0.1\"\n\
         rules_path = \"{}\"\n\
         inbound_rules_path = \"{}\"\n\
         tls_verify_upstream = false\n\
         dry_run = false\n\
         [billing_check]\n\
         enabled = true\n\
         tolerance_pct = 15.0\n",
        upstream_url,
        port,
        outbound_rules_path().display(),
        inbound_rules_path().display(),
    )
    .unwrap();
    cfg.flush().unwrap();
    let cfg_path = cfg.path().to_owned();
    std::mem::forget(cfg);

    let proc = Command::new(sieve_binary())
        .arg("start")
        .arg("--config")
        .arg(&cfg_path)
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn billing daemon");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(300),
        )
        .is_ok()
        {
            break;
        }
        assert!(Instant::now() < deadline, "billing daemon 未监听 :{port}");
        std::thread::sleep(Duration::from_millis(100));
    }
    Some(BillingDaemon {
        base_url: format!("http://127.0.0.1:{port}"),
        home,
        proc,
    })
}

/// 断言 usage.db 最新一条记录是 relay / overbilled / 偏差 >15%。
fn assert_last_usage_overbilled(home: &Path) {
    let usage_db = home.join("usage.db");
    assert!(usage_db.exists(), "billing 应建 usage.db");
    let conn = rusqlite::Connection::open_with_flags(
        &usage_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap();
    let (verdict, trust, dev): (String, String, Option<f64>) = conn
        .query_row(
            "SELECT verdict, trust, deviation_pct FROM usage_records ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("usage_records 应有记录");
    assert_eq!(
        trust, "relay",
        "127.0.0.1 mock 上游应判 relay（非官方 host）"
    );
    assert_eq!(verdict, "overbilled", "relay 虚高 usage 应检出 overbilled");
    assert!(
        dev.unwrap_or(0.0) > 15.0,
        "偏差应远超容差 15%，实际 {dev:?}"
    );
}

fn anthropic_billing_req() -> String {
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hello world" }],
    })
    .to_string()
}
fn openai_billing_req() -> String {
    serde_json::json!({
        "model": "gpt-4o",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hello world" }],
    })
    .to_string()
}

/// Route 2/4 · **Anthropic SSE**：relay 在 `message_start.usage.input_tokens` +
/// `message_delta.usage.output_tokens` 虚报 → SSE 观测器累计后检出。
#[tokio::test]
async fn phase_a_billing_anthropic_sse_overbilled() {
    let payload = responses::anthropic_sse_bytes_with_usage("hi", 80_000, 80_000);
    let mock = spawn_mock_streaming_upstream("text/event-stream", move |_req| {
        let p = payload.clone();
        async move { (hyper::StatusCode::OK, p) }
    })
    .await;
    let Some(d) = spawn_billing_daemon(&mock.url()) else {
        return;
    };
    let (status, _h, _b) = post_json(&d.base_url, "/v1/messages", anthropic_billing_req()).await;
    assert_eq!(status, 200, "Anthropic SSE benign 应 200 透传");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_last_usage_overbilled(d.home.path());
}

/// Route 3/4 · **OpenAI JSON**：relay 在顶层 `usage.prompt_tokens/completion_tokens` 虚报。
#[tokio::test]
async fn phase_a_billing_openai_json_overbilled() {
    let mock = spawn_mock_upstream(|_req| async {
        responses::openai_json_response_with_usage("hi", 80_000, 80_000)
    })
    .await;
    let Some(d) = spawn_billing_daemon(&mock.url()) else {
        return;
    };
    let (status, _h, _b) =
        post_json(&d.base_url, "/v1/chat/completions", openai_billing_req()).await;
    assert_eq!(status, 200, "OpenAI JSON benign 应 200 透传");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_last_usage_overbilled(d.home.path());
}

/// Route 4/4 · **OpenAI SSE**：relay 在末尾 usage chunk（`choices:[]` + `usage`）虚报。
/// 依赖 `OpenAiSseParser` 把 usage-only chunk 归一化为 `MessageDelta`（usage 观测扩展）。
#[tokio::test]
async fn phase_a_billing_openai_sse_overbilled() {
    let payload = responses::openai_sse_bytes_with_usage("hi", 80_000, 80_000);
    let mock = spawn_mock_streaming_upstream("text/event-stream", move |_req| {
        let p = payload.clone();
        async move { (hyper::StatusCode::OK, p) }
    })
    .await;
    let Some(d) = spawn_billing_daemon(&mock.url()) else {
        return;
    };
    let (status, _h, _b) =
        post_json(&d.base_url, "/v1/chat/completions", openai_billing_req()).await;
    assert_eq!(status, 200, "OpenAI SSE benign 应 200 透传");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_last_usage_overbilled(d.home.path());
}

// ════════════════════════════ Phase B：入站拦截 ════════════════════════════════

/// Phase B-1：Anthropic SSE —— 上游返回含 `eth_signTransaction` tool_use 的流式响应 →
/// daemon 注入 `sieve_blocked` + IN-CR-05 rule_id（IN-CR-05-EVM，Critical fail-closed）。
///
/// 关联 IN-CR-05 / 硬约束 #16（content-type 路由矩阵）。
#[tokio::test]
async fn phase_b_inbound_anthropic_sse_blocks_signing_tool() {
    let payload =
        responses::anthropic_tool_use_sse_bytes("eth_signTransaction", "{\"to\":\"0xabc\"}");
    let mock = spawn_mock_streaming_upstream("text/event-stream", move |_req| {
        let p = payload.clone();
        async move { (hyper::StatusCode::OK, p) }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5", "max_tokens": 16, "stream": true,
        "messages": [{ "role": "user", "content": "hi" }],
    })
    .to_string();
    let (status, _h, raw) = post_json(&guard.base_url(), "/v1/messages", body).await;
    let body_str = String::from_utf8_lossy(&raw);

    assert_eq!(status, 200, "上游 200 应保留（sieve_blocked 注入 body）");
    assert!(
        body_str.contains("sieve_blocked"),
        "Anthropic SSE 应注入 sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-05"),
        "应含 IN-CR-05 rule_id:\n{body_str}"
    );
}

/// Phase B-2：Anthropic 非流式 JSON —— 含 `eth_signTransaction` tool_use 的 JSON 响应被
/// 替换为含 `sieve_blocked` + IN-CR-05。覆盖硬约束 #16 的 Anthropic JSON 组合。
///
/// 关联 lessons.md 2026-04-27（非流式 JSON 入站漏检漏洞）/ IN-CR-05。
#[tokio::test]
async fn phase_b_inbound_anthropic_json_blocks_signing_tool() {
    let json_body = serde_json::json!({
        "id": "msg_01", "type": "message", "role": "assistant", "model": "claude-sonnet-4-5",
        "content": [{
            "type": "tool_use", "id": "toolu_01", "name": "eth_signTransaction",
            "input": { "to": "0xdeadbeef", "value": "1000000000000000000" }
        }],
        "stop_reason": "tool_use",
        "usage": { "input_tokens": 10, "output_tokens": 50 }
    });
    let body_bytes = bytes::Bytes::from(json_body.to_string());
    let mock = spawn_mock_upstream(move |_req| {
        let b = body_bytes.clone();
        async move {
            http::Response::builder()
                .status(200)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(http_body_util::Full::new(b))
                .unwrap()
        }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    // 无 stream:true → 非流式 JSON 路径
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5", "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hi" }],
    })
    .to_string();
    let (_status, _h, raw) = post_json(&guard.base_url(), "/v1/messages", body).await;
    let body_str = String::from_utf8_lossy(&raw);

    assert!(
        body_str.contains("sieve_blocked"),
        "非流式 JSON 含签名工具应触发 sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-05"),
        "应含 IN-CR-05 rule_id:\n{body_str}"
    );

    // 审计完整性（2026-06-18 接线，硬约束 #16 anthropic_json 路径）：该入站 block 必须落
    // audit（direction=inbound + disposition=blocked）。fire-and-forget spawn，sleep 后再读。
    tokio::time::sleep(Duration::from_millis(800)).await;
    let rows = read_audit_rows(&guard.audit_db());
    assert!(
        rows.iter()
            .any(|(rule, _sev, direction, disposition, _dec, _provider)| {
                direction == "inbound" && disposition == "blocked" && rule.contains("IN-CR-05")
            }),
        "anthropic JSON 入站 block 应落 inbound/blocked 审计行；实际: {rows:?}"
    );
}

/// Phase B-3：OpenAI SSE —— prompt seed 地址 A，上游 SSE 仅含近似地址 B（Levenshtein=1）→
/// IN-CR-01 截流注入 sieve_blocked。覆盖 OpenAI SSE 组合 + 地址替换算法路径。
///
/// 关联 R9-#1 / IN-CR-01 / OpenAI 协议路径分发。
#[tokio::test]
async fn phase_b_inbound_openai_sse_blocks_address_substitution() {
    // prompt 含地址 A（...1234A），SSE 仅含地址 B（...1234B，末字符 A→B，Levenshtein=1）。
    let payload = responses::openai_sse_bytes(&[
        "please send to ",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f1234B",
    ]);
    let mock = spawn_mock_streaming_upstream("text/event-stream", move |_req| {
        let p = payload.clone();
        async move { (hyper::StatusCode::OK, p) }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let body = serde_json::json!({
        "model": "gpt-4o", "stream": true,
        "messages": [{ "role": "user", "content": "please transfer to 0x742d35Cc6634C0532925a3b844Bc9e7595f1234A from my wallet" }],
    })
    .to_string();
    let (status, _h, raw) = post_json(&guard.base_url(), "/v1/chat/completions", body).await;
    let body_str = String::from_utf8_lossy(&raw);

    assert_eq!(status, 200, "上游 200 应保留");
    assert!(
        body_str.contains("sieve_blocked"),
        "OpenAI SSE 地址替换应注入 sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-01"),
        "应含 IN-CR-01 rule_id:\n{body_str}"
    );

    // 审计完整性（2026-06-18 接线，openai_sse 路径）：该入站 block 应落 inbound/blocked 审计。
    // fire-and-forget spawn，sleep 后再读（SSE 路径落库窗口可能略长，给充足时间）。
    tokio::time::sleep(Duration::from_millis(800)).await;
    let rows = read_audit_rows(&guard.audit_db());
    assert!(
        rows.iter()
            .any(|(rule, _sev, direction, disposition, _dec, _provider)| {
                direction == "inbound" && disposition == "blocked" && rule.contains("IN-CR-01")
            }),
        "OpenAI SSE 入站 block 应落 inbound/blocked 审计行；实际: {rows:?}"
    );
}

/// Phase B-3b：OpenAI 非流式 JSON —— `choices[].message.tool_calls` 含 `eth_signTransaction`
/// 的 JSON 响应被替换为含 `sieve_blocked` + IN-CR-05，且该入站 block 落 audit（inbound/blocked）。
///
/// 补 content-type 矩阵 openai_json 路径的 audit 覆盖——此前 OpenAI 入站 block 测试
/// 只有 SSE 路径，JSON 路径无 block 审计断言（重蹈「只挂一条路径」P0 漏洞风险）。
/// 关联硬约束 #16 / handle_openai_json_inbound / IN-CR-05-EVM。
#[tokio::test]
async fn phase_b_inbound_openai_json_blocks_signing_tool() {
    // IN-CR-05-EVM 触发 payload：choices[].message.tool_calls 含 eth_signTransaction。
    let attack_json = serde_json::json!({
        "id": "chatcmpl-01", "object": "chat.completion", "created": 1_700_000_000u64,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant", "content": serde_json::Value::Null,
                "tool_calls": [{
                    "id": "tc1", "type": "function",
                    "function": {
                        "name": "eth_signTransaction",
                        "arguments": "{\"to\":\"0xdeadbeef\",\"value\":\"1000000000000000000\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": { "prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30 }
    });
    let attack_bytes = bytes::Bytes::from(attack_json.to_string());
    let mock = spawn_mock_upstream(move |_req| {
        let b = attack_bytes.clone();
        async move {
            http::Response::builder()
                .status(200)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(http_body_util::Full::new(b))
                .unwrap()
        }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    // 无 stream:true → 非流式 JSON 路径（handle_openai_json_inbound）。
    let body = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"run"}]}"#.to_string();
    let (status, _h, raw) = post_json(&guard.base_url(), "/v1/chat/completions", body).await;
    let body_str = String::from_utf8_lossy(&raw);

    assert_eq!(status, 200, "上游 200 应保留（sieve_blocked 注入 body）");
    assert!(
        body_str.contains("sieve_blocked"),
        "OpenAI 非流式 JSON 含签名工具应触发 sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-05"),
        "应含 IN-CR-05 rule_id:\n{body_str}"
    );

    // 审计完整性（2026-06-18 接线，硬约束 #16 openai_json 路径）：入站 block 必须落
    // audit（direction=inbound + disposition=blocked）。fire-and-forget spawn，sleep 后读。
    tokio::time::sleep(Duration::from_millis(800)).await;
    let rows = read_audit_rows(&guard.audit_db());
    assert!(
        rows.iter()
            .any(|(rule, _sev, direction, disposition, _dec, _provider)| {
                direction == "inbound" && disposition == "blocked" && rule.contains("IN-CR-05")
            }),
        "openai JSON 入站 block 应落 inbound/blocked 审计行；实际: {rows:?}"
    );
}

/// Phase B-4（反例）：benign Anthropic SSE（无攻击 payload）→ 正常透传，无 sieve_blocked。
///
/// 关联 Critical 拦截 FP < 0.5%。
#[tokio::test]
async fn phase_b_inbound_benign_sse_passes_through() {
    let payload = responses::anthropic_sse_bytes(&["Hello, ", "how can I help you today?"]);
    let mock = spawn_mock_streaming_upstream("text/event-stream", move |_req| {
        let p = payload.clone();
        async move { (hyper::StatusCode::OK, p) }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5", "max_tokens": 16, "stream": true,
        "messages": [{ "role": "user", "content": "hi" }],
    })
    .to_string();
    let (status, _h, raw) = post_json(&guard.base_url(), "/v1/messages", body).await;
    let body_str = String::from_utf8_lossy(&raw);

    assert_eq!(status, 200, "benign 应正常透传 200");
    assert!(
        !body_str.contains("sieve_blocked"),
        "benign 响应不应注入 sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("message_stop"),
        "benign 响应应完整透传（含 message_stop）:\n{body_str}"
    );
    assert!(
        body_str.contains("Hello"),
        "benign 响应应保留原始文本:\n{body_str}"
    );
}

// ══════════════════════ Phase C：no-client-policy 三策略 ════════════════════════

/// Phase C-1：三策略对 **non-critical** gui_popup（OUT-06 JWT）的 HTTP 行为分流。
///
/// 实测（探针核实）daemon 无 client 时的真实分流：
/// - `auto-block`            → 426（Deny，最保守 fail-closed）
/// - `auto-warn`            → 200（Allow 放行）
/// - `hold-and-fail-closed` → 200（无 GUI 立即 fallback 到 default_on_timeout=redact → 脱敏 200）
///
/// 关联 no_client_policy。
#[tokio::test]
async fn phase_c_three_policies_diverge_on_noncritical() {
    // 每个 policy 独立 mock 上游 + 独立 daemon（独立 SIEVE_HOME + 端口）。
    let mock_block =
        spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let Some(d_block) = spawn_daemon_with_policy(&mock_block.url(), "auto-block") else {
        return;
    };
    let (s_block, _h, _b) = post_json(&d_block.base_url, "/v1/messages", out06_jwt_body()).await;
    assert_eq!(
        s_block, 426,
        "auto-block + non-critical gui_popup 应 Deny → 426"
    );

    let mock_warn =
        spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let Some(d_warn) = spawn_daemon_with_policy(&mock_warn.url(), "auto-warn") else {
        return;
    };
    let (s_warn, _h, _b) = post_json(&d_warn.base_url, "/v1/messages", out06_jwt_body()).await;
    assert_eq!(
        s_warn, 200,
        "auto-warn + non-critical gui_popup 应 Allow → 200"
    );

    let mock_hold =
        spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let Some(d_hold) = spawn_daemon_with_policy(&mock_hold.url(), "hold-and-fail-closed") else {
        return;
    };
    let (s_hold, _h, _b) = post_json(&d_hold.base_url, "/v1/messages", out06_jwt_body()).await;
    assert_eq!(
        s_hold, 200,
        "hold-and-fail-closed + OUT-06(default=redact) 无 GUI 应 fallback 脱敏 → 200"
    );
}

/// Phase C-2：**Critical**（OUT-07 PEM，fail-closed）不受 no_client_policy 影响 —— 三策略
/// 全部 426。验证硬约束 #3 #8（Critical 强制走 IPC，不被 auto-allow / auto-warn 放行）。
///
/// 关联硬约束 #3 #8 / `!any_critical` 守卫 / critical_lock::FAIL_CLOSED_RULES。
#[tokio::test]
async fn phase_c_critical_ignores_policy_always_426() {
    for policy in ["auto-block", "auto-warn", "hold-and-fail-closed"] {
        let mock =
            spawn_mock_upstream(|_req| async { responses::anthropic_json_response("nope") }).await;
        let Some(daemon) = spawn_daemon_with_policy(&mock.url(), policy) else {
            return;
        };
        let (status, _h, _b) = post_json(&daemon.base_url, "/v1/messages", out07_pem_body()).await;
        assert_eq!(
            status, 426,
            "Critical OUT-07 在 policy={policy} 下应仍 fail-closed 426（不受 policy 影响）"
        );
    }
}

// ══════════════════════ Phase C2：完整决策流（mock GUI）═══════════════════════

/// 模拟 GUI 客户端：连 IPC socket → 通知 ready → 跳过 hello/heartbeat 通知帧 → 解析
/// `sieve.request_decision` 的真实 request_id → 用传入 decision 回 DecisionResponse。
///
/// lift 自 `outbound_block.rs::mock_gui_respond_with_ready`（簇 B 教训：必须 method 正过滤，
/// 否则对 hello 握手帧解析 request_id 失败 → 断连 → daemon fallback Block 污染断言）。
async fn mock_gui_respond(
    socket_path: &Path,
    decision: sieve_ipc::DecisionAction,
    ready_tx: tokio::sync::oneshot::Sender<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let mut stream = None;
    for _ in 0..200 {
        match UnixStream::connect(socket_path).await {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    let stream = stream.ok_or("IPC socket not ready after 20s")?;
    // 让 IPC server 完成 handle_connection spawn + gui_writer 注册。
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _ = ready_tx.send(());

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let rpc: serde_json::Value = serde_json::from_str(&line)?;
        if rpc.get("method").and_then(|m| m.as_str()) != Some("sieve.request_decision") {
            continue; // 跳过 hello / heartbeat 等通知帧
        }
        let params = rpc.get("params").ok_or("no params")?;
        let real_id: uuid::Uuid =
            serde_json::from_value(params["request_id"].clone()).map_err(|e| e.to_string())?;

        let resp = sieve_ipc::protocol::DecisionResponse {
            request_id: real_id,
            decision,
            decided_at: chrono::Utc::now(),
            by_user: true,
            remember: false,
            context_hint: None,
            ui_phase_when_clicked: None,
        };
        let rpc_resp = sieve_ipc::protocol::jsonrpc::Response {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::to_value(&resp)?),
            error: None,
            id: serde_json::Value::String(real_id.to_string()),
        };
        let mut payload = serde_json::to_string(&rpc_resp)?;
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await?;
        break;
    }
    Ok(())
}

/// Phase C2（完整决策流）：mock GUI 连 IPC 当决策客户端 → 出站 OUT-07 hold 等决策 →
/// GUI 回 Deny → 被 hold 的请求返回 426，上游未被调用。
///
/// 这是 dogfood「headless CLI 当决策客户端」的核心闭环（任务允许用直连 IPC 方式替代
/// `sieve decisions resolve`——后者因 nested-runtime bug panic 不可用，见文件头注释 §4）。
///
/// 关联 R2-#1（出站 HoldForDecision）/ 二维处置矩阵 / no_client_policy。
#[tokio::test]
async fn phase_c2_mock_gui_deny_returns_426() {
    let upstream_hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let hits = upstream_hits.clone();
    let mock = spawn_mock_upstream(move |_req| {
        let hits = hits.clone();
        async move {
            hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            responses::anthropic_json_response("should-not-reach")
        }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let socket = guard.ipc_socket();

    // mock GUI 后台连 IPC（connected_clients>0，避开 no-client 默认 auto-block），回 Deny。
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond(&socket, sieve_ipc::DecisionAction::Deny, ready_tx).await;
    });
    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    let (status, _h, _b) = post_json(&guard.base_url(), "/v1/messages", out07_pem_body()).await;
    let _ = ipc_task.await;

    assert_eq!(status, 426, "GUI Deny → 出站 hold 请求应返回 426");
    assert_eq!(
        upstream_hits.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "GUI Deny 后上游不应被调用"
    );
}

/// Phase C2（反例）：mock GUI 回 Allow → 同一 OUT-07 hold 请求放行到上游（200）。
///
/// 与 Deny 对照，证明决策客户端的 Allow/Deny 双向都被 daemon 正确路由到被 hold 的 HTTP 请求。
#[tokio::test]
async fn phase_c2_mock_gui_allow_forwards_to_upstream() {
    let upstream_hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let hits = upstream_hits.clone();
    let mock = spawn_mock_upstream(move |_req| {
        let hits = hits.clone();
        async move {
            hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            responses::anthropic_json_response("upstream-ok")
        }
    })
    .await;

    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let socket = guard.ipc_socket();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond(&socket, sieve_ipc::DecisionAction::Allow, ready_tx).await;
    });
    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    let (status, _h, _b) = post_json(&guard.base_url(), "/v1/messages", out07_pem_body()).await;
    let _ = ipc_task.await;

    assert_eq!(status, 200, "GUI Allow → 出站 hold 请求应放行到上游 200");
    assert_eq!(
        upstream_hits.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "GUI Allow 后上游应被调用一次"
    );
}

// ════════════════════════════ Phase D：审计闭环 ════════════════════════════════

/// Phase D-1：detection 审计已接线 —— OUT-01 出站脱敏写入 `OutboundRedacted` 审计行，
/// 且可经 headless `sieve audit query` CLI 查到（直接读 SQLite + CLI 双重验证）。
///
/// 此前 daemon 对 detection 流量（OUT-* 脱敏 / IN-CR-* 拦截 / GUI 决策）**零 audit 写入**
/// （headless dogfood e2e 抓出，2026-06-18）；已于同日接线：`gated_request_decision` 写
/// `DecisionMade`（所有 gui_popup 决策 + no-client-policy）、出站脱敏写 `OutboundRedacted`。
///
/// 关联审计数据模型 / provider_id 列。
#[tokio::test]
async fn phase_d_detection_audit_wired_and_queryable() {
    let mock = spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };

    // 跑出站 OUT-01（脱敏）+ benign，制造 detection 流量。
    let _ = post_json(&guard.base_url(), "/v1/messages", out01_key_body()).await;
    let benign = serde_json::json!({
        "model": "claude-sonnet-4-5", "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hello world" }],
    })
    .to_string();
    let _ = post_json(&guard.base_url(), "/v1/messages", benign).await;

    // audit 写入是异步 spawn 的；给落库窗口。
    tokio::time::sleep(Duration::from_millis(800)).await;

    let db = guard.audit_db();
    assert!(db.exists(), "audit.db 应已由 daemon 初始化");

    // 正向断言①（直接读 SQLite）：OUT-01 脱敏应写入一条 OUT-01 detection 审计行，
    // 含 provider_id（v3 schema 列）。
    let detection_rows: Vec<_> = read_audit_rows(&db)
        .into_iter()
        .filter(|(rule, ..)| rule.starts_with("OUT-") || rule.starts_with("IN-"))
        .collect();
    assert!(
        detection_rows
            .iter()
            .any(|(rule, _sev, _dir, _disp, _dec, provider)| {
                rule.starts_with("OUT-01") && !provider.is_empty()
            }),
        "OUT-01 脱敏应写入带 provider_id 的 OutboundRedacted 审计行；实际: {detection_rows:?}"
    );

    // 正向断言②（headless CLI）：`sieve audit query` 能查到该 OUT-01 行（nested-runtime
    // panic 已修，CLI 现可用作 dogfood 审计查询入口）。
    let home = guard.sieve_home().to_owned();
    let out = tokio::task::spawn_blocking(move || {
        sieve_testing::cli::run_sieve_cli_with_home(&["audit", "query", "--format", "jsonl"], &home)
    })
    .await
    .unwrap();
    assert!(
        out.status.success(),
        "sieve audit query 应成功: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OUT-01"),
        "sieve audit query jsonl 输出应含 OUT-01 detection 行: {stdout}"
    );

    // 正向断言③（入站拦截审计，2026-06-18 接线）：入站 Critical fail-closed block
    // 此前一律不落 audit（真机 dogfood 抓出），现 spawn_inbound_blocked_audit 补齐。
    // 复用 phase_b 的 eth_signTransaction → IN-CR-05 触发一次 anthropic JSON block，
    // 断言 audit 出现 direction=inbound + disposition=blocked + rule_id 含 IN-CR-05 的行。
    let json_block_body = serde_json::json!({
        "id": "msg_01", "type": "message", "role": "assistant", "model": "claude-sonnet-4-5",
        "content": [{
            "type": "tool_use", "id": "toolu_01", "name": "eth_signTransaction",
            "input": { "to": "0xdeadbeef", "value": "1000000000000000000" }
        }],
        "stop_reason": "tool_use",
        "usage": { "input_tokens": 10, "output_tokens": 50 }
    });
    let block_bytes = bytes::Bytes::from(json_block_body.to_string());
    let block_mock = spawn_mock_upstream(move |_req| {
        let b = block_bytes.clone();
        async move {
            http::Response::builder()
                .status(200)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(http_body_util::Full::new(b))
                .unwrap()
        }
    })
    .await;
    let Some(block_guard) = spawn_daemon(DaemonConfig::new(block_mock.url())) else {
        return;
    };
    let req_body = serde_json::json!({
        "model": "claude-sonnet-4-5", "max_tokens": 16,
        "messages": [{ "role": "user", "content": "hi" }],
    })
    .to_string();
    let (_s, _h, raw) = post_json(&block_guard.base_url(), "/v1/messages", req_body).await;
    let raw_str = String::from_utf8_lossy(&raw);
    assert!(
        raw_str.contains("sieve_blocked") && raw_str.contains("IN-CR-05"),
        "前置条件：入站 JSON 应被 IN-CR-05 拦截:\n{raw_str}"
    );

    // fire-and-forget 审计（tokio::spawn），断言前给落库窗口（同 phase_d 上面 800ms）。
    tokio::time::sleep(Duration::from_millis(800)).await;
    let block_db = block_guard.audit_db();
    let inbound_block_rows = read_audit_rows(&block_db);
    assert!(
        inbound_block_rows.iter().any(
            |(rule, _sev, direction, disposition, _dec, _provider)| {
                direction == "inbound" && disposition == "blocked" && rule.contains("IN-CR-05")
            }
        ),
        "入站 anthropic JSON block 应写入 direction=inbound + disposition=blocked + IN-CR-05 审计行；实际: {inbound_block_rows:?}"
    );

    // 正向断言④（`--severity` 过滤回归，2026-06-18 真机 dogfood 抓出）：审计列存**小写**
    // severity，此前 `run_query` 用首字母大写字面量 "Critical" 匹配小写列，致 `--severity`
    // 过滤**永远返回空**；现 `LOWER(severity)` 大小写不敏感。用 block_guard 的 critical
    // 入站行端到端验证 CLI 真实路径（单测走 query_rows 绕过了 run_query 的 match）。
    let block_home = block_guard.sieve_home().to_owned();
    let sev_out = tokio::task::spawn_blocking(move || {
        sieve_testing::cli::run_sieve_cli_with_home(
            &[
                "audit",
                "query",
                "--severity",
                "critical",
                "--format",
                "jsonl",
            ],
            &block_home,
        )
    })
    .await
    .unwrap();
    assert!(
        sev_out.status.success(),
        "sieve audit query --severity critical 应成功: {}",
        String::from_utf8_lossy(&sev_out.stderr)
    );
    let sev_stdout = String::from_utf8_lossy(&sev_out.stdout);
    assert!(
        sev_stdout.contains("IN-CR-05"),
        "sieve audit query --severity critical 应查到 critical 入站拦截行\
         （此前大写字面量匹配小写列致空）；实际: {sev_stdout}"
    );
}

// 注：`wait_for_ipc` 在 Phase C 故意不用（会污染 connected_clients，见文件头注释 §2）；
// Phase C2 用 mock_gui_respond 内部自带 socket 轮询 + ready 信号，不依赖 wait_for_ipc。

/// Phase D-2：`sieve audit query` CLI 正常运行（nested-runtime panic 已修复，回归锚点）。
///
/// 此前 `main()` 是 `#[tokio::main]` 而 `commands::audit::run` 内又
/// `Builder::new_current_thread().block_on()` → "Cannot start a runtime from within a
/// runtime" panic（exit 134），`sieve audit` 任一子命令完全不可用。已于 2026-06-18 修复
/// （`run` 改 async 委托 `run_async`，由 `main` 直接 `.await`，见 tasks/lessons.md）。
///
/// 本测试断言：CLI 干净退出（无 nested-runtime panic）、stdout 是合法 jsonl（可空）。
/// detection 行的正向验证见 [`phase_d_detection_audit_wired_and_queryable`]。
///
/// 关联 `sieve audit` CLI。
#[tokio::test]
async fn phase_d_audit_cli_runs_without_nested_runtime_panic() {
    let mock = spawn_mock_upstream(|_req| async { responses::anthropic_json_response("ok") }).await;
    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        return;
    };
    let _ = post_json(&guard.base_url(), "/v1/messages", out01_key_body()).await;
    tokio::time::sleep(Duration::from_millis(300)).await;

    let home = guard.sieve_home().to_owned();
    let out = tokio::task::spawn_blocking(move || {
        sieve_testing::cli::run_sieve_cli_with_home(&["audit", "query", "--format", "jsonl"], &home)
    })
    .await
    .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Cannot start a runtime from within a runtime"),
        "nested-runtime panic 不应再出现：{stderr}"
    );
    assert!(
        out.status.success(),
        "sieve audit query 应干净退出（audit.db 已由 daemon 创建）；stderr={stderr}"
    );
    // stdout 每个非空行必须是合法 JSON（jsonl 格式）。可为空（detection 未接线，见下个测试）。
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("audit jsonl 行非法 JSON: {e}: {line}"));
    }
}
