# Sieve API 参考

> **状态：设计阶段（Pre-Code），接口未冻结。**
> 当前文档反映 PRD v1.3 的设计意图，**v1 接口将在 Week 12 GA 时冻结**，破坏性变更走 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)。
> 在冻结前，任何字段、状态码、配置项的细节都可能调整。

---

## 0. 文档定位

Sieve **不是** 面向开发者的 SDK / library，而是一个本地 HTTP 反向代理，因此本文所述的"API"分为三层：

1. **反向代理 API**（对 Claude Code 等上游客户端暴露） —— 透明转发 Anthropic Messages 协议
2. **本地管理 API**（仅 `127.0.0.1`） —— 健康检查、白名单管理、审计查询、规则刷新
3. **配置文件 schema + 环境变量 + CLI 退出码** —— 二进制运行时契约

> **不存在云端 API。**Sieve 完全本地运行，绝不联网做 token verification（[PRD §9](../prd/sieve-prd-v1.3.md) 硬约束 #2）。

---

## 1. 反向代理端点（对 Claude Code）

### 1.1 监听地址


| 项           | 值                                            |
| ----------- | -------------------------------------------- |
| 协议          | `http://`（**明文，仅本地回环**）                      |
| 默认端口        | `11453`                                      |
| 默认绑定地址      | `127.0.0.1`（**强制，配置层禁止改成 `0.0.0.0` 或公网 IP**） |
| 完整 base URL | `http://127.0.0.1:11453`                     |


### 1.2 用户侧接入方式

```bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:11453
export ANTHROPIC_AUTH_TOKEN=<your-real-anthropic-or-router-key>
```

> Sieve 是中间层，**不替代** 用户原有的 Anthropic 官方 key 或中转站 key —— 透传到上游需要原 key。

### 1.3 透明转发的 Anthropic 路由清单


| 方法     | 路径                          | 说明                                                         |
| ------ | --------------------------- | ---------------------------------------------------------- |
| `POST` | `/v1/messages`              | 主要消息 API，**含 SSE 流**；Sieve 在请求体（出站）和 SSE chunk 边界（入站）做注入检测 |
| `POST` | `/v1/messages/count_tokens` | token 计数，无流式；只做出站规则扫描                                      |
| `GET`  | `/v1/models`                | 模型清单，纯透传                                                   |
| `*`    | `/v1/...`（其他 Anthropic 路由）  | **Phase 1 默认 501 Not Implemented**，避免静默放过未审计协议             |


### 1.4 协议兼容性承诺

- **100% 透明转发** —— 不修改请求 / 响应字段，包括 header、`metadata`、`tool_use` 结构、SSE event 类型与顺序
- **不重命名、不重排、不补字段** —— 出站完全旁路（仅扫描）；入站仅在命中 Critical 时改写流
- 上游真实 endpoint 由配置 `[upstream].url` 决定（默认 `https://api.anthropic.com`），用户可指向中转站

### 1.5 行为差异（命中检测时）

参见 §5《处置矩阵 → HTTP 行为》。**唯一的非透传场景**：

- **出站 Critical**：返回 `426 Upgrade Required` + Sieve 自定义 JSON body，**不向上游发出请求**
- **入站 Critical**（SSE 流中检测到危险 tool_use）：在 SSE 流中插入 Sieve 自定义 event，并阻断后续 chunk，等待用户 CLI 弹窗确认结果

### 1.6 Sieve 自定义 JSON Body 格式

出站 Critical 拦截时返回的 body：

```json
{
  "type": "sieve_block",
  "severity": "critical",
  "rule_id": "OUT-09",
  "rule_name": "BIP39 mnemonic with valid SHA-256 checksum",
  "fingerprint": "OUT-09:7a3b9c1d",
  "message": "Sieve detected a 12-word BIP39 mnemonic with a valid checksum. Outbound blocked.",
  "remediation": [
    "Replace the secret with [REDACTED-MNEMONIC] and re-send.",
    "If you believe this is a false positive, run `sieve sieveignore add OUT-09:7a3b9c1d`."
  ],
  "docs_url": "https://github.com/doskey/sieve/blob/main/docs/api/api-reference.md#5-处置矩阵--http-行为"
}
```

