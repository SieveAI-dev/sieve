# SPEC-002: HIPS 弹窗行为规格

> Version: v1.0 — 2026-04-28
> Status: Stable
> 关联：HIPS 弹窗 / 终端确认交互能力（v1.4 引入）

---

## 1. 目标

定义 Sieve 所有弹窗/提示机制的完整行为规格，包括：
- GUI 弹窗 vs Hook 终端弹窗的触发条件
- 超时策略与默认行为
- 倒计时视觉三段设计
- 多 issue 合并弹窗逻辑
- SSE keep-alive 机制
- Settings preset 矩阵
- 硬约束与失败降级

---

## 2. 弹窗类型与触发矩阵

### 2.1 两种弹窗机制

| 机制 | 实现 | 使用场景 |
|------|------|---------|
| **GUI 弹窗** | macOS SwiftUI 独立进程，通过 Unix socket JSON-RPC 与主代理通信 | 内容需可视化对比；内容长需阅读；需展示链上信息 |
| **Hook 终端弹窗** | `sieve-hook` TTY stdout，Claude Code PreToolUse hook | tool_use 类简单 y/n 决策；用户已在终端；一句话能讲清的危险 |

### 2.2 触发条件矩阵

| 检测项 | 方向 | 弹窗类型 | 触发时机 |
|-------|------|---------|---------|
| OUT-01~05, OUT-12 | 出站 | 无弹窗 | 自动脱敏，状态栏 5 秒提示 |
| OUT-06, OUT-08 | 出站 | GUI 弹窗 | 主代理出站 filter 命中后 hold 请求 |
| OUT-07, OUT-09, OUT-10 | 出站 | GUI 弹窗（强警告） | 主代理出站 filter 命中后 hold 请求 |
| OUT-11 | 出站 | 无弹窗 | 状态栏标记 |
| IN-CR-01 | 入站 | GUI 弹窗 | SSE 流 hold，主代理检测到地址替换后暂停转发 |
| IN-CR-02, IN-CR-03, IN-CR-04 | 入站 | Hook 终端弹窗 | SSE 流正常转发，sieve-hook 在 PreToolUse 拦截 |
| IN-CR-05 | 入站 | GUI 弹窗 | SSE 流 hold，主代理检测到签名 tool_use 后暂停转发 |
| IN-GEN-01, IN-GEN-02, IN-GEN-03 | 入站 | Hook 终端弹窗 | SSE 流正常转发，sieve-hook 在 PreToolUse 拦截 |
| IN-GEN-04 | 入站 | GUI 弹窗 | SSE 流 hold，主代理检测到 markdown exfil 后暂停转发 |
| IN-GEN-05 | 入站 | 无弹窗 | 状态栏标记 |

**GUI 弹窗类共同特征**：主代理 hold SSE 流（停止向 Claude Code 转发字节），等待 GUI 进程返回决策后继续或中止。

**Hook 类共同特征**：主代理不修改 SSE 流，正常转发；写 pending 文件后由 `sieve-hook` 在 Claude Code 的 PreToolUse 阶段拦截（见 SPEC-001）。

---

## 3. 超时策略表（默认 preset）

每行标注 `default_on_timeout` 语义：

| 检测项 | 超时时长 | 超时默认 | 弹窗类型 |
|-------|---------|---------|---------|
| OUT-01~05, OUT-12 | 无超时 | 自动脱敏（不弹窗）| — |
| OUT-06, OUT-08 | 15 秒 | `allow`（脱敏后发送）| GUI 弹窗 |
| OUT-07, OUT-09, OUT-10 | 60 秒 | `deny`（完全拦截）| GUI 弹窗强警告 |
| OUT-11 | 无超时 | 状态栏标记（不弹窗）| — |
| IN-CR-01 | 60 秒 | `deny`（拒绝）| GUI 弹窗 |
| IN-CR-02 | 30 秒 | `deny`（拒绝）| Hook 终端弹窗 |
| IN-CR-03 | 30 秒 | `deny`（拒绝）| Hook 终端弹窗 |
| IN-CR-04 | 60 秒 | `deny`（拒绝）| Hook 终端弹窗 |
| IN-CR-05 | 120 秒 | `deny`（拒绝）| GUI 弹窗 |
| IN-GEN-01~03 | 30 秒 | `deny`（拒绝）| Hook 终端弹窗 |
| IN-GEN-04 | 30 秒 | `deny`（拒绝）| GUI 弹窗 |
| IN-GEN-05 | 无超时 | 状态栏标记（不弹窗）| — |

