# PRD v1.4 同步计划

> 触发时间：2026-04-27
> 触发原因：用户发布 PRD v1.4（HIPS 弹窗 + Native GUI + setup 工具 + Claude Code hooks 双层防御）
> 盘点子代理：a4607e356306b327f（文档）+ a39d2a784bf46a8c0（代码）
>
> **本文档目的**：在动手改任何文件之前，先把 v1.4 引入的所有冲突、歧义、需用户拍板的决策点列清楚，等用户确认后再派执行子代理。

---

## 0. v1.4 引入的核心架构改动（5 条）

| # | 改动 | 影响面 |
|---|------|-------|
| 1 | 处置矩阵从一维四级 → 二维（出站/入站 × 严重度）| 几乎所有文档 + 所有规则定义 + engine_adapter |
| 2 | OUT-01~05/12 自动脱敏（不弹窗）| outbound pipeline + api-reference §5 + user-stories US-01 + 现有 6 条出站测试 |
| 3 | HIPS 弹窗架构 + 5.4 节超时策略表 | 新规则字段（disposition / timeout_seconds / default_on_timeout）+ 新 ADR |
| 4 | Native GUI App 提到 Phase 1 必做（macOS SwiftUI 独立进程）| **与 architecture.md §6 当前明确列出的 `❌ 桌面 GUI App` 直接冲突** + 部署形态全面重写 |
| 5 | Claude Code hooks 双层防御（sieve-hook 工具）| **与 ADR-007 当前 Week 3 落地的 sieve_blocked 截流机制部分冲突** + 新 ADR + 5 个入站集成测试 |

附加变化：
- §9 新增第 11-13 条硬约束（不在协议层撒谎 / 不装本地 CA / 出站脱敏不打断）
- Phase 1 仅 macOS（Windows/Linux 推 Phase 2）
- IPC 协议新增（JSON-RPC over Unix socket + 文件锁 JSON 文件）
- sieve setup/doctor/uninstall 命令
- 操作系统级拦截推到 Phase 3

---

## 1. ⚠ 必须先拍板的决策点（执行前必答）

### Q1. GUI App 仓库结构

**事实**：v1.4 §6.4 说 GUI 是 SwiftUI 独立进程，Rust 这边只做 IPC server。当前 Rust 仓库里没有 Swift 代码。

**选项**：
- **A. monorepo**：在 Rust 仓库加 `gui-macos/` 子目录，Xcode 工程跟 Cargo workspace 并列
- **B. 独立仓库**：`sieve-gui-macos` 单独 git repo，跟 Rust 仓库通过 IPC 协议规格协调
- **C. Phase 1 暂用 stdout mock GUI**（v1.4 §10.1 Week 3 已经这么写了），SwiftUI App 真正动工时间 = Week 5

**推荐 C → 然后 B**：Week 5 之前用 stdout mock，给 IPC 协议时间打磨；Week 5 起 GUI 进独立仓库（doskey 一人维护，monorepo 没价值，反而拖慢 Rust CI）

### Q2. sieve-hook 是独立 crate 还是 sieve-cli 的 `[[bin]]`

**事实**：v1.4 §14 Open Question 8 已经把这个列为未决；hook 是每次 PreToolUse 都 fork 的进程，启动速度敏感。

**选项**：
- **A. 独立 crate `sieve-hook`**：依赖只有 `serde_json` + `fd-lock`，启动 < 50ms
- **B. sieve-cli 的 `[[bin]]` `sieve-hook`**：复用 config 解析，但二进制带上 vectorscan/rusqlite 等重依赖，启动 200ms+
- **C. shell 脚本**：维护成本高，跨平台麻烦

**推荐 A**：启动时延对 PreToolUse hook 体验影响巨大；config 路径常量可以抽到 `sieve-shared` 微 crate

### Q3. "不在 Anthropic API 协议层撒谎" 与场景 B 中止路径的边界

**矛盾**：
- v1.4 §9 第 11 条："不在 Anthropic API 协议层撒谎，不修改 stop_reason / id / usage"
- v1.4 §4.2 场景 B 中止路径：用户选 [中止] 时 "Sieve 替换 SSE 流为 user-friendly error，Claude Code 优雅终止"
- v1.4 §6.7 双层防御：Hook 类（IN-CR-02~04）"不修改 SSE 流"，但 GUI 类（IN-CR-01/05）需要 hold 流然后处置

