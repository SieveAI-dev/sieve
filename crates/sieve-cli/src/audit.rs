//! 审计日志（关联 data-model.md §审计 + ADR-007）。
//!
//! Week 1：仅占位 schema 与目录初始化。Week 4 起接入实际事件写入。
//!
//! 设计约束（ADR-007）：
//! - SQLite append-only；BEFORE UPDATE / DELETE 触发器拒绝修改（Week 4 建表时实施）。
//! - 不引入 `rusqlite` 依赖，直到 Week 4 实际写入需求确立（避免早期锁定版本）。
//!
//! Week 4 接入时需补充的内容：
//! - `rusqlite` / `sqlx` 依赖与建表 DDL；
//! - `AuditEvent` 枚举（Request / Response / Block / Allow / Error）；
//! - `AuditStore::append` 异步写入接口；
//! - BEFORE UPDATE / DELETE 触发器 SQL。

use anyhow::Result;
use std::path::Path;

/// 审计存储句柄（Week 1 占位）。
///
/// Week 4 起持有 SQLite 连接池；当前仅确保目录存在。
pub struct AuditStore;

impl AuditStore {
    /// 初始化审计存储。
    ///
    /// Week 1：仅创建父目录（若不存在），不建表、不打开数据库文件。
    /// Week 4：将在此处打开 / 迁移 SQLite，并建立 append-only 触发器。
    ///
    /// # Errors
    /// 目录创建失败时返回错误（Week 1 实际不可能失败，因 `create_dir_all` 忽略已存在）。
    pub fn init(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        tracing::debug!(path = %path.display(), "audit store placeholder initialized");
        Ok(Self)
    }
}
