# SPEC-005 r2 Confirmation Review

## 总评
r2 闭环度约 **85%**。r1 的主要方向都已落文档，尤其是 fan-out 顺序、inflight 生命周期、`daemon_boot_id`、manifest、命名标量类型和大部分 6 列 schema 表。但还不建议直接进入代码改造阶段：§1.3.1 新帧算法仍有一个合法粘包被误判 oversize 的边界 bug，另外 §9.2 response、`OriginHop`、health nested DTO 等 schema 仍不够机器化。建议先做一个很小的 r3 文档修订。

## r1 问题闭环情况
| # | r1 问题摘要 | r2 状态 | 备注 |
|---|---|---|---|
| P1-1 | 帧接收算法丢同 read 后续帧 | CLOSED，但引入新 P1 | §1.3.1 已保留 remainder 并循环扫描 newline，原丢帧问题闭合；但“append 后先按总 buffer > 1 MiB 关闭”会误杀“刚好 1 MiB 帧 + 下一帧前缀”的合法粘包。 |
| P1-2 | `origin_request_id` 缺 fan-out/result 顺序和 GUI inflight 生命周期 | CLOSED | §10.0.1 规定 daemon 先 fan-out 再 result；§10.0.2 规定 inflight id 加入/移除/60s TTL；§9.1 补了 `set_paused` 必须 fan-out。 |
| P1-3 | §4A 标量规则未全局落表 | PARTIALLY CLOSED | §4A 加了硬规则，字段表大多改成 `Timestamp` / `UnixMs` / `Uuid`。遗漏：`OriginHop.timestamp` 只有 JSON 示例和 §4A 清单，没有 6 列字段表；§4A 仍列了已废弃的 `until`；§9.2 `applied_at` 没有 response 字段表。 |
| P1-4 | 6 列规范只覆盖局部 schema | PARTIALLY CLOSED | 现有多数 wire schema 表头/分隔行列数对齐；但 §6.1.1 仍写“字段表 4 列规范”，§9.2 response 没表，§9.5 的 `PresetSnapshot` / `AuditDbSnapshot` / `ListenSnapshot` 等嵌套 DTO 仍靠示例或 inline 描述。 |
| P1-5 | fixture 同步缺机器可执行 manifest | PARTIALLY CLOSED | §14.3 已加 manifest schema、sha256、commit 绑定、发布/同步流程；但 fixture 命名的 `scenario` 文案带前导 `_`，和实际 `request_decision__minimal.json` 冲突；release discovery 仍写“release notes 或 endpoint”，不够确定。 |
| P2-1 | `remove_graylist` 尾注重引入弱歧义 | CLOSED | §9.8 现在明确当前协议 fingerprint 不存在 MUST 返回 `-32004`，`removed:false` 只留给未来 `idempotent` 模式。 |
| P2-2 | 重连 toast 全部描述成 daemon 重启 | PARTIALLY CLOSED | §3.2 加 `daemon_boot_id`，§3.4 区分 boot id 一致/不一致；但“本地无缓存”被归为 daemon 重启，第一次连接可能误弹“daemon 已重启”。 |
| P2-3 | 协议不匹配缺最小 GUI 文案 | CLOSED | §3.3 已规定不兼容 UI 状态、最小事实文案、至少一个可点击操作、禁用写入操作。当前不是过强约束，三选一是“至少一个”。 |
| G-P1-1 | 所有 wire schema 表统一 required/default/null | PARTIALLY CLOSED | 大部分完成；但缺 §9.2 response 与若干 nested DTO 表。 |
| G-P1-2 | 禁止裸 `String (ISO 8601)` / `String (UUID)` 的硬规则 | PARTIALLY CLOSED | 硬规则已加，裸旧写法基本清掉；但深层 `OriginHop` 未落 6 列表，`until` stale。 |
| G-P2-1 | UUID 示例占位不合法 | CLOSED | 示例 UUID 已换成完整 lowercase hyphenated 格式；manifest 里的 `8a3b9c1d...` 是 commit hash，不算 UUID。 |

## r2 引入的新问题
### P0（新发现的阻断）
无。

