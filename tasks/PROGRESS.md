# Sieve daemon · 进度

> 上次更新：2026-06-18
> 当前阶段：**PoC 就绪度 ≈ 代码 95% / 真机验证 40%——CLI 版 PoC 现在就能 demo（差 commit/push + 用户给 2 把 API key + 真机跑 30min）；GUI 弹窗版再叠加一台有 Xcode 的机 + 半天联调。无功能性代码缺口阻塞 demo。** dogfood 已完全自动化；自动化抓出的真 bug（zstd 字节序 / headless CLI 嵌套 runtime panic / 6 类跨仓 schema 漂移 / detection 审计未接线）全部已修。
> 质量基线：**workspace 799 passed / 0 failed / 7 ignored**；fmt/clippy/deny 全绿；dogfood.sh 一键 11 场景全绿；GUI swiftc 探针 21/21（注：GUI 仓 swift test 因 CLT ABI 跑不起来，只 swiftc 验证）。
> ⚠️ **本会话所有修复均未提交**：daemon 仓 ~41 文件未提交 + 1 commit 未 push；GUI 仓 ~27 文件未提交。

## 🎯 到可演示 PoC alpha 还缺什么（2026-06-18 差距分析，详见 docs/review/2026-06-18-poc-readiness.html）

> PoC 定义：真实 crypto dev 本地装 sieve、Claude Code 流量经 sieve、亲眼看到「出站脱敏 / 入站拦截 / HIPS 弹窗」端到端生效的可演示 alpha（不要 GA：无需签名分发链/公开 repo/多平台/Stripe）。

### 线 A · CLI 版 PoC（除真 key 无硬阻塞，我侧 1-2h）
- [ ] **A1** commit/push 本会话全部修复（daemon ~41 文件 + GUI ~27 文件；不提交则环境拉到旧坏版本）〔我，trivial，**等用户批准提交**〕
- [ ] **A2** cargo build + `sieve setup` 一键注入 ANTHROPIC_BASE_URL + PreToolUse hook 〔我，trivial〕
- [ ] **A3** CLI 真启动 smoke：`sieve decisions watch` / `sieve audit tail -f` 各跑一秒不 panic 〔我，trivial〕
- [ ] **A4** 🚫 **真流量验证（§3.3）**：起真 Claude Code 会话 → 粘 .env 看脱敏(OUT-01) + 触发 signTransaction/curl|sh 看拦截(IN-CR-05/02)〔**需用户给 ANTHROPIC_API_KEY + DeepSeek key**，我驱动，small〕
- [ ] **A5** 审计真落库验证：真流量触发后查 `~/.sieve/audit.db` 的 DecisionMade/OutboundRedacted 行 〔我，small〕
- [ ] **A6** path prefix 抓包：tcpdump 确认 upstream path 含 `/anthropic`（DeepSeek）〔我，trivial〕

### 线 B · GUI 弹窗完整版（叠加，需 Xcode 机，就位后我侧约半天）
- [ ] **B1** 🚫 Xcode.app 就位（开发机仅 CommandLineTools）〔**需用户装/借机**〕
- [ ] **B2** xcodebuild 编译 GUI（SwiftUI 层从未被编译过）→ BUILD SUCCEEDED 〔我驱动〕
- [ ] **B3** 生成 Sparkle EdDSA 密钥对，公钥替换 PLACEHOLDER（否则自动更新崩）〔我，small〕
- [ ] **B4** 清数据完整性债：list_rules 注入用户规则 / purge_history 联调 / direction 字段孤立路径测试 / wait_for_ipc 计数污染 〔我，small×4〕
- [ ] **B5** 真 daemon↔真 GUI 联调：菜单栏五态 → BIP39 触发 → HIPS 弹窗倒计时三段 → 拒绝 → 红图标 + History 增条 + 导出 CSV 〔我驱动，medium〕

### 🚫 PoC 范围外（GA 才做，别现在碰）
Linux/Windows 多平台 + 拦截引擎 trait 非 macOS 实现 / 网络隔离 enforcement(ADR-027) / 行为序列升 Block(需 4 周≥50 样本) / 运维服务端全栈(TODO-13~16) / 签名分发链(sigstore/notarization/appcast/.dmg) / Stripe+海外主体 / 闭测招募+Discord。

### 最大风险（PoC 路上）
🔴 真 Claude Code 行为未知（全验证基于 mock，真 API SSE/tool_use 可能与 fixture 微差）；🔴 GUI 能否在机上构建+签名（SwiftUI 层从未 xcodebuild 编译）；🟡 真 key 可得性（卡用户侧）；🟡 真 FP 率未知（当前 0% 是 fuzzing 模拟，真合约可能误报）。

---

## 当前阶段一句话

unix-style 改造 + ADR-030 sieve-updater 客户端 + docs 全部落地（2026-05-05）；2026-05-06~07 修复 SIEVE_HOME 测试隔离 bug、清理归档、重构 README/LICENSE、起草 ADR-031/032（cc-switch 互操作 + Orchesis 借鉴，Proposed 未过）、扩展 SPEC-005 listeners[] 数组。等用户 dogfood 验证 + 运维侧 TODO-13~16 落地。

2026-05-05 单日完成 unix-style 改造 v2.x 全部 5 项（TODO-1~5）并落地 12 个 commits：
ADR-026 multi-listener（含 forwarder path prefix / Config schema / multi-listener accept loop /
协议错位 fail-closed / 审计 provider_id / IPC HealthResult.listeners / doctor 升级）+
ADR-028 IPC 协议中性化 / sieve-ipc 模块化 / sieve decisions CLI / sieve audit CLI。
TODO-6 Network jail enforcement 推后到 v3.x post-GA opt-in。

ADR-030 sieve-updater crate + SPEC-006 + docs 同步 2026-05-05 同日完成（TODO-7~12 + TODO-17/18）。

**2026-06-07 P0 修复（workflow 全量审查发现）**：daemon 全量测试实为 747 passed / 13 failed / 7 ignored——此前本文与 CLAUDE.md 记的「760 passed / 0 failed」把测试**总数**误作通过数（CHANGELOG 当时如实记了 13 failed 但未跟进）。已修复全部 13 个：**簇 A**（产品 bug，9 个）legacy/单-upstream 未声明 protocol 配置下 OpenAI `/v1/chat/completions` 被 ADR-026 协议错位误判 400 → 新增 `Protocol::Auto` 默认态按 path 自适应；**簇 B**（测试 bug，4 个）GUI popup mock 未跳过 SPEC-005 `sieve.hello` 握手帧致假性 426。现 **760 passed / 0 failed / 7 ignored**，fmt/clippy 全绿。详见主里程碑 2026-06-07 条目。

---

## ✅ 已完成 Epic：Dogfood 完全自动化（2026-06-18 单日落地）

