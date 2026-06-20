# ADR-038: 超额计费检测（独立 token 核算 + 对抗不可信中转站）

## 状态
**Accepted**
> 决策日期：2026-06-19
> 范围：sieve daemon 对经过的 LLM 流量做独立 token 核算，按上游信任分级交叉比对 relay 声明的 `usage`，偏差超容差报警；统计严格本地、永不上传
> 关联：[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（绝不联网做 verifier——本 ADR 核心张力）/ [ADR-018](./ADR-018-openai-protocol-adaptation.md)（双协议）/ [ADR-026](./ADR-026-port-based-listener-routing.md)（per-port 路由 + provider_id，信任分级前提）/ [ADR-033](./ADR-033-upstream-proxy.md)（上游可经 relay）/ [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)（never upload 隐私承诺）/ [SPEC-010](../specs/SPEC-010-overbilling-detection.md)（工程级详细设计，Phase 2）
> 关联 PRD：v2.0 §9 #2、§9 #11、§11.2、§11.3

## 背景

Sieve 的卖点是「中转站会改写你的流量」。中转站（relay）返回的 `usage` 字段（`input_tokens` / `output_tokens` 等）**只是 relay 完全控制的响应体里的一段 JSON**，可以乘 1.5、加常数地虚报来多收钱——这是真实存在的灰产。Sieve 作为夹在 agent 与上游之间的代理，**两个方向的原始字节都有**，具备结构性优势去独立核算、戳穿虚报。

当前代码库**无任何 token 计数 / tiktoken / usage 解析逻辑**（`max_tokens` 仅在协议层出现，不做统计）。[ADR-026](./ADR-026-port-based-listener-routing.md) 的 per-port 路由 + `provider_id` 已经能区分「这个 listener 指向谁」，但尚无「官方直连 vs 中转 relay」的信任标记——这是本特性信任分级的核心前提。

### 把功能定准（整个设计的关键认知）

**目标不是「逐 token 精确对账」，而是「超额计费异常检测」。** 不需要精确就能抓欺诈：乘 1.5 = 多报 50%，远高于任何 tokenizer 的噪声（±5~10%），藏不住。独立计数只要够「抓系统性虚报」即可，不追求逐字节相等。这决定了：Anthropic 输出无公开 tokenizer 只能近似估算——**对抓「乘 1.5」量级完全够用**。

## 决策

### 1. 按上游信任分级（不是二选一）

引入上游信任标记，区分两类：

| 信任级 | 判定 | `usage` 处理 |
|--------|------|-------------|
| `official`（官方直连） | url host ∈ {`api.anthropic.com`, `api.openai.com`}（可配置扩展） | `usage` **权威**，直接采纳，不必自己算 |
| `relay`（经中转） | 其余所有上游 | `usage` 视为**未经验证的声明**，必须独立核算 + 交叉比对，偏差超容差**报警** |

信任级在 `[[upstream]]` 上**按 host 自动派生**，可显式 `trust = "official" | "relay"` 覆盖（保守默认：无法判定时按 `relay` 处理，fail-closed 倾向——把可信当不可信只多算一次，把不可信当可信会漏掉欺诈）。复用 [ADR-026](./ADR-026-port-based-listener-routing.md) 的 `provider_id` 做审计归因。

### 2. 独立计数：永远优先权威信源，只在对抗不可信 relay 时才自己算，自己算也不手搓 tokenizer

#### 输入侧
- **OpenAI** → 用 **tiktoken**（GPT-4o 及更新用 `o200k_base`，老模型 `cl100k_base`），把 chat 消息每条/每轮的框架开销（per-message / per-reply tokens）也算进去，接近精确。用 `tiktoken-rs` crate（不手搓 BPE）。
- **Anthropic** → **Claude 无公开 tokenizer**。两种独立计数来源，二选一（见决策 5 的红线裁定）：
  - **(默认) 本地近似估算**：明确标为估算，零新增出站，对抓「乘 1.5」量级够用。
  - **(opt-in，需审核裁定是否触红线) 调官方 `POST /v1/messages/count_tokens`（直连、绕过 relay）**：拿权威输入数再跟 relay 比对。该接口有独立 rate limit、不按 token 计费，等于近乎免费的第二信源。官方自称「估计值」，可能跟计费输入差几个 token（系统加的不计费 token），**对抓量级虚报完全够用**。

#### 输出侧
- **OpenAI** → tiktoken 数已代理的补全文本，接近精确。
- **Anthropic** → 无公开 tokenizer，只能**近似估算**（明确标为估算）。

#### 计费比对
独立计数 × **官方公开单价**（本地内置价表，按 model + 信任级查）= 应收成本，跟 relay 声明 / 实际账单比对。偏差 = `|relay_claim - independent_count| / independent_count`，超过 `tolerance_pct`（**默认 15%**，已裁定 2026-06-19——25% 太高、涉及成本，15% 仍远高于 tokenizer 噪声 ±5~10% 而远低于欺诈量级 +50%，兼顾低误报与成本敏感）→ 报警（StatusBar 通知 + 写本地 usage 记录，**不阻断流量**——这是计费监督，不是安全拦截，不引入新 Block 路径）。

#### 流式
`usage` 在最后一个事件里（OpenAI 流式需开 `include_usage`）；独立计数时**累计完整文本再 tokenize**。**读 relay 声明的 usage 无需改 SSE parser**（2026-06-19 勘察校正）：`SseEvent::MessageDelta { usage: Option<Value> }`（`crates/sieve-core/src/sse/parser.rs:111-116`）已捕获 Anthropic 最终 `output_tokens`，`MessageStart.message` 已含起始 `usage`——观测器只需在 daemon 的 tee 转发流里 match 这两个 event 取 usage（生产路径目前无消费者）。聚合完整补全文本走 `sieve_core::tool_use_aggregator`（**注意：模块名是 `tool_use_aggregator`，非 `sse::aggregator`**），独立计数逻辑为全新代码。被拦下、没发到上游的请求自然无 `usage`，这是**可接受的小缺口**。

### 3. 实现落点（结构性优势：作为代理，双向原始字节都有）

- **配置与默认（已裁定 2026-06-19）**：整个特性**可配置、默认全关**。`[billing_check].enabled` 默认 `false`、`count_tokens_optin` 默认 `false`——不开启则零行为变化、零新增出站、零计算开销。
- 核算挂在 pipeline **响应完成后**的观测节点，**off the hot path**（fire-and-forget，不阻塞转发，对齐 PRD §9 性能预算 + 现有审计 fire-and-forget 模式）。
- 仅当上游为 `relay` 且 `[billing_check].enabled = true` 时启用；`official` 直连不核算（`usage` 权威）。
- tiktoken vocab 文件需随二进制分发或首启缓存——评估打包进二进制（避免运行时下载，PRD §9 #2）。

### 4. 隐私红线（不可放宽）

**token 用量恰恰就是 README 发誓「从不上传」的那个 usage record。统计数据严格本地、永不上传**，只在本地 GUI / CLI 可见。即使想做聚合产品分析也不行——那直接撕了隐私承诺。本 ADR 呼应 [SPEC-006 §9.1](../specs/SPEC-006-update-and-telemetry.md) 的「禁止上传」表格（prompt / response / API key / 任何使用记录均禁）。

统计落本地 SQLite（`~/.sieve/usage.db`，0600，append-only，结构上独立于 `audit.db`——token 用量是新数据域，但与 `events` 经 `request_id` 关联，不污染审计模型）。无任何出站上报路径。

### 5. 核心张力：`count_tokens` 直连是否触 PRD §9 #2「绝不联网做 verifier」（**必须审核裁定**）

PRD §9 #2 是最硬的红线：「任何外部 token / 签名 / 规则的远端校验都摧毁产品定位」，唯一允许出站为 ①用户主动调的上游 LLM ②`updates.sieveai.dev` ③`cdn.sieveai.dev`。

Anthropic 的 `count_tokens` 直连**字面上正是「联网做 token 的远端校验」**——这是本特性最大的设计张力，属信任边界级决策，**必须由审核裁定**：

- **正面论证（倾向允许）**：它调的是**官方上游本身**（`api.anthropic.com`，与用户已在调的同一可信方），**绕过 relay** 去戳穿 relay 的谎；它**不是** Sieve phone-home 到任何 Sieve 自营服务器（§9 #2 的本意是「Sieve 不能要求自己的云后端 / 不能把数据外送给 Sieve」，LiteLLM 同构风险）。请求体是**脱敏后**内容（复用 redaction），不泄密钥。
- **反面论证（倾向禁止）**：它是 **Sieve 主动发起**的、**非用户直接触发**的出站，且语义上确实是「远端 token 校验」。开此口子可能侵蚀 §9 #2 的绝对性叙事。

**已裁定（2026-06-19）：路径 (C)**——
- **默认不联网做任何事情**：Anthropic 输入侧默认**只用本地近似估算**（决策 2 默认路径），零新增出站，§9 #2 一字不破。
- `count_tokens` 直连作为**独立开关 `count_tokens_optin`，默认 `false`**，**只有用户显式开启才生效**；启用时 config / UI 必须显著警示「这会向官方 endpoint（`api.anthropic.com`）发起一次 Sieve 主动出站」。后续在用户界面中明确体现该开关及其含义。
- 开启后仅打官方 endpoint、仅对 `relay` 上游生效、请求体为脱敏后内容；文档把它与「联网做 verifier（Sieve 自营云后端）」严格区分。

### 6. 路线图（本次 out of scope，登记为未来方向）

两个比 token 虚报危害更大的相邻骗术，本次**不实现**，登记备查：
1. **模型偷换**：点 Opus、偷跑 Haiku、按 Opus 计费（需响应指纹 / 质量探针，更难）。
2. **缓存状态撒谎**：Anthropic cache 读/写价格差很多，relay 可谎报 cache miss 按全价收；`usage` 里有 `cache_read_input_tokens` / `cache_creation_input_tokens` 但同样可被篡改。

### 硬约束分析（必须逐条成立才接受）

- **PRD §9 #2（绝不联网做 verifier）**：**已裁定路径 (C)**——默认不联网（本地 tiktoken / 近似估算，零新增出站）；`count_tokens` 直连为独立开关 `count_tokens_optin`，默认关，仅用户显式开启才生效。默认姿态下 §9 #2 一字不破。✔
- **PRD §9 #11（不在协议层撒谎，不伪造 usage）**：Sieve **只读 / 只比对** `usage`，**绝不改写**上游或 relay 的 `usage` 字段（即使检出虚报也只报警，不动协议）。✔
- **PRD §11.2 / §11.3（never upload）**：统计严格本地，无任何上报路径。✔ 决策 4 守护。
- **PRD §9 #15 精神（保守起步，不引入新 Block 路径）**：计费异常仅 StatusBar 报警，不阻断流量。✔

## 影响

### 正面影响
- 把「中转站不可信」从口号变成**可量化戳穿的能力**：检出系统性 token 虚报（乘 1.5 类灰产），强化产品定位。
- 利用 Sieve 作为代理的结构性优势（双向原始字节），独立核算，不依赖 relay 自报。
- 信任分级让官方直连零开销（`usage` 权威）、只对 relay 付计算代价，精准不浪费。
- 统计本地可见，给用户「我这单到底该花多少」的透明度。

### 负面影响
- **默认姿态下 Anthropic 输入核算只能近似**（`count_tokens` 默认关）：精度低于 OpenAI 侧（有 tiktoken），但够抓「乘 1.5」量级；用户需更高精度可显式开 `count_tokens_optin`（接受一次官方直连出站）。
- 新增依赖 `tiktoken-rs`（+ vocab 文件打包，增二进制体积），过 cargo-deny + pin（PRD §9 #6）。
- Anthropic 输出侧无公开 tokenizer，只能近似估算——必须明确标注「估算」，避免用户误以为是精确账单。
- 价表内置 = 需随单价变动维护（model 单价漂移时更新规则包 / 二进制）。
- 流式被拦下的请求无 `usage`，留小缺口（可接受）。

### 需要更新的文档（derivation rule）
- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行（P0）
- [data-model.md](./data-model.md)：新增「§N token 用量核算」——`usage.db` schema、独立计数记录结构、信任级派生、价表结构；`[[upstream]] trust` 字段（P0）
- [api-reference.md](../api/api-reference.md) §3 config schema：新增 `[billing_check]` 段（`enabled` / `tolerance_pct` / `count_tokens_optin`）+ `[[upstream]].trust` 字段；§2 新增 `sieve usage` 查询子命令（P1）
- [glossary.md](./glossary.md)：新增术语 `超额计费检测 / overbilling detection` / `信任分级（official / relay）` / `独立 token 核算` / `tiktoken` / `count_tokens 直连`（P0）
- 新建 [SPEC-010-overbilling-detection.md](../specs/SPEC-010-overbilling-detection.md)：tiktoken 框架开销算法、流式累计 tokenize、信任级状态机、容差比对、relay 虚报样本测试矩阵 + [specs/INDEX.md](../specs/INDEX.md) 加行（P0，Phase 2）
- [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)：交叉引用「usage 统计严格本地」呼应 never-upload（P1）
- [CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Added 条目（P1）
- **回归测试硬约束**：「relay 虚报（乘 1.5 / 加常数）被检出」样本测试 + 「usage 永不上传」断言（Phase 2 必带）

## 相关文档
- [SPEC-010: 超额计费检测](../specs/SPEC-010-overbilling-detection.md)（Phase 2 落地）
- [ADR-003: 完全本地运行，绝不联网做 verifier](./ADR-003-local-only-no-cloud-verifier.md)
- [ADR-026: Port-based listener routing](./ADR-026-port-based-listener-routing.md)
- [ADR-033: 上游转发代理支持](./ADR-033-upstream-proxy.md)
- [SPEC-006: 更新通道 + 装机遥测（never upload 承诺）](../specs/SPEC-006-update-and-telemetry.md)
