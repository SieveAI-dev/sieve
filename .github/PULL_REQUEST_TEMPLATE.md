# Pull Request

## 摘要

<!-- 一两句说明本 PR 做了什么、为什么要做 -->

## 关联

- 关联 Issue: #
- 关联 PRD 章节: <!-- 如 PRD §5.1 OUT-09 BIP39 -->
- 关联 ADR: <!-- 如 ADR-002 / ADR-006 -->

## 类型

- [ ] feat: 新功能
- [ ] fix: bug 修复
- [ ] refactor: 重构（无行为变更）
- [ ] perf: 性能优化
- [ ] docs: 文档变更
- [ ] chore: 构建 / 配置 / 依赖

## 自检清单（提交前必勾，参见 [.cursorrules §五](../.cursorrules)）

- [ ] `cargo fmt --check` 通过
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` 通过
- [ ] 涉及 SSE / 规则 / 工具调用判定的改动有对应 fuzz / 单元测试（PRD §9 #5）
- [ ] **PRD §9 十六条硬约束未被绕过**（任一处放宽必须显式说明并 doskey 批准）
- [ ] CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- [ ] 关联文档（requirements / design / api / guides）已同步
- [ ] 临时文档（`_temp-` / `_draft-`）已清理或归档

## 检测项变更（如适用）

如果本 PR 增删 / 修改了 `OUT-*` / `IN-CR-*` / `IN-GEN-*` / `IN-MCP-*` 检测项：

- 影响的检测项 ID:
- 处置等级变化（如有，从 X → Y）:
- FP 上限验证（在哪个 benchmark 数据集跑过？）:
- 关联 user-stories 更新:
- 关联 architecture.md / api-reference.md 更新:

## 性能影响（如适用）

如果可能影响 hot path：

- P99 延迟变化:
- 内存峰值变化:
- 二进制大小变化:
- 启动时间变化:

> 性能预算见 PRD §6.4：P99 < 20ms / 内存 < 100 MB / 二进制 < 20 MB / 启动 < 500 ms。

## Breaking Changes

如果是 breaking change（影响接口 / 配置 schema / 检测项行为）：

- [ ] CHANGELOG 已加 `[BREAKING]` 前缀
- [ ] **如涉及 PRD §9 十六条硬约束变化，已经 doskey 显式确认**（默认拒绝）
- [ ] 用户迁移路径已写明
- [ ] PRD-sieve.md 版本演进表 + 对应 ADR 已同步

## 安全相关（如适用）

- [ ] 不引入任何远端 verifier / 数据上报（[ADR-003](../docs/design/ADR-003-local-only-no-cloud-verifier.md)）
- [ ] 不放宽 fail-closed Critical 行为（[ADR-007](../docs/design/ADR-007-fail-closed-critical-actions.md)）
- [ ] 依赖版本已 pin（[ADR-006](../docs/design/ADR-006-sigstore-reproducible-build.md)），新增依赖已说明必要性
- [ ] 不破坏 sigstore + reproducible build pipeline（Tier 1 必须双构建一致）

## 备注

<!-- 任何 reviewer 需要知道的额外信息 -->
