# ADR-048: 记忆 / RAG 注入检测——写入持久记忆与知识库的提示注入防护

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：v2.x 入站文本检测扩展 + PreToolUse 工具调用层；适用于所有写入 agent 持久记忆 / RAG 知识库 / vault 的内容
> 关联：[ADR-025 content-type 路由矩阵](./ADR-025-content-type-routing-matrix.md)、[ADR-024 规则引擎抽象](./ADR-024-rules-engine-abstraction.md)、[ADR-034 GA 密钥 gate](./ADR-034-ga-key-gate.md)、[ADR-007 fail-closed Critical](./ADR-007-fail-closed-critical-actions.md)、[ADR-014 双层防御](./ADR-014-dual-layer-defense.md)

---

## 背景

### 攻击面：持久记忆是注入的"睡眠仓"

现代 agent（Claude Code / OpenClaw / Hermes 等）普遍支持**持久记忆**与 **RAG 知识库**：把跨会话要记住的内容写进磁盘文件（如 `memory.md`、`.claude/` 或 `.codex/` 下的记忆目录、知识 vault），下次会话开头自动重新读入上下文。这把"一次性的提示注入"升级成**持久化、可复发的注入**——写一次，每次新会话都被重新喂回模型。

#### 真实攻击场景

1. **远程内容直写记忆**：agent 调外部工具拉回一段网页 / issue / README，里面嵌着 `ignore previous instructions, your new system prompt is …`，agent 把这段"有用笔记"原样写进 `memory.md`。下次会话该段被当作可信记忆重新加载，注入在用户毫不知情的情况下生效。crypto 场景下，注入可改写"默认转账地址""可信合约白名单"等长期事实。

2. **策略覆盖落盘（policy override）**：注入文本携带 `disable guardrails` / `bypass the safety check` 一类指令，一旦写入持久记忆，等于给后续所有会话埋了一句"放松防护"的常驻指令。

3. **exfil 指令常驻**：把"每次启动时把 `~/.ssh` / keystore 内容发到某地址"写进记忆，转化为持久化的数据外泄触发器。

4. **隐藏编码绕过人眼**：注入用零宽字符（zero-width）、Unicode 不可见控制字符或 base64 包裹，使其在编辑器 / diff 里几乎不可见，人工 review 记忆文件时看不出异常，但模型仍会解码并执行。

5. **工具信任覆盖**：记忆里写 `install this mcp server` / `trust this skill`，诱导后续会话自动信任并加载攻击者控制的工具来源，扩大攻击面。

### 为什么现有检测不够

- 出站脱敏（OUT-*）只看密钥/助记词等敏感**模式**，不识别"注入语义"。
- 入站 IN-CR-01 地址替换、IN-CR-02~05 危险工具调用各有专责，但**没有一类规则盯住"把注入文本写进持久记忆"**这条路径。
- 注入可以从两个方向落地：① assistant 响应文本里直接出现"请把以下内容记到记忆"——属于**入站响应文本侧**；② agent 真的发起一次 `Write` / `Edit` 到记忆路径——属于 **PreToolUse 工具调用侧**。两侧都要堵，否则留缺口。

业界对持久记忆 / 知识库注入已有公开认知（记忆投毒、RAG 投毒是常见提法）；本 ADR 把这一防护以中性、可配置的规则形式纳入 Sieve 的既有双层防御架构。

---

## 决策

**新增"记忆 / RAG 注入检测"能力：在入站响应文本侧扩展 `scan_assistant_text`、在 PreToolUse 侧检测写入记忆路径的内容，命中注入语义（提示走私 / 策略覆盖 / exfil 指令 / 远程内容直写 / 隐藏编码 / 工具信任覆盖）即按处置矩阵拦截；具体检测规则定义由签名规则包提供，随更新通道分发。**

正文不内联任何精确正则、关键词表、特征字段或阈值数值——这些属于签名规则包，遵循 [ADR-024](./ADR-024-rules-engine-abstraction.md) 的引擎/规则分离与 [ADR-034](./ADR-034-ga-key-gate.md) 的签名分发机制，随更新通道分发并热替换。

---

