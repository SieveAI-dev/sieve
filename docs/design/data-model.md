# Sieve 数据模型设计

> **状态**：设计阶段 / 锁定执行
> **文档版本**：v1.0 / 2026-04-27
> **依据 PRD**：[`docs/prd/sieve-prd-v1.3.md`](../prd/sieve-prd-v1.3.md)
> **范围**：Phase 1 内部数据结构、配置文件、审计日志、规则签名、license 数据格式

---

## 1. UnifiedMessage 内部表示

### 1.1 设计原则

- **Phase 1 实质上是 Anthropic Messages API 的内部映射**——所有字段语义对齐 Anthropic，避免提前泛化导致的胶水成本；
- **接口预留**：`UnifiedMessage` 设计成可扩展，但 Phase 1 **不实现** OpenAI / OpenRouter 适配（详见 [ADR-004](./ADR-004-anthropic-first-unified-interface.md)）；
- 所有检测器只与 `UnifiedMessage` 交互，不直接访问原始 JSON ——这是未来可加适配器的基础。

### 1.2 Rust 伪代码

```rust
pub struct UnifiedMessage {
    pub role: Role,
    pub content_blocks: Vec<ContentBlock>,
    pub tool_uses: Vec<ToolUseBlock>,
    pub tool_results: Vec<ToolResultBlock>,
    pub metadata: MessageMetadata,
}

pub enum Role { System, User, Assistant, Tool }

pub enum ContentBlock {
    Text { text: String, span: ContentSpan },
    Image { source: ImageSource, span: ContentSpan },
}

pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub raw_partial: Option<String>,
    pub span: ContentSpan,
}

pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: Vec<ContentBlock>,
    pub is_error: bool,
}

pub struct MessageMetadata {
    pub session_id: SessionId,
    pub direction: Direction,
    pub upstream_provider: UpstreamProvider,
    pub received_at: Instant,
}

pub enum Direction { Outbound, Inbound }
pub enum UpstreamProvider { Anthropic, Relay(String) }
```

### 1.3 字段说明

| 字段 | 含义 | Phase 1 实现 |
|------|------|-------------|
| `role` | 角色（system/user/assistant/tool） | ✅ 与 Anthropic 一致 |
| `content_blocks` | 文本 + 图片块（Anthropic 原生 multi-modal） | ✅ 仅 Text，Image 透传不扫描 |
| `tool_uses` | assistant 发起的工具调用 | ✅ 必须聚合到完整 JSON 才检测（Tool Use Aggregator） |
| `tool_results` | tool 角色返回的结果 | ✅ Phase 1 仅记录元信息，不深入检测 |
| `metadata.session_id` | 用于 AddressGuard 维护对话历史 | ✅ |
| `metadata.direction` | Outbound / Inbound | ✅ |
| `raw_partial` | partial JSON 增量缓冲（流式 SSE 必需） | ✅ |

**接口预留点**：`UpstreamProvider` 枚举、`ContentBlock` 的扩展形态、`ToolUseBlock.input` 用 `serde_json::Value`（不强 schema），以便未来加 OpenAI function calling / OpenRouter routing 元信息时不需要重写检测器。

---

## 2. 检测结果数据结构

```rust
pub struct Detection {
    pub id: Uuid,                      // Phase 1 用 uuid::Uuid v4（crates/sieve-core/src/detection.rs）
    pub rule_id: String,
    pub severity: Severity,
    pub action: Action,                // Week 2 新增：Block / Redact / WarnConfirm / MarkOnly / SilentLog
    pub source: ContentSource,         // Week 2 新增：命中文本来源标识
    pub span: ContentSpan,
    pub evidence_truncated: String,    // Week 2 改名（原 evidence）：脱敏后的证据片段，绝不含原始密钥
    pub fingerprint: String,           // SHA-256("{rule_id}:{normalized_content}") 前 8 字节 = 16 hex 字符
}

pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

pub enum Action {
    Block,
    Redact { placeholder: String },
    WarnConfirm { countdown_secs: u8 },
    MarkOnly,
    SilentLog,
}

pub struct ContentSpan {
    pub offset: usize,
    pub length: usize,
}

pub enum ContentSource {
    OutboundUserText,
    OutboundSystemText,
    InboundAssistantText,
    InboundToolUseInput { tool_use_id: String },
}
```

字段说明：

- `rule_id`：稳定字符串 ID，例如 `OUT-09`、`IN-CR-05`、`IN-GEN-02`，与 PRD §5.1 / §5.2 表格一一对应；
- `evidence_truncated`：**只存最小必要片段**（如脱敏后的密钥前缀 + 字符数），**绝不存完整 prompt**；详见 §6 审计日志；
- `fingerprint`：用于 `.sieveignore` 白名单匹配（详见 §4），格式见 §4.1；
- `Action::Redact` 的 `placeholder`：标准占位符（如 `[REDACTED-PRIVATE-KEY]`），不做差异化生成。

