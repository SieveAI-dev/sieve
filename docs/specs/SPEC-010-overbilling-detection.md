# SPEC-010: 超额计费检测（独立 token 核算 + relay 虚报戳穿）

> Version: v0.1 — 2026-06-19
> 状态：Draft（ADR-038 落地详规，Phase 2 未实现）
> 关联 ADR：[ADR-038](../design/ADR-038-overbilling-detection.md)（本 SPEC 唯一权威决策源）/ [ADR-003 amended](../design/ADR-003-local-only-no-cloud-verifier.md)（绝不联网做 verifier，§4/§6 核心张力）/ [ADR-018](../design/ADR-018-openai-protocol-adaptation.md)（双协议）/ [ADR-026](../design/ADR-026-port-based-listener-routing.md)（per-port 路由 + provider_id，信任分级前提）/ [ADR-033](../design/ADR-033-upstream-proxy.md)（上游可经 relay）
> 关联 SPEC：[SPEC-006 §9.1](SPEC-006-update-and-telemetry.md)（never upload 禁传表，§7 隐私呼应）/ [SPEC-007](SPEC-007-upstream-proxy.md)（上游代理，count_tokens 直连复用）
> 关联 PRD：sieve-prd-v2.0.md §9 #2 / §9 #11 / §11.2 / §11.3

---

## §0 文档定位

本 SPEC 定义 sieve daemon 对经过的 LLM 流量做**独立 token 核算**，按上游信任分级交叉比对 relay 声明的 `usage`，偏差超容差时报警的工程级实现规格。范围：`sieve-cli` 信任分级派生 + token 计数器 + `usage.db` 存储；`sieve-core` SSE 全文聚合 + OpenAI usage 解析补齐。

**核心认知（贯穿全文）**：本特性是**异常检测**，**不是**逐 token 对账。乘 1.5 = 多报 50%，远高于任何 tokenizer 的噪声（±5~10%），藏不住——独立计数只要够「抓系统性虚报」即可，不追求逐字节相等。这条认知决定了 Anthropic 输出无公开 tokenizer 也能近似估算够用（§4）。

**本 SPEC 不描述**：
- 信任分级以外的安全检测（属 pipeline detection，不在本特性范围）
- 模型偷换 / cache 状态撒谎检测（§9 登记为 out-of-scope）
- 内置价表的具体数值维护流程（属规则包 / 二进制发布，本 SPEC 只定义结构）

**两处与早期措辞的代码勘察纠正**（实现时以本 SPEC 为准）：

1. ADR-038 §2 流式小节写「复用现有 SSE Aggregator（`sieve_core::sse::aggregator`）」——**该模块路径不存在**。实际聚合器是 `sieve_core::tool_use_aggregator::Aggregator`（`lib.rs` re-export 为 `sieve_core::Aggregator`），且其 `BlockState::Text.buf` 在 `content_block_stop` 时被丢弃，**当前不暴露 assistant 输出全文**。§5 给出补齐方案。
2. 「入站经 redaction map 替换后内容」的提法在出站语境正确，**入站不存在该机制**——入站 `RedactAndAllow` 与 `Allow` 合并、原样放行原始 frame，`sieve-policy` lint `InboundAutoRedactForbidden` 明令禁止入站 auto_redact。本特性是**响应观测**（统计上游回报的 token），不涉及入站脱敏，此纠正仅为避免实现者误接。

---

## §1 背景与目标

### 痛点

Sieve 的卖点是「中转站会改写你的流量」。中转站（relay）返回的 `usage` 字段（`input_tokens` / `output_tokens` / `cache_*` 等）**只是 relay 完全控制的响应体里的一段 JSON**，可以乘 1.5、加常数地虚报来多收钱——这是真实存在的灰产。Sieve 夹在 agent 与上游之间，**两个方向的原始字节都有**，具备结构性优势独立核算、戳穿虚报。

当前代码库**无任何 token 计数 / tiktoken / usage 解析逻辑**：

