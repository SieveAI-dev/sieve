OpenAI Codex v0.125.0 (research preview)
--------
workdir: /Users/doskey/src/sieve/docs/review
model: gpt-5.5
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/doskey/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019dd2ff-876c-78a0-8285-fb24b0718374
--------
user
current changes
exec
/bin/zsh -lc 'git status --short --branch' in /Users/doskey/src/sieve/docs/review
 succeeded in 0ms:
## main...origin/main
 M ../../.cursorrules
 M ../../CLAUDE.md
 M ../../Cargo.lock
 M ../../Cargo.toml
 M ../../README.md
 M ../../SECURITY.md
 M ../../crates/sieve-cli/Cargo.toml
 M ../../crates/sieve-cli/src/audit.rs
 M ../../crates/sieve-cli/src/cli.rs
 M ../../crates/sieve-cli/src/config.rs
 M ../../crates/sieve-cli/src/daemon.rs
 M ../../crates/sieve-cli/src/engine_adapter.rs
 M ../../crates/sieve-cli/src/main.rs
 M ../../crates/sieve-cli/tests/inbound_block.rs
 M ../../crates/sieve-cli/tests/outbound_block.rs
 M ../../crates/sieve-core/Cargo.toml
 M ../../crates/sieve-core/src/detection.rs
 M ../../crates/sieve-core/src/pipeline/inbound.rs
 M ../../crates/sieve-core/src/pipeline/mod.rs
 M ../../crates/sieve-rules/benches/scan_bench.rs
 M ../../crates/sieve-rules/rules/inbound.toml
 M ../../crates/sieve-rules/rules/outbound.toml
 M ../../crates/sieve-rules/src/critical_lock.rs
 M ../../crates/sieve-rules/src/engine/mod.rs
 M ../../crates/sieve-rules/src/manifest.rs
 M ../../crates/sieve-rules/tests/inbound_rules.rs
 M ../../crates/sieve-rules/tests/outbound_rules.rs
 M ../api/api-reference.md
 M ../changelog/CHANGELOG.md
 M ../design/ADR-001-rust-tech-stack.md
 M ../design/ADR-002-rule-engine-only-phase1.md
 M ../design/ADR-003-local-only-no-cloud-verifier.md
 M ../design/ADR-004-anthropic-first-unified-interface.md
 M ../design/ADR-005-overseas-legal-entity.md
 M ../design/ADR-006-sigstore-reproducible-build.md
 M ../design/ADR-007-fail-closed-critical-actions.md
 M ../design/ADR-011-private-until-ga.md
 M ../design/ADR-INDEX.md
 M ../design/architecture.md
 M ../design/data-model.md
 M ../glossary.md
 M ../guides/deployment.md
 M ../guides/development.md
 M ../requirements/PRD-sieve.md
 M ../requirements/user-stories.md
 M ../../tasks/lessons.md
 M ../../tasks/roadmap.md
?? ../../crates/sieve-cli/src/commands/
?? ../../crates/sieve-cli/tests/audit_append_only.rs
?? ../../crates/sieve-cli/tests/sieve_setup_dry_run.rs
?? ../../crates/sieve-core/src/pipeline/inbound_hold.rs
?? ../../crates/sieve-core/src/pipeline/inbound_hook.rs
?? ../../crates/sieve-core/src/pipeline/outbound_redact.rs
?? ../../crates/sieve-hook/
?? ../../crates/sieve-ipc/
?? ../design/ADR-012-native-gui-app-phase1.md
?? ../design/ADR-013-ipc-protocol.md
?? ../design/ADR-014-dual-layer-defense.md
?? ../design/ADR-015-sieve-setup-tool.md
?? ../design/ADR-016-disposition-matrix-2d.md
?? ../design/ADR-018-openai-protocol-adaptation.md
?? ../design/ADR-019-x-sieve-origin-header.md
?? ../prd/sieve-prd-v1.4.md
?? ../prd/sieve-prd-v1.5.md
?? ../specs/
?? ../../librust_out.rlib
?? ../../tasks/known-issues-v1.4.md
?? ../../tasks/todo.md

exec
/bin/zsh -lc "git rev-parse --show-toplevel && git diff --stat && git diff --cached --stat && find ../.. -maxdepth 4 -type f -not -path '../../.git/*' | sed 's#"'^../../##'"' | sort | grep -E '"'^(crates|docs|tasks|Cargo|README|SECURITY|CLAUDE|'"\\.cursorrules|librust_out)' | head -200" in /Users/doskey/src/sieve/docs/review
 succeeded in 592ms:
.cursorrules
CLAUDE.md
Cargo.lock
Cargo.toml
README.md
SECURITY.md
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
docs/.DS_Store
docs/api/api-reference.md
docs/changelog/CHANGELOG.md
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
docs/design/ADR-018-openai-protocol-adaptation.md
docs/design/ADR-019-x-sieve-origin-header.md
docs/design/ADR-INDEX.md
docs/design/architecture.md
docs/design/data-model.md
docs/glossary.md
docs/guides/deployment.md
docs/guides/development.md
docs/prd/sieve-prd-v1.0.md
docs/prd/sieve-prd-v1.1.md
docs/prd/sieve-prd-v1.2.md
docs/prd/sieve-prd-v1.3.md
docs/prd/sieve-prd-v1.4.md
docs/prd/sieve-prd-v1.5.md
docs/requirements/PRD-sieve.md
docs/requirements/user-stories.md
docs/research/deep-research-report.md
docs/review/2026-04-27-codex-week-1-3-implementation-review.md
docs/review/2026-04-28-codex-review-v1.4-r2.log
docs/review/2026-04-28-codex-review-v1.4-r3.log
docs/review/2026-04-28-codex-review-v1.4.log
docs/specs/SPEC-001-sieve-hook-protocol.md
docs/specs/SPEC-002-hips-popup-behavior.md
docs/specs/SPEC-003-sieve-setup-tool.md
docs/specs/SPEC-004-multi-agent-setup.md
librust_out.rlib
tasks/known-issues-v1.4.md
tasks/lessons.md
tasks/roadmap.md
tasks/todo.md

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/audit.rs crates/sieve-cli/src/config.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/detection.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/manifest.rs Cargo.toml crates/sieve-cli/Cargo.toml crates/sieve-core/Cargo.toml | sed -n '1,240p'" in /Users/doskey/src/sieve/docs/review
 succeeded in 0ms:
exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/audit.rs crates/sieve-cli/src/config.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/detection.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/manifest.rs Cargo.toml crates/sieve-cli/Cargo.toml crates/sieve-core/Cargo.toml | sed -n '1,260p'" in /Users/doskey/src/sieve
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
index 24b05c8..673aa7b 100644
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
@@ -44,6 +47,9 @@ http-body-util = { workspace = true }
 http = { workspace = true }
 bytes = { workspace = true }
 sieve-core = { path = "../sieve-core" }
+sieve-ipc = { path = "../sieve-ipc" }
 anyhow = "1"
 tempfile = "3"
 serde_json = { workspace = true }
+rusqlite = { version = "0.31", features = ["bundled"] }
+uuid = { version = "1", features = ["v4"] }
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

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/audit.rs crates/sieve-cli/src/config.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/detection.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/manifest.rs Cargo.toml crates/sieve-cli/Cargo.toml crates/sieve-core/Cargo.toml | sed -n '260,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
index 350dc84..a26030a 100644
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
@@ -57,6 +76,75 @@ pub struct Config {
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
@@ -87,6 +175,13 @@ fn default() -> Self {
             sieveignore_path: None,
             dry_run: false,
             inbound_rules_path: None,
+            ipc_socket_path: default_ipc_socket(),

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/config.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-cli/src/main.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/manifest.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
diff --git a/crates/sieve-cli/src/config.rs b/crates/sieve-cli/src/config.rs
index 350dc84..a26030a 100644
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
@@ -57,6 +76,75 @@ pub struct Config {
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
@@ -87,6 +175,13 @@ fn default() -> Self {
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
@@ -175,16 +270,20 @@ pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
             .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
     }
 
-    /// 解析审计日志路径。`log_path` 显式给定时直接用,否则回退到 `~/.sieve/audit.db`。
+    /// 解析审计日志路径。优先级：`audit_db_path` > `log_path` > `~/.sieve/audit.db`。
     ///
     /// # Errors
     /// `$HOME` 不存在或不可识别时返回错误。
     pub fn audit_db_path(&self) -> Result<PathBuf> {
+        if let Some(p) = &self.audit_db_path {
+            return Ok(p.clone());
+        }
         if let Some(p) = &self.log_path {
             return Ok(p.clone());
         }
-        let home = std::env::var_os("HOME")
-            .ok_or_else(|| anyhow!("HOME env var not set; specify log_path explicitly"))?;
+        let home = std::env::var_os("HOME").ok_or_else(|| {
+            anyhow!("HOME env var not set; specify audit_db_path or log_path explicitly")
+        })?;
         Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
     }
 }
@@ -294,4 +393,52 @@ fn resolved_sieveignore_path_explicit() {
             PathBuf::from("/my/.sieveignore")
         );
     }
+
+    // ── R2-#6 audit_db_path 优先级链测试 ────────────────────────────────────
+
+    #[test]
+    fn audit_db_path_explicit_field_wins() {
+        // audit_db_path 字段优先于 log_path 和默认值
+        let toml_str = r#"
+            upstream_url = "https://api.anthropic.com"
+            port = 11453
+            audit_db_path = "/custom/audit.db"
+            log_path = "/old/log.db"
+        "#;
+        let c: Config = toml::from_str(toml_str).unwrap();
+        let path = c.audit_db_path().unwrap();
+        assert_eq!(
+            path,
+            PathBuf::from("/custom/audit.db"),
+            "audit_db_path 字段应优先于 log_path"
+        );
+    }
+
+    #[test]
+    fn audit_db_path_falls_back_to_log_path() {
+        // 没有 audit_db_path 时应回退到 log_path
+        let toml_str = r#"
+            upstream_url = "https://api.anthropic.com"
+            port = 11453
+            log_path = "/old/log.db"
+        "#;
+        let c: Config = toml::from_str(toml_str).unwrap();
+        let path = c.audit_db_path().unwrap();
+        assert_eq!(path, PathBuf::from("/old/log.db"), "应回退到 log_path");
+    }
+
+    #[test]
+    fn audit_db_path_falls_back_to_default() {
+        // 两个字段都没有时，应回退到 ~/.sieve/audit.db
+        // 假设 HOME 已设置（CI 环境通常有）
+        if std::env::var_os("HOME").is_none() {
+            return; // HOME 未设置时跳过
+        }
+        let c = Config::default();
+        let path = c.audit_db_path().unwrap();
+        assert!(
+            path.ends_with(".sieve/audit.db"),
+            "默认路径应以 .sieve/audit.db 结尾，实际: {path:?}"
+        );
+    }
 }
diff --git a/crates/sieve-cli/src/daemon.rs b/crates/sieve-cli/src/daemon.rs
index aed965a..fc6c54d 100644
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

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/daemon.rs | sed -n '80,360p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
 
@@ -192,11 +235,35 @@ async fn proxy_inner(
         }
 
         // 4. 决策：
-        //    - fail-closed Critical 规则：无视 dry_run，永远 block（PRD §9 #3）
-        //    - 非 fail-closed Critical：dry_run=true 时仅 warn，dry_run=false 时 block
+        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
+        //    b. fail-closed Critical Block → 426（PRD §9 #3）
+        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
+        //    d. GuiPopup（Action::HoldForDecision）→ hold HTTP 长连接等 GUI 决策（R2-#1）
+        //    e. 其余 → 透传
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
@@ -214,6 +281,165 @@ async fn proxy_inner(
             return Ok(build_426_response(&cloned));
         }
 
+        // 4d. 出站 GuiPopup（HoldForDecision）：hold HTTP 长连接等待 GUI 决策（R2-#1 修复）。
+        //
+        // 出站请求是非流式 HTTP：body 已 collect，无需 SSE keep-alive（入站才需要）。
+        // 客户端等待期间持有普通 HTTP 长连接（reqwest / Claude Code client 的超时决定等待上限）。
+        //
+        // 决策映射：
+        //   Allow → 原 body 转发上游
+        //   RedactAndAllow → redact_hits 非空则脱敏，否则原 body 转发
+        //   Deny → 426 拒绝
+        //   超时 → 按 default_on_timeout（OUT-06/08 = Redact，OUT-07/09/10 = Block）
+        //
+        // 关联：PRD v1.4 §5.4.2 出站超时策略表、ADR-016（二维处置矩阵）。
+        let hold_detections_outbound: Vec<&sieve_core::Detection> = all_detections
+            .iter()
+            .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
+            .collect();
+
+        if !hold_detections_outbound.is_empty() {
+            if let Some(ref ipc_server) = ipc {
+                use chrono::Utc;
+
+                let request_id = uuid::Uuid::new_v4();
+                let (timeout_seconds, default_on_timeout) = hold_detections_outbound
+                    .iter()
+                    .find_map(|d| {
+                        if let Action::HoldForDecision {
+                            timeout_seconds, ..
+                        } = d.action
+                        {
+                            // 取第一个 HoldForDecision detection 的规则 timeout/default
+                            // default_on_timeout 从 detection 的 rule_id 对应规则读取，
+                            // 此处用 Block 作为保守默认（规则未设则 fail-closed）
+                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
+                        } else {
+                            None
+                        }
+                    })
+                    .unwrap_or((60, sieve_ipc::DefaultOnTimeout::Block));
+
+                let ipc_detections = hold_detections_outbound
+                    .iter()
+                    .map(|d| sieve_ipc::protocol::DetectionPayload {
+                        rule_id: d.rule_id.clone(),
+                        severity: map_severity_to_ipc(d.severity),
+                        disposition: sieve_ipc::Disposition::GuiPopup,
+                        title: format!("出站检测命中：{}", d.rule_id),
+                        one_line_summary: d.evidence_truncated.clone(),
+                        details: serde_json::json!({}),
+                    })
+                    .collect();
+
+                let ipc_req = sieve_ipc::DecisionRequest {
+                    request_id,
+                    created_at: Utc::now(),
+                    timeout_seconds,
+                    default_on_timeout,
+                    detections: ipc_detections,
+                };
+
+                // 出站 hold：无 SSE keep-alive，直接 await 决策
+                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
+                let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;
+
+                match outcome {
+                    Ok(resp) => match resp.decision {
+                        sieve_ipc::DecisionAction::Allow => {
+                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
+                            // 继续往下，走正常转发路径
+                        }
+                        sieve_ipc::DecisionAction::RedactAndAllow => {
+                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
+                            // 若有 redact_hits 则脱敏，否则原 body 转发（与 Allow 同逻辑）
+                            // 直接 fall-through 到下方 redact_hits 处理
+                        }
+                        sieve_ipc::DecisionAction::Deny => {
+                            tracing::warn!("OUTBOUND GUI: Deny → 426");
+                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
+                                .iter()
+                                .map(|d| (*d).clone())
+                                .collect();
+                            return Ok(build_426_response(&held));
+                        }
+                    },
+                    Err(e) => {
+                        // IPC 错误：按 default_on_timeout 兜底（fail-closed）
+                        tracing::warn!(error = %e, "OUTBOUND GUI: IPC error, fail-closed → 426");
+                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
+                            .iter()
+                            .map(|d| (*d).clone())
+                            .collect();
+                        return Ok(build_426_response(&held));
+                    }
+                }
+            } else {
+                // IPC 未初始化：fail-closed → 426
+                tracing::warn!("OUTBOUND GUI: IPC not initialized, fail-closed → 426");
+                let held: Vec<sieve_core::Detection> = hold_detections_outbound
+                    .iter()
+                    .map(|d| (*d).clone())
+                    .collect();
+                return Ok(build_426_response(&held));
+            }
+        }
+
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
@@ -224,9 +450,7 @@ async fn proxy_inner(
             }
         }
 
-        // 5. prompt 地址 seed：把出站 prompt 中的 EVM 地址预先注入 InboundFilter 会话，
-        //    使首轮地址替换（prompt 地址 A → 响应地址 B）可被 IN-CR-01 检测。
-        //    关联 PRD §4.2 / P0-3 修复。
+        // 5. prompt 地址 seed
         for (_, text) in &texts {
             if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                 tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
@@ -238,6 +462,7 @@ async fn proxy_inner(
             forwarder,
             inbound_filter,
             dry_run,
+            ipc,
             parts,
             body_bytes,
         )

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/daemon.rs | sed -n '360,760p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
         )
@@ -251,15 +476,21 @@ async fn proxy_inner(
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
@@ -286,13 +517,11 @@ async fn forward_with_inbound_inspection(
 
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
@@ -308,10 +537,9 @@ async fn forward_with_inbound_inspection(
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
@@ -329,13 +557,21 @@ async fn forward_with_inbound_inspection(
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
@@ -343,17 +579,135 @@ async fn forward_with_inbound_inspection(
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
+                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
+                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
+                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
+                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。
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
+                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
+                                    // 然后继续转发后续 SSE。
+                                    if tx
+                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
+                                        .await
+                                        .is_err()
+                                    {
+                                        return;
+                                    }
+                                    continue;
+                                }
+                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
+                                    // 修 R2-#3：用户拒绝时不发触发帧，直接注入 sieve_blocked 并关流。
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
@@ -367,11 +721,15 @@ async fn forward_with_inbound_inspection(
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
@@ -379,7 +737,23 @@ async fn forward_with_inbound_inspection(
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
 
@@ -391,33 +765,34 @@ async fn forward_with_inbound_inspection(
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
@@ -425,25 +800,99 @@ fn collect_blocking_detections(
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

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/daemon.rs | sed -n '760,1120p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
@@ -537,9 +986,6 @@ async fn forward_streaming(
 }
 
 /// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
-///
-/// body 为 JSON，含命中规则 ID / fingerprint / 操作指引。
-/// 时间戳当前为 UNIX epoch 秒，Week 4 引入 chrono 后改为完整 RFC3339。
 fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
     let blocked_at = epoch_secs_string();
     let detections_json: Vec<serde_json::Value> = detections
@@ -610,8 +1056,6 @@ fn empty_body() -> ResponseBody {
 }
 
 /// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
-///
-/// 畸形 partial_json 不对应具体文本 span，evidence_truncated 存 tool_id。
 fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
@@ -629,8 +1073,6 @@ fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
 }
 
 /// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
-///
-/// 容量超限不对应具体文本 span，因此 span 设 [0, 0)，evidence_truncated 为空。
 fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
     use sieve_core::detection::{Action, ContentSource};
     use sieve_core::protocol::unified_message::ContentSpan;
@@ -646,3 +1088,127 @@ fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Dete
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

exec
/bin/zsh -lc "sed -n '1,160p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
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
use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
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

exec
/bin/zsh -lc "find crates/sieve-ipc crates/sieve-hook crates/sieve-cli/src/commands crates/sieve-core/src/pipeline -maxdepth 3 -type f | sort | xargs -n1 sh -c 'echo --- "'$0; sed -n "1,240p" "$0"'"'" in /Users/doskey/src/sieve
 succeeded in 130ms:
--- crates/sieve-ipc/src/decision_file.rs
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{decisions_dir, ensure_dirs, locks_dir},
    protocol::{DecisionAction, DecisionResponse},
};

/// 将 [`DecisionResponse`] 写入 `<base>/decisions/<request_id>.json`。
///
/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁，确保并发写入安全
///（hook 与 GUI 极少同时操作同一 request_id，但防御性加锁是正确做法）。
///
/// 关联：SPEC-001 §3.3（决策文件写入规约）。
pub fn write_decision(resp: &DecisionResponse, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let lock_path = locks_dir(base).join(format!("{}.lock", resp.request_id));
    let dec_path = decisions_dir(base).join(format!("{}.json", resp.request_id));

    // 创建锁文件（若不存在），然后加独占写锁。
    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;

    let mut lock = RwLock::new(lock_file);
    {
        let _guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;

        let json = serde_json::to_string_pretty(resp)?;
        std::fs::write(&dec_path, json.as_bytes())?;
    }

    // decisions 写入成功后，清理对应的 pending 文件。
    // 删除失败不是致命错误（竞争/权限），仅打 warning，不向上返回错误。
    // Unix 上 unlink 不受 fd-lock 影响，可安全删除。
    // 关联：SPEC-001 §4.3（清理机制）。
    let pending_path = crate::paths::pending_dir(base).join(format!("{}.json", resp.request_id));
    if let Err(e) = std::fs::remove_file(&pending_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "sieve-ipc: warning: failed to remove pending file {}: {e}",
                pending_path.display()
            );
        }
    }

    Ok(dec_path)
}

/// 轮询等待 `<base>/decisions/<request_id>.json` 出现并读取。
///
/// 轮询间隔 50 ms，对 30–120 s 的用户响应超时来说 CPU 开销可忽略。
/// 选择轮询而非 inotify/notify 是为了跨平台简单性；Phase 1 仅 macOS，
/// 但未来 Linux 支持时轮询同样生效，不需要额外适配。
///
/// 超时后按 `default_on_timeout` 构造兜底响应。关联：ADR-013 §4.2。
pub async fn wait_for_decision(
    request_id: Uuid,
    base: &Path,
    timeout: Duration,
    default_on_timeout: crate::protocol::DefaultOnTimeout,
) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let deadline = tokio::time::Instant::now() + timeout;
    let poll_interval = Duration::from_millis(50);

    loop {
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let resp: DecisionResponse = serde_json::from_str(&content)?;
            return Ok(resp);
        }

        if tokio::time::Instant::now() >= deadline {
            // 超时：按 default_on_timeout 构造兜底响应。
            let action = match default_on_timeout {
                crate::protocol::DefaultOnTimeout::Block => DecisionAction::Deny,
                crate::protocol::DefaultOnTimeout::Allow => DecisionAction::Allow,
                crate::protocol::DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
            };
            return Ok(DecisionResponse {
                request_id,
                decision: action,
                decided_at: Utc::now(),
                by_user: false,
                remember: false,
            });
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// 同步版读取决策文件（hook 侧使用，不依赖 tokio）。
pub fn read_decision(request_id: Uuid, base: &Path) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            // 不存在时通过 IpcError::PendingNotFound 复用（语义相近）
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let resp: DecisionResponse = serde_json::from_str(&content)?;
    Ok(resp)
}
--- crates/sieve-ipc/src/error.rs
use thiserror::Error;

/// IPC 层错误枚举。
///
/// 关联规格：ADR-013（IPC 协议）、SPEC-001（sieve-hook 文件协议）。
#[derive(Debug, Error)]
pub enum IpcError {
    /// Unix socket 绑定或连接失败。
    #[error("socket error: {0}")]
    Socket(#[from] std::io::Error),

    /// JSON 序列化 / 反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 请求在规定超时内未收到决策响应。
    #[error("decision timeout for request {request_id}")]
    Timeout { request_id: uuid::Uuid },

    /// pending 文件已超过 stale 阈值（10 分钟），视为过期拒绝。
    ///
    /// fail-closed：过期请求不允许放行，防止残留文件被重放。
    #[error("pending file is stale (created_at too old) for request {request_id}")]
    StalePending { request_id: uuid::Uuid },

    /// pending 文件不存在——此请求未经代理标记，可 fail-open。
    #[error("pending file not found for request {request_id}")]
    PendingNotFound { request_id: uuid::Uuid },

    /// 文件加锁失败。
    #[error("file lock error: {0}")]
    FileLock(String),

    /// $HOME 环境变量缺失，无法确定 sieve_home 路径。
    #[error("$HOME environment variable is not set")]
    HomeNotFound,

    /// JSON-RPC 响应中携带了错误对象。
    #[error("json-rpc error {code}: {message}")]
    JsonRpcError { code: i64, message: String },

    /// 对端发送了无法识别的 JSON-RPC method 或响应格式异常。
    #[error("unexpected json-rpc response: {0}")]
    UnexpectedResponse(String),
}
--- crates/sieve-ipc/src/lib.rs
// sieve-ipc: JSON-RPC 2.0 over Unix socket + pending/decision 文件协议库。
//
// 供 sieve-cli（主代理）调用，向 GUI（sieve-gui-macos）或 hook（sieve-hook）
// 传递决策请求并等待响应。关联：ADR-013（IPC 协议）、ADR-014（双层防御）。

pub mod decision_file;
pub mod error;
pub mod paths;
pub mod pending_file;
pub mod protocol;
pub mod socket_client;
pub mod socket_server;

// 常用类型直接 re-export，调用方无需深层 import。
pub use error::IpcError;
pub use protocol::{
    DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout, DetectionPayload,
    Disposition, Severity,
};
pub use socket_server::IpcServer;

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::protocol::*;

    // ── 协议 round-trip ──────────────────────────────────────────────────────

    #[test]
    fn decision_request_round_trip() {
        let req = DecisionRequest {
            request_id: Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::HookTerminal,
                title: "私钥检测".to_owned(),
                one_line_summary: "检测到 BIP39 助记词（12 词，checksum 通过）".to_owned(),
                details: serde_json::json!({ "word_count": 12 }),
            }],
        };

        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: DecisionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, req.request_id);
        assert_eq!(decoded.detections[0].rule_id, "IN-CR-01");
        assert_eq!(decoded.default_on_timeout, DefaultOnTimeout::Block);
    }

    #[test]
    fn decision_response_round_trip() {
        let resp = DecisionResponse {
            request_id: Uuid::now_v7(),
            decision: DecisionAction::Deny,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
        };

        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: DecisionResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, resp.request_id);
        assert_eq!(decoded.decision, DecisionAction::Deny);
        assert!(decoded.by_user);
        assert!(!decoded.remember);
    }

    #[test]
    fn disposition_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Disposition::GuiPopup).unwrap(),
            "\"gui_popup\""
        );
        assert_eq!(
            serde_json::to_string(&Disposition::HookTerminal).unwrap(),
            "\"hook_terminal\""
        );
    }

    #[test]
    fn severity_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Severity::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn decision_action_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&DecisionAction::RedactAndAllow).unwrap(),
            "\"redact_and_allow\""
        );
    }

    // ── jsonrpc envelope ────────────────────────────────────────────────────

    #[test]
    fn jsonrpc_request_omits_null_id() {
        let req = jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: "ping".to_owned(),
            params: None,
            id: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // 通知请求不携带 id 字段。
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn jsonrpc_call_includes_id() {
        let req = jsonrpc::Request::call(
            "request_decision",
            serde_json::json!({}),
            serde_json::Value::String("abc".to_owned()),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"request_decision\""));
    }
}

