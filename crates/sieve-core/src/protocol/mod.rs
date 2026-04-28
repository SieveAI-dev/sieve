//! 协议层（Anthropic Messages API + OpenAI Chat Completions + UnifiedMessage）。
//!
//! - [`anthropic`]：Anthropic Messages API schema（Phase 1，ADR-004）
//! - [`openai`]：OpenAI Chat Completions schema（Phase 1 Week 6，ADR-018）
//! - [`unified_message`]：Sieve 内部统一消息表示

pub mod anthropic;
pub mod openai;
pub mod unified_message;

pub use openai::{
    OpenAIDelta, OpenAIFunctionCall, OpenAIFunctionCallDelta, OpenAIFunctionDef, OpenAIMessage,
    OpenAIRequest, OpenAIStreamingChunk, OpenAITool, OpenAIToolCall, OpenAIToolCallDelta,
};
