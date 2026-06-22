# ADR-047: 工具身份与执行漂移检测

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：PreToolUse 执行层（OS 工具调用边界），跨 Claude Code / OpenClaw / Hermes 三家；不涉及响应流四路由
> 关联：[ADR-014](./ADR-014-dual-layer-defense.md)（双层防御，hook 边界拦截）、[ADR-023](./ADR-023-process-context-audit.md)（进程上下文）、[ADR-024](./ADR-024-rules-engine-abstraction.md)（规则引擎抽象 / 可热替换签名规则包）、[ADR-034](./ADR-034-ga-key-gate.md)（签名规则包密钥 gate）

---

## 背景

### 为什么需要工具身份检测

Sieve 现有入站检测把"危险"绑定在**工具调用的参数文本**上（危险 shell、敏感路径、地址替换）。但有一类攻击不改参数、只改"被执行的到底是什么程序"——即**工具身份漂移**：表面上 agent 调用的是 `git`、`gh`、`kubectl` 或某个钱包 CLI，实际解析到的可执行文件已被替换成攻击者控制的同名程序。参数看起来完全正常，但执行体是假的。

### 触发的真实攻击场景（crypto 视角）

1. **command shadowing**：恶意步骤先在 workspace、用户家目录的 `bin/`、或临时目录里落一个名为 `git` 的脚本，随后让 agent 执行 `git push`。若该目录在 `PATH` 中靠前，agent 看似在做正常的版本控制，实际跑的是攻击者的 `git`——它可以在 push 的同时把 `~/.ethereum/keystore`、`.env`、助记词文件偷偷外传。被 shadow 的目标越是高频可信工具（`git` / `gh` / `kubectl` / 钱包 CLI），agent 越不会起疑。
2. **PATH 顺序劫持**：不替换文件，而是把一个攻击者目录 prepend 到 `PATH`，让后续所有 `git` / 钱包 CLI 调用解析到该目录里的同名假程序。
3. **alias / shell function 覆盖**：`alias git='...'` 或定义同名 shell function，使后续看似普通的工具调用走改写后的命令。
4. **symlink 替换**：把可信工具路径替换成指向攻击者脚本的符号链接，路径字符串不变但 inode 已变。
5. **`npx` / `uvx` / `pnpm dlx` / `bunx` 远程取包即执行**：`npx <url>` / `uvx <git+...>` / `pnpm dlx <archive>` / 以及任何 `@latest` 浮动版本，会在执行瞬间从网络拉取**未固定**的代码并直接跑。对 crypto 开发者，这等于让一段当下才确定内容的代码触达本地密钥环境。
6. **write-then-run dropper**：同一会话里先写出一个脚本/二进制（dropper），紧接着执行它。单看"写文件"或"执行文件"都不危险，组合起来是经典投放-执行链。

这些攻击的共同点：**危险不在工具参数的语义里，而在"这次调用真正会执行的身份/来源"里**。现有按参数文本判危的规则覆盖不到。

---

## 决策

**在 PreToolUse 执行层（OS 工具调用边界）新增一类"工具身份与执行漂移"检测：解析每次工具调用的真实执行体与来源，对 command shadowing、PATH 顺序劫持、alias/function 覆盖、symlink 替换、远程取包即执行（`npx`/`uvx`/`pnpm dlx`/`bunx` 指向 URL/git/archive/`@latest`）、write-then-run dropper 六类漂移做判定；高危身份漂移走 fail-closed 人工确认，可疑来源走提示确认。具体识别规则的精确定义由签名规则包提供、随更新通道分发，本 ADR 只锁定决策、接线点与验收。**

---