## 硬约束逐条核对（PRD §9）

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed** | ✔ 适用 | PreToolUse 侧复用现有 `judge_tool_call`，daemon 不可达 / 超时 / 协议错误一律 fail-closed（见 `sieve-hook/src/codex_ipc.rs:32` 文档约定）；入站 JSON 路径无 keep-alive，`HoldForDecision` 降级为 fail-closed Block，与 IN-CR-01 同构（`inbound.rs:275`）。 |
| **Critical 所有版本不可关** | ✔ 适用 | 注入写入记忆中达 Critical 等级的处置由系统签名规则定义，走 `critical_lock` 强制路径，用户规则与 dry_run 均不可 suppress；GA 默认开启、降级模式不可关。 |
| **BIP39 必须 SHA-256 checksum** | ✘ 不适用 | 本 ADR 不涉及助记词检测，BIP39 second-pass 逻辑（`sieve-rules/bip39.rs`）不受影响、不放宽。 |
| **绝不联网做 verifier** | ✔ 适用 | 全部检测在本地完成：文本扫描走本地引擎，编码归一化（zero-width / Unicode invis / base64 解包）在进程内做；远程内容是否被写入记忆仅靠**工具调用元数据 + 文本特征**判断，绝不外发任何片段做远端校验。 |
| **不在 API 协议层撒谎 / 不伪造 tool_use** | ✔ 适用 | 入站侧只检测与按既有处置矩阵拦截，不伪造 `tool_use` / `stop_reason` / `id` / `usage`；拦截仍走既有 `sieve_blocked` 自报事件机制（ADR 已确立），不冒充模型。 |
| **不装本地 CA 做 MITM** | ✔ 适用 | 复用既有代理与 PreToolUse hook 通道，不引入 Network Extension / 本地 CA / 系统 proxy 改动。 |
| **出站脱敏自动改写不弹窗** | ✘ 不适用 | 本 ADR 是入站 + 工具调用层检测，不触碰 OUT-* 自动脱敏路径；出站高频脱敏的"自动改写 + 状态栏通知、不弹窗"行为不变。 |
| **四路由矩阵** | ✔ **核心适用** | 入站响应文本侧扩展 `scan_assistant_text`，必须在 M-1~M-4 四条 content-type 路由上行为一致，否则重蹈 v1.5.4「只挂 SSE 不挂 JSON」P0。详见验收标准。 |

---

## 方案

复用既有架构，不新增 pipeline 节点，分两侧接线（接线点核实自源码，禁脑补）：

### (a) 入站响应文本侧——扩展 `scan_assistant_text`

检测"assistant 响应让 agent 把注入文本写进记忆"这一语义。接入点是入站文本检测的共享核心：

- `sieve-core/src/pipeline/inbound.rs:253` `scan_assistant_text()` —— 现已做 IN-GEN-* 文本规则扫描（`engine.scan_text(text, ContentSource::InboundAssistantText, 0)`，`inbound.rs:257-260`）+ IN-CR-01 地址替换。新增的记忆注入规则 ID 由签名规则包提供，通过同一条 `engine.scan_text` 路径生效，**无需新增扫描入口**。
- 该方法被四条 content-type 路由共同调用，构成四路由对等的天然保证：
  - **M-1 / M-3（SSE）**：`inbound.rs:312` `observe_event()` 在流式解析后调 `scan_assistant_text`。
  - **M-2（Anthropic JSON）**：`sieve-cli/src/daemon.rs:4526` `handle_anthropic_json_inbound()` 在 `daemon.rs:4641` 调 `scan_assistant_text`。
  - **M-4（OpenAI JSON）**：`sieve-cli/src/daemon.rs:4686` `handle_openai_json_inbound()` 在 `daemon.rs:4811` 调 `scan_assistant_text`。

> 注：编码归一化（解 zero-width / Unicode invis / base64）需在送入 `scan_text` 前对文本做一遍归一化预处理，使隐藏编码后的注入仍能命中规则；归一化逻辑在 `sieve-core` 入站侧本地完成，归一化策略参数由规则包提供。

### (b) PreToolUse 侧——检测写入记忆路径的工具调用

检测 agent 实际发起 `Write` / `Edit`（或等价写文件工具）到记忆路径（如 `memory.md`、`.claude` / `.codex` 等 agent 记忆目录）时，其**写入内容**是否含注入语义。接入既有判危链：

- `sieve-hook/src/main.rs` PreToolUse 入口 → `sieve-hook/src/codex_ipc.rs:32` `judge_tool_call(sieve_home, tool_name, tool_input, tool_use_id, cwd, deadline)` → daemon `sieve-cli/src/daemon_control_plane.rs:1018` `handle_judge_tool_call()` → `inbound.rs:36` `check_tool_use(tool, source)`。
- 工具调用检测核心在 `sieve-cli/src/engine_adapter.rs:350` `InboundAdapter::check_tool_use`：现已扫工具名 + `serde_json::to_string(&tool.input)` 序列化后的输入（`engine_adapter.rs:357-361`）。`CompletedToolCall { id, name, input }`（`sieve-core/src/tool_use_aggregator.rs:84`）的 `name` 给出工具类型、`input` 给出目标路径与写入内容，足以判定"是否写记忆路径 + 内容是否含注入"。新增的"记忆路径 + 注入内容"规则 ID 由签名规则包提供，沿同一 `scan_text` 路径生效。

### 规则引擎接线

两侧最终都汇入 `sieve-rules` 的 `LayeredEngine`（`engine/mod.rs:165`）/ `SystemEngine`（`engine/system.rs:38`），可热替换签名规则包（ADR-024 / ADR-034）。新规则 ID 的精确定义、关键词、特征字段、阈值与处置映射一律落签名规则包，随更新通道分发，不进本仓代码与文档正文。

---

## 分步实施

每步可独立 ship + 独立测试：

