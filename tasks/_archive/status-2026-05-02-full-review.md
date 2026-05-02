---
title: Sieve 项目全量状态审查
date: 2026-05-02
scope: docs + Rust workspace + CI gates + product readiness
status: review
---

# Sieve 项目全量状态审查（2026-05-02）

## 0. 结论

当前不是 GA ready。GA（General Availability，正式公开可用）前仍有门禁、source of truth、供应链、安装信任链和 dogfood 闭环问题。

代码主干的核心实现已经比较完整：`cargo build`、`cargo fmt`、`cargo clippy` 通过，规则数据集 FP（False Positive，误报）为 0，主攻击数据集 recall 为 99.71%。但默认测试门禁仍红，`cargo deny` 本地供应链门禁不可用，文档权威源已经分裂，且普通用户安装、信任、付费链路没有闭合。

本轮没有发现已证实的运行时 P0 安全绕过。当前 P0 主要是治理/门禁类：继续开发会被错误文档和红色测试门禁误导。

## 1. 本轮检查范围

缩写说明：

- PRD：Product Requirements Document，产品需求文档。
- ADR：Architecture Decision Record，架构决策记录。
- CI：Continuous Integration，持续集成。
- SSE：Server-Sent Events，服务器发送事件。
- IPC：Inter-Process Communication，进程间通信。
- HIPS：Host-based Intrusion Prevention System，主机型入侵防御系统。

覆盖内容：

- Git 状态：`main...origin/main`，检查开始时工作区干净。
- Workspace：`sieve-core` / `sieve-ipc` / `sieve-rules` / `sieve-cli` / `sieve-policy` / `sieve-hook` / `fuzz`，`fuzz_afl` 被 workspace 排除。
- 文档：README、AGENTS、CLAUDE、.cursorrules、PRD 指针、ADR 索引、architecture、development、CHANGELOG、status、known issues、v2 pending、GitHub templates、external landing/articles。
- 代码：`crates/` 下核心 Rust 源码、tests、fuzz target、CI workflow。
- 数据集：规则 recall / FP ignored test。

执行结果：

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo build --workspace --locked` | 通过 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --workspace --locked --no-fail-fast` | 失败：1 个 `doctor` 集成测试失败 |
| `cargo test --workspace --features sieve-cli/sequence_detection --locked --no-fail-fast` | 失败：同一个 `doctor` 集成测试失败；行为序列 e2e 本身通过 |
| `cargo test -p sieve-rules --release --test dataset_fp_rate --locked -- --ignored --nocapture` | 通过；主攻击数据集 694/696 = 99.71%，良性 Critical FP 0/1070 = 0% |
| `cargo deny check` | 失败：本地 `cargo-deny 0.18.3` 无法解析 advisory database 中 CVSS 4.0 |

说明：`cargo test` 第一次在沙箱内因 macOS socket 权限报 `Operation not permitted`，按沙箱规则重新用授权环境跑，以上失败是授权环境下的真实失败。

## 2. 优先级定义

- P0：会阻断正确开发/合并，或可能导致硬约束被错误执行。
- P1：GA 前必须解决，否则无法可信发布、无法通过供应链/安装/信任门槛。
- P2：重要缺口，会影响 dogfood、稳定性、覆盖率或关键承诺，但不一定立即阻断所有开发。
- P3：一致性、清理、文档漂移、体验和仓库卫生问题。

## 3. P0 问题

### P0-1：source of truth 分裂，当前开发入口会把人带回 PRD v1.5/v1.3

证据：

- `docs/requirements/PRD-sieve.md:5-15` 明确当前活动版本是 PRD v2.0，且已锁定执行。
- `README.md:13` 仍写当前最新 PRD 是 `sieve-prd-v1.5.md`。
- `README.md:84` 写 `PRD-sieve.md` 指向 PRD v1.5，`README.md:94` 写历史归档只到 v1.5。
- `AGENTS.md:21`、`CLAUDE.md:21` 仍写 PRD v1.5 是唯一权威源。
- `.cursorrules:3-5` 写来自 PRD v1.5 且项目仍是 Pre-Code；但 `.cursorrules:89-91` 又列 v2.0 新增第 14-16 条硬约束。
- `.cursorrules:140`、`.cursorrules:148` 仍写 PRD §9 是十三条硬约束。
- `.github/PULL_REQUEST_TEMPLATE.md:27`、`.github/PULL_REQUEST_TEMPLATE.md:58` 仍写十条硬约束；issue template 仍指 PRD v1.3。

