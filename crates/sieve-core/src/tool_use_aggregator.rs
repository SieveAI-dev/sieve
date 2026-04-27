//! Tool Use Aggregator：跨多个 SSE event 累积 partial_json，complete block_stop 后 deserialize。
//!
//! 关联 PRD §6.2 Pipeline 节点 ⑦（入站流式检测）。
//!
//! P0-5 容量上限：blocks 数量、partial_json 大小、text buffer 大小均有上限，防止恶意上游 OOM。

use crate::sse::parser::{SseDelta, SseEvent};
use std::collections::HashMap;

/// 同时允许打开的最大 tool_use/text 块数量（P0-5 / IN-CAP-02）。
pub const MAX_OPEN_BLOCKS: usize = 32;

/// 单个 tool_use 块 partial_json 累积上限（P0-5 / IN-CAP-02，1 MiB）。
pub const MAX_TOOL_JSON_BYTES: usize = 1 << 20;

/// 单个 text 块 buffer 累积上限（P0-5 / IN-CAP-02，1 MiB）。
pub const MAX_TEXT_BUFFER_BYTES: usize = 1 << 20;

/// Aggregator 可能返回的结构化错误（P0-5 容量上限 + 预留 P0-6 malformed JSON）。
#[derive(Debug, Clone, PartialEq)]
pub enum AggregatorError {
    /// 同时打开的块数量超过 [`MAX_OPEN_BLOCKS`]。
    ///
    /// 检测 ID：IN-CAP-02。
    TooManyOpenBlocks {
        /// 当前块数量。
        count: usize,
        /// 配置的上限。
        max: usize,
    },
    /// 单个 tool_use 块 partial_json 超过 [`MAX_TOOL_JSON_BYTES`]。
    ///
    /// 检测 ID：IN-CAP-02。
    PartialJsonTooLarge {
        /// 当前累积字节数。
        len: usize,
        /// 配置的上限。
        max: usize,
    },
    /// 单个 text 块 buffer 超过 [`MAX_TEXT_BUFFER_BYTES`]。
    ///
    /// 检测 ID：IN-CAP-02。
    TextBufferTooLarge {
        /// 当前累积字节数。
        len: usize,
        /// 配置的上限。
        max: usize,
    },
    /// tool_use partial_json 解析失败（预留 P0-6 MalformedToolUse 路径，暂未实装）。
    ///
    /// P0-6 任务激活后将切换到 fail-closed 行为。
    MalformedToolUse {
        /// 工具调用 ID。
        tool_id: String,
        /// 解析错误描述。
        error: String,
    },
}

impl std::fmt::Display for AggregatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregatorError::TooManyOpenBlocks { count, max } => {
                write!(f, "IN-CAP-02: 打开的块数量超限 ({count} > {max})")
            }
            AggregatorError::PartialJsonTooLarge { len, max } => {
                write!(f, "IN-CAP-02: partial_json 超限 ({len} > {max} bytes)")
            }
            AggregatorError::TextBufferTooLarge { len, max } => {
                write!(f, "IN-CAP-02: text buffer 超限 ({len} > {max} bytes)")
            }
            AggregatorError::MalformedToolUse { tool_id, error } => {
                write!(f, "tool_use {tool_id} partial_json 解析失败: {error}")
            }
        }
    }
}

