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
| R5（A1+D3 验证）| 2 (1 P1 + 1 P2) | **修完 4 旧条目** ✅（R3-#2 / R3-#6 / R4-#7 / R4-#8）| **+2 R5 次生** | A1+D3 4 个修全部落地；R4-#7+#8 修引入 R5-#1（半配置回滚）+ R5-#2（canary 规则路径）|
| R6（A2 验证）| 4 (1 P1 + 3 P2) | **修完 R5-#1/#2 + 4 R6** ✅ | 0 | F-A2a/b 修 R6 后无残留 |
| R7（A2 R6 后验证）| 5 (2 P1 + 3 P2) | **修完 6 R6 残留 + 5 R7** ✅ | 0 | F-A2c/d/e 修 R7 后无残留 |
| R8（A2 R7 后验证）| 4 (0 P1 + 4 P2) | **修完 5 R7 残留 + 4 R8** ✅ | 0 | F-A2f/g/h 修 R8 后无残留；问题严重度递减 |
| R9（A2 R8 后验证）| 2 (1 P1 + 1 P2) | **R9-#1 修；R9-#2 P2 登记**（用户决策"P0/P1 修，P2 登记"） | 1 (R9-#2) | A2 主体收口 |
| **R10（2026-04-29 全量 review）**| 10 (5 P1 + 5 P2) | **5 P1 修：F-Full-1/4/5；5 P2 登记** | 5 (P2 only) | 全量 vs pre-v1.4-refactor 基线；F-1 OpenAI 上游路由 / F-2 规则部署（R3-#1 真修） / F-3 非 Claude 也装 daemon / F-4 RedactAndAllow（R3-#3 真修） / F-5 sieve-hook main corrupt（R3-#6 真修） |
| **R11（2026-04-27 codex review）**| 2 (1 P1 + 1 P2) | **2 全修** ✅ | 0 | R11-#1 daemon 未接入 upstream_routes（P1）/ R11-#2 Anthropic 路径 default_on_timeout 硬编码（P2）|

**核心 lesson**：v1.4 是大型架构翻转（一维处置 → 二维 + IPC + 双层防御 + GUI 独立仓库），单次"按规格实现"无法覆盖全部 fail-closed 路径。残留的 6 个问题都需要**端到端真实跑通**才能彻底验证，等 GUI App 在独立仓库落地后回头一次性闭环。

---

## 二、待修问题清单（按优先级）

### ~~P1-R3-#1：setup 不部署规则文件，daemon 启动加载失败~~ ✅ Fixed（F-2）

**修复方案**：方案 B（`include_str!` 嵌入 + setup 时写出）。
- 新模块 `crates/sieve-cli/src/embedded_rules.rs`：编译期把 outbound.toml + inbound.toml 打入二进制
- `install_shared_daemon()` 中调 `embedded_rules::install_to(&sieve_home/rules)` 写出规则
- 规则路径与 `sieve.toml` 中 `rules_path` / `inbound_rules_path` 完全匹配，daemon 启动即可加载
- 新增集成测试 `f2_rules_deployed_to_sieve_home_on_setup` 验证文件存在且内容非空

---

### ~~P1-R3-#2：Hook pending 写入失败时 fail-open~~（已修复）

**修复**：`write_hook_pending_silent` 改为 `write_hook_pending_or_fail_closed`（返回 `Result`）；写失败时注入 `sieve_blocked` SSE event 并截流。提取 `write_hook_pending_to(d, base)` 供单元测试注入路径，新增 2 个测试全部通过。

---

### ~~P1-R3-#3：RedactAndAllow 漏脱敏，token 原样发上游~~ **[Fixed]**

**位置**：`crates/sieve-cli/src/daemon.rs`（Anthropic 路径 + OpenAI 路径）

**症状**（已修复）：
- GUI 对 OUT-06/08（gui_popup）命中返回 `RedactAndAllow` 时，当前代码 fall-through 到下方 `redact_hits` 收集逻辑
- 但 `redact_hits` 只收 `Action::Redact`，**不包含当前 HoldForDecision 的 span**
- 如果同一请求没有同时命中 AutoRedact 类规则，JWT / Stripe token **原样转发给上游**
- 用户以为 GUI 选了"脱敏后发送"，实际上 secret 仍然泄漏

