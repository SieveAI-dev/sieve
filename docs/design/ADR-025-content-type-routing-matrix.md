# ADR-025: content-type 路由矩阵——v1.5.4 P0 教训永久化为工程不变量

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：v2.0 全周期；适用于所有新增入站功能（用户规则 / 三态决策灰名单 / 行为序列窗口 / 进程上下文反查）
> 关联 PRD：v2.0 §9 #16、v2.0 §5.7.4 双路径不变量
> 关联 review：codex review PRD v2.0 §E1 / §E2 / §E3
> 直接来源：[CHANGELOG v1.5.4 P0 修复](../changelog/CHANGELOG.md)

---

## 背景

### v1.5.4 双 P0 漏洞：入站检测在非流式路径完全失效

2026-05-01，v1.5.4 patch 修了两个 P0 安全漏洞（CHANGELOG v1.5.4）：

**P0 漏洞 1（Anthropic 非流式路径）**：
daemon 的入站检测逻辑原本只挂在 SSE 流（`text/event-stream`）解析器上。当 Anthropic API 返回 `Content-Type: application/json`（非流式模式）时，响应体中的 `tool_use` block 绕过了所有入站规则——包括 Critical 等级的 IN-CR-02/03/04/05 / IN-GEN-*，共 70 条规则全部失效。

**P0 漏洞 2（OpenAI stream=false 路径）**：
`proxy_openai` 的 `stream=false` 分支直接调用 `forward_raw`，跳过整个入站检测路径。由于 OpenAI API 默认 `stream=false`，这意味着 OpenAI 入站规则从来没有真正生效过——70 条规则在 OpenAI 路径的实际拦截率为 0%。

**根因分析**：
入站检测逻辑只挂在 SSE parser 后，而 daemon 有 4 条分叉的 content-type 路由路径：

```
响应 content-type 路由
├── Anthropic + text/event-stream  → SSE parser → InboundFilter ✅（原有）
├── Anthropic + application/json   → forward_raw               ❌（漏洞 1）
├── OpenAI   + text/event-stream   → SSE parser → InboundFilter ✅（原有）
└── OpenAI   + application/json    → forward_raw               ❌（漏洞 2）
```

v1.5.4 通过新增 `handle_anthropic_json_inbound()` 和 `handle_openai_json_inbound()` 补全了后两条路径。但这是**事后补救**——根本问题是没有约束机制保证"新功能必须覆盖所有 4 条路径"。

### v2.0 新增功能的回归风险

codex review §E1 / §E2 指出：v2.0 新增的 4 类功能（用户规则 / 三态决策灰名单 / 行为序列窗口 / 进程上下文反查）如果重蹈同样的覆辙，只挂在 SSE parser 后，则：
- 用户规则 → 只对 SSE 响应生效，JSON 响应绕过
- 灰名单查询 → SSE 路径的 fingerprint 与 JSON 路径计算不一致，导致灰名单静默失效
- 行为序列窗口 → 只在 SSE 路径更新，JSON path 的 tool_use 不进入序列（R-V20-04）
- 进程上下文反查 → 只在 SSE scan 时记录 caller_exe，JSON 路径的 audit 字段为 NULL

这 4 类功能是 v2.0 的核心差异化卖点，任何一类重新引入 v1.5.4 P0 漏洞模式，都会被 HIPS 定位的用户视为产品级安全漏洞（PRD §2 "不能再有 v1.5.4 这种 P0 漏洞"）。

### 为什么需要 ADR 而不是仅在 PRD 里写硬约束

PRD §9 #16 已经作为硬约束写入，但约束不会自动执行。本 ADR 的目的是：
1. 记录约束的来源和理由（可追溯性）
2. 定义具体的工程守护机制（测试矩阵 + CI gate + PR 模板）
3. 明确"守护机制在 daemon 重构时的继承义务"（v2.1 拦截引擎抽象时必须保持这一不变量）

---

## 决策

### 1. content-type 路由矩阵定义（4 类强制组合）

v2.0 起，所有新增入站能力必须有集成测试覆盖以下 4 类组合：

