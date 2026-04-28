# ADR-016: 处置矩阵从一维四级升级为二维（出站/入站 × 严重度）

## 状态

**已接受**

> 决策日期：2026-04-28
> 范围：规则 manifest 结构 + 处置矩阵编码 + UX 哲学分层
> 关联 PRD：[v1.4 §5.3、§5.4、§9 第 13 条](../prd/sieve-prd-v1.5.md)

---

## 背景

### v1.3 的一维四级模型

v1.3 处置矩阵是单轴四级：Critical / High / Medium / Low，对应 Block / Warn+Block / Warn / Passthrough。

```
Critical → 强制弹窗确认，fail-closed
High     → 弹窗 + 默认允许
Medium   → 状态栏提示
Low      → 静默记录
```

这一模型在 v1.3 单一 UX 路径下够用。但 v1.4 引入了两个让一维模型崩溃的新现实：

**现实 1：出站和入站的 UX 哲学根本不同**

- **出站**（Claude Code 发送用户请求时）：用户**主动发起**，包含敏感内容是"用户可能不知道自己发了什么"——脱敏继续，不打断，不审判；
- **入站**（上游模型响应中推送工具调用时）：用户**被动接收**，工具调用来自"上游不可信环境"——fail-closed，不点不放行，默认怀疑；

强行用同一四级描述两种哲学，导致"Medium 出站"和"Medium 入站"的 UX 行为完全不同，却共用同一 `severity` 字段，让规则 manifest 难以读懂。

**现实 2：`disposition` 是执行路径，不是严重度**

v1.4 §6.7 双层防御把入站规则分为 HookTerminal（终端 y/n）和 GuiPopup（HIPS 弹窗）两条路径，决定因素不是"这个规则多严重"，而是"这个规则的 context 够不够用终端展示"。严重度不够用于路由分流。

**现实 3：出站唯一例外**

OUT-07（BIP39）/ OUT-09（私钥 WIF）/ OUT-10（私钥 raw hex）在出站也必须弹窗——这是 Sieve 差异化点：校验位通过的高确定性私钥/助记词出站，不能"脱敏继续"。这条例外在一维模型里无法表达（Critical 出站被处理为 AutoRedact，但这三条要 GuiPopup）。

---

## 决策

### 1. 二维矩阵结构

两个独立轴：

- **方向轴**（direction）：`Outbound`（出站，Claude Code 发出的请求）/ `Inbound`（入站，上游模型响应）
- **严重度轴**（severity）：Critical / High / Medium / Low（保持兼容）

两轴合并确定 `disposition`（执行路径），但 `disposition` 可以由规则 manifest 显式覆盖（用于 OUT-07/09/10 例外场景）。

### 2. Disposition 枚举

```rust
pub enum Disposition {
    /// 自动脱敏并继续（出站主路径：OUT-01~06/08/11/12）
    AutoRedact,
    /// GUI HIPS 弹窗，hold SSE 流（入站 GUI 类：IN-CR-01/05，IN-GEN-04；出站例外：OUT-07/09/10）
    GuiPopup,
    /// Hook 终端 y/n（入站 Hook 类：IN-CR-02/03/04，IN-GEN-01~03）
    HookTerminal,
    /// 菜单栏状态栏提示，不打断（低优先级入站通用规则）
    StatusBar,
}
```

### 3. 规则 manifest 新增字段

```toml
# 规则 manifest 示例（crates/sieve-rules/rules/*.toml）
[rules.OUT_01]
direction = "Outbound"
severity  = "Critical"
disposition = "AutoRedact"        # 明确写（不走矩阵默认推导）
timeout_seconds = null            # 不适用
default_on_timeout = null         # 不适用

[rules.IN_CR_05]
direction = "Inbound"
severity  = "Critical"
disposition = "GuiPopup"
timeout_seconds = 120
default_on_timeout = "Block"

[rules.IN_CR_02]
direction = "Inbound"
severity  = "Critical"
disposition = "HookTerminal"
timeout_seconds = 30
default_on_timeout = "Block"

[rules.OUT_07]
direction = "Outbound"
severity  = "Critical"
disposition = "GuiPopup"          # 例外：出站高确定性助记词必须弹窗
timeout_seconds = 60
default_on_timeout = "Block"
```

**字段语义**：
- `disposition`：显式指定执行路径；**不可省略**（`#[serde(default)]` 仅用于旧 TOML 兼容过渡期，Week 5 后强制写）
- `timeout_seconds: Option<u32>`：`null` 表示不适用（AutoRedact / StatusBar 场景）；GuiPopup 和 HookTerminal 必须设置
- `default_on_timeout`：`"Block"` | `"Allow"` | `"Redact"`；Critical fail-closed 规则**只允许** `"Block"`

### 4. 矩阵默认推导规则（`disposition` 未明确写时的回退）

