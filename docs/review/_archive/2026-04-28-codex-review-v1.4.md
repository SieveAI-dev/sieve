OpenAI Codex v0.125.0 (research preview)
--------
workdir: /Users/doskey/src/sieve
model: gpt-5.5
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/doskey/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019dd1bd-411d-7b42-a502-460cdb1c9d98
--------
user
current changes
exec
/bin/zsh -lc 'git status --short && git rev-parse --show-toplevel' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
?? tasks/todo.md
/Users/doskey/src/sieve

exec
/bin/zsh -lc "git diff --stat && printf '\\n--- staged ---\\n' && git diff --cached --stat && printf '\\n--- untracked tree ---\\n' && find crates/sieve-cli/src/commands crates/sieve-cli/tests crates/sieve-core/src/pipeline crates/sieve-hook crates/sieve-ipc docs/design docs/specs docs/prd tasks -maxdepth 3 -type f 2>/dev/null | sort" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
 .cursorrules                                       |  21 +-
 CLAUDE.md                                          |  40 +-
 Cargo.lock                                         | Bin 56453 -> 64212 bytes
 Cargo.toml                                         |   6 +
 README.md                                          |  12 +-
 SECURITY.md                                        |  14 +-
 crates/sieve-cli/Cargo.toml                        |   4 +
 crates/sieve-cli/src/audit.rs                      | 376 +++++++++++++++++--
 crates/sieve-cli/src/cli.rs                        |  40 ++
 crates/sieve-cli/src/config.rs                     |  98 ++++-
 crates/sieve-cli/src/daemon.rs                     | 416 ++++++++++++++++++---
 crates/sieve-cli/src/engine_adapter.rs             |  76 +++-
 crates/sieve-cli/src/main.rs                       |  18 +-
 crates/sieve-core/Cargo.toml                       |   7 +
 crates/sieve-core/src/detection.rs                 |  28 +-
 crates/sieve-core/src/pipeline/inbound.rs          |   2 +-
 crates/sieve-core/src/pipeline/mod.rs              | 390 ++++++++++++++++++-
 crates/sieve-rules/benches/scan_bench.rs           |   5 +-
 crates/sieve-rules/rules/inbound.toml              |  99 +++++
 crates/sieve-rules/rules/outbound.toml             |  26 ++
 crates/sieve-rules/src/critical_lock.rs            | 170 ++++++++-
 crates/sieve-rules/src/engine/mod.rs               |   3 +
 crates/sieve-rules/src/manifest.rs                 | 219 ++++++++++-
 crates/sieve-rules/tests/inbound_rules.rs          | 142 +++++++
 crates/sieve-rules/tests/outbound_rules.rs         |  77 ++++
 docs/api/api-reference.md                          | 166 ++++++--
 docs/changelog/CHANGELOG.md                        |  43 +++
 docs/design/ADR-001-rust-tech-stack.md             |   8 +-
 docs/design/ADR-002-rule-engine-only-phase1.md     |  16 +-
 .../design/ADR-003-local-only-no-cloud-verifier.md |  16 +-
 .../ADR-004-anthropic-first-unified-interface.md   |  32 +-
 docs/design/ADR-005-overseas-legal-entity.md       |  22 +-
 docs/design/ADR-006-sigstore-reproducible-build.md |  24 +-
 .../design/ADR-007-fail-closed-critical-actions.md |  26 ++
 docs/design/ADR-011-private-until-ga.md            |   8 +-
 docs/design/ADR-INDEX.md                           |   9 +-
 docs/design/architecture.md                        | 213 ++++++-----
 docs/design/data-model.md                          | 121 ++++--
 docs/glossary.md                                   |  50 +--
 docs/guides/deployment.md                          | 280 ++++++--------
 docs/guides/development.md                         |  70 +++-
 docs/requirements/PRD-sieve.md                     |  47 +--
 docs/requirements/user-stories.md                  | 178 ++++++---
 tasks/roadmap.md                                   |  77 ++--
 44 files changed, 3014 insertions(+), 681 deletions(-)

--- staged ---

--- untracked tree ---
crates/sieve-cli/src/commands/doctor.rs
crates/sieve-cli/src/commands/mod.rs
crates/sieve-cli/src/commands/setup.rs
crates/sieve-cli/src/commands/uninstall.rs
crates/sieve-cli/tests/audit_append_only.rs
crates/sieve-cli/tests/inbound_block.rs
crates/sieve-cli/tests/outbound_block.rs
crates/sieve-cli/tests/proxy_passthrough.rs
crates/sieve-cli/tests/sieve_setup_dry_run.rs
crates/sieve-core/src/pipeline/inbound.rs
crates/sieve-core/src/pipeline/inbound_hold.rs
crates/sieve-core/src/pipeline/inbound_hook.rs
crates/sieve-core/src/pipeline/mod.rs
crates/sieve-core/src/pipeline/outbound.rs
crates/sieve-core/src/pipeline/outbound_redact.rs
crates/sieve-core/src/pipeline/streaming.rs
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
docs/design/ADR-001-rust-tech-stack.md
docs/design/ADR-002-rule-engine-only-phase1.md
docs/design/ADR-003-local-only-no-cloud-verifier.md
docs/design/ADR-004-anthropic-first-unified-interface.md
docs/design/ADR-005-overseas-legal-entity.md
docs/design/ADR-006-sigstore-reproducible-build.md
docs/design/ADR-007-fail-closed-critical-actions.md
docs/design/ADR-011-private-until-ga.md
docs/design/ADR-012-native-gui-app-phase1.md
docs/design/ADR-013-ipc-protocol.md
docs/design/ADR-014-dual-layer-defense.md
docs/design/ADR-015-sieve-setup-tool.md
docs/design/ADR-016-disposition-matrix-2d.md
docs/design/ADR-INDEX.md
docs/design/architecture.md
docs/design/data-model.md
docs/prd/sieve-prd-v1.0.md
docs/prd/sieve-prd-v1.1.md
docs/prd/sieve-prd-v1.2.md
docs/prd/sieve-prd-v1.3.md
docs/prd/sieve-prd-v1.4.md
docs/specs/SPEC-001-sieve-hook-protocol.md
docs/specs/SPEC-002-hips-popup-behavior.md
docs/specs/SPEC-003-sieve-setup-tool.md
tasks/lessons.md
tasks/roadmap.md
tasks/todo.md

exec
/bin/zsh -lc "git diff -- Cargo.toml crates/sieve-cli/Cargo.toml crates/sieve-core/Cargo.toml && printf '\\n--- hook cargo ---\\n' && cat crates/sieve-hook/Cargo.toml && printf '\\n--- ipc cargo ---\\n' && cat crates/sieve-ipc/Cargo.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

--- hook cargo ---
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

--- ipc cargo ---
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

exec
/bin/zsh -lc 'git diff -- crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-cli/src/config.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/audit.rs crates/sieve-cli/src/engine_adapter.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
diff --git a/crates/sieve-cli/src/daemon.rs b/crates/sieve-cli/src/daemon.rs
index aed965a..729f052 100644
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
+use sieve_core::pipeline::outbound_redact::{redact_body_bytes, RedactHit};
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
 
@@ -192,11 +235,29 @@ async fn proxy_inner(
         }
 
         // 4. 决策：
-        //    - fail-closed Critical 规则：无视 dry_run，永远 block（PRD §9 #3）
-        //    - 非 fail-closed Critical：dry_run=true 时仅 warn，dry_run=false 时 block
+        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
+        //    b. fail-closed Critical Block → 426（PRD §9 #3）
+        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
+        //    d. 其余 → 透传
+
+        // 4a. 收集需要脱敏的 hit
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
@@ -214,6 +275,46 @@ async fn proxy_inner(
             return Ok(build_426_response(&cloned));
         }
 
+        // 4a. AutoRedact：脱敏 body bytes 后转发（不返回 426）
+        if !redact_hits.is_empty() {
+            tracing::info!(
+                count = redact_hits.len(),
+                rules = redact_hits
+                    .iter()
+                    .map(|h| h.rule_id.as_str())
+                    .collect::<Vec<_>>()
+                    .join(", "),
+                "OUTBOUND AUTO-REDACT"
+            );
+            let result = redact_body_bytes(&body_bytes, &redact_hits);
+            let new_body = Bytes::from(result.body);
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
@@ -224,9 +325,7 @@ async fn proxy_inner(
             }
         }
 
-        // 5. prompt 地址 seed：把出站 prompt 中的 EVM 地址预先注入 InboundFilter 会话，
-        //    使首轮地址替换（prompt 地址 A → 响应地址 B）可被 IN-CR-01 检测。
-        //    关联 PRD §4.2 / P0-3 修复。
+        // 5. prompt 地址 seed
         for (_, text) in &texts {
             if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                 tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
@@ -238,6 +337,7 @@ async fn proxy_inner(
             forwarder,
             inbound_filter,
             dry_run,
+            ipc,
             parts,
             body_bytes,
         )
@@ -251,15 +351,21 @@ async fn proxy_inner(
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
@@ -286,13 +392,11 @@ async fn forward_with_inbound_inspection(
 
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
@@ -308,10 +412,9 @@ async fn forward_with_inbound_inspection(
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
@@ -329,13 +432,132 @@ async fn forward_with_inbound_inspection(
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
 
+                    // 1. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
+                    for d in &hook_detections {
+                        write_hook_pending_silent(d);
+                    }
+
+                    // 2. GUI 类：hold 流 + keep-alive + 等用户决策
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
+                    // 3. Block 类：注入 sieve_blocked 并截流
                     if !blocking.is_empty() {
                         tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                         for d in &blocking {
@@ -343,17 +565,16 @@ async fn forward_with_inbound_inspection(
                         }
                         let blocked_payload = build_sieve_blocked_sse(&blocking);
                         let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
-                        // drop tx → channel 关闭 → 客户端收到 EOF
                         return;
                     }
 
-                    // 无 blocking：透传原始 frame（字节级一致）
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
@@ -367,11 +588,15 @@ async fn forward_with_inbound_inspection(
             }
         }
 
-        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event，
-        // 同样走完整 blocking 决策——修复 P0-4 / PRD §9 #5 "提前断流"硬约束。
+        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
         let flushed = parser.flush();
-        let blocking =
-            collect_blocking_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
+        let (blocking, hook_detections, _hold_detections) =
+            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
+
+        for d in &hook_detections {
+            write_hook_pending_silent(d);
+        }
+
         if !blocking.is_empty() {
             tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
             for d in &blocking {
@@ -379,7 +604,6 @@ async fn forward_with_inbound_inspection(
             }
             let blocked_payload = build_sieve_blocked_sse(&blocking);
             let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
-            // drop tx → 客户端收到 EOF
         }
     });
 
@@ -391,33 +615,34 @@ async fn forward_with_inbound_inspection(
     Ok(Response::from_parts(resp_parts, response_body))
 }
 
-/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，返回应触发 blocking 的 [`Detection`] 列表。
+/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
+/// - `blocking`：`Action::Block` 需立即截流的 detections
+/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
+/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
+///
+/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
 ///
-/// 被 push_chunk 分支和 flush 分支共同调用，确保两条路径走完全相同的 blocking 决策逻辑，
-/// 修复 P0-4 / PRD §9 #5"提前断流"硬约束：flush 出来的残留 event 同样必须阻断 Critical。
-fn collect_blocking_detections(
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
@@ -425,25 +650,99 @@ fn collect_blocking_detections(
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
+            }
+        }
+    }
+
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
             }
         }
     }
 
-    // 决策：fail-closed Critical 永远阻断；非 fail-closed 遵 dry_run
-    critical_hits
-        .into_iter()
-        .filter(|d| sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
-        .collect()
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
@@ -537,9 +836,6 @@ async fn forward_streaming(
 }
 
 /// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
-///
-/// body 为 JSON，含命中规则 ID / fingerprint / 操作指引。
-/// 时间戳当前为 UNIX epoch 秒，Week 4 引入 chrono 后改为完整 RFC3339。
 fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
     let blocked_at = epoch_secs_string();
     let detections_json: Vec<serde_json::Value> = detections
@@ -610,8 +906,6 @@ fn empty_body() -> ResponseBody {
 }
 
 /// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
-///
-/// 畸形 partial_json 不对应具体文本 span，evidence_truncated 存 tool_id。
 fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
@@ -629,8 +923,6 @@ fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
 }
 
 /// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
-///
-/// 容量超限不对应具体文本 span，因此 span 设 [0, 0)，evidence_truncated 为空。
 fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
diff --git a/crates/sieve-cli/src/engine_adapter.rs b/crates/sieve-cli/src/engine_adapter.rs
index 374a174..2e4b278 100644
--- a/crates/sieve-cli/src/engine_adapter.rs
+++ b/crates/sieve-cli/src/engine_adapter.rs
@@ -50,13 +50,47 @@ fn map_severity(r: RulesSeverity) -> Severity {
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
 fn map_action(r: RulesAction) -> Action {
     match r {
         RulesAction::Block => Action::Block,
-        RulesAction::Warn => Action::WarnConfirm { countdown_secs: 5 },
+        RulesAction::Warn => Action::HookMark,
         RulesAction::Mark => Action::MarkOnly,
         RulesAction::Allow => Action::SilentLog,
     }
@@ -131,7 +165,18 @@ fn scan_text(
             let raw_action = rule.map(|r| r.action).unwrap_or(RulesAction::Block);
             let enforced_action =
                 sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw_action);
-            let action = map_action(enforced_action);
+            // v1.4：disposition 路由优先于 raw action 映射
+            let action = if enforced_action == RulesAction::Block {
+                // fail-closed 强制 Block，跳过 disposition 路由
+                Action::Block
+            } else {
+                rule.map(|r| {
+                    let disp = r.effective_disposition();
+                    let timeout = r.timeout_seconds.unwrap_or(60);
+                    map_action_by_disposition(disp, enforced_action, &hit.rule_id, timeout)
+                })
+                .unwrap_or_else(|| map_action(enforced_action))
+            };
 
             let evidence_truncated = redact_evidence(matched_text);
             let fp = fingerprint(&hit.rule_id, matched_text);
@@ -207,7 +252,20 @@ fn scan_text(
             let severity = rule
                 .map(|r| map_severity(r.severity))
                 .unwrap_or(Severity::Critical);
-            let action = rule.map(|r| map_action(r.action)).unwrap_or(Action::Block);
+            // v1.4：disposition 路由
+            let action = rule
+                .map(|r| {
+                    let raw = r.action;
+                    let enforced = sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw);
+                    if enforced == RulesAction::Block {
+                        Action::Block
+                    } else {
+                        let disp = r.effective_disposition();
+                        let timeout = r.timeout_seconds.unwrap_or(60);
+                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
+                    }
+                })
+                .unwrap_or(Action::Block);
             let evidence_truncated = redact_evidence(matched_text);
             let fp = fingerprint(&hit.rule_id, matched_text);
 
@@ -284,6 +342,9 @@ fn make_rule(
             keywords: vec![],
             allowlist_regexes: vec![],
             allowlist_stopwords: vec![],
+            disposition: None,
+            timeout_seconds: None,
+            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
         }
     }
 
@@ -327,9 +388,10 @@ fn scan_no_match_returns_empty() {
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
diff --git a/crates/sieve-cli/src/main.rs b/crates/sieve-cli/src/main.rs
index ff180aa..7e7d761 100644
--- a/crates/sieve-cli/src/main.rs
+++ b/crates/sieve-cli/src/main.rs
@@ -1,7 +1,11 @@
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
 
 #![forbid(unsafe_code)]
 
@@ -13,6 +17,7 @@
 
 mod audit;
 mod cli;
+mod commands;
 mod config;
 mod daemon;
 mod engine_adapter;
@@ -125,6 +130,15 @@ async fn main() -> Result<()> {
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

exec
/bin/zsh -lc "find crates -maxdepth 3 -type f | sort | sed -n '1,200p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-cli/Cargo.toml
crates/sieve-cli/src/audit.rs
crates/sieve-cli/src/cli.rs
crates/sieve-cli/src/config.rs
crates/sieve-cli/src/daemon.rs
crates/sieve-cli/src/engine_adapter.rs
crates/sieve-cli/src/main.rs
crates/sieve-cli/tests/audit_append_only.rs
crates/sieve-cli/tests/inbound_block.rs
crates/sieve-cli/tests/outbound_block.rs
crates/sieve-cli/tests/proxy_passthrough.rs
crates/sieve-cli/tests/sieve_setup_dry_run.rs
crates/sieve-core/Cargo.toml
crates/sieve-core/src/address_guard.rs
crates/sieve-core/src/detection.rs
crates/sieve-core/src/error.rs
crates/sieve-core/src/fuzz_helpers.rs
crates/sieve-core/src/lib.rs
crates/sieve-core/src/tool_use_aggregator.rs
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
crates/sieve-rules/Cargo.toml
crates/sieve-rules/benches/scan_bench.rs
crates/sieve-rules/rules/inbound.toml
crates/sieve-rules/rules/outbound.toml
crates/sieve-rules/src/bip39.rs
crates/sieve-rules/src/critical_lock.rs
crates/sieve-rules/src/ed25519.rs
crates/sieve-rules/src/error.rs
crates/sieve-rules/src/lib.rs
crates/sieve-rules/src/loader.rs
crates/sieve-rules/src/manifest.rs
crates/sieve-rules/src/placeholder.rs
crates/sieve-rules/src/wordlist.rs
crates/sieve-rules/tests/bip39_pipeline.rs
crates/sieve-rules/tests/inbound_rules.rs
crates/sieve-rules/tests/outbound_rules.rs
crates/sieve-rules/wordlist/english.txt

exec
/bin/zsh -lc 'find crates/sieve-core/src -maxdepth 4 -type f | sort' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-core/src/address_guard.rs
crates/sieve-core/src/detection.rs
crates/sieve-core/src/error.rs
crates/sieve-core/src/forwarder/mod.rs
crates/sieve-core/src/fuzz_helpers.rs
crates/sieve-core/src/lib.rs
crates/sieve-core/src/pipeline/inbound.rs
crates/sieve-core/src/pipeline/inbound_hold.rs
crates/sieve-core/src/pipeline/inbound_hook.rs
crates/sieve-core/src/pipeline/mod.rs
crates/sieve-core/src/pipeline/outbound.rs
crates/sieve-core/src/pipeline/outbound_redact.rs
crates/sieve-core/src/pipeline/streaming.rs
crates/sieve-core/src/protocol/anthropic.rs
crates/sieve-core/src/protocol/mod.rs
crates/sieve-core/src/protocol/unified_message.rs
crates/sieve-core/src/sse/mod.rs
crates/sieve-core/src/sse/parser.rs
crates/sieve-core/src/sse/passthrough.rs
crates/sieve-core/src/tool_use_aggregator.rs

exec
/bin/zsh -lc "git diff -- crates/sieve-core/src/detection.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/pipeline/inbound.rs && printf '\\n--- untracked pipeline files ---\\n' && for f in crates/sieve-core/src/pipeline/outbound.rs crates/sieve-core/src/pipeline/streaming.rs crates/sieve-core/src/pipeline/inbound_hold.rs crates/sieve-core/src/pipeline/inbound_hook.rs crates/sieve-core/src/pipeline/outbound_redact.rs; do echo '###' "'$f; sed -n '"'1,260p' "'$f; done' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

--- untracked pipeline files ---
### crates/sieve-core/src/pipeline/outbound.rs
//! 出站规则匹配节点（Week 2 起实现）。
//!
//! 关联 PRD §5.1 出站检测 P0 表 + ADR-002 纯规则引擎。
//!
//! Week 2 由 sieve-cli 在启动时把 sieve-rules 的 VectorscanEngine 适配到
//! [`OutboundEngine`] trait，避免 sieve-core 直接依赖 sieve-rules（见 .cursorrules §3.3）。

use crate::detection::Detection;
use crate::error::SieveCoreResult;
use crate::pipeline::PipelineNode;
use crate::protocol::unified_message::UnifiedMessage;
use std::collections::HashSet;
use std::sync::Arc;

/// 出站规则扫描的抽象引擎接口。
///
/// 由 sieve-rules 的 VectorscanEngine 在 sieve-cli 启动时实现并注入，保持
/// sieve-core 不依赖 sieve-rules（crate 边界，.cursorrules §3.3）。
pub trait OutboundEngine: Send + Sync {
    /// 扫描文本，返回命中列表（已应用 placeholder 黑名单 + per-rule allowlist 过滤）。
    ///
    /// - `input`：待扫描的 UTF-8 文本。
    /// - `source`：内容来源标记（用于填充 Detection.source）。
    /// - `body_byte_offset`：该文本在原始请求 body 中的起始字节偏移（用于生成绝对 span）。
    fn scan_text(
        &self,
        input: &str,
        source: crate::detection::ContentSource,
        body_byte_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>>;
}

/// 出站规则匹配 Pipeline 节点。
///
/// 只扫 [`crate::protocol::unified_message::Role::User`] 和
/// [`crate::protocol::unified_message::Role::System`] 角色的 Text 内容块；
/// Assistant / Tool 消息跳过（出站方向不含这两种角色）。
pub struct OutboundFilter {
    engine: Arc<dyn OutboundEngine>,
    /// `.sieveignore` 加载的 fingerprint 集合（O(1) 查询）。
    sieveignore: Arc<HashSet<String>>,
}

impl OutboundFilter {
    /// 新建 OutboundFilter。
    pub fn new(engine: Arc<dyn OutboundEngine>, sieveignore: Arc<HashSet<String>>) -> Self {
        Self {
            engine,
            sieveignore,
        }
    }
}

impl PipelineNode for OutboundFilter {
    fn name(&self) -> &str {
        "outbound-filter"
    }

    fn process(&self, msg: &mut UnifiedMessage) -> SieveCoreResult<Vec<Detection>> {
        use crate::detection::ContentSource;
        use crate::protocol::unified_message::{ContentBlock, Role};

        // 出站消息只扫 User / System 角色的 Text 块。
        let source = match msg.role {
            Role::System => ContentSource::OutboundSystemText,
            Role::User => ContentSource::OutboundUserText,
            _ => return Ok(vec![]),
        };

        let mut all_hits: Vec<Detection> = Vec::new();

        for block in &msg.content_blocks {
            if let ContentBlock::Text { text, span } = block {
                let body_offset = span.map(|s| s.start).unwrap_or(0);
                let hits = self.engine.scan_text(text, source, body_offset)?;
                for d in hits {
                    if !self.sieveignore.contains(&d.fingerprint) {
                        all_hits.push(d);
                    }
                }
            }
        }

        Ok(all_hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
    use crate::protocol::unified_message::{
        ContentBlock, ContentSpan, Direction, MessageMetadata, Role, UnifiedMessage,
        UpstreamProvider,
    };
    use std::time::SystemTime;
    use uuid::Uuid;

    /// Mock OutboundEngine：固定命中 "secret" 字符串。
    struct MockEngine;

    impl OutboundEngine for MockEngine {
        fn scan_text(
            &self,
            input: &str,
            source: ContentSource,
            body_offset: usize,
        ) -> SieveCoreResult<Vec<Detection>> {
            if let Some(idx) = input.find("secret") {
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "OUT-MOCK".into(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    span: ContentSpan {
                        start: body_offset + idx,
                        end: body_offset + idx + "secret".len(),
                    },
                    evidence_truncated: "***".into(),
                    fingerprint: fingerprint("OUT-MOCK", "secret"),
                }])
            } else {
                Ok(vec![])
            }
        }
    }

    fn user_msg(text: &str) -> UnifiedMessage {
        UnifiedMessage {
            role: Role::User,
            content_blocks: vec![ContentBlock::Text {
                text: text.into(),
                span: None,
            }],
            tool_uses: vec![],
            tool_results: vec![],
            metadata: MessageMetadata {
                session_id: "test".into(),
                direction: Direction::Outbound,
                upstream_provider: UpstreamProvider::Anthropic,
                received_at: SystemTime::UNIX_EPOCH,
            },
        }
    }

    #[test]
    fn user_message_with_secret_is_detected() {
        let filter = OutboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let mut msg = user_msg("paste my secret here");
        let hits = filter.process(&mut msg).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-MOCK");
        assert_eq!(hits[0].severity, Severity::Critical);
    }

    #[test]
    fn assistant_message_skipped() {
        let filter = OutboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let mut msg = user_msg("paste my secret here");
        msg.role = Role::Assistant;
        let hits = filter.process(&mut msg).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn sieveignore_filters_out_known_fingerprint() {
        let fp = fingerprint("OUT-MOCK", "secret");
        let mut ignore = HashSet::new();
        ignore.insert(fp);
        let filter = OutboundFilter::new(Arc::new(MockEngine), Arc::new(ignore));
        let mut msg = user_msg("paste my secret here");
        let hits = filter.process(&mut msg).unwrap();
        assert!(hits.is_empty());
    }
}
### crates/sieve-core/src/pipeline/streaming.rs
//! 入站流式 Pipeline 节点 trait（关联 PRD §6.2 Pipeline 节点 ⑦）。
//!
//! Week 3 起 InboundFilter 实现本 trait；sieve-cli 通过 Arc<dyn StreamingPipelineNode>
//! 将其注入到流式代理处理循环中。

use crate::detection::Detection;
use crate::error::SieveCoreResult;
use crate::sse::parser::SseEvent;
use crate::tool_use_aggregator::CompletedToolCall;

/// 入站流式 Pipeline 节点接口。
///
/// 每个 SSE event 到达后调用 [`observe_event`]；
/// Tool Use 聚合完成后调用 [`on_tool_use_complete`]；
/// 流结束时调用 [`on_message_stop`]。
///
/// 所有方法返回 [`Vec<Detection>`]，空列表表示无命中。
pub trait StreamingPipelineNode: Send + Sync {
    /// 节点名（用于审计日志，需稳定不变）。
    fn name(&self) -> &str;

    /// 观察一个 SSE event，返回命中列表。
    ///
    /// 实现者应保持幂等性（同一 event 不应被重复处理）。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn observe_event(&mut self, event: &SseEvent) -> SieveCoreResult<Vec<Detection>>;

    /// 工具调用聚合完成回调，返回命中列表。
    ///
    /// 在 [`crate::tool_use_aggregator::Aggregator::process`] 返回
    /// `Some(CompletedToolCall)` 后由调用方触发。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn on_tool_use_complete(&mut self, tool: &CompletedToolCall)
        -> SieveCoreResult<Vec<Detection>>;

    /// 流结束回调（message_stop event 后调用），返回命中列表。
    ///
    /// 实现者可在此做会话级聚合检测（如 BIP39 助记词拼接检测）。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn on_message_stop(&mut self) -> SieveCoreResult<Vec<Detection>>;
}
### crates/sieve-core/src/pipeline/inbound_hold.rs
//! 入站 GUI 类 hold 流路径（GuiPopup disposition）。
//!
//! 命中 IN-CR-01/05、IN-GEN-04 等 GuiPopup 规则时，hold 住 SSE 流，通过 IpcServer
//! 等待用户在 GUI 做出决策；同时每 25 秒向调用方提供的 channel 发送一条 SSE keep-alive
//! comment（`: keep-alive\n\n`），防止客户端因无数据而超时断开。
//!
//! 关联：ADR-014 §GUI 路径、SPEC-002（keep-alive 规约）、ADR-013（IPC 协议）。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::warn;

use sieve_ipc::{DecisionAction, DecisionRequest, DefaultOnTimeout, IpcServer};

/// Keep-alive 注释间隔（PRD v1.4 §6.7 要求 ≤ 30 s，取 25 s 留余量）。
const KEEP_ALIVE_INTERVAL_SECS: u64 = 25;

/// Keep-alive SSE comment 字节（RFC 8895 §9.2：以 `:` 开头的行是注释，客户端忽略）。
const KEEP_ALIVE_BYTES: &[u8] = b": keep-alive\n\n";

/// Hold 路径专用错误。
#[derive(Debug, Error)]
pub enum HoldError {
    /// IPC 等待决策失败。
    #[error("IPC decision error: {0}")]
    Ipc(#[from] sieve_ipc::IpcError),
}

/// [`hold_and_decide`] 的返回值，表示 hold 结束后的处置动作。
#[derive(Debug, PartialEq, Eq)]
pub enum HoldOutcome {
    /// 用户允许（或超时 default_on_timeout = Allow）→ 继续转发原始 SSE。
    Allow,
    /// 用户允许且要求脱敏（仅出站脱敏类，入站实际等价 Allow）→ 继续转发。
    RedactAndAllow,
    /// 用户拒绝（或超时 default_on_timeout = Block）→ 注入 `sieve_blocked` event 并关流。
    Deny {
        /// 拒绝原因（来自 rule_id 列表或 "timeout"）。
        reason: String,
    },
}

/// Hold 住当前 SSE 流，通过 [`IpcServer`] 等待用户决策，同时发送 keep-alive。
///
/// # 行为
/// 1. 注册 keep-alive task（每 [`KEEP_ALIVE_INTERVAL_SECS`] 秒向 `keep_alive_tx` 发送
///    `: keep-alive\n\n`），daemon 把它写入 SSE 流；
/// 2. 并发等待 `ipc.request_decision(req, timeout)` 返回；
/// 3. 决策返回后停掉 keep-alive task，返回 [`HoldOutcome`]。
///
/// # 超时
/// 超时由 `req.timeout_seconds` 决定（传给 IpcServer）；超时时按 `req.default_on_timeout` 处理：
/// - `Block` → `HoldOutcome::Deny`
/// - `Allow` → `HoldOutcome::Allow`
/// - `Redact` → `HoldOutcome::RedactAndAllow`（入站场景少见，逻辑完整性保留）
///
/// 关联：ADR-014 §GUI 路径、SPEC-002 §keep-alive。
pub async fn hold_and_decide(
    ipc: Arc<IpcServer>,
    req: DecisionRequest,
    keep_alive_tx: mpsc::Sender<Bytes>,
) -> Result<HoldOutcome, HoldError> {
    let timeout_secs = u64::from(req.timeout_seconds).max(1);
    let default_on_timeout = req.default_on_timeout;
    let rule_ids: String = req
        .detections
        .iter()
        .map(|d| d.rule_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // 启动 keep-alive task
    let ka_tx = keep_alive_tx.clone();
    let ka_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL_SECS));
        interval.tick().await; // 第一次 tick 立即返回（elapsed），跳过
        loop {
            interval.tick().await;
            if ka_tx
                .send(Bytes::from_static(KEEP_ALIVE_BYTES))
                .await
                .is_err()
            {
                // 接收端已关闭，停止发送
                break;
            }
        }
    });

    // 等待 IPC 决策
    let timeout = Duration::from_secs(timeout_secs);
    let result = ipc.request_decision(req, timeout).await;

    // 停掉 keep-alive（无论成功失败）
    ka_handle.abort();

    let resp = match result {
        Ok(r) => r,
        Err(e) => {
            warn!("IPC decision error: {e}; falling back to default_on_timeout");
            // IPC 错误按超时兜底
            return Ok(timeout_outcome(default_on_timeout, &rule_ids));
        }
    };

    let outcome = match resp.decision {
        DecisionAction::Allow => HoldOutcome::Allow,
        DecisionAction::RedactAndAllow => HoldOutcome::RedactAndAllow,
        DecisionAction::Deny => HoldOutcome::Deny {
            reason: if resp.by_user {
                format!("用户拒绝（rules: {rule_ids}）")
            } else {
                format!("超时 default-block（rules: {rule_ids}）")
            },
        },
    };

    Ok(outcome)
}

/// 按 [`DefaultOnTimeout`] 构造超时结果。
fn timeout_outcome(dot: DefaultOnTimeout, rule_ids: &str) -> HoldOutcome {
    match dot {
        DefaultOnTimeout::Block => HoldOutcome::Deny {
            reason: format!("超时 fail-closed（rules: {rule_ids}）"),
        },
        DefaultOnTimeout::Allow => HoldOutcome::Allow,
        DefaultOnTimeout::Redact => HoldOutcome::RedactAndAllow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sieve_ipc::protocol::{DecisionResponse, DetectionPayload, Disposition, Severity};
    use uuid::Uuid;

    fn make_request(
        id: Uuid,
        timeout_seconds: u32,
        default_on_timeout: DefaultOnTimeout,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds,
            default_on_timeout,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::GuiPopup,
                title: "地址替换检测".to_owned(),
                one_line_summary: "检测到可疑地址替换".to_owned(),
                details: serde_json::json!({}),
            }],
        }
    }

    fn make_ipc_server() -> (Arc<IpcServer>, tokio::net::UnixListener) {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        // 把 tmp 路径 leak 到测试生命周期（tempfile 会在 drop 时清理，但 socket 不影响测试）
        std::mem::forget(tmp);
        IpcServer::bind(socket_path)
            .map(|(s, l)| (Arc::new(s), l))
            .unwrap()
    }

    // ── Mock IPC 返回 Allow ───────────────────────────────────────────────────

    #[tokio::test]
    async fn ipc_allow_returns_allow_outcome() {
        let (server, listener) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 5, DefaultOnTimeout::Block);

        // 50ms 后注入 Allow 决策
        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            inject_srv
                .inject_decision(DecisionResponse {
                    request_id: id,
                    decision: DecisionAction::Allow,
                    decided_at: Utc::now(),
                    by_user: true,
                    remember: false,
                })
                .await;
        });

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }

    // ── Mock IPC 返回 Deny ────────────────────────────────────────────────────

    #[tokio::test]
    async fn ipc_deny_returns_deny_outcome() {
        let (server, listener) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 5, DefaultOnTimeout::Block);

        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            inject_srv
                .inject_decision(DecisionResponse {
                    request_id: id,
                    decision: DecisionAction::Deny,
                    decided_at: Utc::now(),
                    by_user: true,
                    remember: false,
                })
                .await;
        });

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert!(matches!(outcome, HoldOutcome::Deny { .. }));
    }

    // ── 超时 default_on_timeout = Block ──────────────────────────────────────

    #[tokio::test]
    async fn timeout_with_block_returns_deny() {
        let (server, listener) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        // timeout_seconds = 1 但我们传极短 duration（100ms）给 IpcServer
        // 用 timeout_seconds = 0（最小值 max(0,1) = 1s）——太慢；改用 mock 方式
        // 实际测 timeout：直接用非常短的 timeout_seconds，IpcServer 内部用 Duration
        // 这里 timeout_seconds=1 传给 hold_and_decide → Duration::from_secs(1)；
        // 但测试不等 1s。改法：覆写 Duration 计算，或用 tokio::time::pause/advance。
        // 简化：使用 tokio::time::pause() + advance()
        tokio::time::pause();

        let req = make_request(id, 1, DefaultOnTimeout::Block);
        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
### crates/sieve-core/src/pipeline/inbound_hook.rs
//! 入站 Hook 类路径（HookTerminal disposition）。
//!
//! 命中 IN-CR-02~04、IN-GEN-01~03 等 HookTerminal 规则时，写入 IPC pending 文件，
//! **不修改 SSE 流**——流由调用方（daemon）原样转发给客户端。
//! sieve-hook 二进制会在 PreToolUse 阶段读取 pending 文件并在 TTY 拦截。
//!
//! 关联：ADR-014 §Hook 路径、SPEC-001（pending 文件写入规约）。

use sieve_ipc::{paths::sieve_home, pending_file::write_pending, DecisionRequest};
use thiserror::Error;
use uuid::Uuid;

/// Hook 路径专用错误。
#[derive(Debug, Error)]
pub enum HookError {
    /// IPC 操作失败（目录创建 / 文件写入 / 锁获取）。
    #[error("IPC error: {0}")]
    Ipc(#[from] sieve_ipc::IpcError),
}

/// 写入 IPC pending 文件，通知 sieve-hook 在 PreToolUse 阶段拦截。
///
/// # 行为
/// - 在 `~/.sieve/pending/<request_id>.json`（或 `$SIEVE_HOME`）写入 [`DecisionRequest`]；
/// - **不修改 SSE 流**——调用方负责原样转发；
/// - 返回 `Ok(())` 表示文件已写入，daemon 可继续转发。
///
/// # 错误
/// 目录创建或文件写入失败时返回 [`HookError::Ipc`]。
///
/// 关联：ADR-014 §Hook 路径、SPEC-001 §3.1。
pub fn write_hook_pending(request_id: Uuid, req: &DecisionRequest) -> Result<(), HookError> {
    let _ = request_id; // request_id 已包含在 req.request_id 中，此参数保留供调用侧校验
    let base = sieve_home()?;
    write_pending(req, &base)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sieve_ipc::{
        pending_file::read_pending,
        protocol::{DefaultOnTimeout, DetectionPayload, Disposition, Severity},
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-02".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::HookTerminal,
                title: "危险 shell 命令".to_owned(),
                one_line_summary: "检测到 rm -rf 命令".to_owned(),
                details: serde_json::json!({ "command": "rm -rf /tmp" }),
            }],
        }
    }

    #[test]
    fn write_and_read_pending_file() {
        // 使用独立 tmpdir 直接调用底层 write_pending / read_pending，
        // 避免并发测试互相污染 SIEVE_HOME 全局变量。
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let id = Uuid::now_v7();
        let req = make_request(id);

        // 直接写入指定 base 目录
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();

        // 验证文件内容正确
        let read_back = read_pending(id, base).unwrap();
        assert_eq!(read_back.request_id, id);
        assert_eq!(read_back.detections.len(), 1);
        assert_eq!(read_back.detections[0].rule_id, "IN-CR-02");
    }

    #[test]
    fn write_hook_pending_idempotent_on_same_id() {
        // 使用独立 tmpdir 避免污染 SIEVE_HOME 全局变量（并发测试安全）
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let id = Uuid::now_v7();
        let req = make_request(id);

        // 写两次不应 panic
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();

        // 最终文件可正常读取
        let read_back = read_pending(id, base).unwrap();
        assert_eq!(read_back.request_id, id);
    }
}
### crates/sieve-core/src/pipeline/outbound_redact.rs
//! 出站自动脱敏路径（AutoRedact disposition，OUT-01~05/12）。
//!
//! 把命中范围的字节替换为 `[REDACTED:<rule_id>]` 占位符，重组 HTTP body bytes。
//!
//! 关联：PRD v1.4 §6.1（出站 AutoRedact 路径）、ADR-016（二维处置矩阵）。

