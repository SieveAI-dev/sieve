# ADR-018: sieve-core 新增 OpenAI Chat Completions 协议适配层

## 状态

**已接受**（v1.5 锁定执行）

> 决策日期：2026-04-28
> 范围：Phase 1 Week 6 实现，`crates/sieve-core/src/protocol/openai.rs`
> 关联 PRD：[v1.5 §6.1、§6.3、§9 第 9 条](../prd/sieve-prd-v2.0.md)
> 取代约束：本 ADR 将 [ADR-004](./ADR-004-anthropic-first-unified-interface.md) 中"UnifiedMessage 接口预留但不实现 OpenAI"的限制正式解除

---

## 背景

### ADR-004 的历史约束

[ADR-004](./ADR-004-anthropic-first-unified-interface.md) 在 v1.4 阶段做出决策：Phase 1 只适配 Anthropic Messages API，UnifiedMessage 接口预留 OpenAI 字段但**不实现**转换逻辑。

这个决策在 v1.4 的单一适配目标（仅 Claude Code）下是合理的——避免过度设计，推迟到真正需要时再做。

### v1.5 触发了实现需要

v1.5 扩展适配三家 AI agent：**Claude Code + OpenClaw + Hermes**。其中：

- **OpenClaw**（Peter Steinberger 的多通道消息网关）：主用 OpenAI Chat Completions 兼容协议，支持 200+ 模型（Claude / GPT-4o / DeepSeek / Gemini / Ollama）。用户把 Sieve 的 `127.0.0.1:11453` 设为所有 provider 的 `base_url` 时，OpenClaw 发来的请求格式是 OpenAI Chat Completions，不是 Anthropic Messages API。
- **Hermes Agent**（Nous Research 的 multi-provider 编排器）：文档明确"任何 OpenAI-compatible LLM provider"，provider config 统一用 `openai_base_url` 字段。

**实际情况**：如果不实现 OpenAI 协议适配层，Sieve 在 OpenClaw / Hermes 用户的两条链路上完全是盲的——它们发来 OpenAI 格式的请求，Sieve 解析失败，要么 panic，要么透传不经过任何检测，是安全漏洞。

### 为什么不分开搞两套 pipeline

另一个候选方案是：Anthropic 链路走 Anthropic pipeline，OpenAI 链路走 OpenAI pipeline，两套完全隔离。

被拒绝原因：
1. **规则引擎重复**：sieve-rules / dispatch / IPC / GUI 全套会在两个 pipeline 里分别维护，代码量翻倍，一人项目不可接受。
2. **规则语义漂移**：IN-CR-05（签名调用 fail-closed）在两套 pipeline 里分别定义，版本管理风险极高。
3. **数据模型不统一**：GUI 弹窗、审计日志、IPC schema 要么面向 UnifiedMessage 一个字段集，要么同时维护两套，工程复杂度远超收益。

### 引入大型依赖的代价

`async-openai`（Rust OpenAI 客户端库）是候选，提供完整 schema 定义。但：
- 它是客户端库，schema 定义和代理解析场景不完全对齐（含大量客户端发送逻辑，不是服务端接收逻辑）
- 拖入 30+ 个传递依赖，增加供应链攻击面（违背 PRD §9 第 6 条精神）
- Sieve 实际只需要 `messages` / `tools` / `stream` 这几个字段的解析——自写 serde 结构体是更轻的选择

---

## 决策

**在 `sieve-core` 新增 `crates/sieve-core/src/protocol/openai.rs` 模块，实现 OpenAI Chat Completions 协议的双向解析，并完成到 UnifiedMessage / UnifiedDelta 的转换。Pipeline / dispatch / sieve-rules / IPC 全部跑在 UnifiedMessage 上，不感知上游是 Anthropic 还是 OpenAI。**

### 核心数据结构

