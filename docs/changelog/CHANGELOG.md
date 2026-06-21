# Changelog

本文件记录 Sieve 所有显著变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

> v1 公开 API 在首个稳定版发布后冻结，破坏性变更走 SemVer。冻结范围参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)。

---

## [Unreleased] — 2026-05-07

### Security — IN-CR-01 地址替换在两条 JSON 路由 by-construction 不设防（v1.5.4 同型 P0，2026-06-20，ADR-025 / PRD §9 #16）

- **缺口**：IN-CR-01 地址替换走 `InboundFilter::observe_event`（响应文本类），此前只挂 SSE；`handle_anthropic_json_inbound` / `handle_openai_json_inbound` 只调 `on_tool_use_complete`、对 text 块 `continue`，两条非流式 `application/json` 路径**从不扫 assistant 文本** → `stream=false` 时地址替换攻击零拦截（护城河缺口，与 v1.5.4 同型）。`content_type_matrix.rs` 四路由既有测试全用 tool_use payload，JSON 文本路径是无测假绿。
- **修复**：抽出 `InboundFilter::scan_assistant_text` 作为 SSE `observe_event` 与两条 JSON handler 的共享文本检测核心；JSON 路径用 `anthropic_completion_text` / `openai_completion_text` 提取 assistant 文本喂入，`HoldForDecision` 降级为 fail-closed `Block`；新增 `classify_json_inbound_detection` 消除两 handler 重复处置逻辑。
- **测试**：四路由 TEXT-trigger 集成测试齐全（JSON 两路由接线前 RED、接线后 GREEN）：`ucsb_attack_1_address_substitution_blocked`（A-SSE）/ `openai_prompt_address_seed_blocks_address_substitution`（O-SSE）/ `content_type_matrix_anthropic_json_in_cr01_text_substitution`（A-JSON）/ `content_type_matrix_openai_json_in_cr01_text_substitution`（O-JSON）。
- **文档**：`architecture.md §7.5` 修复方案补步骤 6（原 §7.5 把 IN-CR-01 与 IN-CR-05 捆绑宣称"已闭合"，对文本类 IN-CR-01 不准确）。

### Added — 加密审计日志 full 档（write-only logging）（2026-06-19，ADR-037）

- **新增三档 logging level（`off` / `metadata`（默认）/ `full`（opt-in）），扩展现有审计模型而非另起平行模型**。`[audit].level` 默认 `metadata` = 当前已发布行为（`audit.db` 现已在写最小脱敏元信息），零行为变化；`off` 什么都不留，`full` 由用户显式 opt-in 开启。本 ADR 真正的**新增能力是 `full` 档**，且默认关闭、完全可配置。
- **`full` 档落地 write-only logging（只写不可读的归档）**：归档**只存脱敏后内容**（红线——脱敏先于落盘，绝不写原始流量），用 **age recipient-only 混合加密**——daemon 进程**只持公钥**，运行时只能往里追加、无法解开任何历史归档；解密私钥经 **scrypt** 口令派生离线保护，与 daemon 物理隔离。哈希链（hash chain）防归档被静默改写/删条，叠加保留期（retention）自动清理过期内容。
- **威胁模型按条兑现**：运行时被攻陷（live malware）也读不出历史归档（daemon 无私钥）；整盘被拷 / `~/.sieve/` 误同步云盘 / 误 `git add` 拿到的都是密文。**口令丢失 = 归档永久不可读，by design**——这是安全属性不是缺陷。
- 详见 [ADR-037](../design/ADR-037-encrypted-audit-log.md)（关联 ADR-003 完全本地 / ADR-016 脱敏路径 / SPEC-009 工程级详细设计，Phase 2 落地）。

### Added — 超额计费检测（独立 token 核算对抗 relay）（2026-06-19，ADR-038）

- **新增对经过流量的独立 token 核算，交叉比对中转站（relay）声明的 `usage`，偏差超容差报警**——戳穿「relay 把 `usage` 乘 1.5 多收钱」的灰产。目标是**异常检测而非逐 token 精确对账**：乘 1.5 = 多报 50%，远高于 tokenizer 噪声，藏不住。`[billing_check].enabled` 默认 **false**，完全 opt-in、可配置。
- **上游信任分级 `official` / `relay`**：按 `[[upstream]]` 的 url host **自动派生**（`api.anthropic.com` / `api.openai.com` → `official` 直接采纳 `usage`；其余 → `relay` 视为未经验证的声明、独立核算比对），可显式 `trust` 覆盖；**无法判定时保守按 `relay`**（fail-closed 倾向）。复用 ADR-026 `provider_id` 做审计归因。
- **独立计数永远优先权威信源、不手搓 tokenizer**：OpenAI 用 **tiktoken**（GPT-4o+ `o200k_base` / 老模型 `cl100k_base`，含 per-message 框架开销，`tiktoken-rs`）；Anthropic 无公开 tokenizer，**默认本地近似估算**（明确标为估算、零新增出站，对抓「乘 1.5」量级够用），调官方 `count_tokens` 直连拿权威输入数为**独立开关 `count_tokens_optin`、默认关**。
- **容差默认 15%**：偏差超阈值**报警不阻断**（不打断工作流）。`usage.db` 统计**严格本地、永不上传**（呼应 SPEC-006 never upload 隐私承诺 + PRD §9 #2 绝不联网做 verifier）。
- 详见 [ADR-038](../design/ADR-038-overbilling-detection.md)（关联 ADR-003 / ADR-026 信任分级前提 / SPEC-006 隐私承诺 / SPEC-010 工程级详细设计，Phase 2）。

### Security — write-only logging 抗运行时攻陷 + usage 统计永不上传（2026-06-19，ADR-037 / ADR-038）

- **加密审计 `full` 档以 write-only logging（daemon 只持公钥 + 私钥 scrypt 离线保护 + 哈希链）兑现「运行时被攻陷也读不出历史归档」**：即便 live malware 拿下 daemon 进程，也无私钥解开任何过往归档，从根上避免 `full` 档把本地变成明文流量库（LiteLLM 投毒事件的同构风险）。落盘内容恒为脱敏后副本，原始流量绝不触盘。
- **超额计费检测的 `usage.db` 统计严格本地留存、永不出网**：核算与比对全程不联网，不破 PRD §9 #2「绝不联网做 verifier」与 SPEC-006「never upload」承诺。

### Added — 一行自校验安装：curl|bash + Homebrew + cargo install（2026-06-19，ADR-036）

- **新增自校验 `curl|bash` 安装器 `scripts/install.sh`**：一行装 `sieve` CLI/daemon——`set -euo pipefail` + curl `--proto '=https' --tlsv1.2 -fsSL`；下载 release 裸二进制 + 同名 `.sigstore.json`（cosign keyless bundle），**落地前自动校验**（有 cosign 用 sigstore 验签 / 无 cosign 回退对照 `SHA256SUMS` 的 sha256 + 明确警告），**任一校验失败立即退出、不安装（fail-closed）** → 装 `~/.local/bin` → 提示 `sieve setup`。只装 CLI/daemon，**GUI 不走 curl|sh**（继续签名 .dmg / brew cask）。macOS 工作，Linux 预留位。
- **新增 Homebrew packaging（`packaging/homebrew/`）**：formula `sieve.rb`（CLI）+ cask `Casks/sieve.rb`（GUI .app）+ `README.md`（复制到独立 tap `SieveAI-dev/homebrew-sieve` + 发版填 sha256 流程）。brew 原生 sha256 自动校验。sha256 当前为 pre-GA 占位（全零，故意 fail-closed）。
- **cargo install metadata**：补 `sieve-cli` 的 homepage/keywords/categories + workspace `repository` 填实 owner（`<owner>` → `SieveAI-dev`）+ 加 `homepage`。`cargo install --git https://github.com/SieveAI-dev/sieve sieve-cli` 现可用；`cargo install sieve`（crates.io）标注 Phase 2（`publish=false`，需 workspace 全部 crate 发版）。
- **手动 cosign 验签从安装主路径下沉为"可选"**：README.md / README.zh-CN.md / deployment.md 安装段重排——一行命令（brew / curl|bash / cargo）打头，手动 cosign + 可复现构建下沉到"给偏执狂的完整验证"小节；删去 README 中英 "Sieve does not provide a `curl … | sh`…" 说教段。签名 / 可复现构建本身（ADR-006）与 fail-closed 校验**一字不动**，只是把"用户手动验"自动化为"安装器替你验"。详见 [ADR-036](../design/ADR-036-self-verifying-installer.md)。

### Fixed — IPC accept 循环遇瞬态错误即退出的可用性单点（工程评估抓出，2026-06-19）

- **修复 `sieve-ipc::socket_server::IpcServer::run` 的 accept 循环遇任何错误即 `break` 终止整个循环的缺陷**（行为变更/可用性）：原 `Err(e) => { error!(...); break; }` 使单次**瞬态**错误——进程/系统 fd 耗尽（EMFILE/ENFILE）或对端在 accept 完成前断开（ECONNABORTED）——永久击穿整个 IPC 控制面 daemon，GUI 弹窗 / hook pending / reload 全部失效且 daemon 无自愈、需重启恢复；对 fail-closed 安全代理是可用性单点。修复：参照 hyper server 的 accept 错误分类，连接级瞬态错误（`is_connection_error`：ConnectionRefused/Aborted/Reset）立即重试、其余错误（典型 fd 耗尽）退避 `ACCEPT_ERROR_BACKOFF`=100ms 后重试避免 busy-loop，**任何 accept 错误都不再退出循环**；真正不可恢复的 listener 损坏交由 launchd KeepAlive 重启进程。新增 2 个单测覆盖错误分类（连接级 → 立即重试 / 资源类 EMFILE=24 → 退避）。

### Fixed — 跨仓 IPC wire schema 漂移 ×6 + detection 审计接线（dogfood 自动化抓出，2026-06-18）

- **修复 6 类 daemon↔GUI wire schema 漂移**（跨仓 fixture 一致性测试抓出，多个致命——daemon 推这些通知时 GUI 解码失败 disconnected、阻塞真机 GUI dogfood）。以 SPEC-005 为权威源逐个判定 canonical：
  - **D4 `sieve.paused_changed` 缺 `source`**（SPEC §10.2 required）→ daemon `PausedChangedNotify` 补 `source` 字段。
  - **D6 `sieve.purge_history.purged_at` 类型错**：daemon 发 epoch ms 数字，SPEC §11B 规定 `Timestamp`(ISO8601) → daemon `PurgeHistoryResult.purged_at` `i64`→`DateTime<Utc>`(ISO 串)。
  - **D3 `sieve.preset_changed`**：GUI 多要 SPEC 无的 `preset` 字段 → GUI `PresetChangedParams` 删 `preset`。
  - **D5 `sieve.notify_status_bar`**：GUI `EventNotifyParams` schema 整体错位 → 重写为 daemon `StatusBarNotify`(notify_id/created_at/kind/title/detail/rule_id/auto_dismiss_seconds)。
  - **D7 `sieve.evaluate.would_recommendation`**：GUI 当 String，SPEC §6.1.4 是 `Recommendation` 对象 → GUI 改对象。
  - **D1/D2 `preset`/`mode` 值 `"default"`**：实为 2026-06-11 改名前的**陈旧 fixture**（daemon live 经 `format!("{:?}", Preset::Standard)` 实发 `"standard"`），仅校正 fixture。两仓 fixture 字节对齐 + 跨仓断言翻转为解码成功。
- **接线 detection 审计**（此前出/入站/决策结果零 audit 写入，`sieve audit query` 查不到核心流量）：`gated_request_decision` 写 `DecisionMade`（所有 gui_popup 决策 + no-client-policy auto-block/warn）、出站脱敏（Anthropic + OpenAI）写 `OutboundRedacted`（fire-and-forget，不阻塞热路径，PRD §9 性能预算；raw_json=None 不持久化 secret）。`dogfood_e2e.rs` Phase D 正向断言审计可经 SQLite + `sieve audit query` 查到。
- **接线入站 Critical 拦截审计**（行为变更/审计完整性，真机 dogfood 抓出，2026-06-18）：上一条接线了出站脱敏与用户决策，但**纯 fail-closed 拦截 block 路径仍零 audit**——拦截入站高危工具调用（如 `eth_signTransaction`→IN-CR-05-EVM）注入 `sieve_blocked` 时，`audit_events` 表无任何 inbound 记录。修复：全部 10 个入站 block 路径（SSE + JSON × Anthropic + OpenAI × 自动 block + GUI deny）补写 `InboundBlocked` 审计事件，含 `rule_id`/`severity`（真实严重度）/`path_label`/`caller` 元数据（fire-and-forget，不阻塞热路径；整体序列化天然零 secret，PRD §5.6.1 / §9 #13）。`dogfood_e2e.rs` Phase B（anthropic_json + openai_json）+ Phase D 断言入站 block 落 `direction=inbound` + `disposition=blocked` 审计行，覆盖 ADR-025 content-type 矩阵两条 JSON 路径（避免重蹈「只挂一条路径」P0 漏洞）。
- **修 `sieve audit query --severity` 过滤永远返回空**（预存 bug，真机 dogfood 抓出，2026-06-18）：审计 `severity` 列由 daemon 各路径统一写**小写**（`format!("{:?}", sev).to_lowercase()`），但 `commands/audit.rs::run_query` 用首字母大写字面量（`"Critical"`）作 SQL 参数 → `WHERE severity = 'Critical'` 对小写列**永不命中**，`--severity <任何级别>` 一律空。单测 `query_severity_filter` 用大写 fixture + 大写查询自洽通过，掩盖了生产失效（fixture 与生产数据大小写不一致）。修复：match 改小写 + `WHERE LOWER(severity) = ?`（大小写不敏感，兼容历史数据）；单测 fixture 还原真实小写 + 加大写输入 cross-case 断言；`dogfood_e2e.rs` Phase D 加 `sieve audit query --severity critical` 端到端断言覆盖 `run_query` 真实路径。
- **修 `sieve audit` / `sieve decisions` CLI 嵌套 runtime panic**：`#[tokio::main]` 内又 `block_on` → "Cannot start a runtime from within a runtime"(exit 134)，两命令完全不可用。`run()` 改 async 委托 `run_async`、由 `main` 直接 `.await`。
- 详见 tasks/lessons.md 2026-06-18（含「Explore agent 把陈旧 fixture 误当 live 漂移」元教训）。

### Fixed — sieve-updater ZSTD 解压魔数字节序反置，规则包永不解压（2026-06-18）

- **修复 `sieve-updater::install.rs::ZSTD_MAGIC` 字节序写反的 bug**：常量为 `[0xFD,0x2F,0xB5,0x28]`，但 zstd 帧 magic `0xFD2FB528` 在磁盘上小端存储为 `28 B5 2F FD`（RFC 8878 §3.1.1）。魔数检查对**任何真实 zstd 流永远不匹配** → `decompress_zstd` 永远走「当原始字节」fallback → 下载的规则包被**原样（压缩态）写盘**，sieve-rules 加载必失败。**整条规则热更新通道在生产中是坏的**，因现有 zstd 单测全部假阳性（用明文走 fallback / 用同一错误常量当输入 / 只断言文件存在）而隐藏。
- 修复：常量改为 `[0x28,0xB5,0x2F,0xFD]`；`happy_path` 单测补「安装内容 = 解压后 JSON」回环断言；新增 `tests/updater_e2e.rs` 闭环 e2e 断言端到端内容。详见 tasks/lessons.md 2026-06-18。

### Added — dogfood 自动化基建（2026-06-18）

- **`crates/sieve-testing`**（新 dev crate，`publish=false`）：共享 e2e harness——`DaemonGuard`/`spawn_daemon`（SIEVE_HOME tempdir 隔离 + 端口/IPC 就绪轮询）、`spawn_mock_upstream`（Anthropic+OpenAI，SSE/JSON/tool_use 响应构造）、`http_post`（内置瞬时连接错误重试，消除并发 flake）、`run_sieve_cli`（驱动真实子命令）。
- **`sieve-updater` 加 `SIEVE_CACHE_DIR` env 覆盖**（`cache_dir.rs`）：缓存路径可隔离到临时目录，hermetic 测试不污染真实用户缓存（install-id / 规则 staging 经 `cache_dir()` 自动覆盖）。
- **`sieve-updater` 加 `SIEVE_UPDATE_ALLOW_HTTP` 测试接缝**（`tls.rs`，`#[cfg(debug_assertions)]`）：debug 构建允许明文 HTTP 出站以指向 localhost mock；**release/GA 构建编译期消除该分支，恒 `https_only()`**（PRD §9 #2 不破）。
- **`scripts/smoke_test.py --mock-only`**：本地 mock Anthropic 上游（`tls_verify_upstream=false`），无需真 API key/网络即跑全套透传/SSE/tool_use/脱敏断言；修出 OUT-01 426→auto_redact 过时断言。
- **`scripts/dogfood.sh`**：一键 hermetic dogfood 入口（构建 + cargo e2e + smoke + updater 闭环）。
- **`crates/sieve-updater/tests/updater_e2e.rs`**：§14 闭环自动化（install-id 首启/幂等/删后重生、fetch→download→sha256→zstd 解压→原子落盘、失败模式、公钥 None skip、遥测 uid 开关）。

### Security — GA 编译期密钥 gate（ADR-034，2026-06-11）

- **新增 `ga_keys` cargo feature 作为 GA release build 的编译期密钥 gate**：启用时，若规则签名公钥（`sieve-updater::TRUSTED_PUBKEY = None`）或 X-Sieve-Origin 公钥（`sieve-ipc::SIEVE_ORIGIN_PUBLIC_KEY` 全零）仍为占位，则 **编译失败（E0080）**，阻止 fail-open 验签进入 GA 二进制，机器强制兑现 `SECURITY.md` 验签承诺（修复 2026-06-07 审查 §5 标记的 GA 硬阻塞）。
- **alpha build（默认无 `ga_keys`）行为完全不变**：规则验签仍 skip+warn、origin 公钥仍占位，逐字节一致、零运行时开销、零新依赖。
- 实现：`sieve-cli` 的 `ga_keys` 传递给 `sieve-updater` + `sieve-ipc`；各 crate 用 `#[cfg(feature = "ga_keys")]` const assert（Rust const panic）在编译期判定占位。GA release pipeline 须 `cargo build --release --features ga_keys`，公钥未就位即无法出包。
- 详见 [ADR-034](../design/ADR-034-ga-key-gate.md)；关联 ADR-006 签名基建（GCP KMS，TODO-14）。

### Fixed — preset mode 跨仓契约漂移：daemon 仍发 v1 旧值 `default`（2026-06-11）

