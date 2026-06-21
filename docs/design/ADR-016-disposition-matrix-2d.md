# ADR-016: 处置矩阵从一维四级升级为二维（出站/入站 × 严重度）

## 状态

**已接受**

> 决策日期：2026-04-28
> 范围：规则 manifest 结构 + 处置矩阵编码 + UX 哲学分层

---

## 背景

### v1.3 的一维四级模型

v1.3 处置矩阵是单轴四级：Critical / High / Medium / Low，对应 Block / Warn+Block / Warn / Passthrough。

这一模型在 v1.3 单一 UX 路径下够用。但 v1.4 引入了两个让一维模型崩溃的新现实：

**现实 1：出站和入站的 UX 哲学根本不同**

- **出站**（Claude Code 发送用户请求时）：用户**主动发起**，包含敏感内容是"用户可能不知道自己发了什么"——脱敏继续，不打断，不审判；
- **入站**（上游模型响应中推送工具调用时）：用户**被动接收**，工具调用来自"上游不可信环境"——fail-closed，不点不放行，默认怀疑；

强行用同一四级描述两种哲学，导致"Medium 出站"和"Medium 入站"的 UX 行为完全不同，却共用同一 `severity` 字段，让规则 manifest 难以读懂。

**现实 2：`disposition` 是执行路径，不是严重度**

v1.4 双层防御把入站规则分为 HookTerminal（终端 y/n）和 GuiPopup（HIPS 弹窗）两条路径，决定因素不是"这个规则多严重"，而是"这个规则的 context 够不够用终端展示"。严重度不够用于路由分流。

**现实 3：出站存在需要弹窗的高确定性加密资产例外**

校验位通过的高确定性私钥/助记词出站属于 Sieve 差异化点：这类场景在出站方向也必须弹窗人工确认，不能走自动脱敏继续的主路径。这条例外在一维模型里无法表达。

---

## 决策

### 1. 二维矩阵结构

两个独立轴：

- **方向轴**（direction）：`Outbound`（出站，Claude Code 发出的请求）/ `Inbound`（入站，上游模型响应）
- **严重度轴**（severity）：Critical / High / Medium / Low（保持兼容）

两轴合并确定 `disposition`（执行路径），但 `disposition` 可以由规则 manifest 显式覆盖（用于出站高确定性加密资产例外场景）。

### 2. Disposition 枚举

```rust
pub enum Disposition {
    /// 自动脱敏并继续（出站主路径）
    AutoRedact,
    /// GUI HIPS 弹窗，hold SSE 流（入站 GUI 类；出站高确定性加密资产例外）
    GuiPopup,
    /// Hook 终端 y/n（入站 Hook 类）
    HookTerminal,
    /// 菜单栏状态栏提示，不打断（低优先级入站通用规则）
    StatusBar,
}
```

### 3. 规则 manifest 新增字段

规则 manifest（`crates/sieve-rules/rules/*.toml`）新增三个字段：

**字段语义**：
- `disposition`：显式指定执行路径；**不可省略**（`#[serde(default)]` 仅用于旧 TOML 兼容过渡期，Week 5 后强制写）
- `timeout_seconds: Option<u32>`：`null` 表示不适用（AutoRedact / StatusBar 场景）；GuiPopup 和 HookTerminal 必须设置
- `default_on_timeout`：`"Block"` | `"Allow"` | `"Redact"`；Critical fail-closed 规则**只允许** `"Block"`

具体规则的 timeout 窗口与超时默认动作不随本文档公开发布，写在对应规则 manifest TOML 中。

### 4. 矩阵默认推导规则（`disposition` 未明确写时的回退）

矩阵推导规则按 direction × severity 两轴给出默认 disposition，优先覆盖入站 Critical 走 GuiPopup / Hook 类例外走 HookTerminal / 出站高确定性加密资产走 GuiPopup。

具体的完整映射表写在各规则 manifest 中，矩阵推导仅用于过渡期向后兼容；新规则**必须显式写 disposition**。

### 5. 出站高确定性加密资产弹窗例外

校验位通过的高确定性私钥 / 助记词出站，Sieve 对此类场景强制人工确认（而非仅 AutoRedact）——高确定性加密资产泄露一旦发生不可逆。

受影响规则在 manifest 中必须显式写 `disposition = "GuiPopup"`，不走矩阵推导，防止未来矩阵调整意外影响。

### 6. Hook 类规则显式标注

部分入站 Critical 规则虽然 `severity = "Critical"`，但 `disposition = "HookTerminal"`（覆盖矩阵默认的 GuiPopup）。理由见 ADR-014——这些规则的拦截点在 Claude Code PreToolUse 边界，GUI 弹窗路径不适用。

### 7. 影响代码模块

- `crates/sieve-rules/src/manifest.rs`：`RuleEntry` 加 `disposition: Disposition` + `timeout_seconds: Option<u32>` + `default_on_timeout: Option<DefaultAction>`
- `crates/sieve-rules/src/critical_lock.rs`：`FAIL_CLOSED_RULES` 仍维护 Critical 规则集；同时新增 `HOOK_RULES`（HookTerminal disposition 集）和 `GUI_RULES`（GuiPopup disposition 集）
- `crates/sieve-core/src/pipeline/engine_adapter.rs`：按 disposition 路由到 `outbound_redact` / `inbound_hold` / `inbound_hook` 三条分支
- `crates/sieve-cli/src/config.rs`：新增 `preset` 字段（Minimal / Standard / Paranoid），影响 StatusBar 规则的阈值，不影响 Critical

---

## 影响

### 正面影响

1. **UX 哲学分层清晰**：出站"帮用户擦屁股不打断"/ 入站"fail-closed 不点不放行"在 manifest 层面明确表达；
2. **例外可表达**：出站高确定性加密资产的 GuiPopup 异常行为有了显式字段，不再靠隐式特判；
3. **可进化**：`Disposition` 枚举新增变体不破坏现有规则（manifest 用旧值仍然有效）；
4. **规则可读性**：工程师读规则 TOML 文件即可理解该规则的 UX 行为，不需要交叉查文档。

### 负面影响

1. **manifest 文件改动量大**：所有现有规则 TOML 需要补 `disposition` + `timeout_seconds` + `default_on_timeout` 三个字段；
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

- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（Critical 的 default_on_timeout 只能是 Block）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— disposition 驱动的双层 pipeline 分流
- [data-model.md](./data-model.md) —— 处置矩阵编码与配置字段
- [architecture.md](./architecture.md) —— engine_adapter 路由逻辑
