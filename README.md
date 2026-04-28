# Sieve

> **本地 LLM 流量代理 · Crypto-native 开发者的最后一道闸**

Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），夹在 AI 编码 agent（首期只支持 Claude Code）和上游模型之间，做双向安全检测，专门服务 crypto-native 开发者。在不可逆动作（签名 / 转账 / 部署）前强制插入认知摩擦，防止私钥泄漏、地址替换、危险工具调用导致的资产损失。

---

## 项目状态

**项目状态**：Week 2 出站规则引擎完成 (2026-04-27) — OUT-01~12 P0 规则上线，BIP39 SHA-256 校验，426 出站拦截，86 测试全过，e2e smoke 26/26。release 二进制 9.0 MB。repo 保持 private 至 Week 12 GA，见 [ADR-011](docs/design/ADR-011-private-until-ga.md)。

- 当前最新 PRD：[sieve-prd-v1.5.md](./docs/prd/sieve-prd-v1.5.md)（已锁定执行）
- 12 周里程碑：8 周 dogfood + 4 周闭测 → GA 开源

---

## 核心叙事（四句话）

1. **上游不可信**：你用的中转站可能在改你的 tool_call，官方 API 出问题不会赔你私钥被盗的钱
2. **没人能替你兜底**：钱包安全产品看不见你的 prompt，LLM 安全产品不懂 crypto，DLP 不在你工作流里
3. **Sieve 在客户端最后一道闸**：完全本地运行，字节流双向扫描，从不上传你的数据
4. **你不只是相信我们，你能验证我们**：开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视，绝不成为新的供应链风险

> 详见 PRD §1.2

---

## 关键差异化（四点护城河）

1. **LLM 流量层位置**——独占
2. **完全本地零云依赖**——LLM Guard 之外只有我
3. **Crypto 专项检测**——19 家 LLM/DLP 全无，9 家 AI Agent 安全工具全无
4. **双向检测 + fail-closed**——钱包安全产品看不到 prompt，Sieve 看得到

> 详见 PRD §2.3

---

## 12 周里程碑摘要


| 阶段                    | 时间窗       | 一句话                                                            |
| --------------------- | --------- | -------------------------------------------------------------- |
| **Phase A · dogfood** | Week 1-8  | 基础设施 + 出入站规则 + benchmark 数据集 + doskey 自用 8 周打磨                 |
| **Phase B · 闭测**      | Week 9-12 | 5+5 名海外 hackathon builder / 审计研究员闭测，GA 同步发"中转站揭黑" + "自证清白"两篇文章 |
| **Phase C · 维护**      | Week 13+  | 每周 5-10 小时慢节奏：规则库每周更新、季度大版本、按真实需求推进 Phase 2                    |


> 详见 PRD §10

---

## 定价


| 阶段         | 价格                        | 内容                              |
| ---------- | ------------------------- | ------------------------------- |
| **14 天试用** | $0                        | 全功能                             |
| **正式版**    | **$49/月**（年付 $490，省 2 个月） | 全功能                             |
| **降级模式**   | $0                        | 试用结束未付费：**只读警告**，不再 Critical 拦截 |


> 详见 PRD §7

---

## 合规提示

> ⚠️ **海外公司主体 + 中国大陆境内不做 to-C 公开商业化**
>
> - 公司必须海外注册（**首选香港有限公司或新加坡 Pte Ltd**），不接受大陆个人/个体户作为 Stripe 收款主体
> - **境内渠道发研究内容，境外渠道发产品营销**——Twitter / Hacker News / Mirror 是主战场，微信公众号 / 小红书 / 知乎 / B 站不规划
> - Sieve 完全本地运行 + 不上传 prompt → 不触发数据出境合规
> - 详见 PRD §11.5（中国大陆合规边界）

---

## 文档导航