---

## 3. 处置矩阵编码

> 默认映射，可被 `config.toml` 中 `severity_overrides` 修改，但 Critical 不可关闭（详见 [ADR-007](./ADR-007-fail-closed-critical-actions.md)）。

| Severity | 默认 Action | 用户可见 | 可否被 config 覆盖 |
|----------|------------|---------|-------------------|
| Critical | `Block` + `WarnConfirm{countdown=0}`（强制确认） | 全屏 / 阻塞式弹窗 | ❌ 不可（任何 override 启动时被忽略并打印警告） |
| High | `WarnConfirm{countdown=5}` | 弹窗 + 5 秒倒计时 | ✅ 可降级到 `MarkOnly` |
| Medium | `MarkOnly` | 状态栏图标 / SQLite 记录 | ✅ 可升级或降级 |
| Low | `SilentLog` | 仅 SQLite | ✅ 可升级到 `MarkOnly` |

特殊规则映射（详见 PRD §5.1、§5.2）：

| Rule ID | 默认 Severity | 备注 |
|---------|--------------|------|
| OUT-01 ~ OUT-05 | Critical | 各类高确定度密钥 |
| OUT-06、OUT-08 | High | ETH / Solana 私钥 entropy 边界模糊 |
| OUT-07、OUT-09、OUT-10 | Critical | WIF / BIP39 / Keystore 有结构化校验位 |
| OUT-11 | Medium | .env 仅 Medium，避免误伤 |
| IN-CR-05 | Critical（**永不可降级**） | 签名工具调用 |
| IN-GEN-01~03 | Critical（**永不可降级**） | rm -rf / curl\|sh / eval(base64) |

---

## 4. `.sieveignore` 文件格式

### 4.1 fingerprint 算法

```
fingerprint = sha256("{rule_id}:{content_normalized}")[:16]
```

- `content_normalized`：UTF-8、NFC、移除前后空白；密钥类内容截断到前 32 字节计算（避免长度差异导致 fingerprint 不稳定）；
- 截取前 16 字符（hex）≈ 64 bit，碰撞概率在单用户规模可忽略；
- 不同 rule_id 即使内容相同也得到不同 fingerprint —— 防止"忽略 OUT-06 ETH 私钥"误覆盖到"OUT-09 BIP39"。

### 4.2 文件语法

- 一行一条 `<fingerprint>` 或 `<fingerprint> # 备注`；
- `#` 起始的整行视为注释；
- 空行忽略；
- **不支持**环境变量展开 / glob / 正则 —— 故意保持简单，避免被攻击者通过环境变量注入跳过检测。

### 4.3 示例

```
# .sieveignore — Sieve 学习型白名单
# 路径：~/.sieve/sieveignore（也可放仓库根目录 .sieveignore）
# 格式：每行一个 16 字符 hex fingerprint，可加 # 注释

# ===== OUT-04 JWT =====
a1b2c3d4e5f60708  # demo JWT used in unit tests

# ===== OUT-09 BIP39 =====
9988776655443322  # canary mnemonic (honeypot wallet only)

# ===== IN-CR-01 AddressGuard =====
deadbeef00112233  # known dev fixture address 0x000...dead
```

### 4.4 加载行为

- 启动时一次性加载，存入 `HashSet<Fingerprint>`，O(1) 查询；
- 文件变更需重启 daemon（Phase 1 不做热加载）；
- 加载失败（语法错误）启动时报错并拒绝启动 —— 不静默跳过。

---

## 5. 配置文件 `~/.sieve/config.toml`

### 5.1 完整字段表

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `upstream_url` | string | `"https://api.anthropic.com"` | 上游 API base URL；可指[redacted] |
| `port` | u16 | `11453` | 本地监听端口 |
| `bind_addr` | string | `"127.0.0.1"` | 监听地址，**禁止**改成 `0.0.0.0`（启动时报错） |
| `rules_path` | string | `"~/.sieve/rules-v{N}.tar.zst"` | 规则文件路径 |
| `sieveignore_path` | string | `"~/.sieve/sieveignore"` | 白名单路径 |
| `log_path` | string | `"~/.sieve/audit.db"` | 审计日志 SQLite 路径 |
| `log_level` | enum | `"info"` | trace / debug / info / warn / error |
| `license_key` | string | `""` | JWT-like license（详见 §8） |
| `severity_overrides` | table | `{}` | 规则级 severity 覆盖（Critical 规则的覆盖会被忽略） |
| `telemetry_enabled` | bool | `false` | **强制 false**，启动时如设 true 会被强制改回并打印警告 |
| `tls_verify_upstream` | bool | `true` | 是否校验上游 TLS 证书；改 false 启动时打印 WARN |
| `dry_run` | bool | `false` | 干跑模式：Critical 命中只 tracing::warn! 记录，不返 426，继续转发上游（用于规则调试）。CLI `--dry-run` flag 出现即覆盖为 true。 |

