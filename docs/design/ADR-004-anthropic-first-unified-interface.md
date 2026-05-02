# ADR-004: Phase 1 只适配 Anthropic Messages API，UnifiedMessage 接口预留

## 状态

**已接受**（v1.4 锁定执行）

> 决策日期：2026-04-26
> 范围：Phase 1（12 周 GA），仅 Claude Code 客户端 + Anthropic Messages API 协议
> 关联 PRD：[v1.4 §6.1、§9.9](../prd/_archive/sieve-prd-v1.5.md)

---

## 背景

Sieve 是 LLM 流量层代理。理论上"完整版"应该支持：

- **客户端**：Claude Code、OpenAI Codex / GPT 系列 SDK、Cursor、Aider、自写 agent、OpenClaw、Hermes、各种 framework；
- **协议**：Anthropic Messages API、OpenAI Chat Completions、OpenRouter（多模型路由）、Google Gemini、本地 Ollama；
- **传输**：HTTP/JSON、SSE、WebSocket、gRPC（一些企业 SDK）。

如果按"完整版"设计 Phase 1，会陷入两个陷阱：

1. **协议抽象过度**：为支持 N 家协议，要做 LCD（最小公倍数）抽象，最终既不能很好支持 Anthropic 也不能很好支持 OpenAI；
2. **测试覆盖不可能**：12 周时间、单人开发，每多一家协议都需要至少 2 周（适配 + 边界 fuzz + dogfood）。

更关键的是：**P0 用户群的 80% 是 Claude Code 重度用户**（PRD §3.1）。Anthropic Messages API 加 Claude Code 加 SSE，已经覆盖了 P0 用户的全部使用场景。

但同时不能完全把架构写死成"只能跑 Anthropic"——如果 Phase 2 要扩到 OpenAI / OpenRouter，整套检测器、配置、审计逻辑应当复用，不该重写。

这是一个典型的"**抽象多了浪费 / 抽象少了重写**"的权衡。

PRD §9.9 的硬约束给出了答案：**"Phase 1 只做 Claude Code，UnifiedMessage 接口预留——公理 7，不为想象用户写代码"**。

## 决策

### 1. Phase 1 只实现 Anthropic Messages API 协议适配

具体范围：

