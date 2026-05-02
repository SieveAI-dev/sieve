# ADR-019: X-Sieve-Origin HTTP Header 协议——sub-agent 嵌套调用链元数据传递

## 状态

**已接受**（v1.5 锁定执行）

> 决策日期：2026-04-28
> 范围：Phase 1 Week 7 实现，`crates/sieve-core/src/forwarder/` + IPC schema 扩展
> 关联 PRD：[v1.5 §4.6 场景 F、§6.5 IPC 协议扩展](../prd/sieve-prd-v2.0.md)

---

## 背景

### 问题：Hermes 嵌套调用产生双重弹窗或漏弹

PRD v2.0 §4.6 场景 F 描述了一种典型的 multi-agent 嵌套调用路径：

```
用户 → Hermes Agent → Sieve 主代理 → 上游 LLM
        （Hermes 决定 delegate 给 Claude Code）
        Hermes → 启动 claude-code 子进程
                 claude-code → Sieve 主代理（另一个 Sieve 实例或同一个）→ Anthropic API
```

在这条链路上存在两个独立的 Sieve 检测点：
1. Hermes 主代理请求经过 Sieve → 可能触发 GUI 弹窗
2. Hermes delegate 给 Claude Code 后，Claude Code 请求再次经过 Sieve → 又可能触发 GUI 弹窗

**双重弹窗问题**：同一用户意图（"帮我写 ERC20 转账脚本"）被拆为两个相互不关联的 GUI 弹窗询问。用户在 Hermes 层已经点了 Allow，Claude Code 层再弹一次，体验极差，且用户无法理解两次弹窗的关系。

**漏弹问题**（更严重）：如果两个 Sieve 实例各自超时后都 fail-closed，一次危险操作被拆成两段分别过了两个低阈值，结果反而都放行了。这违背了 PRD §9 第 3 条（fail-closed 不可关）的精神。

### 为什么不用共享状态数据库

一个候选方案是：Sieve 把所有 pending 请求存到 SQLite，两个实例（或同一实例的两个请求）通过 SQLite 查询判断是否同一链路已有 Allow 记录。

被拒绝原因：
1. **多进程写入风险**：如果用户同时运行多个 agent，多个 Sieve 实例同时写 SQLite 需要文件锁，复杂度高；
2. **request_id 对齐难度**：Hermes 发出的请求和 Claude Code 发出的请求在 SQLite 里无法自然关联，除非额外机制传递 ID；
3. **根本原因是没有调用链 ID**：所有方案的前提都是"让下游 Sieve 知道这是上游已 Allow 链路的继续"，这需要一个显式的调用链标识符在 HTTP 层传递——这就是 X-Sieve-Origin header 要解决的问题。

### 为什么不用 IPC 广播

另一个候选是：上游 Sieve 实例通过 IPC 广播给下游 Sieve 实例"request_id 已 Allow"。

被拒绝原因：
1. **拓扑假设过强**：假设所有 Sieve 实例能互相发现并建立 IPC 连接，但 OpenClaw / Hermes 的 sub-agent 可能在不同 $HOME 路径下运行；
2. **时序问题**：IPC 广播到下游需要时间，而 HTTP 请求是同步的，下游 Sieve 可能在收到 IPC 之前已经弹出 GUI；
3. **HTTP header 更简单**：header 随请求携带，零额外延迟，无时序问题。

---

## 决策

**引入 HTTP header `X-Sieve-Origin`，由上游 agent 或 Sieve 主代理在发起 sub-agent 请求时注入，携带调用链元数据。下游 Sieve 实例解析此 header，结合 IPC pending 表查询，决定是否跳过重复弹窗。**

### Header 格式规范

```
X-Sieve-Origin: <source_agent>:<request_id>:<chain_depth>
```

字段定义：

| 字段 | 类型 | 说明 |
|------|------|------|
| `source_agent` | 枚举字符串 | 调用链的根来源 agent，取值见下表 |
| `request_id` | UUID v4 | 调用链根请求的 ID（所有嵌套层共享同一个） |
| `chain_depth` | u8 | 当前层级深度，0 = 用户直接调 agent |

`source_agent` 取值：

