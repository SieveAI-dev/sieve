# ADR-032: 选择性借鉴 Orchesis 的工程实践与营销叙事

## 状态
**Proposed**
> 决策日期：2026-05-06（草案，未通过）
> 范围：竞品 Orchesis 的借鉴策略决议——立即抄 / 探索性抄 / 永不抄
> 关联：[ADR-022](./ADR-022-behavior-sequence-window.md) / [ADR-029](./ADR-029-free-first-defer-monetization.md) / [ADR-027](./ADR-027-network-jail-enforcement.md) / [ADR-026](./ADR-026-port-based-listener-routing.md) / [ADR-006](./ADR-006-sigstore-reproducible-build.md) / [ADR-003](./ADR-003-local-only-no-cloud-verifier.md)
> Tags: competitive-strategy, roadmap, marketing

> **本 ADR 仅记录决策。完整竞品分析（Orchesis 架构、特色功能清单、与 Sieve 的差异化对比、营销叙事借鉴细节）见 [`research/2026-05-06-orchesis-analysis.md`](../research/2026-05-06-orchesis-analysis.md)。**

## Context

Orchesis 是 AI agent 安全领域的主要竞品（MITM proxy 同构架构）。2026-05 完成全面调研后，需要用 ADR 锁定借鉴策略，避免后续在 v2.x 收尾或 v3.x 启动时反复争论。

立 ADR 的三个动机：

1. **路线图取舍要锁定**：Orchesis 17 阶段并非全部都该抄，需明确「立即 / 探索 / 永不」三档
2. **避免与 [ADR-029](./ADR-029-free-first-defer-monetization.md) 冲突**：免费工具属于"装机量优先"延伸，但需要工程资源；要明确投入比例
3. **不污染 v2.x 节奏**：v2.x 已全部落地等用户验证，任何借鉴项不能塞进 v2.x

## Options Considered

### Option 1：全面对标——抄 Orchesis 17 阶段管线

把 Sieve 重构为 Orchesis 同构架构，所有功能（路由、预算、auto-heal、跨 agent 关联）齐头并进。

- 优点：架构对等，营销可直接对标
- 缺点：工程量极大（一人公司不可承受）；Sieve 与 Orchesis 用户画像差异（crypto dev vs 通用 AI agent）决定大量阶段 ROI 低
- **结论：排除**

### Option 2：选择性借鉴——按"复杂度 × 差异化价值"二维矩阵筛选

只抄低复杂度高价值项 + 探索性高价值项，明确排除非安全核心项。

- 优点：对一人公司节奏友好，与 [ADR-029](./ADR-029-free-first-defer-monetization.md) 装机量优先对齐
- 缺点：放弃 Orchesis 的"全栈"叙事
- **结论：采纳**（Phase 1 不做企业销售，比较劣势可接受）

### Option 3：完全无视——纯 crypto 垂直路线

不借鉴任何 Orchesis 项。

- 优点：差异化最强
- 缺点：错过免费工具获客漏斗与循环检测（crypto 场景天然适用，是加分项不是分散）
- **结论：排除**

## Decision

选 **Option 2**（选择性借鉴）。具体落地分 5 个子决策：

### 决策 1：三阶段学习路线

| 优先级 | 借鉴项 | 目标版本 | 复杂度 | 差异化价值 |
|--------|-------|---------|--------|-----------|
| **立即** | 免费浏览器端安全扫描工具 | GA 前 | 低 | 社区认知 |
| **立即** | 循环检测（同一 tool_call N 次→阻断） | v2.2+ | 低 | 高（crypto 场景特别适用）|
| **探索** | 上下文压缩（80–90% token 节省） | v3.x | 高 | 极强（proxy 位置天然优势）|

