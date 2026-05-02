# PRD: Sieve（活动版本入口）

## 基本信息

- **版本**：v2.0
- **创建日期**：2026-05-01（v2.0 锁定执行）
- **最早起源**：v1.0（详见 [版本演进表](#版本演进表)）
- **负责人**：doskey
- **状态**：✅ **已确认锁定执行**（HIPS 改造版：用户规则系统 + 三态决策 + 行为序列 beta + 规则引擎抽象 + 进程上下文记录）

---

## 📌 当前活动版本指针

> **当前活动 PRD 全文 → [../prd/sieve-prd-v2.0.md](../prd/sieve-prd-v2.0.md)**

v2.0 在 v1.5 基础上启动**完整 HIPS 改造**（依据 [HIPS Readiness Assessment](../review/2026-05-01-hips-readiness-assessment.md) 评估"Sieve 当前 HIPS 70%"），目标 GA 时达到 HIPS 90%。Phase A（Week 5-8）+ Phase B（Week 9-12）拆分，**GA Week 12 时间表不变**。

5 项 HIPS 能力改造（codex review 后瘦身 + 加固）：

1. **用户规则系统**（§5.5）：单文件 `~/.sieve/rules/user.toml` + 11 类安全约束 + 4 个核心子命令；用户规则只能 High Ask/Warn/Mark，不能 override 系统 Critical
2. **三态决策 + 灰名单**（§5.4）：Allow / Deny / Ask 三态 + 灰名单 schema；**Critical 锁三道防线**（IPC daemon 计算 / 接收时二次校验 / GUI Remember checkbox 灰显）守住 PRD §9 #3 fail-closed 不被绕过
3. **行为序列联动**（§5.7）：滑动窗口 N=10 / TTL=5min + 结构化安全特征（非 hash）+ 3 条 IN-SEQ-* 启发式；**GA 默认关闭**（beta opt-in），仅 StatusBar 通知
4. **规则引擎抽象**（§6.3）：`MatchEngine::scan(ScanRequest) -> ScanReport` + LayeredEngine 合并顺序强约束（系统规则先行，用户规则不能 suppress）
5. **进程上下文记录**（§5.6）：audit schema 加 caller_pid + caller_exe（cwd/ppid 推 v2.1）

**v2.0 新增 1 个 crate**：`sieve-policy`（用户规则加载 + lint + 灰名单 + 编辑器）。**拦截引擎抽象（trait + 多平台）推 v2.1**（review 后撤回）。

**v2.0 新增 PRD §9 三条硬约束**：
- #14 用户规则 fail-safe（用户规则错误绝不影响系统规则功能）
- #15 行为序列保守起步 + GA 默认关闭
- #16 所有入站能力必须经过 content-type 路由矩阵测试（v1.5.4 P0 教训永久化）

**v2.0 6 个新 ADR**：ADR-020（用户规则）/ ADR-021（三态决策 + Critical 锁）/ ADR-022（行为序列窗口）/ ADR-023（进程上下文）/ ADR-024（规则引擎抽象）/ ADR-025（content-type 路由矩阵）

详细修订历史 + codex review 反馈见 [v2.0 PRD §15 changelog](../prd/sieve-prd-v2.0.md#v15--v20-changelog)。

---

## 一句话介绍

> Sieve 是一个完全本地运行的 LLM 流量代理，在 AI 编码 agent 和上游模型之间做双向安全检测，服务于 crypto 开发者和 DeFi 重度用户，在不可逆动作（签名/转账/部署）前强制插入认知摩擦，防止私钥泄漏、地址替换、危险工具调用导致的资产损失。
>
> — 引自 [PRD v2.0 §1.1](../prd/sieve-prd-v2.0.md#11-一句话)

---

## 核心叙事（四句话）

1. **上游不可信**：你用的中转站可能在改你的 tool_call，官方 API 出问题不会赔你私钥被盗的钱
2. **没人能替你兜底**：钱包安全产品看不见你的 prompt，LLM 安全产品不懂 crypto，DLP 不在你工作流里
3. **Sieve 在客户端最后一道闸**：完全本地运行，字节流双向扫描，从不上传你的数据
4. **你不只是相信我们，你能验证我们**：开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视，绝不成为新的供应链风险

> — 引自 [PRD v2.0 §1.2](../prd/sieve-prd-v2.0.md#12-四句话核心叙事v13-加第-4-句)

---

## 版本演进表


| 版本       | 日期                       | 主要变化                                                                                  | 文件                                                              |
| -------- | ------------------------ | ------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| v1.0     | 初版                       | 第一性原理推导 + 单 agent 架构 + 12 周里程碑骨架                                                      | [../prd/sieve-prd-v1.0.md](../prd/sieve-prd-v1.0.md)            |
| v1.1     | 中间版                      | 检测项 ID 化（OUT-01~12 / IN-CR-* / IN-GEN-*）+ 处置矩阵 + 误报率预算                                | [../prd/sieve-prd-v1.1.md](../prd/sieve-prd-v1.1.md)            |
| v1.2     | 第一性原理 + 性能预算定稿版          | 性能预算具体化（P99 < 20ms）+ Rust 技术栈定稿 + 数据飞轮简化版 + 公理 12（FP < 0.5%）                          | [../prd/sieve-prd-v1.2.md](../prd/sieve-prd-v1.2.md)            |
| v1.3     | 2026-04-26                 | 8 条 GPT-5.5 review 改动：合规边界 + 自证清白叙事 + MCP Phase 2 + 数据合作清单 + benchmark 具体化 + 闭测画像 | [../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md) |
| v1.4 | 2026-04-27 | HIPS 弹窗架构 + Native GUI App + setup 自动配置 + Claude Code hooks 双层防御：14 条改动，§9 新增第 11-13 条硬约束 | [../prd/sieve-prd-v1.4.md](../prd/sieve-prd-v1.4.md) |
| v1.5 | 2026-04-28 | Multi-Agent 扩展（Claude Code + OpenClaw + Hermes 三家适配）：15 条改动，§9 第 9 条重写，新增 IN-GEN-06 / IN-CR-06 检测项 | [../prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md) |
| **v2.0** | **2026-05-01**（**当前活动**） | **HIPS 改造**：5 项能力（用户规则 / 三态决策 + Critical 锁 / 行为序列 beta / 规则引擎抽象 / 进程上下文）+ Phase A/B 拆分 + GA Week 12 不变；新增 sieve-policy crate；新增 §9 #14/#15/#16 三条硬约束；新增 6 ADR-020~025；新增 IN-SEQ-01/02/03 序列检测；4 轮 review feedback（codex 9 Must + 3 Should 全部落地） | [../prd/sieve-prd-v2.0.md](../prd/sieve-prd-v2.0.md) ← **当前活动** |


> **历史版本归档原则**：`docs/prd/` 下文件**不修改**，只新增。所有讨论 / 引用一律以 v2.0 为准，旧版本仅供追溯演进逻辑。

---

## 为什么不直接用 PRD v1.3 这个文件？

三个原因：

1. **目录约束**：项目级 `.cursorrules` 与用户规则要求 `docs/requirements/` 是需求文档的标准位置，而 `docs/prd/` 是 PRD 历史归档目录。`PRD-sieve.md` 作为入口能让所有跨文档引用稳定指向 `docs/requirements/PRD-sieve.md`，而不必每次升级版本就改一遍引用路径。
2. **入口与归档分离**：`docs/prd/sieve-prd-v1.3.md` 是定稿后的快照（不可改），本文件是入口指针（可改），便于版本演进。
3. **跨文档引用便利**：`docs/design/`、`docs/api/`、`docs/changelog/` 全部引用本文件作为 PRD 入口，本文件再代理到具体版本，单点切换、零迁移成本。

---

## 相关下游文档


| 文档     | 关系                                            | 链接                                                         |
| ------ | --------------------------------------------- | ---------------------------------------------------------- |
| 用户故事   | 本 PRD 的用户角色 + 场景的故事化展开                        | [./user-stories.md](./user-stories.md)                     |
| 架构设计   | 本 PRD §6 技术架构的实现细化                            | [../design/architecture.md](../design/architecture.md)     |
| 数据模型   | fingerprint / SQLite schema / license 数据结构   | [../design/data-model.md](../design/data-model.md)         |
| ADR 索引 | 7 个已接受 ADR + 候选 ADR 列表                        | [../design/ADR-INDEX.md](../design/ADR-INDEX.md)           |
| API 参考 | 本 PRD §6.1 协议层（Anthropic Messages + SSE）的接口规范 | [../api/api-reference.md](../api/api-reference.md)         |
| 开发指南   | 构建 / 测试 / SSE fuzz / benchmark / 规则编写         | [../guides/development.md](../guides/development.md)       |
| 部署指南   | 安装 / 签名验证 / 服务运行 / 升级回滚 / FAQ                  | [../guides/deployment.md](../guides/deployment.md)         |
| 变更日志   | 本 PRD 各次版本变更与下游代码 / 配置变更的对应记录                 | [../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)     |
| 术语表   | 项目专业术语统一定义                                    | [../glossary.md](../glossary.md)                           |
| Roadmap | 12 周里程碑可勾选执行清单                               | [../../tasks/roadmap.md](../../tasks/roadmap.md)           |
| 项目入口   | 项目级总览                                         | [../../README.md](../../README.md)                         |


---

## 上游变更触发的下游更新清单

> 当 PRD（本文件或 `docs/prd/sieve-prd-vX.X.md`）发生变更时，必须按下表检查并更新下游文档。

### A. 检测项变更（PRD §5）


| 变化类型                                   | 必须更新的下游                                                                                                           |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 新增 / 删除 OUT-* / IN-CR-* / IN-GEN-* 检测项 | `user-stories.md`（关联 US 编号）+ `design/architecture.md`（pipeline 节点）+ `api/api-reference.md`（如暴露配置）+ `CHANGELOG.md` |
| FP 上限调整                                | `design/architecture.md`（误报率预算章）+ `CHANGELOG.md`                                                                  |
| 处置等级变化（如 Medium → Critical）            | `user-stories.md`（验收标准）+ `CHANGELOG.md`（标注**行为变更**）                                                               |


### B. 协议 / 架构变更（PRD §6）


| 变化类型                                                   | 必须更新的下游                                                                                           |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| 新增上游协议（如 Phase 2 接入 OpenAI）                            | `design/architecture.md` + `design/ADR-XXX.md` + `api/api-reference.md` + `guides/development.md` |
| Pipeline 节点增删 / 顺序调整                                   | `design/architecture.md`（架构图）+ `CHANGELOG.md`                                                     |
| 性能预算调整（P99 / 内存 / 二进制大小）                               | `design/architecture.md` §性能预算 + `guides/development.md`（benchmark 命令）                            |
| crate 边界（`sieve-core` / `sieve-rules` / `sieve-cli`）变化 | `.cursorrules` §3.3 + `design/architecture.md`                                                    |


### C. 商业 / 合规变更（PRD §7、§11）


| 变化类型                 | 必须更新的下游                                                         |
| -------------------- | --------------------------------------------------------------- |
| 定价 / 试用周期 / 降级模式调整   | `README.md`（定价段）+ `user-stories.md`（US-12 降级模式）+ `CHANGELOG.md` |
| 法律实体 / 渠道策略变化        | `README.md`（合规提示框）+ `CHANGELOG.md`                              |
| 开源策略变化（如延迟开源 / 范围调整） | `README.md`（自证清白段）+ `CHANGELOG.md`                              |


### D. 里程碑变更（PRD §10）


| 变化类型                     | 必须更新的下游                                        |
| ------------------------ | ---------------------------------------------- |
| Week 编号 / 完成定义调整         | `README.md`（12 周里程碑摘要表）+ `CHANGELOG.md`        |
| 闭测画像 / benchmark 数据集大小变化 | `user-stories.md`（验收标准的 FP 阈值）+ `CHANGELOG.md` |


### E. 工程硬约束变更（PRD §9 十六条）

> ⚠️ §9 十六条硬约束**默认不允许放宽**，任何修改必须：
>
> 1. 在本文件版本演进表新增一行
> 2. 在 `docs/design/` 下写一份 ADR
> 3. 在 `.cursorrules` §二同步更新
> 4. 在 `CHANGELOG.md` 标注**[BREAKING]** 前缀

---

## 备注

- 本文件随 PRD 主版本升级（如 v1.5 → v1.6）必须更新"📌 当前活动版本指针"段落与"版本演进表"
- 历史版本（v1.0~v1.4）的 `docs/prd/sieve-prd-vX.X.md` **不修改**，仅供追溯
- 临时调研 / 草稿放 `docs/research/` 或 `docs/_temp/`，不要污染本文件