> **达成**：821 行手动 checklist → 一键 `scripts/dogfood.sh`（hermetic，零真 API/网络/GUI）+ 零 secret CI。质量基线 **799 passed / 0 failed / 7 ignored**，fmt/clippy 全绿。
> 交付：`sieve-testing` 共享 harness / `dogfood_e2e.rs` 11 headless 场景 / `updater_e2e.rs` 8 闭环测试 / GUI 仓 81 跨仓 fixture / `smoke_test.py --mock-only` / `dogfood.sh` / CI nextest+dogfood job / SPEC-008 HTML。
> **自动化首轮抓出真 bug**：已修 zstd 字节序（热更新生产失效）+ headless CLI 嵌套 runtime panic；待排期 detection 审计未接线 + 6 类跨仓 schema 漂移（见下方 🚫 段，测试已锚定）。
>
> 目标：把 821 行手动联调 checklist（`docs/guides/manual-integration-test.md` 16 节）变成**无真 API key、无真网络、无真 GUI 的一键 + 零 secret CI 端到端测试**。
> 范围决策（2026-06-18 用户签字）：① GUI 用 `sieve decisions` headless CLI 替代当决策客户端，GUI 仓只保跨仓 fixture 一致性测试，不碰 XCUITest；② 本地一键 + CI 无 secret 都要 → 全程 hermetic mock upstream。
> 设计源：5-agent 测绘 workflow（wf_b0644690）综合 —— 去 EXT 后 28/32 能力块（87.5%）可自动化。harness 形态 = 新 dev crate `sieve-testing` + cargo 集成测试为主，`smoke_test.py --mock-only` 为辅。
> 承重事实（已核验）：`SIEVE_CACHE_DIR` 不存在（cache_dir.rs 硬用 $HOME/Library/Caches）；updater `download.rs:27`/`manifest.rs:92` 双 `.https_only()`（mock manifest 需 TLS，但上游 mock 走 plain HTTP，daemon 有 `tls_verify_upstream=false`）；`outbound_block.rs` 已有 DaemonGuard/spawn_mock_upstream 可抽取；无任何现存共享 test crate。

### P0 · 去真依赖 + 基建前置（基线 cargo 1.88 @ ~/.cargo/bin，非 PATH 默认，脚本需自带）
- [x] **P0.1** sieve-updater 加 `SIEVE_CACHE_DIR` env 覆盖（cache_dir.rs `CACHE_DIR_ENV` + override 纯函数 + 3 单测；install_id/runner 经 cache_dir() 自动覆盖）✅ 2026-06-18
- [x] **P0.2** 抽取 `crates/sieve-testing`（paths/upstream(含 responses)/daemon(DaemonGuard+DaemonConfig)/http(含 decode_chunked)/cli(run_sieve_cli)；self_test 端到端通；clippy+fmt 绿）✅ 2026-06-18（**未回改现有测试**——纯去重降级为 P2 可选；decisions/audit 走 run_sieve_cli shell out 真 CLI 而非重写 JSON-RPC）
- [x] **P0.3** `smoke_test.py --mock-only`（本地 mock Anthropic 上游 + tls_verify_upstream=false；fake key→401 注入 cloudflare 头；29/29 通过；**修出 OUT-01 426→auto_redact 过时断言**，见 lessons 2026-06-18）✅
- [x] **P0.4** 测试隔离骨架（SIEVE_HOME tmpdir + wait_for_ipc 轮询，在 sieve-testing 落地）+ `scripts/dogfood.sh` 一键入口（实跑全过）✅
  - 加固：`http_post` 加瞬时连接错误重试（4 次线性退避），消除 daemon 启动窗口/高并发 flake；self_test 3/3 稳过
  - **已知存量 flake**：`outbound_block.rs::r11`（ConnectionReset）在全 workspace 高并发下偶发（隔离 3/3 过，非逻辑 bug）→ 全量回归 780 passed；CI 用 nextest retries 治理（P2）。dogfood.sh 只跑 `-p sieve-testing`+smoke，不触此 flake

### P1 · Mock 服务 + Headless Harness
- [x] **P1.1** Updater mock（**改用更简方案**：plain-HTTP mock 复用 sieve-testing + `tls.rs` 的 `SIEVE_UPDATE_ALLOW_HTTP` 接缝，`#[cfg(debug_assertions)]` release 编译期消除恒 https_only；放弃 rcgen/TLS 复杂度，同等 GA 安全）✅ 2026-06-18
- [x] **P1.2** Headless harness Phase A-D（`dogfood_e2e.rs` 11 测试：出站脱敏 OUT-01 / 入站拦截 IN-CR-01/05 含 content-type 矩阵 / no-client-policy 三策略 + Critical 强制 IPC / mock-GUI 决策流 Allow+Deny / audit 闭环；5/5 稳过）✅ 2026-06-18
  - 🐛 抓出 4 daemon bug：**P0-A** `sieve audit`/`decisions` 嵌套 runtime panic（**我已修**）；**P0-B** detection 审计全未接线（待排期）；harness no_client_policy/wait_for_ipc（已修/已注明）
- [x] **P1.3** 跨仓 17-method fixture 一致性测试入 GUI 仓（81 fixture 字节一致 + Swift 消费测试，`#expect(throws:)` 钉死现状当红线）✅ 2026-06-18
  - 🐛 抓出 6 类跨仓 wire schema 漂移（多个致命，会让真机 GUI dogfood 崩）：hello preset / preset_changed / paused_changed / notify_status_bar / purge_history / evaluate（待排期）
- [x] **P1.4** Updater 闭环 e2e（`updater_e2e.rs` 8 测试：§14.1 install-id / §14.4 fetch→download→sha256→**zstd 解压**→原子落盘 / §14.5 失败模式 / §14.6 公钥 None skip / 遥测 uid 开关；跨平台 hermetic 非 macOS-only）✅ 2026-06-18
  - 🐛 **抓出真 bug**：`install.rs::ZSTD_MAGIC` 字节序反置（`FD2FB528` 磁盘小端应为 `28 B5 2F FD`）→ 真实 zstd 规则包永不解压、原样压缩写盘，sieve-rules 加载必失败。整条 ADR-030 热更新生产中是坏的，现有单测全假阳性掩盖。已修 + 回归断言（见 lessons/CHANGELOG 2026-06-18）

### P2 · CI + 文档 + 收尾
- [x] **P2 CI**：`ci.yml` `test` job 转 cargo-nextest（`.config/nextest.toml` retries=2 治存量 r11 flake）+ doctest；新增 `dogfood` job（build + `smoke_test.py --mock-only`，零 secret）✅
- [x] **P2 文档**：SPEC-008 dogfood 自动化 HTML 设计文档（3 内联 SVG）；CHANGELOG（zstd 修复 + dogfood 基建）；lessons（4 条）；PROGRESS 同步 ✅
- [x] **P2 收尾**：全量回归 **799 passed / 0 failed / 7 ignored** + clippy/fmt 全绿 + `dogfood.sh` 一键终验全过 + manual-integration-test.md 加自动化横幅 ✅ 2026-06-18
- [ ] 可选 IPC（待 P0-B 一起做，不阻塞）：`sieve.export_history` / `sieve.list_inflight`

