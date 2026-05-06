# Orchesis 竞品分析

> 调研日期：2026-05-06
> 调研者：doskey
> 关联决策：[ADR-032](../design/ADR-032-orchesis-strategy.md)（借鉴策略决议）

---

## 0. 摘要

Orchesis 是 AI agent 安全领域的主要竞品之一，与 Sieve 同构（MITM proxy 架构），但用户画像（通用 AI agent）与 Sieve（crypto-native 开发者）有差异。本文整理 2026-05 对其架构、产品策略、营销叙事的全面分析，作为 [ADR-032](../design/ADR-032-orchesis-strategy.md) "选择性借鉴 vs 全面对标 vs 完全无视" 三方案选择的事实基础。

**核心结论**：选择性借鉴——抄低复杂度高价值项（免费工具、循环检测）+ 探索性高价值项（上下文压缩、分阶段管线），明确排除非安全核心项（Thompson Sampling、per-request 预算、auto-heal）。

---

## 1. Orchesis 关键事实

### 1.1 架构形态

- MITM proxy（与 Sieve 同构）
- 17 阶段管线分治：Security / Context Engine / Threat Intel / Cost Optimizer / Observability 等独立阶段
- 各阶段松耦合，可独立 enable/disable

### 1.2 工程实践

- 零依赖单二进制
- 快速安装（一行 curl 即可起服务）
- pinned deps + reproducible build
- 透明日志，所有阶段决策可审

### 1.3 营销叙事

- 主打 "3 impossibility theorems + 26 formal results" —— 把工程承诺包装成数学定理
- 强调形式化基础（formal methods 背景）
- 抓眼球博客标题：
  - "43% of MCP configs run bare shell"
  - "I left my AI agent running overnight. $47,000 bill"

### 1.4 获客漏斗

- **入口**：免费浏览器端工具（MCP Scanner / Security Scorecard / 静态 Proxy 配置检查）—— 零摩擦体验
- **中段**：社区认知建立（GitHub stars / HN 顶帖 / Twitter 影响力）
- **转化**：付费 proxy（核心商业产品）

### 1.5 特色功能清单

| 功能 | Sieve 评估 |
|------|-----------|
| 循环检测（同一 tool_call N 次→阻断） | 高价值，crypto 场景特别适用 |
| 上下文压缩（80–90% token 节省） | proxy 位置天然优势，v3.x 探索 |
| Thompson Sampling 模型路由 | 与 Sieve 多 listener 模型冲突，不抄 |
| Per-request 预算执行 | 非安全核心，不抄 |
| Fleet-level 跨 agent 关联 | v2.x multi-listener 已部分覆盖，跨 agent 关联非 P0 |
| Auto-heal 6 种恢复策略 | 非安全核心，Phase 2 可考虑 |

---

## 2. 与 Sieve 的差异化对比

### 2.1 用户画像

| 维度 | Orchesis | Sieve |
|------|----------|-------|
| 目标用户 | 通用 AI agent 开发者 / 企业 | crypto-native 开发者（钱包、合约审计、链上工具构建者）|
| 核心场景 | 多 agent 编排、cost control、可观测性 | 私钥泄漏防护、地址替换检测、Critical 工具调用 fail-closed |
| 营销战场 | 通用开发者社区（HN / Reddit r/programming） | crypto 开发者社区（Twitter crypto-tw、Mirror、HN）|

### 2.2 技术差异化