## 硬约束逐条核对

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed High-Risk Tool Policy Gate（PRD §9 #3）** | ✔ | 命中身份漂移类 Critical 规则后，沿用既有判危链 `handle_judge_tool_call`（`daemon_control_plane.rs:1018`）：只对 `critical_lock::is_fail_closed` 命中的规则强制人工确认，决策链错误 / GUI 失联 / 超时一律 `default_on_timeout=Block` → deny（`daemon_control_plane.rs:1082,1120-1130`）；client 端 hook 还有自身 deadline 兜底 fail-closed（`sieve-hook/src/main.rs:147-148`）。本 ADR 不新建任何旁路。 |
| **Critical 在所有版本（含降级模式）不可关闭（PRD §9 #8）** | ✔ | 凡定级为系统 Critical 的身份漂移规则进入 `critical_lock` 的 fail-closed 名单，受现有自我保护机制约束（用户规则 lint 禁止 suppress/override 系统 Critical，dry_run 对 fail-closed 无效）。本 ADR 不引入"可关闭的 Critical"。 |
| **BIP39 必须做 SHA-256 checksum 验证（PRD §9 #4）** | ✔（不适用，且不削弱） | 本 ADR 是执行层身份检测，与助记词识别无关，不触碰 BIP39 路径；现有 SHA-256 checksum 验证不受任何影响。 |
| **绝不联网做 verifier（PRD §9 #2）** | ✔ | 所有身份解析（PATH 查找、可执行文件路径归一化、symlink 解析、命令字符串解析）全部本地完成，不向任何远端发起校验。规则定义经现有更新通道分发后**本地**加载匹配（ADR-024/ADR-034），匹配过程零出站。注意：检测"`npx <url>` 即将取包"靠**解析命令文本**判断目标是远程来源，**不实际去取那个 URL**，也不联网验证它。 |
| **不在 API 协议层撒谎 / 不伪造 tool_use（PRD §9 #11）** | ✔ | 判定走 PreToolUse hook 与 IPC `sieve.judge_tool_call`，**完全不经过响应 SSE/JSON 转发路径**，不注入、不改写、不伪造任何 `tool_use` / `stop_reason` / `id` / `usage`。拦截结果通过 hook 退出契约（block/deny）表达，是 Sieve 自报裁决，不冒充模型。 |
| **不装本地 CA 做 MITM（PRD §9 #12）** | ✔ | 本能力在 OS 执行边界做检测，不触碰 TLS、不安装 CA、不改系统 proxy。 |
| **出站脱敏自动改写不弹窗（PRD §9 #13）** | ✔（不适用） | 这是出站脱敏类（OUT-*）的 UX 约束；本 ADR 属入站执行层危险拦截，按 fail-closed 走人工确认，与出站脱敏的"不弹窗自动改写"是两条独立路径，互不影响。 |
| **四路由 content-type 矩阵（PRD §9 #16 / ADR-025）** | ✘ 不适用 | 四路由矩阵约束的是**响应侧**入站检测（Anthropic SSE/JSON + OpenAI SSE/JSON 四类响应流）。工具身份漂移的判定发生在 **PreToolUse hook → `sieve.judge_tool_call`** 这条结构化工具调用链上，不解析任何上游响应流，因此 content-type 四路由矩阵在本 ADR 不适用（见下「验收标准」对此的显式说明与替代覆盖矩阵）。 |

---

## 方案

### 接线点（PreToolUse 执行层判危链，全部为现有链路，本 ADR 只加规则与解析）

```
agent PreToolUse hook（Claude Code / Codex / Hermes 各自子命令）
  └─ sieve-hook/src/main.rs:181（run_codex_command）/ :255（run_hermes_command）
        └─ codex_ipc::judge_tool_call(base, tool_name, tool_input, tool_use_id, cwd, deadline)
              —— sieve-hook/src/codex_ipc.rs:32
              —— 发 JSON-RPC sieve.judge_tool_call（带 tool_name / tool_input / cwd）
        └─ daemon: daemon_control_plane.rs:1018 handle_judge_tool_call
              └─ InboundEngine::check_tool_use(&CompletedToolCall, ContentSource::InboundToolUseInput)
                    —— trait 定义 sieve-core/src/pipeline/inbound.rs:36
                    —— CompletedToolCall { id, name, input }  (tool_use_aggregator.rs:84)
              └─ 命中 critical_lock::is_fail_closed → gated_request_decision（GUI 弹窗 + 审计 + 超时 Block）
```

要点：

