//! sieve-updater 端到端 hermetic 闭环测试（手动联调 checklist §14.1/14.4/14.5/14.6 自动化）。
//!
//! 全程本地 plain-HTTP mock（复用 `sieve-testing::spawn_mock_upstream`）+ debug 构建的
//! `SIEVE_UPDATE_ALLOW_HTTP` 接缝（见 `src/tls.rs`：release/GA 编译期消除，恒 https_only）。
//! 无真网络、无真 manifest 服务、无 TLS 握手依赖。
//!
//! 覆盖：
//! - §14.4 完整闭环 fetch_manifest → download_rules → install_rules（sha256 + 签名 skip + zstd
//!   解压 + 原子 rename + current.json symlink + latest_version.json）。
//! - §14.1 install-id 首启生成 / 幂等 / 删后重生（经 `SIEVE_CACHE_DIR` 隔离，P0.1）。
//! - §14.5 失败模式：sha256 mismatch / 服务不可达 / 坏 zstd / 超大响应——各自精确报错，不 panic。
//! - §14.6 公钥 None 占位：签名校验被 skip（非静默通过；WARN 由 `verify_signature` 发出）。
//! - 遥测参数：uid 在 telemetry 开启时出现在 manifest query，关闭时缺失（ADR-030）。
//!
//! `.cursorrules §3.2`：测试代码允许 `.unwrap()`。

use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use serial_test::serial;
use sha2::{Digest, Sha256};
use sieve_core::forwarder::ProxyConfig;
use sieve_testing::upstream::{spawn_mock_upstream, MockUpstream};
use sieve_updater::download::{download_rules, DEFAULT_MAX_RULES_SIZE};
use sieve_updater::error::UpdaterError;
use sieve_updater::install::install_rules;
use sieve_updater::manifest::{fetch_manifest, ManifestParams};

const ALLOW_HTTP: &str = "SIEVE_UPDATE_ALLOW_HTTP";
const CACHE_DIR: &str = "SIEVE_CACHE_DIR";
const RULE_VERSION: &str = "2026-06-18.1";

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn zstd_encode(data: &[u8]) -> Vec<u8> {
    zstd::encode_all(data, 3).expect("zstd encode")
}

