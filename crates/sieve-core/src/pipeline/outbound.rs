//! 出站规则匹配节点（Week 2 起实现）。
//!
//! 关联出站检测 P0 表 + 纯规则引擎。
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
                    source_channel: None,
                    origin_chain_depth: 0,
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