影响：

新代码和后续 agent 很可能按 v1.5/v1.3 的旧规则工作，漏掉 PRD v2.0 新增的 #14 用户规则不能压制系统 Critical、#15 行为序列保守启动、#16 content-type 路由矩阵。这是治理层 P0。

建议：

统一更新 `AGENTS.md`、`CLAUDE.md`、`.cursorrules`、`README.md`、`SECURITY.md`、`docs/guides/development.md`、GitHub templates，把活动 PRD、硬约束数量、crate 列表、状态描述一次性切到 v2.0/v2.1。`docs/requirements/PRD-sieve.md` 保持唯一入口。

### P0-2：默认测试门禁红，当前 CI 合并信号不可信

证据：

- `cargo test --workspace --locked --no-fail-fast` 失败。
- 失败用例：`crates/sieve-cli/tests/doctor.rs:715` 的 `r10_5_t2_doctor_openclaw_daemon_not_running_exits_nonzero`。
- 同文件 `crates/sieve-cli/tests/doctor.rs:675-677` 的另一个测试用固定端口 `127.0.0.1:11453` 模拟 daemon 在线。
- T2 只在开头检查一次端口空闲，随后仍可能与 T1 并行竞态；失败时 T2 看到 daemon 正在监听，导致本该非零退出的测试成功退出。

影响：

当前 `ci.yml` 的 test job 运行 `cargo test --workspace --locked`，主分支/PR 可能因同一竞态红，也可能偶发绿。`tasks/status-2026-05-01.md` 把测试质量写成 100%，但实际默认门禁不是绿。

建议：

不要继续接受“已知竞态所以忽略”的状态。修法二选一：

- 给 doctor 测试注入可配置端口，每个测试用独立临时端口。
- 或用 `serial_test` 串行化所有依赖 `11453` 的 doctor/setup 测试。

## 4. P1 问题

### P1-1：供应链门禁本地不可用，`cargo deny check` 失败

证据：

- `cargo deny check` 在授权环境下失败。
- 失败原因不是依赖本身被拒，而是 `cargo-deny 0.18.3` 无法解析 advisory database 中 `RUSTSEC-2026-0073.md` 的 CVSS 4.0。

影响：

项目把 sigstore、reproducible build、pinned deps 当成硬约束。现在本地 pre-merge 自检无法完成供应链审计，属于 GA 前必须修的门禁问题。

建议：

升级或固定支持 CVSS 4.0 的 `cargo-deny` 版本，并确认 GitHub Action 与本地版本一致。修完后把命令结果写回 status 文档。

### P1-2：`X-Sieve-Origin` 签名公钥仍是占位，ADR-019 生产闭环未完成

证据：

- `crates/sieve-ipc/src/origin_header.rs:23` 写明 GA 前替换真实密钥。
- `tasks/v2-pending.md:34-41` 也把真实 Ed25519 公钥列为 GA 前任务。

影响：

多 agent provenance 只能做格式和降级处理，不能宣称生产级签名验证闭环。这个问题不一定影响当前单机 dogfood，但会影响 GA 安全承诺。

建议：

生成正式 Ed25519 keypair，把公钥纳入 repo 或 release artifact，私钥进入 1Password/hardware key；新增正反向签名验证测试，并更新 ADR-019/SECURITY/部署文档。

### P1-3：`sieve setup` 只对 Claude 跑 doctor，OpenClaw/Hermes apply 后没有同级验证和回滚

证据：