### 🚫 不自动化（保留手动验收）
- 真 API 出/入站规则触发（OUT-*/IN-CR-* 真 Claude Code 流量）—— Critical FP<0.5% 由离线 vectorscan fuzz + deterministic 样本集保证，不靠 e2e
- GUI 可视层（菜单栏 5 状态/Toast/Settings 6 Tab/Onboarding 权限）—— 人工 dogfood，一轮 2-3h

---

## ✅ 主里程碑

### 2026-06-10~11 SPEC-007 上游代理 + 三项 dev 债清理 + preset 跨仓漂移修复（审查 follow-up）

承接 2026-06-07 审查：先落地 **SPEC-007 上游转发代理**（8 commits，本地未 push 待 dogfood 冲烟），再清剩余可做 dev 债三项 + 修一个由防漂移测试现场抓出的真 bug。质量基线 **775 passed / 0 failed / 7 ignored**，fmt/clippy/deny 全绿；GUI swift test 137 passed。

- **SPEC-007 上游转发代理（HTTP CONNECT + SOCKS5）**：ProxyConnector 替换 Forwarder 底层 connector，TLS 端到端不变（不 MITM）；每 upstream + 全局 + env 优先级链；updater 复用；ADR-033 + 8 commits（`b480736`..`9604926`，**本地未 push**）。解决受限网络 dogfood 第一跳直连即断。Task 11 真机冲烟 = 用户 action item。
- **① Ed25519 GA 编译期密钥 gate（ADR-034，`1fffb8e`）**：`ga_keys` feature 下占位公钥（updater `TRUSTED_PUBKEY=None` / origin `SIEVE_ORIGIN_PUBLIC_KEY` 全零）编译失败（E0080），阻 fail-open 验签进 GA 二进制；alpha build 逐字节不变。修审查 §5 GA 硬阻塞。
- **② 热路径 7 处 expect 重构（`16bd513`）**：daemon proxy_inner 4 处用 `ProxyRequestBody` enum 让非法态类型层不可表达（不可达兜底 500）；socket_server 3 处 lock 改 `into_inner` 毒化恢复。消除 panic=DoS，兑现 CLAUDE.md「请求路径禁 panic」。content-type 路由矩阵 + 全集成测试零回归。
- **③ SPEC §14 fixture 防漂移落地（sieve `2c51c96` + GUI `b3075b0`）**：daemon health fixture 补 `listeners[]` + `schema_v2_fixtures.rs` 落实 §14.1 全 result 双向稳定（此前全单向、名存实亡）；GUI 仓建 `Tests/SieveGUITests/Fixtures/v2/` + `IPCSchemaV2FixtureTests.swift` 消费 daemon 权威 fixture 副本（§14.2）。
- **④ preset mode 跨仓漂移修复（`ae20fd3`）—— 由 ③ 现场抓出**：daemon 漏做 SPEC-005 §5.6 的 v1→v2 `default`→`standard` 重命名（`config::Preset::Default` variant + daemon_control_plane String + setup 模板仍发旧值），GUI 只认 `standard` → 解码失败 → disconnected（**直接卡死真机 dogfood 连通，极可能是 dogfood 跑不起来的元凶之一**）。config enum 改名 Standard（`serde alias="default"` 兼容旧 toml）+ daemon_control_plane normalize + setup 模板 + 文档同步。详见 [lessons.md](lessons.md) 同日条目。
- **GUI 仓 follow-up（待 daemon push）**：`upstream-references.md` pin 待 daemon push 后回填到含 fixture 改动的 commit；SPEC §14.3 release 打包 + sync 脚本自动化属发布基建，记 GA 前 follow-up。

### 2026-06-07 P0 修复：13 个红测试归零（workflow 全量审查）

全量工程状态审查（11-agent workflow）实跑 `cargo test --workspace --no-fail-fast`，发现真实基线是 **747 passed / 13 failed / 7 ignored**，而非本文/CLAUDE.md 长期声称的「760 passed / 0 failed」（760 实为测试**总数**被误作通过数；CHANGELOG 当时如实记了 13 failed 但未跟进修复，随后进入 dogfood 冻结期）。

- **簇 A（产品 bug，9 个测试）**：`config.rs::resolved_upstreams()` 把 legacy `upstream_url` 硬编码成 `protocol: Protocol::Anthropic`，叠加 ADR-026 §决策4 协议错位检查 → 任何 legacy/未声明协议配置发 OpenAI `/v1/chat/completions` 都被 fail-closed 400，违反 ADR-026 §决策1 向后兼容 + PRD §9 #16/#9。修复（产品负责人拍板方向）：`config::Protocol` 加 `#[default] Auto` 第三态，legacy/省略 protocol 映射为 Auto 按 path 自适应、不错位；仅**显式声明** anthropic/openai 才强制错位 400（fail-closed 对显式声明者完全保留）。改 config.rs + daemon.rs（2 处 match 补 Auto）+ sieve-ipc/health.rs 文档，**0 改业务/集成测试**。
- **簇 B（测试 bug，4 个测试）**：`outbound_block.rs::mock_gui_respond_with_ready` 未跳过 SPEC-005 §3 `sieve.hello` 握手帧 → 解析 request_id 失败 → mock 断连 → daemon try_send Closed → fallback Block → 假性 426。产品代码正确（真实 GUI 连接实测 200）。修复：mock 加 method 正过滤只处理 `sieve.request_decision`；连带修正 `outbound_gui_popup_deny_returns_426` 此前的假阳性（崩溃 fallback 恰好=426）。
- **验证**：`cargo test --workspace --no-fail-fast` → **760 passed / 0 failed / 7 ignored**；`cargo fmt --all --check` 干净；`cargo clippy --workspace --all-targets -- -D warnings` 0 warning。下游文档（CHANGELOG/ADR-026/api-reference/data-model）同步更新。

### 2026-05-07 文档 + 配置层后续收尾（commits 2e38e44 / 7cd60e7 / b299463 / 14269f8 / ac12a70 / 7108a45）

