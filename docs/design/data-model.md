# Sieve 数据模型设计

> **状态**：设计阶段 / 锁定执行
> **文档版本**：v2.0 / 2026-05-01
> **依据 PRD**：[`docs/prd/sieve-prd-v2.0.md`](../prd/sieve-prd-v2.0.md)
> **范围**：Phase 1 内部数据结构、配置文件、审计日志、规则签名、license 数据格式；v2.0 新增灰名单 schema、user.toml schema、HoldOutcome 枚举、AuditEvent v2

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

## 3. 处置矩阵编码（v1.4 二维矩阵）

> v1.4 重构为二维矩阵（出站/入站 × 严重度），引入 `Disposition` 执行路径字段，替代原一维四级模型（详见 [ADR-016](./ADR-016-disposition-matrix-2d.md)）。Critical 不可关闭（[ADR-007](./ADR-007-fail-closed-critical-actions.md)）。

### 3.1 Disposition 枚举

对应 `crates/sieve-rules/src/manifest.rs` + `crates/sieve-ipc/src/protocol.rs` 中的 `Disposition` 枚举（两处定义镜像，避免循环依赖）：

| Disposition | 执行路径 | 代理行为 | 用户界面 |
|-------------|---------|---------|---------|
| `AutoRedact` | 出站主路径 | 自动脱敏改写 body bytes 后转发，**不弹窗，不返 426** | 状态栏静默显示 |
| `GuiPopup` | 入站 GUI 类 / 出站例外 | hold SSE 流，每 25s 发 keep-alive comment；经 IPC 通道 A 通知 Native GUI App | HIPS 弹窗（倒计时 + 授权/拒绝/typed data 详情） |
| `HookTerminal` | 入站 Hook 类 | **不修改 SSE 流**，写 `~/.sieve/pending/<id>.json`；由 sieve-hook 在 PreToolUse 阶段拦截 | TTY y/n 倒计时（终端内联） |
| `StatusBar` | 低优先级通知 | 透传 + IPC 通道 A 发 `sieve.event_notify` | 菜单栏图标/数字徽章 |

**DefaultOnTimeout 枚举**（对应 `manifest.rs` 中 `DefaultOnTimeout`）：
- `Block`：超时 fail-closed，Critical 规则**只允许此值**
- `Redact`：超时后脱敏转发（出站可选）
- `Allow`：超时放行（仅 IN-GEN Relaxed preset 可用）

### 3.2 二维矩阵默认推导

| 方向 × 严重度 | 默认 Disposition | 备注 |
|-------------|----------------|------|
| Outbound × Critical | `AutoRedact` | 出站"帮用户擦屁股不打断"哲学 |
| Outbound × High | `AutoRedact` | 同上 |
| Outbound × Medium | `StatusBar` | |
| Outbound × Low | `StatusBar` | |
| Inbound × Critical | `GuiPopup` | Hook 类规则用 manifest 显式覆盖为 `HookTerminal` |
| Inbound × High | `GuiPopup` | |
| Inbound × Medium | `StatusBar` | |
| Inbound × Low | `StatusBar` | |

矩阵推导仅用于过渡期兼容；规则 TOML 中**必须显式写 `disposition`**（Week 5 后强制）。

### 3.3 规则特殊映射表

| Rule ID | Severity | Disposition | timeout_seconds | default_on_timeout | 备注 |
|---------|----------|-------------|----------------|--------------------|------|
| OUT-01~05/12 | Critical | `AutoRedact` | — | — | 高确定度密钥，自动脱敏不打断 |
| OUT-06/08 | High | `AutoRedact` | — | — | ETH/Solana entropy 边界模糊，脱敏继续 |
| OUT-07/09/10 | Critical | `GuiPopup` | 60 | Block | **Sieve 差异化点**：校验位通过的高确定性助记词/私钥，出站必须人工确认（覆盖矩阵默认 AutoRedact） |
| OUT-11 | Medium | `StatusBar` | — | — | .env 仅状态栏，避免误伤 |
| IN-CR-01 | Critical | `GuiPopup` | 60 | Block | 地址替换，需展示地址对比 diff |
| IN-CR-02/03 | Critical | `HookTerminal` | 30 | Block | 危险 shell / 敏感路径 |
| IN-CR-04（9 条） | Critical | `HookTerminal` | 60 | Block | 持久化机制 |
| IN-CR-05 | Critical | `GuiPopup` | 120 | Block | 签名工具调用，120s 读 typed data |
| IN-GEN-01~03 | High | `HookTerminal` | 30 | Block | 危险 shell / 远程脚本 / 编码执行 |
| IN-GEN-04 | Medium | `GuiPopup` | 30 | Block | markdown exfil |
| IN-GEN-05 | Low | `StatusBar` | — | — | 低风险通知 |

