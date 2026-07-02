# Sieve API 参考

> **状态：早期预览（0.1.0-alpha），接口未冻结。**
> 当前文档反映实现事实（v1.4/v1.5 章节保留，较新内容标注"v2.0 新增"或"v2.1 新增"），**接口将在首个稳定版（GA）发布时冻结**，破坏性变更走 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)。
> 在冻结前，任何字段、状态码、配置项的细节都可能调整。

---

## 0. 文档定位

Sieve **不是** 面向开发者的 SDK / library，而是一个本地 HTTP 反向代理，因此本文所述的"API"分为三层：

1. **反向代理 API**（对 Claude Code 等上游客户端暴露） —— 透明转发 Anthropic Messages 协议
2. **本地管理 API**（仅 `127.0.0.1`） —— 健康检查、白名单管理、审计查询、规则刷新
3. **配置文件 schema + 环境变量 + CLI 退出码** —— 二进制运行时契约

> **不存在云端 API。**Sieve 完全本地运行，绝不联网做 token verification（核心硬约束）。

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

### 1.3 透明转发的路由清单（v1.5 扩展）

**Anthropic Messages API 路由**（Claude Code）：

| 方法     | 路径                          | 说明                                                         |
| ------ | --------------------------- | ---------------------------------------------------------- |
| `POST` | `/v1/messages`              | 主要消息 API，**含 SSE 流**；Sieve 在请求体（出站）和 SSE chunk 边界（入站）做注入检测 |
| `POST` | `/v1/messages/count_tokens` | token 计数，无流式；只做出站规则扫描                                      |
| `GET`  | `/v1/models`                | 模型清单，纯透传                                                   |
| `*`    | `/v1/...`（其他 Anthropic 路由）  | **Phase 1 默认 501 Not Implemented**，避免静默放过未审计协议             |

**OpenAI Chat Completions 路由（v1.5 新增）**（OpenClaw / Hermes）：

| 方法     | 路径                     | 说明                                                                     |
| ------ | ---------------------- | ---------------------------------------------------------------------- |
| `POST` | `/v1/chat/completions` | OpenAI Chat Completions API，**含 SSE 流**；双向检测逻辑与 Anthropic 路由相同，协议层透明转换 |
| `*`    | `/v1/...`（其他 OpenAI 路由） | **Phase 1 默认 501 Not Implemented** |

### 1.4 协议兼容性承诺

**v1.5 新增：OpenAI Chat Completions 协议支持**

v1.5 起，Sieve 额外支持 OpenAI Chat Completions 协议，用于 OpenClaw / Hermes 等 multi-provider agent 的接入：

- 上游 endpoint 根据请求路径分发：`/v1/messages` → Anthropic 协议；`/v1/chat/completions` → OpenAI 协议
- 两种协议在 Protocol Layer 内部转换为同一 `UnifiedMessage` 中间表示，下游 Filter Pipeline 无感知
- SSE 格式差异由协议适配器消化：Anthropic 用 `event/data` 多事件结构；OpenAI 用 `data: {...}` + `data: [DONE]` 结束标记
- 详见 [architecture.md §10.2](../design/architecture.md)

---

**底线约束**：**绝不在 Anthropic API 协议层伪造 `tool_use` / `stop_reason` / `id` / `usage` / `type` 字段**。这是产品承诺，不是实现细节。

允许的 SSE 操作：
- 发送 `: keep-alive\n\n` SSE comment 行（纯协议注释，不影响下游解析）
- 截断流（关闭 HTTP response）
- 注入 `sieve_blocked` event（Sieve 自报的拦截通知，**不冒充模型内容**）

三种处置路径对客户端可见性的差异：

| Disposition | 客户端（Claude Code）观察到的行为 | 协议层是否修改 SSE 流 |
|-------------|-------------------------------|-------------------|
| `AutoRedact` | 请求 body 改写后转发（敏感内容脱敏），上游响应**直通**，客户端无感知 | 仅改写请求，SSE 不修改 |
| `GuiPopup` | SSE 连接保持但**流暂停**，期间每 25s 收到 `: keep-alive\n\n` comment；用户允许后恢复转发；用户拒绝后收到 `sieve_blocked` event + 流关闭 | hold 流 + 可能注入 sieve_blocked |
| `HookTerminal` | SSE **原样透传**，Claude Code 内部收到完整 tool_use；但在 PreToolUse 阶段由 sieve-hook 决定是否执行；用户拒绝时 Claude Code 自行报告工具调用被拒绝 | **不修改 SSE 流** |
| `StatusBar` | 透传，客户端无感知 | 不修改 |

> 上游真实 endpoint 由 `[upstream].url` 决定（默认 `https://api.anthropic.com`），用户可指向官方直连或任意 OpenAI/Anthropic 协议兼容的中转站。

### 1.5 行为差异（命中检测时，v1.4 双层防御）

v1.4 将入站规则按 disposition 分为两类，拦截行为不同：

**Hook 类（IN-CR-02~04 / IN-GEN-01~03）——代理不修改 SSE 流**：
- 代理检测命中后写 `~/.sieve/pending/<id>.json`，SSE **原样透传**给 Claude Code
- Claude Code 收到完整 tool_use 后触发 PreToolUse hook，调用 `sieve-hook check`
- sieve-hook 读 pending 文件，TTY 展示规则摘要 + y/n 倒计时
- 用户 `n` / 超时 30s → exit 1 → Claude Code **拒绝执行**该工具（HTTP 层面无变化）
- 用户 `y` → exit 0 → Claude Code 正常执行

**GUI 类（IN-CR-01/05 / IN-GEN-04）——代理 hold SSE 流**：
- 代理检测命中后 **hold** HTTP 响应，不向客户端发送后续 SSE chunk
- 每 25 秒发送 `: keep-alive\n\n` SSE comment 防止 Claude Code HTTP 超时
- 通过 IPC 通道 A（Unix socket JSON-RPC `sieve.request_decision`）通知 Native GUI App
- GUI 展示 HIPS 弹窗（含地址对比 diff / typed data 详情，120s 倒计时）
- 用户**允许** → `sieve.decision_response allow` → 代理继续转发 SSE
- 用户**拒绝** / 超时 → `sieve.decision_response deny` → 代理注入 `sieve_blocked` event + 关闭流（fail-closed）
- GUI 进程失联（30s 无响应）→ 代理超时 → fail-closed

**v2.0 IPC 协议扩展**（三态决策）：

- `sieve.request_decision` 新增字段：
  - `allow_remember: bool` —— **daemon 端计算**该值（不让 GUI 决定）。当 `rule_id` 在 `critical_lock::FAIL_CLOSED_RULES` 时强制 false（fail-closed 不被绕过）
- `sieve.decision_response` 新增字段：
  - `remember: bool` —— GUI 用户在弹窗勾选 "永久允许"。daemon 收到 true 时**必须二次校验** rule_id 是否允许 Remember；不允许则忽略 + 写 audit ERROR
  - `context_hint: Option<String>` —— GUI 表单输入的备注（"Vitalik 地址 read-only balanceOf 调用" 等），写入灰名单 JSON
- 灰名单存储：`~/.sieve/decisions/<digest>.json`（文件名 hex digest，0600 权限，atomic rename，no-follow symlink；所有变更写 audit.db）
- 内置 Critical 规则的 GUI 弹窗 Remember checkbox **必须 disabled+灰显**，tooltip 解释"内置 Critical 规则保护核心安全场景，不允许永久绕过"

**出站 AutoRedact（OUT-01~05/06/08/11/12）**：
- 请求 body 中敏感内容自动脱敏替换，**不返 426**，直接转发到上游，上游响应直通
- 状态栏静默通知（不打断用户流程）

**出站 GuiPopup（OUT-07/09/10——高确定性助记词/私钥）**：
- hold 出站请求，弹 HIPS 弹窗等用户确认（Sieve 差异化点，普通 DLP 工具不做这一步）
- 用户允许 → 原文转发；用户拒绝 → 返 `426 Upgrade Required` + `sieve_blocked` JSON

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
  "docs_url": "https://github.com/SieveAI-dev/sieve/blob/main/docs/api/api-reference.md#5-处置矩阵--http-行为"
}
```

入站 Critical 在 SSE 流中的完整序列（fail-closed）：

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
- **降级模式不影响入站 Critical**：降级模式用户的入站 Critical 仍然 fail-closed

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
  "version": "0.1.0-alpha",
  "rules_version": "2026-04-26.1",
  "rules_sha256": "9f8e7d6c...",
  "sigstore_bundle_url": "https://github.com/SieveAI-dev/sieve/releases/download/v0.1.0/sieve-darwin-arm64.sigstore",
  "build": {
    "rustc": "1.80.0",
    "git_sha": "abcdef1234567890",
    "reproducible": true
  }
}
```

> `reproducible: true` 表示该二进制可由 sigstore 签名 + 可复现构建流程复现。

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


> **绝不返回原文。**只返回 fingerprint + 元信息（核心硬约束 + 数据本地化）。详细字段定义见 [data-model.md](../design/data-model.md)。

