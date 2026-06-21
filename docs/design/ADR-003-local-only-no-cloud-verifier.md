# ADR-003: 完全本地运行，绝不联网做 token verifier

## 状态

**Accepted**（v1.4 锁定）—— **2026-05-05 部分修订（更新通道遥测）**：

- **保留不变**：「token verifier 不联网」核心决策永久性。所有 secret / 私钥 / 助记词 / 上下文检测仍纯本地完成，不发外部 host 做 active validity check。
- **本次修订**：「绝对禁止 telemetry」反模式表条款被修订为「**仅允许复用规则更新通道做匿名 install 统计**」（每天 4 次，默认开启，三个环境变量可关闭）。原条款"telemetry 自动上报"被狭义化为"独立心跳通道"。
- **保留不变**：仍**禁止**任何上传 prompt / response / API key / 使用记录 / 设备序列号 / 账号信息。

> 原决策日期：2026-04-26
> 修订日期：2026-05-05（部分修订，见上）
> 范围：Sieve 全产品周期，所有版本

---

## 背景

Sieve 的核心检测能力之一是识别敏感 token / secret（API key / 私钥 / JWT 等）。业界一种常见的检测增强手段是 **active validity check**——把疑似 token 发到对应服务（如 OpenAI `/v1/models`）做一次轻请求，根据响应判断"这个 key 是不是真有效"。GitHub secret scanning 的"validity"标签就是这个机制。

这种做法的工程诱惑很大：

- 把 entropy + 前缀 + checksum 的"概率匹配"升级为"确定性匹配"；
- FP 几乎降为 0；
- Recall 接近 100%；
- 实现简单（一次 HTTP 请求）。

但对 Sieve 来说，**这个诱惑是产品定位的反命题**。Sieve 的核心定位：

1. **上游不可信**：你用的中转站可能在改你的 tool_call
2. **没人能替你兜底**：钱包安全产品看不见你的 prompt
3. **Sieve 在客户端最后一道闸**：完全本地运行，字节流双向扫描，**从不上传你的数据**
4. **你不只是相信我们，你能验证我们**：开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志

第三条是产品的物理边界，第四条是产品的可验证性承诺。**任何形式的"把 token 发出去验证一下"都直接摧毁这两条**。

Sieve 是面向对数据出境极度敏感的用户群（如 crypto-native 开发者）的客户端本地产品，"完全本地零上传"是核心信任承诺。

## 决策

**所有检测纯本地。Sieve 任何分支都不会向 Anthropic 以外的 host 发出请求。**

具体边界如下：

### 必须做的

1. **检测全部本地完成**：vectorscan + serde_json + entropy + checksum，全部在 Rust 进程内闭环；
2. **规则文件单向下载**：每天 4 次（每 6h 一次）从更新服务器拉 manifest，按 sha256 + ed25519 校验后从 CDN 拉规则包，**只下载，不上传 prompt/response/key**；签名验证用编译期硬编码 Ed25519 公钥（详见 [data-model.md §7](./data-model.md)）；
3. **license 验证完全离线**：用本地公钥校验 JWT-like 签名 + exp 校验 + 设备绑定本地存储，**不联网 verify**（详见 [data-model.md §8](./data-model.md)）；
4. **审计日志只存本地**：`~/.sieve/audit.db`，append-only，**只存 fingerprint + 元信息，不存原始 prompt**（详见 [data-model.md §6](./data-model.md)）；
5. **崩溃报告 / 错误上报**：Phase 1 **完全不做**——任何错误信息都打到本地日志，用户主动复制到 GitHub issue。

### 绝对禁止（反模式）

| 反模式 | 为什么禁止 |
|--------|----------|
| ❌ **远程查询 token 是否有效**（active validity check） | 把用户的真实 secret 发到第三方 host = 摧毁产品定位 |
| ❌ **远程白名单查询**（"这个地址是不是已知协议合约"） | 即使只发地址而非 prompt，也会泄漏用户的 on-chain 行为；Phase 2 引入 Drainer 黑名单时只能用**本地副本**模式 |
| ❌ **独立 telemetry 心跳通道**（专门的 phone home 端点）<br>⚠️ **2026-05-05 修订**：原条款禁止「任何 telemetry，哪怕匿名 + opt-out」。现修订为：**仅禁止独立心跳通道**；允许复用规则更新通道附带匿名 `install_id`（UUIDv4，本地随机生成，无设备/账号绑定）做匿名 install 统计，**默认开启，三个 env var 可关闭**（`SIEVE_NO_TELEMETRY` 仅省略 uid 字段、`SIEVE_NO_UPDATE` 完全跳过更新检查、`SIEVE_UPDATE_URL` 覆盖更新源）。技术约束三道：**复用更新通道** + **匿名 UUID** + **可关闭**。 | 独立心跳仍打穿信任壁垒；但**复用更新通道** + **匿名 UUID** + **可关闭** 三道约束下，验证者用 tcpdump 仍能完整审视上传内容（仅 `v / os / arch / uid / ch`，无 prompt/key/usage） |
| ❌ **崩溃报告自动上报**（Sentry / Bugsnag 类） | 同 token verifier 论证;不做 |
| ❌ **"匿名 fingerprint 用于优化规则"**（即上传命中规则的统计） | 即便匿名,上传命中即泄漏用户实际触发了哪些 secret/地址类型;与更新通道附带的 install id（仅匿名 install 统计,不含命中信息）相互独立 |
| ⚠️ **`telemetry_enabled` 配置项**<br>**2026-05-05 修订**：原条款要求 `config.toml` `telemetry_enabled` **强制 false**。现修订为：`[update]` 段允许 `telemetry = true`（默认开启，可在 toml 或 env var `SIEVE_NO_TELEMETRY` 关闭）。 | 仅在三个 env var 约束 + 隐私文案明示前提下放开 |
| ❌ **license server / activation server** | license 完全离线验证，没有 server 端，自然没有 phone home |
| ⚠️ **规则更新检查携带客户端信息**<br>**2026-05-05 修订**：原条款规定「拉规则包是 plain GET，**不带 user agent 之外的任何 header**，URL 不带 query 参数」。现修订为：manifest URL 允许带 `?v=&os=&arch=&uid=&ch=` 5 个 query 参数 + UA `sieve-updater/<v>`；不带 cookie / Auth header / Referer / 详细系统版本。 | 5 字段是**匿名 install 统计的最小集**（uid 可关），签名校验在客户端，不影响规则正文链路 |

