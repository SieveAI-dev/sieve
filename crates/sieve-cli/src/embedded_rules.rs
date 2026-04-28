//! 内嵌规则文件（F-2 修复：setup 时写出到 ~/.sieve/rules/）。
//!
//! ## 设计
//!
//! 用 `include_str!` 把 sieve-rules 的两个规则 TOML 编入 sieve-cli 二进制，
//! setup 时调 `install_to()` 写到 `~/.sieve/rules/`，daemon 启动后可直接加载。
//! 体积增量 < 100 KB（出站 ~8 KB + 入站 ~15 KB）。
//!
//! ## 关联
//!
//! - known-issues-v1.4.md P1-R3-#1（现已修复）
//! - SPEC-003 §setup

use anyhow::{Context, Result};
use std::path::Path;

/// 内嵌的出站规则 TOML 内容（编译期打入二进制）。
pub const OUTBOUND_RULES: &str = include_str!("../../sieve-rules/rules/outbound.toml");

/// 内嵌的入站规则 TOML 内容（编译期打入二进制）。
pub const INBOUND_RULES: &str = include_str!("../../sieve-rules/rules/inbound.toml");

/// 把两个规则文件写到 `dir`（`~/.sieve/rules/`）。
///
/// `dir` 不存在时自动创建。已存在的文件直接覆盖（幂等）。
///
/// # Errors
///
/// 目录创建失败或文件写入失败时返回 `Err`。
pub fn install_to(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("创建规则目录 {} 失败", dir.display()))?;
    std::fs::write(dir.join("outbound.toml"), OUTBOUND_RULES)
        .with_context(|| format!("写入 outbound.toml 到 {} 失败", dir.display()))?;
    std::fs::write(dir.join("inbound.toml"), INBOUND_RULES)
        .with_context(|| format!("写入 inbound.toml 到 {} 失败", dir.display()))?;
    Ok(())
}