**推荐解读**（需要写进 ADR-014 澄清）：
- **GUI 弹窗类**（IN-CR-01 地址替换 / IN-CR-05 签名）：用户选中止时**允许**截流并发 SSE error event，因为这是用户授权的中止，不是 Sieve 替模型说话
- **Hook 类**（IN-CR-02~04 + IN-GEN-01~03）：**禁止**修改 SSE 流；事后由 Claude Code 在 PreToolUse 阶段拦
- 当前代码 `build_sieve_blocked_sse()` 对 Hook 类的调用必须删除，对 GUI 类的调用保留（语义改为 "用户中止时返回的优雅 error"）

**请用户确认**这个解读是否符合 v1.4 本意。

### Q4. SSE 流 hold 期间的 keep-alive 算不算"修改流"

**事实**：IN-CR-05 签名类 hold 流最长 120 秒，Claude Code HTTP client 默认超时大概 30-60 秒，会先 abort。

**选项**：
- **A. 发 SSE comment 行 `: keep-alive\n\n`**（Anthropic 的 SSE 协议允许注释，不影响下游解析）
- **B. 不发，让连接自然超时**——但这意味着 IN-CR-05 在多数情况下无法走 hold 流路径
- **C. hold 流 ≤ 25 秒**——和 §5.4.2 的 120 秒矛盾

**推荐 A**：comment 行不是协议数据，符合 §9 第 11 条精神；但需要在 ADR-014 写明这一选择

### Q5. Phase 1 macOS only vs CI 多平台编译

**事实**：当前 sigstore CI 已覆盖 macOS / Linux / Windows 全平台编译。v1.4 §6.6 仅 macOS。

**选项**：
- **A. CI 继续多平台编译，setup/doctor/uninstall 在非 macOS runtime 报 "not supported"**（最小改动）
- **B. 用 cfg 把 setup/doctor/uninstall 在非 macOS 不编译，CI 只跑 macOS**
- **C. CI 只跑 macOS，删掉 Linux/Windows 配置**

**推荐 A**：daemon + 出入站规则引擎跨平台维护成本几乎为零，把 macOS only 限制在 GUI 安装链路上即可；将来 Phase 2 接 Linux/Windows GUI 时不用再翻文档

### Q6. 现有 7+ 集成测试要重写还是临时 ignore

**事实**：以下测试与 v1.4 处置语义直接冲突：
- `outbound_block.rs::fake_anthropic_key_blocked_with_426`（OUT-01 应自动脱敏，不返 426）
- `outbound_block.rs::dry_run_fail_closed_still_blocks`（OUT-01 不再有 fail-closed 截流概念）
- `inbound_block.rs::ucsb_attack_2_dangerous_shell_in_tool_use_blocked`（IN-CR-02 改为 Hook 类，不修改 SSE）
- `inbound_block.rs::in_cr_04_persistence_shell_rc_blocked`（IN-CR-04 改为 Hook 类）
- `inbound_block.rs::ucsb_attack_3_signing_tool_blocked`（IN-CR-05 改为 GUI hold 流）
- `inbound_block.rs::ucsb_attack_1_address_substitution_blocked`（IN-CR-01 改为 GUI hold 流）
- `inbound_block.rs::ucsb_attack_4_markdown_exfil_warn_only_passes_through`（IN-GEN-04 改为 GUI 弹窗，不再透传）
- `inbound_block.rs::in_cr_03_sensitive_path_warn_passes_through`（IN-CR-03 改为 Hook 类）

**选项**：
- **A. 全部重写，需要 mock IPC GUI 响应**（工作量大，但断言重新对齐 v1.4）
- **B. 部分 #[ignore]，标 TODO 等 IPC server 落地后再写**
- **C. 拆成两批：能直接验证字节级行为的现在重写；需要 mock GUI 的等 IPC server 落地**

**推荐 C**：拆成阶段，避免 Week 5 之前 CI 全红

### Q7. ADR-007 fail-closed 已落地的 sieve_blocked 截流机制怎么处理