**修复**：`RedactAndAllow` 分支显式把 `hold_detections_outbound` 的 span 加入 `redact_hits`（去重），
Anthropic + OpenAI 两条路径都已修复。3 个新集成测试全部通过：
- `r3_fix_gui_redact_and_allow_anthropic_redacts_pem`（仅 OUT-07 GUI 类，RedactAndAllow → 脱敏）
- `r3_fix_gui_redact_and_allow_openai_redacts_stripe_key`（OUT-08 OpenAI 路径）
- `r3_fix_gui_redact_and_allow_mixed_both_spans_redacted`（OUT-01+OUT-07 混合，两个 span 都脱敏）
- `r3_fix_gui_allow_forwards_original_body_regression`（Allow 回归：不脱敏）

**影响**：直接泄漏用户 token，破坏 v1.4 §9 第 13 条出站脱敏承诺

---

### ~~P1-R3-#6：sieve-hook 启发式扫遇坏 pending fail-open~~ **[Fixed × 2]**

**位置**：`crates/sieve-hook/src/pending.rs` + `crates/sieve-hook/src/main.rs`（均已修复）

**症状**（已修复）：
- 启发式扫描 `~/.sieve/pending/` 时如果某个文件写到一半 / 损坏 / 读权限异常 → 旧代码直接 `continue` skip
- 如果**所有** fresh pending 都是损坏的 → fresh=[] → exit 0 fail-open
- 应该被 HookTerminal 拦截的工具调用被放行

**修复方案（第一次，lib.rs 路径）**：
- `ScanResult` 新增 `corrupt_paths: Vec<PathBuf>` 字段
- IO 读取失败或 JSON 解析失败 → 加入 `corrupt_paths`（不再 skip）
- `lib::run_check_heuristic` 新决策表：
  - `corrupt_paths` 非空 → 立即 fail-closed（exit 1），打 stderr 提示
  - `fresh` 非空（corrupt=[]）→ 正常弹窗流程
  - 全空 / 仅 stale → 原有行为不变
- 新增 7 个单元/集成测试覆盖 corrupt 路径

**修复方案（第二次，main.rs 生产 binary 路径，F-5）**：
- **问题**：lib 已修，但生产 binary `main.rs` 的 `run_heuristic` 独立实现，未同步 corrupt_paths 检查，仍然 fail-open
- 在 `main.rs::run_heuristic` 开头加 corrupt_paths 优先检查，与 `lib::run_check_heuristic` 行为完全对齐
- 新增 3 个集成测试（`tests/main_fail_closed.rs`）直接跑生产 binary 验证：corrupt→exit 1，空→exit 0，混合→exit 1

**影响**：生产环境 fail-closed 漏洞完全消除（lib + binary 两条路径均已修）

---

### ~~P2-R3-#4：出站 GUI 类 timeout 硬编码 Block，规则配置不生效~~ **[Fixed]**

**修复方案**：
- `detection.rs` 新增 `DefaultOnTimeout` 镜像枚举
- `Action::HoldForDecision` 新增 `default_on_timeout` 字段
- `engine_adapter::map_action_by_disposition` 从 `RuleEntry.default_on_timeout` 读取并传递
- daemon Anthropic + OpenAI hold 路径：`fold` 取多条 hold_detections 最严策略；IPC 超时和 IPC 未初始化时按规则 `default_on_timeout` 兜底（Redact → 脱敏转发，Block → 426，Allow → 放行）
- 新增单元测试 `out06_jwt_hold_for_decision_has_redact_timeout` + `out07_pem_hold_for_decision_has_block_timeout`

---

### ~~P2-R4-#7：doctor canary 检查太弱，会误报通过~~ ✅ Fixed

**位置**：`crates/sieve-cli/src/commands/doctor.rs`

**修复方案**：采用本地引擎直接 scan 方案（方案4）。
- 废弃原 HTTP 请求验证（401/502 透传误判根本原因）
- `check_canary_local_engine()`：直接调用 `VectorscanEngine::compile(outbound_rules).scan(canary_token)`
- canary token 精确匹配 OUT-01 pattern（`sk-ant-api03-[a-zA-Z0-9_\-]{93}AA`）
- 输出明确标注「仅验证规则引擎 + daemon listening；端到端验证需手动测」
- 新增集成测试 `tests/doctor.rs::canary_token_hits_out01_in_local_engine`（T1）+ `canary_check_fails_when_rules_file_missing`（T2）