**设计原则**：
1. 危险等级越高，给的时间越长——用户需要时间读懂才能决策
2. 出站默认 fail-open（脱敏后继续），入站默认 fail-closed（拒绝）
3. 签名类 120 秒——typed data 需要时间阅读
4. 校验位通过的私钥/助记词例外——即使是出站也强弹窗，默认拒绝

---

## 4. 倒计时视觉三段设计

适用于所有有超时的 GUI 弹窗（Hook 终端弹窗参考 §4.3）。

### 4.1 GUI 弹窗三段

进度条从满到空，颜色随时间变化：

```
阶段 1（前 50% 时间）：温和
  - 进度条颜色：系统蓝色（macOS accent color）
  - 倒计时数字：次要文本颜色（gray）
  - 动效：平滑减少

阶段 2（中间 30% 时间）：警告
  - 进度条颜色：橙色 #FF6B00
  - 倒计时数字：橙色，字重加粗
  - 动效：平滑减少

阶段 3（最后 20% 时间）：紧急
  - 进度条颜色：红色 #FF3B30（macOS 红）
  - 倒计时数字：红色，字重加粗
  - 动效：闪烁（0.5s 周期，opacity 1.0 → 0.5）
```

示例：IN-CR-05（120 秒超时）的三段时间边界：
- 阶段 1：120s → 60s（前 60 秒）
- 阶段 2：60s → 24s（中间 36 秒）
- 阶段 3：24s → 0s（最后 24 秒）

### 4.2 阶段切换时机

阶段切换在 UI 主线程每秒更新一次，采用 `remaining_seconds` 计算：

```
remaining_ratio = remaining_seconds / timeout_seconds
阶段 1: remaining_ratio > 0.50
阶段 2: 0.20 < remaining_ratio ≤ 0.50
阶段 3: remaining_ratio ≤ 0.20
```

### 4.3 Hook 终端弹窗倒计时

Hook 终端弹窗使用 ANSI 转义码在同一行刷新倒计时数字。无颜色三段区分（简化为两段）：
- `remaining > 10s`：白色数字
- `remaining ≤ 10s`：ANSI 红色（`\x1b[31m`）数字

---

## 5. 多 issue 合并弹窗规则

### 5.1 合并触发条件

同一个 SSE 请求中检测到多个问题（不同 rule_id）时，必须合并到一个 GUI 弹窗，不允许弹多个窗口。

### 5.2 合并时的弹窗内容

```
标题："Sieve 检测到 N 个安全问题"

列表（按 severity 排序，critical > high > medium）：
  🚨 [必须确认] IN-CR-05 签名工具调用 (signTransaction)
  ⚠  [高危] IN-GEN-04 Markdown 图片外链
  ℹ  [中危] IN-GEN-01 危险 shell 模式

倒计时：由最高优先级规则的 timeout_seconds 决定（取最小值，时间最短的为基准）
默认超时行为：所有 critical 项 deny，其余按各自 default_on_timeout
```

### 5.3 多 issue 合并按钮 Schema

```jsonc
// GUI 收到的 request_decision 请求（多 issue 情形）
{
  "id": "req_merged_xyz",
  "method": "request_decision",
  "params": {
    "merged": true,
    "issues": [
      {
        "rule_id": "IN-CR-05",
        "severity": "critical",
        "title": "签名工具调用：signTransaction",
        "details": { "tool_name": "signTransaction", "args_preview": "..." },
        "default_on_timeout": "deny"
      },
      {
        "rule_id": "IN-GEN-04",
        "severity": "high",
        "title": "Markdown 图片外链检测",
        "details": { "url": "https://attacker.com/steal?q=..." },
        "default_on_timeout": "deny"
      }
    ],
    "timeout_seconds": 30,           // 取各 issue timeout 的最小值
    "default_on_timeout": "deny"     // 有任一 critical 则整体 deny
  }
}

// GUI 返回的决策（多 issue 情形）
{
  "id": "req_merged_xyz",
  "result": {
    "merged_decision": "partial",    // "deny_all" | "allow_all" | "partial"
    "per_issue": [
      { "rule_id": "IN-CR-05", "decision": "deny" },
      { "rule_id": "IN-GEN-04", "decision": "allow" }
    ],
    "remember": false
  }
}
```

**按钮布局**（优先级从左到右）：

| 按钮文本 | merged_decision 值 | 可用条件 |
|---------|-------------------|---------|
| 拒绝全部 | `deny_all` | 始终可用 |
| 仅允许非 Critical 项 | `partial`（Critical deny，其余 allow）| 存在非 Critical 项时显示 |
| 全部允许 | `allow_all` | 无 Critical 项时才显示（有 Critical 项时此按钮不渲染）|