/// 单个脱敏命中范围（half-open `[start, end)`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactHit {
    /// 命中规则 ID（如 `OUT-01`）。
    pub rule_id: String,
    /// 命中起始字节偏移（含）。
    pub start: usize,
    /// 命中结束字节偏移（不含）。
    pub end: usize,
}

/// [`redact_body_bytes`] 的返回值。
#[derive(Debug)]
pub struct RedactResult {
    /// 脱敏后的 body bytes。
    pub body: Vec<u8>,
    /// 实际发生脱敏的数量（合并后的 span 数）。
    pub redacted_count: usize,
    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
    pub redacted_summary: String,
}

/// 在 `body` slice 中把 `pos` 向左移动到最近的 UTF-8 字符起始位置。
///
/// UTF-8 continuation byte 以 `10xxxxxx`（`0x80..=0xBF`）开头；
/// 如 body 含非 ASCII 字符（如中文 JSON 字段），正则可能给出 continuation byte 偏移，
/// 此函数保证不截断多字节字符。
pub fn align_to_utf8_char_start(body: &[u8], pos: usize) -> usize {
    if pos >= body.len() {
        return body.len();
    }
    let mut p = pos;
    while p > 0 && (body[p] & 0xC0) == 0x80 {
        p -= 1;
    }
    p
}

/// 把命中范围的字节替换为占位符，返回 [`RedactResult`]。
///
/// # 算法
/// 1. 每个 hit 的 `start`/`end` 先做 UTF-8 字符边界对齐（`align_to_utf8_char_start`）；
/// 2. 按 `start` 升序排序；
/// 3. 合并重叠 / 相邻 span（多个 span 合并时 `rule_id` 取最左命中）；
/// 4. 逐段复制原始字节，用 `[REDACTED:<rule_id>]` 替换各合并 span。
///
/// 如果 `hits` 为空，原样返回 body（`body.to_vec()`，最小拷贝）。
///
/// 关联：ADR-016 §AutoRedact 路径。
pub fn redact_body_bytes(body: &[u8], hits: &[RedactHit]) -> RedactResult {
    if hits.is_empty() {
        return RedactResult {
            body: body.to_vec(),
            redacted_count: 0,
            redacted_summary: String::new(),
        };
    }

    // 1. 对齐 UTF-8 边界
    let mut sorted: Vec<RedactHit> = hits
        .iter()
        .map(|h| RedactHit {
            rule_id: h.rule_id.clone(),
            start: align_to_utf8_char_start(body, h.start.min(body.len())),
            end: align_to_utf8_char_start(body, h.end.min(body.len())),
        })
        .collect();

    // 2. 按 start 升序排序
    sorted.sort_by_key(|h| h.start);

    // 3. 合并重叠 / 相邻 span
    let mut merged: Vec<(usize, usize, String)> = Vec::new();
    for hit in &sorted {
        let start = hit.start;
        let end = hit.end;
        if start >= end {
            // 对齐后 span 变空，跳过
            continue;
        }
        if let Some(last) = merged.last_mut() {
            if start <= last.1 {
                // 重叠或紧邻：扩展结束边界，rule_id 保持第一个
                if end > last.1 {
                    last.1 = end;
                }
            } else {
                merged.push((start, end, hit.rule_id.clone()));
            }
        } else {
            merged.push((start, end, hit.rule_id.clone()));
        }
    }

    let redacted_count = merged.len();
    let redacted_summary = merged
        .iter()
        .map(|(_, _, rule_id)| rule_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // 4. 重组 body
    let mut result: Vec<u8> = Vec::with_capacity(body.len());
    let mut cursor = 0usize;

    for (start, end, rule_id) in &merged {
        if cursor < *start {
            result.extend_from_slice(&body[cursor..*start]);
        }
        let placeholder = format!("[REDACTED:{rule_id}]");
        result.extend_from_slice(placeholder.as_bytes());
        cursor = *end;
    }
    if cursor < body.len() {
        result.extend_from_slice(&body[cursor..]);
    }

    RedactResult {
        body: result,
        redacted_count,
        redacted_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(rule_id: &str, start: usize, end: usize) -> RedactHit {
        RedactHit {
            rule_id: rule_id.to_string(),
            start,
            end,
        }
    }

    // ── 1. 单 span ───────────────────────────────────────────────────────────

    #[test]
    fn single_span_middle() {
        // "hello secret world"
        //  0     6     12   17
        let body = b"hello secret world";
        let hits = [hit("OUT-01", 6, 12)]; // "secret"
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 1);
        assert_eq!(r.redacted_summary, "OUT-01");
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "hello [REDACTED:OUT-01] world");
    }

    // ── 2. 多 span（不重叠）──────────────────────────────────────────────────

    #[test]
    fn multiple_non_overlapping_spans() {
        // "a secret b key c"
        //  0 2      8 10  13 15
        let body = b"a secret b key c";
        let hits = [hit("OUT-01", 2, 8), hit("OUT-03", 11, 14)];
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 2);
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "a [REDACTED:OUT-01] b [REDACTED:OUT-03] c");
    }

    // ── 3. 重叠 span 合并 ────────────────────────────────────────────────────

    #[test]
    fn overlapping_spans_merged() {
        let body = b"0123456789";
        // [1,6) 和 [4,9) 重叠 → 合并为 [1,9)，rule_id 取第一个 OUT-01
        let hits = [hit("OUT-01", 1, 6), hit("OUT-02", 4, 9)];
        let r = redact_body_bytes(body, &hits);
        assert_eq!(
            r.redacted_count, 1,
            "two overlapping spans must merge into one"
        );
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "0[REDACTED:OUT-01]9");
    }

    // ── 4. UTF-8 边界对齐 ────────────────────────────────────────────────────

    #[test]
    fn utf8_boundary_alignment() {
        // "ab中cd"：bytes: [a, b, 中(3 bytes), c, d]
        // 偏移：a=0, b=1, 中[0]=2, 中[1]=3, 中[2]=4, c=5, d=6
        let body = "ab中cd".as_bytes();
        // byte 3 和 4 是 '中' 的 continuation byte，align 应向左到 2
        assert_eq!(align_to_utf8_char_start(body, 3), 2);
        assert_eq!(align_to_utf8_char_start(body, 4), 2);
        // byte 5 是 'c'，本身是起始，不需要移动
        assert_eq!(align_to_utf8_char_start(body, 5), 5);
        // 超出 body 长度时返回 body.len()
        assert_eq!(align_to_utf8_char_start(body, 100), body.len());
    }

    #[test]
    fn utf8_body_redact_aligned() {
        // body: "密钥:sk-xxx" — 确保 hit 落在 UTF-8 continuation byte 时不 panic
        let body = "密钥:sk-xxx".as_bytes();
        // '密' 占 3 字节，start=1 是 continuation byte → 对齐后变 start=0
        // end 也对齐；实际替换从字符边界开始
        let hits = [hit("OUT-01", 1, body.len())];
        let r = redact_body_bytes(body, &hits);
        // 不 panic，体内可正常解析
        assert_eq!(r.redacted_count, 1);
    }

    // ── 5. 空 hits ───────────────────────────────────────────────────────────

    #[test]
    fn empty_hits_returns_original() {
        let body = b"no secrets here";
        let r = redact_body_bytes(body, &[]);
        assert_eq!(r.redacted_count, 0);
        assert_eq!(r.body, body);
        assert!(r.redacted_summary.is_empty());
    }

    // ── 额外：span 超出 body 长度 clamp ──────────────────────────────────────

    #[test]
    fn span_clamped_to_body_len() {
        let body = b"hello";
        let hits = [hit("OUT-01", 3, 100)]; // end 超出 body 长度
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 1);
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "hel[REDACTED:OUT-01]");
    }
}

exec
/bin/zsh -lc 'cargo check --workspace --all-targets' in /Users/doskey/src/sieve
 succeeded in 1260ms:
    Checking sieve-ipc v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-ipc)
    Checking sieve-rules v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-rules)
    Checking sieve-hook v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-hook)
    Checking sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
    Checking sieve-fuzz v0.0.0 (/Users/doskey/src/sieve/fuzz)
    Checking sieve-cli v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s

exec
/bin/zsh -lc 'cargo test --workspace --all-targets' in /Users/doskey/src/sieve
 exited 101 in 788ms:
   Compiling sieve-rules v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-rules)
   Compiling sieve-hook v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-hook)
   Compiling sieve-fuzz v0.0.0 (/Users/doskey/src/sieve/fuzz)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.66s
     Running unittests src/main.rs (target/debug/deps/sieve-eea63f8e71059d6b)

running 18 tests
test config::tests::listen_addr_parses ... ok
test config::tests::defaults_are_sane ... ok
test config::tests::resolved_rules_path_explicit ... ok
test config::tests::resolved_sieveignore_path_explicit ... ok
test config::tests::resolved_rules_path_fallback ... ok
test engine_adapter::tests::map_action_warn_becomes_hook_mark ... ok
test engine_adapter::tests::redact_evidence_long ... ok
test config::tests::unknown_field_rejected ... ok
test engine_adapter::tests::redact_evidence_short ... ok
test config::tests::parse_dry_run_and_rules_path ... ok
test config::tests::parse_minimal_toml ... ok
test config::tests::parse_full_toml ... ok
test engine_adapter::tests::scan_no_match_returns_empty ... ok
test audit::tests::update_trigger_blocks ... ok
test audit::tests::decision_event_stores_decision_field ... ok
test audit::tests::write_and_read_events ... ok
test engine_adapter::tests::span_offset_applied ... ok
test engine_adapter::tests::scan_detects_pattern ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audit_append_only.rs (target/debug/deps/audit_append_only-7963e579460992e0)

running 3 tests
test delete_is_rejected_by_trigger ... ok
test update_is_rejected_by_trigger ... ok
test write_3_events_and_read_back ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/inbound_block.rs (target/debug/deps/inbound_block-ebd9dd42f9aab019)

running 10 tests
test address_substitution_from_prompt_seed_blocks ... FAILED
test malformed_tool_use_partial_json_blocks ... FAILED
test ucsb_attack_1_address_substitution_blocked ... FAILED
test ucsb_attack_4_markdown_exfil_warn_only_passes_through ... FAILED
test unterminated_final_event_still_blocks_critical ... FAILED
test ucsb_attack_3_signing_tool_blocked ... FAILED
test benign_response_passes_through_unchanged ... FAILED
test in_cr_04_persistence_shell_rc_blocked ... FAILED
test ucsb_attack_2_dangerous_shell_in_tool_use_blocked ... FAILED
test in_cr_03_sensitive_path_warn_passes_through ... FAILED

failures:

---- address_substitution_from_prompt_seed_blocks stdout ----

thread 'address_substitution_from_prompt_seed_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- malformed_tool_use_partial_json_blocks stdout ----

thread 'malformed_tool_use_partial_json_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_1_address_substitution_blocked stdout ----

thread 'ucsb_attack_1_address_substitution_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_4_markdown_exfil_warn_only_passes_through stdout ----

thread 'ucsb_attack_4_markdown_exfil_warn_only_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- unterminated_final_event_still_blocks_critical stdout ----

thread 'unterminated_final_event_still_blocks_critical' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- ucsb_attack_3_signing_tool_blocked stdout ----

thread 'ucsb_attack_3_signing_tool_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- benign_response_passes_through_unchanged stdout ----

thread 'benign_response_passes_through_unchanged' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_04_persistence_shell_rc_blocked stdout ----

thread 'in_cr_04_persistence_shell_rc_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_2_dangerous_shell_in_tool_use_blocked stdout ----

thread 'ucsb_attack_2_dangerous_shell_in_tool_use_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
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
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/inbound_block.rs | sed -n '1,130p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
     1	//! Sieve daemon 入站拦截集成测试（UCSB 4 类攻击 PoC，关联 PRD §10.1 Week 3 完成定义）。
     2	//!
     3	//! 启动真实 sieve 二进制 + mock 上游（返回带攻击 payload 的 SSE 流）+ 客户端发请求，
     4	//! 验证：
     5	//! 1. IN-CR-01 地址替换 — 同一会话内文本含原地址 + 一字符不同的地址 → 截流
     6	//! 2. IN-CR-02 危险 shell 命令 — tool_use input 含 `rm -rf /` → 截流
     7	//! 3. IN-CR-05 签名工具 — tool_use 名为 `eth_signTransaction` → 截流
     8	//! 4. IN-GEN-04 markdown exfil — text_delta 含 markdown image with query string → warn 不阻断
     9	//!    （Week 4 由旧 IN-CR-04 重命名归入 IN-GEN-* 命名空间）
    10	//!
    11	//! 入站截流场景：sieve 注入 sieve_blocked event 后 drop tx，hyper StreamBody 结束；
    12	//! 若上游响应带 content-length，sieve 透传该 header 后注入额外字节导致 HTTP 长度不一致。
    13	//! 因此 mock upstream 使用 StreamBody（无 content-length），迫使 hyper 用 chunked transfer。
    14	//!
    15	//! .cursorrules §3.2：测试代码允许使用 .unwrap()。
    16	
    17	use bytes::Bytes;
    18	use http_body_util::{BodyExt, StreamBody};
    19	use hyper::body::{Frame, Incoming};
    20	use hyper::server::conn::http1 as server_http1;
    21	use hyper::service::service_fn;
    22	use hyper::{Request, Response};
    23	use hyper_util::rt::TokioIo;
    24	use std::convert::Infallible;
    25	use std::io::Write as _;
    26	use std::net::{SocketAddr, TcpListener as StdListener};
    27	use std::path::PathBuf;
    28	use std::process::{Child, Command, Stdio};
    29	use std::time::{Duration, Instant};
    30	use tokio::net::TcpListener;
    31	use tokio::sync::oneshot;
    32	
    33	// ─── helpers ──────────────────────────────────────────────────────────────────
    34	
    35	fn find_free_port() -> u16 {
    36	    let l = StdListener::bind("127.0.0.1:0").unwrap();
    37	    l.local_addr().unwrap().port()
    38	}
    39	
    40	fn workspace_root() -> PathBuf {
    41	    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    42	    p.pop(); // sieve-cli → crates/
    43	    p.pop(); // crates/ → workspace root
    44	    p
    45	}
    46	
    47	fn sieve_binary() -> PathBuf {
    48	    let root = workspace_root();
    49	    let release = root.join("target/release/sieve");
    50	    if release.exists() {
    51	        return release;
    52	    }
    53	    root.join("target/debug/sieve")
    54	}
    55	
    56	fn outbound_rules_path() -> PathBuf {
    57	    workspace_root().join("crates/sieve-rules/rules/outbound.toml")
    58	}
    59	
    60	fn inbound_rules_path() -> PathBuf {
    61	    workspace_root().join("crates/sieve-rules/rules/inbound.toml")
    62	}
    63	
    64	/// 把 (event_name, data) 列表序列化为 SSE bytes。
    65	fn sse_response(events: &[(&str, &str)]) -> Bytes {
    66	    let mut s = String::new();
    67	    for (event_name, data) in events {
    68	        s.push_str(&format!("event: {event_name}\ndata: {data}\n\n"));
    69	    }
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
   106	                        let svc = service_fn(move |req: Request<Incoming>| {
   107	                            let r = r.clone();
   108	                            async move {
   109	                                let (parts, body) = req.into_parts();
   110	                                let bytes = body
   111	                                    .collect()
   112	                                    .await
   113	                                    .unwrap_or_default()
   114	                                    .to_bytes();
   115	                                let req_collected = Request::from_parts(parts, bytes);
   116	                                let (status, body_bytes) = r(req_collected).await;
   117	                                // 用 StreamBody（无 content-length），让 sieve 也不透传 content-length
   118	                                let resp: Response<MockBody> = Response::builder()
   119	                                    .status(status)
   120	                                    .header(http::header::CONTENT_TYPE, "text/event-stream")
   121	                                    .body(bytes_to_chunked_body(body_bytes))
   122	                                    .unwrap();
   123	                                Ok::<_, Infallible>(resp)
   124	                            }
   125	                        });
   126	                        let _ = server_http1::Builder::new()
   127	                            .serve_connection(io, svc)
   128	                            .await;
   129	                    });
   130	                }

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-cli/src/config.rs && sed -n '260,560p' crates/sieve-cli/src/config.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 配置加载（关联 docs/design/data-model.md §配置）。
//!
//! Phase 1 字段：`upstream_url` / `port` / `bind_addr` / `log_path` /
//! `tls_verify_upstream`。
//! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
//! Week 3 新增：`inbound_rules_path`（入站规则路径）。
//! Week 5 新增：`ipc_socket_path` / `pending_dir` / `decisions_dir` /
//!              `preset` / `launchd_plist_path` / `gui_socket_enabled` /
//!              `audit_db_path`（SPEC-003 / data-model.md §5）。
//! `#[serde(deny_unknown_fields)]` 确保配置文件中的危险字段（如
//! `disable_critical`）被强制拒绝，不会静默忽略。

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 检测预设级别（SPEC-003 / data-model.md §5）。
///
/// - `Strict`：所有规则最高灵敏度
/// - `Default`：推荐平衡配置（默认）
/// - `Relaxed`：降低误报，适合受信任环境
/// - `Custom`：完全自定义（忽略内置默认值）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Strict,
    #[default]
    Default,
    Relaxed,
    Custom,
}

/// Sieve 顶层配置。
///
/// 对应 `sieve.toml`（ADR-003 / data-model.md §配置）。
/// 文件不存在时 [`Config::load`] 返回 [`Config::default`]。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 上游 LLM API 端点（默认 `https://api.anthropic.com`）。
    #[serde(default = "default_upstream")]
    pub upstream_url: String,

    /// 本地代理监听端口（默认 11453，PRD §6.1）。
    #[serde(default = "default_port")]
    pub port: u16,

    /// 监听地址。**强制 `127.0.0.1`**（ADR-003 / PRD §9 #2 完全本地）。
    /// 任何其他值都会触发 [`Config::enforce_safety_invariants`] 中的 exit(1)。
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    /// 审计日志路径（SQLite），`None` 时由 daemon 决定默认路径。
    #[serde(default)]
    pub log_path: Option<PathBuf>,

    /// 是否校验上游 TLS 证书（默认 `true`；测试可关，会打印 WARN）。
    #[serde(default = "default_tls_verify")]
    pub tls_verify_upstream: bool,

    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
    #[serde(default)]
    pub rules_path: Option<PathBuf>,

    /// `.sieveignore` 路径（默认 `~/.sieve/sieveignore`）。
    #[serde(default)]
    pub sieveignore_path: Option<PathBuf>,

    /// 仅记录命中，不实际拦截（dry-run 模式，默认 `false`）。
    ///
    /// `true` 时 [`Config::enforce_safety_invariants`] 会打印 WARN。
    /// CLI `--dry-run` flag 出现时会覆盖此值为 `true`（见 cli.rs）。
    #[serde(default)]
    pub dry_run: bool,

    /// 入站规则 toml 路径（Week 3，默认 `crates/sieve-rules/rules/inbound.toml`）。
    #[serde(default)]
    pub inbound_rules_path: Option<PathBuf>,

    // ── Week 5 新字段（SPEC-003 / data-model.md §5）────────────────────────
    // Week 6+ 会在 daemon 启动时读取这些字段；当前仅反序列化使用，暂时 allow dead_code。
    /// Unix socket 路径（GUI / sieve-hook 连接用，默认 `~/.sieve/ipc.sock`）。
    #[serde(default = "default_ipc_socket")]
    #[allow(dead_code)]
    pub ipc_socket_path: PathBuf,

    /// 待决策文件目录（默认 `~/.sieve/pending/`）。
    #[serde(default = "default_pending_dir")]
    #[allow(dead_code)]
    pub pending_dir: PathBuf,

    /// 决策文件目录（默认 `~/.sieve/decisions/`）。
    #[serde(default = "default_decisions_dir")]
    #[allow(dead_code)]
    pub decisions_dir: PathBuf,

    /// 检测预设级别（默认 `Default`）。
    #[serde(default)]
    #[allow(dead_code)]
    pub preset: Preset,

    /// launchd plist 路径（macOS，默认 `~/Library/LaunchAgents/com.sieve.daemon.plist`）。
    #[serde(default = "default_launchd_plist")]
    #[allow(dead_code)]
    pub launchd_plist_path: PathBuf,

    /// 是否启用 GUI Unix socket（默认 `false`；Week 6+ 启用）。
    #[serde(default = "default_gui_socket_enabled")]
    #[allow(dead_code)]
    pub gui_socket_enabled: bool,

    /// SQLite 审计数据库路径（Week 5；`None` 时沿用 `log_path` 或 `~/.sieve/audit.db`）。
    #[serde(default)]
    #[allow(dead_code)]
    pub audit_db_path: Option<PathBuf>,
}

fn home_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn sieve_home() -> PathBuf {
    home_path().join(".sieve")
}

fn default_ipc_socket() -> PathBuf {
    sieve_home().join("ipc.sock")
}

fn default_pending_dir() -> PathBuf {
    sieve_home().join("pending")
}

fn default_decisions_dir() -> PathBuf {
    sieve_home().join("decisions")
}

fn default_launchd_plist() -> PathBuf {
    home_path()
        .join("Library")
        .join("LaunchAgents")
        .join("com.sieve.daemon.plist")
}

fn default_gui_socket_enabled() -> bool {
    false
}

fn default_upstream() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_port() -> u16 {
    11453
}

fn default_bind_addr() -> String {
    "127.0.0.1".to_string()
}

fn default_tls_verify() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            upstream_url: default_upstream(),
            port: default_port(),
            bind_addr: default_bind_addr(),
            log_path: None,
            tls_verify_upstream: default_tls_verify(),
            rules_path: None,
            sieveignore_path: None,
            dry_run: false,
            inbound_rules_path: None,
            ipc_socket_path: default_ipc_socket(),
            pending_dir: default_pending_dir(),
            decisions_dir: default_decisions_dir(),
            preset: Preset::default(),
            launchd_plist_path: default_launchd_plist(),
            gui_socket_enabled: default_gui_socket_enabled(),
            audit_db_path: None,
        }
    }
}