- `docs/specs/SPEC-004-multi-agent-setup.md:73-80` 要求 `sieve doctor --agent openclaw/hermes`。
- `crates/sieve-cli/src/commands/setup.rs:267` trait 已有 `doctor_check`。
- `crates/sieve-cli/src/commands/setup.rs:821` 和 `:1227` 分别实现了 OpenClaw/Hermes doctor。
- 但实际 setup 收尾处 `crates/sieve-cli/src/commands/setup.rs:1599-1608` 仍只验证 Claude，注释还写“其他 agent 为 stub，跳过”。

影响：

OpenClaw/Hermes 配置写坏时，setup 可能已经修改用户配置却不立即验证，也不会按 agent 回滚。对普通用户这是安装信任问题。

建议：

遍历 `applied_ctxs`，对每个已 apply 的 agent 调对应 adapter 的 `doctor_check`；失败时只回滚该 agent 的写入，并返回非零。同步补 OpenClaw/Hermes 非 dry-run setup 后 doctor 失败路径测试。

### P1-4：普通用户采用链路未闭合，不能用“代码 100%”替代产品 ready

证据：

- `tasks/status-2026-05-01.md:16` 写“代码 100% / 文档 100% / 商业化 0%”。
- `tasks/status-2026-05-01.md:90-99` 写部署链路 70%，`.dmg` + notarization 仍阻塞。
- `tasks/status-2026-05-01.md:126-134` 商业化 0%，公司、Stripe/Coinbase Commerce、license、域名、邮箱、Apple Developer 都未闭合。
- `tasks/doskey-todo.md:105-119` `.dmg` 打包、公证、cosign、release workflow 仍未完成。

影响：

普通用户今天不会放心安装一个没有签名/公证安装包、没有 GUI 可见反馈、没有支付/license 闭环、没有 dogfood 证据的安全代理。产品层面仍是 dogfood 准备态，不是 beta/GA ready。

建议：

把“工程完成度”和“用户可采用度”拆开写状态。GA 前至少闭合：签名 `.dmg`、notarization、GUI 状态面板、license 或试用策略、域名/邮箱、安装回滚、真实 dogfood 截图/日志。

## 5. P2 问题

### P2-1：IPC `request_decision` 仍可能被 GUI writer 队列背压卡住

证据：

- `tasks/known-issues-v1.4.md:231-234` 记录 P2-R10-#4：`mpsc.send().await` 写队列满会阻塞。
- 当前代码 `crates/sieve-ipc/src/socket_server.rs:523` 取 GUI sender，`crates/sieve-ipc/src/socket_server.rs:550` 仍调用 `sender.send(payload).await`。
- 同文件 broadcast 路径已使用 `try_send`，但 `request_decision` 路径未改。

影响：

GUI 卡死或消费慢时，超过队列容量可能让 SSE hold 请求长时间挂住。安全上可能 fail-closed，但用户体验会表现为请求无响应。

建议：

改为 `try_send`，Full/Closed 立即返回 `GuiUnavailable` 或进入明确 timeout fallback。补一个队列满时 `request_decision` 不阻塞的 tokio 测试。

### P2-2：`sieve rules enable/disable` 不触发 hot reload，且输出仍说必须重启 daemon

证据：

- `crates/sieve-cli/src/commands/rules.rs:194-216` 只有编辑器路径发送 `sieve.reload_user_rules`。
- `crates/sieve-cli/src/commands/rules.rs:384` 在 toggle 后输出“daemon hot-reload 待 Week 6 落地；本次改动需重启 daemon 才生效”。
- `crates/sieve-cli/src/daemon.rs:659-684` 实际已经实现 v2.1 zero-downtime reload listener。

影响：

用户通过 CLI 启用/禁用规则后，daemon 不会立即更新内存中的 LayeredEngine。文档声称 hot swap 100%，但实际 toggle 路径不是热更新。

建议：

把 reload 通知抽成共用函数，`edit`、`enable`、`disable`、后续 create/delete 都调用。消息改成“已通知 daemon reload；daemon 未运行时需重启”。

### P2-3：用户规则 compiled database size 上限没有真正执行

证据：

