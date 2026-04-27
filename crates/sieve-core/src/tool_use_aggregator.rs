//! Tool Use Aggregator：跨多个 SSE event 累积 partial_json，complete block_stop 后 deserialize。
//!
//! 关联 PRD §6.2 Pipeline 节点 ⑦（入站流式检测）。

use crate::sse::parser::{SseDelta, SseEvent};
use std::collections::HashMap;

/// 聚合完成的工具调用（content_block_stop 时产出）。
#[derive(Debug, Clone)]
pub struct CompletedToolCall {
    /// 工具调用 ID（toolu_xxx）。
    pub id: String,
    /// 工具名。
    pub name: String,
    /// 已完整解析的参数 JSON。
    pub input: serde_json::Value,
}

/// 内部块状态。
#[derive(Debug, Clone)]
enum BlockState {
    /// 文本块。
    Text {
        /// 已累积文本（暂不使用，预留 Week 4 扩展）。
        buf: String,
    },
    /// 工具调用块。
    ToolUse {
        /// 工具调用 ID。
        id: String,
        /// 工具名。
        name: String,
        /// 累积的 partial_json 片段。
        partial_json: String,
    },
}

/// Tool Use 跨 chunk 聚合器。
///
/// 典型用法：
/// ```rust
/// use sieve_core::tool_use_aggregator::Aggregator;
/// use sieve_core::sse::parser::{SseEvent, SseDelta};
///
/// let mut agg = Aggregator::new();
/// // 处理 SSE events...
/// ```
pub struct Aggregator {
    blocks: HashMap<u32, BlockState>,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Aggregator {
    /// 新建聚合器。
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    /// 处理一个 SseEvent，content_block_stop 时可能返回 CompletedToolCall。
    ///
    /// 其余 event 返回 `None`。
    pub fn process(&mut self, event: &SseEvent) -> Option<CompletedToolCall> {
        match event {
            SseEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                let block_type = content_block
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if block_type == "tool_use" {
                    let id = content_block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = content_block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    self.blocks.insert(
                        *index,
                        BlockState::ToolUse {
                            id,
                            name,
                            partial_json: String::new(),
                        },
                    );
                } else if block_type == "text" {
                    self.blocks
                        .insert(*index, BlockState::Text { buf: String::new() });
                }
                None
            }
            SseEvent::ContentBlockDelta { index, delta } => {
                if let Some(block) = self.blocks.get_mut(index) {
                    match (block, delta) {
                        (BlockState::Text { buf }, SseDelta::TextDelta { text }) => {
                            buf.push_str(text);
                        }
                        (
                            BlockState::ToolUse { partial_json, .. },
                            SseDelta::InputJsonDelta {
                                partial_json: incoming,
                            },
                        ) => {
                            partial_json.push_str(incoming);
                        }
                        _ => {}
                    }
                }
                None
            }
            SseEvent::ContentBlockStop { index } => {
                if let Some(BlockState::ToolUse {
                    id,
                    name,
                    partial_json,
                }) = self.blocks.remove(index)
                {
                    match serde_json::from_str::<serde_json::Value>(&partial_json) {
                        Ok(input) => Some(CompletedToolCall { id, name, input }),
                        Err(e) => {
                            tracing::warn!(
                                tool_id = %id,
                                error = %e,
                                "tool_use partial_json parse failed"
                            );
                            None
                        }
                    }
                } else {
                    self.blocks.remove(index);
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sse::parser::{SseDelta, SseEvent};

    #[test]
    fn aggregate_tool_use_split_partial_json() {
        let mut a = Aggregator::new();
        let start = SseEvent::ContentBlockStart {
            index: 1,
            content_block: serde_json::json!({"type":"tool_use","id":"toolu_x","name":"get_weather","input":{}}),
        };
        a.process(&start);
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"city": "San "#.into(),
            },
        });
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"Francisco"}"#.into(),
            },
        });
        let result = a.process(&SseEvent::ContentBlockStop { index: 1 });
        let tool = result.expect("should complete");
        assert_eq!(tool.id, "toolu_x");
        assert_eq!(tool.name, "get_weather");
        assert_eq!(
            tool.input.get("city").and_then(|v| v.as_str()),
            Some("San Francisco")
        );
    }

    #[test]
    fn aggregate_text_block_no_completion() {
        let mut a = Aggregator::new();
        a.process(&SseEvent::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type":"text","text":""}),
        });
        a.process(&SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta { text: "hi".into() },
        });
        let result = a.process(&SseEvent::ContentBlockStop { index: 0 });
        assert!(
            result.is_none(),
            "text block should not produce CompletedToolCall"
        );
    }

    #[test]
    fn malformed_partial_json_returns_none() {
        let mut a = Aggregator::new();
        a.process(&SseEvent::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type":"tool_use","id":"x","name":"y"}),
        });
        a.process(&SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: "{not json".into(),
            },
        });
        let result = a.process(&SseEvent::ContentBlockStop { index: 0 });
        assert!(result.is_none());
    }

    #[test]
    fn unknown_block_type_stop_returns_none() {
        let mut a = Aggregator::new();
        // 未注册的 index
        let result = a.process(&SseEvent::ContentBlockStop { index: 99 });
        assert!(result.is_none());
    }

    #[test]
    fn multiple_blocks_independent() {
        let mut a = Aggregator::new();
        // 两个并行块
        a.process(&SseEvent::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type":"text","text":""}),
        });
        a.process(&SseEvent::ContentBlockStart {
            index: 1,
            content_block: serde_json::json!({"type":"tool_use","id":"toolu_y","name":"foo"}),
        });
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"k":1}"#.into(),
            },
        });
        let r0 = a.process(&SseEvent::ContentBlockStop { index: 0 });
        assert!(r0.is_none());
        let r1 = a.process(&SseEvent::ContentBlockStop { index: 1 });
        assert!(r1.is_some());
        assert_eq!(r1.unwrap().name, "foo");
    }
}
