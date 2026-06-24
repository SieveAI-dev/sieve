# SPEC 索引

> 最后更新：2026-06-19

本目录收纳 Sieve daemon 的工程级技术规格（功能落地详细度高于设计概述）。每个 SPEC 一个模块，禁止合并。版本号写在文件第一行 `> Version:` 标注里。

---

## 已落地

| 编号 | 标题 | 版本 | 状态 | 文件 |
|------|------|------|------|------|
| SPEC-001 | sieve-hook 文件 IPC 协议 | v1.0 | Stable | [SPEC-001-sieve-hook-protocol.md](SPEC-001-sieve-hook-protocol.md) |
| SPEC-002 | HIPS 弹窗行为规格 | v1.0 | Stable | [SPEC-002-hips-popup-behavior.md](SPEC-002-hips-popup-behavior.md) |
| SPEC-003 | sieve setup 工具行为规格 | v1.0 | Stable | [SPEC-003-sieve-setup-tool.md](SPEC-003-sieve-setup-tool.md) |
| SPEC-004 | multi-agent setup 配置注入规格 | v1.0 | Stable | [SPEC-004-multi-agent-setup.md](SPEC-004-multi-agent-setup.md) |
| SPEC-005 | Sieve daemon ↔ client IPC 协议（v2 wire schema 权威源） | v2.0 | Frozen | [SPEC-005-ipc-protocol.md](SPEC-005-ipc-protocol.md) |
| SPEC-006 | 更新通道 + 匿名装机统计 manifest 协议 | v0.1 | Draft | [SPEC-006-update-and-telemetry.md](SPEC-006-update-and-telemetry.md) |
| SPEC-007 | 上游转发代理支持 | v0.1 | Stable | [SPEC-007-upstream-proxy.md](SPEC-007-upstream-proxy.md) |

---

## 命名 / 状态 / 更新规则

- 编号三位、递增不跳号；废弃 SPEC 也保留占位，文件标记 `Status: Superseded by SPEC-NNN`。
- 文件名：`SPEC-NNN-功能名.md`，全英文，连字符分词。
- 状态语义：
  - **Draft** — 写作中，可随意改
  - **Stable** — 评审通过、生效中；修改递增 minor 版本（v1.0 → v1.1）
  - **Frozen** — 与发布版本绑定的快照，禁止任何修改；下一版本另开文件（如 SPEC-005 v2.0 → v3.0 时新建 `SPEC-005-ipc-protocol-v3.md`）
  - **Deprecated** — 不再维护
  - **Superseded** — 被新 SPEC 取代

---

## 何时新增 SPEC

- 一项架构决策落地时，工程实现细节超出一句话决策 + 三五条后果能描述的范围
- 跨 crate / 跨仓库（daemon ↔ GUI）的契约（IPC schema、协议字段、错误码语义）
- 复杂状态机（HIPS 弹窗倒计时、setup/uninstall 三阶段、入站 hold/inbound）

不写 SPEC：单 crate 内部实现、可以由 rustdoc + 单测描述清楚的逻辑、Phase 2 想象功能。

---

## 关联文档

- [架构](../design/architecture.md)
- [API 参考](../api/api-reference.md)
- 文档规范
