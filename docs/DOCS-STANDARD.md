# 文档体系规范 v2.0

> 适用范围：本仓库（`sieve` daemon / Rust workspace）所有 Markdown 文档
> 上次更新：2026-05-03
> 上游来源：全局 `~/.claude/CLAUDE.md` DOCS-STANDARD v2.0

---

## 0. 原则

> 安全性 > 一致性 > 完整性 > 简洁性

- **一文件一职责**。一个 ADR 一个决策，一个 SPEC 一个模块。散落的小文件必须归目录；单文件不建目录。
- **ADR 只增不改**。决策变了写新 ADR，旧的标记为「被取代」。**禁止**在 ADR 文件里改写已发布的决策。
- **research/（外部事物调研） vs review/（本项目产出物评审）严格区分**。
- **CLAUDE.md 引用而非复制**，控制在 300 行以内。
- **文档正文中文，文件名全英文**。
- **版本化文档（PRD、architecture、api-reference、Spec）**第一行下面标注 `> Version: vX.Y — YYYY-MM-DD`。

---

## 1. 目录结构

```
docs/
├── DOCS-STANDARD.md          ← 本文件
├── glossary.md               ← 术语表
├── requirements/
│   ├── PRD-sieve.md          ← 活动版本入口指针（指向当前 PRD 版本）
│   └── user-stories.md
├── prd/                      ← 项目偏差：PRD 历史快照仓
│   ├── sieve-prd-vX.Y.md     ← 当前活动版本（一份）
│   └── _archive/             ← 历史版本快照（永不修改）
├── design/                   ← 项目偏差：ADR 平铺，不分 adr/ 子目录
│   ├── architecture.md       ← 系统架构
│   ├── data-model.md         ← 数据模型
│   ├── ADR-INDEX.md          ← ADR 索引
│   └── ADR-NNN-*.md          ← 单个决策
├── specs/
│   ├── INDEX.md              ← SPEC 索引
│   └── SPEC-NNN-*.md         ← 单个功能技术规格
├── api/
│   └── api-reference.md      ← API 参考
├── changelog/                ← 项目偏差：CHANGELOG 在 docs/changelog/
│   └── CHANGELOG.md
├── guides/
│   ├── development.md
│   └── deployment.md
├── research/                 ← 对外部事物的调研（竞品、技术）
├── review/                   ← 对本项目产出物的评审
│   └── _archive/             ← 历史 review 归档（超过 1 个月或被取代）
└── external/                 ← 第三方参考资料
tasks/
├── PROGRESS.md               ← 单一进度真实源（任务前先看，完成后必更新）
├── roadmap.md                ← 12 周里程碑 Roadmap
├── lessons.md                ← 经验沉淀
└── _archive/                 ← 过期 todo / status 快照 / 临时报告归档
```

**`PROGRESS.md` 必含五段**：当前阶段一句话 / ✅ 已完成（按时间倒序）/ 🚧 进行中（≤3 项）/ ⏭ 下一步（按 P0/P1/P2 优先级，可勾选）/ 🚫 阻塞或等决策。任何时候打开应能在 30 秒内回答"现在做什么、做到哪、下一步是什么"。临时分析产物用 `_` 前缀（如 `_gap-*.md`），并入 PROGRESS 后立即删除。

**禁止建立的目录**：`docs/notes/`、`docs/temp/`、`docs/wip/`、`docs/misc/`。所有内容必须找到合适的归属目录。

### 1.1 项目偏差（与上游 v2.0 标准的差异，已显式记录）

| 偏差 | 标准要求 | 本仓库做法 | 理由 |
|------|---------|-----------|------|
| ADR 目录结构 | `docs/design/adr/INDEX.md` + `adr/ADR-NNN-*.md` | `docs/design/ADR-INDEX.md` + `design/ADR-NNN-*.md` 平铺 | 24 个 ADR 与 architecture / data-model 同级查找更顺手；迁移代价过高（所有交叉引用都要改） |
| PRD 历史版本 | 单一文件，版本写在文件内容里 | `docs/prd/` 下每版本一个文件，活动版本入口在 `docs/requirements/PRD-sieve.md` | 项目早期 v1.0~v2.0 演进密集，保留版本快照便于追溯 GPT-5.5 review / dogfood 改动来源；旧版本（v1.0~v1.5）归档到 `docs/prd/_archive/`，永不修改 |
| CHANGELOG 位置 | 标准未规定 | `docs/changelog/CHANGELOG.md` | 项目根保持简洁；以 docs/ 子目录形式 vs 根 CHANGELOG.md 是工程惯例选择，本仓 18+ 处文档引用使用此路径 |

如未来要消除偏差，必须在 ADR 中显式记录决策（`ADR-NNN-doc-structure-realignment.md`），并一次性迁移所有引用。

---

## 2. 命名规则

| 类型 | 模式 | 示例 |
|------|-----|------|
| ADR | `ADR-NNN-描述-用-连字符.md` | `ADR-021-tri-state-decision-and-graylist.md` |
| SPEC | `SPEC-NNN-功能名.md` | `SPEC-005-ipc-protocol.md` |
| PRD | `sieve-prd-vX.Y.md` | `sieve-prd-v2.0.md` |
| Review | `YYYY-MM-DD-来源-类型.md` | `2026-05-02-codex-spec-005-review-r5.md` |
| Research | `YYYY-MM-DD-主题.md` 或 `主题名.md` | `2026-04-28-week4-benchmark-results.md` |

ADR / SPEC 编号规则：

- 三位编号，**递增不跳号**（已废弃的也保留占位，文件标记 `Status: superseded by ADR-NNN`）
- 编号一旦发布就不改