#[cfg(test)]
mod file_tests {
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    use super::{
        decision_file::{wait_for_decision, write_decision},
        pending_file::{read_pending, write_pending},
        protocol::*,
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
        }
    }

    // ── pending_file ─────────────────────────────────────────────────────────

    #[test]
    fn pending_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_request(id);

        let path = write_pending(&req, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = read_pending(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    #[test]
    fn pending_not_found_error() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let err = read_pending(id, tmp.path()).unwrap_err();
        assert!(matches!(err, crate::IpcError::PendingNotFound { .. }));
    }

    #[test]
    fn pending_file_lock_two_tasks() {
        // 两个线程抢同一个 pending 文件——后者等前者释放锁后写入。
        // 验证不出现数据损坏（最终文件可被正确解析）。
        use std::sync::Arc;
        use std::sync::Barrier;
        use std::thread;

        let tmp = tempfile::tempdir().unwrap();
        let base = Arc::new(tmp.path().to_owned());
        let id = Uuid::now_v7();
        let barrier = Arc::new(Barrier::new(2));

        let base1 = Arc::clone(&base);
        let barrier1 = Arc::clone(&barrier);
        let t1 = thread::spawn(move || {
            barrier1.wait();
            let req = make_request(id);
            write_pending(&req, &base1).unwrap();
        });

        let base2 = Arc::clone(&base);
        let barrier2 = Arc::clone(&barrier);
        let t2 = thread::spawn(move || {
            barrier2.wait();
            let req = make_request(id);
            write_pending(&req, &base2).unwrap();
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // 文件仍可被正确解析（两次写入串行化）。
        let read_back = read_pending(id, &base).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    // ── decision_file ────────────────────────────────────────────────────────

    #[test]
    fn decision_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let resp = DecisionResponse {
            request_id: id,
            decision: DecisionAction::Allow,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
        };

        let path = write_decision(&resp, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = super::decision_file::read_decision(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
        assert_eq!(read_back.decision, DecisionAction::Allow);
    }

    #[tokio::test]
    async fn wait_for_decision_timeout_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 极短超时，不写决策文件，应返回 Block（default_on_timeout = Block）。
        let resp = wait_for_decision(
            id,
            tmp.path(),
--- crates/sieve-ipc/src/paths.rs
use std::path::PathBuf;

use crate::error::IpcError;

/// 计算 sieve home 目录。
///
/// 优先级：`$SIEVE_HOME` 环境变量 > `$HOME/.sieve`。
/// $HOME 缺失时返回 [`IpcError::HomeNotFound`]。
///
/// 关联：SPEC-001 §2.1（目录结构）。
pub fn sieve_home() -> Result<PathBuf, IpcError> {
    if let Ok(val) = std::env::var("SIEVE_HOME") {
        return Ok(PathBuf::from(val));
    }
    let home = std::env::var("HOME").map_err(|_| IpcError::HomeNotFound)?;
    Ok(PathBuf::from(home).join(".sieve"))
}

/// `<sieve_home>/pending/` 目录，存放主代理写入的待决策文件。
pub fn pending_dir(base: &std::path::Path) -> PathBuf {
    base.join("pending")
}

/// `<sieve_home>/decisions/` 目录，存放 hook/GUI 写入的决策文件。
pub fn decisions_dir(base: &std::path::Path) -> PathBuf {
    base.join("decisions")
}

/// `<sieve_home>/locks/` 目录，存放文件锁占位符。
pub fn locks_dir(base: &std::path::Path) -> PathBuf {
    base.join("locks")
}

/// `<sieve_home>/ipc.sock` Unix socket 路径（主代理监听，GUI 连接）。
pub fn ipc_socket_path(base: &std::path::Path) -> PathBuf {
    base.join("ipc.sock")
}

/// 确保所有子目录存在，不存在时递归创建。
///
/// 幂等——多次调用安全。
pub fn ensure_dirs(base: &std::path::Path) -> Result<(), IpcError> {
    for dir in [pending_dir(base), decisions_dir(base), locks_dir(base)] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
--- crates/sieve-ipc/src/pending_file.rs
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
--- crates/sieve-ipc/src/protocol.rs
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
--- crates/sieve-ipc/src/socket_client.rs
use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

use crate::{
    error::IpcError,
    protocol::{jsonrpc, DecisionResponse},
};

/// 测试 / mock GUI 用的 IPC 客户端。
///
/// 连接服务端 socket，发送 JSON-RPC response（模拟 GUI 完成决策后的回调）。
/// 不在生产主路径使用——主路径的 GUI 是独立进程（sieve-gui-macos）。
///
/// 关联：ADR-013 §3（协议传输）。
pub struct IpcClient {
    socket_path: std::path::PathBuf,
}

impl IpcClient {
    /// 创建指向 `socket_path` 的客户端（不立即连接）。
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_owned(),
        }
    }

    /// 向服务端发送一个 [`DecisionResponse`]（换行分隔 JSON-RPC response 格式）。
    pub async fn send_decision(&self, resp: &DecisionResponse) -> Result<(), IpcError> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        let rpc_resp = jsonrpc::Response {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::to_value(resp)?),
            error: None,
            id: serde_json::Value::String(resp.request_id.to_string()),
        };
        let mut payload = serde_json::to_string(&rpc_resp)?;
        payload.push('\n');
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }

    /// 从 socket 读取一条换行分隔的 JSON-RPC request（服务端推来的决策请求）。
    ///
    /// 主要用于 mock GUI 侧读取请求并回复。
    pub async fn recv_request(&self) -> Result<serde_json::Value, IpcError> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        let line = lines.next_line().await?.ok_or_else(|| {
            IpcError::UnexpectedResponse("connection closed without data".to_owned())
        })?;
        let val: serde_json::Value = serde_json::from_str(&line)?;
        Ok(val)
    }

    /// 等待来自服务端的决策请求，自动回复指定的决策动作（测试辅助）。
    pub async fn auto_respond(
        socket_path: impl AsRef<Path>,
        request_id: Uuid,
        decision: crate::protocol::DecisionAction,
    ) -> Result<(), IpcError> {
        let path = socket_path.as_ref().to_owned();
        // 短暂重试以等待服务端就绪。
        let stream = retry_connect(&path, 5, std::time::Duration::from_millis(20)).await?;
        let (reader_half, mut writer_half) = stream.into_split();
        let mut lines = BufReader::new(reader_half).lines();

        // 读一条请求（忽略内容，只要 request_id 匹配就回）。
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let resp = DecisionResponse {
                request_id,
                decision,
                decided_at: chrono::Utc::now(),
                by_user: true,
                remember: false,
            };
            let rpc_resp = jsonrpc::Response {
                jsonrpc: "2.0".to_owned(),
                result: Some(serde_json::to_value(&resp)?),
                error: None,
                id: serde_json::Value::String(request_id.to_string()),
            };
            let mut payload = serde_json::to_string(&rpc_resp)?;
            payload.push('\n');
            writer_half.write_all(payload.as_bytes()).await?;
            break;
        }
        Ok(())
    }
}

/// 连接重试辅助——服务端 spawn 后稍有延迟才就绪。
async fn retry_connect(
    path: &std::path::Path,
    attempts: u32,
    delay: std::time::Duration,
) -> Result<UnixStream, IpcError> {
    let mut last_err = None;
    for _ in 0..attempts {
        match UnixStream::connect(path).await {
            Ok(s) => return Ok(s),
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(delay).await;
            }
        }
    }
    Err(IpcError::Socket(last_err.unwrap()))
}
--- crates/sieve-ipc/src/socket_server.rs
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::IpcError,
    protocol::{DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout},
};

/// pending map：request_id → oneshot 发送端，等待 GUI 回复。
type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;

/// GUI 客户端的写通道：向其发送换行分隔的 JSON 字符串即可推送到对端。
///
/// 使用 mpsc 而非直接持有 WriteHalf，这样写检测（`send` 失败）就能代替
/// TCP keepalive 检测 GUI 进程崩溃。通道容量设为 32，满了则视为 GUI 卡死。
type GuiWriter = Arc<Mutex<Option<mpsc::Sender<String>>>>;

/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
///
/// # 连接语义
///
/// - GUI 启动后主动连接此 socket，保持长连接。
/// - 同一时刻只允许一个 GUI 客户端（多连接时拒绝第二个，记录警告）。
/// - GUI 断线后 `gui_writer` 自动清空；下一次 `request_decision` 立即 fallback。
///
/// # 双向通信模型
///
/// ```text
/// [主代理]  ─request_decision JSON-RPC request─▶  [GUI]
/// [主代理]  ◀─decision_response JSON-RPC response─  [GUI]
/// ```
///
/// 每个方向在同一条 TCP/Unix 连接上用换行分隔的 JSON-RPC 帧传输。
/// `handle_connection` 负责从 GUI 读取响应帧并派发到 `pending` map；
/// `request_decision` 通过 `gui_writer` mpsc 通道写入请求帧。
///
/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
pub struct IpcServer {
    socket_path: PathBuf,
    pending: PendingMap,
    /// 当前已连接的 GUI 客户端写通道；无 GUI 时为 None。
    gui_writer: GuiWriter,
}

impl IpcServer {
    /// 绑定 Unix socket 并返回服务端实例。
    ///
    /// socket_path 已存在时先删除旧文件（daemon 重启场景）。
    pub fn bind(socket_path: PathBuf) -> Result<(Self, UnixListener), IpcError> {
        // 旧 socket 文件存在则先删除，否则 bind 会失败。
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        let server = Self {
            socket_path,
            pending: Arc::new(Mutex::new(HashMap::new())),
            gui_writer: Arc::new(Mutex::new(None)),
        };
        Ok((server, listener))
    }