- **SIEVE_HOME 透传 bug fix**（2e38e44）：`config.rs::sieve_home()` 对齐 `sieve_ipc` 实现，`audit_db_path` / `sieveignore` 改走该函数；5 个集成测试注入 `SIEVE_HOME` tempdir + `SIEVE_NO_UPDATE=1` + `SIEVE_NO_TELEMETRY=1` 达到完整隔离；workspace 747 passed。
- **tasks/_archive 清理**（7cd60e7）：删除 12 份过期归档；landing-page 占位移除；`.gitignore` 排除 `tasks/_archive/`。
- **README / LICENSE 重构同步 ADR-029**（b299463）：加架构图 + 隐私声明独立成段；client/上游范围精确化（Claude Code / Codex / Cursor + Anthropic / OpenAI / 中转站）；LICENSE 移除 $49/月 硬定价。
- **ADR-005 措辞中性化**（14269f8）：个人身份表述改为「创始人/境内自然人身份」，对外发布友好。
- **ADR-031 cc-switch 互操作 + ADR-032 Orchesis 借鉴策略（Proposed 草案）**（ac12a70）：路径 A/B/C 决策框架已立；调研报告 `docs/research/2026-05-06-orchesis-analysis.md` 新增；待通过前不动代码。
- **SPEC-005 §9.5 listeners[] 扩展**（7108a45）：health 响应 `listeners[]` 数组向后兼容扩展，旧 `listen` 标 deprecated；`manual-integration-test.md` 勾选 fmt/clippy 已过。

### 2026-05-05 sieve-updater 规则下载 + 原子替换闭环收尾

新增 `download.rs`（download_rules，hyper-rustls，50 MiB 上限）/ `install.rs`（7 步原子写入：sha256 + ed25519 + zstd 解压 + .tmp + rename + current.json symlink + latest_version.json）/ runner.rs 接通完整流程 + retry_with_backoff + 两个新常量。error.rs 新增 DecompressFailed / ResponseTooLarge。新增 14 个单元测试（35 total），workspace 760 passed。SPEC-006 §3.3 + §10 补完整；CHANGELOG 同步。热加载留 TODO 待 sieve-rules 接通。

### 2026-05-05 ADR-030 sieve-updater crate 落地（TODO-7~12 + TODO-17/18）
- sieve-updater 独立 crate 骨架设计（manifest 协议客户端 / install-id / 6h 定时器 / 三个 env var / ed25519 + sha256 校验 / 失败重试指数退避）
- SPEC-006 manifest 协议规格 v0.1 新建（~350 行，含 wire format / 流程图 / 测试矩阵 14 项）
- CLAUDE.md 七个 Crate 表（六个 → 七个，新增 sieve-updater 行）
- .cursorrules §3.3 七个 crate 边界表同步
- architecture.md §2.1 新增 sieve-updater 模块行
- api-reference.md §8 manifest 接口章节（原 §8 错误码表改为 §9）
- development.md §13 三个环境变量开发者指南
- deployment.md §13 企业自托管镜像章节
- data-model.md §13 服务端遥测日志 schema（SQL DDL + DAU/MAU/留存 SQL 模板）
- README.md 核心叙事 #3 隐私声明段落
- CHANGELOG.md [Unreleased] sieve-updater + 三个 env var + [update] toml 段 + SPEC-006 条目
- PROGRESS.md TODO-7~12 + TODO-17/18 全部标记 ✅

### 2026-05-05 unix-style 改造 TODO-3a · SPEC-005 协议术语中性化（ADR-028）
- §0 文档定位重写：明确 client-agnostic + 引用 ADR-028
- 段落术语清洗 ~371 处：「GUI 端」→「client 端」/「daemon → GUI」→「daemon → client」/「popup」→「decision request/event」
- gui_popup wire 字段值**保持不变**（向后兼容硬要求），加 ADR-028 标注说明语义中性化
- ui_phase / §3.4 UI 文案 / §6.1.4 recommendation 加 admonition：标注「GUI client 参考实现，headless 可忽略」
- §9 标题「GUI 控制面方法」→「控制面方法」，§10 多 GUI 回声防护 → 多 client 回声防护
- §16 变更记录加 v2.0-adr028 条目；文档头部加协议变更日志
- SPEC-005 净改动 +201 / -170 行
- commit: 69664c3

### 2026-05-05 unix-style 改造 TODO-3b · sieve-ipc crate 内部模块化（ADR-028）
- crates/sieve-ipc/src/protocol.rs 拆分为 protocol/ 子目录（envelope / decision / handshake / rules / audit / health / notify）
- crates/sieve-ipc/src/socket_server.rs → server/socket_server.rs
- crates/sieve-ipc/src/socket_client.rs → client/connection.rs
- 新增 protocol/README.md：SPEC-005 权威源声明 + 零 IO 约束
- lib.rs re-export 100% 兼容 + 向后兼容别名（socket_client / socket_server 路径仍可用）
- 验证：sieve-ipc 单独 106 passed / workspace clippy 0 / fmt clean
- commit: 0ba0350

### 2026-05-05 unix-style 改造 TODO-5 · sieve audit unix-pipeable CLI（ADR-028）
- 新增 sieve audit 子命令：tail [-f] / query [--since DUR] / show <id>
- 直接读 ~/.sieve/audit.db SQLite，不通过 IPC
- jsonl 输出格式（每行一个 JSON object，方便接 jq / fluentd）
- 支持过滤：--severity / --rule-id / --provider-id（v3 schema 新列）
- crates/sieve-cli/src/commands/audit.rs 新增（510 行）+ 7 个单元测试
- commit: 7a1415d

### 2026-05-05 unix-style 改造 TODO-4 · sieve decisions headless CLI（ADR-028）
- 新增 sieve decisions 子命令：watch / show / resolve --approve|--block|--warn
- 新增 sieve start --no-client-policy=auto-block|auto-warn|hold-and-fail-closed flag
- daemon::gated_request_decision 透传 NoClientPolicy：connected_clients == 0 + 非 Critical 时按策略快速返回
- DaemonRunOpts 透传 run → accept_loop → proxy → proxy_inner/proxy_openai
- raw JSON-RPC over UnixStream，不引入 IPC 客户端 typed schema 依赖
- crates/sieve-cli/src/commands/decisions.rs 新增（778 行总，含 daemon.rs 改动）+ 5 个单元测试
- commit: 8717442

### 2026-05-05 unix-style 改造 ADR-026 follow-up · SPEC-003 doctor + SPEC-004 §4.2 + deployment（文档）
- SPEC-003 §4.2b 新增 multi-listener 体检（条件性输出，仅 [[upstream]] > 1 时打印）
- SPEC-004 §4.2.6 加 header routing vs port routing 分工对比表
- deployment.md §6a 新增 Multi-listener 部署章节（5 小节：配置 / 端口规划 / launchd / 故障排查 / Pro Mode 前向引用）
- 共 +135 行，纯文档无代码
- commit: 16bc0e7

