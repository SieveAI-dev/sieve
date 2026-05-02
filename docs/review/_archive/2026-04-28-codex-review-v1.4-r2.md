OpenAI Codex v0.125.0 (research preview)
--------
workdir: /Users/doskey/src/sieve
model: gpt-5.5
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/doskey/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019dd1e1-3cb5-7830-ac29-5a9a689f10d1
--------
user
current changes
exec
/bin/zsh -lc 'git status --short --branch && git diff --stat && git diff --cached --stat && git ls-files --others --exclude-standard' in /Users/doskey/src/sieve
 succeeded in 0ms:
## main...origin/main
 M .cursorrules
 M CLAUDE.md
 M Cargo.lock
 M Cargo.toml
 M README.md
 M SECURITY.md
 M crates/sieve-cli/Cargo.toml
 M crates/sieve-cli/src/audit.rs
 M crates/sieve-cli/src/cli.rs
 M crates/sieve-cli/src/config.rs
 M crates/sieve-cli/src/daemon.rs
 M crates/sieve-cli/src/engine_adapter.rs
 M crates/sieve-cli/src/main.rs
 M crates/sieve-core/Cargo.toml
 M crates/sieve-core/src/detection.rs
 M crates/sieve-core/src/pipeline/inbound.rs
 M crates/sieve-core/src/pipeline/mod.rs
 M crates/sieve-rules/benches/scan_bench.rs
 M crates/sieve-rules/rules/inbound.toml
 M crates/sieve-rules/rules/outbound.toml
 M crates/sieve-rules/src/critical_lock.rs
 M crates/sieve-rules/src/engine/mod.rs
 M crates/sieve-rules/src/manifest.rs
 M crates/sieve-rules/tests/inbound_rules.rs
 M crates/sieve-rules/tests/outbound_rules.rs
 M docs/api/api-reference.md
 M docs/changelog/CHANGELOG.md
 M docs/design/ADR-001-rust-tech-stack.md
 M docs/design/ADR-002-rule-engine-only-phase1.md
 M docs/design/ADR-003-local-only-no-cloud-verifier.md
 M docs/design/ADR-004-anthropic-first-unified-interface.md
 M docs/design/ADR-005-overseas-legal-entity.md
 M docs/design/ADR-006-sigstore-reproducible-build.md
 M docs/design/ADR-007-fail-closed-critical-actions.md
 M docs/design/ADR-011-private-until-ga.md
 M docs/design/ADR-INDEX.md
 M docs/design/architecture.md
 M docs/design/data-model.md
 M docs/glossary.md
 M docs/guides/deployment.md
 M docs/guides/development.md
 M docs/requirements/PRD-sieve.md
 M docs/requirements/user-stories.md
 M tasks/roadmap.md
?? crates/sieve-cli/src/commands/
?? crates/sieve-cli/tests/audit_append_only.rs
?? crates/sieve-cli/tests/sieve_setup_dry_run.rs
?? crates/sieve-core/src/pipeline/inbound_hold.rs
?? crates/sieve-core/src/pipeline/inbound_hook.rs
?? crates/sieve-core/src/pipeline/outbound_redact.rs
?? crates/sieve-hook/
?? crates/sieve-ipc/
?? docs/design/ADR-012-native-gui-app-phase1.md
?? docs/design/ADR-013-ipc-protocol.md
?? docs/design/ADR-014-dual-layer-defense.md
?? docs/design/ADR-015-sieve-setup-tool.md
?? docs/design/ADR-016-disposition-matrix-2d.md
?? docs/prd/sieve-prd-v1.4.md
?? docs/specs/
?? librust_out.rlib
?? tasks/todo.md
 .cursorrules                                       |  21 +-
 CLAUDE.md                                          |  40 +-
 Cargo.lock                                         | Bin 56453 -> 64212 bytes
 Cargo.toml                                         |   6 +
 README.md                                          |  12 +-
 SECURITY.md                                        |  14 +-
 crates/sieve-cli/Cargo.toml                        |   4 +
 crates/sieve-cli/src/audit.rs                      | 376 ++++++++++++-
 crates/sieve-cli/src/cli.rs                        |  40 ++
 crates/sieve-cli/src/config.rs                     |  98 +++-
 crates/sieve-cli/src/daemon.rs                     | 583 ++++++++++++++++++---
 crates/sieve-cli/src/engine_adapter.rs             | 207 +++++++-
 crates/sieve-cli/src/main.rs                       |  22 +-
 crates/sieve-core/Cargo.toml                       |   7 +
 crates/sieve-core/src/detection.rs                 |  28 +-
 crates/sieve-core/src/pipeline/inbound.rs          |   2 +-
 crates/sieve-core/src/pipeline/mod.rs              | 390 +++++++++++++-
 crates/sieve-rules/benches/scan_bench.rs           |   5 +-
 crates/sieve-rules/rules/inbound.toml              |  99 ++++
 crates/sieve-rules/rules/outbound.toml             |  26 +
 crates/sieve-rules/src/critical_lock.rs            | 170 +++++-
 crates/sieve-rules/src/engine/mod.rs               |   3 +
 crates/sieve-rules/src/manifest.rs                 | 219 +++++++-
 crates/sieve-rules/tests/inbound_rules.rs          | 142 +++++
 crates/sieve-rules/tests/outbound_rules.rs         |  77 +++
 docs/api/api-reference.md                          | 166 ++++--
 docs/changelog/CHANGELOG.md                        |  43 ++
 docs/design/ADR-001-rust-tech-stack.md             |   8 +-
 docs/design/ADR-002-rule-engine-only-phase1.md     |  16 +-
 .../design/ADR-003-local-only-no-cloud-verifier.md |  16 +-
 .../ADR-004-anthropic-first-unified-interface.md   |  32 +-
 docs/design/ADR-005-overseas-legal-entity.md       |  22 +-
 docs/design/ADR-006-sigstore-reproducible-build.md |  24 +-
 .../design/ADR-007-fail-closed-critical-actions.md |  26 +
 docs/design/ADR-011-private-until-ga.md            |   8 +-
 docs/design/ADR-INDEX.md                           |   9 +-
 docs/design/architecture.md                        | 213 +++++---
 docs/design/data-model.md                          | 121 ++++-
 docs/glossary.md                                   |  50 +-
 docs/guides/deployment.md                          | 280 ++++------
 docs/guides/development.md                         |  70 ++-
 docs/requirements/PRD-sieve.md                     |  47 +-
 docs/requirements/user-stories.md                  | 178 +++++--
 tasks/roadmap.md                                   |  77 +--
 44 files changed, 3311 insertions(+), 686 deletions(-)
crates/sieve-cli/src/commands/doctor.rs
crates/sieve-cli/src/commands/mod.rs
crates/sieve-cli/src/commands/setup.rs
crates/sieve-cli/src/commands/uninstall.rs
crates/sieve-cli/tests/audit_append_only.rs
crates/sieve-cli/tests/sieve_setup_dry_run.rs
crates/sieve-core/src/pipeline/inbound_hold.rs
crates/sieve-core/src/pipeline/inbound_hook.rs
crates/sieve-core/src/pipeline/outbound_redact.rs
crates/sieve-hook/Cargo.toml
crates/sieve-hook/benches/hook_startup.rs
crates/sieve-hook/src/decision.rs
crates/sieve-hook/src/error.rs
crates/sieve-hook/src/lib.rs
crates/sieve-hook/src/main.rs
crates/sieve-hook/src/pending.rs
crates/sieve-hook/src/protocol.rs
crates/sieve-ipc/Cargo.toml
crates/sieve-ipc/src/decision_file.rs
crates/sieve-ipc/src/error.rs
crates/sieve-ipc/src/lib.rs
crates/sieve-ipc/src/paths.rs
crates/sieve-ipc/src/pending_file.rs
crates/sieve-ipc/src/protocol.rs
crates/sieve-ipc/src/socket_client.rs
crates/sieve-ipc/src/socket_server.rs
docs/design/ADR-012-native-gui-app-phase1.md
docs/design/ADR-013-ipc-protocol.md
docs/design/ADR-014-dual-layer-defense.md
docs/design/ADR-015-sieve-setup-tool.md
docs/design/ADR-016-disposition-matrix-2d.md
docs/prd/sieve-prd-v1.4.md
docs/specs/SPEC-001-sieve-hook-protocol.md
docs/specs/SPEC-002-hips-popup-behavior.md
docs/specs/SPEC-003-sieve-setup-tool.md
librust_out.rlib
tasks/todo.md

exec
/bin/zsh -lc "git diff -- Cargo.toml crates/sieve-cli/Cargo.toml crates/sieve-core/Cargo.toml crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-cli/src/config.rs && sed -n '1,220p' crates/sieve-cli/src/commands/setup.rs && sed -n '1,220p' crates/sieve-cli/src/commands/doctor.rs && sed -n '1,220p' crates/sieve-cli/src/commands/uninstall.rs && sed -n '1,80p' crates/sieve-cli/src/commands/mod.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/Cargo.toml b/Cargo.toml
index 3be5e56..4e201d9 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -4,6 +4,8 @@ members = [
     "crates/sieve-core",
     "crates/sieve-rules",
     "crates/sieve-cli",
+    "crates/sieve-ipc",
+    "crates/sieve-hook",
     "fuzz",
 ]
 exclude = [
@@ -35,6 +37,10 @@ serde_json = "1"
 toml = "0.8"
 thiserror = "2"
 tracing = "0.1"
+uuid = { version = "1", features = ["v7", "serde"] }
+chrono = { version = "0.4", default-features = false, features = ["serde", "clock"] }
+fd-lock = "4"
+clap = { version = "4", features = ["derive"] }
 
 [profile.release]
 codegen-units = 1
diff --git a/crates/sieve-cli/Cargo.toml b/crates/sieve-cli/Cargo.toml
index 24b05c8..6209d1a 100644
--- a/crates/sieve-cli/Cargo.toml
+++ b/crates/sieve-cli/Cargo.toml
@@ -16,6 +16,9 @@ path = "src/main.rs"
 [dependencies]
 sieve-core = { path = "../sieve-core" }
 sieve-rules = { path = "../sieve-rules" }
+sieve-ipc = { path = "../sieve-ipc" }
+rusqlite = { version = "0.31", features = ["bundled"] }
+chrono = { workspace = true }
 
 tokio = { workspace = true, features = ["full"] }
 hyper = { workspace = true, features = ["http1", "http2", "server"] }
@@ -47,3 +50,4 @@ sieve-core = { path = "../sieve-core" }
 anyhow = "1"
 tempfile = "3"
 serde_json = { workspace = true }
+rusqlite = { version = "0.31", features = ["bundled"] }
diff --git a/crates/sieve-cli/src/cli.rs b/crates/sieve-cli/src/cli.rs
index d585b18..79526a7 100644
--- a/crates/sieve-cli/src/cli.rs
+++ b/crates/sieve-cli/src/cli.rs
@@ -3,6 +3,9 @@
 //! 设计约束（ADR-007）：**禁止任何 --disable-critical / --yolo flag**。
 //! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
 //! 不暴露给 CLI。
+//!
+//! Week 5 新增（ADR-015 / SPEC-003）：`setup` / `doctor` / `uninstall` 子命令，
+//! 仅 macOS Phase 1 支持；非 macOS 编译进友好错误 stub。
 
 use clap::{Parser, Subcommand};
 use std::path::PathBuf;
@@ -34,4 +37,41 @@ pub enum Command {
     },
     /// 打印版本号并退出。
     Version,
+    /// 自动配置 Claude Code 环境（仅 macOS Phase 1）。
+    ///
+    /// 修改 `~/.claude/settings.json`，注册 launchd plist，写审计 setup 日志。
+    /// 关联：ADR-015 / SPEC-003 §setup。
+    Setup(SetupArgs),
+    /// 诊断 Sieve 安装状态（仅 macOS Phase 1）。
+    ///
+    /// 检查 settings.json / hook / daemon / launchd / canary 共 5 项。
+    /// 关联：ADR-015 / SPEC-003 §doctor。
+    Doctor,
+    /// 干净回滚 setup 的所有改动（仅 macOS Phase 1）。
+    ///
+    /// 从备份目录恢复原文件，卸载 launchd plist。
+    /// 关联：ADR-015 / SPEC-003 §uninstall。
+    Uninstall(UninstallArgs),
+}
+
+/// `sieve setup` 参数（ADR-015 / SPEC-003 §setup）。
+#[derive(clap::Args, Debug)]
+pub struct SetupArgs {
+    /// 不实际改文件，仅打印 diff（dry-run 模式）。
+    #[arg(long)]
+    pub dry_run: bool,
+    /// 不询问确认，直接执行（CI / 自动化用；仍打印 diff）。
+    #[arg(long)]
+    pub yes: bool,
+}
+
+/// `sieve uninstall` 参数（ADR-015 / SPEC-003 §uninstall）。
+#[derive(clap::Args, Debug)]
+pub struct UninstallArgs {
+    /// 不实际改文件，仅打印将恢复的内容。
+    #[arg(long)]
+    pub dry_run: bool,
+    /// 不询问确认，直接执行。
+    #[arg(long)]
+    pub yes: bool,
 }
diff --git a/crates/sieve-cli/src/config.rs b/crates/sieve-cli/src/config.rs
index 350dc84..a052d44 100644
--- a/crates/sieve-cli/src/config.rs
+++ b/crates/sieve-cli/src/config.rs
@@ -4,13 +4,32 @@
 //! `tls_verify_upstream`。
 //! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
 //! Week 3 新增：`inbound_rules_path`（入站规则路径）。
+//! Week 5 新增：`ipc_socket_path` / `pending_dir` / `decisions_dir` /
+//!              `preset` / `launchd_plist_path` / `gui_socket_enabled` /
+//!              `audit_db_path`（SPEC-003 / data-model.md §5）。
 //! `#[serde(deny_unknown_fields)]` 确保配置文件中的危险字段（如
 //! `disable_critical`）被强制拒绝，不会静默忽略。
 
 use anyhow::{anyhow, Context, Result};
-use serde::Deserialize;
+use serde::{Deserialize, Serialize};
 use std::path::{Path, PathBuf};
 
+/// 检测预设级别（SPEC-003 / data-model.md §5）。
+///
+/// - `Strict`：所有规则最高灵敏度
+/// - `Default`：推荐平衡配置（默认）
+/// - `Relaxed`：降低误报，适合受信任环境
+/// - `Custom`：完全自定义（忽略内置默认值）
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
+#[serde(rename_all = "snake_case")]
+pub enum Preset {
+    Strict,
+    #[default]
+    Default,
+    Relaxed,
+    Custom,
+}
+
 /// Sieve 顶层配置。
 ///
 /// 对应 `sieve.toml`（ADR-003 / data-model.md §配置）。
@@ -57,6 +76,76 @@ pub struct Config {
     /// 入站规则 toml 路径（Week 3，默认 `crates/sieve-rules/rules/inbound.toml`）。
     #[serde(default)]
     pub inbound_rules_path: Option<PathBuf>,
+
+    // ── Week 5 新字段（SPEC-003 / data-model.md §5）────────────────────────
+    // Week 6+ 会在 daemon 启动时读取这些字段；当前仅反序列化使用，暂时 allow dead_code。
+    /// Unix socket 路径（GUI / sieve-hook 连接用，默认 `~/.sieve/ipc.sock`）。
+    #[serde(default = "default_ipc_socket")]
+    #[allow(dead_code)]
+    pub ipc_socket_path: PathBuf,
+
+    /// 待决策文件目录（默认 `~/.sieve/pending/`）。
+    #[serde(default = "default_pending_dir")]
+    #[allow(dead_code)]
+    pub pending_dir: PathBuf,
+
+    /// 决策文件目录（默认 `~/.sieve/decisions/`）。
+    #[serde(default = "default_decisions_dir")]
+    #[allow(dead_code)]
+    pub decisions_dir: PathBuf,
+
+    /// 检测预设级别（默认 `Default`）。
+    #[serde(default)]
+    #[allow(dead_code)]
+    pub preset: Preset,
+
+    /// launchd plist 路径（macOS，默认 `~/Library/LaunchAgents/com.sieve.daemon.plist`）。
+    #[serde(default = "default_launchd_plist")]
+    #[allow(dead_code)]
+    pub launchd_plist_path: PathBuf,
+
+    /// 是否启用 GUI Unix socket（默认 `false`；Week 6+ 启用）。
+    #[serde(default = "default_gui_socket_enabled")]
+    #[allow(dead_code)]
+    pub gui_socket_enabled: bool,
+
+    /// SQLite 审计数据库路径（Week 5；`None` 时沿用 `log_path` 或 `~/.sieve/audit.db`）。
+    #[serde(default)]
+    #[allow(dead_code)]
+    pub audit_db_path: Option<PathBuf>,
+}
+
+fn home_path() -> PathBuf {
+    std::env::var_os("HOME")
+        .map(PathBuf::from)
+        .unwrap_or_else(|| PathBuf::from("."))
+}
+
+fn sieve_home() -> PathBuf {
+    home_path().join(".sieve")
+}
+
+fn default_ipc_socket() -> PathBuf {
+    sieve_home().join("ipc.sock")
+}
+
+fn default_pending_dir() -> PathBuf {
+    sieve_home().join("pending")
+}
+
+fn default_decisions_dir() -> PathBuf {
+    sieve_home().join("decisions")
+}
+
+fn default_launchd_plist() -> PathBuf {
+    home_path()
+        .join("Library")
+        .join("LaunchAgents")
+        .join("com.sieve.daemon.plist")
+}
+
+fn default_gui_socket_enabled() -> bool {
+    false
 }
 
 fn default_upstream() -> String {
@@ -87,6 +176,13 @@ fn default() -> Self {
             sieveignore_path: None,
             dry_run: false,
             inbound_rules_path: None,
+            ipc_socket_path: default_ipc_socket(),
+            pending_dir: default_pending_dir(),
+            decisions_dir: default_decisions_dir(),
+            preset: Preset::default(),
+            launchd_plist_path: default_launchd_plist(),
+            gui_socket_enabled: default_gui_socket_enabled(),
+            audit_db_path: None,
         }
     }
 }
diff --git a/crates/sieve-cli/src/main.rs b/crates/sieve-cli/src/main.rs
index ff180aa..e1bf315 100644
--- a/crates/sieve-cli/src/main.rs
+++ b/crates/sieve-cli/src/main.rs
@@ -1,9 +1,15 @@
 //! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
 //!
-//! 唯一子命令：`sieve start [--config <path>] [--dry-run]` 启动 daemon；
-//! `sieve version` 打印版本号。
+//! 子命令：
+//! - `sieve start [--config <path>] [--dry-run]`：启动 daemon
+//! - `sieve version`：打印版本号
+//! - `sieve setup [--dry-run] [--yes]`：自动配置 Claude Code（仅 macOS，ADR-015）
+//! - `sieve doctor`：诊断 Sieve 安装状态（仅 macOS）
+//! - `sieve uninstall [--dry-run] [--yes]`：回滚 setup 改动（仅 macOS）
 
-#![forbid(unsafe_code)]
+// unsafe_code 在生产代码中禁止（等效 forbid），测试代码通过 #[allow(unsafe_code)] 豁免
+// 以支持 Rust 1.80+ 的 std::env::set_var 必须用 unsafe {} 的要求。
+#![deny(unsafe_code)]
 
 use anyhow::{Context, Result};
 use clap::Parser;
@@ -13,6 +19,7 @@
 
 mod audit;
 mod cli;
+mod commands;
 mod config;
 mod daemon;
 mod engine_adapter;
@@ -125,6 +132,15 @@ async fn main() -> Result<()> {
         Command::Version => {
             println!("sieve {}", env!("CARGO_PKG_VERSION"));
         }
+        Command::Setup(args) => {
+            commands::setup::run(args)?;
+        }
+        Command::Doctor => {
+            commands::doctor::run()?;
+        }
+        Command::Uninstall(args) => {
+            commands::uninstall::run(args)?;
+        }
     }
 
     Ok(())
diff --git a/crates/sieve-core/Cargo.toml b/crates/sieve-core/Cargo.toml
index 15b1e60..a225017 100644
--- a/crates/sieve-core/Cargo.toml
+++ b/crates/sieve-core/Cargo.toml
@@ -14,6 +14,9 @@ default = ["forwarder"]
 # forwarder：启用 hyper/rustls/tokio 网络栈与 aws-lc-rs C 依赖。
 # 关闭后 sieve-core 仅保留纯 Rust 模块（sse/aggregator/protocol/pipeline 等），
 # 用于 cargo fuzz 等需要 sanitizer instrumentation 的场景，避免 ASan 链接 sancov 符号失败。
+#
+# v1.4 注：dispatch / inbound_hold / inbound_hook 依赖 bytes + tokio(async)，
+# 这些模块通过 #[cfg(feature = "forwarder")] 与 fuzz no-feature 场景隔离。
 forwarder = [
     "dep:tokio",
     "dep:hyper",
@@ -38,6 +41,8 @@ http = { workspace = true, optional = true }
 http-body = { workspace = true, optional = true }
 http-body-util = { workspace = true, optional = true }
 bytes = { workspace = true, optional = true }
+# v1.4：sieve-ipc 用于 pipeline dispatch / hook / hold 路径（forwarder feature 下）
+sieve-ipc = { path = "../sieve-ipc" }
 serde = { workspace = true }
 serde_json = { workspace = true }
 thiserror = { workspace = true }
@@ -45,6 +50,8 @@ tracing = { workspace = true }
 uuid = { version = "1", features = ["v4", "serde"] }
 sha2 = "0.10"
 strsim = "0.11"
+chrono = { workspace = true }
 
 [dev-dependencies]
 tokio = { workspace = true, features = ["full", "test-util"] }
+tempfile = "3"
//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup）。
//!
//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
//!
//! 步骤：
//! 1. 检测 `~/.claude/settings.json` 是否存在
//! 2. 计算 diff（ANTHROPIC_BASE_URL + PreToolUse hook + launchd plist）
//! 3. dry-run 打印 diff，非 --yes 等待用户确认
//! 4. 备份原文件到 `~/.sieve/backups/<RFC3339>/`
//! 5. 写 `~/.sieve/sieve.toml`（默认配置，绝对路径）
//! 6. 修改 settings.json（解析失败则 abort，不写任何内容）
//! 7. 写 launchd plist（命令包含 `--config <abs_path>/sieve.toml`）+ `launchctl load -w`
//! 8. 写 setup.log（JSON Lines，含 created_new 字段）
//! 9. 自动调用 doctor 验证
//!
//! 错误恢复：任意步骤失败 → 反向回滚已做改动。

use crate::cli::SetupArgs;
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::commands::doctor;
    use anyhow::{anyhow, bail, Context};
    use chrono::Utc;
    use serde_json::Value;
    use std::fs;
    use std::io::{self, Write as IoWrite};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// setup.log 每行的结构（JSON Lines）。
    ///
    /// `created_new`：true 表示 setup 前该文件不存在，由 setup 新建；
    /// uninstall 时 `created_new=true` 的文件直接删除，`false` 的从备份恢复。
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SetupLogEntry {
        pub timestamp: String,
        pub action: String,
        pub path: Option<String>,
        pub detail: Option<String>,
        /// setup 前该文件是否不存在（新建 vs 覆盖）。
        #[serde(default)]
        pub created_new: bool,
    }

    impl SetupLogEntry {
        pub(super) fn new(action: impl Into<String>) -> Self {
            Self {
                timestamp: Utc::now().to_rfc3339(),
                action: action.into(),
                path: None,
                detail: None,
                created_new: false,
            }
        }

        pub(super) fn with_path(mut self, path: impl Into<String>) -> Self {
            self.path = Some(path.into());
            self
        }

        pub(super) fn with_detail(mut self, detail: impl Into<String>) -> Self {
            self.detail = Some(detail.into());
            self
        }

        pub(super) fn with_created_new(mut self, created_new: bool) -> Self {
            self.created_new = created_new;
            self
        }
    }

    /// setup 执行上下文，用于错误时反向回滚。
    struct SetupContext {
        backup_dir: PathBuf,
        /// 已写入的文件路径，错误时按逆序恢复。
        written_files: Vec<PathBuf>,
        /// 已执行的 launchctl load，错误时需要 unload。
        launchd_loaded: Option<PathBuf>,
    }

    impl SetupContext {
        fn new(backup_dir: PathBuf) -> Self {
            Self {
                backup_dir,
                written_files: Vec::new(),
                launchd_loaded: None,
            }
        }

        /// 回滚所有已做改动（从备份目录恢复）。
        fn rollback(&self) {
            eprintln!("[sieve setup] 回滚已做改动…");

            if let Some(plist) = &self.launchd_loaded {
                let _ = Command::new("launchctl")
                    .args(["unload", &plist.to_string_lossy()])
                    .status();
                eprintln!("  ↩ launchctl unload {}", plist.display());
            }

            for path in self.written_files.iter().rev() {
                // 计算备份中的相对路径：去掉 HOME 前缀
                let home = std::env::var("HOME").unwrap_or_default();
                let rel = path.strip_prefix(&home).unwrap_or(path.as_path());
                let backup_src = self.backup_dir.join(rel);
                if backup_src.exists() {
                    if let Err(e) = fs::copy(&backup_src, path) {
                        eprintln!("  ✗ 恢复 {} 失败: {}", path.display(), e);
                    } else {
                        eprintln!("  ↩ 恢复 {}", path.display());
                    }
                } else {
                    // 备份不存在说明是新建的，直接删除
                    let _ = fs::remove_file(path);
                    eprintln!("  ↩ 删除新建文件 {}", path.display());
                }
            }
        }
    }

    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
    pub fn run(args: SetupArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);

        let settings_path = home_path.join(".claude").join("settings.json");
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
        let backup_dir = sieve_home.join("backups").join(&backup_ts);
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let setup_log_path = sieve_home.join("setup.log");

        // ── 1. 读取现有 settings.json（允许不存在；解析失败则 abort，不覆盖用户文件）
        let settings_existed_before = settings_path.exists();
        let existing_settings: Value = if settings_existed_before {
            let raw =
                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
            // Strip JSON 注释（简单处理：删除 // 行注释）
            let stripped = strip_json_comments(&raw);
            serde_json::from_str(&stripped).map_err(|e| {
                anyhow!(
                    "无法解析 ~/.claude/settings.json：{}。\n\
                     请用 JSON 校验工具修复后重试。setup 已 abort，未做任何改动。",
                    e
                )
            })?
        } else {
            serde_json::json!({})
        };
        // sieve.toml 将写入 ~/.sieve/sieve.toml（绝对路径）
        let sieve_toml_path = sieve_home.join("sieve.toml");

        // ── 2. 计算 diff
        let sieve_url = "http://127.0.0.1:11453";
        let hook_entry = serde_json::json!({
            "matcher": ".*",
            "hooks": [{"type": "command", "command": "sieve-hook check"}]
        });

        let current_base_url = existing_settings
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|v| v.as_str())
            .unwrap_or("<未设置>");
        let has_hook = existing_settings
            .pointer("/hooks/PreToolUse")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        let plist_content = build_plist_content(&sieve_toml_path)?;

        // ── 3. 打印 diff
        println!("=== sieve setup diff ===");
        println!(
            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
            current_base_url, sieve_url
        );
        if has_hook {
            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
        } else {
            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
        }
        if sieve_toml_path.exists() {
            println!(
                "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
                sieve_toml_path.display()
            );
        } else {
            println!("[sieve.toml] 新建 {}", sieve_toml_path.display());
        }
        println!(
            "[launchd] 写入 {} (含 --config {})",
            plist_path.display(),
            sieve_toml_path.display()
        );
        println!("[launchd] 执行 launchctl load -w");
        println!("========================");

        // ── 4. dry-run 直接返回
//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor）。
//!
//! 5 项检查：
//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
//! 2. hooks.PreToolUse 是否含 sieve-hook check
//! 3. daemon 是否在 :11453 监听（TCP 连接）
//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
//! 5. canary 拦截测试（构造 OUT-01 已知字符串，验证 daemon 改写）
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。

use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::io::Write as IoWrite;
    use std::process::Command;

    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
    pub fn run() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_default();
        let settings_path = std::path::PathBuf::from(&home)
            .join(".claude")
            .join("settings.json");

        let mut all_ok = true;

        // ── 检查 1: ANTHROPIC_BASE_URL
        let check1 = check_base_url(&settings_path);
        print_check(
            "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
            check1,
        );
        all_ok &= check1;

        // ── 检查 2: PreToolUse hook
        let check2 = check_hook_registered(&settings_path);
        print_check(
            "settings.json: hooks.PreToolUse 含 sieve-hook check",
            check2,
        );
        all_ok &= check2;

        // ── 检查 3: daemon 监听 :11453
        let check3 = check_daemon_listening();
        print_check("daemon 在 127.0.0.1:11453 监听", check3);
        all_ok &= check3;

        // ── 检查 4: launchd 状态
        let check4 = check_launchd();
        print_check("launchd com.sieve.daemon 已加载", check4);
        all_ok &= check4;

        // ── 检查 5: canary 拦截测试
        let check5 = check_canary();
        print_check("canary 拦截测试（OUT-01 脱敏）", check5);
        all_ok &= check5;

        // ── 汇总
        println!();
        if all_ok {
            println!("✅ 所有检查通过，Sieve 运行正常。");
        } else {
            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
        }

        Ok(())
    }

    fn print_check(label: &str, ok: bool) {
        let icon = if ok { "✅" } else { "❌" };
        println!("  {} {}", icon, label);
    }

    /// 检查 settings.json 中 ANTHROPIC_BASE_URL。
    fn check_base_url(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|x| x.as_str())
            .map(|s| s == "http://127.0.0.1:11453")
            .unwrap_or(false)
    }

    /// 检查 PreToolUse hook 是否含 sieve-hook check。
    fn check_hook_registered(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/hooks/PreToolUse")
            .and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    /// 尝试 TCP 连接 127.0.0.1:11453，成功则 daemon 在监听。
    fn check_daemon_listening() -> bool {
        use std::net::TcpStream;
        use std::time::Duration;
        TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
    }

    /// 检查 launchctl list 是否含 com.sieve.daemon。
    fn check_launchd() -> bool {
        let Ok(output) = Command::new("launchctl").arg("list").output() else {
            return false;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("com.sieve.daemon")
    }

    /// Canary 拦截测试：向 daemon 发送含 OUT-01 特征的请求，
    /// 验证响应中已脱敏（不含原始 sk- token）。
    ///
    /// 注意：此测试仅在 daemon 运行时有意义；daemon 未运行时直接返回 false。
    fn check_canary() -> bool {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        // daemon 未运行直接 false
        let Ok(mut stream) = TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        ) else {
            return false;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));

        // 构造含已知 OUT-01 特征（sk-ant-api03-... 格式）的请求体
        // 注意：这里使用测试用虚假 token，格式符合 OUT-01 模式
        let canary_token = "sk-ant-api03-canary-test-aaaabbbbccccdddd-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX_AA";
        let body = serde_json::json!({
            "model": "claude-3-5-haiku-20241022",
            "max_tokens": 1,
            "messages": [{
                "role": "user",
                "content": format!("hello {canary_token}")
            }]
        })
        .to_string();

        let request = format!(
            "POST /v1/messages HTTP/1.1\r\n\
             Host: 127.0.0.1:11453\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             x-api-key: test\r\n\
             anthropic-version: 2023-06-01\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );

        if stream.write_all(request.as_bytes()).is_err() {
            return false;
        }

        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);

        // 验证响应中不含原始 canary token（已被脱敏/拦截）
        !response.contains(canary_token)
    }

    // 抑制 IoWrite 未使用警告
    const _: fn() = || {
        let _ = std::io::stdout().flush();
    };
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve doctor` 非 macOS 占位实现。
    pub fn run() -> Result<()> {
        anyhow::bail!(
            "sieve doctor is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}
//! `sieve uninstall` 命令实现（ADR-015 / SPEC-003 §uninstall）。
//!
//! 步骤：
//! 1. 读 `~/.sieve/setup.log` 反向遍历 entries（了解 backup_dir + created_new 标志）
//! 2. dry-run 打印将恢复的内容
//! 3. 非 --yes 等待用户确认
//! 4. 按 setup.log 记录的 created_new 字段决定还原策略：
//!    - `created_new = true`：setup 前不存在，直接删除（恢复"原状"）
//!    - `created_new = false`：仅移除 Sieve entries（ANTHROPIC_BASE_URL + sieve-hook），
//!      保留用户 setup 后添加的其他配置
//! 5. `launchctl unload` 并删除 plist 文件
//! 6. 提示用户手动删 `~/.sieve/`
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。

use crate::cli::UninstallArgs;
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use anyhow::{anyhow, Context};
    use std::fs;
    use std::io::{self, Write as IoWrite};
    use std::path::PathBuf;
    use std::process::Command;

    /// setup.log entry 镜像（只读取需要的字段）。
    #[derive(serde::Deserialize)]
    struct SetupLogEntry {
        action: String,
        path: Option<String>,
        detail: Option<String>,
        #[serde(default)]
        created_new: bool,
    }

    /// 记录 setup 写入文件的还原策略。
    pub(super) struct FileRestoreInfo {
        /// 文件绝对路径。
        pub(super) path: PathBuf,
        /// true → setup 前不存在，uninstall 时删除；false → 仅移除 Sieve entries。
        pub(super) created_new: bool,
    }

    /// 运行 `sieve uninstall`。关联 ADR-015 / SPEC-003 §uninstall。
    pub fn run(args: UninstallArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let setup_log_path = sieve_home.join("setup.log");
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let backups_root = sieve_home.join("backups");

        // ── 1. 读取 setup.log，找到最新 backup_dir + 各文件 created_new 标志
        let (latest_backup, file_restore_infos) = read_setup_log(&setup_log_path, &backups_root);

        // ── 2. 打印将要恢复的内容
        println!("=== sieve uninstall 预览 ===");
        if !file_restore_infos.is_empty() {
            for info in &file_restore_infos {
                if info.created_new {
                    println!("[restore] 删除（setup 新建）: {}", info.path.display());
                } else {
                    println!("[restore] 移除 Sieve entries: {}", info.path.display());
                }
            }
        } else if let Some(ref bd) = latest_backup {
            println!("[restore] 从备份目录恢复: {}", bd.display());
            list_backup_files(bd);
        } else {
            println!("[restore] 未找到 setup.log 记录，将跳过文件恢复");
        }
        if plist_path.exists() {
            println!("[launchd] launchctl unload {}", plist_path.display());
            println!("[launchd] 删除 {}", plist_path.display());
        }
        println!("[提示] ~/.sieve/ 目录将保留（含审计日志），请手动删除：");
        println!("       rm -rf {}", sieve_home.display());
        println!("===========================");

        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 3. 等待用户确认
        if !args.yes {
            print!("继续执行以上操作？[y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("已取消。");
                return Ok(());
            }
        }

        // ── 4. 按 created_new 标志决定还原策略
        if !file_restore_infos.is_empty() {
            restore_files(&file_restore_infos, &home_path, latest_backup.as_deref())?;
        } else if let Some(ref bd) = latest_backup {
            // 旧格式 setup.log（无 created_new），退回全量备份恢复
            restore_from_backup(bd, &home_path)?;
        }

        // ── 5. 卸载 launchd
        if plist_path.exists() {
            let status = Command::new("launchctl")
                .args(["unload", &plist_path.to_string_lossy()])
                .status();
            match status {
                Ok(s) if s.success() => println!("[uninstall] ✅ launchd 服务已卸载"),
                Ok(s) => eprintln!("[uninstall] ⚠ launchctl unload 返回: {:?}", s.code()),
                Err(e) => eprintln!("[uninstall] ⚠ launchctl unload 失败: {e}"),
            }
            if let Err(e) = fs::remove_file(&plist_path) {
                eprintln!("[uninstall] ⚠ 删除 plist 失败: {e}");
            } else {
                println!("[uninstall] ✅ plist 已删除");
            }
        }

        // ── 6. 提示手动删除
        println!();
        println!("✅ 卸载完成。");
        println!("提示：审计日志和备份文件保留在 {}", sieve_home.display());
        println!("如需彻底清除，请手动运行：");
        println!("  rm -rf {}", sieve_home.display());

        Ok(())
    }

    /// 从 setup.log 读取最新 backup_dir 和文件还原信息。
    ///
    /// 返回 (latest_backup_dir, file_restore_infos)。
    /// file_restore_infos 为空时表示 setup.log 是旧格式，退回全量备份恢复。
    fn read_setup_log(
        setup_log: &std::path::Path,
        backups_root: &std::path::Path,
    ) -> (Option<PathBuf>, Vec<FileRestoreInfo>) {
        let Ok(raw) = fs::read_to_string(setup_log) else {
            // setup.log 不存在，扫描 backups/ 最新目录兜底
            return (find_latest_backup_dir(backups_root), vec![]);
        };

        let entries: Vec<SetupLogEntry> = raw
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        // 找最新 setup_complete entry 的 backup_dir
        let latest_backup = entries
            .iter()
            .rev()
            .find(|e| e.action == "setup_complete")
            .and_then(|e| e.detail.as_deref())
            .and_then(|d| d.strip_prefix("backup_dir="))
            .map(PathBuf::from);

        // 收集文件 action（settings_updated / sieve_toml_written），取最新一次 setup 的记录
        // 策略：找最后一个 setup_complete 之后的所有文件 action
        let last_setup_idx = entries
            .iter()
            .rposition(|e| e.action == "setup_complete")
            .unwrap_or(0);

        let file_actions = ["settings_updated", "sieve_toml_written"];
        let infos: Vec<FileRestoreInfo> = entries[last_setup_idx..]
            .iter()
            .filter(|e| file_actions.contains(&e.action.as_str()))
            .filter_map(|e| {
                let path_str = e.path.as_deref()?;
                Some(FileRestoreInfo {
                    path: PathBuf::from(path_str),
                    created_new: e.created_new,
                })
            })
            .collect();

        // 如果没有文件记录（旧格式 setup.log），返回空 infos 触发备份恢复兜底
        let backup = latest_backup.or_else(|| find_latest_backup_dir(backups_root));
        (backup, infos)
    }

    /// 扫描 backups/ 下最新目录（按名称字典序，RFC3339 时间戳排序正确）。
    fn find_latest_backup_dir(backups_root: &std::path::Path) -> Option<PathBuf> {
        if !backups_root.exists() {
            return None;
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(backups_root)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        entries.sort();
        entries.into_iter().next_back()
    }

    /// 按 created_new 标志还原文件。
    ///
    /// - `created_new = true`：setup 前不存在，直接删除
    /// - `created_new = false`：仅从文件内移除 Sieve entries（保留用户其他配置）
    pub(super) fn restore_files(
        infos: &[FileRestoreInfo],
        _home_path: &std::path::Path,
        backup_dir: Option<&std::path::Path>,
    ) -> Result<()> {
//! Week 5 CLI 子命令模块（ADR-015 / SPEC-003）。
//!
//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `uninstall`：干净回滚 setup 改动（仅 macOS）

pub mod doctor;
pub mod setup;
pub mod uninstall;

exec
/bin/zsh -lc "sed -n '120,420p' crates/sieve-cli/src/commands/setup.rs && sed -n '120,420p' crates/sieve-cli/src/commands/uninstall.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
                    } else {
                        eprintln!("  ↩ 恢复 {}", path.display());
                    }
                } else {
                    // 备份不存在说明是新建的，直接删除
                    let _ = fs::remove_file(path);
                    eprintln!("  ↩ 删除新建文件 {}", path.display());
                }
            }
        }
    }

    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
    pub fn run(args: SetupArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);

        let settings_path = home_path.join(".claude").join("settings.json");
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
        let backup_dir = sieve_home.join("backups").join(&backup_ts);
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let setup_log_path = sieve_home.join("setup.log");

        // ── 1. 读取现有 settings.json（允许不存在；解析失败则 abort，不覆盖用户文件）
        let settings_existed_before = settings_path.exists();
        let existing_settings: Value = if settings_existed_before {
            let raw =
                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
            // Strip JSON 注释（简单处理：删除 // 行注释）
            let stripped = strip_json_comments(&raw);
            serde_json::from_str(&stripped).map_err(|e| {
                anyhow!(
                    "无法解析 ~/.claude/settings.json：{}。\n\
                     请用 JSON 校验工具修复后重试。setup 已 abort，未做任何改动。",
                    e
                )
            })?
        } else {
            serde_json::json!({})
        };
        // sieve.toml 将写入 ~/.sieve/sieve.toml（绝对路径）
        let sieve_toml_path = sieve_home.join("sieve.toml");

        // ── 2. 计算 diff
        let sieve_url = "http://127.0.0.1:11453";
        let hook_entry = serde_json::json!({
            "matcher": ".*",
            "hooks": [{"type": "command", "command": "sieve-hook check"}]
        });

        let current_base_url = existing_settings
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|v| v.as_str())
            .unwrap_or("<未设置>");
        let has_hook = existing_settings
            .pointer("/hooks/PreToolUse")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        let plist_content = build_plist_content(&sieve_toml_path)?;

        // ── 3. 打印 diff
        println!("=== sieve setup diff ===");
        println!(
            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
            current_base_url, sieve_url
        );
        if has_hook {
            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
        } else {
            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
        }
        if sieve_toml_path.exists() {
            println!(
                "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
                sieve_toml_path.display()
            );
        } else {
            println!("[sieve.toml] 新建 {}", sieve_toml_path.display());
        }
        println!(
            "[launchd] 写入 {} (含 --config {})",
            plist_path.display(),
            sieve_toml_path.display()
        );
        println!("[launchd] 执行 launchctl load -w");
        println!("========================");

        // ── 4. dry-run 直接返回
        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 5. 等待用户确认
        if !args.yes {
            print!("继续执行以上操作？[y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("已取消。");
                return Ok(());
            }
        }

        // ── 6. 备份
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
        let mut ctx = SetupContext::new(backup_dir.clone());

        let result = do_setup(
            &mut ctx,
            &home_path,
            &settings_path,
            &plist_path,
            &sieve_toml_path,
            &setup_log_path,
            &backup_dir,
            existing_settings,
            settings_existed_before,
            sieve_url,
            hook_entry,
            plist_content,
        );

        if let Err(ref e) = result {
            eprintln!("[sieve setup] 失败: {e}");
            ctx.rollback();
            return result;
        }

        // ── 9. 自动跑 doctor 验证
        println!("\n[sieve setup] 正在验证安装…");
        doctor::run()?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn do_setup(
        ctx: &mut SetupContext,
        home_path: &Path,
        settings_path: &Path,
        plist_path: &Path,
        sieve_toml_path: &Path,
        setup_log_path: &Path,
        backup_dir: &Path,
        mut existing_settings: Value,
        settings_existed_before: bool,
        sieve_url: &str,
        hook_entry: Value,
        plist_content: String,
    ) -> Result<()> {
        // 备份 settings.json（仅在文件已存在时）
        if settings_existed_before {
            let rel = settings_path
                .strip_prefix(home_path)
                .unwrap_or(settings_path);
            let backup_dest = backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
        }

        // 修改 settings.json
        {
            let env = existing_settings
                .get_mut("env")
                .and_then(|v| v.as_object_mut())
                .map(|obj| {
                    obj.insert(
                        "ANTHROPIC_BASE_URL".to_string(),
                        serde_json::json!(sieve_url),
                    );
                })
                .is_some();
            if !env {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "env".to_string(),
                    serde_json::json!({"ANTHROPIC_BASE_URL": sieve_url}),
                );
            }

            // 追加 PreToolUse hook（幂等：已存在则跳过）
            let hooks_obj = existing_settings
                .get_mut("hooks")
                .and_then(|v| v.as_object_mut());
            if let Some(hooks) = hooks_obj {
                let pre_tool = hooks
                    .entry("PreToolUse")
                    .or_insert_with(|| serde_json::json!([]));
                if let Some(arr) = pre_tool.as_array_mut() {
                    let already = arr.iter().any(|item| {
                        item.pointer("/hooks/0/command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.contains("sieve-hook"))
                            .unwrap_or(false)
                    });
                    if !already {
                        arr.push(hook_entry);
                    }
                }
            } else {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "hooks".to_string(),
                    serde_json::json!({"PreToolUse": [hook_entry]}),
                );
            }

            // 确保父目录存在
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json_str = serde_json::to_string_pretty(&existing_settings)?;
            fs::write(settings_path, json_str.as_bytes()).context("写入 settings.json 失败")?;
            ctx.written_files.push(settings_path.to_path_buf());
            println!("[setup] ✅ settings.json 已更新");
        }

        // 写 ~/.sieve/sieve.toml（绝对路径配置，供 launchd plist 引用）
        let sieve_toml_existed_before = sieve_toml_path.exists();
        {
            if sieve_toml_existed_before {
                // 备份已有 sieve.toml
                let rel = sieve_toml_path
                    .strip_prefix(home_path)
                    .unwrap_or(sieve_toml_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
            }
            if let Some(parent) = sieve_toml_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_content = build_default_sieve_toml(sieve_toml_path)?;
            fs::write(sieve_toml_path, toml_content.as_bytes()).context("写入 sieve.toml 失败")?;
            ctx.written_files.push(sieve_toml_path.to_path_buf());
            println!("[setup] ✅ sieve.toml 写入 {}", sieve_toml_path.display());
        }

        // 写 launchd plist
        {
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 备份已有 plist
            if plist_path.exists() {
                let rel = plist_path.strip_prefix(home_path).unwrap_or(plist_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
            }
            fs::write(plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
            ctx.written_files.push(plist_path.to_path_buf());
            println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());
        }

        // launchctl load
        {
            let status = Command::new("launchctl")
                .args(["load", "-w", &plist_path.to_string_lossy()])
                .status()
                .context("执行 launchctl load 失败")?;
            if !status.success() {
                bail!("launchctl load 返回非零: {:?}", status.code());
            }
            ctx.launchd_loaded = Some(plist_path.to_path_buf());
            println!("[setup] ✅ launchd 服务已加载");
        }

        // 写 setup.log（含 created_new 字段，供 uninstall 精确还原）
        {
            let entries: Vec<SetupLogEntry> = vec![
                SetupLogEntry::new("setup_complete")
                    .with_detail(format!("backup_dir={}", backup_dir.display())),
                SetupLogEntry::new("settings_updated")
                    .with_path(settings_path.to_string_lossy().to_string())
        if plist_path.exists() {
            let status = Command::new("launchctl")
                .args(["unload", &plist_path.to_string_lossy()])
                .status();
            match status {
                Ok(s) if s.success() => println!("[uninstall] ✅ launchd 服务已卸载"),
                Ok(s) => eprintln!("[uninstall] ⚠ launchctl unload 返回: {:?}", s.code()),
                Err(e) => eprintln!("[uninstall] ⚠ launchctl unload 失败: {e}"),
            }
            if let Err(e) = fs::remove_file(&plist_path) {
                eprintln!("[uninstall] ⚠ 删除 plist 失败: {e}");
            } else {
                println!("[uninstall] ✅ plist 已删除");
            }
        }

        // ── 6. 提示手动删除
        println!();
        println!("✅ 卸载完成。");
        println!("提示：审计日志和备份文件保留在 {}", sieve_home.display());
        println!("如需彻底清除，请手动运行：");
        println!("  rm -rf {}", sieve_home.display());

        Ok(())
    }

    /// 从 setup.log 读取最新 backup_dir 和文件还原信息。
    ///
    /// 返回 (latest_backup_dir, file_restore_infos)。
    /// file_restore_infos 为空时表示 setup.log 是旧格式，退回全量备份恢复。
    fn read_setup_log(
        setup_log: &std::path::Path,
        backups_root: &std::path::Path,
    ) -> (Option<PathBuf>, Vec<FileRestoreInfo>) {
        let Ok(raw) = fs::read_to_string(setup_log) else {
            // setup.log 不存在，扫描 backups/ 最新目录兜底
            return (find_latest_backup_dir(backups_root), vec![]);
        };

        let entries: Vec<SetupLogEntry> = raw
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        // 找最新 setup_complete entry 的 backup_dir
        let latest_backup = entries
            .iter()
            .rev()
            .find(|e| e.action == "setup_complete")
            .and_then(|e| e.detail.as_deref())
            .and_then(|d| d.strip_prefix("backup_dir="))
            .map(PathBuf::from);

        // 收集文件 action（settings_updated / sieve_toml_written），取最新一次 setup 的记录
        // 策略：找最后一个 setup_complete 之后的所有文件 action
        let last_setup_idx = entries
            .iter()
            .rposition(|e| e.action == "setup_complete")
            .unwrap_or(0);

        let file_actions = ["settings_updated", "sieve_toml_written"];
        let infos: Vec<FileRestoreInfo> = entries[last_setup_idx..]
            .iter()
            .filter(|e| file_actions.contains(&e.action.as_str()))
            .filter_map(|e| {
                let path_str = e.path.as_deref()?;
                Some(FileRestoreInfo {
                    path: PathBuf::from(path_str),
                    created_new: e.created_new,
                })
            })
            .collect();

        // 如果没有文件记录（旧格式 setup.log），返回空 infos 触发备份恢复兜底
        let backup = latest_backup.or_else(|| find_latest_backup_dir(backups_root));
        (backup, infos)
    }

    /// 扫描 backups/ 下最新目录（按名称字典序，RFC3339 时间戳排序正确）。
    fn find_latest_backup_dir(backups_root: &std::path::Path) -> Option<PathBuf> {
        if !backups_root.exists() {
            return None;
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(backups_root)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        entries.sort();
        entries.into_iter().next_back()
    }

    /// 按 created_new 标志还原文件。
    ///
    /// - `created_new = true`：setup 前不存在，直接删除
    /// - `created_new = false`：仅从文件内移除 Sieve entries（保留用户其他配置）
    pub(super) fn restore_files(
        infos: &[FileRestoreInfo],
        _home_path: &std::path::Path,
        backup_dir: Option<&std::path::Path>,
    ) -> Result<()> {
        for info in infos {
            if !info.path.exists() {
                println!("[uninstall] 跳过（文件不存在）: {}", info.path.display());
                continue;
            }

            if info.created_new {
                // setup 前不存在 → 删除整个文件
                fs::remove_file(&info.path)
                    .with_context(|| format!("删除 setup 新建文件 {} 失败", info.path.display()))?;
                println!("[uninstall] ✅ 删除（setup 新建）: {}", info.path.display());
            } else {
                // setup 前已存在 → 仅移除 Sieve entries，保留用户其他配置
                // 对 settings.json：移除 env.ANTHROPIC_BASE_URL + hooks.PreToolUse 中 sieve-hook 条目
                let extension = info.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if extension == "json" {
                    match remove_sieve_entries_from_settings(&info.path) {
                        Ok(()) => {
                            println!("[uninstall] ✅ 移除 Sieve entries: {}", info.path.display());
                        }
                        Err(e) => {
                            // 移除 entries 失败，退回备份恢复
                            eprintln!("[uninstall] ⚠ 移除 entries 失败: {e}，尝试从备份恢复");
                            if let Some(bd) = backup_dir {
                                restore_file_from_backup(bd, &info.path)?;
                            }
                        }
                    }
                } else if extension == "toml" {
                    // sieve.toml 属于 Sieve 自己，直接删除
                    fs::remove_file(&info.path)
                        .with_context(|| format!("删除 {} 失败", info.path.display()))?;
                    println!("[uninstall] ✅ 删除: {}", info.path.display());
                } else {
                    // 其他文件：从备份恢复
                    if let Some(bd) = backup_dir {
                        restore_file_from_backup(bd, &info.path)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 从 settings.json 中移除 Sieve 注入的 entries，保留用户其他配置。
    ///
    /// 移除：
    /// - `env.ANTHROPIC_BASE_URL`（若值为 `http://127.0.0.1:11453`）
    /// - `hooks.PreToolUse` 数组中包含 `sieve-hook` 的条目
    pub(super) fn remove_sieve_entries_from_settings(
        settings_path: &std::path::Path,
    ) -> Result<()> {
        let raw = fs::read_to_string(settings_path)
            .with_context(|| format!("读取 {} 失败", settings_path.display()))?;
        let mut v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("解析 {} 失败", settings_path.display()))?;

        // 移除 env.ANTHROPIC_BASE_URL（仅当值为 sieve url 时）
        if let Some(env) = v.get_mut("env").and_then(|e| e.as_object_mut()) {
            if env
                .get("ANTHROPIC_BASE_URL")
                .and_then(|u| u.as_str())
                .map(|s| s == "http://127.0.0.1:11453")
                .unwrap_or(false)
            {
                env.remove("ANTHROPIC_BASE_URL");
                // 如果 env 对象变空，也一并移除（避免留下空对象）
                if env.is_empty() {
                    v.as_object_mut().map(|obj| obj.remove("env"));
                }
            }
        }

        // 移除 hooks.PreToolUse 中含 sieve-hook 的条目
        if let Some(pre_tool) = v
            .pointer_mut("/hooks/PreToolUse")
            .and_then(|a| a.as_array_mut())
        {
            pre_tool.retain(|item| {
                !item
                    .pointer("/hooks/0/command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("sieve-hook"))
                    .unwrap_or(false)
            });
        }
        // 如果 hooks.PreToolUse 变空，移除该 key
        let pre_tool_empty = v
            .pointer("/hooks/PreToolUse")
            .and_then(|a| a.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false);
        if pre_tool_empty {
            if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                hooks.remove("PreToolUse");
                if hooks.is_empty() {
                    v.as_object_mut().map(|obj| obj.remove("hooks"));
                }
            }
        }

        let json_str = serde_json::to_string_pretty(&v)?;
        fs::write(settings_path, json_str.as_bytes())
            .with_context(|| format!("写入 {} 失败", settings_path.display()))?;
        Ok(())
    }

    /// 从备份目录恢复单个文件。
    fn restore_file_from_backup(
        backup_dir: &std::path::Path,
        target: &std::path::Path,
    ) -> Result<()> {
        // 计算 backup 中的对应路径（target 的绝对路径去掉 HOME 前缀）
        let home = std::env::var("HOME").unwrap_or_default();
        let rel = target.strip_prefix(&home).unwrap_or(target);
        let backup_src = backup_dir.join(rel);
        if backup_src.exists() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&backup_src, target).with_context(|| {
                format!(
                    "从备份恢复 {} → {} 失败",
                    backup_src.display(),
                    target.display()
                )
            })?;
            println!("[uninstall] ✅ 从备份恢复: {}", target.display());
        } else {
            eprintln!("[uninstall] ⚠ 备份文件不存在: {}", backup_src.display());
        }
        Ok(())
    }

    /// 打印备份目录中的文件列表。
    fn list_backup_files(backup_dir: &std::path::Path) {
        if let Ok(walker) = fs::read_dir(backup_dir) {
            for entry in walker.flatten() {
                println!("  - {}", entry.path().display());
            }
        }
    }

    /// 将备份目录中的文件逐一恢复到 home 下对应路径（旧格式 setup.log 兜底）。
    fn restore_from_backup(
        backup_dir: &std::path::Path,
        home_path: &std::path::Path,
    ) -> Result<()> {
        restore_dir_recursive(backup_dir, backup_dir, home_path)
    }

    fn restore_dir_recursive(
        root: &std::path::Path,
        current: &std::path::Path,
        home_path: &std::path::Path,
    ) -> Result<()> {
        for entry in fs::read_dir(current)
            .with_context(|| format!("读取备份目录 {} 失败", current.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                restore_dir_recursive(root, &path, home_path)?;
            } else {
                // 计算目标路径：backup_root/rel → home/rel
                let rel = path.strip_prefix(root).unwrap_or(path.as_path());
                let dest = home_path.join(rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest).with_context(|| {
                    format!("恢复 {} → {} 失败", path.display(), dest.display())
                })?;
                println!("[uninstall] ✅ 恢复 {}", dest.display());
            }
        }
        Ok(())
    }
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve uninstall` 非 macOS 占位实现。
    pub fn run(_args: UninstallArgs) -> Result<()> {
        anyhow::bail!(
            "sieve uninstall is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}

// ──────────────────────────────── 单元测试 ──────────────────────────────────

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '250,520p'; nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '1,320p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   250	            &backup_dir,
   251	            existing_settings,
   252	            settings_existed_before,
   253	            sieve_url,
   254	            hook_entry,
   255	            plist_content,
   256	        );
   257	
   258	        if let Err(ref e) = result {
   259	            eprintln!("[sieve setup] 失败: {e}");
   260	            ctx.rollback();
   261	            return result;
   262	        }
   263	
   264	        // ── 9. 自动跑 doctor 验证
   265	        println!("\n[sieve setup] 正在验证安装…");
   266	        doctor::run()?;
   267	
   268	        Ok(())
   269	    }
   270	
   271	    #[allow(clippy::too_many_arguments)]
   272	    fn do_setup(
   273	        ctx: &mut SetupContext,
   274	        home_path: &Path,
   275	        settings_path: &Path,
   276	        plist_path: &Path,
   277	        sieve_toml_path: &Path,
   278	        setup_log_path: &Path,
   279	        backup_dir: &Path,
   280	        mut existing_settings: Value,
   281	        settings_existed_before: bool,
   282	        sieve_url: &str,
   283	        hook_entry: Value,
   284	        plist_content: String,
   285	    ) -> Result<()> {
   286	        // 备份 settings.json（仅在文件已存在时）
   287	        if settings_existed_before {
   288	            let rel = settings_path
   289	                .strip_prefix(home_path)
   290	                .unwrap_or(settings_path);
   291	            let backup_dest = backup_dir.join(rel);
   292	            if let Some(parent) = backup_dest.parent() {
   293	                fs::create_dir_all(parent)?;
   294	            }
   295	            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
   296	        }
   297	
   298	        // 修改 settings.json
   299	        {
   300	            let env = existing_settings
   301	                .get_mut("env")
   302	                .and_then(|v| v.as_object_mut())
   303	                .map(|obj| {
   304	                    obj.insert(
   305	                        "ANTHROPIC_BASE_URL".to_string(),
   306	                        serde_json::json!(sieve_url),
   307	                    );
   308	                })
   309	                .is_some();
   310	            if !env {
   311	                let obj = existing_settings
   312	                    .as_object_mut()
   313	                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
   314	                obj.insert(
   315	                    "env".to_string(),
   316	                    serde_json::json!({"ANTHROPIC_BASE_URL": sieve_url}),
   317	                );
   318	            }
   319	
   320	            // 追加 PreToolUse hook（幂等：已存在则跳过）
   321	            let hooks_obj = existing_settings
   322	                .get_mut("hooks")
   323	                .and_then(|v| v.as_object_mut());
   324	            if let Some(hooks) = hooks_obj {
   325	                let pre_tool = hooks
   326	                    .entry("PreToolUse")
   327	                    .or_insert_with(|| serde_json::json!([]));
   328	                if let Some(arr) = pre_tool.as_array_mut() {
   329	                    let already = arr.iter().any(|item| {
   330	                        item.pointer("/hooks/0/command")
   331	                            .and_then(|c| c.as_str())
   332	                            .map(|c| c.contains("sieve-hook"))
   333	                            .unwrap_or(false)
   334	                    });
   335	                    if !already {
   336	                        arr.push(hook_entry);
   337	                    }
   338	                }
   339	            } else {
   340	                let obj = existing_settings
   341	                    .as_object_mut()
   342	                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
   343	                obj.insert(
   344	                    "hooks".to_string(),
   345	                    serde_json::json!({"PreToolUse": [hook_entry]}),
   346	                );
   347	            }
   348	
   349	            // 确保父目录存在
   350	            if let Some(parent) = settings_path.parent() {
   351	                fs::create_dir_all(parent)?;
   352	            }
   353	            let json_str = serde_json::to_string_pretty(&existing_settings)?;
   354	            fs::write(settings_path, json_str.as_bytes()).context("写入 settings.json 失败")?;
   355	            ctx.written_files.push(settings_path.to_path_buf());
   356	            println!("[setup] ✅ settings.json 已更新");
   357	        }
   358	
   359	        // 写 ~/.sieve/sieve.toml（绝对路径配置，供 launchd plist 引用）
   360	        let sieve_toml_existed_before = sieve_toml_path.exists();
   361	        {
   362	            if sieve_toml_existed_before {
   363	                // 备份已有 sieve.toml
   364	                let rel = sieve_toml_path
   365	                    .strip_prefix(home_path)
   366	                    .unwrap_or(sieve_toml_path);
   367	                let backup_dest = backup_dir.join(rel);
   368	                if let Some(parent) = backup_dest.parent() {
   369	                    fs::create_dir_all(parent)?;
   370	                }
   371	                fs::copy(sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
   372	            }
   373	            if let Some(parent) = sieve_toml_path.parent() {
   374	                fs::create_dir_all(parent)?;
   375	            }
   376	            let toml_content = build_default_sieve_toml(sieve_toml_path)?;
   377	            fs::write(sieve_toml_path, toml_content.as_bytes()).context("写入 sieve.toml 失败")?;
   378	            ctx.written_files.push(sieve_toml_path.to_path_buf());
   379	            println!("[setup] ✅ sieve.toml 写入 {}", sieve_toml_path.display());
   380	        }
   381	
   382	        // 写 launchd plist
   383	        {
   384	            if let Some(parent) = plist_path.parent() {
   385	                fs::create_dir_all(parent)?;
   386	            }
   387	            // 备份已有 plist
   388	            if plist_path.exists() {
   389	                let rel = plist_path.strip_prefix(home_path).unwrap_or(plist_path);
   390	                let backup_dest = backup_dir.join(rel);
   391	                if let Some(parent) = backup_dest.parent() {
   392	                    fs::create_dir_all(parent)?;
   393	                }
   394	                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
   395	            }
   396	            fs::write(plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
   397	            ctx.written_files.push(plist_path.to_path_buf());
   398	            println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());
   399	        }
   400	
   401	        // launchctl load
   402	        {
   403	            let status = Command::new("launchctl")
   404	                .args(["load", "-w", &plist_path.to_string_lossy()])
   405	                .status()
   406	                .context("执行 launchctl load 失败")?;
   407	            if !status.success() {
   408	                bail!("launchctl load 返回非零: {:?}", status.code());
   409	            }
   410	            ctx.launchd_loaded = Some(plist_path.to_path_buf());
   411	            println!("[setup] ✅ launchd 服务已加载");
   412	        }
   413	
   414	        // 写 setup.log（含 created_new 字段，供 uninstall 精确还原）
   415	        {
   416	            let entries: Vec<SetupLogEntry> = vec![
   417	                SetupLogEntry::new("setup_complete")
   418	                    .with_detail(format!("backup_dir={}", backup_dir.display())),
   419	                SetupLogEntry::new("settings_updated")
   420	                    .with_path(settings_path.to_string_lossy().to_string())
   421	                    .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse")
   422	                    .with_created_new(!settings_existed_before),
   423	                SetupLogEntry::new("sieve_toml_written")
   424	                    .with_path(sieve_toml_path.to_string_lossy().to_string())
   425	                    .with_created_new(!sieve_toml_existed_before),
   426	                SetupLogEntry::new("launchd_loaded")
   427	                    .with_path(plist_path.to_string_lossy().to_string()),
   428	            ];
   429	            let mut file = std::fs::OpenOptions::new()
   430	                .create(true)
   431	                .append(true)
   432	                .open(setup_log_path)
   433	                .context("打开 setup.log 失败")?;
   434	            for entry in &entries {
   435	                let line = serde_json::to_string(entry)? + "\n";
   436	                file.write_all(line.as_bytes())?;
   437	            }
   438	            println!("[setup] ✅ setup.log 写入 {}", setup_log_path.display());
   439	        }
   440	
   441	        Ok(())
   442	    }
   443	
   444	    /// 构建 launchd plist 内容（使用当前 sieve 二进制路径 + 绝对路径 --config）。
   445	    ///
   446	    /// plist 中 ProgramArguments 必须使用绝对路径，且 --config 指向绝对配置文件，
   447	    /// 否则 launchd 从根目录启动时找不到相对路径规则文件，daemon 会立即退出。
   448	    /// WorkingDirectory 兜底设置为 sieve_home（~/.sieve）。
   449	    pub(super) fn build_plist_content(sieve_toml_path: &Path) -> Result<String> {
   450	        let sieve_bin = std::env::current_exe().context("获取当前二进制路径失败")?;
   451	        let sieve_home =
   452	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
   453	        let log_path = sieve_home.join("daemon.log");
   454	        let err_path = sieve_home.join("daemon.err");
   455	        // config 路径必须是绝对路径
   456	        let config_abs = if sieve_toml_path.is_absolute() {
   457	            sieve_toml_path.to_path_buf()
   458	        } else {
   459	            std::env::current_dir()
   460	                .unwrap_or_default()
   461	                .join(sieve_toml_path)
   462	        };
   463	
   464	        Ok(format!(
   465	            r#"<?xml version="1.0" encoding="UTF-8"?>
   466	<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
   467	  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   468	<plist version="1.0">
   469	<dict>
   470	  <key>Label</key>
   471	  <string>com.sieve.daemon</string>
   472	  <key>ProgramArguments</key>
   473	  <array>
   474	    <string>{bin}</string>
   475	    <string>start</string>
   476	    <string>--config</string>
   477	    <string>{config}</string>
   478	  </array>
   479	  <key>WorkingDirectory</key>
   480	  <string>{work_dir}</string>
   481	  <key>RunAtLoad</key>
   482	  <true/>
   483	  <key>KeepAlive</key>
   484	  <true/>
   485	  <key>StandardOutPath</key>
   486	  <string>{log}</string>
   487	  <key>StandardErrorPath</key>
   488	  <string>{err}</string>
   489	</dict>
   490	</plist>
   491	"#,
   492	            bin = sieve_bin.display(),
   493	            config = config_abs.display(),
   494	            work_dir = sieve_home.display(),
   495	            log = log_path.display(),
   496	            err = err_path.display(),
   497	        ))
   498	    }
   499	
   500	    /// 构建默认 sieve.toml 内容（所有路径使用绝对路径）。
   501	    pub(super) fn build_default_sieve_toml(sieve_toml_path: &Path) -> Result<String> {
   502	        let sieve_home = sieve_toml_path
   503	            .parent()
   504	            .ok_or_else(|| anyhow!("sieve.toml 路径无父目录"))?;
   505	        let rules_dir = sieve_home.join("rules");
   506	        let audit_db = sieve_home.join("audit.db");
   507	
   508	        Ok(format!(
   509	            r#"# sieve.toml — 由 `sieve setup` 自动生成，所有路径为绝对路径
   510	# 修改后需重启 daemon：launchctl kickstart -k gui/$(id -u)/com.sieve.daemon
   511	
   512	[proxy]
   513	listen = "127.0.0.1:11453"
   514	upstream = "https://api.anthropic.com"
   515	
   516	[rules]
   517	# 规则文件目录（绝对路径，launchd 从 / 启动时不依赖 cwd）
   518	dir = "{rules_dir}"
   519	
   520	[audit]
     1	//! `sieve uninstall` 命令实现（ADR-015 / SPEC-003 §uninstall）。
     2	//!
     3	//! 步骤：
     4	//! 1. 读 `~/.sieve/setup.log` 反向遍历 entries（了解 backup_dir + created_new 标志）
     5	//! 2. dry-run 打印将恢复的内容
     6	//! 3. 非 --yes 等待用户确认
     7	//! 4. 按 setup.log 记录的 created_new 字段决定还原策略：
     8	//!    - `created_new = true`：setup 前不存在，直接删除（恢复"原状"）
     9	//!    - `created_new = false`：仅移除 Sieve entries（ANTHROPIC_BASE_URL + sieve-hook），
    10	//!      保留用户 setup 后添加的其他配置
    11	//! 5. `launchctl unload` 并删除 plist 文件
    12	//! 6. 提示用户手动删 `~/.sieve/`
    13	//!
    14	//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
    15	
    16	use crate::cli::UninstallArgs;
    17	use anyhow::Result;
    18	
    19	#[cfg(target_os = "macos")]
    20	pub use macos::run;
    21	
    22	#[cfg(not(target_os = "macos"))]
    23	pub use stub::run;
    24	
    25	// ──────────────────────────────── macOS 实现 ────────────────────────────────
    26	
    27	#[cfg(target_os = "macos")]
    28	mod macos {
    29	    use super::*;
    30	    use anyhow::{anyhow, Context};
    31	    use std::fs;
    32	    use std::io::{self, Write as IoWrite};
    33	    use std::path::PathBuf;
    34	    use std::process::Command;
    35	
    36	    /// setup.log entry 镜像（只读取需要的字段）。
    37	    #[derive(serde::Deserialize)]
    38	    struct SetupLogEntry {
    39	        action: String,
    40	        path: Option<String>,
    41	        detail: Option<String>,
    42	        #[serde(default)]
    43	        created_new: bool,
    44	    }
    45	
    46	    /// 记录 setup 写入文件的还原策略。
    47	    pub(super) struct FileRestoreInfo {
    48	        /// 文件绝对路径。
    49	        pub(super) path: PathBuf,
    50	        /// true → setup 前不存在，uninstall 时删除；false → 仅移除 Sieve entries。
    51	        pub(super) created_new: bool,
    52	    }
    53	
    54	    /// 运行 `sieve uninstall`。关联 ADR-015 / SPEC-003 §uninstall。
    55	    pub fn run(args: UninstallArgs) -> Result<()> {
    56	        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
    57	        let home_path = PathBuf::from(&home);
    58	        let sieve_home =
    59	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
    60	        let setup_log_path = sieve_home.join("setup.log");
    61	        let plist_path = home_path
    62	            .join("Library")
    63	            .join("LaunchAgents")
    64	            .join("com.sieve.daemon.plist");
    65	        let backups_root = sieve_home.join("backups");
    66	
    67	        // ── 1. 读取 setup.log，找到最新 backup_dir + 各文件 created_new 标志
    68	        let (latest_backup, file_restore_infos) = read_setup_log(&setup_log_path, &backups_root);
    69	
    70	        // ── 2. 打印将要恢复的内容
    71	        println!("=== sieve uninstall 预览 ===");
    72	        if !file_restore_infos.is_empty() {
    73	            for info in &file_restore_infos {
    74	                if info.created_new {
    75	                    println!("[restore] 删除（setup 新建）: {}", info.path.display());
    76	                } else {
    77	                    println!("[restore] 移除 Sieve entries: {}", info.path.display());
    78	                }
    79	            }
    80	        } else if let Some(ref bd) = latest_backup {
    81	            println!("[restore] 从备份目录恢复: {}", bd.display());
    82	            list_backup_files(bd);
    83	        } else {
    84	            println!("[restore] 未找到 setup.log 记录，将跳过文件恢复");
    85	        }
    86	        if plist_path.exists() {
    87	            println!("[launchd] launchctl unload {}", plist_path.display());
    88	            println!("[launchd] 删除 {}", plist_path.display());
    89	        }
    90	        println!("[提示] ~/.sieve/ 目录将保留（含审计日志），请手动删除：");
    91	        println!("       rm -rf {}", sieve_home.display());
    92	        println!("===========================");
    93	
    94	        if args.dry_run {
    95	            println!("[dry-run] 未做任何改动。");
    96	            return Ok(());
    97	        }
    98	
    99	        // ── 3. 等待用户确认
   100	        if !args.yes {
   101	            print!("继续执行以上操作？[y/N] ");
   102	            io::stdout().flush()?;
   103	            let mut input = String::new();
   104	            io::stdin().read_line(&mut input)?;
   105	            if !input.trim().eq_ignore_ascii_case("y") {
   106	                println!("已取消。");
   107	                return Ok(());
   108	            }
   109	        }
   110	
   111	        // ── 4. 按 created_new 标志决定还原策略
   112	        if !file_restore_infos.is_empty() {
   113	            restore_files(&file_restore_infos, &home_path, latest_backup.as_deref())?;
   114	        } else if let Some(ref bd) = latest_backup {
   115	            // 旧格式 setup.log（无 created_new），退回全量备份恢复
   116	            restore_from_backup(bd, &home_path)?;
   117	        }
   118	
   119	        // ── 5. 卸载 launchd
   120	        if plist_path.exists() {
   121	            let status = Command::new("launchctl")
   122	                .args(["unload", &plist_path.to_string_lossy()])
   123	                .status();
   124	            match status {
   125	                Ok(s) if s.success() => println!("[uninstall] ✅ launchd 服务已卸载"),
   126	                Ok(s) => eprintln!("[uninstall] ⚠ launchctl unload 返回: {:?}", s.code()),
   127	                Err(e) => eprintln!("[uninstall] ⚠ launchctl unload 失败: {e}"),
   128	            }
   129	            if let Err(e) = fs::remove_file(&plist_path) {
   130	                eprintln!("[uninstall] ⚠ 删除 plist 失败: {e}");
   131	            } else {
   132	                println!("[uninstall] ✅ plist 已删除");
   133	            }
   134	        }
   135	
   136	        // ── 6. 提示手动删除
   137	        println!();
   138	        println!("✅ 卸载完成。");
   139	        println!("提示：审计日志和备份文件保留在 {}", sieve_home.display());
   140	        println!("如需彻底清除，请手动运行：");
   141	        println!("  rm -rf {}", sieve_home.display());
   142	
   143	        Ok(())
   144	    }
   145	
   146	    /// 从 setup.log 读取最新 backup_dir 和文件还原信息。
   147	    ///
   148	    /// 返回 (latest_backup_dir, file_restore_infos)。
   149	    /// file_restore_infos 为空时表示 setup.log 是旧格式，退回全量备份恢复。
   150	    fn read_setup_log(
   151	        setup_log: &std::path::Path,
   152	        backups_root: &std::path::Path,
   153	    ) -> (Option<PathBuf>, Vec<FileRestoreInfo>) {
   154	        let Ok(raw) = fs::read_to_string(setup_log) else {
   155	            // setup.log 不存在，扫描 backups/ 最新目录兜底
   156	            return (find_latest_backup_dir(backups_root), vec![]);
   157	        };
   158	
   159	        let entries: Vec<SetupLogEntry> = raw
   160	            .lines()
   161	            .filter_map(|line| serde_json::from_str(line).ok())
   162	            .collect();
   163	
   164	        // 找最新 setup_complete entry 的 backup_dir
   165	        let latest_backup = entries
   166	            .iter()
   167	            .rev()
   168	            .find(|e| e.action == "setup_complete")
   169	            .and_then(|e| e.detail.as_deref())
   170	            .and_then(|d| d.strip_prefix("backup_dir="))
   171	            .map(PathBuf::from);
   172	
   173	        // 收集文件 action（settings_updated / sieve_toml_written），取最新一次 setup 的记录
   174	        // 策略：找最后一个 setup_complete 之后的所有文件 action
   175	        let last_setup_idx = entries
   176	            .iter()
   177	            .rposition(|e| e.action == "setup_complete")
   178	            .unwrap_or(0);
   179	
   180	        let file_actions = ["settings_updated", "sieve_toml_written"];
   181	        let infos: Vec<FileRestoreInfo> = entries[last_setup_idx..]
   182	            .iter()
   183	            .filter(|e| file_actions.contains(&e.action.as_str()))
   184	            .filter_map(|e| {
   185	                let path_str = e.path.as_deref()?;
   186	                Some(FileRestoreInfo {
   187	                    path: PathBuf::from(path_str),
   188	                    created_new: e.created_new,
   189	                })
   190	            })
   191	            .collect();
   192	
   193	        // 如果没有文件记录（旧格式 setup.log），返回空 infos 触发备份恢复兜底
   194	        let backup = latest_backup.or_else(|| find_latest_backup_dir(backups_root));
   195	        (backup, infos)
   196	    }
   197	
   198	    /// 扫描 backups/ 下最新目录（按名称字典序，RFC3339 时间戳排序正确）。
   199	    fn find_latest_backup_dir(backups_root: &std::path::Path) -> Option<PathBuf> {
   200	        if !backups_root.exists() {
   201	            return None;
   202	        }
   203	        let mut entries: Vec<PathBuf> = fs::read_dir(backups_root)
   204	            .ok()?
   205	            .filter_map(|e| e.ok().map(|e| e.path()))
   206	            .filter(|p| p.is_dir())
   207	            .collect();
   208	        entries.sort();
   209	        entries.into_iter().next_back()
   210	    }
   211	
   212	    /// 按 created_new 标志还原文件。
   213	    ///
   214	    /// - `created_new = true`：setup 前不存在，直接删除
   215	    /// - `created_new = false`：仅从文件内移除 Sieve entries（保留用户其他配置）
   216	    pub(super) fn restore_files(
   217	        infos: &[FileRestoreInfo],
   218	        _home_path: &std::path::Path,
   219	        backup_dir: Option<&std::path::Path>,
   220	    ) -> Result<()> {
   221	        for info in infos {
   222	            if !info.path.exists() {
   223	                println!("[uninstall] 跳过（文件不存在）: {}", info.path.display());
   224	                continue;
   225	            }
   226	
   227	            if info.created_new {
   228	                // setup 前不存在 → 删除整个文件
   229	                fs::remove_file(&info.path)
   230	                    .with_context(|| format!("删除 setup 新建文件 {} 失败", info.path.display()))?;
   231	                println!("[uninstall] ✅ 删除（setup 新建）: {}", info.path.display());
   232	            } else {
   233	                // setup 前已存在 → 仅移除 Sieve entries，保留用户其他配置
   234	                // 对 settings.json：移除 env.ANTHROPIC_BASE_URL + hooks.PreToolUse 中 sieve-hook 条目
   235	                let extension = info.path.extension().and_then(|e| e.to_str()).unwrap_or("");
   236	                if extension == "json" {
   237	                    match remove_sieve_entries_from_settings(&info.path) {
   238	                        Ok(()) => {
   239	                            println!("[uninstall] ✅ 移除 Sieve entries: {}", info.path.display());
   240	                        }
   241	                        Err(e) => {
   242	                            // 移除 entries 失败，退回备份恢复
   243	                            eprintln!("[uninstall] ⚠ 移除 entries 失败: {e}，尝试从备份恢复");
   244	                            if let Some(bd) = backup_dir {
   245	                                restore_file_from_backup(bd, &info.path)?;
   246	                            }
   247	                        }
   248	                    }
   249	                } else if extension == "toml" {
   250	                    // sieve.toml 属于 Sieve 自己，直接删除
   251	                    fs::remove_file(&info.path)
   252	                        .with_context(|| format!("删除 {} 失败", info.path.display()))?;
   253	                    println!("[uninstall] ✅ 删除: {}", info.path.display());
   254	                } else {
   255	                    // 其他文件：从备份恢复
   256	                    if let Some(bd) = backup_dir {
   257	                        restore_file_from_backup(bd, &info.path)?;
   258	                    }
   259	                }
   260	            }
   261	        }
   262	        Ok(())
   263	    }
   264	
   265	    /// 从 settings.json 中移除 Sieve 注入的 entries，保留用户其他配置。
   266	    ///
   267	    /// 移除：
   268	    /// - `env.ANTHROPIC_BASE_URL`（若值为 `http://127.0.0.1:11453`）
   269	    /// - `hooks.PreToolUse` 数组中包含 `sieve-hook` 的条目
   270	    pub(super) fn remove_sieve_entries_from_settings(
   271	        settings_path: &std::path::Path,
   272	    ) -> Result<()> {
   273	        let raw = fs::read_to_string(settings_path)
   274	            .with_context(|| format!("读取 {} 失败", settings_path.display()))?;
   275	        let mut v: serde_json::Value = serde_json::from_str(&raw)
   276	            .with_context(|| format!("解析 {} 失败", settings_path.display()))?;
   277	
   278	        // 移除 env.ANTHROPIC_BASE_URL（仅当值为 sieve url 时）
   279	        if let Some(env) = v.get_mut("env").and_then(|e| e.as_object_mut()) {
   280	            if env
   281	                .get("ANTHROPIC_BASE_URL")
   282	                .and_then(|u| u.as_str())
   283	                .map(|s| s == "http://127.0.0.1:11453")
   284	                .unwrap_or(false)
   285	            {
   286	                env.remove("ANTHROPIC_BASE_URL");
   287	                // 如果 env 对象变空，也一并移除（避免留下空对象）
   288	                if env.is_empty() {
   289	                    v.as_object_mut().map(|obj| obj.remove("env"));
   290	                }
   291	            }
   292	        }
   293	
   294	        // 移除 hooks.PreToolUse 中含 sieve-hook 的条目
   295	        if let Some(pre_tool) = v
   296	            .pointer_mut("/hooks/PreToolUse")
   297	            .and_then(|a| a.as_array_mut())
   298	        {
   299	            pre_tool.retain(|item| {
   300	                !item
   301	                    .pointer("/hooks/0/command")
   302	                    .and_then(|c| c.as_str())
   303	                    .map(|c| c.contains("sieve-hook"))
   304	                    .unwrap_or(false)
   305	            });
   306	        }
   307	        // 如果 hooks.PreToolUse 变空，移除该 key
   308	        let pre_tool_empty = v
   309	            .pointer("/hooks/PreToolUse")
   310	            .and_then(|a| a.as_array())
   311	            .map(|a| a.is_empty())
   312	            .unwrap_or(false);
   313	        if pre_tool_empty {
   314	            if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
   315	                hooks.remove("PreToolUse");
   316	                if hooks.is_empty() {
   317	                    v.as_object_mut().map(|obj| obj.remove("hooks"));
   318	                }
   319	            }
   320	        }

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '520,760p'; nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '320,520p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   520	[audit]
   521	# 审计日志数据库（绝对路径）
   522	db = "{audit_db}"
   523	"#,
   524	            rules_dir = rules_dir.display(),
   525	            audit_db = audit_db.display(),
   526	        ))
   527	    }
   528	
   529	    /// 简单去除 `// ...` 行注释（不处理字符串内的 `//`，够用于 settings.json）。
   530	    pub(super) fn strip_json_comments(s: &str) -> String {
   531	        s.lines()
   532	            .map(|line| {
   533	                // 找到不在引号内的 `//`
   534	                let mut in_string = false;
   535	                let mut escaped = false;
   536	                let mut comment_start = None;
   537	                let chars: Vec<char> = line.chars().collect();
   538	                let mut i = 0;
   539	                while i < chars.len() {
   540	                    if escaped {
   541	                        escaped = false;
   542	                    } else if chars[i] == '\\' && in_string {
   543	                        escaped = true;
   544	                    } else if chars[i] == '"' {
   545	                        in_string = !in_string;
   546	                    } else if !in_string
   547	                        && chars[i] == '/'
   548	                        && i + 1 < chars.len()
   549	                        && chars[i + 1] == '/'
   550	                    {
   551	                        comment_start = Some(i);
   552	                        break;
   553	                    }
   554	                    i += 1;
   555	                }
   556	                if let Some(pos) = comment_start {
   557	                    line[..pos].to_string()
   558	                } else {
   559	                    line.to_string()
   560	                }
   561	            })
   562	            .collect::<Vec<_>>()
   563	            .join("\n")
   564	    }
   565	}
   566	
   567	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   568	
   569	#[cfg(not(target_os = "macos"))]
   570	mod stub {
   571	    use super::*;
   572	
   573	    /// `sieve setup` 非 macOS 占位实现。
   574	    /// Phase 1 仅支持 macOS；Linux/Windows 在 Phase 2 规划（ADR-015）。
   575	    pub fn run(_args: SetupArgs) -> Result<()> {
   576	        anyhow::bail!(
   577	            "sieve setup is macOS only in Phase 1. \
   578	             Linux/Windows support is planned for Phase 2."
   579	        )
   580	    }
   581	}
   582	
   583	// ──────────────────────────────── 单元测试 ──────────────────────────────────
   584	
   585	#[cfg(test)]
   586	#[cfg(target_os = "macos")]
   587	mod tests {
   588	    use super::macos::{
   589	        build_default_sieve_toml, build_plist_content, strip_json_comments, SetupLogEntry,
   590	    };
   591	    use tempfile::tempdir;
   592	
   593	    // ── 测试 #1：plist 包含 --config <绝对路径>/sieve.toml ──────────────────
   594	    // 修复 #6 验证：launchd plist 必须含绝对路径 --config 和 WorkingDirectory
   595	
   596	    #[test]
   597	    fn plist_contains_absolute_config_flag() {
   598	        let dir = tempdir().unwrap();
   599	        let sieve_toml = dir.path().join("sieve.toml");
   600	        let plist = build_plist_content(&sieve_toml).unwrap();
   601	
   602	        assert!(
   603	            plist.contains("<string>--config</string>"),
   604	            "plist 必须包含 --config 参数: {plist}"
   605	        );
   606	        let config_str = sieve_toml.to_string_lossy();
   607	        assert!(
   608	            plist.contains(config_str.as_ref()),
   609	            "plist 必须包含 sieve.toml 绝对路径 {config_str}: {plist}"
   610	        );
   611	        assert!(
   612	            plist.contains("<key>WorkingDirectory</key>"),
   613	            "plist 必须包含 WorkingDirectory: {plist}"
   614	        );
   615	    }
   616	
   617	    // ── 测试 #2：解析失败的 JSON 返回 Err（不 fallback 到空对象）──────────────
   618	    // 修复 #8 核心：strip_json_comments + serde_json::from_str 失败路径
   619	
   620	    #[test]
   621	    fn bad_json_parse_returns_error_not_empty_object() {
   622	        // 尾逗号是无效 JSON，strip_json_comments 无法修复
   623	        let bad_json = r#"{"env": {"SOME_KEY": "value",},}"#;
   624	        let stripped = strip_json_comments(bad_json);
   625	        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);
   626	
   627	        // 修复前是 unwrap_or_else(|_| {}) 导致覆盖用户数据；修复后必须返回 Err
   628	        assert!(
   629	            result.is_err(),
   630	            "尾逗号 JSON 应解析失败，不得 fallback 到空对象"
   631	        );
   632	    }
   633	
   634	    // ── 测试 #3：SetupLogEntry 序列化 created_new 字段 ──────────────────────
   635	    // 修复 #9 数据基础：setup.log 正确记录 created_new=true/false
   636	
   637	    #[test]
   638	    fn setup_log_entry_created_new_serializes_correctly() {
   639	        let entry_new = SetupLogEntry::new("settings_updated")
   640	            .with_path("/tmp/test.json".to_string())
   641	            .with_created_new(true);
   642	        let json = serde_json::to_string(&entry_new).unwrap();
   643	        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
   644	        assert_eq!(
   645	            v.get("created_new").and_then(|c| c.as_bool()),
   646	            Some(true),
   647	            "新建文件 created_new 应序列化为 true: {json}"
   648	        );
   649	
   650	        let entry_existing = SetupLogEntry::new("settings_updated")
   651	            .with_path("/tmp/test.json".to_string())
   652	            .with_created_new(false);
   653	        let json2 = serde_json::to_string(&entry_existing).unwrap();
   654	        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
   655	        assert_eq!(
   656	            v2.get("created_new").and_then(|c| c.as_bool()),
   657	            Some(false),
   658	            "已有文件 created_new 应序列化为 false: {json2}"
   659	        );
   660	    }
   661	
   662	    // ── sieve.toml 使用绝对路径 ─────────────────────────────────────────────
   663	
   664	    #[test]
   665	    fn default_sieve_toml_has_absolute_paths() {
   666	        let dir = tempdir().unwrap();
   667	        let sieve_toml = dir.path().join("sieve.toml");
   668	        let content = build_default_sieve_toml(&sieve_toml).unwrap();
   669	
   670	        assert!(
   671	            content.contains(&format!("dir = \"{}", dir.path().display())),
   672	            "rules.dir 必须是绝对路径: {content}"
   673	        );
   674	        assert!(
   675	            content.contains(&format!("db = \"{}", dir.path().display())),
   676	            "audit.db 必须是绝对路径: {content}"
   677	        );
   678	    }
   679	}
   320	        }
   321	
   322	        let json_str = serde_json::to_string_pretty(&v)?;
   323	        fs::write(settings_path, json_str.as_bytes())
   324	            .with_context(|| format!("写入 {} 失败", settings_path.display()))?;
   325	        Ok(())
   326	    }
   327	
   328	    /// 从备份目录恢复单个文件。
   329	    fn restore_file_from_backup(
   330	        backup_dir: &std::path::Path,
   331	        target: &std::path::Path,
   332	    ) -> Result<()> {
   333	        // 计算 backup 中的对应路径（target 的绝对路径去掉 HOME 前缀）
   334	        let home = std::env::var("HOME").unwrap_or_default();
   335	        let rel = target.strip_prefix(&home).unwrap_or(target);
   336	        let backup_src = backup_dir.join(rel);
   337	        if backup_src.exists() {
   338	            if let Some(parent) = target.parent() {
   339	                fs::create_dir_all(parent)?;
   340	            }
   341	            fs::copy(&backup_src, target).with_context(|| {
   342	                format!(
   343	                    "从备份恢复 {} → {} 失败",
   344	                    backup_src.display(),
   345	                    target.display()
   346	                )
   347	            })?;
   348	            println!("[uninstall] ✅ 从备份恢复: {}", target.display());
   349	        } else {
   350	            eprintln!("[uninstall] ⚠ 备份文件不存在: {}", backup_src.display());
   351	        }
   352	        Ok(())
   353	    }
   354	
   355	    /// 打印备份目录中的文件列表。
   356	    fn list_backup_files(backup_dir: &std::path::Path) {
   357	        if let Ok(walker) = fs::read_dir(backup_dir) {
   358	            for entry in walker.flatten() {
   359	                println!("  - {}", entry.path().display());
   360	            }
   361	        }
   362	    }
   363	
   364	    /// 将备份目录中的文件逐一恢复到 home 下对应路径（旧格式 setup.log 兜底）。
   365	    fn restore_from_backup(
   366	        backup_dir: &std::path::Path,
   367	        home_path: &std::path::Path,
   368	    ) -> Result<()> {
   369	        restore_dir_recursive(backup_dir, backup_dir, home_path)
   370	    }
   371	
   372	    fn restore_dir_recursive(
   373	        root: &std::path::Path,
   374	        current: &std::path::Path,
   375	        home_path: &std::path::Path,
   376	    ) -> Result<()> {
   377	        for entry in fs::read_dir(current)
   378	            .with_context(|| format!("读取备份目录 {} 失败", current.display()))?
   379	        {
   380	            let entry = entry?;
   381	            let path = entry.path();
   382	            if path.is_dir() {
   383	                restore_dir_recursive(root, &path, home_path)?;
   384	            } else {
   385	                // 计算目标路径：backup_root/rel → home/rel
   386	                let rel = path.strip_prefix(root).unwrap_or(path.as_path());
   387	                let dest = home_path.join(rel);
   388	                if let Some(parent) = dest.parent() {
   389	                    fs::create_dir_all(parent)?;
   390	                }
   391	                fs::copy(&path, &dest).with_context(|| {
   392	                    format!("恢复 {} → {} 失败", path.display(), dest.display())
   393	                })?;
   394	                println!("[uninstall] ✅ 恢复 {}", dest.display());
   395	            }
   396	        }
   397	        Ok(())
   398	    }
   399	}
   400	
   401	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   402	
   403	#[cfg(not(target_os = "macos"))]
   404	mod stub {
   405	    use super::*;
   406	
   407	    /// `sieve uninstall` 非 macOS 占位实现。
   408	    pub fn run(_args: UninstallArgs) -> Result<()> {
   409	        anyhow::bail!(
   410	            "sieve uninstall is macOS only in Phase 1. \
   411	             Linux/Windows support is planned for Phase 2."
   412	        )
   413	    }
   414	}
   415	
   416	// ──────────────────────────────── 单元测试 ──────────────────────────────────
   417	
   418	#[cfg(test)]
   419	#[cfg(target_os = "macos")]
   420	mod tests {
   421	    use super::macos::{restore_files, FileRestoreInfo};
   422	    use std::fs;
   423	    use tempfile::tempdir;
   424	
   425	    // ── 测试 #4：uninstall 在 created_new=true entry 上删除整个文件 ─────────
   426	
   427	    #[test]
   428	    fn uninstall_created_new_true_deletes_file() {
   429	        let dir = tempdir().unwrap();
   430	        let settings = dir.path().join("settings.json");
   431	        fs::write(
   432	            &settings,
   433	            r#"{"env":{"ANTHROPIC_BASE_URL":"http://127.0.0.1:11453"}}"#,
   434	        )
   435	        .unwrap();
   436	
   437	        let infos = vec![FileRestoreInfo {
   438	            path: settings.clone(),
   439	            created_new: true,
   440	        }];
   441	
   442	        restore_files(&infos, dir.path(), None).unwrap();
   443	
   444	        assert!(
   445	            !settings.exists(),
   446	            "created_new=true 时 uninstall 应删除整个文件"
   447	        );
   448	    }
   449	
   450	    // ── 测试 #5：uninstall 在 created_new=false entry 上仅移除 Sieve entries ─
   451	
   452	    #[test]
   453	    fn uninstall_created_new_false_removes_sieve_entries_only() {
   454	        let dir = tempdir().unwrap();
   455	        let settings = dir.path().join("settings.json");
   456	
   457	        // 模拟 setup 后的 settings.json：包含 Sieve entries 和用户原有配置
   458	        let content = serde_json::json!({
   459	            "env": {
   460	                "ANTHROPIC_BASE_URL": "http://127.0.0.1:11453",
   461	                "USER_VAR": "user_value"
   462	            },
   463	            "hooks": {
   464	                "PreToolUse": [
   465	                    {
   466	                        "matcher": ".*",
   467	                        "hooks": [{"type": "command", "command": "sieve-hook check"}]
   468	                    },
   469	                    {
   470	                        "matcher": ".*",
   471	                        "hooks": [{"type": "command", "command": "user-hook"}]
   472	                    }
   473	                ]
   474	            },
   475	            "model": "claude-opus-4-5"
   476	        });
   477	        fs::write(&settings, serde_json::to_string_pretty(&content).unwrap()).unwrap();
   478	
   479	        let infos = vec![FileRestoreInfo {
   480	            path: settings.clone(),
   481	            created_new: false,
   482	        }];
   483	
   484	        restore_files(&infos, dir.path(), None).unwrap();
   485	
   486	        assert!(settings.exists(), "created_new=false 时文件应保留");
   487	
   488	        let result: serde_json::Value =
   489	            serde_json::from_str(&fs::read_to_string(&settings).unwrap()).unwrap();
   490	
   491	        // Sieve entries 应被移除
   492	        assert!(
   493	            result.pointer("/env/ANTHROPIC_BASE_URL").is_none(),
   494	            "ANTHROPIC_BASE_URL 应被移除"
   495	        );
   496	        // 用户原有字段应保留
   497	        assert_eq!(
   498	            result.pointer("/env/USER_VAR").and_then(|v| v.as_str()),
   499	            Some("user_value"),
   500	            "用户 env 变量应保留"
   501	        );
   502	        // 用户的其他 hook 应保留
   503	        let pre_tool = result
   504	            .pointer("/hooks/PreToolUse")
   505	            .and_then(|a| a.as_array())
   506	            .unwrap();
   507	        assert_eq!(pre_tool.len(), 1, "只应剩 1 个用户 hook");
   508	        assert!(
   509	            pre_tool[0]
   510	                .pointer("/hooks/0/command")
   511	                .and_then(|c| c.as_str())
   512	                .map(|c| c.contains("user-hook"))
   513	                .unwrap_or(false),
   514	            "用户 hook 应保留"
   515	        );
   516	        // model 等其他字段应保留
   517	        assert_eq!(
   518	            result.get("model").and_then(|v| v.as_str()),
   519	            Some("claude-opus-4-5"),
   520	            "model 字段应保留"

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '520,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   520	            "model 字段应保留"
   521	        );
   522	    }
   523	}

exec
/bin/zsh -lc 'git diff -- crates/sieve-cli/src/audit.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/pipeline/inbound.rs crates/sieve-core/src/detection.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/crates/sieve-cli/src/audit.rs b/crates/sieve-cli/src/audit.rs
index d557190..8e5e00a 100644
--- a/crates/sieve-cli/src/audit.rs
+++ b/crates/sieve-cli/src/audit.rs
@@ -1,38 +1,370 @@
-//! 审计日志（关联 data-model.md §审计 + ADR-007）。
+//! 审计日志（关联 data-model.md §审计 + ADR-007 + ADR-014）。
 //!
-//! Week 1：仅占位 schema 与目录初始化。Week 4 起接入实际事件写入。
+//! Week 5 起接入 SQLite append-only 存储。
 //!
-//! 设计约束（ADR-007）：
-//! - SQLite append-only；BEFORE UPDATE / DELETE 触发器拒绝修改（Week 4 建表时实施）。
-//! - 不引入 `rusqlite` 依赖，直到 Week 4 实际写入需求确立（避免早期锁定版本）。
-//!
-//! Week 4 接入时需补充的内容：
-//! - `rusqlite` / `sqlx` 依赖与建表 DDL；
-//! - `AuditEvent` 枚举（Request / Response / Block / Allow / Error）；
-//! - `AuditStore::append` 异步写入接口；
-//! - BEFORE UPDATE / DELETE 触发器 SQL。
+//! 设计约束（ADR-007 / ADR-014）：
+//! - SQLite append-only：BEFORE UPDATE / DELETE 触发器拒绝修改。
+//! - 异步写入接口：`tokio::task::spawn_blocking` + internal `Mutex` 串行化。
+//! - 不暴露 `rusqlite` 类型到 crate 外部。
 
-use anyhow::Result;
+use anyhow::{Context, Result};
+use chrono::Utc;
+use rusqlite::{params, Connection};
+use serde::{Deserialize, Serialize};
 use std::path::Path;
+use std::sync::{Arc, Mutex};
+
+// ─────────────────────────── AuditEvent ────────────────────────────────────
+
+/// 审计事件枚举（关联 PRD §5.4 处置矩阵 + ADR-014 双层防御日志需求）。
+// 方法在 daemon 完整接入前不被调用；Week 6 移除此 allow。
+#[allow(dead_code)]
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(tag = "kind", rename_all = "snake_case")]
+pub enum AuditEvent {
+    /// 出站请求中检测到敏感内容并脱敏。
+    OutboundRedacted {
+        rule_id: String,
+        severity: String,
+        request_id: String,
+        raw_json: Option<String>,
+    },
+    /// 入站响应 hook 标记了疑似高危工具调用。
+    InboundHookMarked {
+        rule_id: String,
+        severity: String,
+        request_id: String,
+        raw_json: Option<String>,
+    },
+    /// 入站高危工具调用等待用户决策。
+    InboundDecisionRequested {
+        rule_id: String,
+        severity: String,
+        request_id: String,
+        raw_json: Option<String>,
+    },
+    /// 用户对高危工具调用给出决策（Allow / Block）。
+    InboundDecisionResolved {
+        rule_id: String,
+        severity: String,
+        decision: String,
+        request_id: String,
+        raw_json: Option<String>,
+    },
+    /// 状态栏通知已发送。
+    StatusBarNotified {
+        rule_id: String,
+        severity: String,
+        request_id: String,
+        raw_json: Option<String>,
+    },
+}
+
+// impl 方法仅在 tests 和 append 中使用；Week 6 接入后移除此 allow。
+#[allow(dead_code)]
+impl AuditEvent {
+    fn direction(&self) -> &'static str {
+        match self {
+            Self::OutboundRedacted { .. } => "outbound",
+            Self::InboundHookMarked { .. }
+            | Self::InboundDecisionRequested { .. }
+            | Self::InboundDecisionResolved { .. }
+            | Self::StatusBarNotified { .. } => "inbound",
+        }
+    }
+
+    fn rule_id(&self) -> &str {
+        match self {
+            Self::OutboundRedacted { rule_id, .. }
+            | Self::InboundHookMarked { rule_id, .. }
+            | Self::InboundDecisionRequested { rule_id, .. }
+            | Self::InboundDecisionResolved { rule_id, .. }
+            | Self::StatusBarNotified { rule_id, .. } => rule_id,
+        }
+    }
+
+    fn severity(&self) -> &str {
+        match self {
+            Self::OutboundRedacted { severity, .. }
+            | Self::InboundHookMarked { severity, .. }
+            | Self::InboundDecisionRequested { severity, .. }
+            | Self::InboundDecisionResolved { severity, .. }
+            | Self::StatusBarNotified { severity, .. } => severity,
+        }
+    }
+
+    fn disposition(&self) -> &'static str {
+        match self {
+            Self::OutboundRedacted { .. } => "redact",
+            Self::InboundHookMarked { .. } => "mark",
+            Self::InboundDecisionRequested { .. } => "pending",
+            Self::InboundDecisionResolved { .. } => "resolved",
+            Self::StatusBarNotified { .. } => "notify",
+        }
+    }
+
+    fn decision(&self) -> Option<&str> {
+        if let Self::InboundDecisionResolved { decision, .. } = self {
+            Some(decision)
+        } else {
+            None
+        }
+    }
+
+    fn request_id(&self) -> &str {
+        match self {
+            Self::OutboundRedacted { request_id, .. }
+            | Self::InboundHookMarked { request_id, .. }
+            | Self::InboundDecisionRequested { request_id, .. }
+            | Self::InboundDecisionResolved { request_id, .. }
+            | Self::StatusBarNotified { request_id, .. } => request_id,
+        }
+    }
+
+    fn raw_json(&self) -> Option<&str> {
+        match self {
+            Self::OutboundRedacted { raw_json, .. }
+            | Self::InboundHookMarked { raw_json, .. }
+            | Self::InboundDecisionRequested { raw_json, .. }
+            | Self::InboundDecisionResolved { raw_json, .. }
+            | Self::StatusBarNotified { raw_json, .. } => raw_json.as_deref(),
+        }
+    }
+}
 
-/// 审计存储句柄（Week 1 占位）。
+// ─────────────────────────── AuditStore ────────────────────────────────────
+
+/// 审计存储句柄（SQLite append-only）。
 ///
-/// Week 4 起持有 SQLite 连接池；当前仅确保目录存在。
-pub struct AuditStore;
+/// Week 5 起持有真实 SQLite 连接；线程安全通过 `Arc<Mutex<Connection>>` 实现。
+/// 关联 ADR-014 双层防御日志需求。
+// Week 5：`conn` / `append` 在 daemon 完整接入前不被调用，加 allow 避免 dead_code lint。
+// Week 6 接入后移除这个属性。
+#[allow(dead_code)]
+pub struct AuditStore {
+    conn: Arc<Mutex<Connection>>,
+}
 
+// `append` 在 daemon 完整接入前不被 main.rs 调用；Week 6 移除此 allow。
+#[allow(dead_code)]
 impl AuditStore {
-    /// 初始化审计存储。
+    /// 初始化审计存储：打开 SQLite，创建表，安装 append-only 触发器。
     ///
-    /// Week 1：仅创建父目录（若不存在），不建表、不打开数据库文件。
-    /// Week 4：将在此处打开 / 迁移 SQLite，并建立 append-only 触发器。
+    /// 幂等——文件已存在时不重建表。
     ///
     /// # Errors
-    /// 目录创建失败时返回错误（Week 1 实际不可能失败，因 `create_dir_all` 忽略已存在）。
+    /// SQLite 打开或 DDL 执行失败时返回错误。
     pub fn init(path: &Path) -> Result<Self> {
         if let Some(parent) = path.parent() {
-            std::fs::create_dir_all(parent)?;
+            std::fs::create_dir_all(parent)
+                .with_context(|| format!("创建审计目录 {} 失败", parent.display()))?;
+        }
+
+        let conn = Connection::open(path)
+            .with_context(|| format!("打开审计数据库 {} 失败", path.display()))?;
+
+        // 建表
+        conn.execute_batch(CREATE_TABLE_DDL)
+            .context("创建 audit_events 表失败")?;
+
+        // 安装 append-only 触发器（幂等：IF NOT EXISTS 不会重建）
+        conn.execute_batch(APPEND_ONLY_TRIGGERS_DDL)
+            .context("安装 append-only 触发器失败")?;
+
+        tracing::debug!(path = %path.display(), "audit store initialized (SQLite)");
+        Ok(Self {
+            conn: Arc::new(Mutex::new(conn)),
+        })
+    }
+
+    /// 异步写入一条审计事件（spawn_blocking + Mutex 串行化）。
+    ///
+    /// # Errors
+    /// SQLite 写入失败时返回错误。
+    pub async fn append(&self, event: AuditEvent) -> Result<()> {
+        let conn = Arc::clone(&self.conn);
+        tokio::task::spawn_blocking(move || {
+            let guard = conn
+                .lock()
+                .map_err(|e| anyhow::anyhow!("audit mutex poisoned: {e}"))?;
+            let timestamp = Utc::now().to_rfc3339();
+            let raw_json = serde_json::to_string(&event).ok();
+            guard.execute(
+                INSERT_SQL,
+                params![
+                    timestamp,
+                    event.direction(),
+                    event.rule_id(),
+                    event.severity(),
+                    event.disposition(),
+                    event.decision(),
+                    event.request_id(),
+                    // 优先使用事件自带的 raw_json，否则用序列化整个事件
+                    event.raw_json().or(raw_json.as_deref()),
+                ],
+            )?;
+            Ok::<(), anyhow::Error>(())
+        })
+        .await
+        .context("spawn_blocking failed")??;
+        Ok(())
+    }
+}
+
+// ─────────────────────────── SQL 常量 ──────────────────────────────────────
+
+const CREATE_TABLE_DDL: &str = r#"
+CREATE TABLE IF NOT EXISTS audit_events (
+    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
+    timestamp_rfc3339   TEXT    NOT NULL,
+    direction           TEXT    NOT NULL,   -- 'outbound' | 'inbound'
+    rule_id             TEXT    NOT NULL,
+    severity            TEXT    NOT NULL,   -- 'Critical' | 'High' | 'Medium' | 'Low'
+    disposition         TEXT    NOT NULL,   -- 'redact' | 'mark' | 'pending' | 'resolved' | 'notify'
+    decision            TEXT,               -- 'Allow' | 'Block' | NULL
+    request_id          TEXT    NOT NULL,
+    raw_json            TEXT
+);
+"#;
+
+/// append-only 触发器：拒绝 UPDATE / DELETE（ADR-007 / ADR-014）。
+const APPEND_ONLY_TRIGGERS_DDL: &str = r#"
+CREATE TRIGGER IF NOT EXISTS no_update
+BEFORE UPDATE ON audit_events
+BEGIN
+    SELECT RAISE(FAIL, 'audit_events is append-only: UPDATE is forbidden');
+END;
+
+CREATE TRIGGER IF NOT EXISTS no_delete
+BEFORE DELETE ON audit_events
+BEGIN
+    SELECT RAISE(FAIL, 'audit_events is append-only: DELETE is forbidden');
+END;
+"#;
+
+// Week 6 接入后移除此 allow。
+#[allow(dead_code)]
+const INSERT_SQL: &str = r#"
+INSERT INTO audit_events
+    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
+VALUES
+    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
+"#;
+
+// ─────────────────────────── 单元测试 ───────────────────────────────────────
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use tempfile::tempdir;
+
+    fn make_event(n: u32) -> AuditEvent {
+        AuditEvent::OutboundRedacted {
+            rule_id: format!("OUT-0{n}"),
+            severity: "Critical".to_string(),
+            request_id: format!("req-{n}"),
+            raw_json: Some(format!("{{\"test\":{n}}}")),
+        }
+    }
+
+    fn make_decision_event() -> AuditEvent {
+        AuditEvent::InboundDecisionResolved {
+            rule_id: "IN-CR-01".to_string(),
+            severity: "Critical".to_string(),
+            decision: "Block".to_string(),
+            request_id: "req-decision".to_string(),
+            raw_json: None,
+        }
+    }
+
+    #[tokio::test]
+    async fn write_and_read_events() {
+        let dir = tempdir().unwrap();
+        let db_path = dir.path().join("audit.db");
+        let store = AuditStore::init(&db_path).expect("init failed");
+
+        for i in 1..=5 {
+            store.append(make_event(i)).await.expect("append failed");
+        }
+
+        // 直接用 rusqlite 验证
+        let conn = Connection::open(&db_path).unwrap();
+        let count: i64 = conn
+            .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
+            .unwrap();
+        assert_eq!(count, 5, "应有 5 条记录");
+
+        let rule_id: String = conn
+            .query_row("SELECT rule_id FROM audit_events WHERE id = 1", [], |r| {
+                r.get(0)
+            })
+            .unwrap();
+        assert_eq!(rule_id, "OUT-01");
+    }
+
+    #[tokio::test]
+    async fn decision_event_stores_decision_field() {
+        let dir = tempdir().unwrap();
+        let db_path = dir.path().join("audit_decision.db");
+        let store = AuditStore::init(&db_path).expect("init failed");
+
+        store.append(make_decision_event()).await.unwrap();
+
+        let conn = Connection::open(&db_path).unwrap();
+        let decision: Option<String> = conn
+            .query_row("SELECT decision FROM audit_events WHERE id = 1", [], |r| {
+                r.get(0)
+            })
+            .unwrap();
+        assert_eq!(decision.as_deref(), Some("Block"));
+    }
+
+    #[test]
+    fn update_trigger_blocks() {
+        let dir = tempdir().unwrap();
+        let db_path = dir.path().join("audit_trigger.db");
+        let store = AuditStore::init(&db_path).expect("init failed");
+
+        // 同步插一条记录
+        {
+            let guard = store.conn.lock().unwrap();
+            guard
+                .execute(
+                    INSERT_SQL,
+                    params![
+                        Utc::now().to_rfc3339(),
+                        "outbound",
+                        "OUT-01",
+                        "Critical",
+                        "redact",
+                        Option::<String>::None,
+                        "req-1",
+                        Option::<String>::None,
+                    ],
+                )
+                .unwrap();
+        }
+
+        // 尝试 UPDATE → 应该失败
+        {
+            let guard = store.conn.lock().unwrap();
+            let result = guard.execute(
+                "UPDATE audit_events SET rule_id = 'hacked' WHERE id = 1",
+                [],
+            );
+            assert!(result.is_err(), "UPDATE 应该被触发器拒绝");
+            let err_msg = result.unwrap_err().to_string();
+            assert!(
+                err_msg.contains("append-only"),
+                "错误信息应含 append-only，实际: {err_msg}"
+            );
+        }
+
+        // 尝试 DELETE → 应该失败
+        {
+            let guard = store.conn.lock().unwrap();
+            let result = guard.execute("DELETE FROM audit_events WHERE id = 1", []);
+            assert!(result.is_err(), "DELETE 应该被触发器拒绝");
         }
-        tracing::debug!(path = %path.display(), "audit store placeholder initialized");
-        Ok(Self)
     }
 }
diff --git a/crates/sieve-cli/src/daemon.rs b/crates/sieve-cli/src/daemon.rs
index aed965a..7e92d54 100644
--- a/crates/sieve-cli/src/daemon.rs
+++ b/crates/sieve-cli/src/daemon.rs
@@ -5,7 +5,13 @@
 //!
 //! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
 //!
-//! 关联 PRD §5.1 / §5.2 / ADR-002 / ADR-007 / ADR-008（426 状态码候选）。
+//! Week 4（v1.4）：
+//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
+//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
+//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
+//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
+//!
+//! 关联：PRD v1.4 §6.1 §6.7 / ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。
 
 use anyhow::{anyhow, Context, Result};
 use bytes::Bytes;
@@ -16,8 +22,10 @@
 use hyper::{Request, Response};
 use hyper_util::rt::{TokioExecutor, TokioIo};
 use hyper_util::server::conn::auto;
+use sieve_core::detection::Action;
 use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
 use sieve_core::pipeline::outbound::OutboundFilter;
+use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
 use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
 use sieve_core::sse::parser::SseParser;
 use sieve_core::tool_use_aggregator::Aggregator;
@@ -25,6 +33,7 @@
 use std::collections::HashSet;
 use std::sync::Arc;
 use tokio::net::TcpListener;
+use tokio::sync::mpsc;
 use tokio_stream::wrappers::ReceiverStream;
 
 use crate::config::Config;
@@ -38,6 +47,8 @@
 /// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
 /// `cfg.dry_run` 决定是否实际拦截。
 ///
+/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
+///
 /// # Errors
 /// bind 端口失败或 Forwarder 初始化失败时返回错误。
 pub async fn run(
@@ -51,6 +62,34 @@ pub async fn run(
     let forwarder =
         Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);
 
+    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
+    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
+    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
+    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
+        Ok(home) => {
+            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
+            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
+                Ok((server, listener)) => {
+                    let server = Arc::new(server);
+                    let srv_clone = Arc::clone(&server);
+                    tokio::spawn(async move {
+                        srv_clone.run(listener).await;
+                    });
+                    tracing::info!(socket = %socket_path.display(), "IPC server started");
+                    Some(server)
+                }
+                Err(e) => {
+                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
+                    None
+                }
+            }
+        }
+        Err(e) => {
+            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
+            None
+        }
+    };
+
     let listener = TcpListener::bind(listen)
         .await
         .with_context(|| format!("bind {}", listen))?;
@@ -75,6 +114,7 @@ pub async fn run(
         let filter = filter.clone();
         let inbound_engine = inbound_engine.clone();
         let inbound_sieveignore = inbound_sieveignore.clone();
+        let ipc_server = ipc_server.clone();
 
         tokio::spawn(async move {
             let io = TokioIo::new(stream);
@@ -84,7 +124,8 @@ pub async fn run(
                 // 每连接独立 InboundFilter（&mut self trait 要求）
                 let ib_filter =
                     InboundFilter::new(inbound_engine.clone(), inbound_sieveignore.clone());
-                async move { proxy(f, flt, ib_filter, dry_run, req).await }
+                let ipc = ipc_server.clone();
+                async move { proxy(f, flt, ib_filter, dry_run, ipc, req).await }
             });
 
             if let Err(e) = auto::Builder::new(TokioExecutor::new())
@@ -103,9 +144,10 @@ async fn proxy(
     filter: Arc<OutboundFilter>,
     inbound_filter: InboundFilter,
     dry_run: bool,
+    ipc: Option<Arc<sieve_ipc::IpcServer>>,
     req: Request<Incoming>,
 ) -> Result<Response<ResponseBody>, hyper::Error> {
-    match proxy_inner(forwarder, filter, inbound_filter, dry_run, req).await {
+    match proxy_inner(forwarder, filter, inbound_filter, dry_run, ipc, req).await {
         Ok(resp) => Ok(resp),
         Err(e) => {
             tracing::error!(error = %e, "proxy failed");
@@ -122,13 +164,14 @@ async fn proxy(
 
 /// 核心代理逻辑。
 ///
-/// - POST /v1/messages → collect body → 出站扫描 → 426 或入站 SSE tee 检测
+/// - POST /v1/messages → collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测
 /// - 其他路径 → 流式透传（Week 1 行为）
 async fn proxy_inner(
     forwarder: Arc<Forwarder>,
     filter: Arc<OutboundFilter>,
     inbound_filter: InboundFilter,
     dry_run: bool,
+    ipc: Option<Arc<sieve_ipc::IpcServer>>,
     req: Request<Incoming>,
 ) -> Result<Response<ResponseBody>> {
     let (parts, body) = req.into_parts();
@@ -155,7 +198,7 @@ async fn proxy_inner(
                 }
             };
 
-        // 3. 提取文本段 → 逐段扫描（OutboundFilter 通过 OutboundEngine::scan_text 调用）
+        // 3. 提取文本段 → 逐段扫描
         let texts = anthropic_req.extract_text_content();
         let mut all_detections: Vec<sieve_core::Detection> = Vec::new();
 
@@ -192,11 +235,34 @@ async fn proxy_inner(
         }
 
         // 4. 决策：
-        //    - fail-closed Critical 规则：无视 dry_run，永远 block（PRD §9 #3）
-        //    - 非 fail-closed Critical：dry_run=true 时仅 warn，dry_run=false 时 block
+        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
+        //    b. fail-closed Critical Block → 426（PRD §9 #3）
+        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
+        //    d. 其余 → 透传
+
+        // 4a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
+        //
+        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
+        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
+        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
+        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
+        let redact_hits: Vec<RedactHit> = all_detections
+            .iter()
+            .filter(|d| matches!(d.action, Action::Redact { .. }))
+            .map(|d| RedactHit {
+                rule_id: d.rule_id.clone(),
+                start: d.span.start,
+                end: d.span.end,
+            })
+            .collect();
+
+        // 4b/c. 收集需要 Block 的 detection
         let blocking: Vec<&sieve_core::Detection> = all_detections
             .iter()
             .filter(|d| {
+                if d.action != Action::Block {
+                    return false;
+                }
                 if d.severity != sieve_core::Severity::Critical {
                     return false;
                 }
@@ -214,6 +280,61 @@ async fn proxy_inner(
             return Ok(build_426_response(&cloned));
         }
 
+        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
+        //
+        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
+        // 1. redact_segments() 在文本字符串层替换
+        // 2. 把替换后的文本写回 AnthropicRequest messages
+        // 3. serde_json 重新序列化为新 body
+        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
+        if !redact_hits.is_empty() {
+            let seg_result = redact_segments(&texts, &redact_hits);
+            tracing::info!(
+                count = seg_result.redacted_count,
+                rules = %seg_result.redacted_summary,
+                "OUTBOUND AUTO-REDACT"
+            );
+
+            // 把替换后文本写回 AnthropicRequest，然后重新序列化
+            let new_body_bytes =
+                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
+                    .and_then(|req| {
+                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
+                    })?;
+
+            // 验证脱敏后 JSON 仍然合法（关键回归断言）
+            if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
+                return Err(anyhow!("redact_segments 产生了非法 JSON，fail-closed 拦截"));
+            }
+
+            let new_body = Bytes::from(new_body_bytes);
+            let new_len = new_body.len();
+
+            // 更新 Content-Length header
+            let mut new_parts = parts.clone();
+            new_parts.headers.insert(
+                http::header::CONTENT_LENGTH,
+                http::HeaderValue::from(new_len),
+            );
+
+            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
+            for (_, text) in &texts {
+                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
+                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
+                }
+            }
+
+            return forward_with_inbound_inspection(
+                forwarder,
+                inbound_filter,
+                dry_run,
+                ipc,
+                new_parts,
+                new_body,
+            )
+            .await;
+        }
+
         if dry_run && !all_detections.is_empty() {
             tracing::warn!(
                 count = all_detections.len(),
@@ -224,9 +345,7 @@ async fn proxy_inner(
             }
         }
 
-        // 5. prompt 地址 seed：把出站 prompt 中的 EVM 地址预先注入 InboundFilter 会话，
-        //    使首轮地址替换（prompt 地址 A → 响应地址 B）可被 IN-CR-01 检测。
-        //    关联 PRD §4.2 / P0-3 修复。
+        // 5. prompt 地址 seed
         for (_, text) in &texts {
             if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                 tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
@@ -238,6 +357,7 @@ async fn proxy_inner(
             forwarder,
             inbound_filter,
             dry_run,
+            ipc,
             parts,
             body_bytes,
         )
@@ -251,15 +371,21 @@ async fn proxy_inner(
 /// 透传并同步做入站 SSE 解析检测（tee 模式）。
 ///
 /// 字节流同时被：
-/// 1. 原样 forward 给客户端（via unbounded_channel）
+/// 1. 原样 forward 给客户端（via bounded channel）
 /// 2. 异步喂给 SseParser → Aggregator → InboundFilter 检测
 ///
-/// 如果检测到 fail-closed Critical（或非 dry_run Critical），向客户端注入
-/// `sieve_blocked` SSE event 后截断流（drop tx）。
+/// v1.4 分支逻辑：
+/// - `Action::Block`（fail-closed Critical）→ 注入 `sieve_blocked` event 并截流
+/// - `Action::HookMark` → 写 IPC pending 文件，SSE 流原样转发（**不注入 sieve_blocked**）
+/// - `Action::HoldForDecision` → hold 流 + keep-alive，等用户决策
+/// - 其余 → 透传
+///
+/// 关联：ADR-014 §双层防御、ADR-016 §dispatch 路由、PRD v1.4 §6.7。
 async fn forward_with_inbound_inspection(
     forwarder: Arc<Forwarder>,
     mut inbound_filter: InboundFilter,
     dry_run: bool,
+    ipc: Option<Arc<sieve_ipc::IpcServer>>,
     mut parts: http::request::Parts,
     body_bytes: Bytes,
 ) -> Result<Response<ResponseBody>> {
@@ -286,13 +412,11 @@ async fn forward_with_inbound_inspection(
 
     let (mut resp_parts, resp_body) = upstream_resp.into_parts();
 
-    // 入站响应可能被 sieve 注入 sieve_blocked event 截流,实际 body 长度不一定等于上游
-    // content-length。**剥掉 content-length 强制 chunked transfer**,否则 hyper client 会按
-    // content-length 截断,导致截流时注入的 sieve_blocked event 无法到达客户端。
+    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
+    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
     resp_parts.headers.remove(http::header::CONTENT_LENGTH);
 
-    // P0-5：bounded channel，深度 64，上游读取自然受背压限制（替代 unbounded_channel）。
-    // 发送端 .send().await → 接收端消费慢时阻塞 spawn 任务，避免内存无界增长。
+    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
     const INBOUND_CHANNEL_DEPTH: usize = 64;
     let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
         INBOUND_CHANNEL_DEPTH,
@@ -308,10 +432,9 @@ async fn forward_with_inbound_inspection(
         while let Some(frame_result) = stream.next().await {
             match frame_result {
                 Ok(frame) => {
-                    // 只对 data frame 做解析；trailer / 其他 frame 直接透传
                     let Some(frame_bytes) = frame.data_ref().cloned() else {
                         if tx.send(Ok(frame)).await.is_err() {
-                            return; // 接收端已 drop，停止读取
+                            return;
                         }
                         continue;
                     };
@@ -329,13 +452,21 @@ async fn forward_with_inbound_inspection(
                         }
                     };
 
-                    let blocking = collect_blocking_detections(
+                    // 收集本批 events 的 detections，按 action 分组处理
+                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                         &events,
                         &mut inbound_filter,
                         &mut aggregator,
                         dry_run,
                     );
 
+                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
+                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
+                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
+                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
+                    // 关联：ADR-014 §双层防御、PRD §9 #3。
+
+                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                     if !blocking.is_empty() {
                         tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                         for d in &blocking {
@@ -343,17 +474,133 @@ async fn forward_with_inbound_inspection(
                         }
                         let blocked_payload = build_sieve_blocked_sse(&blocking);
                         let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
-                        // drop tx → channel 关闭 → 客户端收到 EOF
                         return;
                     }
 
-                    // 无 blocking：透传原始 frame（字节级一致）
+                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
+                    for d in &hook_detections {
+                        write_hook_pending_silent(d);
+                    }
+
+                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
+                    if !hold_detections.is_empty() {
+                        if let Some(ref ipc_server) = ipc {
+                            // keep-alive channel：daemon 把心跳写入 SSE 流
+                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
+                            let tx_ka = tx.clone();
+
+                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
+                            // 然后再 hold——这样客户端已经看到触发 event，
+                            // hold 期间只收到 keep-alive comment。
+                            if tx
+                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
+                                .await
+                                .is_err()
+                            {
+                                return;
+                            }
+
+                            // 启动 keep-alive 转发 task
+                            let ka_fwd_handle = tokio::spawn(async move {
+                                while let Some(ka_bytes) = ka_rx.recv().await {
+                                    if tx_ka
+                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
+                                        .await
+                                        .is_err()
+                                    {
+                                        break;
+                                    }
+                                }
+                            });
+
+                            // 构造 IPC 请求
+                            use chrono::Utc;
+                            let request_id = uuid::Uuid::new_v4();
+                            let timeout_seconds = hold_detections
+                                .iter()
+                                .find_map(|d| {
+                                    if let Action::HoldForDecision {
+                                        timeout_seconds, ..
+                                    } = d.action
+                                    {
+                                        Some(timeout_seconds)
+                                    } else {
+                                        None
+                                    }
+                                })
+                                .unwrap_or(60);
+
+                            let ipc_detections = hold_detections
+                                .iter()
+                                .map(|d| sieve_ipc::protocol::DetectionPayload {
+                                    rule_id: d.rule_id.clone(),
+                                    severity: map_severity_to_ipc(d.severity),
+                                    disposition: sieve_ipc::Disposition::GuiPopup,
+                                    title: format!("检测命中：{}", d.rule_id),
+                                    one_line_summary: d.evidence_truncated.clone(),
+                                    details: serde_json::json!({}),
+                                })
+                                .collect();
+
+                            let ipc_req = sieve_ipc::DecisionRequest {
+                                request_id,
+                                created_at: Utc::now(),
+                                timeout_seconds,
+                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
+                                detections: ipc_detections,
+                            };
+
+                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
+                                Arc::clone(ipc_server),
+                                ipc_req,
+                                ka_tx,
+                            )
+                            .await;
+
+                            ka_fwd_handle.abort();
+
+                            match outcome {
+                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
+                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
+                                    // 允许：继续转发后续 SSE 帧
+                                    // 当前帧已在 hold 前发出，继续循环
+                                    continue;
+                                }
+                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
+                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
+                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
+                                    let _ = tx
+                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
+                                        .await;
+                                    return;
+                                }
+                                Err(e) => {
+                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
+                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
+                                    let _ = tx
+                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
+                                        .await;
+                                    return;
+                                }
+                            }
+                        } else {
+                            // IPC 未初始化：fail-closed，阻断
+                            tracing::warn!(
+                                "GuiPopup detection but IPC server not initialized; fail-closed"
+                            );
+                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
+                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
+                            return;
+                        }
+                    }
+
+                    // 无 blocking / hold：透传原始 frame
                     if tx
                         .send(Ok(hyper::body::Frame::data(frame_bytes)))
                         .await
                         .is_err()
                     {
-                        return; // 接收端已 drop
+                        return;
                     }
                 }
                 Err(e) => {
@@ -367,11 +614,15 @@ async fn forward_with_inbound_inspection(
             }
         }
 
-        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event，
-        // 同样走完整 blocking 决策——修复 P0-4 / PRD §9 #5 "提前断流"硬约束。
+        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
         let flushed = parser.flush();
-        let blocking =
-            collect_blocking_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
+        let (blocking, hook_detections, flush_hold_detections) =
+            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
+
+        for d in &hook_detections {
+            write_hook_pending_silent(d);
+        }
+
         if !blocking.is_empty() {
             tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
             for d in &blocking {
@@ -379,7 +630,23 @@ async fn forward_with_inbound_inspection(
             }
             let blocked_payload = build_sieve_blocked_sse(&blocking);
             let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
-            // drop tx → 客户端收到 EOF
+            return;
+        }
+
+        // 修 #5（flush 阶段 hold 丢失修复）：
+        // flush 路径的 HoldForDecision 命中不能静默丢弃。
+        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
+        // 关联：ADR-014 §双层防御、PRD §9 #3。
+        if !flush_hold_detections.is_empty() {
+            tracing::warn!(
+                count = flush_hold_detections.len(),
+                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
+            );
+            for d in &flush_hold_detections {
+                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
+            }
+            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
+            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
         }
     });
 
@@ -391,33 +658,34 @@ async fn forward_with_inbound_inspection(
     Ok(Response::from_parts(resp_parts, response_body))
 }
 
-/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，返回应触发 blocking 的 [`Detection`] 列表。
+/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
+/// - `blocking`：`Action::Block` 需立即截流的 detections
+/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
+/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
 ///
-/// 被 push_chunk 分支和 flush 分支共同调用，确保两条路径走完全相同的 blocking 决策逻辑，
-/// 修复 P0-4 / PRD §9 #5"提前断流"硬约束：flush 出来的残留 event 同样必须阻断 Critical。
-fn collect_blocking_detections(
+/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
+///
+/// 关联 ADR-016 §dispatch 路由、ADR-014 §双层防御。
+fn classify_inbound_detections(
     events: &[sieve_core::sse::parser::SseEvent],
     inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
     aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
     dry_run: bool,
-) -> Vec<sieve_core::Detection> {
-    let mut critical_hits: Vec<sieve_core::Detection> = Vec::new();
+) -> (
+    Vec<sieve_core::Detection>,
+    Vec<sieve_core::Detection>,
+    Vec<sieve_core::Detection>,
+) {
+    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();
 
     for evt in events {
         match inbound_filter.observe_event(evt) {
-            Ok(hits) => critical_hits.extend(
-                hits.into_iter()
-                    .filter(|d| d.severity == sieve_core::Severity::Critical),
-            ),
+            Ok(hits) => all_hits.extend(hits),
             Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
         }
-        // P0-5/P0-6：aggregator.process 返回 Result；容量超限或 malformed tool_use 时 fail-closed
         match aggregator.process(evt) {
             Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
-                Ok(hits) => critical_hits.extend(
-                    hits.into_iter()
-                        .filter(|d| d.severity == sieve_core::Severity::Critical),
-                ),
+                Ok(hits) => all_hits.extend(hits),
                 Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
             },
             Ok(None) => {}
@@ -425,25 +693,99 @@ fn collect_blocking_detections(
                 ref tool_id,
                 ref error,
             }) => {
-                // P0-6 fail-closed：畸形 tool_use JSON 不等价于"无风险"（PRD §9 #3）
-                // 攻击者可故意发畸形 JSON 绕过 IN-CR-05 签名工具检测
                 tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
-                critical_hits.push(build_malformed_tool_use_detection(tool_id));
+                all_hits.push(build_malformed_tool_use_detection(tool_id));
             }
             Err(e) => {
-                // 容量超限（TooManyOpenBlocks / PartialJsonTooLarge / TextBufferTooLarge）：
-                // fail-closed，注入 IN-CAP-02 检测
                 tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
-                critical_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
+                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
             }
         }
     }
 
-    // 决策：fail-closed Critical 永远阻断；非 fail-closed 遵 dry_run
-    critical_hits
-        .into_iter()
-        .filter(|d| sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
-        .collect()
+    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
+    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
+    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();
+
+    for d in all_hits {
+        match &d.action {
+            Action::Block => {
+                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
+                if d.severity == sieve_core::Severity::Critical
+                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
+                {
+                    blocking.push(d);
+                }
+                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
+            }
+            Action::HookMark => {
+                // Hook 类：写 pending 文件，SSE 流继续转发
+                hook_detections.push(d);
+            }
+            Action::HoldForDecision { .. } => {
+                // GUI 类：hold 流等决策
+                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
+                hold_detections.push(d);
+            }
+            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
+                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
+            }
+        }
+    }
+
+    (blocking, hook_detections, hold_detections)
+}
+
+/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
+///
+/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
+/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
+fn write_hook_pending_silent(d: &sieve_core::Detection) {
+    use chrono::Utc;
+
+    let sieve_home = match sieve_ipc::paths::sieve_home() {
+        Ok(h) => h,
+        Err(e) => {
+            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
+            return;
+        }
+    };
+
+    let request_id = uuid::Uuid::new_v4();
+    let ipc_req = sieve_ipc::DecisionRequest {
+        request_id,
+        created_at: Utc::now(),
+        timeout_seconds: 60,
+        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
+        detections: vec![sieve_ipc::protocol::DetectionPayload {
+            rule_id: d.rule_id.clone(),
+            severity: map_severity_to_ipc(d.severity),
+            disposition: sieve_ipc::Disposition::HookTerminal,
+            title: format!("检测命中：{}", d.rule_id),
+            one_line_summary: d.evidence_truncated.clone(),
+            details: serde_json::json!({}),
+        }],
+    };
+
+    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
+        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
+    } else {
+        tracing::info!(
+            rule = %d.rule_id,
+            request_id = %request_id,
+            "HookMark: pending file written, SSE stream continues"
+        );
+    }
+}
+
+/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
+fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
+    match s {
+        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
+        sieve_core::Severity::High => sieve_ipc::Severity::High,
+        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
+        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
+    }
 }
 
 /// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
@@ -537,9 +879,6 @@ async fn forward_streaming(
 }
 
 /// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
-///
-/// body 为 JSON，含命中规则 ID / fingerprint / 操作指引。
-/// 时间戳当前为 UNIX epoch 秒，Week 4 引入 chrono 后改为完整 RFC3339。
 fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
     let blocked_at = epoch_secs_string();
     let detections_json: Vec<serde_json::Value> = detections
@@ -610,8 +949,6 @@ fn empty_body() -> ResponseBody {
 }
 
 /// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
-///
-/// 畸形 partial_json 不对应具体文本 span，evidence_truncated 存 tool_id。
 fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
@@ -629,8 +966,6 @@ fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
 }
 
 /// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
-///
-/// 容量超限不对应具体文本 span，因此 span 设 [0, 0)，evidence_truncated 为空。
 fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
@@ -646,3 +981,127 @@ fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Dete
         fingerprint: fingerprint_key.into(),
     }
 }
+
+/// 把脱敏后的文本段列表写回 [`AnthropicRequest`] 并返回新 request。
+///
+/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
+/// `redacted_texts` 是 `redact_segments()` 返回的替换后文本列表（顺序对应）。
+///
+/// 实现逻辑：遍历 messages，对每个文本 content 按 segment 索引匹配并替换。
+///
+/// # Errors
+/// 如果 `redacted_texts` 长度与 `original_texts` 不一致，返回错误。
+///
+/// 关联：PRD v1.4 §6.1（AutoRedact 路径），修 #1（AutoRedact 偏移修复）。
+fn apply_redacted_texts_to_request(
+    req: &sieve_core::protocol::anthropic::AnthropicRequest,
+    original_texts: &[(usize, String)],
+    redacted_texts: &[String],
+) -> Result<sieve_core::protocol::anthropic::AnthropicRequest> {
+    if original_texts.len() != redacted_texts.len() {
+        return Err(anyhow!(
+            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
+            redacted_texts.len(),
+            original_texts.len()
+        ));
+    }
+
+    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
+    let mut seg_idx = 0usize;
+
+    let mut new_messages: Vec<sieve_core::protocol::anthropic::AnthropicMessage> = Vec::new();
+    for msg in &req.messages {
+        let new_content = match &msg.content {
+            serde_json::Value::String(_) => {
+                // String 类型：一个 segment
+                let replacement = redacted_texts
+                    .get(seg_idx)
+                    .cloned()
+                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
+                seg_idx += 1;
+                serde_json::Value::String(replacement)
+            }
+            serde_json::Value::Array(blocks) => {
+                let mut new_blocks = Vec::with_capacity(blocks.len());
+                for block in blocks {
+                    if let Some(block_obj) = block.as_object() {
+                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
+                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
+                        {
+                            let replacement =
+                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
+                                    block_obj
+                                        .get("text")
+                                        .and_then(|v| v.as_str())
+                                        .unwrap_or("")
+                                        .to_string()
+                                });
+                            seg_idx += 1;
+                            let mut new_obj = block_obj.clone();
+                            new_obj
+                                .insert("text".to_string(), serde_json::Value::String(replacement));
+                            new_blocks.push(serde_json::Value::Object(new_obj));
+                            continue;
+                        }
+                    }
+                    new_blocks.push(block.clone());
+                }
+                serde_json::Value::Array(new_blocks)
+            }
+            other => other.clone(),
+        };
+        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {
+            role: msg.role.clone(),
+            content: new_content,
+        });
+    }
+
+    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
+    let new_system = if let Some(system) = &req.system {
+        if system.as_str().is_some() {
+            let replacement = redacted_texts
+                .get(seg_idx)
+                .cloned()
+                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
+            seg_idx += 1;
+            Some(serde_json::Value::String(replacement))
+        } else if let Some(blocks) = system.as_array() {
+            let mut new_blocks = Vec::with_capacity(blocks.len());
+            for block in blocks {
+                if block.get("text").and_then(|v| v.as_str()).is_some() {
+                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
+                        block
+                            .get("text")
+                            .and_then(|v| v.as_str())
+                            .unwrap_or("")
+                            .to_string()
+                    });
+                    seg_idx += 1;
+                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
+                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
+                    new_blocks.push(serde_json::Value::Object(new_obj));
+                } else {
+                    new_blocks.push(block.clone());
+                }
+            }
+            Some(serde_json::Value::Array(new_blocks))
+        } else {
+            Some(system.clone())
+        }
+    } else {
+        None
+    };
+
+    let _ = seg_idx; // 消除 unused variable 警告
+
+    Ok(sieve_core::protocol::anthropic::AnthropicRequest {
+        model: req.model.clone(),
+        max_tokens: req.max_tokens,
+        messages: new_messages,
+        stream: req.stream,
+        system: new_system,
+        tools: req.tools.clone(),
+        tool_choice: req.tool_choice.clone(),
+        extra: req.extra.clone(),
+    })
+}
diff --git a/crates/sieve-cli/src/engine_adapter.rs b/crates/sieve-cli/src/engine_adapter.rs
index 374a174..8a6b5d6 100644
--- a/crates/sieve-cli/src/engine_adapter.rs
+++ b/crates/sieve-cli/src/engine_adapter.rs
@@ -50,13 +50,51 @@ fn map_severity(r: RulesSeverity) -> Severity {
     }
 }
 
-/// 把 `sieve_rules::Action` 映射为 `sieve_core::Action`。
+/// 根据 `RuleEntry.disposition` 和 `RulesAction` 映射为 `sieve_core::Action`。
 ///
-/// `Warn` → `WarnConfirm { countdown_secs: 5 }`：Week 4 实际接入弹窗，此处占位。
+/// v1.4 重构：优先按 `effective_disposition()` 路由，`RulesAction` 作为兜底。
+///
+/// | Disposition       | Action                                       |
+/// |-------------------|----------------------------------------------|
+/// | AutoRedact        | `Redact { placeholder }`                     |
+/// | GuiPopup          | `HoldForDecision { request_id, timeout_s }`  |
+/// | HookTerminal      | `HookMark`                                   |
+/// | StatusBar         | `MarkOnly`                                   |
+///
+/// `timeout_seconds` / `default_on_timeout` 取自 `RuleEntry`，不再硬编码 5。
+///
+/// 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
+fn map_action_by_disposition(
+    disposition: sieve_rules::manifest::Disposition,
+    _rule_action: RulesAction,
+    rule_id: &str,
+    timeout_seconds: u32,
+) -> Action {
+    use sieve_rules::manifest::Disposition;
+    match disposition {
+        Disposition::AutoRedact => Action::Redact {
+            placeholder: format!("[REDACTED:{rule_id}]"),
+        },
+        Disposition::GuiPopup => Action::HoldForDecision {
+            request_id: uuid::Uuid::new_v4(),
+            timeout_seconds,
+        },
+        Disposition::HookTerminal => Action::HookMark,
+        Disposition::StatusBar => Action::MarkOnly,
+    }
+}
+
+/// 旧接口：仅用 `RulesAction` 映射（兜底，无 disposition 信息时使用）。
+///
+/// `Warn` → `HookMark`（v1.4 后 Warn 一律走 HookTerminal 路径）。
+///
+/// 注：修 #2 后生产路径不再调用此函数（disposition 优先），
+/// 保留用于单元测试验证 Warn → HookMark 的语义不变。
+#[allow(dead_code)]
 fn map_action(r: RulesAction) -> Action {
     match r {
         RulesAction::Block => Action::Block,
-        RulesAction::Warn => Action::WarnConfirm { countdown_secs: 5 },
+        RulesAction::Warn => Action::HookMark,
         RulesAction::Mark => Action::MarkOnly,
         RulesAction::Allow => Action::SilentLog,
     }
@@ -127,11 +165,35 @@ fn scan_text(
                 .map(|r| map_severity(r.severity))
                 .unwrap_or(Severity::Critical);
 
-            // critical_lock 强制：fail-closed 规则 action 一律覆盖为 Block
-            let raw_action = rule.map(|r| r.action).unwrap_or(RulesAction::Block);
-            let enforced_action =
-                sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw_action);
-            let action = map_action(enforced_action);
+            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复，入站侧）。
+            //
+            // 规则显式写了 disposition 时直接路由；
+            // disposition=None 且 fail-closed 时才强制 Block。
+            // 这确保 IN-CR-02（hook_terminal）/ IN-CR-05（gui_popup）即使在 fail-closed
+            // 名单里也能走正确的 HookMark / HoldForDecision 路径（不被截成 Block）。
+            //
+            // 关联：ADR-016（二维处置矩阵）、ADR-014（双层防御）、PRD v1.4 §5.4。
+            let action = if let Some(r) = rule {
+                if let Some(disp) = r.disposition {
+                    // 显式 disposition：直接路由，不经过 enforce_action
+                    let timeout = r.timeout_seconds.unwrap_or(60);
+                    map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
+                } else {
+                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
+                    let enforced =
+                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
+                    if enforced == RulesAction::Block {
+                        Action::Block
+                    } else {
+                        let disp = r.effective_disposition();
+                        let timeout = r.timeout_seconds.unwrap_or(60);
+                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
+                    }
+                }
+            } else {
+                // 规则表中找不到：fail-closed Block
+                Action::Block
+            };
 
             let evidence_truncated = redact_evidence(matched_text);
             let fp = fingerprint(&hit.rule_id, matched_text);
@@ -207,7 +269,33 @@ fn scan_text(
             let severity = rule
                 .map(|r| map_severity(r.severity))
                 .unwrap_or(Severity::Critical);
-            let action = rule.map(|r| map_action(r.action)).unwrap_or(Action::Block);
+            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复）。
+            //
+            // 规则显式写了 disposition 时，**直接按 disposition 路由**——
+            // 这确保 OUT-01（auto_redact）即使在 fail-closed 名单里也走 Redact 而非 Block。
+            // 只有 disposition=None（旧规则 / 无显式配置）且 fail-closed 时，才走 Block。
+            //
+            // 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
+            let action = rule
+                .map(|r| {
+                    if let Some(disp) = r.disposition {
+                        // 显式 disposition：直接路由，不经过 enforce_action
+                        let timeout = r.timeout_seconds.unwrap_or(60);
+                        map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
+                    } else {
+                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
+                        let enforced =
+                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
+                        if enforced == RulesAction::Block {
+                            Action::Block
+                        } else {
+                            let disp = r.effective_disposition();
+                            let timeout = r.timeout_seconds.unwrap_or(60);
+                            map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
+                        }
+                    }
+                })
+                .unwrap_or(Action::Block);
             let evidence_truncated = redact_evidence(matched_text);
             let fp = fingerprint(&hit.rule_id, matched_text);
 
@@ -284,6 +372,9 @@ fn make_rule(
             keywords: vec![],
             allowlist_regexes: vec![],
             allowlist_stopwords: vec![],
+            disposition: None,
+            timeout_seconds: None,
+            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
         }
     }
 
@@ -327,9 +418,10 @@ fn scan_no_match_returns_empty() {
     }
 
     #[test]
-    fn map_action_warn_becomes_warn_confirm() {
+    fn map_action_warn_becomes_hook_mark() {
+        // v1.4：Warn 一律走 HookTerminal 路径（HookMark action）
         let a = map_action(RulesAction::Warn);
-        assert!(matches!(a, Action::WarnConfirm { countdown_secs: 5 }));
+        assert!(matches!(a, Action::HookMark));
     }
 
     #[test]
@@ -364,4 +456,97 @@ fn span_offset_applied() {
         assert_eq!(hits[0].span.start, 104); // 100 + 4
         assert_eq!(hits[0].span.end, 109); // 100 + 9
     }
+
+    // ── 修 #2 回归：disposition 优先于 enforce_action ──────────────────────────
+
+    /// disposition=auto_redact 即使 action=block（fail-closed 名单）也走 Redact 路径。
+    ///
+    /// 修 #2（路由短路修复）：OUT-01 等 AutoRedact 规则在 fail-closed 名单里，
+    /// 旧代码 enforce_action 会把 action 强制变 Block，跳过 disposition 路由。
+    /// 修复后：显式 disposition 优先，OUT-01 必须走 Action::Redact 而非 Action::Block。
+    #[test]
+    fn disposition_auto_redact_beats_enforce_action() {
+        let mut rule = make_rule(
+            "OUT-01", // 在 fail-closed 名单里
+            r"sk-ant",
+            RulesSeverity::Critical,
+            RulesAction::Block,
+        );
+        rule.disposition = Some(sieve_rules::manifest::Disposition::AutoRedact);
+
+        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
+        let adapter = OutboundAdapter::new(Arc::new(engine), vec![rule]);
+
+        let hits = adapter
+            .scan_text("my sk-ant-key here", ContentSource::OutboundUserText, 0)
+            .unwrap();
+        assert_eq!(hits.len(), 1);
+        assert_eq!(hits[0].rule_id, "OUT-01");
+        // 关键断言：应该是 Redact，不是 Block
+        assert!(
+            matches!(hits[0].action, Action::Redact { .. }),
+            "disposition=auto_redact 应走 Redact 路径，实际: {:?}",
+            hits[0].action
+        );
+    }
+
+    /// disposition=hook_terminal 即使在 fail-closed 名单里也走 HookMark 路径。
+    ///
+    /// 修 #2 回归：IN-CR-02 等 HookTerminal 规则不应被 enforce_action 截成 Block。
+    #[test]
+    fn disposition_hook_terminal_beats_enforce_action() {
+        let mut rule = make_rule(
+            "IN-CR-02", // 在 fail-closed 名单里
+            r"rm -rf",
+            RulesSeverity::Critical,
+            RulesAction::Block,
+        );
+        rule.disposition = Some(sieve_rules::manifest::Disposition::HookTerminal);
+
+        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
+        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);
+
+        let hits = adapter
+            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
+            .unwrap();
+        assert_eq!(hits.len(), 1);
+        assert_eq!(hits[0].rule_id, "IN-CR-02");
+        // 关键断言：应该是 HookMark，不是 Block
+        assert!(
+            matches!(hits[0].action, Action::HookMark),
+            "disposition=hook_terminal 应走 HookMark 路径，实际: {:?}",
+            hits[0].action
+        );
+    }
+
+    /// disposition=gui_popup 即使在 fail-closed 名单里也走 HoldForDecision 路径。
+    #[test]
+    fn disposition_gui_popup_beats_enforce_action() {
+        let mut rule = make_rule(
+            "IN-CR-05-EVM", // 在 fail-closed 名单里
+            r"eth_signTypedData",
+            RulesSeverity::Critical,
+            RulesAction::Block,
+        );
+        rule.disposition = Some(sieve_rules::manifest::Disposition::GuiPopup);
+        rule.timeout_seconds = Some(60);
+
+        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
+        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);
+
+        let hits = adapter
+            .scan_text(
+                "call eth_signTypedData method",
+                ContentSource::InboundAssistantText,
+                0,
+            )
+            .unwrap();
+        assert_eq!(hits.len(), 1);
+        // 关键断言：应该是 HoldForDecision，不是 Block
+        assert!(
+            matches!(hits[0].action, Action::HoldForDecision { .. }),
+            "disposition=gui_popup 应走 HoldForDecision 路径，实际: {:?}",
+            hits[0].action
+        );
+    }
 }
diff --git a/crates/sieve-core/src/detection.rs b/crates/sieve-core/src/detection.rs
index e7f030a..a67f19c 100644
--- a/crates/sieve-core/src/detection.rs
+++ b/crates/sieve-core/src/detection.rs
@@ -19,23 +19,35 @@ pub enum Severity {
     Critical,
 }
 
-/// 命中处置动作（关联 PRD §5.1 P0 表的"处置"）。
+/// 命中处置动作（关联 PRD v1.4 §5.4 / ADR-016 二维处置矩阵）。
+///
+/// v1.4 重构：按 `Disposition` 路由，废弃 `WarnConfirm`。
+/// - `HookMark`：Hook 类命中，写 IPC pending 文件，SSE 流原样转发（ADR-014 §Hook 路径）。
+/// - `HoldForDecision`：GUI 类命中，hold 住 SSE 流等待用户决策（ADR-014 §GUI 路径）。
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
 #[serde(tag = "type", rename_all = "snake_case")]
 pub enum Action {
-    /// 直接拦截（出站 Critical 默认动作）。
+    /// 直接拦截（极端场景 / 出站 Critical fail-closed）。
     Block,
-    /// 脱敏（替换为 placeholder），Week 4 起实现。
+    /// 自动脱敏：替换为 `[REDACTED:<rule_id>]` 占位符（AutoRedact disposition，OUT-01~05/12）。
     Redact {
         /// 替换用占位符文本。
         placeholder: String,
     },
-    /// 弹窗倒计时人工确认，Week 4 起实现。
-    WarnConfirm {
-        /// 倒计时秒数。
-        countdown_secs: u32,
+    /// Hook 类：写 IPC pending 文件，SSE 流原样转发（IN-CR-02~04、IN-GEN-01~03）。
+    ///
+    /// 关联 ADR-014 §Hook 路径、SPEC-001。
+    HookMark,
+    /// GUI 类：hold 住 SSE 流，通过 IpcServer 等待用户决策（IN-CR-01/05、IN-GEN-04）。
+    ///
+    /// 关联 ADR-014 §GUI 路径、SPEC-002。
+    HoldForDecision {
+        /// 请求唯一标识（UUIDv4），用于 IPC 匹配。
+        request_id: uuid::Uuid,
+        /// 等待超时秒数（来自 `RuleEntry.timeout_seconds`）。
+        timeout_seconds: u32,
     },
-    /// 仅审计，不影响流量。
+    /// 仅审计，不影响流量（StatusBar disposition）。
     MarkOnly,
     /// 静默记录（用于 dry_run / canary）。
     SilentLog,
diff --git a/crates/sieve-core/src/pipeline/inbound.rs b/crates/sieve-core/src/pipeline/inbound.rs
index e3a0bcb..809d64f 100644
--- a/crates/sieve-core/src/pipeline/inbound.rs
+++ b/crates/sieve-core/src/pipeline/inbound.rs
@@ -198,7 +198,7 @@ fn scan_text(
                     id: Uuid::new_v4(),
                     rule_id: "IN-GEN-01".into(),
                     severity: Severity::High,
-                    action: Action::WarnConfirm { countdown_secs: 10 },
+                    action: Action::HookMark,
                     source,
                     span: ContentSpan { start: 0, end: 15 },
                     evidence_truncated: "suspicious_high".into(),
diff --git a/crates/sieve-core/src/pipeline/mod.rs b/crates/sieve-core/src/pipeline/mod.rs
index 8c3dcce..853bbbf 100644
--- a/crates/sieve-core/src/pipeline/mod.rs
+++ b/crates/sieve-core/src/pipeline/mod.rs
@@ -1,13 +1,40 @@
-//! Pipeline 节点（架构图 ②⑦）：Week 2 起填充实现。
+//! Pipeline 节点（架构图 ②⑦）及 v1.4 统一 dispatch 入口。
+//!
+//! `dispatch` 根据 Detection 的 `action` 路由到：
+//! - `Redact` → [`outbound_redact`] 脱敏路径（AutoRedact disposition）
+//! - `HookMark` → [`inbound_hook`] 写 pending 文件（SSE 原样转发）
+//! - `HoldForDecision` → [`inbound_hold`] hold 流 + keep-alive（GuiPopup disposition）
+//! - `MarkOnly` / `SilentLog` → StatusBarOnly 透传
+//!
+//! `dispatch` 及 hold/hook 子模块仅在 `forwarder` feature 下编译（依赖 bytes + tokio async），
+//! 与 `cargo fuzz --no-default-features` 场景隔离。
+//!
+//! 关联：ADR-014（双层防御）、ADR-016（二维处置矩阵）、PRD v1.4 §6.1 §6.7。
 
 pub mod inbound;
 pub mod outbound;
+pub mod outbound_redact;
 pub mod streaming;
 
+// forwarder feature 下才编译 hold / hook（依赖 bytes + tokio async）
+#[cfg(feature = "forwarder")]
+pub mod inbound_hold;
+#[cfg(feature = "forwarder")]
+pub mod inbound_hook;
+
 use crate::detection::Detection;
 use crate::error::SieveCoreResult;
 use crate::protocol::unified_message::UnifiedMessage;
 
+pub use outbound_redact::{align_to_utf8_char_start, redact_body_bytes, RedactHit, RedactResult};
+
+#[cfg(feature = "forwarder")]
+pub use inbound_hold::{HoldError, HoldOutcome};
+#[cfg(feature = "forwarder")]
+pub use inbound_hook::HookError;
+
+// ── Pipeline Node trait ──────────────────────────────────────────────────────
+
 /// Pipeline 节点 trait。
 ///
 /// Week 2 起 process 返回命中列表；Week 3 起入站节点也返回 Vec<Detection>
@@ -24,3 +51,364 @@ pub trait PipelineNode: Send + Sync {
     /// 处理失败时返回对应 [`crate::error::SieveCoreError`]。
     fn process(&self, msg: &mut UnifiedMessage) -> SieveCoreResult<Vec<Detection>>;
 }
+
+// ── dispatch（仅 forwarder feature）─────────────────────────────────────────
+
+#[cfg(feature = "forwarder")]
+pub use dispatch_impl::{dispatch, Direction, DispatchResult, PipelineError};
+
+#[cfg(feature = "forwarder")]
+mod dispatch_impl {
+    use std::sync::Arc;
+
+    use bytes::Bytes;
+    use thiserror::Error;
+    use tokio::sync::mpsc;
+    use uuid::Uuid;
+
+    use crate::detection::{Action, Detection, Severity};
+    use crate::pipeline::inbound_hold::{self, HoldError, HoldOutcome};
+    use crate::pipeline::inbound_hook::HookError;
+    use crate::pipeline::outbound_redact::{self, RedactHit};
+
+    /// 流量方向。
+    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
+    pub enum Direction {
+        /// 出站（客户端 → Anthropic API）。
+        Outbound,
+        /// 入站（Anthropic API → 客户端）。
+        Inbound,
+    }
+
+    /// Pipeline dispatch 专用错误。
+    ///
+    /// 关联 `.cursorrules §3.2`：库 crate 用 `thiserror`，禁 `anyhow`。
+    #[derive(Debug, Error)]
+    pub enum PipelineError {
+        /// Hook 类 pending 文件写入失败。
+        #[error("hook error: {0}")]
+        Hook(#[from] HookError),
+        /// GUI 类 hold 失败（IPC 错误）。
+        #[error("hold error: {0}")]
+        Hold(#[from] HoldError),
+        /// IPC 服务未初始化（GuiPopup detection 但 ipc 参数为 None）。
+        #[error("IPC server not initialized for GuiPopup detection")]
+        IpcNotInitialized,
+        /// keep-alive channel 未提供（GuiPopup detection 但 keep_alive_tx 参数为 None）。
+        #[error("keep-alive channel not provided for GuiPopup detection")]
+        KeepAliveChannelMissing,
+    }
+
+    /// `dispatch` 的返回值，指示 daemon 下一步动作。
+    ///
+    /// 关联 ADR-016 二维处置矩阵 / ADR-014 双层防御路径。
+    #[derive(Debug)]
+    pub enum DispatchResult {
+        /// 透传原样 body / SSE 流（无任何命中，或 StatusBar 静默）。
+        Passthrough,
+        /// 改写 body bytes 后转发（出站 AutoRedact）。
+        RewriteBody(Bytes),
+        /// 用户允许（GUI 类 hold 后通过）→ daemon 继续转发剩余 SSE。
+        AllowAfterHold,
+        /// 用户拒绝（GUI 类 hold 后拒绝）→ daemon 截流注入 `sieve_blocked` event。
+        DenyWithBlock(String),
+        /// Hook 类已写 IPC pending 文件 → daemon 原样转发 SSE 流。
+        HookMarked,
+        /// StatusBar 静默通知（不打断流程）。
+        StatusBarOnly,
+    }
+
+    /// 根据 detection 的 `action` 决定下一步动作，这是 daemon `proxy_inner` 调用的统一入口。
+    ///
+    /// # 路由优先级（高 → 低）
+    /// `Block` > `HoldForDecision`（GuiPopup）> `HookMark`（HookTerminal）> `Redact`（AutoRedact）> `MarkOnly`
+    ///
+    /// 关联：ADR-016 §dispatch 路由、ADR-014 §双层防御。
+    pub async fn dispatch(
+        _direction: Direction,
+        detections: Vec<Detection>,
+        ipc: Option<Arc<sieve_ipc::IpcServer>>,
+        request_id: Uuid,
+        body_bytes: Option<&[u8]>,
+        keep_alive_tx: Option<mpsc::Sender<Bytes>>,
+    ) -> Result<DispatchResult, PipelineError> {
+        if detections.is_empty() {
+            return Ok(DispatchResult::Passthrough);
+        }
+
+        let mut has_block = false;
+        let mut hold_detections: Vec<&Detection> = Vec::new();
+        let mut hook_detections: Vec<&Detection> = Vec::new();
+        let mut redact_hits: Vec<RedactHit> = Vec::new();
+        let mut all_status_only = true;
+
+        for d in &detections {
+            match &d.action {
+                Action::Block => {
+                    has_block = true;
+                    all_status_only = false;
+                }
+                Action::HoldForDecision { .. } => {
+                    hold_detections.push(d);
+                    all_status_only = false;
+                }
+                Action::HookMark => {
+                    hook_detections.push(d);
+                    all_status_only = false;
+                }
+                Action::Redact { .. } => {
+                    redact_hits.push(RedactHit {
+                        rule_id: d.rule_id.clone(),
+                        start: d.span.start,
+                        end: d.span.end,
+                    });
+                    all_status_only = false;
+                }
+                Action::MarkOnly | Action::SilentLog => {
+                    // 静默 / 状态栏，不改变 all_status_only 的含义
+                }
+            }
+        }
+
+        if all_status_only {
+            return Ok(DispatchResult::StatusBarOnly);
+        }
+
+        // Block 优先
+        if has_block {
+            let reason = detections
+                .iter()
+                .filter(|d| d.action == Action::Block)
+                .map(|d| d.rule_id.as_str())
+                .collect::<Vec<_>>()
+                .join(", ");
+            return Ok(DispatchResult::DenyWithBlock(format!(
+                "block（rules: {reason}）"
+            )));
+        }
+
+        // GuiPopup：hold 流等待用户决策
+        if !hold_detections.is_empty() {
+            let ipc = ipc.ok_or(PipelineError::IpcNotInitialized)?;
+            let ka_tx = keep_alive_tx.ok_or(PipelineError::KeepAliveChannelMissing)?;
+
+            let (hold_request_id, timeout_seconds) = hold_detections
+                .iter()
+                .find_map(|d| {
+                    if let Action::HoldForDecision {
+                        request_id,
+                        timeout_seconds,
+                    } = d.action
+                    {
+                        Some((request_id, timeout_seconds))
+                    } else {
+                        None
+                    }
+                })
+                .unwrap_or((request_id, 60));
+
+            use chrono::Utc;
+            use sieve_ipc::protocol::DetectionPayload;
+
+            let ipc_detections: Vec<DetectionPayload> = hold_detections
+                .iter()
+                .map(|d| DetectionPayload {
+                    rule_id: d.rule_id.clone(),
+                    severity: map_severity_to_ipc(d.severity),
+                    disposition: sieve_ipc::Disposition::GuiPopup,
+                    title: format!("检测命中：{}", d.rule_id),
+                    one_line_summary: d.evidence_truncated.clone(),
+                    details: serde_json::json!({}),
+                })
+                .collect();
+
+            let ipc_req = sieve_ipc::DecisionRequest {
+                request_id: hold_request_id,
+                created_at: Utc::now(),
+                timeout_seconds,
+                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
+                detections: ipc_detections,
+            };
+
+            let outcome = inbound_hold::hold_and_decide(ipc, ipc_req, ka_tx).await?;
+            return match outcome {
+                HoldOutcome::Allow | HoldOutcome::RedactAndAllow => {
+                    Ok(DispatchResult::AllowAfterHold)
+                }
+                HoldOutcome::Deny { reason } => Ok(DispatchResult::DenyWithBlock(reason)),
+            };
+        }
+
+        // HookTerminal：写 pending 文件，SSE 原样转发
+        if !hook_detections.is_empty() {
+            use chrono::Utc;
+            use sieve_ipc::protocol::DetectionPayload;
+
+            let sieve_home = sieve_ipc::paths::sieve_home()
+                .map_err(|e| PipelineError::Hook(HookError::Ipc(e)))?;
+
+            let ipc_detections: Vec<DetectionPayload> = hook_detections
+                .iter()
+                .map(|d| DetectionPayload {
+                    rule_id: d.rule_id.clone(),
+                    severity: map_severity_to_ipc(d.severity),
+                    disposition: sieve_ipc::Disposition::HookTerminal,
+                    title: format!("检测命中：{}", d.rule_id),
+                    one_line_summary: d.evidence_truncated.clone(),
+                    details: serde_json::json!({}),
+                })
+                .collect();
+
+            let ipc_req = sieve_ipc::DecisionRequest {
+                request_id,
+                created_at: Utc::now(),
+                timeout_seconds: 60,
+                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
+                detections: ipc_detections,
+            };
+
+            sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home)
+                .map_err(|e| PipelineError::Hook(HookError::Ipc(e)))?;
+
+            return Ok(DispatchResult::HookMarked);
+        }
+
+        // AutoRedact：脱敏 body bytes
+        if !redact_hits.is_empty() {
+            let body = body_bytes.unwrap_or(&[]);
+            let result = outbound_redact::redact_body_bytes(body, &redact_hits);
+            return Ok(DispatchResult::RewriteBody(Bytes::from(result.body)));
+        }
+
+        Ok(DispatchResult::Passthrough)
+    }
+
+    /// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
+    fn map_severity_to_ipc(s: Severity) -> sieve_ipc::Severity {
+        match s {
+            Severity::Critical => sieve_ipc::Severity::Critical,
+            Severity::High => sieve_ipc::Severity::High,
+            Severity::Medium => sieve_ipc::Severity::Medium,
+            Severity::Low => sieve_ipc::Severity::Low,
+        }
+    }
+
+    #[cfg(test)]
+    mod tests {
+        use super::*;
+        use crate::detection::{Action, ContentSource, Detection, Severity};
+        use crate::protocol::unified_message::ContentSpan;
+
+        fn make_detection(rule_id: &str, action: Action) -> Detection {
+            Detection {
+                id: Uuid::new_v4(),
+                rule_id: rule_id.to_string(),
+                severity: Severity::Critical,
+                action,
+                source: ContentSource::InboundAssistantText,
+                span: ContentSpan { start: 0, end: 5 },
+                evidence_truncated: "sk-an".to_string(),
+                fingerprint: "abc123".to_string(),
+            }
+        }
+
+        // ── 1. 空 detections → Passthrough ───────────────────────────────────
+
+        #[tokio::test]
+        async fn dispatch_empty_returns_passthrough() {
+            let result = dispatch(Direction::Inbound, vec![], None, Uuid::new_v4(), None, None)
+                .await
+                .unwrap();
+            assert!(matches!(result, DispatchResult::Passthrough));
+        }
+
+        // ── 2. MarkOnly → StatusBarOnly ───────────────────────────────────────
+
+        #[tokio::test]
+        async fn dispatch_mark_only_returns_status_bar() {
+            let detections = vec![make_detection("OUT-11", Action::MarkOnly)];
+            let result = dispatch(
+                Direction::Outbound,
+                detections,
+                None,
+                Uuid::new_v4(),
+                None,
+                None,
+            )
+            .await
+            .unwrap();
+            assert!(matches!(result, DispatchResult::StatusBarOnly));
+        }
+
+        // ── 3. Block → DenyWithBlock ──────────────────────────────────────────
+
+        #[tokio::test]
+        async fn dispatch_block_returns_deny() {
+            let detections = vec![make_detection("IN-CR-99", Action::Block)];
+            let result = dispatch(
+                Direction::Inbound,
+                detections,
+                None,
+                Uuid::new_v4(),
+                None,
+                None,
+            )
+            .await
+            .unwrap();
+            assert!(matches!(result, DispatchResult::DenyWithBlock(_)));
+        }
+
+        // ── 4. Redact → RewriteBody ───────────────────────────────────────────
+
+        #[tokio::test]
+        async fn dispatch_redact_returns_rewrite_body() {
+            let mut d = make_detection(
+                "OUT-01",
+                Action::Redact {
+                    placeholder: "[REDACTED]".to_string(),
+                },
+            );
+            d.span = ContentSpan { start: 0, end: 5 };
+
+            let body = b"sk-antXXXXX rest of body";
+            let result = dispatch(
+                Direction::Outbound,
+                vec![d],
+                None,
+                Uuid::new_v4(),
+                Some(body),
+                None,
+            )
+            .await
+            .unwrap();
+            assert!(matches!(result, DispatchResult::RewriteBody(_)));
+            if let DispatchResult::RewriteBody(new_body) = result {
+                let s = String::from_utf8(new_body.to_vec()).unwrap();
+                assert!(s.contains("[REDACTED:OUT-01]"), "body: {s}");
+            }
+        }
+
+        // ── 5. HookMark → HookMarked ──────────────────────────────────────────
+
+        #[tokio::test]
+        async fn dispatch_hook_mark_returns_hook_marked() {
+            let tmp = tempfile::tempdir().unwrap();
+            std::env::set_var("SIEVE_HOME", tmp.path().to_str().unwrap());
+
+            let detections = vec![make_detection("IN-CR-02", Action::HookMark)];
+            let result = dispatch(
+                Direction::Inbound,
+                detections,
+                None,
+                Uuid::new_v4(),
+                None,
+                None,
+            )
+            .await
+            .unwrap();
+            assert!(matches!(result, DispatchResult::HookMarked));
+
+            std::env::remove_var("SIEVE_HOME");
+        }
+    }
+}

exec
/bin/zsh -lc 'cat Cargo.toml; cat crates/sieve-cli/Cargo.toml; cat crates/sieve-core/Cargo.toml; cat crates/sieve-ipc/Cargo.toml; cat crates/sieve-hook/Cargo.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
[workspace]
resolver = "2"
members = [
    "crates/sieve-core",
    "crates/sieve-rules",
    "crates/sieve-cli",
    "crates/sieve-ipc",
    "crates/sieve-hook",
    "fuzz",
]
exclude = [
    "fuzz_afl",  # AFL crate 与 cargo fuzz 编译器要求不同，workspace 隔离
]

[workspace.package]
version = "0.1.0-alpha"
edition = "2021"
rust-version = "1.87"
license = "MIT"
repository = "https://github.com/<owner>/sieve"
authors = ["doskey <doskey.lee@gmail.com>"]

[workspace.dependencies]
# 用于跨 crate 共享版本(各 crate Cargo.toml 用 .workspace = true 引用)
tokio = { version = "1", default-features = false }
hyper = { version = "1", default-features = false }
hyper-util = { version = "0.1", default-features = false }
hyper-rustls = { version = "0.27", default-features = false }
rustls = { version = "0.23", default-features = false }
webpki-roots = "0.26"
http = "1"
http-body = "1"
http-body-util = "0.1"
bytes = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", default-features = false, features = ["serde", "clock"] }
fd-lock = "4"
clap = { version = "4", features = ["derive"] }

[profile.release]
codegen-units = 1
lto = "fat"
strip = "symbols"
panic = "abort"
debug = false
opt-level = 3

[profile.release-with-debug]
inherits = "release"
debug = true
strip = "none"
[package]
name = "sieve-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve CLI daemon: hyper proxy server, config, audit log"
publish = false

[[bin]]
name = "sieve"
path = "src/main.rs"

[dependencies]
sieve-core = { path = "../sieve-core" }
sieve-rules = { path = "../sieve-rules" }
sieve-ipc = { path = "../sieve-ipc" }
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { workspace = true }

tokio = { workspace = true, features = ["full"] }
hyper = { workspace = true, features = ["http1", "http2", "server"] }
hyper-util = { workspace = true, features = ["tokio", "server-auto", "server-graceful", "service"] }
http = { workspace = true }
http-body = { workspace = true }
http-body-util = { workspace = true }
bytes = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }

clap = { version = "4", features = ["derive"] }
anyhow = "1"
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
uuid = { version = "1", features = ["v4"] }
serde_json = { workspace = true }
tokio-stream = { version = "0.1", features = ["sync"] }
futures-util = "0.3"

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }
hyper = { workspace = true, features = ["http1", "server", "client"] }
hyper-util = { workspace = true, features = ["tokio", "client-legacy", "http1", "service", "server"] }
http-body-util = { workspace = true }
http = { workspace = true }
bytes = { workspace = true }
sieve-core = { path = "../sieve-core" }
anyhow = "1"
tempfile = "3"
serde_json = { workspace = true }
rusqlite = { version = "0.31", features = ["bundled"] }
[package]
name = "sieve-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve core: protocol, forwarder, SSE pipeline (Anthropic-only Phase 1)"
publish = false

[features]
default = ["forwarder"]
# forwarder：启用 hyper/rustls/tokio 网络栈与 aws-lc-rs C 依赖。
# 关闭后 sieve-core 仅保留纯 Rust 模块（sse/aggregator/protocol/pipeline 等），
# 用于 cargo fuzz 等需要 sanitizer instrumentation 的场景，避免 ASan 链接 sancov 符号失败。
#
# v1.4 注：dispatch / inbound_hold / inbound_hook 依赖 bytes + tokio(async)，
# 这些模块通过 #[cfg(feature = "forwarder")] 与 fuzz no-feature 场景隔离。
forwarder = [
    "dep:tokio",
    "dep:hyper",
    "dep:hyper-util",
    "dep:hyper-rustls",
    "dep:rustls",
    "dep:webpki-roots",
    "dep:http",
    "dep:http-body",
    "dep:http-body-util",
    "dep:bytes",
]

[dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "net", "io-util", "macros", "sync", "time"], optional = true }
hyper = { workspace = true, features = ["http1", "http2", "client", "server"], optional = true }
hyper-util = { workspace = true, features = ["tokio", "client-legacy", "http1", "http2"], optional = true }
hyper-rustls = { workspace = true, features = ["aws-lc-rs", "http1", "http2", "webpki-tokio", "tls12"], optional = true }
rustls = { workspace = true, features = ["aws-lc-rs", "tls12"], optional = true }
webpki-roots = { workspace = true, optional = true }
http = { workspace = true, optional = true }
http-body = { workspace = true, optional = true }
http-body-util = { workspace = true, optional = true }
bytes = { workspace = true, optional = true }
# v1.4：sieve-ipc 用于 pipeline dispatch / hook / hold 路径（forwarder feature 下）
sieve-ipc = { path = "../sieve-ipc" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
uuid = { version = "1", features = ["v4", "serde"] }
sha2 = "0.10"
strsim = "0.11"
chrono = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }
tempfile = "3"
[package]
name = "sieve-ipc"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve IPC: JSON-RPC 2.0 over Unix socket + pending/decision file protocol (ADR-013)"
publish = false

[dependencies]
tokio = { workspace = true, features = ["net", "fs", "sync", "time", "rt-multi-thread", "macros", "io-util"] }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
fd-lock = { workspace = true }

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["full", "test-util"] }
[package]
name = "sieve-hook"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve PreToolUse hook: TTY confirmation for Critical detections (macOS, Phase 1)"
publish = false

# Phase 1 macOS only。
[target.'cfg(target_os = "macos")'.dependencies]

[[bin]]
name = "sieve-hook"
path = "src/main.rs"

[lib]
name = "sieve_hook_lib"
path = "src/lib.rs"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
fd-lock = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
clap = { workspace = true }

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"

[dev-dependencies.criterion]
version = "0.5"
features = ["html_reports"]

[[bench]]
name = "hook_startup"
harness = false

exec
/bin/zsh -lc "grep -R \"new_v4\\|new_v7\" -n crates/sieve-ipc crates/sieve-hook crates/sieve-core/src crates/sieve-cli/src | head -100" in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-core/src/pipeline/mod.rs:304:                id: Uuid::new_v4(),
crates/sieve-core/src/pipeline/mod.rs:319:            let result = dispatch(Direction::Inbound, vec![], None, Uuid::new_v4(), None, None)
crates/sieve-core/src/pipeline/mod.rs:334:                Uuid::new_v4(),
crates/sieve-core/src/pipeline/mod.rs:352:                Uuid::new_v4(),
crates/sieve-core/src/pipeline/mod.rs:378:                Uuid::new_v4(),
crates/sieve-core/src/pipeline/mod.rs:403:                Uuid::new_v4(),
crates/sieve-core/src/pipeline/inbound.rs:130:                        id: Uuid::new_v4(),
crates/sieve-core/src/pipeline/inbound.rs:186:                    id: Uuid::new_v4(),
crates/sieve-core/src/pipeline/inbound.rs:198:                    id: Uuid::new_v4(),
crates/sieve-core/src/pipeline/inbound.rs:219:                    id: Uuid::new_v4(),
crates/sieve-core/src/pipeline/outbound.rs:111:                    id: Uuid::new_v4(),
crates/sieve-cli/src/daemon.rs:518:                            let request_id = uuid::Uuid::new_v4();
crates/sieve-cli/src/daemon.rs:754:    let request_id = uuid::Uuid::new_v4();
crates/sieve-cli/src/daemon.rs:957:        id: Uuid::new_v4(),
crates/sieve-cli/src/daemon.rs:974:        id: Uuid::new_v4(),
crates/sieve-cli/src/engine_adapter.rs:79:            request_id: uuid::Uuid::new_v4(),
crates/sieve-cli/src/engine_adapter.rs:202:                id: Uuid::new_v4(),
crates/sieve-cli/src/engine_adapter.rs:303:                id: Uuid::new_v4(),
crates/sieve-cli/src/engine_adapter.rs:331:                    id: Uuid::new_v4(),

exec
/bin/zsh -lc 'cargo test --workspace --all-targets' in /Users/doskey/src/sieve
 exited 101 in 734ms:
   Compiling sieve-hook v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-hook)
   Compiling sieve-rules v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-rules)
   Compiling sieve-fuzz v0.0.0 (/Users/doskey/src/sieve/fuzz)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.64s
     Running unittests src/main.rs (target/debug/deps/sieve-eea63f8e71059d6b)

running 27 tests
test config::tests::defaults_are_sane ... ok
test commands::setup::tests::bad_json_parse_returns_error_not_empty_object ... ok
test config::tests::listen_addr_parses ... ok
test commands::setup::tests::setup_log_entry_created_new_serializes_correctly ... ok
test config::tests::resolved_rules_path_fallback ... ok
test config::tests::parse_dry_run_and_rules_path ... ok
test config::tests::resolved_rules_path_explicit ... ok
test config::tests::parse_minimal_toml ... ok
test config::tests::parse_full_toml ... ok
test commands::setup::tests::default_sieve_toml_has_absolute_paths ... ok
test config::tests::resolved_sieveignore_path_explicit ... ok
test config::tests::unknown_field_rejected ... ok
test commands::setup::tests::plist_contains_absolute_config_flag ... ok
test engine_adapter::tests::map_action_warn_becomes_hook_mark ... ok
test engine_adapter::tests::redact_evidence_long ... ok
test engine_adapter::tests::redact_evidence_short ... ok
test commands::uninstall::tests::uninstall_created_new_true_deletes_file ... ok
test commands::uninstall::tests::uninstall_created_new_false_removes_sieve_entries_only ... ok
test engine_adapter::tests::scan_no_match_returns_empty ... ok
test audit::tests::update_trigger_blocks ... ok
test audit::tests::decision_event_stores_decision_field ... ok
test audit::tests::write_and_read_events ... ok
test engine_adapter::tests::disposition_hook_terminal_beats_enforce_action ... ok
test engine_adapter::tests::span_offset_applied ... ok
test engine_adapter::tests::scan_detects_pattern ... ok
test engine_adapter::tests::disposition_gui_popup_beats_enforce_action ... ok
test engine_adapter::tests::disposition_auto_redact_beats_enforce_action ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audit_append_only.rs (target/debug/deps/audit_append_only-7963e579460992e0)

running 3 tests
test update_is_rejected_by_trigger ... ok
test delete_is_rejected_by_trigger ... ok
test write_3_events_and_read_back ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/inbound_block.rs (target/debug/deps/inbound_block-ebd9dd42f9aab019)

running 10 tests
test malformed_tool_use_partial_json_blocks ... FAILED
test address_substitution_from_prompt_seed_blocks ... FAILED
test ucsb_attack_4_markdown_exfil_warn_only_passes_through ... FAILED
test in_cr_04_persistence_shell_rc_blocked ... FAILED
test ucsb_attack_3_signing_tool_blocked ... FAILED
test unterminated_final_event_still_blocks_critical ... FAILED
test ucsb_attack_2_dangerous_shell_in_tool_use_blocked ... FAILED
test ucsb_attack_1_address_substitution_blocked ... FAILED
test benign_response_passes_through_unchanged ... FAILED
test in_cr_03_sensitive_path_warn_passes_through ... FAILED

failures:

---- malformed_tool_use_partial_json_blocks stdout ----

thread 'malformed_tool_use_partial_json_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- address_substitution_from_prompt_seed_blocks stdout ----

thread 'address_substitution_from_prompt_seed_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_4_markdown_exfil_warn_only_passes_through stdout ----

thread 'ucsb_attack_4_markdown_exfil_warn_only_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_04_persistence_shell_rc_blocked stdout ----

thread 'in_cr_04_persistence_shell_rc_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_3_signing_tool_blocked stdout ----

thread 'ucsb_attack_3_signing_tool_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- unterminated_final_event_still_blocks_critical stdout ----

thread 'unterminated_final_event_still_blocks_critical' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- ucsb_attack_2_dangerous_shell_in_tool_use_blocked stdout ----

thread 'ucsb_attack_2_dangerous_shell_in_tool_use_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_1_address_substitution_blocked stdout ----

thread 'ucsb_attack_1_address_substitution_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- benign_response_passes_through_unchanged stdout ----

thread 'benign_response_passes_through_unchanged' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_03_sensitive_path_warn_passes_through stdout ----

thread 'in_cr_03_sensitive_path_warn_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }


failures:
    address_substitution_from_prompt_seed_blocks
    benign_response_passes_through_unchanged
    in_cr_03_sensitive_path_warn_passes_through
    in_cr_04_persistence_shell_rc_blocked
    malformed_tool_use_partial_json_blocks
    ucsb_attack_1_address_substitution_blocked
    ucsb_attack_2_dangerous_shell_in_tool_use_blocked
    ucsb_attack_3_signing_tool_blocked
    ucsb_attack_4_markdown_exfil_warn_only_passes_through
    unterminated_final_event_still_blocks_critical

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p sieve-cli --test inbound_block`

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/inbound_block.rs | sed -n '70,105p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
    70	    Bytes::from(s)
    71	}
    72	
    73	/// mock 上游 StreamBody 类型（size_hint unknown → hyper 用 chunked transfer，不加 content-length）。
    74	type MockBody = StreamBody<tokio_stream::Once<Result<Frame<Bytes>, Infallible>>>;
    75	
    76	/// 把 Bytes 包成 StreamBody（无 exact size_hint）。
    77	///
    78	/// hyper 对 `Full<Bytes>` 会自动加 content-length；StreamBody unknown size 时用 chunked。
    79	/// sieve 透传 content-length 到客户端，注入 sieve_blocked 后实际 body 超出长度，HTTP 协议错误。
    80	fn bytes_to_chunked_body(data: Bytes) -> MockBody {
    81	    let stream = tokio_stream::once(Ok::<_, Infallible>(Frame::data(data)));
    82	    StreamBody::new(stream)
    83	}
    84	
    85	/// 在 :0 端口启动 plain-HTTP mock 上游（chunked transfer），返回 (addr, shutdown sender)。
    86	///
    87	/// responder 返回 (status, body_bytes)；Content-Type 固定为 `text/event-stream`。
    88	async fn spawn_mock_sse_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
    89	where
    90	    F: Fn(Request<Bytes>) -> Fut + Clone + Send + Sync + 'static,
    91	    Fut: std::future::Future<Output = (hyper::StatusCode, Bytes)> + Send,
    92	{
    93	    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    94	    let addr = listener.local_addr().unwrap();
    95	    let (tx, mut rx) = oneshot::channel::<()>();
    96	
    97	    tokio::spawn(async move {
    98	        loop {
    99	            tokio::select! {
   100	                _ = &mut rx => break,
   101	                accept = listener.accept() => {
   102	                    let Ok((stream, _)) = accept else { continue };
   103	                    let io = TokioIo::new(stream);
   104	                    let r = responder.clone();
   105	                    tokio::spawn(async move {

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '1,260p' && nl -ba crates/sieve-cli/src/daemon.rs | sed -n '260,620p' && nl -ba crates/sieve-cli/src/daemon.rs | sed -n '620,820p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	//! 透传 daemon（架构图节点 ①③⑤⑧）。
     2	//!
     3	//! Week 2：POST /v1/messages body 收集 → 出站规则扫描 → Critical 命中时返回 426；
     4	//! 非 messages 路径 / 解析失败 / 无命中 → 流式透传（Week 1 行为保持不变）。
     5	//!
     6	//! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
     7	//!
     8	//! Week 4（v1.4）：
     9	//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
    10	//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
    11	//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
    12	//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
    13	//!
    14	//! 关联：PRD v1.4 §6.1 §6.7 / ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。
    15	
    16	use anyhow::{anyhow, Context, Result};
    17	use bytes::Bytes;
    18	use futures_util::StreamExt as _;
    19	use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
    20	use hyper::body::Incoming;
    21	use hyper::service::service_fn;
    22	use hyper::{Request, Response};
    23	use hyper_util::rt::{TokioExecutor, TokioIo};
    24	use hyper_util::server::conn::auto;
    25	use sieve_core::detection::Action;
    26	use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
    27	use sieve_core::pipeline::outbound::OutboundFilter;
    28	use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
    29	use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
    30	use sieve_core::sse::parser::SseParser;
    31	use sieve_core::tool_use_aggregator::Aggregator;
    32	use sieve_core::Forwarder;
    33	use std::collections::HashSet;
    34	use std::sync::Arc;
    35	use tokio::net::TcpListener;
    36	use tokio::sync::mpsc;
    37	use tokio_stream::wrappers::ReceiverStream;
    38	
    39	use crate::config::Config;
    40	
    41	/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
    42	type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;
    43	
    44	/// 启动 daemon，永久阻塞直到进程收到信号。
    45	///
    46	/// `filter` 是出站规则引擎包装；`inbound_engine` + `inbound_sieveignore` 用于每连接构造
    47	/// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
    48	/// `cfg.dry_run` 决定是否实际拦截。
    49	///
    50	/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
    51	///
    52	/// # Errors
    53	/// bind 端口失败或 Forwarder 初始化失败时返回错误。
    54	pub async fn run(
    55	    cfg: Config,
    56	    filter: Arc<OutboundFilter>,
    57	    inbound_engine: Arc<dyn InboundEngine>,
    58	    inbound_sieveignore: Arc<HashSet<String>>,
    59	) -> Result<()> {
    60	    let listen = cfg.listen_addr()?;
    61	    let dry_run = cfg.dry_run;
    62	    let forwarder =
    63	        Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);
    64	
    65	    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
    66	    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
    67	    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
    68	    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
    69	        Ok(home) => {
    70	            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
    71	            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
    72	                Ok((server, listener)) => {
    73	                    let server = Arc::new(server);
    74	                    let srv_clone = Arc::clone(&server);
    75	                    tokio::spawn(async move {
    76	                        srv_clone.run(listener).await;
    77	                    });
    78	                    tracing::info!(socket = %socket_path.display(), "IPC server started");
    79	                    Some(server)
    80	                }
    81	                Err(e) => {
    82	                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
    83	                    None
    84	                }
    85	            }
    86	        }
    87	        Err(e) => {
    88	            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
    89	            None
    90	        }
    91	    };
    92	
    93	    let listener = TcpListener::bind(listen)
    94	        .await
    95	        .with_context(|| format!("bind {}", listen))?;
    96	
    97	    tracing::info!(
    98	        listen = %listen,
    99	        upstream = %cfg.upstream_url,
   100	        dry_run = dry_run,
   101	        "sieve daemon started"
   102	    );
   103	
   104	    loop {
   105	        let (stream, peer) = match listener.accept().await {
   106	            Ok(v) => v,
   107	            Err(e) => {
   108	                tracing::warn!(error = %e, "accept failed");
   109	                continue;
   110	            }
   111	        };
   112	
   113	        let forwarder = forwarder.clone();
   114	        let filter = filter.clone();
   115	        let inbound_engine = inbound_engine.clone();
   116	        let inbound_sieveignore = inbound_sieveignore.clone();
   117	        let ipc_server = ipc_server.clone();
   118	
   119	        tokio::spawn(async move {
   120	            let io = TokioIo::new(stream);
   121	            let svc = service_fn(move |req| {
   122	                let f = forwarder.clone();
   123	                let flt = filter.clone();
   124	                // 每连接独立 InboundFilter（&mut self trait 要求）
   125	                let ib_filter =
   126	                    InboundFilter::new(inbound_engine.clone(), inbound_sieveignore.clone());
   127	                let ipc = ipc_server.clone();
   128	                async move { proxy(f, flt, ib_filter, dry_run, ipc, req).await }
   129	            });
   130	
   131	            if let Err(e) = auto::Builder::new(TokioExecutor::new())
   132	                .serve_connection(io, svc)
   133	                .await
   134	            {
   135	                tracing::debug!(peer = %peer, error = %e, "connection closed with error");
   136	            }
   137	        });
   138	    }
   139	}
   140	
   141	/// 请求入口：捕获 `proxy_inner` 的所有错误，转换为 502 Bad Gateway 响应。
   142	async fn proxy(
   143	    forwarder: Arc<Forwarder>,
   144	    filter: Arc<OutboundFilter>,
   145	    inbound_filter: InboundFilter,
   146	    dry_run: bool,
   147	    ipc: Option<Arc<sieve_ipc::IpcServer>>,
   148	    req: Request<Incoming>,
   149	) -> Result<Response<ResponseBody>, hyper::Error> {
   150	    match proxy_inner(forwarder, filter, inbound_filter, dry_run, ipc, req).await {
   151	        Ok(resp) => Ok(resp),
   152	        Err(e) => {
   153	            tracing::error!(error = %e, "proxy failed");
   154	            let body = format!("sieve proxy error: {e}");
   155	            let resp = Response::builder()
   156	                .status(http::StatusCode::BAD_GATEWAY)
   157	                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
   158	                .body(string_body(body))
   159	                .unwrap_or_else(|_| Response::new(empty_body()));
   160	            Ok(resp)
   161	        }
   162	    }
   163	}
   164	
   165	/// 核心代理逻辑。
   166	///
   167	/// - POST /v1/messages → collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测
   168	/// - 其他路径 → 流式透传（Week 1 行为）
   169	async fn proxy_inner(
   170	    forwarder: Arc<Forwarder>,
   171	    filter: Arc<OutboundFilter>,
   172	    inbound_filter: InboundFilter,
   173	    dry_run: bool,
   174	    ipc: Option<Arc<sieve_ipc::IpcServer>>,
   175	    req: Request<Incoming>,
   176	) -> Result<Response<ResponseBody>> {
   177	    let (parts, body) = req.into_parts();
   178	    let path = parts.uri.path().to_string();
   179	    let method = parts.method.clone();
   180	
   181	    let is_messages_post = method == http::Method::POST && path == "/v1/messages";
   182	
   183	    if is_messages_post {
   184	        // 1. collect 完整 body（出站扫描需要全文）
   185	        let collected = body
   186	            .collect()
   187	            .await
   188	            .map_err(|e| anyhow!("collect body: {e}"))?;
   189	        let body_bytes = collected.to_bytes();
   190	
   191	        // 2. 解析 AnthropicRequest；解析失败则直接透传（上游会返回 400）
   192	        let anthropic_req: sieve_core::protocol::anthropic::AnthropicRequest =
   193	            match serde_json::from_slice(&body_bytes) {
   194	                Ok(r) => r,
   195	                Err(e) => {
   196	                    tracing::debug!("non-anthropic body, passing through: {e}");
   197	                    return forward_raw(forwarder, parts, body_bytes).await;
   198	                }
   199	            };
   200	
   201	        // 3. 提取文本段 → 逐段扫描
   202	        let texts = anthropic_req.extract_text_content();
   203	        let mut all_detections: Vec<sieve_core::Detection> = Vec::new();
   204	
   205	        for (offset, text) in &texts {
   206	            use sieve_core::pipeline::PipelineNode;
   207	            use sieve_core::protocol::unified_message::{
   208	                ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
   209	            };
   210	            use std::time::SystemTime;
   211	
   212	            let mut msg = sieve_core::UnifiedMessage {
   213	                role: sieve_core::Role::User,
   214	                content_blocks: vec![ContentBlock::Text {
   215	                    text: text.clone(),
   216	                    span: Some(ContentSpan {
   217	                        start: *offset,
   218	                        end: *offset + text.len(),
   219	                    }),
   220	                }],
   221	                tool_uses: vec![],
   222	                tool_results: vec![],
   223	                metadata: MessageMetadata {
   224	                    session_id: "outbound-scan".into(),
   225	                    direction: Direction::Outbound,
   226	                    upstream_provider: UpstreamProvider::Anthropic,
   227	                    received_at: SystemTime::now(),
   228	                },
   229	            };
   230	
   231	            let hits = filter
   232	                .process(&mut msg)
   233	                .map_err(|e| anyhow!("outbound filter: {e}"))?;
   234	            all_detections.extend(hits);
   235	        }
   236	
   237	        // 4. 决策：
   238	        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
   239	        //    b. fail-closed Critical Block → 426（PRD §9 #3）
   240	        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
   241	        //    d. 其余 → 透传
   242	
   243	        // 4a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
   244	        //
   245	        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
   246	        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
   247	        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
   248	        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
   249	        let redact_hits: Vec<RedactHit> = all_detections
   250	            .iter()
   251	            .filter(|d| matches!(d.action, Action::Redact { .. }))
   252	            .map(|d| RedactHit {
   253	                rule_id: d.rule_id.clone(),
   254	                start: d.span.start,
   255	                end: d.span.end,
   256	            })
   257	            .collect();
   258	
   259	        // 4b/c. 收集需要 Block 的 detection
   260	        let blocking: Vec<&sieve_core::Detection> = all_detections
   260	        let blocking: Vec<&sieve_core::Detection> = all_detections
   261	            .iter()
   262	            .filter(|d| {
   263	                if d.action != Action::Block {
   264	                    return false;
   265	                }
   266	                if d.severity != sieve_core::Severity::Critical {
   267	                    return false;
   268	                }
   269	                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
   270	            })
   271	            .collect();
   272	
   273	        if !blocking.is_empty() {
   274	            tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED");
   275	            for d in &blocking {
   276	                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection");
   277	            }
   278	            let cloned: Vec<sieve_core::Detection> =
   279	                blocking.iter().map(|d| (*d).clone()).collect();
   280	            return Ok(build_426_response(&cloned));
   281	        }
   282	
   283	        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
   284	        //
   285	        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
   286	        // 1. redact_segments() 在文本字符串层替换
   287	        // 2. 把替换后的文本写回 AnthropicRequest messages
   288	        // 3. serde_json 重新序列化为新 body
   289	        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
   290	        if !redact_hits.is_empty() {
   291	            let seg_result = redact_segments(&texts, &redact_hits);
   292	            tracing::info!(
   293	                count = seg_result.redacted_count,
   294	                rules = %seg_result.redacted_summary,
   295	                "OUTBOUND AUTO-REDACT"
   296	            );
   297	
   298	            // 把替换后文本写回 AnthropicRequest，然后重新序列化
   299	            let new_body_bytes =
   300	                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
   301	                    .and_then(|req| {
   302	                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
   303	                    })?;
   304	
   305	            // 验证脱敏后 JSON 仍然合法（关键回归断言）
   306	            if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
   307	                return Err(anyhow!("redact_segments 产生了非法 JSON，fail-closed 拦截"));
   308	            }
   309	
   310	            let new_body = Bytes::from(new_body_bytes);
   311	            let new_len = new_body.len();
   312	
   313	            // 更新 Content-Length header
   314	            let mut new_parts = parts.clone();
   315	            new_parts.headers.insert(
   316	                http::header::CONTENT_LENGTH,
   317	                http::HeaderValue::from(new_len),
   318	            );
   319	
   320	            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
   321	            for (_, text) in &texts {
   322	                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
   323	                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
   324	                }
   325	            }
   326	
   327	            return forward_with_inbound_inspection(
   328	                forwarder,
   329	                inbound_filter,
   330	                dry_run,
   331	                ipc,
   332	                new_parts,
   333	                new_body,
   334	            )
   335	            .await;
   336	        }
   337	
   338	        if dry_run && !all_detections.is_empty() {
   339	            tracing::warn!(
   340	                count = all_detections.len(),
   341	                "OUTBOUND DRY-RUN: would have flagged"
   342	            );
   343	            for d in &all_detections {
   344	                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection (dry_run)");
   345	            }
   346	        }
   347	
   348	        // 5. prompt 地址 seed
   349	        for (_, text) in &texts {
   350	            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
   351	                tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
   352	            }
   353	        }
   354	
   355	        // 6. 出站通过 → 入站 SSE tee 截流检测
   356	        return forward_with_inbound_inspection(
   357	            forwarder,
   358	            inbound_filter,
   359	            dry_run,
   360	            ipc,
   361	            parts,
   362	            body_bytes,
   363	        )
   364	        .await;
   365	    }
   366	
   367	    // 非 messages 路径：Week 1 流式透传
   368	    forward_streaming(forwarder, parts, body).await
   369	}
   370	
   371	/// 透传并同步做入站 SSE 解析检测（tee 模式）。
   372	///
   373	/// 字节流同时被：
   374	/// 1. 原样 forward 给客户端（via bounded channel）
   375	/// 2. 异步喂给 SseParser → Aggregator → InboundFilter 检测
   376	///
   377	/// v1.4 分支逻辑：
   378	/// - `Action::Block`（fail-closed Critical）→ 注入 `sieve_blocked` event 并截流
   379	/// - `Action::HookMark` → 写 IPC pending 文件，SSE 流原样转发（**不注入 sieve_blocked**）
   380	/// - `Action::HoldForDecision` → hold 流 + keep-alive，等用户决策
   381	/// - 其余 → 透传
   382	///
   383	/// 关联：ADR-014 §双层防御、ADR-016 §dispatch 路由、PRD v1.4 §6.7。
   384	async fn forward_with_inbound_inspection(
   385	    forwarder: Arc<Forwarder>,
   386	    mut inbound_filter: InboundFilter,
   387	    dry_run: bool,
   388	    ipc: Option<Arc<sieve_ipc::IpcServer>>,
   389	    mut parts: http::request::Parts,
   390	    body_bytes: Bytes,
   391	) -> Result<Response<ResponseBody>> {
   392	    use http_body_util::Full;
   393	
   394	    let new_uri = forwarder
   395	        .rewrite_uri(&parts.uri)
   396	        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
   397	    parts.uri = new_uri;
   398	    parts.headers.remove(http::header::HOST);
   399	    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
   400	        .map_err(|e| anyhow!("invalid host header: {e}"))?;
   401	    parts.headers.insert(http::header::HOST, host_val);
   402	
   403	    let upstream_body = Full::new(body_bytes)
   404	        .map_err(|e| -> hyper::Error { match e {} })
   405	        .boxed();
   406	    let upstream_req = Request::from_parts(parts, upstream_body);
   407	
   408	    let upstream_resp = forwarder
   409	        .forward(upstream_req)
   410	        .await
   411	        .map_err(|e| anyhow!("forward: {e}"))?;
   412	
   413	    let (mut resp_parts, resp_body) = upstream_resp.into_parts();
   414	
   415	    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
   416	    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
   417	    resp_parts.headers.remove(http::header::CONTENT_LENGTH);
   418	
   419	    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
   420	    const INBOUND_CHANNEL_DEPTH: usize = 64;
   421	    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
   422	        INBOUND_CHANNEL_DEPTH,
   423	    );
   424	
   425	    tokio::spawn(async move {
   426	        let mut parser = SseParser::new();
   427	        let mut aggregator = Aggregator::new();
   428	
   429	        use http_body_util::BodyStream;
   430	        let mut stream = BodyStream::new(resp_body);
   431	
   432	        while let Some(frame_result) = stream.next().await {
   433	            match frame_result {
   434	                Ok(frame) => {
   435	                    let Some(frame_bytes) = frame.data_ref().cloned() else {
   436	                        if tx.send(Ok(frame)).await.is_err() {
   437	                            return;
   438	                        }
   439	                        continue;
   440	                    };
   441	
   442	                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
   443	                    let events = match parser.push_chunk(&frame_bytes) {
   444	                        Ok(evts) => evts,
   445	                        Err(e) => {
   446	                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
   447	                            let cap_detection =
   448	                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
   449	                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
   450	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   451	                            return;
   452	                        }
   453	                    };
   454	
   455	                    // 收集本批 events 的 detections，按 action 分组处理
   456	                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
   457	                        &events,
   458	                        &mut inbound_filter,
   459	                        &mut aggregator,
   460	                        dry_run,
   461	                    );
   462	
   463	                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
   464	                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
   465	                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
   466	                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
   467	                    // 关联：ADR-014 §双层防御、PRD §9 #3。
   468	
   469	                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
   470	                    if !blocking.is_empty() {
   471	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   472	                        for d in &blocking {
   473	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   474	                        }
   475	                        let blocked_payload = build_sieve_blocked_sse(&blocking);
   476	                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   477	                        return;
   478	                    }
   479	
   480	                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
   481	                    for d in &hook_detections {
   482	                        write_hook_pending_silent(d);
   483	                    }
   484	
   485	                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
   486	                    if !hold_detections.is_empty() {
   487	                        if let Some(ref ipc_server) = ipc {
   488	                            // keep-alive channel：daemon 把心跳写入 SSE 流
   489	                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
   490	                            let tx_ka = tx.clone();
   491	
   492	                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
   493	                            // 然后再 hold——这样客户端已经看到触发 event，
   494	                            // hold 期间只收到 keep-alive comment。
   495	                            if tx
   496	                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
   497	                                .await
   498	                                .is_err()
   499	                            {
   500	                                return;
   501	                            }
   502	
   503	                            // 启动 keep-alive 转发 task
   504	                            let ka_fwd_handle = tokio::spawn(async move {
   505	                                while let Some(ka_bytes) = ka_rx.recv().await {
   506	                                    if tx_ka
   507	                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
   508	                                        .await
   509	                                        .is_err()
   510	                                    {
   511	                                        break;
   512	                                    }
   513	                                }
   514	                            });
   515	
   516	                            // 构造 IPC 请求
   517	                            use chrono::Utc;
   518	                            let request_id = uuid::Uuid::new_v4();
   519	                            let timeout_seconds = hold_detections
   520	                                .iter()
   521	                                .find_map(|d| {
   522	                                    if let Action::HoldForDecision {
   523	                                        timeout_seconds, ..
   524	                                    } = d.action
   525	                                    {
   526	                                        Some(timeout_seconds)
   527	                                    } else {
   528	                                        None
   529	                                    }
   530	                                })
   531	                                .unwrap_or(60);
   532	
   533	                            let ipc_detections = hold_detections
   534	                                .iter()
   535	                                .map(|d| sieve_ipc::protocol::DetectionPayload {
   536	                                    rule_id: d.rule_id.clone(),
   537	                                    severity: map_severity_to_ipc(d.severity),
   538	                                    disposition: sieve_ipc::Disposition::GuiPopup,
   539	                                    title: format!("检测命中：{}", d.rule_id),
   540	                                    one_line_summary: d.evidence_truncated.clone(),
   541	                                    details: serde_json::json!({}),
   542	                                })
   543	                                .collect();
   544	
   545	                            let ipc_req = sieve_ipc::DecisionRequest {
   546	                                request_id,
   547	                                created_at: Utc::now(),
   548	                                timeout_seconds,
   549	                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   550	                                detections: ipc_detections,
   551	                            };
   552	
   553	                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
   554	                                Arc::clone(ipc_server),
   555	                                ipc_req,
   556	                                ka_tx,
   557	                            )
   558	                            .await;
   559	
   560	                            ka_fwd_handle.abort();
   561	
   562	                            match outcome {
   563	                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
   564	                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
   565	                                    // 允许：继续转发后续 SSE 帧
   566	                                    // 当前帧已在 hold 前发出，继续循环
   567	                                    continue;
   568	                                }
   569	                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
   570	                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
   571	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   572	                                    let _ = tx
   573	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   574	                                        .await;
   575	                                    return;
   576	                                }
   577	                                Err(e) => {
   578	                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
   579	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   580	                                    let _ = tx
   581	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   582	                                        .await;
   583	                                    return;
   584	                                }
   585	                            }
   586	                        } else {
   587	                            // IPC 未初始化：fail-closed，阻断
   588	                            tracing::warn!(
   589	                                "GuiPopup detection but IPC server not initialized; fail-closed"
   590	                            );
   591	                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   592	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   593	                            return;
   594	                        }
   595	                    }
   596	
   597	                    // 无 blocking / hold：透传原始 frame
   598	                    if tx
   599	                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
   600	                        .await
   601	                        .is_err()
   602	                    {
   603	                        return;
   604	                    }
   605	                }
   606	                Err(e) => {
   607	                    let _ = tx
   608	                        .send(Err(std::io::Error::other(format!(
   609	                            "upstream body error: {e}"
   610	                        ))))
   611	                        .await;
   612	                    return;
   613	                }
   614	            }
   615	        }
   616	
   617	        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
   618	        let flushed = parser.flush();
   619	        let (blocking, hook_detections, flush_hold_detections) =
   620	            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
   620	            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
   621	
   622	        for d in &hook_detections {
   623	            write_hook_pending_silent(d);
   624	        }
   625	
   626	        if !blocking.is_empty() {
   627	            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
   628	            for d in &blocking {
   629	                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
   630	            }
   631	            let blocked_payload = build_sieve_blocked_sse(&blocking);
   632	            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   633	            return;
   634	        }
   635	
   636	        // 修 #5（flush 阶段 hold 丢失修复）：
   637	        // flush 路径的 HoldForDecision 命中不能静默丢弃。
   638	        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
   639	        // 关联：ADR-014 §双层防御、PRD §9 #3。
   640	        if !flush_hold_detections.is_empty() {
   641	            tracing::warn!(
   642	                count = flush_hold_detections.len(),
   643	                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
   644	            );
   645	            for d in &flush_hold_detections {
   646	                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
   647	            }
   648	            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
   649	            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   650	        }
   651	    });
   652	
   653	    let body_stream = ReceiverStream::new(rx);
   654	    let response_body: ResponseBody = StreamBody::new(body_stream)
   655	        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
   656	        .boxed();
   657	
   658	    Ok(Response::from_parts(resp_parts, response_body))
   659	}
   660	
   661	/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
   662	/// - `blocking`：`Action::Block` 需立即截流的 detections
   663	/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
   664	/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
   665	///
   666	/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
   667	///
   668	/// 关联 ADR-016 §dispatch 路由、ADR-014 §双层防御。
   669	fn classify_inbound_detections(
   670	    events: &[sieve_core::sse::parser::SseEvent],
   671	    inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
   672	    aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
   673	    dry_run: bool,
   674	) -> (
   675	    Vec<sieve_core::Detection>,
   676	    Vec<sieve_core::Detection>,
   677	    Vec<sieve_core::Detection>,
   678	) {
   679	    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();
   680	
   681	    for evt in events {
   682	        match inbound_filter.observe_event(evt) {
   683	            Ok(hits) => all_hits.extend(hits),
   684	            Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
   685	        }
   686	        match aggregator.process(evt) {
   687	            Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
   688	                Ok(hits) => all_hits.extend(hits),
   689	                Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
   690	            },
   691	            Ok(None) => {}
   692	            Err(sieve_core::tool_use_aggregator::AggregatorError::MalformedToolUse {
   693	                ref tool_id,
   694	                ref error,
   695	            }) => {
   696	                tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
   697	                all_hits.push(build_malformed_tool_use_detection(tool_id));
   698	            }
   699	            Err(e) => {
   700	                tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
   701	                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
   702	            }
   703	        }
   704	    }
   705	
   706	    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
   707	    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
   708	    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();
   709	
   710	    for d in all_hits {
   711	        match &d.action {
   712	            Action::Block => {
   713	                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
   714	                if d.severity == sieve_core::Severity::Critical
   715	                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
   716	                {
   717	                    blocking.push(d);
   718	                }
   719	                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
   720	            }
   721	            Action::HookMark => {
   722	                // Hook 类：写 pending 文件，SSE 流继续转发
   723	                hook_detections.push(d);
   724	            }
   725	            Action::HoldForDecision { .. } => {
   726	                // GUI 类：hold 流等决策
   727	                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
   728	                hold_detections.push(d);
   729	            }
   730	            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
   731	                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
   732	            }
   733	        }
   734	    }
   735	
   736	    (blocking, hook_detections, hold_detections)
   737	}
   738	
   739	/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
   740	///
   741	/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
   742	/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
   743	fn write_hook_pending_silent(d: &sieve_core::Detection) {
   744	    use chrono::Utc;
   745	
   746	    let sieve_home = match sieve_ipc::paths::sieve_home() {
   747	        Ok(h) => h,
   748	        Err(e) => {
   749	            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
   750	            return;
   751	        }
   752	    };
   753	
   754	    let request_id = uuid::Uuid::new_v4();
   755	    let ipc_req = sieve_ipc::DecisionRequest {
   756	        request_id,
   757	        created_at: Utc::now(),
   758	        timeout_seconds: 60,
   759	        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   760	        detections: vec![sieve_ipc::protocol::DetectionPayload {
   761	            rule_id: d.rule_id.clone(),
   762	            severity: map_severity_to_ipc(d.severity),
   763	            disposition: sieve_ipc::Disposition::HookTerminal,
   764	            title: format!("检测命中：{}", d.rule_id),
   765	            one_line_summary: d.evidence_truncated.clone(),
   766	            details: serde_json::json!({}),
   767	        }],
   768	    };
   769	
   770	    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
   771	        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
   772	    } else {
   773	        tracing::info!(
   774	            rule = %d.rule_id,
   775	            request_id = %request_id,
   776	            "HookMark: pending file written, SSE stream continues"
   777	        );
   778	    }
   779	}
   780	
   781	/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
   782	fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
   783	    match s {
   784	        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
   785	        sieve_core::Severity::High => sieve_ipc::Severity::High,
   786	        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
   787	        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
   788	    }
   789	}
   790	
   791	/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
   792	fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
   793	    let payload = serde_json::json!({
   794	        "type": "sieve_blocked",
   795	        "blocked_at": epoch_secs_string(),
   796	        "detections": detections.iter().map(|d| serde_json::json!({
   797	            "rule_id": d.rule_id,
   798	            "severity": d.severity,
   799	            "fingerprint": d.fingerprint,
   800	        })).collect::<Vec<_>>(),
   801	        "guidance": {
   802	            "zh": format!(
   803	                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
   804	                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
   805	                detections.len()
   806	            ),
   807	            "en": format!(
   808	                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
   809	                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
   810	                detections.len()
   811	            ),
   812	        }
   813	    });
   814	    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
   815	}
   816	
   817	/// 用已收集的 body bytes 重新构造请求并转发。
   818	async fn forward_raw(
   819	    forwarder: Arc<Forwarder>,
   820	    mut parts: http::request::Parts,

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '420,660p'; nl -ba crates/sieve-cli/src/daemon.rs | sed -n '820,1040p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   420	    const INBOUND_CHANNEL_DEPTH: usize = 64;
   421	    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
   422	        INBOUND_CHANNEL_DEPTH,
   423	    );
   424	
   425	    tokio::spawn(async move {
   426	        let mut parser = SseParser::new();
   427	        let mut aggregator = Aggregator::new();
   428	
   429	        use http_body_util::BodyStream;
   430	        let mut stream = BodyStream::new(resp_body);
   431	
   432	        while let Some(frame_result) = stream.next().await {
   433	            match frame_result {
   434	                Ok(frame) => {
   435	                    let Some(frame_bytes) = frame.data_ref().cloned() else {
   436	                        if tx.send(Ok(frame)).await.is_err() {
   437	                            return;
   438	                        }
   439	                        continue;
   440	                    };
   441	
   442	                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
   443	                    let events = match parser.push_chunk(&frame_bytes) {
   444	                        Ok(evts) => evts,
   445	                        Err(e) => {
   446	                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
   447	                            let cap_detection =
   448	                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
   449	                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
   450	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   451	                            return;
   452	                        }
   453	                    };
   454	
   455	                    // 收集本批 events 的 detections，按 action 分组处理
   456	                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
   457	                        &events,
   458	                        &mut inbound_filter,
   459	                        &mut aggregator,
   460	                        dry_run,
   461	                    );
   462	
   463	                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
   464	                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
   465	                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
   466	                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
   467	                    // 关联：ADR-014 §双层防御、PRD §9 #3。
   468	
   469	                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
   470	                    if !blocking.is_empty() {
   471	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   472	                        for d in &blocking {
   473	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   474	                        }
   475	                        let blocked_payload = build_sieve_blocked_sse(&blocking);
   476	                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   477	                        return;
   478	                    }
   479	
   480	                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
   481	                    for d in &hook_detections {
   482	                        write_hook_pending_silent(d);
   483	                    }
   484	
   485	                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
   486	                    if !hold_detections.is_empty() {
   487	                        if let Some(ref ipc_server) = ipc {
   488	                            // keep-alive channel：daemon 把心跳写入 SSE 流
   489	                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
   490	                            let tx_ka = tx.clone();
   491	
   492	                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
   493	                            // 然后再 hold——这样客户端已经看到触发 event，
   494	                            // hold 期间只收到 keep-alive comment。
   495	                            if tx
   496	                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
   497	                                .await
   498	                                .is_err()
   499	                            {
   500	                                return;
   501	                            }
   502	
   503	                            // 启动 keep-alive 转发 task
   504	                            let ka_fwd_handle = tokio::spawn(async move {
   505	                                while let Some(ka_bytes) = ka_rx.recv().await {
   506	                                    if tx_ka
   507	                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
   508	                                        .await
   509	                                        .is_err()
   510	                                    {
   511	                                        break;
   512	                                    }
   513	                                }
   514	                            });
   515	
   516	                            // 构造 IPC 请求
   517	                            use chrono::Utc;
   518	                            let request_id = uuid::Uuid::new_v4();
   519	                            let timeout_seconds = hold_detections
   520	                                .iter()
   521	                                .find_map(|d| {
   522	                                    if let Action::HoldForDecision {
   523	                                        timeout_seconds, ..
   524	                                    } = d.action
   525	                                    {
   526	                                        Some(timeout_seconds)
   527	                                    } else {
   528	                                        None
   529	                                    }
   530	                                })
   531	                                .unwrap_or(60);
   532	
   533	                            let ipc_detections = hold_detections
   534	                                .iter()
   535	                                .map(|d| sieve_ipc::protocol::DetectionPayload {
   536	                                    rule_id: d.rule_id.clone(),
   537	                                    severity: map_severity_to_ipc(d.severity),
   538	                                    disposition: sieve_ipc::Disposition::GuiPopup,
   539	                                    title: format!("检测命中：{}", d.rule_id),
   540	                                    one_line_summary: d.evidence_truncated.clone(),
   541	                                    details: serde_json::json!({}),
   542	                                })
   543	                                .collect();
   544	
   545	                            let ipc_req = sieve_ipc::DecisionRequest {
   546	                                request_id,
   547	                                created_at: Utc::now(),
   548	                                timeout_seconds,
   549	                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   550	                                detections: ipc_detections,
   551	                            };
   552	
   553	                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
   554	                                Arc::clone(ipc_server),
   555	                                ipc_req,
   556	                                ka_tx,
   557	                            )
   558	                            .await;
   559	
   560	                            ka_fwd_handle.abort();
   561	
   562	                            match outcome {
   563	                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
   564	                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
   565	                                    // 允许：继续转发后续 SSE 帧
   566	                                    // 当前帧已在 hold 前发出，继续循环
   567	                                    continue;
   568	                                }
   569	                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
   570	                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
   571	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   572	                                    let _ = tx
   573	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   574	                                        .await;
   575	                                    return;
   576	                                }
   577	                                Err(e) => {
   578	                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
   579	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   580	                                    let _ = tx
   581	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   582	                                        .await;
   583	                                    return;
   584	                                }
   585	                            }
   586	                        } else {
   587	                            // IPC 未初始化：fail-closed，阻断
   588	                            tracing::warn!(
   589	                                "GuiPopup detection but IPC server not initialized; fail-closed"
   590	                            );
   591	                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   592	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   593	                            return;
   594	                        }
   595	                    }
   596	
   597	                    // 无 blocking / hold：透传原始 frame
   598	                    if tx
   599	                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
   600	                        .await
   601	                        .is_err()
   602	                    {
   603	                        return;
   604	                    }
   605	                }
   606	                Err(e) => {
   607	                    let _ = tx
   608	                        .send(Err(std::io::Error::other(format!(
   609	                            "upstream body error: {e}"
   610	                        ))))
   611	                        .await;
   612	                    return;
   613	                }
   614	            }
   615	        }
   616	
   617	        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
   618	        let flushed = parser.flush();
   619	        let (blocking, hook_detections, flush_hold_detections) =
   620	            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
   621	
   622	        for d in &hook_detections {
   623	            write_hook_pending_silent(d);
   624	        }
   625	
   626	        if !blocking.is_empty() {
   627	            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
   628	            for d in &blocking {
   629	                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
   630	            }
   631	            let blocked_payload = build_sieve_blocked_sse(&blocking);
   632	            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   633	            return;
   634	        }
   635	
   636	        // 修 #5（flush 阶段 hold 丢失修复）：
   637	        // flush 路径的 HoldForDecision 命中不能静默丢弃。
   638	        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
   639	        // 关联：ADR-014 §双层防御、PRD §9 #3。
   640	        if !flush_hold_detections.is_empty() {
   641	            tracing::warn!(
   642	                count = flush_hold_detections.len(),
   643	                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
   644	            );
   645	            for d in &flush_hold_detections {
   646	                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
   647	            }
   648	            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
   649	            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   650	        }
   651	    });
   652	
   653	    let body_stream = ReceiverStream::new(rx);
   654	    let response_body: ResponseBody = StreamBody::new(body_stream)
   655	        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
   656	        .boxed();
   657	
   658	    Ok(Response::from_parts(resp_parts, response_body))
   659	}
   660	
   820	    mut parts: http::request::Parts,
   821	    body_bytes: Bytes,
   822	) -> Result<Response<ResponseBody>> {
   823	    use http_body_util::Full;
   824	
   825	    let new_uri = forwarder
   826	        .rewrite_uri(&parts.uri)
   827	        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
   828	    parts.uri = new_uri;
   829	    parts.headers.remove(http::header::HOST);
   830	    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
   831	        .map_err(|e| anyhow!("invalid host header: {e}"))?;
   832	    parts.headers.insert(http::header::HOST, host_val);
   833	
   834	    let upstream_body = Full::new(body_bytes)
   835	        .map_err(|e| -> hyper::Error { match e {} })
   836	        .boxed();
   837	    let upstream_req = Request::from_parts(parts, upstream_body);
   838	
   839	    let upstream_resp = forwarder
   840	        .forward(upstream_req)
   841	        .await
   842	        .map_err(|e| anyhow!("forward: {e}"))?;
   843	
   844	    let (resp_parts, resp_body) = upstream_resp.into_parts();
   845	    let body: ResponseBody = resp_body
   846	        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
   847	        .boxed();
   848	    Ok(Response::from_parts(resp_parts, body))
   849	}
   850	
   851	/// 流式透传（Week 1 路径），不缓冲 body。
   852	async fn forward_streaming(
   853	    forwarder: Arc<Forwarder>,
   854	    mut parts: http::request::Parts,
   855	    body: Incoming,
   856	) -> Result<Response<ResponseBody>> {
   857	    let new_uri = forwarder
   858	        .rewrite_uri(&parts.uri)
   859	        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
   860	    parts.uri = new_uri;
   861	    parts.headers.remove(http::header::HOST);
   862	    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
   863	        .map_err(|e| anyhow!("invalid host header: {e}"))?;
   864	    parts.headers.insert(http::header::HOST, host_val);
   865	
   866	    let upstream_body = body.map_err(|e| -> hyper::Error { e }).boxed();
   867	    let upstream_req = Request::from_parts(parts, upstream_body);
   868	
   869	    let upstream_resp = forwarder
   870	        .forward(upstream_req)
   871	        .await
   872	        .map_err(|e| anyhow!("forward: {e}"))?;
   873	
   874	    let (resp_parts, resp_body) = upstream_resp.into_parts();
   875	    let body: ResponseBody = resp_body
   876	        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
   877	        .boxed();
   878	    Ok(Response::from_parts(resp_parts, body))
   879	}
   880	
   881	/// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
   882	fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
   883	    let blocked_at = epoch_secs_string();
   884	    let detections_json: Vec<serde_json::Value> = detections
   885	        .iter()
   886	        .map(|d| {
   887	            serde_json::json!({
   888	                "rule_id": d.rule_id,
   889	                "severity": d.severity,
   890	                "fingerprint": d.fingerprint,
   891	            })
   892	        })
   893	        .collect();
   894	    let body_json = serde_json::json!({
   895	        "type": "sieve_blocked",
   896	        "blocked_at": blocked_at,
   897	        "detections": detections_json,
   898	        "guidance": {
   899	            "zh": format!(
   900	                "Sieve 检测到 {} 条出站 Critical 命中。请检查后用 .sieveignore 加入 fingerprint 白名单，或重新发送脱敏消息。",
   901	                detections.len()
   902	            ),
   903	            "en": format!(
   904	                "Sieve blocked {} outbound critical detection(s). Review your message, then either redact or add fingerprint(s) to .sieveignore.",
   905	                detections.len()
   906	            ),
   907	        }
   908	    });
   909	    let body_bytes = Bytes::from(body_json.to_string());
   910	    Response::builder()
   911	        .status(http::StatusCode::UPGRADE_REQUIRED) // 426
   912	        .header(
   913	            http::header::CONTENT_TYPE,
   914	            "application/json; charset=utf-8",
   915	        )
   916	        .body(bytes_body(body_bytes))
   917	        .unwrap_or_else(|_| Response::new(empty_body()))
   918	}
   919	
   920	/// 返回 UNIX epoch 秒字符串（Phase 1 简化，Week 4 改 RFC3339）。
   921	fn epoch_secs_string() -> String {
   922	    use std::time::{SystemTime, UNIX_EPOCH};
   923	    let secs = SystemTime::now()
   924	        .duration_since(UNIX_EPOCH)
   925	        .map(|d| d.as_secs())
   926	        .unwrap_or(0);
   927	    secs.to_string()
   928	}
   929	
   930	/// 把字节包成 `ResponseBody`。
   931	fn bytes_body(b: Bytes) -> ResponseBody {
   932	    use http_body_util::Full;
   933	    Full::new(b)
   934	        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
   935	        .boxed()
   936	}
   937	
   938	/// 把字符串包成 `ResponseBody`（用于错误响应）。
   939	fn string_body(s: String) -> ResponseBody {
   940	    bytes_body(Bytes::from(s))
   941	}
   942	
   943	/// 空 body（fallback 错误响应）。
   944	fn empty_body() -> ResponseBody {
   945	    use http_body_util::Empty;
   946	    Empty::<Bytes>::new()
   947	        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
   948	        .boxed()
   949	}
   950	
   951	/// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
   952	fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
   953	    use sieve_core::detection::{Action, ContentSource};
   954	    use sieve_core::protocol::unified_message::ContentSpan;
   955	    use uuid::Uuid;
   956	    sieve_core::Detection {
   957	        id: Uuid::new_v4(),
   958	        rule_id: "IN-CR-05-MALFORMED".into(),
   959	        severity: sieve_core::Severity::Critical,
   960	        action: Action::Block,
   961	        source: ContentSource::InboundAssistantText,
   962	        span: ContentSpan { start: 0, end: 0 },
   963	        evidence_truncated: format!("tool_id={tool_id}"),
   964	        fingerprint: "malformed-tool-use-partial-json".into(),
   965	    }
   966	}
   967	
   968	/// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
   969	fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
   970	    use sieve_core::detection::{Action, ContentSource};
   971	    use sieve_core::protocol::unified_message::ContentSpan;
   972	    use uuid::Uuid;
   973	    sieve_core::Detection {
   974	        id: Uuid::new_v4(),
   975	        rule_id: rule_id.into(),
   976	        severity: sieve_core::Severity::Critical,
   977	        action: Action::Block,
   978	        source: ContentSource::InboundAssistantText,
   979	        span: ContentSpan { start: 0, end: 0 },
   980	        evidence_truncated: String::new(),
   981	        fingerprint: fingerprint_key.into(),
   982	    }
   983	}
   984	
   985	/// 把脱敏后的文本段列表写回 [`AnthropicRequest`] 并返回新 request。
   986	///
   987	/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
   988	/// `redacted_texts` 是 `redact_segments()` 返回的替换后文本列表（顺序对应）。
   989	///
   990	/// 实现逻辑：遍历 messages，对每个文本 content 按 segment 索引匹配并替换。
   991	///
   992	/// # Errors
   993	/// 如果 `redacted_texts` 长度与 `original_texts` 不一致，返回错误。
   994	///
   995	/// 关联：PRD v1.4 §6.1（AutoRedact 路径），修 #1（AutoRedact 偏移修复）。
   996	fn apply_redacted_texts_to_request(
   997	    req: &sieve_core::protocol::anthropic::AnthropicRequest,
   998	    original_texts: &[(usize, String)],
   999	    redacted_texts: &[String],
  1000	) -> Result<sieve_core::protocol::anthropic::AnthropicRequest> {
  1001	    if original_texts.len() != redacted_texts.len() {
  1002	        return Err(anyhow!(
  1003	            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
  1004	            redacted_texts.len(),
  1005	            original_texts.len()
  1006	        ));
  1007	    }
  1008	
  1009	    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
  1010	    let mut seg_idx = 0usize;
  1011	
  1012	    let mut new_messages: Vec<sieve_core::protocol::anthropic::AnthropicMessage> = Vec::new();
  1013	    for msg in &req.messages {
  1014	        let new_content = match &msg.content {
  1015	            serde_json::Value::String(_) => {
  1016	                // String 类型：一个 segment
  1017	                let replacement = redacted_texts
  1018	                    .get(seg_idx)
  1019	                    .cloned()
  1020	                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
  1021	                seg_idx += 1;
  1022	                serde_json::Value::String(replacement)
  1023	            }
  1024	            serde_json::Value::Array(blocks) => {
  1025	                let mut new_blocks = Vec::with_capacity(blocks.len());
  1026	                for block in blocks {
  1027	                    if let Some(block_obj) = block.as_object() {
  1028	                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
  1029	                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
  1030	                        {
  1031	                            let replacement =
  1032	                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
  1033	                                    block_obj
  1034	                                        .get("text")
  1035	                                        .and_then(|v| v.as_str())
  1036	                                        .unwrap_or("")
  1037	                                        .to_string()
  1038	                                });
  1039	                            seg_idx += 1;
  1040	                            let mut new_obj = block_obj.clone();

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '1040,1120p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
  1040	                            let mut new_obj = block_obj.clone();
  1041	                            new_obj
  1042	                                .insert("text".to_string(), serde_json::Value::String(replacement));
  1043	                            new_blocks.push(serde_json::Value::Object(new_obj));
  1044	                            continue;
  1045	                        }
  1046	                    }
  1047	                    new_blocks.push(block.clone());
  1048	                }
  1049	                serde_json::Value::Array(new_blocks)
  1050	            }
  1051	            other => other.clone(),
  1052	        };
  1053	        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {
  1054	            role: msg.role.clone(),
  1055	            content: new_content,
  1056	        });
  1057	    }
  1058	
  1059	    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
  1060	    let new_system = if let Some(system) = &req.system {
  1061	        if system.as_str().is_some() {
  1062	            let replacement = redacted_texts
  1063	                .get(seg_idx)
  1064	                .cloned()
  1065	                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
  1066	            seg_idx += 1;
  1067	            Some(serde_json::Value::String(replacement))
  1068	        } else if let Some(blocks) = system.as_array() {
  1069	            let mut new_blocks = Vec::with_capacity(blocks.len());
  1070	            for block in blocks {
  1071	                if block.get("text").and_then(|v| v.as_str()).is_some() {
  1072	                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
  1073	                        block
  1074	                            .get("text")
  1075	                            .and_then(|v| v.as_str())
  1076	                            .unwrap_or("")
  1077	                            .to_string()
  1078	                    });
  1079	                    seg_idx += 1;
  1080	                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
  1081	                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
  1082	                    new_blocks.push(serde_json::Value::Object(new_obj));
  1083	                } else {
  1084	                    new_blocks.push(block.clone());
  1085	                }
  1086	            }
  1087	            Some(serde_json::Value::Array(new_blocks))
  1088	        } else {
  1089	            Some(system.clone())
  1090	        }
  1091	    } else {
  1092	        None
  1093	    };
  1094	
  1095	    let _ = seg_idx; // 消除 unused variable 警告
  1096	
  1097	    Ok(sieve_core::protocol::anthropic::AnthropicRequest {
  1098	        model: req.model.clone(),
  1099	        max_tokens: req.max_tokens,
  1100	        messages: new_messages,
  1101	        stream: req.stream,
  1102	        system: new_system,
  1103	        tools: req.tools.clone(),
  1104	        tool_choice: req.tool_choice.clone(),
  1105	        extra: req.extra.clone(),
  1106	    })
  1107	}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/engine_adapter.rs | sed -n '1,420p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	//! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` /
     2	//! `sieve_core::InboundEngine` trait。
     3	//!
     4	//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，
     5	//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
     6	//!
     7	//! 关联 ADR-002 / PRD §5.1 / Week 2 出站 / Week 3 入站拦截集成。
     8	
     9	use sieve_core::detection::{fingerprint, Action, ContentSource, Detection, Severity};
    10	use sieve_core::error::SieveCoreResult;
    11	use sieve_core::pipeline::inbound::InboundEngine;
    12	use sieve_core::pipeline::outbound::OutboundEngine;
    13	use sieve_core::protocol::unified_message::ContentSpan;
    14	use sieve_core::tool_use_aggregator::CompletedToolCall;
    15	use sieve_rules::engine::{MatchEngine, VectorscanEngine};
    16	use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};
    17	use std::collections::HashMap;
    18	use std::sync::Arc;
    19	use uuid::Uuid;
    20	
    21	/// `VectorscanEngine` 包装，实现 `sieve_core::OutboundEngine`。
    22	///
    23	/// 内部持有规则反查表（`rule_id → RuleEntry`），用于从 `MatchHit` 取真实 severity/action。
    24	pub struct OutboundAdapter {
    25	    engine: Arc<VectorscanEngine>,
    26	    /// rule_id → RuleEntry 反查表，用于从 MatchHit 映射元数据。
    27	    rule_lookup: HashMap<String, RuleEntry>,
    28	}
    29	
    30	impl OutboundAdapter {
    31	    /// 构造 adapter。
    32	    ///
    33	    /// `rules` 与 `VectorscanEngine::compile` 传入的规则集一致，用于构建反查表。
    34	    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
    35	        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
    36	        Self {
    37	            engine,
    38	            rule_lookup,
    39	        }
    40	    }
    41	}
    42	
    43	/// 把 `sieve_rules::Severity` 映射为 `sieve_core::Severity`。
    44	fn map_severity(r: RulesSeverity) -> Severity {
    45	    match r {
    46	        RulesSeverity::Low => Severity::Low,
    47	        RulesSeverity::Medium => Severity::Medium,
    48	        RulesSeverity::High => Severity::High,
    49	        RulesSeverity::Critical => Severity::Critical,
    50	    }
    51	}
    52	
    53	/// 根据 `RuleEntry.disposition` 和 `RulesAction` 映射为 `sieve_core::Action`。
    54	///
    55	/// v1.4 重构：优先按 `effective_disposition()` 路由，`RulesAction` 作为兜底。
    56	///
    57	/// | Disposition       | Action                                       |
    58	/// |-------------------|----------------------------------------------|
    59	/// | AutoRedact        | `Redact { placeholder }`                     |
    60	/// | GuiPopup          | `HoldForDecision { request_id, timeout_s }`  |
    61	/// | HookTerminal      | `HookMark`                                   |
    62	/// | StatusBar         | `MarkOnly`                                   |
    63	///
    64	/// `timeout_seconds` / `default_on_timeout` 取自 `RuleEntry`，不再硬编码 5。
    65	///
    66	/// 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
    67	fn map_action_by_disposition(
    68	    disposition: sieve_rules::manifest::Disposition,
    69	    _rule_action: RulesAction,
    70	    rule_id: &str,
    71	    timeout_seconds: u32,
    72	) -> Action {
    73	    use sieve_rules::manifest::Disposition;
    74	    match disposition {
    75	        Disposition::AutoRedact => Action::Redact {
    76	            placeholder: format!("[REDACTED:{rule_id}]"),
    77	        },
    78	        Disposition::GuiPopup => Action::HoldForDecision {
    79	            request_id: uuid::Uuid::new_v4(),
    80	            timeout_seconds,
    81	        },
    82	        Disposition::HookTerminal => Action::HookMark,
    83	        Disposition::StatusBar => Action::MarkOnly,
    84	    }
    85	}
    86	
    87	/// 旧接口：仅用 `RulesAction` 映射（兜底，无 disposition 信息时使用）。
    88	///
    89	/// `Warn` → `HookMark`（v1.4 后 Warn 一律走 HookTerminal 路径）。
    90	///
    91	/// 注：修 #2 后生产路径不再调用此函数（disposition 优先），
    92	/// 保留用于单元测试验证 Warn → HookMark 的语义不变。
    93	#[allow(dead_code)]
    94	fn map_action(r: RulesAction) -> Action {
    95	    match r {
    96	        RulesAction::Block => Action::Block,
    97	        RulesAction::Warn => Action::HookMark,
    98	        RulesAction::Mark => Action::MarkOnly,
    99	        RulesAction::Allow => Action::SilentLog,
   100	    }
   101	}
   102	
   103	/// 截断并脱敏证据片段（用于 `Detection.evidence_truncated`）。
   104	///
   105	/// 超过 8 字符时，保留前 4 + `***` + 后 4，防止原始密钥写入审计日志。
   106	fn redact_evidence(matched: &str) -> String {
   107	    let chars: Vec<char> = matched.chars().collect();
   108	    let len = chars.len();
   109	    if len <= 8 {
   110	        "*".repeat(len)
   111	    } else {
   112	        let head: String = chars[..4].iter().collect();
   113	        let tail: String = chars[len - 4..].iter().collect();
   114	        format!("{head}***{tail}")
   115	    }
   116	}
   117	
   118	/// `VectorscanEngine` 包装，实现 `sieve_core::InboundEngine`。
   119	///
   120	/// 与 [`OutboundAdapter`] 共用辅助函数（`map_severity` / `map_action` / `redact_evidence`），
   121	/// 额外在工具调用检查中调用 `sieve_rules::critical_lock::enforce_action` 保证 fail-closed。
   122	pub struct InboundAdapter {
   123	    engine: Arc<VectorscanEngine>,
   124	    /// rule_id → RuleEntry 反查表。
   125	    rule_lookup: HashMap<String, RuleEntry>,
   126	}
   127	
   128	impl InboundAdapter {
   129	    /// 构造 adapter。
   130	    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
   131	        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
   132	        Self {
   133	            engine,
   134	            rule_lookup,
   135	        }
   136	    }
   137	}
   138	
   139	impl InboundEngine for InboundAdapter {
   140	    fn scan_text(
   141	        &self,
   142	        input: &str,
   143	        source: ContentSource,
   144	        body_offset: usize,
   145	    ) -> SieveCoreResult<Vec<Detection>> {
   146	        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
   147	            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
   148	        })?;
   149	
   150	        let mut detections = Vec::new();
   151	        for hit in hits {
   152	            let rule = self.rule_lookup.get(&hit.rule_id);
   153	
   154	            let evidence_start = hit.start.min(input.len());
   155	            let evidence_end = hit.end.min(input.len());
   156	            let matched_text = &input[evidence_start..evidence_end];
   157	
   158	            if let Some(r) = rule {
   159	                if self.engine.is_excluded(matched_text, r) {
   160	                    continue;
   161	                }
   162	            }
   163	
   164	            let severity = rule
   165	                .map(|r| map_severity(r.severity))
   166	                .unwrap_or(Severity::Critical);
   167	
   168	            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复，入站侧）。
   169	            //
   170	            // 规则显式写了 disposition 时直接路由；
   171	            // disposition=None 且 fail-closed 时才强制 Block。
   172	            // 这确保 IN-CR-02（hook_terminal）/ IN-CR-05（gui_popup）即使在 fail-closed
   173	            // 名单里也能走正确的 HookMark / HoldForDecision 路径（不被截成 Block）。
   174	            //
   175	            // 关联：ADR-016（二维处置矩阵）、ADR-014（双层防御）、PRD v1.4 §5.4。
   176	            let action = if let Some(r) = rule {
   177	                if let Some(disp) = r.disposition {
   178	                    // 显式 disposition：直接路由，不经过 enforce_action
   179	                    let timeout = r.timeout_seconds.unwrap_or(60);
   180	                    map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
   181	                } else {
   182	                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
   183	                    let enforced =
   184	                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
   185	                    if enforced == RulesAction::Block {
   186	                        Action::Block
   187	                    } else {
   188	                        let disp = r.effective_disposition();
   189	                        let timeout = r.timeout_seconds.unwrap_or(60);
   190	                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
   191	                    }
   192	                }
   193	            } else {
   194	                // 规则表中找不到：fail-closed Block
   195	                Action::Block
   196	            };
   197	
   198	            let evidence_truncated = redact_evidence(matched_text);
   199	            let fp = fingerprint(&hit.rule_id, matched_text);
   200	
   201	            detections.push(Detection {
   202	                id: Uuid::new_v4(),
   203	                rule_id: hit.rule_id.clone(),
   204	                severity,
   205	                action,
   206	                source,
   207	                span: ContentSpan {
   208	                    start: body_offset + hit.start,
   209	                    end: body_offset + hit.end,
   210	                },
   211	                evidence_truncated,
   212	                fingerprint: fp,
   213	            });
   214	        }
   215	        Ok(detections)
   216	    }
   217	
   218	    fn check_tool_use(
   219	        &self,
   220	        tool: &CompletedToolCall,
   221	        source: ContentSource,
   222	    ) -> SieveCoreResult<Vec<Detection>> {
   223	        let mut hits = Vec::new();
   224	        // 1. 工具名扫描（IN-CR-05 签名工具）
   225	        hits.extend(self.scan_text(&tool.name, source, 0)?);
   226	        // 2. 工具输入序列化扫描（IN-CR-02 危险 shell 等）
   227	        if let Ok(input_str) = serde_json::to_string(&tool.input) {
   228	            hits.extend(self.scan_text(&input_str, source, 0)?);
   229	        }
   230	        Ok(hits)
   231	    }
   232	}
   233	
   234	impl OutboundEngine for OutboundAdapter {
   235	    /// 扫描文本，返回已过滤（per-rule allowlist）的命中列表，并执行 BIP39 second-pass。
   236	    ///
   237	    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
   238	    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
   239	    ///
   240	    /// BIP39 second-pass（PRD §9 #4）：vectorscan 之后独立扫描。
   241	    /// 先提取全部在词表的连续词窗口，再做 SHA-256 checksum 验证，
   242	    /// **仅 checksum 通过才生成 Critical Detection**。
   243	    /// 词表命中但 checksum 失败的窗口**不得**定级 Critical（差异化要求）。
   244	    fn scan_text(
   245	        &self,
   246	        input: &str,
   247	        source: ContentSource,
   248	        body_byte_offset: usize,
   249	    ) -> SieveCoreResult<Vec<Detection>> {
   250	        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
   251	            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
   252	        })?;
   253	
   254	        let mut detections = Vec::new();
   255	        for hit in hits {
   256	            let rule = self.rule_lookup.get(&hit.rule_id);
   257	
   258	            // per-rule allowlist 过滤
   259	            let evidence_start = hit.start.min(input.len());
   260	            let evidence_end = hit.end.min(input.len());
   261	            let matched_text = &input[evidence_start..evidence_end];
   262	
   263	            if let Some(r) = rule {
   264	                if self.engine.is_excluded(matched_text, r) {
   265	                    continue;
   266	                }
   267	            }
   268	
   269	            let severity = rule
   270	                .map(|r| map_severity(r.severity))
   271	                .unwrap_or(Severity::Critical);
   272	            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复）。
   273	            //
   274	            // 规则显式写了 disposition 时，**直接按 disposition 路由**——
   275	            // 这确保 OUT-01（auto_redact）即使在 fail-closed 名单里也走 Redact 而非 Block。
   276	            // 只有 disposition=None（旧规则 / 无显式配置）且 fail-closed 时，才走 Block。
   277	            //
   278	            // 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
   279	            let action = rule
   280	                .map(|r| {
   281	                    if let Some(disp) = r.disposition {
   282	                        // 显式 disposition：直接路由，不经过 enforce_action
   283	                        let timeout = r.timeout_seconds.unwrap_or(60);
   284	                        map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
   285	                    } else {
   286	                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
   287	                        let enforced =
   288	                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
   289	                        if enforced == RulesAction::Block {
   290	                            Action::Block
   291	                        } else {
   292	                            let disp = r.effective_disposition();
   293	                            let timeout = r.timeout_seconds.unwrap_or(60);
   294	                            map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
   295	                        }
   296	                    }
   297	                })
   298	                .unwrap_or(Action::Block);
   299	            let evidence_truncated = redact_evidence(matched_text);
   300	            let fp = fingerprint(&hit.rule_id, matched_text);
   301	
   302	            detections.push(Detection {
   303	                id: Uuid::new_v4(),
   304	                rule_id: hit.rule_id.clone(),
   305	                severity,
   306	                action,
   307	                source,
   308	                span: ContentSpan {
   309	                    start: body_byte_offset + hit.start,
   310	                    end: body_byte_offset + hit.end,
   311	                },
   312	                evidence_truncated,
   313	                fingerprint: fp,
   314	            });
   315	        }
   316	
   317	        // BIP39 second-pass（关联 PRD §9 #4 差异化点）
   318	        // vectorscan 不覆盖 BIP39，此处独立扫描：
   319	        // 1. 按空白分词，提取全在词表的连续窗口
   320	        // 2. 对每个窗口做 SHA-256 checksum 验证
   321	        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
   322	        let wl = sieve_rules::wordlist::wordlist_index();
   323	        let tokens: Vec<&str> = input.split_whitespace().collect();
   324	        let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
   325	        for window in candidates {
   326	            if sieve_rules::bip39::verify_checksum(&window, wl) {
   327	                let window_text = window.join(" ");
   328	                let evidence_truncated = redact_evidence(&window_text);
   329	                let fp = fingerprint("OUT-09", &window_text);
   330	                detections.push(Detection {
   331	                    id: Uuid::new_v4(),
   332	                    rule_id: "OUT-09".to_string(),
   333	                    severity: Severity::Critical,
   334	                    action: Action::Block,
   335	                    source,
   336	                    // span 为整个输入范围的近似（无精确字节偏移）
   337	                    span: ContentSpan {
   338	                        start: body_byte_offset,
   339	                        end: body_byte_offset + input.len(),
   340	                    },
   341	                    evidence_truncated,
   342	                    fingerprint: fp,
   343	                });
   344	                // 同一文本只需报一次（找到一个有效助记词即触发拦截）
   345	                break;
   346	            }
   347	        }
   348	
   349	        Ok(detections)
   350	    }
   351	}
   352	
   353	#[cfg(test)]
   354	mod tests {
   355	    use super::*;
   356	    use sieve_rules::engine::VectorscanEngine;
   357	    use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};
   358	
   359	    fn make_rule(
   360	        id: &str,
   361	        pattern: &str,
   362	        severity: RulesSeverity,
   363	        action: RulesAction,
   364	    ) -> RuleEntry {
   365	        RuleEntry {
   366	            id: id.into(),
   367	            description: id.into(),
   368	            pattern: pattern.into(),
   369	            severity,
   370	            action,
   371	            entropy_min: None,
   372	            keywords: vec![],
   373	            allowlist_regexes: vec![],
   374	            allowlist_stopwords: vec![],
   375	            disposition: None,
   376	            timeout_seconds: None,
   377	            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
   378	        }
   379	    }
   380	
   381	    #[test]
   382	    fn scan_detects_pattern() {
   383	        let rules = vec![make_rule(
   384	            "OUT-TEST",
   385	            r"secret",
   386	            RulesSeverity::Critical,
   387	            RulesAction::Block,
   388	        )];
   389	        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
   390	        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
   391	        let hits = adapter
   392	            .scan_text("my secret key", ContentSource::OutboundUserText, 0)
   393	            .unwrap();
   394	        assert_eq!(hits.len(), 1);
   395	        assert_eq!(hits[0].rule_id, "OUT-TEST");
   396	        assert_eq!(hits[0].severity, Severity::Critical);
   397	        assert!(matches!(hits[0].action, Action::Block));
   398	    }
   399	
   400	    #[test]
   401	    fn scan_no_match_returns_empty() {
   402	        let rules = vec![make_rule(
   403	            "OUT-TEST",
   404	            r"secret",
   405	            RulesSeverity::High,
   406	            RulesAction::Warn,
   407	        )];
   408	        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
   409	        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
   410	        let hits = adapter
   411	            .scan_text(
   412	                "nothing suspicious here",
   413	                ContentSource::OutboundUserText,
   414	                0,
   415	            )
   416	            .unwrap();
   417	        assert!(hits.is_empty());
   418	    }
   419	
   420	    #[test]

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/engine_adapter.rs | sed -n '420,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   420	    #[test]
   421	    fn map_action_warn_becomes_hook_mark() {
   422	        // v1.4：Warn 一律走 HookTerminal 路径（HookMark action）
   423	        let a = map_action(RulesAction::Warn);
   424	        assert!(matches!(a, Action::HookMark));
   425	    }
   426	
   427	    #[test]
   428	    fn redact_evidence_short() {
   429	        let r = redact_evidence("abc");
   430	        assert_eq!(r, "***");
   431	    }
   432	
   433	    #[test]
   434	    fn redact_evidence_long() {
   435	        let r = redact_evidence("1234567890abcdef");
   436	        assert!(r.starts_with("1234"));
   437	        assert!(r.ends_with("cdef"));
   438	        assert!(r.contains("***"));
   439	    }
   440	
   441	    #[test]
   442	    fn span_offset_applied() {
   443	        let rules = vec![make_rule(
   444	            "OUT-OFF",
   445	            r"hello",
   446	            RulesSeverity::Low,
   447	            RulesAction::Mark,
   448	        )];
   449	        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
   450	        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
   451	        // offset=100, text starts at byte 0 within "say hello", pattern at 4..9
   452	        let hits = adapter
   453	            .scan_text("say hello", ContentSource::OutboundSystemText, 100)
   454	            .unwrap();
   455	        assert_eq!(hits.len(), 1);
   456	        assert_eq!(hits[0].span.start, 104); // 100 + 4
   457	        assert_eq!(hits[0].span.end, 109); // 100 + 9
   458	    }
   459	
   460	    // ── 修 #2 回归：disposition 优先于 enforce_action ──────────────────────────
   461	
   462	    /// disposition=auto_redact 即使 action=block（fail-closed 名单）也走 Redact 路径。
   463	    ///
   464	    /// 修 #2（路由短路修复）：OUT-01 等 AutoRedact 规则在 fail-closed 名单里，
   465	    /// 旧代码 enforce_action 会把 action 强制变 Block，跳过 disposition 路由。
   466	    /// 修复后：显式 disposition 优先，OUT-01 必须走 Action::Redact 而非 Action::Block。
   467	    #[test]
   468	    fn disposition_auto_redact_beats_enforce_action() {
   469	        let mut rule = make_rule(
   470	            "OUT-01", // 在 fail-closed 名单里
   471	            r"sk-ant",
   472	            RulesSeverity::Critical,
   473	            RulesAction::Block,
   474	        );
   475	        rule.disposition = Some(sieve_rules::manifest::Disposition::AutoRedact);
   476	
   477	        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
   478	        let adapter = OutboundAdapter::new(Arc::new(engine), vec![rule]);
   479	
   480	        let hits = adapter
   481	            .scan_text("my sk-ant-key here", ContentSource::OutboundUserText, 0)
   482	            .unwrap();
   483	        assert_eq!(hits.len(), 1);
   484	        assert_eq!(hits[0].rule_id, "OUT-01");
   485	        // 关键断言：应该是 Redact，不是 Block
   486	        assert!(
   487	            matches!(hits[0].action, Action::Redact { .. }),
   488	            "disposition=auto_redact 应走 Redact 路径，实际: {:?}",
   489	            hits[0].action
   490	        );
   491	    }
   492	
   493	    /// disposition=hook_terminal 即使在 fail-closed 名单里也走 HookMark 路径。
   494	    ///
   495	    /// 修 #2 回归：IN-CR-02 等 HookTerminal 规则不应被 enforce_action 截成 Block。
   496	    #[test]
   497	    fn disposition_hook_terminal_beats_enforce_action() {
   498	        let mut rule = make_rule(
   499	            "IN-CR-02", // 在 fail-closed 名单里
   500	            r"rm -rf",
   501	            RulesSeverity::Critical,
   502	            RulesAction::Block,
   503	        );
   504	        rule.disposition = Some(sieve_rules::manifest::Disposition::HookTerminal);
   505	
   506	        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
   507	        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);
   508	
   509	        let hits = adapter
   510	            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
   511	            .unwrap();
   512	        assert_eq!(hits.len(), 1);
   513	        assert_eq!(hits[0].rule_id, "IN-CR-02");
   514	        // 关键断言：应该是 HookMark，不是 Block
   515	        assert!(
   516	            matches!(hits[0].action, Action::HookMark),
   517	            "disposition=hook_terminal 应走 HookMark 路径，实际: {:?}",
   518	            hits[0].action
   519	        );
   520	    }
   521	
   522	    /// disposition=gui_popup 即使在 fail-closed 名单里也走 HoldForDecision 路径。
   523	    #[test]
   524	    fn disposition_gui_popup_beats_enforce_action() {
   525	        let mut rule = make_rule(
   526	            "IN-CR-05-EVM", // 在 fail-closed 名单里
   527	            r"eth_signTypedData",
   528	            RulesSeverity::Critical,
   529	            RulesAction::Block,
   530	        );
   531	        rule.disposition = Some(sieve_rules::manifest::Disposition::GuiPopup);
   532	        rule.timeout_seconds = Some(60);
   533	
   534	        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
   535	        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);
   536	
   537	        let hits = adapter
   538	            .scan_text(
   539	                "call eth_signTypedData method",
   540	                ContentSource::InboundAssistantText,
   541	                0,
   542	            )
   543	            .unwrap();
   544	        assert_eq!(hits.len(), 1);
   545	        // 关键断言：应该是 HoldForDecision，不是 Block
   546	        assert!(
   547	            matches!(hits[0].action, Action::HoldForDecision { .. }),
   548	            "disposition=gui_popup 应走 HoldForDecision 路径，实际: {:?}",
   549	            hits[0].action
   550	        );
   551	    }
   552	}

exec
/bin/zsh -lc 'git diff -- crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/engine/mod.rs crates/sieve-rules/rules/outbound.toml crates/sieve-rules/rules/inbound.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/crates/sieve-rules/rules/inbound.toml b/crates/sieve-rules/rules/inbound.toml
index ff6c5d7..98c5e8a 100644
--- a/crates/sieve-rules/rules/inbound.toml
+++ b/crates/sieve-rules/rules/inbound.toml
@@ -13,6 +13,9 @@ description = "Address substitution attack (handled by sieve-core::address_guard
 pattern = "__ADDRESS_GUARD_PLACEHOLDER__"
 severity = "critical"
 action = "block"
+disposition = "gui_popup"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 # IN-CR-02 危险 shell 命令（Semgrep command-injection 风格）
 [[rules]]
@@ -22,6 +25,9 @@ pattern = '''(?i)rm\s+-rf?\s+[/~*]'''
 severity = "critical"
 action = "block"
 keywords = ["rm"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-02-CURL-PIPE"
@@ -30,6 +36,9 @@ pattern = '''(?i)curl\s+\S+\s*\|\s*(ba)?sh'''
 severity = "critical"
 action = "block"
 keywords = ["curl"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-02-WGET-PIPE"
@@ -38,6 +47,9 @@ pattern = '''(?i)wget\s+\S+\s*\|\s*(ba)?sh'''
 severity = "critical"
 action = "block"
 keywords = ["wget"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-02-EVAL"
@@ -46,6 +58,9 @@ pattern = '''(?i)eval\s*[\(\$"]'''
 severity = "critical"
 action = "block"
 keywords = ["eval"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-02-NC-REVERSE"
@@ -54,6 +69,9 @@ pattern = '''(?i)nc\s+(-e|--sh-exec)\s+/bin/(ba)?sh'''
 severity = "critical"
 action = "block"
 keywords = ["nc"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-02-DD-WIPE"
@@ -62,6 +80,9 @@ pattern = '''(?i)dd\s+if=/dev/zero\s+of=/dev/'''
 severity = "critical"
 action = "block"
 keywords = ["dd"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 # ---------------------------------------------------------------------------
 # IN-CR-03 敏感路径访问（PRD §5.2，Week 4）
@@ -76,6 +97,9 @@ severity = "high"
 action = "warn"
 keywords = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"]
 allowlist_regexes = ['''\.pub\b''']
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-SSH-DIR"
@@ -85,6 +109,9 @@ severity = "high"
 action = "warn"
 keywords = [".ssh"]
 allowlist_regexes = ['''~/\.ssh/(?:known_hosts|authorized_keys|config|environment)\b''']
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-AWS-CREDS"
@@ -93,6 +120,9 @@ pattern = '''(?i)\.aws/credentials\b'''
 severity = "high"
 action = "warn"
 keywords = ["credentials"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-DOTENV"
@@ -102,6 +132,9 @@ severity = "high"
 action = "warn"
 keywords = [".env"]
 allowlist_regexes = ['''(?i)\.env\.(?:example|template|sample|dist|test|ci|cypress)\b''']
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-ETH-KEYSTORE"
@@ -110,6 +143,9 @@ pattern = '''(?i)UTC--[0-9T\-Z\.]{19,32}--[a-fA-F0-9]{40}\b'''
 severity = "high"
 action = "warn"
 keywords = ["UTC--"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-GPG-DIR"
@@ -118,6 +154,9 @@ pattern = '''~/\.gnupg(?:/[a-zA-Z0-9_\-\.]+)?'''
 severity = "high"
 action = "warn"
 keywords = [".gnupg"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-NETRC"
@@ -128,6 +167,9 @@ pattern = '''\.netrc\b'''
 severity = "high"
 action = "warn"
 keywords = [".netrc"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-MACOS-KEYCHAIN"
@@ -136,6 +178,9 @@ pattern = '''\b(?:login|System)\.keychain(?:-db)?\b'''
 severity = "high"
 action = "warn"
 keywords = ["keychain"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-GCP-CREDS"
@@ -144,6 +189,9 @@ pattern = '''(?i)\.config/gcloud/(?:application_default_credentials\.json|legacy
 severity = "high"
 action = "warn"
 keywords = ["gcloud"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-03-SOLANA-KEYPAIR"
@@ -152,6 +200,9 @@ pattern = '''(?i)\.config/solana/[a-zA-Z0-9_\-]+\.json\b'''
 severity = "high"
 action = "warn"
 keywords = ["solana"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 # ---------------------------------------------------------------------------
 # IN-CR-04 持久化机制（PRD §5.2 / US-07，Week 4）
@@ -169,6 +220,9 @@ pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*\.(?:bashrc|bash_profile|bash_login|
 severity = "critical"
 action = "block"
 keywords = ["bashrc", "zshrc", "bash_profile", "zprofile"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-CRONTAB"
@@ -177,6 +231,9 @@ pattern = '''\bcrontab\s+(?:-e\b|-r\b|<)'''
 severity = "critical"
 action = "block"
 keywords = ["crontab"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-CRON-D-WRITE"
@@ -185,6 +242,9 @@ pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*/etc/cron\.(?:d|daily|hourly|monthly
 severity = "critical"
 action = "block"
 keywords = ["/etc/cron"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-LAUNCHCTL"
@@ -193,6 +253,9 @@ pattern = '''\blaunchctl\s+(?:load|bootstrap|enable|kickstart|asuser)\b'''
 severity = "critical"
 action = "block"
 keywords = ["launchctl"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-LAUNCH-AGENT-PLIST"
@@ -201,6 +264,9 @@ pattern = '''(?:>>?|tee\s+(?:-a\s+)?|cp\s+\S+\s+|mv\s+\S+\s+|cat\s+>\s*)[^\n;]*L
 severity = "critical"
 action = "block"
 keywords = ["LaunchAgents", "LaunchDaemons"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-SYSTEMCTL-ENABLE"
@@ -209,6 +275,9 @@ pattern = '''\bsystemctl\s+(?:--user\s+)?(?:enable|start|daemon-reload)\b'''
 severity = "critical"
 action = "block"
 keywords = ["systemctl"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-SYSTEMD-UNIT-WRITE"
@@ -217,6 +286,9 @@ pattern = '''(?:>>?|tee\s+(?:-a\s+)?|cat\s+>\s*)[^\n;]*(?:/etc/systemd/system|\.
 severity = "critical"
 action = "block"
 keywords = ["systemd"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-FISH-CONFIG"
@@ -225,6 +297,9 @@ pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*\.config/fish/(?:config\.fish|conf\.
 severity = "critical"
 action = "block"
 keywords = ["fish"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-04-LOGIN-ITEMS"
@@ -233,6 +308,9 @@ pattern = '''(?i)(?:\bdefaults\s+write\b[^\n;]*\bLoginItems\b|\bosascript\b[^\n;
 severity = "critical"
 action = "block"
 keywords = ["LoginItems", "osascript"]
+disposition = "hook_terminal"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 # IN-GEN-04 markdown exfil（warn 级别；Week 4 由旧 IN-CR-04 重命名归入 IN-GEN-* 命名空间，
 # 关联 PRD §5.2 / US-08，详见 CHANGELOG [BREAKING]）
@@ -242,6 +320,9 @@ description = "Markdown image with URL query params (potential data exfil)"
 pattern = '''!\[[^\]]*\]\(https?://[^)]+[\?&][^)]*\)'''
 severity = "high"
 action = "warn"
+disposition = "gui_popup"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 # IN-CR-05 签名工具白名单（EVM + Solana + Bitcoin 全拦）
 [[rules]]
@@ -251,6 +332,9 @@ pattern = '''(?i)\b(eth_sendTransaction|eth_signTransaction|eth_signTypedData(_v
 severity = "critical"
 action = "block"
 keywords = ["sign"]
+disposition = "gui_popup"
+timeout_seconds = 120
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-05-SOLANA"
@@ -259,6 +343,9 @@ pattern = '''(?i)\b(signTransaction|signMessage|partialSign|signAllTransactions|
 severity = "critical"
 action = "block"
 keywords = ["sign"]
+disposition = "gui_popup"
+timeout_seconds = 120
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-CR-05-BITCOIN"
@@ -267,6 +354,9 @@ pattern = '''(?i)\b(signRawTransaction(With(Key|Wallet))?|signMessage)\b'''
 severity = "critical"
 action = "block"
 keywords = ["sign"]
+disposition = "gui_popup"
+timeout_seconds = 120
+default_on_timeout = "block"
 
 # IN-GEN-01~03 候选（Week 4 完整化）
 [[rules]]
@@ -275,6 +365,9 @@ description = "Markdown javascript: URI"
 pattern = '''(?i)\[.*?\]\(javascript:'''
 severity = "critical"
 action = "block"
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-GEN-02"
@@ -282,6 +375,9 @@ description = "Inline HTML img with external URL"
 pattern = '''(?i)<img\s[^>]*src\s*=\s*["']https?://'''
 severity = "high"
 action = "warn"
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
 
 [[rules]]
 id = "IN-GEN-03"
@@ -290,3 +386,6 @@ pattern = '''(?i)bash\s+-c\s+["']'''
 severity = "critical"
 action = "block"
 keywords = ["bash"]
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
diff --git a/crates/sieve-rules/rules/outbound.toml b/crates/sieve-rules/rules/outbound.toml
index a956987..d4315cb 100644
--- a/crates/sieve-rules/rules/outbound.toml
+++ b/crates/sieve-rules/rules/outbound.toml
@@ -20,6 +20,8 @@ entropy_min = 4.5
 keywords = ["sk-ant-api03"]
 allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
 allowlist_stopwords = []
+disposition = "auto_redact"
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-02: OpenAI API Key
@@ -37,6 +39,8 @@ entropy_min = 4.5
 keywords = ["T3BlbkFJ"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "auto_redact"
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-03: AWS Access Key ID
@@ -52,6 +56,8 @@ entropy_min = 3.0
 keywords = ["AKIA", "ASIA", "ABIA", "ACCA"]
 allowlist_regexes = []
 allowlist_stopwords = ["AKIAIOSFODNN7EXAMPLE"]  # AWS 官方文档示例 key
+disposition = "auto_redact"
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-04: GitHub Personal Access Token
@@ -67,6 +73,8 @@ entropy_min = 4.0
 keywords = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "auto_redact"
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-05: Google Cloud API Key
@@ -82,6 +90,8 @@ entropy_min = 4.0
 keywords = ["AIza"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "auto_redact"
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-06: JWT Token
@@ -98,6 +108,9 @@ entropy_min = 3.5
 keywords = ["eyJ"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "gui_popup"
+timeout_seconds = 15
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-07: PEM Private Key Header
@@ -114,6 +127,9 @@ entropy_min = 0.0
 keywords = ["-----BEGIN"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "gui_popup"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 # ---------------------------------------------------------------------------
 # OUT-08: Stripe Live Secret / Publishable / Restricted Key
@@ -129,6 +145,9 @@ entropy_min = 3.5
 keywords = ["_live_"]
 allowlist_regexes = ['(?i)test|example']
 allowlist_stopwords = []
+disposition = "gui_popup"
+timeout_seconds = 15
+default_on_timeout = "redact"
 
 # ---------------------------------------------------------------------------
 # OUT-09: Slack Token
@@ -144,6 +163,9 @@ entropy_min = 3.0
 keywords = ["xoxb", "xoxp", "xoxa", "xoxs"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "gui_popup"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 # ---------------------------------------------------------------------------
 # OUT-10: OpenSSH Private Key Header
@@ -159,6 +181,9 @@ entropy_min = 0.0
 keywords = ["BEGIN OPENSSH"]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "gui_popup"
+timeout_seconds = 60
+default_on_timeout = "block"
 
 # ---------------------------------------------------------------------------
 # OUT-11: Discord Bot Token
@@ -175,6 +200,7 @@ entropy_min = 3.5
 keywords = ["."]
 allowlist_regexes = []
 allowlist_stopwords = []
+disposition = "status_bar"
 
 # ---------------------------------------------------------------------------
 # OUT-09（BIP39 助记词）在 engine_adapter 中通过 second-pass 实现，
diff --git a/crates/sieve-rules/src/critical_lock.rs b/crates/sieve-rules/src/critical_lock.rs
index 73e7d26..52bec1a 100644
--- a/crates/sieve-rules/src/critical_lock.rs
+++ b/crates/sieve-rules/src/critical_lock.rs
@@ -1,21 +1,34 @@
-//! Critical 规则强制 fail-closed 名单（关联 ADR-007）。
+//! Critical 规则强制 fail-closed 名单（关联 ADR-007 / ADR-014 / PRD v1.4 §5.4）。
 //!
-//! 此清单中的规则，无论 config 如何设置（包括 dry_run = true），
-//! 命中时 action 强制为 Block，无视 manifest 中的 action 字段。
+//! ## 语义说明
+//!
+//! - [`FAIL_CLOSED_RULES`]：**不可关闭、不可永久白名单**的规则集合（所有 Critical），
+//!   包括 Hook 类——Hook 的 fail-closed 由 sieve-hook 侧实现，但代理侧同样不允许绕过。
+//! - [`HOOK_RULES`]：disposition=HookTerminal 的规则（IN-CR-02~04 + IN-GEN-01~03），
+//!   命中后写 IPC pending file，由 sieve-hook 在 PreToolUse 阶段拦截。
+//! - [`GUI_RULES`]：disposition=GuiPopup 的规则（IN-CR-01/05 + IN-GEN-04 + OUT-06~10），
+//!   命中后 hold SSE 流并通过 IPC 弹出 GUI 等待决策。
+//!
+//! 变更需走 ADR（关联 ADR-007 §2 / ADR-014 §"disposition 矩阵"）。
 
 use crate::manifest::Action;
 
-/// fail-closed 规则 ID 清单。变更需走 ADR（关联 ADR-007 §2 / §"Week N 落地范围"）。
+/// fail-closed 规则 ID 清单。
+///
+/// 包含所有 Critical 规则（IN-CR-* + 出站 Critical OUT-*）。Hook 类规则的
+/// fail-closed 由 sieve-hook 实现，但本清单同样列入以保证代理侧不可旁路。
+/// 变更此清单需更新对应 ADR（ADR-007 §2）。
 pub const FAIL_CLOSED_RULES: &[&str] = &[
-    // 入站
+    // IN-CR-01：地址替换（gui_popup，sieve-core::address_guard 实现）
     "IN-CR-01",
+    // IN-CR-02：危险 shell 命令（hook_terminal）
     "IN-CR-02",
     "IN-CR-02-CURL-PIPE",
     "IN-CR-02-WGET-PIPE",
     "IN-CR-02-EVAL",
     "IN-CR-02-NC-REVERSE",
     "IN-CR-02-DD-WIPE",
-    // IN-CR-04 持久化机制（Week 4 落地，PRD §5.2 / US-07，写持久化文件 = 后门埋点）
+    // IN-CR-04 持久化机制（hook_terminal，Week 4 落地，PRD §5.2 / US-07）
     "IN-CR-04-SHELL-RC-APPEND",
     "IN-CR-04-CRONTAB",
     "IN-CR-04-CRON-D-WRITE",
@@ -25,25 +38,82 @@
     "IN-CR-04-SYSTEMD-UNIT-WRITE",
     "IN-CR-04-FISH-CONFIG",
     "IN-CR-04-LOGIN-ITEMS",
+    // IN-CR-05：签名工具（gui_popup，签名不可逆，PRD §9 #3）
     "IN-CR-05-EVM",
     "IN-CR-05-SOLANA",
     "IN-CR-05-BITCOIN",
     "IN-CR-05-MALFORMED", // P0-6: malformed tool_use partial_json fail-closed（PRD §9 #3）
+    // IN-GEN-01/03：JS URI + bash -c（hook_terminal）
     "IN-GEN-01",
     "IN-GEN-03",
-    // 出站（全部 OUT-01~12）
+    // 出站 Critical（auto_redact 或 gui_popup，timeout default_on_timeout=block）
     "OUT-01",
     "OUT-02",
     "OUT-03",
     "OUT-04",
-    "OUT-05",
+    "OUT-07",
+    "OUT-08",
+    "OUT-09",
+    "OUT-10",
+];
+
+/// disposition=HookTerminal 的规则集合（PRD v1.4 §5.4.1 / ADR-014）。
+///
+/// 这些规则命中后，代理侧**不截断 SSE 流**，而是写 IPC pending file，
+/// 由 sieve-hook 在 Claude Code PreToolUse 钩子阶段拦截决策。
+pub const HOOK_RULES: &[&str] = &[
+    // IN-CR-02：危险 shell 命令
+    "IN-CR-02",
+    "IN-CR-02-CURL-PIPE",
+    "IN-CR-02-WGET-PIPE",
+    "IN-CR-02-EVAL",
+    "IN-CR-02-NC-REVERSE",
+    "IN-CR-02-DD-WIPE",
+    // IN-CR-03：敏感路径访问
+    "IN-CR-03-SSH-PRIVATE",
+    "IN-CR-03-SSH-DIR",
+    "IN-CR-03-AWS-CREDS",
+    "IN-CR-03-DOTENV",
+    "IN-CR-03-ETH-KEYSTORE",
+    "IN-CR-03-GPG-DIR",
+    "IN-CR-03-NETRC",
+    "IN-CR-03-MACOS-KEYCHAIN",
+    "IN-CR-03-GCP-CREDS",
+    "IN-CR-03-SOLANA-KEYPAIR",
+    // IN-CR-04：持久化机制
+    "IN-CR-04-SHELL-RC-APPEND",
+    "IN-CR-04-CRONTAB",
+    "IN-CR-04-CRON-D-WRITE",
+    "IN-CR-04-LAUNCHCTL",
+    "IN-CR-04-LAUNCH-AGENT-PLIST",
+    "IN-CR-04-SYSTEMCTL-ENABLE",
+    "IN-CR-04-SYSTEMD-UNIT-WRITE",
+    "IN-CR-04-FISH-CONFIG",
+    "IN-CR-04-LOGIN-ITEMS",
+    // IN-GEN-01~03：JS URI + 外链 img + bash -c
+    "IN-GEN-01",
+    "IN-GEN-02",
+    "IN-GEN-03",
+];
+
+/// disposition=GuiPopup 的规则集合（PRD v1.4 §5.4.1 / ADR-014）。
+///
+/// 这些规则命中后，代理侧 hold SSE 流，通过 IPC 通知 GUI 弹窗等待用户决策。
+pub const GUI_RULES: &[&str] = &[
+    // 入站 Critical：地址替换 + 签名工具
+    "IN-CR-01",
+    "IN-CR-05-EVM",
+    "IN-CR-05-SOLANA",
+    "IN-CR-05-BITCOIN",
+    "IN-CR-05-MALFORMED",
+    // IN-GEN-04：markdown exfil
+    "IN-GEN-04",
+    // 出站：JWT + PEM + Stripe + Slack + OpenSSH
     "OUT-06",
     "OUT-07",
     "OUT-08",
     "OUT-09",
     "OUT-10",
-    "OUT-11",
-    "OUT-12",
 ];
 
 /// 检查给定 rule_id 是否在 fail-closed 名单中。
@@ -51,6 +121,16 @@ pub fn is_fail_closed(rule_id: &str) -> bool {
     FAIL_CLOSED_RULES.contains(&rule_id)
 }
 
+/// 检查给定 rule_id 是否为 HookTerminal 处置规则。
+pub fn is_hook_rule(rule_id: &str) -> bool {
+    HOOK_RULES.contains(&rule_id)
+}
+
+/// 检查给定 rule_id 是否为 GuiPopup 处置规则。
+pub fn is_gui_rule(rule_id: &str) -> bool {
+    GUI_RULES.contains(&rule_id)
+}
+
 /// 强制覆盖 action：fail-closed 规则一律返回 Block。
 pub fn enforce_action(rule_id: &str, requested: Action) -> Action {
     if is_fail_closed(rule_id) {
@@ -74,7 +154,7 @@ fn known_critical_rules_in_list() {
     #[test]
     fn unknown_rule_not_failclosed() {
         assert!(!is_fail_closed("UNKNOWN-RULE"));
-        // IN-GEN-04 markdown exfil 是 high warn（Week 4 由旧 IN-CR-04 重命名）
+        // IN-GEN-04 markdown exfil 是 high warn（gui_popup，不在 fail-closed 名单）
         assert!(!is_fail_closed("IN-GEN-04"));
         // 旧 ID 不再存在；显式断言以防回归
         assert!(!is_fail_closed("IN-CR-04"));
@@ -104,4 +184,72 @@ fn enforce_overrides_action() {
             Action::Block
         );
     }
+
+    /// HOOK_RULES 与 GUI_RULES 不应有重叠（两个 disposition 互斥）。
+    #[test]
+    fn hook_and_gui_rules_are_disjoint() {
+        for id in HOOK_RULES {
+            assert!(
+                !GUI_RULES.contains(id),
+                "rule {id} is in both HOOK_RULES and GUI_RULES — disposition must be unique"
+            );
+        }
+    }
+
+    /// FAIL_CLOSED_RULES 必须包含所有 IN-CR-* Critical 规则。
+    #[test]
+    fn fail_closed_covers_all_in_cr() {
+        let in_cr_critical = [
+            "IN-CR-01",
+            "IN-CR-02",
+            "IN-CR-02-CURL-PIPE",
+            "IN-CR-02-WGET-PIPE",
+            "IN-CR-02-EVAL",
+            "IN-CR-02-NC-REVERSE",
+            "IN-CR-02-DD-WIPE",
+            "IN-CR-04-SHELL-RC-APPEND",
+            "IN-CR-04-CRONTAB",
+            "IN-CR-04-CRON-D-WRITE",
+            "IN-CR-04-LAUNCHCTL",
+            "IN-CR-04-LAUNCH-AGENT-PLIST",
+            "IN-CR-04-SYSTEMCTL-ENABLE",
+            "IN-CR-04-SYSTEMD-UNIT-WRITE",
+            "IN-CR-04-FISH-CONFIG",
+            "IN-CR-04-LOGIN-ITEMS",
+            "IN-CR-05-EVM",
+            "IN-CR-05-SOLANA",
+            "IN-CR-05-BITCOIN",
+        ];
+        for id in in_cr_critical {
+            assert!(
+                is_fail_closed(id),
+                "Critical rule {id} must be in FAIL_CLOSED_RULES"
+            );
+        }
+    }
+
+    /// IN-CR-02 系列必须在 HOOK_RULES 中。
+    #[test]
+    fn in_cr_02_in_hook_rules() {
+        for id in [
+            "IN-CR-02",
+            "IN-CR-02-CURL-PIPE",
+            "IN-CR-02-WGET-PIPE",
+            "IN-CR-02-EVAL",
+            "IN-CR-02-NC-REVERSE",
+            "IN-CR-02-DD-WIPE",
+        ] {
+            assert!(is_hook_rule(id), "{id} must be in HOOK_RULES");
+            assert!(!is_gui_rule(id), "{id} must NOT be in GUI_RULES");
+        }
+    }
+
+    /// IN-CR-05 系列必须在 GUI_RULES 中。
+    #[test]
+    fn in_cr_05_in_gui_rules() {
+        for id in ["IN-CR-05-EVM", "IN-CR-05-SOLANA", "IN-CR-05-BITCOIN"] {
+            assert!(is_gui_rule(id), "{id} must be in GUI_RULES");
+            assert!(!is_hook_rule(id), "{id} must NOT be in HOOK_RULES");
+        }
+    }
 }
diff --git a/crates/sieve-rules/src/engine/mod.rs b/crates/sieve-rules/src/engine/mod.rs
index 99a3221..d69d59b 100644
--- a/crates/sieve-rules/src/engine/mod.rs
+++ b/crates/sieve-rules/src/engine/mod.rs
@@ -168,6 +168,9 @@ fn rule(id: &str, pattern: &str, severity: Severity) -> RuleEntry {
             keywords: vec![],
             allowlist_regexes: vec![],
             allowlist_stopwords: vec![],
+            disposition: None,
+            timeout_seconds: None,
+            default_on_timeout: crate::manifest::DefaultOnTimeout::Block,
         }
     }
 
diff --git a/crates/sieve-rules/src/manifest.rs b/crates/sieve-rules/src/manifest.rs
index 98dcddb..aaf9846 100644
--- a/crates/sieve-rules/src/manifest.rs
+++ b/crates/sieve-rules/src/manifest.rs
@@ -1,6 +1,4 @@
-//! 规则包 manifest（关联 ADR-002 / data-model.md）。
-//!
-//! 实际 manifest schema 在 Week 2 完整实现，Week 1 占位以验证 serde 可用。
+//! 规则包 manifest（关联 ADR-002 / data-model.md / PRD v1.4 §5.3 §5.4）。
 
 use serde::{Deserialize, Serialize};
 
@@ -42,6 +40,68 @@ pub struct RuleEntry {
     /// 允许放行的停用词列表（命中后检查，任一出现则不定级 Critical）。
     #[serde(default)]
     pub allowlist_stopwords: Vec<String>,
+    /// 处置形式（PRD v1.4 §5.4.1）。
+    ///
+    /// `None` 表示 TOML 未显式写，调用 [`RuleEntry::effective_disposition`] 获取
+    /// 按 severity 保守推断的值：Critical → [`Disposition::GuiPopup`]，
+    /// 其他 → [`Disposition::StatusBar`]。
+    #[serde(default)]
+    pub disposition: Option<Disposition>,
+    /// 等待 GUI/hook 决策的超时秒数（`None` = 不超时，适用于 AutoRedact / StatusBar）。
+    #[serde(default)]
+    pub timeout_seconds: Option<u32>,
+    /// 超时后的默认处置（PRD v1.4 §5.4.2）。
+    #[serde(default = "default_on_timeout_block")]
+    pub default_on_timeout: DefaultOnTimeout,
+}
+
+impl RuleEntry {
+    /// 返回规则的最终处置形式（PRD v1.4 §5.4.1）。
+    ///
+    /// TOML 未显式写 `disposition` 时，按 severity 保守推断：
+    /// - [`Severity::Critical`] → [`Disposition::GuiPopup`]
+    /// - 其他 → [`Disposition::StatusBar`]
+    pub fn effective_disposition(&self) -> Disposition {
+        self.disposition.unwrap_or(match self.severity {
+            Severity::Critical => Disposition::GuiPopup,
+            _ => Disposition::StatusBar,
+        })
+    }
+}
+
+/// 规则触发后的处置形式（PRD v1.4 §5.4.1 / ADR-016）。
+///
+/// 决定命中后产物如何到达用户：自动改写、GUI 弹窗、hook 拦截还是静默通知。
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum Disposition {
+    /// 自动脱敏改写 body bytes 后转发，不弹窗（OUT-01~05/12）。
+    AutoRedact,
+    /// hold 住 SSE 流，通过 IPC 通知 GUI 弹窗等待决策（IN-CR-01/05、IN-GEN-04、OUT-06~10）。
+    GuiPopup,
+    /// 不修改 SSE 流，写 IPC pending file，由 sieve-hook 在 PreToolUse 阶段拦截
+    /// （IN-CR-02~04、IN-GEN-01~03）。
+    HookTerminal,
+    /// 状态栏通知，不打断用户流程（OUT-11、IN-GEN-05）。
+    StatusBar,
+}
+
+/// 规则超时后的默认处置（PRD v1.4 §5.4.2）。
+///
+/// 当 GUI 弹窗或 hook 等待超过 `timeout_seconds` 后触发此动作。
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum DefaultOnTimeout {
+    /// 脱敏后发送（出站默认 fail-open 到脱敏）。
+    Redact,
+    /// 拒绝（入站默认 fail-closed）。
+    Block,
+    /// 允许通过（仅 IN-GEN Relaxed preset 用）。
+    Allow,
+}
+
+fn default_on_timeout_block() -> DefaultOnTimeout {
+    DefaultOnTimeout::Block
 }
 
 /// 严重等级。
@@ -125,4 +185,157 @@ fn action_serde() {
         let json = serde_json::to_string(&a).unwrap();
         assert_eq!(json, "\"block\"");
     }
+
+    // -------------------------------------------------------------------------
+    // PRD v1.4 §5.4 新字段测试
+    // -------------------------------------------------------------------------
+
+    /// 旧格式 TOML（无 disposition / timeout_seconds / default_on_timeout）
+    /// 必须能正常解析，不 break 现有规则文件。
+    #[test]
+    fn old_toml_without_disposition_parses_ok() {
+        let toml = r#"
+[[rules]]
+id = "OUT-01"
+description = "test"
+pattern = "secret"
+severity = "critical"
+action = "block"
+"#;
+        #[derive(serde::Deserialize)]
+        struct F {
+            rules: Vec<RuleEntry>,
+        }
+        let f: F = toml::from_str(toml).unwrap();
+        let r = &f.rules[0];
+        assert!(r.disposition.is_none());
+        assert!(r.timeout_seconds.is_none());
+        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
+    }
+
+    /// Critical 规则未写 disposition 时 effective_disposition → GuiPopup。
+    #[test]
+    fn effective_disposition_critical_defaults_to_gui_popup() {
+        let toml = r#"
+[[rules]]
+id = "IN-CR-02"
+description = "test"
+pattern = "rm"
+severity = "critical"
+action = "block"
+"#;
+        #[derive(serde::Deserialize)]
+        struct F {
+            rules: Vec<RuleEntry>,
+        }
+        let f: F = toml::from_str(toml).unwrap();
+        assert_eq!(
+            f.rules[0].effective_disposition(),
+            Disposition::GuiPopup,
+            "Critical without explicit disposition must default to GuiPopup"
+        );
+    }
+
+    /// 非 Critical 规则未写 disposition 时 effective_disposition → StatusBar。
+    #[test]
+    fn effective_disposition_non_critical_defaults_to_status_bar() {
+        let toml = r#"
+[[rules]]
+id = "IN-GEN-02"
+description = "test"
+pattern = "img"
+severity = "high"
+action = "warn"
+"#;
+        #[derive(serde::Deserialize)]
+        struct F {
+            rules: Vec<RuleEntry>,
+        }
+        let f: F = toml::from_str(toml).unwrap();
+        assert_eq!(
+            f.rules[0].effective_disposition(),
+            Disposition::StatusBar,
+            "Non-critical without explicit disposition must default to StatusBar"
+        );
+    }
+
+    /// 显式写了 disposition = "hook_terminal" 时必须正确解析。
+    #[test]
+    fn explicit_hook_terminal_disposition_parses() {
+        let toml = r#"
+[[rules]]
+id = "IN-CR-02"
+description = "test"
+pattern = "rm"
+severity = "critical"
+action = "block"
+disposition = "hook_terminal"
+timeout_seconds = 30
+default_on_timeout = "block"
+"#;
+        #[derive(serde::Deserialize)]
+        struct F {
+            rules: Vec<RuleEntry>,
+        }
+        let f: F = toml::from_str(toml).unwrap();
+        let r = &f.rules[0];
+        assert_eq!(r.effective_disposition(), Disposition::HookTerminal);
+        assert_eq!(r.timeout_seconds, Some(30));
+        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
+    }
+
+    /// disposition = "auto_redact" + default_on_timeout = "redact" 正确解析。
+    #[test]
+    fn auto_redact_disposition_parses() {
+        let toml = r#"
+[[rules]]
+id = "OUT-01"
+description = "test"
+pattern = "sk-ant"
+severity = "critical"
+action = "block"
+disposition = "auto_redact"
+default_on_timeout = "redact"
+"#;
+        #[derive(serde::Deserialize)]
+        struct F {
+            rules: Vec<RuleEntry>,
+        }
+        let f: F = toml::from_str(toml).unwrap();
+        let r = &f.rules[0];
+        assert_eq!(r.effective_disposition(), Disposition::AutoRedact);
+        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Redact);
+        assert!(r.timeout_seconds.is_none());
+    }
+
+    /// Disposition 枚举 serde snake_case 正确。
+    #[test]
+    fn disposition_serde_roundtrip() {
+        for (d, expected) in [
+            (Disposition::AutoRedact, "\"auto_redact\""),
+            (Disposition::GuiPopup, "\"gui_popup\""),
+            (Disposition::HookTerminal, "\"hook_terminal\""),
+            (Disposition::StatusBar, "\"status_bar\""),
+        ] {
+            let json = serde_json::to_string(&d).unwrap();
+            assert_eq!(json, expected);
+            let back: Disposition = serde_json::from_str(&json).unwrap();
+            assert_eq!(back, d);
+        }
+    }
+
+    /// DefaultOnTimeout 枚举 serde snake_case 正确。
+    #[test]
+    fn default_on_timeout_serde_roundtrip() {
+        for (d, expected) in [
+            (DefaultOnTimeout::Redact, "\"redact\""),
+            (DefaultOnTimeout::Block, "\"block\""),
+            (DefaultOnTimeout::Allow, "\"allow\""),
+        ] {
+            let json = serde_json::to_string(&d).unwrap();
+            assert_eq!(json, expected);
+            let back: DefaultOnTimeout = serde_json::from_str(&json).unwrap();
+            assert_eq!(back, d);
+        }
+    }
 }

exec
/bin/zsh -lc 'grep -n "''^id = '"\\\"OUT\" -A8 crates/sieve-rules/rules/outbound.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
14:id = "OUT-01"
15-description = "Anthropic API key (sk-ant-api03-...)"
16-pattern = 'sk-ant-api03-[a-zA-Z0-9_\-]{93}AA'
17-severity = "critical"
18-action = "block"
19-entropy_min = 4.5
20-keywords = ["sk-ant-api03"]
21-allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
22-allowlist_stopwords = []
--
33:id = "OUT-02"
34-description = "OpenAI API key (sk-... legacy + sk-proj-/sk-svcacct-/sk-admin-)"
35-pattern = 'sk-(?:proj|svcacct|admin)-[A-Za-z0-9_\-]{58,65}T3BlbkFJ[A-Za-z0-9_\-]{58,65}|sk-[a-zA-Z0-9]{20}T3BlbkFJ[a-zA-Z0-9]{20}'
36-severity = "critical"
37-action = "block"
38-entropy_min = 4.5
39-keywords = ["T3BlbkFJ"]
40-allowlist_regexes = []
41-allowlist_stopwords = []
--
50:id = "OUT-03"
51-description = "AWS Access Key ID (AKIA / ASIA / ABIA / ACCA / A3T)"
52-pattern = '(?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16}'
53-severity = "critical"
54-action = "block"
55-entropy_min = 3.0
56-keywords = ["AKIA", "ASIA", "ABIA", "ACCA"]
57-allowlist_regexes = []
58-allowlist_stopwords = ["AKIAIOSFODNN7EXAMPLE"]  # AWS 官方文档示例 key
--
67:id = "OUT-04"
68-description = "GitHub PAT (ghp_/gho_/ghu_/ghs_/ghr_)"
69-pattern = 'gh[pousr]_[0-9a-zA-Z]{36}'
70-severity = "critical"
71-action = "block"
72-entropy_min = 4.0
73-keywords = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
74-allowlist_regexes = []
75-allowlist_stopwords = []
--
84:id = "OUT-05"
85-description = "Google Cloud API Key (AIza...)"
86-pattern = 'AIza[0-9A-Za-z_\-]{35}'
87-severity = "high"
88-action = "block"
89-entropy_min = 4.0
90-keywords = ["AIza"]
91-allowlist_regexes = []
92-allowlist_stopwords = []
--
102:id = "OUT-06"
103-description = "JWT Token (eyJ...)"
104-pattern = 'ey[A-Za-z0-9_\-]{16,}\.ey[A-Za-z0-9_\/\-]{16,}\.[A-Za-z0-9_\/\-]{10,}'
105-severity = "high"
106-action = "block"
107-entropy_min = 3.5
108-keywords = ["eyJ"]
109-allowlist_regexes = []
110-allowlist_stopwords = []
--
121:id = "OUT-07"
122-description = "PEM Private Key (RSA / EC / DSA / generic)"
123-pattern = '-----BEGIN[ A-Z0-9_\-]{0,60}PRIVATE KEY[ A-Z]{0,20}-----'
124-severity = "critical"
125-action = "block"
126-entropy_min = 0.0
127-keywords = ["-----BEGIN"]
128-allowlist_regexes = []
129-allowlist_stopwords = []
--
139:id = "OUT-08"
140-description = "Stripe Live Key (sk_live_/pk_live_/rk_live_)"
141-pattern = '(?:sk|pk|rk)_live_[a-zA-Z0-9]{10,99}'
142-severity = "critical"
143-action = "block"
144-entropy_min = 3.5
145-keywords = ["_live_"]
146-allowlist_regexes = ['(?i)test|example']
147-allowlist_stopwords = []
--
157:id = "OUT-09"
158-description = "Slack Token (xoxb-/xoxp-/xoxa-/xoxs-)"
159-pattern = 'xox[bpas]\-[0-9A-Za-z\-]{10,}'
160-severity = "high"
161-action = "block"
162-entropy_min = 3.0
163-keywords = ["xoxb", "xoxp", "xoxa", "xoxs"]
164-allowlist_regexes = []
165-allowlist_stopwords = []
--
175:id = "OUT-10"
176-description = "OpenSSH Private Key (-----BEGIN OPENSSH PRIVATE KEY-----)"
177-pattern = '-----BEGIN OPENSSH PRIVATE KEY-----'
178-severity = "critical"
179-action = "block"
180-entropy_min = 0.0
181-keywords = ["BEGIN OPENSSH"]
182-allowlist_regexes = []
183-allowlist_stopwords = []
--
194:id = "OUT-11"
195-description = "Discord Bot Token"
196-pattern = '[A-Za-z0-9_\-]{24,28}\.[A-Za-z0-9_\-]{6}\.[A-Za-z0-9_\-]{27,38}'
197-severity = "high"
198-action = "block"
199-entropy_min = 3.5
200-keywords = ["."]
201-allowlist_regexes = []
202-allowlist_stopwords = []

exec
/bin/zsh -lc "for f in crates/sieve-ipc/src/*.rs crates/sieve-hook/src/*.rs; do echo '---' "'$f; nl -ba $f | sed -n '"'1,260p'; done" in /Users/doskey/src/sieve
 succeeded in 0ms:
--- crates/sieve-ipc/src/decision_file.rs
     1	use std::path::{Path, PathBuf};
     2	use std::time::Duration;
     3	
     4	use chrono::Utc;
     5	use fd_lock::RwLock;
     6	use uuid::Uuid;
     7	
     8	use crate::{
     9	    error::IpcError,
    10	    paths::{decisions_dir, ensure_dirs, locks_dir},
    11	    protocol::{DecisionAction, DecisionResponse},
    12	};
    13	
    14	/// 将 [`DecisionResponse`] 写入 `<base>/decisions/<request_id>.json`。
    15	///
    16	/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁，确保并发写入安全
    17	///（hook 与 GUI 极少同时操作同一 request_id，但防御性加锁是正确做法）。
    18	///
    19	/// 关联：SPEC-001 §3.3（决策文件写入规约）。
    20	pub fn write_decision(resp: &DecisionResponse, base: &Path) -> Result<PathBuf, IpcError> {
    21	    ensure_dirs(base)?;
    22	    let lock_path = locks_dir(base).join(format!("{}.lock", resp.request_id));
    23	    let dec_path = decisions_dir(base).join(format!("{}.json", resp.request_id));
    24	
    25	    // 创建锁文件（若不存在），然后加独占写锁。
    26	    let lock_file = std::fs::OpenOptions::new()
    27	        .write(true)
    28	        .create(true)
    29	        .truncate(false)
    30	        .open(&lock_path)?;
    31	
    32	    let mut lock = RwLock::new(lock_file);
    33	    {
    34	        let _guard = lock
    35	            .write()
    36	            .map_err(|e| IpcError::FileLock(e.to_string()))?;
    37	
    38	        let json = serde_json::to_string_pretty(resp)?;
    39	        std::fs::write(&dec_path, json.as_bytes())?;
    40	    }
    41	
    42	    Ok(dec_path)
    43	}
    44	
    45	/// 轮询等待 `<base>/decisions/<request_id>.json` 出现并读取。
    46	///
    47	/// 轮询间隔 50 ms，对 30–120 s 的用户响应超时来说 CPU 开销可忽略。
    48	/// 选择轮询而非 inotify/notify 是为了跨平台简单性；Phase 1 仅 macOS，
    49	/// 但未来 Linux 支持时轮询同样生效，不需要额外适配。
    50	///
    51	/// 超时后按 `default_on_timeout` 构造兜底响应。关联：ADR-013 §4.2。
    52	pub async fn wait_for_decision(
    53	    request_id: Uuid,
    54	    base: &Path,
    55	    timeout: Duration,
    56	    default_on_timeout: crate::protocol::DefaultOnTimeout,
    57	) -> Result<DecisionResponse, IpcError> {
    58	    let path = decisions_dir(base).join(format!("{request_id}.json"));
    59	    let deadline = tokio::time::Instant::now() + timeout;
    60	    let poll_interval = Duration::from_millis(50);
    61	
    62	    loop {
    63	        if path.exists() {
    64	            let content = tokio::fs::read_to_string(&path).await?;
    65	            let resp: DecisionResponse = serde_json::from_str(&content)?;
    66	            return Ok(resp);
    67	        }
    68	
    69	        if tokio::time::Instant::now() >= deadline {
    70	            // 超时：按 default_on_timeout 构造兜底响应。
    71	            let action = match default_on_timeout {
    72	                crate::protocol::DefaultOnTimeout::Block => DecisionAction::Deny,
    73	                crate::protocol::DefaultOnTimeout::Allow => DecisionAction::Allow,
    74	                crate::protocol::DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
    75	            };
    76	            return Ok(DecisionResponse {
    77	                request_id,
    78	                decision: action,
    79	                decided_at: Utc::now(),
    80	                by_user: false,
    81	                remember: false,
    82	            });
    83	        }
    84	
    85	        tokio::time::sleep(poll_interval).await;
    86	    }
    87	}
    88	
    89	/// 同步版读取决策文件（hook 侧使用，不依赖 tokio）。
    90	pub fn read_decision(request_id: Uuid, base: &Path) -> Result<DecisionResponse, IpcError> {
    91	    let path = decisions_dir(base).join(format!("{request_id}.json"));
    92	    let content = std::fs::read_to_string(&path).map_err(|e| {
    93	        if e.kind() == std::io::ErrorKind::NotFound {
    94	            // 不存在时通过 IpcError::PendingNotFound 复用（语义相近）
    95	            IpcError::PendingNotFound { request_id }
    96	        } else {
    97	            IpcError::Socket(e)
    98	        }
    99	    })?;
   100	    let resp: DecisionResponse = serde_json::from_str(&content)?;
   101	    Ok(resp)
   102	}
--- crates/sieve-ipc/src/error.rs
     1	use thiserror::Error;
     2	
     3	/// IPC 层错误枚举。
     4	///
     5	/// 关联规格：ADR-013（IPC 协议）、SPEC-001（sieve-hook 文件协议）。
     6	#[derive(Debug, Error)]
     7	pub enum IpcError {
     8	    /// Unix socket 绑定或连接失败。
     9	    #[error("socket error: {0}")]
    10	    Socket(#[from] std::io::Error),
    11	
    12	    /// JSON 序列化 / 反序列化失败。
    13	    #[error("json error: {0}")]
    14	    Json(#[from] serde_json::Error),
    15	
    16	    /// 请求在规定超时内未收到决策响应。
    17	    #[error("decision timeout for request {request_id}")]
    18	    Timeout { request_id: uuid::Uuid },
    19	
    20	    /// pending 文件已超过 stale 阈值（10 分钟），视为过期拒绝。
    21	    ///
    22	    /// fail-closed：过期请求不允许放行，防止残留文件被重放。
    23	    #[error("pending file is stale (created_at too old) for request {request_id}")]
    24	    StalePending { request_id: uuid::Uuid },
    25	
    26	    /// pending 文件不存在——此请求未经代理标记，可 fail-open。
    27	    #[error("pending file not found for request {request_id}")]
    28	    PendingNotFound { request_id: uuid::Uuid },
    29	
    30	    /// 文件加锁失败。
    31	    #[error("file lock error: {0}")]
    32	    FileLock(String),
    33	
    34	    /// $HOME 环境变量缺失，无法确定 sieve_home 路径。
    35	    #[error("$HOME environment variable is not set")]
    36	    HomeNotFound,
    37	
    38	    /// JSON-RPC 响应中携带了错误对象。
    39	    #[error("json-rpc error {code}: {message}")]
    40	    JsonRpcError { code: i64, message: String },
    41	
    42	    /// 对端发送了无法识别的 JSON-RPC method 或响应格式异常。
    43	    #[error("unexpected json-rpc response: {0}")]
    44	    UnexpectedResponse(String),
    45	}
--- crates/sieve-ipc/src/lib.rs
     1	// sieve-ipc: JSON-RPC 2.0 over Unix socket + pending/decision 文件协议库。
     2	//
     3	// 供 sieve-cli（主代理）调用，向 GUI（sieve-gui-macos）或 hook（sieve-hook）
     4	// 传递决策请求并等待响应。关联：ADR-013（IPC 协议）、ADR-014（双层防御）。
     5	
     6	pub mod decision_file;
     7	pub mod error;
     8	pub mod paths;
     9	pub mod pending_file;
    10	pub mod protocol;
    11	pub mod socket_client;
    12	pub mod socket_server;
    13	
    14	// 常用类型直接 re-export，调用方无需深层 import。
    15	pub use error::IpcError;
    16	pub use protocol::{
    17	    DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout, DetectionPayload,
    18	    Disposition, Severity,
    19	};
    20	pub use socket_server::IpcServer;
    21	
    22	#[cfg(test)]
    23	mod tests {
    24	    use chrono::Utc;
    25	    use uuid::Uuid;
    26	
    27	    use super::protocol::*;
    28	
    29	    // ── 协议 round-trip ──────────────────────────────────────────────────────
    30	
    31	    #[test]
    32	    fn decision_request_round_trip() {
    33	        let req = DecisionRequest {
    34	            request_id: Uuid::now_v7(),
    35	            created_at: Utc::now(),
    36	            timeout_seconds: 60,
    37	            default_on_timeout: DefaultOnTimeout::Block,
    38	            detections: vec![DetectionPayload {
    39	                rule_id: "IN-CR-01".to_owned(),
    40	                severity: Severity::Critical,
    41	                disposition: Disposition::HookTerminal,
    42	                title: "私钥检测".to_owned(),
    43	                one_line_summary: "检测到 BIP39 助记词（12 词，checksum 通过）".to_owned(),
    44	                details: serde_json::json!({ "word_count": 12 }),
    45	            }],
    46	        };
    47	
    48	        let json = serde_json::to_string(&req).expect("serialize");
    49	        let decoded: DecisionRequest = serde_json::from_str(&json).expect("deserialize");
    50	        assert_eq!(decoded.request_id, req.request_id);
    51	        assert_eq!(decoded.detections[0].rule_id, "IN-CR-01");
    52	        assert_eq!(decoded.default_on_timeout, DefaultOnTimeout::Block);
    53	    }
    54	
    55	    #[test]
    56	    fn decision_response_round_trip() {
    57	        let resp = DecisionResponse {
    58	            request_id: Uuid::now_v7(),
    59	            decision: DecisionAction::Deny,
    60	            decided_at: Utc::now(),
    61	            by_user: true,
    62	            remember: false,
    63	        };
    64	
    65	        let json = serde_json::to_string(&resp).expect("serialize");
    66	        let decoded: DecisionResponse = serde_json::from_str(&json).expect("deserialize");
    67	        assert_eq!(decoded.request_id, resp.request_id);
    68	        assert_eq!(decoded.decision, DecisionAction::Deny);
    69	        assert!(decoded.by_user);
    70	        assert!(!decoded.remember);
    71	    }
    72	
    73	    #[test]
    74	    fn disposition_serde_snake_case() {
    75	        assert_eq!(
    76	            serde_json::to_string(&Disposition::GuiPopup).unwrap(),
    77	            "\"gui_popup\""
    78	        );
    79	        assert_eq!(
    80	            serde_json::to_string(&Disposition::HookTerminal).unwrap(),
    81	            "\"hook_terminal\""
    82	        );
    83	    }
    84	
    85	    #[test]
    86	    fn severity_serde_snake_case() {
    87	        assert_eq!(
    88	            serde_json::to_string(&Severity::Critical).unwrap(),
    89	            "\"critical\""
    90	        );
    91	    }
    92	
    93	    #[test]
    94	    fn decision_action_serde_snake_case() {
    95	        assert_eq!(
    96	            serde_json::to_string(&DecisionAction::RedactAndAllow).unwrap(),
    97	            "\"redact_and_allow\""
    98	        );
    99	    }
   100	
   101	    // ── jsonrpc envelope ────────────────────────────────────────────────────
   102	
   103	    #[test]
   104	    fn jsonrpc_request_omits_null_id() {
   105	        let req = jsonrpc::Request {
   106	            jsonrpc: "2.0".to_owned(),
   107	            method: "ping".to_owned(),
   108	            params: None,
   109	            id: None,
   110	        };
   111	        let json = serde_json::to_string(&req).unwrap();
   112	        // 通知请求不携带 id 字段。
   113	        assert!(!json.contains("\"id\""));
   114	    }
   115	
   116	    #[test]
   117	    fn jsonrpc_call_includes_id() {
   118	        let req = jsonrpc::Request::call(
   119	            "request_decision",
   120	            serde_json::json!({}),
   121	            serde_json::Value::String("abc".to_owned()),
   122	        );
   123	        let json = serde_json::to_string(&req).unwrap();
   124	        assert!(json.contains("\"id\""));
   125	        assert!(json.contains("\"request_decision\""));
   126	    }
   127	}
   128	
   129	#[cfg(test)]
   130	mod file_tests {
   131	    use chrono::Utc;
   132	    use std::time::Duration;
   133	    use uuid::Uuid;
   134	
   135	    use super::{
   136	        decision_file::{wait_for_decision, write_decision},
   137	        pending_file::{read_pending, write_pending},
   138	        protocol::*,
   139	    };
   140	
   141	    fn make_request(id: Uuid) -> DecisionRequest {
   142	        DecisionRequest {
   143	            request_id: id,
   144	            created_at: Utc::now(),
   145	            timeout_seconds: 60,
   146	            default_on_timeout: DefaultOnTimeout::Block,
   147	            detections: vec![],
   148	        }
   149	    }
   150	
   151	    // ── pending_file ─────────────────────────────────────────────────────────
   152	
   153	    #[test]
   154	    fn pending_write_and_read() {
   155	        let tmp = tempfile::tempdir().unwrap();
   156	        let id = Uuid::now_v7();
   157	        let req = make_request(id);
   158	
   159	        let path = write_pending(&req, tmp.path()).unwrap();
   160	        assert!(path.exists());
   161	
   162	        let read_back = read_pending(id, tmp.path()).unwrap();
   163	        assert_eq!(read_back.request_id, id);
   164	    }
   165	
   166	    #[test]
   167	    fn pending_not_found_error() {
   168	        let tmp = tempfile::tempdir().unwrap();
   169	        let id = Uuid::now_v7();
   170	        let err = read_pending(id, tmp.path()).unwrap_err();
   171	        assert!(matches!(err, crate::IpcError::PendingNotFound { .. }));
   172	    }
   173	
   174	    #[test]
   175	    fn pending_file_lock_two_tasks() {
   176	        // 两个线程抢同一个 pending 文件——后者等前者释放锁后写入。
   177	        // 验证不出现数据损坏（最终文件可被正确解析）。
   178	        use std::sync::Arc;
   179	        use std::sync::Barrier;
   180	        use std::thread;
   181	
   182	        let tmp = tempfile::tempdir().unwrap();
   183	        let base = Arc::new(tmp.path().to_owned());
   184	        let id = Uuid::now_v7();
   185	        let barrier = Arc::new(Barrier::new(2));
   186	
   187	        let base1 = Arc::clone(&base);
   188	        let barrier1 = Arc::clone(&barrier);
   189	        let t1 = thread::spawn(move || {
   190	            barrier1.wait();
   191	            let req = make_request(id);
   192	            write_pending(&req, &base1).unwrap();
   193	        });
   194	
   195	        let base2 = Arc::clone(&base);
   196	        let barrier2 = Arc::clone(&barrier);
   197	        let t2 = thread::spawn(move || {
   198	            barrier2.wait();
   199	            let req = make_request(id);
   200	            write_pending(&req, &base2).unwrap();
   201	        });
   202	
   203	        t1.join().unwrap();
   204	        t2.join().unwrap();
   205	
   206	        // 文件仍可被正确解析（两次写入串行化）。
   207	        let read_back = read_pending(id, &base).unwrap();
   208	        assert_eq!(read_back.request_id, id);
   209	    }
   210	
   211	    // ── decision_file ────────────────────────────────────────────────────────
   212	
   213	    #[test]
   214	    fn decision_write_and_read() {
   215	        let tmp = tempfile::tempdir().unwrap();
   216	        let id = Uuid::now_v7();
   217	        let resp = DecisionResponse {
   218	            request_id: id,
   219	            decision: DecisionAction::Allow,
   220	            decided_at: Utc::now(),
   221	            by_user: true,
   222	            remember: false,
   223	        };
   224	
   225	        let path = write_decision(&resp, tmp.path()).unwrap();
   226	        assert!(path.exists());
   227	
   228	        let read_back = super::decision_file::read_decision(id, tmp.path()).unwrap();
   229	        assert_eq!(read_back.request_id, id);
   230	        assert_eq!(read_back.decision, DecisionAction::Allow);
   231	    }
   232	
   233	    #[tokio::test]
   234	    async fn wait_for_decision_timeout_fallback() {
   235	        let tmp = tempfile::tempdir().unwrap();
   236	        let id = Uuid::now_v7();
   237	        // 极短超时，不写决策文件，应返回 Block（default_on_timeout = Block）。
   238	        let resp = wait_for_decision(
   239	            id,
   240	            tmp.path(),
   241	            Duration::from_millis(100),
   242	            DefaultOnTimeout::Block,
   243	        )
   244	        .await
   245	        .unwrap();
   246	        assert_eq!(resp.decision, DecisionAction::Deny);
   247	        assert!(!resp.by_user);
   248	    }
   249	
   250	    #[tokio::test]
   251	    async fn wait_for_decision_found() {
   252	        let tmp = tempfile::tempdir().unwrap();
   253	        let id = Uuid::now_v7();
   254	        let base = tmp.path().to_owned();
   255	
   256	        // 50ms 后写决策文件，模拟用户操作。
   257	        let base_clone = base.clone();
   258	        tokio::spawn(async move {
   259	            tokio::time::sleep(Duration::from_millis(50)).await;
   260	            let resp = DecisionResponse {
--- crates/sieve-ipc/src/paths.rs
     1	use std::path::PathBuf;
     2	
     3	use crate::error::IpcError;
     4	
     5	/// 计算 sieve home 目录。
     6	///
     7	/// 优先级：`$SIEVE_HOME` 环境变量 > `$HOME/.sieve`。
     8	/// $HOME 缺失时返回 [`IpcError::HomeNotFound`]。
     9	///
    10	/// 关联：SPEC-001 §2.1（目录结构）。
    11	pub fn sieve_home() -> Result<PathBuf, IpcError> {
    12	    if let Ok(val) = std::env::var("SIEVE_HOME") {
    13	        return Ok(PathBuf::from(val));
    14	    }
    15	    let home = std::env::var("HOME").map_err(|_| IpcError::HomeNotFound)?;
    16	    Ok(PathBuf::from(home).join(".sieve"))
    17	}
    18	
    19	/// `<sieve_home>/pending/` 目录，存放主代理写入的待决策文件。
    20	pub fn pending_dir(base: &std::path::Path) -> PathBuf {
    21	    base.join("pending")
    22	}
    23	
    24	/// `<sieve_home>/decisions/` 目录，存放 hook/GUI 写入的决策文件。
    25	pub fn decisions_dir(base: &std::path::Path) -> PathBuf {
    26	    base.join("decisions")
    27	}
    28	
    29	/// `<sieve_home>/locks/` 目录，存放文件锁占位符。
    30	pub fn locks_dir(base: &std::path::Path) -> PathBuf {
    31	    base.join("locks")
    32	}
    33	
    34	/// `<sieve_home>/ipc.sock` Unix socket 路径（主代理监听，GUI 连接）。
    35	pub fn ipc_socket_path(base: &std::path::Path) -> PathBuf {
    36	    base.join("ipc.sock")
    37	}
    38	
    39	/// 确保所有子目录存在，不存在时递归创建。
    40	///
    41	/// 幂等——多次调用安全。
    42	pub fn ensure_dirs(base: &std::path::Path) -> Result<(), IpcError> {
    43	    for dir in [pending_dir(base), decisions_dir(base), locks_dir(base)] {
    44	        std::fs::create_dir_all(&dir)?;
    45	    }
    46	    Ok(())
    47	}
--- crates/sieve-ipc/src/pending_file.rs
     1	use std::path::{Path, PathBuf};
     2	
     3	use fd_lock::RwLock;
     4	use uuid::Uuid;
     5	
     6	use crate::{
     7	    error::IpcError,
     8	    paths::{ensure_dirs, pending_dir},
     9	    protocol::DecisionRequest,
    10	};
    11	
    12	/// 将 [`DecisionRequest`] 写入 `<base>/pending/<request_id>.json`。
    13	///
    14	/// 写入前用 fd-lock 对目标文件加独占写锁，防止并发写入同一 request_id（极少见
    15	/// 但理论可行）。文件以 pretty JSON 格式写入，方便调试和 hook 侧直接读取。
    16	///
    17	/// 关联：SPEC-001 §3.1（pending 文件写入规约）、ADR-013 §4（文件协议备用路径）。
    18	pub fn write_pending(req: &DecisionRequest, base: &Path) -> Result<PathBuf, IpcError> {
    19	    ensure_dirs(base)?;
    20	    let dir = pending_dir(base);
    21	    let path = dir.join(format!("{}.json", req.request_id));
    22	
    23	    // 打开（或创建）文件，然后加独占写锁再写内容。
    24	    // 使用 std::fs::OpenOptions 而非 std::fs::write，以便 fd-lock 持有文件描述符。
    25	    let file = std::fs::OpenOptions::new()
    26	        .write(true)
    27	        .create(true)
    28	        .truncate(true)
    29	        .open(&path)?;
    30	
    31	    let mut lock = RwLock::new(file);
    32	    {
    33	        let mut guard = lock
    34	            .write()
    35	            .map_err(|e| IpcError::FileLock(e.to_string()))?;
    36	        let json = serde_json::to_string_pretty(req)?;
    37	        use std::io::Write;
    38	        guard.write_all(json.as_bytes())?;
    39	    }
    40	
    41	    Ok(path)
    42	}
    43	
    44	/// 读取并解析 `<base>/pending/<request_id>.json`。
    45	///
    46	/// 返回：
    47	/// - `Ok(DecisionRequest)` 成功
    48	/// - `Err(IpcError::PendingNotFound)` 文件不存在
    49	/// - `Err(IpcError::Json)` 解析失败
    50	pub fn read_pending(request_id: Uuid, base: &Path) -> Result<DecisionRequest, IpcError> {
    51	    let path = pending_dir(base).join(format!("{request_id}.json"));
    52	    let content = std::fs::read_to_string(&path).map_err(|e| {
    53	        if e.kind() == std::io::ErrorKind::NotFound {
    54	            IpcError::PendingNotFound { request_id }
    55	        } else {
    56	            IpcError::Socket(e)
    57	        }
    58	    })?;
    59	    let req: DecisionRequest = serde_json::from_str(&content)?;
    60	    Ok(req)
    61	}
--- crates/sieve-ipc/src/protocol.rs
     1	use chrono::{DateTime, Utc};
     2	use serde::{Deserialize, Serialize};
     3	use uuid::Uuid;
     4	
     5	// ── Enums ────────────────────────────────────────────────────────────────────
     6	
     7	/// 检测结果的最终处置方式。
     8	///
     9	/// 与 sieve-rules 中的处置枚举镜像，IPC 层独立定义以避免循环依赖。
    10	/// 关联：ADR-014（双层防御）、SPEC-001。
    11	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    12	#[serde(rename_all = "snake_case")]
    13	pub enum Disposition {
    14	    /// 自动脱敏——出站阶段替换敏感内容后放行，无需人工确认。
    15	    AutoRedact,
    16	    /// 弹出 GUI 窗口（sieve-gui-macos）请求用户确认。
    17	    GuiPopup,
    18	    /// 调用 PreToolUse hook（sieve-hook 二进制）在 TTY 请求用户确认。
    19	    HookTerminal,
    20	    /// 在状态栏静默提示，不打断流程。
    21	    StatusBar,
    22	}
    23	
    24	/// 超时后的默认决策。
    25	///
    26	/// Critical 规则强制使用 Block，不允许下游覆盖。关联：ADR-014 §fail-closed。
    27	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    28	#[serde(rename_all = "snake_case")]
    29	pub enum DefaultOnTimeout {
    30	    /// 脱敏后放行（适用于 AutoRedact 类型的超时回退）。
    31	    Redact,
    32	    /// 阻断——fail-closed，Critical 规则的强制回退策略。
    33	    Block,
    34	    /// 放行——仅适用于低优先级通知类规则。
    35	    Allow,
    36	}
    37	
    38	/// 检测命中的严重等级。
    39	///
    40	/// 关联：PRD §4 检测项分级、ADR-014。
    41	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    42	#[serde(rename_all = "snake_case")]
    43	pub enum Severity {
    44	    /// 最高级：签名、转账、部署等不可逆动作，强制人工确认，不可关闭。
    45	    Critical,
    46	    /// 高危：可逆但高风险操作。
    47	    High,
    48	    /// 中等：潜在风险，默认提示但可配置。
    49	    Medium,
    50	    /// 低危：信息提示。
    51	    Low,
    52	}
    53	
    54	// ── Detection payload ────────────────────────────────────────────────────────
    55	
    56	/// 单条检测命中的 IPC 表示。
    57	///
    58	/// 去掉规则匹配内部细节（正则 / offset），只保留 GUI/hook 渲染所需字段。
    59	/// 关联：SPEC-001 §3.2、SPEC-002 §2.1。
    60	#[derive(Debug, Clone, Serialize, Deserialize)]
    61	pub struct DetectionPayload {
    62	    /// 规则 ID，例如 `IN-CR-01`。用于 hook 终端显示和日志关联。
    63	    pub rule_id: String,
    64	    /// 严重等级。
    65	    pub severity: Severity,
    66	    /// 处置方式。
    67	    pub disposition: Disposition,
    68	    /// 简短标题，在 GUI 标题栏或 hook 首行显示。
    69	    pub title: String,
    70	    /// 单行摘要，不超过 120 字符，用于 hook 终端和通知消息。
    71	    pub one_line_summary: String,
    72	    /// 扩展详情，结构由各规则自定义（GUI 侧渲染详细视图用）。
    73	    pub details: serde_json::Value,
    74	}
    75	
    76	// ── Request / Response ───────────────────────────────────────────────────────
    77	
    78	/// 主代理 → GUI / Hook 的决策请求。
    79	///
    80	/// JSON-RPC 2.0 method = `"request_decision"`，通过 Unix socket 或 pending
    81	/// 文件协议传输。关联：ADR-013 §3、SPEC-001 §3.1。
    82	#[derive(Debug, Clone, Serialize, Deserialize)]
    83	pub struct DecisionRequest {
    84	    /// 全局唯一请求 ID（UUIDv7，含时间戳，便于排序和 stale 检测）。
    85	    pub request_id: Uuid,
    86	    /// 请求创建时间（UTC）。hook 侧用于 stale 检测（> 10 分钟视为过期）。
    87	    pub created_at: DateTime<Utc>,
    88	    /// 用户响应超时时长（秒）。范围 30–120，由规则配置决定。
    89	    pub timeout_seconds: u32,
    90	    /// 超时后的默认决策。Critical 规则此字段服务端强制为 `Block`。
    91	    pub default_on_timeout: DefaultOnTimeout,
    92	    /// 本次请求触发的所有检测命中列表（可多条）。
    93	    pub detections: Vec<DetectionPayload>,
    94	}
    95	
    96	/// 用户或超时产生的决策动作。
    97	///
    98	/// 关联：SPEC-001 §3.3、ADR-014 §决策流程。
    99	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   100	#[serde(rename_all = "snake_case")]
   101	pub enum DecisionAction {
   102	    /// 用户允许：GUI 类继续转发原始 SSE，Hook 类返回 exit 0。
   103	    Allow,
   104	    /// 用户拒绝：GUI 类截流注入 `sieve_blocked` event，Hook 类返回 exit 1。
   105	    Deny,
   106	    /// 仅出站脱敏类：按规则 redact 占位符替换后转发。
   107	    RedactAndAllow,
   108	}
   109	
   110	/// GUI / Hook → 主代理的决策响应。
   111	///
   112	/// 写入 `<sieve_home>/decisions/<request_id>.json` 或通过 socket 返回。
   113	/// 关联：ADR-013 §3.4、SPEC-001 §3.3。
   114	#[derive(Debug, Clone, Serialize, Deserialize)]
   115	pub struct DecisionResponse {
   116	    /// 对应的请求 ID，用于主代理侧匹配 oneshot channel。
   117	    pub request_id: Uuid,
   118	    /// 决策动作。
   119	    pub decision: DecisionAction,
   120	    /// 决策时间（UTC）。
   121	    pub decided_at: DateTime<Utc>,
   122	    /// `true` 表示用户主动操作，`false` 表示超时默认。
   123	    pub by_user: bool,
   124	    /// 是否记住此次决策（同规则 + 同 tool 不再询问）。
   125	    ///
   126	    /// Critical severity 的决策此字段服务端强制写 `false`，即使用户请求记住也拒绝。
   127	    pub remember: bool,
   128	}
   129	
   130	// ── JSON-RPC 2.0 envelope ────────────────────────────────────────────────────
   131	
   132	/// JSON-RPC 2.0 协议封装。
   133	///
   134	/// 手写实现以避免引入大型 jsonrpc crate 依赖。关联：ADR-013 §2（传输协议选型）。
   135	pub mod jsonrpc {
   136	    use serde::{Deserialize, Serialize};
   137	    use serde_json::Value;
   138	
   139	    /// JSON-RPC 2.0 请求（通知或有 id 的调用）。
   140	    #[derive(Debug, Clone, Serialize, Deserialize)]
   141	    pub struct Request {
   142	        pub jsonrpc: String,
   143	        pub method: String,
   144	        #[serde(skip_serializing_if = "Option::is_none")]
   145	        pub params: Option<Value>,
   146	        #[serde(skip_serializing_if = "Option::is_none")]
   147	        pub id: Option<Value>,
   148	    }
   149	
   150	    impl Request {
   151	        /// 构造一个有 id 的调用请求。
   152	        pub fn call(method: impl Into<String>, params: Value, id: Value) -> Self {
   153	            Self {
   154	                jsonrpc: "2.0".to_owned(),
   155	                method: method.into(),
   156	                params: Some(params),
   157	                id: Some(id),
   158	            }
   159	        }
   160	    }
   161	
   162	    /// JSON-RPC 2.0 成功响应。
   163	    #[derive(Debug, Clone, Serialize, Deserialize)]
   164	    pub struct Response {
   165	        pub jsonrpc: String,
   166	        #[serde(skip_serializing_if = "Option::is_none")]
   167	        pub result: Option<Value>,
   168	        #[serde(skip_serializing_if = "Option::is_none")]
   169	        pub error: Option<ErrorObject>,
   170	        pub id: Value,
   171	    }
   172	
   173	    /// JSON-RPC 2.0 错误对象。
   174	    #[derive(Debug, Clone, Serialize, Deserialize)]
   175	    pub struct ErrorObject {
   176	        pub code: i64,
   177	        pub message: String,
   178	        #[serde(skip_serializing_if = "Option::is_none")]
   179	        pub data: Option<Value>,
   180	    }
   181	}
--- crates/sieve-ipc/src/socket_client.rs
     1	use std::path::Path;
     2	
     3	use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
     4	use tokio::net::UnixStream;
     5	use uuid::Uuid;
     6	
     7	use crate::{
     8	    error::IpcError,
     9	    protocol::{jsonrpc, DecisionResponse},
    10	};
    11	
    12	/// 测试 / mock GUI 用的 IPC 客户端。
    13	///
    14	/// 连接服务端 socket，发送 JSON-RPC response（模拟 GUI 完成决策后的回调）。
    15	/// 不在生产主路径使用——主路径的 GUI 是独立进程（sieve-gui-macos）。
    16	///
    17	/// 关联：ADR-013 §3（协议传输）。
    18	pub struct IpcClient {
    19	    socket_path: std::path::PathBuf,
    20	}
    21	
    22	impl IpcClient {
    23	    /// 创建指向 `socket_path` 的客户端（不立即连接）。
    24	    pub fn new(socket_path: impl AsRef<Path>) -> Self {
    25	        Self {
    26	            socket_path: socket_path.as_ref().to_owned(),
    27	        }
    28	    }
    29	
    30	    /// 向服务端发送一个 [`DecisionResponse`]（换行分隔 JSON-RPC response 格式）。
    31	    pub async fn send_decision(&self, resp: &DecisionResponse) -> Result<(), IpcError> {
    32	        let mut stream = UnixStream::connect(&self.socket_path).await?;
    33	        let rpc_resp = jsonrpc::Response {
    34	            jsonrpc: "2.0".to_owned(),
    35	            result: Some(serde_json::to_value(resp)?),
    36	            error: None,
    37	            id: serde_json::Value::String(resp.request_id.to_string()),
    38	        };
    39	        let mut payload = serde_json::to_string(&rpc_resp)?;
    40	        payload.push('\n');
    41	        stream.write_all(payload.as_bytes()).await?;
    42	        Ok(())
    43	    }
    44	
    45	    /// 从 socket 读取一条换行分隔的 JSON-RPC request（服务端推来的决策请求）。
    46	    ///
    47	    /// 主要用于 mock GUI 侧读取请求并回复。
    48	    pub async fn recv_request(&self) -> Result<serde_json::Value, IpcError> {
    49	        let stream = UnixStream::connect(&self.socket_path).await?;
    50	        let reader = BufReader::new(stream);
    51	        let mut lines = reader.lines();
    52	        let line = lines.next_line().await?.ok_or_else(|| {
    53	            IpcError::UnexpectedResponse("connection closed without data".to_owned())
    54	        })?;
    55	        let val: serde_json::Value = serde_json::from_str(&line)?;
    56	        Ok(val)
    57	    }
    58	
    59	    /// 等待来自服务端的决策请求，自动回复指定的决策动作（测试辅助）。
    60	    pub async fn auto_respond(
    61	        socket_path: impl AsRef<Path>,
    62	        request_id: Uuid,
    63	        decision: crate::protocol::DecisionAction,
    64	    ) -> Result<(), IpcError> {
    65	        let path = socket_path.as_ref().to_owned();
    66	        // 短暂重试以等待服务端就绪。
    67	        let stream = retry_connect(&path, 5, std::time::Duration::from_millis(20)).await?;
    68	        let (reader_half, mut writer_half) = stream.into_split();
    69	        let mut lines = BufReader::new(reader_half).lines();
    70	
    71	        // 读一条请求（忽略内容，只要 request_id 匹配就回）。
    72	        while let Some(line) = lines.next_line().await? {
    73	            if line.trim().is_empty() {
    74	                continue;
    75	            }
    76	            let resp = DecisionResponse {
    77	                request_id,
    78	                decision,
    79	                decided_at: chrono::Utc::now(),
    80	                by_user: true,
    81	                remember: false,
    82	            };
    83	            let rpc_resp = jsonrpc::Response {
    84	                jsonrpc: "2.0".to_owned(),
    85	                result: Some(serde_json::to_value(&resp)?),
    86	                error: None,
    87	                id: serde_json::Value::String(request_id.to_string()),
    88	            };
    89	            let mut payload = serde_json::to_string(&rpc_resp)?;
    90	            payload.push('\n');
    91	            writer_half.write_all(payload.as_bytes()).await?;
    92	            break;
    93	        }
    94	        Ok(())
    95	    }
    96	}
    97	
    98	/// 连接重试辅助——服务端 spawn 后稍有延迟才就绪。
    99	async fn retry_connect(
   100	    path: &std::path::Path,
   101	    attempts: u32,
   102	    delay: std::time::Duration,
   103	) -> Result<UnixStream, IpcError> {
   104	    let mut last_err = None;
   105	    for _ in 0..attempts {
   106	        match UnixStream::connect(path).await {
   107	            Ok(s) => return Ok(s),
   108	            Err(e) => {
   109	                last_err = Some(e);
   110	                tokio::time::sleep(delay).await;
   111	            }
   112	        }
   113	    }
   114	    Err(IpcError::Socket(last_err.unwrap()))
   115	}
--- crates/sieve-ipc/src/socket_server.rs
     1	use std::collections::HashMap;
     2	use std::path::PathBuf;
     3	use std::sync::Arc;
     4	use std::time::Duration;
     5	
     6	use chrono::Utc;
     7	use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
     8	use tokio::net::{UnixListener, UnixStream};
     9	use tokio::sync::{mpsc, oneshot, Mutex};
    10	use tracing::{debug, error, info, warn};
    11	use uuid::Uuid;
    12	
    13	use crate::{
    14	    error::IpcError,
    15	    protocol::{DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout},
    16	};
    17	
    18	/// pending map：request_id → oneshot 发送端，等待 GUI 回复。
    19	type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;
    20	
    21	/// GUI 客户端的写通道：向其发送换行分隔的 JSON 字符串即可推送到对端。
    22	///
    23	/// 使用 mpsc 而非直接持有 WriteHalf，这样写检测（`send` 失败）就能代替
    24	/// TCP keepalive 检测 GUI 进程崩溃。通道容量设为 32，满了则视为 GUI 卡死。
    25	type GuiWriter = Arc<Mutex<Option<mpsc::Sender<String>>>>;
    26	
    27	/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
    28	///
    29	/// # 连接语义
    30	///
    31	/// - GUI 启动后主动连接此 socket，保持长连接。
    32	/// - 同一时刻只允许一个 GUI 客户端（多连接时拒绝第二个，记录警告）。
    33	/// - GUI 断线后 `gui_writer` 自动清空；下一次 `request_decision` 立即 fallback。
    34	///
    35	/// # 双向通信模型
    36	///
    37	/// ```text
    38	/// [主代理]  ─request_decision JSON-RPC request─▶  [GUI]
    39	/// [主代理]  ◀─decision_response JSON-RPC response─  [GUI]
    40	/// ```
    41	///
    42	/// 每个方向在同一条 TCP/Unix 连接上用换行分隔的 JSON-RPC 帧传输。
    43	/// `handle_connection` 负责从 GUI 读取响应帧并派发到 `pending` map；
    44	/// `request_decision` 通过 `gui_writer` mpsc 通道写入请求帧。
    45	///
    46	/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
    47	pub struct IpcServer {
    48	    socket_path: PathBuf,
    49	    pending: PendingMap,
    50	    /// 当前已连接的 GUI 客户端写通道；无 GUI 时为 None。
    51	    gui_writer: GuiWriter,
    52	}
    53	
    54	impl IpcServer {
    55	    /// 绑定 Unix socket 并返回服务端实例。
    56	    ///
    57	    /// socket_path 已存在时先删除旧文件（daemon 重启场景）。
    58	    pub fn bind(socket_path: PathBuf) -> Result<(Self, UnixListener), IpcError> {
    59	        // 旧 socket 文件存在则先删除，否则 bind 会失败。
    60	        if socket_path.exists() {
    61	            std::fs::remove_file(&socket_path)?;
    62	        }
    63	        let listener = UnixListener::bind(&socket_path)?;
    64	        let server = Self {
    65	            socket_path,
    66	            pending: Arc::new(Mutex::new(HashMap::new())),
    67	            gui_writer: Arc::new(Mutex::new(None)),
    68	        };
    69	        Ok((server, listener))
    70	    }
    71	
    72	    /// 运行 accept 循环，处理来自 GUI 的长连接。
    73	    ///
    74	    /// 每个连接独立 spawn；同一时刻只接受一个 GUI 客户端，多余的直接关闭。
    75	    pub async fn run(&self, listener: UnixListener) {
    76	        info!(socket = %self.socket_path.display(), "IPC server listening");
    77	        loop {
    78	            match listener.accept().await {
    79	                Ok((stream, _addr)) => {
    80	                    let pending = Arc::clone(&self.pending);
    81	                    let gui_writer = Arc::clone(&self.gui_writer);
    82	
    83	                    // 检查是否已有 GUI 客户端。
    84	                    // 用 try_lock 避免阻塞 accept 循环；如果锁被占用就放通并让
    85	                    // handle_connection 内部处理（竞态概率极低）。
    86	                    {
    87	                        let mut guard = gui_writer.lock().await;
    88	                        if guard.is_some() {
    89	                            warn!("second GUI client attempted to connect; rejecting");
    90	                            // 直接 drop stream 关闭连接，不 spawn 处理。
    91	                            drop(stream);
    92	                            continue;
    93	                        }
    94	                        // 还没有 GUI 客户端——创建 mpsc 通道，把发送端存入 gui_writer，
    95	                        // 接收端传给 handle_connection 用于写回 GUI。
    96	                        let (tx, rx) = mpsc::channel::<String>(32);
    97	                        *guard = Some(tx);
    98	                        drop(guard);
    99	
   100	                        tokio::spawn(async move {
   101	                            if let Err(e) =
   102	                                handle_connection(stream, pending, gui_writer.clone(), rx).await
   103	                            {
   104	                                error!("IPC connection error: {e}");
   105	                            }
   106	                            // 连接断开后清理 gui_writer，下一个 GUI 可以重连。
   107	                            let mut w = gui_writer.lock().await;
   108	                            *w = None;
   109	                            info!("GUI client disconnected; gui_writer cleared");
   110	                        });
   111	                    }
   112	                }
   113	                Err(e) => {
   114	                    error!("IPC accept error: {e}");
   115	                    break;
   116	                }
   117	            }
   118	        }
   119	    }
   120	
   121	    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
   122	    ///
   123	    /// # 行为
   124	    ///
   125	    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
   126	    ///   （等超时无意义——没人能决策。）
   127	    /// - 如果 GUI 写通道已满或 GUI 进程崩溃（mpsc send 失败）：立即 fallback。
   128	    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
   129	    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
   130	    pub async fn request_decision(
   131	        &self,
   132	        req: DecisionRequest,
   133	        timeout: Duration,
   134	    ) -> Result<DecisionResponse, IpcError> {
   135	        let request_id = req.request_id;
   136	        let default_on_timeout = req.default_on_timeout;
   137	
   138	        // 1. 检查 GUI 是否已连接。
   139	        let sender = {
   140	            let guard = self.gui_writer.lock().await;
   141	            guard.clone()
   142	        };
   143	
   144	        let Some(sender) = sender else {
   145	            // 没有 GUI——立即 fallback，不消耗超时时间。
   146	            debug!(%request_id, "no GUI client connected; immediate fallback");
   147	            return Ok(make_timeout_fallback(request_id, default_on_timeout));
   148	        };
   149	
   150	        // 2. 注册 oneshot channel，等待 GUI 回复。
   151	        let (tx, rx) = oneshot::channel::<DecisionResponse>();
   152	        {
   153	            let mut map = self.pending.lock().await;
   154	            map.insert(request_id, tx);
   155	        }
   156	
   157	        // 3. 通过 mpsc 通道把请求推到 handle_connection 的写循环，
   158	        //    再由写循环写入真正的 GUI socket 连接。
   159	        let rpc_req = crate::protocol::jsonrpc::Request::call(
   160	            "request_decision",
   161	            serde_json::to_value(&req)?,
   162	            serde_json::Value::String(request_id.to_string()),
   163	        );
   164	        let mut payload = serde_json::to_string(&rpc_req)?;
   165	        payload.push('\n');
   166	
   167	        if let Err(_e) = sender.send(payload).await {
   168	            // GUI 写通道关闭（GUI 进程崩溃或通道满），立即 fallback。
   169	            warn!(%request_id, "GUI writer channel closed; immediate fallback");
   170	            self.pending.lock().await.remove(&request_id);
   171	            return Ok(make_timeout_fallback(request_id, default_on_timeout));
   172	        }
   173	
   174	        // 4. 等待 GUI 回复或超时。
   175	        match tokio::time::timeout(timeout, rx).await {
   176	            Ok(Ok(resp)) => Ok(resp),
   177	            Ok(Err(_)) => {
   178	                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
   179	                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
   180	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   181	            }
   182	            Err(_elapsed) => {
   183	                // 超时，清理 pending map。
   184	                self.pending.lock().await.remove(&request_id);
   185	                warn!(%request_id, "decision timeout");
   186	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   187	            }
   188	        }
   189	    }
   190	
   191	    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
   192	    pub async fn inject_decision(&self, resp: DecisionResponse) {
   193	        let mut map = self.pending.lock().await;
   194	        if let Some(tx) = map.remove(&resp.request_id) {
   195	            let _ = tx.send(resp);
   196	        }
   197	    }
   198	}
   199	
   200	/// 处理单个 GUI 长连接。
   201	///
   202	/// 同时管理两个方向：
   203	/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map。
   204	/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
   205	///
   206	/// 任一方向出错（GUI 断线 / 写失败）都会退出，调用方负责清理 `gui_writer`。
   207	async fn handle_connection(
   208	    stream: UnixStream,
   209	    pending: PendingMap,
   210	    gui_writer: GuiWriter,
   211	    mut write_rx: mpsc::Receiver<String>,
   212	) -> Result<(), IpcError> {
   213	    info!("GUI client connected");
   214	
   215	    let (read_half, mut write_half) = stream.into_split();
   216	    let mut lines = BufReader::new(read_half).lines();
   217	
   218	    loop {
   219	        tokio::select! {
   220	            // 读方向：GUI 发来 decision_response。
   221	            line_result = lines.next_line() => {
   222	                match line_result? {
   223	                    None => {
   224	                        // GUI 关闭连接。
   225	                        info!("GUI client closed connection");
   226	                        break;
   227	                    }
   228	                    Some(line) => {
   229	                        let line = line.trim().to_owned();
   230	                        if line.is_empty() {
   231	                            continue;
   232	                        }
   233	                        debug!(raw = %line, "received IPC message from GUI");
   234	                        dispatch_response(&line, &pending).await;
   235	                    }
   236	                }
   237	            }
   238	
   239	            // 写方向：主代理 push request_decision 给 GUI。
   240	            msg = write_rx.recv() => {
   241	                match msg {
   242	                    None => {
   243	                        // 发送端已丢弃（IpcServer 被 drop），退出。
   244	                        debug!("GUI write channel closed");
   245	                        break;
   246	                    }
   247	                    Some(payload) => {
   248	                        if let Err(e) = write_half.write_all(payload.as_bytes()).await {
   249	                            warn!("failed to write to GUI socket: {e}");
   250	                            break;
   251	                        }
   252	                    }
   253	                }
   254	            }
   255	        }
   256	    }
   257	
   258	    // 连接断开：把所有 pending oneshot 全部触发 fallback（drop sender）。
   259	    // 丢弃 sender 会让 rx 收到 Err(RecvError)，request_decision 走 fallback。
   260	    let mut map = pending.lock().await;
--- crates/sieve-hook/src/decision.rs
     1	use std::path::Path;
     2	
     3	use chrono::Utc;
     4	use fd_lock::RwLock;
     5	use uuid::Uuid;
     6	
     7	use crate::protocol::DecisionResponse;
     8	
     9	/// hook 侧决策结果。
    10	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
    11	pub enum DecisionOutcome {
    12	    /// 用户允许，hook 返回 exit 0。
    13	    Allow,
    14	    /// 用户拒绝或超时 fail-closed，hook 返回 exit 1。
    15	    Deny,
    16	}
    17	
    18	/// 将决策结果写入 `<base>/decisions/<request_id>.json`。
    19	///
    20	/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁。
    21	///
    22	/// Critical 规则 `remember` 永远 `false`，由调用方（main.rs）强制传入 false。
    23	/// 关联：SPEC-001 §3.3（决策文件写入）、ADR-014（Critical 不可记住）。
    24	pub fn write_decision(
    25	    request_id: Uuid,
    26	    outcome: &DecisionOutcome,
    27	    base: &Path,
    28	) -> Result<(), String> {
    29	    // 确保目录存在。
    30	    let decisions_dir = base.join("decisions");
    31	    let locks_dir = base.join("locks");
    32	    std::fs::create_dir_all(&decisions_dir).map_err(|e| e.to_string())?;
    33	    std::fs::create_dir_all(&locks_dir).map_err(|e| e.to_string())?;
    34	
    35	    let lock_path = locks_dir.join(format!("{request_id}.lock"));
    36	    let dec_path = decisions_dir.join(format!("{request_id}.json"));
    37	
    38	    let lock_file = std::fs::OpenOptions::new()
    39	        .write(true)
    40	        .create(true)
    41	        .truncate(false)
    42	        .open(&lock_path)
    43	        .map_err(|e| e.to_string())?;
    44	
    45	    let mut lock = RwLock::new(lock_file);
    46	    let _guard = lock.write().map_err(|e| e.to_string())?;
    47	
    48	    let decision_str = match outcome {
    49	        DecisionOutcome::Allow => "allow",
    50	        DecisionOutcome::Deny => "deny",
    51	    };
    52	
    53	    let resp = DecisionResponse {
    54	        request_id,
    55	        decision: decision_str.to_owned(),
    56	        decided_at: Utc::now(),
    57	        by_user: true,
    58	        // Critical 规则 remember 强制 false（SPEC-001 §4.4）。
    59	        remember: false,
    60	    };
    61	
    62	    let json = serde_json::to_string_pretty(&resp).map_err(|e| e.to_string())?;
    63	    std::fs::write(&dec_path, json.as_bytes()).map_err(|e| e.to_string())?;
    64	
    65	    Ok(())
    66	}
--- crates/sieve-hook/src/error.rs
     1	/// pending 文件读取阶段的错误。
     2	///
     3	/// 独立定义（不依赖 sieve-ipc）以保持 sieve-hook 零重依赖目标。
     4	/// 关联：SPEC-001 §4（hook 决策流程）。
     5	pub enum PendingError {
     6	    /// pending 文件不存在——Sieve 代理未标记此请求，可 fail-open。
     7	    NotFound,
     8	    /// pending 文件存在但 created_at > stale 阈值，fail-closed。
     9	    Stale,
    10	    /// JSON 解析失败，fail-closed。
    11	    ParseError(String),
    12	    /// 其他 IO 错误。
    13	    IoError(String),
    14	}
--- crates/sieve-hook/src/lib.rs
     1	// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
     2	// main.rs 通过 use sieve_hook_lib::* 复用这些定义。
     3	
     4	pub mod decision;
     5	pub mod error;
     6	pub mod pending;
     7	pub mod protocol;
     8	
     9	use std::path::Path;
    10	use uuid::Uuid;
    11	
    12	use decision::{write_decision, DecisionOutcome};
    13	use error::PendingError;
    14	use pending::{read_pending_checked, scan_pending_dir};
    15	
    16	const STALE_THRESHOLD_SECS: i64 = 600;
    17	
    18	/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
    19	///
    20	/// pending 文件不存在 → exit 0（fail-open）
    21	/// pending 文件存在但已过期 → exit 1（fail-closed）
    22	/// JSON 解析失败 → exit 1（fail-closed）
    23	/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
    24	///
    25	/// 返回进程退出码：0 = 允许，1 = 拒绝。
    26	/// 关联：SPEC-001 §4（hook 决策流程）。
    27	pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    28	    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
    29	        Err(PendingError::NotFound) => 0,
    30	        Err(PendingError::Stale) => {
    31	            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
    32	            1
    33	        }
    34	        Err(PendingError::ParseError(e)) => {
    35	            eprintln!("sieve-hook: failed to parse pending file: {e}");
    36	            1
    37	        }
    38	        Err(PendingError::IoError(e)) => {
    39	            eprintln!("sieve-hook: IO error reading pending file: {e}");
    40	            1
    41	        }
    42	        Ok(req) => {
    43	            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
    44	            let outcome = match req.default_on_timeout {
    45	                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
    46	                _ => DecisionOutcome::Deny,
    47	            };
    48	            if let Err(e) = write_decision(request_id, &outcome, base) {
    49	                eprintln!("sieve-hook: failed to write decision: {e}");
    50	            }
    51	            match outcome {
    52	                DecisionOutcome::Allow => 0,
    53	                DecisionOutcome::Deny => 1,
    54	            }
    55	        }
    56	    }
    57	}
    58	
    59	/// 启发式运行逻辑：无 request_id 时扫目录。
    60	///
    61	/// 优先级 3（SPEC-001 §4.3）：
    62	/// - 零 fresh pending → fail-open（exit 0）
    63	/// - stale 文件 → 删除 + warn + fail-open（exit 0）
    64	/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
    65	///   多 pending 时用户一次决策广播给所有 request_id。
    66	///
    67	/// 返回进程退出码：0 = 允许，1 = 拒绝。
    68	/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
    69	pub fn run_check_heuristic(base: &Path) -> i32 {
    70	    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);
    71	
    72	    // 删除 stale 文件 + 打 warning。
    73	    for stale_path in &scan.stale_paths {
    74	        eprintln!(
    75	            "sieve-hook: warning: stale pending file deleted: {}",
    76	            stale_path.display()
    77	        );
    78	        let _ = std::fs::remove_file(stale_path);
    79	    }
    80	
    81	    if scan.fresh.is_empty() {
    82	        // 零 pending：Sieve 代理未标记任何请求，fail-open。
    83	        return 0;
    84	    }
    85	
    86	    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    87	    // （非 TTY 路径：直接按策略决定，不弹提示。）
    88	    let outcome = decide_outcome_for_requests(&scan.fresh);
    89	
    90	    // 广播决策给所有 pending request_id。
    91	    for req in &scan.fresh {
    92	        if let Err(e) = write_decision(req.request_id, &outcome, base) {
    93	            eprintln!(
    94	                "sieve-hook: failed to write decision for {}: {e}",
    95	                req.request_id
    96	            );
    97	        }
    98	    }
    99	
   100	    match outcome {
   101	        DecisionOutcome::Allow => 0,
   102	        DecisionOutcome::Deny => 1,
   103	    }
   104	}
   105	
   106	/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
   107	fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
   108	    for req in reqs {
   109	        match req.default_on_timeout {
   110	            protocol::DefaultOnTimeout::Allow => {}
   111	            _ => return DecisionOutcome::Deny,
   112	        }
   113	    }
   114	    DecisionOutcome::Allow
   115	}
   116	
   117	#[cfg(test)]
   118	mod tests {
   119	    use chrono::{Duration, Utc};
   120	    use std::path::Path;
   121	    use uuid::Uuid;
   122	
   123	    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};
   124	
   125	    fn write_pending_json(base: &Path, req: &DecisionRequest) {
   126	        let dir = base.join("pending");
   127	        std::fs::create_dir_all(&dir).unwrap();
   128	        let json = serde_json::to_string_pretty(req).unwrap();
   129	        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
   130	    }
   131	
   132	    fn make_req(
   133	        id: Uuid,
   134	        dot: DefaultOnTimeout,
   135	        created_at: chrono::DateTime<Utc>,
   136	    ) -> DecisionRequest {
   137	        DecisionRequest {
   138	            request_id: id,
   139	            created_at,
   140	            timeout_seconds: 30,
   141	            default_on_timeout: dot,
   142	            detections: vec![],
   143	        }
   144	    }
   145	
   146	    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────
   147	
   148	    #[test]
   149	    fn pending_not_found_returns_0() {
   150	        let tmp = tempfile::tempdir().unwrap();
   151	        let id = Uuid::now_v7();
   152	        let code = super::run_check(id, tmp.path());
   153	        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
   154	    }
   155	
   156	    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────
   157	
   158	    #[test]
   159	    fn pending_stale_returns_1() {
   160	        let tmp = tempfile::tempdir().unwrap();
   161	        let id = Uuid::now_v7();
   162	        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
   163	        let stale_time = Utc::now() - Duration::minutes(11);
   164	        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
   165	        write_pending_json(tmp.path(), &req);
   166	        let code = super::run_check(id, tmp.path());
   167	        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
   168	    }
   169	
   170	    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────
   171	
   172	    #[test]
   173	    fn pending_parse_error_returns_1() {
   174	        let tmp = tempfile::tempdir().unwrap();
   175	        let id = Uuid::now_v7();
   176	        let dir = tmp.path().join("pending");
   177	        std::fs::create_dir_all(&dir).unwrap();
   178	        // 写入非法 JSON。
   179	        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
   180	        let code = super::run_check(id, tmp.path());
   181	        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
   182	    }
   183	
   184	    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────
   185	
   186	    #[test]
   187	    fn pending_allow_on_timeout_returns_0() {
   188	        let tmp = tempfile::tempdir().unwrap();
   189	        let id = Uuid::now_v7();
   190	        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
   191	        write_pending_json(tmp.path(), &req);
   192	        let code = super::run_check(id, tmp.path());
   193	        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
   194	    }
   195	
   196	    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────
   197	
   198	    #[test]
   199	    fn pending_block_on_timeout_returns_1() {
   200	        let tmp = tempfile::tempdir().unwrap();
   201	        let id = Uuid::now_v7();
   202	        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
   203	        write_pending_json(tmp.path(), &req);
   204	        let code = super::run_check(id, tmp.path());
   205	        assert_eq!(code, 1, "default_on_timeout=Block should return exit 1");
   206	    }
   207	
   208	    // ── Critical detection 记录的 decision.remember 永远 false ─────────────
   209	
   210	    #[test]
   211	    fn critical_decision_remember_is_false() {
   212	        let tmp = tempfile::tempdir().unwrap();
   213	        let id = Uuid::now_v7();
   214	        let req = DecisionRequest {
   215	            request_id: id,
   216	            created_at: Utc::now(),
   217	            timeout_seconds: 30,
   218	            default_on_timeout: DefaultOnTimeout::Allow,
   219	            detections: vec![DetectionPayload {
   220	                rule_id: "IN-CR-01".to_owned(),
   221	                severity: "critical".to_owned(),
   222	                disposition: "hook_terminal".to_owned(),
   223	                title: "Test".to_owned(),
   224	                one_line_summary: "test".to_owned(),
   225	                details: serde_json::Value::Null,
   226	            }],
   227	        };
   228	        write_pending_json(tmp.path(), &req);
   229	        super::run_check(id, tmp.path());
   230	
   231	        // 读取写入的 decision 文件，验证 remember=false。
   232	        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
   233	        let content = std::fs::read_to_string(dec_path).unwrap();
   234	        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
   235	        assert_eq!(resp["remember"], serde_json::Value::Bool(false));
   236	    }
   237	
   238	    // ════════════════════════════════════════════════════════════════════════
   239	    // 启发式匹配路径（run_check_heuristic）的 7 个新测试
   240	    // ════════════════════════════════════════════════════════════════════════
   241	
   242	    // 测试 1：零 pending 文件 → exit 0（fail-open）
   243	    #[test]
   244	    fn heuristic_zero_pending_fail_open() {
   245	        let tmp = tempfile::tempdir().unwrap();
   246	        // pending 目录不存在，模拟全新安装。
   247	        let code = super::run_check_heuristic(tmp.path());
   248	        assert_eq!(code, 0, "zero pending should fail-open (exit 0)");
   249	    }
   250	
   251	    // 测试 2：单 pending 文件 + default_on_timeout=Allow → exit 0
   252	    #[test]
   253	    fn heuristic_single_pending_allow() {
   254	        let tmp = tempfile::tempdir().unwrap();
   255	        let id = Uuid::now_v7();
   256	        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
   257	        write_pending_json(tmp.path(), &req);
   258	
   259	        let code = super::run_check_heuristic(tmp.path());
   260	        assert_eq!(code, 0, "single Allow pending should return exit 0");
--- crates/sieve-hook/src/main.rs
     1	// sieve-hook: Claude Code PreToolUse hook 二进制。
     2	//
     3	// 夹在 Claude Code tool_use 调用与实际执行之间，对命中 Critical 规则的工具调用
     4	// 在 TTY 显示危险摘要并等待用户确认。
     5	//
     6	// 启动时延目标 < 50ms（依赖仅 serde_json + fd-lock + clap，无 tokio / vectorscan）。
     7	// 关联：SPEC-001（hook 文件协议）、SPEC-002（弹窗行为规范）、ADR-014（双层防御）。
     8	
     9	use std::io::{self, BufRead, Write};
    10	use std::path::PathBuf;
    11	use std::time::Duration;
    12	
    13	use clap::Parser;
    14	use uuid::Uuid;
    15	
    16	// 从 lib target 引入共享模块，避免重复定义。
    17	use sieve_hook_lib::decision::{write_decision, DecisionOutcome};
    18	use sieve_hook_lib::error::PendingError;
    19	use sieve_hook_lib::pending::{read_pending_checked, scan_pending_dir};
    20	use sieve_hook_lib::protocol;
    21	
    22	const STALE_THRESHOLD_SECS: i64 = 600;
    23	
    24	/// sieve-hook: PreToolUse 安全确认 hook（Phase 1 macOS）。
    25	#[derive(Parser, Debug)]
    26	#[command(name = "sieve-hook", about = "Sieve PreToolUse safety hook")]
    27	struct Cli {
    28	    #[command(subcommand)]
    29	    command: Command,
    30	}
    31	
    32	#[derive(clap::Subcommand, Debug)]
    33	enum Command {
    34	    /// 检查 pending 决策请求并请求用户确认。
    35	    Check {
    36	        /// 决策请求 ID（UUID）；未传则读 $SIEVE_REQUEST_ID。
    37	        #[arg(long)]
    38	        request_id: Option<String>,
    39	
    40	        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
    41	        #[arg(long)]
    42	        sieve_home: Option<PathBuf>,
    43	    },
    44	}
    45	
    46	fn main() {
    47	    let cli = Cli::parse();
    48	    let Command::Check {
    49	        request_id,
    50	        sieve_home,
    51	    } = cli.command;
    52	
    53	    // 解析 sieve_home：flag > env > default。
    54	    let base = sieve_home
    55	        .or_else(|| std::env::var("SIEVE_HOME").ok().map(PathBuf::from))
    56	        .or_else(|| {
    57	            std::env::var("HOME")
    58	                .ok()
    59	                .map(|h| PathBuf::from(h).join(".sieve"))
    60	        })
    61	        .unwrap_or_else(|| {
    62	            eprintln!("sieve-hook: cannot determine sieve home directory ($HOME not set)");
    63	            std::process::exit(1);
    64	        });
    65	
    66	    // 解析 request_id：优先级 1（flag）> 优先级 2（env）> 优先级 3（启发式扫目录）。
    67	    // 优先级 3 是关键修复：Claude Code settings.json 注册静态命令时无法传 request_id，
    68	    // 必须走启发式路径；零 pending 时 fail-open（exit 0），不阻断正常工具调用。
    69	    // 关联：SPEC-001 §4.3（启发式查 pending 目录）。
    70	    let explicit_id = request_id.or_else(|| std::env::var("SIEVE_REQUEST_ID").ok());
    71	
    72	    let exit_code = match explicit_id {
    73	        Some(id_str) => {
    74	            let request_id = match Uuid::parse_str(&id_str) {
    75	                Ok(id) => id,
    76	                Err(e) => {
    77	                    eprintln!("sieve-hook: invalid request ID `{id_str}`: {e}");
    78	                    std::process::exit(1);
    79	                }
    80	            };
    81	            run(request_id, &base)
    82	        }
    83	        None => {
    84	            // 优先级 3：启发式扫目录。
    85	            run_heuristic(&base)
    86	        }
    87	    };
    88	
    89	    std::process::exit(exit_code);
    90	}
    91	
    92	/// 核心逻辑，返回进程退出码（0 = 允许，1 = 拒绝）。
    93	///
    94	/// 关联：SPEC-001 §4（hook 决策流程）。
    95	fn run(request_id: Uuid, base: &std::path::Path) -> i32 {
    96	    let req = match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
    97	        Ok(r) => r,
    98	        Err(PendingError::NotFound) => {
    99	            // fail-open：Sieve 代理未标记此请求，放行。
   100	            return 0;
   101	        }
   102	        Err(PendingError::Stale) => {
   103	            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
   104	            return 1;
   105	        }
   106	        Err(PendingError::ParseError(e)) => {
   107	            eprintln!("sieve-hook: failed to parse pending file: {e}");
   108	            return 1;
   109	        }
   110	        Err(PendingError::IoError(e)) => {
   111	            eprintln!("sieve-hook: IO error reading pending file: {e}");
   112	            return 1;
   113	        }
   114	    };
   115	
   116	    // 打印危险摘要（SPEC-002 §2：多 issue 合并风格）。
   117	    print_summary(&req);
   118	
   119	    // 倒计时交互。
   120	    let outcome = prompt_user(&req);
   121	
   122	    // 写决策文件。
   123	    if let Err(e) = write_decision(request_id, &outcome, base) {
   124	        eprintln!("sieve-hook: failed to write decision: {e}");
   125	    }
   126	
   127	    match outcome {
   128	        DecisionOutcome::Allow => 0,
   129	        DecisionOutcome::Deny => 1,
   130	    }
   131	}
   132	
   133	/// 打印危险摘要到 stderr（TTY 终端显示）。
   134	///
   135	/// 关联：SPEC-002 §2.1（多 issue 合并显示）。
   136	fn print_summary(req: &protocol::DecisionRequest) {
   137	    let n = req.detections.len();
   138	    eprintln!();
   139	    eprintln!("┌─ Sieve 安全警告 ({n} 条检测) ────────────────────────────────");
   140	    for (i, det) in req.detections.iter().enumerate() {
   141	        let severity_tag = match det.severity.as_str() {
   142	            "critical" => "CRITICAL",
   143	            "high" => "HIGH    ",
   144	            "medium" => "MEDIUM  ",
   145	            _ => "LOW     ",
   146	        };
   147	        eprintln!(
   148	            "│ [{:2}] [{severity_tag}] {} — {}",
   149	            i + 1,
   150	            det.rule_id,
   151	            det.title
   152	        );
   153	        eprintln!("│       {}", det.one_line_summary);
   154	    }
   155	    eprintln!("└────────────────────────────────────────────────────────────");
   156	    eprintln!();
   157	}
   158	
   159	/// TTY 倒计时交互，返回用户决策。
   160	///
   161	/// - 输入 `y`/`Y` → Allow（exit 0）
   162	/// - 输入 `n`/`N`/回车（默认拒绝）→ Deny（exit 1）
   163	/// - 倒计时到 → 按 default_on_timeout 决定
   164	///
   165	/// 用 `spawn thread + mpsc channel` 实现非阻塞输入，避免引入 tokio。
   166	fn prompt_user(req: &protocol::DecisionRequest) -> DecisionOutcome {
   167	    let timeout = Duration::from_secs(req.timeout_seconds as u64);
   168	    let deadline = std::time::Instant::now() + timeout;
   169	
   170	    let stdin = io::stdin();
   171	    let (tx, rx) = std::sync::mpsc::channel::<String>();
   172	    std::thread::spawn(move || {
   173	        let mut line = String::new();
   174	        let _ = stdin.lock().read_line(&mut line);
   175	        let _ = tx.send(line);
   176	    });
   177	
   178	    loop {
   179	        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
   180	        eprint!(
   181	            "\r允许此操作？[y/N]（{} 秒后默认{}） > ",
   182	            remaining.as_secs(),
   183	            default_label(req.default_on_timeout)
   184	        );
   185	        let _ = io::stderr().flush();
   186	
   187	        match rx.recv_timeout(Duration::from_millis(100)) {
   188	            Ok(line) => {
   189	                eprintln!();
   190	                return match line.trim().to_lowercase().as_str() {
   191	                    "y" => DecisionOutcome::Allow,
   192	                    _ => DecisionOutcome::Deny,
   193	                };
   194	            }
   195	            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
   196	                if std::time::Instant::now() >= deadline {
   197	                    eprintln!();
   198	                    return match req.default_on_timeout {
   199	                        protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
   200	                        _ => DecisionOutcome::Deny,
   201	                    };
   202	                }
   203	            }
   204	            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
   205	                eprintln!();
   206	                return DecisionOutcome::Deny;
   207	            }
   208	        }
   209	    }
   210	}
   211	
   212	fn default_label(dot: protocol::DefaultOnTimeout) -> &'static str {
   213	    match dot {
   214	        protocol::DefaultOnTimeout::Allow => "允许",
   215	        _ => "拒绝",
   216	    }
   217	}
   218	
   219	/// 启发式路径：无 request_id 时扫目录。
   220	///
   221	/// - 零 fresh pending → fail-open（exit 0）
   222	/// - stale 文件 → 删除 + warn + fail-open（exit 0）
   223	/// - 有 fresh pending → 合并显示所有 detection，TTY 弹窗确认，广播决策
   224	///
   225	/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
   226	fn run_heuristic(base: &std::path::Path) -> i32 {
   227	    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);
   228	
   229	    // 删除 stale 文件 + 打 warning。
   230	    for stale_path in &scan.stale_paths {
   231	        eprintln!(
   232	            "sieve-hook: warning: stale pending file deleted: {}",
   233	            stale_path.display()
   234	        );
   235	        let _ = std::fs::remove_file(stale_path);
   236	    }
   237	
   238	    if scan.fresh.is_empty() {
   239	        // 零 pending：Sieve 代理未标记任何请求，fail-open。
   240	        return 0;
   241	    }
   242	
   243	    // 合并所有 detection 到一个"虚拟"请求以统一显示。
   244	    // timeout_seconds 和 default_on_timeout 取最严的策略（任一 Block/Redact → Deny）。
   245	    let merged = merge_requests(&scan.fresh);
   246	    print_summary(&merged);
   247	    let outcome = prompt_user(&merged);
   248	
   249	    // 广播决策给所有 pending request_id。
   250	    for req in &scan.fresh {
   251	        if let Err(e) = write_decision(req.request_id, &outcome, base) {
   252	            eprintln!(
   253	                "sieve-hook: failed to write decision for {}: {e}",
   254	                req.request_id
   255	            );
   256	        }
   257	    }
   258	
   259	    match outcome {
   260	        DecisionOutcome::Allow => 0,
--- crates/sieve-hook/src/pending.rs
     1	use std::path::Path;
     2	
     3	use uuid::Uuid;
     4	
     5	use crate::{error::PendingError, protocol::DecisionRequest};
     6	
     7	/// 读取并验证 pending 文件。
     8	///
     9	/// 返回：
    10	/// - `Ok(DecisionRequest)` — 文件存在、未过期、解析成功
    11	/// - `Err(PendingError::NotFound)` — 文件不存在（fail-open）
    12	/// - `Err(PendingError::Stale)` — created_at 超过 `stale_threshold_secs`（fail-closed）
    13	/// - `Err(PendingError::ParseError)` — JSON 解析失败（fail-closed）
    14	/// - `Err(PendingError::IoError)` — 其他 IO 错误
    15	///
    16	/// 关联：SPEC-001 §4.2（stale 检测）。
    17	pub fn read_pending_checked(
    18	    request_id: Uuid,
    19	    base: &Path,
    20	    stale_threshold_secs: i64,
    21	) -> Result<DecisionRequest, PendingError> {
    22	    let path = base.join("pending").join(format!("{request_id}.json"));
    23	
    24	    let content = match std::fs::read_to_string(&path) {
    25	        Ok(c) => c,
    26	        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
    27	            return Err(PendingError::NotFound);
    28	        }
    29	        Err(e) => return Err(PendingError::IoError(e.to_string())),
    30	    };
    31	
    32	    let req: DecisionRequest =
    33	        serde_json::from_str(&content).map_err(|e| PendingError::ParseError(e.to_string()))?;
    34	
    35	    // stale 检测：created_at 超过阈值视为过期，fail-closed。
    36	    let age_secs = chrono::Utc::now()
    37	        .signed_duration_since(req.created_at)
    38	        .num_seconds();
    39	    if age_secs > stale_threshold_secs {
    40	        return Err(PendingError::Stale);
    41	    }
    42	
    43	    Ok(req)
    44	}
    45	
    46	/// 启发式扫目录结果。
    47	pub struct ScanResult {
    48	    /// 所有有效（未过期）的 pending 请求，按 created_at 升序排列。
    49	    pub fresh: Vec<DecisionRequest>,
    50	    /// 过期的 pending 文件路径（供调用方删除）。
    51	    pub stale_paths: Vec<std::path::PathBuf>,
    52	}
    53	
    54	/// 扫描 `<base>/pending/` 目录，收集所有未过期的 pending 文件。
    55	///
    56	/// 用于 SIEVE_REQUEST_ID 未设置时的启发式匹配路径。
    57	/// 按 created_at 升序排列，避免随机顺序引起非确定性行为。
    58	///
    59	/// 关联：SPEC-001 §4.3（启发式查 pending 目录）。
    60	pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
    61	    let pending_dir = base.join("pending");
    62	    let mut fresh: Vec<DecisionRequest> = Vec::new();
    63	    let mut stale_paths: Vec<std::path::PathBuf> = Vec::new();
    64	
    65	    let entries = match std::fs::read_dir(&pending_dir) {
    66	        Ok(e) => e,
    67	        Err(_) => {
    68	            // 目录不存在或无权读 → 视为空目录，fail-open。
    69	            return ScanResult { fresh, stale_paths };
    70	        }
    71	    };
    72	
    73	    let now = chrono::Utc::now();
    74	
    75	    for entry in entries.flatten() {
    76	        let path = entry.path();
    77	        // 只处理 .json 文件。
    78	        if path.extension().and_then(|e| e.to_str()) != Some("json") {
    79	            continue;
    80	        }
    81	        let content = match std::fs::read_to_string(&path) {
    82	            Ok(c) => c,
    83	            Err(_) => continue,
    84	        };
    85	        let req: DecisionRequest = match serde_json::from_str(&content) {
    86	            Ok(r) => r,
    87	            Err(_) => continue, // 解析失败的文件跳过。
    88	        };
    89	        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
    90	        if age_secs > stale_threshold_secs {
    91	            stale_paths.push(path);
    92	        } else {
    93	            fresh.push(req);
    94	        }
    95	    }
    96	
    97	    // 按 created_at 升序排列，保证确定性。
    98	    fresh.sort_by_key(|r| r.created_at);
    99	
   100	    ScanResult { fresh, stale_paths }
   101	}
--- crates/sieve-hook/src/protocol.rs
     1	// hook 侧轻量协议结构体，与 sieve-ipc 的 protocol.rs 保持字段对齐，
     2	// 但独立定义避免 tokio/tracing 等依赖拖入二进制。
     3	//
     4	// 关联：SPEC-001 §3（文件协议 schema）。
     5	
     6	use chrono::{DateTime, Utc};
     7	use serde::{Deserialize, Serialize};
     8	use uuid::Uuid;
     9	
    10	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    11	#[serde(rename_all = "snake_case")]
    12	pub enum DefaultOnTimeout {
    13	    Redact,
    14	    Block,
    15	    Allow,
    16	}
    17	
    18	#[derive(Debug, Clone, Serialize, Deserialize)]
    19	pub struct DetectionPayload {
    20	    pub rule_id: String,
    21	    pub severity: String,
    22	    pub disposition: String,
    23	    pub title: String,
    24	    pub one_line_summary: String,
    25	    pub details: serde_json::Value,
    26	}
    27	
    28	/// 与 sieve-ipc 的 DecisionRequest 字段完全对齐，用于反序列化 pending 文件。
    29	#[derive(Debug, Clone, Serialize, Deserialize)]
    30	pub struct DecisionRequest {
    31	    pub request_id: Uuid,
    32	    pub created_at: DateTime<Utc>,
    33	    pub timeout_seconds: u32,
    34	    pub default_on_timeout: DefaultOnTimeout,
    35	    pub detections: Vec<DetectionPayload>,
    36	}
    37	
    38	/// 写入 decisions/<id>.json 的结构。
    39	#[derive(Debug, Clone, Serialize, Deserialize)]
    40	pub struct DecisionResponse {
    41	    pub request_id: Uuid,
    42	    pub decision: String, // "allow" | "deny"
    43	    pub decided_at: DateTime<Utc>,
    44	    pub by_user: bool,
    45	    pub remember: bool,
    46	}

exec
/bin/zsh -lc "nl -ba crates/sieve-ipc/src/pending_file.rs | sed -n '1,220p'; nl -ba crates/sieve-ipc/src/protocol.rs | sed -n '1,260p'; nl -ba crates/sieve-ipc/src/socket_server.rs | sed -n '1,320p'; nl -ba crates/sieve-ipc/src/socket_client.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	use std::path::{Path, PathBuf};
     2	
     3	use fd_lock::RwLock;
     4	use uuid::Uuid;
     5	
     6	use crate::{
     7	    error::IpcError,
     8	    paths::{ensure_dirs, pending_dir},
     9	    protocol::DecisionRequest,
    10	};
    11	
    12	/// 将 [`DecisionRequest`] 写入 `<base>/pending/<request_id>.json`。
    13	///
    14	/// 写入前用 fd-lock 对目标文件加独占写锁，防止并发写入同一 request_id（极少见
    15	/// 但理论可行）。文件以 pretty JSON 格式写入，方便调试和 hook 侧直接读取。
    16	///
    17	/// 关联：SPEC-001 §3.1（pending 文件写入规约）、ADR-013 §4（文件协议备用路径）。
    18	pub fn write_pending(req: &DecisionRequest, base: &Path) -> Result<PathBuf, IpcError> {
    19	    ensure_dirs(base)?;
    20	    let dir = pending_dir(base);
    21	    let path = dir.join(format!("{}.json", req.request_id));
    22	
    23	    // 打开（或创建）文件，然后加独占写锁再写内容。
    24	    // 使用 std::fs::OpenOptions 而非 std::fs::write，以便 fd-lock 持有文件描述符。
    25	    let file = std::fs::OpenOptions::new()
    26	        .write(true)
    27	        .create(true)
    28	        .truncate(true)
    29	        .open(&path)?;
    30	
    31	    let mut lock = RwLock::new(file);
    32	    {
    33	        let mut guard = lock
    34	            .write()
    35	            .map_err(|e| IpcError::FileLock(e.to_string()))?;
    36	        let json = serde_json::to_string_pretty(req)?;
    37	        use std::io::Write;
    38	        guard.write_all(json.as_bytes())?;
    39	    }
    40	
    41	    Ok(path)
    42	}
    43	
    44	/// 读取并解析 `<base>/pending/<request_id>.json`。
    45	///
    46	/// 返回：
    47	/// - `Ok(DecisionRequest)` 成功
    48	/// - `Err(IpcError::PendingNotFound)` 文件不存在
    49	/// - `Err(IpcError::Json)` 解析失败
    50	pub fn read_pending(request_id: Uuid, base: &Path) -> Result<DecisionRequest, IpcError> {
    51	    let path = pending_dir(base).join(format!("{request_id}.json"));
    52	    let content = std::fs::read_to_string(&path).map_err(|e| {
    53	        if e.kind() == std::io::ErrorKind::NotFound {
    54	            IpcError::PendingNotFound { request_id }
    55	        } else {
    56	            IpcError::Socket(e)
    57	        }
    58	    })?;
    59	    let req: DecisionRequest = serde_json::from_str(&content)?;
    60	    Ok(req)
    61	}
     1	use chrono::{DateTime, Utc};
     2	use serde::{Deserialize, Serialize};
     3	use uuid::Uuid;
     4	
     5	// ── Enums ────────────────────────────────────────────────────────────────────
     6	
     7	/// 检测结果的最终处置方式。
     8	///
     9	/// 与 sieve-rules 中的处置枚举镜像，IPC 层独立定义以避免循环依赖。
    10	/// 关联：ADR-014（双层防御）、SPEC-001。
    11	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    12	#[serde(rename_all = "snake_case")]
    13	pub enum Disposition {
    14	    /// 自动脱敏——出站阶段替换敏感内容后放行，无需人工确认。
    15	    AutoRedact,
    16	    /// 弹出 GUI 窗口（sieve-gui-macos）请求用户确认。
    17	    GuiPopup,
    18	    /// 调用 PreToolUse hook（sieve-hook 二进制）在 TTY 请求用户确认。
    19	    HookTerminal,
    20	    /// 在状态栏静默提示，不打断流程。
    21	    StatusBar,
    22	}
    23	
    24	/// 超时后的默认决策。
    25	///
    26	/// Critical 规则强制使用 Block，不允许下游覆盖。关联：ADR-014 §fail-closed。
    27	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    28	#[serde(rename_all = "snake_case")]
    29	pub enum DefaultOnTimeout {
    30	    /// 脱敏后放行（适用于 AutoRedact 类型的超时回退）。
    31	    Redact,
    32	    /// 阻断——fail-closed，Critical 规则的强制回退策略。
    33	    Block,
    34	    /// 放行——仅适用于低优先级通知类规则。
    35	    Allow,
    36	}
    37	
    38	/// 检测命中的严重等级。
    39	///
    40	/// 关联：PRD §4 检测项分级、ADR-014。
    41	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    42	#[serde(rename_all = "snake_case")]
    43	pub enum Severity {
    44	    /// 最高级：签名、转账、部署等不可逆动作，强制人工确认，不可关闭。
    45	    Critical,
    46	    /// 高危：可逆但高风险操作。
    47	    High,
    48	    /// 中等：潜在风险，默认提示但可配置。
    49	    Medium,
    50	    /// 低危：信息提示。
    51	    Low,
    52	}
    53	
    54	// ── Detection payload ────────────────────────────────────────────────────────
    55	
    56	/// 单条检测命中的 IPC 表示。
    57	///
    58	/// 去掉规则匹配内部细节（正则 / offset），只保留 GUI/hook 渲染所需字段。
    59	/// 关联：SPEC-001 §3.2、SPEC-002 §2.1。
    60	#[derive(Debug, Clone, Serialize, Deserialize)]
    61	pub struct DetectionPayload {
    62	    /// 规则 ID，例如 `IN-CR-01`。用于 hook 终端显示和日志关联。
    63	    pub rule_id: String,
    64	    /// 严重等级。
    65	    pub severity: Severity,
    66	    /// 处置方式。
    67	    pub disposition: Disposition,
    68	    /// 简短标题，在 GUI 标题栏或 hook 首行显示。
    69	    pub title: String,
    70	    /// 单行摘要，不超过 120 字符，用于 hook 终端和通知消息。
    71	    pub one_line_summary: String,
    72	    /// 扩展详情，结构由各规则自定义（GUI 侧渲染详细视图用）。
    73	    pub details: serde_json::Value,
    74	}
    75	
    76	// ── Request / Response ───────────────────────────────────────────────────────
    77	
    78	/// 主代理 → GUI / Hook 的决策请求。
    79	///
    80	/// JSON-RPC 2.0 method = `"request_decision"`，通过 Unix socket 或 pending
    81	/// 文件协议传输。关联：ADR-013 §3、SPEC-001 §3.1。
    82	#[derive(Debug, Clone, Serialize, Deserialize)]
    83	pub struct DecisionRequest {
    84	    /// 全局唯一请求 ID（UUIDv7，含时间戳，便于排序和 stale 检测）。
    85	    pub request_id: Uuid,
    86	    /// 请求创建时间（UTC）。hook 侧用于 stale 检测（> 10 分钟视为过期）。
    87	    pub created_at: DateTime<Utc>,
    88	    /// 用户响应超时时长（秒）。范围 30–120，由规则配置决定。
    89	    pub timeout_seconds: u32,
    90	    /// 超时后的默认决策。Critical 规则此字段服务端强制为 `Block`。
    91	    pub default_on_timeout: DefaultOnTimeout,
    92	    /// 本次请求触发的所有检测命中列表（可多条）。
    93	    pub detections: Vec<DetectionPayload>,
    94	}
    95	
    96	/// 用户或超时产生的决策动作。
    97	///
    98	/// 关联：SPEC-001 §3.3、ADR-014 §决策流程。
    99	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   100	#[serde(rename_all = "snake_case")]
   101	pub enum DecisionAction {
   102	    /// 用户允许：GUI 类继续转发原始 SSE，Hook 类返回 exit 0。
   103	    Allow,
   104	    /// 用户拒绝：GUI 类截流注入 `sieve_blocked` event，Hook 类返回 exit 1。
   105	    Deny,
   106	    /// 仅出站脱敏类：按规则 redact 占位符替换后转发。
   107	    RedactAndAllow,
   108	}
   109	
   110	/// GUI / Hook → 主代理的决策响应。
   111	///
   112	/// 写入 `<sieve_home>/decisions/<request_id>.json` 或通过 socket 返回。
   113	/// 关联：ADR-013 §3.4、SPEC-001 §3.3。
   114	#[derive(Debug, Clone, Serialize, Deserialize)]
   115	pub struct DecisionResponse {
   116	    /// 对应的请求 ID，用于主代理侧匹配 oneshot channel。
   117	    pub request_id: Uuid,
   118	    /// 决策动作。
   119	    pub decision: DecisionAction,
   120	    /// 决策时间（UTC）。
   121	    pub decided_at: DateTime<Utc>,
   122	    /// `true` 表示用户主动操作，`false` 表示超时默认。
   123	    pub by_user: bool,
   124	    /// 是否记住此次决策（同规则 + 同 tool 不再询问）。
   125	    ///
   126	    /// Critical severity 的决策此字段服务端强制写 `false`，即使用户请求记住也拒绝。
   127	    pub remember: bool,
   128	}
   129	
   130	// ── JSON-RPC 2.0 envelope ────────────────────────────────────────────────────
   131	
   132	/// JSON-RPC 2.0 协议封装。
   133	///
   134	/// 手写实现以避免引入大型 jsonrpc crate 依赖。关联：ADR-013 §2（传输协议选型）。
   135	pub mod jsonrpc {
   136	    use serde::{Deserialize, Serialize};
   137	    use serde_json::Value;
   138	
   139	    /// JSON-RPC 2.0 请求（通知或有 id 的调用）。
   140	    #[derive(Debug, Clone, Serialize, Deserialize)]
   141	    pub struct Request {
   142	        pub jsonrpc: String,
   143	        pub method: String,
   144	        #[serde(skip_serializing_if = "Option::is_none")]
   145	        pub params: Option<Value>,
   146	        #[serde(skip_serializing_if = "Option::is_none")]
   147	        pub id: Option<Value>,
   148	    }
   149	
   150	    impl Request {
   151	        /// 构造一个有 id 的调用请求。
   152	        pub fn call(method: impl Into<String>, params: Value, id: Value) -> Self {
   153	            Self {
   154	                jsonrpc: "2.0".to_owned(),
   155	                method: method.into(),
   156	                params: Some(params),
   157	                id: Some(id),
   158	            }
   159	        }
   160	    }
   161	
   162	    /// JSON-RPC 2.0 成功响应。
   163	    #[derive(Debug, Clone, Serialize, Deserialize)]
   164	    pub struct Response {
   165	        pub jsonrpc: String,
   166	        #[serde(skip_serializing_if = "Option::is_none")]
   167	        pub result: Option<Value>,
   168	        #[serde(skip_serializing_if = "Option::is_none")]
   169	        pub error: Option<ErrorObject>,
   170	        pub id: Value,
   171	    }
   172	
   173	    /// JSON-RPC 2.0 错误对象。
   174	    #[derive(Debug, Clone, Serialize, Deserialize)]
   175	    pub struct ErrorObject {
   176	        pub code: i64,
   177	        pub message: String,
   178	        #[serde(skip_serializing_if = "Option::is_none")]
   179	        pub data: Option<Value>,
   180	    }
   181	}
     1	use std::collections::HashMap;
     2	use std::path::PathBuf;
     3	use std::sync::Arc;
     4	use std::time::Duration;
     5	
     6	use chrono::Utc;
     7	use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
     8	use tokio::net::{UnixListener, UnixStream};
     9	use tokio::sync::{mpsc, oneshot, Mutex};
    10	use tracing::{debug, error, info, warn};
    11	use uuid::Uuid;
    12	
    13	use crate::{
    14	    error::IpcError,
    15	    protocol::{DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout},
    16	};
    17	
    18	/// pending map：request_id → oneshot 发送端，等待 GUI 回复。
    19	type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;
    20	
    21	/// GUI 客户端的写通道：向其发送换行分隔的 JSON 字符串即可推送到对端。
    22	///
    23	/// 使用 mpsc 而非直接持有 WriteHalf，这样写检测（`send` 失败）就能代替
    24	/// TCP keepalive 检测 GUI 进程崩溃。通道容量设为 32，满了则视为 GUI 卡死。
    25	type GuiWriter = Arc<Mutex<Option<mpsc::Sender<String>>>>;
    26	
    27	/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
    28	///
    29	/// # 连接语义
    30	///
    31	/// - GUI 启动后主动连接此 socket，保持长连接。
    32	/// - 同一时刻只允许一个 GUI 客户端（多连接时拒绝第二个，记录警告）。
    33	/// - GUI 断线后 `gui_writer` 自动清空；下一次 `request_decision` 立即 fallback。
    34	///
    35	/// # 双向通信模型
    36	///
    37	/// ```text
    38	/// [主代理]  ─request_decision JSON-RPC request─▶  [GUI]
    39	/// [主代理]  ◀─decision_response JSON-RPC response─  [GUI]
    40	/// ```
    41	///
    42	/// 每个方向在同一条 TCP/Unix 连接上用换行分隔的 JSON-RPC 帧传输。
    43	/// `handle_connection` 负责从 GUI 读取响应帧并派发到 `pending` map；
    44	/// `request_decision` 通过 `gui_writer` mpsc 通道写入请求帧。
    45	///
    46	/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
    47	pub struct IpcServer {
    48	    socket_path: PathBuf,
    49	    pending: PendingMap,
    50	    /// 当前已连接的 GUI 客户端写通道；无 GUI 时为 None。
    51	    gui_writer: GuiWriter,
    52	}
    53	
    54	impl IpcServer {
    55	    /// 绑定 Unix socket 并返回服务端实例。
    56	    ///
    57	    /// socket_path 已存在时先删除旧文件（daemon 重启场景）。
    58	    pub fn bind(socket_path: PathBuf) -> Result<(Self, UnixListener), IpcError> {
    59	        // 旧 socket 文件存在则先删除，否则 bind 会失败。
    60	        if socket_path.exists() {
    61	            std::fs::remove_file(&socket_path)?;
    62	        }
    63	        let listener = UnixListener::bind(&socket_path)?;
    64	        let server = Self {
    65	            socket_path,
    66	            pending: Arc::new(Mutex::new(HashMap::new())),
    67	            gui_writer: Arc::new(Mutex::new(None)),
    68	        };
    69	        Ok((server, listener))
    70	    }
    71	
    72	    /// 运行 accept 循环，处理来自 GUI 的长连接。
    73	    ///
    74	    /// 每个连接独立 spawn；同一时刻只接受一个 GUI 客户端，多余的直接关闭。
    75	    pub async fn run(&self, listener: UnixListener) {
    76	        info!(socket = %self.socket_path.display(), "IPC server listening");
    77	        loop {
    78	            match listener.accept().await {
    79	                Ok((stream, _addr)) => {
    80	                    let pending = Arc::clone(&self.pending);
    81	                    let gui_writer = Arc::clone(&self.gui_writer);
    82	
    83	                    // 检查是否已有 GUI 客户端。
    84	                    // 用 try_lock 避免阻塞 accept 循环；如果锁被占用就放通并让
    85	                    // handle_connection 内部处理（竞态概率极低）。
    86	                    {
    87	                        let mut guard = gui_writer.lock().await;
    88	                        if guard.is_some() {
    89	                            warn!("second GUI client attempted to connect; rejecting");
    90	                            // 直接 drop stream 关闭连接，不 spawn 处理。
    91	                            drop(stream);
    92	                            continue;
    93	                        }
    94	                        // 还没有 GUI 客户端——创建 mpsc 通道，把发送端存入 gui_writer，
    95	                        // 接收端传给 handle_connection 用于写回 GUI。
    96	                        let (tx, rx) = mpsc::channel::<String>(32);
    97	                        *guard = Some(tx);
    98	                        drop(guard);
    99	
   100	                        tokio::spawn(async move {
   101	                            if let Err(e) =
   102	                                handle_connection(stream, pending, gui_writer.clone(), rx).await
   103	                            {
   104	                                error!("IPC connection error: {e}");
   105	                            }
   106	                            // 连接断开后清理 gui_writer，下一个 GUI 可以重连。
   107	                            let mut w = gui_writer.lock().await;
   108	                            *w = None;
   109	                            info!("GUI client disconnected; gui_writer cleared");
   110	                        });
   111	                    }
   112	                }
   113	                Err(e) => {
   114	                    error!("IPC accept error: {e}");
   115	                    break;
   116	                }
   117	            }
   118	        }
   119	    }
   120	
   121	    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
   122	    ///
   123	    /// # 行为
   124	    ///
   125	    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
   126	    ///   （等超时无意义——没人能决策。）
   127	    /// - 如果 GUI 写通道已满或 GUI 进程崩溃（mpsc send 失败）：立即 fallback。
   128	    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
   129	    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
   130	    pub async fn request_decision(
   131	        &self,
   132	        req: DecisionRequest,
   133	        timeout: Duration,
   134	    ) -> Result<DecisionResponse, IpcError> {
   135	        let request_id = req.request_id;
   136	        let default_on_timeout = req.default_on_timeout;
   137	
   138	        // 1. 检查 GUI 是否已连接。
   139	        let sender = {
   140	            let guard = self.gui_writer.lock().await;
   141	            guard.clone()
   142	        };
   143	
   144	        let Some(sender) = sender else {
   145	            // 没有 GUI——立即 fallback，不消耗超时时间。
   146	            debug!(%request_id, "no GUI client connected; immediate fallback");
   147	            return Ok(make_timeout_fallback(request_id, default_on_timeout));
   148	        };
   149	
   150	        // 2. 注册 oneshot channel，等待 GUI 回复。
   151	        let (tx, rx) = oneshot::channel::<DecisionResponse>();
   152	        {
   153	            let mut map = self.pending.lock().await;
   154	            map.insert(request_id, tx);
   155	        }
   156	
   157	        // 3. 通过 mpsc 通道把请求推到 handle_connection 的写循环，
   158	        //    再由写循环写入真正的 GUI socket 连接。
   159	        let rpc_req = crate::protocol::jsonrpc::Request::call(
   160	            "request_decision",
   161	            serde_json::to_value(&req)?,
   162	            serde_json::Value::String(request_id.to_string()),
   163	        );
   164	        let mut payload = serde_json::to_string(&rpc_req)?;
   165	        payload.push('\n');
   166	
   167	        if let Err(_e) = sender.send(payload).await {
   168	            // GUI 写通道关闭（GUI 进程崩溃或通道满），立即 fallback。
   169	            warn!(%request_id, "GUI writer channel closed; immediate fallback");
   170	            self.pending.lock().await.remove(&request_id);
   171	            return Ok(make_timeout_fallback(request_id, default_on_timeout));
   172	        }
   173	
   174	        // 4. 等待 GUI 回复或超时。
   175	        match tokio::time::timeout(timeout, rx).await {
   176	            Ok(Ok(resp)) => Ok(resp),
   177	            Ok(Err(_)) => {
   178	                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
   179	                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
   180	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   181	            }
   182	            Err(_elapsed) => {
   183	                // 超时，清理 pending map。
   184	                self.pending.lock().await.remove(&request_id);
   185	                warn!(%request_id, "decision timeout");
   186	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   187	            }
   188	        }
   189	    }
   190	
   191	    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
   192	    pub async fn inject_decision(&self, resp: DecisionResponse) {
   193	        let mut map = self.pending.lock().await;
   194	        if let Some(tx) = map.remove(&resp.request_id) {
   195	            let _ = tx.send(resp);
   196	        }
   197	    }
   198	}
   199	
   200	/// 处理单个 GUI 长连接。
   201	///
   202	/// 同时管理两个方向：
   203	/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map。
   204	/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
   205	///
   206	/// 任一方向出错（GUI 断线 / 写失败）都会退出，调用方负责清理 `gui_writer`。
   207	async fn handle_connection(
   208	    stream: UnixStream,
   209	    pending: PendingMap,
   210	    gui_writer: GuiWriter,
   211	    mut write_rx: mpsc::Receiver<String>,
   212	) -> Result<(), IpcError> {
   213	    info!("GUI client connected");
   214	
   215	    let (read_half, mut write_half) = stream.into_split();
   216	    let mut lines = BufReader::new(read_half).lines();
   217	
   218	    loop {
   219	        tokio::select! {
   220	            // 读方向：GUI 发来 decision_response。
   221	            line_result = lines.next_line() => {
   222	                match line_result? {
   223	                    None => {
   224	                        // GUI 关闭连接。
   225	                        info!("GUI client closed connection");
   226	                        break;
   227	                    }
   228	                    Some(line) => {
   229	                        let line = line.trim().to_owned();
   230	                        if line.is_empty() {
   231	                            continue;
   232	                        }
   233	                        debug!(raw = %line, "received IPC message from GUI");
   234	                        dispatch_response(&line, &pending).await;
   235	                    }
   236	                }
   237	            }
   238	
   239	            // 写方向：主代理 push request_decision 给 GUI。
   240	            msg = write_rx.recv() => {
   241	                match msg {
   242	                    None => {
   243	                        // 发送端已丢弃（IpcServer 被 drop），退出。
   244	                        debug!("GUI write channel closed");
   245	                        break;
   246	                    }
   247	                    Some(payload) => {
   248	                        if let Err(e) = write_half.write_all(payload.as_bytes()).await {
   249	                            warn!("failed to write to GUI socket: {e}");
   250	                            break;
   251	                        }
   252	                    }
   253	                }
   254	            }
   255	        }
   256	    }
   257	
   258	    // 连接断开：把所有 pending oneshot 全部触发 fallback（drop sender）。
   259	    // 丢弃 sender 会让 rx 收到 Err(RecvError)，request_decision 走 fallback。
   260	    let mut map = pending.lock().await;
   261	    let count = map.len();
   262	    if count > 0 {
   263	        warn!(
   264	            pending_count = count,
   265	            "GUI disconnected with pending requests; dropping all"
   266	        );
   267	        map.clear(); // 清空 map，sender 被 drop，所有等待者收到 Err 并 fallback。
   268	    }
   269	    // gui_writer 由 run() 的 spawn closure 在此函数返回后清理。
   270	    drop(gui_writer); // 显式 drop 避免编译器警告。
   271	
   272	    Ok(())
   273	}
   274	
   275	/// 解析 GUI 发来的一行 JSON-RPC response 并派发到 pending map。
   276	async fn dispatch_response(line: &str, pending: &PendingMap) {
   277	    let rpc: crate::protocol::jsonrpc::Response = match serde_json::from_str(line) {
   278	        Ok(v) => v,
   279	        Err(e) => {
   280	            warn!("failed to parse IPC response from GUI: {e}");
   281	            return;
   282	        }
   283	    };
   284	
   285	    if let Some(err_obj) = &rpc.error {
   286	        error!(
   287	            code = err_obj.code,
   288	            message = %err_obj.message,
   289	            "GUI returned rpc error"
   290	        );
   291	        return;
   292	    }
   293	
   294	    if let Some(result) = rpc.result {
   295	        match serde_json::from_value::<DecisionResponse>(result) {
   296	            Ok(resp) => {
   297	                let mut map = pending.lock().await;
   298	                if let Some(tx) = map.remove(&resp.request_id) {
   299	                    let _ = tx.send(resp);
   300	                } else {
   301	                    warn!(
   302	                        request_id = %resp.request_id,
   303	                        "no pending request for this decision"
   304	                    );
   305	                }
   306	            }
   307	            Err(e) => {
   308	                warn!("failed to deserialize DecisionResponse: {e}");
   309	            }
   310	        }
   311	    }
   312	}
   313	
   314	fn make_timeout_fallback(
   315	    request_id: Uuid,
   316	    default_on_timeout: DefaultOnTimeout,
   317	) -> DecisionResponse {
   318	    let action = match default_on_timeout {
   319	        DefaultOnTimeout::Block => DecisionAction::Deny,
   320	        DefaultOnTimeout::Allow => DecisionAction::Allow,
     1	use std::path::Path;
     2	
     3	use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
     4	use tokio::net::UnixStream;
     5	use uuid::Uuid;
     6	
     7	use crate::{
     8	    error::IpcError,
     9	    protocol::{jsonrpc, DecisionResponse},
    10	};
    11	
    12	/// 测试 / mock GUI 用的 IPC 客户端。
    13	///
    14	/// 连接服务端 socket，发送 JSON-RPC response（模拟 GUI 完成决策后的回调）。
    15	/// 不在生产主路径使用——主路径的 GUI 是独立进程（sieve-gui-macos）。
    16	///
    17	/// 关联：ADR-013 §3（协议传输）。
    18	pub struct IpcClient {
    19	    socket_path: std::path::PathBuf,
    20	}
    21	
    22	impl IpcClient {
    23	    /// 创建指向 `socket_path` 的客户端（不立即连接）。
    24	    pub fn new(socket_path: impl AsRef<Path>) -> Self {
    25	        Self {
    26	            socket_path: socket_path.as_ref().to_owned(),
    27	        }
    28	    }
    29	
    30	    /// 向服务端发送一个 [`DecisionResponse`]（换行分隔 JSON-RPC response 格式）。
    31	    pub async fn send_decision(&self, resp: &DecisionResponse) -> Result<(), IpcError> {
    32	        let mut stream = UnixStream::connect(&self.socket_path).await?;
    33	        let rpc_resp = jsonrpc::Response {
    34	            jsonrpc: "2.0".to_owned(),
    35	            result: Some(serde_json::to_value(resp)?),
    36	            error: None,
    37	            id: serde_json::Value::String(resp.request_id.to_string()),
    38	        };
    39	        let mut payload = serde_json::to_string(&rpc_resp)?;
    40	        payload.push('\n');
    41	        stream.write_all(payload.as_bytes()).await?;
    42	        Ok(())
    43	    }
    44	
    45	    /// 从 socket 读取一条换行分隔的 JSON-RPC request（服务端推来的决策请求）。
    46	    ///
    47	    /// 主要用于 mock GUI 侧读取请求并回复。
    48	    pub async fn recv_request(&self) -> Result<serde_json::Value, IpcError> {
    49	        let stream = UnixStream::connect(&self.socket_path).await?;
    50	        let reader = BufReader::new(stream);
    51	        let mut lines = reader.lines();
    52	        let line = lines.next_line().await?.ok_or_else(|| {
    53	            IpcError::UnexpectedResponse("connection closed without data".to_owned())
    54	        })?;
    55	        let val: serde_json::Value = serde_json::from_str(&line)?;
    56	        Ok(val)
    57	    }
    58	
    59	    /// 等待来自服务端的决策请求，自动回复指定的决策动作（测试辅助）。
    60	    pub async fn auto_respond(
    61	        socket_path: impl AsRef<Path>,
    62	        request_id: Uuid,
    63	        decision: crate::protocol::DecisionAction,
    64	    ) -> Result<(), IpcError> {
    65	        let path = socket_path.as_ref().to_owned();
    66	        // 短暂重试以等待服务端就绪。
    67	        let stream = retry_connect(&path, 5, std::time::Duration::from_millis(20)).await?;
    68	        let (reader_half, mut writer_half) = stream.into_split();
    69	        let mut lines = BufReader::new(reader_half).lines();
    70	
    71	        // 读一条请求（忽略内容，只要 request_id 匹配就回）。
    72	        while let Some(line) = lines.next_line().await? {
    73	            if line.trim().is_empty() {
    74	                continue;
    75	            }
    76	            let resp = DecisionResponse {
    77	                request_id,
    78	                decision,
    79	                decided_at: chrono::Utc::now(),
    80	                by_user: true,
    81	                remember: false,
    82	            };
    83	            let rpc_resp = jsonrpc::Response {
    84	                jsonrpc: "2.0".to_owned(),
    85	                result: Some(serde_json::to_value(&resp)?),
    86	                error: None,
    87	                id: serde_json::Value::String(request_id.to_string()),
    88	            };
    89	            let mut payload = serde_json::to_string(&rpc_resp)?;
    90	            payload.push('\n');
    91	            writer_half.write_all(payload.as_bytes()).await?;
    92	            break;
    93	        }
    94	        Ok(())
    95	    }
    96	}
    97	
    98	/// 连接重试辅助——服务端 spawn 后稍有延迟才就绪。
    99	async fn retry_connect(
   100	    path: &std::path::Path,
   101	    attempts: u32,
   102	    delay: std::time::Duration,
   103	) -> Result<UnixStream, IpcError> {
   104	    let mut last_err = None;
   105	    for _ in 0..attempts {
   106	        match UnixStream::connect(path).await {
   107	            Ok(s) => return Ok(s),
   108	            Err(e) => {
   109	                last_err = Some(e);
   110	                tokio::time::sleep(delay).await;
   111	            }
   112	        }
   113	    }
   114	    Err(IpcError::Socket(last_err.unwrap()))
   115	}

exec
/bin/zsh -lc "nl -ba crates/sieve-ipc/src/socket_server.rs | sed -n '314,420p'; nl -ba crates/sieve-hook/src/decision.rs | sed -n '1,220p'; nl -ba crates/sieve-hook/src/error.rs | sed -n '1,180p'; nl -ba crates/sieve-hook/src/lib.rs | sed -n '1,120p'; nl -ba crates/sieve-hook/src/main.rs | sed -n '219,380p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   314	fn make_timeout_fallback(
   315	    request_id: Uuid,
   316	    default_on_timeout: DefaultOnTimeout,
   317	) -> DecisionResponse {
   318	    let action = match default_on_timeout {
   319	        DefaultOnTimeout::Block => DecisionAction::Deny,
   320	        DefaultOnTimeout::Allow => DecisionAction::Allow,
   321	        DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
   322	    };
   323	    DecisionResponse {
   324	        request_id,
   325	        decision: action,
   326	        decided_at: Utc::now(),
   327	        by_user: false,
   328	        remember: false,
   329	    }
   330	}
     1	use std::path::Path;
     2	
     3	use chrono::Utc;
     4	use fd_lock::RwLock;
     5	use uuid::Uuid;
     6	
     7	use crate::protocol::DecisionResponse;
     8	
     9	/// hook 侧决策结果。
    10	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
    11	pub enum DecisionOutcome {
    12	    /// 用户允许，hook 返回 exit 0。
    13	    Allow,
    14	    /// 用户拒绝或超时 fail-closed，hook 返回 exit 1。
    15	    Deny,
    16	}
    17	
    18	/// 将决策结果写入 `<base>/decisions/<request_id>.json`。
    19	///
    20	/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁。
    21	///
    22	/// Critical 规则 `remember` 永远 `false`，由调用方（main.rs）强制传入 false。
    23	/// 关联：SPEC-001 §3.3（决策文件写入）、ADR-014（Critical 不可记住）。
    24	pub fn write_decision(
    25	    request_id: Uuid,
    26	    outcome: &DecisionOutcome,
    27	    base: &Path,
    28	) -> Result<(), String> {
    29	    // 确保目录存在。
    30	    let decisions_dir = base.join("decisions");
    31	    let locks_dir = base.join("locks");
    32	    std::fs::create_dir_all(&decisions_dir).map_err(|e| e.to_string())?;
    33	    std::fs::create_dir_all(&locks_dir).map_err(|e| e.to_string())?;
    34	
    35	    let lock_path = locks_dir.join(format!("{request_id}.lock"));
    36	    let dec_path = decisions_dir.join(format!("{request_id}.json"));
    37	
    38	    let lock_file = std::fs::OpenOptions::new()
    39	        .write(true)
    40	        .create(true)
    41	        .truncate(false)
    42	        .open(&lock_path)
    43	        .map_err(|e| e.to_string())?;
    44	
    45	    let mut lock = RwLock::new(lock_file);
    46	    let _guard = lock.write().map_err(|e| e.to_string())?;
    47	
    48	    let decision_str = match outcome {
    49	        DecisionOutcome::Allow => "allow",
    50	        DecisionOutcome::Deny => "deny",
    51	    };
    52	
    53	    let resp = DecisionResponse {
    54	        request_id,
    55	        decision: decision_str.to_owned(),
    56	        decided_at: Utc::now(),
    57	        by_user: true,
    58	        // Critical 规则 remember 强制 false（SPEC-001 §4.4）。
    59	        remember: false,
    60	    };
    61	
    62	    let json = serde_json::to_string_pretty(&resp).map_err(|e| e.to_string())?;
    63	    std::fs::write(&dec_path, json.as_bytes()).map_err(|e| e.to_string())?;
    64	
    65	    Ok(())
    66	}
     1	/// pending 文件读取阶段的错误。
     2	///
     3	/// 独立定义（不依赖 sieve-ipc）以保持 sieve-hook 零重依赖目标。
     4	/// 关联：SPEC-001 §4（hook 决策流程）。
     5	pub enum PendingError {
     6	    /// pending 文件不存在——Sieve 代理未标记此请求，可 fail-open。
     7	    NotFound,
     8	    /// pending 文件存在但 created_at > stale 阈值，fail-closed。
     9	    Stale,
    10	    /// JSON 解析失败，fail-closed。
    11	    ParseError(String),
    12	    /// 其他 IO 错误。
    13	    IoError(String),
    14	}
     1	// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
     2	// main.rs 通过 use sieve_hook_lib::* 复用这些定义。
     3	
     4	pub mod decision;
     5	pub mod error;
     6	pub mod pending;
     7	pub mod protocol;
     8	
     9	use std::path::Path;
    10	use uuid::Uuid;
    11	
    12	use decision::{write_decision, DecisionOutcome};
    13	use error::PendingError;
    14	use pending::{read_pending_checked, scan_pending_dir};
    15	
    16	const STALE_THRESHOLD_SECS: i64 = 600;
    17	
    18	/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
    19	///
    20	/// pending 文件不存在 → exit 0（fail-open）
    21	/// pending 文件存在但已过期 → exit 1（fail-closed）
    22	/// JSON 解析失败 → exit 1（fail-closed）
    23	/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
    24	///
    25	/// 返回进程退出码：0 = 允许，1 = 拒绝。
    26	/// 关联：SPEC-001 §4（hook 决策流程）。
    27	pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    28	    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
    29	        Err(PendingError::NotFound) => 0,
    30	        Err(PendingError::Stale) => {
    31	            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
    32	            1
    33	        }
    34	        Err(PendingError::ParseError(e)) => {
    35	            eprintln!("sieve-hook: failed to parse pending file: {e}");
    36	            1
    37	        }
    38	        Err(PendingError::IoError(e)) => {
    39	            eprintln!("sieve-hook: IO error reading pending file: {e}");
    40	            1
    41	        }
    42	        Ok(req) => {
    43	            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
    44	            let outcome = match req.default_on_timeout {
    45	                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
    46	                _ => DecisionOutcome::Deny,
    47	            };
    48	            if let Err(e) = write_decision(request_id, &outcome, base) {
    49	                eprintln!("sieve-hook: failed to write decision: {e}");
    50	            }
    51	            match outcome {
    52	                DecisionOutcome::Allow => 0,
    53	                DecisionOutcome::Deny => 1,
    54	            }
    55	        }
    56	    }
    57	}
    58	
    59	/// 启发式运行逻辑：无 request_id 时扫目录。
    60	///
    61	/// 优先级 3（SPEC-001 §4.3）：
    62	/// - 零 fresh pending → fail-open（exit 0）
    63	/// - stale 文件 → 删除 + warn + fail-open（exit 0）
    64	/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
    65	///   多 pending 时用户一次决策广播给所有 request_id。
    66	///
    67	/// 返回进程退出码：0 = 允许，1 = 拒绝。
    68	/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
    69	pub fn run_check_heuristic(base: &Path) -> i32 {
    70	    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);
    71	
    72	    // 删除 stale 文件 + 打 warning。
    73	    for stale_path in &scan.stale_paths {
    74	        eprintln!(
    75	            "sieve-hook: warning: stale pending file deleted: {}",
    76	            stale_path.display()
    77	        );
    78	        let _ = std::fs::remove_file(stale_path);
    79	    }
    80	
    81	    if scan.fresh.is_empty() {
    82	        // 零 pending：Sieve 代理未标记任何请求，fail-open。
    83	        return 0;
    84	    }
    85	
    86	    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    87	    // （非 TTY 路径：直接按策略决定，不弹提示。）
    88	    let outcome = decide_outcome_for_requests(&scan.fresh);
    89	
    90	    // 广播决策给所有 pending request_id。
    91	    for req in &scan.fresh {
    92	        if let Err(e) = write_decision(req.request_id, &outcome, base) {
    93	            eprintln!(
    94	                "sieve-hook: failed to write decision for {}: {e}",
    95	                req.request_id
    96	            );
    97	        }
    98	    }
    99	
   100	    match outcome {
   101	        DecisionOutcome::Allow => 0,
   102	        DecisionOutcome::Deny => 1,
   103	    }
   104	}
   105	
   106	/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
   107	fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
   108	    for req in reqs {
   109	        match req.default_on_timeout {
   110	            protocol::DefaultOnTimeout::Allow => {}
   111	            _ => return DecisionOutcome::Deny,
   112	        }
   113	    }
   114	    DecisionOutcome::Allow
   115	}
   116	
   117	#[cfg(test)]
   118	mod tests {
   119	    use chrono::{Duration, Utc};
   120	    use std::path::Path;
   219	/// 启发式路径：无 request_id 时扫目录。
   220	///
   221	/// - 零 fresh pending → fail-open（exit 0）
   222	/// - stale 文件 → 删除 + warn + fail-open（exit 0）
   223	/// - 有 fresh pending → 合并显示所有 detection，TTY 弹窗确认，广播决策
   224	///
   225	/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
   226	fn run_heuristic(base: &std::path::Path) -> i32 {
   227	    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);
   228	
   229	    // 删除 stale 文件 + 打 warning。
   230	    for stale_path in &scan.stale_paths {
   231	        eprintln!(
   232	            "sieve-hook: warning: stale pending file deleted: {}",
   233	            stale_path.display()
   234	        );
   235	        let _ = std::fs::remove_file(stale_path);
   236	    }
   237	
   238	    if scan.fresh.is_empty() {
   239	        // 零 pending：Sieve 代理未标记任何请求，fail-open。
   240	        return 0;
   241	    }
   242	
   243	    // 合并所有 detection 到一个"虚拟"请求以统一显示。
   244	    // timeout_seconds 和 default_on_timeout 取最严的策略（任一 Block/Redact → Deny）。
   245	    let merged = merge_requests(&scan.fresh);
   246	    print_summary(&merged);
   247	    let outcome = prompt_user(&merged);
   248	
   249	    // 广播决策给所有 pending request_id。
   250	    for req in &scan.fresh {
   251	        if let Err(e) = write_decision(req.request_id, &outcome, base) {
   252	            eprintln!(
   253	                "sieve-hook: failed to write decision for {}: {e}",
   254	                req.request_id
   255	            );
   256	        }
   257	    }
   258	
   259	    match outcome {
   260	        DecisionOutcome::Allow => 0,
   261	        DecisionOutcome::Deny => 1,
   262	    }
   263	}
   264	
   265	/// 合并多个 DecisionRequest 的 detection，取最严 default_on_timeout。
   266	fn merge_requests(reqs: &[protocol::DecisionRequest]) -> protocol::DecisionRequest {
   267	    let mut all_detections = Vec::new();
   268	    let mut worst_timeout = protocol::DefaultOnTimeout::Allow;
   269	    let mut min_timeout_secs = u32::MAX;
   270	
   271	    for req in reqs {
   272	        all_detections.extend(req.detections.clone());
   273	        // 最严策略：Block/Redact > Allow。
   274	        match req.default_on_timeout {
   275	            protocol::DefaultOnTimeout::Allow => {}
   276	            other => worst_timeout = other,
   277	        }
   278	        if req.timeout_seconds < min_timeout_secs {
   279	            min_timeout_secs = req.timeout_seconds;
   280	        }
   281	    }
   282	
   283	    let timeout_secs = if min_timeout_secs == u32::MAX {
   284	        30
   285	    } else {
   286	        min_timeout_secs
   287	    };
   288	
   289	    protocol::DecisionRequest {
   290	        // 启发式合并场景使用第一个请求的 id（仅用于日志）。
   291	        request_id: reqs[0].request_id,
   292	        created_at: reqs[0].created_at,
   293	        timeout_seconds: timeout_secs,
   294	        default_on_timeout: worst_timeout,
   295	        detections: all_detections,
   296	    }
   297	}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/main.rs | sed -n '1,180p'; nl -ba crates/sieve-cli/src/config.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
     2	//!
     3	//! 子命令：
     4	//! - `sieve start [--config <path>] [--dry-run]`：启动 daemon
     5	//! - `sieve version`：打印版本号
     6	//! - `sieve setup [--dry-run] [--yes]`：自动配置 Claude Code（仅 macOS，ADR-015）
     7	//! - `sieve doctor`：诊断 Sieve 安装状态（仅 macOS）
     8	//! - `sieve uninstall [--dry-run] [--yes]`：回滚 setup 改动（仅 macOS）
     9	
    10	// unsafe_code 在生产代码中禁止（等效 forbid），测试代码通过 #[allow(unsafe_code)] 豁免
    11	// 以支持 Rust 1.80+ 的 std::env::set_var 必须用 unsafe {} 的要求。
    12	#![deny(unsafe_code)]
    13	
    14	use anyhow::{Context, Result};
    15	use clap::Parser;
    16	use std::collections::HashSet;
    17	use std::path::Path;
    18	use std::sync::Arc;
    19	
    20	mod audit;
    21	mod cli;
    22	mod commands;
    23	mod config;
    24	mod daemon;
    25	mod engine_adapter;
    26	
    27	use audit::AuditStore;
    28	use cli::{Cli, Command};
    29	use engine_adapter::{InboundAdapter, OutboundAdapter};
    30	use sieve_core::pipeline::outbound::OutboundFilter;
    31	use sieve_rules::engine::VectorscanEngine;
    32	use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
    33	
    34	#[tokio::main]
    35	async fn main() -> Result<()> {
    36	    init_tracing();
    37	
    38	    let cli = Cli::parse();
    39	
    40	    match cli.command {
    41	        Command::Start {
    42	            config: cfg_path,
    43	            dry_run: cli_dry_run,
    44	        } => {
    45	            let mut cfg = config::Config::load(&cfg_path)
    46	                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;
    47	
    48	            // CLI --dry-run 出现（true）时覆盖 config 中的值；
    49	            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
    50	            if cli_dry_run {
    51	                cfg.dry_run = true;
    52	            }
    53	
    54	            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)
    55	
    56	            let audit_path = cfg.audit_db_path()?;
    57	            let _audit = AuditStore::init(&audit_path)
    58	                .with_context(|| format!("init audit store at {}", audit_path.display()))?;
    59	
    60	            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
    61	            let rules_path = cfg.resolved_rules_path();
    62	            tracing::info!(path = %rules_path.display(), "loading outbound rules");
    63	            let rules = load_outbound_rules(&rules_path).with_context(|| {
    64	                format!(
    65	                    "failed to load outbound rules from {}; \
    66	                     set rules_path in sieve.toml or ensure the default path exists",
    67	                    rules_path.display()
    68	                )
    69	            })?;
    70	            tracing::info!(count = rules.len(), "outbound rules loaded");
    71	
    72	            // 编译出站 vectorscan db（fail-closed）
    73	            let engine = VectorscanEngine::compile(rules.clone())
    74	                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
    75	            let adapter = OutboundAdapter::new(Arc::new(engine), rules);
    76	
    77	            // 加载 .sieveignore（出站 + 入站共用同一份）
    78	            let sieveignore_path = cfg.resolved_sieveignore_path();
    79	            let sieveignore = load_sieveignore(&sieveignore_path);
    80	            tracing::info!(
    81	                path = %sieveignore_path.display(),
    82	                entries = sieveignore.len(),
    83	                "sieveignore loaded"
    84	            );
    85	            let sieveignore_arc = Arc::new(sieveignore);
    86	
    87	            let filter = Arc::new(OutboundFilter::new(
    88	                Arc::new(adapter),
    89	                Arc::clone(&sieveignore_arc),
    90	            ));
    91	
    92	            // 加载入站规则（fail-closed，ADR-007）
    93	            let inbound_rules_path = cfg.resolved_inbound_rules_path();
    94	            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
    95	            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
    96	                format!(
    97	                    "failed to load inbound rules from {}; \
    98	                         set inbound_rules_path in sieve.toml or ensure the default path exists",
    99	                    inbound_rules_path.display()
   100	                )
   101	            })?;
   102	
   103	            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
   104	            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
   105	                .iter()
   106	                .cloned()
   107	                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
   108	            tracing::info!(
   109	                count = vectorscan_rules.len(),
   110	                placeholders = placeholder_rules.len(),
   111	                "inbound rules partitioned"
   112	            );
   113	
   114	            // 编译入站 vectorscan db（独立实例，fail-closed）
   115	            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
   116	                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;
   117	            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
   118	            let inbound_adapter =
   119	                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);
   120	
   121	            // YOLO mode 运行时审计（防御性双保险）
   122	            audit_yolo_disabled(&cfg)?;
   123	
   124	            daemon::run(
   125	                cfg,
   126	                filter,
   127	                Arc::new(inbound_adapter),
   128	                Arc::clone(&sieveignore_arc),
   129	            )
   130	            .await?;
   131	        }
   132	        Command::Version => {
   133	            println!("sieve {}", env!("CARGO_PKG_VERSION"));
   134	        }
   135	        Command::Setup(args) => {
   136	            commands::setup::run(args)?;
   137	        }
   138	        Command::Doctor => {
   139	            commands::doctor::run()?;
   140	        }
   141	        Command::Uninstall(args) => {
   142	            commands::uninstall::run(args)?;
   143	        }
   144	    }
   145	
   146	    Ok(())
   147	}
   148	
   149	/// 防御性检查：确认配置中无任何试图禁用 Critical 检测的字段。
   150	///
   151	/// Phase 1 实现：`Config` 已用 `#[serde(deny_unknown_fields)]` 在反序列化时拒绝
   152	/// 所有未知字段（含 `disable_critical` / `yolo` / `bypass` 等），此函数作为
   153	/// 运行时第二道防线，仅记录审计日志。
   154	///
   155	/// # Errors
   156	/// 当前实现不返回错误；签名保留 `Result<()>` 便于 Week 4 扩展检查逻辑。
   157	fn audit_yolo_disabled(cfg: &config::Config) -> Result<()> {
   158	    // dry_run 模式下 fail-closed 规则仍强制 Block（ADR-007 §2）
   159	    if cfg.dry_run {
   160	        tracing::warn!(
   161	            "dry_run=true: non-fail-closed Critical detections will only be logged, \
   162	             NOT blocked. Fail-closed rules (IN-CR-01/02/05/IN-GEN-01/03/OUT-01~12) \
   163	             remain enforced regardless."
   164	        );
   165	    }
   166	    tracing::info!("YOLO mode audit: passed (no critical-disable fields detected)");
   167	    Ok(())
   168	}
   169	
   170	/// 从文件加载 `.sieveignore` fingerprint 白名单。
   171	///
   172	/// 文件不存在时静默返回空集合（正常状态）；读取失败时打印 WARN 并返回空集合。
   173	/// 每行一个 fingerprint，支持 `#` 注释行和空行。
   174	fn load_sieveignore(path: &Path) -> HashSet<String> {
   175	    if !path.exists() {
   176	        return HashSet::new();
   177	    }
   178	    match std::fs::read_to_string(path) {
   179	        Ok(s) => s
   180	            .lines()
     1	//! 配置加载（关联 docs/design/data-model.md §配置）。
     2	//!
     3	//! Phase 1 字段：`upstream_url` / `port` / `bind_addr` / `log_path` /
     4	//! `tls_verify_upstream`。
     5	//! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
     6	//! Week 3 新增：`inbound_rules_path`（入站规则路径）。
     7	//! Week 5 新增：`ipc_socket_path` / `pending_dir` / `decisions_dir` /
     8	//!              `preset` / `launchd_plist_path` / `gui_socket_enabled` /
     9	//!              `audit_db_path`（SPEC-003 / data-model.md §5）。
    10	//! `#[serde(deny_unknown_fields)]` 确保配置文件中的危险字段（如
    11	//! `disable_critical`）被强制拒绝，不会静默忽略。
    12	
    13	use anyhow::{anyhow, Context, Result};
    14	use serde::{Deserialize, Serialize};
    15	use std::path::{Path, PathBuf};
    16	
    17	/// 检测预设级别（SPEC-003 / data-model.md §5）。
    18	///
    19	/// - `Strict`：所有规则最高灵敏度
    20	/// - `Default`：推荐平衡配置（默认）
    21	/// - `Relaxed`：降低误报，适合受信任环境
    22	/// - `Custom`：完全自定义（忽略内置默认值）
    23	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    24	#[serde(rename_all = "snake_case")]
    25	pub enum Preset {
    26	    Strict,
    27	    #[default]
    28	    Default,
    29	    Relaxed,
    30	    Custom,
    31	}
    32	
    33	/// Sieve 顶层配置。
    34	///
    35	/// 对应 `sieve.toml`（ADR-003 / data-model.md §配置）。
    36	/// 文件不存在时 [`Config::load`] 返回 [`Config::default`]。
    37	#[derive(Debug, Clone, Deserialize)]
    38	#[serde(deny_unknown_fields)]
    39	pub struct Config {
    40	    /// 上游 LLM API 端点（默认 `https://api.anthropic.com`）。
    41	    #[serde(default = "default_upstream")]
    42	    pub upstream_url: String,
    43	
    44	    /// 本地代理监听端口（默认 11453，PRD §6.1）。
    45	    #[serde(default = "default_port")]
    46	    pub port: u16,
    47	
    48	    /// 监听地址。**强制 `127.0.0.1`**（ADR-003 / PRD §9 #2 完全本地）。
    49	    /// 任何其他值都会触发 [`Config::enforce_safety_invariants`] 中的 exit(1)。
    50	    #[serde(default = "default_bind_addr")]
    51	    pub bind_addr: String,
    52	
    53	    /// 审计日志路径（SQLite），`None` 时由 daemon 决定默认路径。
    54	    #[serde(default)]
    55	    pub log_path: Option<PathBuf>,
    56	
    57	    /// 是否校验上游 TLS 证书（默认 `true`；测试可关，会打印 WARN）。
    58	    #[serde(default = "default_tls_verify")]
    59	    pub tls_verify_upstream: bool,
    60	
    61	    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
    62	    #[serde(default)]
    63	    pub rules_path: Option<PathBuf>,
    64	
    65	    /// `.sieveignore` 路径（默认 `~/.sieve/sieveignore`）。
    66	    #[serde(default)]
    67	    pub sieveignore_path: Option<PathBuf>,
    68	
    69	    /// 仅记录命中，不实际拦截（dry-run 模式，默认 `false`）。
    70	    ///
    71	    /// `true` 时 [`Config::enforce_safety_invariants`] 会打印 WARN。
    72	    /// CLI `--dry-run` flag 出现时会覆盖此值为 `true`（见 cli.rs）。
    73	    #[serde(default)]
    74	    pub dry_run: bool,
    75	
    76	    /// 入站规则 toml 路径（Week 3，默认 `crates/sieve-rules/rules/inbound.toml`）。
    77	    #[serde(default)]
    78	    pub inbound_rules_path: Option<PathBuf>,
    79	
    80	    // ── Week 5 新字段（SPEC-003 / data-model.md §5）────────────────────────
    81	    // Week 6+ 会在 daemon 启动时读取这些字段；当前仅反序列化使用，暂时 allow dead_code。
    82	    /// Unix socket 路径（GUI / sieve-hook 连接用，默认 `~/.sieve/ipc.sock`）。
    83	    #[serde(default = "default_ipc_socket")]
    84	    #[allow(dead_code)]
    85	    pub ipc_socket_path: PathBuf,
    86	
    87	    /// 待决策文件目录（默认 `~/.sieve/pending/`）。
    88	    #[serde(default = "default_pending_dir")]
    89	    #[allow(dead_code)]
    90	    pub pending_dir: PathBuf,
    91	
    92	    /// 决策文件目录（默认 `~/.sieve/decisions/`）。
    93	    #[serde(default = "default_decisions_dir")]
    94	    #[allow(dead_code)]
    95	    pub decisions_dir: PathBuf,
    96	
    97	    /// 检测预设级别（默认 `Default`）。
    98	    #[serde(default)]
    99	    #[allow(dead_code)]
   100	    pub preset: Preset,
   101	
   102	    /// launchd plist 路径（macOS，默认 `~/Library/LaunchAgents/com.sieve.daemon.plist`）。
   103	    #[serde(default = "default_launchd_plist")]
   104	    #[allow(dead_code)]
   105	    pub launchd_plist_path: PathBuf,
   106	
   107	    /// 是否启用 GUI Unix socket（默认 `false`；Week 6+ 启用）。
   108	    #[serde(default = "default_gui_socket_enabled")]
   109	    #[allow(dead_code)]
   110	    pub gui_socket_enabled: bool,
   111	
   112	    /// SQLite 审计数据库路径（Week 5；`None` 时沿用 `log_path` 或 `~/.sieve/audit.db`）。
   113	    #[serde(default)]
   114	    #[allow(dead_code)]
   115	    pub audit_db_path: Option<PathBuf>,
   116	}
   117	
   118	fn home_path() -> PathBuf {
   119	    std::env::var_os("HOME")
   120	        .map(PathBuf::from)
   121	        .unwrap_or_else(|| PathBuf::from("."))
   122	}
   123	
   124	fn sieve_home() -> PathBuf {
   125	    home_path().join(".sieve")
   126	}
   127	
   128	fn default_ipc_socket() -> PathBuf {
   129	    sieve_home().join("ipc.sock")
   130	}
   131	
   132	fn default_pending_dir() -> PathBuf {
   133	    sieve_home().join("pending")
   134	}
   135	
   136	fn default_decisions_dir() -> PathBuf {
   137	    sieve_home().join("decisions")
   138	}
   139	
   140	fn default_launchd_plist() -> PathBuf {
   141	    home_path()
   142	        .join("Library")
   143	        .join("LaunchAgents")
   144	        .join("com.sieve.daemon.plist")
   145	}
   146	
   147	fn default_gui_socket_enabled() -> bool {
   148	    false
   149	}
   150	
   151	fn default_upstream() -> String {
   152	    "https://api.anthropic.com".to_string()
   153	}
   154	
   155	fn default_port() -> u16 {
   156	    11453
   157	}
   158	
   159	fn default_bind_addr() -> String {
   160	    "127.0.0.1".to_string()
   161	}
   162	
   163	fn default_tls_verify() -> bool {
   164	    true
   165	}
   166	
   167	impl Default for Config {
   168	    fn default() -> Self {
   169	        Self {
   170	            upstream_url: default_upstream(),
   171	            port: default_port(),
   172	            bind_addr: default_bind_addr(),
   173	            log_path: None,
   174	            tls_verify_upstream: default_tls_verify(),
   175	            rules_path: None,
   176	            sieveignore_path: None,
   177	            dry_run: false,
   178	            inbound_rules_path: None,
   179	            ipc_socket_path: default_ipc_socket(),
   180	            pending_dir: default_pending_dir(),
   181	            decisions_dir: default_decisions_dir(),
   182	            preset: Preset::default(),
   183	            launchd_plist_path: default_launchd_plist(),
   184	            gui_socket_enabled: default_gui_socket_enabled(),
   185	            audit_db_path: None,
   186	        }
   187	    }
   188	}
   189	
   190	impl Config {
   191	    /// 从 TOML 文件加载配置；文件不存在时返回 [`Config::default`]。
   192	    ///
   193	    /// # Errors
   194	    /// 文件存在但读取或解析失败时返回错误。
   195	    pub fn load(path: &Path) -> Result<Self> {
   196	        if !path.exists() {
   197	            tracing::warn!(path = %path.display(), "config file not found, using defaults");
   198	            return Ok(Self::default());
   199	        }
   200	        let s = std::fs::read_to_string(path)
   201	            .with_context(|| format!("read config {}", path.display()))?;
   202	        let cfg: Self =
   203	            toml::from_str(&s).with_context(|| format!("parse config {}", path.display()))?;
   204	        Ok(cfg)
   205	    }
   206	
   207	    /// 强制安全不变量：`bind_addr` 必须是 `127.0.0.1`，否则打印 FATAL 并 `exit(1)`。
   208	    ///
   209	    /// 关联 ADR-003 / PRD §9 #2 / data-model.md §配置。
   210	    /// 不提供 fallback，不 warn 后继续：非 loopback 绑定是配置错误，
   211	    /// 悄悄启动会暴露代理到局域网，违反"完全本地"承诺。
   212	    pub fn enforce_safety_invariants(&self) {
   213	        if self.bind_addr != "127.0.0.1" {
   214	            eprintln!(
   215	                "FATAL: bind_addr must be 127.0.0.1 (got {:?}). \
   216	                 Sieve refuses to bind on a non-loopback address. See ADR-003.",
   217	                self.bind_addr
   218	            );
   219	            std::process::exit(1);
   220	        }
   221	
   222	        if !self.tls_verify_upstream {
   223	            tracing::warn!(
   224	                "tls_verify_upstream=false: upstream TLS certificate NOT verified. \
   225	                 Only use in controlled test environments."
   226	            );
   227	        }
   228	
   229	        if self.dry_run {
   230	            tracing::warn!("dry_run mode: detections logged but not blocked");
   231	        }
   232	    }
   233	
   234	    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
   235	    pub fn resolved_rules_path(&self) -> PathBuf {
   236	        if let Some(p) = &self.rules_path {
   237	            return p.clone();
   238	        }
   239	        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
   240	    }
   241	
   242	    /// 解析入站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/inbound.toml`（相对 cwd）。
   243	    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
   244	        if let Some(p) = &self.inbound_rules_path {
   245	            return p.clone();
   246	        }
   247	        PathBuf::from("crates/sieve-rules/rules/inbound.toml")
   248	    }
   249	
   250	    /// 解析 `.sieveignore` 路径。显式给定时直接用，否则回退到 `~/.sieve/sieveignore`。
   251	    ///
   252	    /// 若 `HOME` 不可读则 fallback 到 `.sieve/sieveignore`（相对 cwd）并打印 WARN。
   253	    pub fn resolved_sieveignore_path(&self) -> PathBuf {
   254	        if let Some(p) = &self.sieveignore_path {
   255	            return p.clone();
   256	        }
   257	        if let Some(home) = std::env::var_os("HOME") {
   258	            return PathBuf::from(home).join(".sieve").join("sieveignore");
   259	        }
   260	        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/config.rs | sed -n '260,380p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   260	        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");
   261	        PathBuf::from(".sieve").join("sieveignore")
   262	    }
   263	
   264	    /// 拼接监听 SocketAddr。
   265	    ///
   266	    /// # Errors
   267	    /// `bind_addr` 或 `port` 无法解析为合法 SocketAddr 时返回错误。
   268	    pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
   269	        format!("{}:{}", self.bind_addr, self.port)
   270	            .parse()
   271	            .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
   272	    }
   273	
   274	    /// 解析审计日志路径。`log_path` 显式给定时直接用,否则回退到 `~/.sieve/audit.db`。
   275	    ///
   276	    /// # Errors
   277	    /// `$HOME` 不存在或不可识别时返回错误。
   278	    pub fn audit_db_path(&self) -> Result<PathBuf> {
   279	        if let Some(p) = &self.log_path {
   280	            return Ok(p.clone());
   281	        }
   282	        let home = std::env::var_os("HOME")
   283	            .ok_or_else(|| anyhow!("HOME env var not set; specify log_path explicitly"))?;
   284	        Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
   285	    }
   286	}
   287	
   288	#[cfg(test)]
   289	mod tests {
   290	    use super::*;
   291	
   292	    #[test]
   293	    fn defaults_are_sane() {
   294	        let c = Config::default();
   295	        assert_eq!(c.bind_addr, "127.0.0.1");
   296	        assert_eq!(c.port, 11453);
   297	        assert_eq!(c.upstream_url, "https://api.anthropic.com");
   298	        assert!(c.tls_verify_upstream);
   299	        assert!(c.log_path.is_none());
   300	    }
   301	
   302	    #[test]
   303	    fn listen_addr_parses() {
   304	        let c = Config::default();
   305	        let addr = c.listen_addr().unwrap();
   306	        assert_eq!(addr.port(), 11453);
   307	        assert!(addr.ip().is_loopback());
   308	    }
   309	
   310	    #[test]
   311	    fn parse_minimal_toml() {
   312	        let toml_str = r#"
   313	            upstream_url = "https://api.anthropic.com"
   314	            port = 11453
   315	        "#;
   316	        let c: Config = toml::from_str(toml_str).unwrap();
   317	        assert_eq!(c.bind_addr, "127.0.0.1");
   318	        assert!(c.tls_verify_upstream);
   319	    }
   320	
   321	    #[test]
   322	    fn parse_full_toml() {
   323	        let toml_str = r#"
   324	            upstream_url = "https://api.anthropic.com"
   325	            port = 12000
   326	            bind_addr = "127.0.0.1"
   327	            tls_verify_upstream = false
   328	            log_path = "/tmp/audit.db"
   329	        "#;
   330	        let c: Config = toml::from_str(toml_str).unwrap();
   331	        assert_eq!(c.port, 12000);
   332	        assert!(!c.tls_verify_upstream);
   333	        assert_eq!(c.log_path.unwrap().to_str().unwrap(), "/tmp/audit.db");
   334	    }
   335	
   336	    #[test]
   337	    fn unknown_field_rejected() {
   338	        let toml_str = r#"
   339	            upstream_url = "https://api.anthropic.com"
   340	            disable_critical = true
   341	        "#;
   342	        let result: Result<Config, _> = toml::from_str(toml_str);
   343	        assert!(
   344	            result.is_err(),
   345	            "must reject unknown fields (deny_unknown_fields)"
   346	        );
   347	    }
   348	
   349	    #[test]
   350	    fn parse_dry_run_and_rules_path() {
   351	        let toml_str = r#"
   352	            upstream_url = "https://api.anthropic.com"
   353	            port = 11453
   354	            dry_run = true
   355	            rules_path = "/x.toml"
   356	        "#;
   357	        let c: Config = toml::from_str(toml_str).unwrap();
   358	        assert!(c.dry_run, "dry_run should be true");
   359	        assert_eq!(c.rules_path.as_ref().unwrap().to_str().unwrap(), "/x.toml");
   360	    }
   361	
   362	    #[test]
   363	    fn resolved_rules_path_explicit() {
   364	        let c = Config {
   365	            rules_path: Some(PathBuf::from("/custom/rules.toml")),
   366	            ..Config::default()
   367	        };
   368	        assert_eq!(c.resolved_rules_path(), PathBuf::from("/custom/rules.toml"));
   369	    }
   370	
   371	    #[test]
   372	    fn resolved_rules_path_fallback() {
   373	        let c = Config::default();
   374	        let p = c.resolved_rules_path();
   375	        assert!(
   376	            p.ends_with("outbound.toml"),
   377	            "fallback should end with outbound.toml, got {:?}",
   378	            p
   379	        );
   380	    }

exec
/bin/zsh -lc 'cargo check -p sieve-core --no-default-features' in /Users/doskey/src/sieve
 succeeded in 1547ms:
    Checking sieve-ipc v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-ipc)
    Checking sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/sieve_setup_dry_run.rs | sed -n '1,260p'; nl -ba crates/sieve-cli/tests/audit_append_only.rs | sed -n '1,220p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	//! `sieve setup --dry-run` 集成测试（SPEC-003 §setup）。
     2	//!
     3	//! 仅 macOS 编译运行（`#[cfg(target_os = "macos")]`）。
     4	//! 验证：dry-run 模式不修改原文件。
     5	
     6	#![cfg(target_os = "macos")]
     7	
     8	use std::fs;
     9	use tempfile::tempdir;
    10	
    11	/// 构造一个 fake `settings.json` 并运行 setup --dry-run；
    12	/// 断言原文件内容未被改动。
    13	#[test]
    14	fn dry_run_does_not_modify_settings() {
    15	    let dir = tempdir().unwrap();
    16	    let fake_home = dir.path().to_path_buf();
    17	
    18	    // 建 .claude/ 目录和 fake settings.json
    19	    let claude_dir = fake_home.join(".claude");
    20	    fs::create_dir_all(&claude_dir).unwrap();
    21	    let settings_path = claude_dir.join("settings.json");
    22	    let original_content = r#"{"env": {"SOME_KEY": "some_value"}}"#;
    23	    fs::write(&settings_path, original_content).unwrap();
    24	
    25	    // 也建 .sieve/ 目录（setup 需要写 setup.log）
    26	    let sieve_dir = fake_home.join(".sieve");
    27	    fs::create_dir_all(&sieve_dir).unwrap();
    28	
    29	    // 设置 HOME 为 fake_home，SIEVE_HOME 为 fake .sieve
    30	    // 注意：因为 setup 内部直接读 HOME env var，我们通过子进程的方式测试
    31	    // 这里使用 std::env::set_var（测试环境可接受）
    32	    let orig_home = std::env::var("HOME").unwrap_or_default();
    33	    let orig_sieve_home = std::env::var("SIEVE_HOME").unwrap_or_default();
    34	
    35	    // SAFETY: 仅在单线程测试环境中修改 env var
    36	    unsafe {
    37	        std::env::set_var("HOME", fake_home.to_str().unwrap());
    38	        std::env::set_var("SIEVE_HOME", sieve_dir.to_str().unwrap());
    39	    }
    40	
    41	    // 调用 setup::run（dry_run=true, yes=true）
    42	    // dry-run 应该不修改任何文件
    43	    // 由于 setup::run 会调用 `launchctl` 等系统命令，这里仅验证核心逻辑：
    44	    // dry-run 模式下 settings.json 不变
    45	    //
    46	    // 注意：完整 setup 调用会因为 launchctl 不在 CI 测试环境中工作而失败；
    47	    // 这里我们直接测试文件不变的断言（dry-run 在打印 diff 后直接 return）
    48	    use sieve_cli_test_helpers::*;
    49	    let result = run_setup_dry_run(&settings_path);
    50	
    51	    // 恢复 env var
    52	    unsafe {
    53	        std::env::set_var("HOME", &orig_home);
    54	        if orig_sieve_home.is_empty() {
    55	            std::env::remove_var("SIEVE_HOME");
    56	        } else {
    57	            std::env::set_var("SIEVE_HOME", &orig_sieve_home);
    58	        }
    59	    }
    60	
    61	    // 无论 setup 是否返回错误，settings.json 必须保持原始内容
    62	    let actual_content = fs::read_to_string(&settings_path).unwrap();
    63	    assert_eq!(
    64	        actual_content, original_content,
    65	        "dry-run 模式不应修改 settings.json"
    66	    );
    67	
    68	    // dry-run 应该成功（无需 launchctl）
    69	    result.expect("setup --dry-run 应该成功");
    70	}
    71	
    72	/// 包含测试辅助函数的伪模块（直接内联，避免额外文件）。
    73	mod sieve_cli_test_helpers {
    74	    use anyhow::Result;
    75	    use std::path::Path;
    76	
    77	    /// 直接调用 setup::run with dry_run=true, yes=true。
    78	    pub fn run_setup_dry_run(_settings_path: &Path) -> Result<()> {
    79	        // 直接引用 sieve-cli 内部模块（integration test 与 lib 共享 crate）
    80	        // 集成测试通过 cargo test 链接到 sieve-cli 的库部分
    81	        // 由于 sieve-cli 是 binary crate（只有 main.rs），这里用 Command 子进程测试
    82	        // 实际验证逻辑：dry-run 不修改文件（已在调用方验证）
    83	        Ok(())
    84	    }
    85	}
     1	//! `AuditStore` append-only 集成测试（ADR-007 / ADR-014）。
     2	//!
     3	//! 验证：写 3 条 → SELECT 能读到；UPDATE / DELETE 被触发器拒绝。
     4	//!
     5	//! 注意：由于 sieve-cli 是纯 binary crate，这里通过子进程或直接用 rusqlite 验证。
     6	//! audit.rs 中已有 `#[cfg(test)]` 单元测试覆盖同等逻辑；本集成测试作为补充验证。
     7	
     8	use rusqlite::{params, Connection};
     9	use tempfile::tempdir;
    10	
    11	const CREATE_DDL: &str = r#"
    12	CREATE TABLE IF NOT EXISTS audit_events (
    13	    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    14	    timestamp_rfc3339   TEXT    NOT NULL,
    15	    direction           TEXT    NOT NULL,
    16	    rule_id             TEXT    NOT NULL,
    17	    severity            TEXT    NOT NULL,
    18	    disposition         TEXT    NOT NULL,
    19	    decision            TEXT,
    20	    request_id          TEXT    NOT NULL,
    21	    raw_json            TEXT
    22	);
    23	"#;
    24	
    25	const TRIGGERS_DDL: &str = r#"
    26	CREATE TRIGGER IF NOT EXISTS no_update
    27	BEFORE UPDATE ON audit_events
    28	BEGIN
    29	    SELECT RAISE(FAIL, 'audit_events is append-only: UPDATE is forbidden');
    30	END;
    31	
    32	CREATE TRIGGER IF NOT EXISTS no_delete
    33	BEFORE DELETE ON audit_events
    34	BEGIN
    35	    SELECT RAISE(FAIL, 'audit_events is append-only: DELETE is forbidden');
    36	END;
    37	"#;
    38	
    39	const INSERT_SQL: &str = r#"
    40	INSERT INTO audit_events
    41	    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
    42	VALUES
    43	    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
    44	"#;
    45	
    46	fn setup_db(path: &std::path::Path) -> Connection {
    47	    let conn = Connection::open(path).unwrap();
    48	    conn.execute_batch(CREATE_DDL).unwrap();
    49	    conn.execute_batch(TRIGGERS_DDL).unwrap();
    50	    conn
    51	}
    52	
    53	#[test]
    54	fn write_3_events_and_read_back() {
    55	    let dir = tempdir().unwrap();
    56	    let db_path = dir.path().join("test_audit.db");
    57	    let conn = setup_db(&db_path);
    58	
    59	    for i in 1..=3u32 {
    60	        conn.execute(
    61	            INSERT_SQL,
    62	            params![
    63	                format!("2026-04-27T00:0{i}:00Z"),
    64	                "outbound",
    65	                format!("OUT-0{i}"),
    66	                "Critical",
    67	                "redact",
    68	                Option::<String>::None,
    69	                format!("req-{i}"),
    70	                Option::<String>::None,
    71	            ],
    72	        )
    73	        .unwrap();
    74	    }
    75	
    76	    let count: i64 = conn
    77	        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
    78	        .unwrap();
    79	    assert_eq!(count, 3, "应有 3 条记录");
    80	
    81	    let rule_id: String = conn
    82	        .query_row("SELECT rule_id FROM audit_events WHERE id = 2", [], |r| {
    83	            r.get(0)
    84	        })
    85	        .unwrap();
    86	    assert_eq!(rule_id, "OUT-02");
    87	}
    88	
    89	#[test]
    90	fn update_is_rejected_by_trigger() {
    91	    let dir = tempdir().unwrap();
    92	    let db_path = dir.path().join("test_trigger_update.db");
    93	    let conn = setup_db(&db_path);
    94	
    95	    conn.execute(
    96	        INSERT_SQL,
    97	        params![
    98	            "2026-04-27T00:00:00Z",
    99	            "inbound",
   100	            "IN-CR-01",
   101	            "Critical",
   102	            "pending",
   103	            Option::<String>::None,
   104	            "req-x",
   105	            Option::<String>::None,
   106	        ],
   107	    )
   108	    .unwrap();
   109	
   110	    let result = conn.execute(
   111	        "UPDATE audit_events SET rule_id = 'tampered' WHERE id = 1",
   112	        [],
   113	    );
   114	    assert!(result.is_err(), "UPDATE 应被触发器拒绝");
   115	    let msg = result.unwrap_err().to_string();
   116	    assert!(
   117	        msg.contains("append-only"),
   118	        "错误信息应含 'append-only'，实际: {msg}"
   119	    );
   120	}
   121	
   122	#[test]
   123	fn delete_is_rejected_by_trigger() {
   124	    let dir = tempdir().unwrap();
   125	    let db_path = dir.path().join("test_trigger_delete.db");
   126	    let conn = setup_db(&db_path);
   127	
   128	    conn.execute(
   129	        INSERT_SQL,
   130	        params![
   131	            "2026-04-27T00:00:00Z",
   132	            "inbound",
   133	            "IN-CR-02",
   134	            "Critical",
   135	            "resolved",
   136	            Some("Block"),
   137	            "req-y",
   138	            Option::<String>::None,
   139	        ],
   140	    )
   141	    .unwrap();
   142	
   143	    let result = conn.execute("DELETE FROM audit_events WHERE id = 1", []);
   144	    assert!(result.is_err(), "DELETE 应被触发器拒绝");
   145	    let msg = result.unwrap_err().to_string();
   146	    assert!(
   147	        msg.contains("append-only"),
   148	        "错误信息应含 'append-only'，实际: {msg}"
   149	    );
   150	}

exec
/bin/zsh -lc "nl -ba crates/sieve-core/src/pipeline/outbound_redact.rs | sed -n '1,300p'; nl -ba crates/sieve-core/src/pipeline/inbound_hold.rs | sed -n '1,280p'; nl -ba crates/sieve-core/src/pipeline/inbound_hook.rs | sed -n '1,220p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	//! 出站自动脱敏路径（AutoRedact disposition，OUT-01~05/12）。
     2	//!
     3	//! 提供两套 API：
     4	//! - [`redact_body_bytes`]：在 raw body bytes 中按绝对字节偏移替换（fuzz/单测保留）。
     5	//! - [`redact_segments`]：在解析后的文本段列表中按累计字符偏移替换，
     6	//!   返回替换后的文本段列表，由调用方重新序列化 JSON——这是 daemon AutoRedact 路径
     7	//!   的正确用法（修 #1：AutoRedact 偏移修复）。
     8	//!
     9	//! 关联：PRD v1.4 §6.1（出站 AutoRedact 路径）、ADR-016（二维处置矩阵）。
    10	
    11	/// 单个脱敏命中范围（half-open `[start, end)`）。
    12	#[derive(Debug, Clone, PartialEq, Eq)]
    13	pub struct RedactHit {
    14	    /// 命中规则 ID（如 `OUT-01`）。
    15	    pub rule_id: String,
    16	    /// 命中起始字节偏移（含）。
    17	    pub start: usize,
    18	    /// 命中结束字节偏移（不含）。
    19	    pub end: usize,
    20	}
    21	
    22	/// [`redact_body_bytes`] 的返回值。
    23	#[derive(Debug)]
    24	pub struct RedactResult {
    25	    /// 脱敏后的 body bytes。
    26	    pub body: Vec<u8>,
    27	    /// 实际发生脱敏的数量（合并后的 span 数）。
    28	    pub redacted_count: usize,
    29	    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
    30	    pub redacted_summary: String,
    31	}
    32	
    33	/// 在 `body` slice 中把 `pos` 向左移动到最近的 UTF-8 字符起始位置。
    34	///
    35	/// UTF-8 continuation byte 以 `10xxxxxx`（`0x80..=0xBF`）开头；
    36	/// 如 body 含非 ASCII 字符（如中文 JSON 字段），正则可能给出 continuation byte 偏移，
    37	/// 此函数保证不截断多字节字符。
    38	pub fn align_to_utf8_char_start(body: &[u8], pos: usize) -> usize {
    39	    if pos >= body.len() {
    40	        return body.len();
    41	    }
    42	    let mut p = pos;
    43	    while p > 0 && (body[p] & 0xC0) == 0x80 {
    44	        p -= 1;
    45	    }
    46	    p
    47	}
    48	
    49	/// 把命中范围的字节替换为占位符，返回 [`RedactResult`]。
    50	///
    51	/// # 算法
    52	/// 1. 每个 hit 的 `start`/`end` 先做 UTF-8 字符边界对齐（`align_to_utf8_char_start`）；
    53	/// 2. 按 `start` 升序排序；
    54	/// 3. 合并重叠 / 相邻 span（多个 span 合并时 `rule_id` 取最左命中）；
    55	/// 4. 逐段复制原始字节，用 `[REDACTED:<rule_id>]` 替换各合并 span。
    56	///
    57	/// 如果 `hits` 为空，原样返回 body（`body.to_vec()`，最小拷贝）。
    58	///
    59	/// 关联：ADR-016 §AutoRedact 路径。
    60	pub fn redact_body_bytes(body: &[u8], hits: &[RedactHit]) -> RedactResult {
    61	    if hits.is_empty() {
    62	        return RedactResult {
    63	            body: body.to_vec(),
    64	            redacted_count: 0,
    65	            redacted_summary: String::new(),
    66	        };
    67	    }
    68	
    69	    // 1. 对齐 UTF-8 边界
    70	    let mut sorted: Vec<RedactHit> = hits
    71	        .iter()
    72	        .map(|h| RedactHit {
    73	            rule_id: h.rule_id.clone(),
    74	            start: align_to_utf8_char_start(body, h.start.min(body.len())),
    75	            end: align_to_utf8_char_start(body, h.end.min(body.len())),
    76	        })
    77	        .collect();
    78	
    79	    // 2. 按 start 升序排序
    80	    sorted.sort_by_key(|h| h.start);
    81	
    82	    // 3. 合并重叠 / 相邻 span
    83	    let mut merged: Vec<(usize, usize, String)> = Vec::new();
    84	    for hit in &sorted {
    85	        let start = hit.start;
    86	        let end = hit.end;
    87	        if start >= end {
    88	            // 对齐后 span 变空，跳过
    89	            continue;
    90	        }
    91	        if let Some(last) = merged.last_mut() {
    92	            if start <= last.1 {
    93	                // 重叠或紧邻：扩展结束边界，rule_id 保持第一个
    94	                if end > last.1 {
    95	                    last.1 = end;
    96	                }
    97	            } else {
    98	                merged.push((start, end, hit.rule_id.clone()));
    99	            }
   100	        } else {
   101	            merged.push((start, end, hit.rule_id.clone()));
   102	        }
   103	    }
   104	
   105	    let redacted_count = merged.len();
   106	    let redacted_summary = merged
   107	        .iter()
   108	        .map(|(_, _, rule_id)| rule_id.as_str())
   109	        .collect::<Vec<_>>()
   110	        .join(", ");
   111	
   112	    // 4. 重组 body
   113	    let mut result: Vec<u8> = Vec::with_capacity(body.len());
   114	    let mut cursor = 0usize;
   115	
   116	    for (start, end, rule_id) in &merged {
   117	        if cursor < *start {
   118	            result.extend_from_slice(&body[cursor..*start]);
   119	        }
   120	        let placeholder = format!("[REDACTED:{rule_id}]");
   121	        result.extend_from_slice(placeholder.as_bytes());
   122	        cursor = *end;
   123	    }
   124	    if cursor < body.len() {
   125	        result.extend_from_slice(&body[cursor..]);
   126	    }
   127	
   128	    RedactResult {
   129	        body: result,
   130	        redacted_count,
   131	        redacted_summary,
   132	    }
   133	}
   134	
   135	/// 文本段级脱敏结果（对应 [`redact_segments`] 的输出）。
   136	#[derive(Debug)]
   137	pub struct SegmentRedactResult {
   138	    /// 脱敏后的文本段列表，顺序与输入 `segments` 一一对应。
   139	    pub texts: Vec<String>,
   140	    /// 实际发生脱敏的总数量（合并后的 span 数，跨所有段）。
   141	    pub redacted_count: usize,
   142	    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
   143	    pub redacted_summary: String,
   144	}
   145	
   146	/// 在解析后的文本段列表中按**累计字符偏移**做脱敏替换。
   147	///
   148	/// # 背景（修 #1：AutoRedact 偏移修复）
   149	///
   150	/// [`Detection.span`] 的 `start`/`end` 是 `extract_text_content()` 返回的
   151	/// **累计文本字符偏移**（即 `body_byte_offset + vectorscan_offset`），
   152	/// 而非 raw JSON body 的字节偏移。直接把这些偏移喂给 [`redact_body_bytes`]
   153	/// 会写错 raw body 的字节范围，无法正确擦除 secret。
   154	///
   155	/// 正确做法：在每个文本段字符串内计算段内偏移后做字符串替换，
   156	/// 然后由调用方把替换后的文本重新填入 JSON 并重新序列化。
   157	///
   158	/// # 参数
   159	/// - `segments`：`(segment_global_start_offset, segment_text)` 列表，
   160	///   顺序与 `AnthropicRequest::extract_text_content()` 返回值一致。
   161	/// - `hits`：要脱敏的命中列表，`start`/`end` 是累计字符偏移（`Detection.span`）。
   162	///
   163	/// # 返回
   164	/// [`SegmentRedactResult`]，其中 `texts` 顺序对应输入 `segments`。
   165	///
   166	/// 关联：PRD v1.4 §6.1（AutoRedact 路径）、ADR-016（二维处置矩阵）。
   167	pub fn redact_segments(segments: &[(usize, String)], hits: &[RedactHit]) -> SegmentRedactResult {
   168	    if hits.is_empty() {
   169	        return SegmentRedactResult {
   170	            texts: segments.iter().map(|(_, t)| t.clone()).collect(),
   171	            redacted_count: 0,
   172	            redacted_summary: String::new(),
   173	        };
   174	    }
   175	
   176	    let mut total_redacted = 0usize;
   177	    let mut all_rule_ids: Vec<String> = Vec::new();
   178	    let mut result_texts: Vec<String> = Vec::with_capacity(segments.len());
   179	
   180	    for (seg_idx, (seg_start, seg_text)) in segments.iter().enumerate() {
   181	        let seg_end = seg_start + seg_text.len();
   182	
   183	        // 过滤出与当前段有交集的 hit（累计偏移范围与段范围重叠）
   184	        let seg_hits: Vec<RedactHit> = hits
   185	            .iter()
   186	            .filter(|h| h.start < seg_end && h.end > *seg_start)
   187	            .map(|h| {
   188	                // 把全局偏移转换为段内字符偏移（clamp 到段边界）
   189	                let local_start = h.start.saturating_sub(*seg_start).min(seg_text.len());
   190	                let local_end = h.end.saturating_sub(*seg_start).min(seg_text.len());
   191	                RedactHit {
   192	                    rule_id: h.rule_id.clone(),
   193	                    start: local_start,
   194	                    end: local_end,
   195	                }
   196	            })
   197	            .collect();
   198	
   199	        if seg_hits.is_empty() {
   200	            result_texts.push(seg_text.clone());
   201	            continue;
   202	        }
   203	
   204	        // 在 UTF-8 字符串上做 redact（按字节偏移，text 是 UTF-8 已验证）
   205	        let text_bytes = seg_text.as_bytes();
   206	        let redact_result = redact_body_bytes(text_bytes, &seg_hits);
   207	
   208	        total_redacted += redact_result.redacted_count;
   209	        if !redact_result.redacted_summary.is_empty() {
   210	            all_rule_ids.push(redact_result.redacted_summary.clone());
   211	        }
   212	
   213	        // redact_body_bytes 保证输出有效 UTF-8（placeholder 是 ASCII，原始文本是 UTF-8）
   214	        // Safety: redact_body_bytes 对齐 UTF-8 边界，placeholder 是纯 ASCII
   215	        let new_text = String::from_utf8(redact_result.body).unwrap_or_else(|_| seg_text.clone()); // 极端回退：保留原文
   216	        result_texts.push(new_text);
   217	
   218	        // suppress unused variable lint for seg_idx
   219	        let _ = seg_idx;
   220	    }
   221	
   222	    SegmentRedactResult {
   223	        texts: result_texts,
   224	        redacted_count: total_redacted,
   225	        redacted_summary: all_rule_ids.join(", "),
   226	    }
   227	}
   228	
   229	#[cfg(test)]
   230	mod tests {
   231	    use super::*;
   232	
   233	    fn hit(rule_id: &str, start: usize, end: usize) -> RedactHit {
   234	        RedactHit {
   235	            rule_id: rule_id.to_string(),
   236	            start,
   237	            end,
   238	        }
   239	    }
   240	
   241	    // ── 1. 单 span ───────────────────────────────────────────────────────────
   242	
   243	    #[test]
   244	    fn single_span_middle() {
   245	        // "hello secret world"
   246	        //  0     6     12   17
   247	        let body = b"hello secret world";
   248	        let hits = [hit("OUT-01", 6, 12)]; // "secret"
   249	        let r = redact_body_bytes(body, &hits);
   250	        assert_eq!(r.redacted_count, 1);
   251	        assert_eq!(r.redacted_summary, "OUT-01");
   252	        let s = String::from_utf8(r.body).unwrap();
   253	        assert_eq!(s, "hello [REDACTED:OUT-01] world");
   254	    }
   255	
   256	    // ── 2. 多 span（不重叠）──────────────────────────────────────────────────
   257	
   258	    #[test]
   259	    fn multiple_non_overlapping_spans() {
   260	        // "a secret b key c"
   261	        //  0 2      8 10  13 15
   262	        let body = b"a secret b key c";
   263	        let hits = [hit("OUT-01", 2, 8), hit("OUT-03", 11, 14)];
   264	        let r = redact_body_bytes(body, &hits);
   265	        assert_eq!(r.redacted_count, 2);
   266	        let s = String::from_utf8(r.body).unwrap();
   267	        assert_eq!(s, "a [REDACTED:OUT-01] b [REDACTED:OUT-03] c");
   268	    }
   269	
   270	    // ── 3. 重叠 span 合并 ────────────────────────────────────────────────────
   271	
   272	    #[test]
   273	    fn overlapping_spans_merged() {
   274	        let body = b"0123456789";
   275	        // [1,6) 和 [4,9) 重叠 → 合并为 [1,9)，rule_id 取第一个 OUT-01
   276	        let hits = [hit("OUT-01", 1, 6), hit("OUT-02", 4, 9)];
   277	        let r = redact_body_bytes(body, &hits);
   278	        assert_eq!(
   279	            r.redacted_count, 1,
   280	            "two overlapping spans must merge into one"
   281	        );
   282	        let s = String::from_utf8(r.body).unwrap();
   283	        assert_eq!(s, "0[REDACTED:OUT-01]9");
   284	    }
   285	
   286	    // ── 4. UTF-8 边界对齐 ────────────────────────────────────────────────────
   287	
   288	    #[test]
   289	    fn utf8_boundary_alignment() {
   290	        // "ab中cd"：bytes: [a, b, 中(3 bytes), c, d]
   291	        // 偏移：a=0, b=1, 中[0]=2, 中[1]=3, 中[2]=4, c=5, d=6
   292	        let body = "ab中cd".as_bytes();
   293	        // byte 3 和 4 是 '中' 的 continuation byte，align 应向左到 2
   294	        assert_eq!(align_to_utf8_char_start(body, 3), 2);
   295	        assert_eq!(align_to_utf8_char_start(body, 4), 2);
   296	        // byte 5 是 'c'，本身是起始，不需要移动
   297	        assert_eq!(align_to_utf8_char_start(body, 5), 5);
   298	        // 超出 body 长度时返回 body.len()
   299	        assert_eq!(align_to_utf8_char_start(body, 100), body.len());
   300	    }
     1	//! 入站 GUI 类 hold 流路径（GuiPopup disposition）。
     2	//!
     3	//! 命中 IN-CR-01/05、IN-GEN-04 等 GuiPopup 规则时，hold 住 SSE 流，通过 IpcServer
     4	//! 等待用户在 GUI 做出决策；同时每 25 秒向调用方提供的 channel 发送一条 SSE keep-alive
     5	//! comment（`: keep-alive\n\n`），防止客户端因无数据而超时断开。
     6	//!
     7	//! 关联：ADR-014 §GUI 路径、SPEC-002（keep-alive 规约）、ADR-013（IPC 协议）。
     8	
     9	use std::sync::Arc;
    10	use std::time::Duration;
    11	
    12	use bytes::Bytes;
    13	use thiserror::Error;
    14	use tokio::sync::mpsc;
    15	use tracing::warn;
    16	
    17	use sieve_ipc::{DecisionAction, DecisionRequest, DefaultOnTimeout, IpcServer};
    18	
    19	/// Keep-alive 注释间隔（PRD v1.4 §6.7 要求 ≤ 30 s，取 25 s 留余量）。
    20	const KEEP_ALIVE_INTERVAL_SECS: u64 = 25;
    21	
    22	/// Keep-alive SSE comment 字节（RFC 8895 §9.2：以 `:` 开头的行是注释，客户端忽略）。
    23	const KEEP_ALIVE_BYTES: &[u8] = b": keep-alive\n\n";
    24	
    25	/// Hold 路径专用错误。
    26	#[derive(Debug, Error)]
    27	pub enum HoldError {
    28	    /// IPC 等待决策失败。
    29	    #[error("IPC decision error: {0}")]
    30	    Ipc(#[from] sieve_ipc::IpcError),
    31	}
    32	
    33	/// [`hold_and_decide`] 的返回值，表示 hold 结束后的处置动作。
    34	#[derive(Debug, PartialEq, Eq)]
    35	pub enum HoldOutcome {
    36	    /// 用户允许（或超时 default_on_timeout = Allow）→ 继续转发原始 SSE。
    37	    Allow,
    38	    /// 用户允许且要求脱敏（仅出站脱敏类，入站实际等价 Allow）→ 继续转发。
    39	    RedactAndAllow,
    40	    /// 用户拒绝（或超时 default_on_timeout = Block）→ 注入 `sieve_blocked` event 并关流。
    41	    Deny {
    42	        /// 拒绝原因（来自 rule_id 列表或 "timeout"）。
    43	        reason: String,
    44	    },
    45	}
    46	
    47	/// Hold 住当前 SSE 流，通过 [`IpcServer`] 等待用户决策，同时发送 keep-alive。
    48	///
    49	/// # 行为
    50	/// 1. 注册 keep-alive task（每 [`KEEP_ALIVE_INTERVAL_SECS`] 秒向 `keep_alive_tx` 发送
    51	///    `: keep-alive\n\n`），daemon 把它写入 SSE 流；
    52	/// 2. 并发等待 `ipc.request_decision(req, timeout)` 返回；
    53	/// 3. 决策返回后停掉 keep-alive task，返回 [`HoldOutcome`]。
    54	///
    55	/// # 超时
    56	/// 超时由 `req.timeout_seconds` 决定（传给 IpcServer）；超时时按 `req.default_on_timeout` 处理：
    57	/// - `Block` → `HoldOutcome::Deny`
    58	/// - `Allow` → `HoldOutcome::Allow`
    59	/// - `Redact` → `HoldOutcome::RedactAndAllow`（入站场景少见，逻辑完整性保留）
    60	///
    61	/// 关联：ADR-014 §GUI 路径、SPEC-002 §keep-alive。
    62	pub async fn hold_and_decide(
    63	    ipc: Arc<IpcServer>,
    64	    req: DecisionRequest,
    65	    keep_alive_tx: mpsc::Sender<Bytes>,
    66	) -> Result<HoldOutcome, HoldError> {
    67	    let timeout_secs = u64::from(req.timeout_seconds).max(1);
    68	    let default_on_timeout = req.default_on_timeout;
    69	    let rule_ids: String = req
    70	        .detections
    71	        .iter()
    72	        .map(|d| d.rule_id.as_str())
    73	        .collect::<Vec<_>>()
    74	        .join(", ");
    75	
    76	    // 启动 keep-alive task
    77	    let ka_tx = keep_alive_tx.clone();
    78	    let ka_handle = tokio::spawn(async move {
    79	        let mut interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL_SECS));
    80	        interval.tick().await; // 第一次 tick 立即返回（elapsed），跳过
    81	        loop {
    82	            interval.tick().await;
    83	            if ka_tx
    84	                .send(Bytes::from_static(KEEP_ALIVE_BYTES))
    85	                .await
    86	                .is_err()
    87	            {
    88	                // 接收端已关闭，停止发送
    89	                break;
    90	            }
    91	        }
    92	    });
    93	
    94	    // 等待 IPC 决策
    95	    let timeout = Duration::from_secs(timeout_secs);
    96	    let result = ipc.request_decision(req, timeout).await;
    97	
    98	    // 停掉 keep-alive（无论成功失败）
    99	    ka_handle.abort();
   100	
   101	    let resp = match result {
   102	        Ok(r) => r,
   103	        Err(e) => {
   104	            warn!("IPC decision error: {e}; falling back to default_on_timeout");
   105	            // IPC 错误按超时兜底
   106	            return Ok(timeout_outcome(default_on_timeout, &rule_ids));
   107	        }
   108	    };
   109	
   110	    let outcome = match resp.decision {
   111	        DecisionAction::Allow => HoldOutcome::Allow,
   112	        DecisionAction::RedactAndAllow => HoldOutcome::RedactAndAllow,
   113	        DecisionAction::Deny => HoldOutcome::Deny {
   114	            reason: if resp.by_user {
   115	                format!("用户拒绝（rules: {rule_ids}）")
   116	            } else {
   117	                format!("超时 default-block（rules: {rule_ids}）")
   118	            },
   119	        },
   120	    };
   121	
   122	    Ok(outcome)
   123	}
   124	
   125	/// 按 [`DefaultOnTimeout`] 构造超时结果。
   126	fn timeout_outcome(dot: DefaultOnTimeout, rule_ids: &str) -> HoldOutcome {
   127	    match dot {
   128	        DefaultOnTimeout::Block => HoldOutcome::Deny {
   129	            reason: format!("超时 fail-closed（rules: {rule_ids}）"),
   130	        },
   131	        DefaultOnTimeout::Allow => HoldOutcome::Allow,
   132	        DefaultOnTimeout::Redact => HoldOutcome::RedactAndAllow,
   133	    }
   134	}
   135	
   136	#[cfg(test)]
   137	mod tests {
   138	    use super::*;
   139	    use chrono::Utc;
   140	    use sieve_ipc::protocol::{DecisionResponse, DetectionPayload, Disposition, Severity};
   141	    use uuid::Uuid;
   142	
   143	    fn make_request(
   144	        id: Uuid,
   145	        timeout_seconds: u32,
   146	        default_on_timeout: DefaultOnTimeout,
   147	    ) -> DecisionRequest {
   148	        DecisionRequest {
   149	            request_id: id,
   150	            created_at: Utc::now(),
   151	            timeout_seconds,
   152	            default_on_timeout,
   153	            detections: vec![DetectionPayload {
   154	                rule_id: "IN-CR-01".to_owned(),
   155	                severity: Severity::Critical,
   156	                disposition: Disposition::GuiPopup,
   157	                title: "地址替换检测".to_owned(),
   158	                one_line_summary: "检测到可疑地址替换".to_owned(),
   159	                details: serde_json::json!({}),
   160	            }],
   161	        }
   162	    }
   163	
   164	    fn make_ipc_server() -> (Arc<IpcServer>, tokio::net::UnixListener, std::path::PathBuf) {
   165	        let tmp = tempfile::tempdir().unwrap();
   166	        let socket_path = tmp.path().join("ipc.sock");
   167	        // 把 tmp 路径 leak 到测试生命周期（tempfile 会在 drop 时清理，但 socket 不影响测试）
   168	        std::mem::forget(tmp);
   169	        let path = socket_path.clone();
   170	        IpcServer::bind(socket_path)
   171	            .map(|(s, l)| (Arc::new(s), l, path))
   172	            .unwrap()
   173	    }
   174	
   175	    // ── Mock IPC 返回 Allow ───────────────────────────────────────────────────
   176	
   177	    #[tokio::test]
   178	    async fn ipc_allow_returns_allow_outcome() {
   179	        let (server, listener, socket_path) = make_ipc_server();
   180	        let srv = Arc::clone(&server);
   181	        tokio::spawn(async move { srv.run(listener).await });
   182	        tokio::time::sleep(Duration::from_millis(10)).await;
   183	
   184	        // 模拟 GUI 客户端连接：使 gui_writer 有值，让 request_decision 注册 oneshot
   185	        // 而不是在步骤 1 因无 GUI 连接而立即 fallback（修 #2 相关：inject_decision 需先有注册）。
   186	        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
   187	            .await
   188	            .expect("connect to IPC socket failed");
   189	        tokio::time::sleep(Duration::from_millis(10)).await;
   190	
   191	        let id = Uuid::now_v7();
   192	        let req = make_request(id, 5, DefaultOnTimeout::Block);
   193	
   194	        // 50ms 后注入 Allow 决策（此时 pending map 里已有 oneshot sender）
   195	        let inject_srv = Arc::clone(&server);
   196	        tokio::spawn(async move {
   197	            tokio::time::sleep(Duration::from_millis(50)).await;
   198	            inject_srv
   199	                .inject_decision(DecisionResponse {
   200	                    request_id: id,
   201	                    decision: DecisionAction::Allow,
   202	                    decided_at: Utc::now(),
   203	                    by_user: true,
   204	                    remember: false,
   205	                })
   206	                .await;
   207	        });
   208	
   209	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   210	        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
   211	            .await
   212	            .unwrap();
   213	        assert_eq!(outcome, HoldOutcome::Allow);
   214	    }
   215	
   216	    // ── Mock IPC 返回 Deny ────────────────────────────────────────────────────
   217	
   218	    #[tokio::test]
   219	    async fn ipc_deny_returns_deny_outcome() {
   220	        let (server, listener, socket_path) = make_ipc_server();
   221	        let srv = Arc::clone(&server);
   222	        tokio::spawn(async move { srv.run(listener).await });
   223	        tokio::time::sleep(Duration::from_millis(10)).await;
   224	
   225	        // 模拟 GUI 客户端连接（同 Allow 测试，确保 inject_decision 能工作）
   226	        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
   227	            .await
   228	            .expect("connect to IPC socket failed");
   229	        tokio::time::sleep(Duration::from_millis(10)).await;
   230	
   231	        let id = Uuid::now_v7();
   232	        let req = make_request(id, 5, DefaultOnTimeout::Block);
   233	
   234	        let inject_srv = Arc::clone(&server);
   235	        tokio::spawn(async move {
   236	            tokio::time::sleep(Duration::from_millis(50)).await;
   237	            inject_srv
   238	                .inject_decision(DecisionResponse {
   239	                    request_id: id,
   240	                    decision: DecisionAction::Deny,
   241	                    decided_at: Utc::now(),
   242	                    by_user: true,
   243	                    remember: false,
   244	                })
   245	                .await;
   246	        });
   247	
   248	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   249	        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
   250	            .await
   251	            .unwrap();
   252	        assert!(matches!(outcome, HoldOutcome::Deny { .. }));
   253	    }
   254	
   255	    // ── 超时 default_on_timeout = Block ──────────────────────────────────────
   256	
   257	    #[tokio::test]
   258	    async fn timeout_with_block_returns_deny() {
   259	        let (server, listener, _socket_path) = make_ipc_server();
   260	        let srv = Arc::clone(&server);
   261	        tokio::spawn(async move { srv.run(listener).await });
   262	        tokio::time::sleep(Duration::from_millis(10)).await;
   263	
   264	        let id = Uuid::now_v7();
   265	        // 使用 tokio::time::pause() + advance() 模拟超时（无需等 1s）
   266	        tokio::time::pause();
   267	
   268	        let req = make_request(id, 1, DefaultOnTimeout::Block);
   269	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   270	
   271	        let ipc_clone = Arc::clone(&server);
   272	        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });
   273	
   274	        // 推进 2 秒让超时触发
   275	        tokio::time::advance(Duration::from_secs(2)).await;
   276	        tokio::time::resume();
   277	
   278	        let outcome = task.await.unwrap().unwrap();
   279	        assert!(
   280	            matches!(outcome, HoldOutcome::Deny { .. }),
     1	//! 入站 Hook 类路径（HookTerminal disposition）。
     2	//!
     3	//! 命中 IN-CR-02~04、IN-GEN-01~03 等 HookTerminal 规则时，写入 IPC pending 文件，
     4	//! **不修改 SSE 流**——流由调用方（daemon）原样转发给客户端。
     5	//! sieve-hook 二进制会在 PreToolUse 阶段读取 pending 文件并在 TTY 拦截。
     6	//!
     7	//! 关联：ADR-014 §Hook 路径、SPEC-001（pending 文件写入规约）。
     8	
     9	use sieve_ipc::{paths::sieve_home, pending_file::write_pending, DecisionRequest};
    10	use thiserror::Error;
    11	use uuid::Uuid;
    12	
    13	/// Hook 路径专用错误。
    14	#[derive(Debug, Error)]
    15	pub enum HookError {
    16	    /// IPC 操作失败（目录创建 / 文件写入 / 锁获取）。
    17	    #[error("IPC error: {0}")]
    18	    Ipc(#[from] sieve_ipc::IpcError),
    19	}
    20	
    21	/// 写入 IPC pending 文件，通知 sieve-hook 在 PreToolUse 阶段拦截。
    22	///
    23	/// # 行为
    24	/// - 在 `~/.sieve/pending/<request_id>.json`（或 `$SIEVE_HOME`）写入 [`DecisionRequest`]；
    25	/// - **不修改 SSE 流**——调用方负责原样转发；
    26	/// - 返回 `Ok(())` 表示文件已写入，daemon 可继续转发。
    27	///
    28	/// # 错误
    29	/// 目录创建或文件写入失败时返回 [`HookError::Ipc`]。
    30	///
    31	/// 关联：ADR-014 §Hook 路径、SPEC-001 §3.1。
    32	pub fn write_hook_pending(request_id: Uuid, req: &DecisionRequest) -> Result<(), HookError> {
    33	    let _ = request_id; // request_id 已包含在 req.request_id 中，此参数保留供调用侧校验
    34	    let base = sieve_home()?;
    35	    write_pending(req, &base)?;
    36	    Ok(())
    37	}
    38	
    39	#[cfg(test)]
    40	mod tests {
    41	    use super::*;
    42	    use chrono::Utc;
    43	    use sieve_ipc::{
    44	        pending_file::read_pending,
    45	        protocol::{DefaultOnTimeout, DetectionPayload, Disposition, Severity},
    46	    };
    47	
    48	    fn make_request(id: Uuid) -> DecisionRequest {
    49	        DecisionRequest {
    50	            request_id: id,
    51	            created_at: Utc::now(),
    52	            timeout_seconds: 30,
    53	            default_on_timeout: DefaultOnTimeout::Block,
    54	            detections: vec![DetectionPayload {
    55	                rule_id: "IN-CR-02".to_owned(),
    56	                severity: Severity::Critical,
    57	                disposition: Disposition::HookTerminal,
    58	                title: "危险 shell 命令".to_owned(),
    59	                one_line_summary: "检测到 rm -rf 命令".to_owned(),
    60	                details: serde_json::json!({ "command": "rm -rf /tmp" }),
    61	            }],
    62	        }
    63	    }
    64	
    65	    #[test]
    66	    fn write_and_read_pending_file() {
    67	        // 使用独立 tmpdir 直接调用底层 write_pending / read_pending，
    68	        // 避免并发测试互相污染 SIEVE_HOME 全局变量。
    69	        let tmp = tempfile::tempdir().unwrap();
    70	        let base = tmp.path();
    71	
    72	        let id = Uuid::now_v7();
    73	        let req = make_request(id);
    74	
    75	        // 直接写入指定 base 目录
    76	        sieve_ipc::pending_file::write_pending(&req, base).unwrap();
    77	
    78	        // 验证文件内容正确
    79	        let read_back = read_pending(id, base).unwrap();
    80	        assert_eq!(read_back.request_id, id);
    81	        assert_eq!(read_back.detections.len(), 1);
    82	        assert_eq!(read_back.detections[0].rule_id, "IN-CR-02");
    83	    }
    84	
    85	    #[test]
    86	    fn write_hook_pending_idempotent_on_same_id() {
    87	        // 使用独立 tmpdir 避免污染 SIEVE_HOME 全局变量（并发测试安全）
    88	        let tmp = tempfile::tempdir().unwrap();
    89	        let base = tmp.path();
    90	
    91	        let id = Uuid::now_v7();
    92	        let req = make_request(id);
    93	
    94	        // 写两次不应 panic
    95	        sieve_ipc::pending_file::write_pending(&req, base).unwrap();
    96	        sieve_ipc::pending_file::write_pending(&req, base).unwrap();
    97	
    98	        // 最终文件可正常读取
    99	        let read_back = read_pending(id, base).unwrap();
   100	        assert_eq!(read_back.request_id, id);
   101	    }
   102	}

exec
/bin/zsh -lc "grep -R \"fn extract_text_content\" -n crates/sieve-core/src && nl -ba crates/sieve-core/src/protocol/anthropic.rs | sed -n '1,240p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-core/src/protocol/anthropic.rs:56:    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
     1	//! Anthropic Messages API 请求/响应 schema（子集）。
     2	//!
     3	//! 文档: <https://docs.anthropic.com/en/api/messages>
     4	//! 关联 PRD §6.1 Phase 1 边界。
     5	//!
     6	//! 只实现 Phase 1 需要的字段；extra 字段通过 `#[serde(flatten)]` 保留，
     7	//! 确保原始 body 可无损转发到上游。
     8	
     9	use serde::{Deserialize, Serialize};
    10	use std::collections::BTreeMap;
    11	
    12	/// POST /v1/messages 请求 body。
    13	///
    14	/// 关联 PRD §6.1：Phase 1 只解析 Anthropic 格式，其他 provider 预留 (ADR-004)。
    15	#[derive(Debug, Clone, Serialize, Deserialize)]
    16	pub struct AnthropicRequest {
    17	    /// 模型名（如 claude-sonnet-4-6）。
    18	    pub model: String,
    19	    /// 最大生成 token 数。
    20	    pub max_tokens: u32,
    21	    /// 消息列表。
    22	    pub messages: Vec<AnthropicMessage>,
    23	    /// 是否流式（SSE）。
    24	    #[serde(default)]
    25	    pub stream: bool,
    26	    /// 系统提示（string 或 content blocks）。
    27	    #[serde(skip_serializing_if = "Option::is_none")]
    28	    pub system: Option<serde_json::Value>,
    29	    /// 工具定义列表。
    30	    #[serde(skip_serializing_if = "Option::is_none")]
    31	    pub tools: Option<serde_json::Value>,
    32	    /// 工具选择策略。
    33	    #[serde(skip_serializing_if = "Option::is_none")]
    34	    pub tool_choice: Option<serde_json::Value>,
    35	    /// 其他字段（向前兼容，不在乎也不丢弃）。
    36	    #[serde(flatten)]
    37	    pub extra: BTreeMap<String, serde_json::Value>,
    38	}
    39	
    40	/// Anthropic Messages API 单条消息。
    41	#[derive(Debug, Clone, Serialize, Deserialize)]
    42	pub struct AnthropicMessage {
    43	    /// 角色（"user" 或 "assistant"）。
    44	    pub role: String,
    45	    /// 内容（string 或 content block 数组）。
    46	    pub content: serde_json::Value,
    47	}
    48	
    49	impl AnthropicRequest {
    50	    /// 提取所有 message content 中的文本（string content 或 type=text content block）。
    51	    ///
    52	    /// 返回 `(近似 body 字节偏移, text)` 列表。Phase 1 偏移仅供审计参考；精确 span 由
    53	    /// vectorscan 在单条文本内 scan 时给出（start/end 是相对该 text 的偏移）。
    54	    ///
    55	    /// 同时追加 `system` 字段中的文本（string 或 content blocks）。
    56	    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
    57	        let mut result = Vec::new();
    58	        let mut cursor = 0usize;
    59	        for msg in &self.messages {
    60	            match &msg.content {
    61	                serde_json::Value::String(s) => {
    62	                    result.push((cursor, s.clone()));
    63	                    cursor += s.len();
    64	                }
    65	                serde_json::Value::Array(blocks) => {
    66	                    for block in blocks {
    67	                        if let Some(block_obj) = block.as_object() {
    68	                            if block_obj.get("type").and_then(|v| v.as_str()) == Some("text") {
    69	                                if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
    70	                                    result.push((cursor, text.to_string()));
    71	                                    cursor += text.len();
    72	                                }
    73	                            }
    74	                        }
    75	                    }
    76	                }
    77	                _ => {}
    78	            }
    79	        }
    80	        // 同时扫 system prompt（若有）
    81	        if let Some(system) = &self.system {
    82	            if let Some(s) = system.as_str() {
    83	                result.push((cursor, s.to_string()));
    84	            } else if let Some(blocks) = system.as_array() {
    85	                for block in blocks {
    86	                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
    87	                        result.push((cursor, text.to_string()));
    88	                        cursor += text.len();
    89	                    }
    90	                }
    91	            }
    92	        }
    93	        result
    94	    }
    95	}
    96	
    97	#[cfg(test)]
    98	mod tests {
    99	    use super::*;
   100	
   101	    #[test]
   102	    fn parse_minimal_request() {
   103	        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1024,"messages":[{"role":"user","content":"hi"}]}"#;
   104	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   105	        assert_eq!(req.model, "claude-sonnet-4-6");
   106	        assert_eq!(req.messages.len(), 1);
   107	        assert!(!req.stream);
   108	        assert!(req.extra.is_empty());
   109	    }
   110	
   111	    #[test]
   112	    fn parse_streaming_request_with_tools() {
   113	        let json = r#"{
   114	            "model": "claude-opus-4-5",
   115	            "max_tokens": 4096,
   116	            "stream": true,
   117	            "messages": [{"role": "user", "content": "hello"}],
   118	            "tools": [{"name": "bash", "description": "run shell"}],
   119	            "unknown_future_field": 42
   120	        }"#;
   121	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   122	        assert!(req.stream);
   123	        assert!(req.tools.is_some());
   124	        // 未知字段被 flatten 保留，不丢弃
   125	        assert!(req.extra.contains_key("unknown_future_field"));
   126	    }
   127	
   128	    #[test]
   129	    fn roundtrip_preserves_extra_fields() {
   130	        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1,"messages":[],"custom_key":"custom_value"}"#;
   131	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   132	        let re_serialized = serde_json::to_string(&req).unwrap();
   133	        assert!(re_serialized.contains("custom_key"));
   134	        assert!(re_serialized.contains("custom_value"));
   135	    }
   136	}
   137	
   138	#[cfg(test)]
   139	mod tests_extract {
   140	    use super::*;
   141	
   142	    #[test]
   143	    fn extract_simple_string_content() {
   144	        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":"hello"}]}"#;
   145	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   146	        let texts = req.extract_text_content();
   147	        assert_eq!(texts.len(), 1);
   148	        assert_eq!(texts[0].1, "hello");
   149	    }
   150	
   151	    #[test]
   152	    fn extract_content_blocks() {
   153	        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":[{"type":"text","text":"hi"},{"type":"text","text":"world"}]}]}"#;
   154	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   155	        let texts = req.extract_text_content();
   156	        assert_eq!(texts.len(), 2);
   157	        assert_eq!(texts[0].1, "hi");
   158	        assert_eq!(texts[1].1, "world");
   159	    }
   160	
   161	    #[test]
   162	    fn extract_with_system_prompt() {
   163	        let json = r#"{"model":"x","max_tokens":1,"system":"You are helpful","messages":[{"role":"user","content":"q"}]}"#;
   164	        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
   165	        let texts = req.extract_text_content();
   166	        assert_eq!(texts.len(), 2);
   167	        // system 在最后一项
   168	        assert!(texts.iter().any(|(_, t)| t == "You are helpful"));
   169	    }
   170	}

exec
/bin/zsh -lc 'git diff --cached --stat' in /Users/doskey/src/sieve
 succeeded in 0ms:
exec
/bin/zsh -lc "git diff -- .cursorrules CLAUDE.md | sed -n '1,220p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/.cursorrules b/.cursorrules
index ba8962f..f55f506 100644
--- a/.cursorrules
+++ b/.cursorrules
@@ -1,6 +1,6 @@
 # Sieve · Cursor 项目规则
 
-> 本文件是 Sieve 项目的 Cursor 工作规则，分两层：**文档规则**（沿用用户级 `.cursorrules` 模板骨架）+ **Sieve 工程硬约束**（来自 PRD v1.3 §9 与项目级共识）。
+> 本文件是 Sieve 项目的 Cursor 工作规则，分两层：**文档规则**（沿用用户级 `.cursorrules` 模板骨架）+ **Sieve 工程硬约束**（来自 PRD v1.4 §9 与项目级共识）。
 >
 > 当前项目状态：**设计阶段（Pre-Code）**，所有约束按"将开始写代码"语境提前生效。
 
@@ -13,7 +13,7 @@
 1. **改代码前必须先读** `@docs/` 中相关文档（需求 → 设计 → API），不盲目重构
 2. **完成代码修改后必须更新所有关联文档**，禁止单独更新一个文档而忽略关联文档
 3. **临时文档**使用前缀 `_temp-` 或 `_draft-`，存放于 `docs/_temp/`，**任务完成后 24 小时内删除或归档**
-4. PRD v1.3 全文不复制到其他文档，所有跨文档引用一律使用相对路径链接
+4. PRD v1.4 全文不复制到其他文档，所有跨文档引用一律使用相对路径链接
 
 ### 1.2 文档位置约定
 
@@ -43,7 +43,7 @@ docs/
 ```
 README.md
   ↓ 链接到
-docs/requirements/PRD-sieve.md → docs/prd/sieve-prd-v1.3.md（活动版本）
+docs/requirements/PRD-sieve.md → docs/prd/sieve-prd-v1.4.md（活动版本）
   ↓ 关联
 docs/design/architecture.md
   ↓ 约束
@@ -69,7 +69,7 @@ docs/changelog/CHANGELOG.md
 
 ---
 
-## 二、Sieve 工程硬约束（来自 PRD v1.3 §9）
+## 二、Sieve 工程硬约束（来自 PRD v1.4 §9，十三条）
 
 每条都是"做错就死"，**不是优化项**，**不接受任何阶段性妥协**：
 
@@ -83,6 +83,9 @@ docs/changelog/CHANGELOG.md
 8. **Critical 在所有版本（包括降级模式）不可关闭**——产品安全承诺，不是用户偏好
 9. **Phase 1 只做 Claude Code，UnifiedMessage 接口预留**——公理 7，**不为想象用户写代码**；OpenAI / Gemini / OpenRouter 接入等真有用户主动要再做
 10. **Week 12 GA 时一次性公开 repo + 代码 + 文档，GA 前完全私有**——见 [ADR-011](./docs/design/ADR-011-private-until-ga.md)；sigstore + reproducible build pipeline GA 前照常跑通（关联 ADR-006）
+11. **不在 Anthropic API 协议层撒谎**——不伪造 tool_use / stop_reason / id / usage / type；拦截发生时允许截 SSE 流注入 `sieve_blocked` event（Sieve 自报，不是冒充模型）；keep-alive comment 行不算伪造
+12. **不装本地 CA 做 MITM**——Network Extension / 本地 CA 注入 / 系统 proxy 修改推 Phase 3 选购，Phase 1/2 不做
+13. **出站脱敏不打断工作流**——OUT-01~05/12 高频类自动脱敏 + 状态栏 5s 通知，不弹窗；弹窗次数过多用户直接禁用产品
 
 ---
 
@@ -103,13 +106,15 @@ docs/changelog/CHANGELOG.md
 
 ### 3.3 模块边界
 
-按 PRD §6 分三个 crate：
+按 PRD §6 分五个 crate：
 
 | crate | 职责 | 不允许做 |
 |-------|------|---------|
 | `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
 | `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
 | `sieve-cli` | 入口 / 配置 / 弹窗 / 审计日志（SQLite） | 直接做规则匹配 |
+| `sieve-ipc` | IPC 协议（JSON-RPC over Unix socket + 文件锁 + pending/decisions 文件 IO） | 不参与请求处理 / 不依赖 sieve-core 业务逻辑 |
+| `sieve-hook` | 极简 PreToolUse hook 二进制，启动时延 < 50ms | 不依赖 sieve-core / sieve-rules / sieve-ipc / tokio |
 
 跨 crate 调用走显式 trait，避免互相 import 实现细节。
 
@@ -128,7 +133,7 @@ docs/changelog/CHANGELOG.md
 | 明确需求 | 直接给方案 + 必要文档更新清单 |
 | 模糊需求 | 引用相关 PRD/设计文档 + 给最可能方案 + 1 个关键追问 |
 | 破坏性操作（删文件、改 schema、重构 crate 边界） | **先确认后执行**，给出回退方案 |
-| 涉及 PRD §9 十条硬约束 | **任何放宽都必须显式让用户确认**，默认不允许 |
+| 涉及 PRD §9 十三条硬约束 | **任何放宽都必须显式让用户确认**，默认不允许 |
 
 ---
 
@@ -136,7 +141,7 @@ docs/changelog/CHANGELOG.md
 
 - [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` 通过
 - [ ] 涉及 SSE / 规则 / 工具调用判定的改动有对应 fuzz / 单元测试
-- [ ] PRD §9 十条硬约束未被绕过
+- [ ] PRD §9 十三条硬约束未被绕过
 - [ ] CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
 - [ ] 关联文档（requirements / design / api）已同步
 - [ ] 临时文档（`_temp-` / `_draft-`）已清理或归档
@@ -146,6 +151,6 @@ docs/changelog/CHANGELOG.md
 ## 参考
 
 - 项目入口：[README.md](./README.md)
-- 当前活动 PRD：[docs/prd/sieve-prd-v1.3.md](./docs/prd/sieve-prd-v1.3.md)
+- 当前活动 PRD：[docs/prd/sieve-prd-v1.4.md](./docs/prd/sieve-prd-v1.4.md)
 - 需求入口：[docs/requirements/PRD-sieve.md](./docs/requirements/PRD-sieve.md)
 - 用户故事：[docs/requirements/user-stories.md](./docs/requirements/user-stories.md)
diff --git a/CLAUDE.md b/CLAUDE.md
index dd283f4..8d6dec4 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -18,21 +18,21 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 
 文档冲突时按以下优先级裁决（高 → 低）：
 
-1. **PRD v1.3** — [docs/prd/sieve-prd-v1.3.md](docs/prd/sieve-prd-v1.3.md)（已锁定执行，唯一权威源）
-2. **ADR** — [docs/design/ADR-INDEX.md](docs/design/ADR-INDEX.md)（8 个已接受 + 候选清单 ADR-008/009/010）
+1. **PRD v1.4** — [docs/prd/sieve-prd-v1.4.md](docs/prd/sieve-prd-v1.4.md)（已锁定执行，唯一权威源）
+2. **ADR** — [docs/design/ADR-INDEX.md](docs/design/ADR-INDEX.md)（13 个已接受 + 候选清单 ADR-008/009/010）
 3. **架构 / 数据模型** — [docs/design/architecture.md](docs/design/architecture.md) + [docs/design/data-model.md](docs/design/data-model.md)
 4. **API / 部署 / 开发指南** — `docs/api/` + `docs/guides/`
 5. **README + .cursorrules** — 项目入口与代码规范
 
 约束：
 
-- `docs/requirements/PRD-sieve.md` 是 v1.3 的薄指针，不复制全文
-- `docs/prd/` 下 v1.0~v1.2 是历史归档，**永不修改**
+- `docs/requirements/PRD-sieve.md` 是 v1.4 的薄指针，不复制全文
+- `docs/prd/` 下 v1.0~v1.3 是历史归档，**永不修改**
 - 术语首次出现先去 [docs/glossary.md](docs/glossary.md) 加条目，再在 PRD/ADR 引用
 
 ---
 
-## 不可放宽的硬约束（PRD §9 十条 / .cursorrules §二）
+## 不可放宽的硬约束（PRD §9 十三条 / .cursorrules §二）
 
 任何 PR / 设计变更触碰以下任一条，**默认拒绝**，必须先和用户显式确认才能放宽：
 
@@ -46,19 +46,26 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 8. **Critical 在所有版本（含降级模式）不可关闭** —— 产品安全承诺，不是用户偏好
 9. **Phase 1 仅适配 Claude Code** —— UnifiedMessage 接口预留但**不实现** OpenAI / Gemini / OpenRouter
 10. **Week 12 GA 一次性公开 repo + 代码 + 文档** —— GA 前 repo 完全私有（见 ADR-011）；sigstore CI pipeline 照常跑通
+11. **不在 Anthropic API 协议层撒谎** —— 不伪造 tool_use / stop_reason / id / usage / type；拦截发生时允许截 SSE 流注入 `sieve_blocked` event（Sieve 自报事件，不是冒充模型）；keep-alive comment 行 `: keep-alive\n\n` 不属于伪造
+12. **不装本地 CA 做 MITM** —— Network Extension / 本地 CA 注入 / 系统 proxy 修改推 Phase 3 选购，Phase 1/2 不做
+13. **出站脱敏不打断工作流** —— OUT-01~05/12 高频脱敏类必须自动脱敏 + 状态栏 5s 通知，不弹窗；每天弹几十次的产品没人用
 
 ---
 
-## 三个 Crate（Week 1 落地后强制）
+## 五个 Crate（Week 5 落地后强制）
 
 | crate | 职责 | 禁做 |
 |------|------|-----|
 | `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
 | `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
 | `sieve-cli` | 入口 / 配置 / 弹窗 / 审计日志（SQLite） | 直接做规则匹配 |
+| `sieve-ipc` | IPC 协议（JSON-RPC over Unix socket + 文件锁 + pending/decisions 文件 IO） | 不参与请求处理 / 不依赖 sieve-core 业务逻辑 |
+| `sieve-hook` | 极简 PreToolUse hook 二进制，启动时延 < 50ms，依赖只有 `serde_json` + `fd-lock` + `uuid` + `chrono` + `clap` | 不依赖 sieve-core / sieve-rules / sieve-ipc / tokio |
 
 跨 crate 调用走显式 trait，避免互相 import 实现细节。详见 [.cursorrules §3.3](.cursorrules)。
 
+> Native GUI App 在独立仓库 `sieve-gui-macos`，不在本 workspace。
+
 ---
 
 ## Rust 工具链与规范（开始写代码后强制）
@@ -81,7 +88,7 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 
 - [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` 通过
 - [ ] 涉及 SSE / 规则 / 工具调用判定的改动有对应 fuzz / 单元测试
-- [ ] PRD §9 十条硬约束未被绕过
+- [ ] PRD §9 十三条硬约束未被绕过
 - [ ] CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
 - [ ] 关联文档（requirements / design / api）已同步
 - [ ] 临时文档（`_temp-` / `_draft-`）已清理或归档
@@ -103,7 +110,7 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 | crate 边界变化 | .cursorrules §3.3 + architecture | P0 |
 | 定价 / 试用 / 降级模式 | README + user-stories US-12 + CHANGELOG | P1 |
 | 法律实体 / 渠道策略 | README + ADR-005 + CHANGELOG | P1 |
-| 工程硬约束变化（PRD §9 十条） | PRD-sieve 版本演进表 + ADR + .cursorrules §二 + CHANGELOG `[BREAKING]` | P0 |
+| 工程硬约束变化（PRD §9 十三条） | PRD-sieve 版本演进表 + ADR + .cursorrules §二 + CHANGELOG `[BREAKING]` | P0 |
 | 配置 / `config.toml` schema 变化 | api-reference §3 + deployment + CHANGELOG | P1 |
 | 依赖升级（vectorscan / rustls 等） | CHANGELOG | P2 |
 
@@ -121,9 +128,9 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 - 信任边界（如新增任何上游 verify、改 fail-closed 行为）
 - 商业 / 法律主体变化
 
-候选 ADR 已登记在 INDEX：ADR-008（426 状态码）、ADR-009（Windows 服务）、ADR-010（加密支付通道）。
+候选 ADR：ADR-008（426 状态码）、ADR-009（Windows 服务）、ADR-010（加密支付通道）。已接受：ADR-012（Native GUI App）、ADR-013（IPC 协议）、ADR-014（双层防御）、ADR-015（sieve setup 工具）、ADR-016（处置矩阵二维化）。
 
-**写 SPEC**（`docs/specs/` 当前为空，按需新建）——具体检测算法落地需要工程级规格时（如 BIP39 SHA-256 状态机、地址替换 Levenshtein 算法、SSE 流式 vectorscan 状态机）。Phase 2 功能 SPEC 暂不写——不为想象用户写代码。
+**写 SPEC**（[docs/specs/](docs/specs/) 已落地 SPEC-001/002/003）——具体检测算法落地需要工程级规格时（如 BIP39 SHA-256 状态机、地址替换 Levenshtein 算法、SSE 流式 vectorscan 状态机）。Phase 2 功能 SPEC 暂不写——不为想象用户写代码。
 
 ---
 
@@ -131,14 +138,21 @@ Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），
 
 执行视图见 [tasks/roadmap.md](tasks/roadmap.md)，每周完成定义跟 PRD §10 同步，本文不重复。
 
-**Week 1 关键路径**（必须并行启动，否则 12 周里程碑会延期）：
+**Week 1 关键路径**（已完成）：
 
 1. Rust workspace 骨架（三个 crate）+ hyper/tokio/rustls 跑通透明转发 Anthropic Messages API
 2. UnifiedMessage 内部 schema（Anthropic only，三家接口预留）
-3. **sigstore + reproducible build pipeline 必须本周跑通**（[ADR-006 §4](docs/design/ADR-006-sigstore-reproducible-build.md)，Tier 1 hard gate；这是 PRD §1.2 第 4 句"自证清白"的物质基础）
+3. **sigstore + reproducible build pipeline 本周跑通**（[ADR-006 §4](docs/design/ADR-006-sigstore-reproducible-build.md)，Tier 1 hard gate）
 4. **海外公司注册启动**（[ADR-005](docs/design/ADR-005-overseas-legal-entity.md)，香港 4-6 周才能拿到执照，Week 7-8 Stripe 接入需要）
 5. ~~GitHub repo 公开~~（被 ADR-011 撤销；repo 保持私有至 Week 12 GA）
 
+**Week 5 关键路径**（v1.4 架构落地）：
+
+1. **`sieve-ipc` + `sieve-hook` 骨架跑通**（IPC server JSON-RPC + 文件锁协议，hook 启动时延 < 50ms）
+2. **`sieve setup` 一键安装**：检测 Claude Code → 注册 PreToolUse hook + 写 ANTHROPIC_BASE_URL → 加载 launchd plist；朋友 30 分钟内能装通（[SPEC-003](docs/specs/SPEC-003-sieve-setup-tool.md)）
+3. **出站自动脱敏路径**：OUT-01~05/12 命中后改写 body bytes，不返 426，状态栏 5s 通知
+4. **入站双层防御重构**：Hook 类（IN-CR-02~04）写 IPC pending file 不修改 SSE 流；GUI 类（IN-CR-01/05）hold 流 + keep-alive comment + 用户确认后处置（[ADR-014](docs/design/ADR-014-dual-layer-defense.md)）
+
 ---
 
 ## 常用命令
@@ -184,7 +198,7 @@ cargo bench -p sieve-rules
 
 - **doskey 一人 + Claude Code**，不要假设有团队 / DevRel / 销售 / 数据标注师
 - **海外公司主体 + Stripe + 加密支付双通道**，不接受国内任何收款主体（[ADR-005](docs/design/ADR-005-overseas-legal-entity.md)）
-- **境内不做 to-C 公开商业化营销**——Twitter / Hacker News / Mirror 是主战场，微信 / 小红书 / 知乎 / B 站不规划（[PRD §11.5.2 渠道分级](docs/prd/sieve-prd-v1.3.md#1152-营销渠道分级)）
+- **境内不做 to-C 公开商业化营销**——Twitter / Hacker News / Mirror 是主战场，微信 / 小红书 / 知乎 / B 站不规划（[PRD §11.5.2 渠道分级](docs/prd/sieve-prd-v1.4.md#1152-营销渠道分级)）
 - 18 个月 MRR 目标 ≥ $25K，24 个月 $50K-75K——**不追独角兽，不融资，不招人**
 
 ---

exec
/bin/zsh -lc "git diff -- crates/sieve-rules/tests/outbound_rules.rs crates/sieve-rules/tests/inbound_rules.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/crates/sieve-rules/tests/inbound_rules.rs b/crates/sieve-rules/tests/inbound_rules.rs
index fbe0e73..ae3f68e 100644
--- a/crates/sieve-rules/tests/inbound_rules.rs
+++ b/crates/sieve-rules/tests/inbound_rules.rs
@@ -544,6 +544,148 @@ fn in_cr_04_unrelated_commands_no_hit() {
     }
 }
 
+// ---------------------------------------------------------------------------
+// PRD v1.4 §5.4 新字段解析验证
+// ---------------------------------------------------------------------------
+
+/// 验证入站规则 TOML 中 disposition / timeout_seconds / default_on_timeout 正确解析。
+#[test]
+fn inbound_rules_disposition_fields_parsed() {
+    use sieve_rules::loader::load_inbound_rules;
+    use sieve_rules::manifest::{DefaultOnTimeout, Disposition};
+
+    let path = rules_path();
+    let rules = load_inbound_rules(&path).expect("load inbound.toml failed");
+
+    // IN-CR-01：gui_popup, 60s, block
+    let r = rules.iter().find(|r| r.id == "IN-CR-01").expect("IN-CR-01");
+    assert_eq!(r.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(r.timeout_seconds, Some(60));
+    assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
+
+    // IN-CR-02 系列：hook_terminal, 30s, block
+    for id in [
+        "IN-CR-02",
+        "IN-CR-02-CURL-PIPE",
+        "IN-CR-02-WGET-PIPE",
+        "IN-CR-02-EVAL",
+        "IN-CR-02-NC-REVERSE",
+        "IN-CR-02-DD-WIPE",
+    ] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::HookTerminal,
+            "{id}: expected HookTerminal"
+        );
+        assert_eq!(r.timeout_seconds, Some(30), "{id}: expected 30s timeout");
+        assert_eq!(
+            r.default_on_timeout,
+            DefaultOnTimeout::Block,
+            "{id}: expected Block on timeout"
+        );
+    }
+
+    // IN-CR-03 系列：hook_terminal, 30s, block
+    for id in [
+        "IN-CR-03-SSH-PRIVATE",
+        "IN-CR-03-SSH-DIR",
+        "IN-CR-03-AWS-CREDS",
+        "IN-CR-03-DOTENV",
+        "IN-CR-03-ETH-KEYSTORE",
+        "IN-CR-03-GPG-DIR",
+        "IN-CR-03-NETRC",
+        "IN-CR-03-MACOS-KEYCHAIN",
+        "IN-CR-03-GCP-CREDS",
+        "IN-CR-03-SOLANA-KEYPAIR",
+    ] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::HookTerminal,
+            "{id}: expected HookTerminal"
+        );
+        assert_eq!(r.timeout_seconds, Some(30), "{id}: expected 30s timeout");
+    }
+
+    // IN-CR-04 系列（9 条）：hook_terminal, 60s, block
+    for id in [
+        "IN-CR-04-SHELL-RC-APPEND",
+        "IN-CR-04-CRONTAB",
+        "IN-CR-04-CRON-D-WRITE",
+        "IN-CR-04-LAUNCHCTL",
+        "IN-CR-04-LAUNCH-AGENT-PLIST",
+        "IN-CR-04-SYSTEMCTL-ENABLE",
+        "IN-CR-04-SYSTEMD-UNIT-WRITE",
+        "IN-CR-04-FISH-CONFIG",
+        "IN-CR-04-LOGIN-ITEMS",
+    ] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::HookTerminal,
+            "{id}: expected HookTerminal"
+        );
+        assert_eq!(r.timeout_seconds, Some(60), "{id}: expected 60s timeout");
+        assert_eq!(
+            r.default_on_timeout,
+            DefaultOnTimeout::Block,
+            "{id}: expected Block on timeout"
+        );
+    }
+
+    // IN-CR-05 系列：gui_popup, 120s, block
+    for id in ["IN-CR-05-EVM", "IN-CR-05-SOLANA", "IN-CR-05-BITCOIN"] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::GuiPopup,
+            "{id}: expected GuiPopup"
+        );
+        assert_eq!(r.timeout_seconds, Some(120), "{id}: expected 120s timeout");
+        assert_eq!(
+            r.default_on_timeout,
+            DefaultOnTimeout::Block,
+            "{id}: expected Block on timeout"
+        );
+    }
+
+    // IN-GEN-01~03：hook_terminal, 30s, block
+    for id in ["IN-GEN-01", "IN-GEN-02", "IN-GEN-03"] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::HookTerminal,
+            "{id}: expected HookTerminal"
+        );
+        assert_eq!(r.timeout_seconds, Some(30), "{id}: expected 30s timeout");
+    }
+
+    // IN-GEN-04：gui_popup, 30s, block
+    let r = rules
+        .iter()
+        .find(|r| r.id == "IN-GEN-04")
+        .expect("IN-GEN-04");
+    assert_eq!(r.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(r.timeout_seconds, Some(30));
+    assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
+}
+
 // ---------------------------------------------------------------------------
 // 无害文本不命中
 // ---------------------------------------------------------------------------
diff --git a/crates/sieve-rules/tests/outbound_rules.rs b/crates/sieve-rules/tests/outbound_rules.rs
index f4967d5..225b45f 100644
--- a/crates/sieve-rules/tests/outbound_rules.rs
+++ b/crates/sieve-rules/tests/outbound_rules.rs
@@ -307,6 +307,83 @@ fn out_10_openssh_key() {
     assert_no_hit(&e, "OUT-10", "-----BEGIN OPENSSH PUBLIC KEY-----");
 }
 
+// ---------------------------------------------------------------------------
+// PRD v1.4 §5.4 新字段解析验证
+// ---------------------------------------------------------------------------
+
+/// 验证出站规则 TOML 中 disposition / timeout_seconds / default_on_timeout 正确解析。
+#[test]
+fn outbound_rules_disposition_fields_parsed() {
+    use sieve_rules::loader::load_outbound_rules;
+    use sieve_rules::manifest::{DefaultOnTimeout, Disposition};
+    use std::path::PathBuf;
+
+    let path = {
+        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
+        p.push("rules");
+        p.push("outbound.toml");
+        p
+    };
+    let rules = load_outbound_rules(&path).expect("load outbound.toml failed");
+
+    // OUT-01~05/12: auto_redact，无超时，default_on_timeout=redact
+    for id in ["OUT-01", "OUT-02", "OUT-03", "OUT-04", "OUT-05"] {
+        let r = rules
+            .iter()
+            .find(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} not found"));
+        assert_eq!(
+            r.effective_disposition(),
+            Disposition::AutoRedact,
+            "{id}: expected AutoRedact"
+        );
+        assert!(
+            r.timeout_seconds.is_none(),
+            "{id}: timeout_seconds should be None"
+        );
+        assert_eq!(
+            r.default_on_timeout,
+            DefaultOnTimeout::Redact,
+            "{id}: expected Redact on timeout"
+        );
+    }
+
+    // OUT-06: gui_popup, 15s, redact
+    let out06 = rules.iter().find(|r| r.id == "OUT-06").expect("OUT-06");
+    assert_eq!(out06.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(out06.timeout_seconds, Some(15));
+    assert_eq!(out06.default_on_timeout, DefaultOnTimeout::Redact);
+
+    // OUT-07: gui_popup, 60s, block
+    let out07 = rules.iter().find(|r| r.id == "OUT-07").expect("OUT-07");
+    assert_eq!(out07.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(out07.timeout_seconds, Some(60));
+    assert_eq!(out07.default_on_timeout, DefaultOnTimeout::Block);
+
+    // OUT-08: gui_popup, 15s, redact
+    let out08 = rules.iter().find(|r| r.id == "OUT-08").expect("OUT-08");
+    assert_eq!(out08.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(out08.timeout_seconds, Some(15));
+    assert_eq!(out08.default_on_timeout, DefaultOnTimeout::Redact);
+
+    // OUT-09: gui_popup, 60s, block
+    let out09 = rules.iter().find(|r| r.id == "OUT-09").expect("OUT-09");
+    assert_eq!(out09.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(out09.timeout_seconds, Some(60));
+    assert_eq!(out09.default_on_timeout, DefaultOnTimeout::Block);
+
+    // OUT-10: gui_popup, 60s, block
+    let out10 = rules.iter().find(|r| r.id == "OUT-10").expect("OUT-10");
+    assert_eq!(out10.effective_disposition(), Disposition::GuiPopup);
+    assert_eq!(out10.timeout_seconds, Some(60));
+    assert_eq!(out10.default_on_timeout, DefaultOnTimeout::Block);
+
+    // OUT-11: status_bar，无超时
+    let out11 = rules.iter().find(|r| r.id == "OUT-11").expect("OUT-11");
+    assert_eq!(out11.effective_disposition(), Disposition::StatusBar);
+    assert!(out11.timeout_seconds.is_none());
+}
+
 // ---------------------------------------------------------------------------
 // OUT-11: Discord Bot Token
 // ---------------------------------------------------------------------------

exec
/bin/zsh -lc 'cargo clippy --workspace --all-targets -- -D warnings' in /Users/doskey/src/sieve
 succeeded in 196ms:
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s

exec
/bin/zsh -lc 'cargo fmt --check' in /Users/doskey/src/sieve
 succeeded in 0ms:
exec
/bin/zsh -lc "nl -ba crates/sieve-rules/rules/outbound.toml | sed -n '96,190p'; nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '500,526p'; nl -ba crates/sieve-hook/src/pending.rs | sed -n '80,100p'; nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '244,258p'; nl -ba crates/sieve-cli/src/config.rs | sed -n '107,116p'; nl -ba crates/sieve-cli/src/daemon.rs | sed -n '485,500p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
    96	# ---------------------------------------------------------------------------
    97	# OUT-06: JWT Token
    98	# 格式: eyJ... (header.payload.signature，三段均为 base64url)
    99	# 不用 lookahead，直接匹配三段结构
   100	# ---------------------------------------------------------------------------
   101	[[rules]]
   102	id = "OUT-06"
   103	description = "JWT Token (eyJ...)"
   104	pattern = 'ey[A-Za-z0-9_\-]{16,}\.ey[A-Za-z0-9_\/\-]{16,}\.[A-Za-z0-9_\/\-]{10,}'
   105	severity = "high"
   106	action = "block"
   107	entropy_min = 3.5
   108	keywords = ["eyJ"]
   109	allowlist_regexes = []
   110	allowlist_stopwords = []
   111	disposition = "gui_popup"
   112	timeout_seconds = 15
   113	default_on_timeout = "redact"
   114	
   115	# ---------------------------------------------------------------------------
   116	# OUT-07: PEM Private Key Header
   117	# 覆盖: RSA / EC / DSA / PKCS#8 / generic PRIVATE KEY 头部
   118	# 注意: 不包含 OPENSSH（由 OUT-10 专项覆盖）
   119	# ---------------------------------------------------------------------------
   120	[[rules]]
   121	id = "OUT-07"
   122	description = "PEM Private Key (RSA / EC / DSA / generic)"
   123	pattern = '-----BEGIN[ A-Z0-9_\-]{0,60}PRIVATE KEY[ A-Z]{0,20}-----'
   124	severity = "critical"
   125	action = "block"
   126	entropy_min = 0.0
   127	keywords = ["-----BEGIN"]
   128	allowlist_regexes = []
   129	allowlist_stopwords = []
   130	disposition = "gui_popup"
   131	timeout_seconds = 60
   132	default_on_timeout = "block"
   133	
   134	# ---------------------------------------------------------------------------
   135	# OUT-08: Stripe Live Secret / Publishable / Restricted Key
   136	# 格式: sk_live_/pk_live_/rk_live_ + 10~99 alnum
   137	# ---------------------------------------------------------------------------
   138	[[rules]]
   139	id = "OUT-08"
   140	description = "Stripe Live Key (sk_live_/pk_live_/rk_live_)"
   141	pattern = '(?:sk|pk|rk)_live_[a-zA-Z0-9]{10,99}'
   142	severity = "critical"
   143	action = "block"
   144	entropy_min = 3.5
   145	keywords = ["_live_"]
   146	allowlist_regexes = ['(?i)test|example']
   147	allowlist_stopwords = []
   148	disposition = "gui_popup"
   149	timeout_seconds = 15
   150	default_on_timeout = "redact"
   151	
   152	# ---------------------------------------------------------------------------
   153	# OUT-09: Slack Token
   154	# 格式: xoxb-/xoxp-/xoxa-/xoxs- + 10+ alnum/-
   155	# ---------------------------------------------------------------------------
   156	[[rules]]
   157	id = "OUT-09"
   158	description = "Slack Token (xoxb-/xoxp-/xoxa-/xoxs-)"
   159	pattern = 'xox[bpas]\-[0-9A-Za-z\-]{10,}'
   160	severity = "high"
   161	action = "block"
   162	entropy_min = 3.0
   163	keywords = ["xoxb", "xoxp", "xoxa", "xoxs"]
   164	allowlist_regexes = []
   165	allowlist_stopwords = []
   166	disposition = "gui_popup"
   167	timeout_seconds = 60
   168	default_on_timeout = "block"
   169	
   170	# ---------------------------------------------------------------------------
   171	# OUT-10: OpenSSH Private Key Header
   172	# 专项规则，不依赖 OUT-07 的通用 PEM 规则
   173	# ---------------------------------------------------------------------------
   174	[[rules]]
   175	id = "OUT-10"
   176	description = "OpenSSH Private Key (-----BEGIN OPENSSH PRIVATE KEY-----)"
   177	pattern = '-----BEGIN OPENSSH PRIVATE KEY-----'
   178	severity = "critical"
   179	action = "block"
   180	entropy_min = 0.0
   181	keywords = ["BEGIN OPENSSH"]
   182	allowlist_regexes = []
   183	allowlist_stopwords = []
   184	disposition = "gui_popup"
   185	timeout_seconds = 60
   186	default_on_timeout = "block"
   187	
   188	# ---------------------------------------------------------------------------
   189	# OUT-11: Discord Bot Token
   190	# 格式: 24~28 base64url . 6 base64url . 27~38 base64url
   500	    /// 构建默认 sieve.toml 内容（所有路径使用绝对路径）。
   501	    pub(super) fn build_default_sieve_toml(sieve_toml_path: &Path) -> Result<String> {
   502	        let sieve_home = sieve_toml_path
   503	            .parent()
   504	            .ok_or_else(|| anyhow!("sieve.toml 路径无父目录"))?;
   505	        let rules_dir = sieve_home.join("rules");
   506	        let audit_db = sieve_home.join("audit.db");
   507	
   508	        Ok(format!(
   509	            r#"# sieve.toml — 由 `sieve setup` 自动生成，所有路径为绝对路径
   510	# 修改后需重启 daemon：launchctl kickstart -k gui/$(id -u)/com.sieve.daemon
   511	
   512	[proxy]
   513	listen = "127.0.0.1:11453"
   514	upstream = "https://api.anthropic.com"
   515	
   516	[rules]
   517	# 规则文件目录（绝对路径，launchd 从 / 启动时不依赖 cwd）
   518	dir = "{rules_dir}"
   519	
   520	[audit]
   521	# 审计日志数据库（绝对路径）
   522	db = "{audit_db}"
   523	"#,
   524	            rules_dir = rules_dir.display(),
   525	            audit_db = audit_db.display(),
   526	        ))
    80	        }
    81	        let content = match std::fs::read_to_string(&path) {
    82	            Ok(c) => c,
    83	            Err(_) => continue,
    84	        };
    85	        let req: DecisionRequest = match serde_json::from_str(&content) {
    86	            Ok(r) => r,
    87	            Err(_) => continue, // 解析失败的文件跳过。
    88	        };
    89	        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
    90	        if age_secs > stale_threshold_secs {
    91	            stale_paths.push(path);
    92	        } else {
    93	            fresh.push(req);
    94	        }
    95	    }
    96	
    97	    // 按 created_at 升序排列，保证确定性。
    98	    fresh.sort_by_key(|r| r.created_at);
    99	
   100	    ScanResult { fresh, stale_paths }
   244	                            if let Some(bd) = backup_dir {
   245	                                restore_file_from_backup(bd, &info.path)?;
   246	                            }
   247	                        }
   248	                    }
   249	                } else if extension == "toml" {
   250	                    // sieve.toml 属于 Sieve 自己，直接删除
   251	                    fs::remove_file(&info.path)
   252	                        .with_context(|| format!("删除 {} 失败", info.path.display()))?;
   253	                    println!("[uninstall] ✅ 删除: {}", info.path.display());
   254	                } else {
   255	                    // 其他文件：从备份恢复
   256	                    if let Some(bd) = backup_dir {
   257	                        restore_file_from_backup(bd, &info.path)?;
   258	                    }
   107	    /// 是否启用 GUI Unix socket（默认 `false`；Week 6+ 启用）。
   108	    #[serde(default = "default_gui_socket_enabled")]
   109	    #[allow(dead_code)]
   110	    pub gui_socket_enabled: bool,
   111	
   112	    /// SQLite 审计数据库路径（Week 5；`None` 时沿用 `log_path` 或 `~/.sieve/audit.db`）。
   113	    #[serde(default)]
   114	    #[allow(dead_code)]
   115	    pub audit_db_path: Option<PathBuf>,
   116	}
   485	                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
   486	                    if !hold_detections.is_empty() {
   487	                        if let Some(ref ipc_server) = ipc {
   488	                            // keep-alive channel：daemon 把心跳写入 SSE 流
   489	                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
   490	                            let tx_ka = tx.clone();
   491	
   492	                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
   493	                            // 然后再 hold——这样客户端已经看到触发 event，
   494	                            // hold 期间只收到 keep-alive comment。
   495	                            if tx
   496	                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
   497	                                .await
   498	                                .is_err()
   499	                            {
   500	                                return;

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/engine_adapter.rs | sed -n '67,84p'; nl -ba crates/sieve-cli/src/config.rs | sed -n '274,285p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
    67	fn map_action_by_disposition(
    68	    disposition: sieve_rules::manifest::Disposition,
    69	    _rule_action: RulesAction,
    70	    rule_id: &str,
    71	    timeout_seconds: u32,
    72	) -> Action {
    73	    use sieve_rules::manifest::Disposition;
    74	    match disposition {
    75	        Disposition::AutoRedact => Action::Redact {
    76	            placeholder: format!("[REDACTED:{rule_id}]"),
    77	        },
    78	        Disposition::GuiPopup => Action::HoldForDecision {
    79	            request_id: uuid::Uuid::new_v4(),
    80	            timeout_seconds,
    81	        },
    82	        Disposition::HookTerminal => Action::HookMark,
    83	        Disposition::StatusBar => Action::MarkOnly,
    84	    }
   274	    /// 解析审计日志路径。`log_path` 显式给定时直接用,否则回退到 `~/.sieve/audit.db`。
   275	    ///
   276	    /// # Errors
   277	    /// `$HOME` 不存在或不可识别时返回错误。
   278	    pub fn audit_db_path(&self) -> Result<PathBuf> {
   279	        if let Some(p) = &self.log_path {
   280	            return Ok(p.clone());
   281	        }
   282	        let home = std::env::var_os("HOME")
   283	            .ok_or_else(|| anyhow!("HOME env var not set; specify log_path explicitly"))?;
   284	        Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
   285	    }

codex
当前改动包含会让 setup 后 daemon 无法启动的配置生成问题，并且出站 gui_popup 规则会绕过现有拦截路径，属于安全功能回归。另有 pending 生命周期、卸载数据恢复和配置项未生效等问题需要修复。

Full review comments:

- [P1] 为出站 gui_popup 接入实际拦截 — /Users/doskey/src/sieve/crates/sieve-cli/src/engine_adapter.rs:78-81
  当 `OutboundAdapter` 处理 OUT-06/07/08/09/10 这类出站规则时，`gui_popup` 会被映射成 `HoldForDecision`，但出站代理逻辑只处理 `Redact` 和 `Block`；因此例如 OUT-07 私钥命中后既不会弹窗也不会阻断/脱敏，会直接转发给上游。

- [P1] 生成当前 Config 能解析的 setup 配置 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:512-514
  `sieve setup` 写出的 `~/.sieve/sieve.toml` 从 `[proxy]` 表开始，但 `Config` 仍是扁平字段并启用了 `deny_unknown_fields`；launchd 随后执行 `sieve start --config ~/.sieve/sieve.toml` 会在解析到未知字段 `proxy` 时失败，导致一键安装后的 daemon 起不来。

- [P1] Hold 决策前不要透传触发帧 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:492-497
  在 `HoldForDecision` 命中时，这里先把触发检测的 SSE 帧发给客户端再等待用户决策；如果该帧包含最终的危险 tool_use 片段或恶意 assistant 文本，即使用户随后拒绝，客户端也已经收到关键内容。应先缓存当前帧，只有 Allow 后再发送。

- [P2] 跳过已有决策的 pending 请求 — /Users/doskey/src/sieve/crates/sieve-hook/src/pending.rs:93-93
  setup 注册的是静态 `sieve-hook check`，所以之后每次 hook 都会走目录启发式扫描；这里会把所有未过期 pending 都加入，即使对应 `decisions/<id>.json` 已经写入，而 `write_decision` 也不会删除 pending。结果是一次已处理的检测会在 10 分钟内反复弹窗或阻断后续无关工具调用。

- [P2] 卸载时恢复已有 sieve.toml — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/uninstall.rs:249-252
  当用户在 setup 前已经有 `~/.sieve/sieve.toml` 时，setup 会备份并记录 `created_new=false`；但 uninstall 进入这个 `.toml` 分支后直接删除文件，而不是从备份恢复，导致用户原有配置丢失。

- [P2] 实际使用 audit_db_path 配置项 — /Users/doskey/src/sieve/crates/sieve-cli/src/config.rs:112-115
  这里新增并反序列化了 `audit_db_path`，但 `Config::audit_db_path()` 仍只检查旧的 `log_path` 后回退到 `~/.sieve/audit.db`；用户在配置里设置 `audit_db_path` 时会被静默忽略，审计库写到错误位置。
2026-04-28T02:24:52.829482Z ERROR codex_core::session: failed to record rollout items: thread 019dd1e1-3ced-7151-88c3-9ae7f1cf13d3 not found
2026-04-28T02:24:52.865069Z ERROR codex_core::session: failed to record rollout items: thread 019dd1e1-3cb5-7830-ac29-5a9a689f10d1 not found
当前改动包含会让 setup 后 daemon 无法启动的配置生成问题，并且出站 gui_popup 规则会绕过现有拦截路径，属于安全功能回归。另有 pending 生命周期、卸载数据恢复和配置项未生效等问题需要修复。

Full review comments:

- [P1] 为出站 gui_popup 接入实际拦截 — /Users/doskey/src/sieve/crates/sieve-cli/src/engine_adapter.rs:78-81
  当 `OutboundAdapter` 处理 OUT-06/07/08/09/10 这类出站规则时，`gui_popup` 会被映射成 `HoldForDecision`，但出站代理逻辑只处理 `Redact` 和 `Block`；因此例如 OUT-07 私钥命中后既不会弹窗也不会阻断/脱敏，会直接转发给上游。

- [P1] 生成当前 Config 能解析的 setup 配置 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:512-514
  `sieve setup` 写出的 `~/.sieve/sieve.toml` 从 `[proxy]` 表开始，但 `Config` 仍是扁平字段并启用了 `deny_unknown_fields`；launchd 随后执行 `sieve start --config ~/.sieve/sieve.toml` 会在解析到未知字段 `proxy` 时失败，导致一键安装后的 daemon 起不来。

- [P1] Hold 决策前不要透传触发帧 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:492-497
  在 `HoldForDecision` 命中时，这里先把触发检测的 SSE 帧发给客户端再等待用户决策；如果该帧包含最终的危险 tool_use 片段或恶意 assistant 文本，即使用户随后拒绝，客户端也已经收到关键内容。应先缓存当前帧，只有 Allow 后再发送。

- [P2] 跳过已有决策的 pending 请求 — /Users/doskey/src/sieve/crates/sieve-hook/src/pending.rs:93-93
  setup 注册的是静态 `sieve-hook check`，所以之后每次 hook 都会走目录启发式扫描；这里会把所有未过期 pending 都加入，即使对应 `decisions/<id>.json` 已经写入，而 `write_decision` 也不会删除 pending。结果是一次已处理的检测会在 10 分钟内反复弹窗或阻断后续无关工具调用。

- [P2] 卸载时恢复已有 sieve.toml — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/uninstall.rs:249-252
  当用户在 setup 前已经有 `~/.sieve/sieve.toml` 时，setup 会备份并记录 `created_new=false`；但 uninstall 进入这个 `.toml` 分支后直接删除文件，而不是从备份恢复，导致用户原有配置丢失。

- [P2] 实际使用 audit_db_path 配置项 — /Users/doskey/src/sieve/crates/sieve-cli/src/config.rs:112-115
  这里新增并反序列化了 `audit_db_path`，但 `Config::audit_db_path()` 仍只检查旧的 `log_path` 后回退到 `~/.sieve/audit.db`；用户在配置里设置 `audit_db_path` 时会被静默忽略，审计库写到错误位置。
