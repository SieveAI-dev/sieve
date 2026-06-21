# ADR-002: Phase 1 纯规则引擎，不引入本地 ML 模型

## 状态

**已接受**（v1.4 锁定执行）

> 决策日期：2026-04-26
> 范围：Phase 1 全部检测（OUT-01~~12 + IN-CR-01~~05 + IN-GEN-01~05）

---

## 背景

Sieve 是 LLM 流量层的安全代理。业界主流的"AI 安全检测"产品（云端分类器、本地小模型 ONNX 方案等）几乎都把"分类器"作为核心，但这条路线在 Sieve 的设计约束下存在明显冲突：

1. **个人开发者对误报极其敏感**——任何一条 Critical FP 触发，用户就会卸载产品；
2. **本地小模型的精度天花板取决于训练数据质量**——维持可用的数据标注流程需要持续投入，而这不是 Phase 1 的关键路径。

需要在 Phase 1 决定：**走规则引擎，还是走规则 + ML 双层。**

---

## 决策

**Phase 1 完全用规则引擎，不引入任何本地 ML 模型 / ONNX / 分类器。**

Phase 2 视用户实际误报率决定是否引入二阶段轻量模型作为补充层。

### Week 1 前置依赖

本决策的可执行性依赖 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 的 sigstore + reproducible build CI pipeline 在 Week 1 跑通——纯规则引擎方案不依赖外部 ML 服务，但规则更新必须依赖 Ed25519 签名 + Rekor 透明日志才不沦为投毒载体。两者是同一信任链的两个组件，缺一不可。

### 三个独立论证

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

- 用户问"为什么 Sieve 拦了我这条"，规则引擎能给出确切的 rule_id + 命中片段；
- 用户加 `.sieveignore` 也只在规则引擎语义下成立——模型的"再训练 / 微调"对单用户无意义。

#### 论证 2：误报敏感（GitHub secret scanning 演进史佐证）

GitHub secret scanning（2019 年起）走过的演进路径是公开的：

- **早期版本**：高 recall + 高 FP，被开发者吐槽到不可用；
- **2020-2022**：引入 partner secret 规则（每家 SaaS 自己提供前缀 + validity check API），FP 大幅下降；
- **2023+**：增加自定义模式 + push protection；模型只在"不确定的边界 case"用做轻度筛选。

**核心规律**：生产可用的 secret 检测**通常依赖模式 + validity checks + 定制规则**，逐步扩展，而不是全部押在分类器上。

这条规律对个人开发者群体放大：企业用户对 FP 容忍度高（有审批流、可压队列）；个人开发者一次 FP 就关产品；个人开发者会反复触发同一类 FP，规则引擎能直接通过 `.sieveignore` 学习。

#### 论证 3：数据标注成本

本地小模型的精度天花板取决于训练数据质量。一个能跑的二分类器**至少需要 5,000–10,000 条人工标注样本**（参考公开数据），持续维护需要每月新增 200–500 条标注。

**规则引擎绕开了这个瓶颈**：规则可以持续维护，每条规则的"调优成本"是写一个测试用例 + 跑 benchmark，**数量级是分钟，不是月**。

### Phase 2 引入 ML 的触发条件

只有同时满足一定的量化误报阈值和真实用户主动反馈两个条件，才在 Phase 2 启动 ML 子项目。仅满足其一不启动。具体阈值不随本文档公开发布。

ML 子项目即使启动，也定位为**第二层补充**——规则引擎仍然是第一层，ML 只对规则引擎"高置信但有疑虑"的边界 case 做二次判断。永远不会出现"模型判断为安全，规则判断为危险，跳过规则"的逻辑。

---

## 影响

### 正面影响

1. **可解释性 100%**：每个 Detection 都有明确 rule_id 和命中片段，用户可加 `.sieveignore`；
2. **编译期可枚举**：所有规则在 Rust 代码 / 静态资源中，便于审计、便于复现构建（[ADR-006](./ADR-006-sigstore-reproducible-build.md)）；
3. **没有"模型权重"分发问题**：规则文件是签名 tar.zst（详见 [data-model.md §7](./data-model.md)），不存在模型权重的下发协议设计；
4. **二进制小**：< 20 MB 目标可达；如果带 ONNX runtime + 模型权重，至少 80 MB+；
5. **启动快**：< 500 ms 目标可达；ML 模型加载至少 1–3 秒。

### 负面影响

1. **覆盖范围**：纯文本意图识别（如语义层面的 prompt 钓鱼识别）覆盖弱——这类场景不在 Phase 1 P0 范围（P0 都是结构化 secret 与工具调用）；
2. **规则集维护负担**：需要持续投入维护，通过 GitHub issue 公开样本流程缓解；
3. **未来切换成本**：如果 Phase 2 真的引入 ML，需要重新设计规则与模型的协作逻辑——但二阶段补充层模式（规则在前 / 模型在后）已经规避了"重写"。

### 需要更新的文档

- [architecture.md](./architecture.md) §2 模块矩阵已不含 ML 模块；§7 Phase 2 演进路径已列出 ML 触发条件
- [data-model.md](./data-model.md) §7 规则签名文件格式已对齐"规则即数据"的设计
- `docs/guides/development.md`（待编写）—— 写明"如何加一条规则"流程
- `docs/changelog/CHANGELOG.md`（待编写）—— v0.1.0 启动时记录"纯规则引擎"决策

---

## 相关文档

- [architecture.md](./architecture.md) —— 整体架构、Phase 2 演进路径
- [ADR-001](./ADR-001-rust-tech-stack.md) —— Rust 技术栈（vectorscan-rs 是规则引擎的物质基础）
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— sigstore + reproducible build（规则更新信任链）
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical 永不可关（规则引擎确保可解释 + 可审计）
- [data-model.md](./data-model.md) —— 规则签名文件格式