- `protocol/anthropic.rs:20 max_tokens` / `protocol/openai.rs:38 max_tokens` 仅建模请求上限，不做统计。
- SSE parser 的 `SseEvent::MessageDelta { delta, usage: Option<serde_json::Value> }` 已**结构性捕获** Anthropic 的 `usage` 字段，但生产路径**从无消费者**（`Aggregator` 对 `MessageDelta` 落 `_ => Ok(None)`）。
- 全仓 grep `tiktoken` / `tokenizer` / `count_tokens` / `input_tokens` 在 `sieve-cli` 零命中——feature 全新（greenfield）。

[ADR-026](../design/ADR-026-port-based-listener-routing.md) 的 per-port 路由 + `provider_id` 已能区分「listener 指向谁」，但尚无「官方直连 vs 中转 relay」的信任标记——这是本特性信任分级的核心前提。

### 抓欺诈不需要精确（量级分析）

| 量 | 数量级 | 结论 |
|----|--------|------|
| tokenizer 噪声（独立估算 vs 真实 BPE） | ±5~10% | 容差需高于此，避免误报 |
| relay 乘 1.5 虚报 | +50% | 远高于噪声，必被检出 |
| relay 加固定常数虚报 | 随请求大小变化的相对偏差 | 小请求上相对偏差大，易检出 |
| `tolerance_pct` 默认 | 15% | 高于噪声、远低于欺诈量级，兼顾低误报与成本敏感 |

### 目标

1. 按上游信任分级（official / relay），仅对 relay 独立核算 `usage` 并交叉比对。
2. OpenAI 用 tiktoken 接近精确计数；Anthropic 输入默认本地近似估算（标注估算）、可选 `count_tokens` 直连权威核算。
3. 偏差超 `tolerance_pct` → StatusBar 报警 + 落本地 `usage.db`，**不阻断流量、不引入新 Block 路径**。
4. token 用量统计**严格本地、永不上传**（呼应 SPEC-006 §9.1 禁传表）。
5. 整个特性默认全关（`[billing_check].enabled = false`），开启零行为变化基线之外才生效。

---

## §2 配置 schema

### 2.1 `[billing_check]` 段 + `[[upstream]].trust` 字段

```toml
# 超额计费检测（默认全关；开启后仅对 relay 上游生效）
[billing_check]
enabled = false             # 总开关，默认 false（零行为变化 / 零新增出站 / 零计算开销）
tolerance_pct = 15          # 偏差容差百分比，默认 15；超过则报警
count_tokens_optin = false  # Anthropic count_tokens 直连开关，默认 false（§4 §9 #2 张力）

[[upstream]]
url = "https://api.anthropic.com"
protocol = "anthropic"
# 未写 trust → 按 host 派生为 official（§3）

[[upstream]]
port = 8788
url = "http://my-relay.example.com/v1"   # 中转站
protocol = "anthropic"
trust = "relay"             # 可省略；host 不在白名单时自动派生为 relay
```

### 2.2 字段语义

`[billing_check]` 段（`#[serde(default, deny_unknown_fields)]`，整段可省略）：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `enabled` | bool | `false` | 总开关；false 时不做任何计数 / 比对 / 落盘 |
| `tolerance_pct` | u8 | `15` | 偏差容差（百分比）；`>100` 启动期校验报错（§2.3） |
| `count_tokens_optin` | bool | `false` | 仅 Anthropic 生效；true 时输入侧改用官方 `count_tokens` 直连（一次主动出站，§4.2）|

`[[upstream]]` 新增字段（与 SPEC-007 的 `proxy` / `no_proxy` 并列）：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `trust` | `"official"` \| `"relay"` | 按 host 派生（§3），无法判定 → `relay` | 信任级；显式配置覆盖派生结果 |

### 2.3 启动期校验（fail-fast）

新段的硬性校验并入 `config.rs:check_safety_invariants(&self) -> Result<(), String>`（纯函数可单测），自动经 `enforce_safety_invariants()` 获得 `exit(1)` 语义（唯一调用点 `main.rs:76`，在 `Config::load` 之后、`AuditStore::init` 之前）：

```rust
// check_safety_invariants() 内追加（与 bind_addr / 端口冲突校验并列）
if self.billing_check.enabled && self.billing_check.tolerance_pct > 100 {
    return Err(format!(
        "[billing_check].tolerance_pct = {} 非法（必须 0..=100）",
        self.billing_check.tolerance_pct
    ));
}
```