### 2026-05-05 unix-style 改造 TODO-2 Stage E + 余 G · 审计 provider_id + doctor multi-listener + data-model + dev guide（ADR-026）
- AuditStore::append 签名升级：加 `provider_id: &str` 参数
- SQLite schema v2 → v3 migration：ALTER TABLE ADD COLUMN provider_id TEXT NOT NULL DEFAULT 'unknown'
- CREATE TABLE DDL + INSERT_SQL 同步加 provider_id 列
- 新增 `crate::audit::SYSTEM_PROVIDER_ID = "_system"` / `UNKNOWN_PROVIDER_ID = "unknown"` 常量
- 透传链路：`RequestCtx.listener_provider_id` → 8 处 audit.append 调用全部加参数
  （含 try_write_graylist / classify_inbound_detections / record_into_sequence_and_detect /
   handle_anthropic_json_inbound / handle_openai_json_inbound 等 sub-flow 函数签名升级）
- gated_request_decision 加 provider_id 参数（3 处调用同步）
- daemon 系统级事件（control plane / oversize / UserRulesReloaded）用 SYSTEM_PROVIDER_ID
- doctor 升级：新增 ADR-026 multi-listener 体检（读 sieve.toml 解析 upstreams 逐 port TCP 探测）
- docs/design/data-model.md §5.1a 加 `[[upstream]]` 数组 schema + §6.2 events 表 v3 + §6.2b migration
- docs/guides/development.md §3.4a 加 multi-listener 配置实战 + 协议错位测试示例
- 13 处 audit.append 调用点全部同步（含 5 处 audit.rs 内部测试）

### 2026-05-05 unix-style 改造 TODO-2 Stage F + 部分 G · IPC HealthResult listeners + 核心文档同步（ADR-026）
- sieve-ipc::ListenerSnapshot 新 struct（port / addr / provider_id / protocol）
- HealthResult.listeners 数组字段；listen 单字段保留为 listeners[0] 别名（向后兼容）
- daemon RuntimeState 加 listeners 字段；handle_health 填充
- daemon::run 启动时按 cfg.resolved_upstreams() 顺序构造 ListenerSnapshot 数组
- 修复 2 处 pre-existing clippy single_match 触发问题（end_to_end.rs）
- CHANGELOG.md 加 [Unreleased] 2026-05-05 unix-style 改造段（4 个 ADR / 2 个 BREAKING / 1 个 Fix）
- docs/api/api-reference.md §3.3.1 加 Multi-listener 配置实战 schema + 兼容性说明
- docs/design/architecture.md §1.1 加 ADR-026 多 listener 部署拓扑说明
- 验证：workspace 713 passed / clippy 0 / fmt clean
- **GUI 仓 follow-up**：sieve-gui-macos 仓 Swift 代码读 health.listeners 数组（向后兼容期内 listen 单值仍发）

### 2026-05-05 unix-style 改造 TODO-2 Stage B/C/D · multi-listener + 协议错位拒绝（ADR-026）
- ListenerSpec struct + 拆 accept_loop 独立 async fn
- daemon::run 重构：cfg.resolved_upstreams() → Vec<ListenerSpec> → 多 bind（fail-fast）→ spawn N accept_loop
- proxy_inner 协议错位 fail-closed 校验（Anthropic listener 收 /v1/chat/completions → 400；反向亦然）
- build_protocol_mismatch_400 helper（400 + sieve_blocked event payload）
- RequestCtx 加 listener_protocol + listener_provider_id（8 处 ::new + 5 处 destructure 同步）
- 向后兼容：旧 sieve.toml 走 resolved_upstreams 单元素映射，行为不变
- 验证：sieve-cli 226 passed / workspace 713 passed / clippy 0 / fmt clean

### 2026-05-05 unix-style 改造 TODO-2 Stage A · Config schema（ADR-026）
- Protocol enum + UpstreamListener struct + [[upstream]] 数组
- Config::resolved_upstreams 兼容旧字段映射
- check_safety_invariants 拆出可单测函数（端口冲突 / 非 loopback bind 检测）
- 13 个新测试（共 226 sieve-cli passed）

### 2026-05-05 unix-style 改造 TODO-1 · forwarder path prefix 修复（ADR-026）
- `Forwarder` 加 `upstream_path_prefix` 字段，`Forwarder::new` 解析 + trim 末尾 `/`
- `rewrite_uri` 拼接 prefix（DeepSeek `/anthropic` 等中转站现已可用）
- 新增 5 个测试 case：path / path+query / trailing slash / multi-segment / Host header 不变量
- 对外 `upstream_host()` API 零 breaking，5 个调用点未改动
- sieve-core: 173 passed / clippy 0 warnings / fmt clean

### 2026-05-03 v2.0+ 兼容扩展 + 业务层完整化
- SPEC-005 §11A sieve.list_rules + §11B sieve.purge_history（不 bump version）
- daemon 实现两个新 method
- recommendation 字段 daemon 业务层真实注入
- fixtures 81 条（17+2 method × 3 档）

### 2026-05-03 e2e 测试 + 业务层 polish
- 端到端集成测试 harness（6 场景：握手 / heartbeat / 单 issue / merged / 重连丢 inflight / set_paused 串行化）
- pre-existing flake canary_token_hits_out01 修复
- audit oversize callback 注入

### 2026-05-02..03 P1-5 wire DTO 拆分（最大改造）
- 内部 DecisionRequest 与 wire DTO 分离
- 单 issue 平铺 / 多 issue merged + issues[]
- created_at → received_at_daemon
- origin_request_id 真实透传

### 2026-05-02 P0 全部 + P1 大部分
- 帧读取 FrameReader + memchr（移除无界 BufReader::lines）
- sieve.hello + sieve.heartbeat + socket 0600
- 字段对齐：paused_until / origin_request_id / HealthResult 拆分 / NotifyKind / parse_error / fan-out 串行化 / write timeout / pending-leak

### 2026-05-02 协议骨架（4 组双侧同步对）
- 协议版本号 v1 → v2
- 方法名 sieve.* 前缀
- 错误码段位 -32100~99
- decision_response.result required 字段

详细 commit 列表见 git log。

---

## 🚧 进行中
（无 — TODO-1 / TODO-2 已完成；TODO-3~6 待用户验证 TODO-2 联调反馈后启动）
  - [x] Stage A / B+C+D / E / F / G 核心全部完成
  - [ ] Stage G 余项（仅文档 follow-up，不阻塞）：SPEC-003-sieve-setup-tool.md doctor 5 项更新 / SPEC-004-multi-agent-setup.md §4.2 header vs port routing 分工 / deployment.md 多 listener 部署章节
  - **GUI 仓 follow-up**（不在本仓）：sieve-gui-macos 仓 Swift 代码读 `health.listeners` 数组 + SPEC-002 同步（向后兼容期内 `listen` 单字段仍发，不阻塞）

---

## ⏭ 下一步（等用户联调反馈）

### 用户介入项
- 真实 dogfood：启 daemon + GUI 跑 HIPS / Settings / History 流程
- 反馈 bug 或 UX 调整

### unix-style 改造（v2.x，与联调并行）

