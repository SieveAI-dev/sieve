# ADR-011: Week 12 GA 前 repo 完全私有,不做任何 public 暴露

## 状态
**Accepted**
> 决策日期:2026-04-27
> 范围:GA 之前的 repo / 代码 / 文档可见性策略
> 关联 PRD:[v1.4 §9 #10](../prd/sieve-prd-v1.5.md) (取代该条 Day 1 公开子条款) / §10.1 Week 1(修订) / §11.3 开源策略(保留)
> 取代关系:**Supersedes** PRD v1.3 §9 #10 的 "Day 1 GitHub repo 公开 README + 架构文档" 子条款

## 背景

PRD v1.3 §9 #10 原约束:"Day 1 GitHub repo 公开 README + 架构文档,代码 GA 后开源(MIT)"。

设计意图是用 Day 1 公开建立"自证清白"叙事(详见 [ADR-006 sigstore](./ADR-006-sigstore-reproducible-build.md) 与 PRD §1.2 第 4 句)。但实际权衡发现:

- 营销主动权:Day 1 公开 README + 架构图意味着对手可以提前看到 Sieve 的产品形态、规则集设计、定价方向、营销叙事,12 周内复制出 MVP 不困难
- 闭测期间任何 PR / issue / commit 节奏暴露,会让 KOL 接洽时缺乏"独家性"叙事
- GA 当天才公开,搭配文章 1+2+3 同步发布,传播弹药密度更高
- Day 1 公开 README 的"信任叙事"价值,在 Week 9 闭测启动后由 5-10 个真实用户的口碑替代,无需 Day 1 公开换取

## 决策

**Week 12 GA 之前,GitHub repo 完全私有**。
具体含义:

| 项 | GA 之前 | GA 时(Week 12) |
|---|---------|----------------|
| GitHub repo 可见性 | private | public |
| README / 架构文档 | 不公开 | 一次性公开 |
| 代码许可 | 私有,无许可证发布 | 核心引擎 MIT |
| sigstore CI pipeline | 照常跑(release.yml) | 照常跑 + landing page 提供验证教程 |
| Rekor 透明日志 | CI 跑时仍写入(keyless OIDC) | 照常 + 公开验证命令 |
| 闭测 license key | 通过 Discord 私发 | 试用注册自动发 |

**ADR-006 不受影响**:sigstore 签名 + reproducible build pipeline 仍是 Week 1 hard gate,GA 前跑通确保 GA 时即可对外验证。GA 前 release.yml 不绑定 tag push(改为仅 `workflow_dispatch` 触发),用于 dogfood 和闭测分发,减少 Rekor 透明日志记录 release 节奏。GA 时恢复 tag-based release。

## 影响

### 正面影响

- 营销主动权:文章 1(中转站揭黑)+文章 2(自证清白)+ 代码开源 + sigstore 验证四件事 GA 当天打包发布,传播密度最大化
- 减少抄袭风险:对手在 12 周内拿不到产品形态 / 规则设计参考
- 闭测独家性:5-10 个用户的口碑成为 GA 时唯一的早期反馈,KOL 接洽更有"内测专属"叙事
- 内部迭代速度:不需要为"对外可读性"对内部文档进行二次润色

### 负面影响

- "Day 1 公开"的信任叙事让位 — Week 9 闭测启动后必须用 5-10 个真实用户口碑补回。**缓解**:闭测 SLA 严格(24h 反馈),Discord 频道公开
- sigstore CI 跑产生的 Rekor 透明日志条目仍可被外部检索(GitHub Actions OIDC 签名机制) — 项目存在 + release 节奏可能被推断。**缓解**:Week 1-11 的 release.yml 仅在手动 `workflow_dispatch` 触发,不绑定 tag,降低 Rekor 痕迹密度;GA 时正式开始 tag-based release
- LICENSE 文件原先以"文档先公开 + 代码后开源"为论证基础,需重写双许可论证,改为"GA 时同步公开"

### 需要更新的文档

- [x] `docs/prd/sieve-prd-v1.5.md` §9 #10 + §10.1 Week 1 任务清单
- [x] `CLAUDE.md` 硬约束节 #10 + Week 1 关键路径节
- [x] `.cursorrules` §二 第 10 条
- [x] `tasks/roadmap.md` Week 1 / Week 12 任务清单
- [x] `docs/changelog/CHANGELOG.md` 新增 ADR-011 条目
- [x] `docs/design/ADR-INDEX.md` 表格新增 ADR-011 行
- [x] `docs/design/ADR-006-sigstore-reproducible-build.md` "相关文档" 节追加 ADR-011 引用
- [x] `LICENSE` 重写 "Why Two Licenses" 论证 + 删除所有 "Day 1" 措辞
- [x] `README.md` 项目状态行追加 "Week 12 GA 前 repo 私有"
- [x] `.github/workflows/release.yml` on: 改为仅 workflow_dispatch

## 相关文档

- [ADR-006 Sigstore + Reproducible Build](./ADR-006-sigstore-reproducible-build.md)
- [PRD v1.4 §9 工程硬约束](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
- [PRD v1.4 §11.3 开源策略](../prd/sieve-prd-v1.5.md#113-开源策略)
- [tasks/roadmap.md Week 12 GA 发布](../../tasks/roadmap.md)