> `count_tokens_optin = true` 不作为 fail-fast 错误，但启动期 `tracing::warn!` 显著提示「将向 api.anthropic.com 发起 Sieve 主动出站」（§4.2 / §9 #2 张力）。

### 2.4 Config 落地约定（贴合现有先例）

严格照搬 `UpdateConfig` 分段先例（`config.rs`）：

1. 定义 `BillingCheckConfig` 子 struct（`#[derive(Debug, Clone, Deserialize, Serialize)]` + `#[serde(default, deny_unknown_fields)]`），所有字段给默认值；手写 `impl Default`（因 `tolerance_pct` 默认非零）。
2. 在 `Config` struct 紧邻 `pub update: UpdateConfig` 之后加 `#[serde(default)] pub billing_check: BillingCheckConfig`。
3. 在 `Config` 的**手写** `impl Default`（非 derive）补 `billing_check: BillingCheckConfig::default()`——漏补则编译失败。

> `Config` 整体只 derive `Deserialize` 不 derive `Serialize`，子 struct 可保留 `Serialize`（与 `UpdateConfig` 一致，不影响）。`Config` 与 `UpstreamListener` 均带 `#[serde(deny_unknown_fields)]`，新字段必须真实加进 struct，不能靠静默忽略。

---

## §3 信任分级派生（host → official / relay）

### 3.1 派生时机与落点

**裁定：config 加载期派生（请求期零开销）**。在 `config.rs` 新增 `Trust` enum 与 `UpstreamListener::resolved_trust()`，与已有的 `resolved_provider_id()` 并列。`Trust` 照 `Protocol` enum 风格写（`#[serde(rename_all = "snake_case")]` + `#[default]`）：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    Official,
    #[default]
    Relay, // 保守默认：无法判定时按不可信处理（fail-closed 倾向）
}
```

### 3.2 派生规则

```rust
/// 官方直连 host 白名单（MVP 硬编码；ADR-038 决策1 注明「可配置扩展」）。
const OFFICIAL_HOSTS: &[&str] = &["api.anthropic.com", "api.openai.com"];

impl UpstreamListener {
    /// 信任级派生：显式 trust 优先；否则按 url host 判定；解析失败 → Relay。
    /// 仿照 resolved_provider_id() 的 host 提取方式（http::Uri authority().host()）。
    pub fn resolved_trust(&self) -> Trust {
        // 注：若引入显式 Option<Trust> 字段表达「未配置」，此处先判 Some 直接返回；
        // 若用默认 Relay 的非 Option 字段，则配置文件显式写 official 即覆盖派生。
        match http::Uri::try_from(&self.url).ok().and_then(|u| {
            u.authority().map(|a| a.host().to_ascii_lowercase())
        }) {
            Some(host) if OFFICIAL_HOSTS.contains(&host.as_str()) => Trust::Official,
            Some(_) => Trust::Relay,
            None => Trust::Relay, // 解析失败 → 保守按 relay（fail-closed）
        }
    }
}
```

> **fail-closed 方向**：把可信当不可信只多算一次（浪费一点计算），把不可信当可信会漏掉欺诈——所以无法判定时取 `Relay`。

### 3.3 legacy 单 upstream 的 trust 默认

`Config::resolved_upstreams()` 是所有 listener 创建的唯一入口。legacy 单 `upstream_url` 映射时默认 `url = https://api.anthropic.com`，该路径**必须显式映射 `trust = Official`**（host 命中白名单）——否则保守默认 `Relay` 会把官方直连误判成 relay，平白付计算开销。

### 3.4 trust 透传链（复刻 provider_id 流转）

trust 完全复刻 `provider_id` 的运行时透传链（每个传 `listener_provider_id` 的点旁边传 `listener_trust`）：

```
UpstreamListener.resolved_trust()           // config 加载期（§3.2）
  → ListenerSpec.trust                       // daemon::run 构造循环，从 u.resolved_trust()
  → RequestCtx.listener_trust                // accept_loop 从 spec.trust 取（每连接一次，O(1) clone）
  → 响应观测器判断 if listener_trust == Relay && billing_check.enabled  // §5/§6 唯一可信来源
```