**可覆盖规则**（`config.toml severity_overrides`）：
- OUT-11、IN-GEN-04、IN-GEN-05 可降级；
- Critical 规则（OUT-01~05/07/09/10/12、IN-CR-01~05、IN-GEN-01~03 为 High 但 HookTerminal）**不可关闭**，覆盖会被启动时忽略并打印警告。

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
| `upstream_url` | string | `"https://api.anthropic.com"` | 上游 API base URL；可指向中转站 |
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
| `dry_run` | bool | `false` | 干跑模式：Critical 命中只 tracing::warn! 记录，不拦截，继续转发上游（用于规则调试）。CLI `--dry-run` flag 出现即覆盖为 true。 |
| `ipc_socket_path` | path | `"~/.sieve/ipc.sock"` | Unix socket 路径（IPC 通道 A，代理 ↔ GUI App）；可被 `SIEVE_HOME` 覆盖 |
| `pending_dir` | path | `"~/.sieve/pending/"` | Hook 类规则 pending 文件目录（IPC 通道 B）；`sieve setup` 自动创建（权限 0700） |
| `decisions_dir` | path | `"~/.sieve/decisions/"` | Hook 类规则 decisions 文件目录（IPC 通道 B）；`sieve setup` 自动创建（权限 0700） |
| `preset` | enum | `"default"` | 规则集预设：`"strict"` / `"default"` / `"relaxed"` / `"custom"`；影响 StatusBar 类规则阈值，**不影响 Critical**（详见 [ADR-016](./ADR-016-disposition-matrix-2d.md)） |
| `launchd_plist_path` | path | `"~/Library/LaunchAgents/com.sieve.daemon.plist"` | launchd plist 路径（macOS only）；`sieve setup` 生成并 `launchctl bootstrap` |
| `gui_socket_enabled` | bool | `true` | 是否启用 GUI App Unix socket（IPC 通道 A）；设为 `false` 时降级为 macOS `osascript` 系统通知，不弹 HIPS 弹窗（仍 fail-closed） |

### 5.2 `~/.sieve/` 目录结构

`sieve setup` 自动创建，所有目录权限 `0700`，文件权限 `0600`：

```
~/.sieve/
├── config.toml           # 主配置文件
├── ipc.sock              # Unix socket（代理 ↔ GUI App，IPC 通道 A）
├── pending/              # Hook 类规则 pending 文件（代理写，sieve-hook 读）
├── decisions/            # Hook 类规则 decisions 文件（sieve-hook 写，代理读）
├── locks/                # fd-lock 文件锁（防止并发读写）
├── audit.db              # SQLite 审计日志（append-only）
├── rules/                # 已签名规则包目录
├── sieveignore           # 白名单 fingerprint 文件
├── backups/              # sieve setup 改动的原始文件备份
└── setup.log             # sieve setup/doctor/uninstall 操作日志（追加格式）
```

### 5.3 示例

```toml
# ~/.sieve/config.toml
upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
preset = "default"

rules_path = "~/.sieve/rules"
sieveignore_path = "~/.sieve/sieveignore"
log_path = "~/.sieve/audit.db"
log_level = "info"

# IPC（sieve setup 自动配置，通常不需手动设）
ipc_socket_path = "~/.sieve/ipc.sock"
pending_dir = "~/.sieve/pending/"
decisions_dir = "~/.sieve/decisions/"
gui_socket_enabled = true

# macOS 守护（sieve setup 自动写入）
launchd_plist_path = "~/Library/LaunchAgents/com.sieve.daemon.plist"

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
- 当前 schema 版本：**v2**（`PRAGMA user_version = 2`）

### 6.2 `events` 表（schema v2）

```sql
PRAGMA user_version = 2;

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
  evidence_meta   TEXT,                    -- JSON: { "len": N, "prefix": "sk-ant-...", ... }
  caller_pid      INTEGER,                 -- v2 新增：调用方进程 PID（NULL 若反查失败）
  caller_exe      TEXT                     -- v2 新增：调用方可执行文件路径（NULL 若反查失败）
);

CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_session   ON events(session_id);
CREATE INDEX idx_events_rule      ON events(rule_id);

CREATE TRIGGER events_no_update BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;

