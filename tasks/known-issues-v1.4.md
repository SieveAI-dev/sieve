# PRD v1.4 已知问题登记册

> 创建时间：2026-04-28
> 触发：codex review 三轮（R1 / R2 / R3）发现的累计 21 个问题中，前 15 个已修，后 6 个登记不修
> 决策原则：**修不动 = 暴露下层架构缺口**。等 GUI App 落地、setup 真实端到端打通后再回头处理
> 当前状态：cargo fmt + clippy `-D warnings` + 292 个 test 全过；working tree 稳定；可以 commit
> 回滚 tag：`pre-v1.4-refactor`（git checkout pre-v1.4-refactor）

---

## 一、修复历程总结

| 轮次 | codex review 发现 | 修复 | 残留 | 备注 |
|------|------------------|------|------|------|
| R1（首次）| 9 (6 P1 + 3 P2) | 9 全修 | 0 | 派 4 子代理，全绿 |
| R2（验证 R1）| 6 (3 P1 + 3 P2) | 6 全修 | 0 | 派 3 子代理，6 新 bug 都是 R1 修引入的；R1 的旧 9 个无回归 |
| R3（验证 R2）| 6 (4 P1 + 2 P2) | **0**（暂不修） | **6** | 都是 R2 修暴露的更深层架构缺口；继续修会无限循环 |
| R4（v1.5 PRD review）| 6 (3 P1 + 3 P2) | 0 | **+2 doctor** | 4 条是 R3 已登记的；新增 2 条 doctor 问题；v1.5 PRD 文档零问题 ✅ |

**核心 lesson**：v1.4 是大型架构翻转（一维处置 → 二维 + IPC + 双层防御 + GUI 独立仓库），单次"按规格实现"无法覆盖全部 fail-closed 路径。残留的 6 个问题都需要**端到端真实跑通**才能彻底验证，等 GUI App 在独立仓库落地后回头一次性闭环。

---

## 二、待修问题清单（按优先级）

### P1-R3-#1：setup 不部署规则文件，daemon 启动加载失败 🚨

**位置**：`crates/sieve-cli/src/commands/setup.rs:508-509`

**症状**：
- `sieve setup` 生成 `~/.sieve/sieve.toml` 时把 `rules_path` / `inbound_rules_path` 指到 `~/.sieve/rules/*.toml`
- 但 setup 流程**不创建** `~/.sieve/rules/` 目录、**不复制**内置规则
- launchd 启动 `sieve start --config ~/.sieve/sieve.toml` 加载规则失败，daemon 立即退出
- 一键安装实际上**装不上**

**修法（待 GUI 真实安装链路确定后）**：
- 方案 A：setup 时从 `.dmg` 内置 bundle 复制规则到 `~/.sieve/rules/`
- 方案 B：用 `include_str!` 把规则编入二进制，setup 写入 `~/.sieve/rules/`
- 方案 C：默认规则路径用绝对路径指向 `/Applications/Sieve.app/Contents/Resources/rules/`（取决于 .dmg 打包结构）

**影响**：阻断 P0；setup 是 v1.4 §10.1 Week 5 的核心交付，没有这个就没有 "doskey 朋友 30 分钟装上"

**等待依赖**：.dmg 打包方案落地（Week 7-8 或更晚）

---

### P1-R3-#2：Hook pending 写入失败时 fail-open 🚨

**位置**：`crates/sieve-cli/src/daemon.rs:877-879`

**症状**：
- IN-CR-02 / IN-CR-04 等 `hook_terminal` 命中后，daemon 调用 `write_hook_pending_silent()` 写 `~/.sieve/pending/<id>.json`
- 写失败（磁盘满 / 权限错 / IO 异常）→ 当前代码**只 warn 后继续转发 SSE**
- 静态注册的 `sieve-hook check` 启发式扫目录找不到 pending → fail-open exit 0
- **危险工具调用没有任何拦截点**，违反 §9 fail-closed 硬约束

**修法**：
- pending 写失败时**走 fail-closed 路径**：注入 `sieve_blocked` SSE event + 关流
- 或者 daemon 在写失败后改 disposition 为 GuiPopup 路径，强制 hold + GUI 处理

**影响**：违反 PRD §9 第 3 条 + ADR-007 fail-closed 原则。攻击场景罕见但严重——攻击者诱发磁盘满即可绕过

**等待依赖**：无；可独立修复，但需要审计日志同步告警

---

### P1-R3-#3：RedactAndAllow 漏脱敏，token 原样发上游 🚨