**v2 audit schema 扩展（v2.0 新增）**：

events 表通过 v2 migration（commit cd0248d）自动添加两列：

| 新增列 | 类型 | 说明 |
|--------|------|------|
| `caller_pid` | INTEGER NULL | daemon accept loop 通过 macOS `proc_listpids` + `proc_pidfdinfo` 反查连接方 PID；30s LRU cache |
| `caller_exe` | TEXT NULL | 对应 PID 的可执行文件路径（`proc_pidpath`）；非 macOS 平台或反查失败时为 NULL |

新增 AuditEvent 变体（v2.0，对应 `crates/sieve-cli/src/audit.rs`）：

| AuditEvent 变体 | 触发场景 |
|----------------|---------|
| `DecisionMade` | daemon 处理 `sieve.decision_response` 完毕，记录 allow/deny + remember + context_hint |
| `GraylistAdded` | 灰名单条目写入成功（`~/.sieve/decisions/<digest>.json`，原子写，0600） |
| `GraylistCriticalRejected` | daemon 二次校验拒绝 Critical 规则的 remember=true 请求（Critical 锁防线 #2） |
| `GraylistAddFailed` | 灰名单文件写入失败（磁盘权限、路径冲突等），写 audit ERROR |
| `GraylistHit` | 本次 scan 命中已有灰名单条目，直接放行（无弹窗） |
| `SequenceHit` | 行为序列窗口（ToolUseSequence）匹配到 IN-SEQ-* 序列模式，发状态栏通知 |
| `UserRulesReloaded` | `~/.sieve/rules/user.toml` 热加载成功，记录规则数量和 sha256 |
| `UserRulesLoadFailed` | `user.toml` 加载或 lint 失败，记录错误原因，发状态栏通知 |

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

> fingerprint 格式 = `<rule_id>:<sha256_prefix_16_hex>`，例如 `OUT-09:7a3b9c1d5e6f7890`。`sha256_prefix` 取规则匹配内容 SHA-256 的前 **8 字节，以 lowercase hex 编码为 16 个字符**——足以在单用户审计库内唯一标识，且不暴露原文。详见 [data-model.md](../design/data-model.md)。

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

> **更新通道修订（2026-05-05）**：下表中 `[rules_update]` / `[telemetry]` 段为 v1.x 设计参考，已统一为 `[update]` 段（详见 §8 manifest 接口 + [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)）。新代码请使用 `[update]` 段；`[rules_update]` 段保留仅作历史参考,运行时不再读取。

| 段                | 字段                    | 类型      | 默认值                                              | 说明                                                              |
| ---------------- | --------------------- | ------- | ------------------------------------------------ | --------------------------------------------------------------- |
| `[server]`       | `port`                | u16     | `11453`                                          | 反向代理监听端口                                                        |
| `[server]`       | `bind_address`        | string  | `"127.0.0.1"`                                    | **强制 `127.0.0.1`**，写其他值启动失败                          |
| `[server]`       | `management_token`    | string? | `null`                                           | 本地管理 API 的可选 Bearer Token                                       |
| `[server]`       | `binary_fallback`     | bool    | `false`                                          | 启用 `~/.sieve/bin/sieve.prev` 一键回滚                               |
| `[upstream]`     | `url`                 | string  | `"https://api.anthropic.com"`                    | 上游真实 endpoint                                                   |
| `[upstream]`     | `timeout_ms`          | u32     | `120000`                                         | 整体请求超时                                                          |
| `[upstream]`     | `connect_timeout_ms`  | u32     | `5000`                                           | TCP 连接超时                                                        |
| `[upstream]`     | `retry`               | u8      | `0`                                              | **默认不重试**，避免重复执行带副作用的工具调用                                       |
| `[detection]`    | `rules_path`          | path    | `"~/.sieve/rules"`                               | 已签名规则目录                                                         |
| `[detection]`    | `sieveignore_path`    | path    | `"~/.sieve/.sieveignore"`                        | 本地白名单文件（**不上传仓库**）                                              |
| `[detection]`    | `dry_run`             | bool    | `false`                                          | 干跑模式：命中 Critical 只记录不返 426，用于调试规则。CLI `--dry-run` flag 覆盖为 true |
| `[detection]`    | `severity_overrides`  | table   | `{}`                                             | 子表，按 `rule_id` 覆盖默认 severity（仅可降级；**Critical 不可关闭**）            |
| `[storage]`      | `audit_db_path`       | path    | `"~/.sieve/audit.db"`                            | SQLite append-only 审计库                                          |
| `[storage]`      | `log_path`            | path    | `"~/.sieve/logs/sieve.log"`                      | 文本日志，按天 rotate                                                  |
| `[storage]`      | `log_level`           | enum    | `"info"`                                         | `trace` / `debug` / `info` / `warn` / `error`                   |
| `[license]`      | `key`                 | string? | `null`                                           | 预留字段，当前免费版不使用，不启用 license 验证                                   |
| `[license]`      | `offline_grace_days`  | u16     | `30`                                             | 预留字段，当前免费版不启用 license 验证                                       |
| `[rules_update]` | `enabled`             | bool    | `true`                                           | 关闭后等价 `SIEVE_DISABLE_RULES_UPDATE=1`                            |
| `[rules_update]` | `signing_pubkey_path` | path    | `"~/.sieve/keys/sieve-rules.pub"`                | Ed25519 公钥，**fail-closed**                                      |
| `[rules_update]` | `update_url`          | string  | `"https://updates.sieveai.dev/v1/rules.tar.zst"` | 规则包下载地址                                                         |
| `[rules_update]` | `interval_hours`      | u16     | `168`                                            | 自动检查间隔（默认每周）                                                    |
| `[telemetry]`    | `enabled`             | bool    | `**false`（强制，不可改）**                              | **不存在任何 telemetry。**此字段保留仅为让用户在 config 中可视化确认；写 `true` 启动会拒绝并提示 |


### 3.2 `severity_overrides` 子表语义

**只允许降级，不允许升级。`**Critical` 项**禁止**降级。

```toml
[detection.severity_overrides]
"OUT-11" = "low"      # 默认 medium → 降级到 low（允许）
"IN-GEN-04" = "low"   # 默认 high → 降级到 low（允许）
"OUT-09" = "high"     # ❌ 启动失败：BIP39 默认 critical，禁止降级
```

### 3.3.1 Multi-listener 配置（**实际代码 schema**）

> 本节记录的是 `crates/sieve-cli/src/config.rs` 的实际配置结构（截至 2026-05-05）。
> §3.1 表格保留作设计参考；运行时以本节的 schema 为准。

`Config` 顶层字段实际是扁平的（不分 `[server]` / `[upstream]` / `[storage]` 段）。
`[[upstream]]` 数组支持多 listener：

```toml
# 推荐写法（multi-listener）
bind_addr = "127.0.0.1"
tls_verify_upstream = true

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
provider_id = "anthropic"
protocol = "anthropic"

[[upstream]]
port = 11454
url = "https://api.deepseek.com/anthropic"   # path 前缀已正确转发
provider_id = "deepseek"
protocol = "anthropic"

[[upstream]]
port = 11455
url = "https://api.openai.com"
provider_id = "openai"
protocol = "openai"
```

`[[upstream]]` 字段：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `port` | u16 | (必填) | 监听端口（127.0.0.1:port），同 daemon 内必须唯一 |
| `url` | string | (必填) | 上游真实 endpoint，**含 path 前缀**（如 `/anthropic`） |
| `provider_id` | string | URL host 派生 | 审计 / 日志 / IPC 事件标注 |
| `protocol` | enum | `"auto"` | `"auto"`（默认）\| `"anthropic"` \| `"openai"`。`auto` 按请求 path 自适应、**不强制错位**；显式声明 `anthropic`/`openai` 才对 path 做严格校验、错位请求 fail-closed 400 |

**向后兼容**：旧 schema（`upstream_url` + `port` 单字段）继续工作，自动映射成单元素
`upstreams` vec（provider_id = `"anthropic"`，protocol = `Auto`）。省略 `protocol` 字段的
`[[upstream]]` 同样映射为 `Auto`——按 path 自适应、不做错位拒绝，保留单 upstream 双协议能力。
新旧字段同时给
时新字段优先，旧字段被忽略并 WARN。

```toml
# 旧写法（仍可用，自动映射成单 listener）
upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
```

**协议错位 fail-closed 拒绝**（**仅对显式声明 protocol 的 listener 生效**）：
- `protocol = "anthropic"` listener 收到 `/v1/chat/completions` → 400 + `sieve_blocked` event
- `protocol = "openai"` listener 收到 `/v1/messages` → 400
- `protocol = "auto"`（默认 / 未声明 / legacy `upstream_url`）listener **不做错位拒绝**：按请求 path 自适应路由（`/v1/messages` → Anthropic，`/v1/chat/completions` → OpenAI），保留单 upstream 双协议能力
- 其他 path（健康检查 / 透传）保持原行为
- X-Sieve-Provider header routing 不能 override listener 协议