**实现约束**：`allow_all` 按钮对包含任意 Critical issue 的合并弹窗完全不渲染，不是置灰——置灰仍然暗示可被解锁。

### 5.4 合并 vs 独立弹窗的选择逻辑

```
如果 pending issues 属于同一 request_id：
  → 合并（GUI 类合并为一个 JSON-RPC 调用）
  → Hook 类暂不合并（各自独立 TTY 提示，Phase 2 考虑队列）
如果 pending issues 属于不同 request_id：
  → 各自独立弹窗（不跨请求合并）
```

---

## 6. SSE keep-alive（GUI 弹窗 hold 流期间）

### 6.1 目的

GUI 弹窗类规则 hold 流期间，HTTP 连接可能因 Claude Code 客户端超时（通常 30~60 秒）而被 abort。IN-CR-05 超时 120 秒，必须保活连接。

### 6.2 keep-alive 格式

在 hold 流期间，主代理每 25 秒向 Claude Code 发送一个 SSE 注释行：

```
: keep-alive\n\n
```

- `: ` 开头的行是 SSE 注释，不触发 `message` 事件，Claude Code SSE 解析器忽略
- 不修改 `data:` 字段，不影响 stop_reason / id / usage / type（不在协议层伪造模型输出）
- `\n\n` 是 SSE 事件分隔符，保持协议合规

### 6.3 keep-alive 发送时机

```
hold 开始后：
  T+0s:   主代理 hold 流，停止转发后续字节
  T+25s:  发送 ": keep-alive\n\n"
  T+50s:  发送 ": keep-alive\n\n"
  T+75s:  发送 ": keep-alive\n\n"
  ...
  T+timeout_seconds: 超时，按 default_on_timeout 处置，结束 hold
```

**最长 hold 时长** = 规则的 `timeout_seconds`（IN-CR-05 = 120s，最多发 4 条 keep-alive）。

### 6.4 hold 结束后的 SSE 流处置

| 决策 | 主代理行为 |
|------|----------|
| `allow`（用户放行）| 继续转发剩余 SSE 字节（从 hold 点继续）|
| `deny` / 超时 | 向 Claude Code 发送 SSE error event，关闭连接：`data: {"type":"error","error":{"type":"sieve_blocked","message":"..."}}\n\n` |

注意：deny 时发送的 `sieve_blocked` event 是用户授权的中止，不是 Sieve 替模型说话（符合不在协议层伪造模型输出的原则）。

---

## 7. Settings Preset 矩阵

### 7.1 四档 Preset 定义

| Preset | 时长修正 | 出站行为修正 | 入站行为修正 |
|--------|---------|------------|------------|
| **Strict** | 所有有超时的规则 timeout_seconds × 0.5（向上取整）| OUT-06~10 弹窗改为拒绝（不允许脱敏后发送）| 无变化（入站已是 fail-closed）|
| **Default** | 按 §3 超时表原始值 | 按 §3 | 按 §3 |
| **Relaxed** | 所有有超时的规则 timeout_seconds × 2 | 按 §3 | IN-GEN-01~03 改为 fail-open（hook 放行，仅记录）|
| **Custom** | 每条规则单独配置 | 每条规则单独配置 | 每条规则单独配置 |

### 7.2 Preset 配置 Schema（`sieve.toml` 片段）

```toml
[preset]
mode = "default"   # "strict" | "default" | "relaxed" | "custom"

# Custom 模式下的逐规则覆盖（其他模式忽略此段）
[preset.custom]
"IN-CR-02" = { timeout_seconds = 60, default_on_timeout = "deny" }
"IN-GEN-01" = { timeout_seconds = 60, default_on_timeout = "allow" }
# ...其他规则使用 Default 值
```

### 7.3 critical_lock 强制（加载阶段）

以下规则无论 preset 如何配置，以下属性**不允许覆盖**：

```
critical_lock 保护的规则及约束：
  IN-CR-01: default_on_timeout 强制 = "deny"，不允许 remember=true
  IN-CR-05: default_on_timeout 强制 = "deny"，不允许 remember=true
  OUT-07:   default_on_timeout 强制 = "deny"
  OUT-09:   default_on_timeout 强制 = "deny"
  OUT-10:   default_on_timeout 强制 = "deny"
```

`sieve-rules` crate 在加载 preset 配置时，对上述规则的 `default_on_timeout` 字段调用 `critical_lock::validate()`，验证失败直接 panic（daemon 启动失败，而非静默忽略）。