impl Config {
    /// 从 TOML 文件加载配置；文件不存在时返回 [`Config::default`]。
    ///
    /// # Errors
    /// 文件存在但读取或解析失败时返回错误。
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::warn!(path = %path.display(), "config file not found, using defaults");
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let cfg: Self =
            toml::from_str(&s).with_context(|| format!("parse config {}", path.display()))?;
        Ok(cfg)
    }

    /// 强制安全不变量：`bind_addr` 必须是 `127.0.0.1`，否则打印 FATAL 并 `exit(1)`。
    ///
    /// 关联 ADR-003 / PRD §9 #2 / data-model.md §配置。
    /// 不提供 fallback，不 warn 后继续：非 loopback 绑定是配置错误，
    /// 悄悄启动会暴露代理到局域网，违反"完全本地"承诺。
    pub fn enforce_safety_invariants(&self) {
        if self.bind_addr != "127.0.0.1" {
            eprintln!(
                "FATAL: bind_addr must be 127.0.0.1 (got {:?}). \
                 Sieve refuses to bind on a non-loopback address. See ADR-003.",
                self.bind_addr
            );
            std::process::exit(1);
        }

        if !self.tls_verify_upstream {
            tracing::warn!(
                "tls_verify_upstream=false: upstream TLS certificate NOT verified. \
                 Only use in controlled test environments."
            );
        }

        if self.dry_run {
            tracing::warn!("dry_run mode: detections logged but not blocked");
        }
    }

    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
    pub fn resolved_rules_path(&self) -> PathBuf {
        if let Some(p) = &self.rules_path {
            return p.clone();
        }
        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
    }

    /// 解析入站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/inbound.toml`（相对 cwd）。
    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
        if let Some(p) = &self.inbound_rules_path {
            return p.clone();
        }
        PathBuf::from("crates/sieve-rules/rules/inbound.toml")
    }

    /// 解析 `.sieveignore` 路径。显式给定时直接用，否则回退到 `~/.sieve/sieveignore`。
    ///
    /// 若 `HOME` 不可读则 fallback 到 `.sieve/sieveignore`（相对 cwd）并打印 WARN。
    pub fn resolved_sieveignore_path(&self) -> PathBuf {
        if let Some(p) = &self.sieveignore_path {
            return p.clone();
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".sieve").join("sieveignore");
        }
        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");
        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");
        PathBuf::from(".sieve").join("sieveignore")
    }

    /// 拼接监听 SocketAddr。
    ///
    /// # Errors
    /// `bind_addr` 或 `port` 无法解析为合法 SocketAddr 时返回错误。
    pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
        format!("{}:{}", self.bind_addr, self.port)
            .parse()
            .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
    }

    /// 解析审计日志路径。`log_path` 显式给定时直接用,否则回退到 `~/.sieve/audit.db`。
    ///
    /// # Errors
    /// `$HOME` 不存在或不可识别时返回错误。
    pub fn audit_db_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.log_path {
            return Ok(p.clone());
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| anyhow!("HOME env var not set; specify log_path explicitly"))?;
        Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.bind_addr, "127.0.0.1");
        assert_eq!(c.port, 11453);
        assert_eq!(c.upstream_url, "https://api.anthropic.com");
        assert!(c.tls_verify_upstream);
        assert!(c.log_path.is_none());
    }

    #[test]
    fn listen_addr_parses() {
        let c = Config::default();
        let addr = c.listen_addr().unwrap();
        assert_eq!(addr.port(), 11453);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn parse_minimal_toml() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(c.bind_addr, "127.0.0.1");
        assert!(c.tls_verify_upstream);
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 12000
            bind_addr = "127.0.0.1"
            tls_verify_upstream = false
            log_path = "/tmp/audit.db"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(c.port, 12000);
        assert!(!c.tls_verify_upstream);
        assert_eq!(c.log_path.unwrap().to_str().unwrap(), "/tmp/audit.db");
    }

    #[test]
    fn unknown_field_rejected() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            disable_critical = true
        "#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "must reject unknown fields (deny_unknown_fields)"
        );
    }

    #[test]
    fn parse_dry_run_and_rules_path() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
            dry_run = true
            rules_path = "/x.toml"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert!(c.dry_run, "dry_run should be true");
        assert_eq!(c.rules_path.as_ref().unwrap().to_str().unwrap(), "/x.toml");
    }

    #[test]
    fn resolved_rules_path_explicit() {
        let c = Config {
            rules_path: Some(PathBuf::from("/custom/rules.toml")),
            ..Config::default()
        };
        assert_eq!(c.resolved_rules_path(), PathBuf::from("/custom/rules.toml"));
    }

    #[test]
    fn resolved_rules_path_fallback() {
        let c = Config::default();
        let p = c.resolved_rules_path();
        assert!(
            p.ends_with("outbound.toml"),
            "fallback should end with outbound.toml, got {:?}",
            p
        );
    }

    #[test]
    fn resolved_sieveignore_path_explicit() {
        let c = Config {
            sieveignore_path: Some(PathBuf::from("/my/.sieveignore")),
            ..Config::default()
        };
        assert_eq!(
            c.resolved_sieveignore_path(),
            PathBuf::from("/my/.sieveignore")
        );
    }
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-cli/src/daemon.rs && sed -n '260,620p' crates/sieve-cli/src/daemon.rs && sed -n '620,980p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 透传 daemon（架构图节点 ①③⑤⑧）。
//!
//! Week 2：POST /v1/messages body 收集 → 出站规则扫描 → Critical 命中时返回 426；
//! 非 messages 路径 / 解析失败 / 无命中 → 流式透传（Week 1 行为保持不变）。
//!
//! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
//!
//! Week 4（v1.4）：
//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
//!
//! 关联：PRD v1.4 §6.1 §6.7 / ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use futures_util::StreamExt as _;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use sieve_core::detection::Action;
use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_core::pipeline::outbound_redact::{redact_body_bytes, RedactHit};
use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
use sieve_core::sse::parser::SseParser;
use sieve_core::tool_use_aggregator::Aggregator;
use sieve_core::Forwarder;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::Config;

/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

/// 启动 daemon，永久阻塞直到进程收到信号。
///
/// `filter` 是出站规则引擎包装；`inbound_engine` + `inbound_sieveignore` 用于每连接构造
/// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
/// `cfg.dry_run` 决定是否实际拦截。
///
/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
///
/// # Errors
/// bind 端口失败或 Forwarder 初始化失败时返回错误。
pub async fn run(
    cfg: Config,
    filter: Arc<OutboundFilter>,
    inbound_engine: Arc<dyn InboundEngine>,
    inbound_sieveignore: Arc<HashSet<String>>,
) -> Result<()> {
    let listen = cfg.listen_addr()?;
    let dry_run = cfg.dry_run;
    let forwarder =
        Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);

    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
        Ok(home) => {
            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
                Ok((server, listener)) => {
                    let server = Arc::new(server);
                    let srv_clone = Arc::clone(&server);
                    tokio::spawn(async move {
                        srv_clone.run(listener).await;
                    });
                    tracing::info!(socket = %socket_path.display(), "IPC server started");
                    Some(server)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
            None
        }
    };

    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind {}", listen))?;

    tracing::info!(
        listen = %listen,
        upstream = %cfg.upstream_url,
        dry_run = dry_run,
        "sieve daemon started"
    );

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "accept failed");
                continue;
            }
        };

        let forwarder = forwarder.clone();
        let filter = filter.clone();
        let inbound_engine = inbound_engine.clone();
        let inbound_sieveignore = inbound_sieveignore.clone();
        let ipc_server = ipc_server.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                let flt = filter.clone();
                // 每连接独立 InboundFilter（&mut self trait 要求）
                let ib_filter =
                    InboundFilter::new(inbound_engine.clone(), inbound_sieveignore.clone());
                let ipc = ipc_server.clone();
                async move { proxy(f, flt, ib_filter, dry_run, ipc, req).await }
            });

            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                tracing::debug!(peer = %peer, error = %e, "connection closed with error");
            }
        });
    }
}

/// 请求入口：捕获 `proxy_inner` 的所有错误，转换为 502 Bad Gateway 响应。
async fn proxy(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(forwarder, filter, inbound_filter, dry_run, ipc, req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "proxy failed");
            let body = format!("sieve proxy error: {e}");
            let resp = Response::builder()
                .status(http::StatusCode::BAD_GATEWAY)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(string_body(body))
                .unwrap_or_else(|_| Response::new(empty_body()));
            Ok(resp)
        }
    }
}

/// 核心代理逻辑。
///
/// - POST /v1/messages → collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测
/// - 其他路径 → 流式透传（Week 1 行为）
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>> {
    let (parts, body) = req.into_parts();
    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    let is_messages_post = method == http::Method::POST && path == "/v1/messages";

    if is_messages_post {
        // 1. collect 完整 body（出站扫描需要全文）
        let collected = body
            .collect()
            .await
            .map_err(|e| anyhow!("collect body: {e}"))?;
        let body_bytes = collected.to_bytes();

        // 2. 解析 AnthropicRequest；解析失败则直接透传（上游会返回 400）
        let anthropic_req: sieve_core::protocol::anthropic::AnthropicRequest =
            match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("non-anthropic body, passing through: {e}");
                    return forward_raw(forwarder, parts, body_bytes).await;
                }
            };

        // 3. 提取文本段 → 逐段扫描
        let texts = anthropic_req.extract_text_content();
        let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

        for (offset, text) in &texts {
            use sieve_core::pipeline::PipelineNode;
            use sieve_core::protocol::unified_message::{
                ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
            };
            use std::time::SystemTime;

            let mut msg = sieve_core::UnifiedMessage {
                role: sieve_core::Role::User,
                content_blocks: vec![ContentBlock::Text {
                    text: text.clone(),
                    span: Some(ContentSpan {
                        start: *offset,
                        end: *offset + text.len(),
                    }),
                }],
                tool_uses: vec![],
                tool_results: vec![],
                metadata: MessageMetadata {
                    session_id: "outbound-scan".into(),
                    direction: Direction::Outbound,
                    upstream_provider: UpstreamProvider::Anthropic,
                    received_at: SystemTime::now(),
                },
            };

            let hits = filter
                .process(&mut msg)
                .map_err(|e| anyhow!("outbound filter: {e}"))?;
            all_detections.extend(hits);
        }

        // 4. 决策：
        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
        //    b. fail-closed Critical Block → 426（PRD §9 #3）
        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
        //    d. 其余 → 透传

        // 4a. 收集需要脱敏的 hit
        let redact_hits: Vec<RedactHit> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::Redact { .. }))
            .map(|d| RedactHit {
                rule_id: d.rule_id.clone(),
                start: d.span.start,
                end: d.span.end,
            })
            .collect();

        // 4b/c. 收集需要 Block 的 detection
        let blocking: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| {
                if d.action != Action::Block {
                    return false;
                }
                }
                if d.severity != sieve_core::Severity::Critical {
                    return false;
                }
                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
            })
            .collect();

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection");
            }
            let cloned: Vec<sieve_core::Detection> =
                blocking.iter().map(|d| (*d).clone()).collect();
            return Ok(build_426_response(&cloned));
        }

        // 4a. AutoRedact：脱敏 body bytes 后转发（不返回 426）
        if !redact_hits.is_empty() {
            tracing::info!(
                count = redact_hits.len(),
                rules = redact_hits
                    .iter()
                    .map(|h| h.rule_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                "OUTBOUND AUTO-REDACT"
            );
            let result = redact_body_bytes(&body_bytes, &redact_hits);
            let new_body = Bytes::from(result.body);
            let new_len = new_body.len();

            // 更新 Content-Length header
            let mut new_parts = parts.clone();
            new_parts.headers.insert(
                http::header::CONTENT_LENGTH,
                http::HeaderValue::from(new_len),
            );

            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
            for (_, text) in &texts {
                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
                }
            }

            return forward_with_inbound_inspection(
                forwarder,
                inbound_filter,
                dry_run,
                ipc,
                new_parts,
                new_body,
            )
            .await;
        }

        if dry_run && !all_detections.is_empty() {
            tracing::warn!(
                count = all_detections.len(),
                "OUTBOUND DRY-RUN: would have flagged"
            );
            for d in &all_detections {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection (dry_run)");
            }
        }

        // 5. prompt 地址 seed
        for (_, text) in &texts {
            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
            }
        }

        // 6. 出站通过 → 入站 SSE tee 截流检测
        return forward_with_inbound_inspection(
            forwarder,
            inbound_filter,
            dry_run,
            ipc,
            parts,
            body_bytes,
        )
        .await;
    }

    // 非 messages 路径：Week 1 流式透传
    forward_streaming(forwarder, parts, body).await
}

