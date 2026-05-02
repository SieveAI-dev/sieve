# SPEC-005 Code Review

## 总评
方向是对的：SPEC-005 把方法名前缀、枚举大小写、错误码段位、安全防线集中到一个权威源，这是必须做的。但当前 Draft **不建议 merge 进入代码改造阶段**。主要问题不是旧代码没跟上，而是 SPEC 自身还有几处会导致实现歧义或多 GUI 场景错误同步的阻断点。

## P0 阻断（必须修复才能进入代码改造阶段）

### 1. 帧大小上限缺少可执行的安全接收算法
- SPEC 文件位置：`§1.3`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:49-55`
- 问题描述：SPEC 只写“单条消息 1 MiB，超限关闭连接 + audit”，但没有规定接收方必须在累积到 1 MiB 前终止读取。当前 daemon 用 `BufReader::lines()`，如果攻击者持续发送无 `\n` 大帧，会无界缓存；同时还会 debug 记录 raw line。
- 修改建议：明确 MUST 使用 bounded frame reader：累计字节数超过 1 MiB 立即关闭；不得等待换行；不得记录原始 payload；audit 只写 peer / size / reason / timestamp。

### 2. `set_preset_overrides` 的 critical_lock 返回语义和错误码自相矛盾
- SPEC 文件位置：`§9.3`、`§12.3`、`§12.5`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:672-698`, `1081-1083`, `1100-1108`
- 问题描述：`§9.3` 规定 critical_lock override “不应整体失败”，通过 `rejected[]` 部分报告；但 `§12.3` 又定义 `-32001 critical_lock_violated` 用于同一路径，`§12.5` 示例也是 error response。实现者无法判断是返回 result 还是 error。
- 修改建议：二选一。建议保留 `§9.3` 的 partial success 语义：critical_lock 只进 `rejected[]` 并写 audit，`-32001` 标为 reserved / 不用于 `set_preset_overrides`，或删除该业务码示例。

### 3. fan-out 的 `source` 防回声机制不能支持多 GUI
- SPEC 文件位置：`§10.1`、`§10.2`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:997-1004`, `1021-1026`
- 问题描述：`preset_changed.source == "gui"` 只能说明来源类型，不能说明是哪一个 GUI。多 GUI 场景下，A 修改 preset 后，B 也会收到 `source:"gui"`，按 SPEC 会误判为“本 GUI 自身回声”而忽略。`paused_changed` 甚至没有 `source`。
- 修改建议：给所有由 GUI 请求触发的 fan-out 加 `origin_request_id` 或 `origin_client_id`。GUI 只忽略自己 inflight request id 对应的回声，其他 GUI 必须同步。`paused_changed` 应补同样字段。

## P1 应改（强烈建议修复）

### 1. optional / default / null 语义不清
- SPEC 文件位置：`§6.1.1`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:255-273`
- 问题描述：表格里多处写“必需=是”但又给默认值，如 `origin_chain`、`source_channel`、`explicit_chain_depth`。这和 serde 的 `#[serde(default)]` 行为不等价，尤其是字段缺失与显式 `null` 对 `Vec` 不同。
- 修改建议：每个字段拆成三列：`required`、`default if absent`、`null accepted`，并标明 Rust/Swift 解析规则。

### 2. `request_decision` v2 DTO 和现有共享 `DecisionRequest` 缺少迁移边界
- SPEC 文件位置：`§6.1`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:209-355`
- 问题描述：SPEC 定义的是 GUI wire DTO，但 daemon 当前 `DecisionRequest` 还同时服务 GUI socket 与 sieve-hook pending file。SPEC 又明确不描述 hook 文件 IPC。若不写清“内部结构”和“GUI v2 wire DTO”分离，后续 PR 容易破坏 hook。
- 修改建议：新增“daemon 内部 DTO → GUI v2 params 映射”小节，明确 `created_at` 映射为 `received_at_daemon`，`detections[]` 被展开为 single/merged issue，hook pending schema 不受本 SPEC 约束。

### 3. `allow_remember` 三道防线缺少 UI 禁用的规范性措辞
- SPEC 文件位置：`§6.1.1`、`§6.2.1`、`§6.2.2`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:264-265`, `415-416`, `450-452`
- 问题描述：daemon 计算、GUI 编码层强制、daemon 二次校验都有写到；但“GUI UI 必须禁用 Remember checkbox”没有作为 MUST 写清楚。
- 修改建议：补一条可测试要求：`allow_remember=false` 时 GUI MUST 禁用且灰显 Remember 控件，提交层 MUST 强制 `remember=false`，daemon MUST 对单 issue / per_issue 再校验。

