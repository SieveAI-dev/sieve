//! 协议层（Anthropic Messages API + OpenAI Chat Completions + UnifiedMessage）。
//!
//! - [`anthropic`]：Anthropic Messages API schema（Phase 1）
//! - [`openai`]：OpenAI Chat Completions schema（Phase 1 Week 6）
//! - [`unified_message`]：Sieve 内部统一消息表示

pub mod anthropic;
pub mod codec;
pub mod openai;
pub mod unified_message;

pub use codec::{AnthropicCodec, DecodedRequest, OpenAiCodec, ProviderCodec};
pub use openai::{
    OpenAIDelta, OpenAIFunctionCall, OpenAIFunctionCallDelta, OpenAIFunctionDef, OpenAIMessage,
    OpenAIRequest, OpenAIStreamingChunk, OpenAITool, OpenAIToolCall, OpenAIToolCallDelta,
};