1. **复用既有 `check_tool_use` 入口，不新增 IPC 方法**。身份漂移规则作为新规则 ID 进入 `InboundEngine::check_tool_use`（`inbound.rs:36`），与现有危险工具检测共用同一引擎与同一 fail-closed 判危链。`handle_judge_tool_call` 已把 `cwd` 一并随 `tool_input` 喂入（`daemon_control_plane.rs:1062`），为路径解析提供工作目录上下文。
2. **执行体解析为引擎内的判定输入**。引擎在匹配身份漂移规则前，需对工具调用做轻量解析：从命令字符串/参数中识别被调用程序名、解析其 `PATH` 查找结果与归一化路径、识别 `npx`/`uvx`/`pnpm dlx`/`bunx` 的取包目标来源、识别同会话内先写后执行的组合。**这些解析逻辑放在 `sieve-core` 引擎侧，规则的精确匹配定义（哪些路径前缀算"未知/不可信"、哪些来源形态触发、阈值与 disposition 映射）由签名规则包提供，随更新通道分发**（ADR-024 可热替换 + ADR-034 密钥 gate），本 ADR 正文不固化任何 pattern。
3. **规则包加载机制复用现有抽象**。新规则随签名规则包下发，经 `LayeredEngine` 合并；用户规则不能 suppress 这些系统级身份漂移 Critical（沿用 ADR-024 合并顺序与 sieve-policy lint）。
4. **disposition 分级**：高危身份替换/劫持（command shadowing、PATH 劫持、alias/function 覆盖、symlink 替换解析到不可信路径）走 Block / 人工确认（Critical fail-closed）；浮动来源类（`@latest`、远程取包即执行、write-then-run dropper）默认走 prompt（提示确认），是否升 Critical 由签名规则包按风险定义决定，受 FP 预算约束（PRD §9 #7）。

### 与既有 ADR 的边界

- 与 [ADR-014](./ADR-014-dual-layer-defense.md)：本 ADR 完全属于 Hook 类拦截路径（执行边界），不涉及 GUI hold-SSE 类路径；fail-closed 保障由 hook + daemon 双层提供，与 ADR-014 §4 一致。
- 与 [ADR-023](./ADR-023-process-context-audit.md)：进程上下文（caller_exe）是审计归因字段，本 ADR 是执行前判危；两者互补，不重叠。

---

## 分步实施

每步可独立 ship + 独立测试：

1. **第 1 步：执行体解析器（`sieve-core`）**。在引擎侧实现工具调用 → 被调用程序名/归一化执行路径/来源形态的解析器（纯本地、无 IO 除路径 stat），含 `PATH` 查找复现、symlink 归一化、`npx`/`uvx`/`pnpm dlx`/`bunx` 取包目标解析。
   - *测试*：单元测试覆盖每类解析输出（shadow 路径、prepend PATH、alias 串、symlink、远程包 URL、dropper 序列），不依赖规则包即可验证解析正确性。
2. **第 2 步：身份漂移规则族接入 `check_tool_use`**。新规则 ID 进入引擎，复用 `InboundEngine::check_tool_use`（`inbound.rs:36`）路径；规则定义由签名规则包提供（开发期用本地 fixture 规则包验证）。
   - *测试*：给定解析输出 + fixture 规则包，断言命中正确规则 ID 与 disposition。
3. **第 3 步：fail-closed 判危链贯通**。确认 Critical 身份漂移规则经 `handle_judge_tool_call` → `gated_request_decision` 走人工确认，超时/失联 deny。
   - *测试*：模拟 GUI 拒绝 / 超时 / IPC 故障，断言 verdict=deny（fail-closed）。
4. **第 4 步：三家 agent hook 契约对齐**。确认 Claude Code / Codex / Hermes 三个 hook 子命令把 deny 正确翻译为各自的 block 契约（沿用现有 `judge_tool_call` 复用，无需新增映射）。
   - *测试*：每家 hook 子命令在 deny 裁决下输出正确的 block 退出契约。
5. **第 5 步：红队 bypass 用例纳入回归门**。把下文红队用例接入红队 bypass 测试集，作为发布门常驻回归。

---

## 验收标准

### 替代覆盖矩阵（四路由不适用，改用执行层来源矩阵）

> 本 ADR 不解析响应流，**content-type 四路由矩阵（ADR-025）显式不适用**。改以"agent 来源 × 漂移类型"矩阵作为等价覆盖门，确保任一家 agent 都不会留下未覆盖的执行入口：

