# ADR-038: 超额计费检测（可选本地计费核算 + 上游信任分级）

## 状态

**Accepted**

> 决策日期：2026-06-19
> 范围：sieve daemon 对经过的 LLM 流量做可选的独立 token 核算，按上游信任分级交叉比对上游声明的 `usage`，偏差超容差报警；统计严格本地、永不上传
> 关联：[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（绝不联网做 verifier）/ [ADR-018](./ADR-018-openai-protocol-adaptation.md)（双协议）/ [ADR-026](./ADR-026-port-based-listener-routing.md)（per-port 路由 + provider_id）/ [ADR-033](./ADR-033-upstream-proxy.md)（上游可经 relay）/ [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)（never upload 隐私承诺）/ [SPEC-010](../specs/SPEC-010-overbilling-detection.md)（工程级详细设计，Phase 2）

---

## 背景

经过 Sieve 的流量，上游返回的 `usage` 字段（`input_tokens` / `output_tokens` 等）有可能与实际 token 消耗存在偏差。Sieve 作为夹在 agent 与上游之间的代理，具备独立核算的结构性优势——两个方向的原始字节都经过 Sieve。

**目标不是「逐 token 精确对账」，而是「超额计费异常检测」。** 系统性虚报（如比例性放大）会远超 tokenizer 噪声，独立计数用于捕获此类异常，不追求逐字节相等。

当前代码库无任何 token 计数 / usage 解析逻辑。[ADR-026](./ADR-026-port-based-listener-routing.md) 的 per-port 路由 + `provider_id` 已能区分「这个 listener 指向谁」，上游信任分级以此为前提。

---

## 决策

### 1. 上游信任分级

引入上游信任标记，区分两类：

| 信任级 | 判定 | `usage` 处理 |
|--------|------|-------------|
| `official`（官方直连） | 上游 host 为已知官方 endpoint（可配置扩展） | `usage` **权威**，直接采纳，不必自己算 |
| `relay`（经中转） | 其余所有上游 | `usage` 视为**未经验证的声明**，独立核算 + 交叉比对，偏差超容差**报警** |

信任级在 `[[upstream]]` 上**按 host 自动派生**，可显式 `trust = "official" | "relay"` 覆盖（保守默认：无法判定时按 `relay` 处理，fail-closed 倾向）。复用 [ADR-026](./ADR-026-port-based-listener-routing.md) 的 `provider_id` 做审计归因。

### 2. 独立计数：优先权威信源，不手搓 tokenizer

#### 输入侧

- **OpenAI** → 用公开 tokenizer 库（标准 crate，不手搓 BPE），把框架开销也算进去，接近精确。
- **Anthropic** → Claude 无公开 tokenizer。两种计数来源，二选一：
  - **(默认) 本地近似估算**：明确标为估算，零新增出站。
  - **(opt-in) 调官方 count_tokens 直连（绕过 relay）**：独立开关 `count_tokens_optin`，默认 `false`；启用时仅打官方 endpoint、请求体为脱敏后内容。具体算法不随本文档公开发布。

#### 输出侧

- **OpenAI** → tokenizer 数已代理的补全文本，接近精确。
- **Anthropic** → 无公开 tokenizer，只能**近似估算**（明确标为估算）。

#### 计费比对

独立计数 × 本地内置官方公开单价 = 应收成本，跟上游声明比对。偏差超过 `tolerance_pct`（具体阈值不随本文档公开发布）→ 报警（StatusBar 通知 + 写本地 usage 记录，**不阻断流量**——这是计费监督，不是安全拦截，不引入新 Block 路径）。

#### 流式

`usage` 在最后一个事件里（OpenAI 流式需开 `include_usage`）；独立计数时**累计完整文本再 tokenize**。读上游声明的 usage 无需改 SSE parser；聚合完整补全文本走 `sieve_core::tool_use_aggregator`（**注意：模块名是 `tool_use_aggregator`，非 `sse::aggregator`**）。

被拦下、没发到上游的请求自然无 `usage`，这是**可接受的小缺口**。

### 3. 实现落点

- **默认全关**：整个特性可配置、默认全关。`[billing_check].enabled` 默认 `false`、`count_tokens_optin` 默认 `false`——不开启则零行为变化、零新增出站、零计算开销。
- 核算挂在 pipeline **响应完成后**的观测节点，**off the hot path**（fire-and-forget，不阻塞转发，与现有审计 fire-and-forget 模式对齐）。
- 仅当上游为 `relay` 且 `[billing_check].enabled = true` 时启用；`official` 直连不核算（`usage` 权威）。
- tokenizer vocab 文件需随二进制分发或首启缓存——评估打包进二进制（避免运行时下载）。

### 4. 隐私红线（不可放宽）

**统计数据严格本地、永不上传**，只在本地 GUI / CLI 可见。本 ADR 呼应 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) 的「禁止上传」表格。

