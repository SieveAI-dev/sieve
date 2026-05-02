# SPEC-005 r1 Confirmation Review

## 总评
r1 闭环度约 **75%**。三个 r0 P0 的方向都修了，但 `origin_request_id` 的 fan-out 顺序 / inflight 生命周期仍不够可执行，§1.3 新增帧接收算法还引入了丢后续帧的问题。结论：**不建议直接进入代码改造阶段**；先做一个很小的 r2 文档修订，至少修完下方 P1。

## r0 问题闭环情况

### P0

| # | r0 问题摘要 | r1 状态 | 备注 |
|---|---|---|---|
| 1 | 帧大小上限缺少安全接收算法 | PARTIALLY CLOSED | §1.3/§1.3.1 已规定 1 MiB bounded reader、超限即关、禁止 raw payload 日志。但 §1.3.1 第 3 步写“解析完毕后清空 `frame_buf`”，如果一次 read 读到多帧，会丢弃 newline 后面的后续帧。 |
| 2 | `set_preset_overrides` critical_lock 返回语义与错误码冲突 | CLOSED | §9.3 明确 critical_lock 只走 `result.rejected[]` partial success；§12.3 将 `-32001` 标成保留、当前未启用。 |
| 3 | fan-out `source` 防回声不支持多 GUI | PARTIALLY CLOSED | §10.0 加了 `origin_request_id`，§10.1/§10.2 都补了字段，解决了“哪个 GUI”的标识问题。但未规定 fan-out 与 result response 顺序，也未规定 GUI inflight id 何时移除；`sieve.set_paused` 成功后也没有像 §9.2/§9.3 那样明确 MUST fan-out `paused_changed`。 |

### P1

| # | r0 问题摘要 | r1 状态 | 备注 |
|---|---|---|---|
| 1 | optional / default / null 语义不清 | CLOSED | §6.1.1 主表改为 `required / default if absent / null accepted` 六列，并定义缺失与显式 `null` 规则。 |
| 2 | wire DTO 与 daemon 内部 `DecisionRequest` 迁移边界不清 | CLOSED | §6.0 明确 SPEC 只约束 wire DTO，列出 `created_at → received_at_daemon`、`detections[] → single/merged issue`，并声明 hook pending file 不受 SPEC-005 约束。 |
| 3 | `allow_remember` 缺少 UI MUST | CLOSED | §6.1 加“四道防线”，GUI UI 禁用、编码层强制、daemon 二次校验都有 MUST。 |
| 4 | `context_hint` 计数单位和超限处置不清 | CLOSED | §1.3 统一为 200 个 Unicode scalar，GUI MUST 拒绝提交，daemon MUST 返回 `-32602`，不静默截断。 |
| 5 | `remove_graylist` 同时允许 result false 和 error | CLOSED | §9.8 明确不存在 MUST 返回 `-32004 unknown_fingerprint`，`removed=false` 当前协议不会出现。 |
| 6 | Timestamp / UUID 规范不完整 | PARTIALLY CLOSED | §4A 增加了 `Timestamp` / `UnixMs` / `Uuid`，但大量字段表仍写 `String (ISO 8601)` / `String (UUID)`，`OriginHop.timestamp` 未纳入清单，且 §4A 把 `paused` 列为 Timestamp 会和多个 bool `paused` 字段冲突。 |
| 7 | 升级与 fixture 同步流程不可操作 | PARTIALLY CLOSED | §13.3 已改成发布顺序，§14.3 已选择 release artifact + sha256。但 fixture manifest、文件命名全集、SPEC commit pin 到 daemon release tag 的机器可读映射仍不够具体。 |

### P2

| # | r0 问题摘要 | r1 状态 | 备注 |
|---|---|---|---|
| 1 | 章节引用错误 | CLOSED | `preset` 已指向 §5.6；`set_preset` fan-out 已指向 §10.1。 |
| 2 | §11 标题“方法名清单”包含 response | CLOSED | §11 已改为“完整消息清单”，并声明 `sieve.decision_response` 不是 method。 |
| 3 | “最严格”未定义排序 | CLOSED | §5.5 明确 `block > redact > allow`，§6.1.2 引用该排序。 |

### 代码侧差异
r0 中 daemon / Swift 代码差异仍是 **DEFERRED**，按你的范围说明不在本次 r1 confirmation review 里重复展开；它们应在后续代码 PR 逐项关闭。

## r1 引入的新问题

### P0（新发现的阻断）
无新增 P0。

### P1（新发现应改）