**永久白名单（remember=true）硬约束**：`critical_lock` 中定义的规则，写入 `.sieveignore` 的路径在 GUI 和 hook 端均被禁止。Custom preset 下也不例外。

---

## 8. 失败模式与降级

### 8.1 GUI 进程未启动或崩溃

当主代理检测到需要 GUI 弹窗但 Unix socket 连接失败（`ENOENT` 或连接 refused）时：

1. 主代理通过 macOS `UNUserNotificationCenter`（Swift 侧 API，或通过 `osascript` 调用）发系统通知
2. 通知内容：`"Sieve 拦截：[rule_title]，GUI 未运行，已自动拒绝"`
3. 主代理按 `default_on_timeout` 处置（Critical = deny，其余按规则定义）
4. 记录 AuditEvent，`action = "auto_denied_gui_unavailable"`

**硬约束**：GUI 不可用时，Critical 规则严格执行 deny，不降级为 allow。

### 8.2 GUI 进程存在但响应超时

GUI 进程连接成功，但在 `timeout_seconds` 内未返回决策（GUI 卡死或用户未操作）：

1. 主代理在 `timeout_seconds + 2s`（容错余量）后视为超时
2. 按 `default_on_timeout` 处置
3. 记录 AuditEvent，`action = "auto_decided_timeout"`

### 8.3 TTY 不可用（headless 环境）

`sieve-hook` 检测到 stdin 不是 TTY（`isatty(0) == false`）：
1. 按 pending 文件的 `default_on_timeout` 决定
2. 写 decisions 文件，`by = "timeout"`
3. exit 0 或 exit 1

---

## 9. v2.0 暂停 / Preset 切换 / 弹窗取消（2026-05-02 追加）

> 触发：2026-05-02 IPC 协议新增 GUI 控制面方法（`set_paused` / `set_preset` / `set_preset_overrides` / `request_decision_canceled`）后，弹窗主路径需要补几条与之相关的行为约束。
> 范围：本节仅补这三类操作对**弹窗显隐 / 倒计时 / 多 issue 合并**的影响，详细 IPC schema 见 [SPEC-005](SPEC-005-ipc-protocol.md)，不在此重复。

### 9.1 暂停状态下的弹窗行为

`paused = true` 期间（来自 IPC `set_paused` 或 daemon 内部触发），弹窗触发矩阵在 §2.2 基础上做如下覆盖：

| 检测项类别 | 暂停期间是否弹窗 | 暂停期间处置 |
|-----------|---------------|------------|
| **Critical 锁名单**（IN-CR-01 / IN-CR-05 / OUT-07 / OUT-09 / OUT-10）| **仍然弹窗**，不可暂停 | 与 §2.2 完全一致 |
| Hook 类 Critical（IN-CR-02/03/04 / IN-GEN-01~03）| **仍然弹终端**，不可暂停 | 与 §2.2 完全一致 |
| 非 Critical GuiPopup（如 IN-GEN-04 默认值）| 跳过弹窗 | 直接按 `default_on_timeout` 处置（多数为 `deny`），写 audit `kind=auto_decided_paused` |
| AutoRedact（OUT-01~05/12 等）| 不弹窗（与 §2.2 同）| 自动脱敏正常工作（暂停**不**关闭脱敏）|
| StatusBar | 不弹窗（与 §2.2 同）| 仅状态栏图标更新，**不**触发 Toast（避免暂停期还刷屏）|

设计原则（与 Critical fail-closed、Critical 全版本不可关的核心安全约束对齐）：
- "暂停"是**用户对噪音的宽容**，不是对**安全**的让步——内置 Critical 不可暂停。
- 暂停期间被 auto-deny 的非 Critical 弹窗事件**正常审计**（`action_taken=auto_decided_paused`），用户在历史窗口可见"我暂停时被 Sieve 替我拒了 N 件事"。
- 暂停期间不发 `event_notify` 给 GUI（避免 Toast 刷屏）；菜单栏仍按 `paused` 状态（灰色 ◌）展示，命中计数仍递增。

### 9.2 Preset 切换对正在 hold 的弹窗的影响

`set_preset` / `set_preset_overrides` 在以下情况触发：

| 时机 | 行为 |
|------|------|
| 正在 hold 中的请求（弹窗已发但用户未答复）| **不修改**当前弹窗的 `timeout_seconds` / `default_on_timeout` —— 已签发的弹窗按签发时的 preset 走完 |
| 排队中的请求（已发到 daemon 但未弹窗）| 在 GUI 实际弹出前用新 preset 重新计算 timeout / default_on_timeout |
| 切换后新到达的命中 | 用新 preset |