---

### ~~P2-R4-#8：doctor 失败时仍返回 Ok，CI 脚本无法捕获~~ ✅ Fixed

**位置**：`crates/sieve-cli/src/commands/doctor.rs` + `src/main.rs`

**修复方案**：
- `run()` 收集所有失败项到 `Vec<(&str, bool)>`，任一失败返回 `Err("N 项检查失败：...")`
- `main.rs` `Command::Doctor` 分支：`if let Err(e) = run() { eprintln!(...); std::process::exit(1); }`
- 新增集成测试 `tests/doctor.rs::doctor_run_returns_err_when_checks_fail`（T1）+ `sieve_doctor_exits_nonzero_when_checks_fail`（T2，子进程验证 exit code 非零）
- setup 调用路径 `doctor::run()?` 已可正确捕获 Err（setup 回滚由 F-B1 子代理负责）

---

### ~~P2-R3-#5：IN-CR-01 disposition 不生效，gui_popup 配置形同虚设~~ ✅ Fixed

**修复**：`AddressGuardConfig` struct 加入 `InboundFilter`，从 `inbound.toml` IN-CR-01 RuleEntry 读取
`timeout_seconds` / `default_on_timeout`，命中时构造 `Action::HoldForDecision`（不再硬编码 Block）。
`daemon::run` 签名加入 `address_guard_config: AddressGuardConfig` 参数，每连接 `InboundFilter::with_address_guard_config` 传入。
同时补充 `detection.rs` 中 `DefaultOnTimeout` enum 定义及 `HoldForDecision.default_on_timeout` 字段。
4 个新单元测试全通过（cargo test -p sieve-core 140 passed）。

---

### ~~P1-R5-#1：setup 调 doctor 失败时半配置状态~~ ✅ Fixed

**位置**：`crates/sieve-cli/src/commands/setup.rs`

**修复方案**：
- `doctor::run()?` 改为 `if let Err(doctor_err) = doctor::run()` 显式捕获
- 失败时先调 `ctx.rollback()` 再返回 `Err`，携带友好消息："setup 已自动回滚（doctor 验证失败：<原因>）；请检查 doctor 报告"
- `SetupContext` 新增 `#[cfg(test)] fn new_with_written_files(...)` 辅助构造函数
- 新增 2 个单元测试（`macos::tests_rollback`）直接验证 rollback 行为
- 新增集成测试 `tests/setup_doctor_rollback.rs`（T1 dry-run happy-path + T2 doctor 失败回滚验证）

---

### ~~P2-R5-#2：doctor canary 用硬编码规则路径，不读 SIEVE_HOME / sieve.toml~~ ✅ Fixed

**位置**：`crates/sieve-cli/src/commands/doctor.rs`

**修复**：抽出 `resolve_rules_path()` 实现 4 级优先级——`SIEVE_RULES_PATH` > `sieve.toml rules_path` > `$SIEVE_HOME/rules/outbound.toml` > `$HOME/.sieve/rules/outbound.toml`；doctor 输出明确说明所用路径。新增 5 个优先级测试（R5-#2-T1～T5），全部通过。

**等待依赖**：无；改起来不复杂

---

### ~~P1-R7-#1~~ ✅ Fixed (F-A2f) / ~~R7-#2~~ / ~~R7-#3~~ / ~~R7-#4~~ / ~~R7-#5~~ ✅ Fixed (F-A2g/h)

R7 5 条全部修复，详见 codex review R7 log。

---

### ~~P2-R8-#1 4-段签名 header 解析~~ ✅ Fixed (F-A2i)

daemon 改用 sieve_ipc::parse_origin_header 支持 3 段/4 段格式。

---

### ~~P2-R8-#2 入站 chain_depth ≥ 2 升级 HookMark~~ ✅ Fixed (F-A2i)

classify_inbound_detections 加 chain_depth 参数，HookMark → HoldForDecision。

**R8-#2-Inbound（补验证）**：Anthropic 入站路径已在 F-A2i 修复时一并覆盖（`forward_with_inbound_inspection` 传 `meta.chain_depth` 给 `classify_inbound_detections`）。新增单元测试 `r8_inbound_hookmark_upgrades_to_hold_at_chain_depth_2` 补全验证。

---

