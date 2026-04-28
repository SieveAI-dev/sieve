# ADR-002: Phase 1 纯规则引擎，不引入本地 ML 模型

## 状态

**已接受**（v1.4 锁定执行）

> 决策日期：2026-04-26
> 范围：Phase 1 全部检测（OUT-01~~12 + IN-CR-01~~05 + IN-GEN-01~05）
> 关联 PRD：[v1.4 §6.2](../prd/sieve-prd-v1.5.md)

---

## 背景

Sieve 是 LLM 流量层的安全代理。当前业界主流的"AI 安全检测"产品（Lakera Guard、Nightfall、LLM Guard、AWS Bedrock Guardrails 等）几乎都把"分类器"作为核心：把 prompt 喂给一个本地或云端 LLM/小模型，让它判断"这是不是攻击 / 这是不是 PII / 这是不是危险代码"。

如果按这个思路做，Sieve 的 Phase 1 设计会自然演变成：

- 训练 / 微调一个本地小模型（DistilBERT / Phi 量化版）
- 部署 ONNX Runtime / llama.cpp 跑推理
- 规则引擎只做粗筛，模型做精判

但 Sieve 是**[redacted]**，doskey 一个人 + Claude Code 完成 12 周冲刺，决策面临两个第一性原理硬约束：

