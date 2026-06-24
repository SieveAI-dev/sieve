//! 行为序列 kill chain 检测（PRD v2.0 §5.7.2）。
//!
//! 8 条规则全部 severity = High，**仅 StatusBar 通知**（PRD §9 #15 不引入 Block 路径）：
//!
//! - IN-SEQ-01-RECON-EXFIL: FileRead+SensitiveSecret 之后 network_egress
//! - IN-SEQ-02-CLEANUP-AFTER-ATTACK: Shell+network_egress 之后 cleanup_mech
//! - IN-SEQ-03-PERSISTENCE-CHAIN: 3 次 persistence_mech=true 跨不同 tool_name 调用
//! - IN-SEQ-04-ARCHIVE-EXFIL: secret 之后打包再外发（ChecksumConfirmed 高置信短路跳过打包步）
//! - IN-SEQ-05-ENCODE-EXFIL: secret 之后编码/加密再外发或入剪贴板（双条件防 FP）
//! - IN-SEQ-06-CROSS-AGENT-SECRET: 读 secret 者 actor 与外发者 actor 不同（跨 agent 拆分）
//! - IN-SEQ-07-CLIPBOARD-SECRET: secret 之后写入剪贴板
//! - IN-SEQ-08-PUBLIC-ARTIFACT: secret 之后写入/发布到公共产物路径
//!
//! 升级为 Block 路径需满足 ADR-022 三条件（4 周 ≥50 样本 + FP<0.5% + 新 ADR）。

