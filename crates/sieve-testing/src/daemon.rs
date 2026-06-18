//! 真实 sieve daemon 进程管理。
//!
//! lift 自 `sieve-cli/tests/outbound_block.rs::spawn_sieve_daemon_with_home` + `DaemonGuard`。
//! 写临时 `sieve.toml`（含 `tls_verify_upstream=false` + 绝对规则路径 + 隔离的 `SIEVE_HOME`），
//! spawn `sieve start --config ...`，轮询等端口 LISTEN 后返回 [`DaemonGuard`]。

use crate::paths::{find_free_port, inbound_rules_path, outbound_rules_path, sieve_binary};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// daemon 启动配置。
///
/// 字段全部公开，可用 `DaemonConfig { upstream_url, ..Default::default() }` 风格构造，
/// 也可用 builder 风格 setter（[`DaemonConfig::with_upstream`] 等）。
#[derive(Debug, Clone, Default)]
pub struct DaemonConfig {
    /// 上游 URL，如 `http://127.0.0.1:1234`（通常取 `MockUpstream::url()`）。
    pub upstream_url: String,
    /// dry_run 模式（仅记录命中不拦截 Block 路径）。
    pub dry_run: bool,
    /// 自定义 `SIEVE_HOME`；`None` 时自动建 tempdir 隔离（推荐）。
    pub sieve_home: Option<PathBuf>,
    /// 无客户端策略（写入 `sieve.toml` 的 `no_client_policy`），如 `"auto-block"`；`None` 不写。
    pub no_client_policy: Option<String>,
    /// 追加到 `sieve.toml` 末尾的额外 TOML 文本（如自定义字段 / section）。
    pub extra_toml: Option<String>,
    /// 额外 / 覆盖环境变量。可覆盖默认的 `SIEVE_NO_UPDATE` / `SIEVE_NO_TELEMETRY` / `SIEVE_LOG`。
    pub env: Vec<(String, String)>,
}

impl DaemonConfig {
    /// 用指定上游 URL 构造（其余字段取默认）。
    #[must_use]
    pub fn new(upstream_url: impl Into<String>) -> Self {
        Self {
            upstream_url: upstream_url.into(),
            ..Default::default()
        }
    }

    /// 设置上游 URL。
    #[must_use]
    pub fn with_upstream(mut self, url: impl Into<String>) -> Self {
        self.upstream_url = url.into();
        self
    }

    /// 设置 dry_run。
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// 设置 `SIEVE_HOME`。
    #[must_use]
    pub fn with_sieve_home(mut self, home: impl Into<PathBuf>) -> Self {
        self.sieve_home = Some(home.into());
        self
    }

    /// 设置 `no_client_policy`。
    #[must_use]
    pub fn with_no_client_policy(mut self, policy: impl Into<String>) -> Self {
        self.no_client_policy = Some(policy.into());
        self
    }

    /// 追加额外 TOML。
    #[must_use]
    pub fn with_extra_toml(mut self, toml: impl Into<String>) -> Self {
        self.extra_toml = Some(toml.into());
        self
    }

    /// 追加一条环境变量（可多次调用）。
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.push((key.into(), val.into()));
        self
    }
}

/// daemon 进程守卫。Drop 时 SIGKILL + wait，避免僵尸进程；
/// 同时持有临时 config 文件与（自动创建时的）`SIEVE_HOME` tempdir，防止运行期被清理。
pub struct DaemonGuard {
    proc: Child,
    /// 监听端口。
    port: u16,
    /// 实际生效的 `SIEVE_HOME`（无论调用方传入还是自动创建）。
    sieve_home: PathBuf,
    /// 持有临时 config 文件，防进程运行期被删。
    _config_file: tempfile::NamedTempFile,
    /// 自动创建的 `SIEVE_HOME` tempdir；调用方显式传入时为 `None`。
    _sieve_home_dir: Option<tempfile::TempDir>,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

impl DaemonGuard {
    /// daemon 监听端口。
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// daemon base URL，形如 `http://127.0.0.1:<port>`（无尾斜杠）。
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// 生效的 `SIEVE_HOME` 目录。
    #[must_use]
    pub fn sieve_home(&self) -> &Path {
        &self.sieve_home
    }

    /// IPC Unix socket 路径（`<sieve_home>/ipc.sock`，与 `sieve_ipc::paths::ipc_socket_path` 一致）。
    #[must_use]
    pub fn ipc_socket(&self) -> PathBuf {
        self.sieve_home.join("ipc.sock")
    }