1. **个人开发者对误报极其敏感**——任何一条 Critical FP 触发，用户就会卸载产品（[公理 12](../prd/sieve-prd-v1.5.md#65-误报率预算)，Critical FP > 0.5% → 用户禁用）；
2. **单人团队最稀缺的资源不是算力，是数据标注能力**——本地小模型的精度天花板取决于训练数据质量，doskey 一个人扛不动这个标注流程。

需要在 Phase 1 决定：**走规则引擎，还是走规则 + ML 双层。**

## 决策

**Phase 1 完全用规则引擎，不引入任何本地 ML 模型 / ONNX / 分类器。**

Phase 2 视用户实际误报率决定是否引入二阶段轻量模型作为补充层。

### Week 1 前置依赖

本决策的可执行性依赖 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 的 sigstore + reproducible build CI pipeline 在 Week 1 跑通——纯规则引擎方案不依赖外部 ML 服务，但每周规则更新（[PRD §8.3](../prd/sieve-prd-v1.5.md#83-规则更新)）必须依赖 Ed25519 签名 + Rekor 透明日志才不沦为投毒载体。两者是同一信任链的两个组件，缺一不可。

### 三个独立论证（直接对齐 PRD §6.2）

#### 论证 1：结构化优先

编码代理中的高风险事件**绝大多数是结构化的**：

- 私钥形态（前缀 + entropy + 长度 + 校验位）
- BIP39 助记词（词表 2048 词 + SHA-256 校验位）
- 加密地址（`0x[a-fA-F0-9]{40}` + EIP-55 大小写校验）
- EIP-712 typed data 结构
- 函数 selector（4byte）
- 危险 shell 模式（`rm -rf /` / `curl X | sh` / `eval(base64...)`）
- 危险 tool_use 调用（`signTypedData` / `eth_sendTransaction`）

这些场景**比泛文本更适合可解释规则**。规则引擎的精度 + 可解释性，模型给不了：

- 用户问"为什么 Sieve 拦了我这条"，规则引擎能给出确切的 rule_id + 命中片段；模型给出的"概率 0.87"无法消除用户的不安；
- 用户加 `.sieveignore` 也只在规则引擎语义下成立——模型的"再训练 / 微调"对单用户无意义。

#### 论证 2：误报敏感（GitHub secret scanning 演进史佐证）

GitHub secret scanning（2019 年起）走过的演进路径是公开的：

- **早期版本**：高 recall + 高 FP，被开发者吐槽到不可用；
- **2020-2022**：引入 partner secret 规则（每家 SaaS 自己提供前缀 + validity check API），FP 大幅下降；
- **2023+**：增加自定义模式 + push protection；模型只在"不确定的边界 case"用做轻度筛选。

**核心规律**：生产可用的 secret 检测**通常依赖模式 + validity checks + 定制规则**，逐步扩展，而不是全部押在分类器上。

这条规律对个人开发者群体**放大 10 倍**：

- 企业用户对 FP 容忍度高（有审批流、可压队列）；个人开发者一次 FP 就关产品；
- 个人开发者会反复触发同一类 FP（比如调试时 paste 测试 .env），ML 模型每次都要重新判断，规则引擎能直接通过 `.sieveignore` 学习。

#### 论证 3：单人团队最稀缺的资源是数据标注

这是 v1.3 从 GPT-5.5 review 加的核心论证：

- 本地小模型的精度天花板取决于训练数据质量；
- 一个能跑的"判断 prompt 是否包含敏感信息"的二分类器，**至少需要 5,000–10,000 条人工标注样本**（参考 Lakera Guard 公开数据）；
- 持续维护需要每月新增 200–500 条标注（业务漂移）；
- doskey 一个人既要写代码、做 dogfood、写营销文章、跑闭测、做客服，**没有能力开第三条战线做数据标注流程**。

**规则引擎绕开了这个瓶颈**：规则可以靠 doskey 一个人 + Claude Code 持续维护，每条规则的"调优成本"是写一个测试用例 + 跑 benchmark，**数量级是分钟，不是月**。

### Phase 2 引入 ML 的触发条件

**只有同时满足以下两个条件**，才在 Phase 2 启动 ML 子项目：

1. **High FP 持续 4 周 > 5%**（[architecture.md §5 误报率预算](./architecture.md)的 High 上限）；**且**
2. **至少 10 例**真实付费用户主动反馈"误报太多 / 需要更智能的判断"。

仅满足其一不启动。这是为了避免：

- doskey 自己 dogfood 时的过度乐观（"我觉得应该加 ML"）→ 没有真实用户驱动；
- 单条规则 FP 暴涨 → 应该改规则，不应该上模型。

ML 子项目即使启动，也定位为**第二层补充**——规则引擎仍然是第一层，ML 只对规则引擎"高置信但有疑虑"的边界 case 做二次判断。永远不会出现"模型判断为安全，规则判断为危险，跳过规则"的逻辑。

---

## 影响

### 正面影响

1. **12 周内能 GA**：跳过 ML 训练流程节省 4–6 周；
2. **可解释性 100%**：每个 Detection 都有明确 rule_id 和命中片段，用户可加 `.sieveignore`；
3. **编译期可枚举**：所有规则在 Rust 代码 / 静态资源中，便于审计、便于复现构建（[ADR-006](./ADR-006-sigstore-reproducible-build.md)）；
4. **没有"模型权重"分发问题**：规则文件是签名 tar.zst（详见 [data-model.md §7](./data-model.md)），不存在模型权重的下发协议设计；
5. **二进制小**：< 20 MB 目标可达；如果带 ONNX runtime + 模型权重，至少 80 MB+；
6. **启动快**：< 500 ms 目标可达；ML 模型加载至少 1–3 秒。

### 负面影响

1. **覆盖盲区**：纯文本意图识别（"这段 prompt 像不像在套取我的 API key"）规则覆盖弱——但这类场景**不在 Phase 1 P0 范围**（P0 都是结构化 secret 与工具调用）；
2. **规则集维护负担**：每周更新规则库（[PRD §8.3](../prd/sieve-prd-v1.5.md)）需要 doskey 持续投入 5–10 小时；用 Claude Code 辅助 + GitHub issue 公开样本流程缓解；
3. **被竞品 PR 攻击**：Lakera 等会公开宣传"我们用 LLM 判断，更智能"——反击 talking point 是"个人开发者付钱不是为了智能，是为了不烦人 + 不出事"，对应 PRD §1.2 第 2 句"没人能替你兜底"；
4. **未来切换成本**：如果 Phase 2 真的引入 ML，需要重新设计规则与模型的协作逻辑——但二阶段补充层模式（规则在前 / 模型在后）已经在论证里规避了"重写"。

### 需要更新的文档

- [architecture.md](./architecture.md) §2 模块矩阵已不含 ML 模块；§7 Phase 2 演进路径已列出 ML 触发条件
- [data-model.md](./data-model.md) §7 规则签名文件格式已对齐"规则即数据"的设计
- `docs/guides/development.md`（待编写）—— 写明"如何加一条规则"流程
- `docs/changelog/CHANGELOG.md`（待编写）—— v0.1.0 启动时记录"纯规则引擎"决策

---

## 相关文档

- [PRD-sieve v1.4 §6.2](../prd/sieve-prd-v1.5.md) —— 关键技术决策（v1.3 强化论证）
- [PRD-sieve v1.4 §6.5](../prd/sieve-prd-v1.5.md) —— 误报率预算（公理 12）
- [PRD-sieve v1.4 §9.1](../prd/sieve-prd-v1.5.md) —— 工程硬约束
- [architecture.md](./architecture.md) —— 整体架构、Phase 2 演进路径
- [ADR-001](./ADR-001-rust-tech-stack.md) —— Rust 技术栈（vectorscan-rs 是规则引擎的物质基础）
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical 永不可关（规则引擎确保可解释 + 可审计）