- **修复 daemon health / set_preset 发送 v1 旧值 `"default"`、与 SPEC-005 §5.6 + GUI `Preset` enum（`"standard"`）漂移的 bug**。SPEC-005 §5.6 规定 v1 preset mode `"default"` 在 v2 重命名为 `"standard"`、daemon 与各端必须同步替换，但 daemon 侧（`config::Preset::Default` variant + `daemon_control_plane` String 字面量 + setup 模板）从未替换——daemon 推送 `"default"` 时 GUI 解码 `Preset` enum 失败 → disconnected（直接影响真机 dogfood 连通）。
- 修复：`config::Preset` enum `Default` → `Standard`（`#[serde(alias = "default")]` 兼容旧 sieve.toml）；`daemon_control_plane::default_mode` + `handle_set_preset` 校验统一 `"standard"`（兼容旧 client 发来的 `"default"` → normalize）；`sieve setup` 模板 `preset = "standard"`。
- **由本次新落地的 GUI 端 `IPCSchemaV2FixtureTests` 首次消费 daemon 权威 fixture 时发现**——印证 fixture 防漂移机制价值。文档同步：data-model.md / api-reference.md。

### Tested — SPEC §14 fixture 防漂移机制落地（2026-06-11）

- **daemon 权威 fixture `sieve.health/response.full.json` 补 `listeners[]`（ADR-026 multi-listener），`schema_v2_fixtures.rs` 新增 health full 全 result 双向稳定校验**（`to_value(HealthResult) == fixture result`），落实 SPEC §14.1（此前全文件仅单向反序列化、fixture 缺 listeners[] 漂移无人发现）。
- **GUI 仓新增 `Tests/SieveGUITests/Fixtures/v2/` daemon fixture 副本 + `IPCSchemaV2FixtureTests.swift`**（SPEC §14.2），GUI 解码消费 daemon 权威 fixture 而非内联 JSON，杜绝跨仓 schema 漂移。立即发现上条 preset 漂移 bug。

### Added — 上游转发代理支持（ADR-033 / SPEC-007，2026-06-07）

- **daemon 转发上游可经配置的 HTTP CONNECT / SOCKS5 代理出网**，解决受限网络（Shadowrocket / Clash 等规则代理 + 分流、非全局 TUN）下 sieve 上游硬直连不可用的产品缺口。
- config 新增：顶层 `proxy`（全局兜底）+ 每 `[[upstream]]` 的 `proxy` / `no_proxy` 字段。优先级链（高 → 低）：`upstream.no_proxy`(直连) > `upstream.proxy` > 全局 `proxy` > env(`HTTPS_PROXY` 优先于 `ALL_PROXY`) > 直连。
- proxy URL 格式：`http://` / `socks5://` / `socks5h://`，可带 `user:pass@` 认证；解析失败启动期 fail-fast；代理连接失败**明确报错、绝不静默回退直连**。
- 实现：sieve-core 新增 `ProxyConnector`（tower Service）替换 `Forwarder` 底层 connector；**TLS 仍由 hyper-rustls 在隧道之上做——端到端到上游，代理只见密文、不 MITM**（不解密、不装 CA，符合 PRD §9 #12）。
- **updater 复用同机制**（`updates` / `cdn.sieveai.dev`），daemon 用全局代理注入；受限网络下规则更新 / 装机遥测一并可用。
- 详见 [ADR-033](../design/ADR-033-upstream-proxy.md) / [SPEC-007](../specs/SPEC-007-upstream-proxy.md)；文档同步：api-reference §3.3.2 + §4.2、deployment §6b。

### Fixed — legacy/单-upstream OpenAI 路径被协议错位检查误判 400 的回归（2026-06-07）

- **修复 legacy / 单-upstream（未显式声明 `protocol`）配置下 OpenAI `/v1/chat/completions` 请求被 [ADR-026](../design/ADR-026-port-based-listener-routing.md) §决策 4 协议错位检查误判返回 400 的回归**。根因：`config.rs::resolved_upstreams()` 把 legacy `upstream_url` 与省略 `protocol` 字段的 `[[upstream]]` 硬编码映射成 `Protocol::Anthropic`，导致这些 listener 在收到 OpenAI 请求时被 fail-closed 400，违反 ADR-026 §决策 1 向后兼容承诺与 PRD §9 #16 / #9 双协议硬约束。
- **新增 `Protocol::Auto`（默认态）**：legacy `upstream_url` 与省略 `protocol` 字段的 `[[upstream]]` 均映射为 `Auto`；`Auto` listener 按请求 path 自适应路由（`/v1/messages` → Anthropic，`/v1/chat/completions` → OpenAI），**不做协议错位拒绝**，恢复 v1.x 单 upstream 双协议能力。
- **仅当 listener 显式声明 `protocol = "anthropic"` 或 `"openai"` 时，才强制 ADR-026 §决策 4 的 fail-closed 错位 400**（对显式声明者该行为完全保留）。
- 涉及代码：`crates/sieve-cli/src/config.rs`（`Protocol` 加 `Auto` + `resolved_upstreams` legacy 映射改 `Auto`）、`crates/sieve-cli/src/daemon.rs`（错位检查补 `Auto` 分支）、`crates/sieve-ipc/src/protocol/health.rs`（`ListenerSnapshot.protocol` 文档串补 `"auto"`）。

> **2026-05-06~07 变更（纯文档 + 配置层）**：
> 1. SIEVE_HOME 透传 bug fix + 5 个集成测试隔离（commit 2e38e44，2026-05-06）
> 2. tasks/_archive 清理 + landing-page 占位移除（commit 7cd60e7，2026-05-07）
> 3. README / LICENSE 重构（commit b299463，2026-05-07）
> 4. SPEC-005 listeners[] 数组扩展（commit 7108a45，2026-05-07）

### Fixed — SIEVE_HOME 透传 + 测试隔离（commit 2e38e44）

- **`config.rs::sieve_home()` 现优先读 `SIEVE_HOME` env var**（与 `sieve_ipc::paths::sieve_home` 对齐），`audit_db_path` / `sieveignore` 路径全部改走该函数；修复 daemon 设了 `SIEVE_HOME` 仍把 audit DB 写到真实 `~/.sieve/audit.db` 的污染 bug。
- **5 个集成测试 spawn helper 注入测试隔离**：`outbound_block` / `inbound_block` / `multi_agent_routing` / `content_type_matrix` / `sequence_window_e2e` 强制 `SIEVE_HOME` 落到 tempdir + 注入 `SIEVE_NO_UPDATE=1` + `SIEVE_NO_TELEMETRY=1`，杜绝真实 `~/.sieve/` 写入与遥测污染。
- **`r11_anthropic_out0{6,7}_no_ipc` 测试修复**：改用预占 `ipc.sock` 路径为目录触发 EISDIR，代替「不存在路径」旧手段（被 `AuditStore::init` 自动创父目录绕过）。
- 验证：workspace 747 passed / 13 failed / 7 ignored；真实 `~/.sieve/audit.db` mtime 测试前后不变。
- 2026-06-07 修复：上述 13 个失败（OpenAI 协议错位 9 + GUI popup 测试 mock 4）已全部修复，现 760 passed / 0 failed / 7 ignored，fmt + clippy 全绿。

### Changed — README / LICENSE 重构（commit b299463）

- README 重构：加架构图 + 简化项目状态段 + 隐私声明独立成段；支持 client（Claude Code / Codex / Cursor）+ 上游（Anthropic / OpenAI 兼容端点）范围精确化。
- LICENSE：简化授权表述。

### Changed — SPEC-005 listeners[] 数组扩展（commit 7108a45）

- **SPEC-005 §9.5 health 响应新增 `listeners[]`**（向后兼容 v2 内扩展）：每项含 `provider_id` / `protocol`；旧 `listen` 字段保留为 `listeners[0].addr` 别名并标 deprecated。
- `manual-integration-test.md` 勾选 fmt clean / clippy 0 issues 两项已完成。

---

## [Unreleased] — 2026-05-05

> **本日完成 unix-style 改造 v2.x 全部 5 项 TODO**（13 commits）：
> ADR-026 multi-listener + ADR-028 IPC 中性化 + 2 个新 CLI 子命令。
> TODO-6 Network jail（ADR-027）推后到 v3.x post-GA opt-in。
> 验证：workspace 725 passed / clippy 0 / fmt clean。
>
> **本日另立更新通道遥测决策**（无代码变更,纯文档）：更新通道在拉取规则的同请求中附带匿名 install-id。
> 该决策部分修订 [ADR-003](../design/ADR-003-local-only-no-cloud-verifier.md) 的 telemetry 边界条款。

### Changed — 许可表述（纯文档）

- 原 PRD §7 的历史商务模型表述不再适用。
- README.md 项目状态段更新（旧 PRD §7 折叠为历史段）+ 核心叙事 / 关键差异化精确化。

### Changed — 网络边界（ADR-003 amended）

- **ADR-003 §决策段 admonition 修订**：
  - telemetry 边界改为：复用更新通道附带匿名 install-id，不开辟独立心跳通道。
  - `[update].telemetry` 默认开启,可关闭。
  - manifest URL 允许 `?v=&os=&arch=&uid=&ch=` 5 字段 + UA `sieve-updater/<v>`。
  - **保留不变**：「token verifier 不联网」核心决策永久性 + 不上传 prompt / response / API key / 使用记录。
- **唯一允许的出站请求**新增 2 host：`updates.sieveai.dev`（manifest,**不挂 CDN**）+ `cdn.sieveai.dev`（规则正文 zst,带 sha256 + ed25519）。原 `releases.sieve.dev` 占位符废弃。
- 频率：每周 1 次 → **每天 4 次（每 6h 一次）**。
- ADR-INDEX：ADR-003 状态加注 "(amended 2026-05-05)"。

### Added — 遥测协议（§设计冻结,代码待落地）

- **Manifest 协议 v0.1**：
  - 客户端 `GET https://updates.sieveai.dev/v1/manifest?v=<v>&os=<os>&arch=<arch>&uid=<UUIDv4>&ch=<stable|beta>`（仅 TLS 1.2+,无 cookie / Auth）
  - 服务端响应 `{schema, rules: {version, url, sha256, size, signature}, client: {latest, min_supported, deprecation_notice}, next_check_after_seconds}`
  - 服务端日志只存 `ts | uid | v | os | arch | ch | country(geoip)`,**丢原始 IP**;按日去重统计活跃安装数,原始 IP 不落盘。
- **Install UUID**：UUIDv4 纯随机,首次启动生成,`~/Library/Caches/sieve/install-id`（macOS first; Linux / Windows 路径在 Phase 2 跨平台时落地）。文件权限 0600,用户主动删除 = 新装机（接受统计噪声）。
- **三个环境变量开关**（unix-style,任何非空值 = 启用,优先级高于配置文件）：
  - `SIEVE_NO_UPDATE`：跳过更新检查（不发请求,规则冻结,无遥测）—— 启动 banner 必须打印 `update check disabled by SIEVE_NO_UPDATE`
  - `SIEVE_NO_TELEMETRY`：仍发更新请求但省略 `uid` 字段
  - `SIEVE_UPDATE_URL`：覆盖默认更新源 URL（企业自托管镜像）
- **隐私声明文案**（首次启动 onboarding + README + 隐私政策页统一文案）。

### Added — sieve-updater 规则下载 + 原子替换闭环（2026-05-05 收尾）

- **`sieve-updater` 规则文件完整下载 + 原子替换**（SPEC-006 §3.3）：manifest → 版本比对 → download_rules → install_rules（sha256 + ed25519 + zstd 解压 + .tmp + rename + current.json symlink + latest_version.json 原子写）；安装失败 log error 不退出主循环；热加载留 TODO 由 sieve-rules 接通。
- **`download.rs`**：`download_rules(url, max_size)` via hyper-rustls（TLS 1.2+，https_only），50 MiB 上限，指数退避由调用方（runner）负责。
- **`install.rs`**：`install_rules(payload, sha256, sig, version, dest_dir)` 七步原子写入；`read_installed_version(dest_dir)` 读取已安装版本。zstd magic 检测 + fallback 直接当解压结果（测试友好）。Windows 平台 symlink 失败退化为 copy。
- **runner.rs 接通**：`process_manifest` 接通完整下载 + 安装路径；新增 `retry_with_backoff` 通用指数退避 helper；新增常量 `DEFAULT_RULES_DIR = "rules"` / `MAX_RULES_SIZE = 50 MiB`。

### Changed — sieve-updater::error 新增两个 enum 项

- `UpdaterError::DecompressFailed(String)` — zstd 解压失败
- `UpdaterError::ResponseTooLarge { size, max }` — 响应体超出最大大小限制

### Added — sieve-updater crate + manifest 协议（客户端落地）

- **新增 `sieve-updater` crate**：manifest 协议客户端 + install-id 生成 + 6h 定时器 + ed25519 / sha256 双重校验 + 失败重试指数退避（1s / 4s / 16s × 3 次）；独立 crate 职责清晰，GUI 仓后续可复用同一份 manifest schema 与签名校验，避免协议漂移
- **新增三个环境变量**（unix-style，任何非空值视为启用）：
  - `SIEVE_NO_UPDATE`：完全跳过更新检查（不发请求，规则冻结，无遥测）；启动时强制打印 banner `update check disabled by SIEVE_NO_UPDATE`
  - `SIEVE_NO_TELEMETRY`：仍发更新请求但省略 `uid` 字段（仍能拿到规则更新，不参与装机统计）
  - `SIEVE_UPDATE_URL`：覆盖默认更新源 URL（企业自托管镜像 / 私有内网 / 本地 mock 测试）
- **`sieve.toml` 新增 `[update]` 段**：`enabled` / `telemetry` / `url` / `check_interval_hours` / `channel`；env var 优先级始终高于 toml
- **新增 SPEC-006**（manifest 协议规格 v0.1）：wire format + install-id + 三个 env var + 签名校验 + 失败处理 + 测试矩阵（14 项），详见 [docs/specs/SPEC-006-update-and-telemetry.md](../specs/SPEC-006-update-and-telemetry.md)

### Changed — CLAUDE.md 七个 Crate 表（新增 sieve-updater）

- CLAUDE.md「六个 Crate」→「七个 Crate」，新增 `sieve-updater` 行（职责 + 禁做）
- `.cursorrules §3.3` 同步更新七个 crate 边界表
- `docs/design/architecture.md §2.1` 新增 sieve-updater 模块行 + updater task 不在 hot path 说明

### Follow-up（运维侧，GA 前必须落地）

- **运维侧**：域名 `updates.sieveai.dev` / `cdn.sieveai.dev` 注册（GA 前确定）/ ed25519 签名密钥管理（HSM / 单独 build 机 / GCP KMS 之一,写入 ADR-006 follow-up）/ 服务端实现（**倾向 Cloudflare Workers + KV / D1**,manifest 接口天然反 DDoS）/ ch 通道策略（首发 stable 单通道,Phase 2 加 beta）。
- **代码侧**：TRUSTED_PUBKEY TODO-14 GCP KMS 落地后填入真实公钥（当前 None 占位，WARN + 跳过 ed25519 校验）

### unix-style 改造立项（commit cf129a2）

- 新增 [ADR-026](../design/ADR-026-port-based-listener-routing.md) Port-based listener routing
- 新增 [ADR-027](../design/ADR-027-network-jail-enforcement.md) Network jail enforcement（v3.x post-GA opt-in）
- 新增 [ADR-028](../design/ADR-028-ipc-protocol-neutralization.md) IPC 协议中性化 + sieve-ipc 内部模块化
- `tasks/PROGRESS.md` 加「unix-style 改造」段，6 个 TODO 按 P0/P1/P2 排进 v2.x 与 v3.x

### Added — Multi-listener（ADR-026 全部落地）

- **`[[upstream]]` 配置数组**（commit bdcb8de，Stage A）：每项含 `port` / `url` /
  `provider_id` / `protocol`。新增 `Protocol` enum（`anthropic` | `openai`）+
  `UpstreamListener` 配置 struct + `Config::resolved_upstreams()` 兼容方法 +
  `check_safety_invariants()` 可单测函数（端口冲突 + 非 loopback bind 检测）。
- **BREAKING**：`Config.upstream_url` + `port` 标为 deprecated。旧字段保留向后
  兼容（自动映射成单元素 vec），现有 `sieve.toml` 无需改动即可继续工作；
  `upstreams` 非空时旧字段被忽略并 WARN。
- **Multi-listener accept loop + 协议错位 fail-closed**（commit 042c4c9，Stage B+C+D）：
  daemon 重构遍历 `cfg.resolved_upstreams()` 各自 bind + spawn 独立 `accept_loop`，
  任一 bind 失败 fail-fast。哑 client（Claude Code / Codex CLI 等只认 single base_url
  的 agent）通过指向不同 port 切换上游，无须注入路由 header。Anthropic listener 收到
  `/v1/chat/completions` → 400 + `sieve_blocked`；Openai listener 收到 `/v1/messages` → 400。
  其他 path 透传不强制；X-Sieve-Provider header routing 不能 override listener 协议
  （fail-closed 一致性，PRD §9 #3）。`ListenerSpec` struct + `RequestCtx` 加
  `listener_protocol` / `listener_provider_id` 字段。
- **审计 provider_id 透传**（commit b6e716d，Stage E）：`AuditStore::append` 签名
  升级 `(event, provider_id)`；SQLite schema **v2 → v3** migration（`ALTER TABLE
  ADD COLUMN provider_id TEXT NOT NULL DEFAULT 'unknown'`）；新增
  `audit::SYSTEM_PROVIDER_ID = "_system"` / `UNKNOWN_PROVIDER_ID = "unknown"`
  常量；13 处 audit.append 调用点全部同步；daemon 系统级事件（control plane /
  oversize / UserRulesReloaded）用 `_system`。
- **IPC HealthResult.listeners 数组**（commit d90c51b，Stage F）：新增
  `sieve-ipc::ListenerSnapshot`（含 `provider_id` / `protocol`）；
  `HealthResult.listeners: Vec<ListenerSnapshot>` 向后兼容 `listen: ListenSnapshot`
  单字段保留为 `listeners[0]` 别名。GUI 客户端（sieve-gui-macos）需 follow-up
  同步读取新字段。
- **doctor multi-listener 体检**（commit b6e716d）：新增 `check_all_listeners_from_config`
  helper，读 `~/.sieve/sieve.toml` 解析 upstreams 逐 port TCP 探测；仅
  `[[upstream]] > 1` 时打印（避免单 listener 配置冗余）。
- **Fix Forwarder path prefix**（commit 28fbb30，TODO-1）：修复 v1.x bug——
  `upstream_url` 中的 path 前缀被丢弃，导致 DeepSeek 等 Anthropic 兼容入口
  （`https://api.deepseek.com/anthropic`）转发后变成 `/v1/messages` 而非
  `/anthropic/v1/messages`（404）。新增 `Forwarder.upstream_path_prefix` 字段，
  `Host` header 行为不变（仍是纯 authority），5 个调用点零改动。

