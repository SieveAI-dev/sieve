# PRD: Sieve（活动版本入口）

## 基本信息

- **版本**：v1.5
- **创建日期**：2026-04-28（v1.5 锁定执行）
- **最早起源**：v1.0（详见 [版本演进表](#版本演进表)）
- **负责人**：doskey
- **状态**：✅ **已确认锁定执行**（Multi-Agent 扩展版（Claude Code + OpenClaw + Hermes 三家适配），锁定执行）

---

## 📌 当前活动版本指针

> **当前活动 PRD 全文 → [../prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md)**

v1.5 在 v1.4 基础上做了 15 条改动，核心主题为 Multi-Agent 扩展（Claude Code + OpenClaw + Hermes），引擎 100% 复用，差异仅在协议适配 + 配置注入 + 2 条新检测项：

1. **§0 修订说明**：触发原因 + 改动汇总 + 没改的明确说明
2. **§1.1 一句话改写**为"三家 agent + 三类用户"
3. **§1.3 不是什么**：加"不是 multi-agent gateway / 不是 OpenClaw 审计公司"
4. **§3.1 / §3.2 用户画像**新增子细分（OpenClaw 信使型 + Hermes multi-LLM）
5. **§4.5 场景 E**（OpenClaw 跨通道 injection）
6. **§4.6 场景 F**（Hermes sub-agent 嵌套决策传递）
7. **§5.2 入站检测**加 IN-GEN-06 + IN-CR-06
8. **§5.3 处置矩阵**新规则映射
9. **§6.1 整体架构图**重画（三入口 + 嵌套箭头）
10. **§6.5 IPC schema** 加 source_agent / origin_chain / source_channel + X-Sieve-Origin HTTP header 协议
11. **§6.6 部署形态**新增 `sieve setup --agent` 多 agent 参数 + 三家配置注入路径表
12. **§6.7 双层防御**关键澄清：OpenClaw / Hermes 上 Hook 类降级为 GUI hold
13. **§9 第 9 条重写**：从"仅 Claude Code"扩到"三家 + UnifiedMessage 双协议"
14. **§10 Week 6-7 重写**为 OpenAI 协议适配 + multi-agent 集成测试
15. **§12 风险登记新增 5 条** multi-agent 风险；§13 数据合作新增 OpenClaw / Nous Research；§14 Open Questions 第 9-13 条；§15.5 Multi-Agent 扩展参考

---

## 一句话介绍

> Sieve 是一个完全本地运行的 LLM 流量代理，在 AI 编码 agent 和上游模型之间做双向安全检测，服务于 crypto 开发者和 DeFi 重度用户，在不可逆动作（签名/转账/部署）前强制插入认知摩擦，防止私钥泄漏、地址替换、危险工具调用导致的资产损失。
>
> — 引自 [PRD v1.5 §1.1](../prd/sieve-prd-v1.5.md#11-一句话)

---

## 核心叙事（四句话）

1. **上游不可信**：你用的中转站可能在改你的 tool_call，官方 API 出问题不会赔你私钥被盗的钱
2. **没人能替你兜底**：钱包安全产品看不见你的 prompt，LLM 安全产品不懂 crypto，DLP 不在你工作流里
3. **Sieve 在客户端最后一道闸**：完全本地运行，字节流双向扫描，从不上传你的数据
4. **你不只是相信我们，你能验证我们**：开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视，绝不成为新的供应链风险

> — 引自 [PRD v1.5 §1.2](../prd/sieve-prd-v1.5.md#12-四句话核心叙事v13-加第-4-句)

---

## 版本演进表


| 版本       | 日期                       | 主要变化                                                                                  | 文件                                                              |
| -------- | ------------------------ | ------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| v1.0     | 初版                       | 第一性原理推导 + 单 agent 架构 + 12 周里程碑骨架                                                      | [../prd/sieve-prd-v1.0.md](../prd/sieve-prd-v1.0.md)            |
| v1.1     | 中间版                      | 检测项 ID 化（OUT-01~12 / IN-CR-* / IN-GEN-*）+ 处置矩阵 + 误报率预算                                | [../prd/sieve-prd-v1.1.md](../prd/sieve-prd-v1.1.md)            |
| v1.2     | 第一性原理 + 性能预算定稿版          | 性能预算具体化（P99 < 20ms）+ Rust 技术栈定稿 + 数据飞轮简化版 + 公理 12（FP < 0.5%）                          | [../prd/sieve-prd-v1.2.md](../prd/sieve-prd-v1.2.md)            |
| v1.3     | 2026-04-26                 | 8 条 GPT-5.5 review 改动：合规边界 + 自证清白叙事 + MCP Phase 2 + 数据合作清单 + benchmark 具体化 + 闭测画像 | [../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md) |
| v1.4 | 2026-04-27 | HIPS 弹窗架构 + Native GUI App + setup 自动配置 + Claude Code hooks 双层防御：14 条改动，§9 新增第 11-13 条硬约束 | [../prd/sieve-prd-v1.4.md](../prd/sieve-prd-v1.4.md) |
| **v1.5** | **2026-04-28**（**当前活动**） | **Multi-Agent 扩展（Claude Code + OpenClaw + Hermes 三家适配）**：15 条改动，§9 第 9 条重写，新增 IN-GEN-06 / IN-CR-06 检测项 | [../prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md) ← **当前活动** |


> **历史版本归档原则**：`docs/prd/` 下文件**不修改**，只新增。所有讨论 / 引用一律以 v1.5 为准，旧版本仅供追溯演进逻辑。

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


### E. 工程硬约束变更（PRD §9 十三条）

> ⚠️ §9 十三条硬约束**默认不允许放宽**，任何修改必须：
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