### ~~P2-R8-#3 OpenAI stream + AutoRedact 后跳过入站检测~~ ✅ Fixed (F-A2i)

脱敏后仍走 forward_with_openai_inbound_inspection。

---

### ~~P2-R8-#4 OpenAIMessage 缺 flatten extra~~ ✅ Fixed (F-A2j)

OpenAIMessage 加 #[serde(flatten)] extra 字段，AutoRedact 重序列化保留 legacy function_call / 厂商扩展。

---

### ~~P1-R9-#1 OpenAI 入站缺 prompt 地址 seed~~ ✅ Fixed (F-A2k 派出中)

OpenAI 路径 stream=true 时调 inbound_filter.seed_known_addresses_from_text，IN-CR-01 不再绕过。

---

### ~~P2-R9-#2：chain_depth ≥ 2 时 Action::Redact 没升级 GUI~~ **[Fixed]**

**修复方案**：
- Anthropic 出站路径 + OpenAI 出站路径的 `chain_depth ≥ 2` 检查块中加入 `Action::Redact → HoldForDecision` 升级
- Redact 升级时 `default_on_timeout=Redact`（保持原脱敏语义，超时后仍脱敏转发而非 Block）
- 新增单元测试 `r9_fix_redact_upgrades_to_hold_for_decision_at_chain_depth_2`

---

### 2026-04-29 codex 全量 review (R10) 新增 P2 登记

5 条 P2，按用户决策"P0/P1 修，P2 登记"未修：

#### ~~P2-R10-#1：OpenClaw / Hermes 改动未追加 setup.log entry~~ **[Fixed]**
**位置**：`crates/sieve-cli/src/commands/setup.rs`
**症状**：OpenClawAdapter / HermesAdapter apply 后改动只在内存 `ctx.written_files`，没追加 setup.log。`sieve uninstall --agent openclaw` 找不到记录。
**修法**：给 `SetupContext` 加 `setup_log_path` + `append_log_entry`；每个 adapter apply 末尾写 `setup_complete`（含 backup_dir）+ `config_modified` 两条 entry；uninstall `file_actions` 加入 `config_modified`；非 settings.json 的 JSON/YAML 从备份全量恢复。4 个新测试通过（r10_openclaw_apply_writes_setup_log_entry / r10_hermes_apply_writes_setup_log_entry / r10_uninstall_openclaw_restores_backup / r10_uninstall_hermes_restores_backup）。

#### ~~P2-R10-#2：出站 GUI 类 timeout 硬编码 Block~~ **[Fixed]**

与 R3-#4 同根因，已在同一次修复中一并处理。详见 R3-#4 修复方案。

#### P2-R10-#3：sieve-hook 命令用相对路径不可靠
**位置**：`setup.rs:348`
**症状**：setup 写 Claude settings.json 的 hook command 是 `sieve-hook check`（依赖 PATH）。doctor 检字符串通过，PreToolUse 实际执行可能找不到 binary。
**修法**：setup 时把 sieve-hook 绝对路径（`/Applications/Sieve.app/Contents/MacOS/sieve-hook` 或 `~/.sieve/bin/sieve-hook`）写到 settings.json。需要 .dmg 打包链路定下来才能确定。

#### P2-R10-#4：sieve-ipc mpsc.send().await 写队列满阻塞
**位置**：`socket_server.rs:167`
**症状**：注释说"写通道满立即 fallback"，但 `sender.send().await` 实际会等到 mpsc 有容量。GUI 卡死时超过 32 个请求 → request_decision 阻塞 → SSE hold 无限挂起。
**修法**：用 `try_send` 或把 send 也包进 timeout。

#### ~~P2-R10-#5：OpenClaw / Hermes doctor stub 假绿~~ **[Fixed]**
**位置**：`doctor.rs:163`
**症状**：`sieve doctor --agent openclaw/hermes` 走 stub 不调用真实 doctor_check，all_passed 保持 true。配置坏 / daemon 没监听时 doctor 仍 exit 0。
**修法**：doctor.rs 通过 setup::macos 桥接函数 `run_openclaw_doctor_check` / `run_hermes_doctor_check` 调用真实 adapter doctor_check；先 detect 确认安装状态，未安装跳过 + 友好提示，安装但失败则 exit 1。DoctorReport 扩展为含 agent/checks/all_passed 字段。新增 4 个 R10-#5 集成测试（T1-T4）。

---