入站 Critical 在 SSE 流中的完整序列（fail-closed，PRD §9 #3）：

```
# 1) 命中 Critical 时，Sieve 暂停转发上游 chunk 并立即注入 sieve_block：
event: sieve_block
data: {"type":"sieve_block","severity":"critical","rule_id":"IN-CR-05","fingerprint":"IN-CR-05:f1c2a8b9","awaiting_confirmation":true,"timeout_s":30,"event_id":"evt_a1b2"}

# 2) 在用户 CLI 弹窗回应前，**不向客户端发送任何后续 chunk**（包括上游已 buffer 的 message_delta / message_stop）。
#    对客户端表现为：SSE 连接保持，但流暂停。

# 3a) 用户 approve（CLI 退出码 0）→ 解除阻断，按缓冲顺序补发被暂停的 chunk，并注入 sieve_resume：
event: sieve_resume
data: {"type":"sieve_resume","event_id":"evt_a1b2","user_decision":"approve","decided_at_ms":1745683210123}
event: content_block_delta
data: { ... 上游 buffer 中被暂停的 chunk ... }
event: message_stop
data: { ... }

# 3b) 用户 deny / timeout / interrupted → 终止流，注入 sieve_terminate 后 EOF：
event: sieve_terminate
data: {"type":"sieve_terminate","event_id":"evt_a1b2","user_decision":"deny","reason":"user_denied"}
# Sieve 关闭 SSE 连接，不再发送任何 chunk
```

要点：

- **buffer 上限**：等待用户确认期间上游 chunk 暂存到内存 buffer，超过 `[server].critical_buffer_bytes`（默认 256 KB）→ 主动断流并 fail-closed
- **每个被拦截事件都有唯一 `event_id`**：用户 CLI 决策、审计日志、`sieve_resume` / `sieve_terminate` 通过它关联
- `**user_decision` 取值**：`"approve"` / `"deny"` / `"timeout"` / `"interrupted"`（与 §6 CLI 退出码对应）
- **降级模式不影响入站 Critical**：试用过期用户的入站 Critical 仍然 fail-closed（PRD §9 #8 + §7.1）

---

## 2. Sieve 本地管理 API

### 2.1 通用约定


| 项        | 值                                                                                     |
| -------- | ------------------------------------------------------------------------------------- |
| Base URL | `http://127.0.0.1:11453` （与反向代理共用端口）                                                  |
| 路径前缀     | `/_sieve/v1/`                                                                         |
| 响应格式     | `application/json; charset=utf-8`                                                     |
| 鉴权       | **本地 socket 持有 = 默认信任**（操作系统 UID 隔离）；可在 `[server].management_token` 启用额外 Bearer Token |
| 与代理路由冲突  | `/_sieve/` 是保留前缀，不会与 `/v1/messages` 冲突；任何上游请求都不应使用此前缀                                 |


### 2.2 端点清单

#### 2.2.1 健康检查

```
GET /_sieve/v1/healthz
```

响应：

```json
{ "status": "ok", "uptime_s": 3812 }
```

#### 2.2.2 版本与构建信息

```
GET /_sieve/v1/version
```

响应：

```json
{
  "version": "0.1.0-pre",
  "rules_version": "2026-04-26.1",
  "rules_sha256": "9f8e7d6c...",
  "sigstore_bundle_url": "https://github.com/doskey/sieve/releases/download/v0.1.0/sieve-darwin-arm64.sigstore",
  "build": {
    "rustc": "1.80.0",
    "git_sha": "abcdef1234567890",
    "reproducible": true
  }
}
```

> `reproducible: true` 表示该二进制可由 [ADR-006-sigstore-reproducible-build](../design/ADR-006-sigstore-reproducible-build.md) 描述的流程复现。

#### 2.2.3 审计事件查询

```
GET /_sieve/v1/events?since=<unix_ms>&severity=<level>&limit=<N>
```


| 参数         | 类型            | 默认  | 说明                                         |
| ---------- | ------------- | --- | ------------------------------------------ |
| `since`    | int (unix ms) | 无   | 仅返回 `ts >= since` 的事件                      |
| `severity` | enum          | 无   | `critical` / `high` / `medium` / `low`，可重复 |
| `limit`    | int           | 100 | 上限 1000                                    |