### Added — IPC 中性化（ADR-028 全部落地）

- **SPEC-005 协议术语中性化**（commit 69664c3，TODO-3a）：清洗 ~371 处 GUI-中心化
  术语：「GUI 端」→「client 端」/「daemon → GUI」→「daemon → client」/「popup」→
  「decision request/event」。`gui_popup` wire 字段值**保持不变**（向后兼容硬要求）+
  加 ADR-028 标注语义中性化。`ui_phase` / §3.4 UI 文案 / §6.1.4 recommendation
  加 admonition 标注 GUI 实现细节。§9 标题「GUI 控制面方法」→「控制面方法」。
  daemon IPC 协议**不感知 client 形态**（GUI / CLI / TUI / webhook 都是平等 client）。
- **sieve-ipc crate 内部模块化**（commit 0ba0350，TODO-3b）：拆分
  `crates/sieve-ipc/src/protocol.rs` 为 `protocol/` 子目录（envelope / decision /
  handshake / rules / audit / health / notify 7 子文件）；`socket_server.rs` →
  `server/socket_server.rs`；`socket_client.rs` → `client/connection.rs`。新增
  `protocol/README.md`：SPEC-005 权威源声明 + 零 IO 依赖硬约束（仅 import serde /
  chrono / uuid / std；禁止 tokio / hyper / fd-lock / 任何 IO crate）。lib.rs
  re-export 100% 兼容 + 向后兼容别名（`socket_client` / `socket_server` 路径仍可用）。

### Added — 新 CLI 子命令（ADR-028 TODO-4 + TODO-5）

- **`sieve decisions` headless CLI**（commit 8717442，TODO-4）：让 daemon 在
  GUI 不在线时仍可用，CLI 接管决策（远程 SSH / GUI crash / tmux 工作流不再卡死）。
  - `sieve decisions watch [--format jsonl] [--severity SEV]`：流式订阅 pending
    decision events
  - `sieve decisions show <id>`：查询单个 pending 上下文
  - `sieve decisions resolve <id> --approve|--block|--warn [--reason "..."]`
  - 新增 `sieve start --no-client-policy={auto-block|auto-warn|hold-and-fail-closed}`
    flag（默认 auto-block）：daemon 在无 client 接 IPC 时的兜底策略。
  - 实现：raw JSON-RPC over `UnixStream`，连 `~/.sieve/ipc.sock`；CLI 跟 GUI
    共用同一组 IPC method，不引入特权 endpoint。`gated_request_decision` 加
    `no_client_policy` 参数，`connected_clients == 0` 且非 Critical 时按策略快速返回。
  - +5 单元测试。
- **`sieve audit` unix-pipeable CLI**（commit 7a1415d，TODO-5）：审计日志
  unix-pipeable 输出，方便接 jq / fluentd / vector。
  - `sieve audit tail [-f] [--format jsonl] [--limit N]`：最后 N 条 + `--follow`
    流式跟踪
  - `sieve audit query [--since DUR] [--severity SEV] [--rule-id RULE] [--provider-id PROVIDER]`
  - `sieve audit show <id>`
  - 实现：直接读 `~/.sieve/audit.db` SQLite（不通过 IPC）；jsonl 每行一个 JSON
    object；`parse_duration` 解析 `1h/30m/7d`；`tail --follow` 用
    `LIMIT + ORDER BY id DESC` 轮询（500ms 间隔）。
  - +7 单元测试。

### Changed — 文档同步

- **CHANGELOG / api-reference §3.3.1 / architecture §1.1**（commit d90c51b，Stage G 核心）：
  multi-listener 配置 schema + 部署拓扑说明。
- **data-model §5.1a + §6.2 events 表 v3 + §6.2b migration**（commit b6e716d）：
  `[[upstream]]` 数组字段表 + provider_id 列说明 + v2→v3 migration SQL +
  特殊值 `_system` / `unknown`。
- **development.md §3.4a**（commit b6e716d）：multi-listener dev 模式实战
  配置 + 协议错位测试示例。
- **SPEC-003 §4.2b doctor multi-listener 体检 + SPEC-004 §4.2.6 header vs port
  routing 分工 + deployment.md §6a Multi-listener 部署**（commit 16bc0e7）：
  3 份文档同步 +135 行；[redacted]（ADR-027）前向引用。

### Fixed

- **fmt baseline 清理**（commit 39e82a1）：4 个 crate 中 8 处 pre-existing
  fmt 偏差（daemon_control_plane / sieve-ipc / sieve-policy）。无功能变更。
- **audit v3 schema 测试断言**（commit bb5f6a1）：修补 Stage E 漏改的 3 个
  pre-existing 测试：`update_trigger_blocks`（INSERT_SQL 11 参数）/
  `migration_from_v1_preserves_data`（user_version 终为 3）/
  `fresh_database_starts_at_v2`（全新 DB 应是 v3）。

### Follow-up（不阻塞）

- TODO-6 Network jail enforcement（ADR-027，v3.x post-GA opt-in 高级特性）
- GUI 仓 sieve-gui-macos：Swift 代码读 `health.listeners` 数组（向后兼容期内
  `listen` 单字段仍发，daemon 可独立 ship 不阻塞）

---

## [v2.0+ 兼容扩展] — 2026-05-03

### 协议（SPEC-005 v2）
- BREAKING: protocol_version "v1" → "v2"，不向下兼容
- 新增 sieve.hello 握手通知（7 字段）+ sieve.heartbeat 25s 心跳
- request_decision 拆 wire DTO（内部 vs wire 分离），单 issue 平铺 / 多 issue merged + issues[]
- decision_response.result 补 required 字段：request_id / decided_at / by_user / ui_phase_when_clicked
- 错误码段位重划：daemon→GUI -32000~99 / GUI→daemon -32100~99
- 所有 method 名带 sieve. 前缀
- 字段重命名 created_at → received_at_daemon
- 字段名修正：until → paused_until / event_notify → notify_status_bar
- HealthResult.paused 拆 bool + paused_until
- NotifyKind 6 枚举值
- 全部枚举 snake_case lowercase

### 协议（SPEC-005 v2.0+ 兼容扩展，不 bump version）
- 新增 sieve.list_rules method（GUI 规则总览 Table 数据源）
- 新增 sieve.purge_history method（GUI 清空历史，需 GUI Touch ID 二次确认）
- 错误码 -32006 rules_loading / -32007 purge_in_progress

### 实现（daemon）
- 帧读取改用 FrameReader + memchr（移除无界 BufReader::lines），单帧/remainder 1 MiB 限制 + audit 写 oversize 事件
- socket 文件 0600 / 父目录 0700
- fan-out 串行化（先 broadcast 后 result）+ origin_request_id 真实透传
- fan-out write 加 2s bounded timeout（EPIPE/ECONNRESET/EBADF 视为失联）
- JSON 解析失败返回 -32700 不关连接
- recommendation 字段 daemon 业务层真实注入

### 测试
- e2e 集成测试 harness（spawn 真实 daemon + Rust mock GUI client，6 场景）
- v2 fixtures 81 条（17+2 method × 3 档）
- canary_token_hits_out01 路径硬编码 flake 修复

### 善后
- ipc audit oversize callback 注入（之前 callback 未接 SQLite）
- pending decision 收到 GUI 错误响应按段位清理（防泄漏）

---

## [v2.1-gui-control-plane-spec] - 2026-05-02

### 背景

sieve-gui-macos PRD v1.0 起草后，需要 GUI 通过 IPC 操控 daemon 的运行时状态（暂停 / preset 切换 / 灰名单管理 / health 查询 / 沙箱评估）。本次仅扩展协议规格，工程落地随 Week 5 GUI HIPS 主流程一起 ship。

### Added — 文档

- 独立仓库 `sieve-gui-macos/docs/prd/sieve-gui-macos-prd-v1.0.md`：GUI 仓库的根 PRD（菜单栏 / HIPS 弹窗 / 设置 / 历史 / 调试 / Onboarding / IPC 契约 / Critical 锁防线三的 GUI 端约束）
- `docs/design/ADR-013-ipc-protocol.md` Supplement 2026-05-02：通道 A 新增 11 个方法 / 通知（`set_paused` / `set_preset` / `set_preset_overrides` / `reload_config` / `health` / `evaluate` / `list_graylist` / `remove_graylist` / `preset_changed` / `paused_changed` / `request_decision_canceled`）；含完整 schema、错误码、Critical 锁在 `set_preset_overrides` 路径的强校验
- `docs/specs/SPEC-002-hips-popup-behavior.md` §9 新增：暂停状态下的弹窗触发矩阵（Critical 锁仍弹）/ preset 切换对 hold 中弹窗的影响（不动正在显示的）/ `request_decision_canceled` 的 GUI UI 行为（自动关弹 + 历史标记）/ 多 GUI 实例的 broadcast 与 `resolved_by_peer` 取消原因

### Notes

- 协议版本号保持 `v1`（仅扩展方法，未引入 breaking change）
- SPEC-001 不需要更新——新方法全部走 socket 通道 A，不沾文件锁通道 B 边界
- 落地任务清单见 ADR-013 §S.7（Week 5 起跟 GUI HIPS 主流程同步实现）

---

## [v2.1-alpha-engineering] - 2026-05-01

### 背景

把 v2.1 推迟清单里所有纯工程项一次性清空：LayeredEngine zero-downtime hot swap / try_write_graylist 失败路径 audit / 多 GUI broadcast 支持。剩 2 项需 dogfood 数据触发（行为序列升级 Block 的 ADR 评审 + ML 分类器训练），不属代码工作。

### Added

**LayeredEngine zero-downtime hot swap（v2.1-1）**：
- `crates/sieve-rules/src/engine/mod.rs`：`LayeredEngine.user` 字段从 `Option<U>` 改为 `ArcSwap<Option<Arc<U>>>`（lock-free read，atomic pointer swap）
- `LayeredEngine::swap_user(new_user)`：daemon reload listener 调用此方法替换用户引擎，零阻塞所有并发 reader
- `LayeredEngine::new(system, user)` 签名保留向后兼容，内部包装 ArcSwap
- 新增 `arc-swap = "1"` workspace 依赖（lock-free 库，hot path 零开销）
- 新增 2 单元测试：swap 真生效（v1 → v2 → None 切换） + 并发 scan + swap 不阻塞
- `daemon::run` 加 `outbound_layered` / `inbound_layered: Arc<LayeredEngine<...>>` 参数
- `reload_user_rules_best_effort` 重写为 `reload_user_engines`（重新读 + lint + compile + 返回新引擎），reload listener 成功时调 `swap_user(...)` 真正 hot swap

**try_write_graylist 失败路径 audit（v2.1-2）**：
- `crates/sieve-cli/src/audit.rs`：新增 `AuditEvent::GraylistAddFailed { rule_id, error, request_id, caller }` 变体；7 个 getter 全补分支
- `crates/sieve-cli/src/daemon.rs::try_write_graylist`：3 个失败分支（Critical 锁拒绝 / SIEVE_HOME 不可用 / add_entry IO 错）全部加 audit append（spawn task fail-soft，不阻塞决策路径）
- 2 新单元测试：GraylistAddFailed 变体 SQLite round-trip + getter 元数据验证

**多 GUI broadcast 支持（v2.1-4）**：
- `crates/sieve-ipc/src/socket_server.rs`：`gui_writer: Arc<Mutex<Option<Sender>>>` 改为 `gui_writers: Arc<Mutex<Vec<Sender>>>`
- `broadcast_status_bar` 改为 fan-out 实现：drain + try_send 全部 sender，dead sender（`TrySendError::Closed`）即时清理
- `broadcast_status_bar` 从 `async fn` 改为 `fn`（持锁不跨 await，配 std::sync::Mutex）；daemon 端 2 处 await 调用相应去掉
- 新增 2 集成测试：3 GUI 同时收到广播 / GUI 断开后 dead writer lazy 清理；现有 7 测试全过
- `request_decision` 仍单 GUI（`gui_writers[0]`，接连最旧），fan-out 仅扩展 broadcast

### Test

- `cargo test --workspace --no-fail-fast`（feature off）：**633 passed / 1 failed（已知 doctor 竞态）/ 5 ignored**（v2.0-deferred-2 时 627 → +6 测试）
- `cargo test --workspace --features sieve-cli/sequence_detection --no-fail-fast`：**643 passed / 1 failed / 5 ignored**
- `cargo fmt --all -- --check`：clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：clean
- `cargo build --workspace --features sieve-cli/sequence_detection`：clean

### Deferred (非代码项)

仅剩 2 项需 dogfood 数据触发，**不属代码工作**：
- 行为序列升级 Block 类的 ADR 评审（PRD §9 #15 升级触发条件：真实用户连续 4 周 ≥ 50 个序列样本 + FP rate < 0.5%）
- 行为序列 ML 分类器训练（v2.1+ 需 dogfood 数据集积累后做）

至此 PRD v2.0/v2.1 所有需要写代码的部分**全部完成**。

---

## [v2.0-alpha-deferred-2] - 2026-05-01

### 背景

把 v2.0 推迟清单里所有 daemon-side 接入工作一次性清空：IPC 协议扩展（StatusBarNotify + ReloadUserRules）+ HoldOutcome 携带 remember/context_hint + daemon 全接入（audit 透传 / peer_addr / 灰名单写入 / IN-SEQ-* 通知 / hot-reload）。

仅保留 zero-downtime LayeredEngine swap 推 v2.1（需改 sieve-rules，本期 best-effort reload + StatusBar 提示"需重启 daemon 完整生效"）。

### Added

**sieve-ipc 协议扩展（W2A-1）**：
- `StatusBarNotify` + `NotifyKind { SequenceHit, OutboundRedacted, UserRulesLoadFailed, UserRulesReloaded, Generic }` 枚举（PRD §5.4.3 / §5.7 单向通知）
- `ReloadUserRules` 通知（sieve rules edit → daemon，PRD §5.5.5 步骤 4）
- `IpcServer::broadcast_status_bar` 广播给所有连接 GUI（无 GUI 时静默丢弃）
- `IpcServer::reload_rx()` 暴露 mpsc receiver（容量 16），daemon spawn 监听 task
- `send_reload_user_rules_oneshot(socket, trigger_id)` 独立函数给 cli 用
- 7 个新测试 + 4 个 request_decision 集成测试全通过（44 passed / 1 ignored）

**sieve-core HoldOutcome 扩展（W2A-2）**：
- `HoldOutcome::Allow { remember, context_hint }` + `RedactAndAllow { remember, context_hint }`（PRD §5.4.2 灰名单字段透传）
- `Deny { reason }` 不变（Deny 路径不写灰名单）
- `hold_and_decide` 把 IPC `DecisionResponse.remember/context_hint` 透传到 HoldOutcome
- 超时路径强制 `remember: false`（无用户主动决策不触发灰名单）
- 顶层文档注释明确 daemon 必须二次校验 critical_lock（crate 边界，sieve-core 不依赖 sieve-rules）

**daemon-side 全接入（W2B）**：
- **修 4 处 HoldOutcome 解构编译错误**（HoldOutcome 改 enum 结构体变体）
- **`peer_addr_to_pid` 替换 stub** → 调 `process_context::lookup_caller_by_socket_addr(local, peer)`，accept loop 拿 `local_addr` + caller info 合并进 `RequestCtx` 透传
- **AuditStore 透传**：main.rs `Arc<AuditStore>` 注入 daemon::run；新建 `RequestCtx { caller, audit }` 合并参数透传到 proxy → proxy_inner → proxy_openai → forward_with_*_inspection → handle_*_json_inbound
- **7 个新 AuditEvent 变体**：`DecisionMade / GraylistAdded / GraylistCriticalRejected / GraylistHit / SequenceHit / UserRulesReloaded / UserRulesLoadFailed`
- **入站 SSE 灰名单 add_entry**（Anthropic + OpenAI 两条）：消费 HoldOutcome.remember/context_hint，二次校验 critical_lock 后调 try_write_graylist
- **OpenAI 出站灰名单 lookup 对称**：与 Anthropic 出站对称的 `check_graylist_hit` 在 HoldForDecision 块前
- **IN-SEQ-* IPC StatusBar 通知 + audit**：`record_into_sequence_and_detect` 新签名加 ipc_server / audit_store / caller 参数；4 个调用点全更新；命中时 broadcast SequenceHit notify + audit append（feature off 时不影响）
- **`sieve rules edit` 后 IPC notify reload**：lint 通过后 `tokio::runtime::Runtime::new().block_on(send_reload_user_rules_oneshot(...))`，socket 不存在静默跳过
- **daemon reload listener**（best-effort，PRD §5.5.5 妥协实现）：spawn task 监听 reload_rx → 重新读 user.toml + lint + UserEngine::compile 验证 → broadcast UserRulesReloaded / UserRulesLoadFailed 通知 + audit；**不做 zero-downtime hot swap**（推 v2.1，需改 sieve-rules LayeredEngine）

### Test

- 默认 feature：`cargo test --workspace --no-fail-fast` → **627 passed / 1 failed（已知 doctor 竞态）/ 5 ignored**（+ 偶发 outbound_block r11 daemon 启动竞态，单跑通过）
- feature on：`cargo test --workspace --features sieve-cli/sequence_detection --no-fail-fast` → **638 passed / 1 failed（同上）/ 5 ignored**
- `cargo fmt --all -- --check`：clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：clean
- `cargo build --workspace --features sieve-cli/sequence_detection`：clean

### Deferred to v2.1（仅剩工程项）

- **zero-downtime LayeredEngine hot swap**：改 sieve-rules `LayeredEngine.user` 为 `Arc<RwLock<Option<UserEngine>>>`，daemon reload listener 做 atomic swap；本期 best-effort 验证 + StatusBar 告知"需重启 daemon"
- **`try_write_graylist` 内部 audit ERROR 写入**：失败路径目前只 tracing::error，audit 写入留 v2.1
- **行为序列升级 Block 类的 ADR 评审**（PRD §9 #15 升级触发条件：真实用户连续 4 周 ≥ 50 个序列样本 + FP rate < 0.5%）
- **多 GUI 客户端支持**（当前 broadcast_status_bar 单 GUI 约束，ADR-013 扩展）
- **行为序列 ML 分类器**（v2.1+ 训练）

---

## [v2.0-alpha-deferred-1] - 2026-05-01

### 背景

把 v2.0 Phase A/B 推迟清单里的「不依赖 daemon 改动的代码项」一次性清空：用户规则 direction 字段 / 行为序列 e2e 测试矩阵 / criterion CI gate / process_context macOS socket 反查实现。daemon 接入推后续 commit。