/// 透传并同步做入站 SSE 解析检测（tee 模式）。
///
/// 字节流同时被：
/// 1. 原样 forward 给客户端（via bounded channel）
/// 2. 异步喂给 SseParser → Aggregator → InboundFilter 检测
///
/// v1.4 分支逻辑：
/// - `Action::Block`（fail-closed Critical）→ 注入 `sieve_blocked` event 并截流
/// - `Action::HookMark` → 写 IPC pending 文件，SSE 流原样转发（**不注入 sieve_blocked**）
/// - `Action::HoldForDecision` → hold 流 + keep-alive，等用户决策
/// - 其余 → 透传
///
/// 关联：ADR-014 §双层防御、ADR-016 §dispatch 路由、PRD v1.4 §6.7。
async fn forward_with_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (mut resp_parts, resp_body) = upstream_resp.into_parts();

    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    tokio::spawn(async move {
        let mut parser = SseParser::new();
        let mut aggregator = Aggregator::new();

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        while let Some(frame_result) = stream.next().await {
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
                    let events = match parser.push_chunk(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // 收集本批 events 的 detections，按 action 分组处理
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                    );

                    // 1. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
                    for d in &hook_detections {
                        write_hook_pending_silent(d);
                    }

                    // 2. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            // keep-alive channel：daemon 把心跳写入 SSE 流
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
                            // 然后再 hold——这样客户端已经看到触发 event，
                            // hold 期间只收到 keep-alive comment。
                            if tx
                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
                                .await
                                .is_err()
                            {
                                return;
                            }

                            // 启动 keep-alive 转发 task
                            let ka_fwd_handle = tokio::spawn(async move {
                                while let Some(ka_bytes) = ka_rx.recv().await {
                                    if tx_ka
                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });

                            // 构造 IPC 请求
                            use chrono::Utc;
                            let request_id = uuid::Uuid::new_v4();
                            let timeout_seconds = hold_detections
                                .iter()
                                .find_map(|d| {
                                    if let Action::HoldForDecision {
                                        timeout_seconds, ..
                                    } = d.action
                                    {
                                        Some(timeout_seconds)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(60);

                            let ipc_detections = hold_detections
                                .iter()
                                .map(|d| sieve_ipc::protocol::DetectionPayload {
                                    rule_id: d.rule_id.clone(),
                                    severity: map_severity_to_ipc(d.severity),
                                    disposition: sieve_ipc::Disposition::GuiPopup,
                                    title: format!("检测命中：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                })
                                .collect();

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
                                    // 允许：继续转发后续 SSE 帧
                                    // 当前帧已在 hold 前发出，继续循环
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            // IPC 未初始化：fail-closed，阻断
                            tracing::warn!(
                                "GuiPopup detection but IPC server not initialized; fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 3. Block 类：注入 sieve_blocked 并截流
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 无 blocking / hold：透传原始 frame
                    if tx
                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "upstream body error: {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
        let flushed = parser.flush();
        let (blocking, hook_detections, _hold_detections) =
            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);

        for d in &hook_detections {
            write_hook_pending_silent(d);
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
            }
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
/// - `blocking`：`Action::Block` 需立即截流的 detections
/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
///
/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
///
/// 关联 ADR-016 §dispatch 路由、ADR-014 §双层防御。
fn classify_inbound_detections(
    events: &[sieve_core::sse::parser::SseEvent],
    inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
    aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
    dry_run: bool,
) -> (
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
) {
    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();

    for evt in events {
        match inbound_filter.observe_event(evt) {
            Ok(hits) => all_hits.extend(hits),
            Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
        }
        match aggregator.process(evt) {
            Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
                Ok(hits) => all_hits.extend(hits),
                Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
            },
            Ok(None) => {}
            Err(sieve_core::tool_use_aggregator::AggregatorError::MalformedToolUse {
                ref tool_id,
                ref error,
            }) => {
                tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
                all_hits.push(build_malformed_tool_use_detection(tool_id));
            }
            Err(e) => {
                tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
            }
        }
    }

    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();

    for d in all_hits {
        match &d.action {
            Action::Block => {
                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
                if d.severity == sieve_core::Severity::Critical
                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
                {
                    blocking.push(d);
                }
                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
            }
            Action::HookMark => {
                // Hook 类：写 pending 文件，SSE 流继续转发
                hook_detections.push(d);
            }
            Action::HoldForDecision { .. } => {
                // GUI 类：hold 流等决策
                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
                hold_detections.push(d);
            }
            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
            }
        }
    }

    (blocking, hook_detections, hold_detections)
}

/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
///
/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
fn write_hook_pending_silent(d: &sieve_core::Detection) {
    use chrono::Utc;

    let sieve_home = match sieve_ipc::paths::sieve_home() {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
            return;
        }
    };

    let request_id = uuid::Uuid::new_v4();
    let ipc_req = sieve_ipc::DecisionRequest {
        request_id,
        created_at: Utc::now(),
        timeout_seconds: 60,
        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
        detections: vec![sieve_ipc::protocol::DetectionPayload {
            rule_id: d.rule_id.clone(),
            severity: map_severity_to_ipc(d.severity),
            disposition: sieve_ipc::Disposition::HookTerminal,
            title: format!("检测命中：{}", d.rule_id),
            one_line_summary: d.evidence_truncated.clone(),
            details: serde_json::json!({}),
        }],
    };

    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
    } else {
        tracing::info!(
            rule = %d.rule_id,
            request_id = %request_id,
            "HookMark: pending file written, SSE stream continues"
        );
    }
}

/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
    match s {
        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
        sieve_core::Severity::High => sieve_ipc::Severity::High,
        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
    }
}

/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
    let payload = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "detections": detections.iter().map(|d| serde_json::json!({
            "rule_id": d.rule_id,
            "severity": d.severity,
            "fingerprint": d.fingerprint,
        })).collect::<Vec<_>>(),
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
}

/// 用已收集的 body bytes 重新构造请求并转发。
async fn forward_raw(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 流式透传（Week 1 路径），不缓冲 body。
async fn forward_streaming(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body: Incoming,
) -> Result<Response<ResponseBody>> {
    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = body.map_err(|e| -> hyper::Error { e }).boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
    let blocked_at = epoch_secs_string();
    let detections_json: Vec<serde_json::Value> = detections
        .iter()
        .map(|d| {
            serde_json::json!({
                "rule_id": d.rule_id,
                "severity": d.severity,
                "fingerprint": d.fingerprint,
            })
        })
        .collect();
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": blocked_at,
        "detections": detections_json,
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条出站 Critical 命中。请检查后用 .sieveignore 加入 fingerprint 白名单，或重新发送脱敏消息。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} outbound critical detection(s). Review your message, then either redact or add fingerprint(s) to .sieveignore.",
                detections.len()
            ),
        }
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::UPGRADE_REQUIRED) // 426
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 返回 UNIX epoch 秒字符串（Phase 1 简化，Week 4 改 RFC3339）。
fn epoch_secs_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

/// 把字节包成 `ResponseBody`。
fn bytes_body(b: Bytes) -> ResponseBody {
    use http_body_util::Full;
    Full::new(b)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 把字符串包成 `ResponseBody`（用于错误响应）。
fn string_body(s: String) -> ResponseBody {
    bytes_body(Bytes::from(s))
}

/// 空 body（fallback 错误响应）。
fn empty_body() -> ResponseBody {
    use http_body_util::Empty;
    Empty::<Bytes>::new()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: "IN-CR-05-MALFORMED".into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: format!("tool_id={tool_id}"),
        fingerprint: "malformed-tool-use-partial-json".into(),
    }
}

/// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: rule_id.into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: String::new(),
        fingerprint: fingerprint_key.into(),
    }
}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '180,250p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
   243	        // 4a. 收集需要脱敏的 hit
   244	        let redact_hits: Vec<RedactHit> = all_detections
   245	            .iter()
   246	            .filter(|d| matches!(d.action, Action::Redact { .. }))
   247	            .map(|d| RedactHit {
   248	                rule_id: d.rule_id.clone(),
   249	                start: d.span.start,
   250	                end: d.span.end,

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '250,340p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   250	                end: d.span.end,
   251	            })
   252	            .collect();
   253	
   254	        // 4b/c. 收集需要 Block 的 detection
   255	        let blocking: Vec<&sieve_core::Detection> = all_detections
   256	            .iter()
   257	            .filter(|d| {
   258	                if d.action != Action::Block {
   259	                    return false;
   260	                }
   261	                if d.severity != sieve_core::Severity::Critical {
   262	                    return false;
   263	                }
   264	                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
   265	            })
   266	            .collect();
   267	
   268	        if !blocking.is_empty() {
   269	            tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED");
   270	            for d in &blocking {
   271	                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection");
   272	            }
   273	            let cloned: Vec<sieve_core::Detection> =
   274	                blocking.iter().map(|d| (*d).clone()).collect();
   275	            return Ok(build_426_response(&cloned));
   276	        }
   277	
   278	        // 4a. AutoRedact：脱敏 body bytes 后转发（不返回 426）
   279	        if !redact_hits.is_empty() {
   280	            tracing::info!(
   281	                count = redact_hits.len(),
   282	                rules = redact_hits
   283	                    .iter()
   284	                    .map(|h| h.rule_id.as_str())
   285	                    .collect::<Vec<_>>()
   286	                    .join(", "),
   287	                "OUTBOUND AUTO-REDACT"
   288	            );
   289	            let result = redact_body_bytes(&body_bytes, &redact_hits);
   290	            let new_body = Bytes::from(result.body);
   291	            let new_len = new_body.len();
   292	
   293	            // 更新 Content-Length header
   294	            let mut new_parts = parts.clone();
   295	            new_parts.headers.insert(
   296	                http::header::CONTENT_LENGTH,
   297	                http::HeaderValue::from(new_len),
   298	            );
   299	
   300	            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
   301	            for (_, text) in &texts {
   302	                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
   303	                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
   304	                }
   305	            }
   306	
   307	            return forward_with_inbound_inspection(
   308	                forwarder,
   309	                inbound_filter,
   310	                dry_run,
   311	                ipc,
   312	                new_parts,
   313	                new_body,
   314	            )
   315	            .await;
   316	        }
   317	
   318	        if dry_run && !all_detections.is_empty() {
   319	            tracing::warn!(
   320	                count = all_detections.len(),
   321	                "OUTBOUND DRY-RUN: would have flagged"
   322	            );
   323	            for d in &all_detections {
   324	                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection (dry_run)");
   325	            }
   326	        }
   327	
   328	        // 5. prompt 地址 seed
   329	        for (_, text) in &texts {
   330	            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
   331	                tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
   332	            }
   333	        }
   334	
   335	        // 6. 出站通过 → 入站 SSE tee 截流检测
   336	        return forward_with_inbound_inspection(
   337	            forwarder,
   338	            inbound_filter,
   339	            dry_run,
   340	            ipc,

exec
/bin/zsh -lc 'cat crates/sieve-rules/rules/inbound.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
# 入站 P0 规则集（关联 PRD §5.2 + UCSB 论文 4 类攻击）
# 来源：Semgrep command-injection（MIT/Apache-2.0）+ Sieve 自写
#
# Vectorscan PCRE 子集约束：
#   - 禁用 lookahead / lookbehind / 反向引用 / 原子组
#   - 所有 pattern 仅用 (?i) + 字符类 + 量词 + 分组

# IN-CR-01 是地址替换，由 sieve-core::address_guard 实现（strsim Levenshtein），
# 此处仅占位以保持 ID 注册；loader 看到 pattern == "__ADDRESS_GUARD_PLACEHOLDER__" 时跳过 vectorscan 编译。
[[rules]]
id = "IN-CR-01"
description = "Address substitution attack (handled by sieve-core::address_guard with strsim Levenshtein)"
pattern = "__ADDRESS_GUARD_PLACEHOLDER__"
severity = "critical"
action = "block"
disposition = "gui_popup"
timeout_seconds = 60
default_on_timeout = "block"

# IN-CR-02 危险 shell 命令（Semgrep command-injection 风格）
[[rules]]
id = "IN-CR-02"
description = "Destructive rm command"
pattern = '''(?i)rm\s+-rf?\s+[/~*]'''
severity = "critical"
action = "block"
keywords = ["rm"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-02-CURL-PIPE"
description = "Curl-pipe-shell pattern (curl ... | sh)"
pattern = '''(?i)curl\s+\S+\s*\|\s*(ba)?sh'''
severity = "critical"
action = "block"
keywords = ["curl"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-02-WGET-PIPE"
description = "Wget-pipe-shell pattern"
pattern = '''(?i)wget\s+\S+\s*\|\s*(ba)?sh'''
severity = "critical"
action = "block"
keywords = ["wget"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-02-EVAL"
description = "eval with dynamic input"
pattern = '''(?i)eval\s*[\(\$"]'''
severity = "critical"
action = "block"
keywords = ["eval"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-02-NC-REVERSE"
description = "Netcat reverse shell"
pattern = '''(?i)nc\s+(-e|--sh-exec)\s+/bin/(ba)?sh'''
severity = "critical"
action = "block"
keywords = ["nc"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-02-DD-WIPE"
description = "dd disk wipe"
pattern = '''(?i)dd\s+if=/dev/zero\s+of=/dev/'''
severity = "critical"
action = "block"
keywords = ["dd"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

# ---------------------------------------------------------------------------
# IN-CR-03 敏感路径访问（PRD §5.2，Week 4）
# 处置：High warn 5s（合法用例存在，需用户判断；Critical block 误报代价过高）
# 复用 engine_adapter::check_tool_use → tool.input JSON 喂给 vectorscan 的通道。
# ---------------------------------------------------------------------------
[[rules]]
id = "IN-CR-03-SSH-PRIVATE"
description = "SSH private key file (id_rsa / id_ed25519 / id_ecdsa / id_dsa)"
pattern = '''\b(?:id_rsa|id_ed25519|id_ecdsa|id_dsa)(?:\.pub)?\b'''
severity = "high"
action = "warn"
keywords = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"]
allowlist_regexes = ['''\.pub\b''']
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-SSH-DIR"
description = "SSH directory access (~/.ssh/...)"
pattern = '''~/\.ssh(?:/[a-zA-Z0-9_\-\.]+)?'''
severity = "high"
action = "warn"
keywords = [".ssh"]
allowlist_regexes = ['''~/\.ssh/(?:known_hosts|authorized_keys|config|environment)\b''']
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-AWS-CREDS"
description = "AWS credentials file (~/.aws/credentials)"
pattern = '''(?i)\.aws/credentials\b'''
severity = "high"
action = "warn"
keywords = ["credentials"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-DOTENV"
description = "dotenv file (.env / .env.local / .env.production)"
pattern = '''\.env\b(?:\.[a-zA-Z0-9_\-]+)*'''
severity = "high"
action = "warn"
keywords = [".env"]
allowlist_regexes = ['''(?i)\.env\.(?:example|template|sample|dist|test|ci|cypress)\b''']
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-ETH-KEYSTORE"
description = "Ethereum/geth keystore file (UTC--<timestamp>--<40hex>)"
pattern = '''(?i)UTC--[0-9T\-Z\.]{19,32}--[a-fA-F0-9]{40}\b'''
severity = "high"
action = "warn"
keywords = ["UTC--"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-GPG-DIR"
description = "GPG private key directory (~/.gnupg)"
pattern = '''~/\.gnupg(?:/[a-zA-Z0-9_\-\.]+)?'''
severity = "high"
action = "warn"
keywords = [".gnupg"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-NETRC"
description = "netrc credential file"
# 注意：前置不能用 \b——常见路径 `~/.netrc` / `/Users/x/.netrc` 中 `.` 与
# 周围非 word 字符（`/` `~`）之间无 word boundary。仅靠尾部 \b 锚定即可。
pattern = '''\.netrc\b'''
severity = "high"
action = "warn"
keywords = [".netrc"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-MACOS-KEYCHAIN"
description = "macOS Keychain database (login.keychain-db / System.keychain)"
pattern = '''\b(?:login|System)\.keychain(?:-db)?\b'''
severity = "high"
action = "warn"
keywords = ["keychain"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-GCP-CREDS"
description = "GCP application default credentials (~/.config/gcloud/...)"
pattern = '''(?i)\.config/gcloud/(?:application_default_credentials\.json|legacy_credentials/[^\s"']*)'''
severity = "high"
action = "warn"
keywords = ["gcloud"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-CR-03-SOLANA-KEYPAIR"
description = "Solana CLI default keypair (~/.config/solana/*.json)"
pattern = '''(?i)\.config/solana/[a-zA-Z0-9_\-]+\.json\b'''
severity = "high"
action = "warn"
keywords = ["solana"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

# ---------------------------------------------------------------------------
# IN-CR-04 持久化机制（PRD §5.2 / US-07，Week 4）
# 处置：Critical block + fail-closed（YOLO mode 不可关）。写持久化文件 = 后门埋点级别。
# 关联 ADR-007 §"Week 4 落地范围"。
# 设计原则：pattern 锚定"写意图"（>>，tee -a，cp/mv 到目标，crontab -e 等），
# 不拦读路径——避免与 IN-CR-03 read=High 处置冲突。
# 已知 gap：Edit/Write tool 直接写持久化文件无 Bash 重定向上下文，本期不补——
# 配套 launchctl/systemctl/crontab 启用动作仍会被对应规则截获。
# ---------------------------------------------------------------------------
[[rules]]
id = "IN-CR-04-SHELL-RC-APPEND"
description = "Write/append to shell rc files (.bashrc / .zshrc / .bash_profile etc.)"
pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*\.(?:bashrc|bash_profile|bash_login|bash_aliases|profile|zshrc|zprofile|zlogin|zsh_aliases|kshrc)\b'''
severity = "critical"
action = "block"
keywords = ["bashrc", "zshrc", "bash_profile", "zprofile"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-CRONTAB"
description = "crontab edit/install/remove (-e/-r/<)"
pattern = '''\bcrontab\s+(?:-e\b|-r\b|<)'''
severity = "critical"
action = "block"
keywords = ["crontab"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-CRON-D-WRITE"
description = "Write to /etc/cron.{d,daily,hourly,monthly,weekly}/ directories"
pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*/etc/cron\.(?:d|daily|hourly|monthly|weekly|allow|deny)/[^/\s'"\n]+'''
severity = "critical"
action = "block"
keywords = ["/etc/cron"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-LAUNCHCTL"
description = "launchctl load/bootstrap/enable/kickstart (macOS persistence activation)"
pattern = '''\blaunchctl\s+(?:load|bootstrap|enable|kickstart|asuser)\b'''
severity = "critical"
action = "block"
keywords = ["launchctl"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-LAUNCH-AGENT-PLIST"
description = "Write LaunchAgents/LaunchDaemons plist (macOS persistence file)"
pattern = '''(?:>>?|tee\s+(?:-a\s+)?|cp\s+\S+\s+|mv\s+\S+\s+|cat\s+>\s*)[^\n;]*Library/(?:LaunchAgents|LaunchDaemons)/[^/\s'"\n]+\.plist\b'''
severity = "critical"
action = "block"
keywords = ["LaunchAgents", "LaunchDaemons"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-SYSTEMCTL-ENABLE"
description = "systemctl enable/start/daemon-reload (Linux persistence activation)"
pattern = '''\bsystemctl\s+(?:--user\s+)?(?:enable|start|daemon-reload)\b'''
severity = "critical"
action = "block"
keywords = ["systemctl"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-SYSTEMD-UNIT-WRITE"
description = "Write systemd unit file (/etc/systemd/system/*.service or ~/.config/systemd/user/*.service)"
pattern = '''(?:>>?|tee\s+(?:-a\s+)?|cat\s+>\s*)[^\n;]*(?:/etc/systemd/system|\.config/systemd/user)/[^/\s'"\n]+\.(?:service|timer|socket)\b'''
severity = "critical"
action = "block"
keywords = ["systemd"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-FISH-CONFIG"
description = "Append to fish shell config (~/.config/fish/config.fish or conf.d/*.fish)"
pattern = '''(?:>>?|tee\s+(?:-a\s+)?)[^\n;]*\.config/fish/(?:config\.fish|conf\.d/[^/\s'"\n]+\.fish)\b'''
severity = "critical"
action = "block"
keywords = ["fish"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

[[rules]]
id = "IN-CR-04-LOGIN-ITEMS"
description = "macOS Login Items registration (defaults write LoginItems / osascript login items)"
pattern = '''(?i)(?:\bdefaults\s+write\b[^\n;]*\bLoginItems\b|\bosascript\b[^\n;]*(?:System Events|login items))'''
severity = "critical"
action = "block"
keywords = ["LoginItems", "osascript"]
disposition = "hook_terminal"
timeout_seconds = 60
default_on_timeout = "block"

# IN-GEN-04 markdown exfil（warn 级别；Week 4 由旧 IN-CR-04 重命名归入 IN-GEN-* 命名空间，
# 关联 PRD §5.2 / US-08，详见 CHANGELOG [BREAKING]）
[[rules]]
id = "IN-GEN-04"
description = "Markdown image with URL query params (potential data exfil)"
pattern = '''!\[[^\]]*\]\(https?://[^)]+[\?&][^)]*\)'''
severity = "high"
action = "warn"
disposition = "gui_popup"
timeout_seconds = 30
default_on_timeout = "block"

# IN-CR-05 签名工具白名单（EVM + Solana + Bitcoin 全拦）
[[rules]]
id = "IN-CR-05-EVM"
description = "EVM signing tool call (eth_signTransaction / personal_sign / signTypedData_v4 etc.)"
pattern = '''(?i)\b(eth_sendTransaction|eth_signTransaction|eth_signTypedData(_v[34])?|personal_sign|wallet_signTypedData)\b'''
severity = "critical"
action = "block"
keywords = ["sign"]
disposition = "gui_popup"
timeout_seconds = 120
default_on_timeout = "block"

[[rules]]
id = "IN-CR-05-SOLANA"
description = "Solana signing tool call"
pattern = '''(?i)\b(signTransaction|signMessage|partialSign|signAllTransactions|signAndSendTransaction)\b'''
severity = "critical"
action = "block"
keywords = ["sign"]
disposition = "gui_popup"
timeout_seconds = 120
default_on_timeout = "block"

[[rules]]
id = "IN-CR-05-BITCOIN"
description = "Bitcoin signing tool call"
pattern = '''(?i)\b(signRawTransaction(With(Key|Wallet))?|signMessage)\b'''
severity = "critical"
action = "block"
keywords = ["sign"]
disposition = "gui_popup"
timeout_seconds = 120
default_on_timeout = "block"

# IN-GEN-01~03 候选（Week 4 完整化）
[[rules]]
id = "IN-GEN-01"
description = "Markdown javascript: URI"
pattern = '''(?i)\[.*?\]\(javascript:'''
severity = "critical"
action = "block"
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-GEN-02"
description = "Inline HTML img with external URL"
pattern = '''(?i)<img\s[^>]*src\s*=\s*["']https?://'''
severity = "high"
action = "warn"
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

[[rules]]
id = "IN-GEN-03"
description = "Bash -c arbitrary execution"
pattern = '''(?i)bash\s+-c\s+["']'''
severity = "critical"
action = "block"
keywords = ["bash"]
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"

exec
/bin/zsh -lc "for f in crates/sieve-cli/src/commands/*.rs; do echo '###' "'$f; nl -ba $f | sed -n '"'1,260p'; done" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
### crates/sieve-cli/src/commands/doctor.rs
     1	//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor）。
     2	//!
     3	//! 5 项检查：
     4	//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
     5	//! 2. hooks.PreToolUse 是否含 sieve-hook check
     6	//! 3. daemon 是否在 :11453 监听（TCP 连接）
     7	//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
     8	//! 5. canary 拦截测试（构造 OUT-01 已知字符串，验证 daemon 改写）
     9	//!
    10	//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
    11	
    12	use anyhow::Result;
    13	
    14	#[cfg(target_os = "macos")]
    15	pub use macos::run;
    16	
    17	#[cfg(not(target_os = "macos"))]
    18	pub use stub::run;
    19	
    20	// ──────────────────────────────── macOS 实现 ────────────────────────────────
    21	
    22	#[cfg(target_os = "macos")]
    23	mod macos {
    24	    use super::*;
    25	    use std::io::Write as IoWrite;
    26	    use std::process::Command;
    27	
    28	    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
    29	    pub fn run() -> Result<()> {
    30	        let home = std::env::var("HOME").unwrap_or_default();
    31	        let settings_path = std::path::PathBuf::from(&home)
    32	            .join(".claude")
    33	            .join("settings.json");
    34	
    35	        let mut all_ok = true;
    36	
    37	        // ── 检查 1: ANTHROPIC_BASE_URL
    38	        let check1 = check_base_url(&settings_path);
    39	        print_check(
    40	            "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
    41	            check1,
    42	        );
    43	        all_ok &= check1;
    44	
    45	        // ── 检查 2: PreToolUse hook
    46	        let check2 = check_hook_registered(&settings_path);
    47	        print_check(
    48	            "settings.json: hooks.PreToolUse 含 sieve-hook check",
    49	            check2,
    50	        );
    51	        all_ok &= check2;
    52	
    53	        // ── 检查 3: daemon 监听 :11453
    54	        let check3 = check_daemon_listening();
    55	        print_check("daemon 在 127.0.0.1:11453 监听", check3);
    56	        all_ok &= check3;
    57	
    58	        // ── 检查 4: launchd 状态
    59	        let check4 = check_launchd();
    60	        print_check("launchd com.sieve.daemon 已加载", check4);
    61	        all_ok &= check4;
    62	
    63	        // ── 检查 5: canary 拦截测试
    64	        let check5 = check_canary();
    65	        print_check("canary 拦截测试（OUT-01 脱敏）", check5);
    66	        all_ok &= check5;
    67	
    68	        // ── 汇总
    69	        println!();
    70	        if all_ok {
    71	            println!("✅ 所有检查通过，Sieve 运行正常。");
    72	        } else {
    73	            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
    74	        }
    75	
    76	        Ok(())
    77	    }
    78	
    79	    fn print_check(label: &str, ok: bool) {
    80	        let icon = if ok { "✅" } else { "❌" };
    81	        println!("  {} {}", icon, label);
    82	    }
    83	
    84	    /// 检查 settings.json 中 ANTHROPIC_BASE_URL。
    85	    fn check_base_url(path: &std::path::Path) -> bool {
    86	        let Ok(raw) = std::fs::read_to_string(path) else {
    87	            return false;
    88	        };
    89	        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
    90	            return false;
    91	        };
    92	        v.pointer("/env/ANTHROPIC_BASE_URL")
    93	            .and_then(|x| x.as_str())
    94	            .map(|s| s == "http://127.0.0.1:11453")
    95	            .unwrap_or(false)
    96	    }
    97	
    98	    /// 检查 PreToolUse hook 是否含 sieve-hook check。
    99	    fn check_hook_registered(path: &std::path::Path) -> bool {
   100	        let Ok(raw) = std::fs::read_to_string(path) else {
   101	            return false;
   102	        };
   103	        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
   104	            return false;
   105	        };
   106	        v.pointer("/hooks/PreToolUse")
   107	            .and_then(|arr| arr.as_array())
   108	            .map(|arr| {
   109	                arr.iter().any(|item| {
   110	                    item.pointer("/hooks/0/command")
   111	                        .and_then(|c| c.as_str())
   112	                        .map(|c| c.contains("sieve-hook"))
   113	                        .unwrap_or(false)
   114	                })
   115	            })
   116	            .unwrap_or(false)
   117	    }
   118	
   119	    /// 尝试 TCP 连接 127.0.0.1:11453，成功则 daemon 在监听。
   120	    fn check_daemon_listening() -> bool {
   121	        use std::net::TcpStream;
   122	        use std::time::Duration;
   123	        TcpStream::connect_timeout(
   124	            &"127.0.0.1:11453".parse().unwrap(),
   125	            Duration::from_millis(500),
   126	        )
   127	        .is_ok()
   128	    }
   129	
   130	    /// 检查 launchctl list 是否含 com.sieve.daemon。
   131	    fn check_launchd() -> bool {
   132	        let Ok(output) = Command::new("launchctl").arg("list").output() else {
   133	            return false;
   134	        };
   135	        let stdout = String::from_utf8_lossy(&output.stdout);
   136	        stdout.contains("com.sieve.daemon")
   137	    }
   138	
   139	    /// Canary 拦截测试：向 daemon 发送含 OUT-01 特征的请求，
   140	    /// 验证响应中已脱敏（不含原始 sk- token）。
   141	    ///
   142	    /// 注意：此测试仅在 daemon 运行时有意义；daemon 未运行时直接返回 false。
   143	    fn check_canary() -> bool {
   144	        use std::io::{Read, Write};
   145	        use std::net::TcpStream;
   146	        use std::time::Duration;
   147	
   148	        // daemon 未运行直接 false
   149	        let Ok(mut stream) = TcpStream::connect_timeout(
   150	            &"127.0.0.1:11453".parse().unwrap(),
   151	            Duration::from_millis(500),
   152	        ) else {
   153	            return false;
   154	        };
   155	        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
   156	
   157	        // 构造含已知 OUT-01 特征（sk-ant-api03-... 格式）的请求体
   158	        // 注意：这里使用测试用虚假 token，格式符合 OUT-01 模式
   159	        let canary_token = "sk-ant-api03-canary-test-aaaabbbbccccdddd-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX_AA";
   160	        let body = serde_json::json!({
   161	            "model": "claude-3-5-haiku-20241022",
   162	            "max_tokens": 1,
   163	            "messages": [{
   164	                "role": "user",
   165	                "content": format!("hello {canary_token}")
   166	            }]
   167	        })
   168	        .to_string();
   169	
   170	        let request = format!(
   171	            "POST /v1/messages HTTP/1.1\r\n\
   172	             Host: 127.0.0.1:11453\r\n\
   173	             Content-Type: application/json\r\n\
   174	             Content-Length: {}\r\n\
   175	             x-api-key: test\r\n\
   176	             anthropic-version: 2023-06-01\r\n\
   177	             Connection: close\r\n\
   178	             \r\n\
   179	             {}",
   180	            body.len(),
   181	            body
   182	        );
   183	
   184	        if stream.write_all(request.as_bytes()).is_err() {
   185	            return false;
   186	        }
   187	
   188	        let mut response = String::new();
   189	        let _ = stream.read_to_string(&mut response);
   190	
   191	        // 验证响应中不含原始 canary token（已被脱敏/拦截）
   192	        !response.contains(canary_token)
   193	    }
   194	
   195	    // 抑制 IoWrite 未使用警告
   196	    const _: fn() = || {
   197	        let _ = std::io::stdout().flush();
   198	    };
   199	}
   200	
   201	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   202	
   203	#[cfg(not(target_os = "macos"))]
   204	mod stub {
   205	    use super::*;
   206	
   207	    /// `sieve doctor` 非 macOS 占位实现。
   208	    pub fn run() -> Result<()> {
   209	        anyhow::bail!(
   210	            "sieve doctor is macOS only in Phase 1. \
   211	             Linux/Windows support is planned for Phase 2."
   212	        )
   213	    }
   214	}
### crates/sieve-cli/src/commands/mod.rs
     1	//! Week 5 CLI 子命令模块（ADR-015 / SPEC-003）。
     2	//!
     3	//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
     4	//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
     5	//! - `uninstall`：干净回滚 setup 改动（仅 macOS）
     6	
     7	pub mod doctor;
     8	pub mod setup;
     9	pub mod uninstall;
### crates/sieve-cli/src/commands/setup.rs
     1	//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup）。
     2	//!
     3	//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
     4	//!
     5	//! 步骤：
     6	//! 1. 检测 `~/.claude/settings.json` 是否存在
     7	//! 2. 计算 diff（ANTHROPIC_BASE_URL + PreToolUse hook + launchd plist）
     8	//! 3. dry-run 打印 diff，非 --yes 等待用户确认
     9	//! 4. 备份原文件到 `~/.sieve/backups/<RFC3339>/`
    10	//! 5. 修改 settings.json
    11	//! 6. 写 launchd plist + `launchctl load -w`
    12	//! 7. 写 setup.log（JSON Lines）
    13	//! 8. 自动调用 doctor 验证
    14	//!
    15	//! 错误恢复：任意步骤失败 → 反向回滚已做改动。
    16	
    17	use crate::cli::SetupArgs;
    18	use anyhow::Result;
    19	
    20	#[cfg(target_os = "macos")]
    21	pub use macos::run;
    22	
    23	#[cfg(not(target_os = "macos"))]
    24	pub use stub::run;
    25	
    26	// ──────────────────────────────── macOS 实现 ────────────────────────────────
    27	
    28	#[cfg(target_os = "macos")]
    29	mod macos {
    30	    use super::*;
    31	    use crate::commands::doctor;
    32	    use anyhow::{anyhow, bail, Context};
    33	    use chrono::Utc;
    34	    use serde_json::Value;
    35	    use std::fs;
    36	    use std::io::{self, Write as IoWrite};
    37	    use std::path::{Path, PathBuf};
    38	    use std::process::Command;
    39	
    40	    /// setup.log 每行的结构（JSON Lines）。
    41	    #[derive(serde::Serialize)]
    42	    struct SetupLogEntry {
    43	        timestamp: String,
    44	        action: String,
    45	        path: Option<String>,
    46	        detail: Option<String>,
    47	    }
    48	
    49	    impl SetupLogEntry {
    50	        fn new(action: impl Into<String>) -> Self {
    51	            Self {
    52	                timestamp: Utc::now().to_rfc3339(),
    53	                action: action.into(),
    54	                path: None,
    55	                detail: None,
    56	            }
    57	        }
    58	
    59	        fn with_path(mut self, path: impl Into<String>) -> Self {
    60	            self.path = Some(path.into());
    61	            self
    62	        }
    63	
    64	        fn with_detail(mut self, detail: impl Into<String>) -> Self {
    65	            self.detail = Some(detail.into());
    66	            self
    67	        }
    68	    }
    69	
    70	    /// setup 执行上下文，用于错误时反向回滚。
    71	    struct SetupContext {
    72	        backup_dir: PathBuf,
    73	        /// 已写入的文件路径，错误时按逆序恢复。
    74	        written_files: Vec<PathBuf>,
    75	        /// 已执行的 launchctl load，错误时需要 unload。
    76	        launchd_loaded: Option<PathBuf>,
    77	    }
    78	
    79	    impl SetupContext {
    80	        fn new(backup_dir: PathBuf) -> Self {
    81	            Self {
    82	                backup_dir,
    83	                written_files: Vec::new(),
    84	                launchd_loaded: None,
    85	            }
    86	        }
    87	
    88	        /// 回滚所有已做改动（从备份目录恢复）。
    89	        fn rollback(&self) {
    90	            eprintln!("[sieve setup] 回滚已做改动…");
    91	
    92	            if let Some(plist) = &self.launchd_loaded {
    93	                let _ = Command::new("launchctl")
    94	                    .args(["unload", &plist.to_string_lossy()])
    95	                    .status();
    96	                eprintln!("  ↩ launchctl unload {}", plist.display());
    97	            }
    98	
    99	            for path in self.written_files.iter().rev() {
   100	                // 计算备份中的相对路径：去掉 HOME 前缀
   101	                let home = std::env::var("HOME").unwrap_or_default();
   102	                let rel = path.strip_prefix(&home).unwrap_or(path.as_path());
   103	                let backup_src = self.backup_dir.join(rel);
   104	                if backup_src.exists() {
   105	                    if let Err(e) = fs::copy(&backup_src, path) {
   106	                        eprintln!("  ✗ 恢复 {} 失败: {}", path.display(), e);
   107	                    } else {
   108	                        eprintln!("  ↩ 恢复 {}", path.display());
   109	                    }
   110	                } else {
   111	                    // 备份不存在说明是新建的，直接删除
   112	                    let _ = fs::remove_file(path);
   113	                    eprintln!("  ↩ 删除新建文件 {}", path.display());
   114	                }
   115	            }
   116	        }
   117	    }
   118	
   119	    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
   120	    pub fn run(args: SetupArgs) -> Result<()> {
   121	        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
   122	        let home_path = PathBuf::from(&home);
   123	
   124	        let settings_path = home_path.join(".claude").join("settings.json");
   125	        let sieve_home =
   126	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
   127	        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
   128	        let backup_dir = sieve_home.join("backups").join(&backup_ts);
   129	        let plist_path = home_path
   130	            .join("Library")
   131	            .join("LaunchAgents")
   132	            .join("com.sieve.daemon.plist");
   133	        let setup_log_path = sieve_home.join("setup.log");
   134	
   135	        // ── 1. 读取现有 settings.json（允许不存在）
   136	        let existing_settings: Value = if settings_path.exists() {
   137	            let raw =
   138	                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
   139	            // Strip JSON 注释（简单处理：删除 // 行注释）
   140	            let stripped = strip_json_comments(&raw);
   141	            serde_json::from_str(&stripped).unwrap_or_else(|_| serde_json::json!({}))
   142	        } else {
   143	            serde_json::json!({})
   144	        };
   145	
   146	        // ── 2. 计算 diff
   147	        let sieve_url = "http://127.0.0.1:11453";
   148	        let hook_entry = serde_json::json!({
   149	            "matcher": ".*",
   150	            "hooks": [{"type": "command", "command": "sieve-hook check"}]
   151	        });
   152	
   153	        let current_base_url = existing_settings
   154	            .pointer("/env/ANTHROPIC_BASE_URL")
   155	            .and_then(|v| v.as_str())
   156	            .unwrap_or("<未设置>");
   157	        let has_hook = existing_settings
   158	            .pointer("/hooks/PreToolUse")
   159	            .and_then(|v| v.as_array())
   160	            .map(|arr| {
   161	                arr.iter().any(|item| {
   162	                    item.pointer("/hooks/0/command")
   163	                        .and_then(|c| c.as_str())
   164	                        .map(|c| c.contains("sieve-hook"))
   165	                        .unwrap_or(false)
   166	                })
   167	            })
   168	            .unwrap_or(false);
   169	        let plist_content = build_plist_content()?;
   170	
   171	        // ── 3. 打印 diff
   172	        println!("=== sieve setup diff ===");
   173	        println!(
   174	            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
   175	            current_base_url, sieve_url
   176	        );
   177	        if has_hook {
   178	            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
   179	        } else {
   180	            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
   181	        }
   182	        println!("[launchd] 写入 {}", plist_path.display());
   183	        println!("[launchd] 执行 launchctl load -w");
   184	        println!("========================");
   185	
   186	        // ── 4. dry-run 直接返回
   187	        if args.dry_run {
   188	            println!("[dry-run] 未做任何改动。");
   189	            return Ok(());
   190	        }
   191	
   192	        // ── 5. 等待用户确认
   193	        if !args.yes {
   194	            print!("继续执行以上操作？[y/N] ");
   195	            io::stdout().flush()?;
   196	            let mut input = String::new();
   197	            io::stdin().read_line(&mut input)?;
   198	            if !input.trim().eq_ignore_ascii_case("y") {
   199	                println!("已取消。");
   200	                return Ok(());
   201	            }
   202	        }
   203	
   204	        // ── 6. 备份
   205	        fs::create_dir_all(&backup_dir)
   206	            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
   207	        let mut ctx = SetupContext::new(backup_dir.clone());
   208	
   209	        let result = do_setup(
   210	            &mut ctx,
   211	            &home_path,
   212	            &settings_path,
   213	            &plist_path,
   214	            &setup_log_path,
   215	            &backup_dir,
   216	            existing_settings,
   217	            sieve_url,
   218	            hook_entry,
   219	            plist_content,
   220	        );
   221	
   222	        if let Err(ref e) = result {
   223	            eprintln!("[sieve setup] 失败: {e}");
   224	            ctx.rollback();
   225	            return result;
   226	        }
   227	
   228	        // ── 9. 自动跑 doctor 验证
   229	        println!("\n[sieve setup] 正在验证安装…");
   230	        doctor::run()?;
   231	
   232	        Ok(())
   233	    }
   234	
   235	    #[allow(clippy::too_many_arguments)]
   236	    fn do_setup(
   237	        ctx: &mut SetupContext,
   238	        home_path: &Path,
   239	        settings_path: &Path,
   240	        plist_path: &Path,
   241	        setup_log_path: &Path,
   242	        backup_dir: &Path,
   243	        mut existing_settings: Value,
   244	        sieve_url: &str,
   245	        hook_entry: Value,
   246	        plist_content: String,
   247	    ) -> Result<()> {
   248	        // 备份 settings.json
   249	        if settings_path.exists() {
   250	            let rel = settings_path
   251	                .strip_prefix(home_path)
   252	                .unwrap_or(settings_path);
   253	            let backup_dest = backup_dir.join(rel);
   254	            if let Some(parent) = backup_dest.parent() {
   255	                fs::create_dir_all(parent)?;
   256	            }
   257	            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
   258	        }
   259	
   260	        // 修改 settings.json
### crates/sieve-cli/src/commands/uninstall.rs
     1	//! `sieve uninstall` 命令实现（ADR-015 / SPEC-003 §uninstall）。
     2	//!
     3	//! 步骤：
     4	//! 1. 读 `~/.sieve/setup.log` 反向遍历 entries（了解 backup_dir）
     5	//! 2. dry-run 打印将恢复的内容
     6	//! 3. 非 --yes 等待用户确认
     7	//! 4. 从 `~/.sieve/backups/` 恢复原文件
     8	//! 5. `launchctl unload` 并删除 plist 文件
     9	//! 6. 提示用户手动删 `~/.sieve/`
    10	//!
    11	//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
    12	
    13	use crate::cli::UninstallArgs;
    14	use anyhow::Result;
    15	
    16	#[cfg(target_os = "macos")]
    17	pub use macos::run;
    18	
    19	#[cfg(not(target_os = "macos"))]
    20	pub use stub::run;
    21	
    22	// ──────────────────────────────── macOS 实现 ────────────────────────────────
    23	
    24	#[cfg(target_os = "macos")]
    25	mod macos {
    26	    use super::*;
    27	    use anyhow::{anyhow, Context};
    28	    use std::fs;
    29	    use std::io::{self, Write as IoWrite};
    30	    use std::path::PathBuf;
    31	    use std::process::Command;
    32	
    33	    /// 运行 `sieve uninstall`。关联 ADR-015 / SPEC-003 §uninstall。
    34	    pub fn run(args: UninstallArgs) -> Result<()> {
    35	        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
    36	        let home_path = PathBuf::from(&home);
    37	        let sieve_home =
    38	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
    39	        let setup_log_path = sieve_home.join("setup.log");
    40	        let plist_path = home_path
    41	            .join("Library")
    42	            .join("LaunchAgents")
    43	            .join("com.sieve.daemon.plist");
    44	        let backups_root = sieve_home.join("backups");
    45	
    46	        // ── 1. 读取 setup.log，找到最新 backup_dir
    47	        let latest_backup = find_latest_backup(&setup_log_path, &backups_root);
    48	
    49	        // ── 2. 打印将要恢复的内容
    50	        println!("=== sieve uninstall 预览 ===");
    51	        if let Some(ref bd) = latest_backup {
    52	            println!("[restore] 从备份目录恢复: {}", bd.display());
    53	            list_backup_files(bd);
    54	        } else {
    55	            println!("[restore] 未找到备份目录，将跳过文件恢复");
    56	        }
    57	        if plist_path.exists() {
    58	            println!("[launchd] launchctl unload {}", plist_path.display());
    59	            println!("[launchd] 删除 {}", plist_path.display());
    60	        }
    61	        println!("[提示] ~/.sieve/ 目录将保留（含审计日志），请手动删除：");
    62	        println!("       rm -rf {}", sieve_home.display());
    63	        println!("===========================");
    64	
    65	        if args.dry_run {
    66	            println!("[dry-run] 未做任何改动。");
    67	            return Ok(());
    68	        }
    69	
    70	        // ── 3. 等待用户确认
    71	        if !args.yes {
    72	            print!("继续执行以上操作？[y/N] ");
    73	            io::stdout().flush()?;
    74	            let mut input = String::new();
    75	            io::stdin().read_line(&mut input)?;
    76	            if !input.trim().eq_ignore_ascii_case("y") {
    77	                println!("已取消。");
    78	                return Ok(());
    79	            }
    80	        }
    81	
    82	        // ── 4. 从备份恢复文件
    83	        if let Some(ref bd) = latest_backup {
    84	            restore_from_backup(bd, &home_path)?;
    85	        }
    86	
    87	        // ── 5. 卸载 launchd
    88	        if plist_path.exists() {
    89	            let status = Command::new("launchctl")
    90	                .args(["unload", &plist_path.to_string_lossy()])
    91	                .status();
    92	            match status {
    93	                Ok(s) if s.success() => println!("[uninstall] ✅ launchd 服务已卸载"),
    94	                Ok(s) => eprintln!("[uninstall] ⚠ launchctl unload 返回: {:?}", s.code()),
    95	                Err(e) => eprintln!("[uninstall] ⚠ launchctl unload 失败: {e}"),
    96	            }
    97	            if let Err(e) = fs::remove_file(&plist_path) {
    98	                eprintln!("[uninstall] ⚠ 删除 plist 失败: {e}");
    99	            } else {
   100	                println!("[uninstall] ✅ plist 已删除");
   101	            }
   102	        }
   103	
   104	        // ── 6. 提示手动删除
   105	        println!();
   106	        println!("✅ 卸载完成。");
   107	        println!("提示：审计日志和备份文件保留在 {}", sieve_home.display());
   108	        println!("如需彻底清除，请手动运行：");
   109	        println!("  rm -rf {}", sieve_home.display());
   110	
   111	        Ok(())
   112	    }
   113	
   114	    /// 从 setup.log 找最新的 backup_dir；找不到则扫描 backups/ 最新目录。
   115	    fn find_latest_backup(
   116	        setup_log: &std::path::Path,
   117	        backups_root: &std::path::Path,
   118	    ) -> Option<PathBuf> {
   119	        // 先尝试从 setup.log 读取
   120	        if let Ok(raw) = fs::read_to_string(setup_log) {
   121	            let last_backup: Option<PathBuf> = raw
   122	                .lines()
   123	                .rev()
   124	                .filter_map(|line| {
   125	                    let v: serde_json::Value = serde_json::from_str(line).ok()?;
   126	                    if v.get("action")?.as_str()? == "setup_complete" {
   127	                        let detail = v.get("detail")?.as_str()?;
   128	                        let dir = detail.strip_prefix("backup_dir=")?;
   129	                        Some(PathBuf::from(dir))
   130	                    } else {
   131	                        None
   132	                    }
   133	                })
   134	                .next();
   135	            if last_backup.is_some() {
   136	                return last_backup;
   137	            }
   138	        }
   139	
   140	        // 退而求其次：扫描 backups/ 下最新目录（按名称字典序，RFC3339 时间戳排序正确）
   141	        if !backups_root.exists() {
   142	            return None;
   143	        }
   144	        let mut entries: Vec<PathBuf> = fs::read_dir(backups_root)
   145	            .ok()?
   146	            .filter_map(|e| e.ok().map(|e| e.path()))
   147	            .filter(|p| p.is_dir())
   148	            .collect();
   149	        entries.sort();
   150	        entries.into_iter().next_back()
   151	    }
   152	
   153	    /// 打印备份目录中的文件列表。
   154	    fn list_backup_files(backup_dir: &std::path::Path) {
   155	        if let Ok(walker) = fs::read_dir(backup_dir) {
   156	            for entry in walker.flatten() {
   157	                println!("  - {}", entry.path().display());
   158	            }
   159	        }
   160	    }
   161	
   162	    /// 将备份目录中的文件逐一恢复到 home 下对应路径。
   163	    fn restore_from_backup(
   164	        backup_dir: &std::path::Path,
   165	        home_path: &std::path::Path,
   166	    ) -> Result<()> {
   167	        restore_dir_recursive(backup_dir, backup_dir, home_path)
   168	    }
   169	
   170	    fn restore_dir_recursive(
   171	        root: &std::path::Path,
   172	        current: &std::path::Path,
   173	        home_path: &std::path::Path,
   174	    ) -> Result<()> {
   175	        for entry in fs::read_dir(current)
   176	            .with_context(|| format!("读取备份目录 {} 失败", current.display()))?
   177	        {
   178	            let entry = entry?;
   179	            let path = entry.path();
   180	            if path.is_dir() {
   181	                restore_dir_recursive(root, &path, home_path)?;
   182	            } else {
   183	                // 计算目标路径：backup_root/rel → home/rel
   184	                let rel = path.strip_prefix(root).unwrap_or(path.as_path());
   185	                let dest = home_path.join(rel);
   186	                if let Some(parent) = dest.parent() {
   187	                    fs::create_dir_all(parent)?;
   188	                }
   189	                fs::copy(&path, &dest).with_context(|| {
   190	                    format!("恢复 {} → {} 失败", path.display(), dest.display())
   191	                })?;
   192	                println!("[uninstall] ✅ 恢复 {}", dest.display());
   193	            }
   194	        }
   195	        Ok(())
   196	    }
   197	}
   198	
   199	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   200	
   201	#[cfg(not(target_os = "macos"))]
   202	mod stub {
   203	    use super::*;
   204	
   205	    /// `sieve uninstall` 非 macOS 占位实现。
   206	    pub fn run(_args: UninstallArgs) -> Result<()> {
   207	        anyhow::bail!(
   208	            "sieve uninstall is macOS only in Phase 1. \
   209	             Linux/Windows support is planned for Phase 2."
   210	        )
   211	    }
   212	}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '248,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   248	        // 备份 settings.json
   249	        if settings_path.exists() {
   250	            let rel = settings_path
   251	                .strip_prefix(home_path)
   252	                .unwrap_or(settings_path);
   253	            let backup_dest = backup_dir.join(rel);
   254	            if let Some(parent) = backup_dest.parent() {
   255	                fs::create_dir_all(parent)?;
   256	            }
   257	            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
   258	        }
   259	
   260	        // 修改 settings.json
   261	        {
   262	            let env = existing_settings
   263	                .get_mut("env")
   264	                .and_then(|v| v.as_object_mut())
   265	                .map(|obj| {
   266	                    obj.insert(
   267	                        "ANTHROPIC_BASE_URL".to_string(),
   268	                        serde_json::json!(sieve_url),
   269	                    );
   270	                })
   271	                .is_some();
   272	            if !env {
   273	                let obj = existing_settings
   274	                    .as_object_mut()
   275	                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
   276	                obj.insert(
   277	                    "env".to_string(),
   278	                    serde_json::json!({"ANTHROPIC_BASE_URL": sieve_url}),
   279	                );
   280	            }
   281	
   282	            // 追加 PreToolUse hook（幂等：已存在则跳过）
   283	            let hooks_obj = existing_settings
   284	                .get_mut("hooks")
   285	                .and_then(|v| v.as_object_mut());
   286	            if let Some(hooks) = hooks_obj {
   287	                let pre_tool = hooks
   288	                    .entry("PreToolUse")
   289	                    .or_insert_with(|| serde_json::json!([]));
   290	                if let Some(arr) = pre_tool.as_array_mut() {
   291	                    let already = arr.iter().any(|item| {
   292	                        item.pointer("/hooks/0/command")
   293	                            .and_then(|c| c.as_str())
   294	                            .map(|c| c.contains("sieve-hook"))
   295	                            .unwrap_or(false)
   296	                    });
   297	                    if !already {
   298	                        arr.push(hook_entry);
   299	                    }
   300	                }
   301	            } else {
   302	                let obj = existing_settings
   303	                    .as_object_mut()
   304	                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
   305	                obj.insert(
   306	                    "hooks".to_string(),
   307	                    serde_json::json!({"PreToolUse": [hook_entry]}),
   308	                );
   309	            }
   310	
   311	            // 确保父目录存在
   312	            if let Some(parent) = settings_path.parent() {
   313	                fs::create_dir_all(parent)?;
   314	            }
   315	            let json_str = serde_json::to_string_pretty(&existing_settings)?;
   316	            fs::write(settings_path, json_str.as_bytes()).context("写入 settings.json 失败")?;
   317	            ctx.written_files.push(settings_path.to_path_buf());
   318	            println!("[setup] ✅ settings.json 已更新");
   319	        }
   320	
   321	        // 写 launchd plist
   322	        {
   323	            if let Some(parent) = plist_path.parent() {
   324	                fs::create_dir_all(parent)?;
   325	            }
   326	            // 备份已有 plist
   327	            if plist_path.exists() {
   328	                let rel = plist_path.strip_prefix(home_path).unwrap_or(plist_path);
   329	                let backup_dest = backup_dir.join(rel);
   330	                if let Some(parent) = backup_dest.parent() {
   331	                    fs::create_dir_all(parent)?;
   332	                }
   333	                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
   334	            }
   335	            fs::write(plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
   336	            ctx.written_files.push(plist_path.to_path_buf());
   337	            println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());
   338	        }
   339	
   340	        // launchctl load
   341	        {
   342	            let status = Command::new("launchctl")
   343	                .args(["load", "-w", &plist_path.to_string_lossy()])
   344	                .status()
   345	                .context("执行 launchctl load 失败")?;
   346	            if !status.success() {
   347	                bail!("launchctl load 返回非零: {:?}", status.code());
   348	            }
   349	            ctx.launchd_loaded = Some(plist_path.to_path_buf());
   350	            println!("[setup] ✅ launchd 服务已加载");
   351	        }
   352	
   353	        // 写 setup.log
   354	        {
   355	            let entries: Vec<SetupLogEntry> = vec![
   356	                SetupLogEntry::new("setup_complete")
   357	                    .with_detail(format!("backup_dir={}", backup_dir.display())),
   358	                SetupLogEntry::new("settings_updated")
   359	                    .with_path(settings_path.to_string_lossy().to_string())
   360	                    .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse"),
   361	                SetupLogEntry::new("launchd_loaded")
   362	                    .with_path(plist_path.to_string_lossy().to_string()),
   363	            ];
   364	            let mut file = std::fs::OpenOptions::new()
   365	                .create(true)
   366	                .append(true)
   367	                .open(setup_log_path)
   368	                .context("打开 setup.log 失败")?;
   369	            for entry in &entries {
   370	                let line = serde_json::to_string(entry)? + "\n";
   371	                file.write_all(line.as_bytes())?;
   372	            }
   373	            println!("[setup] ✅ setup.log 写入 {}", setup_log_path.display());
   374	        }
   375	
   376	        Ok(())
   377	    }
   378	
   379	    /// 构建 launchd plist 内容（使用当前 sieve 二进制路径）。
   380	    fn build_plist_content() -> Result<String> {
   381	        let sieve_bin = std::env::current_exe().context("获取当前二进制路径失败")?;
   382	        let sieve_home =
   383	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
   384	        let log_path = sieve_home.join("daemon.log");
   385	        let err_path = sieve_home.join("daemon.err");
   386	
   387	        Ok(format!(
   388	            r#"<?xml version="1.0" encoding="UTF-8"?>
   389	<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
   390	  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   391	<plist version="1.0">
   392	<dict>
   393	  <key>Label</key>
   394	  <string>com.sieve.daemon</string>
   395	  <key>ProgramArguments</key>
   396	  <array>
   397	    <string>{bin}</string>
   398	    <string>start</string>
   399	  </array>
   400	  <key>RunAtLoad</key>
   401	  <true/>
   402	  <key>KeepAlive</key>
   403	  <true/>
   404	  <key>StandardOutPath</key>
   405	  <string>{log}</string>
   406	  <key>StandardErrorPath</key>
   407	  <string>{err}</string>
   408	</dict>
   409	</plist>
   410	"#,
   411	            bin = sieve_bin.display(),
   412	            log = log_path.display(),
   413	            err = err_path.display(),
   414	        ))
   415	    }
   416	
   417	    /// 简单去除 `// ...` 行注释（不处理字符串内的 `//`，够用于 settings.json）。
   418	    fn strip_json_comments(s: &str) -> String {
   419	        s.lines()
   420	            .map(|line| {
   421	                // 找到不在引号内的 `//`
   422	                let mut in_string = false;
   423	                let mut escaped = false;
   424	                let mut comment_start = None;
   425	                let chars: Vec<char> = line.chars().collect();
   426	                let mut i = 0;
   427	                while i < chars.len() {
   428	                    if escaped {
   429	                        escaped = false;
   430	                    } else if chars[i] == '\\' && in_string {
   431	                        escaped = true;
   432	                    } else if chars[i] == '"' {
   433	                        in_string = !in_string;
   434	                    } else if !in_string
   435	                        && chars[i] == '/'
   436	                        && i + 1 < chars.len()
   437	                        && chars[i + 1] == '/'
   438	                    {
   439	                        comment_start = Some(i);
   440	                        break;
   441	                    }
   442	                    i += 1;
   443	                }
   444	                if let Some(pos) = comment_start {
   445	                    line[..pos].to_string()
   446	                } else {
   447	                    line.to_string()
   448	                }
   449	            })
   450	            .collect::<Vec<_>>()
   451	            .join("\n")
   452	    }
   453	}
   454	
   455	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   456	
   457	#[cfg(not(target_os = "macos"))]
   458	mod stub {
   459	    use super::*;
   460	
   461	    /// `sieve setup` 非 macOS 占位实现。
   462	    /// Phase 1 仅支持 macOS；Linux/Windows 在 Phase 2 规划（ADR-015）。
   463	    pub fn run(_args: SetupArgs) -> Result<()> {
   464	        anyhow::bail!(
   465	            "sieve setup is macOS only in Phase 1. \
   466	             Linux/Windows support is planned for Phase 2."
   467	        )
   468	    }
   469	}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/main.rs | sed -n '70,130p' && nl -ba crates/sieve-cli/src/config.rs | sed -n '190,245p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
    70	            // 编译出站 vectorscan db（fail-closed）
    71	            let engine = VectorscanEngine::compile(rules.clone())
    72	                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
    73	            let adapter = OutboundAdapter::new(Arc::new(engine), rules);
    74	
    75	            // 加载 .sieveignore（出站 + 入站共用同一份）
    76	            let sieveignore_path = cfg.resolved_sieveignore_path();
    77	            let sieveignore = load_sieveignore(&sieveignore_path);
    78	            tracing::info!(
    79	                path = %sieveignore_path.display(),
    80	                entries = sieveignore.len(),
    81	                "sieveignore loaded"
    82	            );
    83	            let sieveignore_arc = Arc::new(sieveignore);
    84	
    85	            let filter = Arc::new(OutboundFilter::new(
    86	                Arc::new(adapter),
    87	                Arc::clone(&sieveignore_arc),
    88	            ));
    89	
    90	            // 加载入站规则（fail-closed，ADR-007）
    91	            let inbound_rules_path = cfg.resolved_inbound_rules_path();
    92	            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
    93	            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
    94	                format!(
    95	                    "failed to load inbound rules from {}; \
    96	                         set inbound_rules_path in sieve.toml or ensure the default path exists",
    97	                    inbound_rules_path.display()
    98	                )
    99	            })?;
   100	
   101	            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
   102	            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
   103	                .iter()
   104	                .cloned()
   105	                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
   106	            tracing::info!(
   107	                count = vectorscan_rules.len(),
   108	                placeholders = placeholder_rules.len(),
   109	                "inbound rules partitioned"
   110	            );
   111	
   112	            // 编译入站 vectorscan db（独立实例，fail-closed）
   113	            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
   114	                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;
   115	            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
   116	            let inbound_adapter =
   117	                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);
   118	
   119	            // YOLO mode 运行时审计（防御性双保险）
   120	            audit_yolo_disabled(&cfg)?;
   121	
   122	            daemon::run(
   123	                cfg,
   124	                filter,
   125	                Arc::new(inbound_adapter),
   126	                Arc::clone(&sieveignore_arc),
   127	            )
   128	            .await?;
   129	        }
   130	        Command::Version => {
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/main.rs | sed -n '1,75p' && nl -ba crates/sieve-cli/src/config.rs | sed -n '242,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
     1	//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
     2	//!
     3	//! 子命令：
     4	//! - `sieve start [--config <path>] [--dry-run]`：启动 daemon
     5	//! - `sieve version`：打印版本号
     6	//! - `sieve setup [--dry-run] [--yes]`：自动配置 Claude Code（仅 macOS，ADR-015）
     7	//! - `sieve doctor`：诊断 Sieve 安装状态（仅 macOS）
     8	//! - `sieve uninstall [--dry-run] [--yes]`：回滚 setup 改动（仅 macOS）
     9	
    10	#![forbid(unsafe_code)]
    11	
    12	use anyhow::{Context, Result};
    13	use clap::Parser;
    14	use std::collections::HashSet;
    15	use std::path::Path;
    16	use std::sync::Arc;
    17	
    18	mod audit;
    19	mod cli;
    20	mod commands;
    21	mod config;
    22	mod daemon;
    23	mod engine_adapter;
    24	
    25	use audit::AuditStore;
    26	use cli::{Cli, Command};
    27	use engine_adapter::{InboundAdapter, OutboundAdapter};
    28	use sieve_core::pipeline::outbound::OutboundFilter;
    29	use sieve_rules::engine::VectorscanEngine;
    30	use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
    31	
    32	#[tokio::main]
    33	async fn main() -> Result<()> {
    34	    init_tracing();
    35	
    36	    let cli = Cli::parse();
    37	
    38	    match cli.command {
    39	        Command::Start {
    40	            config: cfg_path,
    41	            dry_run: cli_dry_run,
    42	        } => {
    43	            let mut cfg = config::Config::load(&cfg_path)
    44	                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;
    45	
    46	            // CLI --dry-run 出现（true）时覆盖 config 中的值；
    47	            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
    48	            if cli_dry_run {
    49	                cfg.dry_run = true;
    50	            }
    51	
    52	            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)
    53	
    54	            let audit_path = cfg.audit_db_path()?;
    55	            let _audit = AuditStore::init(&audit_path)
    56	                .with_context(|| format!("init audit store at {}", audit_path.display()))?;
    57	
    58	            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
    59	            let rules_path = cfg.resolved_rules_path();
    60	            tracing::info!(path = %rules_path.display(), "loading outbound rules");
    61	            let rules = load_outbound_rules(&rules_path).with_context(|| {
    62	                format!(
    63	                    "failed to load outbound rules from {}; \
    64	                     set rules_path in sieve.toml or ensure the default path exists",
    65	                    rules_path.display()
    66	                )
    67	            })?;
    68	            tracing::info!(count = rules.len(), "outbound rules loaded");
    69	
    70	            // 编译出站 vectorscan db（fail-closed）
    71	            let engine = VectorscanEngine::compile(rules.clone())
    72	                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
    73	            let adapter = OutboundAdapter::new(Arc::new(engine), rules);
    74	
    75	            // 加载 .sieveignore（出站 + 入站共用同一份）
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
/bin/zsh -lc "nl -ba crates/sieve-hook/src/main.rs | sed -n '1,260p' && nl -ba crates/sieve-hook/src/lib.rs | sed -n '1,260p' && grep -R \"check\" -n crates/sieve-hook/src crates/sieve-cli/src/commands docs/specs/SPEC-001-sieve-hook-protocol.md | head -100" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
    19	use sieve_hook_lib::pending::read_pending_checked;
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
    53	    // 解析 request_id：flag > env。
    54	    let id_str = request_id
    55	        .or_else(|| std::env::var("SIEVE_REQUEST_ID").ok())
    56	        .unwrap_or_else(|| {
    57	            eprintln!("sieve-hook: SIEVE_REQUEST_ID not set and --request-id not provided");
    58	            std::process::exit(1);
    59	        });
    60	
    61	    let request_id = match Uuid::parse_str(&id_str) {
    62	        Ok(id) => id,
    63	        Err(e) => {
    64	            eprintln!("sieve-hook: invalid request ID `{id_str}`: {e}");
    65	            std::process::exit(1);
    66	        }
    67	    };
    68	
    69	    // 解析 sieve_home：flag > env > default。
    70	    let base = sieve_home
    71	        .or_else(|| std::env::var("SIEVE_HOME").ok().map(PathBuf::from))
    72	        .or_else(|| {
    73	            std::env::var("HOME")
    74	                .ok()
    75	                .map(|h| PathBuf::from(h).join(".sieve"))
    76	        })
    77	        .unwrap_or_else(|| {
    78	            eprintln!("sieve-hook: cannot determine sieve home directory ($HOME not set)");
    79	            std::process::exit(1);
    80	        });
    81	
    82	    std::process::exit(run(request_id, &base));
    83	}
    84	
    85	/// 核心逻辑，返回进程退出码（0 = 允许，1 = 拒绝）。
    86	///
    87	/// 关联：SPEC-001 §4（hook 决策流程）。
    88	fn run(request_id: Uuid, base: &std::path::Path) -> i32 {
    89	    let req = match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
    90	        Ok(r) => r,
    91	        Err(PendingError::NotFound) => {
    92	            // fail-open：Sieve 代理未标记此请求，放行。
    93	            return 0;
    94	        }
    95	        Err(PendingError::Stale) => {
    96	            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
    97	            return 1;
    98	        }
    99	        Err(PendingError::ParseError(e)) => {
   100	            eprintln!("sieve-hook: failed to parse pending file: {e}");
   101	            return 1;
   102	        }
   103	        Err(PendingError::IoError(e)) => {
   104	            eprintln!("sieve-hook: IO error reading pending file: {e}");
   105	            return 1;
   106	        }
   107	    };
   108	
   109	    // 打印危险摘要（SPEC-002 §2：多 issue 合并风格）。
   110	    print_summary(&req);
   111	
   112	    // 倒计时交互。
   113	    let outcome = prompt_user(&req);
   114	
   115	    // 写决策文件。
   116	    if let Err(e) = write_decision(request_id, &outcome, base) {
   117	        eprintln!("sieve-hook: failed to write decision: {e}");
   118	    }
   119	
   120	    match outcome {
   121	        DecisionOutcome::Allow => 0,
   122	        DecisionOutcome::Deny => 1,
   123	    }
   124	}
   125	
   126	/// 打印危险摘要到 stderr（TTY 终端显示）。
   127	///
   128	/// 关联：SPEC-002 §2.1（多 issue 合并显示）。
   129	fn print_summary(req: &protocol::DecisionRequest) {
   130	    let n = req.detections.len();
   131	    eprintln!();
   132	    eprintln!("┌─ Sieve 安全警告 ({n} 条检测) ────────────────────────────────");
   133	    for (i, det) in req.detections.iter().enumerate() {
   134	        let severity_tag = match det.severity.as_str() {
   135	            "critical" => "CRITICAL",
   136	            "high" => "HIGH    ",
   137	            "medium" => "MEDIUM  ",
   138	            _ => "LOW     ",
   139	        };
   140	        eprintln!(
   141	            "│ [{:2}] [{severity_tag}] {} — {}",
   142	            i + 1,
   143	            det.rule_id,
   144	            det.title
   145	        );
   146	        eprintln!("│       {}", det.one_line_summary);
   147	    }
   148	    eprintln!("└────────────────────────────────────────────────────────────");
   149	    eprintln!();
   150	}
   151	
   152	/// TTY 倒计时交互，返回用户决策。
   153	///
   154	/// - 输入 `y`/`Y` → Allow（exit 0）
   155	/// - 输入 `n`/`N`/回车（默认拒绝）→ Deny（exit 1）
   156	/// - 倒计时到 → 按 default_on_timeout 决定
   157	///
   158	/// 用 `spawn thread + mpsc channel` 实现非阻塞输入，避免引入 tokio。
   159	fn prompt_user(req: &protocol::DecisionRequest) -> DecisionOutcome {
   160	    let timeout = Duration::from_secs(req.timeout_seconds as u64);
   161	    let deadline = std::time::Instant::now() + timeout;
   162	
   163	    let stdin = io::stdin();
   164	    let (tx, rx) = std::sync::mpsc::channel::<String>();
   165	    std::thread::spawn(move || {
   166	        let mut line = String::new();
   167	        let _ = stdin.lock().read_line(&mut line);
   168	        let _ = tx.send(line);
   169	    });
   170	
   171	    loop {
   172	        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
   173	        eprint!(
   174	            "\r允许此操作？[y/N]（{} 秒后默认{}） > ",
   175	            remaining.as_secs(),
   176	            default_label(req.default_on_timeout)
   177	        );
   178	        let _ = io::stderr().flush();
   179	
   180	        match rx.recv_timeout(Duration::from_millis(100)) {
   181	            Ok(line) => {
   182	                eprintln!();
   183	                return match line.trim().to_lowercase().as_str() {
   184	                    "y" => DecisionOutcome::Allow,
   185	                    _ => DecisionOutcome::Deny,
   186	                };
   187	            }
   188	            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
   189	                if std::time::Instant::now() >= deadline {
   190	                    eprintln!();
   191	                    return match req.default_on_timeout {
   192	                        protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
   193	                        _ => DecisionOutcome::Deny,
   194	                    };
   195	                }
   196	            }
   197	            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
   198	                eprintln!();
   199	                return DecisionOutcome::Deny;
   200	            }
   201	        }
   202	    }
   203	}
   204	
   205	fn default_label(dot: protocol::DefaultOnTimeout) -> &'static str {
   206	    match dot {
   207	        protocol::DefaultOnTimeout::Allow => "允许",
   208	        _ => "拒绝",
   209	    }
   210	}
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
    14	use pending::read_pending_checked;
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
    59	#[cfg(test)]
    60	mod tests {
    61	    use chrono::{Duration, Utc};
    62	    use std::path::Path;
    63	    use uuid::Uuid;
    64	
    65	    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};
    66	
    67	    fn write_pending_json(base: &Path, req: &DecisionRequest) {
    68	        let dir = base.join("pending");
    69	        std::fs::create_dir_all(&dir).unwrap();
    70	        let json = serde_json::to_string_pretty(req).unwrap();
    71	        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
    72	    }
    73	
    74	    fn make_req(
    75	        id: Uuid,
    76	        dot: DefaultOnTimeout,
    77	        created_at: chrono::DateTime<Utc>,
    78	    ) -> DecisionRequest {
    79	        DecisionRequest {
    80	            request_id: id,
    81	            created_at,
    82	            timeout_seconds: 30,
    83	            default_on_timeout: dot,
    84	            detections: vec![],
    85	        }
    86	    }
    87	
    88	    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────
    89	
    90	    #[test]
    91	    fn pending_not_found_returns_0() {
    92	        let tmp = tempfile::tempdir().unwrap();
    93	        let id = Uuid::now_v7();
    94	        let code = super::run_check(id, tmp.path());
    95	        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
    96	    }
    97	
    98	    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────
    99	
   100	    #[test]
   101	    fn pending_stale_returns_1() {
   102	        let tmp = tempfile::tempdir().unwrap();
   103	        let id = Uuid::now_v7();
   104	        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
   105	        let stale_time = Utc::now() - Duration::minutes(11);
   106	        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
   107	        write_pending_json(tmp.path(), &req);
   108	        let code = super::run_check(id, tmp.path());
   109	        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
   110	    }
   111	
   112	    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────
   113	
   114	    #[test]
   115	    fn pending_parse_error_returns_1() {
   116	        let tmp = tempfile::tempdir().unwrap();
   117	        let id = Uuid::now_v7();
   118	        let dir = tmp.path().join("pending");
   119	        std::fs::create_dir_all(&dir).unwrap();
   120	        // 写入非法 JSON。
   121	        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
   122	        let code = super::run_check(id, tmp.path());
   123	        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
   124	    }
   125	
   126	    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────
   127	
   128	    #[test]
   129	    fn pending_allow_on_timeout_returns_0() {
   130	        let tmp = tempfile::tempdir().unwrap();
   131	        let id = Uuid::now_v7();
   132	        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
   133	        write_pending_json(tmp.path(), &req);
   134	        let code = super::run_check(id, tmp.path());
   135	        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
   136	    }
   137	
   138	    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────
   139	
   140	    #[test]
   141	    fn pending_block_on_timeout_returns_1() {
   142	        let tmp = tempfile::tempdir().unwrap();
   143	        let id = Uuid::now_v7();
   144	        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
   145	        write_pending_json(tmp.path(), &req);
   146	        let code = super::run_check(id, tmp.path());
   147	        assert_eq!(code, 1, "default_on_timeout=Block should return exit 1");
   148	    }
   149	
   150	    // ── Critical detection 记录的 decision.remember 永远 false ─────────────
   151	
   152	    #[test]
   153	    fn critical_decision_remember_is_false() {
   154	        let tmp = tempfile::tempdir().unwrap();
   155	        let id = Uuid::now_v7();
   156	        let req = DecisionRequest {
   157	            request_id: id,
   158	            created_at: Utc::now(),
   159	            timeout_seconds: 30,
   160	            default_on_timeout: DefaultOnTimeout::Allow,
   161	            detections: vec![DetectionPayload {
   162	                rule_id: "IN-CR-01".to_owned(),
   163	                severity: "critical".to_owned(),
   164	                disposition: "hook_terminal".to_owned(),
   165	                title: "Test".to_owned(),
   166	                one_line_summary: "test".to_owned(),
   167	                details: serde_json::Value::Null,
   168	            }],
   169	        };
   170	        write_pending_json(tmp.path(), &req);
   171	        super::run_check(id, tmp.path());
   172	
   173	        // 读取写入的 decision 文件，验证 remember=false。
   174	        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
   175	        let content = std::fs::read_to_string(dec_path).unwrap();
   176	        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
   177	        assert_eq!(resp["remember"], serde_json::Value::Bool(false));
   178	    }
   179	}
crates/sieve-hook/src/pending.rs:17:pub fn read_pending_checked(
crates/sieve-hook/src/lib.rs:14:use pending::read_pending_checked;
crates/sieve-hook/src/lib.rs:27:pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
crates/sieve-hook/src/lib.rs:28:    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
crates/sieve-hook/src/lib.rs:94:        let code = super::run_check(id, tmp.path());
crates/sieve-hook/src/lib.rs:108:        let code = super::run_check(id, tmp.path());
crates/sieve-hook/src/lib.rs:122:        let code = super::run_check(id, tmp.path());
crates/sieve-hook/src/lib.rs:134:        let code = super::run_check(id, tmp.path());
crates/sieve-hook/src/lib.rs:146:        let code = super::run_check(id, tmp.path());
crates/sieve-hook/src/lib.rs:171:        super::run_check(id, tmp.path());
crates/sieve-hook/src/main.rs:19:use sieve_hook_lib::pending::read_pending_checked;
crates/sieve-hook/src/main.rs:89:    let req = match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
crates/sieve-cli/src/commands/doctor.rs:5://! 2. hooks.PreToolUse 是否含 sieve-hook check
crates/sieve-cli/src/commands/doctor.rs:38:        let check1 = check_base_url(&settings_path);
crates/sieve-cli/src/commands/doctor.rs:39:        print_check(
crates/sieve-cli/src/commands/doctor.rs:41:            check1,
crates/sieve-cli/src/commands/doctor.rs:43:        all_ok &= check1;
crates/sieve-cli/src/commands/doctor.rs:46:        let check2 = check_hook_registered(&settings_path);
crates/sieve-cli/src/commands/doctor.rs:47:        print_check(
crates/sieve-cli/src/commands/doctor.rs:48:            "settings.json: hooks.PreToolUse 含 sieve-hook check",
crates/sieve-cli/src/commands/doctor.rs:49:            check2,
crates/sieve-cli/src/commands/doctor.rs:51:        all_ok &= check2;
crates/sieve-cli/src/commands/doctor.rs:54:        let check3 = check_daemon_listening();
crates/sieve-cli/src/commands/doctor.rs:55:        print_check("daemon 在 127.0.0.1:11453 监听", check3);
crates/sieve-cli/src/commands/doctor.rs:56:        all_ok &= check3;
crates/sieve-cli/src/commands/doctor.rs:59:        let check4 = check_launchd();
crates/sieve-cli/src/commands/doctor.rs:60:        print_check("launchd com.sieve.daemon 已加载", check4);
crates/sieve-cli/src/commands/doctor.rs:61:        all_ok &= check4;
crates/sieve-cli/src/commands/doctor.rs:64:        let check5 = check_canary();
crates/sieve-cli/src/commands/doctor.rs:65:        print_check("canary 拦截测试（OUT-01 脱敏）", check5);
crates/sieve-cli/src/commands/doctor.rs:66:        all_ok &= check5;
crates/sieve-cli/src/commands/doctor.rs:79:    fn print_check(label: &str, ok: bool) {
crates/sieve-cli/src/commands/doctor.rs:85:    fn check_base_url(path: &std::path::Path) -> bool {
crates/sieve-cli/src/commands/doctor.rs:98:    /// 检查 PreToolUse hook 是否含 sieve-hook check。
crates/sieve-cli/src/commands/doctor.rs:99:    fn check_hook_registered(path: &std::path::Path) -> bool {
crates/sieve-cli/src/commands/doctor.rs:120:    fn check_daemon_listening() -> bool {
crates/sieve-cli/src/commands/doctor.rs:131:    fn check_launchd() -> bool {
crates/sieve-cli/src/commands/doctor.rs:143:    fn check_canary() -> bool {
crates/sieve-cli/src/commands/setup.rs:150:            "hooks": [{"type": "command", "command": "sieve-hook check"}]
crates/sieve-cli/src/commands/setup.rs:180:            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
docs/specs/SPEC-001-sieve-hook-protocol.md:137:            "command": "sieve-hook check"
docs/specs/SPEC-001-sieve-hook-protocol.md:146:Claude Code 在每次执行 tool 之前 fork `sieve-hook check`，并通过环境变量传入上下文。

exec
/bin/zsh -lc "sed -n '120,190p' docs/specs/SPEC-001-sieve-hook-protocol.md && sed -n '1,120p' crates/sieve-hook/src/pending.rs && sed -n '1,120p' crates/sieve-hook/src/protocol.rs && sed -n '1,200p' crates/sieve-ipc/src/pending_file.rs && sed -n '1,200p' crates/sieve-ipc/src/protocol.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
---

## 5. sieve-hook 调用约定

### 5.1 Claude Code 注册方式

`sieve setup` 在 `~/.claude/settings.json` 写入：

```jsonc
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "sieve-hook check"
          }
        ]
      }
    ]
  }
}
```

Claude Code 在每次执行 tool 之前 fork `sieve-hook check`，并通过环境变量传入上下文。

### 5.2 环境变量

| 变量 | 来源 | 含义 |
|------|------|------|
| `SIEVE_REQUEST_ID` | Sieve 主代理（未来版本注入）| 当前请求的 UUIDv7 |
| `CLAUDE_TOOL_NAME` | Claude Code（标准 PreToolUse 环境）| 即将执行的 tool 名称 |
| `CLAUDE_TOOL_INPUT` | Claude Code | tool 参数 JSON（部分实现可能不暴露） |

**当 `SIEVE_REQUEST_ID` 不可用时的降级**：`sieve-hook` 扫描 `~/.sieve/pending/` 目录，取最新（按 UUIDv7 时间前缀排序）且未决策的文件，匹配 `CLAUDE_TOOL_NAME`。若多个文件均匹配，取最新一条。若无匹配，执行 §6 "pending 文件不存在"的路径。

### 5.3 TTY 弹窗行为

```
┌─────────────────────────────────────────────────┐
│ ⚠  Sieve 拦截：危险工具调用                     │
│ 规则：IN-CR-02 · 危险 bash                      │
│ 操作：rm -rf /tmp/data                          │
│                                                   │
│ 允许此次执行？[y/N]  30s 后自动拒绝              │
└─────────────────────────────────────────────────┘
```

- 默认答案：`N`（大写表示默认）
- 倒计时由 hook 自己维护（`Instant::now()` 轮询），不依赖代理时钟
- 用户按 `y` + Enter → `decision=allow`
- 用户按 `n` / Enter / Ctrl+C / 超时 → `decision=deny`
- 倒计时格式：每秒刷新同一行 `30s 后自动拒绝` → `29s 后自动拒绝` → …

### 5.4 exit code 语义

| exit code | 含义 | Claude Code 行为 |
|-----------|------|-----------------|
| `0` | allow（放行）| 继续执行 tool |
| `1` | deny（拒绝）| 取消 tool 执行，向用户报告被拦截 |

`sieve-hook` 在写完 decisions 文件后，按 decisions.decision 决定退出码（`allow → 0`，`deny → 1`）。

---

## 6. 错误处理

| 场景 | sieve-hook 行为 | 依据 |
|------|----------------|------|
use std::path::Path;

use uuid::Uuid;

use crate::{error::PendingError, protocol::DecisionRequest};

/// 读取并验证 pending 文件。
///
/// 返回：
/// - `Ok(DecisionRequest)` — 文件存在、未过期、解析成功
/// - `Err(PendingError::NotFound)` — 文件不存在（fail-open）
/// - `Err(PendingError::Stale)` — created_at 超过 `stale_threshold_secs`（fail-closed）
/// - `Err(PendingError::ParseError)` — JSON 解析失败（fail-closed）
/// - `Err(PendingError::IoError)` — 其他 IO 错误
///
/// 关联：SPEC-001 §4.2（stale 检测）。
pub fn read_pending_checked(
    request_id: Uuid,
    base: &Path,
    stale_threshold_secs: i64,
) -> Result<DecisionRequest, PendingError> {
    let path = base.join("pending").join(format!("{request_id}.json"));

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(PendingError::NotFound);
        }
        Err(e) => return Err(PendingError::IoError(e.to_string())),
    };

    let req: DecisionRequest =
        serde_json::from_str(&content).map_err(|e| PendingError::ParseError(e.to_string()))?;

    // stale 检测：created_at 超过阈值视为过期，fail-closed。
    let age_secs = chrono::Utc::now()
        .signed_duration_since(req.created_at)
        .num_seconds();
    if age_secs > stale_threshold_secs {
        return Err(PendingError::Stale);
    }

    Ok(req)
}
// hook 侧轻量协议结构体，与 sieve-ipc 的 protocol.rs 保持字段对齐，
// 但独立定义避免 tokio/tracing 等依赖拖入二进制。
//
// 关联：SPEC-001 §3（文件协议 schema）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    Redact,
    Block,
    Allow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPayload {
    pub rule_id: String,
    pub severity: String,
    pub disposition: String,
    pub title: String,
    pub one_line_summary: String,
    pub details: serde_json::Value,
}

/// 与 sieve-ipc 的 DecisionRequest 字段完全对齐，用于反序列化 pending 文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub request_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub timeout_seconds: u32,
    pub default_on_timeout: DefaultOnTimeout,
    pub detections: Vec<DetectionPayload>,
}

/// 写入 decisions/<id>.json 的结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub request_id: Uuid,
    pub decision: String, // "allow" | "deny"
    pub decided_at: DateTime<Utc>,
    pub by_user: bool,
    pub remember: bool,
}
use std::path::{Path, PathBuf};

use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{ensure_dirs, pending_dir},
    protocol::DecisionRequest,
};

/// 将 [`DecisionRequest`] 写入 `<base>/pending/<request_id>.json`。
///
/// 写入前用 fd-lock 对目标文件加独占写锁，防止并发写入同一 request_id（极少见
/// 但理论可行）。文件以 pretty JSON 格式写入，方便调试和 hook 侧直接读取。
///
/// 关联：SPEC-001 §3.1（pending 文件写入规约）、ADR-013 §4（文件协议备用路径）。
pub fn write_pending(req: &DecisionRequest, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let dir = pending_dir(base);
    let path = dir.join(format!("{}.json", req.request_id));

    // 打开（或创建）文件，然后加独占写锁再写内容。
    // 使用 std::fs::OpenOptions 而非 std::fs::write，以便 fd-lock 持有文件描述符。
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    let mut lock = RwLock::new(file);
    {
        let mut guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;
        let json = serde_json::to_string_pretty(req)?;
        use std::io::Write;
        guard.write_all(json.as_bytes())?;
    }

    Ok(path)
}

/// 读取并解析 `<base>/pending/<request_id>.json`。
///
/// 返回：
/// - `Ok(DecisionRequest)` 成功
/// - `Err(IpcError::PendingNotFound)` 文件不存在
/// - `Err(IpcError::Json)` 解析失败
pub fn read_pending(request_id: Uuid, base: &Path) -> Result<DecisionRequest, IpcError> {
    let path = pending_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let req: DecisionRequest = serde_json::from_str(&content)?;
    Ok(req)
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Enums ────────────────────────────────────────────────────────────────────

/// 检测结果的最终处置方式。
///
/// 与 sieve-rules 中的处置枚举镜像，IPC 层独立定义以避免循环依赖。
/// 关联：ADR-014（双层防御）、SPEC-001。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// 自动脱敏——出站阶段替换敏感内容后放行，无需人工确认。
    AutoRedact,
    /// 弹出 GUI 窗口（sieve-gui-macos）请求用户确认。
    GuiPopup,
    /// 调用 PreToolUse hook（sieve-hook 二进制）在 TTY 请求用户确认。
    HookTerminal,
    /// 在状态栏静默提示，不打断流程。
    StatusBar,
}

/// 超时后的默认决策。
///
/// Critical 规则强制使用 Block，不允许下游覆盖。关联：ADR-014 §fail-closed。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    /// 脱敏后放行（适用于 AutoRedact 类型的超时回退）。
    Redact,
    /// 阻断——fail-closed，Critical 规则的强制回退策略。
    Block,
    /// 放行——仅适用于低优先级通知类规则。
    Allow,
}

/// 检测命中的严重等级。
///
/// 关联：PRD §4 检测项分级、ADR-014。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// 最高级：签名、转账、部署等不可逆动作，强制人工确认，不可关闭。
    Critical,
    /// 高危：可逆但高风险操作。
    High,
    /// 中等：潜在风险，默认提示但可配置。
    Medium,
    /// 低危：信息提示。
    Low,
}

// ── Detection payload ────────────────────────────────────────────────────────

/// 单条检测命中的 IPC 表示。
///
/// 去掉规则匹配内部细节（正则 / offset），只保留 GUI/hook 渲染所需字段。
/// 关联：SPEC-001 §3.2、SPEC-002 §2.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPayload {
    /// 规则 ID，例如 `IN-CR-01`。用于 hook 终端显示和日志关联。
    pub rule_id: String,
    /// 严重等级。
    pub severity: Severity,
    /// 处置方式。
    pub disposition: Disposition,
    /// 简短标题，在 GUI 标题栏或 hook 首行显示。
    pub title: String,
    /// 单行摘要，不超过 120 字符，用于 hook 终端和通知消息。
    pub one_line_summary: String,
    /// 扩展详情，结构由各规则自定义（GUI 侧渲染详细视图用）。
    pub details: serde_json::Value,
}

// ── Request / Response ───────────────────────────────────────────────────────

/// 主代理 → GUI / Hook 的决策请求。
///
/// JSON-RPC 2.0 method = `"request_decision"`，通过 Unix socket 或 pending
/// 文件协议传输。关联：ADR-013 §3、SPEC-001 §3.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    /// 全局唯一请求 ID（UUIDv7，含时间戳，便于排序和 stale 检测）。
    pub request_id: Uuid,
    /// 请求创建时间（UTC）。hook 侧用于 stale 检测（> 10 分钟视为过期）。
    pub created_at: DateTime<Utc>,
    /// 用户响应超时时长（秒）。范围 30–120，由规则配置决定。
    pub timeout_seconds: u32,
    /// 超时后的默认决策。Critical 规则此字段服务端强制为 `Block`。
    pub default_on_timeout: DefaultOnTimeout,
    /// 本次请求触发的所有检测命中列表（可多条）。
    pub detections: Vec<DetectionPayload>,
}

/// 用户或超时产生的决策动作。
///
/// 关联：SPEC-001 §3.3、ADR-014 §决策流程。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    /// 用户允许：GUI 类继续转发原始 SSE，Hook 类返回 exit 0。
    Allow,
    /// 用户拒绝：GUI 类截流注入 `sieve_blocked` event，Hook 类返回 exit 1。
    Deny,
    /// 仅出站脱敏类：按规则 redact 占位符替换后转发。
    RedactAndAllow,
}

/// GUI / Hook → 主代理的决策响应。
///
/// 写入 `<sieve_home>/decisions/<request_id>.json` 或通过 socket 返回。
/// 关联：ADR-013 §3.4、SPEC-001 §3.3。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    /// 对应的请求 ID，用于主代理侧匹配 oneshot channel。
    pub request_id: Uuid,
    /// 决策动作。
    pub decision: DecisionAction,
    /// 决策时间（UTC）。
    pub decided_at: DateTime<Utc>,
    /// `true` 表示用户主动操作，`false` 表示超时默认。
    pub by_user: bool,
    /// 是否记住此次决策（同规则 + 同 tool 不再询问）。
    ///
    /// Critical severity 的决策此字段服务端强制写 `false`，即使用户请求记住也拒绝。
    pub remember: bool,
}

// ── JSON-RPC 2.0 envelope ────────────────────────────────────────────────────

/// JSON-RPC 2.0 协议封装。
///
/// 手写实现以避免引入大型 jsonrpc crate 依赖。关联：ADR-013 §2（传输协议选型）。
pub mod jsonrpc {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// JSON-RPC 2.0 请求（通知或有 id 的调用）。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub jsonrpc: String,
        pub method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub params: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<Value>,
    }

    impl Request {
        /// 构造一个有 id 的调用请求。
        pub fn call(method: impl Into<String>, params: Value, id: Value) -> Self {
            Self {
                jsonrpc: "2.0".to_owned(),
                method: method.into(),
                params: Some(params),
                id: Some(id),
            }
        }
    }

    /// JSON-RPC 2.0 成功响应。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<ErrorObject>,
        pub id: Value,
    }

    /// JSON-RPC 2.0 错误对象。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ErrorObject {
        pub code: i64,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
    }
}

exec
/bin/zsh -lc "cat crates/sieve-cli/tests/sieve_setup_dry_run.rs && sed -n '1,200p' crates/sieve-cli/tests/audit_append_only.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! `sieve setup --dry-run` 集成测试（SPEC-003 §setup）。
//!
//! 仅 macOS 编译运行（`#[cfg(target_os = "macos")]`）。
//! 验证：dry-run 模式不修改原文件。

#![cfg(target_os = "macos")]

use std::fs;
use tempfile::tempdir;

/// 构造一个 fake `settings.json` 并运行 setup --dry-run；
/// 断言原文件内容未被改动。
#[test]
fn dry_run_does_not_modify_settings() {
    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();

    // 建 .claude/ 目录和 fake settings.json
    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let settings_path = claude_dir.join("settings.json");
    let original_content = r#"{"env": {"SOME_KEY": "some_value"}}"#;
    fs::write(&settings_path, original_content).unwrap();

    // 也建 .sieve/ 目录（setup 需要写 setup.log）
    let sieve_dir = fake_home.join(".sieve");
    fs::create_dir_all(&sieve_dir).unwrap();

    // 设置 HOME 为 fake_home，SIEVE_HOME 为 fake .sieve
    // 注意：因为 setup 内部直接读 HOME env var，我们通过子进程的方式测试
    // 这里使用 std::env::set_var（测试环境可接受）
    let orig_home = std::env::var("HOME").unwrap_or_default();
    let orig_sieve_home = std::env::var("SIEVE_HOME").unwrap_or_default();

    // SAFETY: 仅在单线程测试环境中修改 env var
    unsafe {
        std::env::set_var("HOME", fake_home.to_str().unwrap());
        std::env::set_var("SIEVE_HOME", sieve_dir.to_str().unwrap());
    }

    // 调用 setup::run（dry_run=true, yes=true）
    // dry-run 应该不修改任何文件
    // 由于 setup::run 会调用 `launchctl` 等系统命令，这里仅验证核心逻辑：
    // dry-run 模式下 settings.json 不变
    //
    // 注意：完整 setup 调用会因为 launchctl 不在 CI 测试环境中工作而失败；
    // 这里我们直接测试文件不变的断言（dry-run 在打印 diff 后直接 return）
    use sieve_cli_test_helpers::*;
    let result = run_setup_dry_run(&settings_path);

    // 恢复 env var
    unsafe {
        std::env::set_var("HOME", &orig_home);
        if orig_sieve_home.is_empty() {
            std::env::remove_var("SIEVE_HOME");
        } else {
            std::env::set_var("SIEVE_HOME", &orig_sieve_home);
        }
    }

    // 无论 setup 是否返回错误，settings.json 必须保持原始内容
    let actual_content = fs::read_to_string(&settings_path).unwrap();
    assert_eq!(
        actual_content, original_content,
        "dry-run 模式不应修改 settings.json"
    );

    // dry-run 应该成功（无需 launchctl）
    result.expect("setup --dry-run 应该成功");
}

/// 包含测试辅助函数的伪模块（直接内联，避免额外文件）。
mod sieve_cli_test_helpers {
    use anyhow::Result;
    use std::path::Path;

    /// 直接调用 setup::run with dry_run=true, yes=true。
    pub fn run_setup_dry_run(_settings_path: &Path) -> Result<()> {
        // 直接引用 sieve-cli 内部模块（integration test 与 lib 共享 crate）
        // 集成测试通过 cargo test 链接到 sieve-cli 的库部分
        // 由于 sieve-cli 是 binary crate（只有 main.rs），这里用 Command 子进程测试
        // 实际验证逻辑：dry-run 不修改文件（已在调用方验证）
        Ok(())
    }
}
//! `AuditStore` append-only 集成测试（ADR-007 / ADR-014）。
//!
//! 验证：写 3 条 → SELECT 能读到；UPDATE / DELETE 被触发器拒绝。
//!
//! 注意：由于 sieve-cli 是纯 binary crate，这里通过子进程或直接用 rusqlite 验证。
//! audit.rs 中已有 `#[cfg(test)]` 单元测试覆盖同等逻辑；本集成测试作为补充验证。

use rusqlite::{params, Connection};
use tempfile::tempdir;

const CREATE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_rfc3339   TEXT    NOT NULL,
    direction           TEXT    NOT NULL,
    rule_id             TEXT    NOT NULL,
    severity            TEXT    NOT NULL,
    disposition         TEXT    NOT NULL,
    decision            TEXT,
    request_id          TEXT    NOT NULL,
    raw_json            TEXT
);
"#;

const TRIGGERS_DDL: &str = r#"
CREATE TRIGGER IF NOT EXISTS no_update
BEFORE UPDATE ON audit_events
BEGIN
    SELECT RAISE(FAIL, 'audit_events is append-only: UPDATE is forbidden');
END;

CREATE TRIGGER IF NOT EXISTS no_delete
BEFORE DELETE ON audit_events
BEGIN
    SELECT RAISE(FAIL, 'audit_events is append-only: DELETE is forbidden');
END;
"#;

const INSERT_SQL: &str = r#"
INSERT INTO audit_events
    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#;

fn setup_db(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(CREATE_DDL).unwrap();
    conn.execute_batch(TRIGGERS_DDL).unwrap();
    conn
}

#[test]
fn write_3_events_and_read_back() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_audit.db");
    let conn = setup_db(&db_path);

    for i in 1..=3u32 {
        conn.execute(
            INSERT_SQL,
            params![
                format!("2026-04-27T00:0{i}:00Z"),
                "outbound",
                format!("OUT-0{i}"),
                "Critical",
                "redact",
                Option::<String>::None,
                format!("req-{i}"),
                Option::<String>::None,
            ],
        )
        .unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3, "应有 3 条记录");

    let rule_id: String = conn
        .query_row("SELECT rule_id FROM audit_events WHERE id = 2", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rule_id, "OUT-02");
}

#[test]
fn update_is_rejected_by_trigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_trigger_update.db");
    let conn = setup_db(&db_path);

    conn.execute(
        INSERT_SQL,
        params![
            "2026-04-27T00:00:00Z",
            "inbound",
            "IN-CR-01",
            "Critical",
            "pending",
            Option::<String>::None,
            "req-x",
            Option::<String>::None,
        ],
    )
    .unwrap();

    let result = conn.execute(
        "UPDATE audit_events SET rule_id = 'tampered' WHERE id = 1",
        [],
    );
    assert!(result.is_err(), "UPDATE 应被触发器拒绝");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("append-only"),
        "错误信息应含 'append-only'，实际: {msg}"
    );
}

#[test]
fn delete_is_rejected_by_trigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_trigger_delete.db");
    let conn = setup_db(&db_path);

    conn.execute(
        INSERT_SQL,
        params![
            "2026-04-27T00:00:00Z",
            "inbound",
            "IN-CR-02",
            "Critical",
            "resolved",
            Some("Block"),
            "req-y",
            Option::<String>::None,
        ],
    )
    .unwrap();

    let result = conn.execute("DELETE FROM audit_events WHERE id = 1", []);
    assert!(result.is_err(), "DELETE 应被触发器拒绝");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("append-only"),
        "错误信息应含 'append-only'，实际: {msg}"
    );
}

