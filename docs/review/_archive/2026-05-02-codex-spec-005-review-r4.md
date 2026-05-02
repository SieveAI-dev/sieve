# SPEC-005 r3 Confirmation Review

## 总评
r3 闭环度约 **90%**。两个 P1 已实质闭合，核心协议可以进入 daemon/GUI 代码改造阶段；但建议先做一个很小的 r4 文档修订，修掉 §14.1 fixture 旧口径、§10.0.1 对无 fan-out mutating 请求的歧义、§9.5 小节顺序问题。当前没有 P0/P1 阻断。

## r3 review 问题闭环情况
| # | r3 问题摘要 | r3 状态 | 备注 |
|---|---|---|---|
| P1-1 | §1.3.1 oversize 判断顺序误杀合法粘包 | CLOSED | §1.3.1:57-100 已改为先消费完整帧，再判 remainder；EOF 后显式 return。边界覆盖基本清楚。 |
| P1-2 | §9.2 `set_preset` response 缺 `applied_at` 表 | CLOSED | §9.2:787-799 已补 response 示例和 6 列字段表。 |
| P2-1 | §14.3 fixture scenario 命名 `_minimal` 等不一致 | PARTIALLY | §14.3.1:1466-1475 已修为 `minimal/full/null_optional`；但 §14.1:1442 仍写 `*_minimal.json`、`*_full.json` 且“至少 2 条”，和 §14.3.1 的双下划线 + 3 fixture 门槛冲突。 |
| P2-2 | §3.4 首次连接不应弹 daemon 重启 toast | CLOSED | §3.4:196-200 已明确首次连接只初始化 `last_seen_daemon_boot_id`，不弹 toast。 |
| P2-3 | §10.0.2 TTL 作用域需收窄 | CLOSED | §10.0.2:1227-1237 已限定为 GUI-originated mutating control-plane 回声去重，并区分 `request_decision` 120s 业务超时。 |
| P2-4 | §10.0.1 全局串行化 + 慢 GUI bounded write timeout | CLOSED，但有新增 P2 澄清项 | §10.0.1:1211-1219 已补 mutating 串行化和 2s 推荐 timeout；但 `reload_config` 无对应 fan-out，见下方 P2。 |
| G-1 | §6.1.1 “4 列规范”旧标题 | CLOSED | §6.1.1:396-402 已改 6 列；全文只在 §16:1586 作为变更记录提到旧标题，不是规范引用。 |
| G-2 | `OriginHop` 缺完整 6 列表 | CLOSED | §6.1.1:415-429 已补 `agent/action/timestamp` 表。位置可读，但建议以后移到 `origin_chain` 字段后。 |
| G-3 | §9.5 nested DTO 表不完整 | PARTIALLY | 表已补，见 §9.5.1-§9.5.4:973-1021；但小节顺序现在是 9.5.3 → 9.5.1 → 9.5.2 → 9.5.4，且 health 顶层表对 `listen/graylist/ipc` 仍写 inline 描述而非 “见 §9.5.4”。 |
| G-4 | §4A Timestamp 清单仍含废弃 `until` | CLOSED | §4A:237 已移除 `until`，保留 `paused_until/generated_at/OriginHop.timestamp`。其他 `until` 只在字段重命名说明中出现。 |
| G-5 | §14.3 release discovery 不够机器确定 | CLOSED | §14.3.3-§14.3.4:1526-1543 已固定 asset 名 `sieve-ipc-fixtures-v2.tar.zst`，通过 `<release_tag>` 拼 URL，禁止 scrape release notes。 |

## r3 引入的新问题
### P0
无。

### P1
无。

### P2
1. **§14.1 fixture 旧命名/数量口径未同步**  
   位置：§14.1:1442。  
   问题：仍写 `*_minimal.json` / `*_full.json`，且“至少 2 条”；但 §14.3.1:1462-1475 要求 `<message_kind>__<scenario>.json`，且至少 `minimal/full/null_optional` 三种。  
   修改建议：把 §14.1:1442 改为“每个 message kind 至少包含 `<message_kind>__minimal.json`、`<message_kind>__full.json`、`<message_kind>__null_optional.json`”。

2. **§10.0.1 把 `reload_config` 放进 mutating 串行化，但没有对应 fan-out notification**  
   位置：§10.0.1:1211-1219，§9.4:862-894。  
   问题：步骤 2 要“fan-out 对应通知”，但 `sieve.reload_config` 当前没有 `rules_changed/config_reloaded` 之类通知。实现者可能误以为要新增通知。  
   修改建议：明确“无对应 fan-out 的 mutating request，fan-out step 为 no-op，但仍通过同一状态队列串行化 apply/result”；或从括号示例中移除 `reload_config`。

3. **§9.5 nested DTO 小节顺序与引用不一致**  
   位置：§9.5:948-955、957-1007。  
   修改建议：重排为 §9.5.1 `PresetSnapshot`、§9.5.2 `AuditDbSnapshot`、§9.5.3 `RulesSnapshot`、§9.5.4 `ListenSnapshot / GraylistSnapshot / IpcSnapshot`；并把 `listen/graylist/ipc` 顶层说明改成“见 §9.5.4”。

4. **§1.3 总表的 oversize 描述仍可能被读成按总 buffer 判定**  
   位置：§1.3:53，对照 §1.3.1:70-91。  
   修改建议：把“累计字节数超过上限”改成“单条 frame 长度超过上限；无 newline 的 partial frame 自身超过 1 MiB 时关闭”，避免和 r3 新算法冲突。

## 全局遗漏
- §10.0.1:1219 已写 timeout 视为 GUI 失联，但建议同时覆盖普通 write error：`write timeout or write error MUST be treated as client lost; disconnect that GUI and continue result path`。
- §14.3 固定 asset 名本身没问题；`<release_tag>` 已足够区分同一 protocol_version 下多次 daemon release。若要长期可追溯性更硬，可补一句“release tag / fixture asset MUST NOT be overwritten；重复发布同 tag 应 fail unless sha256 identical”。
- §14.3.2 manifest 示例里的 `spec_commit` / `daemon_commit` / `sha256` 使用 `...` 占位（§14.3.2:1485-1493），其中 `sha256` 与表格“64 字符 hex lowercase”不一致。建议换成完整 64 hex 示例，或明确该块是非机器 fixture 示例。

## 结论
r3 已关闭核心 P1，协议主干可以进入代码改造阶段；但在冻结 SPEC 前应先做一个小 r4 文档 patch，重点修 §14.1、§10.0.1、§9.5。修完后不需要再做全量 review，只需 focused confirmation。