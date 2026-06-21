# ADR-028: IPC 协议中性化——去 GUI 假设 + sieve-ipc 内部模块化 + headless decision path

## 状态

**Proposed**

> 决策日期：2026-05-05
> 范围：Phase 2.x（v2.x 工程项，不阻塞 GA；GA 前完成）；sieve-ipc crate 内部重构（不拆 crate）
> 关联 PRD：v2.0 §6.5 IPC 协议、§6.6 GUI App
> 关联 ADR：**延伸 ADR-013**（不 supersede；ADR-013 关于 JSON-RPC over UDS + 文件锁的决策仍有效，本 ADR 在其上做术语清洗 + 模块结构改造）

---

## 背景

### 当前 IPC 协议外漏 GUI 假设

SPEC-005（1750 行 IPC 协议）大量使用 `popup` / `request_decision_canceled` / 「GUI 端解析容错」等术语。协议层 method 名 `decision.popup` / `decision.popup_canceled` 把「用户决策」与「GUI 弹窗」绑死。后果是：协议把 GUI 隐性提升为「特权 client」，其他 client 形态（CLI / TUI / webhook / LLM-as-decider / 远程 web 控制台）成了二等公民。

这违背 sieve「unix-style 工具」的设计方向——iptables-like 的工具不预设 frontend 形态（同期推进的 ADR-026 port listener routing 持相同方向）。

### HIPS hold-stream 设计强依赖 GUI 在线

入站规则 IN-CR-01/05 类「hold SSE 流 + 等用户确认 + 默认超时处置」的设计强假设有 GUI 进程接收 IPC 事件。实际场景：

- 远程 SSH 跑 daemon，无桌面
- GUI crash 或未启动
- 纯 tmux + CLI 工作流
- headless server 部署（CI 沙箱、共享开发机）

这些场景下当前 fallback 只有两条路：fail-closed 砍流，或卡死等超时。没有第三选择，也没有明确的用户提示。

### schema 跟 server 实现耦合

`sieve-ipc` crate 当前把 wire types（envelope / decision / handshake schema）跟 server 实现（accept loop / file lock IO / 业务 dispatch）混在同一层。加新 client（headless CLI 决策面 / 未来 sieve-tui）时要 import 整个 sieve-ipc，被迫拖进 server 依赖。schema 想加自动化测试（property-based / fuzz）也被 server runtime 依赖拖累。

### 拆独立 sieve-protocol crate / 仓的方案被否决

独立 crate 方案是自然的想法，但目前所有触发条件均未达到：

| 触发条件 | 当前状态 |
|---------|---------|
| 第二个 Rust client（sieve-tui / sieve-webhook-bridge）真实立项 | 未立项 |
| SPEC-005 双写（daemon 1750 行 vs GUI 284 行）成为 dogfood 阶段的实际 bug 来源 | 未出现 |
| 出现第三个语言的 client（除 Rust + Swift 外） | 未出现 |
| 维护团队规模增长到需要并行多 client 协作 | 未满足 |

当前双语言 + 早期工程阶段，拆 crate / 拆仓属于过度设计（YAGNI）。**触发条件发生时再写一个新 ADR 决策升级，本 ADR 不预判。**

---

## 决策

### 1. SPEC-005 术语中性化（纯文档 + 字段重命名）

把协议中所有 GUI 假设的术语清洗为中性：

| 旧术语 | 新术语 | 备注 |
|--------|--------|------|
| `decision.popup` | `decision.pending` | method 名重命名 |
| `decision.popup_canceled` | `decision.canceled` | 同上 |
| 「GUI 端解析容错」段落 | 「client 解析容错」段落 | 文档措辞 |
| 「GUI 端期望行为」段落 | 「client 期望行为」段落 | 同上 |
| 「弹窗」「window」「popup」 | 「decision request」「decision event」 | 全文替换 |
| 「GUI 状态机」（在 SPEC-005 内） | 「client 状态机」；GUI-only UI 状态机（hold / paused / disconnected 等显示行为）搬到 GUI 仓 SPEC-002 | 拆分 |

**兼容性策略**：旧 method 名 `decision.popup` / `decision.popup_canceled` 作为 deprecated alias 保留**一个 minor 版本**（v2.x，到下次协议 bump 时移除）。daemon 同时接受新旧名，新代码只发新名。