| 值 | 含义 |
|----|------|
| `claude` | 用户直接调 Claude Code |
| `hermes` | 用户直接调 Hermes（Hermes 发起的请求） |
| `openclaw` | 用户直接调 OpenClaw |
| `hermes-delegate-claude` | Hermes 将任务委托给 Claude Code |
| `hermes-delegate-codex` | Hermes 将任务委托给 Codex CLI |

### 示例

```
# 用户直接调 Claude Code（chain_depth=0，通常无 header 或 Sieve 自动填充）
X-Sieve-Origin: claude:abc-123:0

# 用户直接调 Hermes（chain_depth=0）
X-Sieve-Origin: hermes:def-456:0

# Hermes 将请求委托给 Claude Code（chain_depth=1，同一 request_id）
X-Sieve-Origin: hermes-delegate-claude:def-456:1
```

### chain_depth 语义与安全门限

| chain_depth | 语义 | Sieve 行为 |
|-------------|------|-----------|
| 0 | 用户直接调 agent | 正常检测流程 |
| 1 | agent 调用 sub-agent | 查 IPC pending 表；上游已 Allow → 跳过重复弹窗；未查到 → 正常弹窗 |
| ≥ 2 | 嵌套调用链超过 2 层 | 强制 fail-closed GUI hold + 警告"嵌套调用超过 2 层，可疑行为" |
| ≥ 5 | 极深嵌套 | 直接返回 426（攻击模式，不弹窗，直接拒绝） |

`chain_depth ≥ 5` 的 426 响应是 fail-closed，不是 fail-open——攻击者无法通过增加嵌套深度绕过检测。

### 双重弹窗去重协议

完整的去重流程如下：

```
用户 → Hermes CLI → "帮我写 ERC20 转账脚本"

步骤 1：Hermes 发起请求给上游 LLM
  HTTP: POST /v1/chat/completions
  （无 X-Sieve-Origin，Sieve 判断 chain_depth=0）
  Sieve 检测：触发 IN-CR-05（签名调用）
  GUI 弹窗：
    "Hermes 请求签名 ERC20 转账，允许？"
    [拒绝] [允许]
  用户点"允许" → Sieve 在 IPC pending 表记录：
    { request_id: "def-456", agent: "hermes", decision: Allow, timestamp: T }

步骤 2：Hermes 决定 delegate 给 Claude Code
  Hermes 启动 claude-code 子进程
  Hermes 在 HTTP header 注入（或通过 ANTHROPIC_DEFAULT_HEADERS env var）：
    X-Sieve-Origin: hermes-delegate-claude:def-456:1

步骤 3：Claude Code 发起请求给 Anthropic API（经过 Sieve）
  Sieve 解析 X-Sieve-Origin：
    source_agent = hermes-delegate-claude
    request_id   = def-456（与步骤 1 相同）
    chain_depth  = 1（在安全门限内）

  Sieve 查 IPC pending 表：
    找到 { request_id: "def-456", decision: Allow }
  
  Sieve 判断：同一调用链，上游已 Allow
  ✅ 跳过重复弹窗，直接放行
  （独立的 Critical 规则例外：如 IN-CR-05 在 Claude Code 层触发新的签名动作，仍弹窗）
```

**独立 Critical 规则不受去重影响**：如果 Claude Code 子任务中**新触发**了上游 Hermes 请求中没有的危险动作（例如 Hermes 的上下文里没有签名请求，但 Claude Code 发现了一个额外的 eth_sendTransaction），Sieve 仍然弹窗。去重只针对"同一危险动作在嵌套链里被重复确认"。

### 签名验证（防伪造）

header 值由 Sieve 自身签名，防止攻击者伪造 `chain_depth=1` 绕过弹窗：

```
X-Sieve-Origin: hermes-delegate-claude:def-456:1
X-Sieve-Origin-Sig: <Ed25519 signature over "hermes-delegate-claude:def-456:1">
```

签名密钥：Sieve 在本地生成，存储在 `~/.sieve/ipc.key`（与 IPC 签名密钥同一个，详见 ADR-013）。

验证逻辑：
- 有 `X-Sieve-Origin` 但无 `X-Sieve-Origin-Sig`：按无 header 处理（chain_depth=0）
- 有 `X-Sieve-Origin` 且签名验证失败：fail-closed，426 拒绝，记录审计日志
- 有 `X-Sieve-Origin` 且签名验证通过：按 chain_depth 正常处理