设计理由：用户已经在看的弹窗中途变 timeout 会破坏倒计时三段视觉的连续性，且容易让用户做出基于旧时间预算的错判。

### 9.3 `request_decision_canceled` 的 GUI 行为

daemon 通过 IPC notification 发出 `request_decision_canceled` 时，GUI 必须按以下顺序处理：

1. 查询本地 `request_id` 状态：
   - **未弹窗**（在排队中）→ 移除排队条目，无 UI 反应。
   - **正在显示**（用户尚未点击）→ 立刻关闭弹窗 + 在屏幕中心显示 1.5s 浮层提示："此请求已超时 / 上游断开，Sieve 已自动 `<auto_decision>`"。`auto_decision` 来自 notification params。
   - **已点击但 IPC 在途**（已 `decision_response` 还未 ack）→ 忽略 cancel notification（用户决策为准；daemon 端去重保护）。
2. 在历史窗口中（如打开）追加一条该 `request_id` 的"auto-decided"标记行（关联到 audit.db 由 daemon 写的 `kind=auto_decided_*` 事件）。
3. 写 GUI log，含 `reason` 字段。

`reason` 取值（与 [SPEC-005](SPEC-005-ipc-protocol.md) §5.8 `cancel_reason` 对齐）：

| reason | 含义 | GUI 浮层文案 |
|--------|------|------------|
| `timeout` | daemon 侧倒计时已到 | "请求已超时，已自动 {decision}" |
| `upstream_disconnected` | Anthropic / OpenAI / Gemini 上游连接断了 | "上游连接断开，请求已自动 {decision}" |
| `duplicate_suppressed` | 同一 request_id 因灰名单或重复抑制策略提前决策 | "Sieve 已根据灰名单自动 {decision}" |
| `daemon_shutdown` | daemon 正在关停 | "Sieve daemon 已关停" |

### 9.4 多 GUI 实例的弹窗广播（边缘场景）

理论上同时连两个 GUI 进程到同一 daemon 的场景（dogfood 调试 / 误启动两次）：
- daemon 把同一个 `sieve.request_decision` 同时发给所有连接的 GUI。
- 任一 GUI 先回 `decision_response` 后，daemon 用该决策处置，并对其他 GUI 发 `request_decision_canceled` `reason="resolved_by_peer"`（新增 reason 值，归在 §9.3 表后续追加）。
- 其他 GUI 收到 `resolved_by_peer` 时关闭弹窗但**不**显示浮层（避免迷惑用户）。

### 9.5 与现有 §3 / §4 / §5 章节的关系

- §3 超时策略表 **不变**，是 default preset 下的字面值；strict / relaxed 由 §7.1 修正系数派生；custom 由 `set_preset_overrides` 覆盖。
- §4 倒计时三段视觉 **不变**，与 preset 无关。
- §5 多 issue 合并规则 **不变**，paused 状态下若多 issue 中含 Critical → 整组弹窗（与 paused 无关），不含 Critical → 整组按 §9.1 走 auto-deny（不弹）。

### 9.6 Critical 锁防线一在 popup 行为侧的兑现

弹窗发起时 daemon 必须填 `params.allow_remember`：

| 规则归属 | `allow_remember` |
|---------|------------------|
| `critical_lock::FAIL_CLOSED_RULES` 内（IN-CR-01 / IN-CR-05 / OUT-07 / OUT-09 / OUT-10）| `false`（强制）|
| 其他系统规则 + 用户规则 | `true` |

GUI 在 `false` 时**禁止渲染** Remember checkbox（不允许灰显代替；GUI 侧交互细节见独立仓库 `sieve-gui-macos`）。daemon 在 `decision_response.remember=true` 时做防线二二次校验，违规写 `kind=critical_lock_blocked`。

---

## 10. 未决事项（TBD）

| 编号 | 问题 | 选项 |
|------|------|------|
| TBD-1 | GUI 弹窗多 issue 合并时，"仅允许非 Critical 项"按钮的精确文案 | A. "允许 {N} 个非关键项"；B. "仅允许 {rule_id_list}"；当前选 A，Week 5 UI 阶段确认 |
| TBD-2 | Relaxed preset 中 IN-GEN-01~03 fail-open 是否记录审计日志 | A. 记录（带 action=relaxed_pass）；B. 不记录（噪音）；推荐 A |
| TBD-3 | keep-alive comment 的确切字节内容是否需要 Anthropic 侧确认兼容性 | Week 3 dogfood 期间验证，Claude Code SDK 若报错则换 `: \n\n`（空注释）|