| direction × severity | 默认 disposition |
|---------------------|----------------|
| Outbound × Critical | AutoRedact（例外见 §5）|
| Outbound × High | AutoRedact |
| Outbound × Medium | StatusBar |
| Outbound × Low | StatusBar |
| Inbound × Critical | GuiPopup（例外：Hook 类见 §6）|
| Inbound × High | GuiPopup |
| Inbound × Medium | StatusBar |
| Inbound × Low | StatusBar |

矩阵推导仅用于过渡期向后兼容；新规则**必须显式写 disposition**。

### 5. 出站唯一例外（OUT-07 / OUT-09 / OUT-10）

校验位通过的高确定性私钥 / 助记词出站：
- `direction = "Outbound"`
- `severity = "Critical"`
- `disposition = "GuiPopup"`（覆盖矩阵默认的 AutoRedact）

这是 Sieve 相对竞品的差异化点：普通 DLP 工具只做 AutoRedact，Sieve 对高确定性加密资产泄露强制人工确认。

这条规则在 manifest 中必须显式写 `disposition = "GuiPopup"`，不走矩阵推导，防止未来矩阵调整意外影响。

### 6. Hook 类规则显式标注

IN-CR-02/03/04 和 IN-GEN-01~03 虽然 `severity = "Critical"`，但 `disposition = "HookTerminal"`（覆盖矩阵默认的 GuiPopup）。理由见 ADR-014——这些规则的拦截点在 Claude Code PreToolUse 边界，GUI 弹窗路径不适用。

### 7. 影响代码模块

- `crates/sieve-rules/src/manifest.rs`：`RuleEntry` 加 `disposition: Disposition` + `timeout_seconds: Option<u32>` + `default_on_timeout: Option<DefaultAction>`
- `crates/sieve-rules/src/critical_lock.rs`：`FAIL_CLOSED_RULES` 仍维护 Critical 规则集；同时新增 `HOOK_RULES`（HookTerminal disposition 集）和 `GUI_RULES`（GuiPopup disposition 集）
- `crates/sieve-core/src/pipeline/engine_adapter.rs`：按 disposition 路由到 `outbound_redact` / `inbound_hold` / `inbound_hook` 三条分支
- `crates/sieve-cli/src/config.rs`：新增 `preset` 字段（Minimal / Standard / Paranoid），影响 StatusBar 规则的阈值，不影响 Critical

---

## 影响

### 正面影响

1. **UX 哲学分层清晰**：出站"帮用户擦屁股不打断"/ 入站"fail-closed 不点不放行"在 manifest 层面明确表达；
2. **例外可表达**：OUT-07/09/10 的 GuiPopup 异常行为有了显式字段，不再靠隐式特判；
3. **可进化**：`Disposition` 枚举新增变体不破坏现有规则（manifest 用旧值仍然有效）；
4. **规则可读性**：工程师读规则 TOML 文件即可理解该规则的 UX 行为，不需要交叉查文档。

### 负面影响

1. **manifest 文件改动量大**：所有现有规则 TOML 需要补 `disposition` + `timeout_seconds` + `default_on_timeout` 三个字段（C2 子代理任务）；
2. **向后兼容过渡期**：旧 TOML 没有 `disposition` 字段时走矩阵推导，但新代码路径已按 `disposition` 路由，过渡期行为需要测试覆盖；
3. **`engine_adapter.rs` 复杂度提升**：从单路径改为三路 disposition 分流，需要新增 fuzz 用例；
4. **preset 字段 UX 设计**：Minimal / Standard / Paranoid 对 StatusBar 规则的影响边界需要在 SPEC-002 进一步明确（本 ADR 不做完整规范，只定义字段存在）。

### 需要更新的文档

- `docs/design/data-model.md` §3 —— 处置矩阵编码改为二维表格 + disposition 枚举
- `docs/api/api-reference.md` §5 —— 处置矩阵 → HTTP 行为表（出站脱敏不返 426 等）
- `docs/requirements/user-stories.md` US-09 —— 处置矩阵 UX 哲学描述
- `docs/changelog/CHANGELOG.md` —— `[BREAKING]` 标记：manifest 新增必填字段 disposition

---

## 相关文档

- [PRD-sieve v1.4 §5.3、§5.4](../prd/sieve-prd-v1.5.md) —— 处置矩阵二维化与超时策略表
- [PRD-sieve v1.4 §9 第 13 条](../prd/sieve-prd-v1.5.md) —— "出站脱敏不打断"硬约束
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（Critical 的 default_on_timeout 只能是 Block）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— disposition 驱动的双层 pipeline 分流
- [data-model.md](./data-model.md) —— 处置矩阵编码与配置字段
- [architecture.md](./architecture.md) —— engine_adapter 路由逻辑