**1. 帧接收算法会丢弃同一次 read 里的后续帧**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:61-66`  
问题：Unix stream 不保证一次 read 只含一帧。当前“解析完毕后清空 `frame_buf`”会把 `newline_idx + 1` 后的完整或部分下一帧丢掉。  
修改建议：改为“消费 `[..=newline_idx]` 后保留 remainder，并循环扫描直到 buffer 中没有 newline；每次追加后都先做总长度上限检查”。

**2. `origin_request_id` 防回声规则缺少顺序与生命周期约束**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:1096-1100`, `731`, `790`  
问题：如果 daemon 先发 notification 再发 result，GUI 可忽略自身回声；如果先发 result，GUI 可能已移除 inflight id，随后把自身 fan-out 当外部变更重复处理。  
修改建议：二选一写死：daemon MUST 先 fan-out 再返回 result；或 GUI MUST 将完成的 request id 保留到收到同 id fan-out 或短 TTL 过期。另需明确 `set_paused` 成功后 MUST fan-out `paused_changed`，并带本次 request id。

**3. §4A 标量规则没有真正全局落表**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:188`, `493-531`, `609-612`, `701`, `819`, `875`, `878`, `968`  
问题：全局定义了 `Timestamp` / `Uuid` / `UnixMs`，但局部表仍大量使用 `String (ISO 8601)`、`String (UUID)`、`i64`。`paused` 同名字段既有 bool 又有时间戳，§4A 的“用于 `paused`”会误导实现。  
修改建议：所有相关表统一改成 `Timestamp` / `Uuid` / `UnixMs`；把 health 里的时间字段改名或描述为 `paused_until` 语义；补上 `OriginHop.timestamp: Timestamp`。

**4. 六列表规范只覆盖了 §6.1.1，其他 schema 表仍旧格式**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:343-346`, `387-395`, `493-531`, `609-617`, `680-1066`, `1120-1152`  
问题：r1 写“字段表 4 列规范”但表本身是 6 列；merged issue、decision_response、notify_status_bar、§9 控制面、§10 通知仍没有 `default if absent / null accepted`。  
修改建议：把该规范提升到全局“字段表规范”，并至少升级所有 wire schema 表；非 wire 的说明表可明确标成“非 schema 表”。

**5. fixture 同步机制缺少机器可执行 manifest**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:1303`, `1319-1334`  
问题：已有目录和 artifact 名，但缺少 fixture 文件全集命名、manifest、每文件 sha256、SPEC commit 与 daemon release tag 的绑定格式。双仓库仍可能各自实现不同同步脚本。  
修改建议：规定 artifact 内必须含 `manifest.json`，字段至少包括 `protocol_version`、`spec_commit`、`daemon_commit`、`daemon_version`、`files[{path, sha256, kind, message}]`；同时列出固定命名规则。

### P2（建议）

**1. `remove_graylist` 尾注重新引入弱歧义**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:1065-1082`  
问题：正文说当前协议不会出现 `removed=false`，但尾注又说 `removed:false` 路径保留给并发删除兼容场景。  
修改建议：删除尾注里的当前场景描述，只保留“未来如新增 `params.idempotent` 才允许 result false”。

**2. 重连 toast 把所有重连都描述成 daemon 重启**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:150-159`  
问题：§3.4 覆盖 daemon 重启、网络中断、主动断开，但 UI 文案固定为“daemon 重启”。  
修改建议：hello 增加 `daemon_boot_id` 或 `started_at`，让 GUI 能区分重启；否则文案改成“连接已恢复，本次决策已由系统兜底”。

**3. T1 阶段缺少协议不匹配的最小 GUI 文案约定**  
位置：`sieve/docs/specs/SPEC-005-ipc-protocol.md:1274-1289`  
问题：发布流程依赖旧 GUI 能引导升级，但 SPEC 没给最小可测试文案或行为。  
修改建议：补一段：GUI 收到不支持的 `protocol_version` MUST 显示“需要升级 Sieve GUI 至 v1.0+ / daemon 至 v2.0+”并提供 release 链接或设置入口。

## 全局遗漏
- P0：无。
- P1：所有 wire schema 表应统一使用同一套 required/default/null 规则；否则 fixture 很难覆盖显式 `null` 与缺失字段差异。
- P1：全局标量类型已经定义，但还没有形成“局部字段表不得再写裸 `String (ISO 8601)` / `String (UUID)`”的硬规则。
- P2：示例里的 `"8f3a2b91-..."`、`"9c1d8b73-..."` 与 §4A 的 lowercase hyphenated 36 字符 UUID 要求不一致；建议示例统一使用完整合法 UUID，避免 fixture 作者照抄占位值。

## 结论
**不建议现在进入代码改造阶段。** 最少先修：§1.3 frame remainder、§10.0 fan-out 顺序和 inflight 生命周期、§4A/字段表全局一致性、§14 fixture manifest。修完这些后，SPEC 层基本可以进入 daemon/GUI 代码 PR。