**位置**：`crates/sieve-cli/src/daemon.rs:353-356`

**症状**：
- GUI 对 OUT-06/08（gui_popup）命中返回 `RedactAndAllow` 时，当前代码 fall-through 到下方 `redact_hits` 收集逻辑
- 但 `redact_hits` 只收 `Action::Redact`，**不包含当前 HoldForDecision 的 span**
- 如果同一请求没有同时命中 AutoRedact 类规则，JWT / Stripe token **原样转发给上游**
- 用户以为 GUI 选了"脱敏后发送"，实际上 secret 仍然泄漏

**修法**：
- RedactAndAllow 分支显式把 hold detection 的 span 加入 redact_hits
- 或者把 hold detection 的 disposition 临时升级为 AutoRedact 后重新走脱敏路径

**影响**：直接泄漏用户 token，破坏 v1.4 §9 第 13 条出站脱敏承诺

**等待依赖**：无；GUI 真实跑通后必须立即修

---

### P1-R3-#6：sieve-hook 启发式扫遇坏 pending fail-open 🚨

**位置**：`crates/sieve-hook/src/pending.rs:83-90`

**症状**：
- 启发式扫描 `~/.sieve/pending/` 时如果某个文件写到一半 / 损坏 / 读权限异常 → 当前代码直接 `continue` skip
- 如果**所有** fresh pending 都是损坏的 → fresh=[] → exit 0 fail-open
- 应该被 HookTerminal 拦截的工具调用被放行

**修法**：
- 解析失败的 pending **不能 skip**，应当：
  - 记 stale_paths（让后续逻辑处理）
  - 或直接按 fail-closed 处理：发现解析失败立即 exit 1，让 Claude Code 拒绝
- 启发式扫描的语义需要重新审视：
  - 当前假设"扫不到 = 没有 Sieve 标记 = 让 Claude Code 通过"
  - 但"扫到坏文件 = 不知道 Sieve 怎么判 = 应该保守 fail-closed"

**影响**：违反 fail-closed；攻击者可以人工损坏 pending 绕过

**等待依赖**：无；改起来很简单，但是要决定"扫到 0 个有效 pending 时" 的行为语义

---

### P2-R3-#4：出站 GUI 类 timeout 硬编码 Block，规则配置不生效

**位置**：`crates/sieve-cli/src/daemon.rs:313-317`

**症状**：
- OUT-06 / OUT-08 在规则 TOML 配置 `default_on_timeout = "redact"`
- 但 daemon 给 IPC `request_decision` 硬编码 `default_on_timeout: DefaultOnTimeout::Block`
- 无 GUI 连接或超时时 → 返回 Deny → 426 拒绝
- 用户 / 上游看到的不是预期的"脱敏后转发"

**修法**：
- 从 RuleEntry.default_on_timeout 字段读，而不是硬编码
- engine_adapter::map_action_by_disposition 的 HoldForDecision 分支需要带上 default_on_timeout

**影响**：超时策略的"redact 兜底"失效，所有 hold 变成"非 Allow 即 Block"

**等待依赖**：无；纯实现 bug，独立修

---

### P2-R4-#7：doctor canary 检查太弱，会误报通过

**位置**：`crates/sieve-cli/src/commands/doctor.rs:191-192`

**症状**：
- doctor 只检查响应里**不含**原始 canary token
- 如果 daemon 只是把请求透传到 Anthropic 后拿到 401/502，响应也不会包含 canary token → doctor 误判"脱敏正常"
- 如果 canary token 本身没命中 OUT-01（拼写错 / 格式不匹配），同样会误判通过
- 用户以为 setup 验证通过，实际拦截链路根本没工作

**修法**：
- 校验**真的命中**了本地拦截/脱敏路径——构造一个明确匹配 OUT-01 规则的 token + 验证 upstream 收到的是 redacted body
- 或者用 fake upstream（local stub）拦请求 + 验证收到的 body 已经被改写
- doctor 输出明确区分"未走代理"vs"走了代理但没拦截"

**影响**：v1.5 §6.6 / SPEC-003 §doctor 的核心承诺失效，"装上即可用"无法验证

**等待依赖**：无；可独立修

---

### P2-R4-#8：doctor 失败时仍返回 Ok，CI 脚本无法捕获

**位置**：`crates/sieve-cli/src/commands/doctor.rs:70-76`

**症状**：
- 任一检查项失败时 `doctor::run()` 仍返回 `Ok(())`
- `sieve doctor` 在 CI / 脚本里以 exit 0 成功退出
- `sieve setup` 中的 `doctor::run()?` 无法捕获安装失败
- daemon 没启动 / 规则路径无效 / launchd 异常 → setup 显示成功但实际不可用

