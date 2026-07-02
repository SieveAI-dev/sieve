# SPEC-005: Sieve daemon IPC 协议

> Version: v2.0 — 2026-05-02
> Status: **Frozen**（IPC 协议 v2 wire schema 唯一权威源）
> 取代：daemon 仓库 [`docs/api/api-reference.md §6`](../api/api-reference.md)（旧版 v1.x 描述）+ GUI 仓库 `docs/api/ipc-protocol.md` v1.0 中所有 schema 定义

---

## 协议变更日志

### 2026-05-05 多 listener（v2.x，向后兼容扩展）

- §9.5 health 响应新增 `listeners[]` 数组，每项含 `provider_id` / `protocol`
- 旧 `listen` 字段保留为 `listeners[0]` 别名，标注 deprecated
- 协议版本号不 bump（v2 内向后兼容扩展，client 用 `decodeIfPresent` 兜底）

### 2026-05-05 协议中性化（v2.x，向后兼容）

- 段落术语清洗：「GUI 端」→「client 端」；「弹窗」→「decision request / decision event」（视上下文）
- UI 状态机段落加 admonition 标注「这是 GUI 实现细节，不是协议契约」
- wire 字段名（`gui_popup` disposition 枚举值）**保持不变**（向后兼容硬要求），加注释说明历史语义
- 协议版本号**不 bump**（仍是 v2，向后兼容扩展）

---

## 0. 文档定位与权威性

**SPEC-005 是 daemon (`sieve` 仓库) IPC 协议的唯一权威源。**

本协议描述 daemon 与任意 client（macOS GUI `sieve-gui-macos`、CLI、TUI、webhook bridge 等）之间的语义契约，包括 decision 协议、heartbeat、握手、规则管理等。**daemon 协议层不感知 client 形态**——GUI、CLI、TUI 或其他 headless client 在协议层地位平等，共享同一套 method/notification。

- 任何方法名、字段名、枚举值、错误码、行为约定的变更，**必须先改本文件**，再分别在各仓库落代码
- daemon 仓库 [`docs/api/api-reference.md §6`](../api/api-reference.md) 收敛为本 SPEC 的索引，不再保留 schema 表
- GUI 仓库 `docs/api/ipc-protocol.md` 收敛为"client 实现注解"，仅描述 client 端解析容错、UI 状态机映射、超时降级等本地行为
- GUI 仓库 `docs/external/upstream-references.md` 必须显式引用 SPEC-005 的 commit hash（pin 版本）