    /// 运行 accept 循环，处理来自 GUI 的长连接。
    ///
    /// 每个连接独立 spawn；同一时刻只接受一个 GUI 客户端，多余的直接关闭。
    pub async fn run(&self, listener: UnixListener) {
        info!(socket = %self.socket_path.display(), "IPC server listening");
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let pending = Arc::clone(&self.pending);
                    let gui_writer = Arc::clone(&self.gui_writer);

                    // 检查是否已有 GUI 客户端。
                    // 用 try_lock 避免阻塞 accept 循环；如果锁被占用就放通并让
                    // handle_connection 内部处理（竞态概率极低）。
                    {
                        let mut guard = gui_writer.lock().await;
                        if guard.is_some() {
                            warn!("second GUI client attempted to connect; rejecting");
                            // 直接 drop stream 关闭连接，不 spawn 处理。
                            drop(stream);
                            continue;
                        }
                        // 还没有 GUI 客户端——创建 mpsc 通道，把发送端存入 gui_writer，
                        // 接收端传给 handle_connection 用于写回 GUI。
                        let (tx, rx) = mpsc::channel::<String>(32);
                        *guard = Some(tx);
                        drop(guard);

                        tokio::spawn(async move {
                            if let Err(e) =
                                handle_connection(stream, pending, gui_writer.clone(), rx).await
                            {
                                error!("IPC connection error: {e}");
                            }
                            // 连接断开后清理 gui_writer，下一个 GUI 可以重连。
                            let mut w = gui_writer.lock().await;
                            *w = None;
                            info!("GUI client disconnected; gui_writer cleared");
                        });
                    }
                }
                Err(e) => {
                    error!("IPC accept error: {e}");
                    break;
                }
            }
        }
    }

    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
    ///
    /// # 行为
    ///
    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
    ///   （等超时无意义——没人能决策。）
    /// - 如果 GUI 写通道已满或 GUI 进程崩溃（mpsc send 失败）：立即 fallback。
    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
    pub async fn request_decision(
        &self,
        req: DecisionRequest,
        timeout: Duration,
    ) -> Result<DecisionResponse, IpcError> {
        let request_id = req.request_id;
        let default_on_timeout = req.default_on_timeout;

        // 1. 检查 GUI 是否已连接。
        let sender = {
            let guard = self.gui_writer.lock().await;
            guard.clone()
        };

        let Some(sender) = sender else {
            // 没有 GUI——立即 fallback，不消耗超时时间。
            debug!(%request_id, "no GUI client connected; immediate fallback");
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        };

        // 2. 注册 oneshot channel，等待 GUI 回复。
        let (tx, rx) = oneshot::channel::<DecisionResponse>();
        {
            let mut map = self.pending.lock().await;
            map.insert(request_id, tx);
        }

        // 3. 通过 mpsc 通道把请求推到 handle_connection 的写循环，
        //    再由写循环写入真正的 GUI socket 连接。
        let rpc_req = crate::protocol::jsonrpc::Request::call(
            "request_decision",
            serde_json::to_value(&req)?,
            serde_json::Value::String(request_id.to_string()),
        );
        let mut payload = serde_json::to_string(&rpc_req)?;
        payload.push('\n');

        if let Err(_e) = sender.send(payload).await {
            // GUI 写通道关闭（GUI 进程崩溃或通道满），立即 fallback。
            warn!(%request_id, "GUI writer channel closed; immediate fallback");
            self.pending.lock().await.remove(&request_id);
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 4. 等待 GUI 回复或超时。
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => {
                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
            Err(_elapsed) => {
                // 超时，清理 pending map。
                self.pending.lock().await.remove(&request_id);
                warn!(%request_id, "decision timeout");
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
        }
    }

    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
    pub async fn inject_decision(&self, resp: DecisionResponse) {
        let mut map = self.pending.lock().await;
        if let Some(tx) = map.remove(&resp.request_id) {
            let _ = tx.send(resp);
        }
    }
}

/// 处理单个 GUI 长连接。
///
/// 同时管理两个方向：
/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map。
/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
///
/// 任一方向出错（GUI 断线 / 写失败）都会退出，调用方负责清理 `gui_writer`。
async fn handle_connection(
    stream: UnixStream,
    pending: PendingMap,
    gui_writer: GuiWriter,
    mut write_rx: mpsc::Receiver<String>,
) -> Result<(), IpcError> {
    info!("GUI client connected");

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    loop {
        tokio::select! {
            // 读方向：GUI 发来 decision_response。
            line_result = lines.next_line() => {
                match line_result? {
                    None => {
                        // GUI 关闭连接。
                        info!("GUI client closed connection");
                        break;
                    }
                    Some(line) => {
                        let line = line.trim().to_owned();
                        if line.is_empty() {
                            continue;
                        }
                        debug!(raw = %line, "received IPC message from GUI");
                        dispatch_response(&line, &pending).await;
                    }
                }
            }

            // 写方向：主代理 push request_decision 给 GUI。
            msg = write_rx.recv() => {

exec
/bin/zsh -lc 'find crates/sieve-ipc crates/sieve-hook crates/sieve-cli/src/commands -type f -maxdepth 3 -print' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-ipc/Cargo.toml
crates/sieve-ipc/src/pending_file.rs
crates/sieve-ipc/src/socket_server.rs
crates/sieve-ipc/src/protocol.rs
crates/sieve-ipc/src/error.rs
crates/sieve-ipc/src/decision_file.rs
crates/sieve-ipc/src/socket_client.rs
crates/sieve-ipc/src/lib.rs
crates/sieve-ipc/src/paths.rs
crates/sieve-hook/Cargo.toml
crates/sieve-hook/benches/hook_startup.rs
crates/sieve-hook/src/pending.rs
crates/sieve-hook/src/decision.rs
crates/sieve-hook/src/protocol.rs
crates/sieve-hook/src/error.rs
crates/sieve-hook/src/lib.rs
crates/sieve-hook/src/main.rs
crates/sieve-cli/src/commands/doctor.rs
crates/sieve-cli/src/commands/mod.rs
crates/sieve-cli/src/commands/uninstall.rs
crates/sieve-cli/src/commands/setup.rs

exec
/bin/zsh -lc 'cargo test --workspace --all-targets' in /Users/doskey/src/sieve
 exited 101 in 289ms:
error: failed to open: /Users/doskey/src/sieve/target/debug/.cargo-lock

Caused by:
  Operation not permitted (os error 1)

exec
/bin/zsh -lc 'CARGO_TARGET_DIR=/tmp/sieve-target cargo test --workspace --all-targets' in /Users/doskey/src/sieve
 exited 101 in 49226ms:
   Compiling libc v0.2.186
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling cfg-if v1.0.4
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling once_cell v1.21.4
   Compiling serde_core v1.0.228
   Compiling memchr v2.8.0
   Compiling bitflags v2.11.1
   Compiling serde v1.0.228
   Compiling itoa v1.0.18
   Compiling version_check v0.9.5
   Compiling autocfg v1.5.0
   Compiling pin-project-lite v0.2.17
   Compiling zmij v1.0.21
   Compiling serde_json v1.0.149
   Compiling zeroize v1.8.2
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling tracing-core v0.1.36
   Compiling generic-array v0.14.7
   Compiling num-traits v0.2.19
   Compiling smallvec v1.15.1
   Compiling bytes v1.11.1
   Compiling typenum v1.20.0
   Compiling parking_lot_core v0.9.12
   Compiling strsim v0.11.1
   Compiling zerocopy v0.8.48
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.0
   Compiling indexmap v2.14.0
   Compiling aho-corasick v1.1.4
   Compiling jobserver v0.1.34
   Compiling errno v0.3.14
   Compiling cc v1.2.61
   Compiling regex-syntax v0.8.10
   Compiling cmake v0.1.58
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling anstyle v1.0.14
   Compiling subtle v2.6.1
   Compiling signal-hook-registry v1.4.8
   Compiling socket2 v0.6.3
   Compiling regex-automata v0.4.14
   Compiling mio v1.2.0
   Compiling utf8parse v0.2.2
   Compiling thiserror v2.0.18
   Compiling dunce v1.0.5
   Compiling fs_extra v1.3.0
   Compiling aws-lc-sys v0.40.0
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling core-foundation-sys v0.8.7
   Compiling anstyle-query v1.1.5
   Compiling futures-core v0.3.32
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling iana-time-zone v0.1.65
   Compiling cpufeatures v0.2.17
   Compiling clap_lex v1.1.0
   Compiling crc32fast v1.5.0
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.0
   Compiling regex v1.12.3
   Compiling uuid v1.23.1
   Compiling simd-adler32 v0.3.9
   Compiling aws-lc-rs v1.16.3
   Compiling adler2 v2.0.1
   Compiling miniz_oxide v0.8.9
   Compiling filetime v0.2.27
   Compiling http v1.4.0
   Compiling rustls-pki-types v1.14.1
   Compiling fd-lock v4.0.4
   Compiling futures-sink v0.3.32
   Compiling semver v1.0.28
   Compiling slab v0.4.12
   Compiling rustc_version v0.4.1
   Compiling syn v2.0.117
   Compiling httparse v1.10.1
   Compiling fastrand v2.4.1
   Compiling curve25519-dalek v4.1.3
   Compiling tempfile v3.27.0
   Compiling http-body v1.0.1
   Compiling untrusted v0.9.0
   Compiling try-lock v0.2.5
   Compiling atomic-waker v1.1.2
   Compiling fnv v1.0.7
   Compiling rustls v0.23.39
   Compiling thiserror v1.0.69
   Compiling want v0.3.1
   Compiling futures-channel v0.3.32
   Compiling crossbeam-utils v0.8.21
   Compiling winnow v0.7.15
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling foreign-types-shared v0.3.1
   Compiling toml_write v0.1.2
   Compiling futures-task v0.3.32
   Compiling httpdate v1.0.3
   Compiling signature v2.2.0
   Compiling ed25519 v2.2.3
   Compiling webpki-roots v1.0.7
   Compiling tower-service v0.3.3
   Compiling webpki-roots v0.26.11
   Compiling http-body-util v0.1.3
   Compiling ahash v0.8.12
   Compiling either v1.15.0
   Compiling rayon-core v1.13.0
   Compiling parking_lot v0.12.5
   Compiling plotters-backend v0.3.7
   Compiling ciborium-io v0.2.2
   Compiling pkg-config v0.3.33
   Compiling serde_derive v1.0.228
   Compiling tracing-attributes v0.1.31
   Compiling zerocopy-derive v0.8.48
   Compiling tracing v0.1.44
   Compiling tokio-macros v2.7.0
   Compiling tokio v1.52.1
   Compiling thiserror-impl v2.0.18
   Compiling clap_derive v4.6.1
   Compiling xattr v1.6.1
   Compiling flate2 v1.1.9
   Compiling tar v0.4.45
   Compiling clap v4.6.1
   Compiling vectorscan-rs-sys v0.0.6
   Compiling foreign-types-macros v0.2.3
   Compiling futures-macro v0.3.32
   Compiling thiserror-impl v1.0.69
   Compiling futures-util v0.3.32
   Compiling foreign-types v0.5.0
   Compiling ed25519-dalek v2.2.0
   Compiling crossbeam-epoch v0.9.18
   Compiling vcpkg v0.2.15
   Compiling crossbeam-deque v0.8.6
   Compiling libsqlite3-sys v0.28.0
   Compiling plotters-svg v0.3.7
   Compiling itertools v0.10.5
   Compiling cast v0.3.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling plotters v0.3.7
   Compiling criterion-plot v0.5.0
   Compiling rayon v1.12.0
   Compiling chrono v0.4.44
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling half v2.7.1
   Compiling toml_edit v0.22.27
   Compiling ciborium-ll v0.2.2
   Compiling hashbrown v0.14.5
   Compiling tokio-util v0.7.18
   Compiling ciborium v0.2.2
   Compiling h2 v0.4.13
   Compiling toml v0.8.23
   Compiling sieve-ipc v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-ipc)
   Compiling tinytemplate v1.2.1
   Compiling is-terminal v0.4.17
   Compiling anes v0.1.6
   Compiling lazy_static v1.5.0
   Compiling anyhow v1.0.102
   Compiling oorandom v11.1.5
   Compiling log v0.4.29
   Compiling tracing-log v0.2.0
   Compiling criterion v0.5.1
   Compiling sharded-slab v0.1.7
   Compiling hashlink v0.9.1
   Compiling hyper v1.9.0
   Compiling matchers v0.2.0
   Compiling thread_local v1.1.9
   Compiling fallible-iterator v0.3.0
   Compiling nu-ansi-term v0.50.3
   Compiling fallible-streaming-iterator v0.1.9
   Compiling tokio-stream v0.1.18
   Compiling tracing-subscriber v0.3.23
   Compiling predicates-core v1.0.10
   Compiling float-cmp v0.10.0
   Compiling libfuzzer-sys v0.4.12
   Compiling normalize-line-endings v0.3.0
   Compiling difflib v0.4.0
   Compiling assert_cmd v2.2.1
   Compiling termtree v0.5.1
   Compiling predicates-tree v1.0.13
   Compiling predicates v3.1.4
   Compiling bstr v1.12.1
   Compiling wait-timeout v0.2.1
   Compiling arbitrary v1.4.2
   Compiling hyper-util v0.1.20
   Compiling sieve-hook v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-hook)
   Compiling rusqlite v0.31.0
   Compiling rustls-webpki v0.103.13
   Compiling tokio-rustls v0.26.4
   Compiling hyper-rustls v0.27.9
   Compiling sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
   Compiling sieve-fuzz v0.0.0 (/Users/doskey/src/sieve/fuzz)
   Compiling vectorscan-rs v0.0.6
   Compiling sieve-rules v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-rules)
   Compiling sieve-cli v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 47.07s
     Running unittests src/main.rs (/tmp/sieve-target/debug/deps/sieve-eea63f8e71059d6b)

running 33 tests
test commands::setup::tests::plist_contains_absolute_config_flag ... ok
test commands::setup::tests::setup_log_entry_created_new_serializes_correctly ... ok
test commands::setup::tests::default_sieve_toml_has_absolute_paths ... ok
test commands::setup::tests::bad_json_parse_returns_error_not_empty_object ... ok
test config::tests::audit_db_path_falls_back_to_default ... ok
test config::tests::defaults_are_sane ... ok
test config::tests::listen_addr_parses ... ok
test commands::uninstall::tests::uninstall_toml_created_new_true_deletes_file ... ok
test commands::uninstall::tests::uninstall_created_new_true_deletes_file ... ok
test config::tests::resolved_rules_path_explicit ... ok
test config::tests::resolved_rules_path_fallback ... ok
test config::tests::resolved_sieveignore_path_explicit ... ok
test commands::uninstall::tests::uninstall_created_new_false_removes_sieve_entries_only ... ok
test config::tests::parse_minimal_toml ... ok
test config::tests::unknown_field_rejected ... ok
test config::tests::audit_db_path_falls_back_to_log_path ... ok
test config::tests::parse_dry_run_and_rules_path ... ok
test config::tests::parse_full_toml ... ok
test config::tests::audit_db_path_explicit_field_wins ... ok
test engine_adapter::tests::map_action_warn_becomes_hook_mark ... ok
test engine_adapter::tests::redact_evidence_long ... ok
test engine_adapter::tests::redact_evidence_short ... ok
test commands::setup::tests::default_sieve_toml_parses_as_config ... ok
test commands::uninstall::tests::uninstall_toml_created_new_false_restores_from_backup ... ok
test audit::tests::update_trigger_blocks ... ok
test audit::tests::decision_event_stores_decision_field ... ok
test audit::tests::write_and_read_events ... ok
test engine_adapter::tests::scan_no_match_returns_empty ... ok
test engine_adapter::tests::disposition_hook_terminal_beats_enforce_action ... ok
test engine_adapter::tests::span_offset_applied ... ok
test engine_adapter::tests::disposition_auto_redact_beats_enforce_action ... ok
test engine_adapter::tests::disposition_gui_popup_beats_enforce_action ... ok
test engine_adapter::tests::scan_detects_pattern ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/audit_append_only.rs (/tmp/sieve-target/debug/deps/audit_append_only-7963e579460992e0)

running 3 tests
test update_is_rejected_by_trigger ... ok
test delete_is_rejected_by_trigger ... ok
test write_3_events_and_read_back ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/inbound_block.rs (/tmp/sieve-target/debug/deps/inbound_block-ebd9dd42f9aab019)

running 10 tests
test address_substitution_from_prompt_seed_blocks ... FAILED
test in_cr_03_sensitive_path_warn_passes_through ... FAILED
test malformed_tool_use_partial_json_blocks ... FAILED
test ucsb_attack_3_signing_tool_blocked ... FAILED
test ucsb_attack_2_dangerous_shell_hookmark_passthrough ... FAILED
test ucsb_attack_4_markdown_exfil_failclosed_without_gui ... FAILED
test ucsb_attack_1_address_substitution_blocked ... FAILED
test in_cr_04_persistence_shell_rc_hookmark_passthrough ... FAILED
test unterminated_final_event_still_blocks_critical ... FAILED
test benign_response_passes_through_unchanged ... FAILED

failures:

---- address_substitution_from_prompt_seed_blocks stdout ----

thread 'address_substitution_from_prompt_seed_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_03_sensitive_path_warn_passes_through stdout ----

thread 'in_cr_03_sensitive_path_warn_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- malformed_tool_use_partial_json_blocks stdout ----

thread 'malformed_tool_use_partial_json_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_3_signing_tool_blocked stdout ----

thread 'ucsb_attack_3_signing_tool_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_2_dangerous_shell_hookmark_passthrough stdout ----

thread 'ucsb_attack_2_dangerous_shell_hookmark_passthrough' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- ucsb_attack_4_markdown_exfil_failclosed_without_gui stdout ----

thread 'ucsb_attack_4_markdown_exfil_failclosed_without_gui' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_1_address_substitution_blocked stdout ----

thread 'ucsb_attack_1_address_substitution_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_04_persistence_shell_rc_hookmark_passthrough stdout ----

thread 'in_cr_04_persistence_shell_rc_hookmark_passthrough' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- unterminated_final_event_still_blocks_critical stdout ----

thread 'unterminated_final_event_still_blocks_critical' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- benign_response_passes_through_unchanged stdout ----

thread 'benign_response_passes_through_unchanged' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }


failures:
    address_substitution_from_prompt_seed_blocks
    benign_response_passes_through_unchanged
    in_cr_03_sensitive_path_warn_passes_through
    in_cr_04_persistence_shell_rc_hookmark_passthrough
    malformed_tool_use_partial_json_blocks
    ucsb_attack_1_address_substitution_blocked
    ucsb_attack_2_dangerous_shell_hookmark_passthrough
    ucsb_attack_3_signing_tool_blocked
    ucsb_attack_4_markdown_exfil_failclosed_without_gui
    unterminated_final_event_still_blocks_critical

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p sieve-cli --test inbound_block`

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/inbound_block.rs | sed -n '70,110p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "git diff --name-only && printf '\\n--- untracked files ---\\n' && git ls-files --others --exclude-standard" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
.cursorrules
CLAUDE.md
Cargo.lock
Cargo.toml
README.md
SECURITY.md
crates/sieve-cli/Cargo.toml
crates/sieve-cli/src/audit.rs
crates/sieve-cli/src/cli.rs
crates/sieve-cli/src/config.rs
crates/sieve-cli/src/daemon.rs
crates/sieve-cli/src/engine_adapter.rs
crates/sieve-cli/src/main.rs
crates/sieve-cli/tests/inbound_block.rs
crates/sieve-cli/tests/outbound_block.rs
crates/sieve-core/Cargo.toml
crates/sieve-core/src/detection.rs
crates/sieve-core/src/pipeline/inbound.rs
crates/sieve-core/src/pipeline/mod.rs
crates/sieve-rules/benches/scan_bench.rs
crates/sieve-rules/rules/inbound.toml
crates/sieve-rules/rules/outbound.toml
crates/sieve-rules/src/critical_lock.rs
crates/sieve-rules/src/engine/mod.rs
crates/sieve-rules/src/manifest.rs
crates/sieve-rules/tests/inbound_rules.rs
crates/sieve-rules/tests/outbound_rules.rs
docs/api/api-reference.md
docs/changelog/CHANGELOG.md
docs/design/ADR-001-rust-tech-stack.md
docs/design/ADR-002-rule-engine-only-phase1.md
docs/design/ADR-003-local-only-no-cloud-verifier.md
docs/design/ADR-004-anthropic-first-unified-interface.md
docs/design/ADR-005-overseas-legal-entity.md
docs/design/ADR-006-sigstore-reproducible-build.md
docs/design/ADR-007-fail-closed-critical-actions.md
docs/design/ADR-011-private-until-ga.md
docs/design/ADR-INDEX.md
docs/design/architecture.md
docs/design/data-model.md
docs/glossary.md
docs/guides/deployment.md
docs/guides/development.md
docs/requirements/PRD-sieve.md
docs/requirements/user-stories.md
tasks/lessons.md
tasks/roadmap.md

--- untracked files ---
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
docs/design/ADR-018-openai-protocol-adaptation.md
docs/design/ADR-019-x-sieve-origin-header.md
docs/prd/sieve-prd-v1.4.md
docs/prd/sieve-prd-v1.5.md
docs/specs/SPEC-001-sieve-hook-protocol.md
docs/specs/SPEC-002-hips-popup-behavior.md
docs/specs/SPEC-003-sieve-setup-tool.md
docs/specs/SPEC-004-multi-agent-setup.md
librust_out.rlib
tasks/known-issues-v1.4.md
tasks/todo.md

exec
/bin/zsh -lc "git diff -- crates/sieve-core/src/detection.rs crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/pipeline/inbound.rs crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-cli/src/engine_adapter.rs | sed -n '1,320p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "git diff -- crates/sieve-core/src/pipeline/mod.rs crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs | sed -n '1,360p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "git diff -- crates/sieve-core/src/pipeline/mod.rs crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs | sed -n '360,760p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "git diff -- crates/sieve-rules/src/manifest.rs | sed -n '160,360p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
/bin/zsh -lc "rg \"dispatch\\(\" -n crates | head -50 && rg \"WarnConfirm\" -n" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-core/src/pipeline/mod.rs:127:    pub async fn dispatch(
crates/sieve-core/src/pipeline/mod.rs:319:            let result = dispatch(Direction::Inbound, vec![], None, Uuid::new_v4(), None, None)
crates/sieve-core/src/pipeline/mod.rs:330:            let result = dispatch(
crates/sieve-core/src/pipeline/mod.rs:348:            let result = dispatch(
crates/sieve-core/src/pipeline/mod.rs:374:            let result = dispatch(
crates/sieve-core/src/pipeline/mod.rs:399:            let result = dispatch(
docs/changelog/CHANGELOG.md:261:- 主动 macOS / TUI 弹窗（Action::WarnConfirm 实现）
docs/changelog/CHANGELOG.md:295:- `Action { Block, Redact, WarnConfirm, MarkOnly, SilentLog }`
crates/sieve-core/src/detection.rs:24:/// v1.4 重构：按 `Disposition` 路由，废弃 `WarnConfirm`。
docs/design/data-model.md:84:    pub action: Action,                // Week 2 新增：Block / Redact / WarnConfirm / MarkOnly / SilentLog
docs/design/data-model.md:101:    WarnConfirm { countdown_secs: u8 },

exec
/bin/zsh -lc "sed -n '1,280p' crates/sieve-core/src/pipeline/outbound_redact.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 出站自动脱敏路径（AutoRedact disposition，OUT-01~05/12）。
//!
//! 提供两套 API：
//! - [`redact_body_bytes`]：在 raw body bytes 中按绝对字节偏移替换（fuzz/单测保留）。
//! - [`redact_segments`]：在解析后的文本段列表中按累计字符偏移替换，
//!   返回替换后的文本段列表，由调用方重新序列化 JSON——这是 daemon AutoRedact 路径
//!   的正确用法（修 #1：AutoRedact 偏移修复）。
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

/// 文本段级脱敏结果（对应 [`redact_segments`] 的输出）。
#[derive(Debug)]
pub struct SegmentRedactResult {
    /// 脱敏后的文本段列表，顺序与输入 `segments` 一一对应。
    pub texts: Vec<String>,
    /// 实际发生脱敏的总数量（合并后的 span 数，跨所有段）。
    pub redacted_count: usize,
    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
    pub redacted_summary: String,
}

/// 在解析后的文本段列表中按**累计字符偏移**做脱敏替换。
///
/// # 背景（修 #1：AutoRedact 偏移修复）
///
/// [`Detection.span`] 的 `start`/`end` 是 `extract_text_content()` 返回的
/// **累计文本字符偏移**（即 `body_byte_offset + vectorscan_offset`），
/// 而非 raw JSON body 的字节偏移。直接把这些偏移喂给 [`redact_body_bytes`]
/// 会写错 raw body 的字节范围，无法正确擦除 secret。
///
/// 正确做法：在每个文本段字符串内计算段内偏移后做字符串替换，
/// 然后由调用方把替换后的文本重新填入 JSON 并重新序列化。
///
/// # 参数
/// - `segments`：`(segment_global_start_offset, segment_text)` 列表，
///   顺序与 `AnthropicRequest::extract_text_content()` 返回值一致。
/// - `hits`：要脱敏的命中列表，`start`/`end` 是累计字符偏移（`Detection.span`）。
///
/// # 返回
/// [`SegmentRedactResult`]，其中 `texts` 顺序对应输入 `segments`。
///
/// 关联：PRD v1.4 §6.1（AutoRedact 路径）、ADR-016（二维处置矩阵）。
pub fn redact_segments(segments: &[(usize, String)], hits: &[RedactHit]) -> SegmentRedactResult {
    if hits.is_empty() {
        return SegmentRedactResult {
            texts: segments.iter().map(|(_, t)| t.clone()).collect(),
            redacted_count: 0,
            redacted_summary: String::new(),
        };
    }

    let mut total_redacted = 0usize;
    let mut all_rule_ids: Vec<String> = Vec::new();
    let mut result_texts: Vec<String> = Vec::with_capacity(segments.len());

    for (seg_idx, (seg_start, seg_text)) in segments.iter().enumerate() {
        let seg_end = seg_start + seg_text.len();

        // 过滤出与当前段有交集的 hit（累计偏移范围与段范围重叠）
        let seg_hits: Vec<RedactHit> = hits
            .iter()
            .filter(|h| h.start < seg_end && h.end > *seg_start)
            .map(|h| {
                // 把全局偏移转换为段内字符偏移（clamp 到段边界）
                let local_start = h.start.saturating_sub(*seg_start).min(seg_text.len());
                let local_end = h.end.saturating_sub(*seg_start).min(seg_text.len());
                RedactHit {
                    rule_id: h.rule_id.clone(),
                    start: local_start,
                    end: local_end,
                }
            })
            .collect();

        if seg_hits.is_empty() {
            result_texts.push(seg_text.clone());
            continue;
        }

        // 在 UTF-8 字符串上做 redact（按字节偏移，text 是 UTF-8 已验证）
        let text_bytes = seg_text.as_bytes();
        let redact_result = redact_body_bytes(text_bytes, &seg_hits);

        total_redacted += redact_result.redacted_count;
        if !redact_result.redacted_summary.is_empty() {
            all_rule_ids.push(redact_result.redacted_summary.clone());
        }

        // redact_body_bytes 保证输出有效 UTF-8（placeholder 是 ASCII，原始文本是 UTF-8）
        // Safety: redact_body_bytes 对齐 UTF-8 边界，placeholder 是纯 ASCII
        let new_text = String::from_utf8(redact_result.body).unwrap_or_else(|_| seg_text.clone()); // 极端回退：保留原文
        result_texts.push(new_text);

        // suppress unused variable lint for seg_idx
        let _ = seg_idx;
    }

    SegmentRedactResult {
        texts: result_texts,
        redacted_count: total_redacted,
        redacted_summary: all_rule_ids.join(", "),
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

exec
/bin/zsh -lc "rg \"extract_text_content|ContentSpan|span\" -n crates/sieve-core/src crates/sieve-cli/src/engine_adapter.rs crates/sieve-rules/src | head -100 && sed -n '1,240p' crates/sieve-core/src/protocol/anthropic.rs || true" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
crates/sieve-cli/src/engine_adapter.rs:13:use sieve_core::protocol::unified_message::ContentSpan;
crates/sieve-cli/src/engine_adapter.rs:207:                span: ContentSpan {
crates/sieve-cli/src/engine_adapter.rs:238:    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
crates/sieve-cli/src/engine_adapter.rs:308:                span: ContentSpan {
crates/sieve-cli/src/engine_adapter.rs:336:                    // span 为整个输入范围的近似（无精确字节偏移）
crates/sieve-cli/src/engine_adapter.rs:337:                    span: ContentSpan {
crates/sieve-cli/src/engine_adapter.rs:442:    fn span_offset_applied() {
crates/sieve-cli/src/engine_adapter.rs:456:        assert_eq!(hits[0].span.start, 104); // 100 + 4
crates/sieve-cli/src/engine_adapter.rs:457:        assert_eq!(hits[0].span.end, 109); // 100 + 9
crates/sieve-core/src/detection.rs:3:use crate::protocol::unified_message::ContentSpan;
crates/sieve-core/src/detection.rs:84:    pub span: ContentSpan,
crates/sieve-core/src/protocol/anthropic.rs:52:    /// 返回 `(近似 body 字节偏移, text)` 列表。Phase 1 偏移仅供审计参考；精确 span 由
crates/sieve-core/src/protocol/anthropic.rs:56:    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
crates/sieve-core/src/protocol/anthropic.rs:146:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:155:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:165:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/unified_message.rs:47:pub struct ContentSpan {
crates/sieve-core/src/protocol/unified_message.rs:64:        span: Option<ContentSpan>,
crates/sieve-core/src/protocol/unified_message.rs:166:            span: Some(ContentSpan { start: 0, end: 5 }),
crates/sieve-core/src/pipeline/outbound.rs:24:    /// - `body_byte_offset`：该文本在原始请求 body 中的起始字节偏移（用于生成绝对 span）。
crates/sieve-core/src/pipeline/outbound.rs:73:            if let ContentBlock::Text { text, span } = block {
crates/sieve-core/src/pipeline/outbound.rs:74:                let body_offset = span.map(|s| s.start).unwrap_or(0);
crates/sieve-core/src/pipeline/outbound.rs:93:        ContentBlock, ContentSpan, Direction, MessageMetadata, Role, UnifiedMessage,
crates/sieve-core/src/pipeline/outbound.rs:116:                    span: ContentSpan {
crates/sieve-core/src/pipeline/outbound.rs:134:                span: None,
crates/sieve-core/src/pipeline/outbound_redact.rs:27:    /// 实际发生脱敏的数量（合并后的 span 数）。
crates/sieve-core/src/pipeline/outbound_redact.rs:54:/// 3. 合并重叠 / 相邻 span（多个 span 合并时 `rule_id` 取最左命中）；
crates/sieve-core/src/pipeline/outbound_redact.rs:55:/// 4. 逐段复制原始字节，用 `[REDACTED:<rule_id>]` 替换各合并 span。
crates/sieve-core/src/pipeline/outbound_redact.rs:82:    // 3. 合并重叠 / 相邻 span
crates/sieve-core/src/pipeline/outbound_redact.rs:88:            // 对齐后 span 变空，跳过
crates/sieve-core/src/pipeline/outbound_redact.rs:140:    /// 实际发生脱敏的总数量（合并后的 span 数，跨所有段）。
crates/sieve-core/src/pipeline/outbound_redact.rs:150:/// [`Detection.span`] 的 `start`/`end` 是 `extract_text_content()` 返回的
crates/sieve-core/src/pipeline/outbound_redact.rs:160:///   顺序与 `AnthropicRequest::extract_text_content()` 返回值一致。
crates/sieve-core/src/pipeline/outbound_redact.rs:161:/// - `hits`：要脱敏的命中列表，`start`/`end` 是累计字符偏移（`Detection.span`）。
crates/sieve-core/src/pipeline/outbound_redact.rs:241:    // ── 1. 单 span ───────────────────────────────────────────────────────────
crates/sieve-core/src/pipeline/outbound_redact.rs:244:    fn single_span_middle() {
crates/sieve-core/src/pipeline/outbound_redact.rs:256:    // ── 2. 多 span（不重叠）──────────────────────────────────────────────────
crates/sieve-core/src/pipeline/outbound_redact.rs:259:    fn multiple_non_overlapping_spans() {
crates/sieve-core/src/pipeline/outbound_redact.rs:270:    // ── 3. 重叠 span 合并 ────────────────────────────────────────────────────
crates/sieve-core/src/pipeline/outbound_redact.rs:273:    fn overlapping_spans_merged() {
crates/sieve-core/src/pipeline/outbound_redact.rs:280:            "two overlapping spans must merge into one"
crates/sieve-core/src/pipeline/outbound_redact.rs:325:    // ── 额外：span 超出 body 长度 clamp ──────────────────────────────────────
crates/sieve-core/src/pipeline/outbound_redact.rs:328:    fn span_clamped_to_body_len() {
crates/sieve-core/src/pipeline/inbound.rs:9:use crate::protocol::unified_message::ContentSpan;
crates/sieve-core/src/pipeline/inbound.rs:135:                        span: ContentSpan {
crates/sieve-core/src/pipeline/inbound.rs:169:    use crate::protocol::unified_message::ContentSpan;
crates/sieve-core/src/pipeline/inbound.rs:191:                    span: ContentSpan { start: 0, end: 5 },
crates/sieve-core/src/pipeline/inbound.rs:203:                    span: ContentSpan { start: 0, end: 15 },
crates/sieve-core/src/pipeline/inbound.rs:224:                    span: ContentSpan {
crates/sieve-core/src/pipeline/mod.rs:162:                        start: d.span.start,
crates/sieve-core/src/pipeline/mod.rs:163:                        end: d.span.end,
crates/sieve-core/src/pipeline/mod.rs:300:        use crate::protocol::unified_message::ContentSpan;
crates/sieve-core/src/pipeline/mod.rs:309:                span: ContentSpan { start: 0, end: 5 },
crates/sieve-core/src/pipeline/mod.rs:371:            d.span = ContentSpan { start: 0, end: 5 };
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
/bin/zsh -lc "sed -n '1,260p' crates/sieve-rules/rules/outbound.toml && sed -n '1,260p' crates/sieve-rules/rules/inbound.toml" in /Users/doskey/src/sieve
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

exec
/bin/zsh -lc "sed -n '260,620p' crates/sieve-rules/rules/inbound.toml && sed -n '200,360p' crates/sieve-rules/rules/outbound.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
/bin/zsh -lc "nl -ba crates/sieve-rules/rules/outbound.toml | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
     1	# OUT-01~12 出站 P0 规则集
     2	#
     3	# 关联 PRD §5.1 出站检测目标。
     4	# 来源: gitleaks/gitleaks config/gitleaks.toml (MIT License) + Sieve 自写
     5	#
     6	# 重要约束: 所有 pattern 必须为 vectorscan 兼容 PCRE 子集。
     7	# 禁止使用: lookahead(?=...) / lookbehind(?<=...) / 反向引用\1 / 原子组(?>...) / \C \R \K
     8	
     9	# ---------------------------------------------------------------------------
    10	# OUT-01: Anthropic API Key
    11	# 格式: sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" 结尾
    12	# ---------------------------------------------------------------------------
    13	[[rules]]
    14	id = "OUT-01"
    15	description = "Anthropic API key (sk-ant-api03-...)"
    16	pattern = 'sk-ant-api03-[a-zA-Z0-9_\-]{93}AA'
    17	severity = "critical"
    18	action = "block"
    19	entropy_min = 4.5
    20	keywords = ["sk-ant-api03"]
    21	allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
    22	allowlist_stopwords = []
    23	disposition = "auto_redact"
    24	default_on_timeout = "redact"
    25	
    26	# ---------------------------------------------------------------------------
    27	# OUT-02: OpenAI API Key
    28	# 新格式: sk-proj-/sk-svcacct-/sk-admin- + 58~200 char + T3BlbkFJ + 58~200 char
    29	# 旧格式: sk- + 20 alnum + T3BlbkFJ + 20 alnum
    30	# 两种格式用 alternation 合并，无 lookahead
    31	# ---------------------------------------------------------------------------
    32	[[rules]]
    33	id = "OUT-02"
    34	description = "OpenAI API key (sk-... legacy + sk-proj-/sk-svcacct-/sk-admin-)"
    35	pattern = 'sk-(?:proj|svcacct|admin)-[A-Za-z0-9_\-]{58,65}T3BlbkFJ[A-Za-z0-9_\-]{58,65}|sk-[a-zA-Z0-9]{20}T3BlbkFJ[a-zA-Z0-9]{20}'
    36	severity = "critical"
    37	action = "block"
    38	entropy_min = 4.5
    39	keywords = ["T3BlbkFJ"]
    40	allowlist_regexes = []
    41	allowlist_stopwords = []
    42	disposition = "auto_redact"
    43	default_on_timeout = "redact"
    44	
    45	# ---------------------------------------------------------------------------
    46	# OUT-03: AWS Access Key ID
    47	# 前缀: A3T[A-Z0-9] / AKIA / ASIA / ABIA / ACCA，后跟 16 个 base32 字符
    48	# ---------------------------------------------------------------------------
    49	[[rules]]
    50	id = "OUT-03"
    51	description = "AWS Access Key ID (AKIA / ASIA / ABIA / ACCA / A3T)"
    52	pattern = '(?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16}'
    53	severity = "critical"
    54	action = "block"
    55	entropy_min = 3.0
    56	keywords = ["AKIA", "ASIA", "ABIA", "ACCA"]
    57	allowlist_regexes = []
    58	allowlist_stopwords = ["AKIAIOSFODNN7EXAMPLE"]  # AWS 官方文档示例 key
    59	disposition = "auto_redact"
    60	default_on_timeout = "redact"
    61	
    62	# ---------------------------------------------------------------------------
    63	# OUT-04: GitHub Personal Access Token
    64	# 格式: ghp_/gho_/ghu_/ghs_/ghr_ + 36 个 alnum 字符
    65	# ---------------------------------------------------------------------------
    66	[[rules]]
    67	id = "OUT-04"
    68	description = "GitHub PAT (ghp_/gho_/ghu_/ghs_/ghr_)"
    69	pattern = 'gh[pousr]_[0-9a-zA-Z]{36}'
    70	severity = "critical"
    71	action = "block"
    72	entropy_min = 4.0
    73	keywords = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
    74	allowlist_regexes = []
    75	allowlist_stopwords = []
    76	disposition = "auto_redact"
    77	default_on_timeout = "redact"
    78	
    79	# ---------------------------------------------------------------------------
    80	# OUT-05: Google Cloud API Key
    81	# 格式: AIza + 35 个 alnum/_/-
    82	# ---------------------------------------------------------------------------
    83	[[rules]]
    84	id = "OUT-05"
    85	description = "Google Cloud API Key (AIza...)"
    86	pattern = 'AIza[0-9A-Za-z_\-]{35}'
    87	severity = "high"
    88	action = "block"
    89	entropy_min = 4.0
    90	keywords = ["AIza"]
    91	allowlist_regexes = []
    92	allowlist_stopwords = []
    93	disposition = "auto_redact"
    94	default_on_timeout = "redact"
    95	
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
   191	# 三段由英文句号分隔
   192	# ---------------------------------------------------------------------------
   193	[[rules]]
   194	id = "OUT-11"
   195	description = "Discord Bot Token"
   196	pattern = '[A-Za-z0-9_\-]{24,28}\.[A-Za-z0-9_\-]{6}\.[A-Za-z0-9_\-]{27,38}'
   197	severity = "high"
   198	action = "block"
   199	entropy_min = 3.5
   200	keywords = ["."]
   201	allowlist_regexes = []
   202	allowlist_stopwords = []
   203	disposition = "status_bar"
   204	
   205	# ---------------------------------------------------------------------------
   206	# OUT-09（BIP39 助记词）在 engine_adapter 中通过 second-pass 实现，
   207	# 不使用 vectorscan 占位规则。
   208	# 详见 crates/sieve-cli/src/engine_adapter.rs OutboundAdapter::scan_text。
   209	# 关联 PRD §9 #4 差异化点：SHA-256 checksum 验证在 second-pass 完成。
   210	# ---------------------------------------------------------------------------

exec
/bin/zsh -lc "sed -n '1,380p' crates/sieve-cli/src/engine_adapter.rs" in /Users/doskey/src/sieve
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
///
/// 注：修 #2 后生产路径不再调用此函数（disposition 优先），
/// 保留用于单元测试验证 Warn → HookMark 的语义不变。
#[allow(dead_code)]
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

            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复，入站侧）。
            //
            // 规则显式写了 disposition 时直接路由；
            // disposition=None 且 fail-closed 时才强制 Block。
            // 这确保 IN-CR-02（hook_terminal）/ IN-CR-05（gui_popup）即使在 fail-closed
            // 名单里也能走正确的 HookMark / HoldForDecision 路径（不被截成 Block）。
            //
            // 关联：ADR-016（二维处置矩阵）、ADR-014（双层防御）、PRD v1.4 §5.4。
            let action = if let Some(r) = rule {
                if let Some(disp) = r.disposition {
                    // 显式 disposition：直接路由，不经过 enforce_action
                    let timeout = r.timeout_seconds.unwrap_or(60);
                    map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                } else {
                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
                    let enforced =
                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                    if enforced == RulesAction::Block {
                        Action::Block
                    } else {
                        let disp = r.effective_disposition();
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                    }
                }
            } else {
                // 规则表中找不到：fail-closed Block
                Action::Block
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
            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复）。
            //
            // 规则显式写了 disposition 时，**直接按 disposition 路由**——
            // 这确保 OUT-01（auto_redact）即使在 fail-closed 名单里也走 Redact 而非 Block。
            // 只有 disposition=None（旧规则 / 无显式配置）且 fail-closed 时，才走 Block。
            //
            // 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
            let action = rule
                .map(|r| {
                    if let Some(disp) = r.disposition {
                        // 显式 disposition：直接路由，不经过 enforce_action
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                    } else {
                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
                        let enforced =
                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                        if enforced == RulesAction::Block {
                            Action::Block
                        } else {
                            let disp = r.effective_disposition();
                            let timeout = r.timeout_seconds.unwrap_or(60);
                            map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                        }
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


exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '300,380p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   300	
   301	        if !hold_detections_outbound.is_empty() {
   302	            if let Some(ref ipc_server) = ipc {
   303	                use chrono::Utc;
   304	
   305	                let request_id = uuid::Uuid::new_v4();
   306	                let (timeout_seconds, default_on_timeout) = hold_detections_outbound
   307	                    .iter()
   308	                    .find_map(|d| {
   309	                        if let Action::HoldForDecision {
   310	                            timeout_seconds, ..
   311	                        } = d.action
   312	                        {
   313	                            // 取第一个 HoldForDecision detection 的规则 timeout/default
   314	                            // default_on_timeout 从 detection 的 rule_id 对应规则读取，
   315	                            // 此处用 Block 作为保守默认（规则未设则 fail-closed）
   316	                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
   317	                        } else {
   318	                            None
   319	                        }
   320	                    })
   321	                    .unwrap_or((60, sieve_ipc::DefaultOnTimeout::Block));
   322	
   323	                let ipc_detections = hold_detections_outbound
   324	                    .iter()
   325	                    .map(|d| sieve_ipc::protocol::DetectionPayload {
   326	                        rule_id: d.rule_id.clone(),
   327	                        severity: map_severity_to_ipc(d.severity),
   328	                        disposition: sieve_ipc::Disposition::GuiPopup,
   329	                        title: format!("出站检测命中：{}", d.rule_id),
   330	                        one_line_summary: d.evidence_truncated.clone(),
   331	                        details: serde_json::json!({}),
   332	                    })
   333	                    .collect();
   334	
   335	                let ipc_req = sieve_ipc::DecisionRequest {
   336	                    request_id,
   337	                    created_at: Utc::now(),
   338	                    timeout_seconds,
   339	                    default_on_timeout,
   340	                    detections: ipc_detections,
   341	                };
   342	
   343	                // 出站 hold：无 SSE keep-alive，直接 await 决策
   344	                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
   345	                let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;
   346	
   347	                match outcome {
   348	                    Ok(resp) => match resp.decision {
   349	                        sieve_ipc::DecisionAction::Allow => {
   350	                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
   351	                            // 继续往下，走正常转发路径
   352	                        }
   353	                        sieve_ipc::DecisionAction::RedactAndAllow => {
   354	                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
   355	                            // 若有 redact_hits 则脱敏，否则原 body 转发（与 Allow 同逻辑）
   356	                            // 直接 fall-through 到下方 redact_hits 处理
   357	                        }
   358	                        sieve_ipc::DecisionAction::Deny => {
   359	                            tracing::warn!("OUTBOUND GUI: Deny → 426");
   360	                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
   361	                                .iter()
   362	                                .map(|d| (*d).clone())
   363	                                .collect();
   364	                            return Ok(build_426_response(&held));
   365	                        }
   366	                    },
   367	                    Err(e) => {
   368	                        // IPC 错误：按 default_on_timeout 兜底（fail-closed）
   369	                        tracing::warn!(error = %e, "OUTBOUND GUI: IPC error, fail-closed → 426");
   370	                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
   371	                            .iter()
   372	                            .map(|d| (*d).clone())
   373	                            .collect();
   374	                        return Ok(build_426_response(&held));
   375	                    }
   376	                }
   377	            } else {
   378	                // IPC 未初始化：fail-closed → 426
   379	                tracing::warn!("OUTBOUND GUI: IPC not initialized, fail-closed → 426");
   380	                let held: Vec<sieve_core::Detection> = hold_detections_outbound

exec
/bin/zsh -lc "sed -n '160,320p' crates/sieve-ipc/src/socket_server.rs && nl -ba crates/sieve-cli/src/daemon.rs | sed -n '347,410p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
            "request_decision",
            serde_json::to_value(&req)?,
            serde_json::Value::String(request_id.to_string()),
        );
        let mut payload = serde_json::to_string(&rpc_req)?;
        payload.push('\n');

        if let Err(_e) = sender.send(payload).await {
            // GUI 写通道关闭（GUI 进程崩溃或通道满），立即 fallback。
            warn!(%request_id, "GUI writer channel closed; immediate fallback");
            self.pending.lock().await.remove(&request_id);
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 4. 等待 GUI 回复或超时。
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => {
                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
            Err(_elapsed) => {
                // 超时，清理 pending map。
                self.pending.lock().await.remove(&request_id);
                warn!(%request_id, "decision timeout");
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
        }
    }

    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
    pub async fn inject_decision(&self, resp: DecisionResponse) {
        let mut map = self.pending.lock().await;
        if let Some(tx) = map.remove(&resp.request_id) {
            let _ = tx.send(resp);
        }
    }
}

/// 处理单个 GUI 长连接。
///
/// 同时管理两个方向：
/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map。
/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
///
/// 任一方向出错（GUI 断线 / 写失败）都会退出，调用方负责清理 `gui_writer`。
async fn handle_connection(
    stream: UnixStream,
    pending: PendingMap,
    gui_writer: GuiWriter,
    mut write_rx: mpsc::Receiver<String>,
) -> Result<(), IpcError> {
    info!("GUI client connected");

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    loop {
        tokio::select! {
            // 读方向：GUI 发来 decision_response。
            line_result = lines.next_line() => {
                match line_result? {
                    None => {
                        // GUI 关闭连接。
                        info!("GUI client closed connection");
                        break;
                    }
                    Some(line) => {
                        let line = line.trim().to_owned();
                        if line.is_empty() {
                            continue;
                        }
                        debug!(raw = %line, "received IPC message from GUI");
                        dispatch_response(&line, &pending).await;
                    }
                }
            }

            // 写方向：主代理 push request_decision 给 GUI。
            msg = write_rx.recv() => {
                match msg {
                    None => {
                        // 发送端已丢弃（IpcServer 被 drop），退出。
                        debug!("GUI write channel closed");
                        break;
                    }
                    Some(payload) => {
                        if let Err(e) = write_half.write_all(payload.as_bytes()).await {
                            warn!("failed to write to GUI socket: {e}");
                            break;
                        }
                    }
                }
            }
        }
    }

    // 连接断开：把所有 pending oneshot 全部触发 fallback（drop sender）。
    // 丢弃 sender 会让 rx 收到 Err(RecvError)，request_decision 走 fallback。
    let mut map = pending.lock().await;
    let count = map.len();
    if count > 0 {
        warn!(
            pending_count = count,
            "GUI disconnected with pending requests; dropping all"
        );
        map.clear(); // 清空 map，sender 被 drop，所有等待者收到 Err 并 fallback。
    }
    // gui_writer 由 run() 的 spawn closure 在此函数返回后清理。
    drop(gui_writer); // 显式 drop 避免编译器警告。

    Ok(())
}

/// 解析 GUI 发来的一行 JSON-RPC response 并派发到 pending map。
async fn dispatch_response(line: &str, pending: &PendingMap) {
    let rpc: crate::protocol::jsonrpc::Response = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to parse IPC response from GUI: {e}");
            return;
        }
    };

    if let Some(err_obj) = &rpc.error {
        error!(
            code = err_obj.code,
            message = %err_obj.message,
            "GUI returned rpc error"
        );
        return;
    }

    if let Some(result) = rpc.result {
        match serde_json::from_value::<DecisionResponse>(result) {
            Ok(resp) => {
                let mut map = pending.lock().await;
                if let Some(tx) = map.remove(&resp.request_id) {
                    let _ = tx.send(resp);
                } else {
                    warn!(
                        request_id = %resp.request_id,
                        "no pending request for this decision"
                    );
                }
            }
            Err(e) => {
                warn!("failed to deserialize DecisionResponse: {e}");
            }
        }
    }
}

fn make_timeout_fallback(
    request_id: Uuid,
    default_on_timeout: DefaultOnTimeout,
) -> DecisionResponse {
    let action = match default_on_timeout {
        DefaultOnTimeout::Block => DecisionAction::Deny,
        DefaultOnTimeout::Allow => DecisionAction::Allow,
   347	                match outcome {
   348	                    Ok(resp) => match resp.decision {
   349	                        sieve_ipc::DecisionAction::Allow => {
   350	                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
   351	                            // 继续往下，走正常转发路径
   352	                        }
   353	                        sieve_ipc::DecisionAction::RedactAndAllow => {
   354	                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
   355	                            // 若有 redact_hits 则脱敏，否则原 body 转发（与 Allow 同逻辑）
   356	                            // 直接 fall-through 到下方 redact_hits 处理
   357	                        }
   358	                        sieve_ipc::DecisionAction::Deny => {
   359	                            tracing::warn!("OUTBOUND GUI: Deny → 426");
   360	                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
   361	                                .iter()
   362	                                .map(|d| (*d).clone())
   363	                                .collect();
   364	                            return Ok(build_426_response(&held));
   365	                        }
   366	                    },
   367	                    Err(e) => {
   368	                        // IPC 错误：按 default_on_timeout 兜底（fail-closed）
   369	                        tracing::warn!(error = %e, "OUTBOUND GUI: IPC error, fail-closed → 426");
   370	                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
   371	                            .iter()
   372	                            .map(|d| (*d).clone())
   373	                            .collect();
   374	                        return Ok(build_426_response(&held));
   375	                    }
   376	                }
   377	            } else {
   378	                // IPC 未初始化：fail-closed → 426
   379	                tracing::warn!("OUTBOUND GUI: IPC not initialized, fail-closed → 426");
   380	                let held: Vec<sieve_core::Detection> = hold_detections_outbound
   381	                    .iter()
   382	                    .map(|d| (*d).clone())
   383	                    .collect();
   384	                return Ok(build_426_response(&held));
   385	            }
   386	        }
   387	
   388	        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
   389	        //
   390	        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
   391	        // 1. redact_segments() 在文本字符串层替换
   392	        // 2. 把替换后的文本写回 AnthropicRequest messages
   393	        // 3. serde_json 重新序列化为新 body
   394	        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
   395	        if !redact_hits.is_empty() {
   396	            let seg_result = redact_segments(&texts, &redact_hits);
   397	            tracing::info!(
   398	                count = seg_result.redacted_count,
   399	                rules = %seg_result.redacted_summary,
   400	                "OUTBOUND AUTO-REDACT"
   401	            );
   402	
   403	            // 把替换后文本写回 AnthropicRequest，然后重新序列化
   404	            let new_body_bytes =
   405	                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
   406	                    .and_then(|req| {
   407	                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
   408	                    })?;
   409	
   410	            // 验证脱敏后 JSON 仍然合法（关键回归断言）

exec
/bin/zsh -lc "sed -n '320,380p' crates/sieve-ipc/src/socket_server.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
        DefaultOnTimeout::Allow => DecisionAction::Allow,
        DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
    };
    DecisionResponse {
        request_id,
        decision: action,
        decided_at: Utc::now(),
        by_user: false,
        remember: false,
    }
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-hook/src/main.rs && sed -n '1,300p' crates/sieve-hook/src/pending.rs && sed -n '1,260p' crates/sieve-hook/src/decision.rs && sed -n '1,260p' crates/sieve-hook/src/protocol.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
// sieve-hook: Claude Code PreToolUse hook 二进制。
//
// 夹在 Claude Code tool_use 调用与实际执行之间，对命中 Critical 规则的工具调用
// 在 TTY 显示危险摘要并等待用户确认。
//
// 启动时延目标 < 50ms（依赖仅 serde_json + fd-lock + clap，无 tokio / vectorscan）。
// 关联：SPEC-001（hook 文件协议）、SPEC-002（弹窗行为规范）、ADR-014（双层防御）。

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use uuid::Uuid;

// 从 lib target 引入共享模块，避免重复定义。
use sieve_hook_lib::decision::{write_decision, DecisionOutcome};
use sieve_hook_lib::error::PendingError;
use sieve_hook_lib::pending::{read_pending_checked, scan_pending_dir};
use sieve_hook_lib::protocol;

const STALE_THRESHOLD_SECS: i64 = 600;

/// sieve-hook: PreToolUse 安全确认 hook（Phase 1 macOS）。
#[derive(Parser, Debug)]
#[command(name = "sieve-hook", about = "Sieve PreToolUse safety hook")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// 检查 pending 决策请求并请求用户确认。
    Check {
        /// 决策请求 ID（UUID）；未传则读 $SIEVE_REQUEST_ID。
        #[arg(long)]
        request_id: Option<String>,

        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
        #[arg(long)]
        sieve_home: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let Command::Check {
        request_id,
        sieve_home,
    } = cli.command;

    // 解析 sieve_home：flag > env > default。
    let base = sieve_home
        .or_else(|| std::env::var("SIEVE_HOME").ok().map(PathBuf::from))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".sieve"))
        })
        .unwrap_or_else(|| {
            eprintln!("sieve-hook: cannot determine sieve home directory ($HOME not set)");
            std::process::exit(1);
        });

    // 解析 request_id：优先级 1（flag）> 优先级 2（env）> 优先级 3（启发式扫目录）。
    // 优先级 3 是关键修复：Claude Code settings.json 注册静态命令时无法传 request_id，
    // 必须走启发式路径；零 pending 时 fail-open（exit 0），不阻断正常工具调用。
    // 关联：SPEC-001 §4.3（启发式查 pending 目录）。
    let explicit_id = request_id.or_else(|| std::env::var("SIEVE_REQUEST_ID").ok());

    let exit_code = match explicit_id {
        Some(id_str) => {
            let request_id = match Uuid::parse_str(&id_str) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("sieve-hook: invalid request ID `{id_str}`: {e}");
                    std::process::exit(1);
                }
            };
            run(request_id, &base)
        }
        None => {
            // 优先级 3：启发式扫目录。
            run_heuristic(&base)
        }
    };

    std::process::exit(exit_code);
}

/// 核心逻辑，返回进程退出码（0 = 允许，1 = 拒绝）。
///
/// 关联：SPEC-001 §4（hook 决策流程）。
fn run(request_id: Uuid, base: &std::path::Path) -> i32 {
    let req = match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Ok(r) => r,
        Err(PendingError::NotFound) => {
            // fail-open：Sieve 代理未标记此请求，放行。
            return 0;
        }
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            return 1;
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            return 1;
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            return 1;
        }
    };

    // 打印危险摘要（SPEC-002 §2：多 issue 合并风格）。
    print_summary(&req);

    // 倒计时交互。
    let outcome = prompt_user(&req);

    // 写决策文件。
    if let Err(e) = write_decision(request_id, &outcome, base) {
        eprintln!("sieve-hook: failed to write decision: {e}");
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 打印危险摘要到 stderr（TTY 终端显示）。
///
/// 关联：SPEC-002 §2.1（多 issue 合并显示）。
fn print_summary(req: &protocol::DecisionRequest) {
    let n = req.detections.len();
    eprintln!();
    eprintln!("┌─ Sieve 安全警告 ({n} 条检测) ────────────────────────────────");
    for (i, det) in req.detections.iter().enumerate() {
        let severity_tag = match det.severity.as_str() {
            "critical" => "CRITICAL",
            "high" => "HIGH    ",
            "medium" => "MEDIUM  ",
            _ => "LOW     ",
        };
        eprintln!(
            "│ [{:2}] [{severity_tag}] {} — {}",
            i + 1,
            det.rule_id,
            det.title
        );
        eprintln!("│       {}", det.one_line_summary);
    }
    eprintln!("└────────────────────────────────────────────────────────────");
    eprintln!();
}

/// TTY 倒计时交互，返回用户决策。
///
/// - 输入 `y`/`Y` → Allow（exit 0）
/// - 输入 `n`/`N`/回车（默认拒绝）→ Deny（exit 1）
/// - 倒计时到 → 按 default_on_timeout 决定
///
/// 用 `spawn thread + mpsc channel` 实现非阻塞输入，避免引入 tokio。
fn prompt_user(req: &protocol::DecisionRequest) -> DecisionOutcome {
    let timeout = Duration::from_secs(req.timeout_seconds as u64);
    let deadline = std::time::Instant::now() + timeout;

    let stdin = io::stdin();
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        let _ = tx.send(line);
    });

    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        eprint!(
            "\r允许此操作？[y/N]（{} 秒后默认{}） > ",
            remaining.as_secs(),
            default_label(req.default_on_timeout)
        );
        let _ = io::stderr().flush();

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                eprintln!();
                return match line.trim().to_lowercase().as_str() {
                    "y" => DecisionOutcome::Allow,
                    _ => DecisionOutcome::Deny,
                };
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    eprintln!();
                    return match req.default_on_timeout {
                        protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                        _ => DecisionOutcome::Deny,
                    };
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!();
                return DecisionOutcome::Deny;
            }
        }
    }
}

fn default_label(dot: protocol::DefaultOnTimeout) -> &'static str {
    match dot {
        protocol::DefaultOnTimeout::Allow => "允许",
        _ => "拒绝",
    }
}

/// 启发式路径：无 request_id 时扫目录。
///
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并显示所有 detection，TTY 弹窗确认，广播决策
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
fn run_heuristic(base: &std::path::Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 合并所有 detection 到一个"虚拟"请求以统一显示。
    // timeout_seconds 和 default_on_timeout 取最严的策略（任一 Block/Redact → Deny）。
    let merged = merge_requests(&scan.fresh);
    print_summary(&merged);
    let outcome = prompt_user(&merged);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
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

/// 启发式扫目录结果。
pub struct ScanResult {
    /// 所有有效（未过期）的 pending 请求，按 created_at 升序排列。
    pub fresh: Vec<DecisionRequest>,
    /// 过期的 pending 文件路径（供调用方删除）。
    pub stale_paths: Vec<std::path::PathBuf>,
}

/// 扫描 `<base>/pending/` 目录，收集所有未过期的 pending 文件。
///
/// 用于 SIEVE_REQUEST_ID 未设置时的启发式匹配路径。
/// 按 created_at 升序排列，避免随机顺序引起非确定性行为。
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录）。
pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
    let pending_dir = base.join("pending");
    let mut fresh: Vec<DecisionRequest> = Vec::new();
    let mut stale_paths: Vec<std::path::PathBuf> = Vec::new();

    let entries = match std::fs::read_dir(&pending_dir) {
        Ok(e) => e,
        Err(_) => {
            // 目录不存在或无权读 → 视为空目录，fail-open。
            return ScanResult { fresh, stale_paths };
        }
    };

    let now = chrono::Utc::now();

    let decisions_dir = base.join("decisions");

    for entry in entries.flatten() {
        let path = entry.path();
        // 只处理 .json 文件。
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let req: DecisionRequest = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(_) => continue, // 解析失败的文件跳过。
        };

        // 已决策的 pending 跳过（避免重复弹窗）。
        // 若 decisions/<id>.json 已存在，说明该请求已被处理，不再加入 fresh/stale。
        // 关联：SPEC-001 §4.3（清理机制）。
        let decision_path = decisions_dir.join(format!("{}.json", req.request_id));
        if decision_path.exists() {
            continue;
        }

        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
        if age_secs > stale_threshold_secs {
            stale_paths.push(path);
        } else {
            fresh.push(req);
        }
    }

    // 按 created_at 升序排列，保证确定性。
    fresh.sort_by_key(|r| r.created_at);

    ScanResult { fresh, stale_paths }
}
use std::path::Path;

use chrono::Utc;
use fd_lock::RwLock;
use uuid::Uuid;

use crate::protocol::DecisionResponse;

/// hook 侧决策结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionOutcome {
    /// 用户允许，hook 返回 exit 0。
    Allow,
    /// 用户拒绝或超时 fail-closed，hook 返回 exit 1。
    Deny,
}

/// 将决策结果写入 `<base>/decisions/<request_id>.json`。
///
/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁。
///
/// Critical 规则 `remember` 永远 `false`，由调用方（main.rs）强制传入 false。
/// 关联：SPEC-001 §3.3（决策文件写入）、ADR-014（Critical 不可记住）。
pub fn write_decision(
    request_id: Uuid,
    outcome: &DecisionOutcome,
    base: &Path,
) -> Result<(), String> {
    // 确保目录存在。
    let decisions_dir = base.join("decisions");
    let locks_dir = base.join("locks");
    std::fs::create_dir_all(&decisions_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&locks_dir).map_err(|e| e.to_string())?;

    let lock_path = locks_dir.join(format!("{request_id}.lock"));
    let dec_path = decisions_dir.join(format!("{request_id}.json"));

    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| e.to_string())?;

    let mut lock = RwLock::new(lock_file);
    let _guard = lock.write().map_err(|e| e.to_string())?;

    let decision_str = match outcome {
        DecisionOutcome::Allow => "allow",
        DecisionOutcome::Deny => "deny",
    };

    let resp = DecisionResponse {
        request_id,
        decision: decision_str.to_owned(),
        decided_at: Utc::now(),
        by_user: true,
        // Critical 规则 remember 强制 false（SPEC-001 §4.4）。
        remember: false,
    };

    let json = serde_json::to_string_pretty(&resp).map_err(|e| e.to_string())?;
    std::fs::write(&dec_path, json.as_bytes()).map_err(|e| e.to_string())?;

    // decisions 写入成功后，清理对应的 pending 文件。
    // 删除失败不是致命错误（竞争/权限），仅打 warning。
    // Unix 上持有 fd-lock 的文件仍可 unlink，先删 pending 再释放锁是安全的。
    // 关联：SPEC-001 §4.3（清理机制）。
    let pending_path = base.join("pending").join(format!("{request_id}.json"));
    if let Err(e) = std::fs::remove_file(&pending_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "sieve-hook: warning: failed to remove pending file {}: {e}",
                pending_path.display()
            );
        }
    }

    Ok(())
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

exec
/bin/zsh -lc "sed -n '260,460p' crates/sieve-hook/src/main.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 合并多个 DecisionRequest 的 detection，取最严 default_on_timeout。
fn merge_requests(reqs: &[protocol::DecisionRequest]) -> protocol::DecisionRequest {
    let mut all_detections = Vec::new();
    let mut worst_timeout = protocol::DefaultOnTimeout::Allow;
    let mut min_timeout_secs = u32::MAX;

    for req in reqs {
        all_detections.extend(req.detections.clone());
        // 最严策略：Block/Redact > Allow。
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            other => worst_timeout = other,
        }
        if req.timeout_seconds < min_timeout_secs {
            min_timeout_secs = req.timeout_seconds;
        }
    }

    let timeout_secs = if min_timeout_secs == u32::MAX {
        30
    } else {
        min_timeout_secs
    };

    protocol::DecisionRequest {
        // 启发式合并场景使用第一个请求的 id（仅用于日志）。
        request_id: reqs[0].request_id,
        created_at: reqs[0].created_at,
        timeout_seconds: timeout_secs,
        default_on_timeout: worst_timeout,
        detections: all_detections,
    }
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-ipc/src/pending_file.rs && nl -ba crates/sieve-hook/src/pending.rs | sed -n '60,120p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
    75	    let decisions_dir = base.join("decisions");
    76	
    77	    for entry in entries.flatten() {
    78	        let path = entry.path();
    79	        // 只处理 .json 文件。
    80	        if path.extension().and_then(|e| e.to_str()) != Some("json") {
    81	            continue;
    82	        }
    83	        let content = match std::fs::read_to_string(&path) {
    84	            Ok(c) => c,
    85	            Err(_) => continue,
    86	        };
    87	        let req: DecisionRequest = match serde_json::from_str(&content) {
    88	            Ok(r) => r,
    89	            Err(_) => continue, // 解析失败的文件跳过。
    90	        };
    91	
    92	        // 已决策的 pending 跳过（避免重复弹窗）。
    93	        // 若 decisions/<id>.json 已存在，说明该请求已被处理，不再加入 fresh/stale。
    94	        // 关联：SPEC-001 §4.3（清理机制）。
    95	        let decision_path = decisions_dir.join(format!("{}.json", req.request_id));
    96	        if decision_path.exists() {
    97	            continue;
    98	        }
    99	
   100	        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
   101	        if age_secs > stale_threshold_secs {
   102	            stale_paths.push(path);
   103	        } else {
   104	            fresh.push(req);
   105	        }
   106	    }
   107	
   108	    // 按 created_at 升序排列，保证确定性。
   109	    fresh.sort_by_key(|r| r.created_at);
   110	
   111	    ScanResult { fresh, stale_paths }
   112	}

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/src/main.rs crates/sieve-cli/src/cli.rs crates/sieve-cli/Cargo.toml | sed -n '1,240p' && nl -ba crates/sieve-cli/src/main.rs | sed -n '1,220p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
diff --git a/crates/sieve-cli/Cargo.toml b/crates/sieve-cli/Cargo.toml
index 24b05c8..673aa7b 100644
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
@@ -44,6 +47,9 @@ http-body-util = { workspace = true }
 http = { workspace = true }
 bytes = { workspace = true }
 sieve-core = { path = "../sieve-core" }
+sieve-ipc = { path = "../sieve-ipc" }
 anyhow = "1"
 tempfile = "3"
 serde_json = { workspace = true }
+rusqlite = { version = "0.31", features = ["bundled"] }
+uuid = { version = "1", features = ["v4"] }
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
   181	            .map(str::trim)
   182	            .filter(|l| !l.is_empty() && !l.starts_with('#'))
   183	            .map(String::from)
   184	            .collect(),
   185	        Err(e) => {
   186	            tracing::warn!(
   187	                path = %path.display(),
   188	                error = %e,
   189	                "failed to load .sieveignore; proceeding with empty allowlist"
   190	            );
   191	            HashSet::new()
   192	        }
   193	    }
   194	}
   195	
   196	fn init_tracing() {
   197	    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
   198	
   199	    let filter = EnvFilter::try_from_env("SIEVE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
   200	    tracing_subscriber::registry()
   201	        .with(filter)
   202	        .with(fmt::layer().with_target(false))
   203	        .init();
   204	}

exec
/bin/zsh -lc 'cat crates/sieve-cli/Cargo.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
sieve-ipc = { path = "../sieve-ipc" }
anyhow = "1"
tempfile = "3"
serde_json = { workspace = true }
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }

exec
/bin/zsh -lc 'cat crates/sieve-core/Cargo.toml && cat crates/sieve-ipc/Cargo.toml && cat crates/sieve-hook/Cargo.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '1,280p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
     1	//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup）。
     2	//!
     3	//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
     4	//!
     5	//! 步骤：
     6	//! 1. 检测 `~/.claude/settings.json` 是否存在
     7	//! 2. 计算 diff（ANTHROPIC_BASE_URL + PreToolUse hook + launchd plist）
     8	//! 3. dry-run 打印 diff，非 --yes 等待用户确认
     9	//! 4. 备份原文件到 `~/.sieve/backups/<RFC3339>/`
    10	//! 5. 写 `~/.sieve/sieve.toml`（默认配置，绝对路径）
    11	//! 6. 修改 settings.json（解析失败则 abort，不写任何内容）
    12	//! 7. 写 launchd plist（命令包含 `--config <abs_path>/sieve.toml`）+ `launchctl load -w`
    13	//! 8. 写 setup.log（JSON Lines，含 created_new 字段）
    14	//! 9. 自动调用 doctor 验证
    15	//!
    16	//! 错误恢复：任意步骤失败 → 反向回滚已做改动。
    17	
    18	use crate::cli::SetupArgs;
    19	use anyhow::Result;
    20	
    21	#[cfg(target_os = "macos")]
    22	pub use macos::run;
    23	
    24	#[cfg(not(target_os = "macos"))]
    25	pub use stub::run;
    26	
    27	// ──────────────────────────────── macOS 实现 ────────────────────────────────
    28	
    29	#[cfg(target_os = "macos")]
    30	mod macos {
    31	    use super::*;
    32	    use crate::commands::doctor;
    33	    use anyhow::{anyhow, bail, Context};
    34	    use chrono::Utc;
    35	    use serde_json::Value;
    36	    use std::fs;
    37	    use std::io::{self, Write as IoWrite};
    38	    use std::path::{Path, PathBuf};
    39	    use std::process::Command;
    40	
    41	    /// setup.log 每行的结构（JSON Lines）。
    42	    ///
    43	    /// `created_new`：true 表示 setup 前该文件不存在，由 setup 新建；
    44	    /// uninstall 时 `created_new=true` 的文件直接删除，`false` 的从备份恢复。
    45	    #[derive(serde::Serialize, serde::Deserialize)]
    46	    pub struct SetupLogEntry {
    47	        pub timestamp: String,
    48	        pub action: String,
    49	        pub path: Option<String>,
    50	        pub detail: Option<String>,
    51	        /// setup 前该文件是否不存在（新建 vs 覆盖）。
    52	        #[serde(default)]
    53	        pub created_new: bool,
    54	    }
    55	
    56	    impl SetupLogEntry {
    57	        pub(super) fn new(action: impl Into<String>) -> Self {
    58	            Self {
    59	                timestamp: Utc::now().to_rfc3339(),
    60	                action: action.into(),
    61	                path: None,
    62	                detail: None,
    63	                created_new: false,
    64	            }
    65	        }
    66	
    67	        pub(super) fn with_path(mut self, path: impl Into<String>) -> Self {
    68	            self.path = Some(path.into());
    69	            self
    70	        }
    71	
    72	        pub(super) fn with_detail(mut self, detail: impl Into<String>) -> Self {
    73	            self.detail = Some(detail.into());
    74	            self
    75	        }
    76	
    77	        pub(super) fn with_created_new(mut self, created_new: bool) -> Self {
    78	            self.created_new = created_new;
    79	            self
    80	        }
    81	    }
    82	
    83	    /// setup 执行上下文，用于错误时反向回滚。
    84	    struct SetupContext {
    85	        backup_dir: PathBuf,
    86	        /// 已写入的文件路径，错误时按逆序恢复。
    87	        written_files: Vec<PathBuf>,
    88	        /// 已执行的 launchctl load，错误时需要 unload。
    89	        launchd_loaded: Option<PathBuf>,
    90	    }
    91	
    92	    impl SetupContext {
    93	        fn new(backup_dir: PathBuf) -> Self {
    94	            Self {
    95	                backup_dir,
    96	                written_files: Vec::new(),
    97	                launchd_loaded: None,
    98	            }
    99	        }
   100	
   101	        /// 回滚所有已做改动（从备份目录恢复）。
   102	        fn rollback(&self) {
   103	            eprintln!("[sieve setup] 回滚已做改动…");
   104	
   105	            if let Some(plist) = &self.launchd_loaded {
   106	                let _ = Command::new("launchctl")
   107	                    .args(["unload", &plist.to_string_lossy()])
   108	                    .status();
   109	                eprintln!("  ↩ launchctl unload {}", plist.display());
   110	            }
   111	
   112	            for path in self.written_files.iter().rev() {
   113	                // 计算备份中的相对路径：去掉 HOME 前缀
   114	                let home = std::env::var("HOME").unwrap_or_default();
   115	                let rel = path.strip_prefix(&home).unwrap_or(path.as_path());
   116	                let backup_src = self.backup_dir.join(rel);
   117	                if backup_src.exists() {
   118	                    if let Err(e) = fs::copy(&backup_src, path) {
   119	                        eprintln!("  ✗ 恢复 {} 失败: {}", path.display(), e);
   120	                    } else {
   121	                        eprintln!("  ↩ 恢复 {}", path.display());
   122	                    }
   123	                } else {
   124	                    // 备份不存在说明是新建的，直接删除
   125	                    let _ = fs::remove_file(path);
   126	                    eprintln!("  ↩ 删除新建文件 {}", path.display());
   127	                }
   128	            }
   129	        }
   130	    }
   131	
   132	    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
   133	    pub fn run(args: SetupArgs) -> Result<()> {
   134	        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
   135	        let home_path = PathBuf::from(&home);
   136	
   137	        let settings_path = home_path.join(".claude").join("settings.json");
   138	        let sieve_home =
   139	            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
   140	        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
   141	        let backup_dir = sieve_home.join("backups").join(&backup_ts);
   142	        let plist_path = home_path
   143	            .join("Library")
   144	            .join("LaunchAgents")
   145	            .join("com.sieve.daemon.plist");
   146	        let setup_log_path = sieve_home.join("setup.log");
   147	
   148	        // ── 1. 读取现有 settings.json（允许不存在；解析失败则 abort，不覆盖用户文件）
   149	        let settings_existed_before = settings_path.exists();
   150	        let existing_settings: Value = if settings_existed_before {
   151	            let raw =
   152	                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
   153	            // Strip JSON 注释（简单处理：删除 // 行注释）
   154	            let stripped = strip_json_comments(&raw);
   155	            serde_json::from_str(&stripped).map_err(|e| {
   156	                anyhow!(
   157	                    "无法解析 ~/.claude/settings.json：{}。\n\
   158	                     请用 JSON 校验工具修复后重试。setup 已 abort，未做任何改动。",
   159	                    e
   160	                )
   161	            })?
   162	        } else {
   163	            serde_json::json!({})
   164	        };
   165	        // sieve.toml 将写入 ~/.sieve/sieve.toml（绝对路径）
   166	        let sieve_toml_path = sieve_home.join("sieve.toml");
   167	
   168	        // ── 2. 计算 diff
   169	        let sieve_url = "http://127.0.0.1:11453";
   170	        let hook_entry = serde_json::json!({
   171	            "matcher": ".*",
   172	            "hooks": [{"type": "command", "command": "sieve-hook check"}]
   173	        });
   174	
   175	        let current_base_url = existing_settings
   176	            .pointer("/env/ANTHROPIC_BASE_URL")
   177	            .and_then(|v| v.as_str())
   178	            .unwrap_or("<未设置>");
   179	        let has_hook = existing_settings
   180	            .pointer("/hooks/PreToolUse")
   181	            .and_then(|v| v.as_array())
   182	            .map(|arr| {
   183	                arr.iter().any(|item| {
   184	                    item.pointer("/hooks/0/command")
   185	                        .and_then(|c| c.as_str())
   186	                        .map(|c| c.contains("sieve-hook"))
   187	                        .unwrap_or(false)
   188	                })
   189	            })
   190	            .unwrap_or(false);
   191	        let plist_content = build_plist_content(&sieve_toml_path)?;
   192	
   193	        // ── 3. 打印 diff
   194	        println!("=== sieve setup diff ===");
   195	        println!(
   196	            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
   197	            current_base_url, sieve_url
   198	        );
   199	        if has_hook {
   200	            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
   201	        } else {
   202	            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
   203	        }
   204	        if sieve_toml_path.exists() {
   205	            println!(
   206	                "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
   207	                sieve_toml_path.display()
   208	            );
   209	        } else {
   210	            println!("[sieve.toml] 新建 {}", sieve_toml_path.display());
   211	        }
   212	        println!(
   213	            "[launchd] 写入 {} (含 --config {})",
   214	            plist_path.display(),
   215	            sieve_toml_path.display()
   216	        );
   217	        println!("[launchd] 执行 launchctl load -w");
   218	        println!("========================");
   219	
   220	        // ── 4. dry-run 直接返回
   221	        if args.dry_run {
   222	            println!("[dry-run] 未做任何改动。");
   223	            return Ok(());
   224	        }
   225	
   226	        // ── 5. 等待用户确认
   227	        if !args.yes {
   228	            print!("继续执行以上操作？[y/N] ");
   229	            io::stdout().flush()?;
   230	            let mut input = String::new();
   231	            io::stdin().read_line(&mut input)?;
   232	            if !input.trim().eq_ignore_ascii_case("y") {
   233	                println!("已取消。");
   234	                return Ok(());
   235	            }
   236	        }
   237	
   238	        // ── 6. 备份
   239	        fs::create_dir_all(&backup_dir)
   240	            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
   241	        let mut ctx = SetupContext::new(backup_dir.clone());
   242	
   243	        let result = do_setup(
   244	            &mut ctx,
   245	            &home_path,
   246	            &settings_path,
   247	            &plist_path,
   248	            &sieve_toml_path,
   249	            &setup_log_path,
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '280,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
   501	    ///
   502	    /// 生成的内容与 [`crate::config::Config`] 的扁平字段完全匹配（`deny_unknown_fields`），
   503	    /// 可直接被 `toml::from_str::<Config>()` 反序列化而不报错。
   504	    pub(super) fn build_default_sieve_toml(sieve_toml_path: &Path) -> Result<String> {
   505	        let sieve_home = sieve_toml_path
   506	            .parent()
   507	            .ok_or_else(|| anyhow!("sieve.toml 路径无父目录"))?;
   508	        let rules_path = sieve_home.join("rules").join("outbound.toml");
   509	        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
   510	        let audit_db = sieve_home.join("audit.db");
   511	        let ipc_socket = sieve_home.join("ipc.sock");
   512	        let pending_dir = sieve_home.join("pending");
   513	        let decisions_dir = sieve_home.join("decisions");
   514	        let home = std::env::var_os("HOME")
   515	            .map(std::path::PathBuf::from)
   516	            .unwrap_or_else(|| sieve_home.to_path_buf());
   517	        let launchd_plist = home
   518	            .join("Library")
   519	            .join("LaunchAgents")
   520	            .join("com.sieve.daemon.plist");
   521	
   522	        Ok(format!(
   523	            r#"# sieve.toml — 由 `sieve setup` 自动生成，所有路径为绝对路径
   524	# 修改后需重启 daemon：launchctl kickstart -k gui/$(id -u)/com.sieve.daemon
   525	
   526	upstream_url = "https://api.anthropic.com"
   527	port = 11453
   528	bind_addr = "127.0.0.1"
   529	tls_verify_upstream = true
   530	dry_run = false
   531	preset = "default"
   532	gui_socket_enabled = false
   533	
   534	# 出站规则文件路径（绝对路径，launchd 从 / 启动时不依赖 cwd）
   535	rules_path = "{rules_path}"
   536	
   537	# 入站规则文件路径
   538	inbound_rules_path = "{inbound_rules_path}"
   539	
   540	# 审计日志数据库路径（绝对路径）
   541	audit_db_path = "{audit_db}"
   542	
   543	# IPC Unix socket 路径
   544	ipc_socket_path = "{ipc_socket}"
   545	
   546	# 待决策 / 已决策文件目录
   547	pending_dir = "{pending_dir}"
   548	decisions_dir = "{decisions_dir}"
   549	
   550	# launchd plist 路径（macOS）
   551	launchd_plist_path = "{launchd_plist}"
   552	"#,
   553	            rules_path = rules_path.display(),
   554	            inbound_rules_path = inbound_rules_path.display(),
   555	            audit_db = audit_db.display(),
   556	            ipc_socket = ipc_socket.display(),
   557	            pending_dir = pending_dir.display(),
   558	            decisions_dir = decisions_dir.display(),
   559	            launchd_plist = launchd_plist.display(),
   560	        ))
   561	    }
   562	
   563	    /// 简单去除 `// ...` 行注释（不处理字符串内的 `//`，够用于 settings.json）。
   564	    pub(super) fn strip_json_comments(s: &str) -> String {
   565	        s.lines()
   566	            .map(|line| {
   567	                // 找到不在引号内的 `//`
   568	                let mut in_string = false;
   569	                let mut escaped = false;
   570	                let mut comment_start = None;
   571	                let chars: Vec<char> = line.chars().collect();
   572	                let mut i = 0;
   573	                while i < chars.len() {
   574	                    if escaped {
   575	                        escaped = false;
   576	                    } else if chars[i] == '\\' && in_string {
   577	                        escaped = true;
   578	                    } else if chars[i] == '"' {
   579	                        in_string = !in_string;
   580	                    } else if !in_string
   581	                        && chars[i] == '/'
   582	                        && i + 1 < chars.len()
   583	                        && chars[i + 1] == '/'
   584	                    {
   585	                        comment_start = Some(i);
   586	                        break;
   587	                    }
   588	                    i += 1;
   589	                }
   590	                if let Some(pos) = comment_start {
   591	                    line[..pos].to_string()
   592	                } else {
   593	                    line.to_string()
   594	                }
   595	            })
   596	            .collect::<Vec<_>>()
   597	            .join("\n")
   598	    }
   599	}
   600	
   601	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   602	
   603	#[cfg(not(target_os = "macos"))]
   604	mod stub {
   605	    use super::*;
   606	
   607	    /// `sieve setup` 非 macOS 占位实现。
   608	    /// Phase 1 仅支持 macOS；Linux/Windows 在 Phase 2 规划（ADR-015）。
   609	    pub fn run(_args: SetupArgs) -> Result<()> {
   610	        anyhow::bail!(
   611	            "sieve setup is macOS only in Phase 1. \
   612	             Linux/Windows support is planned for Phase 2."
   613	        )
   614	    }
   615	}
   616	
   617	// ──────────────────────────────── 单元测试 ──────────────────────────────────
   618	
   619	#[cfg(test)]
   620	#[cfg(target_os = "macos")]