响应（示例）：

```json
{
  "events": [
    {
      "id": 90217,
      "ts": 1745683200123,
      "severity": "critical",
      "direction": "outbound",
      "rule_id": "OUT-09",
      "fingerprint": "OUT-09:7a3b9c1d",
      "action": "blocked",
      "request_id": "req_a1b2c3",
      "model": "claude-sonnet-4-5",
      "user_decision": null
    }
  ],
  "next_since": 1745683200124
}
```

字段说明：


| 字段              | 类型    | 取值                                                                                                           |
| --------------- | ----- | ------------------------------------------------------------------------------------------------------------ |
| `severity`      | enum  | `critical` / `high` / `medium` / `low`                                                                       |
| `direction`     | enum  | `outbound`（出站，扫请求体） / `inbound`（入站，扫 SSE 流）                                                                  |
| `action`        | enum  | `blocked` / `warned` / `marked` / `silent`                                                                   |
| `user_decision` | enum? | 仅 Critical 入站事件有：`null`（未确认 / 不需确认） / `"approve"` / `"deny"` / `"timeout"` / `"interrupted"`（与 §6 CLI 退出码对应） |


> **绝不返回原文。**只返回 fingerprint + 元信息（[PRD §9](../prd/sieve-prd-v1.3.md) 硬约束 #2 + §11.3 数据本地化）。详细字段定义见 [data-model.md](../design/data-model.md)。

#### 2.2.4 白名单管理（`.sieveignore`）

加入：

```
POST /_sieve/v1/sieveignore/add
Content-Type: application/json

{ "fingerprint": "OUT-09:7a3b9c1d", "comment": "test mnemonic from unit-test fixture" }
```

响应：

```json
{ "added": true, "fingerprint": "OUT-09:7a3b9c1d", "added_at": 1745683210000 }
```

移除：

```
DELETE /_sieve/v1/sieveignore/{fingerprint}
```

响应：

```json
{ "removed": true, "fingerprint": "OUT-09:7a3b9c1d" }
```

列出：

```
GET /_sieve/v1/sieveignore
```

响应：

```json
{
  "entries": [
    { "fingerprint": "OUT-09:7a3b9c1d", "comment": "...", "added_at": 1745683210000 }
  ]
}
```

> fingerprint 格式 = `<rule_id>:<sha256_prefix_8_hex>`，例如 `OUT-09:7a3b9c1d`。`sha256_prefix` 取规则匹配内容 SHA-256 的前 **8 hex 字符（4 bytes）**——足以在单用户审计库内唯一标识，且不暴露原文。详见 [data-model.md](../design/data-model.md)。

#### 2.2.5 规则刷新

```
POST /_sieve/v1/rules/refresh
```

行为：

1. 从 `[rules_update].update_url` 下载最新规则包
2. **强制 Ed25519 签名验证**（公钥从 `[rules_update].signing_pubkey_path` 加载）
3. 验证通过后落盘到 `[detection].rules_path`
4. 热加载到运行时，旧规则保留一份用于回滚

响应：

```json
{
  "refreshed": true,
  "old_version": "2026-04-19.3",
  "new_version": "2026-04-26.1",
  "rules_sha256": "9f8e7d6c...",
  "verified_by": "ed25519:0x4F...A1"
}
```

签名验证失败：

```
HTTP/1.1 451 Unavailable For Legal Reasons
{
  "error": "signature_verification_failed",
  "message": "rules bundle signature does not match trusted Ed25519 public key; refused to load",
  "expected_pubkey": "ed25519:0x4F...A1"
}
```

> 签名失败 → **fail-closed**，不更新规则，沿用上一份已验证规则。

---

## 3. 配置文件 schema (`~/.sieve/config.toml`)

### 3.1 字段定义