> trust **不进 `audit_events` 表**（`provider_id` 已足够审计归因；token 用量落独立 `usage.db`，§7）。`RequestCtx.listener_trust` 不要重蹈 `listener_provider_id` 的滞后 `#[allow(dead_code)]` 标注——它从构造起即被观测器消费。
>
> trust 派生留在 `sieve-cli` config 层，**不下沉到 `sieve-core::Forwarder`**——Forwarder 是库 crate，按七 crate 边界禁感知 trust 业务概念（只暴露 `upstream_host()` 供需要时查 host）。

---

## §4 独立计数

> 原则：**永远优先权威信源**（official 的 `usage` / `count_tokens` 直连）；只在对抗不可信 relay 时才自己算；自己算也不手搓 tokenizer。

### 4.1 计数矩阵

| 协议 | 方向 | 计数方法 | 精度 | 标注 |
|------|------|---------|------|------|
| OpenAI | 输入 | tiktoken（`o200k_base` / `cl100k_base`）+ per-message 框架开销 | 接近精确 | exact |
| OpenAI | 输出 | tiktoken 数补全文本 | 接近精确 | exact |
| Anthropic | 输入（默认） | 本地近似估算 | ±5~10% | **estimated** |
| Anthropic | 输入（opt-in） | `count_tokens` 直连官方（绕 relay） | 近权威 | authoritative |
| Anthropic | 输出 | 本地近似估算（无公开 tokenizer） | ±5~10% | **estimated** |

> 每条独立计数记录必须携带 `method` 标注（exact / estimated / authoritative），落 `usage.db`（§7）并在 CLI / GUI 显示，避免用户把估算误读为精确账单。

### 4.2 OpenAI：tiktoken

- 选用 `tiktoken-rs` crate（**不手搓 BPE**）。模型 → encoding 映射：GPT-4o 及更新 → `o200k_base`；老模型 → `cl100k_base`。
- **per-message 框架开销**：chat 模型每条消息有固定 token 开销（角色标记 + 分隔符），每轮回复有起始开销。按 OpenAI 公开的 `num_tokens_from_messages` 规则累加：

```rust
/// OpenAI chat 输入 token 估算（含 per-message 框架开销）。
/// 规则源自 OpenAI cookbook num_tokens_from_messages（按 encoding 区分常量）。
fn count_openai_input(messages: &[ChatMessage], enc: &CoreBPE) -> u32 {
    const TOKENS_PER_MESSAGE: u32 = 3; // <|im_start|>{role}\n{content}<|im_end|>\n 框架
    const TOKENS_PER_REPLY: u32 = 3;   // 每次回复起始 <|im_start|>assistant
    let mut total = 0u32;
    for m in messages {
        total += TOKENS_PER_MESSAGE;
        total += enc.encode_ordinary(&m.role).len() as u32;
        total += enc.encode_ordinary(&m.content).len() as u32;
        // name 字段若存在再 +1（OpenAI 规则）
    }
    total + TOKENS_PER_REPLY
}
```

- **vocab 文件离线约束（致命）**：`tiktoken-rs` 若 build.rs 或运行期联网下载 BPE 词表，直接违反 PRD §9 #2「完全本地、唯一允许出站」——**必须选词表内嵌 / 离线加载方案**（评估打包进二进制，增体积但零运行时出站）。这是引入 `tiktoken-rs` 的最大约束，PR 必须实测确认无构建期 / 运行期下载。

### 4.3 Anthropic：默认近似估算 + opt-in count_tokens 直连

#### 默认路径（`count_tokens_optin = false`）

Claude 无公开 tokenizer，默认**本地近似估算**，明确标 `method = estimated`：

```rust
/// Anthropic 近似 token 估算（无公开 tokenizer，标 estimated）。
/// 经验近似：英文约 3.5~4 字符/token，含 CJK / 代码时偏离更大——
/// 但本特性是抓「乘 1.5」量级，±5~10% 噪声完全够用（§1 量级分析）。
fn estimate_anthropic_tokens(text: &str) -> u32 {
    // 占位算法，实现时按语料校准系数；可叠加字符类别加权（CJK / 标点 / 空白）。
    let chars = text.chars().count() as f64;
    (chars / 3.8).ceil() as u32
}
```

#### opt-in 路径（`count_tokens_optin = true`，§9 #2 核心张力）

