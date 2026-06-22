# ADR-046: 有状态出站 exfil 链检测家族——IN-SEQ 从 3 条 kill chain 升级为链族（机制 stub）

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：`sieve-core::sequence` 链族扩展 + `sieve-core::pipeline::inbound` 四路由记录对等；完全继承 [ADR-022](./ADR-022-behavior-sequence-window.md) 的保守起步约束（GA 默认关闭 / 仅 StatusBar / 升级为 Block 需新 ADR）；具体链特征与触发条件由签名规则包定义、随更新通道分发，不在本文公开
> 关联：[ADR-022](./ADR-022-behavior-sequence-window.md)（行为序列窗口）、[ADR-025](./ADR-025-content-type-routing-matrix.md)（四路由矩阵）、[ADR-007](./ADR-007-fail-closed-critical-actions.md)（fail-closed）、[ADR-024](./ADR-024-rules-engine-abstraction.md)（规则引擎抽象）

---

## 背景

### 单步工具检测看不出多步 exfil kill chain

入站单次检测对每个 tool_use 独立评估。ADR-022 已识别出"多步 kill chain 中每一步单独看都良性、组合起来才是攻击"的盲区，并落地了 3 条序列规则（IN-SEQ-01-RECON-EXFIL / IN-SEQ-02-CLEANUP-AFTER-ATTACK / IN-SEQ-03-PERSISTENCE-CHAIN，见 `crates/sieve-core/src/sequence/detector.rs:48/69/90`）。

但这 3 条只覆盖了出站数据窃取（exfil）链型谱的一小部分。真实攻击场景里，被诱导的 agent 把"读取机密 → 外发"这条核心 kill chain 拆成多种变体来规避单步检测：

- **读 secret 后直接外发**：`Read(".env")` / 读 keystore → 后续 upload / webhook / 发布动作。这是 IN-SEQ-01 已覆盖的最基础形态。
- **打包后上传**：先把机密目录归档（压缩为单文件），再上传归档——归档动作单独看是正常打包，上传单独看是正常发布。
- **clipboard 桥接**：把机密写进剪贴板，由链外的另一动作取走，规避"读→直接外发"的直接相邻关系。
- **污染 build/发布产物**：把机密混进将被发布的构建产物目录，借正常的发布流程把数据带出。
- **跨 agent 拆分**：读 secret 的执行体与外发的执行体不是同一个，把一条链拆到两个 actor 上以躲过"同一上下文内读后发"的判定。
- **延迟 / 后台 exfil**：先落地一个持久化或后台机制，由它在稍后时点完成外发，把读与发在时间上拉开。
- **编码即前兆**：在外发前对机密做编码 / 加密 / 重编码（信号源），单独看是无害的数据处理，但与"读机密 + 即将外发"相邻时是明确的 exfil 前兆。

现有 3 条 kill chain 无法识别上述任意一种打包 / 桥接 / 拆分 / 延迟变体——它们都属于"出站 exfil"这一意图族，但形态各异，逐条手写规则既不可扩展也守不住盲区。

### 触发的真实攻击场景

agent 被入站响应里的注入诱导，先读取本地钱包 keystore 目录里的机密，再把它压缩成一个归档文件，最后通过一次"看似正常的发布"把归档带出本机。三步里每一步——读文件、打包、发布——单独评估都不构成 Critical，单步规则与现有 IN-SEQ-01（要求"读机密"与"网络外发"直接构成两步链）都不触发。这正是把单一 exfil 意图拆成多步良性外观操作来规避检测的典型手法。

### 为什么是家族扩展而非新写一堆规则

这些变体共享同一抽象：**有状态地观察一条出站数据流，跨多步把"接触机密"与"使数据离开本机"关联起来**。ADR-022 的滑动窗口已经提供了所需的全部基础设施（会话隔离、结构化特征、不存原始 input、四路由记录点），缺的只是把链型谱系统性扩展开。把它建模成"有状态出站 exfil 链检测家族"，比逐条堆叠规则更可维护，也让具体链配方可以随签名规则包独立演进而不动引擎。

---

## 决策

**把行为序列检测从 ADR-022 的 3 条固定 kill chain 扩展为"有状态出站 exfil 链检测家族"：复用现有 `ToolUseSequence` 滑动窗口与会话隔离机制，覆盖打包后上传 / clipboard 桥接 / 污染发布产物 / 跨 agent 拆分 / 延迟后台外发 / 编码前兆等 exfil 链型；家族完全继承 ADR-022 的保守起步约束（GA 默认关闭、仅 StatusBar 通知、不引入新 Block 路径，升级为 Block 需 4 周 ≥50 样本 + FP<0.5% + 新 ADR），具体链特征与触发条件由签名规则包定义、随更新通道分发。**