### 唯一允许的出站请求

Sieve 进程在运行期**仅**会向以下 host 发起出站请求（**2026-05-05 修订**：新增 manifest + cdn 两个 host）：

| Host | 用途 | 何时发起 |
|------|------|---------|
| `api.anthropic.com` 或 `config.upstream_url`（multi-listener 时是 `cfg.resolved_upstreams()` 各项 url，ADR-026）| 转发 Claude Code / OpenClaw / Hermes 的请求 | 用户主动调用 AI agent 时 |
| `updates.sieveai.dev`（域名占位，GA 前确定）| 拉规则更新 manifest（默认每 6h 一次，带 `?v=&os=&arch=&uid=&ch=` query；manifest 接口**不挂 CDN**，日志能追溯每次请求）| 启动时 + 6h 周期触发，即使内容无变化也照常发请求（附带匿名 install 统计） |
| `cdn.sieveai.dev`（域名占位）| 下载规则正文 zst 包（带 sha256 + ed25519 签名）| manifest 返回新版本时 |

**没有第四个**。任何 PR 引入新的出站 host 调用，CI 必须 hard-fail（用 `cargo deny` + 自定义 lint 检查）。

**用户审视入口**（不变）：所有上述出站均可用 tcpdump / mitmproxy 抓包审视。manifest 请求最多包含 5 字段（`v / os / arch / uid / ch`）+ 标准 TLS 1.2 握手，**永远不含 prompt / response / API key / 用户输入任何字节**。`SIEVE_NO_UPDATE=1` 完全禁用更新检查（含 manifest）；`SIEVE_NO_TELEMETRY=1` 仅省略 uid 字段（其他 4 字段保留）；`SIEVE_UPDATE_URL=...` 改用企业自托管镜像。

---

## 影响

### 正面影响

1. **信任壁垒可验证**（**2026-05-05 修订**：从「没有 phone home」精确化为「phone home 内容可审视且边界明确」）：用户用 `tcpdump` / `mitmproxy` 可以独立验证 Sieve 上传内容**最多包含 5 字段**（`v / os / arch / uid / ch`）且永远不含 prompt / response / API key / 使用记录；并且可用三个 env var（`SIEVE_NO_UPDATE` / `SIEVE_NO_TELEMETRY` / `SIEVE_UPDATE_URL`）一键关闭或改向企业镜像。与 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 配合形成「可验证 + 可审视 + 可关闭」三道信任叙事；
2. **合规边界清晰**：检测全本地 + 不上传 prompt → 不触发数据出境合规；规则库下发是单向静态文件 + 签名 → 不构成"个人信息处理"；匿名 UUID 不绑定账号 / 设备序列号 / IP 长存，服务端 geoip 解析后丢弃原始 IP；
3. **离线可用**：用户在飞机 / 没网环境下，Sieve 仍然完整工作（除了 Anthropic 转发本身需要网络；`SIEVE_NO_UPDATE=1` 自然降级为规则冻结状态）；
4. **运维负担降到最低**：不维护 license server / crash report backend；manifest + CDN 是唯一新增运维点（边缘 Workers 兜底，日志只 7 天）；无 token verifier、无崩溃上报、无命中统计——仍是"工程上的强制简化"；
5. **抗供应链审视**：用户可以用 Wireshark 抓包验证。Sieve 自己被同一套标准审视。**2026-05-05 修订后该承诺明确为**："你能验证 Sieve 上传内容的边界，且可一键关闭"。

### 负面影响

1. **失去 active validity check 能力**：OUT-01 OpenAI key、OUT-02 AWS key 等的 FP 完全靠 entropy + 上下文判断，比 GitHub secret scanning 弱；通过更严格的前缀匹配 + 占位符黑名单缓解；
2. **失去 Drainer 黑名单实时性**：Phase 2 引入 Chainabuse / ScamSniffer 时只能用本地副本（每天/每小时拉一次），延迟 7 天～几小时不等；这是产品定位换来的代价，不可逆；
3. **失去 anonymized analytics**：无法获得规则命中统计（哪些规则被触发、哪条 FP 最频繁）；依赖用户主动反馈（如 GitHub issue）；
4. **license 撤销不可能**：license 完全离线验证，没有 server 端，因此无法远程撤销 —— Phase 1 直接放弃这个能力（依靠 exp + 设备绑定 + 法律 ToS）。

### 需要更新的文档

- [data-model.md](./data-model.md) §5、§6、§8 —— 配置 / 审计 / license 三处已对齐离线优先
- [architecture.md](./architecture.md) §1.2 数据流图 —— 没有任何分支指向第三方 host
- `docs/guides/deployment.md`（待编写）—— 写明"如何用 mitmproxy 验证 Sieve 没 phone home"作为信任叙事的工具

---

## 相关文档

- [architecture.md](./architecture.md) —— 整体架构与数据流
- [data-model.md](./data-model.md) —— 配置、审计、license 数据格式
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— 可验证性的另一支柱
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical 永不可关（与"不上传"共同构成产品安全承诺）