/// 启动一个 mock manifest+CDN 上游，返回 `(mock, 收到的 manifest query 列表)`。
///
/// - `GET /v1/manifest` → manifest JSON，`rules.url` 自引用本机（用请求 Host 头拼）。
/// - `GET /rules/...` → `payload`（zstd 压缩的规则字节）。
///
/// query 列表供断言遥测参数（uid 等）。
async fn spawn_manifest_mock(
    payload: Vec<u8>,
    sha256: String,
) -> (MockUpstream, Arc<Mutex<Vec<String>>>) {
    let queries: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let q = queries.clone();
    let size = payload.len() as u64;
    let payload = Arc::new(payload);

    let mock = spawn_mock_upstream(move |req: Request<Bytes>| {
        let q = q.clone();
        let payload = payload.clone();
        let sha = sha256.clone();
        async move {
            let path = req.uri().path().to_owned();
            if path == "/v1/manifest" {
                if let Some(query) = req.uri().query() {
                    q.lock().unwrap().push(query.to_owned());
                }
                let host = req
                    .headers()
                    .get(hyper::header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("127.0.0.1")
                    .to_owned();
                let rules_url = format!("http://{host}/rules/{RULE_VERSION}.json.zst");
                let body = format!(
                    r#"{{"schema":1,"rules":{{"version":"{RULE_VERSION}","url":"{rules_url}","sha256":"{sha}","size":{size},"signature":"00"}},"next_check_after_seconds":21600}}"#
                );
                Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap()
            } else if path.starts_with("/rules/") {
                Response::builder()
                    .status(200)
                    .header("content-type", "application/octet-stream")
                    .body(Full::new(Bytes::from((*payload).clone())))
                    .unwrap()
            } else {
                Response::builder()
                    .status(404)
                    .body(Full::new(Bytes::from_static(b"not found")))
                    .unwrap()
            }
        }
    })
    .await;

    (mock, queries)
}

fn params(uid: Option<uuid::Uuid>) -> ManifestParams {
    ManifestParams {
        v: "0.1.0-alpha".to_owned(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        uid,
        ch: "stable".to_owned(),
    }
}

// ── §14.4 完整闭环 ─────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn closure_fetch_download_install_atomic() {
    std::env::set_var(ALLOW_HTTP, "1");

    let rule_json = br#"{"rules":[{"id":"OUT-99","pattern":"x"}]}"#;
    let payload = zstd_encode(rule_json);
    let sha = sha256_hex(&payload);
    let (mock, queries) = spawn_manifest_mock(payload.clone(), sha.clone()).await;

    // 1. fetch_manifest（带 uid，telemetry 开）
    let uid = uuid::Uuid::new_v4();
    let manifest = fetch_manifest(
        &format!("{}/v1/manifest", mock.url()),
        params(Some(uid)),
        &ProxyConfig::Direct,
    )
    .await
    .expect("fetch_manifest must succeed against mock");
    let rules = manifest.rules.expect("manifest must carry rules");
    assert_eq!(rules.version, RULE_VERSION);
    assert_eq!(rules.sha256, sha);

    // 遥测：uid 必须出现在 manifest query（telemetry 开启）。
    let q = queries.lock().unwrap().join("&");
    assert!(
        q.contains(&format!("uid={uid}")),
        "uid must appear in query: {q}"
    );
    assert!(q.contains("os="), "os param must appear: {q}");

    // 2. download_rules（用 manifest 给的自引用 URL）
    let downloaded = download_rules(&rules.url, DEFAULT_MAX_RULES_SIZE, &ProxyConfig::Direct)
        .await
        .expect("download_rules must succeed");
    assert_eq!(
        downloaded, payload,
        "downloaded bytes must match served payload"
    );

    // 3. install_rules（sha256 ✓ + 签名 skip + zstd 解压 + 原子落盘）
    let dest = tempfile::tempdir().unwrap();
    let installed = install_rules(
        &downloaded,
        &rules.sha256,
        &rules.signature,
        &rules.version,
        dest.path(),
    )
    .await
    .expect("install_rules must succeed");

    // 校验产物：<version>.json 内容 = 解压后的原始规则 JSON。
    assert!(installed.exists(), "installed file must exist");
    let on_disk = std::fs::read(&installed).unwrap();
    assert_eq!(
        on_disk, rule_json,
        "installed content must be decompressed rule JSON"
    );

    // current.json symlink/拷贝指向已装版本。
    let current = dest.path().join("current.json");
    assert!(current.exists(), "current.json must exist");
    assert_eq!(
        std::fs::read(&current).unwrap(),
        rule_json,
        "current.json must resolve to installed rules"
    );

    // latest_version.json 元数据含版本号。
    let latest = dest.path().join("latest_version.json");
    assert!(latest.exists(), "latest_version.json must exist");
    let meta = std::fs::read_to_string(&latest).unwrap();
    assert!(
        meta.contains(RULE_VERSION),
        "latest_version.json must record version: {meta}"
    );

    std::env::remove_var(ALLOW_HTTP);
}

// ── 遥测：uid 关闭时不出现 ───────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn telemetry_uid_omitted_when_none() {
    std::env::set_var(ALLOW_HTTP, "1");
    let payload = zstd_encode(b"{}");
    let sha = sha256_hex(&payload);
    let (mock, queries) = spawn_manifest_mock(payload, sha).await;

    let _ = fetch_manifest(
        &format!("{}/v1/manifest", mock.url()),
        params(None), // SIEVE_NO_TELEMETRY 等价：uid=None
        &ProxyConfig::Direct,
    )
    .await
    .expect("fetch_manifest must succeed");

    let q = queries.lock().unwrap().join("&");
    assert!(!q.contains("uid="), "uid must be absent when None: {q}");
    assert!(
        q.contains("v=0.1.0-alpha"),
        "non-uid params still present: {q}"
    );
    std::env::remove_var(ALLOW_HTTP);
}