| 入口                                                                       | 用途                                                  |
| ------------------------------------------------------------------------ | --------------------------------------------------- |
| [docs/requirements/PRD-sieve.md](./docs/requirements/PRD-sieve.md)       | 需求文档活动版本入口（指向 PRD v1.5）                             |
| [docs/requirements/user-stories.md](./docs/requirements/user-stories.md) | 用户故事（13 条 P0/P1）                                    |
| [docs/glossary.md](./docs/glossary.md)                                   | **术语表**（54 条专业术语统一定义）                              |
| [docs/design/ADR-INDEX.md](./docs/design/ADR-INDEX.md)                   | **ADR 索引**（7 个已接受 + 候选 ADR）                        |
| [docs/design/architecture.md](./docs/design/architecture.md)             | 架构设计                                                |
| [docs/design/data-model.md](./docs/design/data-model.md)                 | 数据模型（fingerprint / SQLite schema / license）         |
| [docs/api/api-reference.md](./docs/api/api-reference.md)                 | API 参考（Anthropic Messages API + SSE + 本地管理 API）    |
| [docs/guides/development.md](./docs/guides/development.md)               | 开发指南                                                |
| [docs/guides/deployment.md](./docs/guides/deployment.md)                 | 部署与运维指南                                             |
| [docs/changelog/CHANGELOG.md](./docs/changelog/CHANGELOG.md)             | 变更日志                                                |
| [docs/prd/](./docs/prd/)                                                 | PRD 历史归档（v1.0 → v1.5）                               |
| [docs/research/](./docs/research/)                                       | 调研材料（deep-research-report）                          |
| [tasks/roadmap.md](./tasks/roadmap.md)                                   | **12 周里程碑可勾选执行清单**                                  |
| [tasks/lessons.md](./tasks/lessons.md)                                   | 经验教训记录                                              |
| [SECURITY.md](./SECURITY.md)                                             | **安全策略 + 漏洞报告流程**                                  |
| [LICENSE](./LICENSE)                                                     | 许可说明（文档 CC BY-NC-SA 4.0 / 代码 MIT 待 GA）              |
| [CLAUDE.md](./CLAUDE.md)                                                 | Claude Code 项目指引                                    |
| [.github/](./.github/)                                                   | Issue / PR 模板 + Dependabot 配置                       |


---

## 技术栈

**Rust** + **hyper** (HTTP/反代) + **tokio** (async) + **rustls** (TLS) + **vectorscan-rs** (SIMD 多模式正则) + **sonic-rs** (SIMD JSON 流式解析)

> 详见 PRD §6.3

---

## 自证清白（Self-Custody Trust）

Sieve 自己被同一套标准审视：

- **sigstore 签名** + **reproducible build**：每个 release 都可独立复现验证
- **pinned dependencies**：避免 LiteLLM 类供应链事件
- **核心引擎 GA 后开源（MIT）**：Phase 2 高级规则集闭源，但拦截逻辑全部可审
- **透明规则更新日志**：每次规则更新发布 changelog + 哈希，用户可独立验证

> 详见 PRD §1.2 第 4 句、§9 第 6 条、§11.3

---

## 反馈渠道

- **GitHub Issues**：本仓库 issue 列表（公开样本提交也走这里）
- **Twitter**：[@doskey](https://twitter.com/doskey)

---

## 文档关联图

```mermaid
graph TD
    README[README.md<br/>项目入口]
    REQ[docs/requirements/PRD-sieve.md<br/>需求活动版本指针]
    PRDv14[docs/prd/sieve-prd-v1.5.md<br/>当前活动 PRD]
    US[docs/requirements/user-stories.md<br/>用户故事]
    DESIGN[docs/design/architecture.md<br/>架构设计]
    API[docs/api/api-reference.md<br/>API 参考]
    CL[docs/changelog/CHANGELOG.md<br/>变更日志]

    README --> REQ
    README --> DESIGN
    README --> API
    README --> CL
    REQ --> PRDv14
    REQ --> US
    REQ --> DESIGN
    DESIGN --> API
    API --> CL
    DESIGN --> CL
    US --> PRDv14
```



> 派生关系：上游文档变更时，必须检查并更新所有下游文档（详见 [.cursorrules](./.cursorrules) 文档规则段落）。