```rust
// crates/sieve-core/src/protocol/openai.rs

/// OpenAI Chat Completions 请求（服务端接收视角，仅含 Sieve 需要的字段）
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub tools: Vec<OpenAITool>,
    #[serde(default)]
    pub stream: bool,
    // 其余字段忽略（temperature、max_tokens 等对 Sieve 无意义）
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,   // "system" | "user" | "assistant" | "tool"
    pub content: Option<serde_json::Value>, // string 或 content part 数组
    #[serde(default)]
    pub tool_calls: Vec<OpenAIToolCall>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

/// OpenAI function-calling tool（即 Anthropic tool_use 的等价物）
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub kind: String,          // 固定 "function"
    pub function: OpenAIFunction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,          // "function"
    pub function: OpenAIToolCallFunction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIToolCallFunction {
    pub name: String,
    pub arguments: String,     // JSON 字符串（注意不是 serde_json::Value）
}

/// OpenAI SSE streaming delta（`data: {...}\n\n` 逐块推送）
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIStreamingChunk {
    pub id: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChoice {
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,  // "stop" | "tool_calls" | null
    pub index: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<OpenAIToolCallDelta>,  // 增量推送，index 字段标识哪个 tool call
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIToolCallDelta {
    pub index: u32,            // 与前一 chunk 对比，累积 arguments
    pub id: Option<String>,    // 只在第一个 delta 出现
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub function: Option<OpenAIToolCallFunctionDelta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIToolCallFunctionDelta {
    pub name: Option<String>,          // 只在第一个 delta 出现
    pub arguments: Option<String>,     // 增量 JSON 片段，需要累积
}
```

### 转换 trait

```rust
impl TryFrom<OpenAIRequest> for UnifiedMessage {
    type Error = ProtocolError;
    fn try_from(req: OpenAIRequest) -> Result<Self, Self::Error> { ... }
}

impl TryFrom<OpenAIStreamingChunk> for UnifiedDelta {
    type Error = ProtocolError;
    fn try_from(chunk: OpenAIStreamingChunk) -> Result<Self, Self::Error> { ... }
}
```

转换是 `TryFrom` 而不是 `From`，因为存在协议不兼容的边界情况需要返回错误（如 `role=tool` 但缺少 `tool_call_id`）。

### 协议检测

Sieve 在监听端口收到请求时，通过 HTTP path 区分协议：
- `/v1/messages` → Anthropic Messages API（已有 `anthropic.rs` 处理）
- `/v1/chat/completions` → OpenAI Chat Completions（本 ADR 新增）
- 其他 path → 404 + 记录未知协议日志

不依赖 User-Agent header 检测，因为 OpenClaw / Hermes 可能不设置或设置不一致。

---

## SSE 格式差异详解

两种协议的 SSE 格式有结构性差异，SSE Parser 需要同时支持：

### Anthropic SSE 格式

```
event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_stop
data: {"type":"message_stop"}
```

工具调用（Anthropic）：
```
event: content_block_start
data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_01","name":"eth_signTransaction","input":{}}}

event: content_block_delta
data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"to\":"}}
```

### OpenAI SSE 格式

```
data: {"id":"chatcmpl-xxx","choices":[{"delta":{"role":"assistant","content":""},"index":0}]}

data: {"id":"chatcmpl-xxx","choices":[{"delta":{"content":"Hello"},"index":0}]}

data: {"id":"chatcmpl-xxx","choices":[{"delta":{},"finish_reason":"stop","index":0}]}

data: [DONE]
```

工具调用（OpenAI）—— 关键差异：arguments 是增量字符串片段，需要逐块累积：
```
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_xxx","type":"function","function":{"name":"eth_signTransaction","arguments":""}}]},"index":0}]}

data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"to\":"}}]},"index":0}]}

data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"0xDeadBeef\",\"value\":\"1000000000000000000\"}"}}]},"index":0,"finish_reason":"tool_calls"}]}

data: [DONE]
```

### 关键差异汇总

| 维度 | Anthropic | OpenAI |
|------|-----------|--------|
| event 行 | 有（`event: xxx`） | 无 |
| 结束信号 | `message_stop` event | `data: [DONE]` 字面量 |
| 工具调用表示 | `content[].type=tool_use` + `input_json_delta` | `choices[].delta.tool_calls[]` + `arguments` 字符串拼接 |
| 多 content block | 独立 `content_block_start/stop` | 统一在 `choices[]` 里，无显式边界 |
| finish 信号 | `finish_reason` 在 `message_delta` event | `finish_reason` 在最后一个非 DONE chunk |

