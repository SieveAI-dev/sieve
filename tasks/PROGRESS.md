# Sieve daemon · 进度

> 上次更新：2026-05-05
> 当前阶段：**SPEC-005 v2 代码就绪等联调 + unix-style 改造立项（ADR-026/027/028）**

## 当前阶段一句话

SPEC-005 v2.0 frozen + v2.0+ 兼容扩展（list_rules / purge_history）双侧契约全部实现并对齐；
P0 5/5 ✅ · P1 10/10 ✅ · P2 5/6 ✅（P2 仅余 fixtures polish）+ 善后全部 ✅；
e2e 集成测试 harness 6 场景，cargo workspace 696 passed 全绿。
GUI 仓库（sieve-gui-macos）同步完成 swift test 127 passed + xcodebuild SUCCEEDED。
代码侧任务清零，等用户手动联调反馈。

2026-05-05 起：基于「sieve 应做成 iptables-like unix-style 工具」的设计反思，立项三项 ADR 与 6 个工程项（path prefix bug / port-based listener routing / IPC 协议中性化 / headless decision CLI / unix-pipeable audit / network jail enforcement），按 P0/P1/P2 排进 v2.x 与 v3.x。

---

## ✅ 主里程碑

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
（无 — 代码侧任务清零）

---

## ⏭ 下一步（等用户联调反馈）

### 用户介入项
- 真实 dogfood：启 daemon + GUI 跑 HIPS / Settings / History 流程
- 反馈 bug 或 UX 调整

### unix-style 改造（v2.x，与联调并行）

> 关联 ADR：[ADR-026](../docs/design/ADR-026-port-based-listener-routing.md) / [ADR-028](../docs/design/ADR-028-ipc-protocol-neutralization.md)；v3.x 关联 [ADR-027](../docs/design/ADR-027-network-jail-enforcement.md)
> 设计源：2026-05-05 主线讨论 ——「sieve 想做 iptables-like 工具，UI 是众多 client 之一不是特权」

#### P0 · 基础设施（先做，其他依赖）

- [ ] **TODO-1 修 forwarder path prefix bug**（半天）
  - 痛点：`upstream_url` 里的 path 被丢弃，DeepSeek 这种 `https://api.deepseek.com/anthropic` 转不通
  - 改动：`crates/sieve-core/src/forwarder/mod.rs`，`Forwarder::new` 多记 `upstream_path_prefix`，`rewrite_uri` 拼接 prefix + original path
  - 测试：补 `upstream_url` 含 path / 含 path + query / 含 trailing slash 三个 case
  - 关联：ADR-026

- [ ] **TODO-2 Port-based multi-listener**（1-2 天）
  - 痛点：哑 client（Claude Code）不会注入 header，`X-Sieve-Provider` 路由对它无效；同一 client 同进程没法切上游
  - 改动：`Config.upstream_url` → `Config.upstreams: Vec<UpstreamListener>`（含 port / url / provider_id / protocol）；旧字段映射成单元素 vec 兼容；`daemon.rs:734` 单 listener 拆 multi accept loop
  - 显式 protocol 字段替代靠 path 猜
  - 审计 schema 加 `provider_id` 列
  - doctor 加 listener 维度体检
  - 关联：ADR-026

#### P1 · 协议中性化（GUI 不再特权）

- [ ] **TODO-3a SPEC-005 协议术语中性化**（1 天）
  - 改 method 名：`decision.popup` → `decision.pending`、`decision.popup_canceled` → `decision.canceled`（保留旧名作 deprecated alias 一个 minor 版本）
  - 段落重写：「GUI 端」→「client 端」全文替换；GUI-only UI 状态机搬到 sieve-gui-macos 仓 SPEC-002
  - SPEC-005 §0 文档定位更新：daemon 协议只描述语义，不感知 client 形态
  - 关联：ADR-028

- [ ] **TODO-3b sieve-ipc 内部模块化**（半天）
  - 范围：crate 内部 `protocol/` 子模块化，**不拆 crate**
  - 目录：`protocol/` (envelope / decision / handshake / rules / audit) + `server/` + `client/` + `file_ipc/`
  - 硬约束：`protocol/` 只能 import serde / chrono / 标量类型，禁止 IO 依赖
  - 关联：ADR-028

- [ ] **TODO-4 Headless decision CLI**（2 天）
  - 新增 `sieve decisions watch / show / resolve` 子命令
  - 新增 `sieve start --no-client-policy=auto-block|auto-warn|hold-and-fail-closed` flag
  - CLI 跟 GUI 共用同一组 IPC method，不引入特权 endpoint
  - 依赖：TODO-3a / TODO-3b
  - 关联：ADR-028

- [ ] **TODO-5 Audit 层 unix-pipeable**（1-2 天）
  - 新增 `sieve audit tail -f --format jsonl` / `sieve audit query --since 1h --severity critical --jsonl`
  - 新增 `sieve start --emit-events stdout`（daemon 直接吐事件流）
  - 关键：SQLite 留作权威存储，jsonl 是 projection；不增加新写路径
  - 独立可做，无依赖

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

#### 工作量与节奏
- v2.x（GA 前）：TODO-1~5 共约 6-8 个工作日，与联调并行不冲突
- v3.x（GA 后，dogfood 验证后）：TODO-6 约 3-5 个工作日

#### 产出物清单
- 3 份 ADR：ADR-026 / ADR-027 / ADR-028（已立项）
- SPEC-005 修订（TODO-3a 落地时）
- sieve-ipc crate 内部目录调整（TODO-3b）
- CHANGELOG `[BREAKING]` Config schema 升级（TODO-2）

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
