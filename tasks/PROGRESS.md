# Sieve daemon · 进度

> 上次更新：2026-05-02
> 当前阶段：**v2.0 协议代码改造**（SPEC-005 v2 r5 冻结 → 拉齐 sieve-ipc 实现）
> 唯一进度真实源——任何任务完成必须更新本文件。

## 当前阶段一句话

`docs/specs/SPEC-005-ipc-protocol.md`（v2，r5 冻结）已与现有 `crates/sieve-ipc/` 实现做完 gap 分析，整体差距大（P0 × 5 / P1 × 10 / P2 × 6）。下一步按 P0 → P1 → P2 顺序改造，每完成一项必须勾选 + 更新本文件。

---

## ✅ 已完成（按时间倒序）

- **2026-05-02** SPEC-005 v2 协议 r5 冻结 review 通过（`docs/review/2026-05-02-codex-spec-005-review-r5.md`），可进入代码改造
- **2026-05-02** SPEC-005 v2 vs 代码 gap 分析完成（_gap-spec005-vs-code.md，已并入下方"下一步"清单后将删除）
- **2026-05-02** tasks/ 与 docs/review/ 文档大扫除：12 份过期 todo/status/report + 17 份历史 codex review 全部归档到 `_archive/`，建立 PROGRESS.md 单一进度真实源
- **2026-05-01** v2.0 + v2.1 代码 100% 落地（sieve-policy / 三态决策 ask + 灰名单 + Critical 锁 / LayeredEngine / 进程上下文反查 / audit schema migration）
- **Week 5** Phase A 全部完成（参见 `_archive/v2.0-phase-a-plan.md`）

---

## 🚧 进行中

_无。等用户选定下一步执行哪一组 P0 后填入。建议每次最多 1 项进行中，避免主上下文压力。_

---

## ⏭ 下一步（SPEC-005 v2 代码对齐，按优先级）

### P0 阻塞合规（必须先完成，否则与 v2 GUI 无法互操作）

- [ ] **[P0-1]** 帧读取替换无界 `BufReader::lines()` → `read_buf` + 手动 `memchr`（§1.3.1）
  - 文件：`crates/sieve-ipc/src/socket_server.rs:8,627` + `socket_client.rs:52`
  - 补充：单帧 > 1 MiB 关连接 + remainder > 1 MiB 关 + audit `ipc_oversize_frame` + 解析失败不关连接
- [ ] **[P0-2]** 实现 `sieve.hello` 握手通知（§3）
  - 新增 `HelloParams` struct（`protocol_version="v2"` / `daemon_version` / `paused` / `preset` / `uptime_seconds` / `audit_db_user_version` / `daemon_boot_id`）
  - 在 `handle_connection` 起始处作为第一条出站消息发送
- [ ] **[P0-3]** 实现 `sieve.heartbeat` 25 秒心跳（§4）
  - `handle_connection` 写方向加 `tokio::time::interval(25s)`，任何出站帧重置定时器
- [ ] **[P0-4]** `request_decision` 方法名补 `sieve.` 前缀（§11）
  - `socket_server.rs:546` 改 `"sieve.request_decision"`，与 GUI P0 同步
- [ ] **[P0-5]** Socket 文件权限设 `0600`（§1.1）
  - `IpcServer::bind` 后 `set_permissions(0o600)`；`ensure_dirs` 把 sieve_home 设 `0700`

### P1 字段/行为偏差

- [ ] **[P1-1]** `SetPausedResult.until` → `paused_until`（§9.1, §10.2）— 含 `PausedChangedNotify`
- [ ] **[P1-2]** `PresetChangedNotify` + `PausedChangedNotify` 加 `origin_request_id: Option<Uuid>`（§10.0–10.2）
- [ ] **[P1-3]** `HealthResult.paused` 拆为 `paused: bool` + 独立 `paused_until: Option<DateTime<Utc>>`（§9.5）
- [ ] **[P1-4]** `DecisionResponse` 加 `ui_phase_when_clicked: Option<UiPhase>`（§6.2.1, §5.10）
- [ ] **[P1-5]** `sieve.request_decision` 拆 wire DTO（§6.0, §6.1）— 字段展开 + `merged: true` + `received_at_daemon`；这是改造工作量最大的一项
- [ ] **[P1-6]** `protocol_version` 字符串全部 `"v1"` → `"v2"`（含 `tests/control_plane_dispatch.rs:52,142`）
- [ ] **[P1-7]** `NotifyKind` 加 `HookTerminal` 变体（§5.9）
- [ ] **[P1-8]** JSON 解析失败返回 `-32700 parse_error` 而非静默 return（§1.3.1, §12.2）— 加 `PARSE_ERROR` 常量
- [ ] **[P1-9]** `sieve.set_paused` 响应前强制 fan-out（§10.0.1）— 改 `ControlPlaneRequest` 回执结构，让 `forward_reply` 在写 result 前先 broadcast
- [ ] **[P1-10]** fan-out 写入加 2 秒 bounded write timeout（§10.0.1）— EPIPE/ECONNRESET/EBADF 视为失联

### P2 风格 / 可读性

- [ ] **[P2-1]** `*_count` 字段类型 `usize` → `u32`（§9.4 等）
- [ ] **[P2-2]** P1-5 wire DTO 拆分时把 `created_at` 命名为 `received_at_daemon`
- [ ] **[P2-3]** Timestamp 序列化保证 `Z` 后缀 + 毫秒精度（§4A）
- [ ] **[P2-4]** 多 issue 合并形式（`merged: true` + `issues[]`）实现（§6.1.2, §6.2.2）
- [ ] **[P2-5]** 建立 `tests/fixtures/v2/` + `tests/schema_v2_fixtures.rs`（17 method × 3 = 51 条最低门槛，§14.1）
- [ ] **[P2-6]** `EvaluateRequest.source_agent` 改 `SourceAgent` enum（§5.7）— 测试中废弃 `"claude-code"`

### 双侧契约同步点（必须与 sieve-gui-macos 仓库 PROGRESS 同步推进）

- 协议版本号：daemon P1-6 ↔ GUI P0-1
- `request_decision` 方法名前缀：daemon P0-4 ↔ GUI 接收侧
- `decision_response.result` required 字段（`request_id` / `decided_at` / `by_user` / `ui_phase_when_clicked`）：daemon P1-4 ↔ GUI P0-3 / P1-1
- 错误码段位 `-32100/-32101/-32102`：daemon 端 ↔ GUI P0-2
- `Disposition` / `DefaultOnTimeout` snake_case：daemon 已对齐 ↔ GUI P1-2/P1-3 待改
- `NotifyKind` 六枚举值：daemon P1-7 ↔ GUI P1-4

### Phase 2 / 长期

详见 `roadmap.md`，本文件不重复。

---

## 🚫 阻塞 / 等决策

- **P1-9 串行化实现方式**：是改 `ControlPlaneRequest::SetPaused` reply 结构让 `forward_reply` 携带 fan-out 指令，还是把 `broadcast_*` 调用直接挪进 IPC server 层？两者各影响一层，需要在动手前定一次。

---

## 完成定义（DoD，每项任务通用）

- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 全过
- 涉及 SSE / 规则 / 工具调用判定的改动 → 对应 fuzz / 单元测试已加
- PRD §9 十六条硬约束未被绕过
- CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- 关联文档（requirements / design / api / SPEC）已同步
- **本文件已勾选 + 移项至「已完成」**

详见 `.cursorrules §五` + 项目根 `CLAUDE.md`。