**事实**：ADR-007 §1 声明 fail-closed 规则在 SSE 流中注入 sieve_blocked event 后关闭连接，Week 3 已经落地（`build_sieve_blocked_sse()` + `FAIL_CLOSED_RULES`）。v1.4 §6.7 把 IN-CR-02~04 这部分剥离出来交给 hook。

**选项**：
- **A. 新写 ADR-014（双层防御）supersede ADR-007 中关于 IN-CR-02~04 的部分**，ADR-007 加补充段说明
- **B. ADR-007 整体重写**
- **C. ADR-007 直接 deprecate**

**推荐 A**：ADR 只增不改，ADR-007 的 fail-closed 原则没错，错的只是 Week 3 实现细节；ADR-014 明确"GUI 弹窗类继续 fail-closed 截流；Hook 类 fail-closed 由 hook 端实现"

### Q8. 是否真的 Week 5 就能做完所有这些

**事实**：代码盘点子代理估算 Rust 侧工作量为 XL（IPC server L + sieve-hook S + setup/doctor/uninstall L + 出站脱敏 M + 入站 hook 重构 M + 入站 GUI hold 流 L + 规则字段扩展 S + audit 接入 M + 测试重写 M），加上 SwiftUI GUI App 工程量。

**选项**：
- **A. 严格按 v1.4 Week 5 执行**（一周冲完，doskey 朋友 30 分钟可装）
- **B. 把 Week 5 拆成 Week 5a（Rust IPC + sieve-hook + setup CLI）+ Week 5b（GUI App）**，roadmap 顺延
- **C. Week 5 只做 Rust 侧（IPC + hook + setup），GUI 用 stdout mock 支撑 dogfood，真 GUI 推到 Week 6-7**

**推荐 C**：v1.4 §10.1 Week 3 已经写"占位 GUI 用 stdout 模拟"，沿用这个节奏；Week 5 关键里程碑改为"sieve setup 一键能装通"+"IPC server 跑通"，GUI 真做留给 Week 6-7

---

## 2. P0 文档同步任务（确认决策点后并行派子代理）

按文档分组，每组一个子代理。

### G1. PRD 薄指针 + 全局引用替换

**子代理边界**：只改链接和版本号，不改任何业务内容。
**文件清单**：
- `docs/requirements/PRD-sieve.md`（薄指针升 v1.4，版本演进表加新行，硬约束改 13 条）
- README.md / CLAUDE.md / .cursorrules / SECURITY.md / tasks/roadmap.md 中所有 `sieve-prd-v1.3.md` 链接 → v1.4
- `docs/glossary.md` 各条目链接 + 端口号修正 `localhost:8734` → `127.0.0.1:11453`（顺手 bug fix）
- `docs/design/ADR-INDEX.md` 模板中 PRD 引用
- `docs/design/ADR-001~006、ADR-011` 状态段、影响段链接

**约束**：不动任何 ADR-007、ADR-004 的实质内容（实质内容由 G2/G3 子代理处理）

### G2. 新 ADR（5 个）

**子代理边界**：新建 ADR-012 ~ ADR-016，全部按 ADR-INDEX 模板格式
- **ADR-012：Native GUI App Phase 1**（撤销 architecture.md §6 当前 ❌ 桌面 GUI 决策）
- **ADR-013：IPC 协议（JSON-RPC over Unix socket + 文件锁 JSON）**
- **ADR-014：双层防御（Sieve 代理 + Claude Code hooks）**（同时 supersede ADR-007 中关于 IN-CR-02~04 截流部分）
- **ADR-015：sieve setup 自动配置工具**
- **ADR-016：处置矩阵二维化（出站/入站 × 严重度）**

**约束**：ADR-007 / ADR-004 不删不改实质，只在末尾加补充段引用新 ADR；ADR-INDEX 同步更新

### G3. architecture + data-model + api-reference 实质重写

