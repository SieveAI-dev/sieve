# GUI 控制面 IPC 方法落地计划（2026-05-02）

> 跟踪：CHANGELOG `[v2.1-gui-control-plane-spec] - 2026-05-02` 的代码侧落地
> 范围：ADR-013 Supplement 2026-05-02 中 11 个新方法 / notification + 6 个错误码 + 7 个新 audit kind
> 工程参考：sieve-gui-macos PRD §6.2 列表 + ADR-013 §S.2-S.7 schema

---

## A. 协议层（sieve-ipc）

- [ ] A1. `crates/sieve-ipc/src/protocol.rs`：8 个 GUI→daemon 方法的 params/result 结构
  - SetPausedRequest / SetPausedResult
  - SetPresetRequest / SetPresetResult
  - SetPresetOverridesRequest / SetPresetOverridesResult（含 RejectedOverride）
  - ReloadConfigRequest / ReloadConfigResult
  - HealthRequest / HealthResult（含 PresetSnapshot / AuditDbSnapshot / RulesSnapshot / GraylistSnapshot / IpcSnapshot 子结构）
  - EvaluateRequest / EvaluateResult（含 EvaluateMatch + EvaluateContentKind）
  - ListGraylistRequest / ListGraylistResult（含 GraylistEntrySummary，去 matched_canonical）
  - RemoveGraylistRequest / RemoveGraylistResult
- [ ] A2. 3 个 daemon→GUI notification 结构
  - PresetChangedNotify
  - PausedChangedNotify
  - RequestDecisionCanceledNotify（含 CancelReason 枚举）
- [ ] A3. `crates/sieve-ipc/src/error.rs`：JSON-RPC error code 常量模块
  - METHOD_NOT_FOUND = -32601（标准）
  - INVALID_PARAMS = -32602（标准）
  - INTERNAL_ERROR = -32603（标准）
  - PROTOCOL_VERSION_MISMATCH = -32000
  - CRITICAL_LOCK_VIOLATED = -32001
  - DAEMON_BUSY = -32002
  - PAYLOAD_TOO_LARGE = -32003
  - UNKNOWN_FINGERPRINT = -32004
  - UNSUPPORTED_IN_PAUSED = -32005

## B. 审计层（sieve-cli/audit）

- [ ] B1. `crates/sieve-cli/src/audit.rs`：新增 7 个 AuditEvent 变体
  - CriticalLockBlocked { rule_id, source }
  - PresetChanged { from_mode, to_mode, source }
  - PresetOverrideApplied { rule_id, timeout_seconds, default_on_timeout, source }
  - PresetOverrideRejected { rule_id, reason, source }
  - PausedSet { until, source }
  - ConfigReloaded { user_rules_errors_count, source }
  - GraylistRemoved { fingerprint, rule_id, removed_by }
- [ ] B2. 7 个 getter 方法（kind/direction/severity/disposition/decision/rule_id/request_id）的 match 分支补全
- [ ] B3. SQLite round-trip 单元测试覆盖每个新变体

## C. graylist 模块（sieve-policy）

- [ ] C1. `crates/sieve-policy/src/graylist.rs`：新增 `list_entries(dir: &Path) -> PolicyResult<Vec<GraylistEntry>>`
  - 按 added_at 倒序
  - 跳过损坏 / fingerprint 不匹配文件（写 WARN）
  - 不递归子目录
- [ ] C2. 单元测试

## D. 服务端路由（sieve-ipc/socket_server）

- [ ] D1. 新增控制面 mpsc channel：`control_tx: mpsc::Sender<ControlPlaneRequest>` + `control_rx`
  - `ControlPlaneRequest` enum 包含 8 个 GUI→daemon 方法 + 每个携带 `oneshot::Sender<Response>` 用于回执
- [ ] D2. `dispatch_message()` 扩展：8 个 method 各加路由分支，反序列化 params → 发送 ControlPlaneRequest → 等待 oneshot → 序列化结果回 GUI
- [ ] D3. 新增 3 个 fan-out broadcast 方法
  - `broadcast_preset_changed(notify)`
  - `broadcast_paused_changed(notify)`
  - `broadcast_request_decision_canceled(notify)`
  - 复用 `broadcast_status_bar` 的 try_send + dead writer lazy 清理模式