调官方 `POST /v1/messages/count_tokens`（**直连、绕过 relay**）拿权威输入数再跟 relay 比对。该接口有独立 rate limit、不按 token 计费，等于近乎免费的第二信源；官方自称「估计值」，可能跟计费输入差几个系统 token，对抓量级虚报够用。

**已裁定路径 (C)（ADR-038 §5，2026-06-19）**：默认不联网（仅本地近似估算，§9 #2 一字不破）；`count_tokens` 直连为独立开关默认关，仅用户显式开启才生效。实现约束：

- **新建专用 Forwarder 指向硬编码 `api.anthropic.com`**——**不能复用 relay listener 的 forwarder**（那指向 relay）：
  ```rust
  // 复用现有 hyper-rustls 栈；proxy 复用 cfg.global_proxy()（受限网络下官方 endpoint 也需出网）
  let direct = Forwarder::new("https://api.anthropic.com", cfg.global_proxy())?;
  let resp = direct.forward(build_count_tokens_request(redacted_body)?).await?;
  ```
- 仅对 `relay` 上游生效；请求体为**脱敏后**内容（复用 `outbound_redact`，不泄密钥）。
- config / UI 必须显著警示「这会向官方 endpoint 发起一次 Sieve 主动出站」（§2.3 启动 warn + 后续 UI 体现）。
- 文档把它与「联网做 verifier（Sieve 自营云后端）」严格区分：它调的是用户已在调的同一可信官方上游，绕 relay 戳穿 relay 的谎，不是 phone-home 到 Sieve 服务器。

---

## §5 流式累计 tokenize

### 5.1 响应完成观测点（fire-and-forget，off hot path）

token 计数器挂在 pipeline **响应完成后**的观测节点，复用现有审计 fire-and-forget 模式（`tokio::spawn`，不阻塞 daemon tee 转发热路径，背压 channel 深度仅 64）。四个落点全覆盖（content-type 路由矩阵，PRD §9 #16；漏挂任一类视为 P0）：

| 路由类 | 落点函数 | 流结束确定点 | usage 来源 |
|--------|---------|-------------|-----------|
| Anthropic SSE | `forward_with_inbound_inspection` 的 tee spawn 闭包尾 | `parser.flush()` 之后、闭包结束前 | `SseEvent::MessageDelta.usage` |
| OpenAI SSE | `forward_with_openai_inbound_inspection` 对称落点 | 对应 `flush()` 之后 | 末帧 `usage`（需 `include_usage`，§5.3）|
| Anthropic JSON | `handle_anthropic_json_inbound`：`resp_body.collect()` 后、return 前 | body 一次性可得 | `resp_json["usage"]` 直读 |
| OpenAI JSON | `handle_openai_json_inbound` 对称落点 | 同上 | `resp_json["usage"]` 直读 |

> 流式 `message_stop` **不保证总到达**（提前断流 / EOF），观测器必须同时挂 `parser.flush()` 后路径，不能只依赖 `SseEvent::MessageStop`，否则断流场景漏计。被拦下（Block / Hold deny）没发到上游的请求自然无 `usage`——观测器逻辑必须**容忍 usage 缺失，不能 fail**（ADR-038 决策2 接受为小缺口）。

### 5.2 全文聚合（独立计数需累计完整补全文本）

独立计数时累计完整文本再 tokenize。**现有 `tool_use_aggregator::Aggregator` 不直接暴露 assistant 输出全文**（§0 纠正点 1：`BlockState::Text.buf` 在 `content_block_stop` 时丢弃）。两条补齐路径，二选一：

- **(A) 扩展 Aggregator**：在 `message_stop` / 流结束时吐出 `BlockState::Text.buf` 累积的全文（当前注释标「预留 Week 4 扩展」）。
- **(B) 在 daemon tee 循环单独累积**：旁挂一个累计器，从 `SseEvent` 的 `SseDelta::TextDelta.text` + `ThinkingDelta.thinking` 自行 `push_str`。

> 两路径均受 `MAX_TEXT_BUFFER_BYTES`（1 MiB）上限约束。token 观测器与 `InboundFilter` 热路径检测**解耦**（独立 fire-and-forget，勿混入检测逻辑）。

