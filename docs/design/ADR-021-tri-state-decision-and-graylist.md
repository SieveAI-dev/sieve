# ADR-021: 三态决策 + 灰名单 + Critical 锁

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase A (Week 5-8)，IPC 协议扩展 + `~/.sieve/decisions/` 目录落地
> 关联 PRD：[v2.0 §5.4、§9 #3、§9 #8](../prd/sieve-prd-v2.0.md)
> 关联 codex review：[2026-05-01 codex review §A3、§C4、Must #1](../review/2026-05-01-codex-review-prd-v2.0.md)
> 关联风险：[R-V20-03 灰名单绕过内置 Critical](../prd/sieve-prd-v2.0.md#12-风险登记册)

---

## 背景

### v1.5 的决策模型缺陷

v1.5 的 `Disposition` 枚举包含 `GuiPopup` / `HookTerminal` 两种"等待决策"形态，但没有明确建模"持久化 ask 状态"——用户在 GUI 弹窗选"允许"后立即通过，下次同样场景仍然弹窗。这带来两个问题：

1. **体验摩擦**：合法操作反复弹窗（如开发者每天 10 次 markdown exfil 触发 IN-GEN-04）导致用户习惯性点允许，降低高危场景的决策质量；
2. **无记忆**：没有"永久允许此次场景"的灰名单概念，无法区分"一次性允许"和"持久信任"。

### 关键风险：灰名单绕过 Critical（codex review §A3 / Must #1）

codex review 发现 v2.0 §5.4 初稿允许 GUI "永久允许此次场景"的范围包含了 IN-CR-05（签名工具 EVM/Solana/Bitcoin）等内置 Critical 规则，等价于用户可以"关闭 Critical 拦截"，直接违反 PRD §9 #3（fail-closed Critical 不可关）和 #8（Critical 在所有版本不可关闭）。

**这是本 ADR 最高优先级约束**，定义为 Critical 锁（critical_lock），三道防线缺一不可。

### 与 ADR-007 的关系

ADR-007 确立了 fail-closed 原则：Critical 规则在所有版本、所有模式下强制执行，YOLO mode 不可关闭。ADR-021 是 ADR-007 在三态决策扩展下的**延伸约束**：三态的引入不能为绕过 Critical 打开新路径。本 ADR 不修改 ADR-007 任何原则，只在 v2.0 新增的"持久化 remember"机制上补加 Critical 锁防线。

---

## 决策

### 1. Decision 三态模型

每条规则命中后，daemon 输出一个 `Decision`：

```
Decision := Allow | Deny | Ask
```

六种有效状态组合及其 remember 权限：

| Decision 形态 | 持久化 | 触发场景 | Remember 允许 |
|--------------|--------|---------|--------------|
| Allow（一次性） | 否 | 系统规则评估为非 Critical，或匹配 allowlist_stopwords | N/A |
| Allow + Remember | 写灰名单 | 用户在 GUI 弹窗选"永久允许" | **仅非 Critical 系统规则 + 用户规则** |
| Deny（一次性） | 否 | 系统规则评估为 Critical + 无用户决策 | N/A |
| Ask（Hook 类，终端） | 否 | Hook 类规则命中（IN-CR-02/03/04），由 sieve-hook 终端 y/n 处理 | 内置 Critical 不可 Remember |
| Ask（GUI 类，Critical） | 否 | IN-CR-01 / IN-CR-05 等内置 Critical | **❌ 强制 disabled** |
| Ask（GUI 类，非 Critical） | 可选 | 用户规则 / IN-GEN-04 | ✅ |

### 2. Critical 锁三道防线（R-V20-03 的工程实现）

**防线一：IPC 协议层——daemon 计算 `allow_remember`**

`sieve.request_decision` JSON-RPC 消息加 `allow_remember: bool` 字段，**由 daemon 根据 `critical_lock::FAIL_CLOSED_RULES` 计算**，不由 GUI 决定：

```
allow_remember = !is_system_critical(rule_id)
```

GUI 收到 `allow_remember=false` 时必须隐藏或禁用 Remember 选项，不得自行覆盖。

**防线二：daemon 收到 `remember=true` 时二次校验**

`sieve.decision_response` 加 `remember: bool` + `context_hint: Option<String>`。daemon 收到 `remember=true` 时：
1. 重新查询 `critical_lock::FAIL_CLOSED_RULES`，验证 rule_id 不在名单内；
2. 不在名单内：写灰名单文件 + 写 audit.db（kind=`graylist_added`）；
3. 在名单内：忽略 remember 字段，写 audit ERROR（"Critical rule remember attempt blocked: {rule_id}"）+ GUI 状态栏通知；

**防线三：GUI 内置 Critical 弹窗强制 disabled+灰显**

GUI 端约束（属 sieve-gui-macos 仓库，此处为 IPC 合约约束）：内置 Critical 弹窗的 Remember checkbox 必须 `disabled=true` + 灰显，tooltip 文本："内置 Critical 规则保护核心安全场景，不允许永久绕过"。

**任何一道防线失效均视为与 v1.5.4 P0 同级别安全漏洞**（PRD v2.0 §12 R-V20-03 极高风险）。

### 3. 灰名单存储 schema

`~/.sieve/decisions/` 目录，每条灰名单一个 JSON 文件，文件名为 `fingerprint` 的 hex digest（不直接暴露 rule_id，防目录遍历泄露检测规则信息）：

```json
{
  "schema_version": 1,
  "fingerprint_version": 1,
  "rule_id": "IN-GEN-04",
  "rule_version": "v1.5.4",
  "fingerprint": "<sha256_64_hex_chars>",
  "fingerprint_inputs": {
    "rule_id": "IN-GEN-04",
    "matched_canonical": "<去空白、统一大小写后的命中片段>",
    "tool_name": "Bash",
    "protocol": "anthropic",
    "content_kind": "tool_use_input",
    "source_agent": "claude-code"
  },
  "decision": "allow",
  "expires_at": null,
  "added_at": 1745683210000,
  "added_by": "gui_user_decision",
  "context_hint": "用户输入的备注（GUI 表单）",
  "match_count_since": 0,
  "audit_event_id": "uuid-v4"
}
```

**fingerprint 计算规范**：`sha256(rule_id || matched_canonical || tool_name || protocol || content_kind || source_agent)`，其中 `matched_canonical` 是命中片段的规范化形式（trim + lowercase）。fingerprint_inputs 字段完整存入文件，方便审计和重新计算验证。

**灰名单查询**：daemon 在规则命中后、Decision 决策前查 `decisions/<digest>.json`。查询前重新计算 fingerprint 验证与文件内 `fingerprint` 字段一致（防人为编辑文件绕过），不一致则忽略该灰名单文件 + 写 audit WARN。

### 4. 文件系统安全约束

| 约束 | 实施方式 |
|------|--------|
| 文件权限 `0600` | 写入时强制 chmod；daemon 读取时拒绝非 0600 文件 |
| 目录权限 `0700` | daemon 启动时检查并修复 |
| No-follow symlink | `open(O_NOFOLLOW)` 语义，拒绝符号链接 |
| Atomic rename 写入 | 先写 `<digest>.json.tmp` 再 rename |
| fingerprint 完整性校验 | 读取时重新计算 fingerprint，与文件内字段比对 |

### 5. 灰名单审计要求

所有灰名单变更**必须写 `audit.db`**：

| 事件 kind | 触发时机 | 必填字段 |
|-----------|---------|---------|
| `graylist_added` | 写入新灰名单文件 | fingerprint / rule_id / added_at / audit_event_id |
| `graylist_removed` | 用户主动删除（v2.1 `sieve rules forget`） | fingerprint / rule_id / removed_at |
| `graylist_expired` | expires_at 到期后清理 | fingerprint / rule_id / expired_at |
| `graylist_integrity_fail` | fingerprint 校验不一致 | fingerprint / rule_id / file_path |
| `critical_lock_blocked` | daemon 收到 Critical 规则 remember=true 请求 | rule_id / source（"ipc_response"）|

### 6. IPC 协议扩展（与 ADR-013 的关系）

ADR-013 定义 IPC 基础协议（JSON-RPC over Unix socket + 文件锁）。本 ADR 在其上扩展两个消息的字段：

- `sieve.request_decision`：新增 `allow_remember: bool`（daemon 计算）
- `sieve.decision_response`：新增 `remember: bool`、`context_hint: Option<String>`

两字段向后兼容（缺失时 `remember=false`，`context_hint=null`）。

---

## 影响

### 正面影响

1. **减少合法操作弹窗频次**：非 Critical 规则命中后用户可一次性选择"永久允许"，后续同场景直接通过，决策质量提高（高危场景弹窗时用户注意力更集中）；
2. **Critical 安全承诺严格兑现**：三道防线确保任何路径都无法为 Critical 规则建立灰名单，ADR-007 fail-closed 原则在 v2.0 三态扩展下完整保留；
3. **审计可追溯**：所有灰名单变更写 audit.db，用户可查"哪条规则被我永久绕过了"，合规可查；
4. **fingerprint 防伪**：灰名单文件被人为编辑时 fingerprint 校验失败，自动无效化，防止本地文件攻击。

### 负面影响

1. **灰名单过期策略未定**：`expires_at=null` 表示永久不过期，v2.0 暂不实现过期机制（v2.1 评估）。如用户换了上下文但历史灰名单仍生效，可能带来意外的自动允许；
2. **fingerprint 规范化需严格一致**：SSE 流式 delta 拼接结果与 JSON full body 的 matched_canonical 计算必须完全等价（否则 SSE 写入的灰名单在 JSON 路径查询时 miss，v1.5.4 教训）。需专项测试覆盖；
3. **GUI 约束属跨 repo 合约**：防线三依赖 sieve-gui-macos 正确实现 disabled+灰显，本 repo 无法直接强制；通过 IPC `allow_remember=false` + daemon 二次校验（防线一 + 防线二）兜底。

### 需要更新的文档

- `docs/api/api-reference.md` §6 —— IPC 消息新增字段（`allow_remember` / `remember` / `context_hint`）
- `docs/design/data-model.md` —— 灰名单 schema 完整定义 + 文件系统约束
- `docs/design/architecture.md` —— `~/.sieve/decisions/` 目录在架构图中的位置
- `docs/design/ADR-007-fail-closed-critical-actions.md` —— 末尾补充段：Critical 锁三道防线是 v2.0 对 fail-closed 原则的延伸实现

---

## 相关文档

- [PRD v2.0 §5.4](../prd/sieve-prd-v2.0.md) —— 三态决策完整需求 + 灰名单 schema
- [PRD v2.0 §9 #3、#8](../prd/sieve-prd-v2.0.md) —— fail-closed Critical 不可关（两条原始硬约束）
- [PRD v2.0 §12 R-V20-03](../prd/sieve-prd-v2.0.md) —— 灰名单绕过 Critical 风险（极高）
- [codex review 2026-05-01 §A3、§C4、Must #1](../review/2026-05-01-codex-review-prd-v2.0.md) —— 强制补 Critical 锁的来源
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（本 ADR 延伸，不修改）
- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议基础（本 ADR 扩展其消息字段）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（Hook 类规则的 Ask 路径由 sieve-hook 处理，不走灰名单）
- [ADR-016](./ADR-016-disposition-matrix-2d.md) —— 处置矩阵二维化（三态 Decision 在矩阵中的位置）
- [ADR-020](./ADR-020-user-rules-system.md) —— 用户规则 Remember 权限（用户规则完全允许灰名单）