| 编号 | 协议 | 响应模式 | Content-Type | daemon 路径 |
|------|------|---------|-------------|-----------|
| M-1 | Anthropic | 流式 SSE | `text/event-stream` | `forward_with_inbound_inspection` SSE 分支 |
| M-2 | Anthropic | 非流式 JSON | `application/json` | `handle_anthropic_json_inbound` |
| M-3 | OpenAI | 流式 SSE | `text/event-stream` | `forward_with_openai_inbound_inspection` SSE 分支 |
| M-4 | OpenAI | 非流式 JSON（默认）| `application/json` | `handle_openai_json_inbound` |

**"覆盖"的定义**：集成测试 mock 上游返回对应 content-type 响应，验证新功能（用户规则命中 / 灰名单查询 / 序列窗口更新 / caller_exe 记录）在该路径上正确生效，结果可断言。

### 2. 实施：集成测试矩阵（crates/sieve-cli/tests/）

**新 test case 命名规范**：`<功能>_<协议>_<模式>_<测试语义>`

示例（v2.0 Phase A Week 6 必须补齐）：

```
inbound_block.rs
├── user_rule_anthropic_sse_block        （M-1：用户规则在 Anthropic SSE 命中）
├── user_rule_anthropic_json_block       （M-2：用户规则在 Anthropic JSON 命中）
├── user_rule_openai_sse_block           （M-3：用户规则在 OpenAI SSE 命中）
├── user_rule_openai_json_block          （M-4：用户规则在 OpenAI JSON 命中）
├── greylist_anthropic_sse_allow         （M-1：灰名单命中后 Anthropic SSE 允许）
├── greylist_anthropic_json_allow        （M-2：灰名单命中后 Anthropic JSON 允许）
├── greylist_openai_sse_allow            （M-3：灰名单命中后 OpenAI SSE 允许）
├── greylist_openai_json_allow           （M-4：灰名单命中后 OpenAI JSON 允许）
├── sequence_anthropic_sse_update        （M-1：Anthropic SSE tool_use 进入序列窗口）
├── sequence_anthropic_json_update       （M-2：Anthropic JSON tool_use 进入序列窗口）
├── sequence_openai_sse_update           （M-3：OpenAI SSE tool_calls 进入序列窗口）
└── sequence_openai_json_update          （M-4：OpenAI JSON tool_calls 进入序列窗口）
```

> 进程上下文（caller_exe）在集成测试中可 mock（测试进程即 caller），不要求真实 `proc_pidpath`；主要验证 M-1~M-4 路径上 audit 事件均有 `caller_pid` 字段。

### 3. CI gate：规则 ID 覆盖检测

v2.0 起，CI job `check-routing-matrix` 验证：**每个新增规则 ID（包括用户规则的 `MY-*` 前缀规则）在 M-1~M-4 这 4 类集成测试里至少各被覆盖一次**。

检测机制：`scripts/check_routing_coverage.sh` 扫描 `crates/sieve-cli/tests/inbound_block.rs`，确认每个 `rule_id` 在 4 类 test case 里都出现。覆盖不全则 CI 失败，PR 不可合并。

> 当前（v1.5.4 后）系统规则 ID 已有 M-1/M-4 覆盖（即 v1.5.4 补的那 2 个 test case）。v2.0 要求补齐 M-2 / M-3 组合的全量覆盖。

### 4. PR 描述模板：强制勾选项

所有涉及入站功能的 PR（新增规则 / 修改 scan 路径 / daemon 重构）在 PR 描述中必须包含以下 checklist：

```markdown
## content-type 路由矩阵覆盖确认

- [ ] M-1 Anthropic + text/event-stream：新功能集成测试已覆盖
- [ ] M-2 Anthropic + application/json：新功能集成测试已覆盖
- [ ] M-3 OpenAI + text/event-stream：新功能集成测试已覆盖
- [ ] M-4 OpenAI + application/json（stream=false）：新功能集成测试已覆盖
```

未全勾选的 PR 在 review 时自动标记 "needs-routing-coverage"，不得合并。

### 5. 守护机制：v2.1 拦截引擎抽象时的继承义务

PRD §6.4 已决定"拦截引擎抽象推 v2.1"（参见 ADR-INDEX 候选中 OQ-V20-07 关闭决策）。v2.1 实施拦截引擎抽象（daemon.rs 重构）时：

**必须保持 content-type 路由矩阵不变量**：新的 `InterceptEngine` trait 或 `MacOSInterceptor` impl 必须保证 M-1~M-4 四条路径在重构后仍然各自调用对应的 inbound 检测函数，且集成测试矩阵全部通过后才能合并。