### Added

- **用户规则 `direction` 字段**（PRD §5.5）：
  - `crates/sieve-policy/src/loader.rs`：`RuleDirection { Outbound, Inbound, Both }` 枚举（默认 Both 兼容旧 user.toml）；`UserRuleEntry.direction` 加 `#[serde(default)]`
  - `crates/sieve-policy/src/engine.rs`：`UserEngine::compile_for_direction(rules, direction)` 按方向过滤
  - `crates/sieve-policy/src/lint.rs`：`InboundAutoRedactForbidden` 触发条件改为 `direction_touches_inbound + auto_redact`
  - `crates/sieve-cli/src/main.rs`：启动时 outbound / inbound 两次加载，分别按 direction 过滤
  - `crates/sieve-cli/src/commands/rules.rs`：模板 + list 输出加 direction 列
  - 4 个新测试：方向分流 / 旧格式默认 Both / inbound+auto_redact 拒绝 / outbound+auto_redact 合法
- **行为序列 e2e 测试矩阵**（PRD §9 #16，feature on 时跑）：
  - `crates/sieve-cli/tests/sequence_window_e2e.rs`（954 行 + 7 测试）
  - 4 类组合 × IN-SEQ-01 全覆盖（Anthropic SSE / JSON + OpenAI SSE / JSON）
  - 额外覆盖 IN-SEQ-02（Anthropic SSE）+ IN-SEQ-03（OpenAI JSON）+ FP 防护（顺序反转不触发）
  - tracing 捕获通过 daemon stdout pipe + bg 线程读 + `SIEVE_LOG=info` filter
- **criterion CI gate**（PRD §6.3.2）：
  - `.github/workflows/bench.yml`（85 行）：main push + PR 触发，macos-14 runner，scan_70_rules baseline 对比
  - `scripts/bench_compare.sh`（88 行）：解析 criterion `estimates.json`，mean 退化 > 10% 失败（criterion 不暴露 P99，--sample-size 10 时 P99 无统计意义；mean 受尾部拉升敏感度足够）
  - 本地 baseline 验证：注入 2× mean 时正确失败 exit 1
- **`process_context::find_pid_by_socket_addr` macOS 实现**（PRD OQ-V20-02）：
  - `crates/sieve-cli/src/process_context.rs`（+420 行）
  - libc FFI：`proc_listpids` + `proc_pidinfo(PROC_PIDLISTFDS)` + `proc_pidfdinfo(PROC_PIDFDSOCKETINFO)` 扫所有进程 FD 匹配 4-tuple
  - 15 处 unsafe 全部带 SAFETY 注释；`socket_fdinfo` 字段偏移用 C sizeof 实测验证（arm64 MacOSX15.4 SDK）
  - 30s LRU peer cache 复用现有模式
  - 3 新测试（含 1 `#[ignore]` 因 SIP 沙箱权限限制）
  - 不引入新 crate，无 `lsof` / `netstat` shell out（OQ-V20-02）

### Changed

- `crates/sieve-cli/Cargo.toml`：`sequence_detection` feature 已上一 commit 加；本期无 deps 变化
- `process_context.rs` `PeerCacheEntry` / `peer_cache` 加 `#[cfg(target_os = "macos")]` gate（修 dead_code lint）

### Test

- `cargo test --workspace --no-fail-fast`（feature off 默认）：**617 passed / 1 failed（已知 doctor 竞态）/ 4 ignored**
- `cargo test --workspace --features sieve-cli/sequence_detection --no-fail-fast`：**628 passed / 1 failed / 4 ignored**（净增 11 个 e2e 测试）
- `cargo fmt + clippy --workspace --all-targets --all-features -- -D warnings`：clean
- `scripts/bench_compare.sh` 本地 注入 2× mean 验证：正确 exit 1

### Deferred to next commit (Wave 2)

下批 daemon-side 接入工作（合并到一个大代理做完，避免 daemon.rs 多代理冲突）：
- `peer_addr_to_pid` daemon 接入：用 `find_pid_by_socket_addr` 替换 stub + 所有 audit_event 写入点透传 caller_pid/caller_exe
- 入站 SSE 灰名单 `add_entry`（`HoldOutcome` 扩展携带 remember + context_hint）
- OpenAI 出站灰名单 `check_graylist_hit` 对称
- IN-SEQ-* IPC StatusBar 通知协议 + daemon 调用接入 + audit `kind: sequence_hit` 写入
- `sieve rules edit` 后 IPC notify daemon hot-reload 用户规则（IPC 协议扩展 + daemon swap UserEngine）

---

## [v2.0-alpha-B-skeleton] - 2026-05-01

### 背景

PRD v2.0 Phase B beta 启动（Week 8 范围）：行为序列窗口骨架 + 3 条 IN-SEQ-* 启发式 + daemon 双路径接入。**默认关闭**（PRD §9 #15），通过 cargo feature `sequence_detection` opt-in 启用。

### Added

- **`sieve-core/sequence` 新模块**（808 行 + 26 测试）：
  - `sequence/mod.rs`（204 行）：`ToolUseRecord` / `ToolUseSequence` / `SequenceConfig`（默认 N=10 / TTL=300s 滑动窗口，PRD §5.7.1）
  - `sequence/feature.rs`（282 行）：从 `tool_name + tool_input` 提取结构化特征（`ToolClass` Shell/FileRead/FileWrite/Network/Other × `PathCategory` SensitiveSecret/Wallet/DotEnv/Code/Tmp/Other + 4 布尔位 network_egress / persistence_mech / cleanup_mech / sensitive_file_hint）；隐私安全：不存原始 input
  - `sequence/detector.rs`（322 行）：3 条启发式 IN-SEQ-* 全 severity=High，**仅 StatusBar 通知**（PRD §9 #15 不引入 Block 路径）：
    - `IN-SEQ-01-RECON-EXFIL`：FileRead+SensitiveSecret 之后 network_egress
    - `IN-SEQ-02-CLEANUP-AFTER-ATTACK`：Shell+network_egress 之后 cleanup_mech
    - `IN-SEQ-03-PERSISTENCE-CHAIN`：3 次 persistence_mech=true 跨**不同 tool_name**
- **`SessionState.sequence_window` + `InboundFilter` 公开方法**：
  - `record_tool_use_into_sequence(tool_name, input, rule_hits)`：feature off 时是 no-op
  - `detect_sequence_hits()`：返回 IN-SEQ-* 命中
  - feature off 时 SessionState 不含 sequence_window 字段，零运行时开销
- **daemon 双路径接入**（PRD §5.7.4 + §9 #16）：
  - 新 helper `record_into_sequence_and_detect`（daemon.rs）：3 处 tool_use 完成路径都调
  - SSE 路径（`forward_with_inbound_inspection`）→ `path_label = "sse"`
  - Anthropic JSON 路径（`handle_anthropic_json_inbound`）→ `path_label = "anthropic-json"`
  - OpenAI JSON 路径（`handle_openai_json_inbound`）→ `path_label = "openai-json"`
  - 命中 IN-SEQ-* 仅 `tracing::info!(target: "sequence_alert", ...)`，**不阻断**（PRD §9 #15）
- **cargo features**：
  - `crates/sieve-core/Cargo.toml`：`sequence_detection = []`（默认关闭）
  - `crates/sieve-cli/Cargo.toml`：`sequence_detection = ["sieve-core/sequence_detection"]`（默认关闭）

### Test

- `cargo test --workspace --no-fail-fast`（feature off）：**610 passed / 1 failed（已知 doctor 竞态，单跑通过）/ 3 ignored**
- `cargo test -p sieve-core --features sequence_detection`：169 passed（含 4 个序列 + InboundFilter 集成测试）
- `cargo test -p sieve-cli --features sequence_detection --no-fail-fast`：207 passed（仍仅 doctor 竞态 1 失败）
- `cargo build --features sieve-cli/sequence_detection`：成功；`cargo clippy --workspace --all-targets --features sieve-cli/sequence_detection -- -D warnings`：clean
- 默认配置 + feature on 配置 fmt + clippy 全 clean

### Deferred to Week 9 / v2.1

- IN-SEQ-* 命中接入 IPC StatusBar 通知 + audit 写入（v2.1）
- e2e 测试矩阵（PRD §9 #16）：mock 攻击序列在 4 类 content-type 组合下都触发 IN-SEQ-*（Week 9 dataset 落地后做）
- 行为序列升级 Block 类的 ADR 评审（v2.1，需真实用户连续 4 周 ≥ 50 个序列样本 + FP rate < 0.5%，PRD §9 #15 升级触发条件）
- ML 分类器训练（v2.1+）

---

## [v2.0-alpha-A-integration] - 2026-05-01

### 背景

PRD v2.0 Phase A 第二批落地：Week 6 接入工作 —— 把 v2.0-alpha-A-skeleton 的骨架接入 daemon 决策路径，加 benchmark + corruption 测试覆盖 PRD §9 #14 / §9 #16 硬约束。

### Added

- **daemon 启动加载用户规则 + LayeredEngine 包装**（`main.rs` + `engine_adapter.rs`）：
  - 出站 + 入站两侧均加载 `~/.sieve/rules/user.toml` → lint → `UserEngine::compile` → `LayeredEngine::new(system, user)`（PRD §6.3 / §5.5.2.1）
  - **fail-safe**（PRD §9 #14）：load 失败 / lint 违规 / compile 错误 → warn 日志 + 用 None 用户引擎，daemon 必须正常启动，系统规则不退化
  - `OutboundAdapter` / `InboundAdapter` 改泛型 `<E: MatchEngine + Send + Sync + 'static>`（默认 `= VectorscanEngine` 兼容旧调用方）
  - 新辅助函数 `load_and_compile_user_engine` + `load_user_engine_fail_safe`
- **daemon 三态决策 allow_remember 计算**（`daemon.rs`）：
  - 6 处 `DecisionRequest` 构造点全部从 hardcoded `false` 改为 `compute_allow_remember(rule_ids)`：任一 rule_id 在 `FAIL_CLOSED_RULES` 中即返 false，否则 true（PRD §5.4.3 + §9 #3）
  - 覆盖：IN-CR-06 OpenClaw skill / 出站 Anthropic / 出站 OpenAI / 入站 Anthropic SSE / 入站 OpenAI SSE / Hook 类 pending 文件
- **daemon 灰名单接入决策路径**（`daemon.rs`）：
  - `check_graylist_hit`：HoldForDecision 前 fingerprint lookup，命中 → 跳过 IPC 弹窗直接放行（出站 Anthropic 路径首发，OpenAI / 入站 SSE 推 v2.1）
  - `try_write_graylist`：收 `DecisionResponse.remember=true && allow_remember=true` 时调 `sieve_policy::graylist::add_entry`（内部二次校验 critical_lock，命中 → `PolicyError::CriticalRuleNotGraylistable`）
  - 灰名单查询失败 → fail-closed（不 fail-open，PRD §9 #14 延伸）
- **daemon caller 进程上下文 stub**（`daemon.rs`）：
  - accept loop 调 `peer_addr_to_pid` + `process_context::lookup_caller`，trace 日志带 caller_pid/caller_exe
  - `peer_addr_to_pid` v2.0 Phase A 返 None stub（PRD OQ-V20-02 决策走系统 API，真实 `proc_listpids` 反查推 v2.1）
  - audit 字段路径打通：A4 已加 schema 字段，本期日志 trace 接入；audit 调用点透传留 v2.1 一行替换
- **集成测试**：
  - `crates/sieve-cli/tests/user_rules_loading.rs`（8 测试）：5 类 corruption 在 daemon 启动路径上的 fail-safe 验证
  - `crates/sieve-cli/tests/graylist_integration.rs`（14 测试）：fingerprint 确定性 / round-trip / Critical 拒绝 / 篡改检测 / allow_remember 计算 / 文件权限
  - `crates/sieve-cli/tests/content_type_matrix.rs`（6 测试）：PRD §9 #16 4 类组合（Anthropic SSE/JSON + OpenAI SSE/JSON）+ audit schema v2 caller 列验证
  - `crates/sieve-policy/tests/corruption.rs`（12 测试）：完整 11 类 lint 违规 + 文件权限/symlink/目录权限/重复 ID/未知字段/超大文件/超量规则
- **规则引擎 benchmark baseline**（PRD §6.3.2）：
  - `crates/sieve-rules/benches/scan_70_rules.rs`：79 系统规则 × 5KB → P50 327µs（目标 P99 < 1ms，远优于目标）
  - `crates/sieve-rules/benches/scan_with_user_rules.rs`：LayeredEngine 79 系统 + 30 用户 → 336µs（vs system_only 347µs，overhead -3%；early return 优于 PRD 要求的 < 20%）

### Changed

- `OutboundAdapter` / `InboundAdapter` 签名改泛型化（向后兼容默认 `= VectorscanEngine`）
- `is_excluded` 从 `VectorscanEngine` 方法迁移为模块级 `is_excluded_by_rule` 函数（适配泛型）
- `crates/sieve-cli/Cargo.toml` 加 `regex = "1"`（engine_adapter 直接用 regex 处理 allowlist）
- `crates/sieve-rules/Cargo.toml` 加 2 个 `[[bench]]` 入口

### Deferred to v2.1

- `peer_addr_to_pid` 真实实现（macOS `proc_listpids` 反查 TCP peer port → PID）
- 入站 SSE 路径灰名单写入（`HoldOutcome` 枚举需扩展携带 `remember` 字段）
- OpenAI 出站灰名单 lookup（与 Anthropic 路径对称）
- daemon 命中/决策处接入 `AuditStore::append`，传入真实 `CallerContext`
- 用户规则 `direction` 字段（按方向分流到 outbound / inbound 两侧）
- vectorscan_rs 暴露 `hs_database_size()` 后补 PatternDbTooLarge 1MB 上限校验
- criterion CI gate 集成（P99 退化 > 10% 失败）

### Test

- `cargo test --workspace --no-fail-fast`：**586 passed / 1 failed（已知 doctor 竞态，单跑通过）/ 3 ignored**（v2.0-alpha-A-skeleton 时 546 → 加 40 个新测试）
- `cargo fmt --all -- --check`：clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：clean

---

## [v2.0-alpha-A-skeleton] - 2026-05-01

### 背景

PRD v2.0 Phase A 第一批落地：5 项 HIPS 升级能力（用户规则 / 三态 ask / 引擎抽象 / 进程上下文 / audit migration）的代码骨架。daemon 接入推 Week 6（本批仅 ship 模块本身 + CLI 入口）。

### Added

- **新 crate `sieve-policy`**（1680 行 + 40 测试）：
  - `loader`：解析 `~/.sieve/rules/user.toml`，含 0600 文件权限 / 0700 目录 / no-follow symlink 三道安全校验（PRD §5.5.3-C）
  - `lint`：11 类用户规则约束（A 语义 / B 资源 / C 文件系统）（PRD §5.5.3）
  - `engine::UserEngine`：包装 `VectorscanEngine`，hits 自动加 `user:` 前缀（PRD §5.5.2.1）
  - `graylist`：`~/.sieve/decisions/<digest>.json` 灰名单 add/lookup/remove，含 Critical 锁（add 前调 `is_critical_locked`，命中 → `PolicyError::CriticalRuleNotGraylistable`）（PRD §5.4.2）
- **`sieve-rules` engine trait 扩展**（向后兼容）：新增 `ScanRequest` / `ScanReport` / `Direction` / `Protocol` / `ContentKind` + `LayeredEngine<S, U>`（系统先行 + Critical 命中立即返回，PRD §6.3.1）
- **`sieve-cli/src/process_context.rs`**（310 行 + 5 测试）：macOS `proc_pidinfo` + `proc_pidpath` PID → CallerInfo 反查，30s LRU cache，反查耗时 P99 < 1ms；非 macOS 返 None stub（PRD §5.6 / §6.6）
- **`sieve-cli` audit schema migration**（v1 → v2）：events 表加 `caller_pid INTEGER` + `caller_exe TEXT` 两列（NULL 可），`PRAGMA user_version` 跟踪版本，全新 DB 直接 v2，老 DB 在事务里 ALTER TABLE；append-only 触发器迁移后仍生效；`AuditEvent` 各 variant 加共享 `CallerContext { pid, exe }` 子结构（serde default 兼容旧 JSON）（PRD §5.6.1）
- **`sieve-ipc` 三态决策协议扩展**（serde default 100% 向后兼容 v1.5）：
  - `DecisionRequest.allow_remember: bool`（daemon 端必须用 `is_critical_locked` 计算，内置 Critical 强制 false）
  - `DecisionResponse.context_hint: Option<String>`（GUI 用户备注，写灰名单 entry）
  - `DecisionResponse.remember` 加 `#[serde(default)]` + 强化二次校验注释（PRD §5.4.2 三道防线）
- **`sieve rules` CLI 子命令**（PRD §5.5.2 §5.5.5，4 个子命令 + 8 测试）：
  - `edit`：调用 `$EDITOR`（fallback vim/nano）→ 关闭后 `sieve-policy` lint → backup 旧版本（保留 10 份）→ 提示重启 daemon
  - `list`：合并展示用户规则（带 enabled/disabled 状态）+ 系统规则数量摘要（70 入站 + 11 出站）
  - `disable <id>` / `enable <id>`：toml 序列化 + atomic rename（`.tmp` → `user.toml`）+ 0600 重置
  - 模板自动写入 `~/.sieve/rules/user.toml`（首次 edit 时），目录 0700 + 文件 0600

### Changed

- `crates/sieve-cli/src/audit.rs`：370 行 → 640 行，新增 `migrate()` + `CallerContext` 子结构
- `crates/sieve-rules/src/engine/mod.rs`：272 行 → 612 行，加 `MatchEngine` 默认方法（保留旧 `scan(&[u8])` 不破坏调用方）
- `Cargo.toml` workspace 加 `crates/sieve-policy` 成员

### Deferred to Week 6

- daemon 接入 `LayeredEngine` 替换现有 `engine_adapter`
- 灰名单查询挂入 daemon 决策路径（命中 → 跳过 GUI 弹窗直接 Allow）
- `sieve rules edit` 完成后 IPC notify daemon hot-reload
- 进程上下文实际写入 audit 路径
- 用户规则 e2e 测试矩阵（PRD §9 #16 4 类 content-type 组合）
- `compiled_pattern_size_bytes` 等 vectorscan_rs 暴露 `hs_database_size()` 后补 1MB 上限

### Test

- 全 workspace `cargo test --workspace --no-fail-fast`：**546 passed / 1 failed（已知 doctor 竞态，单跑通过）/ 3 ignored**
- `cargo fmt --all -- --check`：clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：clean