// ── §14.1 install-id ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn install_id_first_idempotent_regen() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var(CACHE_DIR, cache.path());

    // 首启生成
    let id1 = sieve_updater::install_id::load_or_create_install_id()
        .await
        .expect("first load must create");
    let id_file = cache.path().join("install-id");
    assert!(
        id_file.exists(),
        "install-id file must exist after first load"
    );

    // 幂等：二次相同
    let id2 = sieve_updater::install_id::load_or_create_install_id()
        .await
        .unwrap();
    assert_eq!(id1, id2, "second call must return same UUID");

    // 删后重生：新 UUID
    std::fs::remove_file(&id_file).unwrap();
    let id3 = sieve_updater::install_id::load_or_create_install_id()
        .await
        .unwrap();
    assert!(id_file.exists(), "install-id must be regenerated");
    assert_ne!(id1, id3, "deleted-then-recreated must yield a fresh UUID");

    std::env::remove_var(CACHE_DIR);
}

// ── §14.5 失败模式（各自精确报错，不 panic）────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failure_sha256_mismatch() {
    let payload = zstd_encode(b"{}");
    let dest = tempfile::tempdir().unwrap();
    let wrong_sha = "0".repeat(64);
    let err = install_rules(&payload, &wrong_sha, "00", "v1", dest.path())
        .await
        .expect_err("bad sha256 must fail");
    assert!(
        matches!(err, UpdaterError::Sha256Mismatch { .. }),
        "got {err:?}"
    );
    // 原子性：不留半成品。
    assert!(
        !dest.path().join("v1.json").exists(),
        "no partial install on sha mismatch"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failure_bad_zstd() {
    // 真 zstd magic 头（0x28 0xB5 0x2F 0xFD）存在但内容损坏 → DecompressFailed。
    let mut payload = vec![0x28, 0xB5, 0x2F, 0xFD];
    payload.extend_from_slice(b"this is not a valid zstd stream");
    let sha = sha256_hex(&payload);
    let dest = tempfile::tempdir().unwrap();
    let err = install_rules(&payload, &sha, "00", "v1", dest.path())
        .await
        .expect_err("corrupt zstd must fail");
    assert!(
        matches!(err, UpdaterError::DecompressFailed(_)),
        "got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn failure_server_unreachable() {
    std::env::set_var(ALLOW_HTTP, "1");
    // 绑一个端口随即释放，得到一个大概率无人监听的地址。
    let dead_port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let err = download_rules(
        &format!("http://127.0.0.1:{dead_port}/rules/x.zst"),
        DEFAULT_MAX_RULES_SIZE,
        &ProxyConfig::Direct,
    )
    .await
    .expect_err("unreachable server must fail");
    assert!(matches!(err, UpdaterError::Http(_)), "got {err:?}");
    std::env::remove_var(ALLOW_HTTP);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn failure_response_too_large() {
    std::env::set_var(ALLOW_HTTP, "1");
    let payload = zstd_encode(&vec![b'x'; 4096]);
    let sha = sha256_hex(&payload);
    let (mock, _q) = spawn_manifest_mock(payload, sha).await;
    let err = download_rules(
        &format!("{}/rules/{RULE_VERSION}.json.zst", mock.url()),
        16, // max 16 bytes，远小于 payload
        &ProxyConfig::Direct,
    )
    .await
    .expect_err("oversize response must fail");
    assert!(
        matches!(err, UpdaterError::ResponseTooLarge { .. }),
        "got {err:?}"
    );
    std::env::remove_var(ALLOW_HTTP);
}

// ── §14.6 公钥 None 占位：签名 skip（非静默通过）──────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pubkey_none_skips_signature_check() {
    // TRUSTED_PUBKEY 占位 None → verify_signature skip+warn。即使签名是非法 hex，
    // install 仍成功（证明 skip 路径生效；WARN 由 verify_signature 发出，见 §14.6）。
    let rule_json = b"{}";
    let payload = zstd_encode(rule_json);
    let sha = sha256_hex(&payload);
    let dest = tempfile::tempdir().unwrap();
    let installed = install_rules(&payload, &sha, "not-even-valid-hex", "v1", dest.path())
        .await
        .expect("None pubkey must skip signature and install successfully");
    assert!(installed.exists());
    assert_eq!(std::fs::read(&installed).unwrap(), rule_json);
}