### 4. `context_hint` 200 字符限制的计数单位和处置不够可测
- SPEC 文件位置：`§1.3`、`§6.2`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:55`, `416`, `446`
- 问题描述：`§1.3` 写 UTF-8 codepoint，Swift 当前 `prefix(200)` 是 `Character` 语义，不等价；daemon 截断 + audit warn，但响应 schema 只写 ≤200。
- 修改建议：统一为 Unicode scalar 或 grapheme cluster，并写明 GUI 超限是拒绝提交还是截断；daemon 对超限是截断还是 `-32602` 也应固定一种。

### 5. `remove_graylist` 同时允许 result false 和 error，协议不确定
- SPEC 文件位置：`§9.8`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:971-976`
- 问题描述：`removed=false` 与 `-32004 unknown_fingerprint` “二选一；推荐 error”会导致双端测试 fixture 无法唯一化。
- 修改建议：直接规定 fingerprint 不存在 MUST 返回 `-32004` error；`removed=false` 删除或仅保留给未来明确的幂等模式。

### 6. 时间戳和 UUID 规范不完整
- SPEC 文件位置：多处，例如 `sieve/docs/specs/SPEC-005-ipc-protocol.md:267`, `413`, `527`, `936-940`
- 问题描述：ISO 8601 与 unix milliseconds 混用可以接受，但没有说明选择理由和精确格式；UUID 只写 String(UUID)，未规定 hyphenated lowercase。
- 修改建议：新增全局“标量格式”章节：ISO 时间使用 RFC 3339 UTC `Z`，允许/要求毫秒；灰名单时间若保留 unix ms，说明原因；UUID MUST 为 lowercase hyphenated UUID string。

### 7. 协议升级与 fixture 同步流程不可操作
- SPEC 文件位置：`§13.3`、`§14.2-14.3`，`sieve/docs/specs/SPEC-005-ipc-protocol.md:1137-1148`, `1161-1169`
- 问题描述：写“daemon 先 merge，再 GUI”会造成一段时间 GUI v1 连不上 daemon v2；fixture 同步写“git submodule 或 release artifact”，没有定案。
- 修改建议：定义发布顺序而不是 merge 顺序；引入兼容窗口或 feature flag。fixture 同步必须选一种：建议 daemon 发布带 commit pin 的 fixture artifact，GUI CI 按 pinned commit 拉取。

## P2 建议（行文 / 文档结构 / 长期改进）

### 1. 章节引用错误
- SPEC 文件位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:123`, `647`
- 问题描述：`preset` 引用 `§6.6`，实际应为 `§5.6`；`set_preset` 成功后引用 `§11.1`，实际应为 `§10.1`。
- 修改建议：修正引用，避免后续 review 跳错章节。

### 2. `§11` 标题叫“方法名清单”，但包含非 method 的 response
- SPEC 文件位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:1030-1038`
- 问题描述：`sieve.decision_response` 不是 JSON-RPC method，`§6.2` 也说无独立 method。
- 修改建议：把 `§11` 改名为“消息清单”，或把 response 单独列为 `JSON-RPC response to sieve.request_decision`。

