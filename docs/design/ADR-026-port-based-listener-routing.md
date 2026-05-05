# ADR-026: Port-based listener routing —— 多上游 listener + path prefix 修复

## 状态

**Proposed**

> 决策日期：2026-05-05
> 范围：Phase 2.x daemon（v2.x 工程项，不阻塞 GA；GA 前完成）
> 关联 PRD：[v2.0 §6.1 部署拓扑](../prd/sieve-prd-v2.0.md)、[v2.0 §6.5 IPC 协议](../prd/sieve-prd-v2.0.md)
> 关联 ADR：ADR-013（IPC 协议）、ADR-018（OpenAI 协议适配）、ADR-019（X-Sieve-Origin header）

---

## 背景

### 痛点 1：Forwarder path prefix bug

`crates/sieve-core/src/forwarder/mod.rs:60-117` 的 `Forwarder::new` + `rewrite_uri` 只取 `upstream_url` 的 scheme + authority，**完全丢弃 path 部分**。测试 `rewrite_uri_keeps_path_and_query`（line 152）固化了这个错误假设。

后果：DeepSeek 的 Anthropic 兼容入口 `https://api.deepseek.com/anthropic` 接不通——sieve 把 `/v1/messages` 转发到 `https://api.deepseek.com/v1/messages`（404），正确目标是 `https://api.deepseek.com/anthropic/v1/messages`。所有带 path prefix 的中转站均受此影响。

### 痛点 2：哑 client 无法同时打多个上游

现有 multi-provider 路由方案是 header routing（`X-Sieve-Provider` header + `~/.sieve/upstream-routes.json`，见 SPEC-004 §4.2 + ADR-019）。但 Claude Code / Codex CLI / Cursor 这类 client 只认 `ANTHROPIC_BASE_URL` 一个 env var，**不会主动注入 header**。

结果：header routing 只服务 OpenClaw / Hermes 这种「自己就是多 provider router」的 agent，对哑 client 完全无效——哑 client 只能打一家上游，无法动态切换。

### 痛点 3：协议识别靠 path 猜，跟路由维度错位

sieve 的 Protocol Layer 当前按请求 path 分发：`/v1/messages` → Anthropic，`/v1/chat/completions` → OpenAI。当 client 发错 path（比如 Claude Code 强发 Anthropic 协议但被路由到 OpenAI 上游），sieve 不会显式拒绝，可能让 body 解析错位。协议识别维度（path）与路由维度（upstream）不在同一个配置层，没有统一约束点。

---

## 决策

### 1. Config schema 升级

把 `Config.upstream_url: String` + `Config.port: u16` 升级为 `Config.upstreams: Vec<UpstreamListener>`：

```toml
# sieve.toml v2.x 新 schema

# 默认 listener（兼容现有部署，仍然在 11453 暴露 Anthropic 上游）
[[upstream]]
port        = 11453
url         = "https://api.anthropic.com"
provider_id = "anthropic"
protocol    = "anthropic"

# 多上游 listener
[[upstream]]
port        = 11454
url         = "https://api.deepseek.com/anthropic"
provider_id = "deepseek"
protocol    = "anthropic"

[[upstream]]
port        = 11455
url         = "https://api.openai.com"
provider_id = "openai"
protocol    = "openai"
```

每个 `UpstreamListener` 字段含义：

| 字段 | 类型 | 说明 |
|---|---|---|
| `port` | `u16` | 监听端口；必须 127.0.0.1 绑定，bind_addr 不可改（PRD §9 #2 完全本地） |
| `url` | `String` | 真实上游，含 path prefix |
| `provider_id` | `String` | 用于审计 / IPC 事件标注 / 日志 |
| `protocol` | `Protocol` | 显式声明 `anthropic` \| `openai`，不再靠 path 猜 |

**向后兼容**：旧 `upstream_url` + `port` 字段仍可读，deserialize 时自动映射成单元素 `upstreams` vec。`deny_unknown_fields` 在 migration 完成前放宽（或把 migration 逻辑内联进 `Deserialize` impl）。

### 2. Path prefix 修复

`Forwarder::new` 多记一个 `upstream_path_prefix: String`（trim 末尾 `/`）：