| 维度 | Phase 1 实现 |
|------|-------------|
| 客户端 | 仅 Claude Code（通过 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`） |
| 协议 | Anthropic Messages API + Anthropic SSE 格式 |
| 上游 | `api.anthropic.com` 官方 + 第三方中转站（必须兼容 Anthropic 协议） |
| 模型 | Claude Sonnet / Opus 全系列（不区分） |
| 工具调用 | Anthropic `tool_use` / `tool_result` block |
| Multi-modal | Image block 透传不扫描 |

**Phase 1 不实现**：

- ❌ OpenAI Chat Completions（即使中转站把 OpenAI 协议代理为 Anthropic 也不算"原生支持"）
- ❌ OpenRouter
- ❌ Google Gemini
- ❌ 本地 Ollama
- ❌ Hermes、OpenClaw、Cursor、Aider 等其它客户端
- ❌ WebSocket / gRPC 传输

### 2. UnifiedMessage 设计成可扩展，但不实现扩展

**接口预留点**（详见 [data-model.md §1](./data-model.md)）：

- `UnifiedMessage.metadata.upstream_provider` 是枚举 `enum UpstreamProvider { Anthropic, Relay(String) }`，**预留** `OpenAI / OpenRouter / Gemini` 变体；
- `ContentBlock` 用 enum 形态而非 Anthropic-specific struct，使新增 `FunctionCall { ... }`（OpenAI 风格）变体可加而不破坏检测器；
- `ToolUseBlock.input` 用 `serde_json::Value`，**不强 schema**，避免不同协议的工具调用 schema 强行折叠成 LCD；
- 检测器只与 `UnifiedMessage` 交互，**不**直接访问原始 JSON 或 Anthropic-specific 字段。

**预留 ≠ 实现**：

- 不写"OpenAI 适配的占位代码"；
- 不写"未来用的协议路由器"；
- 不写"Provider trait + 多个空实现"；
- **Phase 1 代码里只有一个 Anthropic 适配器**，`UpstreamProvider` 枚举只有 `Anthropic` 和 `Relay(String)` 两个变体被实际构造。

预留只体现在**类型签名 / 模块划分 / 接口形状**上——保证未来加适配器不需要重写检测器和 SSE Parser。

### 3. 第二适配触发条件

**第二个真实付费用户主动要求** OpenAI / OpenRouter 时启动第二适配器开发。

**不算触发条件**：

- ❌ doskey 自己想要（容易过度乐观）；
- ❌ 0 付费用户的 GitHub issue（信号弱）；
- ❌ 1 个免费试用用户提（试用期间反馈无承诺）；
- ❌ 营销文章评论"为什么不支持 X"（噪声）。

**算触发条件**：

- ✅ 1 个**已付费 ≥ 1 个月**的用户在 Discord / 邮件主动说"我需要 X 才能继续用"；
- ✅ 闭测期间 5–10 个 hackathon builder 中**至少 2 人**有同类需求（聚合信号）。

第二适配器开发预算：约 2–3 周。这把 Phase 2 路线图延后 2–3 周是合理代价。

---

## 影响

### 正面影响

1. **聚焦**：12 周冲刺资源全部对准 Anthropic + Claude Code，覆盖深度（fuzz、benchmark、dogfood）能做到位；
2. **Anthropic 协议适配深度**：可以做 Anthropic-specific 优化（如 partial JSON parser 针对 Claude tool_use 块的细节、SSE event 类型 `content_block_delta` 的精确处理），不必为 LCD 抽象妥协；
3. **测试 / fuzz 范围有界**：SSE 边界 fuzz、tool_use 边界 fuzz 只针对 Anthropic 一种格式，能投入足够时间做 100+ corpus 输入；
4. **接口预留成本可控**：只在类型签名层面体现"未来可扩展"，没有冗余的 trait + 空实现 + 路由层；
5. **公理 7 兑现**：不为想象用户写代码——任何"未来可能要做"的事都 push 到 Phase 2 触发器。

### 负面影响

1. **错过潜在 OpenAI 用户**：OpenAI Codex / Cursor 用户群规模可能比 Claude Code 大；Phase 1 拒绝这部分流量是有机会成本的；用 §3 触发器约束自己别过早响应；
2. **被竞品抢先**：如果 Lakera 等先支持 OpenAI 协议，可能让"先发优势"被打散；反击 talking point 是"crypto + 可验证 + 完全本地"三连——这是 Lakera 不会做的；
3. **接口预留有维护成本**：如果未来发现接口预留**形状不对**（例如 OpenAI 的 function calling schema 完全不能塞进 `ToolUseBlock`），需要重构；缓解：Phase 1 写代码时**主动用一份 OpenAI 协议样本做 paper exercise**，验证抽象形状能容下；
4. **Anthropic 自己改协议**：Anthropic 可能在 12 周内改 Messages API（如新增 thinking blocks）；缓解：保持依赖 Anthropic 官方 SDK 的 schema 定义，不自己 fork。

### 需要更新的文档

- [PRD-sieve v1.4 §6.1](../prd/_archive/sieve-prd-v1.5.md) —— Phase 1 单 agent 架构图已对齐"只 Claude Code"
- [PRD-sieve v1.4 §9.9](../prd/_archive/sieve-prd-v1.5.md) —— 工程硬约束第 9 条已写明此决策
- [PRD-sieve v1.4 §10.3 Phase C](../prd/_archive/sieve-prd-v1.5.md) —— "第二个用户主动要 OpenClaw / Hermes / MCP 适配时再做"
- [data-model.md](./data-model.md) §1 —— UnifiedMessage 接口形状已对齐
- [architecture.md](./architecture.md) §2 —— Protocol Layer 模块职责已对齐
- `docs/api/api-reference.md`（待编写）—— Phase 1 仅文档化 Anthropic Messages API 适配

---

## 相关文档

- [PRD-sieve v1.4 §6.1](../prd/_archive/sieve-prd-v1.5.md) —— Phase 1 单 agent 架构
- [PRD-sieve v1.4 §9.9](../prd/_archive/sieve-prd-v1.5.md) —— "Phase 1 只做 Claude Code"
- [PRD-sieve v1.4 §10.3](../prd/_archive/sieve-prd-v1.5.md) —— 慢节奏维护期的扩展触发条件
- [architecture.md](./architecture.md) —— Protocol Layer 与 Phase 2 演进路径
- [data-model.md](./data-model.md) —— UnifiedMessage 设计与字段说明
- [ADR-002](./ADR-002-rule-engine-only-phase1.md) —— 同样应用"触发条件而非路线图"原则

---

## 2026-04-28 补充（v1.4 双层防御对 Anthropic-first 的延伸含义）

> 本段由 [ADR-014](./ADR-014-dual-layer-defense.md) 引入，不修改本 ADR 原有内容。

**"Phase 1 只适配 Claude Code"的结论仍然成立**，且被 v1.4 进一步强化：

v1.4 §6.7 新增**双层防御**依赖 Claude Code 特有的 `hooks.preToolUse` 机制（通过 `settings.json` 注册，`onError: block` 语义）。这是 Anthropic 私有的客户端扩展协议，不存在于 OpenAI Codex、Cursor、Aider 等其他客户端中。

换言之，**sieve-hook 这一层的 fail-closed 实现是 Claude Code 专有的**——Phase 2 适配 OpenClaw / Hermes 等客户端时，Hook 类规则（IN-CR-02/03/04 / IN-GEN-01~03）的等效拦截机制需要各自重新实现（对等的 pre-execution hook，或者降级为 GUI 弹窗路径）。

本条补充不改变 §3 的第二适配触发条件——Phase 2 适配仍须等到第二个真实付费用户主动要求。届时适配工作量还需额外包含"Hook 类规则的等效拦截机制"，比 v1.3 时代估算的 2-3 周更长。

相关文档：[ADR-014 双层防御](./ADR-014-dual-layer-defense.md)