| 段                | 字段                    | 类型      | 默认值                                              | 说明                                                              |
| ---------------- | --------------------- | ------- | ------------------------------------------------ | --------------------------------------------------------------- |
| `[server]`       | `port`                | u16     | `11453`                                          | 反向代理监听端口                                                        |
| `[server]`       | `bind_address`        | string  | `"127.0.0.1"`                                    | **强制 `127.0.0.1`**，写其他值启动失败（PRD §9 #2）                          |
| `[server]`       | `management_token`    | string? | `null`                                           | 本地管理 API 的可选 Bearer Token                                       |
| `[server]`       | `binary_fallback`     | bool    | `false`                                          | 启用 `~/.sieve/bin/sieve.prev` 一键回滚                               |
| `[upstream]`     | `url`                 | string  | `"https://api.anthropic.com"`                    | 上游真实 endpoint                                                   |
| `[upstream]`     | `timeout_ms`          | u32     | `120000`                                         | 整体请求超时                                                          |
| `[upstream]`     | `connect_timeout_ms`  | u32     | `5000`                                           | TCP 连接超时                                                        |
| `[upstream]`     | `retry`               | u8      | `0`                                              | **默认不重试**，避免重复执行带副作用的工具调用                                       |
| `[detection]`    | `rules_path`          | path    | `"~/.sieve/rules"`                               | 已签名规则目录                                                         |
| `[detection]`    | `sieveignore_path`    | path    | `"~/.sieve/.sieveignore"`                        | 本地白名单文件（**不上传仓库**）                                              |
| `[detection]`    | `severity_overrides`  | table   | `{}`                                             | 子表，按 `rule_id` 覆盖默认 severity（仅可降级；**Critical 不可关闭**）            |
| `[storage]`      | `audit_db_path`       | path    | `"~/.sieve/audit.db"`                            | SQLite append-only 审计库                                          |
| `[storage]`      | `log_path`            | path    | `"~/.sieve/logs/sieve.log"`                      | 文本日志，按天 rotate                                                  |
| `[storage]`      | `log_level`           | enum    | `"info"`                                         | `trace` / `debug` / `info` / `warn` / `error`                   |
| `[license]`      | `key`                 | string? | `null`                                           | License key；缺失时进入 14 天试用                                        |
| `[license]`      | `offline_grace_days`  | u16     | `30`                                             | 无网络时 license 缓存有效期                                              |
| `[rules_update]` | `enabled`             | bool    | `true`                                           | 关闭后等价 `SIEVE_DISABLE_RULES_UPDATE=1`                            |
| `[rules_update]` | `signing_pubkey_path` | path    | `"~/.sieve/keys/sieve-rules.pub"`                | Ed25519 公钥，**fail-closed**                                      |
| `[rules_update]` | `update_url`          | string  | `"https://updates.sieve.tools/v1/rules.tar.zst"` | 规则包下载地址                                                         |
| `[rules_update]` | `interval_hours`      | u16     | `168`                                            | 自动检查间隔（默认每周）                                                    |
| `[telemetry]`    | `enabled`             | bool    | `**false`（强制，不可改）**                              | **不存在任何 telemetry。**此字段保留仅为让用户在 config 中可视化确认；写 `true` 启动会拒绝并提示 |


### 3.2 `severity_overrides` 子表语义

**只允许降级，不允许升级。`**Critical` 项**禁止**降级（PRD §9 #8）。

```toml
[detection.severity_overrides]
"OUT-11" = "low"      # 默认 medium → 降级到 low（允许）
"IN-GEN-04" = "low"   # 默认 high → 降级到 low（允许）
"OUT-09" = "high"     # ❌ 启动失败：BIP39 默认 critical，禁止降级
```

### 3.3 完整示例 `config.toml`

```toml
# Sieve 配置文件
# 路径：~/.sieve/config.toml
# 文档：docs/api/api-reference.md §3

[server]
port = 11453
bind_address = "127.0.0.1"   # 强制本地回环，写其他值启动失败
# management_token = "随便一个长字符串以启用 Bearer 鉴权"
binary_fallback = false

[upstream]
url = "https://api.anthropic.com"
timeout_ms = 120000
connect_timeout_ms = 5000
retry = 0                     # 不重试：避免重复执行带副作用的 tool_use

[detection]
rules_path = "~/.sieve/rules"
sieveignore_path = "~/.sieve/.sieveignore"

  [detection.severity_overrides]
  # 仅可降级，Critical 不可改
  "OUT-11" = "low"
  "IN-GEN-04" = "low"

[storage]
audit_db_path = "~/.sieve/audit.db"
log_path = "~/.sieve/logs/sieve.log"
log_level = "info"

[license]
# key = "sieve-xxx-xxx-xxx"
offline_grace_days = 30

[rules_update]
enabled = true
signing_pubkey_path = "~/.sieve/keys/sieve-rules.pub"
update_url = "https://updates.sieve.tools/v1/rules.tar.zst"
interval_hours = 168

[telemetry]
enabled = false               # 强制 false，写 true 启动失败
```