```rust
// (B) 路径示意：在 tee 闭包内累计，流结束后 spawn 计数
let mut completion_buf = String::new();
while let Some(frame) = body_stream.next().await {
    for ev in parser.push_chunk(&frame)? {
        match &ev {
            SseEvent::ContentBlockDelta { delta: SseDelta::TextDelta { text }, .. } => {
                completion_buf.push_str(text); // 受 1 MiB 上限保护
            }
            SseEvent::MessageDelta { usage: Some(u), .. } => relay_usage = parse_usage(u),
            _ => {}
        }
        forward_frame(ev).await?; // 只读 tee，绝不改流（§7 / §9 #11）
    }
}
let _flushed = parser.flush();
// off hot path：流读完后再 spawn 计数 + 比对
if listener_trust == Trust::Relay && billing_check.enabled {
    let buf = std::mem::take(&mut completion_buf);
    tokio::spawn(async move { account_and_compare(buf, relay_usage, ctx).await });
}
```

### 5.3 OpenAI usage 补齐（比 Anthropic 多一步）

- `OpenAIStreamingChunk`（`protocol/openai.rs:127`）**当前不含 usage 字段**，需补 `usage: Option<...>`。
- OpenAI 流式默认不返 usage，必须**请求侧注入 `stream_options.include_usage = true`**（否则流式拿不到 relay 声明值）——这是请求改写，会动到 outbound 路径而非纯响应观测，实现时注意。
- Anthropic 侧 `usage` 已被 SSE parser 结构性捕获（`MessageStart.message` 含起始 `input_tokens`，`MessageDelta.usage` 含最终 `output_tokens`），**无需改 parser**——观测器累计 / 取末值即可（不能只读 `message_start`）。

---

## §6 计费比对

### 6.1 比对公式

```
应收 token = 独立计数（§4）
偏差 = |relay_claim - independent_count| / independent_count
若 偏差 > tolerance_pct / 100 → 报警
```

输入侧、输出侧分别比对（relay 可能只虚报其中一侧）。应收成本 = 独立计数 × **内置官方单价**（本地价表，按 model + 信任级查）。

```rust
struct BillingComparison {
    request_id: String,
    model: String,
    input_independent: u32,   // §4 计数
    input_relay_claim: u32,   // relay usage.input_tokens
    output_independent: u32,
    output_relay_claim: u32,
    method: CountMethod,      // exact / estimated / authoritative
    input_deviation_pct: f64,
    output_deviation_pct: f64,
    over_tolerance: bool,
}

fn compare(c: &BillingComparison, tolerance_pct: u8) -> bool {
    let tol = tolerance_pct as f64 / 100.0;
    let dev = |claim: u32, indep: u32| -> f64 {
        if indep == 0 { return 0.0; } // 容忍空响应，不除零、不误报
        (claim as i64 - indep as i64).abs() as f64 / indep as f64
    };
    let id = dev(c.input_relay_claim, c.input_independent);
    let od = dev(c.output_relay_claim, c.output_independent);
    id > tol || od > tol
}
```

### 6.2 报警行为

- **超容差 → StatusBar 通知 + 写 `usage.db` 标记 `over_tolerance = true`**。
- **不阻断流量、不引入新 Block 路径**（这是计费监督，不是安全拦截，对齐 PRD §9 #15 精神保守起步）。
- `method = estimated` 的报警文案必须标注「基于近似估算，建议开启 `count_tokens_optin` 获取权威核算」，避免把估算噪声当确凿欺诈呈现给用户。
- official 直连**不比对**（`usage` 权威，§3 trust 守门），零开销。

---

## §7 隐私（严格本地，永不上传）

token 用量恰是 README 发誓「从不上传」的那个 usage record。**统计严格本地、永不上传**，只在本地 GUI / CLI 可见——即使做聚合产品分析也不行，那直接撕了隐私承诺。本节呼应 [SPEC-006 §9.1](SPEC-006-update-and-telemetry.md) 禁传表：

| 允许本地存储（永不上传） | 绝对禁止任何出站 |
|------------------------|----------------|
| 独立计数 token 数（input / output） | 上传到 Sieve 自营服务器 |
| relay 声明的 usage 数值 | 上传到任何第三方 |
| 偏差百分比 / over_tolerance 标记 | 聚合产品分析上报 |
| model 名 / 信任级 / request_id 关联 | prompt / response 正文（本就不入 usage.db）|