CREATE TRIGGER events_no_delete BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;
```

### 6.2a schema v1 → v2 迁移

启动时检查 `PRAGMA user_version`，若为 1 则在单个事务内执行：

```sql
BEGIN;
ALTER TABLE events ADD COLUMN caller_pid INTEGER;
ALTER TABLE events ADD COLUMN caller_exe TEXT;
PRAGMA user_version = 2;
COMMIT;
```

迁移完成后重建 append-only 触发器（SQLite ALTER TABLE 不触发触发器重建，需显式 DROP + RECREATE）。迁移失败则回滚并打印 `ERROR`，daemon 拒绝启动。

### 6.3 字段语义

- `evidence_meta`：JSON 字符串，**仅存元信息**（长度、前缀几个字符、entropy 值），用于事后分析 FP 而不是回放 prompt；
- `user_choice`：仅 Critical / High 在用户交互后写入；Medium / Low 为 NULL；
- `session_id`：与 Claude Code 单次进程绑定（启动时随机），**不**做跨进程关联；
- `caller_pid` / `caller_exe`（v2 新增）：由 accept loop 进程上下文反查注入（[ADR-023](./ADR-023-process-context-audit.md)）；macOS `proc_listpids` + `proc_pidfdinfo` 4-tuple 匹配；反查失败时存 NULL，不阻断请求处理。

> 即使审计文件被攻击者读到，也只能拿到"此用户在此时间点触发过 OUT-09 BIP39 检测"，无法还原任何敏感内容。这是 [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 在数据层的兑现。

---

## 6. 入站 session 状态（Week 3 新增）

### 6.1 InboundFilter SessionState（IN-CR-01）

`InboundFilter` 内部 `SessionState`：

```rust
pub struct SessionState {
    /// 当前会话内出现过的 ETH 地址（lowercase 归一化）
    /// 用于 IN-CR-01 AddressGuard 对比：相同放行，Levenshtein distance ∈[1,3] 且 len 相等触发 Critical
    pub addresses_seen: HashSet<String>,
}
```

Phase 1 per-session 内存，**不跨进程持久化**。Week 5 配置完善后评估 address book 持久化方案。

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
- 撤销机制：Phase 1 **不做** CRL —— 一人公司没有撤销基础设施，离线验证 + exp 已经覆盖 99% 场景；
- **降级模式（试用过期 / license 失效）行为矩阵**——直接对齐 [PRD §7.1 + §9 #8](../prd/sieve-prd-v1.5.md#71-单一定价)：
  - **Critical**：**全部仍然 fail-closed 拦截**（包括出站 OUT-* 与入站 IN-CR-05 / IN-GEN-01~03 等所有 Critical 规则）。这是产品安全承诺，不是付费用户特权（详见 [ADR-007](./ADR-007-fail-closed-critical-actions.md)）；
  - **High**：从"弹窗 + 5 秒倒计时"降级为"仅审计记录，不弹窗"（即"只读警告"）；
  - **Medium / Low**：行为不变（本就静默 / 标记）；
  - **审计日志、规则签名验证、不联网 verifier**（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）等数据安全机制全部不受影响。

---

## 9. AuditEvent 枚举（v2.0 扩展）

### 9.1 CallerContext 共享子结构

所有 v2.0 新增 AuditEvent variant 均携带 `CallerContext`，由进程上下文反查（[ADR-023](./ADR-023-process-context-audit.md)）填充：

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallerContext {
    pub pid: Option<i32>,   // 调用方 PID；反查失败时 None
    pub exe: Option<String>, // 调用方可执行文件路径；反查失败时 None
}
```

### 9.2 v2.0/v2.1 新增 variant