> 关联 ADR：[ADR-026](../docs/design/ADR-026-port-based-listener-routing.md) / [ADR-028](../docs/design/ADR-028-ipc-protocol-neutralization.md)；v3.x 关联 [ADR-027](../docs/design/ADR-027-network-jail-enforcement.md)
> 设计源：2026-05-05 主线讨论 ——「sieve 想做 iptables-like 工具，UI 是众多 client 之一不是特权」

#### P0 · 基础设施（先做，其他依赖）

- [x] ~~**TODO-1 修 forwarder path prefix bug**~~ ✅ 完成 2026-05-05（见「主里程碑」）

- [x] ~~**TODO-2 Port-based multi-listener**~~ ✅ 完成 2026-05-05（Stage A/B+C+D/E/F/G 核心全部落地，见「主里程碑」）

#### P1 · 协议中性化（GUI 不再特权）

- [x] ~~**TODO-3a SPEC-005 协议术语中性化**~~ ✅ 完成 2026-05-05（commit 69664c3，见「主里程碑」）
- [x] ~~**TODO-3b sieve-ipc 内部模块化**~~ ✅ 完成 2026-05-05（commit 0ba0350，见「主里程碑」）
- [x] ~~**TODO-4 Headless decision CLI**~~ ✅ 完成 2026-05-05（commit 8717442，见「主里程碑」）
- [x] ~~**TODO-5 Audit 层 unix-pipeable**~~ ✅ 完成 2026-05-05（commit 7a1415d，见「主里程碑」）

#### P2 · 网络层兜底（v3.x post-GA opt-in，不阻塞 GA）

- [ ] **TODO-6 Network jail enforcement**（3-5 天，v3.x）
  - 新增 `_sieve` 系统用户，daemon 跑在该用户下
  - macOS pf / Linux nftables uid-based egress filter（仅 _sieve 可访问 LLM endpoint:443）
  - 不解 TLS、不装 CA、不动 trust store —— PRD §9 #12 不破
  - 新增 `sieve setup --jail` / `sieve doctor --jail` / `sieve uninstall --jail` 子命令
  - hostname 列表 ship 在 sieve-rules，签名分发；用户可加 `~/.sieve/extra-hosts.txt`
  - **默认关、opt-in、不阻塞 GA**
  - 营销卖点：「Sieve Pro Mode」差异化定位
  - 关联：ADR-027

#### 工作量与节奏（实际）
- v2.x（GA 前）：TODO-1~5 全部完成 2026-05-05（单日 12 commits，主上下文 + 4 子代理并行）
- v3.x（GA 后，dogfood 验证后）：TODO-6 约 3-5 个工作日

---

### 商业化 + 遥测决策落地（ADR-029 / ADR-030 / ADR-003 amended，2026-05-05 立项）

> 关联 ADR：[ADR-029](../docs/design/ADR-029-free-first-defer-monetization.md) 装机量优先 / [ADR-030](../docs/design/ADR-030-update-telemetry-channel.md) 更新通道遥测 / [ADR-003 amended](../docs/design/ADR-003-local-only-no-cloud-verifier.md) 网络边界修订
> 立项原因：ADR-029 把装机量定为 GA 前唯一指标，需要 ADR-030 manifest 协议提供数据来源；ADR-030 修订 ADR-003 「禁 telemetry」反模式条款。决策已 Accepted，落地工作待 GA 前完成。

#### P0 · 代码侧（GA 前必须）

- [x] ~~**TODO-7 sieve-updater crate 骨架**~~ ✅ 完成 2026-05-05
  - 新建 `crates/sieve-updater/`（独立 crate，GUI 仓后续可复用）
  - CLAUDE.md 「六个 Crate」段同步成「七个 Crate」
  - .cursorrules §3.3 + architecture.md §2.1 同步
  - 关联：ADR-030 §待决项 #5

- [x] ~~**TODO-8 manifest 协议客户端**~~ ✅ 完成 2026-05-05
  - `GET https://updates.sieveai.dev/v1/manifest?v=&os=&arch=&uid=&ch=`（仅 TLS 1.2+，无 cookie/Auth）
  - 解析 server response（rules + client + next_check_after_seconds）
  - sha256 + ed25519 签名校验（编译期硬编码公钥，参考 ADR-006）
  - 失败重试策略（指数退避 1s/4s/16s × 3）
  - 关联：ADR-030 §3

- [x] ~~**TODO-9 install id 生成与持久化**~~ ✅ 完成 2026-05-05
  - 首次启动生成 UUIDv4（纯随机，不掺设备/账号信息）
  - 持久化路径：macOS `~/Library/Caches/sieve/install-id`（首发）
  - 文件权限 0600；`cache_dir()` 跨平台抽象（Phase 2 Linux/Windows 路径预留）
  - 关联：ADR-030 §2

- [x] ~~**TODO-10 三个环境变量解析**~~ ✅ 完成 2026-05-05
  - `SIEVE_NO_UPDATE` / `SIEVE_NO_TELEMETRY` / `SIEVE_UPDATE_URL`
  - 启动 banner 打印；优先级 env > toml > default
  - 关联：ADR-030 §5

- [x] ~~**TODO-11 6h 定时器 + 启动立即查一次**~~ ✅ 完成 2026-05-05
  - 启动立即一次 + 6h 周期触发
  - 服务端 `next_check_after_seconds` 动态覆盖
  - 关联：ADR-030 §1

- [x] ~~**TODO-12 sieve.toml `[update]` 段**~~ ✅ 完成 2026-05-05
  - `enabled` / `telemetry` / `url` / `check_interval_hours` / `channel`
  - env var 优先级始终高于 toml
  - 关联：ADR-030 §7

#### P1 · 运维侧（GA 前必须）

- [ ] **TODO-13 域名注册**（依赖 ADR-005 海外主体落地）
  - `updates.sieveai.dev`（manifest，**不挂 CDN**）
  - `cdn.sieveai.dev`（规则正文 zst）
  - 关联：ADR-005 / ADR-030 §待决项 #1

- [ ] **TODO-14 ed25519 签名密钥管理**（1 天）
  - HSM / 单独 build 机 / 1Password Secrets / GCP KMS 之一
  - 写入 ADR-006 follow-up（amendment 或新 ADR）
  - 密钥泄露 = 规则分发被劫持的最大风险点
  - 关联：ADR-030 §待决项 #2 / ADR-006

- [ ] **TODO-15 服务端实现**（2-3 天）
  - 倾向 Cloudflare Workers + KV / D1（零运维 + manifest 接口天然反 DDoS）
  - 备选：自托管 Go / Rust
  - 服务端日志只存 `ts | uid | v | os | arch | ch | country(geoip)`，丢原始 IP（geoip 解析后丢弃，或哈希后保留 ≤7 天反滥用）
  - DAU / MAU / 留存 / 版本分布 / 平台分布全从这一张表算
  - 关联：ADR-030 §4 / §待决项 #4

