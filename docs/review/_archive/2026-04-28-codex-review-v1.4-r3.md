OpenAI Codex v0.125.0 (research preview)
--------
workdir: /Users/doskey/src/sieve
model: gpt-5.5
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/doskey/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019dd204-6645-76a1-a342-04a348e9e8d0
--------
user
current changes
exec
/bin/zsh -lc 'git status --short --branch' in /Users/doskey/src/sieve
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
 M crates/sieve-cli/tests/inbound_block.rs
 M crates/sieve-cli/tests/outbound_block.rs
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

exec
/bin/zsh -lc "git diff --stat && printf '\\n--- staged ---\\n' && git diff --cached --stat" in /Users/doskey/src/sieve
 succeeded in 0ms:
 .cursorrules                                       |  21 +-
 CLAUDE.md                                          |  40 +-
 Cargo.lock                                         | Bin 56453 -> 64212 bytes
 Cargo.toml                                         |   6 +
 README.md                                          |  12 +-
 SECURITY.md                                        |  14 +-
 crates/sieve-cli/Cargo.toml                        |   6 +
 crates/sieve-cli/src/audit.rs                      | 376 ++++++++++-
 crates/sieve-cli/src/cli.rs                        |  40 ++
 crates/sieve-cli/src/config.rs                     | 155 ++++-
 crates/sieve-cli/src/daemon.rs                     | 690 +++++++++++++++++++--
 crates/sieve-cli/src/engine_adapter.rs             | 207 ++++++-
 crates/sieve-cli/src/main.rs                       |  22 +-
 crates/sieve-cli/tests/inbound_block.rs            |  75 ++-
 crates/sieve-cli/tests/outbound_block.rs           | 347 +++++++++--
 crates/sieve-core/Cargo.toml                       |   7 +
 crates/sieve-core/src/detection.rs                 |  28 +-
 crates/sieve-core/src/pipeline/inbound.rs          |   2 +-
 crates/sieve-core/src/pipeline/mod.rs              | 390 +++++++++++-
 crates/sieve-rules/benches/scan_bench.rs           |   5 +-
 crates/sieve-rules/rules/inbound.toml              |  99 +++
 crates/sieve-rules/rules/outbound.toml             |  26 +
 crates/sieve-rules/src/critical_lock.rs            | 170 ++++-
 crates/sieve-rules/src/engine/mod.rs               |   3 +
 crates/sieve-rules/src/manifest.rs                 | 219 ++++++-
 crates/sieve-rules/tests/inbound_rules.rs          | 142 +++++
 crates/sieve-rules/tests/outbound_rules.rs         |  77 +++
 docs/api/api-reference.md                          | 166 +++--
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
 docs/design/architecture.md                        | 213 ++++---
 docs/design/data-model.md                          | 121 +++-
 docs/glossary.md                                   |  50 +-
 docs/guides/deployment.md                          | 280 ++++-----
 docs/guides/development.md                         |  70 ++-
 docs/requirements/PRD-sieve.md                     |  47 +-
 docs/requirements/user-stories.md                  | 178 ++++--
 tasks/roadmap.md                                   |  77 ++-
 46 files changed, 3824 insertions(+), 761 deletions(-)

--- staged ---

exec
/bin/zsh -lc 'git diff -- crates/sieve-cli/src/audit.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs crates/sieve-cli/src/config.rs crates/sieve-cli/src/cli.rs crates/sieve-cli/src/main.rs' in /Users/doskey/src/sieve
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

exec
/bin/zsh -lc "find . -maxdepth 3 -type f | sort | sed 's#"'^./##'"' | head -200" in /Users/doskey/src/sieve
 succeeded in 0ms:
.DS_Store
.cargo/config.toml
.claude/settings.local.json
.cursorrules
.git/AUTO_MERGE
.git/COMMIT_EDITMSG
.git/FETCH_HEAD
.git/HEAD
.git/ORIG_HEAD
.git/config
.git/description
.git/hooks/applypatch-msg.sample
.git/hooks/commit-msg.sample
.git/hooks/fsmonitor-watchman.sample
.git/hooks/post-update.sample
.git/hooks/pre-applypatch.sample
.git/hooks/pre-commit.sample
.git/hooks/pre-merge-commit.sample
.git/hooks/pre-push.sample
.git/hooks/pre-rebase.sample
.git/hooks/pre-receive.sample
.git/hooks/prepare-commit-msg.sample
.git/hooks/push-to-checkout.sample
.git/hooks/sendemail-validate.sample
.git/hooks/update.sample
.git/index
.git/info/exclude
.git/logs/HEAD
.git/opencode
.gitattributes
.github/ISSUE_TEMPLATE/bug_report.md
.github/ISSUE_TEMPLATE/config.yml
.github/ISSUE_TEMPLATE/feature_request.md
.github/ISSUE_TEMPLATE/suspicious_sample.md
.github/PULL_REQUEST_TEMPLATE.md
.github/dependabot.yml
.github/workflows/ci.yml
.github/workflows/fuzz-nightly.yml
.github/workflows/release.yml
.gitignore
CLAUDE.md
Cargo.lock
Cargo.toml
LICENSE
README.md
SECURITY.md
crates/sieve-cli/Cargo.toml
crates/sieve-core/Cargo.toml
crates/sieve-hook/Cargo.toml
crates/sieve-ipc/Cargo.toml
crates/sieve-rules/Cargo.toml
deny.toml
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
docs/requirements/PRD-sieve.md
docs/requirements/user-stories.md
docs/research/deep-research-report.md
docs/review/2026-04-27-codex-week-1-3-implementation-review.md
docs/specs/SPEC-001-sieve-hook-protocol.md
docs/specs/SPEC-002-hips-popup-behavior.md
docs/specs/SPEC-003-sieve-setup-tool.md
fuzz/Cargo.toml
fuzz/fuzz_targets/inbound_filter.rs
fuzz/fuzz_targets/sse_parser.rs
fuzz/fuzz_targets/tool_use_aggregator.rs
fuzz_afl/Cargo.toml
fuzz_afl/src/inbound_filter.rs
fuzz_afl/src/sse_parser.rs
fuzz_afl/src/tool_use_aggregator.rs
librust_out.rlib
rust-toolchain.toml
scripts/smoke_test.py
target/.rustc_info.json
target/CACHEDIR.TAG
target/aarch64-apple-darwin/CACHEDIR.TAG
target/debug/.cargo-lock
target/debug/inbound_filter
target/debug/inbound_filter.d
target/debug/libsieve_core.d
target/debug/libsieve_core.rlib
target/debug/libsieve_hook_lib.d
target/debug/libsieve_hook_lib.rlib
target/debug/libsieve_ipc.d
target/debug/libsieve_ipc.rlib
target/debug/libsieve_rules.d
target/debug/libsieve_rules.rlib
target/debug/sieve
target/debug/sieve-hook
target/debug/sieve-hook.d
target/debug/sieve.d
target/debug/sse_parser
target/debug/sse_parser.d
target/debug/tool_use_aggregator
target/debug/tool_use_aggregator.d
target/release/.cargo-artifact-lock
target/release/.cargo-build-lock
target/release/.cargo-lock
target/release/libsieve_hook_lib.d
target/release/libsieve_hook_lib.rlib
target/release/sieve-hook
target/release/sieve-hook.d
target/release/sieve.d
tasks/lessons.md
tasks/roadmap.md
tasks/todo.md

exec
/bin/zsh -lc 'find crates/sieve-cli/src/commands crates/sieve-hook crates/sieve-ipc crates/sieve-core/src/pipeline -maxdepth 4 -type f | sort' in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-cli/src/commands/doctor.rs
crates/sieve-cli/src/commands/mod.rs
crates/sieve-cli/src/commands/setup.rs
crates/sieve-cli/src/commands/uninstall.rs
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

exec
/bin/zsh -lc 'cargo test --workspace --all-targets' in /Users/doskey/src/sieve
 exited 101 in 776ms:
   Compiling sieve-rules v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-rules)
   Compiling sieve-hook v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-hook)
   Compiling sieve-fuzz v0.0.0 (/Users/doskey/src/sieve/fuzz)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.67s
     Running unittests src/main.rs (target/debug/deps/sieve-eea63f8e71059d6b)

running 33 tests
test commands::setup::tests::bad_json_parse_returns_error_not_empty_object ... ok
test commands::setup::tests::setup_log_entry_created_new_serializes_correctly ... ok
test config::tests::audit_db_path_falls_back_to_default ... ok
test config::tests::audit_db_path_explicit_field_wins ... ok
test config::tests::audit_db_path_falls_back_to_log_path ... ok
test config::tests::defaults_are_sane ... ok
test config::tests::listen_addr_parses ... ok
test commands::setup::tests::default_sieve_toml_has_absolute_paths ... ok
test config::tests::parse_dry_run_and_rules_path ... ok
test config::tests::parse_minimal_toml ... ok
test config::tests::parse_full_toml ... ok
test config::tests::resolved_rules_path_explicit ... ok
test config::tests::resolved_rules_path_fallback ... ok
test config::tests::resolved_sieveignore_path_explicit ... ok
test commands::setup::tests::plist_contains_absolute_config_flag ... ok
test config::tests::unknown_field_rejected ... ok
test engine_adapter::tests::map_action_warn_becomes_hook_mark ... ok
test commands::uninstall::tests::uninstall_toml_created_new_true_deletes_file ... ok
test engine_adapter::tests::redact_evidence_long ... ok
test commands::setup::tests::default_sieve_toml_parses_as_config ... ok
test engine_adapter::tests::redact_evidence_short ... ok
test commands::uninstall::tests::uninstall_created_new_true_deletes_file ... ok
test commands::uninstall::tests::uninstall_created_new_false_removes_sieve_entries_only ... ok
test commands::uninstall::tests::uninstall_toml_created_new_false_restores_from_backup ... ok
test engine_adapter::tests::scan_no_match_returns_empty ... ok
test audit::tests::update_trigger_blocks ... ok
test audit::tests::decision_event_stores_decision_field ... ok
test audit::tests::write_and_read_events ... ok
test engine_adapter::tests::disposition_hook_terminal_beats_enforce_action ... ok
test engine_adapter::tests::disposition_auto_redact_beats_enforce_action ... ok
test engine_adapter::tests::scan_detects_pattern ... ok
test engine_adapter::tests::disposition_gui_popup_beats_enforce_action ... ok
test engine_adapter::tests::span_offset_applied ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audit_append_only.rs (target/debug/deps/audit_append_only-7963e579460992e0)

