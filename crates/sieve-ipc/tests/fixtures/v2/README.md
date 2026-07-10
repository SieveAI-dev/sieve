# SPEC-005 v2 Wire Fixtures

本目录存放 IPC 协议 v2 的 JSON wire 格式 fixture 文件，是 daemon ↔ client 跨仓 wire schema
一致性测试的数据源（GUI 仓持同构 fixture 集，双端各自反序列化，守护 schema 不漂移）。

## 现状

**以实际目录为准**（当前 21 个 method 目录 / 89 个 fixture 文件）。
每个 method 目录对应 SPEC-005 §11 完整消息清单中的一条消息（`decision_response` 这类伪方法名同样
独立成目录），覆盖握手 / 决策 / 控制面 / 历史清除等消息。

**已知缺口**：SPEC-005 §11 完整消息清单共 22 条消息，当前 21/22 有 fixture——
`sieve.judge_tool_call`（§11C，Since v2.x）在 daemon 与 `sieve-hook` 均已实装但尚无 fixture，
为已知待补项（SPEC-005 §14.3.1 已同步标注）。

## 三档 fixture（scenario 档位定义见 SPEC-005 §14.3.1）

每个 method 至少包含三档 fixture（最低门槛以 SPEC-005 §11 完整消息清单 × 3 计，不写死总数；
当前 `sieve.judge_tool_call` 尚缺，见上「已知缺口」）：

- `minimal` — 仅必填字段（所有 optional 字段省略）
- `full` — 全字段（所有 optional 字段都给值）
- `null_optional` — 所有 `null accepted: yes` 的字段显式为 `null`

文件名格式随消息形态：request/response 型为 `request.<scenario>.json` / `response.<scenario>.json`，
notification 型为 `notification.<scenario>.json` 或直接 `<scenario>.json`（如 `sieve.hello/`）。
注意：磁盘实态为**目录式命名**，与 SPEC-005 §14.3.1 规定的扁平命名
`<message_kind>__<scenario>.json` 存在历史错位，属旧账另行处理。
特性专用 fixture 是加项，如
`sieve.request_decision_canceled/notification.resolved_by_peer.json`
（2026-07-06 `request_decision` fan-out 投递引入，`reason="resolved_by_peer"`）。

## 消费入口

`tests/schema_v2_fixtures.rs`：

- 每类消息有显式反序列化断言测试；
- 生成式测试 `all_fixtures_valid_json` 遍历 `fixtures/v2/` 下**所有** `.json` 文件，
  新增 fixture 自动纳入校验，无需登记。

新增/修改 method 时：先改 SPEC-005，再同步补齐本目录三档 fixture 与 GUI 仓对应 fixture。