---

## [v2.0-hips-readiness] - 2026-05-01 [BREAKING]

### 背景

依据 HIPS Readiness Assessment 评估"Sieve 当前 HIPS 70%"，启动**完整 HIPS（Host-based Intrusion Prevention System）改造**。目标 GA 时达到 HIPS 90%。GA Week 12 时间表不变，Phase A（Week 5-8）+ Phase B（Week 9-12）拆分。

经历 4 轮 review feedback：用户 feedback #1/#2 + 自查 feedback #3 范围瘦身 + review feedback #4（9 Must + 3 Should 全部落地），详见 PRD v2.0 §15 changelog + docs/review/2026-05-01-review-prd-v2.0.md。

### Added

- **PRD v2.0**：docs/prd/sieve-prd-v2.0.md（932 行，HIPS 改造全套设计）
- **6 个新 ADR**（v2.0 核心决策）：
  - [ADR-020](../design/ADR-020-user-rules-system.md) 用户规则系统
  - [ADR-021](../design/ADR-021-tri-state-decision-and-graylist.md) 三态决策 + 灰名单 + Critical 锁
  - [ADR-022](../design/ADR-022-behavior-sequence-window.md) 行为序列联动窗口
  - [ADR-023](../design/ADR-023-process-context-audit.md) 进程上下文记录
  - [ADR-024](../design/ADR-024-rules-engine-abstraction.md) 规则引擎抽象
  - [ADR-025](../design/ADR-025-content-type-routing-matrix.md) content-type 路由矩阵
- **新增 1 个 crate**：`sieve-policy`（用户规则加载 + lint + 灰名单 + $EDITOR pipeline，Phase A 落地）
- **3 条新硬约束**（PRD v2.0 §9）：
  - **#14 用户规则 fail-safe**：用户规则错误绝不影响系统规则功能；用户规则只能 High Ask/Warn/Mark，不能 override 系统 Critical
  - **#15 行为序列保守起步 + GA 默认关闭**：IN-SEQ-* 仅 StatusBar 通知 + `[features] sequence_detection = false` 默认关闭 + GA 不承诺行为序列
  - **#16 content-type 路由矩阵硬约束**（v1.5.4 P0 教训永久化）：所有新增入站功能必须有集成测试覆盖 4 类组合（Anthropic SSE/JSON × OpenAI SSE/stream=false JSON）
- **3 条新检测项**（v2.0 Phase B beta）：IN-SEQ-01-RECON-EXFIL / IN-SEQ-02-CLEANUP-AFTER-ATTACK / IN-SEQ-03-PERSISTENCE-CHAIN
- **三态决策灰名单 schema**：`~/.sieve/decisions/<digest>.json`（0600 权限 + atomic rename + no-follow symlink + 所有变更写 audit.db）
- **进程上下文 audit 字段**：`caller_pid` + `caller_exe`（cwd/ppid 推 v2.1）
- **行为序列窗口结构化特征**：`tool_class` / `path_category` / `network_egress` / `persistence_mech` / `cleanup_mech` / `sensitive_file_hint`
- **LayeredEngine 合并顺序约束**：系统 Critical 命中立即返回；非 Critical → 追加用户规则 hits（"user:" 前缀）；用户规则不能 suppress 系统命中
- **`sieve rules` 4 个核心子命令**（Phase A）：`edit`（调 $EDITOR + lint pipeline）/ `list` / `disable` / `enable`

### Changed [BREAKING]

- **PRD §9 硬约束从 13 条扩展到 16 条**（v1.5 → v2.0），新增 #14/#15/#16 详见 .cursorrules §二
- **`docs/requirements/PRD-sieve.md` 指针**从 v1.5 升级到 v2.0
- **`.cursorrules` §二** 同步 16 条硬约束 + §3.3 加 sieve-policy crate 边界
- **MatchEngine trait 接口**（v2.0 Phase A 实施时落地）：`scan(&[u8])` → `scan(ScanRequest) -> ScanReport`，调用方需迁移传递上下文（direction / protocol / content_kind / tool_name / source_agent / caller_exe）
- **IPC 协议扩展**（ADR-013）：`sieve.request_decision` 加 `allow_remember: bool`（daemon 计算）+ `sieve.decision_response` 加 `remember: bool` + `context_hint: Option<String>`，daemon 收到 remember=true 必须二次校验 critical_lock

### Decisions（v2.0 review 后撤回的方案）

- **撤回 sieve-interceptor crate / 拦截引擎抽象**：v2.0 第一版只做 macOS，daemon.rs 当前形态就是事实上的 MacOSInterceptor，强行抽象成 trait 收益要 v2.1+ 多平台时才用得上 → 推 v2.1（PRD §6.4）
- **撤回 ratatui TUI 规则编辑器**：改 `$EDITOR` + lint pipeline，工程量从 5-7 天降到 1-2 天 → GUI 推 v2.1
- **撤回 AI 辅助生成规则**：v2.0 不引入云端 LLM 依赖 → v2.1 评估（OQ-V20-05）
- **撤回 caller_cwd / caller_ppid 字段**：macOS 需 entitlements + 用户授权弹安全提示，部署摩擦大 → 推 v2.1

### Phase 拆分

- **Phase A（Week 5-8）**：用户规则 + 三态 ask（含 Critical 锁）+ 规则引擎抽象 + 进程上下文记录 + `$EDITOR` 编辑器
- **Phase B（Week 9-12，beta 默认关闭）**：行为序列窗口 + 3 条 IN-SEQ-* 启发式 + 双路径不变量
- **GA Week 12** ship features：用户规则 + `$EDITOR` 编辑 + 三态 ask + 进程上下文 + 行为序列 beta opt-in

### 风险登记新增 8 条

R-V20-01~08（用户规则被诱导加白 / 行为序列 FP 失控 / 灰名单绕过 Critical / content-type 路由回归 / 用户规则资源耗尽 / 进程归因错误 / Phase A 范围过载 / 行为序列串会话），详见 PRD v2.0 §12。

### Migration

- 现有 v1.5 系统规则 / 规则文件 / 拦截 pipeline **完全兼容**，v2.0 是叠加扩展不是替换
- 用户无 user.toml 时 daemon 行为与 v1.5 一致（系统规则全功能）
- IPC 客户端不实现 `allow_remember` 字段时 daemon 按 v1.5 行为处理（兼容默认 false）
- audit.db schema migration v1 → v2（自动加 caller_pid/exe 字段，旧记录 NULL）

---

## [v1.5.4-non-streaming-json-inbound-fix] - 2026-05-01

### 背景

Week 4 dogfood 实测发现的 P0 安全漏洞，标记"必须 Week 4 关闭"，至今未关。子代理修复时**顺手发现第二个隐蔽 bug**：OpenAI 路径 `stream=false` 分支原本直接 `forward_raw` 完全跳过入站检测——OpenAI 默认就是 stream=false，意味着 OpenAI 入站检测从来没生效过。

### Fixed [SECURITY]

- 🔴 **P0 漏洞 1（Anthropic 路径）**：daemon 当前只扫 `text/event-stream` SSE 流，`application/json` 非流式响应里的 `tool_use` **整体绕过所有入站规则**（IN-CR-02/03/04/05 / IN-GEN-* 全失效，含 v1.5.x 70 条新规则）。修复：`forward_with_inbound_inspection` 在收上游响应后按 Content-Type 路由，JSON 路径走新增 `handle_anthropic_json_inbound()` 解析 `content[]` → 提取 tool_use → 喂 `InboundFilter::on_tool_use_complete` → Critical 命中替换 body 为 `sieve_blocked` JSON

- 🔴 **P0 漏洞 2（OpenAI 路径，子代理顺手发现）**：`proxy_openai` 的 `stream=false` 分支原本直接 `forward_raw`，**跳过整个入站检测路径**——OpenAI 协议默认就是 stream=false，意味着 OpenAI 入站规则从来没生效过。AutoRedact 的 stream=false 分支同样问题。修复：改为调用 `forward_with_openai_inbound_inspection` 让 Content-Type 路由处理；新增 `handle_openai_json_inbound()` 解析 `choices[].message.tool_calls[]`

### Added

- 4 个新 helper 函数（`crates/sieve-cli/src/daemon.rs`）：
  - `handle_anthropic_json_inbound`：收 body → 解析 `AnthropicResponse.content[]` → tool_use 提取 → InboundFilter
  - `handle_openai_json_inbound`：同上但解析 OpenAI ChatCompletion 格式
  - `build_sieve_blocked_json_body` / `build_sieve_blocked_json_response`：构造非 SSE 格式的拦截响应（保持 PRD §9 #11 协议层诚实，body 内嵌 `sieve_blocked` 字段不冒充 model）
  - `passthrough_json_response`：未命中时原样转发（重建 headers）

- 2 条新集成测试（`crates/sieve-cli/tests/inbound_block.rs`）：
  - `anthropic_non_streaming_json_inbound_block`：mock 上游返回 `application/json` + `eth_signTransaction` tool_use → 验证 daemon 替换 body
  - `openai_non_streaming_json_inbound_block`：同款 OpenAI Chat Completions 格式 + `tool_calls` 数组
  - 新增 `spawn_mock_json_upstream` 测试 helper

### Changed

- `proxy_openai` `stream=false` 分支：`forward_raw` → `forward_with_openai_inbound_inspection`
- AutoRedact `stream=false` 同款修复

### 测试结果

| 指标 | 修复前 | 修复后 |
|------|------|------|
| Anthropic 非流式 JSON tool_use 拦截 | ❌ 完全绕过 | ✅ block |
| OpenAI 非流式 tool_calls 拦截 | ❌ 完全绕过（含 stream=false 默认场景）| ✅ block |
| inbound_block 集成测试 | 12 passed | **14 passed**（+2 新增）|
| dataset_fp_rate | 0% FP / 99.71% recall | **不变**（无回归）|
| cargo fmt --check / clippy -D warnings | pass | pass |

### Impact 评估

**这是 v1.5.x 系列发现的最严重漏洞。**Anthropic SDK 和 OpenAI SDK 的常见配置中，**非流式响应使用率 50%+**——这意味着 v1.5.x 累计加的 70 条入站规则在这些场景**实际拦截率为 0%**。本 patch 把"纸面 70 条规则"恢复到"实际 70 条规则"。

PRD §9 #7 实测数字（FP 0% / Recall 99.71%）此前**仅在 SSE 流模式有效**，本 patch 后才在所有响应模式生效。

---

## [v1.5.3-windows-powershell-pipe] - 2026-05-01

### 背景

v1.5.2 公开攻击复现报告里发现 CVE-2025-6514 Windows PowerShell 变种漏拦（mcp-001），已标记为"建议 Week 6 补"。本次 patch 直接补上，1 条规则一行一行测，10 分钟工作量。

### Added

- **`IN-CR-02-CURL-PIPE-WIN`**：`(?i)(curl|iwr|invoke-webrequest)(?:\s+\S+)+\s*\|\s*(powershell|pwsh)(\.exe)?` —— 覆盖 Windows `curl ... | powershell` / `iwr ... | pwsh` 形态。CVSS 9.6 / 437K+ 下载量影响（CVE-2025-6514 mcp-remote OS Command Injection RCE）。Critical + hook_terminal + 30s timeout default block。allowlist 7 条教学/审计豁免

### 测试结果

| 指标 | v1.5.2 终点 | v1.5.3 | 状态 |
|------|------|------|------|
| Critical FP | 0/1070 = 0% | **0/1070 = 0%** | ✅ |
| Attack Recall（合成） | 694/696 = 99.71% | **694/696 = 99.71%** | ✅（无回归） |
| Public Attack Replay | 51/55 = 92.7% | **52/55 = 94.5%** | ✅ |
| mcp-supply-chain 子桶 | 8/9 = 88.9% | **9/9 = 100%** | ✅ |

### Known limitations

剩 3 条公开复现漏拦全部为**接受盲区**（不计划修）：
- `owasp-003.txt`（OWASP LLM03 RAG 投毒）：无 payload 特征，需源头数据校验
- `real-003.txt` / `real-006.txt`：真实事件中的前端 UI 钓鱼 / 社工邮件，不在 LLM 流量范围

入站规则总数 69 → **70**。

---

## [v1.5.2-blind-spots-and-public-replay] - 2026-05-01

### 背景

v1.5.1 完成后剩 20 条 attack 漏拦记录在 `tasks/2026-05-01-rule-gaps.md`，本次清光（剩 2 条不可能 vectorscan 解决，已说明）。同时做方案 C：复现 55 条**公开发生过的真实攻击**作为信任基线 —— 合成数据再多也不如"已知 CVE / 已黑客攻击事件"有说服力。

### Added

- **BIP39 入站 second-pass**（IN-CR-03-BIP39-INBOUND）：复用 outbound 已有的 `candidate_bip39_windows` + `verify_checksum`，在 `engine_adapter.rs::InboundAdapter::scan_text` 加同款代码块。命中 → Critical + `gui_popup` 60s 弹窗（模型诱导用户输入助记词的场景）。`__BIP39_SECOND_PASS_PLACEHOLDER__` 占位规则跳过 vectorscan 编译（参考 IN-CR-01 写法）

- **12 条新入站规则**（inbound.toml 57 → 69）：
  - **6 条边缘形态**：IN-CR-03-ENV-STANDALONE（`env\n`）、IN-CR-02-CURL-MULTILINE-SECRET（curl + `$VAR` 跨行）、IN-CR-02-PYTHON-OSPOPEN-CURL、IN-CR-02-NPM-PACKAGE-REDIRECT（package.json `npm:@attacker/pkg`）、IN-CR-02-NPM-POSTINSTALL-AUTO、IN-CR-03-PRINTENV pattern 修正（去掉末尾 `\b`）
  - **6 条 BIP39 社工/标签规则**（覆盖 checksum 不通过但有社工话术的场景）：IN-CR-03-BIP39-ENTER-PHRASE、IN-CR-03-BIP39-SEED-VAR、IN-CR-03-BIP39-COLON-LABEL、IN-CR-03-BIP39-TOOL-CMD、IN-CR-03-BIP39-MULTILINGUAL（High）、IN-CR-03-LEDGER-SEED-VERIFY

- **公开攻击复现数据集 55 条**（`bench-data/attacks-public-replay/`，每条带可追溯 URL）：
  - `rugpull-ai/` × 9：KuCoin AI Agent / ElizaOS arXiv 2503.16248 / Unit 42 IDPI / Step Finance $40M
  - `injection-pocs/` × 10：Oasis "Claudy Day" / Lasso Claude Code / PromptArmor / Docker MCP / Aikido PromptPwnd
  - `ctf-replays/` × 8：Awesome Prompt Injection / Open-Prompt-Injection / PromptMap / arXiv HTML AT bypass
  - `owasp-llm-top10/` × 9：LLM01-09 各一条 crypto 场景复现
  - `real-events/` × 10：Ledger 2023-12-14 / Multichain $126M / Lazarus Op99 / Sep 2025 NPM 攻击 / Step Finance
  - `mcp-supply-chain/` × 9：CVE-2025-6514（CVSS 9.6）/ SmartLoader / Oura 2026 / Anthropic MCP 设计缺陷

- **新独立测试**：`public_replay_recall_rate`（dataset_fp_rate.rs）扫 attacks-public-replay/ 输出按子目录命中率，不硬性 assert（揭露盲区目的）

- 任务级文档：`tasks/2026-05-01-public-attack-replay-report.md`（完整漏拦分析）

### Changed

- 总入站规则数 35 → 57 → **69 条**
- 测试数据集 296 → 1896 → **1951 条**（含公开复现 55）

### 测试结果

| 指标 | v1.5.1 终点 | v1.5.2 终点 | 阈值 | 状态 |
|------|------|------|------|------|
| Critical FP | 0/1070 = 0% | **0/1070 = 0%** | < 0.5% | ✅ |
| Attack Recall（合成数据集） | 676/696 = 97.13% | **694/696 = 99.71%** | > 95% | ✅ |
| Public Attack Replay（v1.5.2 新增） | — | **51/55 = 92.7%** | 揭露盲区，无硬阈值 | — |

按桶（合成数据集）：env-leak 100% / shell-rce 100% / signing 100% / private-key 98.33% / 现有 attacks 100% 无回归

按子目录（公开复现）：rugpull-ai 100% / injection-pocs 100% / ctf-replays 100% / owasp-llm-top10 88.9% / real-events 80% / mcp-supply-chain 88.9%

### Known limitations

- **2 条合成数据集仍漏拦**（vectorscan 能力边界，本 sprint 不修）：
  - `fear-privkey-087`：仅"请分享 WIF key"社工话术，无 pattern 特征（FP 风险太高）
  - `fear-privkey-096`：通过 `chrome.storage.local.get` JS API 读 MetaMask `KeyringController.vault`，需 JS 语义分析

- **4 条公开复现漏拦**：
  - 3 条接受盲区（RAG 投毒无 payload 特征 / 纯社工邮件不在 LLM 流量内）
  - **1 条建议下次补**：CVE-2025-6514 Windows `curl ... | powershell` 变种 → 加 `IN-CR-02-CURL-PIPE-WIN` 规则

### 真实事件复现对照（来自 public-attack-replay-report.md）

- **Ledger 2023 Connect Kit 事件**：前端库已被供应链污染，IN-CR-05-ERC2612-PERMIT 在签名前弹窗确认 —— 来源：ledger.com/blog/security-incident-report
- **CVE-2025-6514（恶意 MCP）**：JSON 里藏 curl|bash 拿下开发机，Sieve 在执行前拦截 —— 来源：jfrog.com/blog/2025-6514
- **Lazarus Operation 99（Web3 开发者定向攻击）**：恶意编码挑战的 printenv / 读 Solana keypair / Python urlopen 三类操作分别被三条 IN-CR-03 规则覆盖 —— 来源：thehackernews.com/2025/01/lazarus-group-targets-web3-developers

---

## [v1.5.1-rule-expansion] - 2026-05-01

### 背景

PRD §9 #7 规定 Critical FP < 0.5%、Recall > 95% 是硬约束。Week 4 之前数据集只有 226 attacks + 70 benign，跑出 0% FP / 100% recall —— 数字漂亮但样本太少，对真实用户"一周不误拦"信心不够。本次扩充把 baseline 拉到真实使用决策级别。

### Added

