# ADR-014: 双层防御——Sieve 代理（SSE 层）+ Claude Code PreToolUse Hook（tool 边界层）

## 状态

**已接受**；部分 supersedes ADR-007 中关于 IN-CR-02~04 / IN-GEN-01~03 的截流实现（fail-closed 原则本身不变）

> 决策日期：2026-04-28
> 范围：Phase 1 入站检测的拦截实现路径
> 关联 PRD：[v1.4 §6.7、§9 第 11 条](../prd/_archive/sieve-prd-v1.5.md)
> 关联 ADR：Partially supersedes [ADR-007](./ADR-007-fail-closed-critical-actions.md) §Week 3 落地范围（截流实现部分）

---

## 背景

### Week 3 落地后发现的问题

ADR-007 §Week 3 落地范围在 SSE 代理层对所有 Critical 命中规则注入 `sieve_blocked` SSE event 后关闭连接。实测中发现两类问题：

**问题 1：污染 Claude Code 上下文**
Hook 类规则（IN-CR-02 危险 shell / IN-CR-03 敏感路径 / IN-CR-04 持久化机制）的工具调用参数已经通过 SSE 传给 Claude Code，代理侧截流只能截断"最终执行"，但 Claude Code 的上下文已经包含了这次工具调用意图。下次对话轮次中模型可能继续沿用该意图。

**问题 2：与 PRD v1.4 §9 第 11 条冲突**
v1.4 新增硬约束："不在 Anthropic API 协议层伪造 tool_use / stop_reason / id / usage"。代理截流时注入的 sieve_blocked event 虽然不伪造模型字段，但在 SSE 流中插入人造事件会改变 Claude Code 收到的消息序列，影响其内部状态机对"对话是否完整"的判断。

### 两类规则的 UX 哲学差异

| 规则类型 | 典型规则 | 拦截时机 | UX 诉求 |
|---------|---------|---------|--------|
| **Hook 类**（IntelliJ PreToolUse 阶段）| IN-CR-02/03/04, IN-GEN-01~03 | tool_use 到达 Claude Code 之后、执行之前 | 终端 y/n 即可，不需要可视化 context |
| **GUI 类**（SSE hold 流 + HIPS 弹窗）| IN-CR-01, IN-CR-05, IN-GEN-04 | SSE 流传输过程中（在 tool_use 到达 Claude Code 之前或同步 hold） | 需要展示完整 context（地址对比 / typed data），用户需要充足读取时间 |

这一差异决定了拦截路径必须分叉：**单纯代理层无法解决上下文污染问题；单纯 hook 层无法处理出站泄露和 SSE 中途检测**。

---

## 决策

### 1. 协议层硬约束（底线，不可放宽）

> **禁止在 SSE 流中伪造 `tool_use` block、`stop_reason`、`id`、`usage` 字段**。
> 这是产品承诺，不是实现细节。

允许的 SSE 操作：
- 截断流（关闭 HTTP response）
- 注入 `sieve_blocked` SSE event（这是 Sieve 自报的拦截通知，不冒充模型内容）
- 发送 `: keep-alive\n\n` comment 行（纯 SSE 协议注释，不影响下游解析）

### 2. Hook 类规则——代理不修改 SSE 流

**适用规则**：IN-CR-02（危险 shell crypto）/ IN-CR-03（敏感路径）/ IN-CR-04（持久化机制，9 条子规则）/ IN-GEN-01（危险 shell）/ IN-GEN-02（远程脚本执行）/ IN-GEN-03（编码后执行）

**拦截流程**：

```
SSE 流 → 代理检测命中 → 写 ~/.sieve/pending/<request_id>.json
                     → SSE 流原样透传给 Claude Code（不修改）
                     
Claude Code 收到 tool_use → 触发 PreToolUse hook → 启动 sieve-hook 进程
sieve-hook → 读 pending 文件 → 终端展示规则 + 工具调用摘要 + y/n
                             → 用户选 n → exit 1 → Claude Code 拒绝执行
                             → 用户选 y → exit 0 → Claude Code 执行
                             → 超时 30s → exit 1（fail-closed）
```

**代理侧实现**：`inbound_hook.rs` pipeline 节点，仅做 IPC pending 文件写入，**不调用 `build_sieve_blocked_sse()`**。

**故障模式**：
- pending 文件写失败：打 ERROR 日志，透传 SSE（hook 进程读不到 pending 文件时 exit 1 fail-closed）；
- sieve-hook 未安装/路径错误：Claude Code 的 hook 注册项 exit 1，Claude Code 拒绝执行该工具调用。

### 3. GUI 类规则——代理 hold 流 + IPC 通知 GUI 弹窗

**适用规则**：IN-CR-01（地址替换）/ IN-CR-05（签名工具 EVM/Solana/Bitcoin）/ IN-GEN-04（markdown exfil）

**拦截流程**：