**子代理边界**：
- `docs/design/architecture.md` §1 架构图（加 GUI App + sieve-hook + IPC Channel）、§2 模块矩阵、§6 部署形态（删 ❌ 桌面 GUI / 加三件套）、§7.5 已知限制（说明双层防御对修复路径的影响）、§8 不在 Phase 1 范围（删 GUI 行）
- `docs/design/data-model.md` §3 处置矩阵编码（一维 → 二维）、§3 特殊规则映射表（OUT-01~05 改自动脱敏）、§5 配置 schema（新增 IPC / preset / hook 字段）
- `docs/api/api-reference.md` §1.4 协议兼容性、§1.5 行为差异、§5 处置矩阵→HTTP 行为表（OUT-01~05/12 不返 426）、§4 环境变量（新增 sieve setup）、§6 CLI 退出码（hook + GUI 双协议）

**约束**：保持 mermaid 图风格一致；处置矩阵章节给出明确的二维表格 + 两条 UX 哲学；不修改 ADR

### G4. user-stories 场景重写

**子代理边界**：
- US-01（出站脱敏自动化，不弹窗，不打断）
- US-05（双层防御 + sieve-hook 终端 y/n）
- US-09（处置矩阵从"四级一致"重写为"出站/入站 UX 哲学不同"）
- US-13（Week 5 关联章节描述更新）
- 新增 US-P1-* 故事覆盖：sieve setup 一键安装、HIPS 弹窗 preset 切换、GUI App 菜单栏交互、sieve-hook 终端弹窗

**约束**：故事的 Acceptance Criteria 必须可测；不动 US-02 ~ US-08（私钥/BIP39/审计等无变化的故事）

### G5. development + deployment + roadmap

**子代理边界**：
- `docs/guides/development.md` §1 OS 支持矩阵（Phase 1 macOS / Linux Windows Phase 2）、§2 仓库结构（加 sieve-hook crate）、§3.4 启动 daemon（介绍 sieve setup）、§8 配置（新字段）
- `docs/guides/deployment.md` §2.1 macOS 改 .dmg 安装、§2.2 §2.3 Linux/Windows 标 Phase 2 不可用、§4 Claude Code 接入改为 sieve setup、§5 服务运行模式精简、§9 卸载改为 sieve uninstall
- `tasks/roadmap.md` Week 4 完成定义补 v1.4 二维矩阵、Week 5 完全重写为"Native GUI 占位 + sieve setup + IPC server + sieve-hook"、Week 6 删 Linux/Windows Tier 2、Week 7-8 GUI 真做

**约束**：保持 roadmap.md 与 PRD §10.1 的对齐关系；deployment.md 里 Phase 2 章节不删除，标灰显示

### G6. CHANGELOG + 入口文件硬约束补强

**子代理边界**：
- `docs/changelog/CHANGELOG.md` 新增 v1.4 段，列出 5 条架构改动 + 11-13 条新硬约束
- `CLAUDE.md` §不可放宽的硬约束 加第 11-13 条；§三个 Crate 加 sieve-hook；§何时写 ADR 候选列表更新
- `.cursorrules` §二 工程硬约束 加第 11-13 条

**约束**：CLAUDE.md 控制在 300 行以内（项目规范）

### G7. 新 SPEC（3 个）

**子代理边界**：新建目录 `docs/specs/`，写：
- `SPEC-001-sieve-hook-protocol.md`（pending/decisions JSON schema、文件锁、超时、exit code）
- `SPEC-002-hips-popup-behavior.md`（GUI vs Hook 弹窗触发条件、倒计时三段视觉、多 issue 合并、preset 矩阵）
- `SPEC-003-sieve-setup-tool.md`（setup/doctor/uninstall 行为、检测逻辑、改写内容、回滚）

**约束**：SPEC 在文件内标版本号 v1.0 / 日期 / 关联 ADR 编号

---

## 3. P0 代码同步任务（确认决策点后并行派子代理）

### C1. 新 crate `sieve-ipc` + `sieve-hook` 二进制

**子代理边界**：
- 新增 `crates/sieve-ipc/`：tokio UnixListener server + JSON-RPC 协议（request_decision / decision_response）+ pending file 写入 + decisions file 监听
- 新增 `crates/sieve-hook/`：极简 binary，依赖只有 `serde_json` + `fd-lock`，读 pending file → TTY y/n → 写 decisions file → exit code
- workspace `Cargo.toml` 加 members
- 单元测试 + 启动时延 benchmark

