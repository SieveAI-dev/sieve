# Sieve daemon · 进度

> 上次更新：2026-05-05
> 当前阶段：**unix-style v2.x 落地 + ADR-030 客户端代码 + SPEC-006 落地 + docs 同步；运维侧 TODO-13~16 等海外主体落地后启动**

## 当前阶段一句话

unix-style 改造全部落地（TODO-1~5） + ADR-030 sieve-updater crate 客户端实现 + SPEC-006 manifest 协议起草 + 6 处 docs 同步（TODO-7~12 + TODO-17/18）全部完成；等用户验证代码侧 + 运维侧 TODO-13~16（域名 / KMS / 服务端）待海外主体落地后启动。

2026-05-05 单日完成 unix-style 改造 v2.x 全部 5 项（TODO-1~5）并落地 12 个 commits：
ADR-026 multi-listener（含 forwarder path prefix / Config schema / multi-listener accept loop /
协议错位 fail-closed / 审计 provider_id / IPC HealthResult.listeners / doctor 升级）+
ADR-028 IPC 协议中性化 / sieve-ipc 模块化 / sieve decisions CLI / sieve audit CLI。
TODO-6 Network jail enforcement 推后到 v3.x post-GA opt-in。

ADR-030 sieve-updater crate + SPEC-006 + docs 同步 2026-05-05 同日完成（TODO-7~12 + TODO-17/18）。

---

## ✅ 主里程碑

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