- [ ] **TODO-16 ch 通道策略**（决策）
  - 推荐先 stable 单通道，Phase 2 再加 beta
  - 关联：ADR-030 §待决项 #3

#### P2 · 文档侧（GA 前必须）

- [x] ~~**TODO-17 SPEC-006 manifest 协议详细设计**~~ ✅ 完成 2026-05-05
  - 新建 `docs/specs/SPEC-006-update-and-telemetry.md`（v0.1，~350 行）
  - 覆盖：wire format / install-id / env var / 签名校验 / 失败策略 / 测试矩阵（14 项）
  - 关联：ADR-030 §需要更新的文档

- [x] ~~**TODO-18 docs 同步**~~ ✅ 完成 2026-05-05
  - api-reference.md 新增 §8 manifest 接口章节（原 §8 错误码表改为 §9）
  - development.md 新增 §13 三个环境变量章节
  - deployment.md 新增 §13 企业自托管镜像章节
  - data-model.md 新增 §13 服务端遥测日志 schema（SQL DDL + 指标模板）
  - README.md 核心叙事 #3 后加隐私声明段落
  - CLAUDE.md 七个 Crate 表 + architecture.md §2.1 + .cursorrules §3.3 同步

- [ ] **TODO-19 PRD §11 商业化策略修订**（半天，可与 PRD v2.1 一起做）
  - 引用 ADR-029 替换原 §7 定价表
  - PRD §1.2 第 3 句「完全本地运行,从不上传你的数据」精确化（参考 README §核心叙事第 3 句已修订版本）
  - PRD §9 #2「绝不联网做 verifier」明确边界（参考 CLAUDE.md 已修订版本）
  - PRD §11.2 ToS 同步 ADR-030 隐私文案
  - 关联：ADR-029 §需要更新的文档

#### 产出物（已落地）
- 3 份 ADR：ADR-026 / ADR-027 / ADR-028
- SPEC-005 v2 协议中性化（commit 69664c3）
- sieve-ipc crate 模块化（commit 0ba0350）
- SPEC-003 / SPEC-004 / deployment.md 多 listener 同步（commit 16bc0e7）
- data-model.md / api-reference.md / architecture.md / development.md ADR-026 同步
- CHANGELOG `[BREAKING]` Config schema + IPC schema + audit schema 全部记入

#### 用户验证清单（当前等用户跑）

**完整 step-by-step checklist**：[docs/guides/manual-integration-test.md](../docs/guides/manual-integration-test.md)（16 节,按 §1-§14 逐项勾选 + §15 DoD;全过即 dogfood 就绪）

快速摘要：
- §1 基线：`cargo fmt/clippy/test/deny/build` 全绿 → workspace **760 passed**（含 sieve-updater 35 测试）+ 七个 crate 都在
- §2 旧 schema 向后兼容（旧 `upstream_url` + `port` 仍可用）
- §3 multi-listener 配置（3 listener bind + 端口冲突 fail-fast）
- §4 协议错位 fail-closed（4 个子 case：path mismatch + X-Sieve-Provider 不能 override）
- §5 doctor multi-listener 体检（条件性输出）
- §6 sieve audit tail / query / show（jsonl 接 jq / fluentd）
- §7 SQLite v3 schema 直查（provider_id 分布）
- §8 v2 → v3 migration（如有老 audit.db）
- §9 sieve decisions watch / show / resolve + `--no-client-policy` 三种策略
- §10 forwarder path prefix（DeepSeek 中转站）
- §11/§12 SPEC-005 中性化 + sieve-ipc 模块化（文档/结构级）
- §13 GUI 仓 follow-up
- **§14 sieve-updater 客户端独立闭环（ADR-030/SPEC-006）—— 7 个子节**：14.1 install-id 首启+幂等+删后重生 / 14.2 三个 env var（**SIEVE_NO_UPDATE banner 必可见**）/ 14.3 本地 mock + caddy https 反代 / 14.4 完整闭环（fetch→download→sha256→ed25519 skip WARN→zstd→tmp+rename+symlink+latest_version.json）/ 14.5 三种失败模式不击穿 daemon / 14.6 公钥 None 占位 WARN 必可见 / 14.7 清理
- §15 DoD（全部勾选 → dogfood 就绪）

### 更新通道 + 遥测（ADR-029 / ADR-030，GA 前必须落地）

> 关联 ADR：[ADR-029](../docs/design/ADR-029-free-first-defer-monetization.md)（装机量优先，延后商业化）/ [ADR-030](../docs/design/ADR-030-update-telemetry-channel.md)（更新通道复用为遥测信标）/ [ADR-006](../docs/design/ADR-006-sigstore-reproducible-build.md)（签名分发）
> 设计源：2026-05-05 主线讨论 ——「免费优先 + 用更新检查作为 DAU 信号 + Install UUID + 三个 Unix-style env var 开关」

#### P0 · ADR-030 待决项（动手前必须确认，每项都有默认推荐，确认即可推进）

- [x] ~~**1. 根域名注册**~~ ✅ 已确认 = `sieveai.dev`（2026-05-05 用户签字）。子域 `updates.sieveai.dev`(manifest)/`cdn.sieveai.dev`(规则)/`security@sieveai.dev`(漏洞)。DNS / MX 注册待 ADR-005 海外主体落地后执行。

- [ ] **2. ed25519 签名密钥管理**
  - 风险：密钥泄露 = 全网 Sieve 用户被推恶意规则（信任根）
  - 默认推荐：**GCP KMS**（密钥永不出 HSM / IAM 审计 / 零运维 / 每月签名几次成本可忽略 / 启用版本化 + 跨区域复制做备份）
  - 备选：1Password Secrets（最简单，但密钥需联网取）/ YubiKey（物理不可导出但单点故障）/ air-gapped（一人公司不现实）
  - 落地后写入 [ADR-006](../docs/design/ADR-006-sigstore-reproducible-build.md) follow-up

- [ ] **3. 服务端实现栈**
  - 默认推荐：**Cloudflare Workers + D1**（零运维 / 天然 anti-DDoS / 免费层够前 6 个月装机量 / 与 CDN 一站式）
  - 备选：自托管 Go / Rust（完全可控但要管服务器 + TLS + 监控 + 备份，一人公司心智成本高）
  - 后期日志量大了再迁 ClickHouse / BigQuery，迁移代价可接受

- [x] ~~**4. 客户端 crate 归属**~~ ✅ 已确认 = 新增 `sieve-updater` 独立 crate（2026-05-05 落地,见 commit 待用户验证 + workspace 七个 crate）

- [ ] **5. 发布通道首发策略**
  - 默认推荐：**首发 stable 单通道**（实现最简，符合 ADR-011 GA 节奏）；`?ch=` 参数保留预留扩展（默认 `stable`，服务端忽略其他值）
  - 备选：首发就引入 beta（双套规则文件 + 双套签名 + 用户切换 UI，工程量翻倍）