use super::feature::{PathCategory, SecretConfidence, ToolClass};
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
    if let Some(h) = detect_archive_exfil(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_encode_exfil(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_cross_agent_secret(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_clipboard_secret(&records) {
        hits.push(h);
    }
    if let Some(h) = detect_public_artifact_flow(&records) {
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

/// IN-SEQ-04: secret_confidence≥Heuristic 之后打包（archive_mech）再外发（network_egress）。
///
/// `ChecksumConfirmed` 高置信 secret 短路：跳过中间打包步，secret 之后直接外发即触发。
/// 意图：捕获「读敏感凭证 → 打包中转 → 上传」链（中间打包步会打断 IN-SEQ-01 的直接关联）。
/// 打包与外发可落在同一 record（如 `tar czf - … | curl`）。
fn detect_archive_exfil(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    // 高置信短路：ChecksumConfirmed secret 之后出现 network_egress（打包步可缺省）
    if let Some(secret_idx) = records
        .iter()
        .position(|r| r.secret_confidence == SecretConfidence::ChecksumConfirmed)
    {
        if let Some(exfil_idx) = records
            .iter()
            .enumerate()
            .skip(secret_idx + 1)
            .find(|(_, r)| r.network_egress)
            .map(|(i, _)| i)
        {
            return Some(SequenceHit {
                rule_id: "IN-SEQ-04-ARCHIVE-EXFIL".into(),
                description: "读高置信 secret 后外发（kill chain，checksum 已确认）".into(),
                triggering_indices: vec![secret_idx, exfil_idx],
            });
        }
    }
    // Heuristic 完整链：secret → archive_mech → network_egress
    let secret_idx = records
        .iter()
        .position(|r| r.secret_confidence >= SecretConfidence::Heuristic)?;
    let archive_idx = records
        .iter()
        .enumerate()
        .skip(secret_idx + 1)
        .find(|(_, r)| r.archive_mech)
        .map(|(i, _)| i)?;
    let exfil_idx = records
        .iter()
        .enumerate()
        .skip(archive_idx) // 含 archive record 自身（打包与外发同步）
        .find(|(_, r)| r.network_egress)
        .map(|(i, _)| i)?;
    let mut indices = vec![secret_idx, archive_idx, exfil_idx];
    indices.dedup();
    Some(SequenceHit {
        rule_id: "IN-SEQ-04-ARCHIVE-EXFIL".into(),
        description: "读 secret 后打包外发（kill chain）".into(),
        triggering_indices: indices,
    })
}

/// IN-SEQ-05: secret_confidence≥Heuristic 之后编码/加密（encode_mech）再外发或入剪贴板。
///
/// 双条件（secret + encode）防误报：单独编码（base64 等日常操作）不触发。
/// 编码与外发可落在同一 record（如 `base64 .env | curl --data-binary @- …`）。
fn detect_encode_exfil(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    let secret_idx = records
        .iter()
        .position(|r| r.secret_confidence >= SecretConfidence::Heuristic)?;
    let encode_idx = records
        .iter()
        .enumerate()
        .skip(secret_idx + 1)
        .find(|(_, r)| r.encode_mech)
        .map(|(i, _)| i)?;
    let exfil_idx = records
        .iter()
        .enumerate()
        .skip(encode_idx) // 含 encode record 自身（编码与外发同步）
        .find(|(_, r)| r.network_egress || r.clipboard_mech)
        .map(|(i, _)| i)?;
    let mut indices = vec![secret_idx, encode_idx, exfil_idx];
    indices.dedup();
    Some(SequenceHit {
        rule_id: "IN-SEQ-05-ENCODE-EXFIL".into(),
        description: "读 secret 后编码/加密再外发（kill chain）".into(),
        triggering_indices: indices,
    })
}

/// IN-SEQ-06: 读 secret 的 record（actor=A）之后，外发 record（actor=B）且 B≠A。
///
/// 跨 agent 拆分外泄：单 agent 视角各步合法。actor 缺失时保守不触发（防单 agent 误报）。
fn detect_cross_agent_secret(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    for (secret_idx, sr) in records.iter().enumerate() {
        if sr.secret_confidence < SecretConfidence::Heuristic {
            continue;
        }
        let actor_a = match &sr.actor {
            Some(a) => a,
            None => continue,
        };
        if let Some((exfil_idx, _)) = records
            .iter()
            .enumerate()
            .skip(secret_idx + 1)
            .find(|(_, r)| r.network_egress && matches!(&r.actor, Some(b) if b != actor_a))
        {
            return Some(SequenceHit {
                rule_id: "IN-SEQ-06-CROSS-AGENT-SECRET".into(),
                description: "跨 agent 拆分外泄（读 secret 者 ≠ 外发者，kill chain）".into(),
                triggering_indices: vec![secret_idx, exfil_idx],
            });
        }
    }
    None
}

/// IN-SEQ-07: secret_confidence≥Heuristic 之后写入剪贴板（clipboard_mech）。
///
/// secret 进剪贴板后脱离流量视野，是隐蔽外泄通道（只能告警进入剪贴板，无法跟踪后续）。
/// secret 与 clipboard 可落在同一 record（如 `pbcopy < mnemonic.txt`）。
fn detect_clipboard_secret(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    let secret_idx = records
        .iter()
        .position(|r| r.secret_confidence >= SecretConfidence::Heuristic)?;
    let clip_idx = records
        .iter()
        .enumerate()
        .skip(secret_idx) // 含 secret record 自身（读取与复制同步）
        .find(|(_, r)| r.clipboard_mech)
        .map(|(i, _)| i)?;
    let mut indices = vec![secret_idx, clip_idx];
    indices.dedup();
    Some(SequenceHit {
        rule_id: "IN-SEQ-07-CLIPBOARD-SECRET".into(),
        description: "读 secret 后写入剪贴板（kill chain）".into(),
        triggering_indices: indices,
    })
}

/// IN-SEQ-08: secret_confidence≥Heuristic 之后写入/发布到公共产物路径（public_artifact_target）。
///
/// secret 被写进会公开发布的 build 产物或直接发布（npm publish / gh release 等）。
fn detect_public_artifact_flow(records: &[&ToolUseRecord]) -> Option<SequenceHit> {
    let secret_idx = records
        .iter()
        .position(|r| r.secret_confidence >= SecretConfidence::Heuristic)?;
    let artifact_idx = records
        .iter()
        .enumerate()
        .skip(secret_idx + 1)
        .find(|(_, r)| r.public_artifact_target)
        .map(|(i, _)| i)?;
    Some(SequenceHit {
        rule_id: "IN-SEQ-08-PUBLIC-ARTIFACT".into(),
        description: "读 secret 后写入/发布公共产物（kill chain）".into(),
        triggering_indices: vec![secret_idx, artifact_idx],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence::feature::{PathCategory, SecretConfidence, ToolClass};
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
            ..Default::default()
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

    // ---- IN-SEQ-04~08 出站 exfil 链家族 ----

    #[test]
    fn in_seq_04_archive_exfil_positive() {
        // Read(secret Heuristic) → tar(archive) → curl(network)
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            path_category: Some(PathCategory::SensitiveSecret),
            secret_confidence: SecretConfidence::Heuristic,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            archive_mech: true,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 3_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter().any(|h| h.rule_id == "IN-SEQ-04-ARCHIVE-EXFIL"),
            "secret→archive→network should trigger IN-SEQ-04"
        );
    }

    #[test]
    fn in_seq_04_checksum_shortcut_no_archive() {
        // ChecksumConfirmed secret → curl（无 archive 中间步）→ 高置信短路触发
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::ChecksumConfirmed,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter().any(|h| h.rule_id == "IN-SEQ-04-ARCHIVE-EXFIL"),
            "ChecksumConfirmed secret→network should short-circuit IN-SEQ-04"
        );
    }

    #[test]
    fn in_seq_04_no_secret_no_hit() {
        // tar + curl 但无 secret 上下文 → 不触发（防 FP）
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::Shell,
            archive_mech: true,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits.iter().any(|h| h.rule_id == "IN-SEQ-04-ARCHIVE-EXFIL"),
            "no secret context should not trigger IN-SEQ-04"
        );
    }

    #[test]
    fn in_seq_05_encode_exfil_combined_record() {
        // Read(secret) → base64|curl（encode + network 同一 record）
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            encode_mech: true,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter().any(|h| h.rule_id == "IN-SEQ-05-ENCODE-EXFIL"),
            "secret→(encode+network) should trigger IN-SEQ-05"
        );
    }

    #[test]
    fn in_seq_05_encode_alone_no_hit() {
        // 单独 base64（无 secret）→ 不触发（双条件防 FP）
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::Shell,
            encode_mech: true,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits.iter().any(|h| h.rule_id == "IN-SEQ-05-ENCODE-EXFIL"),
            "encode without secret should not trigger IN-SEQ-05"
        );
    }

    #[test]
    fn in_seq_06_cross_agent_positive() {
        // actor=A 读 secret → actor=B 外发 → 触发
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            actor: Some("agent-a".to_string()),
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            actor: Some("agent-b".to_string()),
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter()
                .any(|h| h.rule_id == "IN-SEQ-06-CROSS-AGENT-SECRET"),
            "A reads secret, B exfils → IN-SEQ-06"
        );
    }

    #[test]
    fn in_seq_06_actor_missing_no_hit() {
        // actor 缺失 → 保守不触发
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits
                .iter()
                .any(|h| h.rule_id == "IN-SEQ-06-CROSS-AGENT-SECRET"),
            "missing actor should not trigger IN-SEQ-06"
        );
    }

    #[test]
    fn in_seq_06_same_actor_no_hit() {
        // 同 actor → 不触发（非跨 agent）
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            actor: Some("agent-a".to_string()),
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            network_egress: true,
            actor: Some("agent-a".to_string()),
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            !hits
                .iter()
                .any(|h| h.rule_id == "IN-SEQ-06-CROSS-AGENT-SECRET"),
            "same actor should not trigger IN-SEQ-06"
        );
    }

    #[test]
    fn in_seq_07_clipboard_positive() {
        // Read(secret) → pbcopy
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::Shell,
            clipboard_mech: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter()
                .any(|h| h.rule_id == "IN-SEQ-07-CLIPBOARD-SECRET"),
            "secret→clipboard should trigger IN-SEQ-07"
        );
    }

    #[test]
    fn in_seq_08_public_artifact_positive() {
        // Read(secret) → Write(dist/ public artifact)
        let mut seq = ToolUseSequence::default();
        seq.record(ToolUseRecord {
            timestamp_ms: 1_000,
            tool_class: ToolClass::FileRead,
            secret_confidence: SecretConfidence::Heuristic,
            ..Default::default()
        });
        seq.record(ToolUseRecord {
            timestamp_ms: 2_000,
            tool_class: ToolClass::FileWrite,
            public_artifact_target: true,
            ..Default::default()
        });
        let hits = detect_kill_chains(&seq);
        assert!(
            hits.iter()
                .any(|h| h.rule_id == "IN-SEQ-08-PUBLIC-ARTIFACT"),
            "secret→public artifact should trigger IN-SEQ-08"
        );
    }
}