exec
/bin/zsh -lc 'git diff -- crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/engine/mod.rs crates/sieve-rules/tests/inbound_rules.rs crates/sieve-rules/tests/outbound_rules.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
/bin/zsh -lc 'cat crates/sieve-rules/rules/outbound.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
# OUT-01~12 出站 P0 规则集
#
# 关联 PRD §5.1 出站检测目标。
# 来源: gitleaks/gitleaks config/gitleaks.toml (MIT License) + Sieve 自写
#
# 重要约束: 所有 pattern 必须为 vectorscan 兼容 PCRE 子集。
# 禁止使用: lookahead(?=...) / lookbehind(?<=...) / 反向引用\1 / 原子组(?>...) / \C \R \K

# ---------------------------------------------------------------------------
# OUT-01: Anthropic API Key
# 格式: sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" 结尾
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-01"
description = "Anthropic API key (sk-ant-api03-...)"
pattern = 'sk-ant-api03-[a-zA-Z0-9_\-]{93}AA'
severity = "critical"
action = "block"
entropy_min = 4.5
keywords = ["sk-ant-api03"]
allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-02: OpenAI API Key
# 新格式: sk-proj-/sk-svcacct-/sk-admin- + 58~200 char + T3BlbkFJ + 58~200 char
# 旧格式: sk- + 20 alnum + T3BlbkFJ + 20 alnum
# 两种格式用 alternation 合并，无 lookahead
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-02"
description = "OpenAI API key (sk-... legacy + sk-proj-/sk-svcacct-/sk-admin-)"
pattern = 'sk-(?:proj|svcacct|admin)-[A-Za-z0-9_\-]{58,65}T3BlbkFJ[A-Za-z0-9_\-]{58,65}|sk-[a-zA-Z0-9]{20}T3BlbkFJ[a-zA-Z0-9]{20}'
severity = "critical"
action = "block"
entropy_min = 4.5
keywords = ["T3BlbkFJ"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-03: AWS Access Key ID
# 前缀: A3T[A-Z0-9] / AKIA / ASIA / ABIA / ACCA，后跟 16 个 base32 字符
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-03"
description = "AWS Access Key ID (AKIA / ASIA / ABIA / ACCA / A3T)"
pattern = '(?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16}'
severity = "critical"
action = "block"
entropy_min = 3.0
keywords = ["AKIA", "ASIA", "ABIA", "ACCA"]
allowlist_regexes = []
allowlist_stopwords = ["AKIAIOSFODNN7EXAMPLE"]  # AWS 官方文档示例 key
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-04: GitHub Personal Access Token
# 格式: ghp_/gho_/ghu_/ghs_/ghr_ + 36 个 alnum 字符
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-04"
description = "GitHub PAT (ghp_/gho_/ghu_/ghs_/ghr_)"
pattern = 'gh[pousr]_[0-9a-zA-Z]{36}'
severity = "critical"
action = "block"
entropy_min = 4.0
keywords = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-05: Google Cloud API Key
# 格式: AIza + 35 个 alnum/_/-
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-05"
description = "Google Cloud API Key (AIza...)"
pattern = 'AIza[0-9A-Za-z_\-]{35}'
severity = "high"
action = "block"
entropy_min = 4.0
keywords = ["AIza"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-06: JWT Token
# 格式: eyJ... (header.payload.signature，三段均为 base64url)
# 不用 lookahead，直接匹配三段结构
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-06"
description = "JWT Token (eyJ...)"
pattern = 'ey[A-Za-z0-9_\-]{16,}\.ey[A-Za-z0-9_\/\-]{16,}\.[A-Za-z0-9_\/\-]{10,}'
severity = "high"
action = "block"
entropy_min = 3.5
keywords = ["eyJ"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 15
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-07: PEM Private Key Header
# 覆盖: RSA / EC / DSA / PKCS#8 / generic PRIVATE KEY 头部
# 注意: 不包含 OPENSSH（由 OUT-10 专项覆盖）
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-07"
description = "PEM Private Key (RSA / EC / DSA / generic)"
pattern = '-----BEGIN[ A-Z0-9_\-]{0,60}PRIVATE KEY[ A-Z]{0,20}-----'
severity = "critical"
action = "block"
entropy_min = 0.0
keywords = ["-----BEGIN"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 60
default_on_timeout = "block"

# ---------------------------------------------------------------------------
# OUT-08: Stripe Live Secret / Publishable / Restricted Key
# 格式: sk_live_/pk_live_/rk_live_ + 10~99 alnum
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-08"
description = "Stripe Live Key (sk_live_/pk_live_/rk_live_)"
pattern = '(?:sk|pk|rk)_live_[a-zA-Z0-9]{10,99}'
severity = "critical"
action = "block"
entropy_min = 3.5
keywords = ["_live_"]
allowlist_regexes = ['(?i)test|example']
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 15
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-09: Slack Token
# 格式: xoxb-/xoxp-/xoxa-/xoxs- + 10+ alnum/-
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-09"
description = "Slack Token (xoxb-/xoxp-/xoxa-/xoxs-)"
pattern = 'xox[bpas]\-[0-9A-Za-z\-]{10,}'
severity = "high"
action = "block"
entropy_min = 3.0
keywords = ["xoxb", "xoxp", "xoxa", "xoxs"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 60
default_on_timeout = "block"

# ---------------------------------------------------------------------------
# OUT-10: OpenSSH Private Key Header
# 专项规则，不依赖 OUT-07 的通用 PEM 规则
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-10"
description = "OpenSSH Private Key (-----BEGIN OPENSSH PRIVATE KEY-----)"
pattern = '-----BEGIN OPENSSH PRIVATE KEY-----'
severity = "critical"
action = "block"
entropy_min = 0.0
keywords = ["BEGIN OPENSSH"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 60
default_on_timeout = "block"

# ---------------------------------------------------------------------------
# OUT-11: Discord Bot Token
# 格式: 24~28 base64url . 6 base64url . 27~38 base64url
# 三段由英文句号分隔
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-11"
description = "Discord Bot Token"
pattern = '[A-Za-z0-9_\-]{24,28}\.[A-Za-z0-9_\-]{6}\.[A-Za-z0-9_\-]{27,38}'
severity = "high"
action = "block"
entropy_min = 3.5
keywords = ["."]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "status_bar"

# ---------------------------------------------------------------------------
# OUT-09（BIP39 助记词）在 engine_adapter 中通过 second-pass 实现，
# 不使用 vectorscan 占位规则。
# 详见 crates/sieve-cli/src/engine_adapter.rs OutboundAdapter::scan_text。
# 关联 PRD §9 #4 差异化点：SHA-256 checksum 验证在 second-pass 完成。
# ---------------------------------------------------------------------------

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/engine_adapter.rs | sed -n '140,190p' && nl -ba crates/sieve-cli/src/engine_adapter.rs | sed -n '236,272p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   140	        body_offset: usize,
   141	    ) -> SieveCoreResult<Vec<Detection>> {
   142	        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
   143	            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
   144	        })?;
   145	
   146	        let mut detections = Vec::new();
   147	        for hit in hits {
   148	            let rule = self.rule_lookup.get(&hit.rule_id);
   149	
   150	            let evidence_start = hit.start.min(input.len());
   151	            let evidence_end = hit.end.min(input.len());
   152	            let matched_text = &input[evidence_start..evidence_end];
   153	
   154	            if let Some(r) = rule {
   155	                if self.engine.is_excluded(matched_text, r) {
   156	                    continue;
   157	                }
   158	            }
   159	
   160	            let severity = rule
   161	                .map(|r| map_severity(r.severity))
   162	                .unwrap_or(Severity::Critical);
   163	
   164	            // critical_lock 强制：fail-closed 规则 action 一律覆盖为 Block
   165	            let raw_action = rule.map(|r| r.action).unwrap_or(RulesAction::Block);
   166	            let enforced_action =
   167	                sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw_action);
   168	            // v1.4：disposition 路由优先于 raw action 映射
   169	            let action = if enforced_action == RulesAction::Block {
   170	                // fail-closed 强制 Block，跳过 disposition 路由
   171	                Action::Block
   172	            } else {
   173	                rule.map(|r| {
   174	                    let disp = r.effective_disposition();
   175	                    let timeout = r.timeout_seconds.unwrap_or(60);
   176	                    map_action_by_disposition(disp, enforced_action, &hit.rule_id, timeout)
   177	                })
   178	                .unwrap_or_else(|| map_action(enforced_action))
   179	            };
   180	
   181	            let evidence_truncated = redact_evidence(matched_text);
   182	            let fp = fingerprint(&hit.rule_id, matched_text);
   183	
   184	            detections.push(Detection {
   185	                id: Uuid::new_v4(),
   186	                rule_id: hit.rule_id.clone(),
   187	                severity,
   188	                action,
   189	                source,
   190	                span: ContentSpan {
   236	
   237	        let mut detections = Vec::new();
   238	        for hit in hits {
   239	            let rule = self.rule_lookup.get(&hit.rule_id);
   240	
   241	            // per-rule allowlist 过滤
   242	            let evidence_start = hit.start.min(input.len());
   243	            let evidence_end = hit.end.min(input.len());
   244	            let matched_text = &input[evidence_start..evidence_end];
   245	
   246	            if let Some(r) = rule {
   247	                if self.engine.is_excluded(matched_text, r) {
   248	                    continue;
   249	                }
   250	            }
   251	
   252	            let severity = rule
   253	                .map(|r| map_severity(r.severity))
   254	                .unwrap_or(Severity::Critical);
   255	            // v1.4：disposition 路由
   256	            let action = rule
   257	                .map(|r| {
   258	                    let raw = r.action;
   259	                    let enforced = sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw);
   260	                    if enforced == RulesAction::Block {
   261	                        Action::Block
   262	                    } else {
   263	                        let disp = r.effective_disposition();
   264	                        let timeout = r.timeout_seconds.unwrap_or(60);
   265	                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
   266	                    }
   267	                })
   268	                .unwrap_or(Action::Block);
   269	            let evidence_truncated = redact_evidence(matched_text);
   270	            let fp = fingerprint(&hit.rule_id, matched_text);
   271	
   272	            detections.push(Detection {

exec
/bin/zsh -lc "sed -n '1,360p' crates/sieve-cli/src/engine_adapter.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` /
//! `sieve_core::InboundEngine` trait。
//!
//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，
//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
//!
//! 关联 ADR-002 / PRD §5.1 / Week 2 出站 / Week 3 入站拦截集成。

use sieve_core::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use sieve_core::error::SieveCoreResult;
use sieve_core::pipeline::inbound::InboundEngine;
use sieve_core::pipeline::outbound::OutboundEngine;
use sieve_core::protocol::unified_message::ContentSpan;
use sieve_core::tool_use_aggregator::CompletedToolCall;
use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// `VectorscanEngine` 包装，实现 `sieve_core::OutboundEngine`。
///
/// 内部持有规则反查表（`rule_id → RuleEntry`），用于从 `MatchHit` 取真实 severity/action。
pub struct OutboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表，用于从 MatchHit 映射元数据。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl OutboundAdapter {
    /// 构造 adapter。
    ///
    /// `rules` 与 `VectorscanEngine::compile` 传入的规则集一致，用于构建反查表。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

/// 把 `sieve_rules::Severity` 映射为 `sieve_core::Severity`。
fn map_severity(r: RulesSeverity) -> Severity {
    match r {
        RulesSeverity::Low => Severity::Low,
        RulesSeverity::Medium => Severity::Medium,
        RulesSeverity::High => Severity::High,
        RulesSeverity::Critical => Severity::Critical,
    }
}

/// 根据 `RuleEntry.disposition` 和 `RulesAction` 映射为 `sieve_core::Action`。
///
/// v1.4 重构：优先按 `effective_disposition()` 路由，`RulesAction` 作为兜底。
///
/// | Disposition       | Action                                       |
/// |-------------------|----------------------------------------------|
/// | AutoRedact        | `Redact { placeholder }`                     |
/// | GuiPopup          | `HoldForDecision { request_id, timeout_s }`  |
/// | HookTerminal      | `HookMark`                                   |
/// | StatusBar         | `MarkOnly`                                   |
///
/// `timeout_seconds` / `default_on_timeout` 取自 `RuleEntry`，不再硬编码 5。
///
/// 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
fn map_action_by_disposition(
    disposition: sieve_rules::manifest::Disposition,
    _rule_action: RulesAction,
    rule_id: &str,
    timeout_seconds: u32,
) -> Action {
    use sieve_rules::manifest::Disposition;
    match disposition {
        Disposition::AutoRedact => Action::Redact {
            placeholder: format!("[REDACTED:{rule_id}]"),
        },
        Disposition::GuiPopup => Action::HoldForDecision {
            request_id: uuid::Uuid::new_v4(),
            timeout_seconds,
        },
        Disposition::HookTerminal => Action::HookMark,
        Disposition::StatusBar => Action::MarkOnly,
    }
}

/// 旧接口：仅用 `RulesAction` 映射（兜底，无 disposition 信息时使用）。
///
/// `Warn` → `HookMark`（v1.4 后 Warn 一律走 HookTerminal 路径）。
fn map_action(r: RulesAction) -> Action {
    match r {
        RulesAction::Block => Action::Block,
        RulesAction::Warn => Action::HookMark,
        RulesAction::Mark => Action::MarkOnly,
        RulesAction::Allow => Action::SilentLog,
    }
}

/// 截断并脱敏证据片段（用于 `Detection.evidence_truncated`）。
///
/// 超过 8 字符时，保留前 4 + `***` + 后 4，防止原始密钥写入审计日志。
fn redact_evidence(matched: &str) -> String {
    let chars: Vec<char> = matched.chars().collect();
    let len = chars.len();
    if len <= 8 {
        "*".repeat(len)
    } else {
        let head: String = chars[..4].iter().collect();
        let tail: String = chars[len - 4..].iter().collect();
        format!("{head}***{tail}")
    }
}

/// `VectorscanEngine` 包装，实现 `sieve_core::InboundEngine`。
///
/// 与 [`OutboundAdapter`] 共用辅助函数（`map_severity` / `map_action` / `redact_evidence`），
/// 额外在工具调用检查中调用 `sieve_rules::critical_lock::enforce_action` 保证 fail-closed。
pub struct InboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl InboundAdapter {
    /// 构造 adapter。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

impl InboundEngine for InboundAdapter {
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);

            // critical_lock 强制：fail-closed 规则 action 一律覆盖为 Block
            let raw_action = rule.map(|r| r.action).unwrap_or(RulesAction::Block);
            let enforced_action =
                sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw_action);
            // v1.4：disposition 路由优先于 raw action 映射
            let action = if enforced_action == RulesAction::Block {
                // fail-closed 强制 Block，跳过 disposition 路由
                Action::Block
            } else {
                rule.map(|r| {
                    let disp = r.effective_disposition();
                    let timeout = r.timeout_seconds.unwrap_or(60);
                    map_action_by_disposition(disp, enforced_action, &hit.rule_id, timeout)
                })
                .unwrap_or_else(|| map_action(enforced_action))
            };

            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_offset + hit.start,
                    end: body_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }
        Ok(detections)
    }

    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();
        // 1. 工具名扫描（IN-CR-05 签名工具）
        hits.extend(self.scan_text(&tool.name, source, 0)?);
        // 2. 工具输入序列化扫描（IN-CR-02 危险 shell 等）
        if let Ok(input_str) = serde_json::to_string(&tool.input) {
            hits.extend(self.scan_text(&input_str, source, 0)?);
        }
        Ok(hits)
    }
}

impl OutboundEngine for OutboundAdapter {
    /// 扫描文本，返回已过滤（per-rule allowlist）的命中列表，并执行 BIP39 second-pass。
    ///
    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
    ///
    /// BIP39 second-pass（PRD §9 #4）：vectorscan 之后独立扫描。
    /// 先提取全部在词表的连续词窗口，再做 SHA-256 checksum 验证，
    /// **仅 checksum 通过才生成 Critical Detection**。
    /// 词表命中但 checksum 失败的窗口**不得**定级 Critical（差异化要求）。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_byte_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            // per-rule allowlist 过滤
            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);
            // v1.4：disposition 路由
            let action = rule
                .map(|r| {
                    let raw = r.action;
                    let enforced = sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw);
                    if enforced == RulesAction::Block {
                        Action::Block
                    } else {
                        let disp = r.effective_disposition();
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                    }
                })
                .unwrap_or(Action::Block);
            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_byte_offset + hit.start,
                    end: body_byte_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }

        // BIP39 second-pass（关联 PRD §9 #4 差异化点）
        // vectorscan 不覆盖 BIP39，此处独立扫描：
        // 1. 按空白分词，提取全在词表的连续窗口
        // 2. 对每个窗口做 SHA-256 checksum 验证
        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
        let wl = sieve_rules::wordlist::wordlist_index();
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
        for window in candidates {
            if sieve_rules::bip39::verify_checksum(&window, wl) {
                let window_text = window.join(" ");
                let evidence_truncated = redact_evidence(&window_text);
                let fp = fingerprint("OUT-09", &window_text);
                detections.push(Detection {
                    id: Uuid::new_v4(),
                    rule_id: "OUT-09".to_string(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    // span 为整个输入范围的近似（无精确字节偏移）
                    span: ContentSpan {
                        start: body_byte_offset,
                        end: body_byte_offset + input.len(),
                    },
                    evidence_truncated,
                    fingerprint: fp,
                });
                // 同一文本只需报一次（找到一个有效助记词即触发拦截）
                break;
            }
        }

        Ok(detections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_rules::engine::VectorscanEngine;
    use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};

    fn make_rule(
        id: &str,
        pattern: &str,
        severity: RulesSeverity,
        action: RulesAction,
    ) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
        }
    }

    #[test]
    fn scan_detects_pattern() {
        let rules = vec![make_rule(
            "OUT-TEST",
            r"secret",
            RulesSeverity::Critical,
            RulesAction::Block,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);

exec
/bin/zsh -lc "git show HEAD:crates/sieve-rules/rules/outbound.toml | sed -n '70,130p' && git show HEAD:crates/sieve-cli/src/engine_adapter.rs | sed -n '130,240p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable

# ---------------------------------------------------------------------------
# OUT-05: Google Cloud API Key
# 格式: AIza + 35 个 alnum/_/-
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-05"
description = "Google Cloud API Key (AIza...)"
pattern = 'AIza[0-9A-Za-z_\-]{35}'
severity = "high"
action = "block"
entropy_min = 4.0
keywords = ["AIza"]
allowlist_regexes = []
allowlist_stopwords = []

# ---------------------------------------------------------------------------
# OUT-06: JWT Token
# 格式: eyJ... (header.payload.signature，三段均为 base64url)
# 不用 lookahead，直接匹配三段结构
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-06"
description = "JWT Token (eyJ...)"
pattern = 'ey[A-Za-z0-9_\-]{16,}\.ey[A-Za-z0-9_\/\-]{16,}\.[A-Za-z0-9_\/\-]{10,}'
severity = "high"
action = "block"
entropy_min = 3.5
keywords = ["eyJ"]
allowlist_regexes = []
allowlist_stopwords = []

# ---------------------------------------------------------------------------
# OUT-07: PEM Private Key Header
# 覆盖: RSA / EC / DSA / PKCS#8 / generic PRIVATE KEY 头部
# 注意: 不包含 OPENSSH（由 OUT-10 专项覆盖）
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-07"
description = "PEM Private Key (RSA / EC / DSA / generic)"
pattern = '-----BEGIN[ A-Z0-9_\-]{0,60}PRIVATE KEY[ A-Z]{0,20}-----'
severity = "critical"
action = "block"
entropy_min = 0.0
keywords = ["-----BEGIN"]
allowlist_regexes = []
allowlist_stopwords = []

# ---------------------------------------------------------------------------
# OUT-08: Stripe Live Secret / Publishable / Restricted Key
# 格式: sk_live_/pk_live_/rk_live_ + 10~99 alnum
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-08"
description = "Stripe Live Key (sk_live_/pk_live_/rk_live_)"
pattern = '(?:sk|pk|rk)_live_[a-zA-Z0-9]{10,99}'
severity = "critical"
action = "block"
entropy_min = 3.5
keywords = ["_live_"]
allowlist_regexes = ['(?i)test|example']
            // critical_lock 强制：fail-closed 规则 action 一律覆盖为 Block
            let raw_action = rule.map(|r| r.action).unwrap_or(RulesAction::Block);
            let enforced_action =
                sieve_rules::critical_lock::enforce_action(&hit.rule_id, raw_action);
            let action = map_action(enforced_action);

            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_offset + hit.start,
                    end: body_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }
        Ok(detections)
    }

    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();
        // 1. 工具名扫描（IN-CR-05 签名工具）
        hits.extend(self.scan_text(&tool.name, source, 0)?);
        // 2. 工具输入序列化扫描（IN-CR-02 危险 shell 等）
        if let Ok(input_str) = serde_json::to_string(&tool.input) {
            hits.extend(self.scan_text(&input_str, source, 0)?);
        }
        Ok(hits)
    }
}

impl OutboundEngine for OutboundAdapter {
    /// 扫描文本，返回已过滤（per-rule allowlist）的命中列表，并执行 BIP39 second-pass。
    ///
    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
    ///
    /// BIP39 second-pass（PRD §9 #4）：vectorscan 之后独立扫描。
    /// 先提取全部在词表的连续词窗口，再做 SHA-256 checksum 验证，
    /// **仅 checksum 通过才生成 Critical Detection**。
    /// 词表命中但 checksum 失败的窗口**不得**定级 Critical（差异化要求）。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_byte_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            // per-rule allowlist 过滤
            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);
            let action = rule.map(|r| map_action(r.action)).unwrap_or(Action::Block);
            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_byte_offset + hit.start,
                    end: body_byte_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }

        // BIP39 second-pass（关联 PRD §9 #4 差异化点）
        // vectorscan 不覆盖 BIP39，此处独立扫描：
        // 1. 按空白分词，提取全在词表的连续窗口
        // 2. 对每个窗口做 SHA-256 checksum 验证
        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
        let wl = sieve_rules::wordlist::wordlist_index();
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
        for window in candidates {
            if sieve_rules::bip39::verify_checksum(&window, wl) {
                let window_text = window.join(" ");
                let evidence_truncated = redact_evidence(&window_text);

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '560,615p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   560	                    // 3. Block 类：注入 sieve_blocked 并截流
   561	                    if !blocking.is_empty() {
   562	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   563	                        for d in &blocking {
   564	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   565	                        }
   566	                        let blocked_payload = build_sieve_blocked_sse(&blocking);
   567	                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   568	                        return;
   569	                    }
   570	
   571	                    // 无 blocking / hold：透传原始 frame
   572	                    if tx
   573	                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
   574	                        .await
   575	                        .is_err()
   576	                    {
   577	                        return;
   578	                    }
   579	                }
   580	                Err(e) => {
   581	                    let _ = tx
   582	                        .send(Err(std::io::Error::other(format!(
   583	                            "upstream body error: {e}"
   584	                        ))))
   585	                        .await;
   586	                    return;
   587	                }
   588	            }
   589	        }
   590	
   591	        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
   592	        let flushed = parser.flush();
   593	        let (blocking, hook_detections, _hold_detections) =
   594	            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
   595	
   596	        for d in &hook_detections {
   597	            write_hook_pending_silent(d);
   598	        }
   599	
   600	        if !blocking.is_empty() {
   601	            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
   602	            for d in &blocking {
   603	                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
   604	            }
   605	            let blocked_payload = build_sieve_blocked_sse(&blocking);
   606	            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   607	        }
   608	    });
   609	
   610	    let body_stream = ReceiverStream::new(rx);
   611	    let response_body: ResponseBody = StreamBody::new(body_stream)
   612	        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
   613	        .boxed();
   614	
   615	    Ok(Response::from_parts(resp_parts, response_body))

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '430,565p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   430	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   431	                            return;
   432	                        }
   433	                    };
   434	
   435	                    // 收集本批 events 的 detections，按 action 分组处理
   436	                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
   437	                        &events,
   438	                        &mut inbound_filter,
   439	                        &mut aggregator,
   440	                        dry_run,
   441	                    );
   442	
   443	                    // 1. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
   444	                    for d in &hook_detections {
   445	                        write_hook_pending_silent(d);
   446	                    }
   447	
   448	                    // 2. GUI 类：hold 流 + keep-alive + 等用户决策
   449	                    if !hold_detections.is_empty() {
   450	                        if let Some(ref ipc_server) = ipc {
   451	                            // keep-alive channel：daemon 把心跳写入 SSE 流
   452	                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
   453	                            let tx_ka = tx.clone();
   454	
   455	                            // 先把当前 frame_bytes（触发命中的那帧）透传给客户端，
   456	                            // 然后再 hold——这样客户端已经看到触发 event，
   457	                            // hold 期间只收到 keep-alive comment。
   458	                            if tx
   459	                                .send(Ok(hyper::body::Frame::data(frame_bytes.clone())))
   460	                                .await
   461	                                .is_err()
   462	                            {
   463	                                return;
   464	                            }
   465	
   466	                            // 启动 keep-alive 转发 task
   467	                            let ka_fwd_handle = tokio::spawn(async move {
   468	                                while let Some(ka_bytes) = ka_rx.recv().await {
   469	                                    if tx_ka
   470	                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
   471	                                        .await
   472	                                        .is_err()
   473	                                    {
   474	                                        break;
   475	                                    }
   476	                                }
   477	                            });
   478	
   479	                            // 构造 IPC 请求
   480	                            use chrono::Utc;
   481	                            let request_id = uuid::Uuid::new_v4();
   482	                            let timeout_seconds = hold_detections
   483	                                .iter()
   484	                                .find_map(|d| {
   485	                                    if let Action::HoldForDecision {
   486	                                        timeout_seconds, ..
   487	                                    } = d.action
   488	                                    {
   489	                                        Some(timeout_seconds)
   490	                                    } else {
   491	                                        None
   492	                                    }
   493	                                })
   494	                                .unwrap_or(60);
   495	
   496	                            let ipc_detections = hold_detections
   497	                                .iter()
   498	                                .map(|d| sieve_ipc::protocol::DetectionPayload {
   499	                                    rule_id: d.rule_id.clone(),
   500	                                    severity: map_severity_to_ipc(d.severity),
   501	                                    disposition: sieve_ipc::Disposition::GuiPopup,
   502	                                    title: format!("检测命中：{}", d.rule_id),
   503	                                    one_line_summary: d.evidence_truncated.clone(),
   504	                                    details: serde_json::json!({}),
   505	                                })
   506	                                .collect();
   507	
   508	                            let ipc_req = sieve_ipc::DecisionRequest {
   509	                                request_id,
   510	                                created_at: Utc::now(),
   511	                                timeout_seconds,
   512	                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   513	                                detections: ipc_detections,
   514	                            };
   515	
   516	                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
   517	                                Arc::clone(ipc_server),
   518	                                ipc_req,
   519	                                ka_tx,
   520	                            )
   521	                            .await;
   522	
   523	                            ka_fwd_handle.abort();
   524	
   525	                            match outcome {
   526	                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
   527	                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
   528	                                    // 允许：继续转发后续 SSE 帧
   529	                                    // 当前帧已在 hold 前发出，继续循环
   530	                                    continue;
   531	                                }
   532	                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
   533	                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
   534	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   535	                                    let _ = tx
   536	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   537	                                        .await;
   538	                                    return;
   539	                                }
   540	                                Err(e) => {
   541	                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
   542	                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   543	                                    let _ = tx
   544	                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
   545	                                        .await;
   546	                                    return;
   547	                                }
   548	                            }
   549	                        } else {
   550	                            // IPC 未初始化：fail-closed，阻断
   551	                            tracing::warn!(
   552	                                "GuiPopup detection but IPC server not initialized; fail-closed"
   553	                            );
   554	                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
   555	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   556	                            return;
   557	                        }
   558	                    }
   559	
   560	                    // 3. Block 类：注入 sieve_blocked 并截流
   561	                    if !blocking.is_empty() {
   562	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   563	                        for d in &blocking {
   564	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   565	                        }

exec
/bin/zsh -lc "for f in crates/sieve-ipc/src/*.rs; do echo '###' "'$f; nl -ba $f | sed -n '"'1,260p'; done" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
### crates/sieve-ipc/src/decision_file.rs
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
### crates/sieve-ipc/src/error.rs
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
### crates/sieve-ipc/src/lib.rs
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
### crates/sieve-ipc/src/paths.rs
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
### crates/sieve-ipc/src/pending_file.rs
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
### crates/sieve-ipc/src/protocol.rs
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
### crates/sieve-ipc/src/socket_client.rs
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
### crates/sieve-ipc/src/socket_server.rs
     1	use std::collections::HashMap;
     2	use std::path::PathBuf;
     3	use std::sync::Arc;
     4	use std::time::Duration;
     5	
     6	use chrono::Utc;
     7	use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
     8	use tokio::net::{UnixListener, UnixStream};
     9	use tokio::sync::{oneshot, Mutex};
    10	use tracing::{debug, error, info, warn};
    11	use uuid::Uuid;
    12	
    13	use crate::{
    14	    error::IpcError,
    15	    protocol::{DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout},
    16	};
    17	
    18	type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;
    19	
    20	/// IPC 服务端，监听 Unix socket，接收 GUI 推送的决策响应。
    21	///
    22	/// 主代理启动后调用 [`IpcServer::bind`] 绑定 socket，然后通过
    23	/// [`IpcServer::request_decision`] 异步等待用户通过 GUI 做出决策。
    24	///
    25	/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
    26	pub struct IpcServer {
    27	    socket_path: PathBuf,
    28	    pending: PendingMap,
    29	}
    30	
    31	impl IpcServer {
    32	    /// 绑定 Unix socket 并返回服务端实例。
    33	    ///
    34	    /// socket_path 已存在时先删除旧文件（daemon 重启场景）。
    35	    pub fn bind(socket_path: PathBuf) -> Result<(Self, UnixListener), IpcError> {
    36	        // 旧 socket 文件存在则先删除，否则 bind 会失败。
    37	        if socket_path.exists() {
    38	            std::fs::remove_file(&socket_path)?;
    39	        }
    40	        let listener = UnixListener::bind(&socket_path)?;
    41	        let server = Self {
    42	            socket_path,
    43	            pending: Arc::new(Mutex::new(HashMap::new())),
    44	        };
    45	        Ok((server, listener))
    46	    }
    47	
    48	    /// 运行 accept 循环，处理来自 GUI 的连接。
    49	    ///
    50	    /// 每个连接独立 spawn，读取换行分隔的 JSON-RPC 响应，
    51	    /// 将结果派发到对应的 oneshot channel。
    52	    pub async fn run(&self, listener: UnixListener) {
    53	        info!(socket = %self.socket_path.display(), "IPC server listening");
    54	        loop {
    55	            match listener.accept().await {
    56	                Ok((stream, _addr)) => {
    57	                    let pending = Arc::clone(&self.pending);
    58	                    tokio::spawn(async move {
    59	                        if let Err(e) = handle_connection(stream, pending).await {
    60	                            error!("IPC connection error: {e}");
    61	                        }
    62	                    });
    63	                }
    64	                Err(e) => {
    65	                    error!("IPC accept error: {e}");
    66	                    break;
    67	                }
    68	            }
    69	        }
    70	    }
    71	
    72	    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
    73	    ///
    74	    /// 内部流程：
    75	    /// 1. 注册 oneshot channel
    76	    /// 2. 序列化 JSON-RPC request 推送给所有连接的客户端（广播第一个可达）
    77	    /// 3. 等待 `timeout` 内的响应，超时则按 `default_on_timeout` 构造兜底响应
    78	    ///
    79	    /// 注意：当前实现通过 socket 文件直接连接 GUI 客户端来发送请求；
    80	    /// 生产环境主代理维护持久连接，此处为接口完整性保留。
    81	    pub async fn request_decision(
    82	        &self,
    83	        req: DecisionRequest,
    84	        timeout: Duration,
    85	    ) -> Result<DecisionResponse, IpcError> {
    86	        let request_id = req.request_id;
    87	        let default_on_timeout = req.default_on_timeout;
    88	
    89	        let (tx, rx) = oneshot::channel::<DecisionResponse>();
    90	        {
    91	            let mut map = self.pending.lock().await;
    92	            map.insert(request_id, tx);
    93	        }
    94	
    95	        // 通过 socket 推送请求给 GUI 客户端。
    96	        // 错误不致命——GUI 可能尚未连接，hook 路径作为兜底。
    97	        if let Err(e) = send_request_via_socket(&self.socket_path, &req).await {
    98	            debug!("Failed to forward request to GUI socket: {e}");
    99	        }
   100	
   101	        match tokio::time::timeout(timeout, rx).await {
   102	            Ok(Ok(resp)) => Ok(resp),
   103	            Ok(Err(_)) => {
   104	                // Sender 已丢弃（连接断开），走超时兜底。
   105	                warn!(%request_id, "decision sender dropped, using timeout fallback");
   106	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   107	            }
   108	            Err(_elapsed) => {
   109	                // 清理 pending map。
   110	                self.pending.lock().await.remove(&request_id);
   111	                warn!(%request_id, "decision timeout");
   112	                Ok(make_timeout_fallback(request_id, default_on_timeout))
   113	            }
   114	        }
   115	    }
   116	
   117	    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
   118	    pub async fn inject_decision(&self, resp: DecisionResponse) {
   119	        let mut map = self.pending.lock().await;
   120	        if let Some(tx) = map.remove(&resp.request_id) {
   121	            let _ = tx.send(resp);
   122	        }
   123	    }
   124	}
   125	
   126	/// 处理单个 GUI 连接：读换行分隔的 JSON-RPC response，派发到 pending map。
   127	async fn handle_connection(stream: UnixStream, pending: PendingMap) -> Result<(), IpcError> {
   128	    let reader = BufReader::new(stream);
   129	    let mut lines = reader.lines();
   130	
   131	    while let Some(line) = lines.next_line().await? {
   132	        let line = line.trim().to_owned();
   133	        if line.is_empty() {
   134	            continue;
   135	        }
   136	        debug!(raw = %line, "received IPC message");
   137	
   138	        let rpc: crate::protocol::jsonrpc::Response = match serde_json::from_str(&line) {
   139	            Ok(v) => v,
   140	            Err(e) => {
   141	                warn!("failed to parse IPC response: {e}");
   142	                continue;
   143	            }
   144	        };
   145	
   146	        if let Some(err_obj) = &rpc.error {
   147	            error!(code = err_obj.code, message = %err_obj.message, "GUI returned rpc error");
   148	            continue;
   149	        }
   150	
   151	        if let Some(result) = rpc.result {
   152	            match serde_json::from_value::<DecisionResponse>(result) {
   153	                Ok(resp) => {
   154	                    let mut map = pending.lock().await;
   155	                    if let Some(tx) = map.remove(&resp.request_id) {
   156	                        let _ = tx.send(resp);
   157	                    } else {
   158	                        warn!(request_id = %resp.request_id, "no pending request for this decision");
   159	                    }
   160	                }
   161	                Err(e) => {
   162	                    warn!("failed to deserialize DecisionResponse: {e}");
   163	                }
   164	            }
   165	        }
   166	    }
   167	
   168	    Ok(())
   169	}
   170	
   171	/// 通过 Unix socket 将 [`DecisionRequest`] 作为 JSON-RPC 请求发送给 GUI。
   172	async fn send_request_via_socket(
   173	    socket_path: &PathBuf,
   174	    req: &DecisionRequest,
   175	) -> Result<(), IpcError> {
   176	    let mut stream = tokio::net::UnixStream::connect(socket_path).await?;
   177	    let rpc_req = crate::protocol::jsonrpc::Request::call(
   178	        "request_decision",
   179	        serde_json::to_value(req)?,
   180	        serde_json::Value::String(req.request_id.to_string()),
   181	    );
   182	    let mut payload = serde_json::to_string(&rpc_req)?;
   183	    payload.push('\n');
   184	    stream.write_all(payload.as_bytes()).await?;
   185	    Ok(())
   186	}
   187	
   188	fn make_timeout_fallback(
   189	    request_id: Uuid,
   190	    default_on_timeout: DefaultOnTimeout,
   191	) -> DecisionResponse {
   192	    let action = match default_on_timeout {
   193	        DefaultOnTimeout::Block => DecisionAction::Deny,
   194	        DefaultOnTimeout::Allow => DecisionAction::Allow,
   195	        DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
   196	    };
   197	    DecisionResponse {
   198	        request_id,
   199	        decision: action,
   200	        decided_at: Utc::now(),
   201	        by_user: false,
   202	        remember: false,
   203	    }
   204	}

exec
/bin/zsh -lc "sed -n '1,320p' crates/sieve-core/src/pipeline/inbound.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 入站规则匹配节点（Week 3 起实现）。
//!
//! 关联 PRD §5.2 入站检测 P0 表 + UCSB 论文 4 类攻击分类。

use crate::address_guard::{check_substitution, extract_eth_addresses};
use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use crate::error::{SieveCoreError, SieveCoreResult};
use crate::pipeline::streaming::StreamingPipelineNode;
use crate::protocol::unified_message::ContentSpan;
use crate::sse::parser::{SseDelta, SseEvent};
use crate::tool_use_aggregator::CompletedToolCall;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 入站引擎抽象接口（由 sieve-cli 把 sieve_rules::VectorscanEngine 适配进来）。
///
/// crate 边界：sieve-core 不直接依赖 sieve-rules，通过本 trait 解耦（.cursorrules §3.3）。
pub trait InboundEngine: Send + Sync {
    /// 扫描文本，返回命中的 Detection 列表。
    ///
    /// # Errors
    /// 扫描失败时返回 [`crate::error::SieveCoreError`]。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>>;

    /// 检查工具调用，返回命中的 Detection 列表。
    ///
    /// # Errors
    /// 检查失败时返回 [`crate::error::SieveCoreError`]。
    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>>;
}

/// 会话级状态（跨 SSE event 保持）。
#[derive(Default)]
pub struct SessionState {
    /// 当前会话中已见过的 ETH 地址集合（用于 IN-CR-01 地址替换检测）。
    pub addresses_seen: HashSet<String>,
}

/// 入站流式过滤节点，实现 [`StreamingPipelineNode`] trait。
pub struct InboundFilter {
    engine: Arc<dyn InboundEngine>,
    session: Mutex<SessionState>,
    /// `.sieveignore` 加载的 fingerprint 集合（O(1) 查询）。
    sieveignore: Arc<HashSet<String>>,
}

impl InboundFilter {
    /// 新建 InboundFilter。
    pub fn new(engine: Arc<dyn InboundEngine>, sieveignore: Arc<HashSet<String>>) -> Self {
        Self {
            engine,
            session: Mutex::new(SessionState::default()),
            sieveignore,
        }
    }

    /// 把出站 prompt 文本中的 EVM 地址 seed 到会话地址集合。
    ///
    /// 须在入站 SSE 检测（[`StreamingPipelineNode::observe_event`]）开始前调用，
    /// 否则首轮地址替换（prompt 地址 A → 响应地址 B）会漏报 IN-CR-01。
    ///
    /// 关联 PRD §4.2 真实攻击场景 / P0-3 修复。
    ///
    /// # Errors
    /// session mutex 中毒时返回 [`SieveCoreError`]。
    pub fn seed_known_addresses_from_text(&self, text: &str) -> SieveCoreResult<()> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| SieveCoreError::Forwarder("session mutex poisoned".into()))?;
        for addr in extract_eth_addresses(text) {
            session.addresses_seen.insert(addr);
        }
        Ok(())
    }

    /// 过滤掉 sieveignore 中已知的 fingerprint。
    ///
    /// PRD §9 #3 #8：Critical severity 永远不被过滤——
    /// `.sieveignore` 白名单仅对 High / Medium / Low 有效。
    fn filter_sieveignore(&self, dets: Vec<Detection>) -> Vec<Detection> {
        dets.into_iter()
            .filter(|d| {
                d.severity == Severity::Critical || !self.sieveignore.contains(&d.fingerprint)
            })
            .collect()
    }
}

