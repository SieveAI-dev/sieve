# ADR-035: 代码 Apache-2.0 + 文档 CC BY-NC-SA 4.0 双许可,即刻生效不等 GA

## 状态
**Accepted**
> 决策日期:2026-06-19
> 范围:仓库代码与文档的正式开源许可证选择 + 生效时点
> 关联:[ADR-011](./ADR-011-private-until-ga.md)(GA 前私有 + 时点)/ [ADR-029](./ADR-029-free-first-defer-monetization.md)(免费优先,延后商业化)/ [ADR-006](./ADR-006-sigstore-reproducible-build.md)(签名分发,自证清白叙事)
> 关联 PRD:[v2.0 §11.3](../prd/sieve-prd-v2.0.md)(开源策略,本决策取代其中 "core engine MIT at GA" 表述)
> 取代关系:**Amends** PRD §11.3 的 "core engine MIT" 表述(license 改 Apache-2.0);**Amends** [ADR-011](./ADR-011-private-until-ga.md) 的 "private until GA / GA 一次性公开" 时点(仓库已提前公开)

## 背景

仓库现已 **public(GA 前闭测,pre-GA closed beta)**。这与两份历史决策的前提冲突,需要正式裁决:

- **PRD §11.3** 原计划 "core engine MIT at GA"——代码许可证为 MIT,且在 GA 时才公开。
- **[ADR-011](./ADR-011-private-until-ga.md)** 原计划 "Week 12 GA 前 repo 完全私有,GA 时一次性公开 + 核心引擎 MIT"。

现实已变:为兑现 Sieve "可验证而非仅信任"(verifiable, not just trusted)的信任叙事——安全产品要让用户能读源码亲自审计,源码已先于 GA 公开。一旦仓库 public,就必须有正式 OSI 认可的许可证,否则:

- 法律上,无许可证的公开代码默认 "All rights reserved",任何人不得合法使用 / fork / 验证,与公开源码的初衷直接矛盾。
- GitHub 无法识别许可证徽章,削弱 "开源可审计" 的可信度信号。

因此需要在仓库公开的当下(而非 GA)就确定并落地正式许可证。同时,原 "MIT" 的选择也需重新评估:Sieve 是安全工具,MIT 不含显式专利授权,而 Apache-2.0 提供明确的专利授权与专利反诉终止条款,更契合安全产品对供应链与专利风险的审慎要求。

## 决策

**1. 代码采用 Apache License 2.0。**

- 相较 MIT,Apache-2.0 增加**显式专利授权**(贡献者授予用户专利许可)与**专利反诉终止**条款,更适合安全工具——降低下游用户与项目自身的专利风险。
- 仓库根 `LICENSE` = Apache-2.0 **全文**,便于 GitHub 自动识别并渲染 `license: Apache-2.0` 徽章。
- `NOTICE` 文件标注归属(Sieve / SieveAI-dev),符合 Apache-2.0 §4(d) 对 NOTICE 的要求。

**2. 文档采用 CC BY-NC-SA 4.0。**

- 范围:`docs/` 全部内容,以及 README / CLAUDE.md 等所有非源码的 Markdown / 配置说明文档。
- 含义:**署名(BY)+ 非商业(NC)+ 相同方式共享(SA)**——任何人可阅读 / 翻译 / 引用 / 二次分发,但**禁止商业再打包**(如把 Sieve 文档包装成付费课程 / 付费知识库牟利)。
- 仓库根 `LICENSE-DOCS` 描述文档许可范围与 CC BY-NC-SA 4.0 链接。

**3. 两者即刻生效,不等 GA。**

- 仓库公开的当下许可证即生效,不再像 ADR-011 那样把许可证发布绑定到 GA 时点。
- 贡献模式:**inbound = outbound**——任何外部贡献默认以 Apache-2.0 提交(代码)/ CC BY-NC-SA 4.0 提交(文档),与仓库出站许可一致,无需额外 CLA。

## 影响

### 正面影响

- GitHub 正确识别并渲染 `Apache-2.0` 许可证徽章,强化 "开源可审计" 信号,兑现 "verifiable not just trusted" 叙事。
- 任何人可在 Apache-2.0 下合法使用 / fork / 验证代码,**含明确的专利授权**——下游用户专利风险更低,契合安全工具定位。
- 文档 CC BY-NC-SA 4.0 允许自由翻译与传播(利于社区与多语言扩散),同时**禁止商业再打包**,保护项目对自有内容的商业权益。
- inbound = outbound 简化贡献流程,无 CLA 摩擦。
- 与 [ADR-029](./ADR-029-free-first-defer-monetization.md) 一致:Phase 1 代码完全免费 + 开源;Phase 2 的高级规则集仍可闭源(高级规则集不在本仓,Apache-2.0 不对其产生传染——Apache-2.0 非 copyleft)。

### 负面影响

- 切换为 Apache-2.0 后,凡历史文档 / README / 徽章中写 "MIT" 的地方都需同步改为 "Apache-2.0",存在一次性清理成本(本 ADR 仅记录决策,文档清理另行执行,见下)。
- 文档 NC(非商业)条款意味着不能像完全宽松许可那样被任意商用复用——这是有意取舍,优先保护项目内容商业权益而非最大化复用面。
- ADR-011 "GA 一次性打包公开换取最大传播密度" 的营销叙事让位于 "提前公开换取可验证信任" ——传播弹药从 "GA 当天集中引爆" 调整为 "公开即可审计" 的持续信任建设(时点取舍,非缺陷)。

### 需要更新的文档

- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行 + ADR-011 行追加 "时点被 ADR-035 修订" 备注(本次完成)
- 仓库根 `LICENSE`(Apache-2.0 全文)/ `LICENSE-DOCS`(CC BY-NC-SA 4.0 说明)/ `NOTICE`(归属)——另行落地
- [docs/prd/sieve-prd-v2.0.md](../prd/sieve-prd-v2.0.md) §11.3:"core engine MIT" 改为 "Apache-2.0",并引用本 ADR——另行落地
- [README.md](../../README.md) / [README.zh-CN.md](../../README.zh-CN.md):license 徽章与说明改为 `Apache-2.0` + 文档 CC BY-NC-SA 4.0——另行落地
- [docs/changelog/CHANGELOG.md](../changelog/CHANGELOG.md):新增 license 决策条目——另行落地

> 本 ADR 仅修改本文件 + ADR-INDEX,其余文档清理在后续任务中执行。

## 相关文档
- [ADR-011: Week 12 GA 前 repo 完全私有](./ADR-011-private-until-ga.md)
- [ADR-029: 装机量优先,延后商业化](./ADR-029-free-first-defer-monetization.md)
- [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](./ADR-006-sigstore-reproducible-build.md)
- [PRD v2.0 §11.3 开源策略](../prd/sieve-prd-v2.0.md)
