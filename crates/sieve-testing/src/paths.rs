//! 文件系统路径 helper（workspace root / sieve 二进制 / 规则文件 / 空闲端口）。
//!
//! lift 自 `sieve-cli/tests/outbound_block.rs`。注意 `CARGO_MANIFEST_DIR` 在本 crate
//! 指向 `crates/sieve-testing`，与 `crates/sieve-cli` 同样 pop 两层到 workspace root。

use std::net::TcpListener as StdListener;
use std::path::PathBuf;

/// 从内核拿一个当前空闲的 TCP 端口（bind :0 后立即释放，返回端口号）。
///
/// 存在固有 TOCTOU 竞态：返回后到被使用前端口可能被他人抢占；
/// 集成测试场景下可接受（沿用源测试行为）。
///
/// # Panics
///
/// bind `127.0.0.1:0` 失败时 panic（系统无可用端口的极端情况）。
#[must_use]
pub fn find_free_port() -> u16 {
    let l = StdListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

/// workspace 根目录。
///
/// 本 crate 的 `CARGO_MANIFEST_DIR` 是 `<root>/crates/sieve-testing`，pop 两层得到 root。
#[must_use]
pub fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // sieve-testing → crates/
    p.pop(); // crates/ → workspace root
    p
}

/// sieve 二进制路径。优先 `target/release/sieve`，不存在时 fallback 到 `target/debug/sieve`。
///
/// 注意：返回的路径未必存在；调用方（如 [`crate::daemon::spawn_daemon`]）会校验并给出
/// 「先 `cargo build` 」的错误提示。
#[must_use]
pub fn sieve_binary() -> PathBuf {
    let root = workspace_root();
    let release = root.join("target/release/sieve");
    if release.exists() {
        return release;
    }
    root.join("target/debug/sieve")
}

/// 系统出站规则文件路径（`crates/sieve-rules/rules/outbound.toml`）。
#[must_use]
pub fn outbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/outbound.toml")
}

/// 系统入站规则文件路径（`crates/sieve-rules/rules/inbound.toml`）。
#[must_use]
pub fn inbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/inbound.toml")
}