PRD 命名说明：本仓库 PRD 文件名包含版本号（项目偏差，见 §1.1）。每次 PRD 实质性变更（≥ 0.1 minor）时，复制为新文件，旧版本归档到 `docs/prd/_archive/`，更新 `docs/requirements/PRD-sieve.md` 入口指针。

---

## 3. ADR 模板

```markdown
# ADR-NNN：{决策一句话}

> Status: Accepted | Proposed | Superseded by ADR-MMM | Deprecated
> Date: YYYY-MM-DD
> Deciders: doskey (+ 评审人列表)
> 关联 PRD：[v2.0 §X、§Y](../prd/sieve-prd-v2.0.md)
> Tags: rules, ipc, signature, build, ...

## Context

为什么要做这个决策？背景、约束、问题。

## Options Considered

### Option 1：{方案 A}
- 优点 / 缺点 / 估计成本

### Option 2：{方案 B}
### Option 3：{方案 C}

## Decision

选了 Option N。**一句话写清楚选了什么**。

## Consequences

- 正面影响
- 负面影响 / 引入的新约束
- 后续需要做的事

## References

- 相关 PRD 章节链接
- 相关 SPEC 链接
- 相关 ADR 链接（前置依赖）
```

---

## 4. SPEC 模板

```markdown
# SPEC-NNN：{模块名}

> Version: vX.Y — YYYY-MM-DD
> Status: Draft | Stable | Frozen
> Owner: doskey
> 关联 ADR：ADR-NNN, ADR-MMM
> 关联 PRD 章节：v2.0 §X.Y

## 0. 摘要 / 1. 范围与非目标 / 2. 用户路径 / 3. 状态机 /
## 4. 数据契约 / 5. 错误与降级 / 6. 性能与硬约束 /
## 7. 测试要求 / 8. 未决事项（OQ）/ 9. 变更记录
```

完整模板对照 `docs/specs/SPEC-005-ipc-protocol.md`。

---

## 5. 文档生命周期

```
       ┌─────────┐    review     ┌─────────┐    sign-off  ┌────────┐
 Draft │  写初稿  │ ───────────→  │  Review  │ ───────────→ │ Stable │
       └─────────┘   团队评审    └─────────┘  收所有意见   └────┬───┘
                                                                │
                              ┌─────────────┐    redesign       │
                              │ Superseded  │ ←─────────────────┘
                              └─────────────┘
```

- **Draft**：写作中，可随意改
- **Review**：发起评审，禁止结构性改动（只接受评审反馈式修改）
- **Stable**：通过评审，正式生效；后续修改递增 minor 版本（v1.0 → v1.1）
- **Frozen**（仅 SPEC）：发布版本对应的快照，禁止任何修改；下一版本另开文件
- **Deprecated**：不再维护，但内容保留供历史参考
- **Superseded**：被新文档取代，文件保留，开头标注 `> Superseded by ADR-MMM`

---

## 6. 上下游文档同步规则

变更触发表（与 [`CLAUDE.md`](../CLAUDE.md) 一致）：

| 场景 | 需更新的文档 | 优先级 |
|------|-------------|-------|
| 新增 / 删除检测项（OUT-* / IN-CR-* / IN-GEN-*） | user-stories + architecture + api-reference + CHANGELOG | P0 |
| 检测项 FP 上限调整 / 处置等级变化 | architecture + ADR + CHANGELOG | P0 |
| 修改 IPC / 架构变更 | 对应 SPEC + ADR + api-reference + CHANGELOG | P0 |
| Pipeline 节点增删 / crate 边界变化 | architecture + .cursorrules §3.3 + CHANGELOG | P0 |
| 工程硬约束变化（PRD §9） | PRD-sieve 版本演进表 + ADR + .cursorrules + CHANGELOG `[BREAKING]` | P0 |
| Bug 修复（涉及逻辑） / 配置变更 | CHANGELOG + 相关文档 | P1 |
| 依赖升级 | CHANGELOG | P2 |

无需文档化的变更：纯格式化、注释优化（不涉及逻辑）、测试补充（无功能变更）。

**与下游 sieve-gui-macos 仓库的同步**：任何 IPC 字段或行为变更必须**两个仓库同时改 SPEC + 协议版本号**，由提交者手动协调（详见 [`SPEC-005-ipc-protocol.md`](specs/SPEC-005-ipc-protocol.md)）。

---

## 7. 链接规范

- 仓库内链接用相对路径：`[architecture](../design/architecture.md)`
- 跨章节锚点用 GitHub 风格 slug：`[§5.2](#52-状态机)`
- PRD 历史版本引用必须走 `_archive/` 路径：`[v1.5 §X](../prd/_archive/sieve-prd-v1.5.md)`

---

## 8. 写作风格

- **结论先行**：每个章节第一段说"是什么 / 为什么"，不要慢热
- **最少充分**：写够支撑决策的最少信息；冗余是负担
- **图优先**：能画图就别只写文字（ASCII art / Mermaid / Markdown 表格都行）
- **避免**：「我们」「让我们」「显而易见」「众所周知」「请注意」
- **保留专有名词英文**：rule_id、Permit2、EIP-712、SSE、daemon、IPC、preset 等

---

## 9. 审核检查表（PR Reviewer 用）

- [ ] 命名符合 §2
- [ ] 版本化文档带 `> Version:` 标注
- [ ] ADR / SPEC 用了对应模板（§3 / §4）
- [ ] PRD 历史版本引用走 `_archive/` 路径
- [ ] 修改了 IPC 相关文档时，SPEC-005 + sieve-gui-macos 仓库 SPEC-008 同步更新
- [ ] 链接路径都是相对路径，没有 hardcode GitHub URL
- [ ] CHANGELOG 有同步条目（`docs/changelog/CHANGELOG.md`）