impl StreamingPipelineNode for InboundFilter {
    fn name(&self) -> &str {
        "inbound-filter"
    }

    fn observe_event(&mut self, event: &SseEvent) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();

        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = event
        {
            // 1. 文本扫描（IN-GEN-* 通用规则 + 危险命令检测）
            hits.extend(
                self.engine
                    .scan_text(text, ContentSource::InboundAssistantText, 0)?,
            );

            // 2. IN-CR-01 地址替换检测
            let addrs = extract_eth_addresses(text);
            let mut session = self
                .session
                .lock()
                .map_err(|_| SieveCoreError::Forwarder("session mutex poisoned".into()))?;

            for addr in addrs {
                if let Some(orig) = check_substitution(&session.addresses_seen, &addr) {
                    let fp = fingerprint("IN-CR-01", &format!("{orig}->{addr}"));
                    hits.push(Detection {
                        id: Uuid::new_v4(),
                        rule_id: "IN-CR-01".into(),
                        severity: Severity::Critical,
                        action: Action::Block,
                        source: ContentSource::InboundAssistantText,
                        span: ContentSpan {
                            start: 0,
                            end: addr.len(),
                        },
                        evidence_truncated: format!("{orig}->{addr}"),
                        fingerprint: fp,
                    });
                }
                session.addresses_seen.insert(addr);
            }
        }