- **测试数据集 296 → 1896 条**（+1600）：
  - `bench-data/attacks-by-fear/{signing,transfer,env-leak,private-key,shell-rce}/` × 120 = 600 条新攻击样本，按"用户最怕的五件事"组织
  - `bench-data/benign-near/{near-OUT-api-keys, near-OUT-tokens, near-OUT-private-keys, near-IN-CR-01-address, near-IN-CR-02-rce, near-IN-CR-03-secret-read, near-IN-CR-04-persistence, near-IN-CR-05-crypto-addr, near-IN-CR-06-misc, extra-generic-multilingual}/` × 100 = 1000 条新 benign 样本，按"看起来像攻击但完全合法"对称分桶（教学/文档/Dockerfile/多语言）
  - 内容多样性预算：60% 真实 web3/dev 文档风格 + 20% 变体 + 10% 多语言（中/日/韩/西/法/德）+ 10% 格式变种（Markdown / JSON tool_use / SSE delta / Dockerfile）

- **22 条新入站规则**（`crates/sieve-rules/rules/inbound.toml`，35 → 57 条）：
  - **env-leak 桶**：IN-CR-03-GREP-CREDS-A/B（grep 扫敏感字段）、IN-CR-03-PRINTENV-CREDS（printenv/env dump）、IN-CR-03-DOCKER-ENV-DUMP（docker exec env）、IN-CR-03-GH-SECRET-LIST（CI secret 枚举）、IN-CR-03-CURL-EXFIL（外发 .env / API key）
  - **private-key 桶**：IN-CR-03-KEYCHAIN-FIND-PASS（macOS Keychain `-w` 导出）、IN-CR-03-METAMASK-VAULT（浏览器扩展 storage 路径）、IN-CR-03-WIN-DPAPI（Windows DPAPI 导出）、IN-CR-03-BITCOIN-DUMPPRIVKEY（`bitcoin-cli dumpprivkey`）、IN-CR-03-GPG-DIR 扩展（含 keystore 目录枚举）
  - **shell-rce 桶**：IN-CR-02-WGET-EXEC（`wget -O /tmp/x.sh && sh`）、IN-CR-02-PYTHON-RCE（`python -c "exec(__import__(...).read())"`）、IN-CR-02-BASE64-PIPE-SH（`base64 -d <<< ... | sh`）、IN-CR-02-MALICIOUS-REGISTRY（npm/pip 非官方 registry）；CURL-PIPE / WGET-PIPE / EVAL pattern 扩展支持 `sudo` 可选前缀
  - **signing 桶**：IN-CR-05-ERC2612-PERMIT（`permit(spender, deadline)` 无限授权签名）、IN-CR-05-WALLETCONNECT-URI（`wc:UUID@1|2` deeplink 签名劫持）

- **bench-data 测试递归读取 + 按桶聚合**（`crates/sieve-rules/tests/dataset_fp_rate.rs`）：
  - `read_samples_recursive` helper 支持子目录扫描
  - 按"桶"输出 per-bucket FP/recall 报告（FP 高时一眼定位是哪类合法场景误伤、recall 漏拦时定位规则盲区）
  - assertion 阈值升级：benign 至少 500 样本（原 50）、attacks 至少 500（原 200）

- 任务级文档：`tasks/2026-05-01-test-data-expansion.md`（计划）、`tasks/2026-05-01-test-data-expansion-report.md`（结果报告）、`tasks/2026-05-01-rule-gaps.md`（已知规则盲区清单，含 BIP39 入站 second-pass 等待后续 sprint）

### Changed

- **规则引擎 `is_excluded` 签名变更**（`crates/sieve-rules/src/engine/mod.rs`，`crates/sieve-cli/src/engine_adapter.rs`，`crates/sieve-rules/tests/inbound_rules.rs`、`tests/dataset_fp_rate.rs`）：
  - 新增 `full_context: &str` 参数，`allowlist_stopwords` 现在在**全文中搜索**而非只在 7-20 字节命中片段里
  - 这是核心机制升级 —— 让"教学语境词"（`the difference between` / `DO NOT RUN` / `compare to a suspicious case` / Dockerfile 安全前缀如 `/var/lib/apt/lists/`）能豁免短命中（如 `eval $`、`rm -rf /`、`systemctl enable`）
  - 调用方需同时传 matched_text + full_context；现有所有调用点已更新

- **22 条新规则 + 9 条现有规则补充 allowlist_stopwords**：教学场景豁免词（"the difference between"、"explain"、"DO NOT"、"NEVER"）、合法初始化（`ssh-agent -s` / `direnv hook` / `starship init` / `pyenv init` / `brew shellenv`）、Dockerfile 安全前缀（`apt/lists`、`var/cache`、`tmp`）、官方 registry 域名（`registry.npmjs.org` / `pypi.org`）等

### 测试结果（PRD §9 #7 验证）

| 指标 | 扩充前 | 扩充后 | 阈值 | 状态 |
|------|-------|-------|------|------|
| Critical FP rate | 0% (70 样本) | **0.00%** (0/1070 样本) | < 0.5% | ✅ 通过 |
| Attack recall rate | 100% (226 样本) | **97.13%** (676/696 样本) | > 95% | ✅ 通过 |

按桶细分：
- benign 11 桶全 0 FP（含 near-IN-CR-02-rce 100/100、near-IN-CR-04-persistence 100/100 等高风险桶）
- attacks-by-fear 4 桶 recall：signing 100% / shell-rce 97.5% / env-leak 97.5% / private-key 88.33%
- 现有 226 attacks 维持 100% recall（无回归）

### Known limitations

- **20 条仍漏拦**（已记录在 `tasks/2026-05-01-rule-gaps.md`）：
  - 14 条在 private-key 桶，全是 BIP39 助记词入站检测，需 second-pass 复用 outbound `validate_bip39`（vectorscan 不适合 alternation 超过 2048 词的 wordlist），推迟到下个 sprint
  - 6 条 shell/env 边缘形态（OS-level 编码绕过 / 内嵌脚本变种），下次迭代加 pattern

### 文档同步

- `docs/design/architecture.md` §5 误报率预算章节加入实测数字
- `docs/guides/development.md` §3.3 补 `cargo test --test dataset_fp_rate -- --ignored` 命令 + bench-data 目录结构说明
- `docs/requirements/user-stories.md` 加 US-21（"用户最怕的五件事"覆盖率验收）

---

## [v1.5-multi-agent] - 2026-04-28

### Architecture（multi-agent 扩展）

- **Phase 1 范围扩展**：Claude Code → Claude Code + OpenClaw + Hermes 三家适配（PRD §9 第 9 条重写）
- **UnifiedMessage 真实运行时支持双协议**：Anthropic Messages API + OpenAI Chat Completions 均解析为同一中间表示，不再仅预留接口（关联 ADR-018）
- **X-Sieve-Origin HTTP header 协议**：sub-agent 嵌套调用追踪，Ed25519 签名防伪造，关联 ADR-019
- **Hook 类规则在 OpenClaw / Hermes 上降级为 GUI hold**：三家 hook 机制不同，最低公分母统一走 GUI hold；Critical fail-closed 规则在三家全保留

### Added

- 2 个新 ADR：ADR-018（OpenAI 协议适配 / OpenAI Chat Completions SSE delta 格式解析）、ADR-019（X-Sieve-Origin header / sub-agent 嵌套调用追踪协议）
- 1 个新 SPEC：SPEC-004（multi-agent setup 配置注入 / `sieve setup --agent` 多 agent 参数）
- **IN-GEN-06**：外部 channel prompt injection 检测（命令式短语 + 来源不可信 channel → Critical GUI hold 60 秒默认拒绝）
- **IN-CR-06**：OpenClaw 动态 skill 加载 fail-closed（Critical block，YOLO mode 不可关）
- `sieve setup --agent claude|openclaw|hermes` 多 agent 参数 + `--all-detected` 自动扫描
- `sieve doctor --all` 验证三家 agent 配置全绿
- `sieve uninstall --all` 一键清理所有 agent 适配（按 `setup.log` 逆序回滚）
- IPC schema 新增字段：`source_agent` / `origin_chain` / `source_channel`（`#[serde(default)]` 向后兼容）
- glossary 新增 8 条 multi-agent 术语：OpenClaw / Hermes / X-Sieve-Origin / chain_depth / origin_chain / source_channel / multi-agent 调用链 / sub-agent 决策传递
- 用户故事新增 US-18（OpenClaw 跨通道 injection 防御）/ US-19（Hermes sub-agent 嵌套决策传递）/ US-20（multi-agent 一键安装）

### Changed [BREAKING]

- **PRD §9 第 9 条重写**（v1.4 → v1.5）：从"Phase 1 仅适配 Claude Code，UnifiedMessage 接口预留"扩展为"Phase 1 GA 适配三家：Claude Code / OpenClaw / Hermes，UnifiedMessage 真实运行时支持 Anthropic + OpenAI 双协议；其他协议（Gemini / Mistral 等）推 Phase 2"
- roadmap Week 6-8 重写：Week 6 = OpenAI 协议适配 + multi-agent setup（8 条任务）；Week 7 = OpenClaw/Hermes 集成测试（5 条任务）；Week 8 = 高强度 dogfood 扩到三家

### Hardened

- `chain_depth ≥ 2` 强制 fail-closed GUI hold，不传递上游决策
- `chain_depth ≥ 5` 直接返回 426，不弹窗（防无限递归调用链）
- X-Sieve-Origin header Ed25519 签名，防伪造注入

### Migration（v1.4 → v1.5）

- v1.4 引擎完全复用：sieve-rules / dispatch / IPC 基础结构 / GUI 弹窗逻辑不变
- v1.4 用户升级 v1.5 不需要重新跑 `sieve setup`（除非要加 OpenClaw / Hermes 适配）
- 老 `sieve.toml` 加新 agent 字段后向后兼容（`#[serde(default)]` 处理缺失字段）
- IPC schema 新增字段向后兼容：旧 sieve-hook 读新 schema 时新字段用默认值，不报错

---

## [v1.4-architecture] - 2026-04-28

### Architecture [BREAKING]

- **处置矩阵二维化**：从一维四级（Critical / High / Medium / Low）→ 二维（出站/入站 × 严重度），规则 manifest 新增 `disposition` / `default_on_timeout` 字段（关联 ADR-016）
- **出站自动脱敏路径**：OUT-01~05/12 高频脱敏类不再弹窗，命中后自动改写请求 body + 状态栏 5s 通知，不打断工作流（关联 PRD §9 第 13 条）
- **HIPS 弹窗架构 + 25s keep-alive comment hold 流**：GUI 弹窗类（IN-CR-01/05）hold SSE 流期间每 25 秒发 `: keep-alive\n\n`，避免 Claude Code HTTP 超时；用户中止时截流注入优雅 error event（关联 SPEC-002）
- **Native GUI App 提到 Phase 1 必做**：macOS SwiftUI 独立进程，独立 git 仓库 `sieve-gui-macos`（不在本 workspace）（关联 ADR-012）
- **双层防御**：Sieve 代理（出站脱敏 + GUI hold 流）+ Claude Code PreToolUse hook（Hook 类阻断）；`sieve-hook` 作为独立 crate 加入 workspace（关联 ADR-014）

### Added

- 新 crate `sieve-ipc`：JSON-RPC over Unix socket + 文件锁 IPC 库（pending / decisions 文件协议，关联 ADR-013）
- 新 crate `sieve-hook`：极简 PreToolUse hook 二进制，依赖仅 `serde_json` + `fd-lock` + `uuid` + `chrono` + `clap`，启动时延 4~5ms（关联 SPEC-001）
- 5 个新 ADR：ADR-012（Native GUI App）、ADR-013（IPC 协议）、ADR-014（双层防御）、ADR-015（sieve setup 工具）、ADR-016（处置矩阵二维化）
- 3 个新 SPEC：SPEC-001（sieve-hook 协议）、SPEC-002（HIPS 弹窗行为）、SPEC-003（sieve setup 工具）
- `sieve-rules` manifest 新字段：`disposition`（AutoRedact / GuiPopup / HookTerminal / StatusBar）、`timeout_seconds`、`default_on_timeout`
- `critical_lock` 新常量：`HOOK_RULES`（25 条 Hook 类规则）、`GUI_RULES`（11 条 GUI 弹窗类规则）；`FAIL_CLOSED_RULES` 扩展为 24 条 Critical 全集

### Changed [BREAKING]

- ADR-007 Week 3 落地的"SSE 流 `sieve_blocked` 截流"对 Hook 类（IN-CR-02~04 / IN-GEN-01~03）的实现改由 `sieve-hook` 在 PreToolUse 阶段完成；fail-closed 原则不变，实现路径变（ADR-014 supersede ADR-007 中 Hook 类相关部分）
- Phase 1 仅 macOS：Linux / Windows 推 Phase 2；sigstore CI 保留多平台编译，`sieve setup` 命令在非 macOS 报友好错误

### Hardened — 新增 PRD §9 硬约束第 11-13 条

- **第 11 条（不在 Anthropic API 协议层撒谎）**：不伪造 tool_use / stop_reason / id / usage / type；拦截发生时允许截 SSE 流注入 `sieve_blocked` event（Sieve 自报事件，不是冒充模型）；keep-alive comment 行 `: keep-alive\n\n` 不属于伪造
- **第 12 条（不装本地 CA 做 MITM）**：Network Extension / 本地 CA 注入 / 系统 proxy 修改均推 Phase 3 选购，Phase 1/2 不做
- **第 13 条（出站脱敏不打断工作流）**：OUT-01~05/12 高频类必须自动脱敏 + 状态栏 5s 通知，不弹窗；每天弹几十次的产品没人用

### Removed / Deprecated

- 撤销 `architecture.md §6` 中的"❌ 桌面 GUI App（Electron / Tauri）"决策（被 ADR-012 翻转）
- 撤销 `deployment.md §2.2 §2.3` Linux/Windows 安装详细步骤（推 Phase 2，保留占位段）

### Migration

- Critical 规则永远 fail-closed，不允许通过任何 preset 关闭
- 旧 `sieve.toml` 加新字段后向后兼容（`#[serde(default)]` 处理缺失字段），但旧 sieve binary 读新 toml 会因 `unknown_fields` 失败
- `pre-v1.4-refactor` git tag 标记重构基线，回滚：`git checkout pre-v1.4-refactor`

---

## [Unreleased]

### Known Issues — Week 4 dogfood 实测发现

#### 🔴 P0：入站检测仅覆盖流式 SSE，非流式 JSON 响应里的 tool_use 完全绕过
- 当前 `daemon::forward_with_inbound_inspection` 把 response body 喂给 `SseParser` +
  `Aggregator` + `InboundFilter`，假定响应是 `text/event-stream` 字节流
- Anthropic Messages API 同样支持非流式调用：客户端不传 `stream:true`（或显式
  `stream:false`）时，响应是单个 `application/json` body。Sieve 当前对这种响应
  原样透传，**所有入站规则失效**——IN-CR-02 / IN-CR-03 / IN-CR-04 / IN-CR-05 /
  IN-GEN-* 都被绕过
- 攻击者只需让 SDK 发非流式请求，就能让模型在 tool_use 里写 `>> ~/.bashrc` /
  `eth_signTransaction` / `rm -rf /` 而 Sieve 完全看不到。PRD §5.2（入站检测是 Sieve 的核心能力）语境下属严重产品级缺陷
- **修复进度**：roadmap Week 4 加入硬阻塞项，2026-05-04 前必须关闭。修复路径：
  daemon 按 response content-type 路由，JSON 分支解析 `AnthropicResponse.content[]`
  → 提取 tool_use → 走 `InboundFilter::on_tool_use_complete`；fail-closed Critical
  时把 body 替换为 `sieve_blocked` 等价 JSON。集成测试加 mock 非流式 upstream
  覆盖
- 详见 tasks/lessons.md「入站检测仅覆盖流式 SSE」 /
  tasks/roadmap.md Week 4

#### claude `-p` headless 默认走 OAuth 直连（dogfood 操作记录）
- Claude Max OAuth 优先级高于 `ANTHROPIC_BASE_URL`，非 `--bare` 模式 claude CLI
  会忽略代理直连 claude.ai 后端
- 必须用 `claude --bare -p` 强制走 `ANTHROPIC_API_KEY` auth 路径才会经过代理
- 不影响 Sieve 代码，只影响 dogfood 流程；development.md 待补「dogfood / 调试」段
- 详见 tasks/lessons.md

### [BREAKING] — Week 4 (2026-04-27)

#### rule ID 重命名：旧 `IN-CR-04` markdown exfil → `IN-GEN-04`
- 原 Week 3 落地的 markdown 图片 exfil 规则错置于 `IN-CR-*`（Crypto 钩子）命名空间。
  按 PRD §5.2，`IN-CR-04` 应是持久化机制；markdown 通用 exfil 归 `IN-GEN-*`
- 行为不变：仍是 high warn / 不入 fail-closed 名单 / 不阻断流量
- **fingerprint 失效**：fingerprint = `sha256(rule_id || matched_text)`，rule_id 改名
  → `~/.sieve/sieveignore` 中以旧 IN-CR-04:* 开头的条目自动失效。该阶段仅内部
  dogfood，无外部用户影响

### Added — Week 4 持久化机制（IN-CR-04 全套）

#### 入站持久化机制检测（IN-CR-04，PRD §5.2 / Roadmap Week 4 / US-07）
- 9 条 IN-CR-04-* 子规则，全部 **Critical block + fail-closed**（YOLO mode 不可关，进
  `FAIL_CLOSED_RULES` 名单），覆盖主流后门埋点路径：
  - `IN-CR-04-SHELL-RC-APPEND`：`>>` / `tee -a` 写 `.bashrc` / `.zshrc` / `.bash_profile` /
    `.zprofile` / `.profile` / `.bash_aliases` / `.kshrc` 等
  - `IN-CR-04-CRONTAB`：`crontab -e` / `<` / `-r`（编辑 / install / 删除；`-l` 仅查看不命中）
  - `IN-CR-04-CRON-D-WRITE`：写 `/etc/cron.{d,daily,hourly,monthly,weekly,allow,deny}/`
  - `IN-CR-04-LAUNCHCTL`：`launchctl load` / `bootstrap` / `enable` / `kickstart` / `asuser`
  - `IN-CR-04-LAUNCH-AGENT-PLIST`：写 `~/Library/LaunchAgents/*.plist` 或
    `/Library/LaunchDaemons/*.plist`（要求 `>` / `cp` / `mv` / `tee` / `cat >` 写意图前缀）
  - `IN-CR-04-SYSTEMCTL-ENABLE`：`systemctl enable` / `--user enable` / `start` /
    `daemon-reload`（不拦 `disable` / `stop` / `status`）
  - `IN-CR-04-SYSTEMD-UNIT-WRITE`：写 `/etc/systemd/system/*.{service,timer,socket}` 或
    `~/.config/systemd/user/*.{service,timer,socket}`
  - `IN-CR-04-FISH-CONFIG`：`>>` / `tee -a` 写 `~/.config/fish/config.fish` / `conf.d/*.fish`
  - `IN-CR-04-LOGIN-ITEMS`：macOS `defaults write LoginItems` 或
    `osascript ... login items`