**约束**：sieve-hook 启动 < 50ms（不依赖 sieve-core/sieve-rules/vectorscan）

### C2. sieve-rules manifest 字段扩展

**子代理边界**：
- `crates/sieve-rules/src/manifest.rs` `RuleEntry` 加 `disposition: Disposition`（AutoRedact / GuiPopup / HookTerminal / StatusBar）+ `timeout_seconds: Option<u32>` + `default_on_timeout`
- 所有现有规则 TOML 文件补字段（按 PRD §5.4.2 超时表填）
- `critical_lock.rs::FAIL_CLOSED_RULES` 移除 IN-CR-02/03/04（改为 HOOK_RULES 列表）；保留 IN-CR-01/05 等 GUI 弹窗类
- `crates/sieve-rules/tests/` 同步加 disposition 字段解析测试

**约束**：`#[serde(default)]` 保证旧 TOML 文件不 break；`critical_lock` 9 条 IN-CR-04 fail-closed 不可直接删除，改为"hook 端 fail-closed"语义

### C3. sieve-core pipeline 重构

**子代理边界**：
- `crates/sieve-core/src/pipeline/outbound.rs` 新增 AUTO_REDACT 路径：OUT-01~05/12 命中后改写 body bytes + 更新 Content-Length，不返 426
- 新增 `crates/sieve-core/src/pipeline/inbound_hook.rs`：tool_use 类不修改 SSE 流，仅写 IPC pending file
- 新增 `crates/sieve-core/src/pipeline/inbound_hold.rs`：GUI 弹窗类 hold 流 + IPC 通知 + 等 oneshot + 超时默认拒绝 + 25s 间隔 keep-alive comment
- `Detection` 加 `HookMark` action（或完全用 disposition 路由）
- 单元 + fuzz 覆盖（outbound redact 字节边界 fuzz 是 PRD §9 #5 强制要求）

**约束**：不在 SSE 流里修改 stop_reason / id / usage / type；keep-alive comment 仅发 `: keep-alive\n\n`；body 改写后 Content-Length 必须更新

### C4. sieve-cli 新子命令 + audit 接入

**子代理边界**：
- `crates/sieve-cli/src/cli.rs` 加 `Setup / Doctor / Uninstall` 子命令
- `crates/sieve-cli/src/commands/setup.rs`：检测 Claude Code → 改 settings.json 注册 hook + 写 ANTHROPIC_BASE_URL → 写 launchd plist + launchctl load → 写 setup.log
- `crates/sieve-cli/src/commands/doctor.rs`：检查环境变量 / hook 注册 / daemon 在跑 / canary secret 拦截测试
- `crates/sieve-cli/src/commands/uninstall.rs`：读 setup.log dry-run + 二次确认后回滚
- `crates/sieve-cli/src/audit.rs`：接入 rusqlite，AuditEvent 异步写入，append-only 触发器
- `crates/sieve-cli/src/config.rs` 加 IPC / preset / launchd 相关字段
- `crates/sieve-cli/src/daemon.rs` `proxy_inner` 删除 build_sieve_blocked_sse 对 Hook 类的调用

**约束**：setup/doctor/uninstall 用 `#[cfg(target_os = "macos")]` runtime 检查（按 Q5 推荐 A），非 macOS 报友好错误；setup 修改用户文件前必须打印将要改的内容并要求确认；uninstall 默认 dry-run

### C5. 集成测试重构

**子代理边界**：
- `tests/outbound_block.rs` `fake_anthropic_key_blocked_with_426` 改为验证 upstream 收到脱敏 body + 响应 200
- `tests/outbound_block.rs` `dry_run_fail_closed_still_blocks` 重新定义为 OUT-07/09/10（校验位通过的 fail-closed）
- `tests/inbound_block.rs` IN-CR-02/03/04 改为验证 IPC pending file 写入 + SSE 流不变
- `tests/inbound_block.rs` IN-CR-01/05 mock IPC GUI 响应（abort / continue 两种）
- 新增 `tests/sieve_hook.rs` 启动时延 + TTY 行为
- 新增 `tests/sieve_setup.rs` 干跑 setup/doctor/uninstall 不实际改用户系统