---

## 4. 环境变量


| 变量                           | 适用方                  | 必需  | 默认                     | 说明                                                                |
| ---------------------------- | -------------------- | --- | ---------------------- | ----------------------------------------------------------------- |
| `ANTHROPIC_BASE_URL`         | **用户侧（Claude Code）** | 是   | 无                      | 指向 Sieve 监听地址，如 `http://127.0.0.1:11453`                          |
| `ANTHROPIC_AUTH_TOKEN`       | **用户侧（Claude Code）** | 是   | 无                      | 用户原 Anthropic / 中转站 key，由 Sieve 透传到上游                             |
| `SIEVE_CONFIG`               | sieve 进程             | 否   | `~/.sieve/config.toml` | 覆盖配置文件路径                                                          |
| `SIEVE_LICENSE_KEY`          | sieve 进程             | 否   | 无                      | 覆盖 `[license].key`                                                |
| `SIEVE_LOG_LEVEL`            | sieve 进程             | 否   | `info`                 | 覆盖 `[storage].log_level`，取值 `trace`/`debug`/`info`/`warn`/`error` |
| `SIEVE_RULES_PATH`           | sieve 进程             | 否   | `~/.sieve/rules`       | 覆盖 `[detection].rules_path`                                       |
| `SIEVE_DISABLE_RULES_UPDATE` | sieve 进程             | 否   | 未设置                    | 设为 `1` 时禁用规则自动更新（仅离线场景）                                           |


> 环境变量优先级 **高于** 配置文件，但 **低于** CLI flag（如有）。

---

## 5. 处置矩阵 → HTTP 行为


| Severity        | 检测点                 | 默认动作                          | 用户侧 HTTP 行为                                                     |
| --------------- | ------------------- | ----------------------------- | --------------------------------------------------------------- |
| 🚨 **Critical** | 出站                  | **阻断，不发上游**                   | `426 Upgrade Required` + Sieve `sieve_block` JSON               |
| 🚨 **Critical** | 入站工具调用 (`tool_use`) | **拦截 SSE 流**，注入 system 消息要求确认 | `200 OK`（SSE 已开启），但 SSE 流插入 `event: sieve_block` 后阻断；CLI 弹出确认窗口 |
| ⚠ **High**      | 出/入站                | 不阻断，注入警告                      | `200 OK`；SSE 流中插入 `event: sieve_warn` 事件并继续                     |
| 📋 **Medium**   | 出/入站                | 仅记录到审计库                       | `200 OK`；不修改流                                                   |
| ℹ **Low**       | 出/入站                | 静默                            | `200 OK`；不修改流                                                   |