- 设计原则：pattern 锚定 Bash 命令的"写意图"（重定向 / tee / crontab 编辑 / launchctl
  load / systemctl enable 等），不拦读路径——避免与 IN-CR-03 read=High 处置冲突
- 已知 gap：Edit/Write 类工具直接写持久化文件（无 Bash 重定向）不被 IN-CR-04 直接命中；
  但配套启用动作（launchctl load / systemctl enable）仍会触发，多步攻击链至少一处被截断
- ADR-007 §"Week 4 落地范围"已记录 traceability，无需新 ADR

#### critical_lock 扩展（sieve-rules）
- `FAIL_CLOSED_RULES` 加 9 条 IN-CR-04-* 子规则
- 新增单测 `in_cr_04_persistence_fail_closed` 验证 9 条全部 fail-closed
- `enforce_action` 单测加 IN-CR-04-CRONTAB 验证 manifest action=Warn 仍强制 Block

#### 测试
- sieve-rules 新增 14 个单测（9 IN-CR-04 正例 + 4 关键 benign 反例 + 1 unrelated benign）
  + 1 critical_lock 单测
- sieve-cli 新增 1 个端到端集成测试（`in_cr_04_persistence_shell_rc_blocked`）：
  tool_use Bash command `>> ~/.bashrc` → SSE 注入 sieve_blocked 含 IN-CR-04-SHELL-RC-APPEND
- 全 workspace 192/192 测试通过（174 → 192，+18，零回归）

### Added — Week 4 进行中 (2026-04-27 起)

#### 入站敏感路径检测（IN-CR-03，PRD §5.2 / Roadmap Week 4）
- 10 条 IN-CR-03-* 子规则，全部 high warn（非 fail-closed Critical，给用户判断空间），完整覆盖 US-07 验收清单（SSH / AWS / GCP / Solana / Ethereum keystore）：
  - `IN-CR-03-SSH-PRIVATE`：SSH 私钥文件名（id_rsa / id_ed25519 / id_ecdsa / id_dsa），allowlist `*.pub`
  - `IN-CR-03-SSH-DIR`：`~/.ssh/...`，allowlist `known_hosts` / `authorized_keys` / `config` / `environment`
  - `IN-CR-03-AWS-CREDS`：`~/.aws/credentials`（不拦 `~/.aws/config`）
  - `IN-CR-03-DOTENV`：`.env` / `.env.local` / `.env.production` 等，allowlist `.env.example` / `.template` / `.sample` / `.dist` / `.test` / `.ci`
  - `IN-CR-03-ETH-KEYSTORE`：geth keystore 文件名 `UTC--<timestamp>--<40hex>`
  - `IN-CR-03-GPG-DIR`：`~/.gnupg/...`
  - `IN-CR-03-NETRC`：`.netrc` 凭据文件
  - `IN-CR-03-MACOS-KEYCHAIN`：`login.keychain-db` / `System.keychain`
  - `IN-CR-03-GCP-CREDS`：`~/.config/gcloud/application_default_credentials.json` / `legacy_credentials/`
  - `IN-CR-03-SOLANA-KEYPAIR`：`~/.config/solana/*.json`（CLI 默认 keypair 位置）
- 复用 IN-CR-02 已有 `engine_adapter::check_tool_use` 通道——`tool.input` JSON 序列化后喂给 vectorscan，无需新增扫描器
- daemon 当前仅记录 detection 到日志（`tracing::warn!`），不阻塞流量；5s 倒计时弹窗 UI 留 Week 5

#### Engine: longest-match-per-start dedup（sieve-rules::engine）
- `VectorscanEngine::scan` 对带量词（`{m,n}` / `(?:..)*`）的 pattern 在多个 endpoint 触发的回调，按 `(rule_id, start)` 去重保留最长 end
- 修复 IN-CR-03-DOTENV / SSH-DIR allowlist 失效问题（短 match `.env` 拿不到完整文件名上下文，绕过 `\.env\.example` 白名单）
- 输出排序确定（`(start, rule_id)` 字典序），下游处理与测试可重现
- 副作用：OUT-06 JWT / OUT-08 Stripe 等贪婪 pattern 现在每个起点仅返回最长 match，detection 数量减少但语义不变

#### 测试
- sieve-rules 新增 16 个单元测试（IN-CR-03 8 正例 + 6 allowlist 反例 + 2 通用 benign）+ 1 个引擎 dedup 测试
- sieve-cli 新增 1 个端到端集成测试（`in_cr_03_sensitive_path_warn_passes_through`）
- 全 workspace 174/174 测试通过（Week 3 → Week 4：158 → 174，+16 新增 0 回归）



#### 入站 SSE 流式处理（sieve-core）
- `SseParser`：增量解析器，push_chunk + flush 接口，**无缓冲整流**
- 5 类边界全覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流
- `Aggregator`：Tool Use partial_json 跨 SSE event 聚合，content_block_stop 后 deserialize
- `StreamingPipelineNode` trait：observe_event / on_tool_use_complete / on_message_stop
- `InboundFilter` impl：入站文本扫描 + tool_use 检查 + .sieveignore 过滤
- `InboundEngine` trait（由 sieve-cli engine_adapter 桥接到 VectorscanEngine）
- `AddressGuard`：IN-CR-01 地址替换检测，strsim Levenshtein，distance ∈[1,3] 且 len 相等触发
- ToolUseBlock 加 `span: Option<ContentSpan>` 字段

#### 入站规则集（sieve-rules/rules/inbound.toml）
- IN-CR-01 地址替换（占位，实际由 AddressGuard strsim 实现）
- IN-CR-02 危险 shell 命令（rm -rf / curl|sh / wget|sh / eval / nc 反弹 / dd 擦盘）
- IN-CR-04 markdown exfil（候选，warn）
- IN-CR-05 签名工具白名单（EVM eth_signTransaction 等 / Solana signTransaction 等 / Bitcoin signRawTransaction 等）
- IN-GEN-01 markdown javascript: URI（critical block）
- IN-GEN-02 inline HTML img（warn）
- IN-GEN-03 bash -c 任意执行（critical block）
- 共 13 条规则，vectorscan PCRE 子集兼容

#### Critical fail-closed 强制（sieve-rules::critical_lock）
- `FAIL_CLOSED_RULES` 常量（出站 OUT-01~12 全部 + 入站 11 条 critical 规则）
- `is_fail_closed(rule_id)` + `enforce_action(rule_id, requested) -> Action`
- 运行时检查：在 OutboundAdapter / InboundAdapter scan 时调用，即使 manifest action 写 allow / mark 也强制为 Block

#### Daemon 入站集成（sieve-cli）
- `daemon::run` 新签名：`(cfg, OutboundFilter, Arc<dyn InboundEngine>, Arc<HashSet<String>>)`
- `forward_with_inbound_inspection`：SSE tee 透传 + 同步 SseParser/Aggregator/InboundFilter，Critical 命中**注入 sieve_blocked event 然后关 channel**（用户拍板：截流策略）
- **剥掉响应 content-length 头**强制 chunked transfer（否则注入 sieve_blocked 时 body 长度对不上 content-length 会被客户端截断）
- `engine_adapter::InboundAdapter` 实现 `sieve_core::InboundEngine`
- main.rs 启动加载 inbound rules，partition __ADDRESS_GUARD_PLACEHOLDER__ 占位规则不传 vectorscan
- `audit_yolo_disabled`：运行时检查 config 字段（deny_unknown_fields 已防御 + 此函数双保险 tracing）

#### Fuzz 双引擎（关联 PRD §9 #5 硬约束）
- `fuzz/`（libFuzzer + cargo fuzz）：3 个 fuzz_target（sse_parser / tool_use_aggregator / inbound_filter）
- `fuzz_afl/`（AFL++ + afl crate）：3 个对应 target
- `fuzz/corpus/sse_parser/` 14 个 seed：正常流 / partial_json 跨 chunk / ping 穿插 / error 中途 / 半行 chunk / C0 控制 / 多 event 粘包 / 提前 EOF / 未知 event / 8KB partial_json / UCSB 4 类攻击 PoC
- `sieve_core::fuzz_helpers` 模块（双引擎共享函数体）
- ci.yml 加 fuzz-quick job：每 push 跑 60s/target 总 3 分钟
- fuzz-nightly.yml 骨架（workflow_dispatch only，Week 6+ 启用 schedule）

#### Bug 修复（Week 2 遗留）
- 出站 dry_run + Critical bug：Week 2 实现允许 dry_run + Critical 时仍 forward 到上游，违反 ADR-007 fail-closed。**Week 3 修复**：fail-closed 名单中的规则在 dry_run 下仍返 426
- 入站截流时 sieve_blocked event 注入与 content-length 冲突：剥掉响应 content-length 强制 chunked

#### 测试与验证
- 单元 + 集成测试 138 个全过（增量：sieve-core 25→56 / sieve-rules 29→50+12 / sieve-cli 15+3+5→15+5+3+5）
- Python smoke 29/29（原 26 + 入站 benign 透传 1 + 真 key 测试 2）
- UCSB 4 类攻击 PoC 测试（`tests/inbound_block.rs` 5 项）全部 ✓
- release 二进制 9.1 MB（Week 2 9.0 → +0.1 MB，strsim + 入站规则）
- cargo bench 编译通过（Week 4 实测性能）
- cargo deny licenses bans sources 全过（加 NCSA for libfuzzer-sys + MIT for fuzz crates）

### Pending（Week 4 起）
- 完整 secret benchmark 数据集（200-500 攻击样本 + 50-100 benign 真实会话回放）
- IN-GEN-04~05 完整规则
- 主动 macOS / TUI 弹窗（Action::WarnConfirm 实现）
- BIP39 Phase 2 multi-language wordlist（目前仅英文）

### Known Issues
- 入站截流时 Claude Code SDK 收到不完整 SSE 序列（text 已发但 message_stop 未发），依赖 SDK 容错；真实 SDK 行为需 dogfood 验证
- IN-CR-04 markdown exfil 当前 warn，Week 4 评估升级 critical 触发条件
- IN-CR-01 阈值 `distance ∈[1,3] 且 len 相等`，Phase 1 仅覆盖 ETH 地址，BTC 地址 Week 4 加
- AFL++ nightly 仅 sse_parser_afl，其他两个 target 待 Week 6+ 补

---

### Added — Week 2 (2026-04-27 完成)

#### 出站规则引擎(sieve-rules)
- `VectorscanEngine`：vectorscan-rs 0.0.6 多模式正则，Block mode + SOM_LEFTMOST 报告精确字节偏移
- `MatchEngine` trait + `MatchHit` 数据结构（关联 ADR-001 / ADR-002）
- `bip39 module`：SHA-256 checksum 验证（PRD §9 #4 差异化点），内嵌 BIP39 官方英文 2048 词 wordlist（MIT）
- `placeholder module`：全局占位符黑名单（YOUR_API_KEY / xxx / 0x0...0 等）
- `loader::load_outbound_rules(toml_path)`：从 toml 加载规则集
- `RuleEntry` 扩展字段：`entropy_min` / `keywords` / `allowlist_regexes` / `allowlist_stopwords`（gitleaks 风格）
- criterion micro benchmark 骨架（`benches/scan_bench.rs`，Week 4 接入完整 secret benchmark）

#### 出站规则集(sieve-rules/rules/outbound.toml)
- OUT-01~12 全部 P0 规则上线（gitleaks MIT 风格 pattern）：
  - OUT-01 Anthropic API key / OUT-02 OpenAI API key（legacy + proj 新格式）
  - OUT-03 AWS Access Key / OUT-04 GitHub PAT / OUT-05 GCP API key
  - OUT-06 JWT / OUT-07 PEM Private Key / OUT-08 Stripe Live Key
  - OUT-09 Slack Token / OUT-10 OpenSSH Private Key / OUT-11 Discord Bot Token
  - OUT-12 BIP39 助记词（占位 pattern，运行时由 bip39 module 二次 checksum 验证）
- 12 条规则 positive + negative case 集成测试全部通过（`tests/outbound_rules.rs`）

#### Detection 数据模型(sieve-core)
- `Detection { id: Uuid, rule_id, severity, action, source, span, evidence_truncated, fingerprint }`
- `Severity { Low, Medium, High, Critical }`
- `Action { Block, Redact, WarnConfirm, MarkOnly, SilentLog }`
- `ContentSource { OutboundUserText, OutboundSystemText, InboundAssistantText, InboundToolUseInput }`
- `fingerprint(rule_id, content) -> 16 hex`（SHA-256 前 8 bytes，关联 docs/design/data-model.md §155-161）
- `PipelineNode::process()` 签名升级：返回 `Vec<Detection>`
- `OutboundFilter`（impl PipelineNode，从 sieveignore 过滤）+ `OutboundEngine` trait（由 sieve-cli engine_adapter 桥接）
- `AnthropicRequest::extract_text_content()`：从 messages + system 提取所有文本内容

#### Daemon 集成(sieve-cli)
- `Config` 加 `rules_path` / `sieveignore_path` / `dry_run` 字段
- `Command::Start` 加 `--dry-run` flag（覆盖 config.dry_run）
- `engine_adapter::OutboundAdapter`：把 sieve_rules::VectorscanEngine 适配到 sieve_core::OutboundEngine trait
- daemon::proxy_inner：POST /v1/messages 走 collect → 解析 AnthropicRequest → extract_text → OutboundFilter::process → Critical 命中且非 dry_run 返 426 JSON；其他路径继续流式透传（保 Week 1 字节级一致）
- 426 JSON schema：`{ type: "sieve_blocked", blocked_at, detections[], guidance: { zh, en } }`
- fail-closed 启动：规则加载失败 / vectorscan 编译失败 → 直接退出，不 fallback 无规则模式（ADR-007）

#### 测试与验证
- 单元测试：sieve-core 25 / sieve-rules 26 / sieve-cli 15 = **66 单元测试**
- 集成测试：`outbound_rules` 12 / `proxy_passthrough` 5 / `outbound_block` 3 = **20 集成测试**
- e2e smoke（`scripts/smoke_test.py`）26 项断言通过（原 21 + 新 4 项 426 拦截 + 1 项 benign 透传）
- `cargo deny check licenses bans sources` 全过
- release 二进制 9.0 MB（< 22MB 预算）

### Pending（Week 3 起）
- 完整 SSE Parser + fuzz corpus（PRD §9 #5）
- 入站规则 IN-CR-01~05（地址替换 / 危险工具调用 / 签名工具）
- ADR-008 出站 Critical 状态码 dogfood 验证（Week 2 期间真实 dogfood 后正式落 ADR）

### Known Issues（Week 2）
- 426 时间戳 Phase 1 用 UNIX epoch 秒（简化），Week 4 引入 chrono 改完整 RFC3339
- BIP39 OUT-12 vectorscan 预筛 pattern 占位符 `__BIP39_PREFILTER_PLACEHOLDER__`，运行时由 bip39 module 动态生成 alternation；集成测试中过滤掉（由 bip39 module 单测覆盖）
- 完整标准 secret benchmark 自建样本数据集留 Week 4（PRD §10.1 原计划）

---

### Verified — 2026-04-27 Week 1 完成定义实跑

#### release.yml workflow_dispatch 首跑(run 24980079580)
- **三 target reproducible build 双 SHA-256 一致 + cosign keyless OIDC 签名 + Rekor 上链 + cosign verify-blob 自验证**:
  - `aarch64-apple-darwin`: `af5c371f1a6531d2a8439425f9d90a5e339fca20a62825b8d895f29c6b883899`
  - `x86_64-apple-darwin`:  `47b729ee298f9dc1d5a3bd0a04f5f30b19983b7c87454b7358442514762164ea`
  - `x86_64-unknown-linux-gnu`: `bbe16fc2faf52a010dd3b3ae172599ec6b7ae9c8cd666c6046d06cfe265065fa`
- 已知遗留:`macos-universal` lipo 合并步骤路径修复(本 commit 含)。**universal binary 不影响 reproducible build pipeline 主路径验证**(三个独立 target 都已成功签名 + 上链)。
- ADR-006 §10 Week 1 hard gate **达成**。