**例外**：`chain_depth=0` 的 header 无需签名（等效于不带 header）。

### Header 注入方式

Hermes 需要在 sub-agent 子进程上注入此 header。两种实现路径：

1. **`ANTHROPIC_DEFAULT_HEADERS` env var**（Phase 1 后期首选）：
   ```bash
   ANTHROPIC_DEFAULT_HEADERS='{"X-Sieve-Origin":"hermes-delegate-claude:def-456:1","X-Sieve-Origin-Sig":"<sig>"}' claude
   ```
   Claude Code SDK 在 `0.2.x+` 版本支持此 env var，会将其合并到所有 HTTP 请求 header 中。

2. **Hermes 内置中间件注入**（Phase 1 后期目标，需给 Hermes 提 PR）：
   Hermes 在 delegate 给 sub-agent 时，通过自己的 HTTP client 中间件注入 header。PRD §14 Open Questions 第 11 条登记此问题待研究。

**不阻塞 GA**：`ANTHROPIC_DEFAULT_HEADERS` 路径是 Hermes 用户手动配置，已足够在 Week 7 集成测试中验证场景 F。给 Hermes 提 PR 是 Phase 1 后期优化，GA 前不强求。

---

## 影响

### 正面影响

1. **消除双重弹窗**：Hermes 嵌套调用 Claude Code 时，用户只看到一次 GUI 弹窗，UX 正常；
2. **调用链可视化**：GUI 弹窗里展示完整调用链（面包屑 UI），用户能看懂"谁调用了谁"；
3. **fail-closed 强化**：chain_depth ≥ 5 直接 426，嵌套攻击无效；
4. **零引擎改动**：去重逻辑在 forwarder 层处理，sieve-rules / dispatch 不感知；
5. **可签名验证**：Ed25519 签名防止 header 伪造，攻击面有限。

### 负面影响

1. **Hermes / OpenClaw 需要配合注入 header**：Phase 1 靠 env var 手动配置，体验不够流畅；需要和 Hermes / OpenClaw 社区协作（PRD §14 第 11 条）；
2. **IPC pending 表需要额外维护**：request_id 超过 TTL（建议 10 分钟）需要清理，否则 pending 表无限增长；
3. **request_id 对齐依赖 Hermes 配合**：如果 Hermes 发起请求时没有生成 request_id，或者 delegate 时没有透传，去重机制失效（退化为有弹窗但没有双重弹窗问题加重）；
4. **Week 7 集成测试覆盖**：场景 F 的端到端测试需要真实 Hermes 进程，集成测试环境搭建有一定成本。

### 需要更新的文档

- [PRD v2.0 §6.5](../prd/sieve-prd-v2.0.md) — X-Sieve-Origin header 格式定义（已在 PRD 中预占位，本 ADR 提供完整规范）
- [architecture.md](./architecture.md) — forwarder 模块加 Origin Header 解析节点（G4 子代理）
- [data-model.md](./data-model.md) — IPC pending 表 schema 加 request_id + chain_depth 字段
- [api-reference.md](../api/api-reference.md) — §3 代理行为说明中加 X-Sieve-Origin header 文档
- `CHANGELOG.md` — 记录 Week 7 header 协议落地

---

## 相关文档

- [PRD v2.0 §4.6 场景 F](../prd/sieve-prd-v2.0.md) — Hermes 嵌套调用问题定义
- [PRD v2.0 §6.5 IPC 协议](../prd/sieve-prd-v2.0.md) — IPC schema 扩展（source_agent / origin_chain 字段）
- [ADR-013](./ADR-013-ipc-protocol.md) — IPC 协议基础（JSON-RPC over Unix socket），本 ADR 在此基础上扩展
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) — fail-closed 约束，chain_depth ≥ 5 直接 426 是其体现
- [ADR-018](./ADR-018-openai-protocol-adaptation.md) — OpenAI 协议适配，嵌套调用链里 OpenAI 请求同样需要传递此 header
- [ADR-014](./ADR-014-dual-layer-defense.md) — 双层防御，本 ADR 解决双层防御在多 agent 嵌套时的 UX 问题