SPEC-005 §0 文档定位段落同步更新：

- 明确 daemon IPC 协议只描述**语义**（decision / heartbeat / hello / rules 管理）
- UI 状态机（hold / paused / disconnected / popup window）等纯显示行为搬到 `sieve-gui-macos` 仓的 SPEC-002 hips-popup-behavior.md
- daemon 不感知 client 是 GUI / CLI / 其他，只发 `decision.pending` 事件，等 `decision.resolve` RPC 回应

### 2. sieve-ipc crate 内部模块化（不拆 crate）

`sieve-ipc` crate 内部重组目录：

```
crates/sieve-ipc/
├── src/
│   ├── lib.rs               ← re-export 对外 surface
│   ├── protocol/            ← 新增：纯 schema 模块，无 IO，无业务逻辑
│   │   ├── mod.rs
│   │   ├── envelope.rs      ← JSON-RPC envelope（request / response / error / notification）
│   │   ├── decision.rs      ← decision.pending / decision.resolve / decision.canceled
│   │   ├── handshake.rs     ← sieve.hello / sieve.heartbeat
│   │   ├── rules.rs         ← sieve.list_rules / sieve.reload_user_rules / sieve.reload_config
│   │   ├── audit.rs         ← sieve.purge_history 等审计 method
│   │   └── README.md        ← 「wire schema 唯一权威源，对应 SPEC-005；修改字段需先改 SPEC-005」
│   ├── server/              ← 已有的 IPC server 实现挪到这里
│   │   ├── mod.rs
│   │   ├── accept_loop.rs
│   │   └── dispatch.rs
│   ├── client/              ← 新增：sieve-cli / 未来 sieve-tui 共用的 client helper
│   │   ├── mod.rs
│   │   └── connection.rs
│   └── file_ipc/            ← 已有的 pending / decisions 文件 IPC（散在各处的集中到此处）
│       └── mod.rs
```

**模块边界硬约束**：

- `protocol/` 模块**只能 import** `serde / chrono / 标量基础类型`，**禁止 import** `tokio / hyper / fd-lock / 任何 IO crate`
- 在 `protocol/mod.rs` 顶部加注释 + 用 cargo deny 规则（或 unit test 检测 import）确保边界
- 用 `#[deny(unused)]` + 在 `lib.rs` 显式 re-export 把对外 surface 限定死

这个约束使 `protocol/` 在未来升级为独立 crate 时的成本等于 `mv` 一个目录。

### 3. Headless decision path（sieve-cli 子命令扩展）

新增 4 个 CLI 子命令，与 GUI 共用同一组 IPC method，**不引入特权 endpoint**：

```bash
sieve decisions watch [--format jsonl] [--severity SEV]   # 流式订阅 pending decision events
sieve decisions show <id>                                  # 看单个 pending 上下文
sieve decisions resolve <id> --approve [--reason "..."]    # 批准
sieve decisions resolve <id> --block [--reason "..."]      # 拒绝
sieve decisions resolve <id> --warn [--reason "..."]       # 标 warn 放行（用户规则可达）
```

新增 daemon flag 控制「无 client 在线时」行为：

```
sieve start --no-client-policy=auto-block | auto-warn | hold-and-fail-closed
                                            ^
                                            默认值：auto-block（最保守）
```

- `hold-and-fail-closed`：等价于 v1.x 当前行为（hold 流等超时），向后兼容
- `auto-block`：无 client 在线时 fast-fail，直接 block，daemon 向 audit 写明 fallback 原因
- `auto-warn`：无 client 在线时 fast-pass + warn 标记，适合低风险 headless 场景

daemon 触发 fallback 时必须在 doctor 输出里写明：「检测到 GUI 未在线 / CLI 未订阅 → 使用 X 策略」，不静默处理。

**重要约束**：CLI 决策面跟 GUI 共用同一组 IPC 方法（`decision.pending` 订阅 + `decision.resolve` RPC），daemon 不感知谁在响应。GUI 没有任何特权 endpoint。

### 4. 边界声明（不在本 ADR 范围内）

以下内容明确排除，触发条件见背景章节，留作未来 ADR：