        Ok(self.filter_sieveignore(hits))
    }

    fn on_tool_use_complete(
        &mut self,
        tool: &CompletedToolCall,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self
            .engine
            .check_tool_use(tool, ContentSource::InboundToolUseInput)?;
        Ok(self.filter_sieveignore(hits))
    }

    fn on_message_stop(&mut self) -> SieveCoreResult<Vec<Detection>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
    use crate::protocol::unified_message::ContentSpan;
    use uuid::Uuid;

    /// Mock InboundEngine：
    /// - 文本含 "rm -rf" → 返回 IN-CR-02 命中
    /// - 工具名含 "signTransaction" → 返回 IN-CR-05 命中
    struct MockEngine;

    impl InboundEngine for MockEngine {
        fn scan_text(
            &self,
            input: &str,
            source: ContentSource,
            _body_offset: usize,
        ) -> SieveCoreResult<Vec<Detection>> {
            if input.contains("rm -rf") {
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-CR-02".into(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    span: ContentSpan { start: 0, end: 5 },
                    evidence_truncated: "**".into(),
                    fingerprint: fingerprint("IN-CR-02", "rm -rf"),
                }])
            } else if input.contains("suspicious_high") {
                // High severity detection，用于验证 sieveignore 可以合法压制非 Critical
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-GEN-01".into(),
                    severity: Severity::High,
                    action: Action::HookMark,
                    source,
                    span: ContentSpan { start: 0, end: 15 },
                    evidence_truncated: "suspicious_high".into(),
                    fingerprint: fingerprint("IN-GEN-01", "suspicious_high"),
                }])
            } else {
                Ok(vec![])
            }
        }

        fn check_tool_use(
            &self,
            tool: &CompletedToolCall,
            source: ContentSource,
        ) -> SieveCoreResult<Vec<Detection>> {
            if tool.name.contains("signTransaction") {
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-CR-05".into(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    span: ContentSpan {
                        start: 0,
                        end: tool.name.len(),
                    },
                    evidence_truncated: tool.name.clone(),
                    fingerprint: fingerprint("IN-CR-05", &tool.name),
                }])
            } else {
                Ok(vec![])
            }
        }
    }

    #[test]
    fn dangerous_shell_in_text_detected() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta {
                text: "run rm -rf /".into(),
            },
        };
        let hits = f.observe_event(&evt).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].rule_id, "IN-CR-02");
    }

    #[test]
    fn signing_tool_call_detected() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let tool = CompletedToolCall {
            id: "x".into(),
            name: "eth_signTransaction".into(),
            input: serde_json::json!({}),
        };
        let hits = f.on_tool_use_complete(&tool).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "IN-CR-05");
    }

    #[test]
    fn address_substitution_detected_across_events() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        // 第一个 event：植入原始地址
        let _ = f
            .observe_event(&SseEvent::ContentBlockDelta {
                index: 0,
                delta: SseDelta::TextDelta {
                    text: "send 0xabcdef1234567890abcdef1234567890abcdef12 here".into(),
                },
            })
            .unwrap();
        // 第二个 event：出现近似（末位 2→3）地址
        let hits = f
            .observe_event(&SseEvent::ContentBlockDelta {
                index: 0,
                delta: SseDelta::TextDelta {
                    text: "actually 0xabcdef1234567890abcdef1234567890abcdef13 here".into(),
                },
            })
            .unwrap();
        assert!(hits.iter().any(|d| d.rule_id == "IN-CR-01"));
    }

    /// sieveignore 可以合法压制 High / Medium 等非 Critical detection。
    /// Critical 不在此测试验证范围——见 sieveignore_does_not_suppress_critical。
    #[test]
    fn sieveignore_filters_non_critical_fingerprint() {
        let fp = fingerprint("IN-GEN-01", "suspicious_high");
        let mut ignore = HashSet::new();
        ignore.insert(fp);
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(ignore));
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta {
                text: "suspicious_high pattern here".into(),
            },
        };
        let hits = f.observe_event(&evt).unwrap();
        assert!(
            hits.is_empty(),
            "sieveignore should suppress High/non-Critical detection"
        );
    }

    #[test]
    fn non_text_delta_event_returns_no_hits() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        // MessageStop 不产生命中
        let hits = f.observe_event(&SseEvent::MessageStop).unwrap();
        assert!(hits.is_empty());
    }

    /// seed_known_addresses_from_text 预注入 prompt 地址，首轮地址替换可被 IN-CR-01 检测。
    ///
    /// 关联 P0-3 / PRD §4.2：prompt 地址 A + SSE 仅出现地址 B → 命中。
    #[test]