```rust
pub enum AuditEvent {
    // ===== v1.5 已有 variant（略）=====

    // ===== v2.0 新增 =====

    /// 用户在 HIPS 弹窗作出允许/拒绝决策
    DecisionMade {
        rule_id: String,
        decision: String,          // "allow" | "deny"
        severity: String,
        by_user: bool,             // true = 用户主动决策；false = 超时 fail-closed
        caller: CallerContext,
        request_id: Uuid,
        timestamp: i64,            // unix ms
    },

    /// 用户允许并选择"记住"，灰名单新增一条
    GraylistAdded {
        rule_id: String,
        fingerprint: String,       // 64-hex SHA-256
        caller: CallerContext,
        expires_at: Option<i64>,   // None = 永不过期
        request_id: Uuid,
        timestamp: i64,
    },

    /// 用户尝试对 Critical 规则添加灰名单，被三道防线锁定拒绝
    GraylistCriticalRejected {
        rule_id: String,
        caller: CallerContext,
        request_id: Uuid,
        timestamp: i64,
    },

    /// 灰名单写入 I/O 失败（v2.1 新增）
    GraylistAddFailed {
        rule_id: String,
        error: String,
        request_id: Uuid,
        caller: CallerContext,
        timestamp: i64,
    },

    /// 命中灰名单，跳过 IPC 弹窗直接放行
    GraylistHit {
        rule_id: String,
        fingerprint: String,       // 64-hex
        caller: CallerContext,
        request_id: Uuid,
        timestamp: i64,
    },

    /// IN-SEQ-* 行为序列命中（仅 StatusBar 通知，不阻断）
    SequenceHit {
        rule_id: String,           // "IN-SEQ-01" | "IN-SEQ-02" | "IN-SEQ-03"
        description: String,       // 人类可读的序列描述
        caller: CallerContext,
        session_id: String,
        timestamp: i64,
    },

    /// 用户规则热加载成功
    UserRulesReloaded {
        success: bool,
        error: Option<String>,     // 加载失败时的错误描述
        rule_count: u32,           // 成功加载的规则条数
        timestamp: i64,
    },

    /// 用户规则加载失败（daemon 以 None engine 继续运行）
    UserRulesLoadFailed {
        error: String,
        timestamp: i64,
    },
}
```

所有新增 variant 中 `caller` 字段标注 `#[serde(default)]`，保证从旧格式反序列化时不报错。

---

## 10. 灰名单 schema（v2.0 PRD §5.4.2）

### 10.1 文件位置与权限

```
~/.sieve/decisions/<sha256_64_hex>.json
```

- 文件名 = fingerprint（64 hex 字符，SHA-256），命名即内容索引；
- 文件权限 `0600`，目录权限 `0700`；
- 写入方式：先写临时文件（同目录），再 `rename` 原子替换，防止写入中断导致文件损坏。

### 10.2 Entry JSON schema

```json
{
  "schema_version": 1,
  "fingerprint_version": 1,
  "rule_id": "IN-CR-02",
  "rule_version": "v2.0",
  "fingerprint": "<64 hex>",
  "fingerprint_inputs": {
    "rule_id": "IN-CR-02",
    "matched_canonical": "<归一化命中文本>",
    "tool_name": "bash",
    "protocol": "anthropic",
    "content_kind": "tool_use_input",
    "source_agent": "claude"
  },
  "decision": "allow",
  "expires_at": null,
  "added_at": 1746057600000,
  "added_by": "gui_user_decision",
  "context_hint": "用户备注（可选）",
  "match_count_since": 0,
  "audit_event_id": "<uuid>"
}
```

### 10.3 fingerprint 计算

```
fingerprint = sha256(
  rule_id + ":" +
  matched_canonical + ":" +
  tool_name + ":" +
  protocol + ":" +
  content_kind + ":" +
  source_agent
)
```

lookup 时**重新计算** fingerprint 并与文件名比对，防篡改。`matched_canonical` 为命中文本的 UTF-8 NFC 归一化 + 前导/尾随空白去除形式。

### 10.4 Critical 锁三道防线

1. **daemon 侧**：`DecisionRequest.allow_remember = true` 时，若 rule severity = Critical，daemon 拒绝调用 `graylist::add_entry`，直接返回 `GraylistCriticalRejected` audit event；
2. **IPC 协议侧**：`DecisionResponse` 中 Critical 规则的 `allow_remember` 字段服务端强制置 false，GUI 侧无法绕过；
3. **文件侧**：`graylist::add_entry` 在写入前再次校验 rule severity，Critical 直接 `return Err(GraylistError::CriticalRuleLocked)`。

---

## 11. user.toml schema（v2.0 PRD §5.5）

### 11.1 顶层结构

```toml
schema_version = 1
created_at = "2026-05-01T00:00:00Z"  # RFC 3339
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
# ... UserRuleEntry 字段（见 §11.2）
```

路径：`~/.sieve/rules/user.toml`；`#[serde(deny_unknown_fields)]` 严格校验，未知字段直接拒绝加载。

### 11.2 UserRuleEntry 字段表

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | 用户规则 ID，格式 `USR-NNN`，跨版本不复用 |
| `description` | string | ✅ | 人类可读描述 |
| `pattern` | string | ✅ | vectorscan PCRE 子集正则 |
| `severity` | enum | ✅ | `low` / `medium` / `high`（用户规则**不允许** `critical`） |
| `action` | enum | ✅ | `mark` / `warn` / `block` |
| `keywords` | `Vec<String>` | — | vectorscan 预筛选 hint |
| `allowlist_stopwords` | `Vec<String>` | — | per-rule 停止词白名单 |
| `disposition` | enum | — | 显式指定 Disposition（覆盖矩阵推导） |
| `direction` | enum | — | `Outbound` / `Inbound` / `Both`（默认 `Both`） |
| `enabled` | bool | — | 默认 `true`；`false` 时规则加载但跳过匹配 |
| `added_at` | string | — | RFC 3339 时间戳（`sieve rules add` 自动填充） |
| `added_by` | string | — | `gui` / `cli` / `manual` |