running 3 tests
test update_is_rejected_by_trigger ... ok
test delete_is_rejected_by_trigger ... ok
test write_3_events_and_read_back ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/inbound_block.rs (target/debug/deps/inbound_block-ebd9dd42f9aab019)

running 10 tests
test ucsb_attack_3_signing_tool_blocked ... FAILED
test address_substitution_from_prompt_seed_blocks ... FAILED
test in_cr_03_sensitive_path_warn_passes_through ... FAILED
test malformed_tool_use_partial_json_blocks ... FAILED
test ucsb_attack_2_dangerous_shell_hookmark_passthrough ... FAILED
test benign_response_passes_through_unchanged ... FAILED
test ucsb_attack_4_markdown_exfil_failclosed_without_gui ... FAILED
test ucsb_attack_1_address_substitution_blocked ... FAILED
test in_cr_04_persistence_shell_rc_hookmark_passthrough ... FAILED
test unterminated_final_event_still_blocks_critical ... FAILED

failures:

---- ucsb_attack_3_signing_tool_blocked stdout ----

thread 'ucsb_attack_3_signing_tool_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- address_substitution_from_prompt_seed_blocks stdout ----

thread 'address_substitution_from_prompt_seed_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_03_sensitive_path_warn_passes_through stdout ----

thread 'in_cr_03_sensitive_path_warn_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- malformed_tool_use_partial_json_blocks stdout ----

thread 'malformed_tool_use_partial_json_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_2_dangerous_shell_hookmark_passthrough stdout ----

thread 'ucsb_attack_2_dangerous_shell_hookmark_passthrough' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- benign_response_passes_through_unchanged stdout ----

thread 'benign_response_passes_through_unchanged' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
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
/bin/zsh -lc 'git diff -- crates/sieve-core/src/pipeline/mod.rs crates/sieve-core/src/pipeline/inbound.rs crates/sieve-core/src/detection.rs crates/sieve-core/Cargo.toml' in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "sed -n '1,260p' crates/sieve-core/src/pipeline/outbound_redact.rs && printf '\\n--- inbound_hook ---\\n' && sed -n '1,260p' crates/sieve-core/src/pipeline/inbound_hook.rs && printf '\\n--- inbound_hold ---\\n' && sed -n '1,320p' crates/sieve-core/src/pipeline/inbound_hold.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

--- inbound_hook ---
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