### P1（新发现应改）
**1. §1.3.1 oversize 判断顺序会误杀合法粘包**  
当前算法在 append 后立即判断 `frame_buf.len() > 1 MiB`，再扫描 newline。若上一轮留下 `1MiB - 16B` 的 partial frame，下一次 read 前 16B 正好补齐 newline，后面又带下一帧前缀，则单条消息合法，但总 buffer 超 1 MiB，会被错误关闭。  
修改建议：append 后先循环找 newline；每个完整 frame 用 `idx + 1 > 1 MiB` 判断单帧是否超限；消费所有完整帧后，若剩余无 newline 且 `frame_buf.len() > 1 MiB` 才关闭。另在 `chunk.is_empty()` 后显式 `return/break`。

**2. §9.2 `set_preset` response schema 缺字段表**  
示例返回 `result.applied_at`，§4A 也列了 `applied_at`，但 §9.2 只有请求表，没有 response 6 列表。  
修改建议：补 `applied_at | Timestamp | yes | — | no | daemon 应用 preset 的时间`。

### P2（新发现应改/建议）
**1. §14.3 fixture 命名规则的 scenario 写法不一致**  
格式是 `<message_kind>__<scenario>.json`，manifest 示例是 `request_decision__minimal.json`，但 bullet 写 `_minimal` / `_full` / `_null_optional`。照字面会生成三下划线。  
修改建议：把 scenario 取值改为 `minimal` / `full` / `null_optional` / `<feature>`；保留双下划线作为唯一分隔符。方法名内部单下划线不会与 `__` 冲突。

**2. §3.4 第一次连接的 `daemon_boot_id` 语义不清**  
“本地无缓存 → daemon 进程重启了 → toast”不适用于首次安装/首次连接。  
修改建议：首次成功连接且无 prior connected state 时只初始化 `last_seen_daemon_boot_id`，不弹重启/恢复 toast；只有发生过断连或存在跨连接 inflight 时才走 §3.4 文案。

**3. §10.0.2 TTL=60s 本身可接受，但作用域要收窄**  
它用于 GUI 发起的控制面 mutating request 回声去重，不应被误读为 daemon→GUI `request_decision` 的 120s 业务超时。  
修改建议：写成“仅适用于 §9.1–§9.3 这类会触发 fan-out 的 GUI-originated mutating requests”；未来若某控制面请求允许 >60s，应令 TTL ≥ 该请求超时。

**4. §10.0.1 建议显式声明全局串行化**  
单线程串行写下“先 fan-out 再 result”没有 race，多 GUI 同时改 preset 时应按 daemon 接收/处理顺序线性化，最后一个变更获胜。  
修改建议：补一句“daemon MUST serialize all mutating control-plane requests through one state transition queue/mutex; no interleaving of apply/fan-out/result across requests”。同时给慢 GUI 写入加 bounded write timeout，避免一个卡住的 socket 阻塞发起方 result。

## 全局遗漏
- §6.1.1 的“字段表 4 列规范”标题仍是旧文案，应改为“字段表 6 列规范”。
- `OriginHop` 应补完整 6 列 schema：`agent`、`action`、`timestamp: Timestamp`，否则深层字段仍不能被 fixture/schema 测试稳定消费。
- §9.5 nested DTO 不完整：`PresetSnapshot`、`AuditDbSnapshot`、`ListenSnapshot`、`GraylistSnapshot`、`IpcSnapshot` 至少要有字段表，不能只靠示例或 inline `{ addr: String }`。
- §4A Timestamp 清单里的 `until` 已被 r2 改名为 `paused_until`，应删除，避免实现者误以为仍有 wire 字段。
- §14.3 release 查询“release notes 或预定义 manifest endpoint”不够机器确定；建议强制固定 asset 命名或 manifest index，CI 不应 scrape release notes。

## 结论
**不建议现在进入代码改造阶段。** 最少先修 §1.3.1 oversize 判断顺序和 §9.2/`OriginHop`/health nested DTO 的 schema 缺口；修完后 SPEC 层基本可以进入 daemon/GUI 代码 PR。