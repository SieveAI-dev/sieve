//! 行为序列 kill chain 检测（PRD v2.0 §5.7.2）。
//!
//! 3 条规则全部 severity = High，**仅 StatusBar 通知**（PRD §9 #15 不引入 Block 路径）：
//!
//! - IN-SEQ-01-RECON-EXFIL: FileRead+SensitiveSecret 之后 network_egress
//! - IN-SEQ-02-CLEANUP-AFTER-ATTACK: Shell+network_egress 之后 cleanup_mech
//! - IN-SEQ-03-PERSISTENCE-CHAIN: 3 次 persistence_mech=true 跨不同 tool_name 调用

use super::feature::{PathCategory, ToolClass};
use super::{ToolUseRecord, ToolUseSequence};

/// 序列检测命中（PRD §5.7.2）。
///
/// severity 固定为 High，处置为 StatusBar 通知（PRD §9 #15 禁止升级为 Block）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceHit {
    /// 规则 ID（IN-SEQ-01 / IN-SEQ-02 / IN-SEQ-03）。
    pub rule_id: String,
    /// 人可读描述。
    pub description: String,
    /// 触发该 hit 的 record 下标（在 `ToolUseSequence::iter()` 顺序里）。
    pub triggering_indices: Vec<usize>,
}

/// 在序列窗口里跑全部 3 条规则，返回所有命中（可多个）。
///
/// 注意：调用本函数不产生副作用，不修改窗口状态。
pub fn detect_kill_chains(seq: &ToolUseSequence) -> Vec<SequenceHit> {
    let records: Vec<&ToolUseRecord> = seq.iter().collect();
    let mut hits = Vec::new();

    if let Some(h) = detect_recon_exfil(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_cleanup_after_attack(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_persistence_chain(&records) {
        hits.push(h);
    }

    hits
}

/// IN-SEQ-01: tool_class=FileRead + path_category=SensitiveSecret 之后某点 network_egress=true。
///
/// 意图：读敏感凭证后立即外发（数据窃取 kill chain 前两步）。
fn detect_recon_exfil(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    let recon_idx = records.iter().position(|r| {
        r.tool_class == ToolClass::FileRead
            && r.path_category == Some(PathCategory::SensitiveSecret)
    })?;
    let exfil_idx = records
        .iter()
        .enumerate()
        .skip(recon_idx + 1)
        .find(|(_, r)| r.network_egress)
        .map(|(i, _)| i)?;
    Some(SequenceHit {
        rule_id: "IN-SEQ-01-RECON-EXFIL".into(),
        description: "读敏感文件后外发请求（kill chain）".into(),
        triggering_indices: vec![recon_idx, exfil_idx],
    })
}

/// IN-SEQ-02: tool_class=Shell + network_egress=true 之后某点 cleanup_mech=true。
///
/// 意图：通过 shell 下载/执行攻击载荷后，清理证据痕迹。
fn detect_cleanup_after_attack(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    let attack_idx = records
        .iter()
        .position(|r| r.tool_class == ToolClass::Shell && r.network_egress)?;
    let cleanup_idx = records
        .iter()
        .enumerate()
        .skip(attack_idx + 1)
        .find(|(_, r)| r.cleanup_mech)
        .map(|(i, _)| i)?;
    Some(SequenceHit {
        rule_id: "IN-SEQ-02-CLEANUP-AFTER-ATTACK".into(),
        description: "执行远程脚本后立即删痕迹（kill chain）".into(),
        triggering_indices: vec![attack_idx, cleanup_idx],
    })
}

/// IN-SEQ-03: 3 次 persistence_mech=true 跨不同 tool_name 调用。
///
/// 意图：在多个不同工具中设置持久化机制，表明有意规避单一拦截点。
/// 注：按不同 `tool_name` 计算跨工具；同一工具名多次触发只算一个。
fn detect_persistence_chain(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    use std::collections::BTreeMap;
    // key = tool_name, value = 各次触发的 index 列表
    let mut by_tool: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, r) in records.iter().enumerate() {
        if r.persistence_mech {
            by_tool.entry(r.tool_name.as_str()).or_default().push(i);
        }
    }
    // 跨 3 个不同 tool_name 才触发
    if by_tool.len() < 3 {
        return None;
    }
    let mut indices: Vec<usize> = by_tool
        .values()
        .filter_map(|v| v.first().copied())
        .collect();
    indices.sort_unstable();
    Some(SequenceHit {
        rule_id: "IN-SEQ-03-PERSISTENCE-CHAIN".into(),
        description: "多机制持久化（kill chain）".into(),
        triggering_indices: indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence::feature::{PathCategory, ToolClass};
    use crate::sequence::{ToolUseRecord, ToolUseSequence};

    fn make_record(
        tool_name: &str,
        tool_class: ToolClass,
        path_category: Option<PathCategory>,
        network_egress: bool,
        persistence_mech: bool,
        cleanup_mech: bool,
    ) -> ToolUseRecord {
        ToolUseRecord {
            timestamp_ms: 1_000,
            tool_name: tool_name.to_string(),
            tool_class,
            path_category,
            network_egress,
            persistence_mech,
            cleanup_mech,
            sensitive_file_hint: false,
            rule_hits: vec![],
        }
    }

    #[test]
    fn empty_sequence_returns_no_hits() {
        let seq = ToolUseSequence::default();
        assert!(detect_kill_chains(&seq).is_empty());
    }

    #[test]
    fn recon_exfil_positive_case() {
        // Read .env → WebFetch exfil → IN-SEQ-01
        let mut seq = ToolUseSequence::default();
        seq.record(make_record(
            "Read",
            ToolClass::FileRead,
            Some(PathCategory::SensitiveSecret),
            false,
            false,
            false,
        ));
        seq.record(make_record(
            "WebFetch",
            ToolClass::Network,
            None,
            true, // network_egress
            false,
            false,
        ));
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter().any(|h| h.rule_id == "IN-SEQ-01-RECON-EXFIL"),
            "should detect IN-SEQ-01"
        );
    }

    #[test]
    fn recon_exfil_wrong_order_no_hit() {
        // 先 curl 后 Read → 顺序反了，不触发 IN-SEQ-01
        let mut seq = ToolUseSequence::default();
        seq.record(make_record(
            "WebFetch",
            ToolClass::Network,
            None,
            true,
            false,
            false,
        ));
        seq.record(make_record(
            "Read",
            ToolClass::FileRead,
            Some(PathCategory::SensitiveSecret),
            false,
            false,
            false,
        ));
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits.iter().any(|h| h.rule_id == "IN-SEQ-01-RECON-EXFIL"),
            "wrong order should not trigger IN-SEQ-01"
        );
    }

    #[test]
    fn cleanup_after_attack_positive_case() {
        // Bash curl → Bash rm -rf → IN-SEQ-02
        let mut seq = ToolUseSequence::default();
        seq.record(make_record(
            "Bash",
            ToolClass::Shell,
            None,
            true, // network_egress
            false,
            false,
        ));
        seq.record(make_record(
            "Bash",
            ToolClass::Shell,
            None,
            false,
            false,
            true, // cleanup_mech
        ));
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter()
                .any(|h| h.rule_id == "IN-SEQ-02-CLEANUP-AFTER-ATTACK"),
            "should detect IN-SEQ-02"
        );
    }

    #[test]
    fn persistence_chain_positive_case() {
        // 3 个不同工具各做持久化 → IN-SEQ-03
        let mut seq = ToolUseSequence::default();
        // Bash: crontab
        seq.record(make_record(
            "Bash",
            ToolClass::Shell,
            None,
            false,
            true, // persistence_mech
            false,
        ));
        // Edit: 修改 .bashrc
        seq.record(make_record(
            "Edit",
            ToolClass::FileWrite,
            None,
            false,
            true,
            false,
        ));
        // Write: 写 launchd plist
        seq.record(make_record(
            "Write",
            ToolClass::FileWrite,
            None,
            false,
            true,
            false,
        ));
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter()
                .any(|h| h.rule_id == "IN-SEQ-03-PERSISTENCE-CHAIN"),
            "3 different tools with persistence_mech → IN-SEQ-03"
        );
    }

    #[test]
    fn persistence_chain_same_tool_no_hit() {
        // 3 次 persistence，但全是 Bash → 只有 1 个 tool_name → 不触发 IN-SEQ-03
        let mut seq = ToolUseSequence::default();
        for _ in 0..3 {
            seq.record(make_record(
                "Bash",
                ToolClass::Shell,
                None,
                false,
                true,
                false,
            ));
        }
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits
                .iter()
                .any(|h| h.rule_id == "IN-SEQ-03-PERSISTENCE-CHAIN"),
            "same tool_name repeated should not trigger IN-SEQ-03"
        );
    }

    #[test]
    fn multiple_hits_simultaneously() {
        // 构造同时触发 IN-SEQ-01 + IN-SEQ-02 的序列
        let mut seq = ToolUseSequence::default();
        // Read .ssh → 触发 SEQ-01 前条件
        seq.record(make_record(
            "Read",
            ToolClass::FileRead,
            Some(PathCategory::SensitiveSecret),
            false,
            false,
            false,
        ));
        // Bash curl → 同时满足 SEQ-01 后条件（network_egress）+ SEQ-02 前条件（Shell+network_egress）
        seq.record(make_record(
            "Bash",
            ToolClass::Shell,
            None,
            true, // network_egress
            false,
            false,
        ));
        // Bash rm -rf → SEQ-02 后条件
        seq.record(make_record(
            "Bash",
            ToolClass::Shell,
            None,
            false,
            false,
            true, // cleanup_mech
        ));
        let hits = detect_kill_chains(&seq);
        assert!(hits.iter().any(|h| h.rule_id == "IN-SEQ-01-RECON-EXFIL"));
        assert!(hits
            .iter()
            .any(|h| h.rule_id == "IN-SEQ-02-CLEANUP-AFTER-ATTACK"));
    }
}