exec
/bin/zsh -lc "rg \"rules\" -n crates/sieve-cli/src/commands/setup.rs | sed -n '1,120p' && rg \"copy\\(|outbound.toml|inbound.toml\" -n crates/sieve-cli/src/commands/setup.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
508:        let rules_path = sieve_home.join("rules").join("outbound.toml");
509:        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
535:rules_path = "{rules_path}"
538:inbound_rules_path = "{inbound_rules_path}"
553:            rules_path = rules_path.display(),
554:            inbound_rules_path = inbound_rules_path.display(),
705:            content.contains(&format!("rules_path = \"{}", dir.path().display())),
706:            "rules_path 必须是绝对路径: {content}"
729:        assert!(cfg.rules_path.is_some(), "rules_path 应有绝对路径");
118:                    if let Err(e) = fs::copy(&backup_src, path) {
295:            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
371:                fs::copy(sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
394:                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
508:        let rules_path = sieve_home.join("rules").join("outbound.toml");
509:        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '1,360p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
   250	                    // toml 文件同样按 created_new 判断：
   251	                    // - created_new=false → setup 前用户已有该文件，从备份恢复
   252	                    // - created_new=true  → setup 新建，但 created_new=true 分支在上面已处理
   253	                    // 此处 created_new 必定为 false（else 分支），从备份恢复用户原文件。
   254	                    if let Some(bd) = backup_dir {
   255	                        restore_file_from_backup(bd, &info.path)?;
   256	                    } else {
   257	                        // 无备份可恢复：只能删除（避免残留 Sieve 配置影响用户）
   258	                        fs::remove_file(&info.path).with_context(|| {
   259	                            format!("删除 {} 失败（无备份）", info.path.display())
   260	                        })?;
   261	                        println!("[uninstall] ✅ 删除（无备份）: {}", info.path.display());
   262	                    }
   263	                } else {
   264	                    // 其他文件：从备份恢复
   265	                    if let Some(bd) = backup_dir {
   266	                        restore_file_from_backup(bd, &info.path)?;
   267	                    }
   268	                }
   269	            }
   270	        }
   271	        Ok(())
   272	    }
   273	
   274	    /// 从 settings.json 中移除 Sieve 注入的 entries，保留用户其他配置。
   275	    ///
   276	    /// 移除：
   277	    /// - `env.ANTHROPIC_BASE_URL`（若值为 `http://127.0.0.1:11453`）
   278	    /// - `hooks.PreToolUse` 数组中包含 `sieve-hook` 的条目
   279	    pub(super) fn remove_sieve_entries_from_settings(
   280	        settings_path: &std::path::Path,
   281	    ) -> Result<()> {
   282	        let raw = fs::read_to_string(settings_path)
   283	            .with_context(|| format!("读取 {} 失败", settings_path.display()))?;
   284	        let mut v: serde_json::Value = serde_json::from_str(&raw)
   285	            .with_context(|| format!("解析 {} 失败", settings_path.display()))?;
   286	
   287	        // 移除 env.ANTHROPIC_BASE_URL（仅当值为 sieve url 时）
   288	        if let Some(env) = v.get_mut("env").and_then(|e| e.as_object_mut()) {
   289	            if env
   290	                .get("ANTHROPIC_BASE_URL")
   291	                .and_then(|u| u.as_str())
   292	                .map(|s| s == "http://127.0.0.1:11453")
   293	                .unwrap_or(false)
   294	            {
   295	                env.remove("ANTHROPIC_BASE_URL");
   296	                // 如果 env 对象变空，也一并移除（避免留下空对象）
   297	                if env.is_empty() {
   298	                    v.as_object_mut().map(|obj| obj.remove("env"));
   299	                }
   300	            }
   301	        }
   302	
   303	        // 移除 hooks.PreToolUse 中含 sieve-hook 的条目
   304	        if let Some(pre_tool) = v
   305	            .pointer_mut("/hooks/PreToolUse")
   306	            .and_then(|a| a.as_array_mut())
   307	        {
   308	            pre_tool.retain(|item| {
   309	                !item
   310	                    .pointer("/hooks/0/command")
   311	                    .and_then(|c| c.as_str())
   312	                    .map(|c| c.contains("sieve-hook"))
   313	                    .unwrap_or(false)
   314	            });
   315	        }
   316	        // 如果 hooks.PreToolUse 变空，移除该 key
   317	        let pre_tool_empty = v
   318	            .pointer("/hooks/PreToolUse")
   319	            .and_then(|a| a.as_array())
   320	            .map(|a| a.is_empty())
   321	            .unwrap_or(false);
   322	        if pre_tool_empty {
   323	            if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
   324	                hooks.remove("PreToolUse");
   325	                if hooks.is_empty() {
   326	                    v.as_object_mut().map(|obj| obj.remove("hooks"));
   327	                }
   328	            }
   329	        }
   330	
   331	        let json_str = serde_json::to_string_pretty(&v)?;
   332	        fs::write(settings_path, json_str.as_bytes())
   333	            .with_context(|| format!("写入 {} 失败", settings_path.display()))?;
   334	        Ok(())
   335	    }
   336	
   337	    /// 从备份目录恢复单个文件。
   338	    fn restore_file_from_backup(
   339	        backup_dir: &std::path::Path,
   340	        target: &std::path::Path,
   341	    ) -> Result<()> {
   342	        // 计算 backup 中的对应路径（target 的绝对路径去掉 HOME 前缀）
   343	        let home = std::env::var("HOME").unwrap_or_default();
   344	        let rel = target.strip_prefix(&home).unwrap_or(target);
   345	        let backup_src = backup_dir.join(rel);
   346	        if backup_src.exists() {
   347	            if let Some(parent) = target.parent() {
   348	                fs::create_dir_all(parent)?;
   349	            }
   350	            fs::copy(&backup_src, target).with_context(|| {
   351	                format!(
   352	                    "从备份恢复 {} → {} 失败",
   353	                    backup_src.display(),
   354	                    target.display()
   355	                )
   356	            })?;
   357	            println!("[uninstall] ✅ 从备份恢复: {}", target.display());
   358	        } else {
   359	            eprintln!("[uninstall] ⚠ 备份文件不存在: {}", backup_src.display());
   360	        }

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/tests/outbound_block.rs crates/sieve-cli/tests/inbound_block.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
diff --git a/crates/sieve-cli/tests/inbound_block.rs b/crates/sieve-cli/tests/inbound_block.rs
index 4a4e424..2217ac0 100644
--- a/crates/sieve-cli/tests/inbound_block.rs
+++ b/crates/sieve-cli/tests/inbound_block.rs
@@ -445,11 +445,16 @@ async fn ucsb_attack_1_address_substitution_blocked() {
 
 // ─── UCSB Attack 2: Dangerous Shell in tool_use input（IN-CR-02）──────────────
 
-/// tool_use input_json_delta 含 `rm -rf /tmp` → 聚合完成后 IN-CR-02 触发截流。
+/// tool_use input_json_delta 含 `rm -rf /tmp` → IN-CR-02 触发 HookMark（hook_terminal）。
 ///
-/// 关联 PRD §5.2 IN-CR-02 / UCSB 论文 attack 2。
+/// IN-CR-02 disposition=hook_terminal：SSE 流原样转发（不截流），写 IPC pending 文件，
+/// 由 sieve-hook 在 PreToolUse 阶段拦截。sieve daemon 本身不注入 sieve_blocked。
+///
+/// 修 #2（disposition 优先）后：IN-CR-02 走 HookMark 路径，而非旧的 Block 路径。
+///
+/// 关联 PRD §5.2 IN-CR-02 / UCSB 论文 attack 2 / ADR-014（双层防御）。
 #[tokio::test]
-async fn ucsb_attack_2_dangerous_shell_in_tool_use_blocked() {
+async fn ucsb_attack_2_dangerous_shell_hookmark_passthrough() {
     let attack_payload = sse_response(&[
         (
             "message_start",
@@ -481,13 +486,15 @@ async fn ucsb_attack_2_dangerous_shell_in_tool_use_blocked() {
     let (_status, body) = fetch_response_body(port).await;
 
     let body_str = String::from_utf8_lossy(&body);
+    // HookMark 路径：SSE 流原样转发（不截流），不注入 sieve_blocked
+    // sieve-hook 在 PreToolUse 阶段处理拦截，daemon 仅写 pending 文件
     assert!(
-        body_str.contains("sieve_blocked"),
-        "expected sieve_blocked event in body:\n{body_str}"
+        !body_str.contains("sieve_blocked"),
+        "IN-CR-02 hook_terminal 路径不应截流，SSE 应原样转发:\n{body_str}"
     );
     assert!(
-        body_str.contains("IN-CR-02"),
-        "expected IN-CR-02 rule in detection:\n{body_str}"
+        body_str.contains("message_stop"),
+        "SSE 流应包含 message_stop（完整透传）:\n{body_str}"
     );
 }
 
@@ -543,10 +550,18 @@ async fn ucsb_attack_3_signing_tool_blocked() {
 
 /// IN-GEN-04 是 high/warn，不在 fail-closed 名单，不截流，响应内容包含原始 event。
 ///
-/// 关联 PRD §5.2 / US-08 / UCSB 论文 attack 4。Week 4 由旧 IN-CR-04 重命名归入
-/// IN-GEN-* 命名空间（[BREAKING]，fingerprint 变化）。
+/// IN-GEN-04 markdown exfil → disposition=gui_popup → HoldForDecision → fail-closed（无 GUI）。
+///
+/// IN-GEN-04 disposition=gui_popup：无 GUI 连接时 fail-closed，注入 sieve_blocked 截流。
+/// 修 #2（disposition 优先）后行为变化：旧版 action=warn（StatusBar/pass-through），
+/// 新版 disposition=gui_popup 优先，无 GUI → Block。
+///
+/// 注：IN-GEN-04 设为 gui_popup 是设计决策（exfil 需要用户确认），但生产中应有 GUI 连接。
+/// 集成测试中无 GUI，因此 fail-closed → sieve_blocked 被注入。
+///
+/// 关联 PRD §5.2 / US-08 / UCSB 论文 attack 4 / ADR-014（双层防御）。
 #[tokio::test]
-async fn ucsb_attack_4_markdown_exfil_warn_only_passes_through() {
+async fn ucsb_attack_4_markdown_exfil_failclosed_without_gui() {
     let attack_payload = sse_response(&[
         (
             "content_block_delta",
@@ -567,36 +582,34 @@ async fn ucsb_attack_4_markdown_exfil_warn_only_passes_through() {
     assert_eq!(
         status,
         hyper::StatusCode::OK,
-        "warn-level rule should not affect status"
+        "SSE response status should be 200 (sieve_blocked is injected into body)"
     );
 
     let body_str = String::from_utf8_lossy(&body);
+    // IN-GEN-04 gui_popup + 无 GUI → fail-closed → sieve_blocked 注入
     assert!(
-        !body_str.contains("sieve_blocked"),
-        "IN-GEN-04 is warn-only, must not inject sieve_blocked:\n{body_str}"
-    );
-    // body 应该包含原始 event（透传，不被截断）
-    assert!(
-        body_str.contains("content_block_delta"),
-        "warn-level response should contain original SSE events:\n{body_str}"
+        body_str.contains("sieve_blocked"),
+        "IN-GEN-04 gui_popup 无 GUI 时应 fail-closed 注入 sieve_blocked:\n{body_str}"
     );
     assert!(
-        body_str.contains("message_stop"),
-        "warn-level response should contain message_stop event:\n{body_str}"
+        body_str.contains("IN-GEN-04"),
+        "sieve_blocked 应包含 IN-GEN-04 rule_id:\n{body_str}"
     );
 }
 
 // ─── IN-CR-04: 持久化机制（Critical block，Week 4，PRD §5.2 / US-07）──────────
 
-/// tool_use Bash command 含 `>> ~/.bashrc` → IN-CR-04-SHELL-RC-APPEND 触发
-/// Critical fail-closed 截流。
+/// tool_use Bash command 含 `>> ~/.bashrc` → IN-CR-04-SHELL-RC-APPEND 触发 HookMark。
 ///
-/// 模型试图把 alias 写进 shell rc 文件 = 后门埋点（每次开 terminal 自动执行）。
-/// IN-CR-04-* 全部进 fail-closed 名单（ADR-007 §"Week 4 落地范围"），YOLO mode 不可关。
+/// IN-CR-04-SHELL-RC-APPEND disposition=hook_terminal：SSE 流原样转发（不截流），
+/// 写 IPC pending 文件，由 sieve-hook 在 PreToolUse 阶段拦截（双层防御）。
+/// daemon 本身不注入 sieve_blocked——截流由 sieve-hook 的 exit code 机制完成。
 ///
-/// 关联 PRD §5.2 IN-CR-04 / Roadmap Week 4 / US-07。
+/// 修 #2（disposition 优先）后：IN-CR-04 走 HookMark 路径，旧的直接 Block 路径已更新。
+///
+/// 关联 PRD §5.2 IN-CR-04 / Roadmap Week 4 / US-07 / ADR-014（双层防御）。
 #[tokio::test]
-async fn in_cr_04_persistence_shell_rc_blocked() {
+async fn in_cr_04_persistence_shell_rc_hookmark_passthrough() {
     let attack_payload = sse_response(&[
         (
             "message_start",
@@ -628,13 +641,15 @@ async fn in_cr_04_persistence_shell_rc_blocked() {
     let (_status, body) = fetch_response_body(port).await;
 
     let body_str = String::from_utf8_lossy(&body);
+    // HookMark 路径：SSE 流原样转发（不截流），不注入 sieve_blocked
+    // 截流由 sieve-hook PreToolUse exit code 机制完成（双层防御设计）
     assert!(
-        body_str.contains("sieve_blocked"),
-        "expected sieve_blocked event for IN-CR-04 persistence:\n{body_str}"
+        !body_str.contains("sieve_blocked"),
+        "IN-CR-04 hook_terminal 路径不应截流，SSE 应原样转发:\n{body_str}"
     );
     assert!(
-        body_str.contains("IN-CR-04-SHELL-RC-APPEND"),
-        "expected IN-CR-04-SHELL-RC-APPEND rule in detection:\n{body_str}"
+        body_str.contains("message_stop"),
+        "SSE 流应包含 message_stop（完整透传）:\n{body_str}"
     );
 }
 
diff --git a/crates/sieve-cli/tests/outbound_block.rs b/crates/sieve-cli/tests/outbound_block.rs
index 9cfc390..76d4da9 100644
--- a/crates/sieve-cli/tests/outbound_block.rs
+++ b/crates/sieve-cli/tests/outbound_block.rs
@@ -107,6 +107,8 @@ struct DaemonGuard {
     proc: Child,
     // 持有 tempfile 引用，防止进程运行期间被删除
     _config_file: tempfile::NamedTempFile,
+    /// 持有 sieve home 临时目录（若设置了），防止 Drop 时被清理
+    _sieve_home: Option<tempfile::TempDir>,
 }
 
 impl Drop for DaemonGuard {
@@ -121,6 +123,17 @@ fn drop(&mut self) {
 /// 写临时 sieve.toml，其中 rules_path 为绝对路径（避免 cwd 歧义）。
 /// `tls_verify_upstream = false`：mock 上游是 plain HTTP，不需要 TLS 握手。
 fn spawn_sieve_daemon(upstream_url: &str, dry_run: bool) -> (u16, DaemonGuard) {
+    spawn_sieve_daemon_with_home(upstream_url, dry_run, None)
+}
+
+/// 启动真实 sieve daemon，支持传入自定义 `sieve_home`（供 IPC 集成测试使用）。
+///
+/// `sieve_home`：若 Some，则设置 `SIEVE_HOME` 环境变量；daemon 会把 IPC socket 放在此目录下。
+fn spawn_sieve_daemon_with_home(
+    upstream_url: &str,
+    dry_run: bool,
+    sieve_home: Option<&std::path::Path>,
+) -> (u16, DaemonGuard) {
     let port = find_free_port();
     let rules = outbound_rules_path();
     assert!(
@@ -161,15 +174,19 @@ fn spawn_sieve_daemon(upstream_url: &str, dry_run: bool) -> (u16, DaemonGuard) {
         binary.display()
     );
 
-    let proc = Command::new(&binary)
-        .arg("start")
+    let mut cmd = Command::new(&binary);
+    cmd.arg("start")
         .arg("--config")
         .arg(config_file.path())
         .env("SIEVE_LOG", "warn")
         .stdout(Stdio::null())
-        .stderr(Stdio::null())
-        .spawn()
-        .expect("spawn sieve daemon");
+        .stderr(Stdio::null());
+
+    if let Some(home) = sieve_home {
+        cmd.env("SIEVE_HOME", home);
+    }
+
+    let proc = cmd.spawn().expect("spawn sieve daemon");
 
     // 等 daemon 监听，最长 10 秒
     let deadline = Instant::now() + Duration::from_secs(10);
@@ -193,6 +210,7 @@ fn spawn_sieve_daemon(upstream_url: &str, dry_run: bool) -> (u16, DaemonGuard) {
         DaemonGuard {
             proc,
             _config_file: config_file,
+            _sieve_home: None,
         },
     )
 }
@@ -225,28 +243,40 @@ fn plain_http_client() -> Client<HttpConnector, Full<Bytes>> {
 
 // ─── 测试 ──────────────────────────────────────────────────────────────────────
 
-/// POST /v1/messages 含 fake Anthropic key → 返回 426 + sieve_blocked JSON body。
+/// POST /v1/messages 含 fake Anthropic key → AutoRedact 脱敏后转发上游（200）。
+///
+/// OUT-01 有 disposition=auto_redact，修 #2（disposition 优先于 enforce_action）后：
+/// fail-closed 名单里的 OUT-01 不再直接 Block，而是先脱敏再转发。
+/// 验证 PRD v1.4 §6.1（AutoRedact 路径）。
 ///
-/// 关联 PRD §5.1 OUT-01 / ADR-008。
+/// 关联 PRD §5.1 OUT-01 / ADR-016（二维处置矩阵）。
 #[tokio::test]
-async fn fake_anthropic_key_blocked_with_426() {
-    // 1. 启动 mock 上游（若 sieve 转发了请求，计数器就不为 0，测试 fail）
+async fn fake_anthropic_key_auto_redacted_and_forwarded() {
+    // 1. 启动 mock 上游（OUT-01 AutoRedact → sieve 脱敏后转发，计数器应为 1）
     let upstream_call_count = Arc::new(AtomicUsize::new(0));
     let counter_clone = upstream_call_count.clone();
 
-    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
+    // 记录上游收到的请求 body（验证 key 已被脱敏）
+    let upstream_body_received = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
+    let body_clone = upstream_body_received.clone();
+
+    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
         let c = counter_clone.clone();
+        let b = body_clone.clone();
         async move {
             c.fetch_add(1, Ordering::SeqCst);
+            let mut guard = b.lock().await;
+            *guard = req.body().clone();
+            drop(guard);
             Response::builder()
                 .status(200)
-                .body(Full::new(Bytes::from_static(b"upstream-not-blocked")))
+                .body(Full::new(Bytes::from_static(b"ok-from-upstream")))
                 .unwrap()
         }
     })
     .await;
 
-    // 2. 启动 sieve daemon（指向 mock 上游，不是真实 Anthropic）
+    // 2. 启动 sieve daemon（指向 mock 上游）
     let (sieve_port, _guard) = spawn_sieve_daemon(
         &format!("http://{upstream_addr}"),
         false, /* dry_run=false */
@@ -268,43 +298,40 @@ async fn fake_anthropic_key_blocked_with_426() {
         .await
         .unwrap();
 
-    // 4. 验证 status = 426
+    // 4. OUT-01 AutoRedact：脱敏后转发，上游返回 200
     assert_eq!(
         resp.status(),
-        StatusCode::UPGRADE_REQUIRED,
-        "expected 426 Upgrade Required (OUT-01 block)"
+        StatusCode::OK,

exec
/bin/zsh -lc "git diff -- crates/sieve-cli/tests/outbound_block.rs | sed -n '260,620p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
+        // 构造回复
+        let resp = sieve_ipc::protocol::DecisionResponse {
+            request_id: real_id,
+            decision,
+            decided_at: chrono::Utc::now(),
+            by_user: true,
+            remember: false,
+        };
+        let rpc_resp = sieve_ipc::protocol::jsonrpc::Response {
+            jsonrpc: "2.0".to_owned(),
+            result: Some(serde_json::to_value(&resp)?),
+            error: None,
+            id: serde_json::Value::String(real_id.to_string()),
+        };
+        let mut payload = serde_json::to_string(&rpc_resp)?;
+        payload.push('\n');
+        writer.write_all(payload.as_bytes()).await?;
+        break;
+    }
+    Ok(())
+}
+
+/// 构造含 PEM private key 的请求 body（触发 OUT-07，disposition=gui_popup）。
+fn pem_key_body() -> String {
+    serde_json::json!({
+        "model": "claude-sonnet-4-5",
+        "max_tokens": 16,
+        "messages": [{
+            "role": "user",
+            "content": "这是我的密钥：-----BEGIN EC PRIVATE KEY-----\nMHQCAQEEINsamplekey\n-----END EC PRIVATE KEY-----",
+        }],
+    })
+    .to_string()
+}
+
+/// OUT-07 GuiPopup hold：GUI Deny → 客户端收到 426，上游未被调用。
+///
+/// 验证 R2-#1 修复：daemon 出站路径正确处理 HoldForDecision action。
+/// 关联：PRD v1.4 §5.4.2（出站超时策略表）、ADR-016（二维处置矩阵）。
+#[tokio::test]
+async fn outbound_gui_popup_deny_returns_426() {
+    let upstream_call_count = Arc::new(AtomicUsize::new(0));
+    let counter_clone = upstream_call_count.clone();
+
+    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
+        let c = counter_clone.clone();
+        async move {
+            c.fetch_add(1, Ordering::SeqCst);
+            Response::builder()
+                .status(200)
+                .body(Full::new(Bytes::from_static(b"should-not-reach")))
+                .unwrap()
+        }
+    })
+    .await;
+
+    // 为 IPC 准备临时目录
+    let sieve_home_dir = tempfile::tempdir().unwrap();
+    let sieve_home = sieve_home_dir.path().to_owned();
+    let socket_path = sieve_home.join("ipc.sock");
+
+    let (sieve_port, _guard) =
+        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));
+
+    // 启动 GUI 模拟任务：先连接 IPC socket（通知 ready），再等 request_decision，回复 Deny
+    let socket_path_clone = socket_path.clone();
+    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
+    let ipc_task = tokio::spawn(async move {
+        let _ = mock_gui_respond_with_ready(
+            &socket_path_clone,
+            sieve_ipc::DecisionAction::Deny,
+            ready_tx,
+        )
+        .await;
+    });
+
+    // 等 GUI 已连接后再发 HTTP 请求，确保 IPC gui_writer 不为 None（最多等 15 秒）
+    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;
+
+    let body = pem_key_body();
+    let client = plain_http_client();
+    let resp = client
+        .request(
+            Request::builder()
+                .method(http::Method::POST)
+                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
+                .header(http::header::CONTENT_TYPE, "application/json")
+                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
+                .body(Full::new(Bytes::from(body)))
+                .unwrap(),
+        )
+        .await
+        .unwrap();
+
+    let _ = ipc_task.await;
+
+    // GUI Deny → 426
+    assert_eq!(
+        resp.status(),
+        StatusCode::UPGRADE_REQUIRED,
+        "OUT-07 GuiPopup GUI Deny 应返回 426"
+    );
+
+    // 上游不应被调用
+    assert_eq!(
+        upstream_call_count.load(Ordering::SeqCst),
+        0,
+        "GUI Deny 后上游不应被调用"
+    );
+}
+
+/// OUT-07 GuiPopup hold：GUI Allow → 请求转发上游，上游返回 200。
+///
+/// 验证 R2-#1 修复：Allow 决策后原 body 转发给上游。
+#[tokio::test]
+async fn outbound_gui_popup_allow_forwards_to_upstream() {
+    let upstream_call_count = Arc::new(AtomicUsize::new(0));
+    let counter_clone = upstream_call_count.clone();
+
+    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
+        let c = counter_clone.clone();
+        async move {
+            c.fetch_add(1, Ordering::SeqCst);
+            Response::builder()
+                .status(200)
+                .body(Full::new(Bytes::from_static(b"upstream-ok")))
+                .unwrap()
+        }
+    })
+    .await;
+
+    let sieve_home_dir = tempfile::tempdir().unwrap();
+    let sieve_home = sieve_home_dir.path().to_owned();
+    let socket_path = sieve_home.join("ipc.sock");
+
+    let (sieve_port, _guard) =
+        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));
+
+    // 启动 GUI 模拟任务：先连接 IPC socket（通知 ready），再等 request_decision，回复 Allow
+    let socket_path_clone = socket_path.clone();
+    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
+    let ipc_task = tokio::spawn(async move {
+        let _ = mock_gui_respond_with_ready(
+            &socket_path_clone,
+            sieve_ipc::DecisionAction::Allow,
+            ready_tx,
+        )
+        .await;
+    });
+
+    // 等 GUI 已连接后再发 HTTP 请求
+    let _ = tokio::time::timeout(Duration::from_secs(5), ready_rx).await;
+
+    let body = pem_key_body();
+    let client = plain_http_client();
+    let resp = client
+        .request(
+            Request::builder()
+                .method(http::Method::POST)
+                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
+                .header(http::header::CONTENT_TYPE, "application/json")
+                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
+                .body(Full::new(Bytes::from(body)))
+                .unwrap(),
+        )
+        .await
+        .unwrap();
+
+    let _ = ipc_task.await;
+
+    // GUI Allow → 请求到达上游（200）
+    assert_eq!(
+        resp.status(),
+        StatusCode::OK,
+        "OUT-07 GuiPopup GUI Allow 后应转发到上游并返回 200"
+    );
+    assert_eq!(
+        upstream_call_count.load(Ordering::SeqCst),
+        1,
+        "GUI Allow 后上游应被调用一次"
+    );
+}

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/outbound_block.rs | sed -n '120,230p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   120	
   121	/// 启动真实 sieve daemon，返回 (listen_port, guard)。
   122	///
   123	/// 写临时 sieve.toml，其中 rules_path 为绝对路径（避免 cwd 歧义）。
   124	/// `tls_verify_upstream = false`：mock 上游是 plain HTTP，不需要 TLS 握手。
   125	fn spawn_sieve_daemon(upstream_url: &str, dry_run: bool) -> (u16, DaemonGuard) {
   126	    spawn_sieve_daemon_with_home(upstream_url, dry_run, None)
   127	}
   128	
   129	/// 启动真实 sieve daemon，支持传入自定义 `sieve_home`（供 IPC 集成测试使用）。
   130	///
   131	/// `sieve_home`：若 Some，则设置 `SIEVE_HOME` 环境变量；daemon 会把 IPC socket 放在此目录下。
   132	fn spawn_sieve_daemon_with_home(
   133	    upstream_url: &str,
   134	    dry_run: bool,
   135	    sieve_home: Option<&std::path::Path>,
   136	) -> (u16, DaemonGuard) {
   137	    let port = find_free_port();
   138	    let rules = outbound_rules_path();
   139	    assert!(
   140	        rules.exists(),
   141	        "outbound rules not found at {}",
   142	        rules.display()
   143	    );
   144	    let inbound_rules = inbound_rules_path();
   145	    assert!(
   146	        inbound_rules.exists(),
   147	        "inbound rules not found at {}",
   148	        inbound_rules.display()
   149	    );
   150	
   151	    let mut config_file = tempfile::NamedTempFile::new().unwrap();
   152	    writeln!(
   153	        config_file,
   154	        r#"upstream_url = "{}"
   155	port = {}
   156	bind_addr = "127.0.0.1"
   157	rules_path = "{}"
   158	inbound_rules_path = "{}"
   159	tls_verify_upstream = false
   160	dry_run = {}
   161	"#,
   162	        upstream_url,
   163	        port,
   164	        rules.display(),
   165	        inbound_rules.display(),
   166	        dry_run,
   167	    )
   168	    .unwrap();
   169	
   170	    let binary = sieve_binary();
   171	    assert!(
   172	        binary.exists(),
   173	        "sieve binary not found at {}; run `cargo build --release` first",
   174	        binary.display()
   175	    );
   176	
   177	    let mut cmd = Command::new(&binary);
   178	    cmd.arg("start")
   179	        .arg("--config")
   180	        .arg(config_file.path())
   181	        .env("SIEVE_LOG", "warn")
   182	        .stdout(Stdio::null())
   183	        .stderr(Stdio::null());
   184	
   185	    if let Some(home) = sieve_home {
   186	        cmd.env("SIEVE_HOME", home);
   187	    }
   188	
   189	    let proc = cmd.spawn().expect("spawn sieve daemon");
   190	
   191	    // 等 daemon 监听，最长 10 秒
   192	    let deadline = Instant::now() + Duration::from_secs(10);
   193	    loop {
   194	        if std::net::TcpStream::connect_timeout(
   195	            &format!("127.0.0.1:{port}").parse().unwrap(),
   196	            Duration::from_millis(500),
   197	        )
   198	        .is_ok()
   199	        {
   200	            break;
   201	        }
   202	        if Instant::now() >= deadline {
   203	            panic!("sieve daemon did not listen on :{port} within 10 s");
   204	        }
   205	        std::thread::sleep(Duration::from_millis(100));
   206	    }
   207	
   208	    (
   209	        port,
   210	        DaemonGuard {
   211	            proc,
   212	            _config_file: config_file,
   213	            _sieve_home: None,
   214	        },
   215	    )
   216	}
   217	
   218	/// 构造含 fake Anthropic key 的 /v1/messages 请求 body。
   219	///
   220	/// key 格式：sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA"（符合 OUT-01 pattern）。
   221	fn fake_key_body() -> String {
   222	    // sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" = 108 chars,符合 OUT-01 pattern
   223	    let suffix_93: String = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-"
   224	        .chars()
   225	        .cycle()
   226	        .take(93)
   227	        .collect();
   228	    let api_key = format!("sk-ant-api03-{}AA", suffix_93);
   229	    serde_json::json!({
   230	        "model": "claude-sonnet-4-5",

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-ipc/src/protocol.rs && sed -n '1,180p' crates/sieve-ipc/src/decision_file.rs && sed -n '1,140p' crates/sieve-ipc/src/paths.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{decisions_dir, ensure_dirs, locks_dir},
    protocol::{DecisionAction, DecisionResponse},
};

/// 将 [`DecisionResponse`] 写入 `<base>/decisions/<request_id>.json`。
///
/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁，确保并发写入安全
///（hook 与 GUI 极少同时操作同一 request_id，但防御性加锁是正确做法）。
///
/// 关联：SPEC-001 §3.3（决策文件写入规约）。
pub fn write_decision(resp: &DecisionResponse, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let lock_path = locks_dir(base).join(format!("{}.lock", resp.request_id));
    let dec_path = decisions_dir(base).join(format!("{}.json", resp.request_id));

    // 创建锁文件（若不存在），然后加独占写锁。
    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;

    let mut lock = RwLock::new(lock_file);
    {
        let _guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;

        let json = serde_json::to_string_pretty(resp)?;
        std::fs::write(&dec_path, json.as_bytes())?;
    }

    // decisions 写入成功后，清理对应的 pending 文件。
    // 删除失败不是致命错误（竞争/权限），仅打 warning，不向上返回错误。
    // Unix 上 unlink 不受 fd-lock 影响，可安全删除。
    // 关联：SPEC-001 §4.3（清理机制）。
    let pending_path = crate::paths::pending_dir(base).join(format!("{}.json", resp.request_id));
    if let Err(e) = std::fs::remove_file(&pending_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "sieve-ipc: warning: failed to remove pending file {}: {e}",
                pending_path.display()
            );
        }
    }

    Ok(dec_path)
}

/// 轮询等待 `<base>/decisions/<request_id>.json` 出现并读取。
///
/// 轮询间隔 50 ms，对 30–120 s 的用户响应超时来说 CPU 开销可忽略。
/// 选择轮询而非 inotify/notify 是为了跨平台简单性；Phase 1 仅 macOS，
/// 但未来 Linux 支持时轮询同样生效，不需要额外适配。
///
/// 超时后按 `default_on_timeout` 构造兜底响应。关联：ADR-013 §4.2。
pub async fn wait_for_decision(
    request_id: Uuid,
    base: &Path,
    timeout: Duration,
    default_on_timeout: crate::protocol::DefaultOnTimeout,
) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let deadline = tokio::time::Instant::now() + timeout;
    let poll_interval = Duration::from_millis(50);

    loop {
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let resp: DecisionResponse = serde_json::from_str(&content)?;
            return Ok(resp);
        }

        if tokio::time::Instant::now() >= deadline {
            // 超时：按 default_on_timeout 构造兜底响应。
            let action = match default_on_timeout {
                crate::protocol::DefaultOnTimeout::Block => DecisionAction::Deny,
                crate::protocol::DefaultOnTimeout::Allow => DecisionAction::Allow,
                crate::protocol::DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
            };
            return Ok(DecisionResponse {
                request_id,
                decision: action,
                decided_at: Utc::now(),
                by_user: false,
                remember: false,
            });
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// 同步版读取决策文件（hook 侧使用，不依赖 tokio）。
pub fn read_decision(request_id: Uuid, base: &Path) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            // 不存在时通过 IpcError::PendingNotFound 复用（语义相近）
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let resp: DecisionResponse = serde_json::from_str(&content)?;
    Ok(resp)
}
use std::path::PathBuf;

use crate::error::IpcError;

/// 计算 sieve home 目录。
///
/// 优先级：`$SIEVE_HOME` 环境变量 > `$HOME/.sieve`。
/// $HOME 缺失时返回 [`IpcError::HomeNotFound`]。
///
/// 关联：SPEC-001 §2.1（目录结构）。
pub fn sieve_home() -> Result<PathBuf, IpcError> {
    if let Ok(val) = std::env::var("SIEVE_HOME") {
        return Ok(PathBuf::from(val));
    }
    let home = std::env::var("HOME").map_err(|_| IpcError::HomeNotFound)?;
    Ok(PathBuf::from(home).join(".sieve"))
}

/// `<sieve_home>/pending/` 目录，存放主代理写入的待决策文件。
pub fn pending_dir(base: &std::path::Path) -> PathBuf {
    base.join("pending")
}

/// `<sieve_home>/decisions/` 目录，存放 hook/GUI 写入的决策文件。
pub fn decisions_dir(base: &std::path::Path) -> PathBuf {
    base.join("decisions")
}

/// `<sieve_home>/locks/` 目录，存放文件锁占位符。
pub fn locks_dir(base: &std::path::Path) -> PathBuf {
    base.join("locks")
}

/// `<sieve_home>/ipc.sock` Unix socket 路径（主代理监听，GUI 连接）。
pub fn ipc_socket_path(base: &std::path::Path) -> PathBuf {
    base.join("ipc.sock")
}

/// 确保所有子目录存在，不存在时递归创建。
///
/// 幂等——多次调用安全。
pub fn ensure_dirs(base: &std::path::Path) -> Result<(), IpcError> {
    for dir in [pending_dir(base), decisions_dir(base), locks_dir(base)] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-core/src/pipeline/inbound_hold.rs && sed -n '1,220p' crates/sieve-core/src/pipeline/inbound_hook.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

    fn make_ipc_server() -> (Arc<IpcServer>, tokio::net::UnixListener, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        // 把 tmp 路径 leak 到测试生命周期（tempfile 会在 drop 时清理，但 socket 不影响测试）
        std::mem::forget(tmp);
        let path = socket_path.clone();
        IpcServer::bind(socket_path)
            .map(|(s, l)| (Arc::new(s), l, path))
            .unwrap()
    }

    // ── Mock IPC 返回 Allow ───────────────────────────────────────────────────

    #[tokio::test]
    async fn ipc_allow_returns_allow_outcome() {
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 模拟 GUI 客户端连接：使 gui_writer 有值，让 request_decision 注册 oneshot
        // 而不是在步骤 1 因无 GUI 连接而立即 fallback（修 #2 相关：inject_decision 需先有注册）。
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 5, DefaultOnTimeout::Block);

        // 50ms 后注入 Allow 决策（此时 pending map 里已有 oneshot sender）
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
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 模拟 GUI 客户端连接（同 Allow 测试，确保 inject_decision 能工作）
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
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
        let (server, listener, _socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
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

exec
/bin/zsh -lc "sed -n '930,1060p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-rules/src/loader.rs && git diff -- crates/sieve-rules/src/engine/mod.rs crates/sieve-rules/benches/scan_bench.rs crates/sieve-rules/tests/inbound_rules.rs crates/sieve-rules/tests/outbound_rules.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
//! 从 toml 加载出站规则集。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct OutboundRulesFile {
    rules: Vec<RuleEntry>,
}

/// 从 toml 文件加载出站规则。
pub fn load_outbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| SieveRulesError::Manifest(format!("read {}: {e}", path.display())))?;
    let f: OutboundRulesFile = toml::from_str(&s)
        .map_err(|e| SieveRulesError::Manifest(format!("parse {}: {e}", path.display())))?;
    Ok(f.rules)
}

/// 加载入站规则集（toml schema 与出站一致）。
pub fn load_inbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    load_outbound_rules(path) // schema 同，直接复用
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_minimal_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-TEST"
description = "test rule"
pattern = "hello"
severity = "critical"
action = "block"
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "OUT-TEST");
    }

    #[test]
    fn loads_rule_with_optional_fields() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-02"
description = "with optional fields"
pattern = "secret"
severity = "high"
action = "warn"
entropy_min = 3.5
keywords = ["secret", "key"]
allowlist_regexes = ["(?i)example"]
allowlist_stopwords = ["test"]
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].entropy_min, Some(3.5));
        assert_eq!(rules[0].keywords.len(), 2);
        assert_eq!(rules[0].allowlist_regexes.len(), 1);
        assert_eq!(rules[0].allowlist_stopwords.len(), 1);
    }

    #[test]
    fn returns_error_on_missing_file() {
        let result = load_outbound_rules(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_on_invalid_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "not valid toml [[[").unwrap();
        let result = load_outbound_rules(f.path());
        assert!(result.is_err());
    }
}
diff --git a/crates/sieve-rules/benches/scan_bench.rs b/crates/sieve-rules/benches/scan_bench.rs
index 5d49c48..980f877 100644
--- a/crates/sieve-rules/benches/scan_bench.rs
+++ b/crates/sieve-rules/benches/scan_bench.rs
@@ -1,6 +1,6 @@
 use criterion::{criterion_group, criterion_main, Criterion, Throughput};
 use sieve_rules::engine::{MatchEngine, VectorscanEngine};
-use sieve_rules::manifest::{Action, RuleEntry, Severity};
+use sieve_rules::manifest::{Action, DefaultOnTimeout, RuleEntry, Severity};
 
 fn build_test_engine() -> VectorscanEngine {
     let rules = vec![RuleEntry {
@@ -13,6 +13,9 @@ fn build_test_engine() -> VectorscanEngine {
         keywords: vec!["sk-ant-api03".to_string()],
         allowlist_regexes: vec![],
         allowlist_stopwords: vec![],
+        disposition: None,
+        timeout_seconds: None,
+        default_on_timeout: DefaultOnTimeout::Block,
     }];
     VectorscanEngine::compile(rules).unwrap()
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

exec
/bin/zsh -lc "git diff -- crates/sieve-rules/tests/outbound_rules.rs | sed -n '80,180p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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
/bin/zsh -lc 'CARGO_TARGET_DIR=/tmp/sieve-target cargo check -p sieve-core --no-default-features' in /Users/doskey/src/sieve
 succeeded in 3583ms:
   Compiling serde_core v1.0.228
    Checking cfg-if v1.0.4
    Checking typenum v1.20.0
   Compiling serde v1.0.228
   Compiling num-traits v0.2.19
    Checking libc v0.2.186
    Checking core-foundation-sys v0.8.7
    Checking once_cell v1.21.4
    Checking pin-project-lite v0.2.17
    Checking bitflags v2.11.1
    Checking zmij v1.0.21
    Checking itoa v1.0.18
    Checking bytes v1.11.1
    Checking memchr v2.8.0
    Checking tracing-core v0.1.36
    Checking thiserror v2.0.18
    Checking strsim v0.11.1
    Checking iana-time-zone v0.1.65
    Checking tracing v0.1.44
    Checking errno v0.3.14
    Checking mio v1.2.0
    Checking getrandom v0.4.2
    Checking socket2 v0.6.3
    Checking cpufeatures v0.2.17
    Checking generic-array v0.14.7
    Checking rustix v1.1.4
    Checking tokio v1.52.1
    Checking block-buffer v0.10.4
    Checking crypto-common v0.1.7
    Checking digest v0.10.7
    Checking sha2 v0.10.9
    Checking fd-lock v4.0.4
    Checking uuid v1.23.1
    Checking serde_json v1.0.149
    Checking chrono v0.4.44
    Checking sieve-ipc v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-ipc)
    Checking sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.50s

exec
/bin/zsh -lc "nl -ba crates/sieve-ipc/src/socket_server.rs | sed -n '120,190p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '120,190p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '190,230p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc "python - <<'PY'
s='sk-ant-api03-canary-test-aaaabbbbccccdddd-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX_AA'
print(len(s), len(s.removeprefix('sk-ant-api03-')[:-2]))
PY" in /Users/doskey/src/sieve
 succeeded in 181ms:
111 96

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '50,80p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
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

exec
/bin/zsh -lc 'cat crates/sieve-ipc/src/error.rs && cat crates/sieve-ipc/src/lib.rs && cat crates/sieve-hook/src/error.rs && cat crates/sieve-hook/src/lib.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
use thiserror::Error;

/// IPC 层错误枚举。
///
/// 关联规格：ADR-013（IPC 协议）、SPEC-001（sieve-hook 文件协议）。
#[derive(Debug, Error)]
pub enum IpcError {
    /// Unix socket 绑定或连接失败。
    #[error("socket error: {0}")]
    Socket(#[from] std::io::Error),

    /// JSON 序列化 / 反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 请求在规定超时内未收到决策响应。
    #[error("decision timeout for request {request_id}")]
    Timeout { request_id: uuid::Uuid },

    /// pending 文件已超过 stale 阈值（10 分钟），视为过期拒绝。
    ///
    /// fail-closed：过期请求不允许放行，防止残留文件被重放。
    #[error("pending file is stale (created_at too old) for request {request_id}")]
    StalePending { request_id: uuid::Uuid },

    /// pending 文件不存在——此请求未经代理标记，可 fail-open。
    #[error("pending file not found for request {request_id}")]
    PendingNotFound { request_id: uuid::Uuid },

    /// 文件加锁失败。
    #[error("file lock error: {0}")]
    FileLock(String),

    /// $HOME 环境变量缺失，无法确定 sieve_home 路径。
    #[error("$HOME environment variable is not set")]
    HomeNotFound,

    /// JSON-RPC 响应中携带了错误对象。
    #[error("json-rpc error {code}: {message}")]
    JsonRpcError { code: i64, message: String },

    /// 对端发送了无法识别的 JSON-RPC method 或响应格式异常。
    #[error("unexpected json-rpc response: {0}")]
    UnexpectedResponse(String),
}
// sieve-ipc: JSON-RPC 2.0 over Unix socket + pending/decision 文件协议库。
//
// 供 sieve-cli（主代理）调用，向 GUI（sieve-gui-macos）或 hook（sieve-hook）
// 传递决策请求并等待响应。关联：ADR-013（IPC 协议）、ADR-014（双层防御）。

pub mod decision_file;
pub mod error;
pub mod paths;
pub mod pending_file;
pub mod protocol;
pub mod socket_client;
pub mod socket_server;

// 常用类型直接 re-export，调用方无需深层 import。
pub use error::IpcError;
pub use protocol::{
    DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout, DetectionPayload,
    Disposition, Severity,
};
pub use socket_server::IpcServer;

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::protocol::*;

    // ── 协议 round-trip ──────────────────────────────────────────────────────

    #[test]
    fn decision_request_round_trip() {
        let req = DecisionRequest {
            request_id: Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::HookTerminal,
                title: "私钥检测".to_owned(),
                one_line_summary: "检测到 BIP39 助记词（12 词，checksum 通过）".to_owned(),
                details: serde_json::json!({ "word_count": 12 }),
            }],
        };

        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: DecisionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, req.request_id);
        assert_eq!(decoded.detections[0].rule_id, "IN-CR-01");
        assert_eq!(decoded.default_on_timeout, DefaultOnTimeout::Block);
    }

    #[test]
    fn decision_response_round_trip() {
        let resp = DecisionResponse {
            request_id: Uuid::now_v7(),
            decision: DecisionAction::Deny,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
        };

        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: DecisionResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, resp.request_id);
        assert_eq!(decoded.decision, DecisionAction::Deny);
        assert!(decoded.by_user);
        assert!(!decoded.remember);
    }

    #[test]
    fn disposition_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Disposition::GuiPopup).unwrap(),
            "\"gui_popup\""
        );
        assert_eq!(
            serde_json::to_string(&Disposition::HookTerminal).unwrap(),
            "\"hook_terminal\""
        );
    }

    #[test]
    fn severity_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Severity::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn decision_action_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&DecisionAction::RedactAndAllow).unwrap(),
            "\"redact_and_allow\""
        );
    }

    // ── jsonrpc envelope ────────────────────────────────────────────────────

    #[test]
    fn jsonrpc_request_omits_null_id() {
        let req = jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: "ping".to_owned(),
            params: None,
            id: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // 通知请求不携带 id 字段。
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn jsonrpc_call_includes_id() {
        let req = jsonrpc::Request::call(
            "request_decision",
            serde_json::json!({}),
            serde_json::Value::String("abc".to_owned()),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"request_decision\""));
    }
}

