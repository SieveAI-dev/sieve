# Sieve API 参考

> **状态：实现阶段（Pre-GA），接口未冻结。**
> 当前文档反映 PRD v2.0/v2.1 的实现事实（v1.4/v1.5 章节保留，v2.0/v2.1 新增内容标注"v2.0 新增"或"v2.1 新增"），**v1 接口将在 Week 12 GA 时冻结**，破坏性变更走 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)。
> 在冻结前，任何字段、状态码、配置项的细节都可能调整。

---

## 0. 文档定位

Sieve **不是** 面向开发者的 SDK / library，而是一个本地 HTTP 反向代理，因此本文所述的"API"分为三层：

1. **反向代理 API**（对 Claude Code 等上游客户端暴露） —— 透明转发 Anthropic Messages 协议
2. **本地管理 API**（仅 `127.0.0.1`） —— 健康检查、白名单管理、审计查询、规则刷新
3. **配置文件 schema + 环境变量 + CLI 退出码** —— 二进制运行时契约

> **不存在云端 API。**Sieve 完全本地运行，绝不联网做 token verification（[PRD §9](../prd/sieve-prd-v1.5.md) 硬约束 #2）。

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
- 详见 [ADR-018](../design/ADR-018-openai-protocol-adaptation.md)

---

**底线约束（PRD §9 第 11 条）**：**绝不在 Anthropic API 协议层伪造 `tool_use` / `stop_reason` / `id` / `usage` / `type` 字段**。这是产品承诺，不是实现细节。

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

> 上游真实 endpoint 由 `[upstream].url` 决定（默认 `https://api.anthropic.com`），用户可指[redacted]。

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

**v2.0 IPC 协议扩展**（详见 [ADR-021](../design/ADR-021-tri-state-decision-and-graylist.md) 三态决策）：

- `sieve.request_decision` 新增字段：
  - `allow_remember: bool` —— **daemon 端计算**该值（不让 GUI 决定）。当 `rule_id` 在 `critical_lock::FAIL_CLOSED_RULES` 时强制 false（PRD §9 #3 + ADR-007 fail-closed 不被绕过）
- `sieve.decision_response` 新增字段：
  - `remember: bool` —— GUI 用户在弹窗勾选 "永久允许"。daemon 收到 true 时**必须二次校验** rule_id 是否允许 Remember；不允许则忽略 + 写 audit ERROR
  - `context_hint: Option<String>` —— GUI 表单输入的备注（"Vitalik 地址 read-only balanceOf 调用" 等），写入灰名单 JSON
- 灰名单存储：`~/.sieve/decisions/<digest>.json`（文件名 hex digest，0600 权限，atomic rename，no-follow symlink；所有变更写 audit.db）
- 内置 Critical 规则的 GUI 弹窗 Remember checkbox **必须 disabled+灰显**，tooltip 解释"内置 Critical 规则保护核心安全场景，不允许永久绕过"

**出站 AutoRedact（OUT-01~05/06/08/11/12）**：
- 请求 body 中敏感内容自动脱敏替换，**不返 426**，直接转发到上游，上游响应直通
- 状态栏静默通知（不打断用户流程，[PRD §9 第 13 条](../prd/sieve-prd-v1.5.md)）

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
- **降级模式不影响入站 Critical**：降级触发用户的入站 Critical 仍然 fail-closed（PRD §9 #8 + §7.1）

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