- `crates/sieve-policy/src/lint.rs:286-287` TODO：等待 `vectorscan_rs` 暴露 `hs_database_size()` 后补 1MB 上限。
- `crates/sieve-rules/src/engine/mod.rs:338-341` `compiled_pattern_size_bytes()` 直接返回 0。
- `tasks/v2-pending.md:18-26` 也把它列为 TODO-EXT-1。

影响：

PRD v2.0 用户规则资源约束还缺一块真实执行逻辑。当前只能靠 pattern 数量、长度、编译时间间接限流，不能证明 1MB compiled DB cap。

建议：

上游 API 出来后补真实 DB size；如果短期拿不到 API，就把“1MB cap”从已完成状态改成“不支持精确测量，当前用编译时间兜底”，避免文档过度承诺。

### P2-4：OpenAI SSE parser fuzz target 存在，但 CI fuzz-quick 没跑

证据：

- `fuzz/fuzz_targets/sse_parser_openai.rs` 已存在。
- `.github/workflows/ci.yml:89-94` 只跑 `sse_parser`、`tool_use_aggregator`、`inbound_filter`，未跑 `sse_parser_openai`。
- `.github/workflows/fuzz-nightly.yml:7-10` AFL++ nightly 仍是 `workflow_dispatch` only。

影响：

PRD §9 #5 要求 SSE 边界 fuzz 全覆盖。Phase 1 已引入 OpenAI 协议，OpenAI SSE parser 不进 quick fuzz gate 会留下协议分支覆盖缺口。

建议：

把 `cargo +nightly fuzz run -s none sse_parser_openai -- -max_total_time=60` 加入 CI quick fuzz。AFL++ nightly 要么启用 schedule，要么明确降级为手动调研工具，不再写成持续门禁。

### P2-5：公开攻击复现集仍有 3 个漏拦样本

证据：

- 本轮 `dataset_fp_rate` ignored test 通过，但 public replay recall 为 52/55 = 94.5%。
- 漏拦：`owasp-llm-top10/owasp-003.txt`、`real-events/real-003.txt`、`real-events/real-006.txt`。
- 主攻击数据集还有 2 个 private-key bucket 漏拦：`fear-privkey-087.txt`、`fear-privkey-096.txt`。

影响：

主数据集 recall 仍高于阈值，但公开真实攻击复现是营销/信任基线。3 个真实复现漏拦需要明确是规则盲区、非目标场景，还是必须补规则。

建议：

给 5 个 miss 建 issue：逐个判定“应补规则 / 接受盲区 / 需要非 vectorscan 路径”。补完后更新 `tasks/2026-05-01-public-attack-replay-report.md` 和 CHANGELOG。

### P2-6：OpenClaw `skill_install_guard` 仍缺真实样本验证

证据：

- `tasks/v2-pending.md:50-64` 要求 Week 7 抓 5+ 个真实 skill install request 样本。
- 当前 `crates/sieve-core/src/skill_install_guard.rs` 是“常见路径模式 + manifest 字段判定”的占位逻辑。

影响：

IN-CR-06 属于 OpenClaw 动态 skill 安装风险。没有真实样本前，覆盖率只能算 synthetic，不能对外宣称已验证。

建议：

在 OpenClaw dogfood 期间采样真实 request，固化为 fixture，并补正反例测试。

### P2-7：`.dmg` / hook 绝对路径问题仍挂起，安装体验会受影响

证据：

- `tasks/status-2026-05-01.md:144` 写 R10-#3 `sieve-hook` 绝对路径等 `.dmg` 打包路径拍板。
- `tasks/doskey-todo.md:105-119` 打包、公证、release workflow 仍未完成。

影响：

没有最终安装目录，hook 注册和回滚就无法完全按真实路径验证。普通用户安装失败时很难自救。

建议：

先定 `.app`/`.dmg` 目录结构，再把 setup 写入路径、doctor 检查、uninstall 回滚、launchd plist 全部绑定到最终路径。

## 6. P3 问题

### P3-1：技术栈文档漂移

证据：

- `README.md:108` 和 `docs/design/architecture.md:181/192` 写 `sonic-rs`。
- `docs/design/architecture.md:183` 写 forwarder 用 `reqwest`。
- 当前 workspace 依赖实际是 hyper/hyper-rustls/serde_json 路径，未见 `sonic-rs` 或 `reqwest` 依赖。