#[cfg(test)]
mod file_tests {
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    use super::{
        decision_file::{wait_for_decision, write_decision},
        pending_file::{read_pending, write_pending},
        protocol::*,
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
        }
    }

    // ── pending_file ─────────────────────────────────────────────────────────

    #[test]
    fn pending_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_request(id);

        let path = write_pending(&req, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = read_pending(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    #[test]
    fn pending_not_found_error() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let err = read_pending(id, tmp.path()).unwrap_err();
        assert!(matches!(err, crate::IpcError::PendingNotFound { .. }));
    }

    #[test]
    fn pending_file_lock_two_tasks() {
        // 两个线程抢同一个 pending 文件——后者等前者释放锁后写入。
        // 验证不出现数据损坏（最终文件可被正确解析）。
        use std::sync::Arc;
        use std::sync::Barrier;
        use std::thread;

        let tmp = tempfile::tempdir().unwrap();
        let base = Arc::new(tmp.path().to_owned());
        let id = Uuid::now_v7();
        let barrier = Arc::new(Barrier::new(2));

        let base1 = Arc::clone(&base);
        let barrier1 = Arc::clone(&barrier);
        let t1 = thread::spawn(move || {
            barrier1.wait();
            let req = make_request(id);
            write_pending(&req, &base1).unwrap();
        });

        let base2 = Arc::clone(&base);
        let barrier2 = Arc::clone(&barrier);
        let t2 = thread::spawn(move || {
            barrier2.wait();
            let req = make_request(id);
            write_pending(&req, &base2).unwrap();
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // 文件仍可被正确解析（两次写入串行化）。
        let read_back = read_pending(id, &base).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    // ── decision_file ────────────────────────────────────────────────────────

    #[test]
    fn decision_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let resp = DecisionResponse {
            request_id: id,
            decision: DecisionAction::Allow,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
        };

        let path = write_decision(&resp, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = super::decision_file::read_decision(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
        assert_eq!(read_back.decision, DecisionAction::Allow);
    }

    #[tokio::test]
    async fn wait_for_decision_timeout_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 极短超时，不写决策文件，应返回 Block（default_on_timeout = Block）。
        let resp = wait_for_decision(
            id,
            tmp.path(),
            Duration::from_millis(100),
            DefaultOnTimeout::Block,
        )
        .await
        .unwrap();
        assert_eq!(resp.decision, DecisionAction::Deny);
        assert!(!resp.by_user);
    }

    #[tokio::test]
    async fn wait_for_decision_found() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let base = tmp.path().to_owned();

        // 50ms 后写决策文件，模拟用户操作。
        let base_clone = base.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let resp = DecisionResponse {
                request_id: id,
                decision: DecisionAction::Allow,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
            };
            write_decision(&resp, &base_clone).unwrap();
        });

        let result = wait_for_decision(id, &base, Duration::from_secs(2), DefaultOnTimeout::Block)
            .await
            .unwrap();
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(result.by_user);
    }
}