这是 v2.1 重构的必达验收标准，不是可选项。v2.1 ADR（届时编写）必须引用本 ADR 作为不变量约束来源。

### 6. 与 PRD §9 硬约束的关系

| 本 ADR 条目 | 对应 PRD §9 约束 | 关系 |
|------------|----------------|------|
| 4 类组合集成测试 | §9 #16（content-type 路由矩阵测试）| 工程实施细节 |
| CI gate + PR 模板 | §9 #16（"CI gate 检测"）| 自动化守护 |
| 双路径不变量（SSE + JSON 同时更新）| §9 #15 + §5.7.4 | 行为序列的特殊约束，本 ADR 普适化 |
| v2.1 继承义务 | §9 #12 不装 MITM / §9 #1 Rust 栈 | daemon 重构时的上游约束 |

**本 ADR 不引入新的硬约束语义**——PRD §9 #2 / §9 #11 / §9 #16 的语义不变。本 ADR 是这些约束的工程实施规格，确保约束可被测试、可被 CI 守护。

### 7. 与 §5.7.4 双路径不变量的关系

PRD §5.7.4 是针对行为序列窗口的专项双路径不变量（"序列窗口更新必须同时覆盖 SSE 路径 + JSON 路径"），违反则视为 P0 漏洞。

本 ADR 将该不变量**普适化**到所有入站功能：不仅行为序列，用户规则评估 / 灰名单查询 / 进程上下文记录都必须在 M-1~M-4 四条路径上生效。§5.7.4 是本 ADR 的一个具体实例，不单独维护。

---

## 影响

### 正面影响

1. **P0 漏洞模式永久封堵**：任何新功能只要通过 CI gate + PR 矩阵检查，就不可能重新引入"只挂 SSE 不挂 JSON"的漏洞模式；
2. **审计可信度**：M-1~M-4 都覆盖后，audit.db 里的 `caller_exe` / 灰名单记录 / 序列窗口数据对所有路径都真实可靠，不会因 content-type 而静默缺失；
3. **HIPS 定位可信度**：PRD §2 说"不能再有 v1.5.4 这种 P0 漏洞"——本 ADR 的 CI gate 是这一承诺的工程担保；
4. **重构安全网**：v2.1 daemon.rs 重构时有明确的验收标准，不依赖 reviewer 记忆"v1.5.4 的教训"。

### 负面影响

1. **测试量翻倍**：每个新功能需要 4 个 test case（M-1~M-4）而非 1 个，测试文件行数增长 4x——但这是必要成本，不是 over-testing；
2. **CI 时间增加**：`check-routing-matrix` job + 4x 集成测试会延长 CI 时间（估计 +30~60s）；
3. **PR 流程摩擦**：PR 描述模板的 4 项强制勾选增加提交者的心智负担——但这是 v1.5.4 P0 漏洞的教训要求，不能省略；
4. **历史存量 test case 的欠债**：v1.5.4 之前的系统规则测试只有 M-1（和新增的 M-4）覆盖，M-2/M-3 是欠债——v2.0 Phase A Week 6 必须补齐。

### 需要更新的文档

- `docs/guides/development.md` —— 加入 content-type 路由矩阵测试规范 + PR 模板说明
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目（ADR-025）
- PR 描述模板（`.github/pull_request_template.md` 或等效文件）—— 加入矩阵覆盖勾选项

---

## 相关文档

- [CHANGELOG v1.5.4 P0 修复](../changelog/CHANGELOG.md) —— 本 ADR 的直接来源，content-type 路由漏洞的根因记录
- PRD v2.0 §9 #16 content-type 路由矩阵硬约束
- PRD v2.0 §5.7.4 双路径不变量
- PRD v2.0 §12 R-V20-04 content-type 路由回归风险
- codex review v2.0 §E1 / §E2 / §E3
- [ADR-024](./ADR-024-rules-engine-abstraction.md) —— ScanRequest 上下文字段（`direction` / `protocol` / `content_kind`）是路由矩阵在引擎层的体现
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（路由矩阵测试保证 Critical 在所有路径都生效）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（路由矩阵约束代理层的 4 条 content-type 路径）
- [ADR-018](./ADR-018-openai-protocol-adaptation.md) —— OpenAI 协议适配（M-3 / M-4 的协议层基础）
