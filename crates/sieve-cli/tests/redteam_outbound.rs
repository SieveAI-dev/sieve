//! ADR-043 红队 bypass 测试集——出站方向（request-side 脱敏）。
//!
//! **这是已知攻击手法的回归基线，不是检测能力的完备性证明。**
//! 红队集只驱动密钥样本并断言期望处置（脱敏改写 / 伪样本放行），**不新增任何
//! 检测规则**；检测规则定义由签名规则包提供、随更新通道分发。规则包缺失时本测试
//! 优雅 SKIP（打印 SKIP 并 return），绝不 panic / fail——这是公开仓的预期态。
//!
//! 覆盖（ADR-043 §红队 bypass 用例清单 #6）：
//! - BIP39 真助记词（真 SHA-256 checksum）→ 出站请求体被脱敏（recall）。
//! - Bitcoin WIF（5HueCGU…）→ 被脱敏。
//! - BIP-32 xprv（xprv9s21…）→ 被脱敏。
//!
//! 全程 hermetic：mock 上游记录 sieve 转发给上游的请求体，断言原始密钥不出现、
//! 出现 `REDACTED` 占位符（脱敏后转发，不弹窗——ADR-043 出站脱敏自动改写约束）。
//!
//! 密钥样本是公开的测试向量（BIP39 / WIF / BIP-32 规范示例），写在测试里是回归样本，
//! 不构成检测规则定义。
//!
//! 注：危险 shell 字符串（变量间接 / 子 shell / eval+base64）属入站响应方向，
//! 放在 `redteam_inbound.rs`，本文件只覆盖出站请求体密钥脱敏。
//!
//! .cursorrules §3.2：测试代码允许使用 .unwrap()。

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::convert::Infallible;
use std::io::Write as _;
use std::net::{SocketAddr, TcpListener as StdListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ─── 基础设施（照搬 outbound_block.rs 模式）─────────────────────────────────────

fn find_free_port() -> u16 {
    let l = StdListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // sieve-cli → crates/
    p.pop(); // crates/ → workspace root
    p
}

fn sieve_binary() -> PathBuf {
    let root = workspace_root();
    let release = root.join("target/release/sieve");
    if release.exists() {
        return release;
    }
    root.join("target/debug/sieve")
}

fn outbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/outbound.toml")
}

fn inbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/inbound.toml")
}

/// 在 :0 端口启动 plain-HTTP mock 上游，返回 (addr, shutdown sender)。
async fn spawn_mock_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
where
    F: Fn(Request<Bytes>) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Response<Full<Bytes>>> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, mut rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                accept = listener.accept() => {
                    let Ok((stream, _)) = accept else { continue };
                    let io = TokioIo::new(stream);
                    let r = responder.clone();
                    tokio::spawn(async move {
                        let svc = service_fn(move |req: Request<Incoming>| {
                            let r = r.clone();
                            async move {
                                let (parts, body) = req.into_parts();
                                let bytes = body.collect().await.unwrap_or_default().to_bytes();
                                let req_collected = Request::from_parts(parts, bytes);
                                Ok::<_, Infallible>(r(req_collected).await)
                            }
                        });
                        let _ = server_http1::Builder::new().serve_connection(io, svc).await;
                    });
                }
            }
        }
    });

    (addr, tx)
}

struct DaemonGuard {
    proc: Child,
    _config_file: tempfile::NamedTempFile,
    _sieve_home: tempfile::TempDir,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

/// 启动真实 sieve daemon，返回 (listen_port, guard)。
///
/// 规则文件缺失（公开仓无签名规则包）→ 打印 SKIP 并返回 None，调用方据此优雅跳过。
fn spawn_sieve_daemon(upstream_url: &str) -> Option<(u16, DaemonGuard)> {
    let port = find_free_port();
    let rules = outbound_rules_path();
    if !rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过红队出站 ({})",
            rules.display()
        );
        return None;
    }
    let inbound_rules = inbound_rules_path();
    if !inbound_rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过红队出站 ({})",
            inbound_rules.display()
        );
        return None;
    }

    let sieve_home = tempfile::tempdir().unwrap();
    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"upstream_url = "{upstream_url}"