| 漂移类型 | Claude Code hook | Codex hook | Hermes hook |
|----------|------------------|------------|-------------|
| command shadowing | ✔ | ✔ | ✔ |
| PATH 顺序劫持 | ✔ | ✔ | ✔ |
| alias / function 覆盖 | ✔ | ✔ | ✔ |
| symlink 替换 | ✔ | ✔ | ✔ |
| 远程取包即执行（`npx`/`uvx`/`pnpm dlx`/`bunx`/`@latest`） | ✔ | ✔ | ✔ |
| write-then-run dropper | ✔ | ✔ | ✔ |

集成测试须对每个新规则 ID 在三家 hook 子命令的 `judge_tool_call` 路径各覆盖一次（漏挂任一家视为缺口）。

### 红队 bypass 用例（发布门 + 常驻回归）

1. workspace `bin/git`（或用户家目录 `bin/`、临时目录）落假 `git`，`PATH` prepend 后执行 `git push` → 必须命中 command shadowing 并 fail-closed 确认。
2. 假冒目标轮换覆盖 `git` / `gh` / `kubectl` / 钱包 CLI 各一例 → 每例均命中。
3. 仅 prepend `PATH`（不落文件）后调用受影响工具 → 命中 PATH 劫持。
4. `alias git='...'` 与同名 shell function 覆盖各一例 → 命中 alias/function 覆盖。
5. 可信工具路径被替换为指向不可信脚本的 symlink → 命中 symlink 替换。
6. `npx <https-url>` / `uvx <git+...>` / `pnpm dlx <archive>` / `bunx <pkg>@latest` 各一例 → 命中远程取包即执行（按规则包定义至少 prompt）。注意：检测只解析命令文本判定来源形态，**不实际去取该 URL**（验证零出站）。
7. 同会话内先写脚本再执行该脚本（dropper）→ 命中 write-then-run。
8. **真负例（FP 门）**：项目内正常 `git commit`/`git push`（解析到系统标准 `git`）、固定版本 `npx some-pkg@1.2.3`（非 `@latest`、非 URL）、正常 `kubectl get pods` → **不得命中**，确保 Critical 拦截 FP < 0.5%（PRD §9 #7）。
9. **fail-closed 验证**：模拟 GUI 失联 / 超时 / IPC 故障下命中 Critical → verdict 必须为 deny。

---

## 风险 / 已知 bypass / 误报面

### 风险

1. **解析复杂度**：shell 命令解析（管道、子 shell、变量展开、引号）天然复杂，解析器需在"足够覆盖常见绕过"与"不引入过重逻辑"之间取舍；过度解析会拉高延迟与 FP。起步保守，复杂 shell 构造按规则包定义逐步覆盖。
2. **PATH/环境快照时效**：判危时看到的 `PATH`/环境与工具实际执行瞬间可能有时间差；解析基于调用上下文（含 `cwd`）尽力复现，但无法保证与执行瞬间完全一致。

### 已知 bypass

1. **解析器盲区**：高度混淆的命令构造（深层嵌套子 shell、运行时拼接命令、间接 exec）可能绕过执行体解析；这类构造往往本身已可疑，可由其他危险 shell 规则兜底，但不保证全覆盖。
2. **运行时环境变更**：在 Sieve 判危之后、工具实际执行之前由其他途径再次修改 `PATH`/alias 的竞态，本能力看不到。
3. **非 hook 路径执行**：绕开 agent 的 PreToolUse hook 直接由其他进程执行的命令不在本能力判定范围内（本能力只覆盖经 `judge_tool_call` 的工具调用边界）。

### 误报面

1. 合法的项目本地工具（workspace 内自带的 `bin/` 工具链、版本管理器 shim 如 `asdf`/`mise`/`volta` 注入的 PATH 垫片）可能被误判为"未知路径执行体"；规则包须维护可信形态以压低 FP。
2. 合法的浮动版本用法（开发者有意 `npx tool@latest`）会触发 prompt——这是设计上的可疑而非危险，靠 prompt（非 Block）+ 用户可记忆决策缓解，但仍构成可感知的打扰面，须纳入 FP 预算评估。