详细原理与落地约束见 [research §3](../research/2026-05-06-orchesis-analysis.md#3-可借鉴维度详解)。

### 决策 2：v3.x 分阶段管线架构方向

v2.x 保持现有线性架构。v3.x 候选：将 Pipeline 重构为分阶段架构（行为序列、威胁情报、成本追踪各自独立成阶段）。

不直接复制 Orchesis 的 17 个阶段命名，按 Sieve 自身领域抽象。

### 决策 3：明确不借鉴清单

| 功能 | 理由 |
|------|------|
| Thompson Sampling 模型路由 | 与 [ADR-026](./ADR-026-port-based-listener-routing.md) port-based listener 路由模型冲突 |
| Per-request 预算执行 | 非安全核心 |
| Fleet-level 跨 agent 关联 | v2.x multi-listener 已部分覆盖，跨 agent 关联非 P0 |
| Auto-heal 6 种恢复策略 | 非安全核心 |

### 决策 4：营销叙事——"Self-Custody Trust"

不直接抄 Orchesis 的"3 impossibility theorems"数学包装（Sieve 无形式化基础），改用工程实证叙事：

**Self-Custody Trust** = pinned deps + sigstore + reproducible build（[ADR-006](./ADR-006-sigstore-reproducible-build.md)）+ 完全本地（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）

复用 Orchesis "代理 vs SDK vs 静态分析" 2×2 对比框架，加 crypto 维度。

### 决策 5：博客内容策略

基于 1951 测试样本 + 55 条真实攻击复现，产出差异化内容。具体标题模板见 [research §4.3](../research/2026-05-06-orchesis-analysis.md#43-博客标题策略)。

约束：内容必须基于真实数据（不编造 case），符合 [PRD §11.5](../prd/sieve-prd-v2.0.md) 中文圈不公开商业化营销约束。

## Consequences

### 正面影响

1. **免费浏览器端工具**：零成本社区认知，与 [ADR-029](./ADR-029-free-first-defer-monetization.md) 对齐
2. **循环检测**：低复杂度高价值，复用现有 [ADR-022](./ADR-022-behavior-sequence-window.md) 行为序列骨架
3. **上下文压缩 v3.x**：proxy 位置天然优势
4. **营销叙事差异化**：工程实证 vs 数学包装
5. **博客内容池**：1951 样本 + 55 攻击复现是天然资产

### 负面影响

1. 上下文压缩增加代理延迟，需评估 P99 预算
2. 循环检测判定逻辑不平凡（完全相同 vs 相似模式），需 v2.2 SPEC 明确状态机
3. 免费浏览器端工具与主产品割裂，独立维护成本
4. 抓眼球标题需要持续产出，否则单篇爆款后流量断崖

### 风险与缓解

- **风险**：v3.x 分阶段管线重构成本超预期。**缓解**：v2.x 验证窗口结束后先做 RFC，确认线性架构有明显瓶颈再立项
- **风险**：Orchesis 推出 crypto 垂直功能直接竞争。**缓解**：Sieve crypto 检测护城河（[PRD §9 #4](../prd/sieve-prd-v2.0.md)）短期不可复制
- **风险**："Self-Custody Trust" 叙事在中文圈难落地。**缓解**：中文圈用"本地审计 + 可验证构建"中性表述

### 需要更新的文档（决策通过后）

- `docs/prd/sieve-prd-v3.0.md`（创建时纳入决策 1 探索项 + 决策 2 v3.x 管线方向）
- [docs/design/architecture.md](./architecture.md)（v3.x 章节增量）
- [tasks/roadmap.md](../../tasks/roadmap.md)（v2.2 加入循环检测，GA 前里程碑加入免费工具）
- GA 营销物料（决策 4 / 决策 5）
- CHANGELOG（[STRATEGY] Orchesis 借鉴决策）

## 待澄清 / 阻塞 ADR 通过的事项

本 ADR 暂留 **Proposed**，**通过前需回答**：

1. 免费浏览器端工具的具体形态与归属仓库（独立仓 vs sieve workspace 新增 crate）
2. 循环检测 N 的取值——需 dogfood 期间收集 ≥ 50 条 crypto 工作流真实样本，避免 FP 失控（[PRD §9 #7](../prd/sieve-prd-v2.0.md) Critical FP < 0.5%）
3. 循环检测「相似模式」判定是否纳入决策 1 范围
4. v3.x 上下文压缩的延迟预算——纯规则裁剪能否在 P99 < 20ms 内做到 50%+ 节省？需 PoC
5. "Self-Custody Trust" vs 中文圈本地化叙事的统一程度
6. 博客产出节奏与作者——一人公司无 DevRel，能否维持月更
7. **本 ADR 通过前不动代码**——只完成调研记录与决策框架

## References

- [research/2026-05-06-orchesis-analysis.md](../research/2026-05-06-orchesis-analysis.md)（完整竞品分析，Orchesis 关键事实 / 差异化对比 / 借鉴维度详解 / 营销叙事 / 不借鉴清单）
- [ADR-022: 行为序列联动窗口](./ADR-022-behavior-sequence-window.md)（循环检测的现有骨架）
- [ADR-026: Port-based listener routing](./ADR-026-port-based-listener-routing.md)（与 Thompson Sampling 路由的冲突点）
- [ADR-029: 装机量优先,延后商业化](./ADR-029-free-first-defer-monetization.md)（免费工具策略对齐）
- [ADR-006: Sigstore + Reproducible Build](./ADR-006-sigstore-reproducible-build.md)（Self-Custody Trust 叙事的工程基础）
- [ADR-003: 完全本地，不联网做 verifier](./ADR-003-local-only-no-cloud-verifier.md)（信任边界基础）
- [PRD v2.0 §11 商业化与营销](../prd/sieve-prd-v2.0.md)