port = {port}
bind_addr = "127.0.0.1"
rules_path = "{rules}"
inbound_rules_path = "{inbound_rules}"
tls_verify_upstream = false
dry_run = false
"#,
        rules = rules.display(),
        inbound_rules = inbound_rules.display()
    )
    .unwrap();

    let binary = sieve_binary();
    assert!(
        binary.exists(),
        "sieve binary not found: {}; run `cargo build` first",
        binary.display()
    );

    let proc = Command::new(&binary)
        .arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        // ADR-030: 测试禁止触发真实更新检查联网 + telemetry 上报
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", sieve_home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sieve daemon");

    wait_for_http_ready(port, Duration::from_secs(10));

    Some((
        port,
        DaemonGuard {
            proc,
            _config_file: config_file,
            _sieve_home: sieve_home,
        },
    ))
}

/// 等 daemon TCP listener 就绪（HTTP-level probe 在 #[tokio::test] 上会死锁，见
/// outbound_block.rs::wait_for_http_ready 注释）。
fn wait_for_http_ready(port: u16, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
        {
            return;
        }
        if Instant::now() >= deadline {
            panic!("sieve daemon did not listen on :{port} within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn plain_http_client() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

// ─── 红队样本：出站密钥脱敏（真校验和 → 必须脱敏）─────────────────────────────────
//
// 公开测试向量：
// - BIP39：标准 12 词全 "abandon … about"（真 SHA-256 checksum，BIP39 规范示例）。
// - Bitcoin WIF：5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ（Base58Check 合法）。
// - BIP-32 xprv：xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi（BIP-32 规范示例）。

const BIP39_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const BITCOIN_WIF: &str = "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ";
const BIP32_XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";

/// 出站密钥红队样本：（标签，敏感字面量，不应原样出现的关键子串）。
fn secret_samples() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("BIP39 真助记词", BIP39_MNEMONIC, "abandon abandon abandon"),
        ("Bitcoin WIF", BITCOIN_WIF, BITCOIN_WIF),
        ("BIP-32 xprv", BIP32_XPRV, BIP32_XPRV),
    ]
}

/// 构造含敏感字面量的 /v1/messages 请求 body。
fn secret_body(secret: &str) -> String {
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": format!("here is my key: {secret}") }],
    })
    .to_string()
}

/// 驱动一个出站密钥样本，返回 sieve 转发给上游的请求体（None = 规则缺失 SKIP）。
///
/// 断言「上游被调用 + 上游收到的 body 不含原始密钥关键子串 + 含 REDACTED 占位符」。
async fn assert_secret_redacted(label: &str, secret: &str, must_not_contain: &str) {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"ok-from-upstream")))
                .unwrap()
        }
    })
    .await;

    let Some((sieve_port, _guard)) = spawn_sieve_daemon(&format!("http://{upstream_addr}")) else {
        return; // 规则缺失 → SKIP（spawn_sieve_daemon 已打印 SKIP）
    };

    let body = secret_body(secret);
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "{label}: 脱敏后应转发上游（200）"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);
    assert!(
        !received_str.is_empty(),
        "{label}: 上游应收到（脱敏后）请求体"
    );
    assert!(
        !received_str.contains(must_not_contain),
        "{label}: 脱敏后上游不应收到原始密钥（{must_not_contain}）；body: {received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "{label}: 脱敏后 body 应含 REDACTED 占位符；body: {received_str}"
    );
}

/// BIP39 真助记词 → 出站脱敏（真 SHA-256 checksum，必须 recall）。
#[tokio::test]
async fn redteam_bip39_real_mnemonic_redacted() {
    assert_secret_redacted("BIP39 真助记词", BIP39_MNEMONIC, "abandon abandon abandon").await;
}

/// Bitcoin WIF → 出站脱敏。
#[tokio::test]
async fn redteam_bitcoin_wif_redacted() {
    assert_secret_redacted("Bitcoin WIF", BITCOIN_WIF, BITCOIN_WIF).await;
}

/// BIP-32 xprv → 出站脱敏。
#[tokio::test]
async fn redteam_bip32_xprv_redacted() {
    assert_secret_redacted("BIP-32 xprv", BIP32_XPRV, BIP32_XPRV).await;
}

/// 全样本回归：逐个驱动，规则缺失时整体 SKIP。
#[tokio::test]
async fn redteam_all_secret_samples_redacted() {
    for (label, secret, must_not_contain) in secret_samples() {
        assert_secret_redacted(label, secret, must_not_contain).await;
    }
}