**修法**：
- 任一检查项失败时 `doctor::run()` 返回 `Err`（含失败项汇总）
- main.rs 把 doctor 的 Err 映射为 exit code != 0
- setup 自动调 doctor 时如果 doctor Err → 触发自动回滚

**影响**：违反 SPEC-003 doctor 设计承诺；CI 集成时假绿灯

**等待依赖**：无；改起来很简单

---

### P2-R3-#5：IN-CR-01 disposition 不生效，gui_popup 配置形同虚设

**位置**：`crates/sieve-rules/rules/inbound.toml:16` 的 disposition 字段；`crates/sieve-core/src/address_guard.rs` 直接构造 `Action::Block`

**症状**：
- IN-CR-01（地址替换）是 vectorscan 占位规则（pattern = `__ADDRESS_GUARD_PLACEHOLDER__`），实际命中由 sieve-core::address_guard 用 strsim Levenshtein 检测
- address_guard 输出 Detection 时**直接构造 `Action::Block`**，**不经过** InboundAdapter 的 disposition 映射
- TOML 里写的 `disposition = "gui_popup"` / `timeout_seconds = 60` / `default_on_timeout = "block"` 全部不生效
- 命中后立即 fail-closed 注入 sieve_blocked，**没有 GUI 弹窗确认机会**

**修法**：
- address_guard 输出时改用 `Action::HoldForDecision { request_id, timeout_seconds }`
- 从 RuleEntry 查 IN-CR-01 的 timeout_seconds + default_on_timeout
- 或者在 InboundAdapter 加一层 post-processing：address_guard 命中后按 disposition 映射

**影响**：v1.4 §4.2 场景 B（地址替换 GUI 弹窗 60s 倒计时让用户人眼对比）**完全不工作**——这是 PRD 的核心场景之一

**等待依赖**：address_guard 与 InboundAdapter 解耦机制需要梳理；中等复杂度

---

## 三、统一回归触发条件

以下事件之一发生时，必须回头逐条修：

1. **GUI App 在 sieve-gui-macos 独立仓库 MVP 完成**（能跑通 IPC 协议握手）→ #3 #4 #5 必修
2. **sieve setup 真实 .dmg 打包链路确定**（规则文件部署方式定下来）→ #1 必修
3. **任何用户开始 dogfood**（doskey 自己用 / 朋友试装）→ #1 #2 #6 必修
4. **第二次 codex review 之前**（下一轮 v1.4 增量 review）→ 全部清空

---

## 四、为什么不现在修

1. **修不动会暴露下层**：R1→R2→R3 三轮每次都暴露上一轮没料到的下层 bug；估计 R4 还会有 4-5 个新问题。这是**架构层次没完整跑通**的症状，不是单点 bug
2. **真正 fix 需要 GUI 端配合**：#3 #4 #5 都涉及 GUI 协议契约的细节，没有 GUI 实现端就只能 mock，mock 又会引入更多假设
3. **当前状态可以 commit**：
   - cargo fmt + clippy `-D warnings` + 292 个 test 全过
   - 文档与代码自洽（PRD v1.4 + 5 ADR + 3 SPEC + 5 crate）
   - GUI 在独立仓库的 contract 已经定下（IPC schema + SPEC-001/002）
   - 已知问题清晰可定位
4. **架构骨架值得保留**：双层防御 / IPC / disposition 二维矩阵 / sieve setup 框架——这些骨架建立了，缺的只是端到端跑通的最后一公里

---

## 五、关联资料

- codex review R1 log: [docs/review/2026-04-28-codex-review-v1.4.md](../docs/review/2026-04-28-codex-review-v1.4.md)
- codex review R2 log: [docs/review/2026-04-28-codex-review-v1.4-r2.md](../docs/review/2026-04-28-codex-review-v1.4-r2.md)
- codex review R3 log: [docs/review/2026-04-28-codex-review-v1.4-r3.md](../docs/review/2026-04-28-codex-review-v1.4-r3.md)
- codex review R4 (v1.5 PRD) log: [docs/review/2026-04-28-codex-review-v1.5.md](../docs/review/2026-04-28-codex-review-v1.5.md)
- v1.4 同步执行计划: `tasks/todo.md`
- v1.5 PRD: [docs/prd/sieve-prd-v1.5.md](../docs/prd/sieve-prd-v1.5.md)
- 回滚基线: `git tag pre-v1.4-refactor`（commit 743e681）
