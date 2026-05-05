# protocol/ — wire schema 唯一权威源

本目录是 sieve daemon IPC 协议的 wire schema Rust 实现，与 SPEC-005 一一对应。

## 关联文档

- **SPEC-005**：`docs/specs/SPEC-005-ipc-protocol.md`（字段定义权威源，本目录跟随）
- **ADR-028**：`docs/design/ADR-028-ipc-protocol-neutralization.md`（本目录结构来源）
- **ADR-013**：`docs/design/ADR-013-ipc-protocol.md`（JSON-RPC over UDS 决策）

## 子模块职责

| 文件 | 内容 |
|------|------|
| `envelope.rs` | JSON-RPC 2.0 Request / Response / ErrorObject |
| `decision.rs` | DecisionRequest / DecisionResponse / DecisionAction 及配套枚举 |
| `handshake.rs` | HelloParams / ReloadUserRules（握手与 reload 通知） |
| `rules.rs` | 规则管理、preset 控制、evaluate 沙盒、灰名单 RPC 类型 |
| `audit.rs` | PurgeHistoryRequest / PurgeHistoryResult |
| `health.rs` | HealthResult 及所有子快照结构 |
| `notify.rs` | StatusBarNotify / PausedChangedNotify / PresetChangedNotify |

## 硬约束：零 IO 依赖

**本目录所有文件只能 import serde / chrono / uuid / std。**

禁止引入：
- `tokio` / 任何异步运行时
- `fd-lock` / 文件锁
- `bytes` / `memchr` / 任何 IO utility
- `tracing` / 日志
- `sieve-core` / `sieve-rules` / `sieve-policy` / 任何业务 crate

此约束使本目录在将来升级为独立 crate 时的成本等于 `mv` 一个目录（ADR-028 §决策 §2）。

## 修改流程

1. **先修改 SPEC-005**（`docs/specs/SPEC-005-ipc-protocol.md`）——字段权威源
2. **再改本目录代码**，保持字段名 / 类型 100% 与 SPEC 一致
3. **更新 CHANGELOG** + 兼容性说明（向后兼容扩展 vs breaking change）
4. 如涉及 method 名变更，参考 ADR-028 §决策 §1 的 alias 期策略
