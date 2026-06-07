# ADR-033: 上游转发代理支持（HTTP CONNECT + SOCKS5）

## 状态
**Accepted**
> 决策日期：2026-06-07
> 范围：sieve daemon 转发上游 LLM 流量 + sieve-updater 出站请求经配置代理出网
> 关联：[ADR-018](./ADR-018-openai-protocol-adaptation.md)（双协议）/ [ADR-026](./ADR-026-port-based-listener-routing.md)（multi-listener）/ [ADR-027](./ADR-027-network-jail-enforcement.md)（network jail，区分系统代理）/ [ADR-030](./ADR-030-update-telemetry-channel.md)（updater 复用）/ [SPEC-007](../specs/SPEC-007-upstream-proxy.md)（工程级详细设计）
> 关联 PRD：[v2.0 §6.1](../prd/sieve-prd-v2.0.md)、§9 #2、§9 #12

## 背景

`Forwarder`（`crates/sieve-core/src/forwarder/mod.rs`）用 hyper-util legacy `Client` + `HttpsConnector` **硬直连**上游，不读 `HTTP_PROXY` / `ALL_PROXY`，config 也无 proxy 字段。在「不挂代理连不上 LLM」的受限网络下（大量 crypto 开发者所处环境：规则代理 + 分流，而非全局 TUN 透明代理）：

```
agent → sieve(本地) → 【直连 api.anthropic.com 失败】
```

唯一现状解法是开全局 TUN 透明代理劫持 sieve 直连——不稳定、并非所有用户可用。结果：受限网络用户开箱即用不了，dogfood 第一跳即断。这是产品在限制性网络环境下的核心缺口。

## 决策

sieve **主动支持配置的上游代理**，不依赖系统透明代理：

1. daemon 转发上游可经配置的 **HTTP CONNECT** 或 **SOCKS5** 代理出网，按 proxy URL scheme 自动选择实现（`http://` → CONNECT，`socks5://` / `socks5h://` → SOCKS5）。
2. 代理可在**每 upstream** 单独配置（`[[upstream]].proxy`）或显式直连（`no_proxy = true`），并有**全局**（顶层 `proxy`）+ **env**（`HTTPS_PROXY` / `ALL_PROXY`）兜底。
3. 优先级链（高 → 低）：`upstream.no_proxy`(直连) > `upstream.proxy` > 全局 `proxy` > env(`HTTPS_PROXY` 优先于 `ALL_PROXY`) > 直连。
4. proxy URL 支持 `user:pass@` 认证；解析失败在启动期 config 校验 fail-fast，不静默忽略。
5. **updater**（`updates.sieveai.dev` manifest / `cdn.sieveai.dev` 规则正文）复用同一机制；daemon 用全局代理注入（[ADR-030](./ADR-030-update-telemetry-channel.md)）。
6. 代理连接失败（拒绝 / 超时 / 认证失败 / CONNECT 非 200 / SOCKS 握手失败）→ 返回明确错误，**绝不静默回退直连**——避免用户以为走代理实则直连，导致请求失败或在不该直连的网络里泄露目标。

实现：sieve-core 新增 `ProxyConnector`（实现 `tower::Service<Uri>`，按配置建立到 target 的 TCP 流），替换 `Forwarder` 底层的 `HttpConnector`；**TLS 仍由 hyper-rustls 在隧道之上做**——端到端到上游，代理只见密文（不解密、不装 CA、不 MITM）。详见 [SPEC-007](../specs/SPEC-007-upstream-proxy.md)。

### 硬约束分析（必须逐条成立才接受）

- **PRD §9 #2（唯一允许出站 / 绝不联网做 verifier）**：代理是**传输层隧道**，出站**目的地不变**（仍仅上游 LLM / `sieveai.dev`）。代理本身是**用户自己配置的可信出口**（推荐本地 Shadowrocket / Clash），不构成「联网做 token / 签名 / 规则的远端校验」。✔ 不违反。
- **PRD §9 #12（不装本地 CA 做 MITM）**：TLS 端到端到上游，sieve **不解密、不注入 CA**；代理（即便是远程）仅见 SNI 目标，**不见 TLS 内容**（prompt / response / API key）。✔ 不违反。
- **与 [ADR-027](./ADR-027-network-jail-enforcement.md) 的区分**：ADR-027 承诺「**不修改系统 `HTTP_PROXY` / `HTTPS_PROXY` / 系统代理设置**」。本决策是 sieve **自身主动走配置的代理**（读 config `proxy` / `no_proxy` + 标准 env），**不碰系统设置**、不替用户修改环境，二者职责正交、互不冲突。

## 影响

### 正面影响
- 受限网络（规则代理 + 分流、非全局 TUN）用户开箱即可用，消除「第一跳直连即断」的产品缺口。
- 每 upstream 可独立配代理或强制直连，灵活适配「部分上游需代理、本地中转站直连」的混合拓扑。
- updater 复用同机制，受限网络下规则更新 / 装机遥测同样可用（不再被网络环境卡死）。
- env 兜底（`HTTPS_PROXY` / `ALL_PROXY`）给零配置便利，符合 Unix 惯例。

### 负面影响
- **隐私边界扩展**：经**远程**代理时，代理可见「你在连 `api.anthropic.com`」（SNI / 目标 IP），但**不可见** prompt / response / API key。文档须明确提示用户使用**可信代理**——推荐本地 Shadowrocket / Clash 出口，避免把目标元数据暴露给不可信远端。
- 新增依赖 `tokio-socks`（SOCKS5 握手，过 cargo-deny）；HTTP CONNECT 自写（控供应链，PRD §9 #6 pinned deps）。
- 代理失败不回退直连意味着代理不可用时请求会明确失败（这是有意的安全选择，非缺陷）。

### 需要更新的文档
- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行（已完成）
- [docs/api/api-reference.md](../api/api-reference.md) §3 config：`proxy` / `[[upstream]].proxy` / `no_proxy` 字段 + 优先级链 + URL 格式
- [docs/guides/deployment.md](../guides/deployment.md)：受限网络（Shadowrocket / Clash）部署小节
- [docs/changelog/CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Added 条目
- [SPEC-007](../specs/SPEC-007-upstream-proxy.md) 状态升 Stable + [specs/INDEX.md](../specs/INDEX.md) 同步

## 相关文档
- [SPEC-007: 上游转发代理支持](../specs/SPEC-007-upstream-proxy.md)
- [ADR-018: OpenAI 协议适配](./ADR-018-openai-protocol-adaptation.md)
- [ADR-026: Port-based listener routing](./ADR-026-port-based-listener-routing.md)
- [ADR-027: Network jail enforcement](./ADR-027-network-jail-enforcement.md)
- [ADR-030: 更新通道复用为遥测信标](./ADR-030-update-telemetry-channel.md)