**本 SPEC 不描述**：
- HTTP 反向代理协议（见 [api-reference.md §1](../api/api-reference.md#1-反向代理端点对-claude-code)）
- sieve-hook 文件 IPC（见 [SPEC-001](SPEC-001-sieve-hook-protocol.md)）
- audit.db schema（见 [data-model.md](../design/data-model.md)）
- HIPS decision request UI 行为（见 [SPEC-002](SPEC-002-hips-popup-behavior.md)）

---

## 1. 传输层

### 1.1 Socket 路径与权限

| 项 | 值 |
|---|---|
| Socket 路径 | `${SIEVE_HOME:-~/.sieve}/ipc.sock` |
| Socket 类型 | Unix Domain Socket（`SOCK_STREAM`） |
| 文件权限 | `0600`（仅 owner 可读写） |
| 父目录权限 | `0700` |
| 绑定时机 | daemon 启动期，bind 前 `unlink` 旧 socket |

client 连接前必须校验 socket 文件权限与所有者；不符合时拒连并提示用户检查 `~/.sieve/` 安装状态。

### 1.2 帧格式

- **每条消息一行 JSON**，以单字节 `\n`（0x0A）终止（newline-delimited JSON, ndjson）
- JSON 内部禁止出现 `\n`（serializer 必须 escape；`serde_json::to_string` 默认行为已满足）
- 编码：UTF-8，无 BOM
- **不支持 JSON-RPC batch**（数组形式的请求列表）

### 1.3 大小上限

| 项 | 值 | 超限行为 |
|---|---|---|
| 单条消息大小 | 1 MiB（含尾部 `\n`） | 接收方 **MUST** 在以下任一条件成立时关闭连接：(a) 解析出的完整 frame 长度（`idx + 1`）超过 1 MiB；(b) 无 newline 的 partial frame 缓冲区自身已超过 1 MiB（疑似无界大帧攻击）。详见 §1.3.1 接收算法。audit 写 `kind=ipc_oversize_frame` 仅含 `peer / size_bytes / closed_at_ms`，**禁止**记录任何 raw payload |
| `sieve.evaluate` 的 `payload` 字段 | 64 KiB | daemon 返 `-32003 payload_too_large` |
| `context_hint` 字段 | 200 个 Unicode scalar（Rust `str::chars().count()` / Swift `String.unicodeScalars.count`） | client **MUST** 在提交层校验并 **拒绝**用户超限提交（不静默截断）；daemon 收到超限值 **MUST** 返 `-32602 invalid_params`，**不**做隐式截断 |

#### 1.3.1 帧接收算法（规范性，双方实现 MUST 遵守）

接收方 **不得** 使用无界 `BufReader::lines()` 或等价 API。Unix stream 不保证一次 read 只含一帧，也不保证一帧不被切到多次 read 中，必须按下面算法保留 remainder 循环扫描：

```
frame_buf = bytes()                       # 跨多次 read 共享
loop:
    chunk = read_syscall(socket, max=64KiB)
    if chunk.is_empty():
        close_connection(reason="eof")
        return                            # EOF 后必须显式退出
    frame_buf.append(chunk)

    # 1) 先循环消费所有完整帧（每帧独立判 oversize，不看 buffer 总长度）
    while True:
        idx = memchr(frame_buf, b'\n')
        if idx is None:
            break                          # 没有完整帧，进入步骤 2
        frame_len = idx + 1                # 含尾部 \n
        if frame_len > 1 MiB:
            close_connection(reason="oversize_frame")
            write_audit(kind="ipc_oversize_frame",
                        fields={peer, size_bytes=frame_len, closed_at_ms})
            return                         # 禁止记录 frame_buf 内容
        frame = frame_buf[..=idx]
        frame_buf = frame_buf[idx+1..]     # 保留 remainder（关键）
        parse_and_dispatch(frame)

    # 2) 检查残留的 partial frame：只有当 remainder 自身已超 1 MiB
    #    且仍没有 newline 时才视为攻击（无界大帧）
    if frame_buf.len() > 1 MiB:
        close_connection(reason="oversize_frame")
        write_audit(kind="ipc_oversize_frame",
                    fields={peer, size_bytes=frame_buf.len(), closed_at_ms})
        return
```

**MUST 约束**：

1. **不得**清空 `frame_buf` —— 必须只切走 `[..=idx]` 部分，保留 `[idx+1..]` 作为下一帧的开端
2. **MUST** 按上述顺序：先消费所有完整帧（按单帧判 oversize），再判 remainder 是否超限。**禁止**在 append 后直接按 `frame_buf.len() > 1 MiB` 关闭连接 —— 这会误杀"接近 1 MiB 的合法帧 + 下一帧前缀"组成的合法粘包
3. **MUST** 在循环内重复扫描 `\n`，不假设一次 read 只含一帧（多帧粘包是常态）；剩余无 newline 的字节是 partial frame，等下一次 read 补齐
4. **禁止**在 debug / trace 日志中记录 `frame_buf` / `frame` / 解析失败的 raw line；允许记录 `len / first 16 bytes 的 sha256 前缀 hex` 等不可重建原文的元信息
5. parse_and_dispatch 失败（JSON 解析错、schema 校验错等）时 **MUST** 仅记录元信息 + 返回 -32700/-32602 错误响应，**不**关闭连接（否则一条恶意消息能 DoS 整个连接）；但 oversize 必须关闭

实现侧推荐：
- Rust：`tokio::io::AsyncReadExt::read_buf` + 手动循环扫描 `memchr::memchr(b'\n', &frame_buf)`；不要 `tokio::io::AsyncBufReadExt::lines()`
- Swift：`NWConnection.receive(minimumIncompleteLength: 1, maximumLength: 65536)` + 累积 `Data` 缓冲，每次 receive 完成回调里循环扫描换行；解析时再 `String(data:encoding:.utf8)`

### 1.4 通信模型

- **JSON-RPC 2.0** 双向：daemon 与 client 双方都可发起 request（含 `id`）和 notification（无 `id`）
- daemon 同一时间可有多个 client 连接（多窗口 / 多用户场景），所有广播类 notification 走 fan-out
- `request_decision` 仅发送给 `client_writers[0]`（最早连接的 client）；当该 client 答复或断开后再切到下一个

---

## 2. 协议版本协商

### 2.1 当前版本

| 字段 | 值 |
|---|---|
| `protocol_version` | `"v2"` |
| 引入 | 2026-05-02（本 SPEC） |
| 取代 | `v1`（GUI 文档 v1.0 + daemon v1.5 实现，schema 错位严重；v2 起协议转为 client-agnostic） |

### 2.2 协商时序

1. client 连接 socket
2. daemon **立即** 主动发 `sieve.hello` notification（见 §3）
3. client 检查 `params.protocol_version`：
   - 在 client 白名单内（当前白名单 `["v2"]`）→ 进入 connected 状态
   - 不在白名单 → 关闭连接，UI 引导升级；client 端不向 daemon 回任何消息
4. daemon 不识别 client 后续请求中的方法名 → 返 `-32601 method_not_found`（保留向前兼容性）

### 2.3 版本号语义

- `vN`（无小版本）— **major 版本，强不兼容**：删字段、改字段语义、改方法名、改枚举集合、改握手时序
- 向前兼容的字段新增（client 必须忽略未知字段）—— 不递增 `protocol_version`；在 SPEC 中以 "Since v2.x" 标注
- 错误码新增（client 必须有 fallback "未知错误" 文案）—— 不递增

---

## 3. 握手：`sieve.hello`

### 3.1 方向与时机

- **方向**：daemon → client
- **类型**：notification（无 `id` 字段）
- **时机**：daemon accept 连接后，**第一条**写入此 socket 的消息必须是 `sieve.hello`
- **重发**：daemon 不主动重发 hello；client 重连建立新 socket 时再发一次

### 3.2 Params Schema

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.hello",
  "params": {
    "protocol_version": "v2",
    "daemon_version": "0.7.2",
    "paused": false,
    "paused_until": null,
    "preset": "standard",
    "uptime_seconds": 14523,
    "audit_db_user_version": 2,
    "daemon_boot_id": "5b1f8c92-3a6d-4e8f-b0a1-7c4d9e2f1a83"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `protocol_version` | String | yes | — | no | 当前固定 `"v2"` |
| `daemon_version` | String | yes | — | no | semver，如 `"0.7.2"`、`"1.0.0-rc1"` |
| `paused` | bool | yes | — | no | daemon 当前是否处于暂停态 |
| `paused_until` | `Timestamp` | no | `null` | yes | 暂停截止时间（UTC，§4A）；`paused=false` 时为 `null`，与 `paused` 配对。client 握手即可正确进入 paused 态并显示「恢复至…」，无需等首条 `sieve.paused_changed` 补齐。v2.x 向后兼容扩展（字段新增，旧 client 忽略未知字段），**不** bump `protocol_version` |
| `preset` | enum (§5.6) | yes | — | no | preset mode 枚举值 |
| `uptime_seconds` | u64 | yes | — | no | daemon 进程启动至今秒数 |
| `audit_db_user_version` | u32 | yes | — | no | `PRAGMA user_version` 当前值（v2.0 起为 `2`） |
| `daemon_boot_id` | `Uuid` | yes | — | no | 每次 daemon 进程启动时新生成；client 用此值判断是否经历过 daemon 重启（见 §3.4） |

### 3.3 client 端期望行为

> **ℹ️ Note**: 以下行为属于 sieve-gui-macos 仓的 GUI 实现细节，
> 不是 daemon IPC 协议契约。新 client（CLI / TUI / webhook）实现时无需对齐这些 UI 显示行为。
> 详细 GUI 显示规范见 sieve-gui-macos 仓 [SPEC-002 hips-popup-behavior.md](../../../sieve-gui-macos/docs/specs/SPEC-002-hips-popup-window.md)。

1. 校验 `protocol_version`，不匹配则关闭连接（不允许字段嗅探兼容）；同时 GUI client **MUST** 进入"协议版本不匹配"UI 状态：
   - 显示模态或显著横幅，文案 **MUST** 至少包含两个事实：(a) "Sieve daemon 与 client 协议版本不兼容"；(b) "需要升级 client 至 v1.0+ 或将 daemon 降级到兼容版本"
   - 文案 **MUST** 提供至少一个可点击操作：打开 GitHub Releases 链接 / 打开"关于"窗口 / 打开"设置 → 检查更新"入口（具体三选一由 client 实现决定）
   - 此状态下 **MUST** 禁用所有 client → daemon 写入操作（暂停 / preset / 灰名单等），仅允许只读浏览本地缓存
2. 缓存 `daemon_version` 到本地用户设置（client 重启后展示"上次连接到 daemon vX.Y.Z"）
3. 缓存 `daemon_boot_id` 到本地（供 §3.4 重连判定）
4. 同步 `paused` / `preset` 到 AppState
5. 标记 connected，开始消费后续消息
6. **清空跨连接 inflight 状态**：见 §3.4

### 3.4 重连后的 inflight 处理（规范性）

client 重连（含 daemon 重启 / 网络中断 / 主动断开重连等所有路径）建立新 socket 后，**MUST**：

1. **丢弃**所有跨上次连接的 `request_decision` inflight 状态（pendingQueue + activeRequest）。理由：daemon 重启后旧 oneshot channel 已失效；daemon 会重新计算所有 inflight 检测的 `default_on_timeout` 兜底。即便是同一个 daemon 进程（仅网络中断后重连），跨连接的 `request_id` 也已经过 fail-closed 处置，client 端再持有旧 id 无意义
2. **不**重发上次连接 inflight 的 `decision_response`。daemon 不会接受跨连接的 response，且重发会污染 audit
3. **UI 文案选择**（以下为 GUI client 参考实现，非协议约束；headless client 可忽略）：根据 `sieve.hello.daemon_boot_id` 与本地缓存的 `last_seen_daemon_boot_id` 比较：
   - **首次连接**（client 进程内此前从未有过 connected state，且本地无 cached boot_id）→ **不弹任何 toast**，仅初始化 `last_seen_daemon_boot_id` 后进入正常 connected 流程
   - boot_id 与本地缓存不一致（且非首次连接）→ daemon 进程重启了 → toast：「daemon 已重启，本次决策已由系统兜底」
   - boot_id 与本地缓存一致 → daemon 还是同一个进程，仅连接中断 → toast：「连接已恢复，本次决策已由系统兜底」
4. **更新缓存**：握手处理完毕后立即把 `daemon_boot_id` 写入本地，供下次重连判定使用
5. **保留**：本地 AppState 中的非 inflight 数据（preset / paused / 设置面板缓存等），等 `sieve.hello` 来后用 hello.params 覆盖

> **daemon 端对应**：
> - daemon 启动时生成新的 `daemon_boot_id`（UUIDv7，含时间戳便于日志关联）；进程生命周期内不变
> - 重启时 oneshot channel 自然失效；client 重连后第一条消息**必须**是新 `sieve.hello`
> - daemon **禁止**主动重发上次未答复的 `request_decision`（旧检测的 SSE 流早已 fail-closed 关闭，重发无意义）

---

## 4. 心跳：`sieve.heartbeat`

### 4.1 行为

- **方向**：daemon → client
- **类型**：notification（无 `id`，无 `params`）
- **周期**：daemon 在没有其他出站消息的 25 秒内必须发一条 heartbeat；任何其他出站消息（含通知与请求）都重置 heartbeat 计时器
- **client 端超时**：30 秒内（含 heartbeat 与任何业务消息）未收到任何来自 daemon 的字节 → 视为失联，关闭并按指数退避重连

### 4.2 Schema

```json
{ "jsonrpc": "2.0", "method": "sieve.heartbeat" }
```

`params` 字段缺省（不发空对象），client 解析时不读取 params。

---

## 4A. 标量格式约定（全局）

本节定义所有方法 / schema 中出现的标量字段的 wire 格式。任何字段表只要标注类型为下表中名字之一，即继承此处规则，不再单独说明。

**全局硬规则（自 v2.0-r2 起）**：所有 wire schema 字段表 **MUST** 使用本节定义的命名类型（`Timestamp` / `UnixMs` / `Uuid` / `JsonRpcId` / 各 enum 类型名），**禁止**再写裸 `String (ISO 8601)` / `String (UUID)` / `i64 (unix ms)` 等局部描述。Spec review / fixture 测试将检查此规则。

| 类型名 | wire 格式 | 备注 |
|---|---|---|
| `Timestamp` | RFC 3339 / ISO 8601 UTC，**MUST** 以 `Z` 后缀（不允许 `+00:00`）；**SHOULD** 含毫秒精度（如 `"2026-05-02T15:03:11.234Z"`）；秒级精度（无毫秒小数）允许，但接收方 **MUST** 接受 0~9 位小数 | 用于所有时间戳字段：`decided_at` / `created_at` / `applied_at` / `reloaded_at` / `started_at` / `received_at_daemon` / `evaluated_at` / `last_reload` / `changed_at` / `paused_until` / `generated_at` / `OriginHop.timestamp` 等 |
| `UnixMs` | i64，unix epoch 毫秒（UTC） | 仅用于灰名单条目的 `added_at` / `expires_at`；选用 unix ms 是为了 audit.db / 灰名单存储跨语言数值排序方便，与 wire 上的 `Timestamp` 字段不混用 |
| `Uuid` | RFC 4122 标准格式：lowercase hyphenated 36 字符（如 `"8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02"`） | 大小写敏感，**禁止** 大写或无连字符变体；**示例 JSON 中 MUST 使用完整合法 UUID**，不允许 `"8f3a..."` 之类占位（避免 fixture 作者误抄） |
| `JsonRpcId` | **String（`Uuid` 格式）** | JSON-RPC 2.0 本身允许 String 或 Number，但**本协议强制 String**：所有 request 的 `id` **MUST** 为 `Uuid` 格式 String（便于审计串联），daemon **MUST NOT** 发 Number id；client 端响应时 echo 原 `id` 字面值。client **MAY** 对收到的非 String id 回 `-32700 parse_error` 而非静默丢弃，但 daemon 既不发 Number，正常路径不触发 |
| `Severity` / `Direction` / 各 enum | snake_case lowercase 字符串，见 §5 | |

**字段名歧义说明**：
- `paused` 字段在 `sieve.hello` / `sieve.set_paused.result` / `sieve.paused_changed` 中是 `bool`（当前是否暂停态）
- 暂停截止时间字段在 v2.0-r2 起统一命名为 `paused_until`（`Timestamp?`），不再重用 `paused` 命名

**实现 MUST**：
- Rust：`chrono::DateTime<Utc>` 用 `to_rfc3339_opts(SecondsFormat::Millis, true)` 序列化（保证 `Z` 后缀 + 毫秒）
- Swift：`ISO8601DateFormatter` + `formatOptions = [.withInternetDateTime, .withFractionalSeconds]`
- UUID：Rust `uuid::Uuid::to_string()` 默认即 lowercase hyphenated；Swift `UUID().uuidString` 默认大写 → **MUST** 在序列化层 `.lowercased()`

---

## 5. 通用枚举

所有枚举均使用 **snake_case lowercase** 序列化（Rust 端 `#[serde(rename_all = "snake_case")]`，Swift 端 `String` raw value 直接写小写）。

### 5.1 `severity`

`"critical"` | `"high"` | `"medium"` | `"low"`

### 5.2 `direction`

`"inbound"` | `"outbound"`

### 5.3 `disposition`

`"auto_redact"` | `"gui_popup"` | `"hook_terminal"` | `"status_bar"`

> **兼容性标注**：`"gui_popup"` 是历史遗留的 wire 字段值，**保持不变**（向后兼容硬要求，改动会 break 现有 client）。语义上应理解为「需要交互式 decision request」，不绑定 GUI 弹窗显示形式。新 client（CLI / TUI / webhook）收到 `disposition == "gui_popup"` 的 `request_decision` 时，应按自身能力响应该 decision request，无需渲染任何弹窗 UI。

### 5.4 `decision_action`（用于 `request_decision_canceled.auto_decision` 等内部字段）

`"allow"` | `"deny"` | `"redact_and_allow"`

### 5.5 `default_on_timeout`

`"block"` | `"allow"` | `"redact"`

**严格度排序**（用于多 issue 合并的 §6.1.2 取最严格规则）：`"block"` > `"redact"` > `"allow"`。即 `block` 最严，`allow` 最宽；任何字段需要"取最严"时按此顺序取最大者。

> Critical 规则的 `default_on_timeout` 由 daemon 强制为 `"block"`，client 不能通过任何方法（含 `set_preset_overrides`）覆盖。

### 5.6 `preset_mode`

`"strict"` | `"standard"` | `"relaxed"` | `"custom"`

> v1 旧值 `"default"` 在 v2 中重命名为 `"standard"`，daemon 与各端代码必须同步替换。

### 5.7 `source_agent`

`"claude"` | `"open_claw"` | `"hermes"` | `"unknown"`

> 注意 OpenClaw 序列化为 `"open_claw"`（带下划线），不是 `"openclaw"`。`sieve.evaluate.source_agent` 字段也使用此枚举（不再使用 `"claude-code"` 等连字符变体）。

### 5.8 `cancel_reason`（`request_decision_canceled.reason`）

`"timeout"` | `"upstream_disconnected"` | `"duplicate_suppressed"` | `"daemon_shutdown"` | `"resolved_by_peer"`

### 5.9 `notify_kind`（`notify_status_bar.kind`）

`"sequence_hit"` | `"outbound_redacted"` | `"hook_terminal"` | `"user_rules_load_failed"` | `"user_rules_reloaded"` | `"generic"`

### 5.10 `ui_phase`（`decision_response.ui_phase_when_clicked`，仅 GUI client 答复时附带）

`"blue"` | `"orange"` | `"red"` —— 对应 SPEC-002 的 decision request 三段倒计时颜色，便于 daemon 审计用户在哪一阶段做的决定。

> **ℹ️ Note**: 此枚举属于 sieve-gui-macos 仓的 GUI 实现细节（倒计时颜色阶段），
> 不是 daemon IPC 协议契约。非 GUI client 答复 `decision_response` 时此字段应为 `null`（字段已标为 optional，见 §6.2）。
> 详细 GUI 显示规范见 sieve-gui-macos 仓 [SPEC-002 hips-popup-behavior.md](../../../sieve-gui-macos/docs/specs/SPEC-002-hips-popup-window.md)。

---

## 6. 决策协议

### 6.0 wire DTO 与 daemon 内部结构的关系（规范性）

**SPEC-005 只定义 wire 字段**——即 client 与 daemon 之间通过 Unix socket JSON-RPC 实际传输的 JSON 字段。daemon 内部的 Rust 数据结构（`crates/sieve-ipc/src/protocol.rs::DecisionRequest`、`crates/sieve-core/...` 中的命中聚合中间表示等）**不**受本 SPEC 直接约束，可以自由演化。

**daemon 实现侧职责**：在序列化为本节定义的 wire 格式前，**MUST** 完成以下转换（v2 升级时的迁移工作量主要集中于此）：

| daemon 内部字段（v1.x 现状） | wire 字段（v2，本 SPEC） | 转换语义 |
|---|---|---|
| `DecisionRequest.created_at` | `received_at_daemon` | 字段重命名（语义相同：daemon 收到原始检测的时间） |
| `DecisionRequest.detections: Vec<DetectionPayload>` | 单 issue：顶层 `rule_id/title/severity/direction/disposition/context/recommendation`<br>多 issue：顶层 `merged: true` + `issues[]` 展开 | daemon 在序列化层选择渲染形态：`detections.len() == 1` → 单 issue 形式；`detections.len() > 1` → 多 issue 合并形式（拆 issue 字段对应见 §6.1.2）。daemon 内部仍可继续使用 `detections[]` |
| daemon 内部产生的"推荐"结构 | wire `recommendation: { decision, confidence, reason }` | daemon 内部聚合 → wire DTO，所有 i18n / 字段裁剪在这一层完成 |
| daemon 内部 `default_on_timeout: DefaultOnTimeout` enum 值 | wire snake_case lowercase 字符串 | serde `#[serde(rename_all = "snake_case")]` |
| daemon `SourceAgent::OpenClaw` | wire `"open_claw"` | serde 默认 |

**与 sieve-hook 文件 IPC 的隔离**：daemon 内部的 `DecisionRequest` 结构同时还服务 `~/.sieve/pending/<id>.json` 文件 IPC（见 [SPEC-001](SPEC-001-sieve-hook-protocol.md)），那条路径**不**走本协议、**不**受本 SPEC 约束。代码改造时 daemon 端必须明确分离：
- pending file 写入（hook 路径）→ 用 daemon 内部结构直接 serialize（保留 v1.x schema 即可）
- socket JSON-RPC 写入（client 路径）→ 经过 wire DTO 适配层，按本 SPEC §6.1 序列化

**禁止**：把本 SPEC 的字段直接映射到 hook pending file，也**禁止**把 hook pending file 的字段透传到 client socket。两条路径的演化解耦。

### 6.1 `sieve.request_decision`（daemon → client request）

#### 6.1.1 单 issue 形式

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.request_decision",
  "id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
  "params": {
    "request_id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
    "rule_id": "IN-CR-05",
    "title": "签名工具调用：signTransaction",
    "severity": "critical",
    "direction": "inbound",
    "disposition": "gui_popup",
    "timeout_seconds": 120,
    "default_on_timeout": "block",
    "allow_remember": false,
    "merged": false,
    "received_at_daemon": "2026-05-02T15:03:11.234Z",
    "context": {
      "template": "signing_tool_use",
      "tool_name": "signTransaction",
      "chain": "Ethereum",
      "chain_id": 1,
      "typed_data": { /* EIP-712 结构 */ },
      "flags": {
        "infinite_amount": true,
        "deadline_zero": true,
        "approve_all": false
      }
    },
    "recommendation": {
      "decision": "deny",
      "confidence": "high",
      "reason": "deadline=0 + 无限 amount 是 Permit2 钓鱼经典模式"
    },
    "source_agent": "claude",
    "origin_chain": [],
    "source_channel": null,
    "explicit_chain_depth": 0
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 与 JSON-RPC `id` 字面量相同 |
| `rule_id` | String | yes | — | no | 命中规则 ID，如 `IN-CR-05`。**恒非空**：单 issue 模式下 daemon 必发，值来自内部 `Detection.rule_id`（规则 ID 白名单，如 `IN-CR-*` / `OUT-*`），无空串路径。client 收到单 issue 请求 **MAY** 断言 `rule_id` 非空；空串/缺失视为协议违反（`-32602`），不得进入"无命中详情仍可批准"状态。见下方注 ① |
| `title` | String | yes | — | no | 已本地化（语言由 daemon 决定）的简短标题 |
| `severity` | enum (§5.1) | yes | — | no | |
| `direction` | enum (§5.2) | yes | — | no | |
| `disposition` | enum (§5.3) | yes | — | no | 决定 client 处理方式（见 §5.3 标注） |
| `timeout_seconds` | u32 | yes | — | no | 范围 **30–120**；daemon 发送前在唯一 choke point 钳制越界值（含多 issue 合并取最小后越界）到 `[30,120]`，client 可假设恒在此区间。见下方注 ② |
| `default_on_timeout` | enum (§5.5) | yes | — | no | Critical 强制 `"block"` |
| `allow_remember` | bool | yes | — | no | daemon 端通过 `is_critical_locked(rule_id)` 计算；内置 Critical 规则强制 `false`，client 不得覆盖此值。详见下方"Allow Remember 四道防线"一节 |
| `merged` | bool | no | `false` | no | `true` 时必须同时给 `issues[]` |
| `received_at_daemon` | `Timestamp` | yes | — | no | 用于 client 计算超时进度条 |
| `context` | Object | no | `null` | yes | 见 §6.1.3；`null` 或缺失时 client 走通用渲染 |
| `recommendation` | Object | no | `null` | yes | 见 §6.1.4 |
| `source_agent` | enum (§5.7) | yes | — | no | 未识别时为 `"unknown"` |
| `origin_chain` | OriginHop[] | no | `[]` | no（必须是数组或缺失，不接受 `null`） | sub-agent 嵌套调用链；空数组表示用户直接调 |
| `source_channel` | String | no | `null` | yes | OpenClaw 跨通道时的来源 channel；其他 agent 为 `null` 或缺失 |
| `explicit_chain_depth` | u32 | no | `null` | yes | `X-Sieve-Origin` header 真实嵌套深度；`null` 或缺失 → 回退到 `origin_chain.length` |
| `provider_id` | String | no | `null` | yes | *(Since v2.x)* 触发本次决策的 listener 上游 provider_id（多 listener 路由）；供 client 按上游过滤（`decisions watch --provider-id` / `list_pending`）。旧 daemon 不发此字段时 client 静默忽略；单 listener / 系统内部路径为 `null` 或缺失 |

**字段表 6 列规范（全文档统一）**：
- **字段**：JSON 中的 key 名
- **类型**：见 §4A 命名类型 + §5 enum 类型名
- **required**：字段是否必须出现在 JSON 中。`yes` = 序列化方 MUST 写入；`no` = 可省略
- **default if absent**：字段缺失时反序列化方 MUST 应用的默认值
- **null accepted**：是否接受显式 `null`。`yes` = `null` 与缺失等价；`no` = `null` 视为格式错误返 `-32602`
- **说明**：语义、约束、参考章节

**注 ①（单 issue `rule_id` 恒非空，规范性）**：单 issue 形式（`merged` 缺失/`false`）下，daemon **MUST** 填非空 `rule_id`（源自内部 `Detection.rule_id`，结构上无 `Option`、值取自规则 ID 白名单）。这是为了让 client 渲染危险确认弹窗时**必有命中规则详情可展示**——避免"只有标题、无任何命中详情却仍可批准"的知情同意失效。client **MUST** 对单 issue 请求校验 `rule_id` 非空，空串/缺失按格式错误（`-32602`）处理并记 audit，不得进入可批准状态。多 issue 合并形式（`merged=true`）顶层无 `rule_id`/`context`（见 §6.1.2），client 以 `merged` 判分支，不以 `rule_id`/`context` 缺失误判。

**注 ②（`timeout_seconds` daemon 钳制，规范性）**：`timeout_seconds` **MUST** 落在 `[30, 120]`。daemon 在唯一发送 choke point（`IpcServer::request_decision`）对 wire 值钳制：越界（含 `0`、多 issue 合并取最小值后 `< 30`、或 `> 120`）一律钳到 `[30,120]` 边界并记 warn，**不拒绝**（拒绝会中断决策流→可能 fail-open）。client（GUI）对该字段无需自做下限校验即可安全用于倒计时，但 **SHOULD** 仍钳制以防御非常规 daemon。daemon 自身的 oneshot 兜底超时（`default_on_timeout`）与下发的 `timeout_seconds` 由调用方保持一致。

**Allow Remember 四道防线**（灰名单决策在协议层的体现）：

| # | 防线 | 责任方 | 协议要求 |
|---|---|---|---|
| 1 | daemon 计算 | daemon | daemon **MUST** 在发送请求前调用 `is_critical_locked(rule_id)`，命中即 `allow_remember=false`；**禁止**让用户配置 override 此字段 |
| 2 | client UI 禁用 | GUI client | `allow_remember=false` 时 GUI client **MUST** 在 decision request UI 中禁用且灰显 Remember 控件，**禁止**渲染为可交互状态（不允许"灰显但仍可点击"，不允许键盘快捷键绕过） |
| 3 | client 编码层强制 | client | client 提交 `decision_response` 前 **MUST** 在编码层强制：当对应请求的 `allow_remember=false` 时，序列化时把 `remember` 字段强制写为 `false`（即便上层错误地传入 `true`） |
| 4 | daemon 二次校验 | daemon | daemon 收到 `remember=true` 时 **MUST** 重新调用 `is_critical_locked(rule_id)` 校验。命中 → 忽略 remember 字段 + 写 audit `kind=graylist_critical_rejected`，不进入灰名单写入路径 |

任意一道防线失效都不影响整体安全：1+4 daemon 双向校验保证 client 端被绕过时仍安全；2+3 client 双重保护防止 UI bug 让用户误操作。代码 PR 必须为每道防线写独立测试。

`OriginHop` schema：

```jsonc
{
  "agent": "claude",
  "action": "delegate",
  "timestamp": "2026-05-02T15:03:09.123Z"
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `agent` | enum (§5.7) | yes | — | no | 此跳的来源 agent |
| `action` | enum | yes | — | no | `"user_input"` / `"delegate"` / `"skill_invoke"` / `"channel_message"` |
| `timestamp` | `Timestamp` | yes | — | no | 此跳发生的时间 |

#### 6.1.2 多 issue 合并形式

`merged: true` 时，顶层字段语义如下：

| 顶层字段 | 取值规则 |
|---|---|
| `severity` | 取所有 issue 中最严重的 |
| `direction` | 全部 issue 同方向时填该方向；混合时 daemon 必须**拆成两个独立 request_decision**，不允许混合 direction merge |
| `disposition` | 全部 issue 同 disposition 时填该值；混合时同上拆请求 |
| `timeout_seconds` | 取所有 issue 中最小 |
| `default_on_timeout` | 按 §5.5 严格度排序（`block` > `redact` > `allow`）取最严格者 |
| `allow_remember` | 任一 issue `allow_remember=false` → 顶层强制 `false` |
| `title` | daemon 生成的合并文案（如"Sieve 检测到 N 个安全问题"） |
| `rule_id` / `context` / `recommendation` | **不发**（避免 client 误用合并请求里的单 issue 字段） |
| `issues` | 数组，每项见下表 |

每个 issue 对象：

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `issue_id` | String | yes | — | no | 本次合并请求内唯一，建议形如 `"i-1"` / `"i-2"` |
| `rule_id` | String | yes | — | no | |
| `title` | String | yes | — | no | |
| `severity` | enum (§5.1) | yes | — | no | |
| `allow_remember` | bool | yes | — | no | 单 issue 维度的 daemon 计算结果 |
| `context` | Object | no | `null` | yes | 见 §6.1.3 |
| `recommendation` | Object | no | `null` | yes | 见 §6.1.4 |

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.request_decision",
  "id": "9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
  "params": {
    "request_id": "9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
    "title": "Sieve 检测到 2 个安全问题",
    "severity": "critical",
    "direction": "inbound",
    "disposition": "gui_popup",
    "timeout_seconds": 30,
    "default_on_timeout": "block",
    "allow_remember": false,
    "merged": true,
    "received_at_daemon": "2026-05-02T15:03:11.234Z",
    "source_agent": "claude",
    "origin_chain": [],
    "source_channel": null,
    "explicit_chain_depth": 0,
    "issues": [
      {
        "issue_id": "i-1",
        "rule_id": "IN-CR-05",
        "title": "签名工具调用：signTransaction",
        "severity": "critical",
        "allow_remember": false,
        "context": { "template": "signing_tool_use", /* ... */ },
        "recommendation": { /* ... */ }
      },
      {
        "issue_id": "i-2",
        "rule_id": "IN-GEN-04",
        "title": "Markdown 图片外链",
        "severity": "high",
        "allow_remember": true,
        "context": { "template": "markdown_exfil", /* ... */ },
        "recommendation": { /* ... */ }
      }
    ]
  }
}
```

#### 6.1.3 `context.template` 模板表

| `template` 值 | 触发规则示例 | 关键字段（除 `template` 外） |
|---|---|---|
| `address_compare` | IN-CR-01 | `original_address: String`, `substituted_address: String`, `chain: String`, `levenshtein: u32` |
| `signing_tool_use` | IN-CR-05 | `tool_name: String`, `chain: String`, `chain_id: u32?`, `typed_data: any`, `flags: { infinite_amount: bool, deadline_zero: bool, approve_all: bool }` |
| `markdown_exfil` | IN-GEN-04 | `markdown_snippet: String`, `urls: String[]`, `reachable: bool[]?` |
| `secret_outbound` | OUT-07/09/10 | `secret_kind: String`, `prefix4: String`, `suffix4: String`, `length: u32`, `hash_short: String` |
| `generic_json` | 兜底 | `payload: any` —— 任意 JSON 子树，client 走通用 key-value 渲染 |

**client 不识别的 `template` 值必须降级为 `generic_json` 处理**，并把整个 context 对象作为 `payload` 渲染。

**敏感数据保护**：
- `secret_outbound` 永远只发 `prefix4` / `suffix4` / 长度 / 短哈希，**不发原文**
- `address_compare` 发完整地址（地址本身就是公开的，不属于敏感数据）
- `signing_tool_use.typed_data` 透传 EIP-712 原始结构（地址、金额、合约调用都是用户即将签名的内容，必须可见）
- `markdown_exfil.markdown_snippet` 限制在 1 KiB 以内

#### 6.1.4 `recommendation` 字段

```jsonc
{
  "decision": "deny",         // §5.4 decision_action 中的 "allow" | "deny"（不含 redact_and_allow）
  "confidence": "high",       // "high" | "medium" | "low"
  "reason": "deadline=0 + 无限 amount 是 Permit2 钓鱼经典模式"  // 已本地化，≤ 240 字符
}
```

> **ℹ️ Note**: 以下「GUI 默认按钮规则」属于 sieve-gui-macos 仓的 GUI 实现细节，
> 不是 daemon IPC 协议契约。新 client（CLI / TUI / webhook）实现时无需对齐这些按钮行为。
> 详细 GUI 显示规范见 sieve-gui-macos 仓 [SPEC-002 hips-popup-behavior.md](../../../sieve-gui-macos/docs/specs/SPEC-002-hips-popup-window.md)。

**GUI 默认按钮规则**（GUI client 实现参考）：当 `recommendation` 缺失或 `recommendation.confidence != "high"` 时，HIPS 主按钮（含键盘 Return 默认）必须为 **拒绝**。详见 [SPEC-002](SPEC-002-hips-popup-behavior.md)。

### 6.2 `sieve.decision_response`（client → daemon response）

client 通过 JSON-RPC response 形式回复 `request_decision`，**复用同一 `id`**，无独立 method 字段。

#### 6.2.1 单 issue 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
  "result": {
    "request_id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
    "decision": "deny",
    "decided_at": "2026-05-02T15:03:18.512Z",
    "by_user": true,
    "remember": false,
    "context_hint": null,
    "ui_phase_when_clicked": "blue"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 必须等于请求中的 `request_id`（与 JSON-RPC `id` 同值） |
| `decision` | enum (§5.4 子集) | yes | — | no | `"allow"` / `"deny"`（client 端不发 `redact_and_allow`） |
| `decided_at` | `Timestamp` | yes | — | no | client 端时钟，daemon 用于审计 |
| `by_user` | bool | yes | — | no | `true` = 用户主动操作；`false` = client 端倒计时归零的 fail-closed 兜底（GUI 倒计时到点主动回 `decision=deny, by_user=false`，而非静默关窗）。daemon **MUST** 幂等处理（见下方注 ③） |
| `remember` | bool | no | `false` | no | 用户勾选"永久允许"；**当 request 中 `allow_remember=false` 时 client 必须强制 `false`**（编码层 reject，UI bug 兜底） |
| `context_hint` | String | no | `null` | yes | 用户备注，≤ 200 个 Unicode scalar（见 §1.3）；client MUST 拒绝超限提交 |
| `ui_phase_when_clicked` | enum (§5.10) | no | `null` | yes | 非 GUI client 或 disposition≠gui_popup 时允许为 `null` |

**注 ③（超时决策回传幂等性，规范性）**：GUI 倒计时归零时**主动**回传 `decision_response`（`decision=deny`、`by_user=false`；merged 请求按 `all_deny`），实现 client 侧 fail-closed，而非依赖 daemon 兜底静默放行。daemon 端同时持有自己的 oneshot 兜底超时（`tokio::time::timeout` + `default_on_timeout`）。两者 **MUST** 按 `request_id` 幂等，对同一请求只处置一次：
- GUI 的 deny 先到 → daemon 从 pending map 取出并消费该 `request_id` 的 oneshot（`pending.remove`），后续自身兜底超时触发时已无对应条目，**MUST NOT** 重复处置。
- daemon 兜底超时先到 → 清理 pending + 广播 `request_decision_canceled`；此后到达的迟到 `decision_response` 因 `request_id` 已不在 pending map，**MUST** 被忽略（仅记日志），不得二次执行动作。

daemon 的兜底超时 **SHOULD** ≥ 下发的 `timeout_seconds`（让 GUI 的显式 deny 优先落定，避免 daemon 抢先兜底）。

#### 6.2.2 多 issue 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
  "result": {
    "request_id": "9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
    "merged_decision": "partial",
    "per_issue": [
      { "issue_id": "i-1", "decision": "deny",  "remember": false, "context_hint": null },
      { "issue_id": "i-2", "decision": "allow", "remember": true,  "context_hint": "测试中允许" }
    ],
    "decided_at": "2026-05-02T15:03:18.512Z",
    "ui_phase_when_clicked": "blue"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 同请求 |
| `merged_decision` | enum | yes | — | no | `"all_deny"` / `"all_allow"` / `"partial"` |
| `per_issue` | Array | yes | — | no | 必须覆盖请求中所有 `issue_id`，顺序不限 |
| `per_issue[].issue_id` | String | yes | — | no | |
| `per_issue[].decision` | enum | yes | — | no | `"allow"` / `"deny"` |
| `per_issue[].remember` | bool | yes | — | no | 单 issue 维度，受该 issue 的 `allow_remember` 约束 |
| `per_issue[].context_hint` | String | no | `null` | yes | ≤ 200 Unicode scalar |
| `decided_at` | `Timestamp` | yes | — | no | |
| `ui_phase_when_clicked` | enum (§5.10) | no | `null` | yes | |

**daemon 端校验**：
- 收到 `remember=true` 时必须二次校验请求侧 `allow_remember=true` **且** `rule_id` 不在 `critical_lock::FAIL_CLOSED_RULES`；任一不满足 → 忽略 remember + 写 audit `kind=graylist_critical_rejected`
- `per_issue` 缺漏任何 `issue_id` → 缺漏的 issue 走 `default_on_timeout` 处置 + 写 audit `kind=decision_response_incomplete`

#### 6.2.3 错误响应

client 在用户主动取消、client 渲染失败、client 进程关停等场景下用 JSON-RPC error 回复，复用同 `id`：

```jsonc
{
  "jsonrpc": "2.0",
  "id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
  "error": {
    "code": -32100,
    "message": "user_canceled_via_window_close"
  }
}
```

错误码列表见 §12。daemon 收到任何 client 错误 → 按 `default_on_timeout` 处置该请求并写 audit `kind=decision_response_error`。

#### 6.2.4 GUI peer 代码签名核验 gate（F1-b，规范性，macOS）

wire 应答通道（本节 §6.2 路径）存在同用户进程抢先连接 socket 冒充 GUI 的攻击面
（连接注册进 `gui_writers` 不做任何身份核验）。daemon 配置 `gui_peer_code_requirement`
（`config.toml` 顶层字段，macOS SecRequirement 语法）后：

- pending 的 `max_severity == critical`（daemon 侧计算，见 §11D，不信 client 自报）
  且应答 `decision ∈ { allow, redact_and_allow }` 时，daemon **MUST** 对该连接对端进程做
  代码签名核验（`getsockopt(LOCAL_PEERTOKEN)` 取 audit token → Security framework
  `SecCodeCopyGuestWithAttributes` + `SecCodeCheckValidity`）。
- 核验未通过 → 该应答 **MUST** 静默改写为 `deny`（`remember` 清零），与 §11E 的
  A 方案改写同范式；不回错误、不断连接。
- 核验按连接懒执行且缓存（同连接至多真验一次）；`deny` 应答与非 Critical 应答不经核验
  （拒绝是安全方向，不加摩擦）。
- 未配置（默认）→ 不核验，daemon 启动时打 warn 记录残余风险。非 macOS 平台配置本字段 →
  恒拒（fail-closed，平台无核验能力）。
- `resolve_decision`（§11E）不经此 gate——它有自己的 A 方案门禁（Critical 不开口子）。

> **威胁模型与残余风险（诚实记录）**：本 gate 回答「对端是不是签名过的 Sieve GUI 二进制」。
> TouchID 在 wire 上零防伪价值（LocalAuthentication 只返回本地 bool，不签发可传递凭证），
> GUI 侧 TouchID 仅是人在场 UX 信号；peer 代码签名核验才是防冒充的真防线。已知残余：
> 挡不住 agent 调用合法签名的 `sieve` CLI——那条路径由 §11E A 方案负责，两条防线合并
> 覆盖两个通道。gate 是配置 opt-in：源码构建 / dogfood 无签名信任锚（daemon 二进制走
> cosign 而非 Apple 签名，无法自派生 Team 锚点），强制默认开启会锁死本地开发。

### 6.3 `sieve.request_decision_canceled`（daemon → client fan-out notification）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.request_decision_canceled",
  "params": {
    "request_id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
    "reason": "timeout",
    "auto_decision": "deny"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 必须对应一条已发出的 `request_decision` |
| `reason` | enum (§5.8) | yes | — | no | |
| `auto_decision` | enum (§5.4) | yes | — | no | daemon 已应用的兜底处置（`"allow"` / `"deny"` / `"redact_and_allow"`） |

client 端处理：
- 该 `request_id` 在 pendingQueue 中 → 移除
- 该 `request_id` 是 activeRequest → 关闭 decision request UI，恢复正常状态
- 不弹任何"被拒绝"提示（daemon 已落审计，避免重复打扰用户）

---

## 7. 状态栏通知：`sieve.notify_status_bar`

### 7.1 行为

- **方向**：daemon → client
- **类型**：notification（无 `id`）
- **fan-out**：广播给所有连接的 client；写入失败的 sender 在下次广播时 lazy 清理
- **用途**：所有不需要打断用户的事件提示（出站脱敏、行为序列命中、用户规则 reload、hook 类规则在 TTY 完成后的事件回报、daemon 内部信息）

### 7.2 Schema

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.notify_status_bar",
  "params": {
    "notify_id": "a1b2c3d4-5e6f-4789-90ab-cdef01234567",
    "created_at": "2026-05-02T15:03:11.234Z",
    "kind": "outbound_redacted",
    "title": "已脱敏：检测到 Anthropic API key",
    "detail": "OUT-01 命中，已替换为 [REDACTED-API-KEY]",
    "rule_id": "OUT-01",
    "auto_dismiss_seconds": 5
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `notify_id` | `Uuid` | yes | — | no | 全局唯一，便于去重 |
| `created_at` | `Timestamp` | yes | — | no | daemon 端时钟 |
| `kind` | enum (§5.9) | yes | — | no | |
| `title` | String | yes | — | no | < 80 字符，状态栏短文案 |
| `detail` | String | no | `null` | yes | 长文案，client tooltip 或副标题展示 |
| `rule_id` | String | no | `null` | yes | 关联规则 ID |
| `auto_dismiss_seconds` | u32 | yes | — | no | `0` = 常驻直到用户关闭 |

### 7.3 NotifyKind 触发场景

| `kind` | 触发场景 | 推荐 `auto_dismiss_seconds` |
|---|---|---|
| `sequence_hit` | 行为序列窗口命中可疑工具调用序列 | `0`（用户需手动确认） |
| `outbound_redacted` | OUT-01~05/12 AutoRedact 命中 | `5` |
| `hook_terminal` | sieve-hook 在 TTY 完成的事件回报，让 client 同步显示一份"刚才在终端发生了什么" | `8` |
| `user_rules_load_failed` | `user.toml` 加载/lint 失败，daemon 已 fail-safe 保留旧规则 | `0` |
| `user_rules_reloaded` | 用户规则成功热加载 | `5` |
| `generic` | 其他 daemon 内部通知 | 由 daemon 决定 |

---

## 8. CLI → daemon 单向：`sieve.reload_user_rules`

### 8.1 用途

`sieve rules edit` / `enable` / `disable` 完成 atomic rename 后，向 daemon 发送一次性信号，触发用户规则热加载。CLI 进程通过短连接发完此 notification 即关闭 socket。

### 8.2 Schema

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.reload_user_rules",
  "params": {
    "trigger_id": "f1e2d3c4-b5a6-4789-8c0d-1e2f3a4b5c6d"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `trigger_id` | `Uuid` | no | `null` | yes | 仅用于日志关联，daemon 不在响应中回 |

### 8.3 daemon 行为

1. 读取 `~/.sieve/rules/user.toml`
2. lint + 编译 UserEngine
3. 成功 → ArcSwap atomic swap LayeredEngine 内的 user 字段；广播 `notify_status_bar { kind: "user_rules_reloaded" }`
4. 失败 → 保留旧引擎；广播 `notify_status_bar { kind: "user_rules_load_failed" }` 并写 audit

---

## 9. 控制面方法（client → daemon request/response）

所有方法均使用 JSON-RPC 2.0 request/response 形式，必须带 `id`。daemon 端实现见 `crates/sieve-ipc/src/socket_server.rs`。

### 9.1 `sieve.set_paused`

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.set_paused",
  "id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
  "params": { "minutes": 30 }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `minutes` | u32 | yes | — | no | 范围 `[0, 60]`；`0` = 立即恢复；超过 60 daemon 返 `-32602 invalid_params` |

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02",
  "result": {
    "paused": true,
    "paused_until": "2026-05-02T15:33:11.234Z",
    "applies_to": ["status_bar", "auto_redact"]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `paused` | bool | yes | — | no | 当前状态（`minutes=0` 时为 `false`） |
| `paused_until` | `Timestamp` | no | `null` | yes | `paused=false` 时为 `null`（v2.0-r2 起字段名从 `until` 改为 `paused_until`，避免与 `paused: bool` 字段名歧义） |
| `applies_to` | String[] | yes | — | no | 暂停影响的 `disposition` 列表（取自 §5.3 enum 值）；**永远不包含 `gui_popup` / `hook_terminal` 中关联 critical_lock 规则的项**（Critical fail-closed 不可绕过；`gui_popup` wire 值含义见 §5.3 标注） |

#### Fan-out 强制

按 §10.0.1，daemon **MUST** 在返回 result 之前先 fan-out 一条 `sieve.paused_changed { source: "gui", origin_request_id: <本请求 id> }`，所有当前连接的 client 都会收到。`minutes=0`（恢复）时也必须 fan-out（`paused=false` + `paused_until=null` + `reason="user_request"`）。

### 9.2 `sieve.set_preset`

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.set_preset",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "params": { "mode": "strict" }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `mode` | enum (§5.6) | yes | — | no | 不在枚举内 daemon 返 `-32602` |

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": { "applied_at": "2026-05-02T15:03:11.234Z" }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `applied_at` | `Timestamp` | yes | — | no | daemon 应用 preset 的时间 |

成功后 daemon 必须 fan-out 一条 `sieve.preset_changed` 通知（见 §10.1），`source: "gui"`，并附 `origin_request_id` 等于本次请求的 `id`，便于发起方 client 识别回声。

### 9.3 `sieve.set_preset_overrides`

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.set_preset_overrides",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "params": {
    "overrides": {
      "OUT-08": { "timeout_seconds": 90, "default_on_timeout": "allow" }
    }
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `overrides` | Object<rule_id → PresetOverride> | yes | — | no | 空对象表示清空所有 overrides |
| `overrides[rule_id].timeout_seconds` | u32 | yes | — | no | 范围 30–120 |
| `overrides[rule_id].default_on_timeout` | enum (§5.5) | yes | — | no | |

**Critical 锁防线（防线二）**：`rule_id` 在 `sieve_rules::critical_lock::FAIL_CLOSED_RULES` 集合内的 override 必须被 daemon 拒绝；**采用 partial success 语义**——daemon **MUST** 返回 JSON-RPC `result`（不是 `error`），通过 `rejected[]` 数组逐条报告被拒绝的 rule_id。

> **错误码 `-32001 critical_lock_violated` 不用于本方法**——`set_preset_overrides` 的 critical_lock 拒绝路径**只**走 `result.rejected[]`。`-32001` 保留给未来其他可能的 critical_lock 拦截路径（详见 §12.3）。

整体失败仅在以下场景返 `error`：
- 任何 override 的 `timeout_seconds` 不在 30–120 → `-32602 invalid_params`
- `default_on_timeout` 不在 §5.5 枚举内 → `-32602 invalid_params`
- 单个 rule_id 不存在但其他可应用 → 进入 `rejected[].reason="unknown_rule"`，不影响其他 rule_id 应用
- `params` 整体格式错 → `-32602`

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "applied": ["OUT-08"],
    "rejected": [
      { "rule_id": "IN-CR-05", "reason": "critical_lock" }
    ]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `applied` | String[] | yes | — | no | 成功应用的 rule_id 列表 |
| `rejected` | RejectedOverride[] | yes | — | no | 被拒绝的 override 列表 |
| `rejected[].rule_id` | String | yes | — | no | |
| `rejected[].reason` | enum | yes | — | no | `"critical_lock"` / `"unknown_rule"` / `"invalid_value"` |

被拒绝的 critical_lock override **必须**写 audit `kind=critical_lock_blocked`。

**只要 `applied` 非空**（即至少一条 override 应用成功）daemon **MUST** fan-out `sieve.preset_changed { source: "gui", origin_request_id: <本请求 id> }`，`overrides` 字段反映最新全集。`applied` 为空（全部 rejected）时**不**广播。

### 9.4 `sieve.reload_config`

#### 请求

```jsonc
{ "jsonrpc": "2.0", "method": "sieve.reload_config", "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08" }
```

`params` 可省略或为空对象 `{}`。

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "reloaded_at": "2026-05-02T15:03:11.234Z",
    "system_rules_count": 47,
    "user_rules_count": 3,
    "user_rules_errors": ["user.toml: rule MY-CURL-PIPE has invalid severity \"extreme\""]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `reloaded_at` | `Timestamp` | yes | — | no | daemon 完成 reload 的时间 |
| `system_rules_count` | u32 | yes | — | no | 系统规则数 |
| `user_rules_count` | u32 | yes | — | no | 用户规则数（lint 失败的规则不计入） |
| `user_rules_errors` | String[] | yes | — | no | lint 警告/错误清单；空数组表示全部通过 |

**fail-safe**：系统规则 lint 失败 → daemon 保留旧引擎 + 返 `-32002 daemon_busy`（视为重载失败但服务不中断）；用户规则失败 → 单条跳过，整体不失败。

### 9.5 `sieve.health`

#### 请求

```jsonc
{ "jsonrpc": "2.0", "method": "sieve.health", "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08" }
```

`params` 可省略或为空对象 `{}`。daemon 端无副作用、不写 audit。

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "daemon_version": "0.7.2",
    "protocol_version": "v2",
    "started_at": "2026-05-02T11:21:30.001Z",
    "uptime_seconds": 14523,
    "preset": {
      "mode": "standard",
      "overrides": {}
    },
    "paused": false,
    "paused_until": null,
    "listen": { "addr": "127.0.0.1", "port": 11453 },
    "listeners": [
      { "addr": "127.0.0.1", "port": 11453, "provider_id": "anthropic", "protocol": "anthropic" }
    ],
    "audit_db": {
      "path": "/Users/foo/.sieve/audit.db",
      "size_bytes": 2048576,
      "schema_version": 2,
      "events_total": 12453,
      "events_today": 142
    },
    "rules": {
      "system_count": 47,
      "user_count": 3,
      "last_reload": "2026-05-02T11:21:31.234Z"
    },
    "graylist": { "active_count": 5 },
    "ipc": { "connected_clients": 1, "total_decisions_inflight": 0 }
  }
}
```

| 顶层字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `daemon_version` | String | yes | — | no | semver |
| `protocol_version` | String | yes | — | no | 当前 `"v2"` |
| `started_at` | `Timestamp` | yes | — | no | |
| `uptime_seconds` | u64 | yes | — | no | |
| `preset` | PresetSnapshot | yes | — | no | 见 §9.5.1 |
| `paused` | bool | yes | — | no | 当前是否暂停态 |
| `paused_until` | `Timestamp` | no | `null` | yes | 暂停截止时间；`paused=false` 时为 `null` |
| `listen` | ListenSnapshot | yes | — | no | **Deprecated since v2.x**：等价于 `listeners[0]`，仅向后兼容旧 client；新 client 应读 `listeners` 数组。见 §9.5.4 |
| `listeners` | ListenerSnapshot[] | yes (since v2.x) | `[]` | no | 多 listener 完整快照；旧 daemon 不发此字段时 client 拿到空数组。见 §9.5.4 |
| `audit_db` | AuditDbSnapshot | yes | — | no | 见 §9.5.2 |
| `rules` | RulesSnapshot | yes | — | no | 见 §9.5.3 |
| `graylist` | GraylistSnapshot | yes | — | no | 见 §9.5.4 |
| `ipc` | IpcSnapshot | yes | — | no | 见 §9.5.4 |

#### 9.5.1 `PresetSnapshot`

```jsonc
{
  "mode": "standard",
  "overrides": { "OUT-08": { "timeout_seconds": 90, "default_on_timeout": "allow" } }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `mode` | enum (§5.6) | yes | — | no | |
| `overrides` | Object<rule_id → PresetOverride> | yes | — | no | 仅 `mode == "custom"` 时可能非空；其他 mode daemon 返回空 object `{}` |

#### 9.5.2 `AuditDbSnapshot`

```jsonc
{
  "path": "/Users/foo/.sieve/audit.db",
  "size_bytes": 2048576,
  "schema_version": 2,
  "events_total": 12453,
  "events_today": 142
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `path` | String | yes | — | no | audit.db 绝对路径 |
| `size_bytes` | u64 | yes | — | no | 文件大小（字节） |
| `schema_version` | u32 | yes | — | no | `PRAGMA user_version` 值 |
| `events_total` | u64 | yes | — | no | 累计 event 总数 |
| `events_today` | u64 | yes | — | no | 当天 event 数；以 daemon 进程时区的 00:00:00 为分界 |

#### 9.5.3 `RulesSnapshot`

```jsonc
{
  "system_count": 47,
  "user_count": 3,
  "last_reload": "2026-05-02T11:21:31.234Z"
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `system_count` | u32 | yes | — | no | |
| `user_count` | u32 | yes | — | no | |
| `last_reload` | `Timestamp` | no | `null` | yes | daemon 启动后未 reload 时为 `null` |

#### 9.5.4 `ListenSnapshot` / `ListenerSnapshot` / `GraylistSnapshot` / `IpcSnapshot`

```jsonc
{ "addr": "127.0.0.1", "port": 11453 }       // ListenSnapshot（deprecated since v2.x）
{ "addr": "127.0.0.1", "port": 11453, "provider_id": "anthropic", "protocol": "anthropic" }
                                              // ListenerSnapshot（since v2.x）
{ "active_count": 5 }                         // GraylistSnapshot
{ "connected_clients": 1, "total_decisions_inflight": 0 }  // IpcSnapshot
```

| DTO | 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|---|
| ListenSnapshot | `addr` | String | yes | — | no | 反向代理监听地址，**MUST** 为 `"127.0.0.1"` |
| ListenSnapshot | `port` | u16 | yes | — | no | 反向代理监听端口（默认 11453） |
| ListenerSnapshot | `addr` | String | yes | — | no | 强制 `"127.0.0.1"`（完全本地运行硬约束） |
| ListenerSnapshot | `port` | u16 | yes | — | no | 监听端口；multi-listener 各自唯一 |
| ListenerSnapshot | `provider_id` | String | yes | — | no | 来自 `sieve.toml [[upstream]] provider_id`；留空时从 URL host 派生 |
| ListenerSnapshot | `protocol` | String | yes | — | no | `"anthropic"` \| `"openai"`；错位请求 fail-closed 400 |
| GraylistSnapshot | `active_count` | u32 | yes | — | no | 当前生效的灰名单条目数（含未过期项） |
| IpcSnapshot | `connected_clients` | u32 | yes | — | no | 当前 IPC socket 已 accept 的 client 数 |
| IpcSnapshot | `total_decisions_inflight` | u32 | yes | — | no | 当前等待 client 响应的 `request_decision` 数 |

**ListenSnapshot vs ListenerSnapshot 版本兼容矩阵**：

- 新 daemon + 新 client：`listeners` 数组权威，`listen` = `listeners[0]` 兼容字段
- 新 daemon + 旧 client：旧 client 忽略 `listeners`，读 `listen` 单值继续工作
- 旧 daemon + 新 client：旧 daemon 不发 `listeners`，client 因 `#[serde(default)]` 拿空数组，应回退读 `listen` 单值

### 9.6 `sieve.evaluate`（沙箱评估）

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.evaluate",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "params": {
    "direction": "outbound",
    "content_kind": "tool_use_input",
    "source_agent": "claude",
    "payload": "<≤ 64 KiB>"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `direction` | enum (§5.2) | yes | — | no | |
| `content_kind` | enum | yes | — | no | `"raw_text"` / `"tool_use_input"` / `"model_response"` |
| `source_agent` | enum (§5.7) | yes | — | no | |
| `payload` | String | yes | — | no | UTF-8；超 64 KiB daemon 返 `-32003 payload_too_large` |

**daemon 不写 audit、不动 SessionState、不影响真实 SSE 流**。

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "evaluated_at": "2026-05-02T15:03:11.234Z",
    "matches": [
      {
        "rule_id": "IN-GEN-02",
        "rule_kind": "system",
        "severity": "critical",
        "disposition": "hook_terminal",
        "matched_pattern_summary": "curl POST to external endpoint",
        "fields_triggered": ["body.text"],
        "would_decision": "deny",
        "would_recommendation": {
          "decision": "deny",
          "confidence": "high",
          "reason": "command 包含 curl POST + 外部域名，疑似数据外泄"
        }
      }
    ],
    "no_match": ["IN-CR-02", "user:MY-CURL-PIPE"]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `evaluated_at` | `Timestamp` | yes | — | no | |
| `matches` | EvaluateMatch[] | yes | — | no | |
| `matches[].rule_id` | String | yes | — | no | |
| `matches[].rule_kind` | enum | yes | — | no | `"system"` / `"user"` |
| `matches[].severity` | enum (§5.1) | yes | — | no | |
| `matches[].disposition` | enum (§5.3) | yes | — | no | |
| `matches[].matched_pattern_summary` | String | yes | — | no | critical_lock 规则命中时仅含规则类型摘要（如 `"BIP39 with checksum match"`），**禁止**回填原 payload 片段 |
| `matches[].fields_triggered` | String[] | yes | — | no | 命中字段路径，如 `["body.text", "tool_use.input.command"]` |
| `matches[].would_decision` | enum (§5.4) | yes | — | no | `"allow"` / `"deny"` / `"redact_and_allow"` |
| `matches[].would_recommendation` | Recommendation | no | `null` | yes | 与 §6.1.4 同结构 |
| `no_match` | String[] | no | `[]` | no | 抽样的未命中 rule_id；不保证完整 |

### 9.7 `sieve.list_graylist`

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.list_graylist",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "params": { "limit": 50, "cursor": null }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `limit` | u32 | no | `50` | yes | 最大值 200 |
| `cursor` | String | no | `null` | yes | 分页游标，从上一次响应的 `next_cursor` 取 |

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "entries": [
      {
        "fingerprint": "7a3f9c1d5e6b8a204c1f3d8e9b2a7c5f1e3d8b9a2c4e6f1a3b5c7d9e0f2a4b6",
        "rule_id": "IN-GEN-04",
        "rule_kind": "system",
        "added_at": 1745683210000,
        "added_by": "user@hostname",
        "context_hint": "测试外链",
        "match_count_since": 4,
        "expires_at": null
      }
    ],
    "next_cursor": null
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `entries[]` | GraylistEntrySummary[] | yes | — | no | |
| `entries[].fingerprint` | String | yes | — | no | hex digest，用于 `remove_graylist` |
| `entries[].rule_id` | String | yes | — | no | |
| `entries[].rule_kind` | enum | yes | — | no | `"system"` / `"user"` |
| `entries[].added_at` | `UnixMs` | yes | — | no | |
| `entries[].added_by` | String | yes | — | no | 添加者标识 |
| `entries[].context_hint` | String | no | `null` | yes | |
| `entries[].match_count_since` | u64 | yes | — | no | 添加后被命中次数 |
| `entries[].expires_at` | `UnixMs` | no | `null` | yes | `null` = 永不过期 |
| `next_cursor` | String | no | `null` | yes | `null` 表示已到末页 |

**隐私保护**：daemon 端**禁止**返回 `fingerprint_inputs.matched_canonical` 或任何原文片段。完整 inputs 查看路径推 v2.x。

### 9.8 `sieve.remove_graylist`

#### 请求

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.remove_graylist",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "params": { "fingerprint": "7a3f9c1d5e6b8a204c1f3d8e9b2a7c5f1e3d8b9a2c4e6f1a3b5c7d9e0f2a4b6" }
}
```

#### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "result": {
    "removed": true,
    "audit_event_id": "evt_a1b2c3"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `removed` | bool | yes | — | no | 删除成功时为 `true`；当前协议下不会出现 `false`（daemon **MUST** 在 fingerprint 不存在时返 `-32004 unknown_fingerprint` error，不进入 result 路径）。本字段保留 `bool` 类型为未来扩展预留 |
| `audit_event_id` | String | yes | — | no | 写入 audit 的 event 唯一 ID |

**fingerprint 不存在的处理**：daemon **MUST** 返 error response，code `-32004`，`data: { "fingerprint": "<原值>" }`：

```jsonc
{
  "jsonrpc": "2.0",
  "id": "4d2e8a1c-9f6b-4730-b8d5-2c5e7f1a9c08",
  "error": {
    "code": -32004,
    "message": "unknown_fingerprint",
    "data": { "fingerprint": "7a3f9c1d5e6b8a204c1f3d8e9b2a7c5f1e3d8b9a2c4e6f1a3b5c7d9e0f2a4b6" }
  }
}
```

> **未来扩展**：若后续协议引入 `params.idempotent: true`，仅在该模式下 daemon 才允许返 `result.removed: false`（表示"指定 fingerprint 不存在但视为成功"）。当前协议**不支持**该模式，`removed: false` 永远不会出现在 wire 上。

---

## 10. 暂停与 preset 状态广播（daemon → client fan-out notification）

### 10.0 多 client 回声防护（适用于本章所有通知）

为支持多 client 并存场景下的回声去重，本章所有 fan-out notification **MUST** 携带 `origin_request_id` 字段：

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `origin_request_id` | `Uuid` | yes | — | yes | 当通知由某条 client request 触发时，**MUST** 设为该 request 的 `id` 字面量；当通知由 CLI / 配置 reload / 自动恢复 / daemon 启动等非 client 路径触发时，**MUST** 设为 `null` |

#### 10.0.1 daemon 端发送顺序与串行化（规范性）

为避免 client 端竞态，daemon **MUST** 严格按以下顺序处理任何由 client request 触发的状态变更（含 §9.1 / §9.2 / §9.3 等会触发 fan-out 通知的 mutating 控制面方法）：

1. 应用变更（修改内部状态）
2. **先** fan-out 对应通知（含 `origin_request_id = <本 request id>`），**等待**写入所有当前连接的 client socket（write buffer flushed 即可，不等 OS-level ack）
3. **再** 返回 `result` response 给发起方 client

**无对应 fan-out 通知的 mutating 请求**（如 `sieve.reload_config` —— 当前协议**未**定义 `rules_changed` / `config_reloaded` 之类通知）：步骤 2 为 no-op（直接跳过），但仍 **MUST** 经过同一状态变更队列串行化 apply 与 result。**禁止**因为没有 fan-out 就跳过串行化，否则会破坏与并行 mutating 请求的线性化顺序。

**全局串行化要求**：daemon **MUST** 通过单一状态变更队列 / mutex 串行化所有 mutating 控制面请求（`set_paused` / `set_preset` / `set_preset_overrides` / `reload_config` 等）；**禁止**多请求间交错执行 apply / fan-out / result 步骤。多 client 同时发起变更时按 daemon 接收顺序线性化，**最后一个生效者获胜**（last-write-wins）。

**只读请求**（`sieve.health` / `sieve.list_graylist` / `sieve.evaluate` 等）**不**经过此队列，可以与 mutating 请求并发处理，避免 read-only 请求被卡住。

**慢 client 防御**：fan-out 写入慢 client 的 socket buffer 时 **MUST** 设 bounded write timeout（推荐 2 秒）；超时**或**任何 write error（EPIPE / ECONNRESET / EBADF 等）**MUST** 视为该 client 失联，断开连接 + 清理其 sender，**不**阻塞当前请求的 result 路径。

> **理由**：发起方 client 收到 result 时，必然已经收到对应的 fan-out 通知，inflight id 集合中该 id 还在 → fan-out 被识别为回声忽略；result 处理路径再做一次本地状态更新。这样状态机不会出现"先收到 result 把 inflight id 移除，再收到 fan-out 误以为是外部变更而重复刷新"的竞态。
>
> 多 client 场景下其他 client 也按此顺序收到通知（origin_request_id 不在它们的 inflight 集合中），会正确同步状态。

#### 10.0.2 client 端 inflight id 生命周期（规范性）

> **作用域**：本节定义的 inflight id 集合**仅用于** §9.1–§9.3 等会触发 fan-out 的 client-originated mutating 控制面请求的回声去重。它**不是** `sieve.request_decision` 的 120 秒业务超时机制（后者完全在 daemon 侧由 oneshot channel + `default_on_timeout` 管理）。两者不要混淆。

client 端维护 `inflight_mutating_request_ids: Set<Uuid>`：

- **加入**：发送任何 mutating 控制面 request 前 **MUST** 把其 id 加入集合
- **移除**：在以下两种情况发生 **后** 移除：
  - 收到对应 result / error response **且** 已经处理了同 id 的 fan-out 通知（按 §10.0.1 的 daemon 顺序，fan-out 必然先到）
  - **或者** 该 request 超时 / 连接断开（明确无可能再收到 fan-out）
- **TTL 兜底**：单条 inflight id 在集合中 **MUST** 有 60 秒 TTL；超过后无论是否收到响应都自动移除（防御 daemon bug 导致的悬挂 id）

> **TTL 选值理由**：当前所有 mutating 控制面方法在 daemon 侧应在 ≤ 5 秒内完成；60 秒已留 12× 余量。未来若新增超过 60 秒的控制面方法，**MUST** 同步把 TTL 调整到 ≥ 该方法的最坏超时 + 安全余量。

**实现简化方案**：client 直接在 result handler 末尾移除 inflight id；按 §10.0.1 daemon 顺序保证，到这一步时同 id 的 fan-out 已经被路由处理完。

> **为什么不用 `source`**：v1 设计的 `source: "gui"` 只能区分通知类型，无法区分"哪个 client"；多 client 场景下 A 改 preset，B 也会按 `source==gui` 判定为回声并忽略，导致 B 状态不同步。`origin_request_id` 直接绑定到本 client 自己的 inflight id，对其他 client 来说该 id 自然不在集合内，会正确刷新。`source` 字段保留作为辅助信息（用于审计 / 日志）。

### 10.1 `sieve.preset_changed`

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.preset_changed",
  "params": {
    "mode": "strict",
    "overrides": {},
    "changed_at": "2026-05-02T15:03:11.234Z",
    "source": "gui",
    "origin_request_id": "8f3a2b91-7c4e-4d8f-9b21-1a3c5e7f9d02"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `mode` | enum (§5.6) | yes | — | no | |
| `overrides` | Object<rule_id → PresetOverride> | yes | — | no | 仅 `mode == "custom"` 时非空；其他 mode 必须为 `{}` |
| `changed_at` | `Timestamp` | yes | — | no | |
| `source` | enum | yes | — | no | `"cli"` / `"gui"` / `"config_reload"` —— 仅作为审计/日志辅助标签，不用于 client 回声判定 |
| `origin_request_id` | `Uuid` | yes | — | yes | 见 §10.0；`source != "gui"` 时**必须**为 `null` |

### 10.2 `sieve.paused_changed`

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.paused_changed",
  "params": {
    "paused": true,
    "paused_until": "2026-05-02T15:33:11.234Z",
    "reason": "user_request",
    "applies_to": ["status_bar", "auto_redact"],
    "source": "gui",
    "origin_request_id": "9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `paused` | bool | yes | — | no | |
| `paused_until` | `Timestamp` | no | `null` | yes | `paused=false` 时为 `null`（v2.0-r2 起字段名从 `until` 改为 `paused_until`） |
| `reason` | enum | yes | — | no | `"user_request"` / `"auto_resumed"` / `"daemon_restart"` |
| `applies_to` | String[] | yes | — | no | 见 §9.1，永远不包含 critical_lock 影响项 |
| `source` | enum | yes | — | no | `"cli"` / `"gui"` / `"config_reload"` / `"daemon"`（自动恢复 / 重启）—— 仅审计/日志用 |
| `origin_request_id` | `Uuid` | yes | — | yes | 见 §10.0；`source != "gui"` 时**必须**为 `null` |

---

## 11. 完整消息清单（速查）

> **命名澄清**：本表列出协议中所有具名 JSON-RPC 消息。`sieve.decision_response` **不是** JSON-RPC method（响应不带 method 字段），列在此处仅为方便对照"它是 `sieve.request_decision` 的响应"。其他所有行都是真正带 `method` 字段的 request 或 notification。

| 消息名 | 方向 | 类型 | 章节 |
|---|---|---|---|
| `sieve.hello` | daemon → client | notification | §3 |
| `sieve.heartbeat` | daemon → client | notification | §4 |
| `sieve.request_decision` | daemon → client | request | §6.1 |
| （`sieve.request_decision` 的 JSON-RPC response，无独立 method 字段） | client → daemon | response（同 id） | §6.2 |
| `sieve.request_decision_canceled` | daemon → client fan-out | notification | §6.3 |
| `sieve.notify_status_bar` | daemon → client fan-out | notification | §7 |
| `sieve.reload_user_rules` | CLI → daemon | notification | §8 |
| `sieve.set_paused` | client → daemon | request | §9.1 |
| `sieve.set_preset` | client → daemon | request | §9.2 |
| `sieve.set_preset_overrides` | client → daemon | request | §9.3 |
| `sieve.reload_config` | client → daemon | request | §9.4 |
| `sieve.health` | client → daemon | request | §9.5 |
| `sieve.evaluate` | client → daemon | request | §9.6 |
| `sieve.list_graylist` | client → daemon | request | §9.7 |
| `sieve.remove_graylist` | client → daemon | request | §9.8 |
| `sieve.preset_changed` | daemon → client fan-out | notification | §10.1 |
| `sieve.paused_changed` | daemon → client fan-out | notification | §10.2 |
| `sieve.list_rules` | client → daemon | request | §11A *(Since v2.0)* |
| `sieve.purge_history` | client → daemon | request | §11B *(Since v2.0)* |
| `sieve.judge_tool_call` | client → daemon | request | §11C *(Since v2.x)* |
| `sieve.list_pending` | client → daemon | request | §11D *(Since v2.x)* |
| `sieve.resolve_decision` | client → daemon | request | §11E *(Since v2.x)* |

> v1 旧方法名 `request_decision`（无 `sieve.` 前缀）在 v2 中已弃用。所有方法名必须以 `sieve.` 前缀。

---

## 11A. `sieve.list_rules` *(Since v2.0)*

> **兼容扩展**：本方法是 v2.0 协议的向前兼容新增，不递增 `protocol_version`。旧版本 daemon 会返回 `-32601 method_not_found`，client **MUST** 在此情况下降级（禁用规则总览 UI，不崩溃）。

client（如 GUI Settings 页面 Detection 规则总览 Table）使用本方法获取所有已加载的规则列表（系统规则 + 用户规则合并后的快照）。

### 请求（client → daemon）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.list_rules",
  "id": "5e7a1c3f-9b2d-4e8a-b6c1-3d5f7e9a1c2b",
  "params": {}
}
```

> `params` 对象可省略或为空对象 `{}`。本方法当前无参数；client **SHOULD** 始终发空 `params`（为未来扩展分页等参数预留）。

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| （无参数） | — | — | — | — | — |

### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "5e7a1c3f-9b2d-4e8a-b6c1-3d5f7e9a1c2b",
  "result": {
    "rules": [
      {
        "rule_id": "IN-CR-01",
        "title": "BIP39 助记词检测",
        "severity": "critical",
        "direction": "inbound",
        "disposition": "gui_popup",
        "default_on_timeout": "block",
        "timeout_seconds": 30,
        "critical_lock": true,
        "enabled": true,
        "rule_kind": "system",
        "description": "检测入站流量中出现的 BIP39 助记词序列（带 SHA-256 checksum 验证）"
      }
    ]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `rules` | `RuleSummary[]` | yes | — | no | 当前已加载的全部规则快照（系统规则 + 用户规则合并后），空规则集返回 `[]` |

#### `RuleSummary` 对象字段

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `rule_id` | String | yes | — | no | 规则唯一标识，如 `"IN-CR-01"`；用户规则使用用户自定义 ID |
| `title` | String | yes | — | no | 规则 UI 显示标题（中文或英文由 daemon locale 决定） |
| `severity` | enum | yes | — | no | `"low"` / `"medium"` / `"high"` / `"critical"` |
| `direction` | enum | yes | — | no | `"inbound"` / `"outbound"` |
| `disposition` | enum | yes | — | no | `"gui_popup"` / `"auto_redact"` / `"status_bar"` / `"hook_terminal"`（`gui_popup` wire 值含义见 §5.3 标注） |
| `default_on_timeout` | enum | no | `null` | yes | 仅 `disposition == "gui_popup"` 时有意义；`"block"` / `"allow"` / `"redact"`；其他 disposition 下**MUST**为 `null` |
| `timeout_seconds` | u32 | no | `null` | yes | 仅 `disposition == "gui_popup"` 时有意义；decision request 自动超时秒数；其他 disposition 下**MUST**为 `null` |
| `critical_lock` | bool | yes | — | no | `true` 表示 client 端禁止编辑此规则（Critical 级系统规则强制为 `true`） |
| `enabled` | bool | yes | — | no | 规则是否启用 |
| `rule_kind` | enum | yes | — | no | `"system"` / `"user"` |
| `description` | String | no | `null` | yes | 规则描述/备注，可能为 `null` |

### 行为描述

1. daemon 在握手完成后即可响应本请求（规则引擎初始化在 daemon 启动时完成）。
2. 返回的列表是**调用时刻的快照**（非实时订阅），client 如需刷新可重新调用。
3. 规则顺序：系统规则在前（按内置 severity 排序，critical 优先），用户规则在后（按加载顺序）。
4. `critical_lock == true` 的规则，client 端**MUST**在 UI 上禁用编辑/关闭控件，避免用户误操作。

### 错误码

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32006` | `rules_loading` | daemon 启动时规则引擎尚未完成初始化（极罕见，通常在 daemon 刚启动的几百毫秒内）；client **SHOULD** 延迟 1 秒后重试 |
| `-32601` | `method_not_found` | 旧版本 daemon 不支持本方法；client **MUST** 降级（禁用规则总览 UI） |

---

## 11B. `sieve.purge_history` *(Since v2.0)*

> **兼容扩展**：本方法是 v2.0 协议的向前兼容新增，不递增 `protocol_version`。旧版本 daemon 会返回 `-32601 method_not_found`，client **MUST** 在此情况下降级（禁用"清空历史"按钮，不崩溃）。

client（如 GUI Settings → Privacy & Data → "清空历史"按钮）触发（Touch ID 或等效二次确认通过后调用）。daemon 删除 audit.db 中所有 audit events 行数据，保留 schema 结构（不 DROP TABLE）。

### 请求（client → daemon）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.purge_history",
  "id": "7b9c2d4e-1f3a-4b8c-a5e2-6d8f1a3c5e7b",
  "params": {
    "confirmed_at": "2026-05-03T08:00:00.000Z"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `confirmed_at` | `Timestamp` | yes | — | no | client 端 Touch ID（或等效授权手势）通过的时刻（UTC）；用于审计日志，不作为幂等 key |

### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "7b9c2d4e-1f3a-4b8c-a5e2-6d8f1a3c5e7b",
  "result": {
    "purged_at": "2026-05-03T08:00:00.123Z",
    "rows_deleted": 4721
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `purged_at` | `Timestamp` | yes | — | no | daemon 实际执行删除完成的时刻（UTC） |
| `rows_deleted` | u64 | yes | — | no | 本次删除的 audit event 行数；`0` 表示历史本就为空，视为成功 |

### 行为描述

1. daemon **MUST** 使用互斥锁保证同一时刻只有一个 purge 在执行；并发调用时第二个请求立即返回 `-32007 purge_in_progress` 错误。
2. 删除范围：`audit_events` 表（或等效表）中**所有行**，使用 `DELETE FROM audit_events`（不 DROP TABLE / 不 VACUUM）。
3. daemon 在执行删除前和删除后各写一条审计记录：`purge_started`（含 `confirmed_at`）和 `purge_completed`（含 `rows_deleted`）；这两条记录本身**不**计入 `rows_deleted`。
4. 本操作**不可逆**，client 端**MUST**在调用前完成二次确认（Touch ID 或等效授权）。

### 错误码

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32007` | `purge_in_progress` | 另一个 purge 正在进行（去重保护）；client **SHOULD** 提示"正在清空，请稍候" |
| `-32601` | `method_not_found` | 旧版本 daemon 不支持本方法；client **MUST** 降级（禁用清空历史按钮） |

---

## 11C. `sieve.judge_tool_call` *(Since v2.x)*

> **兼容扩展**：本方法是 v2 协议的向前兼容新增，不递增 `protocol_version`。旧版本 daemon 会返回 `-32601 method_not_found`，client **MUST** 在此情况下 **fail-closed**（拒绝待判定的工具调用，不放行）。

client（agent 的 PreToolUse hook 等）把 agent **即将执行**的结构化工具调用喂给 daemon，由 daemon 跑入站规则引擎判危：命中 fail-closed Critical 规则时走 GUI 弹窗人工确认 + 审计，回 allow / deny 裁决。让不解析上游响应体的 client（如 hook）也能借 daemon 的规则引擎拿到入站危险工具拦截。

### 请求（client → daemon）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.judge_tool_call",
  "id": "9f1c3a5e-2b4d-4c6e-8a0f-1b3d5f7a9c1e",
  "params": {
    "tool_name": "exec_command",
    "tool_input": { "cmd": "rm -rf /important", "workdir": "/proj" },
    "tool_use_id": "call_abc123",
    "cwd": "/proj",
    "source_agent": "unknown",
    "timeout_ms": 47000
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `tool_name` | string | yes | — | no | 工具名（取值随 agent 而异，daemon 不假定特定值） |
| `tool_input` | object | yes | — | no | 工具输入对象；daemon 对其序列化后全文扫描判危（字段名无关） |
| `tool_use_id` | string | no | `""` | no | 关联上游 tool_use 的 ID |
| `cwd` | string | no | `""` | no | 工具执行工作目录（敏感路径判定用） |
| `source_agent` | string | no | `"unknown"` | no | 来源 agent（审计 / 规则上下文） |
| `timeout_ms` | u32 | no | `0` | no | client 愿等的最长毫秒数；daemon 据此 cap 弹窗 timeout（取 `min(规则 timeout, timeout_ms)`）；`0` = 用规则默认 |

### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "9f1c3a5e-2b4d-4c6e-8a0f-1b3d5f7a9c1e",
  "result": {
    "verdict": "deny",
    "rule_id": "IN-CR-02",
    "reason": "用户拒绝危险工具调用（IN-CR-02）"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `verdict` | string | yes | — | no | `"allow"`（放行）或 `"deny"`（拒绝） |
| `rule_id` | string | no | 省略 | no | 触发裁决的规则 ID；无命中时省略 |
| `reason` | string | no | 省略 | no | deny 原因；client 写进面向用户的提示 |

### 行为描述

1. daemon 用与真实入站 SSE 路径**同一个** `check_tool_use` 检测（扫 `tool_name` + 全文扫 `tool_input`），行为一致。
2. 无 fail-closed Critical 命中 → 立即 `verdict:"allow"`（v2.x 范围：非 Critical 工具检测放行，不为每次工具调用弹窗）。
3. 命中 fail-closed Critical → 走 `sieve.request_decision` GUI 弹窗确认 + 审计；用户 Allow → `allow`，Deny → `deny`。
4. **fail-closed**：引擎错误 / 无 GUI 响应 / 超时（对 Critical 按 `default_on_timeout=Block` 强制拒绝）→ `deny`。client 端到自身 deadline 仍 **MUST** 独立 fail-closed 兜底。
5. daemon 处理本方法可能阻塞等待用户在弹窗确认（数十秒），**MUST** 并发处理，不得串行化阻塞其他控制面请求。

### 错误码

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32601` | `method_not_found` | 旧版本 daemon 不支持本方法；client **MUST** fail-closed（拒绝待判定的工具调用） |

---

## 11D. `sieve.list_pending` *(Since v2.x)*

> **兼容扩展**：本方法是 v2.x 协议的向前兼容新增，不递增 `protocol_version`。旧版本 daemon 返回 `-32601 method_not_found`，client **MUST** 降级（禁用 headless 待决策枚举，不崩溃）。

headless client（`sieve decisions list` / `sieve decisions show`）用本方法**只读**枚举当前所有待决策快照。GUI 通过 `sieve.request_decision` push 实时收决策，无需本方法；本方法服务 headless 场景（发生弹窗时没挂 `watch` 也能事后发现 pending）。

### 请求（client → daemon）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.list_pending",
  "id": "b1f2c3d4-...",
  "params": {}
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| （无参数） | — | — | — | — | 过滤（severity / provider_id）在 client 侧做，daemon 保持薄 |

### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "b1f2c3d4-...",
  "result": {
    "pending": [
      {
        "request_id": "8f3a2b91-...",
        "max_severity": "critical",
        "detections": [
          { "rule_id": "IN-CR-05", "severity": "critical", "title": "签名工具调用", "one_line_summary": "检测到签名工具调用" }
        ],
        "timeout_seconds": 120,
        "default_on_timeout": "block",
        "direction": "inbound",
        "source_agent": "claude",
        "provider_id": "anthropic-main",
        "created_at": "2026-07-02T15:03:11.234Z",
        "age_seconds": 7
      }
    ]
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `pending` | `PendingSnapshot[]` | yes | — | no | 当前所有待决策的只读快照；无 pending 返回 `[]`（空 ≠ 错误） |

#### `PendingSnapshot` 对象字段

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 待决策请求 ID |
| `max_severity` | enum (§5.1) | yes | — | no | **daemon 侧从 detections 计算**的最高严重等级；`resolve_decision` 的 A 方案授权门禁据此判定，**不信 client 自报** |
| `detections` | Array | yes | — | no | 检测命中摘要（`rule_id` / `severity` / `title` / `one_line_summary`），裁剪版不含正则/offset |
| `timeout_seconds` | u32 | yes | — | no | 已钳制在 `[30,120]` |
| `default_on_timeout` | enum (§5.5) | yes | — | no | |
| `direction` | enum (§5.2) | yes | — | no | |
| `source_agent` | enum (§5.7) | yes | — | no | |
| `provider_id` | String | no | `null` | yes | listener 上游 provider_id；单 listener / 系统内部为 `null` |
| `created_at` | `Timestamp` | yes | — | no | daemon 发起本次决策请求的时刻 |
| `age_seconds` | u64 | yes | — | no | daemon 应答时按 `created_at` 现算的已等待秒数 |

### 行为描述

1. daemon 直接读 pending map 返回快照，快速无阻塞（不等 GUI）。
2. 返回是**调用时刻的快照**（非订阅）；client 需刷新则重新调用。
3. `max_severity` 由 daemon 在发起 `request_decision` 时从 detections 计算并存入 pending 条目，本方法只读取，**不接受 client 传入 severity**。

### 错误码

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32601` | `method_not_found` | 旧版本 daemon 不支持本方法；client **MUST** 降级 |

---

## 11E. `sieve.resolve_decision` *(Since v2.x)*

> **兼容扩展**：本方法是 v2.x 协议的向前兼容新增，不递增 `protocol_version`。旧版本 daemon 返回 `-32601 method_not_found`，client **MUST** 降级。

headless client（`sieve decisions resolve`）用本方法解决单个待决策。GUI 仍走 §6.2 的 raw JSON-RPC response 应答路径（GUI 回复它收到的 `request_decision`，天然 JSON-RPC 模式），**不使用本方法**；两条路径共存，daemon 侧殊途同归到同一 pending 的 responder。

### A 方案授权（规范性）

**headless CLI 对 `Critical` 类决策一律静默 deny；`High` 及以下允许 headless 批准。判定在 daemon 端按 pending 的 `max_severity` 做（daemon 侧计算，见 §11D），不信 CLI 自报。**

- 若 pending 的 `max_severity == critical` 且 `decision ∈ { allow, redact_and_allow }`：daemon **MUST** 静默改写为 `deny` 处置该 pending，`effective_decision` 返回 `"deny"`。**不回特殊错误、不提示 GUI 路径**（不向调用方暴露"存在 GUI 绕过路径"）。
- `High` 及以下：按传入 `decision` 处置，`effective_decision` 等于传入值。
- daemon 侧构造的 `DecisionResponse` 恒 `remember=false`（不给 CLI 开永久白名单）、`by_user=true`（headless 主动决策）。
- 审计照常记录（由原始 `request_decision` 调用点在 responder 收到决策后自动写 `DecisionMade`，含静默 deny，无需本方法特殊处理）。

> **理由**：不可逆动作前的认知摩擦要求人在场；同用户模型下，token/Keychain 等本地凭证无法可靠区分「合法 client」与「被注入的自动化 agent」（agent 与 daemon 同用户运行），故 headless 直接不开 `Critical` 批准口子。摩擦只在"批准需人在场"时对自动化对手有效。
>
> GUI wire 应答通道（§6.2）的对应防线是 peer 代码签名核验 gate（§6.2.4，F1-b）：CLI 侧靠
> A 方案、GUI 侧靠 peer 核验，两条路径合并覆盖。

### 请求（client → daemon）

```jsonc
{
  "jsonrpc": "2.0",
  "method": "sieve.resolve_decision",
  "id": "c9d0...",
  "params": {
    "request_id": "8f3a2b91-...",
    "decision": "allow",
    "context_hint": "headless 放行"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `request_id` | `Uuid` | yes | — | no | 目标待决策请求 ID |
| `decision` | enum (§5.4) | yes | — | no | `"allow"` / `"deny"` / `"redact_and_allow"`；Critical 类的 allow/redact_and_allow 会被静默改写为 deny |
| `context_hint` | String | no | `null` | yes | 决策理由（CLI `--reason`），透传写入 audit |

### 响应

```jsonc
{
  "jsonrpc": "2.0",
  "id": "c9d0...",
  "result": {
    "status": "resolved",
    "effective_decision": "deny"
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `status` | enum | yes | — | no | `"resolved"`（已处置）/ `"not_found"`（已超时 / 已被 GUI 解决 / id 不存在，三种情况天然合一） |
| `effective_decision` | enum (§5.4) | no | `null` | yes | 实际生效的决策；`status == not_found` 时缺失。A 方案下 Critical 类可能 ≠ 请求的 `decision` |

### 行为描述

1. daemon 从 pending map remove 目标条目；不存在 → `status: not_found`（`effective_decision` 缺失）。
2. 存在 → 按 A 方案门禁计算 `effective_decision`，构造 `DecisionResponse` 送入 pending 的 responder（让原始 `request_decision` 的 await 返回）。
3. 快速无阻塞（不等 GUI）；对同一 id 再次 resolve → `not_found`（幂等，已消费）。

### 错误码

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32601` | `method_not_found` | 旧版本 daemon 不支持本方法；client **MUST** 降级 |

---

## 12. 错误码体系

### 12.1 段位划分

| 段 | 用途 |
|---|---|
| `-32700 ~ -32600` | JSON-RPC 2.0 标准错误（双向） |
| `-32603` | JSON-RPC `internal_error`（双向） |
| `-32000 ~ -32099` | **daemon → client 业务错误**（保留段） |
| `-32100 ~ -32199` | **client → daemon 业务错误**（保留段） |

### 12.2 标准错误（双向，JSON-RPC 2.0）

| Code | 名称 | 含义 |
|---|---|---|
| `-32700` | `parse_error` | JSON 解析失败 |
| `-32600` | `invalid_request` | 请求格式不符 JSON-RPC 2.0 |
| `-32601` | `method_not_found` | 方法名未知 |
| `-32602` | `invalid_params` | 参数 schema 校验失败 |
| `-32603` | `internal_error` | 服务端内部错误 |

### 12.3 daemon → client 业务错误（`-32000 ~ -32099`）

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32000` | `protocol_version_mismatch` | client 发来的请求中暗示的协议版本 daemon 不支持（保留；当前 daemon 通过 `sieve.hello` 主动协商，不主动用此码） |
| `-32001` | `critical_lock_violated` | **保留，当前未启用**。`set_preset_overrides` 的 critical_lock 拒绝路径走 `result.rejected[]` partial success（见 §9.3），不使用此错误码。本码留待未来其他 critical_lock 强制拦截路径（如 `sieve.set_paused` 试图突破 critical_lock 影响范围等）启用 |
| `-32002` | `daemon_busy` | reload / restart 进行中，请求被排队失败 |
| `-32003` | `payload_too_large` | `sieve.evaluate` 的 `payload` 超过 64 KiB |
| `-32004` | `unknown_fingerprint` | `remove_graylist` 指定的 fingerprint 不存在；`list_graylist` 无此错误（不存在时返回空 `entries[]`） |
| `-32005` | `unsupported_in_paused` | 暂停态下不允许的操作（保留，当前未使用） |
| `-32006` | `rules_loading` | `list_rules`：daemon 启动时规则引擎尚未完成初始化；client SHOULD 延迟 1 秒后重试 *(Since v2.0)* |
| `-32007` | `purge_in_progress` | `purge_history`：另一个 purge 操作正在执行；client SHOULD 提示"正在清空，请稍候" *(Since v2.0)* |

### 12.4 client → daemon 业务错误（`-32100 ~ -32199`）

| Code | 名称 | 触发场景 |
|---|---|---|
| `-32100` | `user_canceled_via_window_close` | 用户关闭 decision request UI 未做决策 |
| `-32101` | `gui_render_failed` | client 端渲染异常（解析 request 失败、SwiftUI 抛错等） |
| `-32102` | `gui_shutdown_during_decision` | client 进程在决策过程中退出 |

> **v1 → v2 错误码迁移**：v1 中 `-32000 ~ -32002` 在 client → daemon 方向被占用为 `user_canceled_via_window_close` / `gui_render_failed` / `gui_shutdown_during_decision`，与 daemon → client 段位冲突。v2 起 client 错误码统一搬至 `-32100+`。

### 12.5 错误对象格式

```jsonc
{
  "jsonrpc": "2.0",
  "id": "<request id>",
  "error": {
    "code": -32003,
    "message": "payload_too_large",
    "data": { "limit": 65536, "actual": 81344 }
  }
}
```

| 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `code` | i32 | yes | — | no | 见 §12.2–12.4 |
| `message` | String | yes | — | no | 与 §12 中"名称"列字面量相同（snake_case）；client 端可基于此选择本地化文案 |
| `data` | any | no | `null` | yes | 错误上下文，结构由 `code` 决定；常见：`{"rule_id": "..."}`、`{"limit": 65536, "actual": 80000}` |

---

## 13. 协议变更与版本递增策略

> **面向外部贡献者**：要给 sieve 加 IPC 方法、改字段、新增错误码，先读完本节再动 `crates/sieve-ipc/src/protocol.rs`。流程（13.3）规定 SPEC PR 必须先于代码 PR 合入；§13.1 / §13.2 决定改动是否构成 breaking change（major bump，需 daemon + client 同步发布）。Schema 测试义务见 §14（daemon 端 fixture 是权威源，client 端必须对齐）。**任何破坏 §13 的 PR 默认 reject**，无论改动多小。

### 13.1 必须递增 `protocol_version`（major）

- 删除任意字段
- 修改字段语义（如 `decision` 增加新枚举值，或重命名枚举值）
- 修改方法名（含前缀）
- 修改握手时序
- 改变错误码段位划分

### 13.2 不递增 `protocol_version`（兼容扩展）

- 新增可选字段（client / daemon 双方必须忽略未知字段）
- 新增方法（client 端必须返回 `-32601 method_not_found` 而不是 crash；daemon 端同样）
- 新增 `error.code`（双方必须有 fallback "未知错误" 文案）
- 新增 enum 值时**必须谨慎**：client 端若有 exhaustive switch 必须先升级才允许 daemon 发新值；折中做法是在 SPEC 中先标"reserved for vN.x"，daemon 实际启用前各仓库都升级

#### v2.0 兼容扩展新增方法

以下方法为 v2.0 协议的向前兼容新增（`protocol_version` 保持 `"v2"` 不变）：

| 方法 | SPEC 章节 | 降级行为（旧 daemon 返回 `-32601`） |
|---|---|---|
| `sieve.list_rules` | §11A | client **MUST** 禁用规则总览 UI，不崩溃 |
| `sieve.purge_history` | §11B | client **MUST** 禁用"清空历史"按钮，不崩溃 |
| `sieve.judge_tool_call` | §11C | client **MUST** fail-closed（拒绝待判定的工具调用，不放行） |

**client 端实现要求**：对上表所有方法，client **MUST** 在发送前检测方法可用性（或通过捕获 `-32601` 错误进入降级模式），不能因 `method_not_found` 触发全局错误处理器。

### 13.3 PR 与发布协调流程

#### 13.3.1 SPEC 与代码 PR 顺序

1. **第一步：SPEC PR**（daemon 仓库）— 改 SPEC-005（如涉及决策性变更，同步记录决策依据）→ review → merge。此 PR 不含任何代码改动
2. **第二步：daemon 代码 PR** — 实现新 schema，更新 `crates/sieve-ipc/src/protocol.rs` + 测试 + fixture（见 §14）→ merge
3. **第三步：client 代码 PR**（如 GUI 仓库）— 更新 Codable structs + IPCRouter + 单元测试，更新 `docs/external/upstream-references.md` 的 SPEC-005 commit pin → merge
4. 三个 PR 在描述中互相引用对方 commit hash + SPEC-005 commit hash

#### 13.3.2 发布顺序（关键，避免 client 先发布却连不上旧 daemon）

合 PR 顺序 ≠ 用户安装顺序。**真正避免破坏的是发布顺序**：

| 阶段 | daemon 发布 | client（GUI）发布 | 兼容窗口语义 |
|---|---|---|---|
| T0 | v1.x（旧协议 v1） | v0.x（旧协议 v1） | 用户在用 |
| T1 | **先发** v2.0（实现新协议 v2，**同时** 拒绝 v1 客户端 with `sieve.hello { protocol_version: "v2" }`） | 还是 v0.x | **v0.x client 连 daemon v2.0 → 立即握手失败 → UI 引导用户升级 client**。daemon 不向下兼容（v2 是 major bump） |
| T2 | v2.0 | **再发** v1.0（实现新协议 v2） | 双方 v2，恢复正常 |

**MUST**：
- daemon v2 发布前 **MUST** 在 release notes 里写明"需要 GUI client v1.0+，旧 client 会进入 disconnected 状态"
- GUI client v1.0 发布前 **MUST** 在 release notes 里写明"需要 daemon v2.0+，旧 daemon 会握手失败"
- daemon v2 发布后 **MUST** 至少有一个 GUI client v1.0 RC 可供用户升级（避免用户升 daemon 后无 client 可用）

**不引入**多协议版本并存的兼容层。每次 major bump 都按上面流程，daemon 一刀切。理由：双 client / 双 daemon 协议矩阵在测试与 audit schema 上是噩梦，"快速强制升级"成本远小于"长期维护多版本"。

#### 13.3.3 Hot-fix 流程

minor / patch（向前兼容的字段新增等）允许跳过 SPEC-only PR，把 SPEC 改动与代码改动放同一 PR。但仍需更新 `upstream-references.md` commit pin。

---

## 14. Schema 一致性测试约定

为防止两个仓库再次发生 schema drift，本 SPEC 配套以下测试义务（在两端代码 PR 中实现）：

### 14.1 daemon 端 fixture（权威源）

- daemon 仓库 `crates/sieve-ipc/tests/fixtures/v2/` 目录下，每个 message kind 至少包含三条 JSON fixture（命名规则见 §14.3.1）：`<message_kind>__minimal.json` / `<message_kind>__full.json` / `<message_kind>__null_optional.json`；feature 专用 fixture（如 `request_decision__merged.json`）按需追加
- 测试文件 `crates/sieve-ipc/tests/schema_v2_fixtures.rs`：
  - 用 `serde_json::from_str` 解析每条 fixture 为对应 struct，断言字段全集
  - 反向 `serde_json::to_value` 序列化后与 fixture 原文（`canonicalize`后）比较，确保**双向稳定**（fixture 是 daemon 序列化 输出的事实，不是手写产物）
- CI 强制 `cargo test -p sieve-ipc --test schema_v2_fixtures` 通过

### 14.2 client 端 fixture 消费

- GUI 仓库 `Tests/SieveGUICoreTests/Fixtures/v2/` 是 daemon fixture 的 **副本**，由发布工具拷贝，**不**手动修改
- 测试文件 `Tests/SieveGUICoreTests/IPCSchemaV2FixtureTests.swift`：用 `JSONDecoder` 解每条 fixture 为对应 Codable struct，断言关键字段
- CI 强制 `swift test --filter IPCSchemaV2FixtureTests` 通过

### 14.3 fixture 同步机制（规范性）

**采用 daemon 发布 release artifact，client 在代码 PR 中按 commit pin 拉取**——不使用 git submodule（避免 client 仓库每次 daemon commit 都触发提交）。

#### 14.3.1 fixture 文件命名（强约束）

daemon 仓库 `crates/sieve-ipc/tests/fixtures/v2/` 目录下文件命名 **MUST** 遵循：

```
<message_kind>__<scenario>.json
```

- `message_kind`：方法名去掉 `sieve.` 前缀，下划线保留（如 `request_decision` / `notify_status_bar` / `set_preset_overrides`）；对 `decision_response` 这类伪方法名也直接用其名字
- `scenario`：场景标识（**不带前导下划线**），全集如下
  - `minimal` — 该方法所有 required 字段的最小有效负载（所有 optional 字段省略）
  - `full` — 满字段（所有 optional 字段都给值）
  - `null_optional` — 所有支持 `null accepted: yes` 的字段显式为 `null`
  - `<feature>` — 特性专用 fixture（如 `request_decision__merged.json`、`set_preset_overrides__critical_lock_rejected.json`、`notify_status_bar__sequence_hit.json`）

> 双下划线 `__` 是 `message_kind` 与 `scenario` 之间的**唯一分隔符**；方法名内部的单下划线（`set_preset_overrides`）不会与之冲突。

每个方法 **MUST** 至少包含 `minimal` + `full` + `null_optional` 三种 fixture（共 17 方法 × 3 = 51 条最低门槛；feature 专用 fixture 是加项）。

#### 14.3.2 manifest.json（强约束）

artifact 包根目录 **MUST** 包含一份 `manifest.json`：

```jsonc
{
  "manifest_version": 1,
  "protocol_version": "v2",
  "spec_commit": "8a3b9c1d4e7f0a2b5c8d1e3f6a9b2c5d8e1f4a7b",
  "spec_path": "docs/specs/SPEC-005-ipc-protocol.md",
  "daemon_commit": "f1e2d3c4b5a69788c0d1e2f3a4b5c6d7e8f9a0b1",
  "daemon_version": "0.7.2",
  "generated_at": "2026-05-02T15:03:11.234Z",
  "files": [
    {
      "path": "request_decision__minimal.json",
      "sha256": "9f8e7d6c5b4a32101a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f7081",
      "kind": "request",
      "method": "sieve.request_decision",
      "scenario": "minimal",
      "size_bytes": 412
    }
    /* ... 其他 fixture ... */
  ]
}
```

| manifest 字段 | 类型 | required | default if absent | null accepted | 说明 |
|---|---|---|---|---|---|
| `manifest_version` | u32 | yes | — | no | 当前 `1`；manifest 自身格式变更时递增 |
| `protocol_version` | String | yes | — | no | 与 SPEC-005 §2 一致，当前 `"v2"` |
| `spec_commit` | String | yes | — | no | SPEC-005 commit hash（git rev-parse HEAD on file） |
| `spec_path` | String | yes | — | no | SPEC 文件相对 daemon 仓库根的路径 |
| `daemon_commit` | String | yes | — | no | daemon 代码 commit hash |
| `daemon_version` | String | yes | — | no | daemon 二进制 semver |
| `generated_at` | `Timestamp` | yes | — | no | manifest 生成时间 |
| `files` | Array | yes | — | no | 所有 fixture 元信息列表 |
| `files[].path` | String | yes | — | no | 相对 artifact 根的路径 |
| `files[].sha256` | String | yes | — | no | 64 字符 hex lowercase |
| `files[].kind` | enum | yes | — | no | `"request"` / `"response"` / `"notification"` |
| `files[].method` | String | yes | — | no | wire 方法名（response 用所对应 request 的方法名） |
| `files[].scenario` | String | yes | — | no | `"minimal"` / `"full"` / `"null_optional"` / 自定义 |
| `files[].size_bytes` | u64 | yes | — | no | |

#### 14.3.3 daemon 端发布

daemon CI 在每次合 SPEC + 代码 PR 并打 release tag 时：
1. 跑 `cargo test -p sieve-ipc --test schema_v2_fixtures` 验证所有 fixture
2. 生成 `manifest.json`（含 spec_commit / daemon_commit / 所有 fixture 的 sha256）
3. 打包 `tar -caf sieve-ipc-fixtures-v2.tar.zst manifest.json *.json`（**固定文件名**，不带 commit hash 后缀，避免 client CI 需要先查 commit 再拼 URL）
4. 同步生成 `sieve-ipc-fixtures-v2.tar.zst.sha256`
5. 上传到 GitHub Release：URL 模板 `https://github.com/SieveAI-dev/sieve/releases/download/<release_tag>/sieve-ipc-fixtures-v2.tar.zst`（其中 `<release_tag>` 是 daemon 的 git tag，如 `v0.7.2`）
6. release notes 中列出 `spec_commit` / `daemon_commit` 关联（仅供人类阅读；client CI **不**依赖 release notes 内容做 discovery）

> **不做 commit-hash-suffix 命名**：旧方案 `sieve-ipc-fixtures-v2-<sha>.tar.zst` 要求 client CI 先查 daemon release 找包含某 spec_commit 的 release，scrape release notes 才能拿到精确 sha 后缀。改为固定命名 + release_tag 后，client CI 可直接用 URL 模板拼接，不依赖任何 release notes 内容。`spec_commit` 校验通过解压后的 `manifest.json` 完成。

#### 14.3.4 client 端同步

GUI 仓库（及其他 client 仓库）根目录 `scripts/sync-ipc-fixtures.sh`：
1. 读 `docs/external/upstream-references.md` 中 SPEC-005 的 `<pinned commit>` + 关联的 `<daemon_release_tag>`（两个字段都 pin，禁止只 pin commit）
2. 直接构造 artifact URL：`https://github.com/SieveAI-dev/sieve/releases/download/<daemon_release_tag>/sieve-ipc-fixtures-v2.tar.zst`（**固定 asset 命名**，不带 commit hash 后缀；同一 release 只有一份 fixture）
3. 同步下载 `sieve-ipc-fixtures-v2.tar.zst.sha256`
4. 校验 sha256 匹配；失败 abort
5. 解压到 `Tests/SieveGUICoreTests/Fixtures/v2/`
6. 校验解压出的 `manifest.json` 中 `spec_commit` 与 `upstream-references.md` pin 的 commit 一致；不一致 abort（防止 release 错配）

> **禁止 scrape release notes**：CI 不应通过解析 release notes 文本去找 fixture artifact；必须通过固定 URL 模板直接拼接。GitHub Release 的 asset 命名稳定性由 daemon CI 保证（见 §14.3.3）。

client 端代码 PR **MUST** 在同一 commit 中包含：(a) 更新 `upstream-references.md` 的 commit pin；(b) 跑 `sync-ipc-fixtures.sh` 后的 fixture 文件变更（含 manifest.json）。`Tests/SieveGUICoreTests/Fixtures/v2/` 目录由 git 跟踪，任何手改在 PR review 中都会暴露。

#### 14.3.5 安全与完整性

- `*.sha256` 文件防御传输错误与基础篡改
- **release tag 不可覆盖**：daemon CI **MUST NOT** 重复发布同一 `<release_tag>` 下的 `sieve-ipc-fixtures-v2.tar.zst`；如需修订必须发新 release tag。重复发布同 tag 时 CI **MUST** fail，除非新旧 sha256 完全相同（幂等重发）
- 长期 TODO：把 fixture artifact 也纳入 sigstore 签名（与 daemon 二进制同流程），抵御 release 仓库泄露的高级威胁

#### 14.3.6 fixture 权威性

所有 fixture 的"权威版本"在 daemon 仓库 `crates/sieve-ipc/tests/fixtures/v2/`；GUI 仓库 `Tests/SieveGUICoreTests/Fixtures/v2/` 只消费、不生产。client 端发现 fixture 有问题 → 提 issue 到 daemon 仓库改源头，**禁止**只在 client 端改副本。

### 14.4 兼容窗口的 fixture 测试

按 §13.3.2，daemon v2 不向下兼容 v1。fixture 测试只覆盖当前活动版本（v2），不保留 v1 fixture。每次 major bump 时旧 fixture 目录整体迁移到 `archive/`，新建 `fixtures/v<N>/`。

---

## 15. 关联文档

- 项目入口：[../../README.md](../../README.md)
- API 总览：[../api/api-reference.md §6](../api/api-reference.md#6-cli-退出码--decision-request-确认协议)（本 SPEC 取代）
- 架构：[../design/architecture.md](../design/architecture.md)
- SPEC-001 sieve-hook 协议：[SPEC-001-sieve-hook-protocol.md](SPEC-001-sieve-hook-protocol.md)
- SPEC-002 HIPS decision request UI 行为：[SPEC-002-hips-popup-behavior.md](SPEC-002-hips-popup-behavior.md)
- 数据模型：[../design/data-model.md](../design/data-model.md)
- GUI 仓库 ipc-protocol（client 实现注解）：`sieve-gui-macos/docs/api/ipc-protocol.md`
- GUI 仓库上游引用：`sieve-gui-macos/docs/external/upstream-references.md`

---

## 16. 变更记录

| 版本 | 日期 | 作者 | 变更 |
|---|---|---|---|
| v2.0-draft | 2026-05-02（早） | SieveAI | 首次起草。统一 daemon 与 GUI 双端协议，bump 至 protocol_version v2，落锤 D1–D8 决策。取代 daemon `api-reference.md §6` 旧版本与 GUI `ipc-protocol.md` v1.0 的 schema 部分。 |
| v2.0-r1 | 2026-05-02（午后） | SieveAI | 经评审反馈。**P0 修复**：(1) §1.3 加入帧大小上限的 bounded frame reader 算法（MUST 1MiB 即关、禁记录 raw payload）；(2) §9.3 `set_preset_overrides` critical_lock 拒绝路径**只**走 partial success `result.rejected[]`，`-32001` 标为 reserved 不用于此方法；(3) §10.0 新增多 GUI 回声防护机制 `origin_request_id`（取代依赖 `source: "gui"` 的歧义判定），`preset_changed` / `paused_changed` 都加此字段。**P1 修复**：(1) §6.1 字段表统一为 6 列含 `required / default if absent / null accepted`；(2) §6.0 新增 wire DTO 与 daemon 内部结构的映射小节，明确 hook pending file 路径独立；(3) §6.1 加"Allow Remember 四道防线"表格；(4) §1.3 + 文档统一 `context_hint` 200 个 Unicode scalar，GUI MUST 拒绝超限提交、daemon MUST 返 -32602；(5) §9.8 `remove_graylist` 不存在时 MUST 返 -32004；(6) §4A 新增"标量格式约定"全局章节（Timestamp / UnixMs / Uuid 规范）；(7) §13.3 重写为发布顺序（不是 merge 顺序），§14.3 新增 fixture artifact + sha256 同步机制。**P2 修复**：(1) §3.2 章节引用 §6.6 → §5.6；§9.2 引用 §11.1 → §10.1；(2) §11 标题"方法名清单"→"消息清单"，区分 method 与 response；(3) §5.5 明确严格度排序 `block > redact > allow`。 |
| v2.0-r2 | 2026-05-02（晚） | SieveAI | 经第二轮评审反馈。**P1 修复**：(1) §1.3.1 帧接收算法重写为 `frame_buf` 保留 remainder + 循环扫描，并补 MUST 约束（每次 append 后查上限、不假设单帧、parse 失败不关闭连接、禁日志原文）；(2) §10.0.1 新增 daemon fan-out 顺序（先广播再返 result），§10.0.2 新增 GUI inflight id 生命周期（含 60s TTL 兜底）；§9.1 `set_paused` 加 fan-out 强制条款（含恢复路径）；(3) §4A 新增全局硬规则：所有 wire schema 表 MUST 使用 `Timestamp` / `UnixMs` / `Uuid` 命名类型，禁止裸 `String (ISO 8601)` / `String (UUID)` / `i64` 局部描述；澄清 `paused: bool` 与 `paused_until: Timestamp` 的字段名歧义；批量替换 §3.2 / §6.2 / §6.3 / §7 / §8 / §9.1 / §9.2 / §9.3 / §9.4 / §9.5 / §9.6 / §9.7 / §9.8 / §10.1 / §10.2 / §12.5 所有字段表为 6 列规范并改用命名类型；(4) §6.1.2 issue 对象表升级；(5) §9.5 拆出 9.5.3 RulesSnapshot 子节；(6) §14.3 重写为带强约束的 fixture 命名（`<message_kind>__<scenario>.json`）+ 强制 manifest.json schema（含 protocol_version / spec_commit / daemon_commit / files[].sha256 等）+ 详细发布与同步流程；(7) §3.2 hello params 加 `daemon_boot_id` 字段，让 GUI 区分"daemon 重启"vs"仅断连"；§3.4 重连处理用此字段选择 toast 文案。**P2 修复**：(1) §3.3 加协议版本不匹配的最小 GUI 文案约定（含可点击操作 + 禁用写入操作）；(2) §9.8 删除"幂等并发删除"措辞，改为明确"未来若引入 idempotent 模式才允许 result false"；(3) 全文示例 JSON 中所有占位 UUID 替换为完整合法 UUID（避免 fixture 作者误抄）。 |
| v2.0-r3 | 2026-05-02（夜） | SieveAI | 经第三轮评审反馈。**P1 修复**：(1) §1.3.1 帧接收 oversize 判断顺序重写——先循环消费完整帧（按单帧 `idx+1 > 1MiB` 判超限），再判 remainder 是否超 1 MiB；修复"接近 1MiB 合法帧 + 下一帧前缀"组合粘包被误杀的边界 bug；EOF 后显式 return；(2) §9.2 `set_preset` response 补 `applied_at` 字段表。**P2 修复**：(1) §14.3 fixture scenario 命名去掉前导下划线（`minimal` 而非 `_minimal`），双下划线只作 `<message_kind>__<scenario>` 唯一分隔；(2) §3.4 加首次连接特例（无 cached boot_id 时不弹任何 toast）；(3) §10.0.2 inflight id 集合作用域明确为 mutating 控制面回声去重，与 `request_decision` 120s 业务超时区分；TTL 选值理由说明；(4) §10.0.1 加全局串行化 MUST + 慢 GUI bounded write timeout 约束。**全局遗漏修复**：(1) §6.1.1 标题"4 列规范"→"6 列规范"并补"字段"+"说明"两列定义；(2) `OriginHop` 补 6 列字段表（`agent` / `action` / `timestamp: Timestamp`）；(3) §9.5.1–§9.5.4 完整化所有 nested DTO 字段表（PresetSnapshot / AuditDbSnapshot / ListenSnapshot / GraylistSnapshot / IpcSnapshot）；(4) §4A Timestamp 清单移除已废弃的 `until`，加入 `generated_at` / `OriginHop.timestamp`；(5) §14.3.3 改为固定 asset 命名 `sieve-ipc-fixtures-v2.tar.zst` + release_tag 拼接，§14.3.4 GUI CI 直接拼 URL 不再 scrape release notes。 |
| v2.0-r4 | 2026-05-02（深夜） | SieveAI | 经第四轮评审反馈。无 P0/P1，全部 P2 + 全局遗漏：(1) §14.1 fixture 命名同步到 §14.3.1 规范（三种 minimal/full/null_optional 强制门槛）；(2) §10.0.1 加无 fan-out 的 mutating 请求（如 `reload_config`）的串行化要求 + 只读请求并发例外 + write error/timeout 都视为失联；(3) §9.5 nested DTO 子节顺序重排为 9.5.1/9.5.2/9.5.3/9.5.4 自然顺序；§9.5 顶层表中 `listen` / `graylist` / `ipc` 引用从 inline 改为"见 §9.5.4"；(4) §1.3 总表 oversize 描述精确化（区分单 frame 超限 vs partial frame 自身超限）；(5) §14.3.5 加 release tag 不可覆盖 MUST（除非 sha256 幂等）；(6) §14.3.2 manifest 示例的 sha256 / commit 占位换为完整 64 hex / 40 hex 字面量。 |
| v2.0-r5 | 2026-05-02（深夜，定稿） | SieveAI | 经第五轮评审反馈，全部 CLOSED，仅一个 P3 格式细节：§9.5 health 顶层表 `preset` 行 `见 9.5.1` 补全为 `见 §9.5.1`。**SPEC 状态从 Draft 升级到 Frozen**，可以进入双仓库代码改造阶段。 |
| v2.0-neutralize | 2026-05-05 | SieveAI | **术语中性化**。纯文档清洗，不改 wire schema、不 bump 协议版本，旧 client 完全兼容。变更：(1) §0 文档定位重写——明确 daemon IPC 协议不感知 client 形态（GUI / CLI / TUI / webhook 协议层地位平等）；(2) 文档头加"协议变更日志"段落（2026-05-05 协议中性化）；(3) 全文段落术语清洗：「GUI 端」→「client 端」；「GUI 连接」→「client 连接」；「daemon → GUI」→「daemon → client」等方向标注；`gui_writers[0]` → `client_writers[0]`；§9 章节标题「GUI 控制面方法」→「控制面方法」；§10.0 「多 GUI 回声防护」→「多 client 回声防护」；§10.0.2 「GUI 端 inflight id」→「client 端 inflight id」；§14.2/14.3.4 「GUI 端 fixture 消费」→「client 端 fixture 消费」；(4) §5.3 `"gui_popup"` disposition 枚举值加兼容性标注注释（保留 wire 值不变，语义说明「不绑定 GUI 显示」）；(5) §5.10 `ui_phase` 加 admonition（GUI 实现细节，headless client 答复时此字段填 null）；(6) §3.3 「GUI 端期望行为」加 admonition 标注（GUI 实现细节，非协议契约）；(7) §6.1.4 recommendation「GUI 默认按钮规则」加 admonition 标注（GUI 实现细节）；(8) §3.4 UI 文案选择段落明确「以下为 GUI client 参考实现，非协议约束，headless client 可忽略」。|
| v2.0-listeners | 2026-05-05 | SieveAI | **多 listener doc-sync**（doc-debt 修复，代码 ship 于 commit d90c51b，文档同步滞后到 2026-05-07）。向后兼容扩展，不改既有 wire schema、不 bump 协议版本。变更：(1) 文档头「协议变更日志」加 2026-05-05 多 listener 条目；(2) §9.5 health 顶层字段表新增 `listeners: ListenerSnapshot[]`（`yes (since v2.x)` / 默认 `[]`）；旧 `listen: ListenSnapshot` 行加 **Deprecated since v2.x** 标注，等价于 `listeners[0]`；(3) §9.5 example response 加 `listeners` 数组示例；(4) §9.5.4 标题扩到「ListenSnapshot / ListenerSnapshot / GraylistSnapshot / IpcSnapshot」，jsonc 块加 ListenerSnapshot 示例 + deprecated 标注，DTO 表内追加 ListenerSnapshot 4 行字段（addr / port / provider_id / protocol）；(5) §9.5.4 末尾加「ListenSnapshot vs ListenerSnapshot 版本兼容矩阵」三档说明（新×新 / 新×旧 / 旧×新）。**daemon 代码权威源**：`crates/sieve-ipc/src/protocol/health.rs:43-110`，`#[serde(default)]` 保证旧 daemon 不发本字段时新 client 拿到空数组不崩。 |