SSE Parser 在 `crates/sieve-core/src/sse/` 中按协议类型分支处理，公共的字节流解析（chunk 分割、`\n\n` 边界）由底层通用逻辑处理，格式解析由 `anthropic.rs` / `openai.rs` 各自负责。

---

## 影响

### 正面影响

1. **引擎 100% 复用**：sieve-rules / dispatch / IPC / GUI 全部跑在 UnifiedMessage 上，OpenAI 协议适配层是一个薄转换层，不动检测逻辑；
2. **三家 agent 统一覆盖**：OpenClaw / Hermes 的流量不再裸奔，IN-CR-01 ~ IN-CR-06 + IN-GEN-01 ~ IN-GEN-06 对它们生效；
3. **审计日志统一**：UnifiedMessage 统一落 SQLite，Anthropic / OpenAI 流量在 audit log 里格式一致；
4. **无大型新依赖**：自写 serde 结构体 + TryFrom 转换，不引入 async-openai 或 openai-api-rust 等库；
5. **未来扩展路径清晰**：Phase 2 加 Gemini / Mistral 时只需新增 `gemini.rs` / `mistral.rs`，同样实现 TryFrom<> 接口即可。

### 负面影响

1. **OpenAI tool call 增量累积状态**：OpenAI SSE 的 `arguments` 是字符串碎片，需要在 `ToolCallAggregator` 里额外维护累积 buffer（Anthropic 的 `input_json_delta` 也需要，但语义更清晰）；需要对应 fuzz 测试覆盖（PRD §9 第 5 条）；
2. **`[DONE]` 字面量处理**：`data: [DONE]` 是非 JSON 内容，需要在 SSE 解析器里显式识别跳过，不能走 `serde_json::from_str`；
3. **协议检测点的路径依赖**：通过 HTTP path 检测协议，要求 OpenClaw / Hermes 调用时确实走标准 path（`/v1/chat/completions`）；如果某家改了 path，检测失败。观察阶段 Week 7 集成测试时验证。

### 约束变化（对 ADR-004 的修正）

ADR-004 §决策 中"UnifiedMessage 接口预留但不实现 OpenAI / Gemini / OpenRouter"的约束，在 **v1.5 Phase 1** 范围内改为：

- **OpenAI Chat Completions**：本 ADR 落地，**不再是预留，是真实实现**
- **Gemini / Mistral / OpenRouter / 本地模型（Ollama）**：维持推迟到 Phase 2

ADR-004 文件本身状态更新为"部分取代（OpenAI 部分）by ADR-018"，原文不删改（遵循 ADR 只增不改原则）。

### 需要更新的文档

- [ADR-004](./ADR-004-anthropic-first-unified-interface.md) — 文件末尾增补"v1.5 后续"段，标注 ADR-018 部分取代
- [architecture.md](./architecture.md) — Protocol Layer 图增加 `openai.rs` 节点（由 G4 子代理执行）
- [data-model.md](./data-model.md) — `OpenAIRequest` / `OpenAIDelta` 结构体示例
- `docs/guides/development.md` — Week 6 新增模块编译说明
- `CHANGELOG.md` — `[BREAKING]` 不涉及；记录协议层扩展为行为变更

---

## 相关文档

- [PRD v2.0 §6.1 整体架构](../prd/sieve-prd-v2.0.md) — Protocol Layer 重画，openai.rs 节点
- [PRD v2.0 §6.3 Rust 技术栈](../prd/sieve-prd-v2.0.md) — 新增依赖：自写 serde（避免引入 async-openai）
- [PRD v2.0 §9 第 9 条](../prd/sieve-prd-v2.0.md) — 硬约束重写：三家 agent + UnifiedMessage 统一
- [ADR-004](./ADR-004-anthropic-first-unified-interface.md) — 被本 ADR 部分取代（OpenAI 部分）
- [ADR-001](./ADR-001-rust-tech-stack.md) — Rust 技术栈决策，本 ADR 在其基础上扩展
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) — fail-closed 约束，OpenAI 协议适配后同样生效
- [ADR-019](./ADR-019-x-sieve-origin-header.md) — X-Sieve-Origin header 协议（配合本 ADR 解决嵌套调用问题）