### 7.1 `usage.db` 存储

- 路径 `~/.sieve/usage.db`，权限 `0600`，append-only（复用 `audit.db` 的 `spawn_blocking` + BEFORE UPDATE/DELETE 触发器模式）。
- **结构上独立于 `audit.db`**（token 用量是新数据域，不污染审计模型），与 `audit_events` 经 `request_id` 关联。
- **无任何出站上报路径**——代码层不存在向 `usage.db` 写入后发网络请求的调用。

```sql
CREATE TABLE usage_records (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_rfc3339 TEXT    NOT NULL,
    request_id        TEXT    NOT NULL,   -- 关联 audit_events.request_id
    provider_id       TEXT    NOT NULL,
    model             TEXT    NOT NULL,
    trust             TEXT    NOT NULL,   -- official / relay（official 通常不入表）
    method            TEXT    NOT NULL,   -- exact / estimated / authoritative
    input_independent  INTEGER NOT NULL,
    input_relay_claim  INTEGER,           -- nullable：缺 usage 时为 NULL
    output_independent INTEGER NOT NULL,
    output_relay_claim INTEGER,
    input_deviation_pct  REAL,
    output_deviation_pct REAL,
    over_tolerance    INTEGER NOT NULL DEFAULT 0  -- bool
);
```

### 7.2 只读不改写（§9 #11）

Sieve **只读 / 只比对** `usage`，**绝不改写**上游或 relay 的 `usage` 字段（即使检出虚报也只报警，不动协议）。计数是旁路观测，tee 循环只读不改流——不伪造 / 不注回 `usage` / `stop_reason`。

### 7.3 CLI 查询（`sieve usage`）

新建顶层子命令 `sieve usage`（只读聚合查询，与 `sieve audit` 同源读 `~/.sieve/`，但读 `usage.db`）：

- 仿 `commands/audit.rs` 结构：`async run → run_async`，`Connection::open_with_flags(READ_ONLY | NO_MUTEX)`，`--format jsonl|pretty`（默认 jsonl）。
- 复用 `parse_duration`（支持 `1h` / `30m` / `7d`）做 `--since`；可 `GROUP BY model / provider_id`，`--over-tolerance` 只看异常记录。
- async 子命令强制 `run → run_async` 委托模式，**禁止内部 `block_on` / 自建 runtime**（嵌套 runtime panic exit 134，CHANGELOG 已记 `sieve audit` / `sieve decisions` 同坑）。
- 输出风格：无颜色库，emoji（✅ / ⚠）+ 中文 + jsonl/pretty 双格式。

---

## §8 测试矩阵

### 8.1 relay 虚报样本测试（回归硬约束，Phase 2 必带）

| 测试 ID | 场景 | 验收标准 |
|---------|------|---------|
| `relay_multiply_1_5_detected` | relay 把 `output_tokens` 乘 1.5 | `over_tolerance = true`，StatusBar 报警 |
| `relay_add_constant_detected` | relay 给 `input_tokens` 加固定常数 | 小请求上偏差 > 15% 被检出 |
| `official_direct_no_false_positive` | official 直连，usage 与独立计数一致 | 不比对 / 不报警（trust 守门）|
| `tokenizer_noise_within_tolerance` | relay 诚实，独立估算有 ±8% 噪声 | `over_tolerance = false`，不误报 |
| `estimated_method_labeled` | Anthropic 默认估算路径 | 记录 `method = estimated`，报警文案标注估算 |
| `usage_missing_no_fail` | 流被 Block / 提前断流，无 usage | 观测器不 panic、不报警、记录可缺失 relay_claim |
| `empty_response_no_div_zero` | 上游空响应 independent = 0 | 偏差计 0，不除零、不误报 |

### 8.2 计数 / 协议覆盖