```rust
// crates/sieve-core/src/forwarder/mod.rs

pub fn new(upstream_url: &str) -> SieveCoreResult<Self> {
    let uri: http::Uri = upstream_url.parse()?;
    let scheme    = uri.scheme_str().unwrap_or("https").to_string();
    let authority = uri.authority().ok_or(...)?.to_string();
    let path_prefix = uri.path().trim_end_matches('/').to_string(); // "" or "/anthropic"
    // ...
}

pub fn rewrite_uri(&self, original: &http::Uri) -> SieveCoreResult<http::Uri> {
    let path_and_query = original.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let new_uri = format!(
        "{}://{}{}{}",
        self.upstream_scheme, self.upstream_authority,
        self.upstream_path_prefix, path_and_query
    );
    http::Uri::try_from(new_uri).map_err(...)
}
```

同步新增测试 case：

- `rewrite_uri_with_path_prefix`：upstream `https://api.deepseek.com/anthropic` + request `/v1/messages` → `https://api.deepseek.com/anthropic/v1/messages`
- `rewrite_uri_path_prefix_with_query`：含 query string 的版本
- `rewrite_uri_path_prefix_trailing_slash`：upstream `https://api.deepseek.com/anthropic/` 的 trim 行为

原有测试 `rewrite_uri_keeps_path_and_query`（line 152）中固化错误假设的断言须同步修正。

### 3. Multi-listener accept loop

`crates/sieve-cli/src/daemon.rs:734` 的单 `TcpListener::bind` 拆成 multi listener：

```rust
let listeners: Vec<(TcpListener, Arc<Forwarder>, ListenerMeta)> = config
    .upstreams
    .iter()
    .map(|u| build_listener(u))
    .collect::<Result<_>>()?; // fail-fast：所有 bind 成功才继续

let mut handles = Vec::new();
for (listener, fwd, meta) in listeners {
    let filter         = filter.clone();
    let ipc            = ipc_server.clone();
    let audit          = audit_store.clone();
    let inbound_engine = inbound_engine.clone();
    handles.push(tokio::spawn(accept_loop(
        listener, fwd, meta, filter, ipc, audit, inbound_engine,
    )));
}
```

**共享**：filter pipeline / IPC server / audit store / inbound_engine 等单例（`Arc` 克隆）。
**不共享**：listener 本身、Forwarder 实例（每上游一份独立连接池）。

**Fail-fast 策略**：N 个 listener 任一 `bind` 失败时直接返回错误退出，不进入 partial-start 状态——避免 doctor 报告与实际监听状态不一致。

### 4. 显式 protocol 声明

每个 listener 带 `protocol: Protocol` 字段，daemon 收到请求后：

- 优先使用 listener 自身 protocol 决策路由，不看 path
- 请求 path 与 protocol 不一致时 → 显式返回 400 Bad Request + 注入 `sieve_blocked` event，不再静默用 path 猜

### 5. 审计 schema 升级

`audit.db` 加字段 `provider_id TEXT NOT NULL DEFAULT 'anthropic'`（SQLite migration），每条审计记录标注命中的 listener。旧行默认填 `'anthropic'`，migration 脚本幂等可重跑。

### 6. 与 header routing 的分工边界

**Header routing 不被 deprecate**，与 port routing 并存：

| 场景 | 用什么 |
|---|---|
| 哑 client（Claude Code / Codex CLI / Cursor 等只认 single base_url） | **Port routing**（一等公民） |
| Smart router agent（OpenClaw / Hermes 这种自己当 LLM 网关的） | **Header routing**（保留 ADR-019 X-Sieve-Origin + SPEC-004 §4.2 现状） |
| 同一 client 同一会话切多家 | sieve 不解决，让 LiteLLM / OpenRouter 当 upstream |

OpenClaw 一个进程要打多家，没法绑多端口；header routing 仍然是它的最佳解，本 ADR 不改变它。

### 7. doctor 升级

`sieve doctor` 加 listener 维度体检：

- 每个 listener 端口可达性（`TcpStream::connect` self-check）
- 每个 upstream 连通性 + TLS 证书校验
- 端口冲突检测（bind 前 + startup 后双重检查）