> **Critical 在所有版本（含降级模式）不可关闭**（[PRD §5.3 / §9 #8](../prd/sieve-prd-v1.3.md)）。试用结束未付费用户进入 [PRD §7.1](../prd/sieve-prd-v1.3.md#71-单一定价) 的"只读警告"模式 —— High/Medium/Low 仅记录不阻断，但 Critical 始终阻断。

---

## 6. CLI 退出码 / 弹窗确认协议

Sieve 的 Critical 阻断需要用户在 CLI 弹窗回应。弹窗子进程的退出码语义：


| 退出码   | 含义                  | 后续动作                                                            |
| ----- | ------------------- | --------------------------------------------------------------- |
| `0`   | 用户确认放行              | 解除 SSE 阻断，继续转发；记录 `user_decision: "approve"`                    |
| `1`   | 用户拒绝                | 终止当前 SSE 流，向客户端返回 `sieve_block` 终止事件；记录 `user_decision: "deny"` |
| `2`   | 超时未响应（默认 30s）       | **fail-closed，按拒绝处理**（PRD §9 #3）；记录 `user_decision: "timeout"`  |
| `130` | `SIGINT`（用户 Ctrl-C） | 等价拒绝；记录 `user_decision: "interrupted"`                          |


> 任何**非零**且**非上述**退出码也按 fail-closed 处理（拒绝）。

---

## 7. 错误码表

### 7.1 标准 4xx / 5xx（Sieve 透传上游或自身产生）


| 状态码                         | 来源          | 含义                                      |
| --------------------------- | ----------- | --------------------------------------- |
| `400 Bad Request`           | 透传上游 / 配置错误 | 客户端请求语法错误                               |
| `401 Unauthorized`          | 透传上游        | 用户的 `ANTHROPIC_AUTH_TOKEN` 无效           |
| `403 Forbidden`             | 透传上游        | 上游拒绝（地区限制、key 权限不足）                     |
| `404 Not Found`             | Sieve       | 路径不在 §1.3 透传清单中                         |
| `408 Request Timeout`       | Sieve       | 上游连接超时（`[upstream].connect_timeout_ms`） |
| `429 Too Many Requests`     | 透传上游        | 上游 rate limit                           |
| `500 Internal Server Error` | Sieve       | Sieve 自身 bug；写入审计库 + 日志                 |
| `502 Bad Gateway`           | Sieve       | 上游连接失败                                  |
| `504 Gateway Timeout`       | Sieve       | 上游响应超时（`[upstream].timeout_ms`）         |


### 7.2 Sieve 自定义 4xx 子段（Critical 拦截相关）


| 状态码                                 | Sieve 语义          | 触发场景                                                                       | Body                     |
| ----------------------------------- | ----------------- | -------------------------------------------------------------------------- | ------------------------ |
| `426 Upgrade Required`              | **Critical 出站阻断** | 出站请求体命中 Critical 规则（如 OUT-09 BIP39 校验通过）                                   | `sieve_block` JSON       |
| `451 Unavailable For Legal Reasons` | **合规 / 安全策略阻断**   | 规则签名验证失败、配置违反硬约束（如 `bind_address != 127.0.0.1`、`telemetry.enabled = true`） | `sieve_block` JSON 或启动错误 |
| `499 Client Closed Request`         | 用户主动拒绝            | CLI 弹窗用户选择拒绝 / 超时 fail-closed                                              | `sieve_block` JSON       |


> 选用 `426` 是产品语义层的"行为升级要求"（用户需脱敏后重发），与 RFC 7231 协议升级语义有偏差，但**实际客户端不会因此崩溃**（Anthropic SDK 把非 2xx 当作错误并显示 body 文本）。该决策待补 ADR-008（见 [ADR-INDEX](../design/ADR-INDEX.md) 候选编号）。

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)
- 数据模型（fingerprint / SQLite schema）：[../design/data-model.md](../design/data-model.md)
- ADR 索引：[../design/ADR-INDEX.md](../design/ADR-INDEX.md)
- ADR-006 sigstore + reproducible build：[../design/ADR-006-sigstore-reproducible-build.md](../design/ADR-006-sigstore-reproducible-build.md)
- 术语表：[../glossary.md](../glossary.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)

---

## 接口冻结声明

- **冻结时间点**：Week 12 GA（参见 [PRD §10.2 Week 12](../prd/sieve-prd-v1.3.md#102-phase-b闭测阶段week-9-12)）
- **冻结范围**：本文 §1（反向代理路由）、§2（管理 API）、§3（配置文件 schema 顶层字段）、§4（环境变量名）、§5（severity → HTTP 状态码映射）、§6（CLI 退出码语义）、§7（自定义错误码 426 / 451 / 499）
- **冻结后变更规则**：
  - **MAJOR**（v2.0.0）：删除字段、改语义、改默认绑定地址、关闭 fail-closed 行为（**永远不会做**）
  - **MINOR**（v1.x.0）：新增可选字段、新增端点、新增检测规则 ID
  - **PATCH**（v1.0.x）：bug 修复、文档纠错、性能优化
- **检测规则 ID 不视为接口** —— 规则增删走 [CHANGELOG](../changelog/CHANGELOG.md) 记录，不触发 SemVer

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。任何 API 变更必须同步更新 [CHANGELOG](../changelog/CHANGELOG.md)。