机制要点：

1. **复用滑动窗口**：链族沿用 `crates/sieve-core/src/sequence/mod.rs:64` 的 `ToolUseSequence`（窗口大小 + TTL 来自 `SequenceConfig`），不新增独立窗口、不持久化、daemon 重启即清空。
2. **结构化特征，不存原始 input**：链族基于 `ToolUseRecord` 的结构化特征枚举推断，沿用 ADR-022 的隐私模型；具体新增特征字段与判别启发式由签名规则包定义，不在本文公开。
3. **会话隔离**：每个 HTTP 连接独立维护 `ToolUseSequence` 实例，禁止跨连接共享——这对"跨 agent 拆分"链型尤其关键，跨 agent 关联只在同一会话窗口内成立，绝不跨连接污染。
4. **四路由对等触发**：序列记录点必须在 M-1~M-4 四条 content-type 路由上对等执行（见下文方案与验收）。
5. **链配方在签名规则包**：具体特征字段 / 触发关键词 / 链组合配方由签名规则包定义，通过规则引擎抽象（[ADR-024](./ADR-024-rules-engine-abstraction.md)）加载、随更新通道分发；本 ADR 只锁定"家族扩展 + 机制接线 + 保守起步约束 + 四路由不变量"。

---

## 硬约束逐条核对

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed** | ✔ | 本家族不触碰 fail-closed 路径。Critical 单次检测命中仍按 [ADR-007](./ADR-007-fail-closed-critical-actions.md) 立即 Block，不等序列完成；序列链族只产生 StatusBar 通知，既不放宽也不替代 fail-closed 判定。 |
| **Critical 在所有版本不可关** | ✔ | 序列链族是 ADR-022 的 beta 能力（feature flag 默认关闭），与 Critical 单次拦截完全正交。Critical 规则在序列 feature 关闭时照常全功能生效；开启序列也不会 suppress 任何 Critical。 |
| **BIP39 必须 SHA-256 checksum** | ✔（不适用） | 本 ADR 不触碰出站 crypto key 检测；BIP39 checksum 验证（`crates/sieve-rules/src/bip39.rs`）路径不变。 |
| **绝不联网做 verifier** | ✔ | 链检测全部在本地滑动窗口内推断，无任何外部校验调用。链配方随更新通道（manifest + 规则正文）分发，属既有允许出站，不构成运行时 verifier。 |
| **不在 API 协议层撒谎 / 不伪造 tool_use** | ✔ | 链族只**观察**已聚合完成的 tool_use 结构化特征并记录，不构造、不改写、不伪造任何 tool_use / stop_reason / id / usage；StatusBar 通知是 Sieve 自报事件，不冒充模型。 |
| **不装本地 CA 做 MITM** | ✔（不适用） | 链检测在既有代理层的入站解析后执行，不引入任何 CA / Network Extension / 系统 proxy 改动。 |
| **出站脱敏自动改写不弹窗** | ✔（不适用） | 本家族属入站行为序列检测，与出站脱敏（OUT-*）路径无交集，不改动其自动改写 + 状态栏通知行为。 |
| **四路由矩阵适用** | ✔ **适用** | 序列记录点必须 M-1~M-4 对等覆盖，否则 JSON 模式用户的 tool_use 不进窗口、链检测静默失效（ADR-025 / ADR-022 §5 双路径不变量）。详见方案与验收。 |
| **ADR-022 保守起步全部约束** | ✔ **完全继承并重申** | GA 默认关闭（feature flag `sequence_detection = false`）；新增链型 severity=High，disposition=StatusBar，**不引入任何新 Block 路径**；升级为 Block 类需 4 周 ≥50 样本 + FP<0.5% + 新 ADR，三条缺一不可，未附新 ADR 的 PR 由 CI hard-fail 阻塞。 |
| **Critical 拦截 FP<0.5%** | ✔ | 本家族不产生 Critical disposition，不进入 Critical FP 预算；若未来某链型申请升级为 Block，须先满足 FP<0.5% 红线（与 Critical 对齐）方可在新 ADR 中评审通过。 |

---

## 方案（与现有 crate / 模块的接线点）

全部接线点已在现有代码就位（ADR-022 落地），本 ADR 在其上做链族扩展，不新增窗口、不改记录点签名。