---

## 影响

### 正面影响

1. **哑 client 多上游可行**：Claude Code 改一个 env var（换端口）就能切上游，无须依赖 client 注入 header；
2. **path prefix 中转站可用**：DeepSeek Anthropic 兼容入口、各类带前缀中转站全部通畅，修复 v1.x 遗留 bug；
3. **审计监控天然按 listener 切片**：`lsof -i :11454` / pcap / netstat / process-context 反查都按端口维度切，可观测性提升；
4. **协议识别更精确**：listener 显式声明 protocol，错位请求被拒而不是静默错路（ADR-025 的 4 类 × N listener 矩阵硬约束得以落地）；
5. **与 unix-style 哲学一致**：每个 listener 是一个独立安全策略端点，跟 systemd socket activation / inetd 同款；为 ADR-027（network jail）按 endpoint host 切片做铺垫。

### 负面影响

1. **Config breaking change**：`upstream_url` + `port` 改 schema，需写 migration / 兼容反序列化；`setup.rs` / `doctor.rs` / `docs/guides/development.md` 全部要同步；
2. **多占端口**：每个上游一个端口，端口规划成本增加（建议 11453 起递增，写进 deployment.md）；
3. **bind 失败处理变复杂**：采用 fail-fast 策略（决策见决策 §3），所有 listener 都 bind 成功才进入 accept loop——部分环境下已占用端口会直接阻止 daemon 启动；
4. **测试矩阵扩大**：multi-listener e2e + listener-protocol 错位拒绝 + 旧 single-upstream 兼容必须各覆盖一份（ADR-025 §4 矩阵从 4 类扩展到 4 类 × N listener）。

### 需要更新的文档

- `docs/design/architecture.md` —— §1.1 部署拓扑图（多 listener）+ §10.2 协议层路由说明
- `docs/design/data-model.md` —— Config schema + audit.db schema 升级
- `docs/api/api-reference.md` —— §3 配置部分
- `docs/guides/development.md` —— 示例配置 + sieve doctor 输出示例
- `docs/guides/deployment.md` —— 多 listener 部署 + launchd plist 端口列表
- `docs/specs/SPEC-003-sieve-setup-tool.md` —— setup 流程同步（如何从单 upstream 升级到多 listener config）
- `docs/specs/SPEC-004-multi-agent-setup.md` —— §4.2 明确 header routing 与 port routing 的分工
- `CHANGELOG.md` —— `[BREAKING]` 标 Config schema 升级
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目

---

## 相关文档

- [PRD v2.0 §6.1 部署拓扑](../prd/sieve-prd-v2.0.md)
- [PRD v2.0 §6.5 IPC 协议](../prd/sieve-prd-v2.0.md)
- [PRD v2.0 §9 #2 完全本地](../prd/sieve-prd-v2.0.md) —— bind_addr 不可改不变量
- [PRD v2.0 §9 #16 content-type 路由矩阵](../prd/sieve-prd-v2.0.md) —— 4 类组合 × N listener 测试硬约束
- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议（本 ADR 的 listener routing 不影响 IPC，UDS 仍单一 `~/.sieve/ipc.sock`）
- [ADR-018](./ADR-018-openai-protocol-adapter.md) —— OpenAI 协议适配（本 ADR 的 `protocol` 字段是 ADR-018 引入的双协议在 listener 层的显式化）
- [ADR-019](./ADR-019-x-sieve-origin-header.md) —— X-Sieve-Origin header（与 header routing 并存，分工见决策 §6）
- [ADR-025](./ADR-025-content-type-routing-matrix.md) —— content-type 路由矩阵（4 类组合测试硬约束 → 升级为 4 类 × N listener 矩阵）
- ADR-027（待写）—— network jail enforcement，按 LLM endpoint host 切片，与本 ADR port 切片对齐
- ADR-028（待写）—— IPC 协议中性化 + sieve-ipc 模块化，独立工作项
- [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md) —— setup 工具（单 upstream → 多 listener 升级流程）
- [SPEC-004](../specs/SPEC-004-multi-agent-setup.md) —— multi-agent setup §4.2（header routing 与 port routing 分工）