### 5.2 示例

```toml
# ~/.sieve/config.toml
upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"

rules_path = "~/.sieve/rules.tar.zst"
sieveignore_path = "~/.sieve/sieveignore"
log_path = "~/.sieve/audit.db"
log_level = "info"

license_key = "eyJhbGciOiJFZERTQSJ9..."

# Severity 覆盖示例：把 OUT-11 .env 升级到 High
[severity_overrides]
"OUT-11" = "High"

# 下面这条会被启动时忽略并打印警告（Critical 不可降级）
"IN-CR-05" = "High"  # 启动 WARN: ignored, IN-CR-05 is fail-closed Critical

telemetry_enabled = false  # 强制 false，写其它值无效
tls_verify_upstream = true
```

---

## 6. 本地审计日志 SQLite Schema

### 6.1 文件与隔离

- 路径：`~/.sieve/audit.db`
- 权限：`0600`（仅当前 user 可读写）
- 写入模式：append-only（通过 `BEFORE UPDATE` / `BEFORE DELETE` 触发器拒绝任何修改）
- **不存原始 prompt 内容**——只存 fingerprint 与最小元信息（PRD §11.2）

### 6.2 `events` 表

```sql
CREATE TABLE events (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp       INTEGER NOT NULL,        -- unix ms
  session_id      TEXT    NOT NULL,
  direction       TEXT    NOT NULL CHECK (direction IN ('outbound','inbound')),
  severity        TEXT    NOT NULL CHECK (severity IN ('critical','high','medium','low')),
  rule_id         TEXT    NOT NULL,
  fingerprint     TEXT    NOT NULL,        -- 16-hex
  action_taken    TEXT    NOT NULL,        -- block/redact/warn_confirm/mark/silent
  user_choice     TEXT,                    -- allow_once/cancel/redact_send/null
  evidence_meta   TEXT                     -- JSON: { "len": N, "prefix": "sk-ant-...", ... }
);

CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_session   ON events(session_id);
CREATE INDEX idx_events_rule      ON events(rule_id);

CREATE TRIGGER events_no_update BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;

CREATE TRIGGER events_no_delete BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;
```

### 6.3 字段语义

- `evidence_meta`：JSON 字符串，**仅存元信息**（长度、前缀几个字符、entropy 值），用于事后分析 FP 而不是回放 prompt；
- `user_choice`：仅 Critical / High 在用户交互后写入；Medium / Low 为 NULL；
- `session_id`：与 Claude Code 单次进程绑定（启动时随机），**不**做跨进程关联。