## E. 客户端辅助（sieve-ipc/socket_client）

- [ ] E1. 公开测试用辅助函数（生产消费者在 sieve-gui-macos 仓库）
  - `send_set_paused(socket_path, minutes) -> Result<SetPausedResult>`
  - `send_health(socket_path) -> Result<HealthResult>`
  - `send_evaluate(socket_path, req) -> Result<EvaluateResult>`
  - 其余方法的 helper 在测试需要时再加，不强求 100% 覆盖
- [ ] E2. `lib.rs` 再导出新增类型 + helper

## F. daemon 集成（sieve-cli/daemon）

- [ ] F1. 新增 RuntimeState 结构（Arc 共享）
  - `paused_until: ArcSwap<Option<DateTime<Utc>>>`
  - `preset_runtime: ArcSwap<RuntimePreset>`（含 mode + overrides HashMap）
  - 共享给 hot path（accept loop / pipeline filter）
- [ ] F2. 8 个 method 的 handler（每个写对应 audit）
- [ ] F3. `set_preset_overrides` 的 critical_lock 拒绝逻辑（防线二第二出口）
- [ ] F4. evaluate handler：调用现有 LayeredEngine::scan_with_context，不写 audit、不查灰名单、不动 SessionState
- [ ] F5. health handler：从 daemon 内部状态汇聚 snapshot
- [ ] F6. paused 状态在 hot path 的消费（仅影响非 Critical AutoRedact/StatusBar/Ask 路径；Critical 路径不变）
- [ ] F7. `try_remove_graylist` 函数（symmetric to try_write_graylist）+ audit GraylistRemoved
- [ ] F8. `request_decision` 超时分支添加 `broadcast_request_decision_canceled` 调用

## G. 配置（sieve-cli/config）

- [ ] G1. 新增 `evaluate_enabled: bool`（默认 true）
  - 用于 ADR-013 §S.4 evaluate 总开关
- [ ] G2. 文档：sieve.toml 示例补一行（在 data-model.md 同步即可，本任务不动文档）

## H. 集成测试（sieve-cli/tests）

- [ ] H1. `tests/ipc_control_plane.rs` 新建：
  - set_paused 全链路（含 Critical 锁规则在暂停时仍弹窗的断言占位 —— 行为侧的实测在 H6）
  - set_preset 全链路 + preset_changed 广播验证
  - **set_preset_overrides critical_lock 拒绝**（核心：防线二的第二出口测试，对应 ADR-013 §S.7 任务清单第 4 条）
  - reload_config（用户规则失败保留 + 系统规则失败拒绝切换）
  - health 字段完整性
  - evaluate sandbox：投喂 payload 验证不写 audit / 不动状态
  - list_graylist + remove_graylist 配对（含 fingerprint 不存在分支）
- [ ] H2. error code 测试：未知方法返回 -32601；超长 payload 返回 -32003
- [ ] H3. broadcast 单测（已有多 GUI broadcast 测试可复用）

## I. 验收

- [ ] I1. `cargo fmt --all -- --check` 通过
- [ ] I2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 通过
- [ ] I3. `cargo test --workspace --locked` 通过
- [ ] I4. Manual smoke test：起 daemon + nc 连 socket 发 health request，确认响应

---

## 不做的事

- sieve-gui-macos 端的 Swift 实现（独立仓库，本任务只跑通 daemon 侧）
- evaluate sandbox 的"不返回原 payload 片段"约束的全面 fuzz（先写主路径，fuzz 列入 W6 落地任务）
- `request_decision_canceled.reason="upstream_disconnected"` 的真实上游断连感知（需要 hyper layer hook，先留 reason 枚举值，触发逻辑只覆盖 timeout / daemon_shutdown / resolved_by_peer）
- `unsupported_in_paused` 错误码当前 enum 占位，无具体方法触发（保留供将来扩展）
- 多 issue 合并 partial allow 的 per_issue audit 拆分（已在 v2.0 deferred-2 落地，本任务不重做）