### 3.3.2 上游转发代理配置（HTTP CONNECT + SOCKS5）

> 见 [SPEC-007](../specs/SPEC-007-upstream-proxy.md)。解决受限网络（Shadowrocket / Clash 等规则代理 + 分流，非全局 TUN）下 sieve 上游硬直连不可用的问题。daemon 转发上游与 updater 出站均可经配置的代理出网，**TLS 端到端到上游，代理只见密文、不 MITM**。

```toml
# 全局兜底代理（可选）：所有未单独配置且未声明 no_proxy 的 upstream 继承
proxy = "socks5://127.0.0.1:6153"

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
protocol = "anthropic"
# 未写 proxy 也未写 no_proxy → 继承全局 proxy

[[upstream]]
port = 11454
url = "https://api.openai.com"
protocol = "openai"
proxy = "http://127.0.0.1:7890"   # 该 upstream 专属代理，覆盖全局

[[upstream]]
port = 11455
url = "http://127.0.0.1:8080"     # 本地中转站
protocol = "openai"
no_proxy = true                    # 显式直连，无视全局 proxy 与 env
```

字段：

| 段 | 字段 | 类型 | 默认 | 说明 |
|----|------|------|------|------|
| 顶层 | `proxy` | string? | `null` | 全局兜底代理 URL，所有未单独配置且未 `no_proxy` 的 upstream 继承 |
| 顶层 | `gui_peer_code_requirement` | string? | `null` | GUI peer 代码签名 requirement（F1-b，macOS SecRequirement 语法，如 `identifier "com.sieve.gui" and anchor apple generic and certificate leaf[subject.OU] = "TEAMID"`）。设置后 GUI wire 应答放行 Critical 前强制核验对端进程签名，未通过静默改写 deny；未设置不核验（daemon 启动 warn）。非 macOS 设置 = 恒拒（fail-closed）。详见 SPEC-005 §6.2.4 |
| `[[upstream]]` | `proxy` | string? | `null` | 该 upstream 专属代理 URL，**覆盖全局 `proxy`** |
| `[[upstream]]` | `no_proxy` | bool | `false` | 显式直连，**优先级最高**，无视全局 `proxy` 与 env |

**优先级链（高 → 低）**：
1. `upstream.no_proxy = true` → 直连
2. `upstream.proxy` → 用之
3. 全局顶层 `proxy` → 用之
4. env `HTTPS_PROXY` / `ALL_PROXY`（`HTTPS_PROXY` 优先于 `ALL_PROXY`，零配置便利兜底）
5. 直连

> 显式 config 优先于 env；`no_proxy = true` 可在任意层级强制直连。

**proxy URL 格式**：
- scheme：`http://`（HTTP CONNECT）/ `socks5://` / `socks5h://`（`h` = 远程 DNS，由代理侧解析），按 scheme 自动选实现
- 认证：`scheme://user:pass@host:port`（本地代理通常无需，远程代理用）
- 解析失败 → **启动期 config 校验 fail-fast 报错**，不静默忽略
- 代理连接失败（拒绝 / 超时 / 认证失败 / CONNECT 非 200 / SOCKS 握手失败）→ **明确报错，绝不静默回退直连**；日志记录代理 `host:port` 但**脱去密码**

> **隐私提示**：经**远程**代理时代理可见 SNI 目标 / 目标 IP（即「你在连 `api.anthropic.com`」），但**不可见** prompt / response / API key。推荐使用**可信本地代理出口**（Shadowrocket / Clash）。

### 3.3.3 加密审计日志配置（`[audit]`）

> 三档 logging level 控制 daemon 落盘审计行为；`full` 档为 opt-in 加密归档（write-only logging），默认不编入主二进制（`audit-crypto` 特性门控）。数据模型见 [data-model.md §14](../design/data-model.md)。

```toml
[audit]
level = "metadata"            # off | metadata（默认）| full
# 以下字段仅 level = "full" 时生效（full 档默认关，须显式 opt-in）
# recipient = "age1qx…"       # age 公钥（recipient），daemon 仅持公钥、结构上无解密能力
# archive_dir = "~/.sieve/archive"
retention_days = 30           # 0 = 永久保留；超期整段密文文件删除
hash_chain = true             # 防篡改哈希链（历史防改写），full 档必做项
rotation = "daily"            # 归档段轮换粒度
```

`[audit]` 字段：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `level` | enum | `"metadata"` | `off`（什么都不留）\| `metadata`（默认，=现状：只写元数据 + 脱敏后片段/哈希，**零行为变化**）\| `full`（**opt-in + 显式警告**，加密归档脱敏后完整内容） |
| `recipient` | string? | `null` | `full` 档必填：age 公钥（`age1…`），daemon **只持公钥**、只能加密追加、**结构上不具备解密能力**（write-only logging）。`level = "full"` 而缺 `recipient` → 启动 fail-fast |
| `archive_dir` | path | `"~/.sieve/archive"` | 加密归档段（archive segment）目录，0700 权限 |
| `retention_days` | u32 | `30` | 保留期；daemon 周期扫描删除超期**整段密文文件**（`full` 档归档上唯一允许的变更），每次清理写一条 `metadata` 审计事件。`0` = 永久保留 |
| `hash_chain` | bool | `true` | 防篡改哈希链（每条归档记录含 `prev_hash` + 单调递增 `seq`），保证历史不可悄悄改写 / 删除 / 重排。**已裁定为 `full` 档必做项** |
| `rotation` | enum | `"daily"` | 归档段轮换粒度（如 `daily`），与 `retention_days` 配合做整段删除 |

**红线**：脱敏必须在内存里、在任何字节碰硬盘之前完成。`full` 档归档的**永远是脱敏后内容**（消费出站 `redact_body_bytes()` 返回值 / 入站经替换后内容），脱敏前的明文密钥永不落盘，无论是否加密。

> ⚠ **`full` 档默认关闭，且口令丢失不可恢复（by design）**：私钥（identity）以口令保护（age scrypt），daemon 正常运行**不需要口令**（只用公钥），口令仅在 ①`sieve audit keygen` 生成密钥对 ②`sieve audit decrypt` 审计解密 两个时刻出现。**口令一旦丢失，identity 无法解锁，归档永久不可读，这不是 bug 而是设计使然**；请在生成后立即把私钥移出本机（密码管理器 / 离线介质）并备份口令。审计解密应在另一台 / 离线机器执行（daemon 机器不留 identity）。

### 3.3.4 本地用量观测配置（`[billing_check]` + `[[upstream]].trust`）

> 可选的本地 token 用量观测：对经过的 LLM 流量做独立 token 核算，并与上游声明的 `usage` 做本地比对，偏差超容差仅 StatusBar 通知（**不阻断流量**）。默认全关，且默认不编入主二进制（`usage` 特性门控）。统计严格本地、**永不上传**（呼应 [SPEC-006 §9.1](../specs/SPEC-006-update-and-telemetry.md)）。

```toml
[billing_check]
enabled = false              # 默认关；不开启则零行为变化、零新增出站、零计算开销
tolerance_pct = 15           # 偏差容差百分比，超过即报警
count_tokens_optin = false   # 默认关（唯一可能触发 Sieve 主动出站的开关，详见警示）

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
protocol = "anthropic"
# trust 缺省按 host 派生：api.anthropic.com / api.openai.com → official，其余 → relay
# trust = "official"         # 可显式覆盖派生结果

[[upstream]]
port = 11454
url = "http://127.0.0.1:8080"   # 本地 / 第三方中转站
protocol = "anthropic"
trust = "relay"              # 经中转：usage 视为未经验证的声明，独立核算 + 交叉比对
```

`[billing_check]` 字段：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `enabled` | bool | `false` | 总开关。默认关 → 零行为变化、零新增出站、零计算开销。仅当上游为 `relay` 且 `enabled = true` 时启用核算（`official` 直连不核算，`usage` 权威） |
| `tolerance_pct` | u8 | `15` | 偏差容差百分比。`偏差 = \|claimed − independent_count\| / independent_count`，超过即本地报警。默认值兼顾低误报与敏感度 |
| `count_tokens_optin` | bool | `false` | **默认 `false`**。开启后 Anthropic 输入侧改用官方 `POST /v1/messages/count_tokens` 直连拿权威输入数。**这是本特性唯一可能触发 Sieve 主动出站的开关**——⚠ 开启即向官方 endpoint（`api.anthropic.com`）发起 Sieve 主动（非用户直接触发）出站，请求体为脱敏后内容；默认关时仅用本地近似估算，零新增出站（不破坏完全本地运行承诺） |

`[[upstream]]` 新增 `trust` 字段：

| 段 | 字段 | 类型 | 默认 | 说明 |
|----|------|------|------|------|
| `[[upstream]]` | `trust` | enum | host 派生 | `official`（官方直连，`usage` 直接采纳）\| `relay`（经中转，`usage` 视为未经验证的声明，做本地独立核算与比对）。**缺省按 url host 派生**：host ∈ {`api.anthropic.com`, `api.openai.com`} → `official`，其余 → `relay`；无法判定时**保守默认 `relay`**（fail-closed 倾向）。可显式 `trust = "official"` / `"relay"` 覆盖派生结果 |