#[cfg(test)]
mod socket_tests {
    //! 验证双向 JSON-RPC over Unix socket 通信模型（ADR-013 §3）。
    //!
    //! 测试用 IpcClient::auto_respond / 手动 socket 连接模拟真实 GUI 客户端行为，
    //! 不再使用旧的 inject_decision 绕过 socket 层。
    use std::sync::Arc;
    use std::time::Duration;

    use chrono::Utc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    use uuid::Uuid;

    use super::{
        protocol::{jsonrpc, *},
        socket_client::IpcClient,
        socket_server::IpcServer,
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
        }
    }

    /// 辅助：启动服务端并返回 Arc<IpcServer>。
    async fn start_server(socket_path: &std::path::Path) -> Arc<IpcServer> {
        let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
        let server = Arc::new(server);
        let s = Arc::clone(&server);
        tokio::spawn(async move { s.run(listener).await });
        // 等服务端就绪。
        tokio::time::sleep(Duration::from_millis(10)).await;
        server
    }

    // ── 测试 1：GUI 连接 → request_decision → GUI 收到 → 回 decision → 主代理拿到 ──

    /// 核心 happy path：双向通信全链路。
    ///
    /// 1. 模拟 GUI 客户端连接并保持长连接。
    /// 2. 主代理调 `request_decision`。
    /// 3. GUI mock 读到 request 后写回 Allow。
    /// 4. 主代理收到 Allow。
    #[tokio::test]
    async fn gui_connect_request_decision_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let id = Uuid::now_v7();

        // 模拟 GUI：连接 socket，读一条 request，写回 Allow。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
                .await
                .expect("auto_respond failed");
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        let req = make_request(id);
        let result = server
            .request_decision(req, Duration::from_secs(3))
            .await
            .unwrap();

        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(result.by_user, "GUI 回复的决策应标记 by_user=true");
    }

    // ── 测试 2：没有 GUI 客户端 → 立即 fallback ──

    /// 没有任何 GUI 连接时，request_decision 必须立即返回 fallback，
    /// 不应等待整个 timeout 时长（性能 + 体验要求）。
    #[tokio::test]
    async fn no_gui_connected_immediate_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![],
        };

        let start = std::time::Instant::now();
        let result = server
            .request_decision(req, Duration::from_secs(5))
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // 没有 GUI，应立即返回（远小于 5s 超时）。
        assert!(
            elapsed < Duration::from_millis(500),
            "no-GUI path should return immediately, got {elapsed:?}"
        );
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(!result.by_user);
    }

    // ── 测试 3：GUI 连接后断线 → pending requests 立即 fallback ──

    /// GUI 建立长连接后意外断线，主代理正在等待的 pending request 应立即 fallback，
    /// 不应等满 timeout。
    #[tokio::test]
    async fn gui_disconnect_triggers_pending_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // 模拟 GUI：连接后保持一小段时间再断线（不回复任何决策）。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            let stream = UnixStream::connect(&path_clone).await.unwrap();
            // 保持 50ms 后 drop（模拟 GUI 崩溃）。
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(stream);
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(20)).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
        };

        let start = std::time::Instant::now();
        // 给很长的 timeout，期望断线后快速 fallback。
        let result = server
            .request_decision(req, Duration::from_secs(10))
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // GUI 断线后 pending oneshot 被 drop，应远早于 10s 超时返回。
        assert!(
            elapsed < Duration::from_secs(3),
            "should fallback quickly after GUI disconnect, got {elapsed:?}"
        );
        assert_eq!(result.decision, DecisionAction::Deny, "Block → Deny");
        assert!(!result.by_user);
    }

    // ── 测试 4：多并发 request_decision，GUI 顺序回复，每个正确路由 ──

    /// 同时发起 3 个 request_decision，GUI mock 逐一回复，验证 request_id 路由正确。
    #[tokio::test]
    async fn concurrent_requests_correctly_routed() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::now_v7()).collect();

        // 模拟 GUI：长连接，读 3 条 request，全部回复 Deny。
        let path_clone = socket_path.clone();
        let ids_clone = ids.clone();
        tokio::spawn(async move {
            // 重试连接直到服务端就绪。
            let stream = {
                let mut last_err = None;
                let mut s = None;
                for _ in 0..10 {
                    match UnixStream::connect(&path_clone).await {
                        Ok(st) => {
                            s = Some(st);
                            break;
                        }
                        Err(e) => {
                            last_err = Some(e);
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                }
                s.unwrap_or_else(|| panic!("connect failed: {:?}", last_err))
            };
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();

            // 收到多少条就回多少条。
            let mut replied = 0usize;
            while replied < ids_clone.len() {
                let Some(line) = lines.next_line().await.unwrap() else {
                    break;
                };
                if line.trim().is_empty() {
                    continue;
                }
                // 解析 request_id，原样回 Deny。
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(rid_str) =
                        val.pointer("/params/request_id").and_then(|v| v.as_str())
                    {
                        if let Ok(rid) = rid_str.parse::<Uuid>() {
                            let resp = DecisionResponse {
                                request_id: rid,
                                decision: DecisionAction::Deny,
                                decided_at: Utc::now(),
                                by_user: true,
                                remember: false,
                            };
                            let rpc_resp = jsonrpc::Response {
                                jsonrpc: "2.0".to_owned(),
                                result: Some(serde_json::to_value(&resp).unwrap()),
                                error: None,
                                id: serde_json::Value::String(rid.to_string()),
                            };
                            let mut payload = serde_json::to_string(&rpc_resp).unwrap();
                            payload.push('\n');
                            write_half.write_all(payload.as_bytes()).await.unwrap();
                            replied += 1;
                        }
                    }
                }
            }
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        // 并发发起 3 个 request_decision。
        let server = Arc::clone(&server);
        let mut handles = vec![];
        for &id in &ids {
            let s = Arc::clone(&server);
            let req = make_request(id);
            handles.push(tokio::spawn(async move {
                s.request_decision(req, Duration::from_secs(5)).await
            }));
        }

        // 收集结果，全部应为 Deny（by_user=true）。
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert_eq!(result.decision, DecisionAction::Deny);
            assert!(result.by_user);
        }
    }

    // ── 测试 5：GUI 启动晚于主代理 → 连上后正常工作 ──

    /// 主代理先启动，GUI 延迟后才连接；
    /// 第一次调用（GUI 未连）立即 fallback；
    /// GUI 连上后的第二次调用能正常路由到 GUI 并拿到 by_user=true 的响应。
    ///
    /// 这验证了"启动顺序无关"的核心契约：主代理不假设 GUI 先起，
    /// GUI 不假设自己必须最后起。
    #[tokio::test]
    async fn gui_connects_late_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // ── 阶段一：GUI 未连，request_decision 立即 fallback ──
        let id_before = Uuid::now_v7();
        let req_before = make_request(id_before);
        let before = server
            .request_decision(req_before, Duration::from_secs(5))
            .await
            .unwrap();
        // 没有 GUI，立即 fallback（by_user=false）。
        assert!(!before.by_user, "GUI 未连时应立即 fallback");

        // ── 阶段二：GUI 连上，request_decision 路由到真实 GUI ──
        let id_after = Uuid::now_v7();
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            // GUI 延迟 100ms 启动（模拟真实延迟）。
            tokio::time::sleep(Duration::from_millis(100)).await;
            IpcClient::auto_respond(path_clone, id_after, DecisionAction::Deny)
                .await
                .expect("auto_respond failed");
        });

        // 等 GUI 建立连接。
        tokio::time::sleep(Duration::from_millis(150)).await;

        let req_after = make_request(id_after);
        let after = server
            .request_decision(req_after, Duration::from_secs(3))
            .await
            .unwrap();
        // GUI 已连接，回复了 Deny，by_user=true。
        assert_eq!(after.decision, DecisionAction::Deny);
        assert!(after.by_user, "GUI 连接后的请求应由 GUI 响应");
    }

    // ── 保留：timeout fallback 验证 ──

    /// 有 GUI 连接但 GUI 不回复——超时后应返回 default_on_timeout fallback。
    #[tokio::test]
    async fn socket_server_timeout_with_connected_gui() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // 模拟 GUI：连接但什么都不回复（只建立连接）。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            let _stream = UnixStream::connect(&path_clone).await.unwrap();
            // 保持连接，不发任何数据，等测试结束。
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(20)).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 1,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![],
        };

        // GUI 连着但不回复，100ms 超时后应返回 Allow（default_on_timeout）。
        let result = server
            .request_decision(req, Duration::from_millis(100))
            .await
            .unwrap();
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(!result.by_user);
    }
}