**约束**：按 Q6 推荐 C 拆两批，先验字节级行为，IPC mock 测试等 C1 落地后写

---

## 4. 验收 Gate（cargo fmt / clippy / test / deny 全绿）

CLAUDE.md 提交前自检 6 条全部要过。

---

## 5. 不在本轮范围（Phase 2+）

- SwiftUI GUI App 真实现（Week 5 用 stdout mock，Week 6-7 真做）
- Linux / Windows 适配
- 操作系统级拦截（Network Extension / 本地 CA）—— v1.4 §6.8 明确推 Phase 3
- OpenAI / Gemini / OpenRouter 协议适配
- 第三方威胁情报数据源接入

---

## 6. 用户决策（2026-04-28 已确认）

- [x] **Q1 → B 独立仓库**：GUI 进 `sieve-gui-macos` 独立 git repo，本轮 Rust 仓库不出现任何 Swift 代码；文档需注明跨仓库协调方式
- [x] **Q2 → A 独立 crate**：新增 `crates/sieve-hook/`，依赖只有 `serde_json` + `fd-lock`，启动 < 50ms
- [x] **Q3 → 核心原则**：**不伪造 tool_use / stop_reason**（避免污染 Claude Code 上下文）；拦截发生时 Sieve **可以截流或注入 sieve_blocked SSE event**，但绝不伪造模型说的内容。Hook 类（IN-CR-02~04）的"阻断"由 sieve-hook 在 Claude Code PreToolUse 阶段完成，Sieve 代理本身不修改 SSE 流；GUI 类（IN-CR-01/05）用户中止时，Sieve 代理可以截 SSE 发 error event 优雅终止
- [x] **Q4 → A 发 SSE keep-alive comment**：hold 流期间每 25 秒发 `: keep-alive\n\n`，避免 Claude Code HTTP 超时；comment 不是协议 data，不算撒谎
- [x] **Q5 → 先只做 macOS**：sigstore CI 砍 Linux / Windows target，仅保留 macOS；setup/doctor/uninstall 用 `#[cfg(target_os = "macos")]` 严格 cfg；development.md / deployment.md 删掉 Linux/Windows 安装段（保留"Phase 2 计划"占位）
- [x] **Q6 → A 全部重写**：7+ 集成测试一次性按 v1.4 二维处置矩阵 + IPC 协议重写；mock GUI 响应用 fake IPC server 完成
- [x] **Q7 → A 新 ADR-014 supersede**：ADR-007 fail-closed 原则保留，Week 3 已落地的 `sieve_blocked` 截流对 IN-CR-02~04 的部分由 ADR-014 取代
- [x] **Q8 → A 严格按 v1.4 Week 5**：Rust 这边一周内冲完 IPC server + sieve-hook + setup/doctor/uninstall + 出站脱敏 + 入站 hook 重构 + 入站 GUI hold 流 + audit SQLite + 测试重写；GUI 由 doskey 在独立仓库平行做

## 7. 执行批次（按依赖关系分批）

**批次 1**（基础与源头，5 子代理并行）：
- G1 全局 PRD 链接替换 + glossary 端口 bug fix
- G2 新 ADR-012~016（5 个）+ ADR-INDEX 更新
- G7 新 SPEC-001~003（3 个，新建 `docs/specs/`）
- C2 sieve-rules manifest 加 disposition/timeout/default_on_timeout 字段
- C1 新 crate sieve-ipc + sieve-hook 骨架

**批次 2**（依赖批次 1 ADR/SPEC + crate 骨架，6 子代理并行）：
- G3 architecture + data-model + api-reference 实质重写
- G4 user-stories 重写 US-01/05/09/13 + 新增 US
- G5 development + deployment + roadmap（删 Linux/Windows）
- G6 CHANGELOG v1.4 段 + CLAUDE.md/.cursorrules 硬约束 11-13
- C3 sieve-core pipeline（出站 redact + 入站 hook + 入站 hold + keep-alive）
- C4 sieve-cli 新子命令 setup/doctor/uninstall + audit SQLite + daemon 删除 Hook 类 sieve_blocked 注入

**批次 3**（依赖批次 2，1 子代理）：
- C5 集成测试一次性按 v1.4 重写