| 用途 | 接线点（file:函数 / 行） |
|------|--------------------------|
| 滑动窗口数据结构 | `crates/sieve-core/src/sequence/mod.rs:64` `ToolUseSequence`（`record()` 在 `mod.rs:88`，会话隔离 + TTL 剔除） |
| 结构化特征提取 | `crates/sieve-core/src/sequence/feature.rs:47` `extract_record()`（`ToolClass` / `PathCategory` 枚举，不存原始 input） |
| 链检测入口 | `crates/sieve-core/src/sequence/detector.rs:28` `detect_kill_chains()`（现有 3 条；家族扩展在此挂入新链型，保持无副作用、不改窗口状态） |
| 序列记录点（feature gate） | `crates/sieve-core/src/pipeline/inbound.rs:128` `InboundFilter::record_tool_use_into_sequence()`；feature 关闭时为 no-op，零开销 |
| 序列检测调用 | `crates/sieve-core/src/pipeline/inbound.rs:160` `InboundFilter::detect_sequence_hits()` |
| 链配方加载 | 规则引擎抽象 [ADR-024](./ADR-024-rules-engine-abstraction.md)（`crates/sieve-rules/src/engine`）；具体链特征 / 触发条件由签名规则包定义、随更新通道分发，可热替换不动引擎代码 |

**四路由记录点（M-1~M-4 对等，ADR-025 / ADR-022 §5）**——`record_tool_use_into_sequence` 必须在四条 content-type 路由上各被调用一次，daemon 侧接线点：

| 编号 | 路由 | daemon 接线点（`crates/sieve-cli/src/daemon.rs`） |
|------|------|---------------------------------------------------|
| M-1 | Anthropic SSE | `forward_with_inbound_inspection`（SSE 聚合 `content_block_stop` 后） |
| M-2 | Anthropic JSON | `handle_anthropic_json_inbound`（解析 `tool_use` block 后） |
| M-3 | OpenAI SSE | `forward_with_openai_inbound_inspection`（SSE delta 拼接 `tool_calls` 后） |
| M-4 | OpenAI JSON（stream=false 默认） | `handle_openai_json_inbound`（解析 `tool_calls` 后） |

链族不改变这四个记录点的接口，只扩展窗口内被 `detect_kill_chains` 识别的链型集合——因此四路由对等关系由现有记录点天然继承，新增链型不需要改动 daemon。

---

## 分步实施（每步可独立 ship + 独立测试）

每步均在 `sequence_detection` feature gate 内，GA 默认关闭，独立 ship 不影响 GA 用户。

1. **链族框架骨架**：在 `detector.rs` 抽出"出站 exfil 链族"分组结构，把现有 3 条规则归入该族，新增链型的挂入点（trait / 注册表）就位。验证：现有 3 条 IN-SEQ-* 单测全绿，新骨架不改变既有命中行为。
2. **结构化特征扩展**：在 `feature.rs` 的特征提取中补齐链族所需的新结构化特征枚举（归档 / 剪贴板桥 / 发布产物 / 编码前兆 / 跨 actor 标记等的结构化抽象；具体字段由签名规则包配方对应）。验证：`extract_record` 对各特征的提取有正反例单测，确认仍不存原始 input。
3. **打包后上传 + 编码前兆链型**：挂入"归档→外发"与"编码→即将外发"链型。验证：正例（归档/编码后外发触发）+ 反例（仅归档无外发不触发）单测。
4. **clipboard 桥 + 污染发布产物链型**：挂入剪贴板桥接与发布产物污染链型。验证：正例 + 反例单测。
5. **跨 agent 拆分 + 延迟后台 exfil 链型**：挂入跨 actor 拆分与延迟/后台外发链型。验证：正例（同会话窗口内拆分关联触发）+ 反例（跨连接不关联）单测，强校验会话隔离。
6. **四路由集成测试补齐**：在 `crates/sieve-cli/tests/` 为每个新链型 ID 补齐 M-1~M-4 各一个 test case，纳入 `content_type_matrix` 覆盖。验证：四路由集成测试全绿，CI 路由覆盖检查通过。

每步均可单独提 PR、单独 ship、单独回归。

---

## 验收标准

### 1. 单元 / 集成测试

- 每个新链型有正例（构造对应攻击序列在窗口内正确触发对应 IN-SEQ-* 链 ID）与至少两个反例（缺关键步骤不触发 / 步骤顺序错位不触发）。
- 会话隔离断言：跨 agent 拆分链型在**同一连接窗口**内可关联触发，在**不同连接**间绝不关联（跨连接不污染）。

