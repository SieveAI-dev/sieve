//! 行为序列窗口（Phase B beta，默认关闭）。
//!
//! # 设计原则
//!
//! - **保守起步**：仅触发 StatusBar 通知，不引入新 Block 路径
//! - **GA 默认关闭**：通过 cargo feature `sequence_detection`，调用方默认禁用
//! - **隐私安全**：只存结构化特征枚举，不存原始 input
//! - **双路径不变量**：调用方必须从 SSE + JSON 两条路径同时调 [`ToolUseSequence::record`]

use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod detector;
pub mod feature;

pub use detector::{detect_kill_chains, SequenceHit};
pub use feature::{extract_record, PathCategory, SecretConfidence, ToolClass};

/// 滑动窗口配置。
#[derive(Debug, Clone)]
pub struct SequenceConfig {
    /// 最大保留事件数，默认 10。
    pub max_size: usize,
    /// 事件过期时长（毫秒），默认 300_000（5 分钟）。
    pub expires_after_ms: u64,
}

impl Default for SequenceConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            expires_after_ms: 300_000,
        }
    }
}

/// 单次工具调用的结构化特征。
///
/// 不存原始 input：用预定义枚举 + 布尔特征替代，便于隐私 + ML 升级路径。
#[derive(Debug, Clone, Default)]
pub struct ToolUseRecord {
    /// Unix epoch 毫秒时间戳；调用方赋值，`record()` 在值为 0 时自动填充。
    pub timestamp_ms: u64,
    /// 工具名（Bash / Read / Write 等）。
    pub tool_name: String,
    /// 工具类型分类，见 [`ToolClass`]。
    pub tool_class: ToolClass,
    /// 路径分类（可选），见 [`PathCategory`]。
    pub path_category: Option<PathCategory>,
    /// 是否含出站网络动作（curl / WebFetch 等）。
    pub network_egress: bool,
    /// 是否含持久化机制（crontab / launchctl / rc 文件等）。
    pub persistence_mech: bool,
    /// 是否含清除痕迹动作（rm -rf / history -c 等）。
    pub cleanup_mech: bool,
    /// input 中是否含敏感文件 hint（.env / id_rsa 等）。
    pub sensitive_file_hint: bool,
    /// secret 识别置信度（None/Heuristic/ChecksumConfirmed），见 [`SecretConfidence`]。
    pub secret_confidence: SecretConfidence,
    /// 是否含打包动作（tar / zip / 7z 等）——exfil 前中转压缩。
    pub archive_mech: bool,
    /// 是否含编码 / 加密动作（base64 / openssl enc / gpg 等）——绕 DLP 的 exfil 前兆。
    pub encode_mech: bool,
    /// 是否含剪贴板写入动作（pbcopy / xclip 等）——secret 脱离流量视野的隐蔽通道。
    pub clipboard_mech: bool,
    /// 是否写入 / 发布到公共产物路径（dist/build/ 或 npm publish 等）。
    pub public_artifact_target: bool,
    /// 是否接触生产数据库 / 凭据（psql / pg_dump / DATABASE_URL 等）。
    pub prod_data_hint: bool,
    /// 来源 actor（source_channel / agent id），用于跨 agent 链判定；调用方填。
    pub actor: Option<String>,
    /// 此次单次检测命中的规则 ID（可能为空）。用于序列规则跨触发关联。
    pub rule_hits: Vec<String>,
}

/// 滑动窗口（ToolUseSequence）。
#[derive(Debug)]
pub struct ToolUseSequence {
    window: VecDeque<ToolUseRecord>,
    config: SequenceConfig,
}

impl Default for ToolUseSequence {
    fn default() -> Self {
        Self::new(SequenceConfig::default())
    }
}

impl ToolUseSequence {
    /// 新建序列窗口，使用给定配置。
    pub fn new(config: SequenceConfig) -> Self {
        Self {
            window: VecDeque::with_capacity(config.max_size),
            config,
        }
    }