> **隐私红线（不可放宽）**：token 用量恰是 README 发誓「从不上传」的那个 usage record。统计落本地 SQLite（`~/.sieve/usage.db`，0600，append-only，独立于 `audit.db`，经 `request_id` 关联），**无任何出站上报路径**，只在本地 GUI / CLI 可见。

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

# [license] 段为预留字段，当前免费版不使用，不启用 license 验证

[rules_update]
enabled = true
signing_pubkey_path = "~/.sieve/keys/sieve-rules.pub"
update_url = "https://updates.sieveai.dev/v1/rules.tar.zst"
interval_hours = 168

[telemetry]
enabled = false               # 强制 false，写 true 启动失败

# 加密审计日志（§3.3.3）：默认 metadata 档 = 现状，零行为变化
[audit]
level = "metadata"            # off | metadata（默认）| full
# full 档（opt-in）字段，默认 metadata 时无效，仅作姿态展示：
# recipient = "age1qx…"       # age 公钥，开 full 档时必填；口令丢失归档永久不可读
# archive_dir = "~/.sieve/archive"
retention_days = 30
hash_chain = true
rotation = "daily"

# 本地用量观测（§3.3.4）：默认全关，零行为变化、零新增出站
[billing_check]
enabled = false               # 默认关
tolerance_pct = 15            # 偏差容差百分比
count_tokens_optin = false    # 默认关；唯一可能触发 Sieve 主动出站的开关

# [[upstream]].trust（§3.3.4）：缺省按 host 派生
# api.anthropic.com / api.openai.com → official，其余 → relay（保守默认）
# [[upstream]]
# port = 11453
# url = "https://api.anthropic.com"
# protocol = "anthropic"
# trust = "official"          # 可显式覆盖派生结果
```

---

## 4. 环境变量与自动配置

### 4.1 `sieve setup` 自动配置（推荐）

v1.4 不再需要手动 `export ANTHROPIC_BASE_URL`。首次安装后运行：

```bash
sieve setup
```

`sieve setup` 自动完成（详见 [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）：
1. 写入 Claude Code `settings.json` → `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`
2. 注册 PreToolUse hook：`sieve-hook check`（`onError: block`）
3. 生成 launchd plist + `launchctl bootstrap`（开机自启）
4. 创建 `~/.sieve/` 目录结构（权限 0700）

修改任何文件前打印 diff 并要求 `y` 确认；原始文件备份到 `~/.sieve/backups/`。

### 4.2 环境变量列表

| 变量 | 适用方 | 必需 | 默认 | 说明 |
|------|--------|------|------|------|
| `ANTHROPIC_BASE_URL` | **用户侧（Claude Code）** | 是 | 无 | 指向 Sieve 监听地址 `http://127.0.0.1:11453`；`sieve setup` 自动配置 |
| `ANTHROPIC_AUTH_TOKEN` | **用户侧（Claude Code）** | 是 | 无 | 用户原 Anthropic / 中转站 key，由 Sieve 透传到上游 |
| `SIEVE_HOME` | sieve 进程 + sieve-hook | 否 | `~/.sieve` | 覆盖整个 IPC 文件路径根（`pending/`、`decisions/`、`ipc.sock` 等均跟随变化） |
| `SIEVE_CONFIG` | sieve 进程 | 否 | `$SIEVE_HOME/config.toml` | 覆盖配置文件路径 |
| `SIEVE_LICENSE_KEY` | sieve 进程 | 否 | 无 | 覆盖 `[license].key`（预留字段，当前免费版不使用） |
| `SIEVE_LOG_LEVEL` | sieve 进程 | 否 | `info` | 覆盖 log 级别，取值 `trace`/`debug`/`info`/`warn`/`error` |
| `SIEVE_RULES_PATH` | sieve 进程 | 否 | `$SIEVE_HOME/rules` | 覆盖规则路径 |
| `SIEVE_NO_UPDATE` | sieve 进程 | 否 | 未设置 | 完全跳过 manifest 更新检查（不发请求,规则冻结,无遥测）—— 离线 / 隔离网络 / CI 测试。命中时启动 banner 必打印 `update check disabled by SIEVE_NO_UPDATE`。详见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) |
| `SIEVE_NO_TELEMETRY` | sieve 进程 | 否 | 未设置 | 仍发 manifest 请求但省略 `uid` 字段（隐私敏感用户仍能拿规则更新） |
| `SIEVE_UPDATE_URL` | sieve 进程 | 否 | `https://updates.sieveai.dev/v1/manifest` | 覆盖默认更新源 URL（企业自托管镜像 / 私有内网 / 本地 mock）|
| `HTTPS_PROXY` | sieve 进程 | 否 | 未设置 | 上游转发 + updater 出站的兜底代理（代理优先级链第 4 级，**优先于 `ALL_PROXY`**）；config `proxy` / `no_proxy` 优先于此 |
| `ALL_PROXY` | sieve 进程 | 否 | 未设置 | 同上兜底代理，`HTTPS_PROXY` 未设时生效 |
| ~~`SIEVE_DISABLE_RULES_UPDATE`~~ | ~~sieve 进程~~ | — | — | **已被 `SIEVE_NO_UPDATE` 替换（2026-05-05）**,旧字段不再读取 |

> 环境变量优先级 **高于** 配置文件，但 **低于** CLI flag（如有）。`SIEVE_HOME` 是总控变量，设置后无需分别覆盖各子路径。

**v1.5 multi-agent 相关（无新环境变量）**：

- multi-agent setup 本身**不引入新的 Sieve 环境变量**——`sieve setup --agent claude|openclaw|hermes|codex` 修改的是目标 agent 自己的配置文件（`settings.json` / `config.toml` / `~/.codex/hooks.json` / `.env`），不是 Sieve 的 env var
- Hermes 启动 Claude Code 子进程时，通过 `ANTHROPIC_DEFAULT_HEADERS` env var 自动注入 `X-Sieve-Origin` header，**用户无需手动配置**；Sieve 仅读取这个 header，不负责写入
- OpenClaw 通过 `X-Sieve-Source-Channel` header（OpenClaw 自身注入）传递来源 channel 元数据，Sieve 读取后用于 IN-GEN-06 外部 channel injection 检测

---

## 5. 处置矩阵 → HTTP 行为（v1.4 二维矩阵）

| Disposition | 适用规则 | 出站/入站 | 代理 HTTP 行为 | 备注 |
|-------------|---------|---------|----------------|------|
| **AutoRedact** | OUT-01~05/12/13 | 出站 | `200 OK` + 改写后的请求 body 转发上游；上游响应**直通**，**不返 426** | 自动脱敏，不打断用户流程；OUT-12/13（WIF/xprv）经 Base58Check second-pass 校验和验证后脱敏 |
| **AutoRedact** | OUT-06/08 | 出站 | 同上 | ETH/Solana entropy 边界模糊，脱敏继续 |
| **GuiPopup**（出站） | OUT-07/09/10 | 出站 | hold 请求；GUI 弹窗；用户允许 → `200 OK` + 原文/脱敏后转发；用户拒绝 → `426 Upgrade Required` + `sieve_blocked` JSON | 高确定性助记词/私钥，Sieve 差异化点 |
| **HookTerminal** | IN-CR-02/03/04，IN-GEN-01~03 | 入站 | `200 OK`（SSE **原样透传**）+ 写 `~/.sieve/pending/<id>.json`；HTTP 层无变化 | sieve-hook 在 PreToolUse 阶段拦截，Claude Code 自行报告拒绝 |
| **GuiPopup**（入站） | IN-CR-01/05/CANARY，IN-GEN-04 | 入站 | `200 OK`（SSE hold）+ 每 25s `: keep-alive\n\n` comment；用户允许 → 继续流；用户拒绝/超时 → 注入 `sieve_blocked` event + EOF | 代理 hold 住 SSE 流等 GUI 决策（最长 120s）；IN-CR-CANARY = 诱饵文件被读触发 |
| **StatusBar** | OUT-11，IN-GEN-05 | 出/入站 | 透传（`200 OK`，不修改流）+ IPC `sieve.event_notify` 菜单栏通知 | 不打断用户，低优先级通知 |

**说明**：
- AutoRedact 出站**不返 426**——脱敏后直接转发，是"帮用户擦屁股"哲学的体现
- HookTerminal 入站**代理不修改 SSE 流**——拦截在 PreToolUse 执行边界，而非 message 边界
- Critical 在所有版本（含降级模式）不可关闭；降级模式下 High 仅审计记录，但 Critical 的 GuiPopup/HookTerminal 行为不变

---

## 6. CLI 退出码 / 弹窗确认协议

v1.4 双层防御引入两种弹窗协议，退出码语义分别描述：

### 6.1 sieve-hook 退出码（Hook 类规则，IPC 通道 B）

`sieve-hook` 是 Claude Code 每次 PreToolUse 都 fork 的进程，退出码直接决定 Claude Code 是否执行该工具：

