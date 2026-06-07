# SPEC-007: 上游转发代理支持

> Version: v0.1 — 2026-06-07
> 状态：Stable（已实现并验证）
> 关联：ADR-018（双协议）、ADR-026（multi-listener）、ADR-027（network jail，区分系统代理）、ADR-030（updater）、PRD §9 #2/#12
> 决策记录：本 spec §9（实现时固化为 ADR-033）

---

## §0 文档定位

本 spec 定义 sieve daemon 转发上游 LLM 流量时经用户配置的 **HTTP CONNECT / SOCKS5 代理** 出网的机制。解决受限网络（Shadowrocket / Clash 等非透明代理）下 sieve 上游硬直连不可用的产品缺口。范围：`sieve-core::Forwarder` 上游连接层 + `sieve-cli` config schema + `sieve-updater` 出站请求。

## §1 背景与目标

### 痛点
`Forwarder::new`（`crates/sieve-core/src/forwarder/mod.rs`）用 hyper-util legacy `Client` + `HttpsConnector` **硬直连**上游，不读 `HTTP_PROXY`/`ALL_PROXY`，config 无 proxy 字段。在「不挂代理连不上 LLM」的网络（大量 crypto 开发者所处环境：规则代理 + 分流，非全局 TUN）下：

```
agent → sieve(本地) → 【直连 api.anthropic.com 失败】
```

唯一现状解法是开全局 TUN 透明代理劫持 sieve 直连——不稳定、并非所有用户可用。结果：受限网络用户开箱即用不了；dogfood 第一跳即断。

### 目标
1. daemon 转发上游可经配置的 HTTP CONNECT / SOCKS5 代理出网
2. 每 upstream 可独立配代理或显式直连，全局 + env 兜底
3. updater（updates/cdn.sieveai.dev）复用同机制
4. 端到端 TLS 不变（代理只见密文，不 MITM）

## §2 配置 schema

```toml
# 全局兜底代理（可选）
proxy = "socks5://127.0.0.1:6153"

[[upstream]]
url = "https://api.anthropic.com"
protocol = "anthropic"
# 未写 proxy → 继承全局 proxy

[[upstream]]
url = "http://127.0.0.1:8080"   # 本地中转站
protocol = "openai"
no_proxy = true                  # 显式直连，无视全局/env
```

字段：
- 顶层 `proxy: Option<String>`：全局兜底代理 URL
- `UpstreamListener.proxy: Option<String>`：该 listener 专属代理（覆盖全局）
- `UpstreamListener.no_proxy: bool`（默认 false）：显式直连，优先级最高

### 优先级链（高 → 低）
1. `upstream.no_proxy = true` → 直连
2. `upstream.proxy` → 用之
3. 全局 `proxy` → 用之
4. env `ALL_PROXY` / `HTTPS_PROXY`（标准惯例，`NO_PROXY` 例外列表生效）
5. 直连

> 显式 config 优先于 env；env 是零配置便利兜底。`no_proxy=true` 可在任意层级强制直连。

## §3 proxy URL 格式

- scheme：`http://` / `socks5://` / `socks5h://`（h = 远程 DNS，代理侧解析）
- 认证：`scheme://user:pass@host:port`（本地代理通常无需；远程代理用）
- 解析失败 → 启动期 config 校验报错（fail-fast，不静默忽略）

## §4 架构（connector 层）

```
target api.anthropic.com:443
 → ProxyConnector 按配置建立到 target 的 TCP 流：
     Direct  : TcpStream::connect(target)
     Http    : TcpStream::connect(proxy) → 发 CONNECT target:443 → 读 200 → 隧道
     Socks5  : tokio_socks::Socks5Stream::connect(proxy, target)
 → hyper-rustls HttpsConnector 在该 TCP 之上做 TLS(SNI=target) + ALPN(h2/http1.1)
 → hyper Client 发请求
```

实现：把现有 `HttpsConnectorBuilder::new()…wrap_connector(HttpConnector)` 的底层 `HttpConnector` 换成自定义 `ProxyConnector`（实现 `tower::Service<Uri>`，返回到 target 的 TCP stream）。**TLS 仍由 hyper-rustls 在隧道之上做——端到端到上游，代理只见密文**（不解密、不装 CA，符合 PRD §9 #12）。