    /// 加一条 record + 自动剔除过期 + 维持窗口上限。
    ///
    /// 若 `rec.timestamp_ms == 0`，自动用当前系统时间填充。
    /// 过期判断：当前 record 时间戳 - 最老 record 时间戳 > expires_after_ms。
    pub fn record(&mut self, mut rec: ToolUseRecord) {
        if rec.timestamp_ms == 0 {
            rec.timestamp_ms = now_ms();
        }
        // 剔除过期（按 expires_after_ms 截断）
        let now = rec.timestamp_ms;
        while let Some(front) = self.window.front() {
            if now.saturating_sub(front.timestamp_ms) > self.config.expires_after_ms {
                self.window.pop_front();
            } else {
                break;
            }
        }
        // 维持上限
        if self.window.len() >= self.config.max_size {
            self.window.pop_front();
        }
        self.window.push_back(rec);
    }

    /// 当前窗口大小。
    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// 窗口是否为空。
    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    /// 提供给 detector 的只读迭代器（时间正序）。
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &ToolUseRecord> {
        self.window.iter()
    }

    /// 清空窗口（用于测试 / 会话结束）。
    pub fn clear(&mut self) {
        self.window.clear();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(tool_name: &str, timestamp_ms: u64) -> ToolUseRecord {
        ToolUseRecord {
            timestamp_ms,
            tool_name: tool_name.to_string(),
            tool_class: ToolClass::Other,
            path_category: None,
            network_egress: false,
            persistence_mech: false,
            cleanup_mech: false,
            sensitive_file_hint: false,
            secret_confidence: SecretConfidence::None,
            archive_mech: false,
            encode_mech: false,
            clipboard_mech: false,
            public_artifact_target: false,
            prod_data_hint: false,
            actor: None,
            rule_hits: vec![],
        }
    }

    #[test]
    fn empty_window_on_default() {
        let seq = ToolUseSequence::default();
        assert!(seq.is_empty());
        assert_eq!(seq.len(), 0);
    }

    #[test]
    fn record_single_item() {
        let mut seq = ToolUseSequence::default();
        seq.record(make_record("Bash", 1_000));
        assert_eq!(seq.len(), 1);
    }

    #[test]
    fn window_evicts_oldest_when_at_max() {
        let config = SequenceConfig {
            max_size: 3,
            expires_after_ms: 300_000,
        };
        let mut seq = ToolUseSequence::new(config);
        seq.record(make_record("A", 1_000));
        seq.record(make_record("B", 2_000));
        seq.record(make_record("C", 3_000));
        assert_eq!(seq.len(), 3);
        // 第四条进来，最老的 A 被挤出
        seq.record(make_record("D", 4_000));
        assert_eq!(seq.len(), 3);
        let names: Vec<&str> = seq.iter().map(|r| r.tool_name.as_str()).collect();
        assert!(!names.contains(&"A"), "A should have been evicted");
        assert!(names.contains(&"D"));
    }

    #[test]
    fn expired_records_are_evicted() {
        let config = SequenceConfig {
            max_size: 10,
            expires_after_ms: 1_000, // 1 秒过期
        };
        let mut seq = ToolUseSequence::new(config);
        // 使用具体的过去时间戳（非 0，避免触发自动填充）
        seq.record(make_record("Old", 1_000));
        // 3000ms 后加入新记录（超过 expires_after_ms=1000），旧记录应被驱逐
        seq.record(make_record("New", 4_000));
        assert_eq!(seq.len(), 1);
        assert_eq!(seq.iter().next().unwrap().tool_name, "New");
    }

    #[test]
    fn zero_timestamp_is_auto_filled() {
        let mut seq = ToolUseSequence::default();
        let rec = make_record("Bash", 0); // timestamp_ms=0，触发自动填充
        seq.record(rec);
        let stored = seq.iter().next().unwrap();
        assert!(
            stored.timestamp_ms > 0,
            "timestamp should be auto-filled when 0"
        );
    }

    #[test]
    fn iter_is_time_ordered() {
        let mut seq = ToolUseSequence::default();
        seq.record(make_record("First", 100));
        seq.record(make_record("Second", 200));
        seq.record(make_record("Third", 300));
        let names: Vec<&str> = seq.iter().map(|r| r.tool_name.as_str()).collect();
        assert_eq!(names, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn clear_empties_window() {
        let mut seq = ToolUseSequence::default();
        seq.record(make_record("X", 1_000));
        seq.record(make_record("Y", 2_000));
        seq.clear();
        assert!(seq.is_empty());
    }
}