--- inbound_hold ---
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
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        // 使用 tokio::time::pause() + advance() 模拟超时（无需等 1s）
        tokio::time::pause();

        let req = make_request(id, 1, DefaultOnTimeout::Block);
        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        let ipc_clone = Arc::clone(&server);
        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });

        // 推进 2 秒让超时触发
        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::time::resume();

        let outcome = task.await.unwrap().unwrap();
        assert!(
            matches!(outcome, HoldOutcome::Deny { .. }),
            "timeout with Block should return Deny, got {outcome:?}"
        );
    }

    // ── 超时 default_on_timeout = Allow ──────────────────────────────────────

    #[tokio::test]
    async fn timeout_with_allow_returns_allow() {
        let (server, listener, _socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        tokio::time::pause();

        let id = Uuid::now_v7();
        let req = make_request(id, 1, DefaultOnTimeout::Allow);
        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        let ipc_clone = Arc::clone(&server);
        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });

        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::time::resume();

        let outcome = task.await.unwrap().unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }

    // ── keep-alive channel 收到数据 ──────────────────────────────────────────

    #[tokio::test]
    async fn keep_alive_sent_before_decision() {
        // 验证 keep-alive channel 在等待期间可接收到消息（无需真等 25s）
        // 只验证 channel 本身不阻塞 hold_and_decide 流程
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;


exec
/bin/zsh -lc "sed -n '320,700p' crates/sieve-core/src/pipeline/inbound_hold.rs && sed -n '260,620p' crates/sieve-core/src/pipeline/outbound_redact.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:

        // 模拟 GUI 客户端连接（使 inject_decision 能工作）
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 30, DefaultOnTimeout::Block);

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        // 注入 Allow 让 hold 快速结束
        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
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

        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }
}
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

    // ── redact_segments 测试（修 #1 回归）─────────────────────────────────────

    fn seg(start: usize, text: &str) -> (usize, String) {
        (start, text.to_string())
    }

    /// 单条 OUT-01 命中：segment 内正确替换，原 secret 不存在于结果中。
    #[test]
    fn segments_single_hit_secret_removed() {
        // 段 0：offset=0, text="my sk-ant-api03-secret key"
        // hit 来自 vectorscan: start=3, end=21（相对 text），累计偏移=0+3=3
        let text = "my sk-ant-api03-secret key";
        let segments = vec![seg(0, text)];
        // 累计偏移 start=3, end=21
        let hits = [hit("OUT-01", 3, 21)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 1);
        assert_eq!(r.redacted_summary, "OUT-01");
        assert_eq!(r.texts.len(), 1);
        // 替换后不含原始 secret 片段
        assert!(!r.texts[0].contains("sk-ant-api03-secret"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
    }

    /// 多条命中（不同段）：各自正确替换。
    #[test]
    fn segments_multiple_hits_different_segments() {
        // 段 0：offset=0, text="secret1 here"（命中 [0,7)）
        // 段 1：offset=12, text="clean text secret2"（命中 [12+10,12+17) = [22,29)）
        let segments = vec![seg(0, "secret1 here"), seg(12, "clean text secret2")];
        let hits = [hit("OUT-01", 0, 7), hit("OUT-02", 22, 29)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 2);
        assert!(!r.texts[0].contains("secret1"));
        assert!(!r.texts[1].contains("secret2"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
        assert!(r.texts[1].contains("[REDACTED:OUT-02]"));
    }

    /// 中文+emoji UTF-8 命中：UTF-8 边界对齐，不破坏 JSON 结构。
    #[test]
    fn segments_utf8_chinese_emoji_hit() {
        // text: "你好😀sk-ant-secret"
        // "你好😀" = 3+3+4=10 bytes，"sk-ant-secret" 从 byte 10 开始
        let text = "你好😀sk-ant-secret";
        let text_byte_len = text.len();
        let segments = vec![seg(0, text)];
        // 命中整个 sk-ant-secret 部分（byte 10..text_len）
        let hits = [hit("OUT-01", 10, text_byte_len)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 1);
        // 替换后 text 是合法 UTF-8
        assert!(std::str::from_utf8(r.texts[0].as_bytes()).is_ok());
        assert!(!r.texts[0].contains("sk-ant-secret"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
    }

    /// hit 不与任何段重叠时：原样保留所有段。
    #[test]
    fn segments_hit_outside_all_segments_no_change() {
        let segments = vec![seg(0, "hello world"), seg(20, "foo bar")];
        // hit 在 [50, 60)，不与任何段重叠
        let hits = [hit("OUT-01", 50, 60)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 0);
        assert_eq!(r.texts[0], "hello world");
        assert_eq!(r.texts[1], "foo bar");
    }
}

exec
/bin/zsh -lc 'rg "extract_text_content|body_byte_offset|scan_text" -n crates/sieve-core crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/engine_adapter.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-cli/src/engine_adapter.rs:140:    fn scan_text(
crates/sieve-cli/src/engine_adapter.rs:225:        hits.extend(self.scan_text(&tool.name, source, 0)?);
crates/sieve-cli/src/engine_adapter.rs:228:            hits.extend(self.scan_text(&input_str, source, 0)?);
crates/sieve-cli/src/engine_adapter.rs:237:    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
crates/sieve-cli/src/engine_adapter.rs:244:    fn scan_text(
crates/sieve-cli/src/engine_adapter.rs:248:        body_byte_offset: usize,
crates/sieve-cli/src/engine_adapter.rs:309:                    start: body_byte_offset + hit.start,
crates/sieve-cli/src/engine_adapter.rs:310:                    end: body_byte_offset + hit.end,
crates/sieve-cli/src/engine_adapter.rs:338:                        start: body_byte_offset,
crates/sieve-cli/src/engine_adapter.rs:339:                        end: body_byte_offset + input.len(),
crates/sieve-cli/src/engine_adapter.rs:392:            .scan_text("my secret key", ContentSource::OutboundUserText, 0)
crates/sieve-cli/src/engine_adapter.rs:411:            .scan_text(
crates/sieve-cli/src/engine_adapter.rs:453:            .scan_text("say hello", ContentSource::OutboundSystemText, 100)
crates/sieve-cli/src/engine_adapter.rs:481:            .scan_text("my sk-ant-key here", ContentSource::OutboundUserText, 0)
crates/sieve-cli/src/engine_adapter.rs:510:            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
crates/sieve-cli/src/engine_adapter.rs:538:            .scan_text(
crates/sieve-cli/src/daemon.rs:202:        let texts = anthropic_req.extract_text_content();
crates/sieve-cli/src/daemon.rs:246:        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
crates/sieve-cli/src/daemon.rs:1094:/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
crates/sieve-cli/src/daemon.rs:1116:    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
crates/sieve-cli/src/daemon.rs:1166:    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
crates/sieve-core/src/protocol/anthropic.rs:56:    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
crates/sieve-core/src/protocol/anthropic.rs:146:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:155:        let texts = req.extract_text_content();
crates/sieve-core/src/protocol/anthropic.rs:165:        let texts = req.extract_text_content();
crates/sieve-core/src/pipeline/outbound.rs:24:    /// - `body_byte_offset`：该文本在原始请求 body 中的起始字节偏移（用于生成绝对 span）。
crates/sieve-core/src/pipeline/outbound.rs:25:    fn scan_text(
crates/sieve-core/src/pipeline/outbound.rs:29:        body_byte_offset: usize,
crates/sieve-core/src/pipeline/outbound.rs:75:                let hits = self.engine.scan_text(text, source, body_offset)?;
crates/sieve-core/src/pipeline/outbound.rs:103:        fn scan_text(
crates/sieve-core/src/pipeline/outbound_redact.rs:150:/// [`Detection.span`] 的 `start`/`end` 是 `extract_text_content()` 返回的
crates/sieve-core/src/pipeline/outbound_redact.rs:151:/// **累计文本字符偏移**（即 `body_byte_offset + vectorscan_offset`），
crates/sieve-core/src/pipeline/outbound_redact.rs:160:///   顺序与 `AnthropicRequest::extract_text_content()` 返回值一致。
crates/sieve-core/src/pipeline/inbound.rs:24:    fn scan_text(
crates/sieve-core/src/pipeline/inbound.rs:116:                    .scan_text(text, ContentSource::InboundAssistantText, 0)?,
crates/sieve-core/src/pipeline/inbound.rs:178:        fn scan_text(

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-core/src/protocol/anthropic.rs && sed -n '180,360p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
        //    d. GuiPopup（Action::HoldForDecision）→ hold HTTP 长连接等 GUI 决策（R2-#1）
        //    e. 其余 → 透传

        // 4a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
        //
        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
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

        // 4d. 出站 GuiPopup（HoldForDecision）：hold HTTP 长连接等待 GUI 决策（R2-#1 修复）。
        //
        // 出站请求是非流式 HTTP：body 已 collect，无需 SSE keep-alive（入站才需要）。
        // 客户端等待期间持有普通 HTTP 长连接（reqwest / Claude Code client 的超时决定等待上限）。
        //
        // 决策映射：
        //   Allow → 原 body 转发上游
        //   RedactAndAllow → redact_hits 非空则脱敏，否则原 body 转发
        //   Deny → 426 拒绝
        //   超时 → 按 default_on_timeout（OUT-06/08 = Redact，OUT-07/09/10 = Block）
        //
        // 关联：PRD v1.4 §5.4.2 出站超时策略表、ADR-016（二维处置矩阵）。
        let hold_detections_outbound: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
            .collect();

        if !hold_detections_outbound.is_empty() {
            if let Some(ref ipc_server) = ipc {
                use chrono::Utc;

                let request_id = uuid::Uuid::new_v4();
                let (timeout_seconds, default_on_timeout) = hold_detections_outbound
                    .iter()
                    .find_map(|d| {
                        if let Action::HoldForDecision {
                            timeout_seconds, ..
                        } = d.action
                        {
                            // 取第一个 HoldForDecision detection 的规则 timeout/default
                            // default_on_timeout 从 detection 的 rule_id 对应规则读取，
                            // 此处用 Block 作为保守默认（规则未设则 fail-closed）
                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((60, sieve_ipc::DefaultOnTimeout::Block));

                let ipc_detections = hold_detections_outbound
                    .iter()
                    .map(|d| sieve_ipc::protocol::DetectionPayload {
                        rule_id: d.rule_id.clone(),
                        severity: map_severity_to_ipc(d.severity),
                        disposition: sieve_ipc::Disposition::GuiPopup,
                        title: format!("出站检测命中：{}", d.rule_id),
                        one_line_summary: d.evidence_truncated.clone(),
                        details: serde_json::json!({}),
                    })
                    .collect();

                let ipc_req = sieve_ipc::DecisionRequest {
                    request_id,
                    created_at: Utc::now(),
                    timeout_seconds,
                    default_on_timeout,
                    detections: ipc_detections,
                };

                // 出站 hold：无 SSE keep-alive，直接 await 决策
                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
                let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;

                match outcome {
                    Ok(resp) => match resp.decision {
                        sieve_ipc::DecisionAction::Allow => {
                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
                            // 继续往下，走正常转发路径
                        }
                        sieve_ipc::DecisionAction::RedactAndAllow => {
                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
                            // 若有 redact_hits 则脱敏，否则原 body 转发（与 Allow 同逻辑）
                            // 直接 fall-through 到下方 redact_hits 处理
                        }
                        sieve_ipc::DecisionAction::Deny => {
                            tracing::warn!("OUTBOUND GUI: Deny → 426");
                            let held: Vec<sieve_core::Detection> = hold_detections_outbound

exec
/bin/zsh -lc "sed -n '360,760p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
                                .iter()
                                .map(|d| (*d).clone())
                                .collect();
                            return Ok(build_426_response(&held));
                        }
                    },
                    Err(e) => {
                        // IPC 错误：按 default_on_timeout 兜底（fail-closed）
                        tracing::warn!(error = %e, "OUTBOUND GUI: IPC error, fail-closed → 426");
                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
                            .iter()
                            .map(|d| (*d).clone())
                            .collect();
                        return Ok(build_426_response(&held));
                    }
                }
            } else {
                // IPC 未初始化：fail-closed → 426
                tracing::warn!("OUTBOUND GUI: IPC not initialized, fail-closed → 426");
                let held: Vec<sieve_core::Detection> = hold_detections_outbound
                    .iter()
                    .map(|d| (*d).clone())
                    .collect();
                return Ok(build_426_response(&held));
            }
        }

        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
        //
        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
        // 1. redact_segments() 在文本字符串层替换
        // 2. 把替换后的文本写回 AnthropicRequest messages
        // 3. serde_json 重新序列化为新 body
        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
        if !redact_hits.is_empty() {
            let seg_result = redact_segments(&texts, &redact_hits);
            tracing::info!(
                count = seg_result.redacted_count,
                rules = %seg_result.redacted_summary,
                "OUTBOUND AUTO-REDACT"
            );

            // 把替换后文本写回 AnthropicRequest，然后重新序列化
            let new_body_bytes =
                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
                    .and_then(|req| {
                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
                    })?;

            // 验证脱敏后 JSON 仍然合法（关键回归断言）
            if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
                return Err(anyhow!("redact_segments 产生了非法 JSON，fail-closed 拦截"));
            }

            let new_body = Bytes::from(new_body_bytes);
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

                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
                    // 关联：ADR-014 §双层防御、PRD §9 #3。

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
                    for d in &hook_detections {
                        write_hook_pending_silent(d);
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            // keep-alive channel：daemon 把心跳写入 SSE 流
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。

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
                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
                                    // 然后继续转发后续 SSE。
                                    if tx
                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    // 修 R2-#3：用户拒绝时不发触发帧，直接注入 sieve_blocked 并关流。
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
        let (blocking, hook_detections, flush_hold_detections) =
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
            return;
        }

        // 修 #5（flush 阶段 hold 丢失修复）：
        // flush 路径的 HoldForDecision 命中不能静默丢弃。
        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
        // 关联：ADR-014 §双层防御、PRD §9 #3。
        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
            }
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }
    });

    let body_stream = ReceiverStream::new(rx);

exec
/bin/zsh -lc "sed -n '760,1160p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
/// - `blocking`：`Action::Block` 需立即截流的 detections
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

/// 把脱敏后的文本段列表写回 [`AnthropicRequest`] 并返回新 request。
///
/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
/// `redacted_texts` 是 `redact_segments()` 返回的替换后文本列表（顺序对应）。
///
/// 实现逻辑：遍历 messages，对每个文本 content 按 segment 索引匹配并替换。
///
/// # Errors
/// 如果 `redacted_texts` 长度与 `original_texts` 不一致，返回错误。
///
/// 关联：PRD v1.4 §6.1（AutoRedact 路径），修 #1（AutoRedact 偏移修复）。
fn apply_redacted_texts_to_request(
    req: &sieve_core::protocol::anthropic::AnthropicRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> Result<sieve_core::protocol::anthropic::AnthropicRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(anyhow!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        ));
    }

    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
    let mut seg_idx = 0usize;

    let mut new_messages: Vec<sieve_core::protocol::anthropic::AnthropicMessage> = Vec::new();
    for msg in &req.messages {
        let new_content = match &msg.content {
            serde_json::Value::String(_) => {
                // String 类型：一个 segment
                let replacement = redacted_texts
                    .get(seg_idx)
                    .cloned()
                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
                seg_idx += 1;
                serde_json::Value::String(replacement)
            }
            serde_json::Value::Array(blocks) => {
                let mut new_blocks = Vec::with_capacity(blocks.len());
                for block in blocks {
                    if let Some(block_obj) = block.as_object() {
                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    block_obj
                                        .get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = block_obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_blocks.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    new_blocks.push(block.clone());
                }
                serde_json::Value::Array(new_blocks)
            }
            other => other.clone(),
        };
        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {

exec
/bin/zsh -lc "sed -n '1160,1245p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {
            role: msg.role.clone(),
            content: new_content,
        });
    }

    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
    let new_system = if let Some(system) = &req.system {
        if system.as_str().is_some() {
            let replacement = redacted_texts
                .get(seg_idx)
                .cloned()
                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
            seg_idx += 1;
            Some(serde_json::Value::String(replacement))
        } else if let Some(blocks) = system.as_array() {
            let mut new_blocks = Vec::with_capacity(blocks.len());
            for block in blocks {
                if block.get("text").and_then(|v| v.as_str()).is_some() {
                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    });
                    seg_idx += 1;
                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
                    new_blocks.push(serde_json::Value::Object(new_obj));
                } else {
                    new_blocks.push(block.clone());
                }
            }
            Some(serde_json::Value::Array(new_blocks))
        } else {
            Some(system.clone())
        }
    } else {
        None
    };

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(sieve_core::protocol::anthropic::AnthropicRequest {
        model: req.model.clone(),
        max_tokens: req.max_tokens,
        messages: new_messages,
        stream: req.stream,
        system: new_system,
        tools: req.tools.clone(),
        tool_choice: req.tool_choice.clone(),
        extra: req.extra.clone(),
    })
}

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-rules/rules/outbound.toml && sed -n '1,260p' crates/sieve-rules/rules/inbound.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "sed -n '260,620p' crates/sieve-rules/rules/inbound.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '286,355p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   286	        // 出站请求是非流式 HTTP：body 已 collect，无需 SSE keep-alive（入站才需要）。
   287	        // 客户端等待期间持有普通 HTTP 长连接（reqwest / Claude Code client 的超时决定等待上限）。
   288	        //
   289	        // 决策映射：
   290	        //   Allow → 原 body 转发上游
   291	        //   RedactAndAllow → redact_hits 非空则脱敏，否则原 body 转发
   292	        //   Deny → 426 拒绝
   293	        //   超时 → 按 default_on_timeout（OUT-06/08 = Redact，OUT-07/09/10 = Block）
   294	        //
   295	        // 关联：PRD v1.4 §5.4.2 出站超时策略表、ADR-016（二维处置矩阵）。
   296	        let hold_detections_outbound: Vec<&sieve_core::Detection> = all_detections
   297	            .iter()
   298	            .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
   299	            .collect();
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '347,410p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "sed -n '1,260p' crates/sieve-ipc/src/socket_server.rs && sed -n '1,240p' crates/sieve-ipc/src/protocol.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "sed -n '260,520p' crates/sieve-ipc/src/socket_server.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc 'git diff -- crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/engine/mod.rs crates/sieve-rules/benches/scan_bench.rs crates/sieve-rules/tests/inbound_rules.rs crates/sieve-rules/tests/outbound_rules.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "sed -n '1,260p' crates/sieve-cli/src/config.rs && printf '\\n--- setup ---\\n' && sed -n '1,360p' crates/sieve-cli/src/commands/setup.rs && printf '\\n--- doctor ---\\n' && sed -n '1,280p' crates/sieve-cli/src/commands/doctor.rs && printf '\\n--- uninstall ---\\n' && sed -n '1,360p' crates/sieve-cli/src/commands/uninstall.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
        PathBuf::from(".sieve").join("sieveignore")

--- setup ---
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

--- doctor ---
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

--- uninstall ---
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
                    // toml 文件同样按 created_new 判断：
                    // - created_new=false → setup 前用户已有该文件，从备份恢复
                    // - created_new=true  → setup 新建，但 created_new=true 分支在上面已处理
                    // 此处 created_new 必定为 false（else 分支），从备份恢复用户原文件。
                    if let Some(bd) = backup_dir {
                        restore_file_from_backup(bd, &info.path)?;
                    } else {
                        // 无备份可恢复：只能删除（避免残留 Sieve 配置影响用户）
                        fs::remove_file(&info.path).with_context(|| {
                            format!("删除 {} 失败（无备份）", info.path.display())
                        })?;
                        println!("[uninstall] ✅ 删除（无备份）: {}", info.path.display());
                    }
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '1,520p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '520,760p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
   621	mod tests {
   622	    use super::macos::{
   623	        build_default_sieve_toml, build_plist_content, strip_json_comments, SetupLogEntry,
   624	    };
   625	    use tempfile::tempdir;
   626	
   627	    // ── 测试 #1：plist 包含 --config <绝对路径>/sieve.toml ──────────────────
   628	    // 修复 #6 验证：launchd plist 必须含绝对路径 --config 和 WorkingDirectory
   629	
   630	    #[test]
   631	    fn plist_contains_absolute_config_flag() {
   632	        let dir = tempdir().unwrap();
   633	        let sieve_toml = dir.path().join("sieve.toml");
   634	        let plist = build_plist_content(&sieve_toml).unwrap();
   635	
   636	        assert!(
   637	            plist.contains("<string>--config</string>"),
   638	            "plist 必须包含 --config 参数: {plist}"
   639	        );
   640	        let config_str = sieve_toml.to_string_lossy();
   641	        assert!(
   642	            plist.contains(config_str.as_ref()),
   643	            "plist 必须包含 sieve.toml 绝对路径 {config_str}: {plist}"
   644	        );
   645	        assert!(
   646	            plist.contains("<key>WorkingDirectory</key>"),
   647	            "plist 必须包含 WorkingDirectory: {plist}"
   648	        );
   649	    }
   650	
   651	    // ── 测试 #2：解析失败的 JSON 返回 Err（不 fallback 到空对象）──────────────
   652	    // 修复 #8 核心：strip_json_comments + serde_json::from_str 失败路径
   653	
   654	    #[test]
   655	    fn bad_json_parse_returns_error_not_empty_object() {
   656	        // 尾逗号是无效 JSON，strip_json_comments 无法修复
   657	        let bad_json = r#"{"env": {"SOME_KEY": "value",},}"#;
   658	        let stripped = strip_json_comments(bad_json);
   659	        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);
   660	
   661	        // 修复前是 unwrap_or_else(|_| {}) 导致覆盖用户数据；修复后必须返回 Err
   662	        assert!(
   663	            result.is_err(),
   664	            "尾逗号 JSON 应解析失败，不得 fallback 到空对象"
   665	        );
   666	    }
   667	
   668	    // ── 测试 #3：SetupLogEntry 序列化 created_new 字段 ──────────────────────
   669	    // 修复 #9 数据基础：setup.log 正确记录 created_new=true/false
   670	
   671	    #[test]
   672	    fn setup_log_entry_created_new_serializes_correctly() {
   673	        let entry_new = SetupLogEntry::new("settings_updated")
   674	            .with_path("/tmp/test.json".to_string())
   675	            .with_created_new(true);
   676	        let json = serde_json::to_string(&entry_new).unwrap();
   677	        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
   678	        assert_eq!(
   679	            v.get("created_new").and_then(|c| c.as_bool()),
   680	            Some(true),
   681	            "新建文件 created_new 应序列化为 true: {json}"
   682	        );
   683	
   684	        let entry_existing = SetupLogEntry::new("settings_updated")
   685	            .with_path("/tmp/test.json".to_string())
   686	            .with_created_new(false);
   687	        let json2 = serde_json::to_string(&entry_existing).unwrap();
   688	        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
   689	        assert_eq!(
   690	            v2.get("created_new").and_then(|c| c.as_bool()),
   691	            Some(false),
   692	            "已有文件 created_new 应序列化为 false: {json2}"
   693	        );
   694	    }
   695	
   696	    // ── sieve.toml 使用绝对路径 ─────────────────────────────────────────────
   697	
   698	    #[test]
   699	    fn default_sieve_toml_has_absolute_paths() {
   700	        let dir = tempdir().unwrap();
   701	        let sieve_toml = dir.path().join("sieve.toml");
   702	        let content = build_default_sieve_toml(&sieve_toml).unwrap();
   703	
   704	        assert!(
   705	            content.contains(&format!("rules_path = \"{}", dir.path().display())),
   706	            "rules_path 必须是绝对路径: {content}"
   707	        );
   708	        assert!(
   709	            content.contains(&format!("audit_db_path = \"{}", dir.path().display())),
   710	            "audit_db_path 必须是绝对路径: {content}"
   711	        );
   712	    }
   713	
   714	    #[test]
   715	    fn default_sieve_toml_parses_as_config() {
   716	        // R2-#2：build_default_sieve_toml 生成的内容必须能被 Config 反序列化
   717	        use crate::config::Config;
   718	
   719	        let dir = tempdir().unwrap();
   720	        let sieve_toml = dir.path().join("sieve.toml");
   721	        let content = build_default_sieve_toml(&sieve_toml).unwrap();
   722	        let cfg: Config = toml::from_str(&content).unwrap_or_else(|e| {
   723	            panic!("build_default_sieve_toml 生成的 TOML 解析失败: {e}\n---\n{content}")
   724	        });
   725	        assert_eq!(cfg.port, 11453);
   726	        assert_eq!(cfg.bind_addr, "127.0.0.1");
   727	        assert_eq!(cfg.upstream_url, "https://api.anthropic.com");
   728	        assert!(cfg.audit_db_path.is_some(), "audit_db_path 应有绝对路径");
   729	        assert!(cfg.rules_path.is_some(), "rules_path 应有绝对路径");
   730	    }
   731	}

exec
/bin/zsh -lc "sed -n '1,180p' crates/sieve-cli/src/daemon.rs && rg \"resolved_rules_path|load_outbound_rules|load_inbound_rules\" -n crates/sieve-cli/src/main.rs crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

crates/sieve-cli/src/main.rs:32:use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
crates/sieve-cli/src/main.rs:61:            let rules_path = cfg.resolved_rules_path();
crates/sieve-cli/src/main.rs:63:            let rules = load_outbound_rules(&rules_path).with_context(|| {
crates/sieve-cli/src/main.rs:95:            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {

exec
/bin/zsh -lc "sed -n '40,112p' crates/sieve-cli/src/main.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
    match cli.command {
        Command::Start {
            config: cfg_path,
            dry_run: cli_dry_run,
        } => {
            let mut cfg = config::Config::load(&cfg_path)
                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;

            // CLI --dry-run 出现（true）时覆盖 config 中的值；
            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
            if cli_dry_run {
                cfg.dry_run = true;
            }

            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)

            let audit_path = cfg.audit_db_path()?;
            let _audit = AuditStore::init(&audit_path)
                .with_context(|| format!("init audit store at {}", audit_path.display()))?;

            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
            let rules_path = cfg.resolved_rules_path();
            tracing::info!(path = %rules_path.display(), "loading outbound rules");
            let rules = load_outbound_rules(&rules_path).with_context(|| {
                format!(
                    "failed to load outbound rules from {}; \
                     set rules_path in sieve.toml or ensure the default path exists",
                    rules_path.display()
                )
            })?;
            tracing::info!(count = rules.len(), "outbound rules loaded");

            // 编译出站 vectorscan db（fail-closed）
            let engine = VectorscanEngine::compile(rules.clone())
                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
            let adapter = OutboundAdapter::new(Arc::new(engine), rules);

            // 加载 .sieveignore（出站 + 入站共用同一份）
            let sieveignore_path = cfg.resolved_sieveignore_path();
            let sieveignore = load_sieveignore(&sieveignore_path);
            tracing::info!(
                path = %sieveignore_path.display(),
                entries = sieveignore.len(),
                "sieveignore loaded"
            );
            let sieveignore_arc = Arc::new(sieveignore);

            let filter = Arc::new(OutboundFilter::new(
                Arc::new(adapter),
                Arc::clone(&sieveignore_arc),
            ));

            // 加载入站规则（fail-closed，ADR-007）
            let inbound_rules_path = cfg.resolved_inbound_rules_path();
            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
                format!(
                    "failed to load inbound rules from {}; \
                         set inbound_rules_path in sieve.toml or ensure the default path exists",
                    inbound_rules_path.display()
                )
            })?;

            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
                .iter()
                .cloned()
                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
            tracing::info!(
                count = vectorscan_rules.len(),
                placeholders = placeholder_rules.len(),
                "inbound rules partitioned"
            );

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '1,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

exec
/bin/zsh -lc "for f in crates/sieve-ipc/src/*.rs crates/sieve-hook/src/*.rs crates/sieve-hook/Cargo.toml crates/sieve-ipc/Cargo.toml; do echo '---' "'$f; sed -n '"'1,260p' "'$f; done' in /Users/doskey/src/sieve
 succeeded in 0ms:
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
--- crates/sieve-hook/src/decision.rs
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
--- crates/sieve-hook/src/error.rs
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
--- crates/sieve-hook/src/lib.rs
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

--- crates/sieve-hook/src/main.rs
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
--- crates/sieve-hook/src/pending.rs
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
--- crates/sieve-hook/src/protocol.rs
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
--- crates/sieve-hook/Cargo.toml
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
--- crates/sieve-ipc/Cargo.toml
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
/bin/zsh -lc "sed -n '1,260p' crates/sieve-hook/src/decision.rs && sed -n '1,180p' crates/sieve-hook/src/error.rs && sed -n '1,120p' crates/sieve-hook/src/lib.rs && sed -n '180,380p' crates/sieve-hook/src/main.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc "cat Cargo.toml && printf '\\n--- cli cargo ---\\n' && cat crates/sieve-cli/Cargo.toml" in /Users/doskey/src/sieve
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

--- cli cargo ---
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
/bin/zsh -lc 'cargo check -p sieve-core --no-default-features' in /Users/doskey/src/sieve
 succeeded in 264ms:
    Checking sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

exec
/bin/zsh -lc 'git diff -- crates/sieve-cli/tests/outbound_block.rs crates/sieve-cli/tests/inbound_block.rs crates/sieve-cli/tests/audit_append_only.rs crates/sieve-cli/tests/sieve_setup_dry_run.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
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
+        "OUT-01 AutoRedact 应脱敏后转发，上游返回 200"
     );
 
-    // 5. 验证 body 是 sieve_blocked JSON
-    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
-    let body_str = String::from_utf8_lossy(&body_bytes);
-    assert!(
-        body_str.contains(r#""type":"sieve_blocked""#),
-        "body missing sieve_blocked type: {body_str}"
+    // 5. 上游应被调用一次
+    assert_eq!(
+        upstream_call_count.load(Ordering::SeqCst),
+        1,
+        "OUT-01 AutoRedact 后上游应被调用"
     );
+
+    // 6. 上游收到的 body 中不应含原始 key（已被 [REDACTED:OUT-01] 替换）
+    let received = upstream_body_received.lock().await.clone();
+    let received_str = String::from_utf8_lossy(&received);
     assert!(
-        body_str.contains("OUT-01"),
-        "body should mention OUT-01 rule: {body_str}"
+        !received_str.contains("sk-ant-api03-"),
+        "脱敏后上游不应收到原始 key：{received_str}"
     );
     assert!(
-        body_str.contains("guidance"),
-        "body should contain guidance field: {body_str}"
-    );
-
-    // 6. 验证上游未被调用
-    assert_eq!(
-        upstream_call_count.load(Ordering::SeqCst),
-        0,
-        "upstream should NOT be called when blocked"
+        received_str.contains("REDACTED"),
+        "脱敏后 body 应含 REDACTED 占位符：{received_str}"
     );
 }
 
-/// dry_run = true 时：OUT-01 属于 fail-closed 规则，即使 dry_run 也返回 426。
+/// dry_run = true 时：OUT-01（disposition=auto_redact）仍然脱敏转发，不受 dry_run 影响。
 ///
-/// 关联 PRD §9 #3 / ADR-007：fail-closed 规则在任何模式（含 dry_run）下都强制 Block。
-/// dry_run 只豁免非 fail-closed 的 Critical；OUT-01~12 全部在 FAIL_CLOSED_RULES 名单。
+/// 修 #2（disposition 优先）后，OUT-01 走 AutoRedact 路径；
+/// dry_run 只影响 Block 路径（是否拦截），不影响 AutoRedact（始终脱敏）。
+/// 验证 PRD v1.4 §6.1（AutoRedact 路径）、ADR-016（二维处置矩阵）。
 #[tokio::test]
-async fn dry_run_fail_closed_still_blocks() {
+async fn dry_run_auto_redact_still_redacts() {
     let upstream_call_count = Arc::new(AtomicUsize::new(0));
     let counter_clone = upstream_call_count.clone();
 
@@ -340,17 +367,17 @@ async fn dry_run_fail_closed_still_blocks() {
         .await
         .unwrap();
 
-    // OUT-01 是 fail-closed：dry_run 不豁免，仍返回 426
+    // OUT-01 AutoRedact：dry_run 不影响脱敏逻辑，脱敏后转发 → 200
     assert_eq!(
         resp.status(),
-        StatusCode::UPGRADE_REQUIRED,
-        "dry_run must NOT bypass fail-closed OUT-01 (PRD §9 #3)"
+        StatusCode::OK,
+        "OUT-01 AutoRedact 在 dry_run 模式下仍然脱敏转发（200）"
     );
-    // 上游不应被调用
+    // 上游应被调用（脱敏后转发）
     assert_eq!(
         upstream_call_count.load(Ordering::SeqCst),
-        0,
-        "upstream must NOT be called when fail-closed rule blocks"
+        1,
+        "OUT-01 AutoRedact 脱敏后上游应被调用"
     );
 }
 
@@ -406,3 +433,239 @@ async fn benign_message_passes_through() {
         "upstream should be called for benign message"
     );
 }
+
+// ─── 出站 GUI hold 测试（R2-#1 修复验证）────────────────────────────────────────
+
+/// 模拟 GUI 客户端：连接 IPC socket，通知 ready channel，等待 request_decision，用真实 request_id 回复。
+///
+/// 时序：
+/// 1. 连接 Unix socket（阻塞等待 socket 出现）
+/// 2. 发送 ready 信号（via `ready_tx`），通知调用方 GUI 已就绪
+/// 3. 阻塞等待服务端推来的 `request_decision` JSON-RPC 帧
+/// 4. 提取真实 request_id，用传入 decision 回复
+///
+/// 从请求帧提取 request_id 而非使用外部传入值，确保 IPC pending map 路由正确。
+async fn mock_gui_respond_with_ready(
+    socket_path: &std::path::Path,
+    decision: sieve_ipc::DecisionAction,
+    ready_tx: tokio::sync::oneshot::Sender<()>,
+) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
+    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
+    use tokio::net::UnixStream;
+
+    // 等 socket 出现
+    let mut stream = None;
+    for _ in 0..200 {
+        match UnixStream::connect(socket_path).await {
+            Ok(s) => {
+                stream = Some(s);
+                break;
+            }
+            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
+        }
+    }
+    let stream = stream.ok_or("IPC socket not ready after 20s")?;
+
+    // 稍等让 IPC server 完成 handle_connection spawn 和 gui_writer 注册（async 调度延迟）
+    tokio::time::sleep(Duration::from_millis(100)).await;
+
+    // 通知主任务：GUI 已连接且 IPC server 已注册，可以发 HTTP 请求了
+    let _ = ready_tx.send(());
+
+    let (reader, mut writer) = stream.into_split();
+    let mut lines = BufReader::new(reader).lines();
+
+    // 读服务端推来的 request_decision 帧
+    while let Some(line) = lines.next_line().await? {
+        let line = line.trim().to_owned();
+        if line.is_empty() {
+            continue;
+        }
+        // 从 JSON-RPC 帧中提取真实 request_id
+        let rpc: serde_json::Value = serde_json::from_str(&line)?;
+        let params = rpc.get("params").ok_or("no params")?;
+        let real_id: uuid::Uuid =
+            serde_json::from_value(params["request_id"].clone()).map_err(|e| e.to_string())?;
+
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
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '540,610p' && nl -ba crates/sieve-cli/src/daemon.rs | sed -n '860,925p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   540	                    let Some(frame_bytes) = frame.data_ref().cloned() else {
   541	                        if tx.send(Ok(frame)).await.is_err() {
   542	                            return;
   543	                        }
   544	                        continue;
   545	                    };
   546	
   547	                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
   548	                    let events = match parser.push_chunk(&frame_bytes) {
   549	                        Ok(evts) => evts,
   550	                        Err(e) => {
   551	                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
   552	                            let cap_detection =
   553	                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
   554	                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
   555	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   556	                            return;
   557	                        }
   558	                    };
   559	
   560	                    // 收集本批 events 的 detections，按 action 分组处理
   561	                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
   562	                        &events,
   563	                        &mut inbound_filter,
   564	                        &mut aggregator,
   565	                        dry_run,
   566	                    );
   567	
   568	                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
   569	                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
   570	                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
   571	                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
   572	                    // 关联：ADR-014 §双层防御、PRD §9 #3。
   573	
   574	                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
   575	                    if !blocking.is_empty() {
   576	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   577	                        for d in &blocking {
   578	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   579	                        }
   580	                        let blocked_payload = build_sieve_blocked_sse(&blocking);
   581	                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   582	                        return;
   583	                    }
   584	
   585	                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
   586	                    for d in &hook_detections {
   587	                        write_hook_pending_silent(d);
   588	                    }
   589	
   590	                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
   591	                    if !hold_detections.is_empty() {
   592	                        if let Some(ref ipc_server) = ipc {
   593	                            // keep-alive channel：daemon 把心跳写入 SSE 流
   594	                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
   595	                            let tx_ka = tx.clone();
   596	
   597	                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
   598	                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
   599	                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
   600	                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。
   601	
   602	                            // 启动 keep-alive 转发 task
   603	                            let ka_fwd_handle = tokio::spawn(async move {
   604	                                while let Some(ka_bytes) = ka_rx.recv().await {
   605	                                    if tx_ka
   606	                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
   607	                                        .await
   608	                                        .is_err()
   609	                                    {
   610	                                        break;
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
   906	            "fingerprint": d.fingerprint,
   907	        })).collect::<Vec<_>>(),
   908	        "guidance": {
   909	            "zh": format!(
   910	                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
   911	                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
   912	                detections.len()
   913	            ),
   914	            "en": format!(
   915	                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
   916	                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
   917	                detections.len()
   918	            ),
   919	        }
   920	    });
   921	    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
   922	}
   923	
   924	/// 用已收集的 body bytes 重新构造请求并转发。
   925	async fn forward_raw(

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/config.rs | sed -n '220,360p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   220	
   221	        if !self.tls_verify_upstream {
   222	            tracing::warn!(
   223	                "tls_verify_upstream=false: upstream TLS certificate NOT verified. \
   224	                 Only use in controlled test environments."
   225	            );
   226	        }
   227	
   228	        if self.dry_run {
   229	            tracing::warn!("dry_run mode: detections logged but not blocked");
   230	        }
   231	    }
   232	
   233	    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
   234	    pub fn resolved_rules_path(&self) -> PathBuf {
   235	        if let Some(p) = &self.rules_path {
   236	            return p.clone();
   237	        }
   238	        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
   239	    }
   240	
   241	    /// 解析入站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/inbound.toml`（相对 cwd）。
   242	    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
   243	        if let Some(p) = &self.inbound_rules_path {
   244	            return p.clone();
   245	        }
   246	        PathBuf::from("crates/sieve-rules/rules/inbound.toml")
   247	    }
   248	
   249	    /// 解析 `.sieveignore` 路径。显式给定时直接用，否则回退到 `~/.sieve/sieveignore`。
   250	    ///
   251	    /// 若 `HOME` 不可读则 fallback 到 `.sieve/sieveignore`（相对 cwd）并打印 WARN。
   252	    pub fn resolved_sieveignore_path(&self) -> PathBuf {
   253	        if let Some(p) = &self.sieveignore_path {
   254	            return p.clone();
   255	        }
   256	        if let Some(home) = std::env::var_os("HOME") {
   257	            return PathBuf::from(home).join(".sieve").join("sieveignore");
   258	        }
   259	        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");
   260	        PathBuf::from(".sieve").join("sieveignore")
   261	    }
   262	
   263	    /// 拼接监听 SocketAddr。
   264	    ///
   265	    /// # Errors
   266	    /// `bind_addr` 或 `port` 无法解析为合法 SocketAddr 时返回错误。
   267	    pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
   268	        format!("{}:{}", self.bind_addr, self.port)
   269	            .parse()
   270	            .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
   271	    }
   272	
   273	    /// 解析审计日志路径。优先级：`audit_db_path` > `log_path` > `~/.sieve/audit.db`。
   274	    ///
   275	    /// # Errors
   276	    /// `$HOME` 不存在或不可识别时返回错误。
   277	    pub fn audit_db_path(&self) -> Result<PathBuf> {
   278	        if let Some(p) = &self.audit_db_path {
   279	            return Ok(p.clone());
   280	        }
   281	        if let Some(p) = &self.log_path {
   282	            return Ok(p.clone());
   283	        }
   284	        let home = std::env::var_os("HOME").ok_or_else(|| {
   285	            anyhow!("HOME env var not set; specify audit_db_path or log_path explicitly")
   286	        })?;
   287	        Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
   288	    }
   289	}
   290	
   291	#[cfg(test)]
   292	mod tests {
   293	    use super::*;
   294	
   295	    #[test]
   296	    fn defaults_are_sane() {
   297	        let c = Config::default();
   298	        assert_eq!(c.bind_addr, "127.0.0.1");
   299	        assert_eq!(c.port, 11453);
   300	        assert_eq!(c.upstream_url, "https://api.anthropic.com");
   301	        assert!(c.tls_verify_upstream);
   302	        assert!(c.log_path.is_none());
   303	    }
   304	
   305	    #[test]
   306	    fn listen_addr_parses() {
   307	        let c = Config::default();
   308	        let addr = c.listen_addr().unwrap();
   309	        assert_eq!(addr.port(), 11453);
   310	        assert!(addr.ip().is_loopback());
   311	    }
   312	
   313	    #[test]
   314	    fn parse_minimal_toml() {
   315	        let toml_str = r#"
   316	            upstream_url = "https://api.anthropic.com"
   317	            port = 11453
   318	        "#;
   319	        let c: Config = toml::from_str(toml_str).unwrap();
   320	        assert_eq!(c.bind_addr, "127.0.0.1");
   321	        assert!(c.tls_verify_upstream);
   322	    }
   323	
   324	    #[test]
   325	    fn parse_full_toml() {
   326	        let toml_str = r#"
   327	            upstream_url = "https://api.anthropic.com"
   328	            port = 12000
   329	            bind_addr = "127.0.0.1"
   330	            tls_verify_upstream = false
   331	            log_path = "/tmp/audit.db"
   332	        "#;
   333	        let c: Config = toml::from_str(toml_str).unwrap();
   334	        assert_eq!(c.port, 12000);
   335	        assert!(!c.tls_verify_upstream);
   336	        assert_eq!(c.log_path.unwrap().to_str().unwrap(), "/tmp/audit.db");
   337	    }
   338	
   339	    #[test]
   340	    fn unknown_field_rejected() {
   341	        let toml_str = r#"
   342	            upstream_url = "https://api.anthropic.com"
   343	            disable_critical = true
   344	        "#;
   345	        let result: Result<Config, _> = toml::from_str(toml_str);
   346	        assert!(
   347	            result.is_err(),
   348	            "must reject unknown fields (deny_unknown_fields)"
   349	        );
   350	    }
   351	
   352	    #[test]
   353	    fn parse_dry_run_and_rules_path() {
   354	        let toml_str = r#"
   355	            upstream_url = "https://api.anthropic.com"
   356	            port = 11453
   357	            dry_run = true
   358	            rules_path = "/x.toml"
   359	        "#;
   360	        let c: Config = toml::from_str(toml_str).unwrap();

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/uninstall.rs | sed -n '180,420p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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
   361	        Ok(())
   362	    }
   363	
   364	    /// 打印备份目录中的文件列表。
   365	    fn list_backup_files(backup_dir: &std::path::Path) {
   366	        if let Ok(walker) = fs::read_dir(backup_dir) {
   367	            for entry in walker.flatten() {
   368	                println!("  - {}", entry.path().display());
   369	            }
   370	        }
   371	    }
   372	
   373	    /// 将备份目录中的文件逐一恢复到 home 下对应路径（旧格式 setup.log 兜底）。
   374	    fn restore_from_backup(
   375	        backup_dir: &std::path::Path,
   376	        home_path: &std::path::Path,
   377	    ) -> Result<()> {
   378	        restore_dir_recursive(backup_dir, backup_dir, home_path)
   379	    }
   380	
   381	    fn restore_dir_recursive(
   382	        root: &std::path::Path,
   383	        current: &std::path::Path,
   384	        home_path: &std::path::Path,
   385	    ) -> Result<()> {
   386	        for entry in fs::read_dir(current)
   387	            .with_context(|| format!("读取备份目录 {} 失败", current.display()))?
   388	        {
   389	            let entry = entry?;
   390	            let path = entry.path();
   391	            if path.is_dir() {
   392	                restore_dir_recursive(root, &path, home_path)?;
   393	            } else {
   394	                // 计算目标路径：backup_root/rel → home/rel
   395	                let rel = path.strip_prefix(root).unwrap_or(path.as_path());
   396	                let dest = home_path.join(rel);
   397	                if let Some(parent) = dest.parent() {
   398	                    fs::create_dir_all(parent)?;
   399	                }
   400	                fs::copy(&path, &dest).with_context(|| {
   401	                    format!("恢复 {} → {} 失败", path.display(), dest.display())
   402	                })?;
   403	                println!("[uninstall] ✅ 恢复 {}", dest.display());
   404	            }
   405	        }
   406	        Ok(())
   407	    }
   408	}
   409	
   410	// ──────────────────────────────── 非 macOS stub ─────────────────────────────
   411	
   412	#[cfg(not(target_os = "macos"))]
   413	mod stub {
   414	    use super::*;
   415	
   416	    /// `sieve uninstall` 非 macOS 占位实现。
   417	    pub fn run(_args: UninstallArgs) -> Result<()> {
   418	        anyhow::bail!(
   419	            "sieve uninstall is macOS only in Phase 1. \
   420	             Linux/Windows support is planned for Phase 2."

exec
/bin/zsh -lc 'cat crates/sieve-cli/tests/sieve_setup_dry_run.rs && cat crates/sieve-cli/tests/audit_append_only.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
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
/bin/zsh -lc 'cargo test -p sieve-rules' in /Users/doskey/src/sieve
 succeeded in 9845ms:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.97s
     Running unittests src/lib.rs (target/debug/deps/sieve_rules-a62cd93e1e8ccc54)

running 42 tests
test critical_lock::tests::fail_closed_covers_all_in_cr ... ok
test critical_lock::tests::in_cr_04_persistence_fail_closed ... ok
test critical_lock::tests::known_critical_rules_in_list ... ok
test critical_lock::tests::in_cr_05_in_gui_rules ... ok
test critical_lock::tests::unknown_rule_not_failclosed ... ok
test critical_lock::tests::in_cr_02_in_hook_rules ... ok
test critical_lock::tests::enforce_overrides_action ... ok
test critical_lock::tests::hook_and_gui_rules_are_disjoint ... ok
test ed25519::tests::empty_verifier_rejects ... ok
test bip39::tests::unknown_word_rejected ... ok
test bip39::tests::wrong_word_count_rejected ... ok
test bip39::tests::invalid_checksum_12_words ... ok
test bip39::tests::known_valid_12_words ... ok
test bip39::tests::known_valid_24_words ... ok
test loader::tests::returns_error_on_missing_file ... ok
test manifest::tests::action_serde ... ok
test manifest::tests::default_on_timeout_serde_roundtrip ... ok
test manifest::tests::disposition_serde_roundtrip ... ok
test loader::tests::returns_error_on_invalid_toml ... ok
test manifest::tests::auto_redact_disposition_parses ... ok
test loader::tests::loads_minimal_toml ... ok
test manifest::tests::effective_disposition_critical_defaults_to_gui_popup ... ok
test loader::tests::loads_rule_with_optional_fields ... ok
test manifest::tests::effective_disposition_non_critical_defaults_to_status_bar ... ok
test manifest::tests::old_toml_without_disposition_parses_ok ... ok
test manifest::tests::parse_manifest_with_rules ... ok
test manifest::tests::explicit_hook_terminal_disposition_parses ... ok
test manifest::tests::parse_minimal_manifest ... ok
test manifest::tests::severity_serde ... ok
test wordlist::tests::about_at_index_3 ... ok
test wordlist::tests::first_word_abandon_at_index_0 ... ok
test wordlist::tests::last_word_zoo_at_index_2047 ... ok
test wordlist::tests::wordlist_has_2048_entries ... ok
test engine::tests::no_match_returns_empty ... ok
test engine::tests::compile_and_scan_simple ... ok
test engine::tests::multiple_patterns_match ... ok
test engine::tests::longest_match_per_start_dedup ... ok
test engine::tests::allowlist_stopword_excludes ... ok
test engine::tests::is_excluded_placeholder ... ok
test placeholder::tests::does_not_match_real_secrets ... ok
test placeholder::tests::detects_common_placeholders ... ok
test engine::tests::allowlist_regex_excludes ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/bip39_pipeline.rs (target/debug/deps/bip39_pipeline-5f133011aac86596)

running 11 tests
test candidate_windows_empty_tokens ... ok
test candidate_windows_skips_non_wordlist_tokens ... ok
test candidate_windows_all_non_wordlist ... ok
test bip39_wrong_word_count_rejected ... ok
test bip39_pipeline_valid_mnemonic_detected ... ok
test candidate_windows_12_words_produces_one_candidate ... ok
test bip39_valid_24_words_passes_checksum ... ok
test bip39_valid_12_words_passes_checksum ... ok
test candidate_windows_24_words_produces_multiple ... ok
test bip39_24_words_wrong_checksum_fails ... ok
test bip39_pipeline_wrong_checksum_not_detected ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/inbound_rules.rs (target/debug/deps/inbound_rules-88d0bd7d8a582e98)

running 40 tests
test in_cr_02_rm_rf ... ok
test in_cr_02_eval ... ok
test in_cr_03_aws_creds_hit ... ok
test benign_dev_text_no_in_cr_03 ... ok
test in_cr_03_dotenv_environment_no_false_match ... ok
test in_cr_03_eth_keystore_hit ... ok
test benign_text_no_hit ... ok
test in_cr_02_curl_pipe ... ok
test in_cr_03_aws_config_not_credentials ... ok
test in_cr_03_gcp_creds_hit ... ok
test in_cr_03_dotenv_hit ... ok
test in_cr_03_gpg_dir_hit ... ok
test in_cr_03_macos_keychain_hit ... ok
test in_cr_03_ssh_dir_hit ... ok
test in_cr_03_netrc_hit ... ok
test in_cr_04_crontab_list_no_hit ... ok
test in_cr_03_solana_keypair_hit ... ok
test in_cr_04_crontab_hit ... ok
test in_cr_03_ssh_private_hit ... ok
test in_cr_04_cron_d_write_hit ... ok
test in_cr_04_systemctl_enable_hit ... ok
test in_cr_04_shell_rc_read_no_hit ... ok
test in_cr_04_fish_config_hit ... ok
test in_cr_04_launchctl_list_no_hit ... ok
test in_cr_04_systemctl_status_no_hit ... ok
test in_cr_04_login_items_hit ... ok
test in_cr_04_launchctl_hit ... ok
test in_cr_04_launch_agent_plist_hit ... ok
test inbound_rules_disposition_fields_parsed ... ok
test in_cr_04_shell_rc_append_hit ... ok
test in_cr_03_ssh_private_pub_excluded ... ok
test in_cr_04_systemd_unit_write_hit ... ok
test in_cr_05_eth_sign ... ok
test in_cr_04_unrelated_commands_no_hit ... ok
test in_gen_01_javascript_uri ... ok
test in_cr_05_solana_sign ... ok
test in_gen_03_bash_c ... ok
test in_gen_04_markdown_exfil ... ok
test in_cr_03_ssh_dir_safe_files_excluded ... ok
test in_cr_03_dotenv_safe_suffixes_excluded ... ok

test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.31s

     Running tests/outbound_rules.rs (target/debug/deps/outbound_rules-941f074bb4fe15e6)

running 13 tests
test out_06_jwt ... ok
test outbound_rules_disposition_fields_parsed ... ok
test out_02_openai_key_proj_format ... ok
test out_08_stripe_live ... ok
test out_07_pem_key ... ok
test out_01_anthropic_key ... ok
test out_02_openai_key_legacy ... ok
test out_05_gcp_api_key ... ok
test out_10_openssh_key ... ok
test out_04_github_pat ... ok
test out_11_discord_token ... ok
test out_03_aws_access_key ... ok
test out_09_slack_token ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s

   Doc-tests sieve_rules

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exec
/bin/zsh -lc 'cargo test -p sieve-core --all-targets' in /Users/doskey/src/sieve
 exited 101 in 1799ms:
   Compiling sieve-core v0.1.0-alpha (/Users/doskey/src/sieve/crates/sieve-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.05s
     Running unittests src/lib.rs (target/debug/deps/sieve_core-1452a01c77c7f112)

running 85 tests
test address_guard::tests::extract_eth ... ok
test address_guard::tests::extract_no_match_short_hex ... ok
test address_guard::tests::extract_multiple_addresses ... ok
test detection::tests::severity_serde_lowercase ... ok
test detection::tests::different_rule_id_different_fingerprint ... ok
test detection::tests::long_content_truncated ... ok
test detection::tests::action_block_serde ... ok
test address_guard::tests::ignores_identical_address ... ok
test address_guard::tests::case_insensitive_comparison ... ok
test address_guard::tests::ignores_different_length ... ok
test detection::tests::same_input_same_fingerprint ... ok
test detection::tests::fingerprint_is_16_hex_chars ... ok
test address_guard::tests::ignores_completely_different_address ... ok
test address_guard::tests::detects_one_char_substitution ... ok
test forwarder::tests::forwarder_new_invalid_uri_returns_error ... ok
test pipeline::inbound::tests::non_text_delta_event_returns_no_hits ... ok
test pipeline::inbound::tests::dangerous_shell_in_text_detected ... ok
test pipeline::inbound::tests::address_substitution_detected_across_events ... ok
test pipeline::inbound::tests::seed_from_prompt_enables_first_round_address_substitution_detection ... ok
test forwarder::tests::rewrite_uri_keeps_path_and_query ... ok
test forwarder::tests::rewrite_uri_root_path ... ok
test forwarder::tests::forwarder_new_parses_https_uri ... ok
test pipeline::inbound::tests::sieveignore_filters_non_critical_fingerprint ... ok
test pipeline::inbound::tests::signing_tool_call_detected ... ok
test pipeline::inbound::tests::sieveignore_does_not_suppress_critical ... ok
test pipeline::dispatch_impl::tests::dispatch_empty_returns_passthrough ... ok
test pipeline::dispatch_impl::tests::dispatch_mark_only_returns_status_bar ... ok
test pipeline::dispatch_impl::tests::dispatch_block_returns_deny ... ok
test pipeline::dispatch_impl::tests::dispatch_redact_returns_rewrite_body ... ok
test pipeline::outbound::tests::sieveignore_filters_out_known_fingerprint ... ok
test pipeline::outbound::tests::assistant_message_skipped ... ok
test pipeline::outbound::tests::user_message_with_secret_is_detected ... ok
test pipeline::outbound_redact::tests::multiple_non_overlapping_spans ... ok
test pipeline::outbound_redact::tests::empty_hits_returns_original ... ok
test pipeline::outbound_redact::tests::overlapping_spans_merged ... ok
test pipeline::outbound_redact::tests::segments_single_hit_secret_removed ... ok
test pipeline::outbound_redact::tests::segments_hit_outside_all_segments_no_change ... ok
test pipeline::outbound_redact::tests::segments_utf8_chinese_emoji_hit ... ok
test pipeline::outbound_redact::tests::segments_multiple_hits_different_segments ... ok
test pipeline::outbound_redact::tests::single_span_middle ... ok
test pipeline::outbound_redact::tests::utf8_boundary_alignment ... ok
test pipeline::outbound_redact::tests::span_clamped_to_body_len ... ok
test pipeline::outbound_redact::tests::utf8_body_redact_aligned ... ok
test protocol::anthropic::tests::parse_minimal_request ... ok
test protocol::anthropic::tests_extract::extract_content_blocks ... ok
test protocol::anthropic::tests::parse_streaming_request_with_tools ... ok
test protocol::anthropic::tests_extract::extract_simple_string_content ... ok
test protocol::anthropic::tests_extract::extract_with_system_prompt ... ok
test protocol::anthropic::tests::roundtrip_preserves_extra_fields ... ok
test protocol::unified_message::tests::direction_serde ... ok
test protocol::unified_message::tests::upstream_provider_anthropic_serde ... ok
test protocol::unified_message::tests::role_serde_lowercase ... ok
test protocol::unified_message::tests::content_block_text_roundtrip ... ok
test sse::parser::tests::empty_chunk_no_events ... ok
test sse::parser::tests::c0_control_chars_in_data ... ok
test sse::parser::tests::crlf_separator_accepted ... ok
test sse::parser::tests::flush_returns_buffer_remainder_at_eof ... ok
test pipeline::inbound_hold::tests::timeout_with_block_returns_deny ... FAILED
test pipeline::inbound_hold::tests::ipc_allow_returns_allow_outcome ... FAILED
test sse::parser::tests::anthropic_message_start_parses ... ok
test sse::parser::tests::multiple_data_lines_joined ... ok
test sse::parser::tests::parse_half_line_chunk ... ok
test pipeline::inbound_hold::tests::timeout_with_allow_returns_allow ... FAILED
test sse::parser::tests::malformed_returns_unknown_not_panic ... ok
test pipeline::inbound_hold::tests::keep_alive_sent_before_decision ... FAILED
test pipeline::inbound_hold::tests::ipc_deny_returns_deny_outcome ... FAILED
test sse::parser::tests::parse_input_json_delta ... ok
test sse::parser::tests::parse_multi_event_packed ... ok
test sse::parser::tests::parse_single_event ... ok
test sse::parser::tests::parse_split_separator ... ok
test sse::parser::tests::parse_text_delta ... ok
test sse::passthrough::tests::observe_arbitrary_bytes_ok ... ok
test sse::passthrough::tests::observe_empty_chunk_ok ... ok
test tool_use_aggregator::tests::aggregate_text_block_no_completion ... ok
test tool_use_aggregator::tests::malformed_partial_json_returns_malformed_error ... ok
test tool_use_aggregator::tests::aggregate_tool_use_split_partial_json ... ok
test tool_use_aggregator::tests::multiple_blocks_independent ... ok
test tool_use_aggregator::tests::unknown_block_type_stop_returns_none ... ok
test tool_use_aggregator::tests::tool_use_malformed_partial_json_returns_malformed_error ... ok
test sse::parser::tests::push_chunk_over_limit_returns_event_too_large ... ok
test tool_use_aggregator::tests::too_many_open_blocks_returns_error ... ok
test tool_use_aggregator::tests::partial_json_over_limit_returns_error ... ok
test pipeline::dispatch_impl::tests::dispatch_hook_mark_returns_hook_marked ... ok
test pipeline::inbound_hook::tests::write_and_read_pending_file ... ok
test pipeline::inbound_hook::tests::write_hook_pending_idempotent_on_same_id ... ok

failures:

---- pipeline::inbound_hold::tests::timeout_with_block_returns_deny stdout ----

thread 'pipeline::inbound_hold::tests::timeout_with_block_returns_deny' panicked at crates/sieve-core/src/pipeline/inbound_hold.rs:172:14:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- pipeline::inbound_hold::tests::ipc_allow_returns_allow_outcome stdout ----

thread 'pipeline::inbound_hold::tests::ipc_allow_returns_allow_outcome' panicked at crates/sieve-core/src/pipeline/inbound_hold.rs:172:14:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })

---- pipeline::inbound_hold::tests::timeout_with_allow_returns_allow stdout ----

thread 'pipeline::inbound_hold::tests::timeout_with_allow_returns_allow' panicked at crates/sieve-core/src/pipeline/inbound_hold.rs:172:14:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })

---- pipeline::inbound_hold::tests::keep_alive_sent_before_decision stdout ----

thread 'pipeline::inbound_hold::tests::keep_alive_sent_before_decision' panicked at crates/sieve-core/src/pipeline/inbound_hold.rs:172:14:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })

---- pipeline::inbound_hold::tests::ipc_deny_returns_deny_outcome stdout ----

thread 'pipeline::inbound_hold::tests::ipc_deny_returns_deny_outcome' panicked at crates/sieve-core/src/pipeline/inbound_hold.rs:172:14:
called `Result::unwrap()` on an `Err` value: Socket(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })


failures:
    pipeline::inbound_hold::tests::ipc_allow_returns_allow_outcome
    pipeline::inbound_hold::tests::ipc_deny_returns_deny_outcome
    pipeline::inbound_hold::tests::keep_alive_sent_before_decision
    pipeline::inbound_hold::tests::timeout_with_allow_returns_allow
    pipeline::inbound_hold::tests::timeout_with_block_returns_deny

test result: FAILED. 80 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p sieve-core --lib`

exec
/bin/zsh -lc "nl -ba crates/sieve-hook/src/pending.rs | sed -n '60,105p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

exec
/bin/zsh -lc "sed -n '1,300p' crates/sieve-ipc/src/socket_client.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

exec
/bin/zsh -lc 'cat fuzz/Cargo.toml && cargo check -p sieve-fuzz' in /Users/doskey/src/sieve
 succeeded in 111ms:
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-core/src/pipeline/inbound.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
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

exec
/bin/zsh -lc "nl -ba crates/sieve-rules/rules/inbound.toml | sed -n '1,32p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
     1	# 入站 P0 规则集（关联 PRD §5.2 + UCSB 论文 4 类攻击）
     2	# 来源：Semgrep command-injection（MIT/Apache-2.0）+ Sieve 自写
     3	#
     4	# Vectorscan PCRE 子集约束：
     5	#   - 禁用 lookahead / lookbehind / 反向引用 / 原子组
     6	#   - 所有 pattern 仅用 (?i) + 字符类 + 量词 + 分组
     7	
     8	# IN-CR-01 是地址替换，由 sieve-core::address_guard 实现（strsim Levenshtein），
     9	# 此处仅占位以保持 ID 注册；loader 看到 pattern == "__ADDRESS_GUARD_PLACEHOLDER__" 时跳过 vectorscan 编译。
    10	[[rules]]
    11	id = "IN-CR-01"
    12	description = "Address substitution attack (handled by sieve-core::address_guard with strsim Levenshtein)"
    13	pattern = "__ADDRESS_GUARD_PLACEHOLDER__"
    14	severity = "critical"
    15	action = "block"
    16	disposition = "gui_popup"
    17	timeout_seconds = 60
    18	default_on_timeout = "block"
    19	
    20	# IN-CR-02 危险 shell 命令（Semgrep command-injection 风格）
    21	[[rules]]
    22	id = "IN-CR-02"
    23	description = "Destructive rm command"
    24	pattern = '''(?i)rm\s+-rf?\s+[/~*]'''
    25	severity = "critical"
    26	action = "block"
    27	keywords = ["rm"]
    28	disposition = "hook_terminal"
    29	timeout_seconds = 30
    30	default_on_timeout = "block"
    31	
    32	[[rules]]

exec
/bin/zsh -lc "sed -n '95,140p' crates/sieve-cli/src/main.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
                format!(
                    "failed to load inbound rules from {}; \
                         set inbound_rules_path in sieve.toml or ensure the default path exists",
                    inbound_rules_path.display()
                )
            })?;

            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
                .iter()
                .cloned()
                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
            tracing::info!(
                count = vectorscan_rules.len(),
                placeholders = placeholder_rules.len(),
                "inbound rules partitioned"
            );

            // 编译入站 vectorscan db（独立实例，fail-closed）
            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;
            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
            let inbound_adapter =
                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);

            // YOLO mode 运行时审计（防御性双保险）
            audit_yolo_disabled(&cfg)?;

            daemon::run(
                cfg,
                filter,
                Arc::new(inbound_adapter),
                Arc::clone(&sieveignore_arc),
            )
            .await?;
        }
        Command::Version => {
            println!("sieve {}", env!("CARGO_PKG_VERSION"));
        }
        Command::Setup(args) => {
            commands::setup::run(args)?;
        }
        Command::Doctor => {
            commands::doctor::run()?;
        }

codex
当前改动让 setup 生成的安装无法正常启动 daemon，并且多条新安全路径存在 fail-open 或红action策略失效问题。即使代码能编译，这些场景会破坏核心拦截/脱敏语义。

Full review comments:

- [P1] 让 setup 安装或引用真实规则文件 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:508-509
  在 fresh `sieve setup` 后，生成的配置把 `rules_path`/`inbound_rules_path` 指到 `~/.sieve/rules/*.toml`，但 setup 流程没有创建该目录或复制内置规则；launchd 启动的 `sieve start --config ~/.sieve/sieve.toml` 会在加载规则时报错退出，导致一键安装不可用。

- [P1] 不要在 Hook pending 写入失败时放行 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:877-879
  当 `IN-CR-02/IN-CR-04` 这类 `hook_terminal` 命中后，如果 `~/.sieve/pending` 不可写、磁盘错误或权限异常，这里只记录 warn 后继续转发 SSE；随后静态 `sieve-hook check` 找不到 pending 会 fail-open，危险工具调用就没有任何拦截点，违反这些规则的 fail-closed 语义。

- [P1] 对 RedactAndAllow 补充 Hold 命中的脱敏范围 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:353-356
  当 GUI 对 `OUT-06/OUT-08` 这类 `gui_popup` 命中返回 `RedactAndAllow` 时，此分支只是 fall through 到下方 `redact_hits`，但 `redact_hits` 只收集 `Action::Redact`，不包含当前 `HoldForDecision` 的 span；因此没有同时命中 AutoRedact 时会把 JWT/Stripe token 原样转发给上游。

- [P2] 按规则传递 outbound GUI 的超时默认动作 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:313-317
  `OUT-06` 和 `OUT-08` 在规则中配置了 `default_on_timeout=redact`，但这里给 IPC 请求硬编码成 `Block`；在无 GUI 连接或 GUI 超时时，`request_decision` 会返回 Deny/426，而不是按规则脱敏后放行，导致新增的超时策略实际不生效。

- [P2] 让 IN-CR-01 的 gui_popup 配置真正生效 — /Users/doskey/src/sieve/crates/sieve-rules/rules/inbound.toml:16-16
  `IN-CR-01` 是 placeholder 规则，实际命中由 `sieve-core::address_guard` 直接构造 `Action::Block`，不会经过 `InboundAdapter` 的 disposition 映射；因此这里新增的 `gui_popup` 配置不会被使用，地址替换仍会立即注入 `sieve_blocked`，不会走 GUI 确认路径。

- [P1] 不要在启发式 hook 中跳过坏 pending 文件 — /Users/doskey/src/sieve/crates/sieve-hook/src/pending.rs:83-90
  默认 setup 注册的是不带 request_id 的 `sieve-hook check`，所以会走这个启发式扫描；如果某个 fresh pending 文件写到一半、损坏或因读错误无法解析，这里直接跳过，可能让 `fresh` 为空并 exit 0，从而放行本应被 HookTerminal 拦截的工具调用。
2026-04-28T03:02:48.332164Z ERROR codex_core::session: failed to record rollout items: thread 019dd204-667f-7190-a8ac-a76791de3823 not found
2026-04-28T03:02:48.366529Z ERROR codex_core::session: failed to record rollout items: thread 019dd204-6645-76a1-a342-04a348e9e8d0 not found
当前改动让 setup 生成的安装无法正常启动 daemon，并且多条新安全路径存在 fail-open 或红action策略失效问题。即使代码能编译，这些场景会破坏核心拦截/脱敏语义。

Full review comments:

- [P1] 让 setup 安装或引用真实规则文件 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/setup.rs:508-509
  在 fresh `sieve setup` 后，生成的配置把 `rules_path`/`inbound_rules_path` 指到 `~/.sieve/rules/*.toml`，但 setup 流程没有创建该目录或复制内置规则；launchd 启动的 `sieve start --config ~/.sieve/sieve.toml` 会在加载规则时报错退出，导致一键安装不可用。

- [P1] 不要在 Hook pending 写入失败时放行 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:877-879
  当 `IN-CR-02/IN-CR-04` 这类 `hook_terminal` 命中后，如果 `~/.sieve/pending` 不可写、磁盘错误或权限异常，这里只记录 warn 后继续转发 SSE；随后静态 `sieve-hook check` 找不到 pending 会 fail-open，危险工具调用就没有任何拦截点，违反这些规则的 fail-closed 语义。

- [P1] 对 RedactAndAllow 补充 Hold 命中的脱敏范围 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:353-356
  当 GUI 对 `OUT-06/OUT-08` 这类 `gui_popup` 命中返回 `RedactAndAllow` 时，此分支只是 fall through 到下方 `redact_hits`，但 `redact_hits` 只收集 `Action::Redact`，不包含当前 `HoldForDecision` 的 span；因此没有同时命中 AutoRedact 时会把 JWT/Stripe token 原样转发给上游。

- [P2] 按规则传递 outbound GUI 的超时默认动作 — /Users/doskey/src/sieve/crates/sieve-cli/src/daemon.rs:313-317
  `OUT-06` 和 `OUT-08` 在规则中配置了 `default_on_timeout=redact`，但这里给 IPC 请求硬编码成 `Block`；在无 GUI 连接或 GUI 超时时，`request_decision` 会返回 Deny/426，而不是按规则脱敏后放行，导致新增的超时策略实际不生效。

- [P2] 让 IN-CR-01 的 gui_popup 配置真正生效 — /Users/doskey/src/sieve/crates/sieve-rules/rules/inbound.toml:16-16
  `IN-CR-01` 是 placeholder 规则，实际命中由 `sieve-core::address_guard` 直接构造 `Action::Block`，不会经过 `InboundAdapter` 的 disposition 映射；因此这里新增的 `gui_popup` 配置不会被使用，地址替换仍会立即注入 `sieve_blocked`，不会走 GUI 确认路径。

- [P1] 不要在启发式 hook 中跳过坏 pending 文件 — /Users/doskey/src/sieve/crates/sieve-hook/src/pending.rs:83-90
  默认 setup 注册的是不带 request_id 的 `sieve-hook check`，所以会走这个启发式扫描；如果某个 fresh pending 文件写到一半、损坏或因读错误无法解析，这里直接跳过，可能让 `fresh` 为空并 exit 0，从而放行本应被 HookTerminal 拦截的工具调用。