统计落本地 SQLite（`~/.sieve/usage.db`，0600，append-only，结构上独立于 `audit.db`——token 用量是新数据域，但与 `events` 经 `request_id` 关联，不污染审计模型）。无任何出站上报路径。

### 5. 核心张力：`count_tokens` 直连与本地优先原则（已裁定）

[ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 的核心约束：「任何外部 token / 签名 / 规则的远端校验都摧毁产品定位」，唯一允许出站为用户主动调的上游 LLM 等。

Anthropic 的 `count_tokens` 直连字面上是「联网做 token 的远端校验」——这是本特性最大的设计张力。

**已裁定（2026-06-19）：默认姿态完全遵守 ADR-003**——Anthropic 输入侧默认只用本地近似估算，零新增出站；`count_tokens` 直连作为独立开关 `count_tokens_optin`，默认 `false`，只有用户显式开启才生效；启用时 config / UI 必须显著警示「这会向官方 endpoint 发起一次 Sieve 主动出站」。

### 6. 未来能力方向（本次 out of scope）

两个相邻能力方向，本次**不实现**，登记备查：

1. **模型偷换检测**：验证实际提供服务的模型与声明模型一致（需响应指纹 / 质量探针，更难）。
2. **缓存状态核查**：`usage` 里有 `cache_read_input_tokens` / `cache_creation_input_tokens`，可与预期缓存行为比对。

---

## 硬约束分析

- **[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（绝不联网做 verifier）**：已裁定——默认不联网（本地 tokenizer / 近似估算，零新增出站）；`count_tokens` 直连为独立开关 `count_tokens_optin`，默认关。✔
- **不在协议层撒谎，不伪造 usage**：Sieve **只读 / 只比对** `usage`，**绝不改写**上游的 `usage` 字段（即使检出异常也只报警，不动协议）。✔
- **never upload**：统计严格本地，无任何上报路径。✔ [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) 守护。
- **保守起步，不引入新 Block 路径**：计费异常仅 StatusBar 报警，不阻断流量。✔

---

## 影响

### 正面影响

- 利用 Sieve 作为代理的结构性优势（双向原始字节），独立核算上游声明，不依赖上游自报；
- 信任分级让官方直连零开销（`usage` 权威）、只对非官方上游付计算代价；
- 统计本地可见，给用户流量成本透明度。

### 负面影响

- **默认姿态下 Anthropic 输入核算只能近似**（`count_tokens` 默认关）；
- 新增 tokenizer 库依赖（+ vocab 文件打包，增二进制体积），过 cargo-deny + pin；
- Anthropic 输出侧无公开 tokenizer，只能近似估算——必须明确标注「估算」；
- 价表内置 = 需随单价变动维护；
- 流式被拦下的请求无 `usage`，留小缺口（可接受）。

### 需要更新的文档

- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行（P0）
- [data-model.md](./data-model.md)：新增「§N token 用量核算」——`usage.db` schema、独立计数记录结构、信任级派生、价表结构；`[[upstream]] trust` 字段（P0）
- [api-reference.md](../api/api-reference.md) §3 config schema：新增 `[billing_check]` 段（`enabled` / `tolerance_pct` / `count_tokens_optin`）+ `[[upstream]].trust` 字段；§2 新增 `sieve usage` 查询子命令（P1）
- [glossary.md](./glossary.md)：新增术语 `超额计费检测 / overbilling detection` / `信任分级（official / relay）` / `独立 token 核算`（P0）
- 新建 [SPEC-010-overbilling-detection.md](../specs/SPEC-010-overbilling-detection.md)：算法规格、流式累计 tokenize、信任级状态机、容差比对、测试矩阵 + [specs/INDEX.md](../specs/INDEX.md) 加行（P0，Phase 2）
- [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)：交叉引用「usage 统计严格本地」呼应 never-upload（P1）
- [CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Added 条目（P1）
- **回归测试硬约束**：「上游虚报（比例性放大 / 加常数）被检出」样本测试 + 「usage 永不上传」断言（Phase 2 必带）

---

## 相关文档

- [SPEC-010: 超额计费检测](../specs/SPEC-010-overbilling-detection.md)（Phase 2 落地）
- [ADR-003: 完全本地运行，绝不联网做 verifier](./ADR-003-local-only-no-cloud-verifier.md)
- [ADR-026: Port-based listener routing](./ADR-026-port-based-listener-routing.md)
- [ADR-033: 上游转发代理支持](./ADR-033-upstream-proxy.md)
- [SPEC-006: 更新通道 + 装机遥测（never upload 承诺）](../specs/SPEC-006-update-and-telemetry.md)