建议：

更新 README 和 architecture 的技术栈表，避免用户按不存在依赖理解架构。

### P3-2：development 文档的测试计数和 PRD 链接漂移

证据：

- `docs/guides/development.md:149` 写默认 633、feature 643。
- `README.md:11` 和 `tasks/status-2026-05-01.md:113` 写 feature 644。
- `docs/guides/development.md:692` 仍写当前活动 PRD 为 v1.5。

建议：

不要在多个文档硬编码测试计数；改成引用最新 status 文件或 CI artifact。PRD 链接统一走 `docs/requirements/PRD-sieve.md`。

### P3-3：benchmark 文档把 mean gate 写成 P99 gate

证据：

- `.github/workflows/bench.yml:18` job 名叫 `bench P99 gate`。
- `.github/workflows/bench.yml:71-72` 注释说明实际用 mean 代理 P99。
- `docs/guides/development.md:295` 写“任何 PR 让 P99 超过 20ms 自动 block 合并”。
- `docs/guides/development.md:316` 又写实际是 mean 退化 > 10%。

建议：

要么实现真正 P99 gate，要么把文档改成“criterion mean regression gate + 手动 P99 smoke benchmark”。

### P3-4：外部营销页面和文章仍有 placeholder

证据：

- `docs/external/landing-page/index.html:18-24` 域名、OG 图、Twitter handle placeholder。
- `docs/external/landing-page/index.html:1194/1228/1234/1443` download/GitHub/checkout placeholder。
- `docs/external/landing-page/index.html:1532-1547` 公司、邮箱、Privacy、Terms、Changelog placeholder。
- `docs/external/article-2-self-verification-zh.md:259` GitHub URL placeholder。
- `docs/external/article-3-ucsb-paper-drainer-zh.md:316` 早期访问链接 placeholder。

建议：

GA 前统一替换真实域名、公司、邮箱、下载、GitHub、隐私条款和付款链接。若还没确定，文档状态不要写“外部发布 ready”。

### P3-5：仓库里有 `.DS_Store`

证据：

- `./.DS_Store`
- `./docs/.DS_Store`
- `./docs/prd/.DS_Store`

建议：

删除并加入 `.gitignore`。这是卫生问题，不影响功能。

### P3-6：若干注释仍停留在旧周次/旧状态

证据：

- `crates/sieve-cli/src/commands/setup.rs:9-10` 仍写 OpenClaw/Hermes adapter 是 stub。
- `crates/sieve-cli/src/commands/setup.rs:1599` 仍写其他 agent doctor 为 stub。
- `crates/sieve-cli/src/commands/rules.rs:8` 仍写 daemon hot-reload 推 Week 6。
- `.github/dependabot.yml:13` 仍写当前 Pre-Code。

建议：

把旧周次注释清掉或改成当前真实状态。旧注释已经开始遮蔽真实实现。

## 7. 建议修复顺序

1. 先修 P0：统一 source of truth，修 doctor 固定端口竞态，让默认 CI 重新变绿。
2. 再修 P1：`cargo deny` 工具链、origin header 真实密钥、setup 全 agent doctor、安装/签名/商业基础设施。
3. 接着修 P2：IPC 背压、rules toggle hot reload、DB size cap、OpenAI SSE fuzz、公开复现 miss、OpenClaw 真实样本。
4. 最后清 P3：文档技术栈、测试计数、benchmark 表述、营销 placeholder、`.DS_Store`、旧注释。

## 8. 当前正向信号

- Rust workspace 架构已经从旧五 crate 演进到六个产品 crate + fuzz crate，模块边界大体清晰。
- `fmt` / `build` / `clippy -D warnings` 通过。
- content-type matrix、sequence_detection e2e、dataset FP/recall 这些高价值测试存在且基本有效。
- 良性 Critical FP 当前为 0/1070，核心数据集 recall 99.71%，规则质量不是当前最大风险。

真正要改的是“把可信发布链条补完”，不是继续堆功能。