### 11.3 RuleDirection 枚举

```rust
pub enum RuleDirection {
    Outbound, // 仅在出站 Filter Pipeline 的 LayeredEngine 生效
    Inbound,  // 仅在入站 Filter Pipeline 的 LayeredEngine 生效
    Both,     // 默认：出/入站都注册
}
```

`direction` 字段决定用户规则被注入到哪侧 `LayeredEngine`。系统规则的方向由 manifest.toml 中 `direction` 字段控制（同一格式）。

### 11.4 lint 11 类约束

加载 `user.toml` 时执行以下 lint（任一失败 → `UserRulesLoadFailed` + daemon 以 `None` user engine 继续运行）：

1. `schema_version` 必须为 1；
2. 每条规则 `id` 不得与系统规则 ID 冲突；
3. 同一文件内 `id` 不得重复；
4. `severity` 不得为 `critical`；
5. `pattern` 必须能被 vectorscan 成功编译；
6. `keywords` 中每个字符串长度 ≥ 3；
7. `allowlist_stopwords` 不得为空字符串；
8. `action = "block"` 且 `severity = "low"` 时发出 WARN（可能误伤）；
9. `direction` 值必须是合法枚举成员；
10. `enabled` 字段若缺失默认 `true`（不报错）；
11. 整个文件大小不超过 256 KB。

详细 lint 规则和错误码见 [ADR-020](./ADR-020-user-rules-system.md)。

---

## 12. HoldOutcome 枚举（v2.0/v2.1）

`HoldOutcome` 是入站 hold 流路径（`inbound_hold.rs`）从用户决策到执行路径的内部枚举，由 IPC `DecisionResponse` 映射而来。

```rust
pub enum HoldOutcome {
    /// 用户允许，放行 SSE 流
    /// remember: 是否写灰名单；context_hint: GUI 用户备注
    Allow {
        remember: bool,
        context_hint: Option<String>,
    },

    /// 用户允许但请求脱敏后放行（出站可选路径）
    RedactAndAllow {
        remember: bool,
        context_hint: Option<String>,
    },

    /// 用户拒绝（或超时 fail-closed）
    Deny {
        reason: String, // "user_denied" | "timeout_fail_closed"
    },
}
```

### 12.1 与 IPC DecisionResponse 的字段映射

| DecisionResponse 字段 | HoldOutcome 字段 | 说明 |
|----------------------|----------------|------|
| `decision = "allow"` | `Allow { .. }` | — |
| `decision = "allow_redact"` | `RedactAndAllow { .. }` | 仅出站支持 |
| `decision = "deny"` | `Deny { reason: "user_denied" }` | — |
| 超时（无响应） | `Deny { reason: "timeout_fail_closed" }` | Critical only |
| `allow_remember` | `remember` | Critical 规则服务端强制 false |
| `context_hint` | `context_hint` | 透传用户备注 |

---

## 13. 相关文档

- [PRD-sieve v2.0](../prd/sieve-prd-v2.0.md) §5、§6.6、§7、§11
- [architecture.md](./architecture.md) —— 模块职责、性能预算、Phase 2 演进路径
- [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) —— 不联网 verifier 的硬约束
- [ADR-004](./ADR-004-anthropic-first-unified-interface.md) —— UnifiedMessage 接口预留
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— 二进制签名与可复现构建
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical 永不可关的产品承诺
- [ADR-020](./ADR-020-user-rules-system.md) —— 用户规则系统（user.toml lint 规则）
- [ADR-021](./ADR-021-tri-state-decision-and-graylist.md) —— 三态决策 + 灰名单（含 Critical 锁）
- [ADR-022](./ADR-022-behavior-sequence-window.md) —— 行为序列窗口（IN-SEQ-*）
- [ADR-023](./ADR-023-process-context-audit.md) —— 进程上下文反查（caller_pid / caller_exe）
- [ADR-024](./ADR-024-rules-engine-abstraction.md) —— 规则引擎抽象 + LayeredEngine
- [ADR-025](./ADR-025-content-type-routing-matrix.md) —— content-type 路由矩阵
- `docs/api/api-reference.md` —— 待编写