- **Sieve 独有**：BIP39 SHA-256 校验（[PRD §9 #4](../prd/sieve-prd-v2.0.md)）、Permit2 签名解析、地址替换 Levenshtein 算法、55 条真实攻击复现
- **Orchesis 独有**：17 阶段管线、Thompson Sampling 路由、形式化方法包装

### 2.3 商业模式

| 维度 | Orchesis | Sieve |
|------|----------|-------|
| 当前阶段 | 已商业化（付费 proxy） | Phase 1 全免费（[ADR-029](../design/ADR-029-free-first-defer-monetization.md)）|
| 主战场 | 海外 SaaS | 海外 + Web3 混合（[ADR-005](../design/ADR-005-overseas-legal-entity.md) 海外公司主体）|

---

## 3. 可借鉴维度详解

### 3.1 免费浏览器端工具（立即可做，GA 前）

**原理**：零成本获取目标社区初始认知。Orchesis 用 MCP Scanner / Security Scorecard 拉用户进漏斗。

**Sieve 候选形态**：
- "Claude Code 安全评分"——上传 `~/.claude/settings.json` + MCP 列表，输出风险报告
- "API Key 泄露扫描"——上传 git diff / 代码片段，扫 OPENAI_API_KEY / ANTHROPIC_API_KEY 等
- "BIP39 助记词检测器"——粘贴文本，本地（浏览器内 WASM）检测助记词模式

**约束**：
- 必须本地运行（与 [ADR-003](../design/ADR-003-local-only-no-cloud-verifier.md) 一致），WASM 优先
- CTA 引导到 Sieve 主产品安装

### 3.2 循环检测（v2.x 落地，2026 Q3）

**原理**：crypto agent 攻击模式中，循环尝试签名 / 转账是高频特征。Orchesis 实现：N 次相同 tool_call → 阻断。

**Sieve 实现路径**：
- 复用现有 IN-SEQ-* 行为序列骨架（[ADR-022](../design/ADR-022-behavior-sequence-window.md)）
- 新增 `IN-SEQ-LOOP` 检测项，参数：N（重复次数阈值）、TTL（窗口时长）
- 重复判定：完全相同的 `tool_call.input` 哈希匹配（v1）；相似模式（参数 Levenshtein 距离）留 v2

**性能预算**：循环检测的 hit-test 是 O(1) 哈希查询，不影响 P99 < 20ms 硬约束。

**ROI 数据**（Orchesis 公开）：
- 比 heartbeat 检查快 450 倍
- 单次 catch 节省 $55–150（API 调用 + agent token）

### 3.3 上下文压缩（v3.x 探索）

**原理**：crypto 开发者常跑长上下文任务（合约分析、链上交易复盘），token 膨胀快。Sieve 在 proxy 位置天然可做压缩——在请求到达模型前裁剪冗余。

**Orchesis 数据**：80–90% token 节省。

**Sieve 落地约束**：
- **不可调用 LLM**——破坏 [ADR-003 完全本地](../design/ADR-003-local-only-no-cloud-verifier.md)（PRD §9 #2）
- **必须在 P99 < 20ms 内完成**——纯规则裁剪可能不够
- v3.x 立项前需 PoC：能否在不调 LLM 的前提下做到 50%+ 节省？

### 3.4 分阶段管线架构（v3.x 重构方向）

**当前 Sieve**：单向线性 pipeline（入站 → 规则匹配 → 出站）。

**Orchesis 17 阶段**：每阶段独立可插拔，便于：
- A/B 测试某个阶段的算法变更
- 选择性 enable/disable（用户按需关闭"成本追踪"等非安全阶段）
- 独立性能 profiling

**Sieve v3.x 候选方向**：
- 行为序列引擎独立成阶段（v2.x 已部分实现）
- 威胁情报阶段独立（与规则引擎解耦）
- 成本追踪阶段独立（用户可关，不影响安全核心）

**不抄项**：Orchesis 的 17 个具体阶段命名不直接复制，按 Sieve 自身领域抽象（crypto 检测阶段独立，token 路由阶段不引入）。

---

## 4. 营销叙事借鉴

### 4.1 "可验证信任" 叙事（替代 Orchesis 数学包装）

Orchesis 用 "3 impossibility theorems" 把工程承诺包装成数学定理。Sieve 当前没有等价的形式化基础，直接抄会失真。

**Sieve 替代叙事**："Self-Custody Trust" = pinned deps + sigstore + reproducible build（[ADR-006](../design/ADR-006-sigstore-reproducible-build.md)）+ 完全本地（[ADR-003](../design/ADR-003-local-only-no-cloud-verifier.md)）

**核心信息**：
- 数学上可验证 + 工程上透明
- "你不只是相信我们，你能验证我们"
- Sieve 是 Rust 单二进制，零依赖叙事天然成立

### 4.2 "代理 vs SDK vs 静态分析" 2×2 对比框架

Orchesis 用 2×2 对比让用户 3 秒理解 proxy 是正确方案。Sieve 可直接复用，加 crypto 维度：

|  | 看得到 prompt | 看不到 prompt |
|---|---|---|
| **懂 crypto** | **Sieve** | 钱包安全产品（MetaMask、Phantom）|
| **不懂 crypto** | LLM 安全产品（Lakera、Promptfoo）| DLP（Forcepoint、Nightfall）|

### 4.3 博客标题策略

Orchesis 标题模板：「具体百分比 + 反直觉事实」。Sieve 候选：

- "我们复现了 55 种 AI agent 攻击——你的 agent 一个都防不住"
- "BIP39 助记词在 AI 对话中泄露的概率比你想象的高 100 倍"
- "我们在 Claude Code 里发现的 7 类 critical 风险，你都中招了"
- "$47,000 是上限？Sieve 看到的 crypto agent 损失中位数是 $312,000"

**约束**：必须基于真实数据（不编造 case），符合 [PRD §11.5](../prd/sieve-prd-v2.0.md) 中文圈不公开商业化营销约束（以英文圈 Twitter / HN / Mirror 为主战场）。

---

## 5. 不借鉴清单（已锁定）

| Orchesis 功能 | 不借鉴理由 |
|--------------|-----------|
| Thompson Sampling 模型路由 | Sieve 不是上游路由器；与 [ADR-026](../design/ADR-026-port-based-listener-routing.md) port-based listener 路由模型冲突；v3.x 可探索但非 P0 |
| Per-request 预算执行 | 非安全核心，低优先级 |
| Fleet-level 跨 agent 关联 | v2.x multi-listener 已支持多 agent（[ADR-026](../design/ADR-026-port-based-listener-routing.md)），但跨 agent 行为关联非 P0 |
| Auto-heal 6 种恢复策略 | 非安全核心，Phase 2 可考虑 |
| 数学定理包装叙事 | Sieve 无形式化基础，强行包装失真；用工程实证叙事（Self-Custody Trust）替代 |

---

## 6. 决策与后续

正式决策见 [ADR-032](../design/ADR-032-orchesis-strategy.md)：选 Option 2（选择性借鉴），分 5 个子决策落地。

后续行动锚点：
- v2.x 验证窗口结束后启动决策 1「立即」档（免费工具 + 循环检测）
- v3.x 立项前完成上下文压缩 PoC（无 LLM 调用约束下的可达节省率）
- GA 前完成「Self-Custody Trust」叙事的 README + landing page 落地

---

## 关联文档

- [ADR-032: Orchesis 借鉴策略决议](../design/ADR-032-orchesis-strategy.md)
- [ADR-022: 行为序列联动窗口](../design/ADR-022-behavior-sequence-window.md)
- [ADR-026: Port-based listener routing](../design/ADR-026-port-based-listener-routing.md)
- [ADR-029: 装机量优先,延后商业化](../design/ADR-029-free-first-defer-monetization.md)
- [ADR-006: Sigstore + Reproducible Build](../design/ADR-006-sigstore-reproducible-build.md)
- [ADR-003: 完全本地，不联网做 verifier](../design/ADR-003-local-only-no-cloud-verifier.md)