> **当前剩余决策**：GCP KMS（推荐,待 TODO-14）/ Cloudflare Workers + D1（推荐,待 TODO-15）/ stable 单通道（推荐,待 TODO-16）。
> 已落地：域名 `sieveai.dev` / `sieve-updater` crate。

#### P0 · 客户端实现（首发 macOS）—— ✅ 全部完成 2026-05-05

- [x] ~~新建 SPEC-006-update-and-telemetry.md~~ ✅ 620 行 / TODO-17
- [x] ~~`cache_dir() -> PathBuf` 跨平台抽象~~ ✅ macOS / Linux / Windows 三分支
- [x] ~~Install UUID 模块（UUIDv4 / 0600 / 删后重生）~~ ✅
- [x] ~~6h 定时器（tokio interval）+ 启动立即查一次~~ ✅
- [x] ~~manifest GET 请求构造（5 query 参数）+ TLS 1.2+~~ ✅
- [x] ~~manifest 响应解析 + ed25519 签名校验 + sha256 校验~~ ✅（公钥 `None` 占位 + WARN 不静默通过，待 TODO-14 GCP KMS 落地填真值）
- [x] ~~**规则文件原子替换 stub**~~ ✅ 完成 2026-05-05：download.rs + install.rs + runner 接通，5 单元测试通过，SPEC-006 §3.3 / §10 收尾，CHANGELOG 同步
- [ ] 三个环境变量解析与优先级（env > toml > default）
- [ ] `SIEVE_NO_UPDATE` 启动 banner 明示打印
- [ ] `[update]` 段加入 sieve.toml schema（enabled / telemetry / url / check_interval_hours）
- [ ] 失败重试策略（指数退避 + 最大重试次数 + 全失败时不影响 daemon 启动）

#### P0 · 服务端实现

- [ ] manifest 接口骨架（不挂 CDN 的自有服务器 / Cloudflare Worker）
- [ ] 日志字段写入：`ts | uid | v | os | arch | ch | country(geoip)`
- [ ] 原始 IP 不落盘（geoip 解析后丢弃，或哈希后保留 ≤7 天硬删）
- [ ] DAU / WAU / MAU / 留存曲线 / 版本分布 / 平台分布 SQL 模板
- [ ] 简单反滥用与限流（同一 IP 每分钟请求上限）
- [ ] 规则正文 CDN 上架 + ed25519 签名 + sha256 manifest 字段填充

#### P0 · 文档同步 —— ✅ 全部完成 2026-05-05（除 PRD §11）

- [x] ~~api-reference.md 新增 §8 manifest 接口章节~~ ✅
- [x] ~~development.md 加三个环境变量说明（§13）~~ ✅
- [x] ~~deployment.md 加企业自托管镜像章节（§13 + §10 旧 env 名清理）~~ ✅
- [x] ~~data-model.md 加服务端日志表 schema（§13）~~ ✅
- [x] ~~README.md 加隐私声明文案~~ ✅
- [x] ~~CHANGELOG `[Unreleased]` 加 manifest 协议 + 三个 env var 条目~~ ✅
- [ ] **PRD §11 商业化策略章节修订**（TODO-19,推后到 PRD v2.1 一起做）

#### 工作量预估
- 客户端：约 3-5 天（含 SPEC-006 起草 + 实现 + 单元测试 + 集成测试）
- 服务端：约 2-3 天（如选 Cloudflare Workers）
- 文档：约 1 天

#### 完成定义
- 本地启动 daemon → 6h 内能看到自身 install-id 出现在服务端日志
- `SIEVE_NO_UPDATE=1 cargo run -p sieve-cli -- start` 启动 banner 明示禁用 + 不发任何更新请求
- `SIEVE_NO_TELEMETRY=1` 启动后请求中无 uid 字段
- `SIEVE_UPDATE_URL=http://localhost:8080/v1/manifest` 能切到本地 mock 服务器
- 删除 install-id 文件后重启，下次请求带新 UUID
- 服务端能跑出 DAU / 留存曲线

---

### 已知小尾巴（不阻塞联调）
- direction 字段在 sieve-core/pipeline 某孤立路径未被完整测试覆盖
- list_rules daemon 端从 LayeredEngine 取规则的实际列表完整性需联调验证
- purge_history daemon 端实际 SQLite delete events 行为需联调验证

### 发布前（Phase 1C，等联调通过后）
- Tier 1 sigstore reproducible build 跑通 release artifact 流程
- GA 准备：Week 12 一次性公开 repo

---

## ✅ 已修：dogfood 自动化首轮抓出的全部真 bug（2026-06-18 同日全修）

> 第二轮目标「修正所有问题」已达成。自动化抓出的 bug 全部修复，测试断言从「锚定缺陷」翻转为「正向验证」。质量基线 **799 passed / 0 failed**，dogfood.sh 全绿，GUI swiftc 探针 21/21。

- ✅ **`install.rs::ZSTD_MAGIC` 字节序反置**（规则包永不解压，热更新生产失效）
- ✅ **`sieve audit`/`sieve decisions` 嵌套 runtime panic**（headless CLI 完全不可用）
- ✅ **detection 审计接线**：`gated_request_decision` 写 `DecisionMade`（所有 gui_popup 决策 + no-client-policy）、出站脱敏(Anthropic+OpenAI)写 `OutboundRedacted`；`sieve audit query` 现查得到核心流量。窄路径（inbound hook-mark / status-bar-only / 出站 direct-block）暂留——当前规则集（OUT 全 redact/gui_popup/status_bar，无纯 Block）不触发 direct-block，决策审计已覆盖 IN-CR gui_popup。
- ✅ **6 类跨仓 wire schema 漂移**（曾阻塞真机 GUI dogfood）：以 SPEC-005 为权威——daemon 侧 D4(paused_changed 补 source)/D6(purge_history purged_at i64→ISO)；GUI 侧 D3(删 preset)/D5(重写 EventNotifyParams)/D7(would_recommendation→对象)；D1/D2 校正陈旧 fixture。两仓 fixture 字节对齐 + 断言翻转。
- ⏭ **次要待办（不阻塞）**：`wait_for_ipc` 探测连接污染 `connected_clients`（daemon IPC eager 清理 gui_writers）；审计窄路径补全；fixture 防漂移测试升级为双向稳定（serialize↔deserialize）。

## 🚫 阻塞 / 等决策
（无）

---

## 完成定义（DoD，每项任务通用）

- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 全过
- 涉及 SSE / 规则 / 工具调用判定的改动 → 对应 fuzz / 单元测试已加
- PRD §9 十六条硬约束未被绕过
- CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- 关联文档（requirements / design / api / SPEC）已同步
- **本文件已勾选 + 移项至「已完成」**

详见 `.cursorrules §五` + 项目根 `CLAUDE.md`。