### 2. 四路由 content_type_matrix（M-1~M-4，ADR-025）

每个新链型 ID 必须在四类 content-type 组合的集成测试里各被覆盖一次，命名沿用 `<功能>_<协议>_<模式>_<语义>`：

| 编号 | 协议 | 响应模式 | 验收：mock 出站 exfil 链序列在该路径触发对应链型 |
|------|------|---------|---------------------------------------------------|
| M-1 | Anthropic | text/event-stream（SSE） | ✅ tool_use 进窗口 + 链型触发 + StatusBar 通知 |
| M-2 | Anthropic | application/json（非流式） | ✅ 同上 |
| M-3 | OpenAI | text/event-stream（stream=true） | ✅ tool_calls 进窗口 + 链型触发 |
| M-4 | OpenAI | application/json（stream=false 默认） | ✅ 同上 |

`scripts/check_routing_coverage.sh` CI gate 验证每个新链型 ID 在 M-1~M-4 四类 test case 里各出现至少一次；覆盖不全 CI 失败、PR 不可合并。任何只挂 SSE 不挂 JSON 的链型视为 P0 漏洞。

### 3. 红队 bypass 用例（必须全部正确触发或被明确判定为超范围）

- **打包后上传**：接触机密 → 归档为单文件 → 发布/上传归档，必须触发 exfil 链通知。
- **clipboard 桥**：机密写入剪贴板 → 后续外发动作取走，必须触发。
- **污染发布产物**：机密混入构建/发布产物目录 → 走发布流程外带，必须触发。
- **跨 agent 拆分**：读机密的 actor 与外发的 actor 不同（同会话窗口内），必须触发；跨连接拆分明确不关联（验证会话隔离，不误报）。
- **延迟/后台 exfil**：先落持久化/后台机制 → 稍后由其完成外发，在窗口 TTL 内必须触发。
- **编码前兆**：外发前对机密做编码/加密/重编码，作为相邻信号增强链判定。
- **反例（防误报）**：纯打包无机密接触、纯编码无外发意图、跨连接的不相关操作——必须不触发。

### 4. 保守起步守护

- CI 验证所有新链型 disposition=StatusBar，无任何新 Block 路径引入。
- CI 验证 `sequence_detection` 默认关闭；feature 关闭时记录点为 no-op、零开销。
- 任何把链型 disposition 改为 Block 的 PR，未附满足"4 周 ≥50 样本 + FP<0.5%"的新 ADR 即被 CI hard-fail 阻塞。

---

## 风险 / 已知 bypass / 误报面

### 已知 bypass

1. **窗口外拉长时间轴**：把读机密与外发拉到超过窗口 TTL 之外，链关联失效。这是滑动窗口模型的固有上限；缓解靠 TTL 取值权衡（保守起步偏短防误报），不靠无限期记忆。
2. **跨连接拆分**：会话隔离是安全不变量（防跨会话污染），但也意味着攻击者若能把读与发分到两条独立连接，单窗口无法关联。这是隔离不变量与跨连接关联能力之间的有意取舍——绝不为提高 recall 而放弃会话隔离。
3. **特征规避**：攻击者用引擎结构化特征未建模的新手法（如不常见的归档/编码工具）外发。缓解靠链配方随签名规则包独立迭代补盲，不动引擎。

### 误报面

1. **合法的"读配置后上传"工作流**：CI/部署脚本读 `.env` 后正常发布，可能与 exfil 链结构相似。这正是 ADR-022 把序列检测设为 beta 默认关闭、仅 StatusBar 不阻断的核心理由——误报只带来一次通知打扰，不阻断合法操作。
2. **编码/归档的高频正常使用**：编码与打包是开发中的高频正常动作，单独绝不触发；只有与"接触机密 + 即将外发"相邻时才作为链信号，双/多条件降误报。
3. **链族扩展带来的 FP 累积**：链型越多，整体误报面越大。家族化的好处是每条链型的 FP 可独立在真实 dogfood 数据上度量；任何链型在升级为 Block 前必须先通过 FP<0.5% 红线，未达标只能停留在 StatusBar。

### 实施风险

- **四路由欠覆盖**：新链型若漏挂某条路由，该 content-type 的用户链检测静默失效——由 ADR-025 的 CI 路由覆盖 gate 守护，视为 P0。
- **会话隔离回归**：跨 agent 拆分链型依赖会话窗口归属正确，隔离实现一旦回归可能跨连接误关联或漏关联——集成测试必须同时断言"同会话触发"与"跨连接不关联"两侧。