#[cfg(test)]
mod paths_tests {
    use super::paths::*;
    use std::sync::Mutex;

    // 任何修改 SIEVE_HOME / HOME 的测试都必须先拿到这把锁。
    // Rust test 默认多线程跑同一个 test binary，env var 是进程全局状态。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn sieve_home_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let orig = std::env::var("SIEVE_HOME").ok();
        std::env::set_var("SIEVE_HOME", "/tmp/test_sieve_override");
        let home = sieve_home().unwrap();
        match orig {
            Some(v) => std::env::set_var("SIEVE_HOME", v),
            None => std::env::remove_var("SIEVE_HOME"),
        }
        assert_eq!(home.to_str().unwrap(), "/tmp/test_sieve_override");
    }

    #[test]
    fn sieve_home_default_uses_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let orig = std::env::var("SIEVE_HOME").ok();
        std::env::remove_var("SIEVE_HOME");
        let home = sieve_home().unwrap();
        if let Some(v) = orig {
            std::env::set_var("SIEVE_HOME", v);
        }
        assert!(home.to_str().unwrap().ends_with(".sieve"));
    }

    #[test]
    fn ensure_dirs_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_dirs(tmp.path()).unwrap();
        ensure_dirs(tmp.path()).unwrap(); // 第二次调用不应报错。
        assert!(pending_dir(tmp.path()).exists());
        assert!(decisions_dir(tmp.path()).exists());
        assert!(locks_dir(tmp.path()).exists());
    }
}
/// pending 文件读取阶段的错误。
///
/// 独立定义（不依赖 sieve-ipc）以保持 sieve-hook 零重依赖目标。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub enum PendingError {
    /// pending 文件不存在——Sieve 代理未标记此请求，可 fail-open。
    NotFound,
    /// pending 文件存在但 created_at > stale 阈值，fail-closed。
    Stale,
    /// JSON 解析失败，fail-closed。
    ParseError(String),
    /// 其他 IO 错误。
    IoError(String),
}
// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
// main.rs 通过 use sieve_hook_lib::* 复用这些定义。