> **绝不返回原文。**只返回 fingerprint + 元信息（[PRD §9](../prd/sieve-prd-v1.5.md) 硬约束 #2 + §11.3 数据本地化）。详细字段定义见 [data-model.md](../design/data-model.md)。

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
| `SequenceHit` | 行为序列窗口（ToolUseSequence）匹配到 IN-SEQ-* kill chain，发状态栏通知 |
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
| `[detection]`    | `dry_run`             | bool    | `false`                                          | 干跑模式：命中 Critical 只记录不返 426，用于调试规则。CLI `--dry-run` flag 覆盖为 true |
| `[detection]`    | `severity_overrides`  | table   | `{}`                                             | 子表，按 `rule_id` 覆盖默认 severity（仅可降级；**Critical 不可关闭**）            |
| `[storage]`      | `audit_db_path`       | path    | `"~/.sieve/audit.db"`                            | SQLite append-only 审计库                                          |
| `[storage]`      | `log_path`            | path    | `"~/.sieve/logs/sieve.log"`                      | 文本日志，按天 rotate                                                  |
| `[storage]`      | `log_level`           | enum    | `"info"`                                         | `trace` / `debug` / `info` / `warn` / `error`                   |
| `[license]`      | `key`                 | string? | `null`                                           | License key；缺失时进入 试用期                                        |
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

## 4. 环境变量与自动配置

### 4.1 `sieve setup` 自动配置（推荐）

v1.4 不再需要手动 `export ANTHROPIC_BASE_URL`。首次安装后运行：

```bash
sieve setup
```