> 即使审计文件被攻击者读到，也只能拿到"此用户在此时间点触发过 OUT-09 BIP39 检测"，无法还原任何敏感内容。这是 [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 在数据层的兑现。

---

## 7. 规则签名文件格式

### 7.1 文件命名

```
rules-v{N}.tar.zst       # 规则包（zstd 压缩 tar）
rules-v{N}.sig           # Ed25519 签名（detached）
rules-v{N}.manifest.json # 元信息
```

- `N` 为单调递增整数；
- 客户端只接受 `N >= current_N`，拒绝降级以防回滚攻击。

### 7.2 `manifest.json` 字段

```json
{
  "version": 42,
  "created_at": "2026-04-26T00:00:00Z",
  "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
  "rule_count": 187,
  "signer_pubkey_id": "sieve-release-2026"
}
```

- `sha256`：`rules-vN.tar.zst` 内容的 SHA-256（不含签名文件）；
- `signer_pubkey_id`：用于在多 key rotation 时定位公钥（Phase 1 只用一个 key，但格式预留）。

### 7.3 验证流程

1. 下载 `rules-vN.tar.zst` + `.sig` + `.manifest.json`
2. 用**硬编码在二进制中的 Ed25519 公钥**校验 `.sig` 是否对 `(manifest.json || tar.zst)` 的拼接合法
3. 校验 `manifest.json.sha256` 与 `tar.zst` 实际 hash 一致
4. 校验 `manifest.json.version > current_version`
5. 全部通过后原子替换；失败则保留旧规则并打印 `WARN`

> Ed25519 公钥**必须**在编译期硬编码（`const SIGNER_PUBKEY: [u8; 32] = ...`），不读 disk / 不读 env，防止配置注入攻击。

### 7.4 规则条目 schema（`outbound.toml` / `inbound.toml`）

规则包解压后的 TOML 格式，每个规则文件包含 `[[rules]]` 数组：

```toml
# crates/sieve-rules/rules/outbound.toml schema
[[rules]]
id           = "OUT-01"            # 规则 ID，跨版本不复用
description  = "Anthropic API key"
pattern      = "sk-ant-api03-[a-zA-Z0-9_\\-]{93}AA"  # vectorscan PCRE 子集
severity     = "critical"          # low / medium / high / critical
action       = "block"             # allow / mark / warn / block
entropy_min  = 3.5                 # Week 2 新增：最低 Shannon entropy（可选，缺省不校验）
keywords     = ["sk-ant"]          # Week 2 新增：vectorscan 预筛选 hint
allowlist_regexes   = ['''xxx+'''] # Week 2 新增：per-rule allowlist 正则
allowlist_stopwords = ["EXAMPLE"]  # Week 2 新增：per-rule allowlist 停止词
```

对应 `RuleEntry` Rust 结构体（`crates/sieve-rules/src/lib.rs`）扩展字段：
- `entropy_min: Option<f32>`：Shannon entropy 下限，低于此值不升 Critical
- `keywords: Vec<String>`：vectorscan 预筛选 hint，全部命中才进正则校验
- `allowlist_regexes: Vec<String>`：每条规则独立的白名单正则，命中则跳过此规则
- `allowlist_stopwords: Vec<String>`：每条规则独立的停止词白名单

---

## 8. License Key 数据结构

### 8.1 格式

JWT-like，**Ed25519 签名**（不用 RSA / HMAC）：

```
<header_b64url>.<payload_b64url>.<signature_b64url>
```

- `header = { "alg": "EdDSA", "kid": "sieve-license-2026" }`
- `payload`：claims 见下表
- `signature`：Ed25519 over `header.payload`

### 8.2 Claims

| 字段 | 类型 | 含义 |
|------|------|------|
| `sub` | string | 用户 email 的 SHA-256 hash（前 16 hex）—— **不存明文 email** |
| `iat` | int | 签发时间（unix s） |
| `exp` | int | 过期时间（trial：iat+14d；pro：iat+1y 或 iat+30d 滚动） |
| `tier` | string | `"trial"` 或 `"pro"` |
| `device_limit` | int | 设备数上限（Phase 1：trial=1，pro=3） |

### 8.3 验证流程

```
1. 启动时读取 config.license_key
2. 用编译期硬编码的 license 公钥校验 signature
3. 校验 exp > now()
4. 校验 tier 合法
5. 通过 → 进入 license 启用模式
   失败 / 缺失 / 过期 → 进入 §7.1 降级模式（只读警告，不再 Critical 拦截）
```

### 8.4 关键性质

- **完全离线验证**：本地公钥 + 签名校验，**不联网 verify**（详见 [ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）；
- 设备绑定：用 `device_id`（macOS：硬件 UUID hash / Linux：machine-id hash）+ `sub` 组合的本地 SQLite 记录；超过 `device_limit` 时 daemon 拒绝在新设备上启动；
- 撤销机制：Phase 1 **不做** CRL —— [redacted]没有撤销基础设施，离线验证 + exp 已经覆盖 99% 场景；
- **降级模式（降级触发 / license 失效）行为矩阵**——直接对齐 [PRD §7.1 + §9 #8](../prd/sieve-prd-v1.3.md#71-单一定价)：
  - **Critical**：**全部仍然 fail-closed 拦截**（包括出站 OUT-* 与入站 IN-CR-05 / IN-GEN-01~03 等所有 Critical 规则）。这是产品安全承诺，不是付费用户特权（详见 [ADR-007](./ADR-007-fail-closed-critical-actions.md)）；
  - **High**：从"弹窗 + 5 秒倒计时"降级为"仅审计记录，不弹窗"（即"只读警告"）；
  - **Medium / Low**：行为不变（本就静默 / 标记）；
  - **审计日志、规则签名验证、不联网 verifier**（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）等数据安全机制全部不受影响。

---

## 9. 相关文档

- [PRD-sieve v1.3](../prd/sieve-prd-v1.3.md) §5、§6.6、§7、§11
- [architecture.md](./architecture.md) —— 模块职责、性能预算、Phase 2 演进路径
- [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) —— 不联网 verifier 的硬约束
- [ADR-004](./ADR-004-anthropic-first-unified-interface.md) —— UnifiedMessage 接口预留
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— 二进制签名与可复现构建
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical 永不可关的产品承诺
- `docs/api/api-reference.md` —— 待编写