| 退出码 | 含义 | Claude Code 行为 | 主代理审计记录 |
|--------|------|-----------------|----------------|
| `0` | allow（用户放行）| 继续执行 tool | `user_decision: "approve"` |
| `1` | deny（用户拒绝 / 超时 / 解析失败 / stale pending 文件） | 取消 tool 执行，向用户报告被拦截 | `user_decision: "deny"` 或 `"timeout"` |

- 超时默认（30s）：写 `decision=deny` 后 exit 1（fail-closed）
- pending 文件不存在：exit 0（代理未标记该工具调用，无需拦截）
- pending 文件 JSON 解析失败 / stale（> 10 分钟）：exit 1（fail-closed）
- 任何异常 exit code（非 0/1）：Claude Code 按 `onError: block` 处理，等价 deny

### 6.2 GUI App 决策协议（GUI 类规则，IPC 通道 A）

> **权威规格已迁移到 [SPEC-005](../specs/SPEC-005-ipc-protocol.md)**（IPC 协议 v2，2026-05-02 起生效）。本节保留为索引：
>
> - 握手 `sieve.hello`、心跳 `sieve.heartbeat`、通用枚举（snake_case lowercase）：[SPEC-005 §3–§5](../specs/SPEC-005-ipc-protocol.md)
> - `sieve.request_decision` 单 issue + 多 issue 合并 schema：[SPEC-005 §6.1](../specs/SPEC-005-ipc-protocol.md#61-sieverequest_decisiondaemon--gui-request)
> - `sieve.decision_response` 单 / 合并响应：[SPEC-005 §6.2](../specs/SPEC-005-ipc-protocol.md#62-sievedecision_responsegui--daemon-response)
> - `sieve.request_decision_canceled` 取消通知：[SPEC-005 §6.3](../specs/SPEC-005-ipc-protocol.md#63-sieverequest_decision_canceleddaemon--gui-fan-out-notification)
>
> **关键约束（不重复 schema，仅强调）**：
> - `default_on_timeout` 取值为 snake_case `"block"` / `"allow"` / `"redact"`；Critical 强制 `"block"`
> - `decision` 取值为 `"allow"` / `"deny"`；`"redact_and_allow"` 仅用于 daemon 内部及 `request_decision_canceled.auto_decision`
> - `allow_remember = false` 时 daemon 收到 `remember=true` 必须二次校验 + 写 audit ERROR（Critical 锁三道防线之防线二）
> - GUI 超时（120s）→ daemon 侧 oneshot 超时触发 `default_on_timeout=block`，记录 `user_decision: "timeout"`
> - 协议方法名一律带 `sieve.` 前缀（v1 中无前缀的 `request_decision` 已在 v2 弃用）

以下为 v1 历史描述，仅供回看；**与 SPEC-005 不一致时一律以 SPEC-005 为准**：

GUI App 是**常驻进程**，无 exit code 概念。决策通过 JSON-RPC 2.0 Unix socket 返回：

```json
{
  "jsonrpc": "2.0",
  "result": {
    "request_id": "<uuid>",
    "decision": "allow" | "deny",
    "decided_at": "<iso8601>",
    "by_user": true,
    "remember": false,
    "context_hint": null
  },
  "id": "<request_id>"
}
```

对应 `crates/sieve-ipc/src/protocol.rs` 中 `DecisionResponse`。`decision` 取值：
- `"allow"` → 代理继续转发 SSE，记录 `user_decision: "approve"`
- `"deny"` → 代理注入 `sieve_blocked` event + 关闭流，记录 `user_decision: "deny"`
- GUI 超时（120s）→ 代理侧 oneshot 超时触发 `default_on_timeout=Block`，记录 `user_decision: "timeout"`

**DecisionRequest 字段表（v2.0 扩展）**：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `request_id` | Uuid | — | 本次弹窗请求的唯一 ID |
| `rule_id` | String | — | 命中规则的 ID（如 `IN-CR-05`） |
| `severity` | String | — | `critical` / `high` / `medium` / `low` |
| `summary` | String | — | 人类可读的规则摘要，GUI 展示 |
| `allow_remember` | bool | `false` | **v2.0 新增**。由 daemon 端通过 `is_critical_locked(rule_id)` 计算后传入，**不由 GUI 决定**。内置 Critical 规则（`FAIL_CLOSED_RULES`）强制为 false，旧 v1.5 客户端不发此字段时 `#[serde(default)]` 兼容为 false。GUI 收到 false 时 **Remember checkbox 必须 disabled+灰显** |

**DecisionResponse 字段表（v2.0 扩展）**：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `request_id` | Uuid | — | 对应 DecisionRequest.request_id |
| `decision` | String | — | `"allow"` / `"deny"` |
| `decided_at` | String | — | ISO 8601 时间戳 |
| `by_user` | bool | — | true = 用户主动操作；false = 超时自动处理 |
| `remember` | bool | `false` | **v2.0 新增**，`#[serde(default)]`。GUI 用户勾选"永久允许"后为 true；daemon 收到 true 时**必须二次校验** rule_id 是否允许 Remember（不允许则忽略并写 audit ERROR），此为 v2.0 双路校验路径 |
| `context_hint` | Option\<String\> | `null` | **v2.0 新增**，`#[serde(default)]`。GUI 弹窗中用户输入的备注（如 "Vitalik 地址 read-only balanceOf 调用"），写入灰名单 entry；旧客户端不发此字段兼容 null |

### 6.3 IPC 单向通知（v2.0/v2.1 新增）

> **权威规格已迁移到 [SPEC-005](../specs/SPEC-005-ipc-protocol.md)**。本节保留为索引：
>
> - `sieve.notify_status_bar`（daemon → GUI fan-out）：[SPEC-005 §7](../specs/SPEC-005-ipc-protocol.md#7-状态栏通知sievenotify_status_bar)，NotifyKind 见 [SPEC-005 §5.9](../specs/SPEC-005-ipc-protocol.md#59-notify_kindnotify_status_barkind)
> - `sieve.reload_user_rules`（CLI → daemon）：[SPEC-005 §8](../specs/SPEC-005-ipc-protocol.md#8-cli--daemon-单向sievereload_user_rules)
> - `sieve.preset_changed` / `sieve.paused_changed`（daemon → GUI fan-out）：[SPEC-005 §10](../specs/SPEC-005-ipc-protocol.md#10-暂停与-preset-状态广播daemon--gui-fan-out-notification)
> - GUI 控制面方法（`sieve.set_paused` / `set_preset` / `set_preset_overrides` / `reload_config` / `health` / `evaluate` / `list_graylist` / `remove_graylist`）：[SPEC-005 §9](../specs/SPEC-005-ipc-protocol.md#9-gui-控制面方法gui--daemon-requestresponse)
> - 错误码体系（双向段位划分，daemon → GUI 用 `-32000~-32099`，GUI → daemon 用 `-32100~-32199`）：[SPEC-005 §12](../specs/SPEC-005-ipc-protocol.md#12-错误码体系)
> - 完整方法名清单：[SPEC-005 §11](../specs/SPEC-005-ipc-protocol.md#11-完整方法名清单速查)

以下为 v1 历史描述，**与 SPEC-005 不一致时一律以 SPEC-005 为准**：

除双向 JSON-RPC 请求/响应外，sieve-ipc v2.0 引入两类单向通知消息，用于 daemon → GUI 的状态推送和 CLI → daemon 的触发信号。

#### 6.3.1 `sieve.notify_status_bar`（daemon → GUI，单向）

daemon 通过 `IpcServer::broadcast_status_bar(notify)` 将 `StatusBarNotify` 广播给所有当前连接的 GUI 客户端（多 GUI 支持），dead sender 在广播时 lazy 清理。

方法名：`sieve.notify_status_bar`（无响应，单向发送）

**StatusBarNotify schema**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `notify_id` | Uuid | 通知唯一 ID |
| `created_at` | String | ISO 8601 时间戳 |
| `kind` | NotifyKind | 通知类型枚举（见下表） |
| `title` | String | 状态栏显示标题（短文本） |
| `detail` | Option\<String\> | 可选详情，GUI tooltip 或副标题 |
| `rule_id` | Option\<String\> | 关联规则 ID（如 `IN-SEQ-01`） |
| `auto_dismiss_seconds` | u32 | 自动消失秒数（0 = 常驻直到用户关闭） |

**NotifyKind 枚举**：

| 值 | 触发场景 |
|----|---------|
| `SequenceHit` | 行为序列窗口命中 IN-SEQ-* 序列模式（High，仅 StatusBar 不阻断） |
| `OutboundRedacted` | 出站脱敏（OUT-01~05/12 AutoRedact 命中，静默通知 5s） |
| `UserRulesLoadFailed` | `~/.sieve/rules/user.toml` 加载/lint 失败 |
| `UserRulesReloaded` | 用户规则热加载成功（`sieve rules edit` / `sieve rules enable/disable` 触发） |
| `Generic` | 其他通知（daemon 内部临时信息） |

示例（SSE 风格伪码，实际走 Unix socket JSON-RPC）：
```json
{
  "jsonrpc": "2.0",
  "method": "sieve.notify_status_bar",
  "params": {
    "notify_id": "a1b2c3d4-...",
    "created_at": "2026-05-01T12:34:56Z",
    "kind": "OutboundRedacted",
    "title": "已脱敏：检测到私钥",
    "detail": "OUT-03 命中，已替换为 [REDACTED-PRIVKEY]",
    "rule_id": "OUT-03",
    "auto_dismiss_seconds": 5
  }
}
```

#### 6.3.2 `sieve.reload_user_rules`（CLI → daemon，单向）

CLI 侧（`sieve rules edit` / `sieve rules enable` / `sieve rules disable`）执行完文件改写后，通过 `sieve_ipc::send_reload_user_rules_oneshot(socket_path, trigger_id)` 向 daemon 发送单向信号。

方法名：`sieve.reload_user_rules`（无响应，单向发送）

**ReloadUserRules schema**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `trigger_id` | Option\<Uuid\> | 可选触发 ID，用于日志关联；`null` = CLI 不关心关联 |

daemon 端通过 `IpcServer::reload_rx()` mpsc channel（容量 16）异步消费此信号，触发 LayeredEngine 热替换（ArcSwap atomic swap），并向所有 GUI 广播 `UserRulesReloaded` 状态栏通知。

#### 6.3.3 GUI 控制面方法（v2.1 新增，sieve-gui-macos 接入）

> 完整 schema / 错误码 / Critical 锁校验细节见 [SPEC-005](../specs/SPEC-005-ipc-protocol.md)（IPC 协议权威源）；本节仅做 API 索引。

**daemon → GUI notifications**：

| 方法 | 用途 | 关键字段 |
|------|------|---------|
| `sieve.preset_changed` | preset 模式或 overrides 变化广播 | `mode` / `overrides` / `source` |
| `sieve.paused_changed` | 暂停状态广播 | `paused` / `until` / `reason` / `applies_to` |
| `sieve.request_decision_canceled` | 已发出的 request_decision 被 daemon 取消 | `request_id` / `reason` / `auto_decision` |

**GUI → daemon requests**（全部走 JSON-RPC 2.0 request/response）：

| 方法 | 用途 | 备注 |
|------|------|-----|
| `sieve.set_paused` | 暂停 / 恢复 | `minutes ∈ [0, 60]`；Critical 锁规则不受暂停影响 |
| `sieve.set_preset` | 切 preset 模式 | `mode ∈ {"strict","standard","relaxed","custom"}`（v1 `"default"` → v2 `"standard"`，SPEC-005 §5.6；daemon 仍兼容旧 `"default"`） |
| `sieve.set_preset_overrides` | Custom preset 逐规则覆盖 | **Critical 锁防线二**：`FAIL_CLOSED_RULES` 集合内的规则被 daemon 拒绝并写 audit `kind=critical_lock_blocked` |
| `sieve.reload_config` | 重载 sieve.toml + 用户规则 | 系统规则 lint 失败保留旧规则；用户规则失败跳过该文件 |
| `sieve.health` | daemon 健康摘要 | 无副作用、无审计 |
| `sieve.evaluate` | 沙箱评估给定 payload | 不写 audit / 不动 SessionState；payload ≤ 64KB；Critical 锁规则命中时摘要不含原 payload 片段 |
| `sieve.list_graylist` | 分页列灰名单 | 不返回 `matched_canonical`（避免间接泄露敏感片段）|
| `sieve.remove_graylist` | 按 fingerprint 删灰名单 | 写 audit `kind=graylist_removed` |

**新增错误码**（在 JSON-RPC 标准 error 上叠加）：

| Code | 含义 |
|------|-----|
| `-32000` | `protocol_version_mismatch` |
| `-32001` | `critical_lock_violated` |
| `-32002` | `daemon_busy`（reload 进行中等）|
| `-32003` | `payload_too_large`（evaluate 超 64KB）|
| `-32004` | `unknown_fingerprint`（list / remove graylist）|
| `-32005` | `unsupported_in_paused`（保留）|

---

### 6.4 sieve rules CLI（v2.0 新增）

`sieve rules` 子命令组，用于管理 `~/.sieve/rules/user.toml` 用户规则文件：

#### `sieve rules edit`

```
sieve rules edit
```

行为：
1. 调用 `$EDITOR`（未设置时 fallback 依次尝试 `vim` → `nano`）打开 `~/.sieve/rules/user.toml`
2. 编辑器关闭后执行 TOML lint 校验（规则 id/severity/direction 字段合法性检查）
3. lint 通过后：原文件 atomic backup 到 `~/.sieve/rules/user.toml.bak.YYYYMMDD-HHMMSS`（保留最近 10 份，旧备份自动清理）
4. atomic rename 写入新规则文件
5. 通过 IPC `sieve.reload_user_rules` 通知 daemon 热加载

lint 失败时：打印错误到 stderr，原文件不修改，不发 IPC 信号，退出码非零。

#### `sieve rules list`

```
sieve rules list
```

合并展示当前用户规则状态（带 `enabled`/`disabled` + `direction` 字段）+ 系统规则数量摘要。不修改任何文件。

#### `sieve rules disable <id>`

```
sieve rules disable <rule_id>
```

将 `~/.sieve/rules/user.toml` 中对应规则项的 `enabled` 字段置为 `false`，TOML 序列化 + atomic rename 写回，触发 IPC reload。`<rule_id>` 不存在时退出码非零并打印错误。

#### `sieve rules enable <id>`

```
sieve rules enable <rule_id>
```

将对应规则项的 `enabled` 字段置为 `true`，流程同 `disable`。

**用户规则约束**（`~/.sieve/rules/user.toml`）：
- `severity` 仅允许 `high` / `medium` / `low`（禁止 `critical`）
- `action` 仅允许 `warn` / `mark` / `ask`（禁止 `block` / `hook_terminal`）
- `direction` 取值 `Outbound` / `Inbound` / `Both`（默认 `Both`，兼容旧 user.toml）
- 用户规则与系统规则并存（LayeredEngine 系统规则先行，命中 critical_lock 立即返回不评估用户规则）

**配置文件路径**：
- 用户规则文件：`~/.sieve/rules/user.toml`
- 自动备份（最近 10 份）：`~/.sieve/rules/user.toml.bak.YYYYMMDD-HHMMSS`
- 灰名单存储：`~/.sieve/decisions/<sha256_64_hex>.json`（文件权限 0600，目录权限 0700，atomic rename，no-follow symlink）
- 系统规则目录：`~/.sieve/rules/`（由 `[detection].rules_path` 配置）

---

### 6.4a sieve decisions CLI（2026-05-05 新增）

Headless 决策面 CLI——让 daemon 在 GUI 不在线时仍可用（远程 SSH / GUI crash / tmux 工作流）。CLI 跟 GUI 共用同一组 IPC method，**不引入特权 endpoint**。

#### `sieve decisions watch`

```
sieve decisions watch [--format jsonl] [--severity SEV]
```

流式订阅 daemon 推送的 pending decision events。`--format jsonl` 每行一个 JSON object（默认），方便接 `jq` / `fluentd` / `vector`。`--severity` 过滤（critical / high / medium / low）。Ctrl+C 优雅退出。

#### `sieve decisions show <id>`

```
sieve decisions show <pending-id>
```

查询单个 pending decision 的完整上下文（detection / origin / caller）。默认 pretty-printed JSON。

#### `sieve decisions resolve`

```
sieve decisions resolve <id> --approve [--reason "..."]
sieve decisions resolve <id> --block   [--reason "..."]
sieve decisions resolve <id> --warn    [--reason "..."]
```

解决单个 pending decision；三选一互斥；`--reason` 可选写入 audit。

#### `sieve start --no-client-policy` flag

```
sieve start --no-client-policy {auto-block|auto-warn|hold-and-fail-closed} ...
```

daemon 在无 client 接 IPC 时的兜底策略：
- `auto-block`（默认）：保守 fail-closed，无 client 时直接 deny
- `auto-warn`：标记 warn 自动放行
- `hold-and-fail-closed`：等待超时后按 `default_on_timeout` 处置（v1.x 行为）

实现：`gated_request_decision` 在 `connected_clients == 0` 且非 Critical 时按策略快速返回。

---

### 6.4b sieve audit CLI（2026-05-05 新增）

Unix-pipeable 审计日志查询 CLI——直接读 `~/.sieve/audit.db`（不通过 IPC），输出 jsonl 方便接管道工具。

> **可选特性边界**：`audit tail` / `audit query` / `audit show`（只读查询）**始终可用**，无需任何 feature。
> 加密档案密钥生命周期子命令（`audit keygen` / `audit rotate-key` / `audit decrypt`，见下文）属**可选特性 `audit-crypto`**，
> 默认二进制不含；需以 `--features audit-crypto` 编译才会出现（依赖 `age` / `sha2` / `base64` 仅启用时编译）。
> 未编入时，配置 `[audit].level = "full"` 优雅降级为 `metadata` 档。

#### `sieve audit tail`

```
sieve audit tail [-f|--follow] [--format jsonl|pretty] [--limit N]
```

显示最后 N 条审计事件（默认 N=20）。`--follow` 流式跟踪新事件（500ms 轮询）。`--format jsonl` 每行一个 JSON object。

#### `sieve audit query`

```
sieve audit query [--since DUR] [--severity SEV] [--rule-id RULE] [--provider-id PROVIDER] [--format jsonl|pretty]
```

按条件过滤查询：
- `--since`：时间范围（`1h` / `30m` / `7d` / `24h`）
- `--severity`：critical / high / medium / low
- `--rule-id`：按 rule_id 过滤
- `--provider-id`：按 listener 上游标识过滤（v3 schema 新列）

#### `sieve audit show <id>`

```
sieve audit show <event-id>
```

显示单条事件完整内容（含 raw_json 字段如有）。

**输出 jsonl schema**（对齐 audit_events 表 v3 schema）：

```json
{"id": 1, "timestamp": "2026-05-05T12:34:56Z", "direction": "outbound", "rule_id": "OUT-01", "severity": "Critical", "disposition": "redact", "decision": null, "request_id": "req-001", "provider_id": "anthropic", "caller_pid": 1234, "caller_exe": "/usr/bin/claude", "raw_json": null}
```

`provider_id` 特殊值：
- `_system`：daemon 系统级事件（control plane / oversize / config reload）
- `unknown`：兜底值（v2 老记录 migration 默认值 / 测试 fixture）
- 普通字符串：来自 `sieve.toml [[upstream]] provider_id` 字段

#### 加密审计日志密钥生命周期（可选特性 `audit-crypto`，2026-06-19 新增）

> **可选特性子命令**：以下三个子命令属可选特性 `audit-crypto`，默认二进制**不含**，需 `--features audit-crypto` 编译才可用（依赖 `age` / `sha2` / `base64` 仅启用时编译）。
> 配套 `full` 档加密归档（§3.3.3 `[audit]` 段）。这三个子命令是 `full` 档的密钥生命周期工具——daemon 端只持公钥（write-only logging），解密 / 审计须用持口令的私钥（identity），**应在另一台 / 离线机器执行**。数据模型见 [data-model.md §14](../design/data-model.md)。

##### `sieve audit keygen`

```
sieve audit keygen [--out <path>]
```

生成 age 密钥对：recipient 公钥写入 `config.toml [audit].recipient`；identity 私钥**以口令保护**（age scrypt）写入文件（0600 权限），并强制提示用户把私钥移出本机（密码管理器 / 离线介质），daemon 不留存 identity。

> ⚠ **口令丢失 = 归档永久不可读（by design）**：口令仅保护私钥，daemon 正常运行不需要它。**口令一旦丢失，identity 无法解锁，所有 `full` 档归档永久无法解密**，这不是 bug 而是设计使然。生成后立即备份口令到密码管理器。覆盖已有密钥前需二次确认。

##### `sieve audit rotate-key`

```
sieve audit rotate-key [--out <path>]
```

生成新密钥对，新归档段（archive segment）改用新 recipient；**旧段保持用旧 recipient 加密**（审计旧段需对应旧 identity）。段头记录 key id 便于审计时定位。口令丢失同样不可恢复，警示同 `keygen`。

##### `sieve audit decrypt`

```
sieve audit decrypt --identity <file> [--out <path>]
```

审计解密：口令解锁 identity → 解密归档段 → 校验哈希链（`prev_hash` + `seq`，检出历史改写 / 删除 / 重排）→ 输出**脱敏后**内容。**应在另一台 / 离线机器执行**（daemon 机器不留 identity）。不走 `audit_db_path` / 只读 SQLite 路径，与 §6.4b 的查询子命令语义独立。

---

### 6.4c sieve usage CLI（可选特性 `usage`，2026-06-19 新增）

> **可选特性子命令**：`sieve usage` 属可选特性 `usage`，默认二进制**不含**，需 `--features usage` 编译才可用（依赖 `tiktoken-rs` 仅启用时编译）。
> 只读查询本地 `~/.sieve/usage.db`（独立于 `audit.db`，0600，append-only），输出 jsonl 方便接管道工具。**统计严格本地、永不上传**，无任何出站上报路径。

#### `sieve usage query`

```
sieve usage query [--since DUR] [--provider-id PROVIDER] [--format jsonl|pretty]
```

按条件查询本地 token 用量核算记录：
- `--since`：时间范围（`1h` / `30m` / `7d` / `24h`，复用 `sieve audit` 的 duration 解析）
- `--provider-id`：按 listener 上游标识过滤（同 audit 的 `provider_id` 语义）
- `--format`：`jsonl`（默认，每行一个 JSON object）/ `pretty`

输出含独立计数 vs relay 声明 `usage` 的比对（仅 `relay` 上游有交叉比对值；`official` 直连采纳上游权威 `usage`）。Anthropic 输入侧在 `count_tokens_optin = false`（默认）时标注为**近似估算**。

> **隐私**：`sieve usage` 仅本地可见，无任何上报路径。token 用量恰是 README 发誓「从不上传」的那个 usage record（呼应 [SPEC-006 §9.1](../specs/SPEC-006-update-and-telemetry.md)）。

---

### 6.5 sieve setup / doctor / uninstall 退出码

标准 UNIX 惯例：`0` = 成功，非零 = 失败（具体错误信息打印到 stderr）。

### 6.6 sieve setup --agent 退出码（v1.5 新增）

`sieve setup --agent` 涉及多家 agent 的配置文件修改，退出码区分"全部成功"、"部分失败已回滚"、"回滚也失败"三种状态：

| 退出码 | 含义 | 用户需做什么 |
|--------|------|-------------|
| `0` | 全部 agent 配置注入成功 | 无需操作，可运行 `sieve doctor` 验证 |
| `1` | 至少一个 agent 配置失败，已自动回滚到备份 | 查看 stderr 错误信息，手动检查失败的 agent 配置后重试 |
| `2` | 部分 agent 配置失败，**且回滚也失败**（紧急状态） | 立即查看 stderr 中"需要手动清理"的步骤；备份文件在 `~/.sieve/backups/`；**不要重试，先手动恢复** |

- 失败回滚时，已成功配置的 agent **不回滚**（部分配置成功是可接受状态）
- 回滚失败（exit 2）通常发生在备份文件损坏或磁盘权限变化时，属于罕见错误
- `sieve doctor --agent <name>` 可在任意时刻检查单个 agent 的接入状态，输出 pass/fail 逐项诊断

---

## 7. X-Sieve-Origin Header 协议（v1.5 新增）

> 本章描述 sub-agent 嵌套调用链的 header 协议。

### 7.1 Header 格式

```
X-Sieve-Origin: <source_agent>:<request_id>:<chain_depth>
```

- `source_agent`：调用链发起方标识符，取值见下表
- `request_id`：调用链根请求 UUID（所有嵌套层共享同一 request_id）
- `chain_depth`：当前 hop 的嵌套深度（0 = 根层，1 = 一层嵌套，依此类推）

**source_agent 取值**：

| 值 | 含义 |
|----|------|
| `claude` | Claude Code 直接调用 |
| `hermes` | Hermes Agent 主进程直接调用 |
| `hermes-delegate-claude` | Hermes delegate 给 Claude Code 子进程 |
| `openclaw` | OpenClaw daemon 调用 |

### 7.2 使用示例

```
# 用户直接用 Claude Code（chain_depth=0）
X-Sieve-Origin: claude:a1b2c3d4-...:0

# 用户直接用 Hermes（chain_depth=0）
X-Sieve-Origin: hermes:def45678-...:0

# Hermes delegate 给 Claude Code（chain_depth=1，同一 request_id）
X-Sieve-Origin: hermes-delegate-claude:def45678-...:1
```

### 7.3 chain_depth 语义与弹窗去重

**弹窗去重协议**：
- `chain_depth > 0` + 同 `request_id` 父层已有 `allow` 记录 → 子层弹窗**去重**，不再重复询问用户
- `chain_depth > 0` + 父层无记录（或为 `deny`）→ 正常弹窗，用户决策后**传播给所有 sub-agent**

**强制升级规则**：
- `chain_depth ≥ 2`：**强制 GUI hold**，无论规则本身的 disposition 是什么——嵌套两层以上的调用过于可疑
- `chain_depth ≥ 5`：**直接返回 426**，拒绝处理，写审计日志 `user_decision: "chain_depth_exceeded"`

### 7.4 Header 签名机制（防伪造）

`X-Sieve-Origin` header 由 Sieve 主代理在 Hermes 注入子进程时**签名**（Ed25519），防止攻击者伪造 header 绕过链深度检测：

- 私钥：由 Sieve 主代理在首次启动时生成，保存在 `~/.sieve/keys/origin-signing.key`（权限 0600）
- 公钥：写入 Claude Code 子进程的环境变量，由 Sieve 在每次请求时验证
- 验证失败：无效签名的 header **视同无 header 处理**（chain_depth=0，走正常弹窗流程，不降级也不拒绝）——防止攻击者通过伪造无效 header 触发 chain_depth ≥ 5 的 426 拒服

---

## 8. Manifest 接口（sieve-updater 使用）

> manifest 接口是 Sieve 唯一允许的主动出站接口。完整协议规格见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

### 8.1 请求格式

```
GET https://updates.sieveai.dev/v1/manifest
  ?v=<client_version>       # 必选，语义版本如 "0.3.1"
  &os=<mac|linux|windows>   # 必选，小写
  &arch=<x64|arm64>         # 必选
  &uid=<UUIDv4>             # 必选（SIEVE_NO_TELEMETRY 时省略）
  &ch=<stable|beta>         # 可选，发布通道，默认 stable
```

**传输层约束**：
- 仅 TLS 1.2+
- 不带 cookie / Authorization header
- `User-Agent: sieve-updater/<v>`（仅客户端版本）
- manifest 接口为保证可用性走自有服务器，**不挂 CDN**

**Query 参数说明**：

| 参数 | 类型 | 必选 | 说明 |
|------|------|------|------|
| `v` | string | 是 | Sieve 客户端语义版本（如 `0.3.1`） |
| `os` | string | 是 | 操作系统：`mac` / `linux` / `windows` |
| `arch` | string | 是 | CPU 架构：`x64` / `arm64` |
| `uid` | UUID | 否 | UUIDv4 install-id（`SIEVE_NO_TELEMETRY` 时省略） |
| `ch` | string | 否 | 发布通道：`stable`（默认）/ `beta`（Phase 2 加） |

### 8.2 响应 JSON Schema

```json
{
  "schema": 1,
  "rules": {
    "version": "2026.05.05.1",
    "url": "https://cdn.sieveai.dev/rules/2026.05.05.1.json.zst",
    "sha256": "ab12cd34ef56789012345678901234567890123456789012345678901234abcd",
    "size": 184320,
    "signature": "ed25519:BASE64URL_ENCODED_SIGNATURE"
  },
  "client": {
    "latest": "0.4.0",
    "min_supported": "0.2.0",
    "deprecation_notice": null
  },
  "next_check_after_seconds": 21600
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `schema` | integer | 协议版本号，当前 `1` |
| `rules.version` | string | 规则包版本（`YYYY.MM.DD.N`） |
| `rules.url` | string | 规则包 CDN 地址（HTTPS） |
| `rules.sha256` | string | 规则包 sha256（64 位小写 hex） |
| `rules.size` | integer | 规则包字节数 |
| `rules.signature` | string | `"ed25519:"` 前缀 + base64url 签名 |
| `client.latest` | string | 最新客户端版本 |
| `client.min_supported` | string | 最低支持版本；低于此时 log warn |
| `next_check_after_seconds` | integer | 服务端指定下次检查间隔（覆盖本地 `check_interval_hours`） |

### 8.3 自托管镜像替换（`SIEVE_UPDATE_URL`）

企业内网或离线环境可通过环境变量覆盖默认 manifest URL：

```bash
export SIEVE_UPDATE_URL=https://updates.internal.corp/sieve/v1/manifest
```

或在 `sieve.toml` 中配置：

```toml
[update]
url = "https://updates.internal.corp/sieve/v1/manifest"
```

自托管服务端必须：
- 响应符合 §8.2 JSON schema
- 支持 TLS 1.2+
- 不需要 `Authorization` header（Sieve 客户端不发认证信息）

### 8.4 约束与隐私

- **不带 cookie / Auth header**：manifest 请求匿名，服务端无法关联到用户身份
- **uid 可关闭**：`SIEVE_NO_TELEMETRY=1` 省略 uid 字段，仍能拿到规则更新
- **完全禁用**：`SIEVE_NO_UPDATE=1` 跳过所有更新请求（离线 / CI 场景）
- **签名验证**：规则包下载后强制 ed25519 + sha256 双重校验，签名失败 fail-closed 保留旧规则

详见 [SPEC-006 §5 签名校验](../specs/SPEC-006-update-and-telemetry.md)。

---

## 9. 错误码表

### 9.1 标准 4xx / 5xx（Sieve 透传上游或自身产生）


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


### 9.2 Sieve 自定义 4xx 子段（Critical 拦截相关）


| 状态码                                 | Sieve 语义          | 触发场景                                                                       | Body                     |
| ----------------------------------- | ----------------- | -------------------------------------------------------------------------- | ------------------------ |
| `426 Upgrade Required`              | **Critical 出站阻断** | 出站请求体命中 Critical 规则（如 OUT-12 BIP39 SHA-256 校验通过）                            | `sieve_blocked` JSON     |
| `451 Unavailable For Legal Reasons` | **合规 / 安全策略阻断**   | 规则签名验证失败、配置违反硬约束（如 `bind_address != 127.0.0.1`、`telemetry.enabled = true`） | `sieve_block` JSON 或启动错误 |
| `499 Client Closed Request`         | 用户主动拒绝            | CLI 弹窗用户选择拒绝 / 超时 fail-closed                                              | `sieve_block` JSON       |

**Week 2 实际实现的 426 Body Schema**：

```json
{
  "type": "sieve_blocked",
  "blocked_at": "<UNIX epoch seconds, Phase 1 简化；Week 4 起改 RFC3339>",
  "detections": [
    {
      "rule_id": "OUT-12",
      "severity": "critical",
      "fingerprint": "a3f1c8d9b2e7..."
    }
  ],
  "guidance": {
    "zh": "Sieve 检测到 N 条出站 Critical 命中。请检查后用 .sieveignore 加入 fingerprint 白名单，或重新发送脱敏消息。",
    "en": "Sieve blocked N outbound critical detections. Review your message, then either redact or add fingerprint(s) to .sieveignore."
  }
}
```

> 选用 `426` 是产品语义层的"行为升级要求"（用户需脱敏后重发），与 RFC 7231 协议升级语义有偏差，但**实际客户端不会因此崩溃**（Anthropic SDK 把非 2xx 当作错误并显示 body 文本）。**状态**：候选方案，Week 2-3 dogfood 验证 Claude Code SDK 行为后正式确定。

### 9.3 入站拦截响应（SSE event 注入）

Sieve 检测到入站 Critical 命中（IN-CR-02 危险 shell / IN-CR-05 签名工具 / IN-CR-01 地址替换 / IN-GEN-01/03 等）时，**不等响应完成**，在 SSE 流中**注入一个额外的 sieve_blocked event，然后关闭连接**：

```
event: sieve_blocked
data: {"type":"sieve_blocked","blocked_at":<unix_epoch>,"detections":[{"rule_id":"IN-CR-05-EVM","severity":"critical","fingerprint":"abc123"}],"guidance":{"zh":"...","en":"..."}}

<连接关闭，客户端收到 EOF>
```

**注意**：此前已发送的 SSE event（包括 message_start / content_block_delta 等）不撤回。客户端可能收到不完整流（text 已部分到达但 message_stop 未发）。Claude Code SDK 通常会处理为响应中断，等同于网络错误。

**为何不等完整 message_stop**：tool_use 一旦发到客户端就可能被执行（IN-CR-05 签名工具就是这种风险）；截流必须发生在 content_block_stop 之后、tool_use 真正发出之前。

**关联约束**：fail-closed Critical（出站用 426，入站用 SSE event）。

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)（含 §10 Multi-Agent 扩展架构）
- 数据模型（fingerprint / SQLite schema）：[../design/data-model.md](../design/data-model.md)
- SPEC-004 multi-agent setup（v1.5 新增）：[../specs/SPEC-004-multi-agent-setup.md](../specs/SPEC-004-multi-agent-setup.md)
- 术语表：[../glossary.md](../glossary.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)

---

## 接口冻结声明

- **冻结时间点**：首个稳定版（GA）发布时
- **冻结范围**：本文 §1（反向代理路由）、§2（管理 API + audit schema）、§3（配置文件 schema 顶层字段）、§4（环境变量名）、§5（severity → HTTP 状态码映射）、§6（CLI 退出码 + IPC 单向通知方法名 + sieve rules 子命令签名）、§7（X-Sieve-Origin header 格式 + chain_depth 语义）、§8（manifest 接口 + SIEVE_* env var 名）、§9（自定义错误码 426 / 451 / 499）
- **冻结后变更规则**：
  - **MAJOR**（v2.0.0）：删除字段、改语义、改默认绑定地址、关闭 fail-closed 行为（**永远不会做**）
  - **MINOR**（v1.x.0）：新增可选字段、新增端点、新增检测规则 ID
  - **PATCH**（v1.0.x）：bug 修复、文档纠错、性能优化
- **检测规则 ID 不视为接口** —— 规则增删走 [CHANGELOG](../changelog/CHANGELOG.md) 记录，不触发 SemVer

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。任何 API 变更必须同步更新 [CHANGELOG](../changelog/CHANGELOG.md)。