#### 端到端 dogfood 验证(PRD §10.1 Week 1 第 1 完成定义)
- 在真机用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude` 启动 Claude Code v2.1.119(Opus 4.7),非流式聊天测试通过(2026-04-27 14:35 时点)。
- e2e smoke test 脚本(`scripts/smoke_test.py`)真机自验 21/21 通过:401 字节级透传 / 4xx 错误码 / 8KB body / 20 路并发 / 真 key 200 / SSE 流式 / tool_use partial_json。

### Added — Week 1 (2026-04-27 启动)

#### 工程骨架
- Cargo workspace + 三 crate（`sieve-core` / `sieve-rules` / `sieve-cli`），关联 .cursorrules §3.3
- `rust-toolchain.toml` 锁定 1.87.0，targets: aarch64-apple-darwin / x86_64-apple-darwin / x86_64-unknown-linux-musl
- `.cargo/config.toml` reproducible build flags（`--remap-path-prefix` + musl 静态链接），关联 ADR-006
- `deny.toml` cargo-deny 策略（出站 host 白名单 + 许可证白名单 + advisories yanked deny），关联 ADR-003

#### sieve-core（透传层）
- `UnifiedMessage` schema（Anthropic-only 实现，UpstreamProvider::Relay variant 预留），关联 PRD §6.1 / ADR-004
- `AnthropicRequest` 解析（serde 子集，#[serde(flatten)] 兼容未识别字段）
- `Forwarder`：hyper 1.x + hyper-rustls 0.27 + rustls 0.23 + aws-lc-rs provider + webpki-roots，ALPN h2+http/1.1
- SSE passthrough（Week 1 字节透传，Week 3 切到 parser）
- `PipelineNode` trait 占位（Week 2 起 OutboundFilter / InboundFilter 实现）
- 错误类型：`thiserror`，**禁止 anyhow**（.cursorrules §3.2）

#### sieve-rules（占位骨架）
- `MatchEngine` trait + `MatchHit` 数据结构占位（Week 2 起 vectorscan 实现）
- `RulesManifest` schema（rules-vN.manifest.json），关联 data-model.md
- `Ed25519 Verifier`（规则包验签占位，Week 5 起做实际下发）
- `vectorscan-rs 0.0.6` 依赖加入（用于三平台编译验证），关联 ADR-001
- `ed25519-dalek 2.x` 依赖加入

#### sieve-cli（daemon）
- `sieve start --config <path>`：hyper 1.x server 监听 127.0.0.1:11453，反向代理到 api.anthropic.com
- 配置加载：TOML，bind_addr **强制 127.0.0.1** 校验（非 loopback → exit(1)），关联 ADR-003 / PRD §9 #2
- 透传逻辑：headers 剥 Host 重写，body 通过 hyper `Incoming` 流式 chunk-by-chunk（SSE 字节级零缓冲）
- `serde(deny_unknown_fields)`：任何未知配置字段直接拒绝
- `audit` 模块占位（Week 4 接入 SQLite append-only）
- tracing-subscriber 日志（`SIEVE_LOG` 环境变量控制等级）
- **未引入 --disable-critical / --yolo flag**，关联 ADR-007

#### CI / CD（关键 - ADR-006 hard gate）
- `.github/workflows/ci.yml`：fmt / clippy / test（ubuntu + macos-14 矩阵）/ cargo-deny
- `.github/workflows/release.yml`：tag v* 触发，矩阵覆盖三 target，**双构建 SHA-256 比对**，**cosign keyless OIDC 签名**（`id-token: write`），Rekor 透明日志上链，sigstore bundle 上传到 GitHub Release
- macOS universal binary（lipo 合 aarch64 + x86_64）

### Changed — 2026-04-27 PRD §9 #10 修订

- **撤销 "Day 1 GitHub repo 公开 README + 架构文档" 承诺**
- 新策略：首个稳定版发布时一次性公开 repo + 代码 + 文档 + sigstore 验证流程
- 影响范围：repo 在公开发布前保持 private；release.yml 不绑定 tag（改为 workflow_dispatch）
- ADR-006 sigstore + reproducible build CI **不受影响**，公开发布前照常跑通；只是不做 public Rekor 验证演示

### Pending（Week 3 起）
- SSE Parser 完整实现 + fuzz corpus（PRD §9 #5）
- 入站 Crypto 钩子（IN-CR-01~05）

### Known Issues
- 本地需安装 Rust toolchain（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- vectorscan-rs 编译需要系统包 `boost` + `ragel`（macOS：`brew install boost ragel`；Linux：`apt-get install libboost-dev ragel`）
- ADR-008 出站 Critical 状态码（候选，dogfood 实测后落 ADR）

---

> 以下为 PRD v1.3 设计阶段计划，**尚未实现**。任何条目在实际编码、测试、签名验证完成前不视为已交付。

### 计划中（Week 1-8）

#### 新增

- **W1 基础设施 + Anthropic 协议**
  - Rust 项目骨架（`sieve-core` / `sieve-rules` / `sieve-cli` workspace）
  - `hyper` + `tokio` + `rustls` HTTP 反向代理跑通
  - 透明转发 Anthropic Messages API（`POST /v1/messages` 含 SSE，`POST /v1/messages/count_tokens`，`GET /v1/models`）
  - `UnifiedMessage` 内部 schema（仅 Anthropic 实现，其他 provider 接口预留，[PRD §9 #9](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - ~~GitHub repo 公开~~ — 已撤销，repo 在公开发布前保持私有
  - **🚨 sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 必须 W1 跑通** —— [PRD §1.2 第 4 句](../prd/_archive/sieve-prd-v1.3.md#12-四句话核心叙事v13-加第-4-句) 自证清白叙事的物质基础
- **W2 出站 P0 规则（OUT-01 ~ OUT-12）**
  - OUT-01 OpenAI / Anthropic API key（前缀 + entropy + 占位符黑名单，FP < 0.1%）
  - OUT-02 AWS Access Key（`AKIA[0-9A-Z]{16}` + 排除官方示例，FP < 0.1%）
  - OUT-03 GitHub Token（前缀 + CRC32 校验，FP < 0.05%）
  - OUT-04 JWT（三段 base64 + header 解码验证，FP < 0.5%）
  - OUT-05 RSA / Ed25519 / SSH 私钥（PEM 头精确匹配，FP < 0.01%）
  - OUT-06 Ethereum 私钥（regex + entropy + 上下文，FP < 1%，**只能 High，不上 Critical**）
  - OUT-07 Bitcoin WIF（base58 + 双 SHA-256 校验位，FP < 0.001%）
  - OUT-08 Solana 私钥（base58 88 字符或 hex 64 字节，FP < 1%）
  - **OUT-09 BIP39 助记词 + SHA-256 checksum 验证**（差异化点，FP < 0.05%；[PRD §9 #4](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - OUT-10 Keystore JSON（Web3 Secret Storage v3 schema，FP < 0.01%）
  - OUT-11 .env 文件特征（多行 KEY=VALUE 密度阈值，FP < 5%，仅 Medium）
  - OUT-12 数据库连接串（URI scheme + 用户名密码字段，FP < 0.5%）
  - 占位符黑名单 + `.sieveignore` 学习型白名单
  - 单元测试覆盖 ≥ 80%
- **W3 入站 Crypto 钩子**
  - SSE Parser + `tool_use` Aggregator
  - **IN-CR-01 地址替换检测**（对话历史 `0x[a-fA-F0-9]{40}` 比对：相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄）
  - **IN-CR-05 签名工具 fail-closed**（`eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 全部强制弹窗，YOLO mode 不可关闭，[PRD §9 #3](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - **大量 SSE fuzz test** 覆盖 PRD §9 #5 列出的 6 类边界
- **W4 入站通用 + 危险 tool call + benchmark 数据集**
  - IN-CR-02 危险工具调用（`bash` 含 `rm -rf` / `curl..|sh` / `eval(base64..)` / `sudo`）
  - IN-CR-03 敏感路径访问（`~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/`）
  - IN-CR-04 持久化机制（`crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc`）
  - IN-GEN-01 危险 shell 模式（`rm -rf /`、fork bomb、`> /dev/sda`、`dd if=/dev/zero`）
  - IN-GEN-02 远程脚本执行（`curl X | sh`、`wget X | bash`、`bash <(curl X)`）
  - IN-GEN-03 编码后执行（`eval(base64.b64decode(...))`、`exec(__import__('os')...)`）
  - IN-GEN-04 Markdown 图片 exfil（`![](http://X.com/?Y=Z)` + 域名不在白名单）
  - IN-GEN-05 Prompt injection 反向（`<|im_start|>`、`[INST]`、`### System:`、`Ignore previous`）
  - 处置矩阵完整实现（Critical / High / Medium / Low → HTTP 行为映射，参见 [API 参考 §5](../api/api-reference.md#5-处置矩阵--http-行为)）
  - CLI 弹窗 + 命令行确认（fail-closed，超时按拒绝，参见 [API 参考 §6](../api/api-reference.md#6-cli-退出码--弹窗确认协议)）
  - **Benchmark 数据集**（[PRD v1.3 §10.1 W4 修订](../prd/_archive/sieve-prd-v1.3.md#101-phase-adogfood-阶段week-1-8)）：
    - 200-500 条合成攻击样本（UCSB 4 类攻击 + drainer 模式 + Pink Drainer 数字化绕过 + npm typosquat + `curl|sh` + eval base64）
    - 50-100 条真实 benign 会话回放（日常 Claude Code 工作录制）
    - canary 测试（假 BIP39、假地址、假 selector、假 .env，使用 honeypot 钱包私钥）
    - 目标：Critical FP < 0.5%，High FP < 5%
- **W5 配置系统 + brew tap**
  - 完整 `config.toml` schema（参见 [API 参考 §3](../api/api-reference.md#3-配置文件-schema-sieveconfigtoml)）
  - 本地 SQLite append-only 审计日志（仅 fingerprint + 元信息，**不存原文**）
  - License 离线验证（**本地 Ed25519 验证，不联网 verify**，[PRD §9 #2](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - brew tap + GitHub Releases 发布流水线
  - 本地管理 API（参见 [API 参考 §2](../api/api-reference.md#2-sieve-本地管理-api)）
- **W6 自用 dogfood + 修 bug**
  - 100% 时间用 Sieve 工作
  - 性能 benchmark 验证 P99 < 20ms（[PRD §6.4](../prd/_archive/sieve-prd-v1.3.md#64-性能预算)）
  - macOS / Linux / Windows 二进制（macOS arm64 + Linux x86_64 为 Tier 1）
  - 收集 false positive，加 `.sieveignore` 默认条目
- **W7-W8 高强度 dogfood**
  - 第一次签名规则库下发测试（Ed25519 验证 fail-closed）
  - 完成定义：用 Sieve 跑 2 周，无 P0 / P1 bug

### 计划中（Phase B 闭测, Week 9-12）

#### 新增

- **W9 闭测启动**
  - 小规模闭测白名单
  - Discord 闭测频道
  - 每天处理反馈
- **W10 闭测 + 修 bug**
  - 修闭测 bug
- **W11 闭测扩大**
  - 闭测扩大
- **W12 公开发布**
  - **代码开源**（[PRD §11.3](../prd/_archive/sieve-prd-v1.3.md#113-开源策略)）
  - 二进制 cosign 签名验证 + reproducible build 验证步骤公开（参见 [部署指南 §3](../guides/deployment.md#3-二进制签名验证必做)）
  - **冻结 v1 公开 API**（参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)）

### 暂不做（明确推迟到 Phase 2）

- 中文 PII（身份证 / 银行卡 / 统一信用代码）
- 内网域名 / 内部代号（用户自定义规则）
- 长代码块识别 + Copyright 提示
- 自定义规则 DSL
- npm / pip typosquat 检测
- Markdown 链接钓鱼
- Unicode 攻击防御（NFC + 控制字符黑名单）
- Calldata 静态解码（4byte 离线 SQLite）
- ERC20 危险 approve（`approve(MAX)` / `setApprovalForAll`）
- EIP-2612 / EIP-7702 滥用
- Drainer 黑名单（Chainabuse + ScamSniffer 集成）
- 协议白名单
- Solidity 后门检测（Slither）
- **MCP 拦截 IN-MCP-01~03**（PRD §5.2 修订，Phase 2 Week 16-20）
- 桌面 App / VS Code 插件
- OpenAI / Gemini / OpenRouter 协议适配（**真实用户主动要求时才做**，[PRD §9 #9](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）

---

## PRD 文档版本演进

> Sieve 项目尚未发布二进制版本，但产品需求文档已迭代 4 版。每版日期 + 一句话差异：

### [PRD v1.0](../prd/_archive/sieve-prd-v1.0.md) - 2026-04-26

- 工程启动前 PRD，覆盖完整产品范围
- 一句话：双向检测的本地 LLM 流量代理，服务 crypto 开发者，反对中转站不可信
- 状态：**已废弃**，被 v1.1 收敛

### [PRD v1.1](../prd/_archive/sieve-prd-v1.1.md) - 2026-04-26

- 范围收敛版：从 v1.0 砍掉一半范围
- 关键改动：
  - MVP 范围砍 50%，只做出站 secret + 危险 tool call + 地址替换 + 签名拦截
  - 三 agent 适配（Claude Code / OpenClaw / Hermes）用统一本地代理
  - sigstore + reproducible build 提到 Phase 1 必交付物
  - 桌面 App、VS Code 插件、Slither、中文 PII 全部推到 Phase 2
  - 节奏：6-8 周冲 MVP + 慢节奏维护
- 状态：**已废弃**，被 v1.2 第一性原理重写覆盖

### [PRD v1.2](../prd/_archive/sieve-prd-v1.2.md) - 2026-04-26

- 第一性原理修订版：用 12 条公理重新推导每个决策
- 关键改动：
  - 降级模式只读警告（公理 11 / 12）
  - 公理 7：Phase 1 **只做 Claude Code**，OpenClaw / Hermes 推迟到第二个用户主动要时
  - 12 周冲 GA（8 周 dogfood + 4 周闭测）
  - 处置矩阵：Critical 阻断 + High 警告 + Medium 标记 + Low 静默
  - "Sieve 的本质不是 LLM 安全产品，是在不可逆动作前插入认知摩擦的保险工具"
- 状态：**被 v1.3 取代**

### [PRD v1.3](../prd/_archive/sieve-prd-v1.3.md) - 2026-04-26（**当前活动版本**）

第一性原理修订版，**锁定执行**。在 v1.2 基础上吸收 review 的工程改动：


| #   | 改动                                                                               | 章节         |
| --- | -------------------------------------------------------------------------------- | ---------- |
| 1   | **"自证清白"从工程细节提到产品定位** —— sigstore + 透明日志作为可验证信任的核心能力                            | §1.2 第 4 句 |
| 2   | **Benchmark 数据集大小具体化** —— 200-500 攻击样本 + 50-100 benign 会话                        | §10.1 W4   |
| 3   | **MCP 拦截放进 Phase 2** —— Claude Code 真实威胁面（IN-MCP-01~03）                          | §5.2       |
| 4   | **用户教育成本作为风险登记**                                                                 | §12        |


附加改动：

- §10.1 W1 sigstore + reproducible build pipeline 必须本周跑通
- §11.3 透明更新日志加入开源策略

---

## 文档结构变更

### [unreleased] - 2026-04-27

#### 新增

- 文档结构初始化：
  - `docs/api/api-reference.md` —— API 参考首版（反向代理 / 本地管理 API / 配置 schema / 环境变量 / 处置矩阵 / 错误码）
  - `docs/guides/development.md` —— 开发指南首版（构建、测试、SSE fuzz、benchmark、规则编写、PR 流程）
  - `docs/guides/deployment.md` —— 部署与运维指南首版（安装、签名验证、服务运行、升级回滚、FAQ）
  - `docs/changelog/CHANGELOG.md` —— 本文件
- 所有文档反映 [PRD v1.3](../prd/_archive/sieve-prd-v1.3.md) 设计意图，**未实现任何代码**

#### 文档审查与一致性修复（2026-04-27）

全量审查 docs/ 文档对 PRD v1.3 的一致性，输出关键冲突清单并修复。

**修复（关键冲突）**：

- 分发与运营相关文档 —— 修正未授权的条目
- [ADR-006](../design/ADR-006-sigstore-reproducible-build.md) —— 显式标注 Tier 1（macOS / Linux）/ Tier 2（Windows）平台分级；承认 Windows reproducible build 推到 Phase 2 是与 PRD §9 #6 全平台理想的暂时偏离，需 PRD 下次修订同步
- [deployment.md §2.3](../guides/deployment.md) —— Windows 部署描述加 Tier 2 标识（Week 6+ 才出二进制 + 签名，reproducible 不承诺）
- [deployment.md §11.1](../guides/deployment.md) —— 补 license 离线过期 → 降级模式自动转换流程
- [api-reference.md §1.6](../api/api-reference.md) —— 补完整入站 Critical SSE 序列（`sieve_block` → buffer 暂停 → `sieve_resume` / `sieve_terminate`）+ buffer 上限 + `event_id` 关联
- [api-reference.md §2.2.3](../api/api-reference.md) —— 补 `user_decision` 字段值域定义（`null` / approve / deny / timeout / interrupted）
- [api-reference.md §2.2.4](../api/api-reference.md) —— 明确 fingerprint = `<rule_id>:<sha256_prefix_8_hex>` 长度规范
- [api-reference.md / deployment.md] 多处过时的"待写"链接修正为已存在文档的真实链接
- 删除 CHANGELOG.md 末尾空白占位符（"模板段落" / "链接"）

**新增文档**：

- [docs/glossary.md](../glossary.md) —— 项目术语表（54 条术语，覆盖产品 / 架构 / 检测 / 安全 / 协议 / 运营 / 合规 7 个主题）
- [docs/design/ADR-INDEX.md](../design/ADR-INDEX.md) —— ADR 索引 + 编号规则 + 候选 ADR 列表（ADR-008 候选 Critical 状态码 / ADR-009 候选 Windows 服务）
- tasks/roadmap.md —— 12 周里程碑可勾选执行清单 + 跨周依赖图
- tasks/lessons.md —— 经验教训记录骨架

**倾向决策（2026-04-27）**：

- **ADR-008（候选）出站 Critical 状态码维持 `426 Upgrade Required`**——api-reference.md §7.2 现有方案。Week 2 dogfood 阶段实测 Claude Code SDK 行为后正式落 ADR；如 SDK 表现异常（自动重试 / 错误信息丢失等）再切换备选方案。已在 tasks/roadmap.md Week 2 任务清单加入验证项。

#### Git 仓库脚手架（2026-04-27）

为 GitHub repo 基础设施准备完整的 git 治理文件：

- **新增** `.gitignore` —— Rust + macOS / Linux / Windows + Sieve 特定（`.sieveignore` / `audit.db` / `*.sigstore` / 临时文档）。**Cargo.lock 不入忽略名单**（reproducible build 要求入库，[ADR-006](../design/ADR-006-sigstore-reproducible-build.md)）
- **新增** `.gitattributes` —— 强制 LF 行尾（reproducible build 跨平台一致性）+ GitHub linguist 语言识别（docs / prd / research 标记 vendored / documentation）+ 二进制文件标记
- **新增** `SECURITY.md` —— 安全漏洞报告流程（安全披露邮箱 GA 前确定）+ 24h/7d/30d 响应 SLA + 自身供应链承诺 + 不在范围清单
- **新增** `LICENSE` —— 双轨许可说明：文档 **CC BY-NC-SA 4.0** / 代码 **MIT**（均在公开发布时同步公开）
- **新增** `.github/ISSUE_TEMPLATE/` —— bug_report / feature_request / **suspicious_sample**（[PRD §8.1](../prd/_archive/sieve-prd-v1.3.md#81-简化版) 用户公开提交可疑样本走这里）+ config.yml（指引安全漏洞走 SECURITY.md，紧急资产损失走 email）
- **新增** `.github/PULL_REQUEST_TEMPLATE.md` —— 对齐 [.cursorrules §五](../../.cursorrules) 自检清单 + PRD §9 硬约束验证 + 检测项变更模板 + Breaking Changes 流程
- **新增** `.github/dependabot.yml` —— Cargo 周更（仅 patch / minor，major 走人工评估，对齐 [PRD §9 #6](../prd/_archive/sieve-prd-v1.3.md#9-工程上必须做对的硬约束) pinned dependencies）+ GitHub Actions 周更 + 关键依赖分组（tokio-stack / simd-stack / crypto-stack）

仓库尚未 `git init`。完成审阅后可执行：
```bash
cd /path/to/sieve
git init
git add -A
git commit -m "chore: initial commit, Pre-Code design phase docs"
# 创建 GitHub repo 后：
git remote add origin <github-url>
git push -u origin main
```

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/_archive/sieve-prd-v1.3.md](../prd/_archive/sieve-prd-v1.3.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 术语表：[../glossary.md](../glossary.md)
- ADR 索引：[../design/ADR-INDEX.md](../design/ADR-INDEX.md)
- Roadmap：../../tasks/roadmap.md

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。
> 任何依赖升级、行为变更、检测项 ID 增删必须在本文记录（`[.cursorrules` §1.5](../../.cursorrules)）。