```
SSE 流中检测命中 → 代理 hold 住 SSE 流
               → 每 25 秒发 `: keep-alive\n\n` comment（防 Claude Code HTTP 超时）
               → JSON-RPC sieve.request_decision → GUI App
               
GUI App → 展示 HIPS 弹窗（120s 倒计时）
        → 用户选"允许"→ sieve.decision_response allow → 代理继续转发 SSE
        → 用户选"拒绝"→ sieve.decision_response deny → 代理截流 + 注入 sieve_blocked event
        → 120s 超时 → 自动 deny（fail-closed）
        
GUI 进程失联（30s 无响应）→ 代理超时 → 截流 + 注入 sieve_blocked event（fail-closed）
```

**代理侧实现**：`inbound_hold.rs` pipeline 节点，hold HTTP response，通过 IPC 通道 A（Unix socket）发送决策请求，等待 `oneshot::Receiver<Decision>`。

### 4. Fail-closed 原则在双层架构下的保证

ADR-007 的 fail-closed 原则**完全保留**，实现路径分两类：

| 场景 | Hook 类 fail-closed 由谁保证 | GUI 类 fail-closed 由谁保证 |
|------|---------------------------|--------------------------|
| 正常流程 | sieve-hook exit 1 → Claude Code 拒绝执行 | 代理截流 + sieve_blocked |
| hook 进程崩溃 | 未注册 hook 时 Claude Code 按默认行为（不执行 hook = 通过）；**需要 sieve setup 注册 hook 为 onError: block** | - |
| GUI App 失联 | - | 代理侧超时 → 截流 + fail-closed |
| IPC 文件写失败 | 代理打 ERROR，hook 读不到 pending → hook exit 1 | - |
| 超时 | sieve-hook 30s exit 1 | 代理 120s 截流 |

**注意**：hook 类的 fail-closed 依赖 Claude Code 把 hook 注册为 `onError: block`，这是 `sieve setup` 命令必须配置的项（见 ADR-015）。

### 5. 删除对 Hook 类规则的 SSE 截流调用

Week 3 落地的 `build_sieve_blocked_sse()` 对 Hook 类规则的调用**必须删除**（`crates/sieve-cli/src/daemon.rs` 中相关分支）。GUI 类规则的 `build_sieve_blocked_sse()` 调用**保留**，语义改为"用户拒绝后的优雅终止 event"。

### 6. 限制与 Phase 2 适配

- sieve-hook 依赖 Claude Code 的 `hooks.preToolUse` 机制（settings.json 注册），是 **Claude Code 特有**的；
- Phase 2 接入 OpenClaw / Hermes 时，Hook 类规则需要各自找对等的 pre-execution hook 机制，或者降级为 GUI 弹窗类；
- 本 ADR 不为 Phase 2 预设实现，遵循 ADR-004 §3 触发条件原则。

---

## 影响

### 正面影响

1. **上下文不污染**：Hook 类 SSE 原样透传，Claude Code 上下文完整，拦截在 execution 边界而非 message 边界；
2. **协议层诚实**：完全满足 PRD v1.4 §9 第 11 条"不伪造"约束；
3. **GUI 体验完整**：GUI 类规则有 120 秒 + 完整可视化 context，用户做决策质量更高；
4. **fail-closed 在所有路径保持**：双层各自 fail-closed，不存在两层都 fail-open 的漏洞路径。

### 负面影响

1. **Hook 类需要 sieve setup 注册**：用户必须运行 `sieve setup` 才能让 hook 生效；未安装的情况下 Hook 类规则静默（只写 pending 文件，但没有 hook 消费）；doctor 命令必须检查 hook 注册状态；
2. **Hook 类拦截点在 execution 边界**：与 GUI 类的"SSE 传输阶段拦截"相比，hook 类来得更晚；模型已经决策发出 tool_use，只是执行被拦了；这对 IN-GEN-02 远程脚本执行是合理的，但对某些想在"模型生成阶段"就中止的需求不满足；
3. **两套代码路径**：`inbound_hook.rs` + `inbound_hold.rs` 各自维护，fuzz 覆盖量翻倍；
4. **Week 3 代码债**：需要删除 `build_sieve_blocked_sse()` 对 Hook 类的调用；这是必须在 Week 5 完成的已知 breaking change。

### 需要更新的文档

- `docs/design/ADR-007-fail-closed-critical-actions.md` —— 末尾加补充段，说明实现路径变化（见本 ADR 任务说明）
- `docs/design/architecture.md` §1、§6 —— 架构图加 sieve-hook 进程 + 双层 IPC 通道
- `docs/design/data-model.md` §3 —— 处置矩阵编码加 `HookTerminal` disposition
- `docs/api/api-reference.md` §6 —— hook exit code 说明

---

## 相关文档

- [PRD-sieve v1.4 §6.7](../prd/_archive/sieve-prd-v1.5.md) —— 双层防御架构
- [PRD-sieve v1.4 §9 第 11 条](../prd/_archive/sieve-prd-v1.5.md) —— "不伪造协议字段"硬约束
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（本 ADR 修改实现路径，不改原则）
- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议（双通道机制）
- [ADR-015](./ADR-015-sieve-setup-tool.md) —— sieve setup（hook 注册 onError: block 的载体）
- [ADR-016](./ADR-016-disposition-matrix-2d.md) —— 处置矩阵二维化（disposition 字段驱动分流）
- [architecture.md](./architecture.md) —— Pipeline 模块与 sieve-hook 进程关系