| 测试 ID | 场景 | 验收标准 |
|---------|------|---------|
| `openai_tiktoken_o200k` | GPT-4o 输入计数 | 与 OpenAI 官方 `num_tokens_from_messages` 一致（含框架开销）|
| `openai_tiktoken_cl100k` | 老模型输入计数 | encoding 映射正确 |
| `openai_include_usage_injected` | OpenAI 流式请求 | 请求体含 `stream_options.include_usage = true` |
| `anthropic_count_tokens_optin_off_no_egress` | `count_tokens_optin = false` | 无任何 api.anthropic.com 主动出站，仅本地估算 |
| `anthropic_count_tokens_optin_on_egress` | `count_tokens_optin = true` | 打官方 endpoint、绕 relay、请求体已脱敏 |
| `content_type_matrix_4` | Anthropic SSE / Anthropic JSON / OpenAI SSE / OpenAI JSON | 四类全挂观测器（PRD §9 #16）|

### 8.3 信任分级 / 隐私

| 测试 ID | 场景 | 验收标准 |
|---------|------|---------|
| `trust_derive_official_host` | url = api.anthropic.com，未写 trust | `resolved_trust() == Official` |
| `trust_derive_relay_default` | url = 任意非白名单 host | `resolved_trust() == Relay` |
| `trust_unparseable_url_relay` | url 解析失败 | `resolved_trust() == Relay`（fail-closed）|
| `trust_explicit_override` | 白名单 host 但显式 `trust = "relay"` | 显式覆盖派生，为 Relay |
| `legacy_upstream_official` | legacy 单 upstream（默认 api.anthropic.com）| 映射 `trust = Official`，不误付计算开销 |
| `usage_never_uploaded` | 全流程断言 | 无任何向外发送 `usage.db` 内容的网络调用（grep 出站 + mock 断言）|
| `tolerance_out_of_range_fails_fast` | `tolerance_pct = 150` 且 enabled | 启动 `exit(1)`，明确错误信息 |

---

## §9 路线图（本次 out of scope）

两个比 token 虚报危害更大的相邻骗术，本次**不实现**，登记备查：

1. **模型偷换**：点 Opus、偷跑 Haiku、按 Opus 计费（需响应指纹 / 质量探针，更难）。
2. **缓存状态撒谎**：Anthropic cache 读/写价格差很多，relay 可谎报 cache miss 按全价收；`usage` 里有 `cache_read_input_tokens` / `cache_creation_input_tokens` 但同样可被篡改。`usage.db` schema（§7.1）预留扩展空间，未来可加 cache token 列做交叉比对。

升级任一为新检测项需走新 ADR（信任边界 / 新出站决策级别）。

---

## §10 变更记录 / 相关文档

### 变更记录

- v0.1（2026-06-19）：初稿，ADR-038 落地详规。固化裁定：三档无关本特性（属 ADR-037）；`[billing_check].enabled` / `count_tokens_optin` 默认 false；`tolerance_pct` 默认 15；信任分级按 host 派生保守默认 relay；usage 严格本地永不上传。纠正两处早期措辞（`sse::aggregator` → `tool_use_aggregator`；入站无 redaction_map 替换）。

### 相关文档

- [ADR-038: 超额计费检测](../design/ADR-038-overbilling-detection.md)（本 SPEC 权威决策源）
- [ADR-003: 完全本地运行，绝不联网做 verifier](../design/ADR-003-local-only-no-cloud-verifier.md)（§4/§6 张力）
- [ADR-026: Port-based listener routing](../design/ADR-026-port-based-listener-routing.md)（provider_id 透传范本）
- [ADR-033: 上游转发代理支持](../design/ADR-033-upstream-proxy.md)（count_tokens 直连 proxy 复用）
- [SPEC-006: 更新通道 + 装机遥测](SPEC-006-update-and-telemetry.md)（§9.1 never upload 禁传表）
- [SPEC-007: 上游转发代理支持](SPEC-007-upstream-proxy.md)（Forwarder 注入模式）
- [docs/api/api-reference.md §3](../api/api-reference.md) — config schema（`[billing_check]` + `[[upstream]].trust`）+ §2 `sieve usage` 子命令
- [docs/design/data-model.md](../design/data-model.md) — `usage.db` schema / 价表结构 / 信任级派生
- [docs/glossary.md](../glossary.md) — 超额计费检测 / 信任分级 / 独立 token 核算 / tiktoken / count_tokens 直连
- [docs/specs/INDEX.md](INDEX.md) — SPEC 索引