`Forwarder::new` 签名增加 proxy 参数（解析后的 `ProxyConfig` enum：`Direct | Http(url) | Socks5(url)`）。

## §5 协议实现与依赖

- **SOCKS5**：`tokio-socks`（成熟、tokio 原生；自写 SOCKS5 握手易错）。新增依赖，过 cargo-deny。
- **HTTP CONNECT**：自写（~50 行：发 `CONNECT host:port HTTP/1.1` + 可选 `Proxy-Authorization: Basic` + 读状态行至 `200` + 透传），不引第三方 HTTP-proxy crate（控供应链，PRD §9 #6 pinned deps）。
- 连接超时复用现有上游超时配置。

## §6 错误处理

- 代理连接失败（拒绝 / 超时 / 认证失败 / CONNECT 非 200 / SOCKS 握手失败）→ 返回明确错误，**绝不静默回退直连**（避免用户以为走代理实则直连，导致请求失败或在不该直连的网络里泄露目标）。
- 日志记录代理 host:port，**脱去密码**。
- 上游 HTTP 错误码语义不变（代理仅传输层）。

## §7 updater 复用

`sieve-updater` 的 manifest（updates.sieveai.dev）与规则下载（cdn.sieveai.dev）请求复用同一 `ProxyConfig`：
- 至少支持 env `ALL_PROXY`/`HTTPS_PROXY` + 全局 config `proxy`
- 否则受限网络下更新检查 / 装机遥测同样不可用

## §8 测试矩阵

| 用例 | 验证 |
|---|---|
| proxy URL 解析 | scheme / auth / 非法 URL fail-fast |
| 优先级链 | upstream.proxy > 全局 > env；no_proxy 强制直连 |
| HTTP CONNECT 连通 | mock CONNECT 代理 + mock 上游，经隧道连通 200 |
| SOCKS5 连通 | mock SOCKS5 代理 + mock 上游，经隧道连通 200 |
| 代理认证 | user:pass 正确/错误（401/拒绝）|
| 代理失败不回退 | 代理拒绝 → 明确错误，上游未被直连 |
| no_proxy 直连 | 全局有 proxy 但 upstream no_proxy → 直连 mock 上游 |
| updater 经代理 | mock 代理 + mock manifest 端点连通 |

复用现有 `spawn_mock_upstream` 模式新增 mock 代理 harness。

## §9 决策记录（实现时固化为 ADR-033）

**决策**：sieve 主动支持配置的上游代理（HTTP CONNECT + SOCKS5），不依赖系统透明代理。

**硬约束分析**：
- **PRD §9 #2（唯一允许出站）**：代理是传输层隧道，出站**目的地不变**（仍仅上游 LLM / sieveai.dev）。代理本身是用户自己配置的本地/可信出口，不构成「联网做 verifier」。✔
- **PRD §9 #12（不装本地 CA 做 MITM）**：TLS 端到端到上游，sieve 不解密、不注入 CA；代理（若远程）仅见 SNI 目标、不见 TLS 内容。✔
- **ADR-027 区分**：ADR-027 承诺「不修改系统 HTTP_PROXY / 系统代理设置」。本 spec 是 sieve **自身主动走配置的代理**，不碰系统设置，二者不冲突。
- **隐私提示**：经远程代理时代理可见「你在连 api.anthropic.com」（SNI/目标 IP），不可见 prompt/response。文档须提示用户使用可信代理（本地 Shadowrocket/Clash 出口为佳）。

## §10 文档同步清单（实现时）

- 新增 `docs/design/ADR-033-upstream-proxy.md` + `docs/design/ADR-INDEX.md` 加行
- `docs/api/api-reference.md` §3 config：proxy / upstream.proxy / no_proxy 字段
- `docs/guides/deployment.md`：受限网络（Shadowrocket/Clash）部署章
- `docs/changelog/CHANGELOG.md`
- `crates/*/CLAUDE.md` / `.cursorrules` 如涉及 crate 边界（sieve-core 新增 proxy connector）

## §11 变更记录

- v0.1（2026-06-07）：初稿，brainstorming 评审通过（HTTP+SOCKS5 按 scheme / 每 upstream+全局+env / updater 纳入 / CONNECT 自写 / 不静默回退）。