pub mod decision;
pub mod error;
pub mod pending;
pub mod protocol;

use std::path::Path;
use uuid::Uuid;

use decision::{write_decision, DecisionOutcome};
use error::PendingError;
use pending::{read_pending_checked, scan_pending_dir};

const STALE_THRESHOLD_SECS: i64 = 600;

/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
///
/// pending 文件不存在 → exit 0（fail-open）
/// pending 文件存在但已过期 → exit 1（fail-closed）
/// JSON 解析失败 → exit 1（fail-closed）
/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Err(PendingError::NotFound) => 0,
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            1
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            1
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            1
        }
        Ok(req) => {
            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
            let outcome = match req.default_on_timeout {
                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                _ => DecisionOutcome::Deny,
            };
            if let Err(e) = write_decision(request_id, &outcome, base) {
                eprintln!("sieve-hook: failed to write decision: {e}");
            }
            match outcome {
                DecisionOutcome::Allow => 0,
                DecisionOutcome::Deny => 1,
            }
        }
    }
}

/// 启发式运行逻辑：无 request_id 时扫目录。
///
/// 优先级 3（SPEC-001 §4.3）：
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
///   多 pending 时用户一次决策广播给所有 request_id。
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
pub fn run_check_heuristic(base: &Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    // （非 TTY 路径：直接按策略决定，不弹提示。）
    let outcome = decide_outcome_for_requests(&scan.fresh);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
    for req in reqs {
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            _ => return DecisionOutcome::Deny,
        }
    }
    DecisionOutcome::Allow
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::path::Path;
    use uuid::Uuid;

    use crate::decision::{self, DecisionOutcome};
    use crate::pending;
    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};

    fn write_pending_json(base: &Path, req: &DecisionRequest) {
        let dir = base.join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        let json = serde_json::to_string_pretty(req).unwrap();
        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
    }

    fn make_req(
        id: Uuid,
        dot: DefaultOnTimeout,
        created_at: chrono::DateTime<Utc>,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at,
            timeout_seconds: 30,
            default_on_timeout: dot,
            detections: vec![],
        }
    }

    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────

    #[test]
    fn pending_not_found_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
    }

    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────

    #[test]
    fn pending_stale_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
    }

    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────

    #[test]
    fn pending_parse_error_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let dir = tmp.path().join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        // 写入非法 JSON。
        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
    }

    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────

    #[test]
    fn pending_allow_on_timeout_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
    }

    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────

    #[test]
    fn pending_block_on_timeout_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "default_on_timeout=Block should return exit 1");
    }

    // ── Critical detection 记录的 decision.remember 永远 false ─────────────

    #[test]
    fn critical_decision_remember_is_false() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: "critical".to_owned(),
                disposition: "hook_terminal".to_owned(),
                title: "Test".to_owned(),
                one_line_summary: "test".to_owned(),
                details: serde_json::Value::Null,
            }],
        };
        write_pending_json(tmp.path(), &req);
        super::run_check(id, tmp.path());

        // 读取写入的 decision 文件，验证 remember=false。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        let content = std::fs::read_to_string(dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["remember"], serde_json::Value::Bool(false));
    }

    // ════════════════════════════════════════════════════════════════════════
    // 启发式匹配路径（run_check_heuristic）的 7 个新测试
    // ════════════════════════════════════════════════════════════════════════

    // 测试 1：零 pending 文件 → exit 0（fail-open）
    #[test]
    fn heuristic_zero_pending_fail_open() {
        let tmp = tempfile::tempdir().unwrap();
        // pending 目录不存在，模拟全新安装。
        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "zero pending should fail-open (exit 0)");
    }

    // 测试 2：单 pending 文件 + default_on_timeout=Allow → exit 0
    #[test]
    fn heuristic_single_pending_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "single Allow pending should return exit 0");

        // 验证 decision 文件已写入。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        assert!(dec_path.exists(), "decision file should be written");
        let content = std::fs::read_to_string(&dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["decision"], "allow");
    }

    // 测试 3：单 pending 文件 + default_on_timeout=Block → exit 1
    #[test]
    fn heuristic_single_pending_block() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 1, "single Block pending should return exit 1");

        // 验证 decision 文件已写入且 decision=deny。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        let content = std::fs::read_to_string(&dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["decision"], "deny");
    }

    // 测试 4：多 pending 文件 → 所有 decision 文件写入，最严策略生效
    #[test]
    fn heuristic_multi_pending_all_decisions_written() {
        let tmp = tempfile::tempdir().unwrap();
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        // id1 = Allow，id2 = Block → 合并后应 Deny。
        write_pending_json(
            tmp.path(),
            &make_req(id1, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id2, DefaultOnTimeout::Block, Utc::now()),
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 1, "mixed pending: Block wins, should return exit 1");

        // 两个 request_id 都应写入 decision 文件。
        for id in [id1, id2] {
            let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
            assert!(dec_path.exists(), "decision for {id} should be written");
            let content = std::fs::read_to_string(&dec_path).unwrap();
            let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(resp["decision"], "deny", "all decisions should be deny");
        }
    }

    // 测试 5：stale pending 文件 → 删除 stale + exit 0（fail-open）
    #[test]
    fn heuristic_stale_pending_deleted_and_fail_open() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Block, stale_time);
        write_pending_json(tmp.path(), &req);

        let pending_file = tmp.path().join("pending").join(format!("{id}.json"));
        assert!(
            pending_file.exists(),
            "stale pending file should exist before run"
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "stale-only pending should fail-open (exit 0)");
        // stale 文件应被删除。
        assert!(
            !pending_file.exists(),
            "stale pending file should be deleted"
        );
    }

    // 测试 6：SIEVE_REQUEST_ID 优先级 — env 设了就走 run_check 路径，不扫目录
    #[test]
    fn env_request_id_takes_priority_over_heuristic() {
        let tmp = tempfile::tempdir().unwrap();
        // 只有 id_env 对应的 pending 文件，另写一个 id_other（不应命中）。
        let id_env = Uuid::now_v7();
        let id_other = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id_env, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id_other, DefaultOnTimeout::Block, Utc::now()),
        );

        // 直接调 run_check（模拟 env 优先级路径）：只查 id_env，应 Allow。
        let code = super::run_check(id_env, tmp.path());
        assert_eq!(
            code, 0,
            "run_check with explicit id should only check that id"
        );

        // id_other 没有对应 decision 文件（未被启发式路径处理）。
        let dec_other = tmp
            .path()
            .join("decisions")
            .join(format!("{id_other}.json"));
        assert!(
            !dec_other.exists(),
            "heuristic should not run when explicit id is provided"
        );
    }

    // 测试 7：多 pending 全 Allow → exit 0
    #[test]
    fn heuristic_multi_pending_all_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id1, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id2, DefaultOnTimeout::Allow, Utc::now()),
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "all-Allow multi pending should return exit 0");

        for id in [id1, id2] {
            let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
            let content = std::fs::read_to_string(&dec_path).unwrap();
            let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(resp["decision"], "allow");
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // P2-#4 修复：scan 跳过已决策 + write_decision 删 pending 的 5 个新测试
    // ════════════════════════════════════════════════════════════════════════

    // 测试 8：scan_pending_dir 跳过已决策的 pending（decisions/<id>.json 存在）
    #[test]
    fn scan_skips_already_decided_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 写入 pending 文件。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Block, Utc::now()),
        );
        // 模拟已写入的 decision 文件。
        let dec_dir = tmp.path().join("decisions");
        std::fs::create_dir_all(&dec_dir).unwrap();
        std::fs::write(dec_dir.join(format!("{id}.json")), b"{}").unwrap();

        let result = pending::scan_pending_dir(tmp.path(), 600);
        assert!(
            result.fresh.is_empty(),
            "scan should skip pending that has a corresponding decision file"
        );
        assert!(
            result.stale_paths.is_empty(),
            "decided pending should not appear in stale_paths either"
        );
    }

    // 测试 9：scan_pending_dir 正常返回无对应 decision 的 fresh pending
    #[test]
    fn scan_returns_undecided_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        // 无 decisions/<id>.json → 应进 fresh。

        let result = pending::scan_pending_dir(tmp.path(), 600);
        assert_eq!(
            result.fresh.len(),
            1,
            "undecided pending should appear in fresh"
        );
        assert_eq!(result.fresh[0].request_id, id);
    }

    // 测试 10：write_decision 完成后 pending/<id>.json 应被删除
    #[test]
    fn write_decision_removes_pending_file() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 先写 pending 文件。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        let pending_path = tmp.path().join("pending").join(format!("{id}.json"));
        assert!(
            pending_path.exists(),
            "pending file should exist before write_decision"
        );

        decision::write_decision(id, &DecisionOutcome::Allow, tmp.path()).unwrap();

        assert!(
            !pending_path.exists(),
            "write_decision should delete the pending file"
        );
    }

    // 测试 11：write_decision 删 pending 失败时不报错（容错）
    #[cfg(unix)]
    #[test]
    fn write_decision_tolerates_pending_delete_failure() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 写 pending 文件，然后让 pending 目录不可写（删除权限受限）。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        let pending_dir = tmp.path().join("pending");
        // 移除目录写权限，使 remove_file 失败。
        std::fs::set_permissions(&pending_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        // write_decision 本身不应因删 pending 失败而返回错误。
        let result = decision::write_decision(id, &DecisionOutcome::Allow, tmp.path());

        // 恢复权限（tempdir drop 时需要能清理）。
        std::fs::set_permissions(&pending_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert!(
            result.is_ok(),
            "write_decision should succeed even if pending file deletion fails"
        );
        // decisions 文件应已写入。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        assert!(dec_path.exists(), "decision file should still be written");
    }

    // 测试 12：完整生命周期——scan → write_decision → 再 scan → fresh=[]（无重复）
    #[test]
    fn full_lifecycle_no_repeat_popup() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );

        // 第一次 scan：应看到 fresh=[id]。
        let result1 = pending::scan_pending_dir(tmp.path(), 600);
        assert_eq!(
            result1.fresh.len(),
            1,
            "first scan should return fresh pending"
        );

        // 模拟用户决策（write_decision 写 decisions + 删 pending）。
        decision::write_decision(id, &DecisionOutcome::Allow, tmp.path()).unwrap();

        // 第二次 scan：pending 已删且 decisions 已存在 → fresh=[]。
        let result2 = pending::scan_pending_dir(tmp.path(), 600);
        assert!(
            result2.fresh.is_empty(),
            "second scan after decision should return empty fresh (no repeated popup)"
        );
        assert!(
            result2.stale_paths.is_empty(),
            "second scan should return empty stale_paths"
        );
    }
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-hook/src/lib.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
// main.rs 通过 use sieve_hook_lib::* 复用这些定义。

pub mod decision;
pub mod error;
pub mod pending;
pub mod protocol;

use std::path::Path;
use uuid::Uuid;

use decision::{write_decision, DecisionOutcome};
use error::PendingError;
use pending::{read_pending_checked, scan_pending_dir};

const STALE_THRESHOLD_SECS: i64 = 600;

/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
///
/// pending 文件不存在 → exit 0（fail-open）
/// pending 文件存在但已过期 → exit 1（fail-closed）
/// JSON 解析失败 → exit 1（fail-closed）
/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Err(PendingError::NotFound) => 0,
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            1
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            1
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            1
        }
        Ok(req) => {
            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
            let outcome = match req.default_on_timeout {
                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                _ => DecisionOutcome::Deny,
            };
            if let Err(e) = write_decision(request_id, &outcome, base) {
                eprintln!("sieve-hook: failed to write decision: {e}");
            }
            match outcome {
                DecisionOutcome::Allow => 0,
                DecisionOutcome::Deny => 1,
            }
        }
    }
}

/// 启发式运行逻辑：无 request_id 时扫目录。
///
/// 优先级 3（SPEC-001 §4.3）：
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
///   多 pending 时用户一次决策广播给所有 request_id。
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
pub fn run_check_heuristic(base: &Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    // （非 TTY 路径：直接按策略决定，不弹提示。）
    let outcome = decide_outcome_for_requests(&scan.fresh);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
    for req in reqs {
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            _ => return DecisionOutcome::Deny,
        }
    }
    DecisionOutcome::Allow
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::path::Path;
    use uuid::Uuid;

    use crate::decision::{self, DecisionOutcome};
    use crate::pending;
    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};

    fn write_pending_json(base: &Path, req: &DecisionRequest) {
        let dir = base.join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        let json = serde_json::to_string_pretty(req).unwrap();
        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
    }

    fn make_req(
        id: Uuid,
        dot: DefaultOnTimeout,
        created_at: chrono::DateTime<Utc>,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at,
            timeout_seconds: 30,
            default_on_timeout: dot,
            detections: vec![],
        }
    }

    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────

    #[test]
    fn pending_not_found_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
    }

    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────

    #[test]
    fn pending_stale_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
    }

    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────

    #[test]
    fn pending_parse_error_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let dir = tmp.path().join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        // 写入非法 JSON。
        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
    }

    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────

    #[test]
    fn pending_allow_on_timeout_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
    }

    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────

    #[test]
    fn pending_block_on_timeout_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "default_on_timeout=Block should return exit 1");
    }

    // ── Critical detection 记录的 decision.remember 永远 false ─────────────

    #[test]
    fn critical_decision_remember_is_false() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: "critical".to_owned(),
                disposition: "hook_terminal".to_owned(),
                title: "Test".to_owned(),
                one_line_summary: "test".to_owned(),
                details: serde_json::Value::Null,
            }],
        };
        write_pending_json(tmp.path(), &req);
        super::run_check(id, tmp.path());

        // 读取写入的 decision 文件，验证 remember=false。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        let content = std::fs::read_to_string(dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["remember"], serde_json::Value::Bool(false));
    }

    // ════════════════════════════════════════════════════════════════════════
    // 启发式匹配路径（run_check_heuristic）的 7 个新测试
    // ════════════════════════════════════════════════════════════════════════

    // 测试 1：零 pending 文件 → exit 0（fail-open）
    #[test]
    fn heuristic_zero_pending_fail_open() {
        let tmp = tempfile::tempdir().unwrap();
        // pending 目录不存在，模拟全新安装。
        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "zero pending should fail-open (exit 0)");
    }

    // 测试 2：单 pending 文件 + default_on_timeout=Allow → exit 0
    #[test]
    fn heuristic_single_pending_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);


exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '830,870p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   830	                hook_detections.push(d);
   831	            }
   832	            Action::HoldForDecision { .. } => {
   833	                // GUI 类：hold 流等决策
   834	                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
   835	                hold_detections.push(d);
   836	            }
   837	            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
   838	                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
   839	            }
   840	        }
   841	    }
   842	
   843	    (blocking, hook_detections, hold_detections)
   844	}
   845	
   846	/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
   847	///
   848	/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
   849	/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
   850	fn write_hook_pending_silent(d: &sieve_core::Detection) {
   851	    use chrono::Utc;
   852	
   853	    let sieve_home = match sieve_ipc::paths::sieve_home() {
   854	        Ok(h) => h,
   855	        Err(e) => {
   856	            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
   857	            return;
   858	        }
   859	    };
   860	
   861	    let request_id = uuid::Uuid::new_v4();
   862	    let ipc_req = sieve_ipc::DecisionRequest {
   863	        request_id,
   864	        created_at: Utc::now(),
   865	        timeout_seconds: 60,
   866	        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   867	        detections: vec![sieve_ipc::protocol::DetectionPayload {
   868	            rule_id: d.rule_id.clone(),
   869	            severity: map_severity_to_ipc(d.severity),
   870	            disposition: sieve_ipc::Disposition::HookTerminal,

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '870,905p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
pyenv: cannot rehash: /Users/doskey/.pyenv/shims isn't writable
   870	            disposition: sieve_ipc::Disposition::HookTerminal,
   871	            title: format!("检测命中：{}", d.rule_id),
   872	            one_line_summary: d.evidence_truncated.clone(),
   873	            details: serde_json::json!({}),
   874	        }],
   875	    };
   876	
   877	    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
   878	        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
   879	    } else {
   880	        tracing::info!(
   881	            rule = %d.rule_id,
   882	            request_id = %request_id,
   883	            "HookMark: pending file written, SSE stream continues"
   884	        );
   885	    }
   886	}
   887	
   888	/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
   889	fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
   890	    match s {
   891	        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
   892	        sieve_core::Severity::High => sieve_ipc::Severity::High,
   893	        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
   894	        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
   895	    }
   896	}
   897	
   898	/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
   899	fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
   900	    let payload = serde_json::json!({
   901	        "type": "sieve_blocked",
   902	        "blocked_at": epoch_secs_string(),
   903	        "detections": detections.iter().map(|d| serde_json::json!({
   904	            "rule_id": d.rule_id,
   905	            "severity": d.severity,

codex
补丁引入了会让 setup 后 daemon 无法启动的配置问题，并且在 Hook/GUI 决策路径上存在 fail-open 或未脱敏发送的安全问题。doctor 还会误报或以成功状态返回，无法可靠暴露这些安装故障。

Full review comments:

- [P1] 让 setup 安装或引用真实规则文件 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:508-509
  在 `sieve setup` 生成的默认配置中，`rules_path` / `inbound_rules_path` 指向 `~/.sieve/rules/*.toml`，但本次 setup 流程没有创建该目录，也没有复制内置规则文件；用户通过 launchd 启动 daemon 时会在 `load_outbound_rules` / `load_inbound_rules` 处直接失败退出，导致自动安装后的服务不可用。

- [P1] 不要在 Hook pending 写入失败时放行 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:877-878
  当 HookTerminal 规则命中但 `SIEVE_HOME`/`HOME` 不可用、目录无写权限或磁盘错误导致 pending 文件写入失败时，这里只记录 warn 并继续转发 SSE；随后 `sieve-hook check` 扫不到 pending 会 fail-open，IN-CR-02/IN-CR-04 这类危险命令就会执行，违背这些规则的 fail-closed 语义。

- [P1] 对 RedactAndAllow 实际脱敏 held 命中 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:353-356
  当 GUI 返回 `RedactAndAllow`（或后续按 OUT-06/OUT-08 的 timeout=redact 产生该决策）时，当前分支只是 fall through 到 `redact_hits`，但 `redact_hits` 只收集 `Action::Redact`，不包含这些 `HoldForDecision` 命中的 span；因此用户选择“脱敏后发送”会把 JWT/Stripe 等原文直接发给上游。

- [P2] 使用规则配置的 default_on_timeout — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:313-316
  OUT-06 和 OUT-08 在规则文件里配置了 `default_on_timeout = "redact"`，但这里把所有出站 `HoldForDecision` 都硬编码成 `Block`；GUI 未响应或未连接时会返回 Deny/426，而不是按规则脱敏后放行，导致这些高风险但可脱敏的请求被错误阻断。

- [P2] 避免 doctor canary 误报通过 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:191-192
  这里只检查响应里不含原始 canary token；如果 daemon 只是把请求透传到 Anthropic 后拿到 401/502，或者这个 canary 本身没有命中 OUT-01，响应同样不会包含该 token，doctor 会误判脱敏正常。应校验命中了本地拦截/脱敏路径，例如使用确实匹配规则的 token 并验证上游收到的是 redacted body。

- [P2] 让 doctor 失败时返回错误 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:70-76
  即使任一检查失败，`doctor::run()` 仍然返回 `Ok(())`；这会让 `sieve doctor` 在 CI/脚本里以成功退出，也让 `sieve setup` 中的 `doctor::run()?` 无法发现安装失败（例如 daemon 没启动或规则路径无效）。
2026-04-28T07:43:42.371840Z ERROR codex_core::session: failed to record rollout items: thread 019dd2ff-87a7-7c20-914e-5439fe4df6f8 not found
2026-04-28T07:43:42.410058Z ERROR codex_core::session: failed to record rollout items: thread 019dd2ff-876c-78a0-8285-fb24b0718374 not found
补丁引入了会让 setup 后 daemon 无法启动的配置问题，并且在 Hook/GUI 决策路径上存在 fail-open 或未脱敏发送的安全问题。doctor 还会误报或以成功状态返回，无法可靠暴露这些安装故障。

Full review comments:

- [P1] 让 setup 安装或引用真实规则文件 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:508-509
  在 `sieve setup` 生成的默认配置中，`rules_path` / `inbound_rules_path` 指向 `~/.sieve/rules/*.toml`，但本次 setup 流程没有创建该目录，也没有复制内置规则文件；用户通过 launchd 启动 daemon 时会在 `load_outbound_rules` / `load_inbound_rules` 处直接失败退出，导致自动安装后的服务不可用。

- [P1] 不要在 Hook pending 写入失败时放行 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:877-878
  当 HookTerminal 规则命中但 `SIEVE_HOME`/`HOME` 不可用、目录无写权限或磁盘错误导致 pending 文件写入失败时，这里只记录 warn 并继续转发 SSE；随后 `sieve-hook check` 扫不到 pending 会 fail-open，IN-CR-02/IN-CR-04 这类危险命令就会执行，违背这些规则的 fail-closed 语义。

- [P1] 对 RedactAndAllow 实际脱敏 held 命中 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:353-356
  当 GUI 返回 `RedactAndAllow`（或后续按 OUT-06/OUT-08 的 timeout=redact 产生该决策）时，当前分支只是 fall through 到 `redact_hits`，但 `redact_hits` 只收集 `Action::Redact`，不包含这些 `HoldForDecision` 命中的 span；因此用户选择“脱敏后发送”会把 JWT/Stripe 等原文直接发给上游。

- [P2] 使用规则配置的 default_on_timeout — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:313-316
  OUT-06 和 OUT-08 在规则文件里配置了 `default_on_timeout = "redact"`，但这里把所有出站 `HoldForDecision` 都硬编码成 `Block`；GUI 未响应或未连接时会返回 Deny/426，而不是按规则脱敏后放行，导致这些高风险但可脱敏的请求被错误阻断。

- [P2] 避免 doctor canary 误报通过 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:191-192
  这里只检查响应里不含原始 canary token；如果 daemon 只是把请求透传到 Anthropic 后拿到 401/502，或者这个 canary 本身没有命中 OUT-01，响应同样不会包含该 token，doctor 会误判脱敏正常。应校验命中了本地拦截/脱敏路径，例如使用确实匹配规则的 token 并验证上游收到的是 redacted body。

- [P2] 让 doctor 失败时返回错误 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:70-76
  即使任一检查失败，`doctor::run()` 仍然返回 `Ok(())`；这会让 `sieve doctor` 在 CI/脚本里以成功退出，也让 `sieve setup` 中的 `doctor::run()?` 无法发现安装失败（例如 daemon 没启动或规则路径无效）。