exec
/bin/zsh -lc "nl -ba crates/sieve-core/src/pipeline/inbound_hold.rs | sed -n '1,280p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
   164	    fn make_ipc_server() -> (Arc<IpcServer>, tokio::net::UnixListener) {
   165	        let tmp = tempfile::tempdir().unwrap();
   166	        let socket_path = tmp.path().join("ipc.sock");
   167	        // 把 tmp 路径 leak 到测试生命周期（tempfile 会在 drop 时清理，但 socket 不影响测试）
   168	        std::mem::forget(tmp);
   169	        IpcServer::bind(socket_path)
   170	            .map(|(s, l)| (Arc::new(s), l))
   171	            .unwrap()
   172	    }
   173	
   174	    // ── Mock IPC 返回 Allow ───────────────────────────────────────────────────
   175	
   176	    #[tokio::test]
   177	    async fn ipc_allow_returns_allow_outcome() {
   178	        let (server, listener) = make_ipc_server();
   179	        let srv = Arc::clone(&server);
   180	        tokio::spawn(async move { srv.run(listener).await });
   181	        tokio::time::sleep(Duration::from_millis(10)).await;
   182	
   183	        let id = Uuid::now_v7();
   184	        let req = make_request(id, 5, DefaultOnTimeout::Block);
   185	
   186	        // 50ms 后注入 Allow 决策
   187	        let inject_srv = Arc::clone(&server);
   188	        tokio::spawn(async move {
   189	            tokio::time::sleep(Duration::from_millis(50)).await;
   190	            inject_srv
   191	                .inject_decision(DecisionResponse {
   192	                    request_id: id,
   193	                    decision: DecisionAction::Allow,
   194	                    decided_at: Utc::now(),
   195	                    by_user: true,
   196	                    remember: false,
   197	                })
   198	                .await;
   199	        });
   200	
   201	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   202	        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
   203	            .await
   204	            .unwrap();
   205	        assert_eq!(outcome, HoldOutcome::Allow);
   206	    }
   207	
   208	    // ── Mock IPC 返回 Deny ────────────────────────────────────────────────────
   209	
   210	    #[tokio::test]
   211	    async fn ipc_deny_returns_deny_outcome() {
   212	        let (server, listener) = make_ipc_server();
   213	        let srv = Arc::clone(&server);
   214	        tokio::spawn(async move { srv.run(listener).await });
   215	        tokio::time::sleep(Duration::from_millis(10)).await;
   216	
   217	        let id = Uuid::now_v7();
   218	        let req = make_request(id, 5, DefaultOnTimeout::Block);
   219	
   220	        let inject_srv = Arc::clone(&server);
   221	        tokio::spawn(async move {
   222	            tokio::time::sleep(Duration::from_millis(50)).await;
   223	            inject_srv
   224	                .inject_decision(DecisionResponse {
   225	                    request_id: id,
   226	                    decision: DecisionAction::Deny,
   227	                    decided_at: Utc::now(),
   228	                    by_user: true,
   229	                    remember: false,
   230	                })
   231	                .await;
   232	        });
   233	
   234	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   235	        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
   236	            .await
   237	            .unwrap();
   238	        assert!(matches!(outcome, HoldOutcome::Deny { .. }));
   239	    }
   240	
   241	    // ── 超时 default_on_timeout = Block ──────────────────────────────────────
   242	
   243	    #[tokio::test]
   244	    async fn timeout_with_block_returns_deny() {
   245	        let (server, listener) = make_ipc_server();
   246	        let srv = Arc::clone(&server);
   247	        tokio::spawn(async move { srv.run(listener).await });
   248	        tokio::time::sleep(Duration::from_millis(10)).await;
   249	
   250	        let id = Uuid::now_v7();
   251	        // timeout_seconds = 1 但我们传极短 duration（100ms）给 IpcServer
   252	        // 用 timeout_seconds = 0（最小值 max(0,1) = 1s）——太慢；改用 mock 方式
   253	        // 实际测 timeout：直接用非常短的 timeout_seconds，IpcServer 内部用 Duration
   254	        // 这里 timeout_seconds=1 传给 hold_and_decide → Duration::from_secs(1)；
   255	        // 但测试不等 1s。改法：覆写 Duration 计算，或用 tokio::time::pause/advance。
   256	        // 简化：使用 tokio::time::pause() + advance()
   257	        tokio::time::pause();
   258	
   259	        let req = make_request(id, 1, DefaultOnTimeout::Block);
   260	        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
   261	
   262	        let ipc_clone = Arc::clone(&server);
   263	        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });
   264	
   265	        // 推进 2 秒让超时触发
   266	        tokio::time::advance(Duration::from_secs(2)).await;
   267	        tokio::time::resume();
   268	
   269	        let outcome = task.await.unwrap().unwrap();
   270	        assert!(
   271	            matches!(outcome, HoldOutcome::Deny { .. }),
   272	            "timeout with Block should return Deny, got {outcome:?}"
   273	        );
   274	    }
   275	
   276	    // ── 超时 default_on_timeout = Allow ──────────────────────────────────────
   277	
   278	    #[tokio::test]
   279	    async fn timeout_with_allow_returns_allow() {
   280	        let (server, listener) = make_ipc_server();

exec
/bin/zsh -lc 'cargo test -p sieve-ipc' in /Users/doskey/src/sieve
 exited 101 in 220ms:

running 18 tests
test paths_tests::sieve_home_env_override ... ok
test tests::decision_action_serde_snake_case ... ok
test file_tests::pending_not_found_error ... ok
test tests::disposition_serde_snake_case ... ok
test tests::jsonrpc_call_includes_id ... ok
test tests::jsonrpc_request_omits_null_id ... ok
test tests::severity_serde_snake_case ... ok
test paths_tests::sieve_home_default_uses_home ... FAILED
test tests::decision_response_round_trip ... ok
test tests::decision_request_round_trip ... ok
test paths_tests::ensure_dirs_idempotent ... ok
test file_tests::decision_write_and_read ... ok
test socket_tests::socket_server_timeout_fallback ... FAILED
test file_tests::pending_write_and_read ... ok
test socket_tests::socket_server_receives_decision ... FAILED
test file_tests::pending_file_lock_two_tasks ... ok
test file_tests::wait_for_decision_timeout_fallback ... ok
test file_tests::wait_for_decision_found ... ok

failures:

---- paths_tests::sieve_home_default_uses_home stdout ----

thread 'paths_tests::sieve_home_default_uses_home' panicked at crates/sieve-ipc/src/lib.rs:385:9:
assertion failed: home.to_str().unwrap().ends_with(".sieve")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- socket_tests::socket_server_timeout_fallback stdout ----

thread 'socket_tests::socket_server_timeout_fallback' panicked at crates/sieve-ipc/src/lib.rs:341:71:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })

---- socket_tests::socket_server_receives_decision stdout ----

thread 'socket_tests::socket_server_receives_decision' panicked at crates/sieve-ipc/src/lib.rs:301:71:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })


failures:
    paths_tests::sieve_home_default_uses_home
    socket_tests::socket_server_receives_decision
    socket_tests::socket_server_timeout_fallback

test result: FAILED. 15 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

error: test failed, to rerun pass `-p sieve-ipc --lib`

exec
/bin/zsh -lc "nl -ba crates/sieve-ipc/src/lib.rs | sed -n '280,410p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   280	    use chrono::Utc;
   281	    use std::time::Duration;
   282	    use uuid::Uuid;
   283	
   284	    use super::{protocol::*, socket_server::IpcServer};
   285	
   286	    fn make_request(id: Uuid) -> DecisionRequest {
   287	        DecisionRequest {
   288	            request_id: id,
   289	            created_at: Utc::now(),
   290	            timeout_seconds: 30,
   291	            default_on_timeout: DefaultOnTimeout::Block,
   292	            detections: vec![],
   293	        }
   294	    }
   295	
   296	    #[tokio::test]
   297	    async fn socket_server_receives_decision() {
   298	        let tmp = tempfile::tempdir().unwrap();
   299	        let socket_path = tmp.path().join("ipc.sock");
   300	
   301	        let (server, listener) = IpcServer::bind(socket_path.clone()).unwrap();
   302	        let server = std::sync::Arc::new(server);
   303	
   304	        // 启动 accept 循环。
   305	        let server_clone = std::sync::Arc::clone(&server);
   306	        tokio::spawn(async move { server_clone.run(listener).await });
   307	
   308	        // 给服务端一点时间就绪。
   309	        tokio::time::sleep(Duration::from_millis(10)).await;
   310	
   311	        let id = Uuid::now_v7();
   312	        let req = make_request(id);
   313	
   314	        // 后台模拟客户端：注入决策响应。
   315	        let server_inject = std::sync::Arc::clone(&server);
   316	        tokio::spawn(async move {
   317	            tokio::time::sleep(Duration::from_millis(50)).await;
   318	            let resp = DecisionResponse {
   319	                request_id: id,
   320	                decision: DecisionAction::Allow,
   321	                decided_at: Utc::now(),
   322	                by_user: true,
   323	                remember: false,
   324	            };
   325	            server_inject.inject_decision(resp).await;
   326	        });
   327	
   328	        let result = server
   329	            .request_decision(req, Duration::from_secs(2))
   330	            .await
   331	            .unwrap();
   332	        assert_eq!(result.decision, DecisionAction::Allow);
   333	        assert!(result.by_user);
   334	    }
   335	
   336	    #[tokio::test]
   337	    async fn socket_server_timeout_fallback() {
   338	        let tmp = tempfile::tempdir().unwrap();
   339	        let socket_path = tmp.path().join("ipc.sock");
   340	
   341	        let (server, listener) = IpcServer::bind(socket_path.clone()).unwrap();
   342	        let server = std::sync::Arc::new(server);
   343	
   344	        let server_clone = std::sync::Arc::clone(&server);
   345	        tokio::spawn(async move { server_clone.run(listener).await });
   346	        tokio::time::sleep(Duration::from_millis(10)).await;
   347	
   348	        let id = Uuid::now_v7();
   349	        let req = DecisionRequest {
   350	            request_id: id,
   351	            created_at: Utc::now(),
   352	            timeout_seconds: 1,
   353	            default_on_timeout: DefaultOnTimeout::Allow,
   354	            detections: vec![],
   355	        };
   356	
   357	        // 极短超时，没有任何客户端响应——应返回 Allow（default_on_timeout）。
   358	        let result = server
   359	            .request_decision(req, Duration::from_millis(100))
   360	            .await
   361	            .unwrap();
   362	        assert_eq!(result.decision, DecisionAction::Allow);
   363	        assert!(!result.by_user);
   364	    }
   365	}
   366	
   367	#[cfg(test)]
   368	mod paths_tests {
   369	    use super::paths::*;
   370	
   371	    #[test]
   372	    fn sieve_home_env_override() {
   373	        // 设置 SIEVE_HOME，验证覆盖生效。
   374	        std::env::set_var("SIEVE_HOME", "/tmp/test_sieve_override");
   375	        let home = sieve_home().unwrap();
   376	        std::env::remove_var("SIEVE_HOME");
   377	        assert_eq!(home.to_str().unwrap(), "/tmp/test_sieve_override");
   378	    }
   379	
   380	    #[test]
   381	    fn sieve_home_default_uses_home() {
   382	        std::env::remove_var("SIEVE_HOME");
   383	        // HOME 应已设置（测试环境）。
   384	        let home = sieve_home().unwrap();
   385	        assert!(home.to_str().unwrap().ends_with(".sieve"));
   386	    }
   387	
   388	    #[test]
   389	    fn ensure_dirs_idempotent() {
   390	        let tmp = tempfile::tempdir().unwrap();
   391	        ensure_dirs(tmp.path()).unwrap();
   392	        ensure_dirs(tmp.path()).unwrap(); // 第二次调用不应报错。
   393	        assert!(pending_dir(tmp.path()).exists());
   394	        assert!(decisions_dir(tmp.path()).exists());
   395	        assert!(locks_dir(tmp.path()).exists());
   396	    }
   397	}

exec
/bin/zsh -lc "nl -ba crates/sieve-hook/src/decision.rs | sed -n '1,220p' && nl -ba crates/sieve-hook/src/error.rs | sed -n '1,160p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "grep -R \"audit_yolo_disabled\" -n crates/sieve-cli/src && nl -ba crates/sieve-cli/src/main.rs | sed -n '130,240p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-cli/src/main.rs:120:            audit_yolo_disabled(&cfg)?;
crates/sieve-cli/src/main.rs:155:fn audit_yolo_disabled(cfg: &config::Config) -> Result<()> {
   130	        Command::Version => {
   131	            println!("sieve {}", env!("CARGO_PKG_VERSION"));
   132	        }
   133	        Command::Setup(args) => {
   134	            commands::setup::run(args)?;
   135	        }
   136	        Command::Doctor => {
   137	            commands::doctor::run()?;
   138	        }
   139	        Command::Uninstall(args) => {
   140	            commands::uninstall::run(args)?;
   141	        }
   142	    }
   143	
   144	    Ok(())
   145	}
   146	
   147	/// 防御性检查：确认配置中无任何试图禁用 Critical 检测的字段。
   148	///
   149	/// Phase 1 实现：`Config` 已用 `#[serde(deny_unknown_fields)]` 在反序列化时拒绝
   150	/// 所有未知字段（含 `disable_critical` / `yolo` / `bypass` 等），此函数作为
   151	/// 运行时第二道防线，仅记录审计日志。
   152	///
   153	/// # Errors
   154	/// 当前实现不返回错误；签名保留 `Result<()>` 便于 Week 4 扩展检查逻辑。
   155	fn audit_yolo_disabled(cfg: &config::Config) -> Result<()> {
   156	    // dry_run 模式下 fail-closed 规则仍强制 Block（ADR-007 §2）
   157	    if cfg.dry_run {
   158	        tracing::warn!(
   159	            "dry_run=true: non-fail-closed Critical detections will only be logged, \
   160	             NOT blocked. Fail-closed rules (IN-CR-01/02/05/IN-GEN-01/03/OUT-01~12) \
   161	             remain enforced regardless."
   162	        );
   163	    }
   164	    tracing::info!("YOLO mode audit: passed (no critical-disable fields detected)");
   165	    Ok(())
   166	}
   167	
   168	/// 从文件加载 `.sieveignore` fingerprint 白名单。
   169	///
   170	/// 文件不存在时静默返回空集合（正常状态）；读取失败时打印 WARN 并返回空集合。
   171	/// 每行一个 fingerprint，支持 `#` 注释行和空行。
   172	fn load_sieveignore(path: &Path) -> HashSet<String> {
   173	    if !path.exists() {
   174	        return HashSet::new();
   175	    }
   176	    match std::fs::read_to_string(path) {
   177	        Ok(s) => s
   178	            .lines()
   179	            .map(str::trim)
   180	            .filter(|l| !l.is_empty() && !l.starts_with('#'))
   181	            .map(String::from)
   182	            .collect(),
   183	        Err(e) => {
   184	            tracing::warn!(
   185	                path = %path.display(),
   186	                error = %e,
   187	                "failed to load .sieveignore; proceeding with empty allowlist"
   188	            );
   189	            HashSet::new()
   190	        }
   191	    }
   192	}
   193	
   194	fn init_tracing() {
   195	    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
   196	
   197	    let filter = EnvFilter::try_from_env("SIEVE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
   198	    tracing_subscriber::registry()
   199	        .with(filter)
   200	        .with(fmt::layer().with_target(false))
   201	        .init();
   202	}

exec
/bin/zsh -lc 'cargo tree -p sieve-core --no-default-features --depth 1' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
├── chrono v0.4.44
├── serde v1.0.228
├── serde_json v1.0.149
├── sha2 v0.10.9
├── sieve-ipc v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-ipc)
├── strsim v0.11.1
├── thiserror v2.0.18
├── tracing v0.1.44
└── uuid v1.23.1
[dev-dependencies]
├── tempfile v3.27.0
└── tokio v1.52.1

exec
/bin/zsh -lc 'git diff --check' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable

exec
/bin/zsh -lc "git diff -U5 -- crates/sieve-cli/src/config.rs | sed -n '1,220p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
diff --git a/crates/sieve-cli/src/config.rs b/crates/sieve-cli/src/config.rs
index 350dc84..a052d44 100644
--- a/crates/sieve-cli/src/config.rs
+++ b/crates/sieve-cli/src/config.rs
@@ -2,17 +2,36 @@
 //!
 //! Phase 1 字段：`upstream_url` / `port` / `bind_addr` / `log_path` /
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
 /// 文件不存在时 [`Config::load`] 返回 [`Config::default`]。
 #[derive(Debug, Clone, Deserialize)]
@@ -55,10 +74,80 @@ pub struct Config {
     pub dry_run: bool,
 
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
     "https://api.anthropic.com".to_string()
 }
@@ -85,10 +174,17 @@ fn default() -> Self {
             tls_verify_upstream: default_tls_verify(),
             rules_path: None,
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
 
 impl Config {

exec
/bin/zsh -lc "grep -R \"extract_text_content\" -n crates/sieve-core/src/protocol crates/sieve-cli/src && sed -n '1,260p' crates/sieve-core/src/protocol/anthropic.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-core/src/protocol/anthropic.rs:56:    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
crates/sieve-core/src/protocol/anthropic.rs:146:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:155:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:165:        let texts = req.extract_text_content();
crates/sieve-cli/src/daemon.rs:202:        let texts = anthropic_req.extract_text_content();
//! Anthropic Messages API 请求/响应 schema（子集）。
//!
//! 文档: <https://docs.anthropic.com/en/api/messages>
//! 关联 PRD §6.1 Phase 1 边界。
//!
//! 只实现 Phase 1 需要的字段；extra 字段通过 `#[serde(flatten)]` 保留，
//! 确保原始 body 可无损转发到上游。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// POST /v1/messages 请求 body。
///
/// 关联 PRD §6.1：Phase 1 只解析 Anthropic 格式，其他 provider 预留 (ADR-004)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequest {
    /// 模型名（如 claude-sonnet-4-6）。
    pub model: String,
    /// 最大生成 token 数。
    pub max_tokens: u32,
    /// 消息列表。
    pub messages: Vec<AnthropicMessage>,
    /// 是否流式（SSE）。
    #[serde(default)]
    pub stream: bool,
    /// 系统提示（string 或 content blocks）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<serde_json::Value>,
    /// 工具定义列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<serde_json::Value>,
    /// 工具选择策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// 其他字段（向前兼容，不在乎也不丢弃）。
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Anthropic Messages API 单条消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    /// 角色（"user" 或 "assistant"）。
    pub role: String,
    /// 内容（string 或 content block 数组）。
    pub content: serde_json::Value,
}

impl AnthropicRequest {
    /// 提取所有 message content 中的文本（string content 或 type=text content block）。
    ///
    /// 返回 `(近似 body 字节偏移, text)` 列表。Phase 1 偏移仅供审计参考；精确 span 由
    /// vectorscan 在单条文本内 scan 时给出（start/end 是相对该 text 的偏移）。
    ///
    /// 同时追加 `system` 字段中的文本（string 或 content blocks）。
    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut cursor = 0usize;
        for msg in &self.messages {
            match &msg.content {
                serde_json::Value::String(s) => {
                    result.push((cursor, s.clone()));
                    cursor += s.len();
                }
                serde_json::Value::Array(blocks) => {
                    for block in blocks {
                        if let Some(block_obj) = block.as_object() {
                            if block_obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                                if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
                                    result.push((cursor, text.to_string()));
                                    cursor += text.len();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // 同时扫 system prompt（若有）
        if let Some(system) = &self.system {
            if let Some(s) = system.as_str() {
                result.push((cursor, s.to_string()));
            } else if let Some(blocks) = system.as_array() {
                for block in blocks {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        result.push((cursor, text.to_string()));
                        cursor += text.len();
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_request() {
        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1024,"messages":[{"role":"user","content":"hi"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "claude-sonnet-4-6");
        assert_eq!(req.messages.len(), 1);
        assert!(!req.stream);
        assert!(req.extra.is_empty());
    }

    #[test]
    fn parse_streaming_request_with_tools() {
        let json = r#"{
            "model": "claude-opus-4-5",
            "max_tokens": 4096,
            "stream": true,
            "messages": [{"role": "user", "content": "hello"}],
            "tools": [{"name": "bash", "description": "run shell"}],
            "unknown_future_field": 42
        }"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        assert!(req.stream);
        assert!(req.tools.is_some());
        // 未知字段被 flatten 保留，不丢弃
        assert!(req.extra.contains_key("unknown_future_field"));
    }

    #[test]
    fn roundtrip_preserves_extra_fields() {
        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1,"messages":[],"custom_key":"custom_value"}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let re_serialized = serde_json::to_string(&req).unwrap();
        assert!(re_serialized.contains("custom_key"));
        assert!(re_serialized.contains("custom_value"));
    }
}

#[cfg(test)]
mod tests_extract {
    use super::*;

    #[test]
    fn extract_simple_string_content() {
        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":"hello"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].1, "hello");
    }

    #[test]
    fn extract_content_blocks() {
        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":[{"type":"text","text":"hi"},{"type":"text","text":"world"}]}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0].1, "hi");
        assert_eq!(texts[1].1, "world");
    }

    #[test]
    fn extract_with_system_prompt() {
        let json = r#"{"model":"x","max_tokens":1,"system":"You are helpful","messages":[{"role":"user","content":"q"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 2);
        // system 在最后一项
        assert!(texts.iter().any(|(_, t)| t == "You are helpful"));
    }
}

exec
/bin/zsh -lc "git show HEAD:crates/sieve-cli/src/engine_adapter.rs | grep -n 'OUT-09' -C2 && git show HEAD:crates/sieve-rules/rules/outbound.toml | grep -n 'OUT-09' -C2" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
231-        // 1. 按空白分词，提取全在词表的连续窗口
232-        // 2. 对每个窗口做 SHA-256 checksum 验证
233:        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
234-        let wl = sieve_rules::wordlist::wordlist_index();
235-        let tokens: Vec<&str> = input.split_whitespace().collect();
--
239-                let window_text = window.join(" ");
240-                let evidence_truncated = redact_evidence(&window_text);
241:                let fp = fingerprint("OUT-09", &window_text);
242-                detections.push(Detection {
243-                    id: Uuid::new_v4(),
244:                    rule_id: "OUT-09".to_string(),
245-                    severity: Severity::Critical,
246-                    action: Action::Block,
132-
133-# ---------------------------------------------------------------------------
134:# OUT-09: Slack Token
135-# 格式: xoxb-/xoxp-/xoxa-/xoxs- + 10+ alnum/-
136-# ---------------------------------------------------------------------------
137-[[rules]]
138:id = "OUT-09"
139-description = "Slack Token (xoxb-/xoxp-/xoxa-/xoxs-)"
140-pattern = 'xox[bpas]\-[0-9A-Za-z\-]{10,}'
--
178-
179-# ---------------------------------------------------------------------------
180:# OUT-09（BIP39 助记词）在 engine_adapter 中通过 second-pass 实现，
181-# 不使用 vectorscan 占位规则。
182-# 详见 crates/sieve-cli/src/engine_adapter.rs OutboundAdapter::scan_text。

codex
当前改动包含多处会使新增 v1.4 路由、自动脱敏、hook/GUI 决策和 setup/uninstall 流程失效的逻辑问题。部分问题会导致安全检测绕过或安装后服务无法正常启动，因此补丁不能视为正确。

Full review comments:

- [P1] 使用真实 body 偏移做自动脱敏 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:247-250
  当 AutoRedact 命中时，这里把 `Detection.span` 直接当作原始 JSON body 的字节范围使用，但这些 span 来自 `extract_text_content()` 的“近似/累计文本偏移”，不是 body 中字符串值的真实位置；因此会改写错误字节，可能破坏 JSON 且留下原始 secret。需要在解析时保留真实 raw body 范围，或修改解析后的 JSON 再序列化。

- [P1] 不要把 manifest 的 block 当成硬阻断 — /Users/doskey/src/sieve/crates/sieve-cli/src/engine_adapter.rs:260-262
  默认规则仍然写着 `action = "block"`，所以这个判断会让 `disposition = "auto_redact"/"hook_terminal"/"gui_popup"` 的规则永远走不到 `map_action_by_disposition`；结果 OUT-01~05 不会脱敏，IN-CR-02/05 也不会进入 hook/GUI 路径。这里和上面的 InboundAdapter 都应只在真正需要硬阻断时返回 `Action::Block`，否则按 disposition 路由。

- [P1] 在无 request_id 时不要拒绝所有 hook — /Users/doskey/src/sieve/crates/sieve-hook/src/main.rs:53-59
  通过 `sieve setup` 注册的命令是静态的 `sieve-hook check`，当前代理也没有把 `SIEVE_REQUEST_ID` 注入 Claude Code；因此正常 PreToolUse 调用会走到这里直接 `exit(1)`，导致所有工具调用都被拒绝。应实现按 pending 目录/工具信息匹配的降级，或无匹配时 fail-open。

- [P1] 先处理 Block 再允许 hold 继续 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:526-530
  如果同一批 SSE event 同时包含 `HoldForDecision` 和 `Action::Block` 命中，用户允许 GUI 后这里 `continue`，下面的 blocking 分支不会执行，fail-closed 命中被绕过。Block 应在任何可被用户允许的 hold 之前处理，或 allow 后仍继续检查本批 blocking。

- [P1] 处理 flush 阶段的 hold 命中 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:593-594
  在 EOF/提前断流的 flush 路径里，`HoldForDecision` 命中被放进 `_hold_detections` 后直接丢弃；如果最后一个未闭合 SSE event 命中 IN-CR-05 或 IN-GEN-04 这类 GUI 规则，流会正常结束而不会 hold 或 fail-closed。这里需要和正常 frame 路径一样处理 hold，至少在无法决策时阻断。

- [P2] 不要把 GUI 请求发回服务端自己的 socket — /Users/doskey/src/sieve/crates/sieve-ipc/src/socket_server.rs:95-99
  `request_decision` 连接的是 `self.socket_path`，也就是 IpcServer 自己监听的 Unix socket；这条连接会被 `handle_connection` 当成 GUI response 处理，真正的 GUI 客户端不会收到 `request_decision`，所以 GUI hold 只能等到超时/default。需要维护已连接 GUI 的写半边并向客户端发送请求，而不是连回服务端 socket。

- [P1] 给 launchd 提供可解析的规则路径 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:395-399
  通过 `sieve setup` 安装后，launchd 不会从仓库根目录启动；但 `sieve start` 的默认规则路径仍是相对的 `crates/sieve-rules/rules/...`。只写 `sieve start` 会让 daemon 在启动时找不到规则并退出，导致 setup 后代理不可用；plist 应传入带绝对规则路径的 config，或设置正确的 `WorkingDirectory`。

- [P2] 解析失败时不要清空 settings.json — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:141-141
  当现有 `settings.json` 包含当前 `strip_json_comments` 不支持的 JSONC（例如尾逗号或块注释）或临时语法错误时，这里吞掉解析错误并改用 `{}`，后续写回会丢掉用户原有 Claude 配置。应在解析失败时返回错误，或使用完整 JSONC 解析并保留原内容。

- [P2] 卸载时移除 setup 新建的 Claude 配置 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/uninstall.rs:82-85
  如果 setup 前不存在 `~/.claude/settings.json`，备份目录里不会有对应文件，uninstall 这里只恢复备份而不会删除 setup 创建的新文件或移除 Sieve entries，导致 `ANTHROPIC_BASE_URL` 和 hook 继续留在 Claude 配置中。需要记录新建文件/条目并在卸载时删除或回滚。
2026-04-28T01:46:13.587057Z ERROR codex_core::session: failed to record rollout items: thread 019dd1bd-4153-70b0-abc8-3c3cb5fc8ee3 not found
2026-04-28T01:46:13.618219Z ERROR codex_core::session: failed to record rollout items: thread 019dd1bd-411d-7b42-a502-460cdb1c9d98 not found
当前改动包含多处会使新增 v1.4 路由、自动脱敏、hook/GUI 决策和 setup/uninstall 流程失效的逻辑问题。部分问题会导致安全检测绕过或安装后服务无法正常启动，因此补丁不能视为正确。

Full review comments:

- [P1] 使用真实 body 偏移做自动脱敏 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:247-250
  当 AutoRedact 命中时，这里把 `Detection.span` 直接当作原始 JSON body 的字节范围使用，但这些 span 来自 `extract_text_content()` 的“近似/累计文本偏移”，不是 body 中字符串值的真实位置；因此会改写错误字节，可能破坏 JSON 且留下原始 secret。需要在解析时保留真实 raw body 范围，或修改解析后的 JSON 再序列化。

- [P1] 不要把 manifest 的 block 当成硬阻断 — /Users/doskey/src/sieve/crates/sieve-cli/src/engine_adapter.rs:260-262
  默认规则仍然写着 `action = "block"`，所以这个判断会让 `disposition = "auto_redact"/"hook_terminal"/"gui_popup"` 的规则永远走不到 `map_action_by_disposition`；结果 OUT-01~05 不会脱敏，IN-CR-02/05 也不会进入 hook/GUI 路径。这里和上面的 InboundAdapter 都应只在真正需要硬阻断时返回 `Action::Block`，否则按 disposition 路由。

- [P1] 在无 request_id 时不要拒绝所有 hook — /Users/doskey/src/sieve/crates/sieve-hook/src/main.rs:53-59
  通过 `sieve setup` 注册的命令是静态的 `sieve-hook check`，当前代理也没有把 `SIEVE_REQUEST_ID` 注入 Claude Code；因此正常 PreToolUse 调用会走到这里直接 `exit(1)`，导致所有工具调用都被拒绝。应实现按 pending 目录/工具信息匹配的降级，或无匹配时 fail-open。

- [P1] 先处理 Block 再允许 hold 继续 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:526-530
  如果同一批 SSE event 同时包含 `HoldForDecision` 和 `Action::Block` 命中，用户允许 GUI 后这里 `continue`，下面的 blocking 分支不会执行，fail-closed 命中被绕过。Block 应在任何可被用户允许的 hold 之前处理，或 allow 后仍继续检查本批 blocking。

- [P1] 处理 flush 阶段的 hold 命中 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:593-594
  在 EOF/提前断流的 flush 路径里，`HoldForDecision` 命中被放进 `_hold_detections` 后直接丢弃；如果最后一个未闭合 SSE event 命中 IN-CR-05 或 IN-GEN-04 这类 GUI 规则，流会正常结束而不会 hold 或 fail-closed。这里需要和正常 frame 路径一样处理 hold，至少在无法决策时阻断。

- [P2] 不要把 GUI 请求发回服务端自己的 socket — /Users/doskey/src/sieve/crates/sieve-ipc/src/socket_server.rs:95-99
  `request_decision` 连接的是 `self.socket_path`，也就是 IpcServer 自己监听的 Unix socket；这条连接会被 `handle_connection` 当成 GUI response 处理，真正的 GUI 客户端不会收到 `request_decision`，所以 GUI hold 只能等到超时/default。需要维护已连接 GUI 的写半边并向客户端发送请求，而不是连回服务端 socket。

- [P1] 给 launchd 提供可解析的规则路径 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:395-399
  通过 `sieve setup` 安装后，launchd 不会从仓库根目录启动；但 `sieve start` 的默认规则路径仍是相对的 `crates/sieve-rules/rules/...`。只写 `sieve start` 会让 daemon 在启动时找不到规则并退出，导致 setup 后代理不可用；plist 应传入带绝对规则路径的 config，或设置正确的 `WorkingDirectory`。

- [P2] 解析失败时不要清空 settings.json — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:141-141
  当现有 `settings.json` 包含当前 `strip_json_comments` 不支持的 JSONC（例如尾逗号或块注释）或临时语法错误时，这里吞掉解析错误并改用 `{}`，后续写回会丢掉用户原有 Claude 配置。应在解析失败时返回错误，或使用完整 JSONC 解析并保留原内容。

- [P2] 卸载时移除 setup 新建的 Claude 配置 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/uninstall.rs:82-85
  如果 setup 前不存在 `~/.claude/settings.json`，备份目录里不会有对应文件，uninstall 这里只恢复备份而不会删除 setup 创建的新文件或移除 Sieve entries，导致 `ANTHROPIC_BASE_URL` 和 hook 继续留在 Claude 配置中。需要记录新建文件/条目并在卸载时删除或回滚。