### ~~P1-R11-#1：daemon 没接 OpenClaw upstream_routes~~ **[Fixed]**

**位置**：`crates/sieve-cli/src/daemon.rs` + `crates/sieve-cli/src/upstream_routes.rs`

**症状**（已修复）：
- F-Full-1 子代理已实现 `upstream_routes.rs`（provider id → upstream URL 映射）
- OpenClawAdapter::apply 写入 `~/.sieve/upstream-routes.json` + 注入 `X-Sieve-Provider` header
- 但 daemon 从未读取该文件，所有请求仍转发到单一 `cfg.upstream_url`（通常是 Anthropic）
- OpenClaw 通过 Sieve 转发 OpenAI / OpenRouter / DeepSeek 请求时全部路由错误

**修复**：
- `run()` 启动时调 `UpstreamRoutes::load()` 加载路由表，对每个 provider 预构建 `Arc<Forwarder>`（含连接池），存入 `HashMap<String, Arc<Forwarder>>`
- `proxy_inner()` 入口解析 `X-Sieve-Provider` header，查 map 得到 provider forwarder；未命中用默认 `forwarder` 兜底
- 转发前从 `parts.headers` 移除 `X-Sieve-Provider`（内部路由 header，不透传给上游）
- 加载失败（文件不存在 / JSON 非法）→ warn + 全量兜底默认上游，daemon 不中断启动
- `upstream_routes.rs`：删除 `#[allow(dead_code)]` 注解；新增 4 个单元测试（insert+iter / 文件不存在 / JSON 正常加载 / JSON 非法报错）
- 集成测试（`tests/outbound_block.rs`）：
  - `r11_provider_header_routes_to_correct_upstream`：有 header + 路由命中 → 打到 provider 上游
  - `r11_unknown_provider_falls_back_to_default_upstream`：header 命中不到 → 兜底默认
  - `r11_missing_routes_json_daemon_starts_normally`：routes.json 不存在 → daemon 正常启动

---

### ~~P2-R11-#2：Anthropic 路径 default_on_timeout 硬编码 Block~~ **[Fixed]**

**位置**：`crates/sieve-cli/src/daemon.rs`（出站 Anthropic hold 路径）

**症状**（已修复）：
- OpenAI 路径已在 R3-#4 修复时正确使用 `merge_strictest_timeout` + `map_dot_to_ipc` 读取规则 `default_on_timeout`
- Anthropic 路径的 `HoldForDecision` hold 段仍用 `find_map` + 硬编码 `sieve_ipc::DefaultOnTimeout::Block`
- OUT-06 JWT（default=Redact）：IPC 超时或未连接 → 错误地返回 426；正确行为应脱敏转发
- IPC 未初始化时同样硬编码 426，不区分 Redact / Allow / Block 三种策略

**修复**：
- IPC 有连接时：用 `fold` + `merge_strictest_timeout` + `map_dot_to_ipc` 取多条 detection 的最严 `default_on_timeout`；IPC error 时三路分支（Redact → 脱敏转发 / Allow → 放行 / Block → 426）
- IPC 未初始化时：用相同 `fold` 逻辑取 `effective_dot`，走同样三路分支
- 与 OpenAI 路径完全镜像（代码逻辑等价）
- 集成测试：
  - `r11_anthropic_out06_default_redact_no_ipc_redacts_and_forwards`：OUT-06 + 无 IPC → 200 + body 含 REDACTED
  - `r11_anthropic_out07_default_block_no_ipc_returns_426`：OUT-07 + 无 IPC → 426（回归断言）

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
- codex review R5 (A1+D3 验证) log: [docs/review/2026-04-28-codex-review-a1-d3.md](../docs/review/2026-04-28-codex-review-a1-d3.md)
- codex review R6-R9 (A2 多轮) log: [docs/review/2026-04-28-codex-review-a2{,-r6,-r7,-r8}.md](../docs/review/)
- codex review R10 (2026-04-29 全量) log: [docs/review/2026-04-29-codex-review-full.md](../docs/review/2026-04-29-codex-review-full.md)
- v1.4 同步执行计划: `tasks/todo.md`
- v1.5 PRD: [docs/prd/sieve-prd-v1.5.md](../docs/prd/sieve-prd-v1.5.md)
- 回滚基线: `git tag pre-v1.4-refactor`（commit 743e681）
