# SPEC-005 r4 Confirmation Review

## 总评
r4 已完整闭环 `sieve/docs/review/2026-05-02-codex-spec-005-review-r4.md` 中全部 P2 + 全局遗漏；未发现 P0/P1/P2 新问题或 regression。§10.0.1 的只读例外明确把 `sieve.evaluate` 排除在 mutating 串行队列外，§9.5 重排后的引用也未错指。可以冻结进入代码改造阶段。

## r4 review 闭环情况
| # | r4 问题 | r4 状态 | 备注 |
|---|---|---|---|
| 1 | §14.1 fixture 命名/数量同步 | CLOSED | `SPEC-005:1446` 已改为每个 message kind 至少三条：`__minimal` / `__full` / `__null_optional`；与 §14.3.1 `SPEC-005:1466-1479` 一致。 |
| 2 | §10.0.1 `reload_config` 无 fan-out 但仍需串行化 | CLOSED | `SPEC-005:1217` 明确无对应 fan-out 的 mutating 请求步骤 2 为 no-op，但仍 MUST 经过同一状态变更队列。 |
| 3 | §9.5 nested DTO 顺序与顶层引用 | CLOSED | 子节已按 `9.5.1/9.5.2/9.5.3/9.5.4` 排列：`SPEC-005:957`、`971`、`991`、`1007`；`listen/graylist/ipc` 顶层均指向 §9.5.4：`SPEC-005:951`、`954`、`955`。 |
| 4 | §1.3 oversize 描述避免总 buffer 误读 | CLOSED | `SPEC-005:53` 已区分完整 frame 长度超限与无 newline partial frame 自身超限，并指向 §1.3.1；算法约束在 `SPEC-005:70-98` 一致。 |
| 5 | §10.0.1 write error/timeout 全覆盖 | CLOSED | `SPEC-005:1223` 明确超时或任何 write error（EPIPE / ECONNRESET / EBADF 等）均视为 GUI 失联，断开并清理 sender，不阻塞 result。 |
| 6 | §14.3 release tag 不可覆盖 | CLOSED | `SPEC-005:1554` 增加 daemon CI MUST NOT 重复发布同一 release tag 下 artifact；除非 sha256 完全相同。 |
| 7 | manifest 示例 sha256/commit 占位 | CLOSED | `SPEC-005:1489`、`1491` 为 40 hex commit；`1497` 为 64 hex sha256。实测长度分别为 40/40/64。 |
| 8 | §16 r4 行声明是否可落证 | CLOSED | `SPEC-005:1592` 的 6 条声明均能对应到上面行号：§14.1、§10.0.1、§9.5、§1.3、§14.3.5、§14.3.2。 |

## 新问题
### P0 / P1 / P2
无。

补充：只发现一个不阻断冻结的 P3 级格式细节：`SPEC-005:948` 写的是 `见 9.5.1`，少了 `§`，但指向存在且不影响实现；可后续顺手改成 `见 §9.5.1`。

## 结论
不需要再做 r5 修订；可以冻结进入代码改造。