### 3. “最严格”未定义排序
- SPEC 文件位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:295`
- 问题描述：`default_on_timeout` 多 issue 合并时写“否则取最严格”，但没有定义 `block/redact/allow` 的严格顺序。
- 修改建议：明确 `block > redact > allow`。

## 与代码侧已发现的差异

### daemon / Rust
- `sieve/crates/sieve-ipc/src/socket_server.rs:545-549` 仍发送 method `"request_decision"`，SPEC 要求 `sieve.request_decision`。
- `sieve/crates/sieve-ipc/src/protocol.rs:179-239` 当前 `DecisionRequest` 是 `created_at + detections[] + DetectionPayload` 形态；SPEC `§6.1` 是 flattened single/merged issue 形态。
- `sieve/crates/sieve-ipc/src/socket_server.rs:624-627` 没有 accept 后首条 `sieve.hello`；也没有 `sieve.heartbeat` timer。
- `sieve/crates/sieve-ipc/src/socket_server.rs:627`, `640-645` 使用 unbounded `lines()` 并记录 raw line，和 `§1.3` 的安全上限不匹配。
- `sieve/crates/sieve-ipc/src/socket_server.rs:760-767` 收到 GUI error response 只 log 后返回；SPEC 要求立即按 `default_on_timeout` 处置并写 audit。
- `sieve/crates/sieve-ipc/src/protocol.rs:37-48` Rust `NotifyKind` 没有 SPEC `§5.9` 的 `hook_terminal`。
- `sieve/crates/sieve-cli/src/daemon_control_plane.rs:283-285` 仍接受 `default` preset；SPEC 改为 `standard`。
- `sieve/crates/sieve-cli/src/daemon_control_plane.rs:327-330`, `379` 当前空 overrides 被拒、timeout 允许 5-600；SPEC 写空对象清空、范围 30-120。
- `sieve/crates/sieve-cli/src/daemon_control_plane.rs:113-118` `applies_to` 返回 `AutoRedact/StatusBar/Ask:non_critical`，SPEC 使用 snake_case disposition。
- `sieve/crates/sieve-cli/src/daemon_control_plane.rs:585-627` `evaluate` 对非 critical 返回 `severity:"unknown"`、`disposition:status_bar`，不满足 SPEC 的 enum 约束。

### GUI / Swift
- `sieve-gui-macos/Sources/Services/IPC/IPCClient.swift:26` 仍只支持 `["v1"]`，SPEC 是 `v2`。
- `sieve-gui-macos/Sources/Models/Enums.swift:24-35`, `45-50` 仍用 `GuiPopup/Block/Strict` 等旧 raw value，SPEC 要求 snake_case lowercase。
- `sieve-gui-macos/Sources/Models/DecisionResponse.swift:29-35`, `95-111` 发送 `responded_at`，缺 `request_id/decided_at/by_user`；`DecisionError` 仍用 `-32000~-32002`。
- `sieve-gui-macos/Sources/Services/IPC/IPCRouter.swift:102-110` 仍监听 `sieve.event_notify`，preset_changed 也按 `preset/changed_by/occurred_at` 解码；SPEC 是 `notify_status_bar` 与 `mode/overrides/changed_at/source`。
- `sieve-gui-macos/Sources/Models/IPCResponses.swift:5-88` 控制面响应 DTO 还是旧 schema，和 SPEC `set_paused/reload_config/health/evaluate` 都不对齐。
- `sieve-gui-macos/Sources/Models/HitSummary.swift:83-127` 灰名单模型期待 `created_at` ISO、`trigger_count`；SPEC 是 `added_at` unix ms、`match_count_since`、`expires_at`。
- `sieve-gui-macos/Sources/Features/Debug/DebugWindowView.swift:148-156` GUI 仍发送 `source_agent:"claude-code"`；SPEC 要求 `claude`。
- `sieve-gui-macos/Sources/Services/IPC/IPCClient.swift:281-287` 重连后会重发所有 inflight；SPEC 未规定 daemon 重启后旧 `request_decision` / 旧 response id 的丢弃策略。

## 结论
不建议当前 Draft merge。建议先修完 P0，再做一次 focused review；P1 中至少 optional/null、时间戳/UUID、fixture 同步流程也应在进入代码 PR 前定稿。未运行测试，本次是静态协议 review。  