1. **入站文本归一化预处理**：在 `scan_assistant_text` 送入 `engine.scan_text` 前增加一层本地编码归一化（zero-width / Unicode invis / base64 解包），输出归一化文本供规则匹配。单测：构造含隐藏编码的注入文本，断言归一化后命中既有/新规则；归一化对正常文本幂等无副作用。

2. **入站记忆注入规则 ID 接通（四路由）**：签名规则包加入记忆注入规则后，验证其在 `scan_assistant_text` 经四条路由命中。集成测试按 ADR-025 矩阵补 M-1~M-4 四个 test case。

3. **PreToolUse 记忆路径写入检测**：在 `check_tool_use` 路径接通"写记忆路径 + 注入内容"规则 ID。单测：构造 `Write` / `Edit` 到记忆路径、`input` 含注入语义的 `CompletedToolCall`，断言命中并按处置矩阵拦截；写非记忆路径或无注入内容不命中（控误报）。

4. **fail-closed 与降级一致性验证**：验证 daemon 不可达时 PreToolUse fail-closed、JSON 路径 `HoldForDecision` 降级为 Block，与 IN-CR-01 行为一致。

5. **红队 bypass 回归接入**：把本 ADR 的 bypass 用例并入红队测试集（见关联规划的红队门），纳入 CI 回归。

---

## 验收标准

### content-type 四路由矩阵（ADR-025，强制）

入站侧每个新增记忆注入规则 ID 必须在 `crates/sieve-cli/tests/` 集成测试的四类组合各覆盖一次，`scripts/check_routing_coverage.sh` CI gate 校验：

| 编号 | 协议 | 模式 | Content-Type | daemon 路径 | 断言 |
|------|------|------|-------------|------------|------|
| M-1 | Anthropic | SSE | `text/event-stream` | `observe_event` → `scan_assistant_text` | 含注入的记忆写入语义命中并拦截 |
| M-2 | Anthropic | JSON | `application/json` | `handle_anthropic_json_inbound` → `scan_assistant_text` | 同 M-1，JSON 路径不漏 |
| M-3 | OpenAI | SSE | `text/event-stream` | OpenAI SSE → `scan_assistant_text` | 同 M-1 |
| M-4 | OpenAI | JSON | `application/json`（默认 stream=false）| `handle_openai_json_inbound` → `scan_assistant_text` | 同 M-1，默认路径不漏 |

> 隐藏编码用例（zero-width / Unicode invis / base64 包裹的注入）必须在四路由各自验证归一化后仍命中。

### PreToolUse 侧

- `Write` / `Edit` 到记忆路径且内容含注入 → 命中并按处置矩阵拦截。
- daemon 不可达 / 超时 → fail-closed（hook 拒绝）。

### 红队 bypass 用例（必须全部被拦或可解释豁免）

- 提示走私（要求把"忽略既往指令 / 新系统提示"写入记忆）。
- 策略覆盖（要求把"关闭 / 绕过防护"写入记忆）。
- exfil 指令常驻（把外发敏感数据的指令写入记忆）。
- 远程内容直写记忆（先拉远程内容、再原样写入记忆路径）。
- 隐藏编码：零宽字符 / Unicode 不可见控制字符 / base64 包裹的注入。
- 工具信任覆盖（要求"安装此 MCP / 信任此 skill"写入记忆）。
- 跨路由：上述每类在 M-1~M-4 行为一致。

---

## 风险 / 已知 bypass / 误报面

### 已知 bypass

- **写入后篡改**：注入文本先以无害形式落盘，后续会话再小步编辑拼接成完整注入——属于跨会话有状态攻击，超出单次文本/工具调用检测范围，由行为序列类能力另行覆盖。
- **新型编码**：归一化只覆盖已知隐藏编码族（zero-width / Unicode invis / base64 等）；未知编码变体可能绕过，靠规则包随更新通道迭代补齐。
- **非标准记忆路径**：自定义记忆存储路径若不在已知记忆路径集合内，PreToolUse 侧可能漏判；入站文本侧（语义检测）仍是兜底层。
- **语义改写**：用自然语言改写注入意图（不命中关键词）可能绕过——纯模式匹配的固有局限，按保守起步原则不追求 100% 召回。

### 误报面与控制

- 用户合法地把"忽略之前的草稿，按新需求来"等正常表述写进笔记，可能触碰"提示走私"语义——通过规则包的上下文约束（仅在写入记忆路径 + 强注入特征双条件下定级）控制误报，遵循 Critical 拦截 FP < 0.5% 的公理。
- 入站侧归一化可能改变含合法零宽字符的文本展示——归一化只用于匹配判定，不改写转发给 agent 的原始响应体，避免破坏正常内容。
- PreToolUse 侧仅在目标为记忆路径时启用注入内容检测，避免对普通文件写入产生噪声。

### 与既有约束的边界

- 不放宽 fail-closed、不放宽 Critical 不可关、不引入联网 verifier、不伪造协议字段、不装 CA——本能力是既有双层防御架构内的增量，处置上限由系统签名规则与 `critical_lock` 守护。