    /// 审计 SQLite DB 路径（`<sieve_home>/audit.db`，与 `Config::audit_db_path` 默认值一致）。
    #[must_use]
    pub fn audit_db(&self) -> PathBuf {
        self.sieve_home.join("audit.db")
    }
}

/// 启动真实 sieve daemon，轮询等端口 LISTEN（最多 10s）后返回 [`DaemonGuard`]。
///
/// 行为（lift 自 `outbound_block.rs::spawn_sieve_daemon_with_home`）：
/// - 写临时 `sieve.toml`：`upstream_url` / `port` / `bind_addr=127.0.0.1` / 绝对规则路径 /
///   `tls_verify_upstream=false` / `dry_run`；可选 `no_client_policy` 与 `extra_toml`。
/// - `SIEVE_HOME`：`cfg.sieve_home` 优先，否则自动建 tempdir 隔离（防污染真实 `~/.sieve`）。
/// - 默认注入 `SIEVE_LOG=warn` / `SIEVE_NO_UPDATE=1` / `SIEVE_NO_TELEMETRY=1`（ADR-030 禁联网），
///   `cfg.env` 可覆盖。
///
/// # Panics
///
/// 以下情况 panic（测试场景下视为前置条件未满足）：
/// - 系统规则文件不存在（先确认 `crates/sieve-rules/rules/*.toml`）；
/// - sieve 二进制不存在（先 `cargo build` 或 `cargo build --release`）；
/// - daemon 在 10s 内未监听端口。
#[must_use]
pub fn spawn_daemon(cfg: DaemonConfig) -> DaemonGuard {
    let port = find_free_port();
    let rules = outbound_rules_path();
    assert!(
        rules.exists(),
        "outbound rules not found at {}",
        rules.display()
    );
    let inbound_rules = inbound_rules_path();
    assert!(
        inbound_rules.exists(),
        "inbound rules not found at {}",
        inbound_rules.display()
    );

    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"upstream_url = "{}"
port = {}
bind_addr = "127.0.0.1"
rules_path = "{}"
inbound_rules_path = "{}"
tls_verify_upstream = false
dry_run = {}
"#,
        cfg.upstream_url,
        port,
        rules.display(),
        inbound_rules.display(),
        cfg.dry_run,
    )
    .unwrap();

    // 注意：`no_client_policy` **不能**写进 sieve.toml —— daemon 的 `Config` 用
    // `#[serde(deny_unknown_fields)]` 且该字段只从 `start --no-client-policy` CLI flag 读，
    // 写进 toml 会让 config 加载失败、daemon 启动超时。下方作为 CLI 参数传入。
    if let Some(extra) = &cfg.extra_toml {
        writeln!(config_file, "{extra}").unwrap();
    }
    config_file.flush().unwrap();

    let binary = sieve_binary();
    assert!(
        binary.exists(),
        "sieve binary not found at {}; run `cargo build` (or `--release`) first",
        binary.display()
    );

    // SIEVE_HOME：调用方未指定则自动 isolate 到 tempdir。
    let owned_home: Option<tempfile::TempDir> = if cfg.sieve_home.is_none() {
        Some(tempfile::tempdir().unwrap())
    } else {
        None
    };
    let effective_home: PathBuf = cfg
        .sieve_home
        .clone()
        .or_else(|| owned_home.as_ref().map(|d| d.path().to_owned()))
        .unwrap();

    let mut cmd = Command::new(&binary);
    cmd.arg("start").arg("--config").arg(config_file.path());
    // no_client_policy 走 CLI flag（daemon 不从 toml 读，见上方注释）。
    if let Some(policy) = &cfg.no_client_policy {
        cmd.arg("--no-client-policy").arg(policy);
    }
    cmd
        // 默认值（可被 cfg.env 覆盖，因 Command::env 后写覆盖前写）。
        .env("SIEVE_LOG", "warn")
        // ADR-030: 测试禁止触发真实 updates.sieveai.dev 联网 + telemetry 上报。
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", &effective_home)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    for (k, v) in &cfg.env {
        cmd.env(k, v);
    }

    let proc = cmd.spawn().expect("spawn sieve daemon");

    wait_for_listen(port, Duration::from_secs(10));

    DaemonGuard {
        proc,
        port,
        sieve_home: effective_home,
        _config_file: config_file,
        _sieve_home_dir: owned_home,
    }
}

/// 轮询等 daemon TCP listener 就绪。
///
/// 不能用 HTTP-level probe（在 `#[tokio::test]` current_thread runtime 上会死锁，
/// 详见 `outbound_block.rs::wait_for_http_ready` 注释）。TCP connect 成功即视为就绪。
///
/// # Panics
///
/// 超时未监听时 panic。
fn wait_for_listen(port: u16, timeout: Duration) {
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
        assert!(
            Instant::now() < deadline,
            "sieve daemon did not listen on :{port} within {timeout:?}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// 轮询等 IPC socket 可连接（消除「daemon 已 LISTEN 但 IPC server 尚未 bind socket」的竞态）。
///
/// 阻塞实现：用 `std::os::unix::net::UnixStream::connect` 重试，每 50ms 一次直到 `timeout`。
/// 返回 `true` 表示连上，`false` 表示超时（不 panic，调用方自行决定是否断言）。
///
/// 仅 Unix 平台有效；非 Unix 平台恒返回 `false`。
#[must_use]
pub fn wait_for_ipc(guard: &DaemonGuard, timeout: Duration) -> bool {
    let socket = guard.ipc_socket();
    let deadline = Instant::now() + timeout;
    loop {
        #[cfg(unix)]
        {
            if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
                return true;
            }
        }
        #[cfg(not(unix))]
        {
            let _ = &socket;
            return false;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