`sieve setup` 自动完成（详见 [ADR-015](../design/ADR-015-sieve-setup-tool.md) / [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）：
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
| `SIEVE_LICENSE_KEY` | sieve 进程 | 否 | 无 | 覆盖 `[license].key` |
| `SIEVE_LOG_LEVEL` | sieve 进程 | 否 | `info` | 覆盖 log 级别，取值 `trace`/`debug`/`info`/`warn`/`error` |
| `SIEVE_RULES_PATH` | sieve 进程 | 否 | `$SIEVE_HOME/rules` | 覆盖规则路径 |
| `SIEVE_DISABLE_RULES_UPDATE` | sieve 进程 | 否 | 未设置 | 设为 `1` 时禁用规则自动更新（仅离线场景） |

> 环境变量优先级 **高于** 配置文件，但 **低于** CLI flag（如有）。`SIEVE_HOME` 是总控变量，设置后无需分别覆盖各子路径。

**v1.5 multi-agent 相关（无新环境变量）**：

- multi-agent setup 本身**不引入新的 Sieve 环境变量**——`sieve setup --agent openclaw|hermes` 修改的是目标 agent 自己的配置文件（`config.toml` / `.env`），不是 Sieve 的 env var
- Hermes 启动 Claude Code 子进程时，通过 `ANTHROPIC_DEFAULT_HEADERS` env var 自动注入 `X-Sieve-Origin` header，**用户无需手动配置**；Sieve 仅读取这个 header，不负责写入
- OpenClaw 通过 `X-Sieve-Source-Channel` header（OpenClaw 自身注入）传递来源 channel 元数据，Sieve 读取后用于 IN-GEN-06 外部 channel injection 检测

---

## 5. 处置矩阵 → HTTP 行为（v1.4 二维矩阵）

| Disposition | 适用规则 | 出站/入站 | 代理 HTTP 行为 | 备注 |
|-------------|---------|---------|----------------|------|
| **AutoRedact** | OUT-01~05/12 | 出站 | `200 OK` + 改写后的请求 body 转发上游；上游响应**直通**，**不返 426** | 自动脱敏，不打断用户流程（PRD §9 第 13 条） |
| **AutoRedact** | OUT-06/08 | 出站 | 同上 | ETH/Solana entropy 边界模糊，脱敏继续 |
| **GuiPopup**（出站） | OUT-07/09/10 | 出站 | hold 请求；GUI 弹窗；用户允许 → `200 OK` + 原文/脱敏后转发；用户拒绝 → `426 Upgrade Required` + `sieve_blocked` JSON | 高确定性助记词/私钥，Sieve 差异化点 |
| **HookTerminal** | IN-CR-02/03/04，IN-GEN-01~03 | 入站 | `200 OK`（SSE **原样透传**）+ 写 `~/.sieve/pending/<id>.json`；HTTP 层无变化 | sieve-hook 在 PreToolUse 阶段拦截，Claude Code 自行报告拒绝 |
| **GuiPopup**（入站） | IN-CR-01/05，IN-GEN-04 | 入站 | `200 OK`（SSE hold）+ 每 25s `: keep-alive\n\n` comment；用户允许 → 继续流；用户拒绝/超时 → 注入 `sieve_blocked` event + EOF | 代理 hold 住 SSE 流等 GUI 决策（最长 120s） |
| **StatusBar** | OUT-11，IN-GEN-05 | 出/入站 | 透传（`200 OK`，不修改流）+ IPC `sieve.event_notify` 菜单栏通知 | 不打断用户，低优先级通知 |

**说明**：
- AutoRedact 出站**不返 426**——脱敏后直接转发，是"帮用户擦屁股"哲学的体现
- HookTerminal 入站**代理不修改 SSE 流**——拦截在 PreToolUse 执行边界，而非 message 边界（[ADR-014](../design/ADR-014-dual-layer-defense.md)）
- Critical 在所有版本（含降级/降级触发模式）不可关闭（[PRD §9 #8](../prd/sieve-prd-v1.5.md)）；降级模式下 High 仅审计记录，但 Critical 的 GuiPopup/HookTerminal 行为不变

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
> - `allow_remember = false` 时 daemon 收到 `remember=true` 必须二次校验 + 写 audit ERROR（PRD §5.4.2 三道防线之防线二）
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
| `allow_remember` | bool | `false` | **v2.0 新增**。由 daemon 端通过 `is_critical_locked(rule_id)` 计算后传入，**不由 GUI 决定**。内置 Critical 规则（`FAIL_CLOSED_RULES`）强制为 false，旧 v1.5 客户端不发此字段时 `#[serde(default)]` 兼容为 false。GUI 收到 false 时 **Remember checkbox 必须 disabled+灰显**（PRD §5.4.3） |

**DecisionResponse 字段表（v2.0 扩展）**：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `request_id` | Uuid | — | 对应 DecisionRequest.request_id |
| `decision` | String | — | `"allow"` / `"deny"` |
| `decided_at` | String | — | ISO 8601 时间戳 |
| `by_user` | bool | — | true = 用户主动操作；false = 超时自动处理 |
| `remember` | bool | `false` | **v2.0 新增**，`#[serde(default)]`。GUI 用户勾选"永久允许"后为 true；daemon 收到 true 时**必须二次校验** rule_id 是否允许 Remember（不允许则忽略并写 audit ERROR），此为 v2.0 双路校验路径（PRD §5.4.2） |
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
| `SequenceHit` | 行为序列窗口命中 IN-SEQ-* kill chain（High，仅 StatusBar 不阻断） |
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

#### 6.3.3 GUI 控制面方法（v2.1 新增，sieve-gui-macos PRD v1.0 接入）

> 完整 schema / 错误码 / Critical 锁校验细节见 [ADR-013 Supplement 2026-05-02](../design/ADR-013-ipc-protocol.md#supplement-2026-05-02--v20-gui-控制面方法扩展)；本节仅做 API 索引。

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
| `sieve.set_preset` | 切 preset 模式 | `mode ∈ {"strict","default","relaxed","custom"}` |
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

`sieve rules` 子命令组（PRD §5.5.2），用于管理 `~/.sieve/rules/user.toml` 用户规则文件：

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

> 本章对应 PRD v2.0 §6.5 + [ADR-019](../design/ADR-019-x-sieve-origin-header.md)，描述 sub-agent 嵌套调用链的 header 协议。

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

详见 [ADR-019](../design/ADR-019-x-sieve-origin-header.md)。

---

## 8. 错误码表

### 8.1 标准 4xx / 5xx（Sieve 透传上游或自身产生）


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


### 8.2 Sieve 自定义 4xx 子段（Critical 拦截相关）


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

> 选用 `426` 是产品语义层的"行为升级要求"（用户需脱敏后重发），与 RFC 7231 协议升级语义有偏差，但**实际客户端不会因此崩溃**（Anthropic SDK 把非 2xx 当作错误并显示 body 文本）。**ADR-008 状态**：候选，Week 2-3 dogfood 验证 Claude Code SDK 行为后正式落 ADR（见 [ADR-INDEX](../design/ADR-INDEX.md)）。

### 8.3 入站拦截响应（SSE event 注入）

Sieve 检测到入站 Critical 命中（IN-CR-02 危险 shell / IN-CR-05 签名工具 / IN-CR-01 地址替换 / IN-GEN-01/03 等）时，**不等响应完成**，在 SSE 流中**注入一个额外的 sieve_blocked event，然后关闭连接**：

```
event: sieve_blocked
data: {"type":"sieve_blocked","blocked_at":<unix_epoch>,"detections":[{"rule_id":"IN-CR-05-EVM","severity":"critical","fingerprint":"abc123"}],"guidance":{"zh":"...","en":"..."}}

<连接关闭，客户端收到 EOF>
```

**注意**：此前已发送的 SSE event（包括 message_start / content_block_delta 等）不撤回。客户端可能收到不完整流（text 已部分到达但 message_stop 未发）。Claude Code SDK 通常会处理为响应中断，等同于网络错误。

**为何不等完整 message_stop**：tool_use 一旦发到客户端就可能被执行（IN-CR-05 签名工具就是这种风险）；截流必须发生在 content_block_stop 之后、tool_use 真正发出之前。

**关联 ADR-007 §1**（fail-closed Critical），ADR-008 候选（出站用 426，入站用 SSE event）。

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)（含 §10 Multi-Agent 扩展架构）
- 数据模型（fingerprint / SQLite schema）：[../design/data-model.md](../design/data-model.md)
- ADR 索引：[../design/ADR-INDEX.md](../design/ADR-INDEX.md)
- ADR-006 sigstore + reproducible build：[../design/ADR-006-sigstore-reproducible-build.md](../design/ADR-006-sigstore-reproducible-build.md)
- ADR-018 OpenAI 协议适配（v1.5 新增）：[../design/ADR-018-openai-protocol-adaptation.md](../design/ADR-018-openai-protocol-adaptation.md)
- ADR-019 X-Sieve-Origin header 协议（v1.5 新增）：[../design/ADR-019-x-sieve-origin-header.md](../design/ADR-019-x-sieve-origin-header.md)
- SPEC-004 multi-agent setup（v1.5 新增）：[../specs/SPEC-004-multi-agent-setup.md](../specs/SPEC-004-multi-agent-setup.md)
- 术语表：[../glossary.md](../glossary.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)

---

## 接口冻结声明

- **冻结时间点**：Week 12 GA（参见 [PRD §10.2 Week 12](../prd/sieve-prd-v1.5.md#102-phase-b闭测阶段week-9-12)）
- **冻结范围**：本文 §1（反向代理路由）、§2（管理 API + audit schema）、§3（配置文件 schema 顶层字段）、§4（环境变量名）、§5（severity → HTTP 状态码映射）、§6（CLI 退出码 + IPC 单向通知方法名 + sieve rules 子命令签名）、§7（X-Sieve-Origin header 格式 + chain_depth 语义）、§8（自定义错误码 426 / 451 / 499）
- **冻结后变更规则**：
  - **MAJOR**（v2.0.0）：删除字段、改语义、改默认绑定地址、关闭 fail-closed 行为（**永远不会做**）
  - **MINOR**（v1.x.0）：新增可选字段、新增端点、新增检测规则 ID
  - **PATCH**（v1.0.x）：bug 修复、文档纠错、性能优化
- **检测规则 ID 不视为接口** —— 规则增删走 [CHANGELOG](../changelog/CHANGELOG.md) 记录，不触发 SemVer

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。任何 API 变更必须同步更新 [CHANGELOG](../changelog/CHANGELOG.md)。