impl std::error::Error for AggregatorError {}

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
    /// 其余 event 返回 `Ok(None)`。
    ///
    /// # Errors
    /// 容量上限触发时返回 [`AggregatorError`]。调用方应将容量错误视为 fail-closed Critical
    ///（IN-CAP-02），注入 sieve_blocked 并截断流。
    ///
    /// 注：MalformedToolUse 变体已声明，P0-6 任务激活后切换为 fail-closed 路径。
    pub fn process(
        &mut self,
        event: &SseEvent,
    ) -> Result<Option<CompletedToolCall>, AggregatorError> {
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
                    // P0-5：创建新 block 前检查数量上限
                    if self.blocks.len() >= MAX_OPEN_BLOCKS {
                        return Err(AggregatorError::TooManyOpenBlocks {
                            count: self.blocks.len(),
                            max: MAX_OPEN_BLOCKS,
                        });
                    }
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
                    // P0-5：创建新 block 前检查数量上限
                    if self.blocks.len() >= MAX_OPEN_BLOCKS {
                        return Err(AggregatorError::TooManyOpenBlocks {
                            count: self.blocks.len(),
                            max: MAX_OPEN_BLOCKS,
                        });
                    }
                    self.blocks
                        .insert(*index, BlockState::Text { buf: String::new() });
                }
                Ok(None)
            }
            SseEvent::ContentBlockDelta { index, delta } => {
                if let Some(block) = self.blocks.get_mut(index) {
                    match (block, delta) {
                        (BlockState::Text { buf }, SseDelta::TextDelta { text }) => {
                            buf.push_str(text);
                            // P0-5：text buffer 大小检查
                            if buf.len() > MAX_TEXT_BUFFER_BYTES {
                                return Err(AggregatorError::TextBufferTooLarge {
                                    len: buf.len(),
                                    max: MAX_TEXT_BUFFER_BYTES,
                                });
                            }
                        }
                        (
                            BlockState::ToolUse { partial_json, .. },
                            SseDelta::InputJsonDelta {
                                partial_json: incoming,
                            },
                        ) => {
                            partial_json.push_str(incoming);
                            // P0-5：partial_json 大小检查
                            if partial_json.len() > MAX_TOOL_JSON_BYTES {
                                return Err(AggregatorError::PartialJsonTooLarge {
                                    len: partial_json.len(),
                                    max: MAX_TOOL_JSON_BYTES,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                Ok(None)
            }
            SseEvent::ContentBlockStop { index } => {
                if let Some(BlockState::ToolUse {
                    id,
                    name,
                    partial_json,
                }) = self.blocks.remove(index)
                {
                    match serde_json::from_str::<serde_json::Value>(&partial_json) {
                        Ok(input) => Ok(Some(CompletedToolCall { id, name, input })),
                        Err(e) => {
                            // P0-6 预留：MalformedToolUse 变体已声明，本次仍 warn+None
                            // P0-6 任务激活后改为 Err(AggregatorError::MalformedToolUse)
                            tracing::warn!(
                                tool_id = %id,
                                error = %e,
                                "tool_use partial_json parse failed"
                            );
                            Ok(None)
                        }
                    }
                } else {
                    self.blocks.remove(index);
                    Ok(None)
                }
            }
            _ => Ok(None),
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
        a.process(&start).unwrap();
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"city": "San "#.into(),
            },
        })
        .unwrap();
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"Francisco"}"#.into(),
            },
        })
        .unwrap();
        let result = a.process(&SseEvent::ContentBlockStop { index: 1 }).unwrap();
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
        })
        .unwrap();
        a.process(&SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta { text: "hi".into() },
        })
        .unwrap();
        let result = a.process(&SseEvent::ContentBlockStop { index: 0 }).unwrap();
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
        })
        .unwrap();
        a.process(&SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: "{not json".into(),
            },
        })
        .unwrap();
        let result = a.process(&SseEvent::ContentBlockStop { index: 0 }).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn unknown_block_type_stop_returns_none() {
        let mut a = Aggregator::new();
        // 未注册的 index
        let result = a
            .process(&SseEvent::ContentBlockStop { index: 99 })
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn multiple_blocks_independent() {
        let mut a = Aggregator::new();
        // 两个并行块
        a.process(&SseEvent::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type":"text","text":""}),
        })
        .unwrap();
        a.process(&SseEvent::ContentBlockStart {
            index: 1,
            content_block: serde_json::json!({"type":"tool_use","id":"toolu_y","name":"foo"}),
        })
        .unwrap();
        a.process(&SseEvent::ContentBlockDelta {
            index: 1,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"k":1}"#.into(),
            },
        })
        .unwrap();
        let r0 = a.process(&SseEvent::ContentBlockStop { index: 0 }).unwrap();
        assert!(r0.is_none());
        let r1 = a.process(&SseEvent::ContentBlockStop { index: 1 }).unwrap();
        assert!(r1.is_some());
        assert_eq!(r1.unwrap().name, "foo");
    }

    // P0-5: Aggregator 容量上限测试

    #[test]
    fn partial_json_over_limit_returns_error() {
        let mut a = Aggregator::new();
        a.process(&SseEvent::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type":"tool_use","id":"t","name":"f"}),
        })
        .unwrap();
        // 构造超过 1 MiB 的 partial_json（一次性发送 MAX_TOOL_JSON_BYTES + 1 字节）
        let big = "x".repeat(MAX_TOOL_JSON_BYTES + 1);
        let result = a.process(&SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta { partial_json: big },
        });
        assert!(
            matches!(
                result,
                Err(AggregatorError::PartialJsonTooLarge { len, max })
                    if len > MAX_TOOL_JSON_BYTES && max == MAX_TOOL_JSON_BYTES
            ),
            "expected PartialJsonTooLarge, got: {:?}",
            result
        );
    }

    #[test]
    fn too_many_open_blocks_returns_error() {
        let mut a = Aggregator::new();
        // 填满 MAX_OPEN_BLOCKS 个 tool_use 块
        for i in 0..MAX_OPEN_BLOCKS as u32 {
            a.process(&SseEvent::ContentBlockStart {
                index: i,
                content_block: serde_json::json!({"type":"tool_use","id":format!("t{i}"),"name":"f"}),
            })
            .unwrap();
        }
        // 第 33 个块应触发上限
        let result = a.process(&SseEvent::ContentBlockStart {
            index: MAX_OPEN_BLOCKS as u32,
            content_block: serde_json::json!({"type":"tool_use","id":"overflow","name":"f"}),
        });
        assert!(
            matches!(
                result,
                Err(AggregatorError::TooManyOpenBlocks { count, max })
                    if count >= MAX_OPEN_BLOCKS && max == MAX_OPEN_BLOCKS
            ),
            "expected TooManyOpenBlocks, got: {:?}",
            result
        );
    }
}