- **拆 sieve-protocol 独立 crate / 独立仓**：YAGNI，四个触发条件任一满足时再决策
- **多语言 codegen**（JSON Schema → Swift Codable）：同上，未来 ADR
- **协议版本 bump 到 v3**：本 ADR 不 bump 版本号（术语重命名走兼容 alias，不破坏向后兼容）
- **TUI / webhook bridge 等新 client 实现**：各自独立工程项，本 ADR 只为它们铺好 protocol 模块边界

---

## 影响

### 正面影响

1. **协议中性化**：SPEC-005 不再外漏 GUI 假设，未来加 CLI / TUI / webhook 等新 client 时无需绕过特权术语；
2. **Headless 工作流可行**：远程 SSH / GUI crash / tmux / headless server 部署有明确 fallback 路径，不再 hold 流卡死；
3. **schema 跟实现解耦**：sieve-ipc 内 `protocol/` 模块零 IO 依赖，未来加 client 只需依赖此模块；fuzz / property test 也只需要 schema，不需要 server runtime；
4. **未来拆 crate 成本极低**：方案 A 模块化后，升级到拆独立 crate 就是 `mv` 一个目录的工作量；
5. **GUI 不再特权**：daemon 协议层不感知 client 形态，跟 ADR-026 unix-style 改造方向一致。

### 负面影响

1. **method 名兼容期**：旧 `decision.popup` 名保留一个 minor 版本作 deprecated alias，daemon 要双向 dispatch 旧新名（少量代码 + 一组测试）；
2. **SPEC-005 文档大改**：1750 行的术语清洗是劳动密集活，但纯字符串替换 + 段落重写，不涉及决策；
3. **GUI 仓同步成本**：sieve-gui-macos 仓 ipc-protocol.md（284 行）+ Swift 代码里使用旧 method 名的地方需在 alias 期内迁移；
4. **`--no-client-policy` 默认值的取舍**：默认 `auto-block` 最保守但用户体验差（HIPS 决策收不到响应直接 block），需要 doctor 输出清晰提示，否则 headless 用户难以诊断问题。

### 需要更新的文档

- `docs/specs/SPEC-005-ipc-protocol.md` —— 术语清洗 + §0 文档定位 + method 名重命名 + alias 期声明
- `docs/design/ADR-013-ipc-protocol.md` —— 加段「2026-05-05 ADR-028 在本决策上做术语中性化 + 模块化（不 supersede）」
- `docs/design/architecture.md` —— §IPC 通道说明 + sieve-ipc crate 模块结构图
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目
- `docs/api/api-reference.md` —— §CLI sieve decisions 子命令 + §IPC method 名 alias 期说明
- `crates/sieve-ipc/src/protocol/README.md` —— 新建，明确「修改字段需先改 SPEC-005」
- `sieve-gui-macos` 仓 `docs/api/ipc-protocol.md` —— 同步 alias 期 + 收敛为「指向 SPEC-005 commit hash 的引用文档」
- `CHANGELOG.md` —— v2.x feature + 旧 method 名 deprecated 标记

---

## 相关文档

- PRD v2.0 §6.5 IPC 协议、§6.6 GUI App
- [ADR-013 IPC 协议](./ADR-013-ipc-protocol.md) —— **延伸**，不 supersede；ADR-013 关于 JSON-RPC over UDS + 文件锁的决策仍有效
- [ADR-019 X-Sieve-Origin header](./ADR-019-x-sieve-origin-header.md) —— client 形态无关，本 ADR 不影响
- [ADR-021 三态决策 + 灰名单](./ADR-021-tri-state-decision-and-graylist.md) —— 三态决策语义跟 client 形态无关，本 ADR 不影响
- [ADR-026 Port-based listener routing](./ADR-026-port-based-listener-routing.md) —— 同期推进的 unix-style 改造，listener 与 IPC 协议各管各，独立可交付
- [ADR-027 Network jail enforcement](./ADR-027-network-jail-enforcement.md) —— v3.x，跟 IPC 协议无关
- [SPEC-005 IPC 协议](../specs/SPEC-005-ipc-protocol.md) —— 本 ADR 直接修订对象
- sieve-gui-macos 仓 SPEC-002 hips-popup-behavior —— GUI-only UI 状态机搬迁目的地
