# Sieve 数据模型设计

> **状态**：设计阶段 / 锁定执行
> **文档版本**：v2.0 / 2026-05-01
> **范围**：内部数据结构、配置文件、审计日志、规则签名、license 数据格式；灰名单 schema、user.toml schema、HoldOutcome 枚举、AuditEvent v2

---

## 1. UnifiedMessage 内部表示

### 1.1 设计原则

- **以 Anthropic Messages API 为内部映射基准**——字段语义对齐 Anthropic，避免提前泛化导致的胶水成本；
- **双协议已落地**：`UnifiedMessage` 同时承载 Anthropic Messages API 与 OpenAI Chat Completions 两套协议的请求/响应，由 ProviderCodec 协议编解码层归一化；
- 所有检测器只与 `UnifiedMessage` 交互，不直接访问原始 JSON ——这是协议适配层与检测层解耦的基础。

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

**扩展点**：`UpstreamProvider` 枚举、`ContentBlock` 的扩展形态、`ToolUseBlock.input` 用 `serde_json::Value`（不强 schema）——OpenAI function calling 已经过 ProviderCodec 归一化进同一套检测器，新增 provider routing 元信息时也不需要重写检测器。

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

- `rule_id`：稳定字符串 ID，例如 `OUT-09`、`IN-CR-05`、`IN-GEN-02`，与出站 / 入站规则定义一一对应；
- `evidence_truncated`：**只存最小必要片段**（如脱敏后的密钥前缀 + 字符数），**绝不存完整 prompt**；详见 §6 审计日志；
- `fingerprint`：用于 `.sieveignore` 白名单匹配（详见 §4），格式见 §4.1；
- `Action::Redact` 的 `placeholder`：标准占位符（如 `[REDACTED-PRIVATE-KEY]`），不做差异化生成。

---

## 3. 处置矩阵编码（v1.4 二维矩阵）

> v1.4 重构为二维矩阵（出站/入站 × 严重度），引入 `Disposition` 执行路径字段，替代原一维四级模型。Critical 不可关闭。

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
- 不同 rule_id 即使内容相同也得到不同 fingerprint —— 防止"忽略 OUT-06 ETH 私钥"误覆盖到"OUT-14 BIP39"。

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

# ===== OUT-14 BIP39 =====
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
| `upstream_url` | string | `"https://api.anthropic.com"` | **已废弃**（向后兼容）：上游 API base URL；映射为单元素 `[[upstream]]`（`protocol = auto`，按 path 自适应、保留双协议）；multi-listener 后推荐用 `[[upstream]]` 数组（见下方 §5.1a） |
| `port` | u16 | `11453` | **已废弃**（向后兼容）：本地监听端口 |
| `[[upstream]]` | array | `[]` | **推荐**：multi-listener 配置数组，每项含 `port` / `url` / `provider_id` / `protocol`（见 §5.1a） |
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
| `preset` | enum | `"standard"` | 规则集预设：`"strict"` / `"standard"` / `"relaxed"` / `"custom"`（v1 旧值 `"default"` 在 v2 重命名为 `"standard"`，SPEC-005 §5.6；config 仍兼容旧 `"default"` alias）；影响 StatusBar 类规则阈值，**不影响 Critical** |
| `launchd_plist_path` | path | `"~/Library/LaunchAgents/com.sieve.daemon.plist"` | launchd plist 路径（macOS only）；`sieve setup` 生成并 `launchctl bootstrap` |
| `gui_socket_enabled` | bool | `true` | 是否启用 GUI App Unix socket（IPC 通道 A）；设为 `false` 时降级为 macOS `osascript` 系统通知，不弹 HIPS 弹窗（仍 fail-closed） |

### 5.1a `[[upstream]]` 数组（multi-listener）

每项 listener 配置：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `port` | u16 | (必填) | 监听端口；同一 daemon 内必须唯一（启动时端口冲突 → exit 1） |
| `url` | string | (必填) | 真实上游 endpoint（**含 path 前缀**，如 `https://api.deepseek.com/anthropic`） |
| `provider_id` | string | URL host 派生 | 上游身份标识；写入 audit `provider_id` 列 + IPC `ListenerSnapshot.provider_id` |
| `protocol` | enum | `"auto"` | `"auto"`（默认）\| `"anthropic"` \| `"openai"`。`auto` 按请求 path 自适应路由（`/v1/messages` → Anthropic，`/v1/chat/completions` → OpenAI）、**不做错位拒绝**；仅显式声明 `anthropic`/`openai` 时才对 path 错位 fail-closed 400 拒绝 |

向后兼容：`[[upstream]]` 为空时，从旧 `upstream_url` + `port` 字段映射成单元素 vec
（`provider_id = "anthropic"`，`protocol = auto`）；省略 `protocol` 字段的 `[[upstream]]` 同样
映射为 `auto`，按 path 自适应、保留单 upstream 双协议能力。两套配置同时给时，`[[upstream]]`
优先，旧字段 ignored 并打 WARN。

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
preset = "standard"

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
- **不存原始 prompt 内容、不存任何命中原文**——见下方 §6.3「脱敏契约（规范性）」
- 当前 schema 版本：**v3**（`PRAGMA user_version = 3`；`full` 档启用时迁 v4，见 §14.6）

### 6.2 `audit_events` 表（schema v3，权威源 = `crates/sieve-cli/src/audit.rs` `CREATE_TABLE_DDL`）

> **表名 = `audit_events`**（不是 `events`）。此前本节误写为 `events` + 含 `evidence_meta` 列，与实现（`audit.rs`）及 §14 归档段所用的 `audit_events` 不一致；GUI（`sieve-gui-macos`）曾按旧 schema 读 `events.evidence_meta`，应同步对齐到本节真实 schema（读 `audit_events`、移除 `evidence_meta` 期望）。

```sql
PRAGMA user_version = 3;

CREATE TABLE IF NOT EXISTS audit_events (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp_rfc3339   TEXT    NOT NULL,    -- RFC 3339 UTC（非 unix ms）
  direction           TEXT    NOT NULL,    -- 'outbound' | 'inbound'
  rule_id             TEXT    NOT NULL,    -- 命中规则 ID（如 IN-CR-05 / OUT-01）
  severity            TEXT    NOT NULL,    -- 'Critical' | 'High' | 'Medium' | 'Low'
  disposition         TEXT    NOT NULL,    -- 'redact' | 'mark' | 'pending' | 'resolved' | 'notify'
  decision            TEXT,                -- 'Allow' | 'Block' | NULL（无用户交互时）
  request_id          TEXT    NOT NULL,    -- 请求 UUID（与 IPC request_id 串联）
  raw_json            TEXT,                -- 序列化的 AuditEvent 元数据 JSON；【绝不含命中原文/密钥/前缀】，见 §6.3
  caller_pid          INTEGER,             -- 调用方 PID（NULL 表示反查失败）
  caller_exe          TEXT,                -- 调用方可执行路径（NULL 表示反查失败）
  provider_id         TEXT    NOT NULL DEFAULT 'unknown'  -- listener 上游身份标识（'_system' = 系统事件）
);

-- append-only：拒绝 UPDATE / DELETE
CREATE TRIGGER IF NOT EXISTS no_update BEFORE UPDATE ON audit_events
BEGIN SELECT RAISE(FAIL, 'audit_events is append-only: UPDATE is forbidden'); END;

CREATE TRIGGER IF NOT EXISTS no_delete BEFORE DELETE ON audit_events
BEGIN SELECT RAISE(FAIL, 'audit_events is append-only: DELETE is forbidden'); END;
```

### 6.2a schema v1 → v2 迁移

启动时检查 `PRAGMA user_version`，若为 1 则在单个事务内执行：

```sql
BEGIN;
ALTER TABLE audit_events ADD COLUMN caller_pid INTEGER;
ALTER TABLE audit_events ADD COLUMN caller_exe TEXT;
PRAGMA user_version = 2;
COMMIT;
```

迁移失败则回滚并打印 `ERROR`，daemon 拒绝启动。

### 6.2b schema v2 → v3 迁移（multi-listener Stage E）

启动时检查 `PRAGMA user_version`，若为 2 则在单个事务内执行：

```sql
BEGIN;
ALTER TABLE audit_events ADD COLUMN provider_id TEXT NOT NULL DEFAULT 'unknown';
PRAGMA user_version = 3;
COMMIT;
```

`provider_id` 列记录每条事件命中哪个 listener 上游。NOT NULL DEFAULT `'unknown'`
保证旧记录填上兜底值；引入 multi-listener 后所有 `audit.append` 调用都从
`RequestCtx.listener_provider_id` 显式传入。特殊值：`_system`（系统级事件，无 listener
上下文）/ `unknown`（v2 老记录迁移默认 / 测试 fixture）/ 普通字符串（来自
`sieve.toml [[upstream]] provider_id` 或 URL host 派生）。迁移失败回滚 + `ERROR` + 拒绝启动。

### 6.3 脱敏契约（规范性）+ 字段语义

**脱敏契约（D-1，规范性，安全不变量）**：

- `audit_events` 表的**任何列都不得持久化命中原文 / 原始密钥 / 助记词 / 地址，也不得存其前缀片段**。
- `raw_json` 列仅写**已序列化的 `AuditEvent` 元数据**（`kind` / `rule_id` / `severity` / `request_id` / `caller`），命中证据字段（`Detection.evidence_truncated` 等）**不进入** `AuditEvent`，故不落库；出站脱敏类事件 `raw_json` 恒为 `NULL`（见 `daemon.rs` `OutboundRedacted { raw_json: None }`）。当前实现全 crate **零** `raw_json: Some(<原文>)` 写入路径。
- **禁止**后续给本表新增任何含命中原文/前缀的列（如 `evidence` / `evidence_meta` / `matched_text`）。GUI 硬约束「不存储原始 prompt / 命中片段，evidence 只在内存持有」在数据层的兑现：GUI 若需展示命中详情，**MUST** 从 IPC `request_decision` 的内存态 `detections[].details` 实时取（弹窗关闭即丢弃），**不得**从 audit.db 读取。
- 违反本契约（任一列出现明文/前缀）视为 P0 安全缺陷。

**字段语义**：

- `raw_json`：见上，仅元数据 JSON，多数事件为 `NULL`。
- `decision`：仅在用户对 Critical / High 交互后写入（`Allow` / `Block`）；无交互为 `NULL`。
- `disposition`：处置方式（`redact` / `mark` / `pending` / `resolved` / `notify`）。
- `caller_pid` / `caller_exe`（v2 新增）：由 accept loop 进程上下文反查注入；macOS `proc_listpids` + `proc_pidfdinfo` 4-tuple 匹配；反查失败存 `NULL`，不阻断请求处理。
- `provider_id`（v3 新增）：本条事件命中的 listener 上游身份；系统级事件填 `_system`；来源链：`sieve.toml [[upstream]] provider_id` → `RequestCtx.listener_provider_id` → `AuditStore::append(event, provider_id)`。

> 即使审计文件被攻击者读到，也只能拿到"此用户在此时间点触发过某规则（如 OUT-14 BIP39）检测"，无法还原任何敏感内容。这是"完全本地、绝不联网 verifier"原则在数据层的兑现。

---

## 6. 入站 session 状态（Week 3 新增）

### 6.1 InboundFilter SessionState（IN-CR-01）

`InboundFilter` 内部 `SessionState`：

```rust
pub struct SessionState {
    /// 当前会话内出现过的 ETH 地址（lowercase 归一化）
    /// 用于 IN-CR-01 AddressGuard 对比：完全相同放行，近似偏差（落在阈值内）触发地址替换告警
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
| `exp` | int | 过期时间（unix s） |
| `tier` | string | 许可级别标识 |
| `device_limit` | int | 设备数上限 |

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

- **完全离线验证**：本地公钥 + 签名校验，**不联网 verify**；
- 设备绑定：用 `device_id`（macOS：硬件 UUID hash / Linux：machine-id hash）+ `sub` 组合的本地 SQLite 记录；超过 `device_limit` 时 daemon 拒绝在新设备上启动；
- 撤销机制：Phase 1 **不做** CRL —— 离线验证 + exp 已经覆盖 99% 场景；
- **降级模式（license 缺失 / 失效）行为矩阵**——Critical 不可关闭的安全承诺在任何许可状态下不变：
  - **Critical**：**全部仍然 fail-closed 拦截**（包括出站 OUT-* 与入站 IN-CR-05 / IN-GEN-01~03 等所有 Critical 规则）。这是产品安全承诺，对任何许可状态都不变；
  - **High**：从"弹窗 + 5 秒倒计时"降级为"仅审计记录，不弹窗"（即"只读警告"）；
  - **Medium / Low**：行为不变（本就静默 / 标记）；
  - **审计日志、规则签名验证、不联网 verifier**等数据安全机制全部不受影响。

---

## 9. AuditEvent 枚举（v2.0 扩展）

### 9.1 CallerContext 共享子结构

所有 v2.0 新增 AuditEvent variant 均携带 `CallerContext`，由进程上下文反查填充：

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

## 10. 灰名单 schema（v2.0）

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

## 11. user.toml schema（v2.0）

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

详细 lint 规则和错误码见 sieve-policy crate 的 user.toml 校验实现。

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

## 13. manifest 客户端字段约定

> 本节记录 manifest 请求（`updates.sieveai.dev`）的客户端发送字段约定与服务端隐私原则，供客户端和服务端对齐。manifest 协议详细规格见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

### 13.1 客户端发送字段

manifest 请求随规则更新检查携带以下匿名字段（每天 4 次，复用更新通道，不新增独立网络访问）：

| 字段 | 含义 | 备注 |
|------|------|------|
| `uid`  | UUIDv4 install-id | `SIEVE_NO_TELEMETRY` 时省略 |
| `v`    | 客户端版本（如 `"0.3.1"`） | — |
| `os`   | 操作系统：`mac` / `linux` / `windows` | — |
| `arch` | CPU 架构：`x64` / `arm64` | — |
| `ch`   | 发布通道（默认 `stable`） | — |

字段白名单与开关（`SIEVE_NO_TELEMETRY` 等）见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) §9.1。

### 13.2 服务端隐私原则

- **原始 IP 不落盘**：geoip 解析后丢弃（或哈希后保留 ≤7 天用于反滥用，过期硬删）
- **uid 可为 NULL**：`SIEVE_NO_TELEMETRY` 省略 uid 时，请求中不包含安装标识符
- **服务端按日去重**：原始 IP 不持久化
- 不存 User-Agent 详情、不存 Referer、不存任何用户输入相关字段

---

## 14. 加密审计归档（`full` 档）

> 本节描述三档 logging level 与 `full` 档加密归档的数据模型。**核心红线**：归档**只存脱敏后内容**，脱敏前的明文密钥（API key / 助记词 / 私钥）永远不落盘，无论是否加密。加密审计归档为可选特性，默认不编入主二进制（`audit-crypto` 特性门控）。

### 14.1 logging level 三档语义

`[audit].level` 配置三档，默认 `metadata`（即第 6 节现状，零行为变化）：

| level | 落盘内容 | 实现 | 默认 |
|-------|---------|------|------|
| `off` | 什么都不留 | 不写 `audit_events` 表，不写归档 | — |
| `metadata` | 审计元数据：时间戳 / 方向 / 命中规则 / 类别 / 动作 / 用户处置 + **脱敏后**最小元信息 | 复用第 6 节 `audit_events` 表（即当前行为，`evidence_meta` 已是脱敏后元信息） | **默认** |
| `full` | 全量内容归档，**只存脱敏后内容**，加密 + 保留期 + 哈希链 | 新增 write-only logging 归档段（见 §14.3~14.5），元数据**仍照常写 `audit_events` 表**，用 `events.id` 关联，不分叉数据模型 | opt-in + 显式警告 |

> 命名裁定（2026-06-19）：中间档定名 `metadata`，**不复用 `decisions`**（已被灰名单目录 + `sieve decisions` CLI 占用）。`full` 档为本节唯一新增能力，默认关闭、完全 opt-in；`level` 默认维持 `metadata` 以免静默丢失现有审计（改默认为 `off` 属行为回退）。

### 14.2 `[audit]` 配置字段

```toml
[audit]
level          = "metadata"        # off | metadata（默认） | full
recipient      = "age1qy...xz"     # full 档必填：age recipient 公钥（X25519），daemon 只持公钥
archive_dir    = "~/.sieve/audit-archive"   # 归档段目录，默认此值
retention_days = 0                 # 保留期：0 = 永久；N = 删除超 N 天的整段密文文件
hash_chain     = true              # full 档必做项
rotation       = "daily"           # 段轮换粒度：daily | weekly | size:<MiB>
```

启动 fail-fast 校验（落在 `check_safety_invariants`，呼应第 5 节配置校验风格）：`level = "full"` 时 `recipient` 必填且须为合法 age 公钥（`age1` 前缀解析通过），`archive_dir` 可写，`retention_days >= 0`，否则 daemon 拒绝启动。

> **write-only logging**：daemon **只持有 recipient 公钥**，结构上**不具备解密能力**——即使运行时被攻陷（威胁模型 1），攻击者也解不开历史归档（机器上根本没有 identity 私钥）。identity 私钥以口令保护（age 原生 scrypt KDF），平时存放于密码管理器 / 离线介质 / 另一台机器，仅在 ①生成密钥对 ②审计解密 两个时刻出现。

### 14.3 归档段（archive segment）文件命名与目录

`full` 档内容落 `~/.sieve/audit-archive/`（`0700` 目录，段文件 `0600`），按 `rotation` 粒度切段：

```
~/.sieve/audit-archive/
├── seg-20260619-0001.age      # 段密文：age recipient 加密，内含若干哈希链记录
├── seg-20260619-0001.head     # 段头明文：key id / genesis_hash / seq 区间 / 段日期（哈希链锚点）
├── seg-20260620-0001.age
└── seg-20260620-0001.head
```

- 段文件名：`seg-<YYYYMMDD>-<NNNN>.age`，日期 + 当日单调段序号（轮换或超 `size:` 阈值时递增 `NNNN`）。
- `.head` 段头记录 `key id`（`signer_pubkey_id` 式标识，轮换后定位对应 identity 解密）+ 段内 `seq` 起止 + `genesis_hash`，便于审计时无需解密即可定位段与校验链完整性。
- 删除整段密文文件是 `full` 档归档上**唯一允许的变更**（区别于 `audit_events` 表的 append-only）；每次 `retention_days` 清理写一条 `metadata` 审计事件记录「删了哪些段」。

### 14.4 加密封装格式（age recipient 混合加密）

每个归档单元用 **hybrid（混合）加密**，不用非对称直接加密大块数据：

```rust
/// 归档单元（已脱敏内容的加密封装），写入段文件
/// 严格红线：plaintext 永远是脱敏后内容（redact_segments / new_body 的产物），绝非原始 body
struct ArchiveUnit {
    seq: u64,                  // 段内单调递增序号（哈希链 + 缺口检测）
    prev_hash: [u8; 32],       // 上一条记录密文的 SHA-256（段首为 genesis_hash）
    event_id: i64,             // 关联 audit_events.id（不分叉数据模型，复用第 6 节表）
    direction: Direction,      // outbound | inbound（仅出站有真实脱敏后内容，见红线说明）
    // —— 以下为 age 混合加密产物 ——
    ciphertext: Vec<u8>,       // AEAD（XChaCha20-Poly1305 / AES-256-GCM）加密的脱敏后内容
    wrapped_data_key: Vec<u8>, // 随机 data key 经 recipient 公钥（X25519）wrap
}
```

封装流程（直接采用 `age` crate，不手搓密码学）：
1. 每个归档单元随机生成对称 **data key**；
2. 用 AEAD 加密**脱敏后内容**得 `ciphertext`；
3. 用 recipient 公钥 wrap data key 得 `wrapped_data_key`；
4. daemon 只持公钥 → 只能加密追加，无解密能力（write-only logging）。

> **红线兑现点**：归档单元的输入**只能**是出站脱敏后产物——`redact_segments()` 返回的 `texts` 或写回后的 `new_body`（`crates/sieve-core/src/pipeline/outbound_redact.rs` + `crates/sieve-cli/src/daemon.rs` AutoRedact 路径，在 `forward` 前、无中间写盘），**绝非** pipeline 入口含原始密钥的 body。当前数据流时序 `detection scan → redact → forward` 已保证脱敏先于落盘。**入站当前架构无脱敏后内容**（`HoldOutcome::RedactAndAllow` 等价 Allow 原样放行，入站 auto-redact 被 sieve-policy lint 明令禁止）——`full` 档入站归档要么是原样放行内容、要么是被 Deny 截流的 `sieve_blocked`；若要求入站也存「脱敏后内容」需新建入站脱敏能力（当前空白）。

### 14.5 哈希链记录结构（防篡改，独立于加密）

加密保证「读不到」，哈希链保证「不被悄悄改写」（两者都要，已裁定必做）：

```rust
/// 段内每条记录的链式锚定（与加密正交）
/// - genesis：段首记录 prev_hash = 段头 .head 中的 genesis_hash（段内起点）
/// - 链接：record[i].prev_hash = SHA-256(record[i-1].ciphertext)
/// - 校验：审计解密时逐条重算并比对；中间删改/重排 → 断链；尾部截断 → seq 缺口
fn link_hash(prev_ciphertext: &[u8]) -> [u8; 32] {
    sha256(prev_ciphertext)   // sieve-cli 需新增 sha2 直接依赖（core/policy/rules/updater 已用 0.10）
}
```

**残余局限（诚实写明）**：哈希链**挡不住「末尾追加伪造」**（持公钥的被攻陷 daemon 可继续合法追加并续链），保证的是**历史不可悄悄改写 / 删除 / 重排**。尾部截断仅能检出 `seq` 缺口，Phase 1 无外部锚点故无法区分「合法新写入未完成」与「恶意截断」。

### 14.6 `audit_events` 表关联（复用第 6 节，不分叉）

`full` 档**不新建并行审计模型**：元数据仍写第 6 节 `audit_events` 表，仅新增一列 `archive_ref` 指回归档段内位置，经 `events.id` 双向关联。schema 从 v3 迁移到 **v4**（迁移模式同第 6.2b 节，单事务 `ALTER TABLE ADD COLUMN` + `PRAGMA user_version`）：

```sql
-- schema v3 → v4 迁移（加密审计归档）
BEGIN;
ALTER TABLE audit_events ADD COLUMN archive_ref TEXT;  -- 形如 "seg-20260619-0001.age#<seq>"，NULL 表示非 full 档/无归档
PRAGMA user_version = 4;
COMMIT;
```

> 命名漂移说明：第 6 节文档历史上写表名 `events` / schema `v2`，**代码实际表名为 `audit_events`、当前 `CURRENT_SCHEMA_VERSION = 3`**（`crates/sieve-cli/src/audit.rs`）。本节以代码为准，`archive_ref` 列加在 `audit_events` 上，bump 至 v4。`evidence_meta` / `session_id` 等亦为第 6 节早期设计意图，代码实际列以 `audit.rs` 的 `CREATE_TABLE_DDL` 为准。

### 14.7 密钥生命周期字段

| 操作 | 命令 | 落点 |
|------|------|------|
| 生成 | `sieve audit keygen` | 生成 X25519 密钥对；recipient 公钥写 `config.toml [audit].recipient`；identity 私钥以口令保护输出，**强制提示移出本机**，文件 `0600`，daemon 不留存 identity |
| 轮换 | `sieve audit rotate-key` | 生成新密钥对，新段用新 recipient；旧段保持旧 recipient 加密；段头 `key id` 定位对应 identity |
| 解密（审计时，离线机器执行） | `sieve audit decrypt --identity <file>` | 口令解锁 identity → 解密段 → 校验哈希链 → 输出脱敏后内容 |

> **口令丢失 = 归档永久不可读（by design）**：identity 不可解锁则归档永久不可读，非 bug。`keygen` 输出与 GUI 开启 `full` 档确认框必须以最显眼方式警示此不可逆性 + 立即备份到密码管理器。

---

## 15. token 用量观测

> 本节描述本地 token 用量观测数据模型。**核心红线**：统计**严格本地、永不上传**（呼应 [SPEC-006 §9.1](../specs/SPEC-006-update-and-telemetry.md) never-upload 承诺）。整个特性默认全关（`[billing_check].enabled = false`），且默认不编入主二进制（`usage` 特性门控）。

### 15.1 信任分级（`official` | `relay`）

在 `[[upstream]]` 上按 url host **自动派生**信任级，可显式 `trust` 覆盖（保守默认：无法判定按 `relay`，fail-closed 倾向）：

| 信任级 | 判定 | `usage` 处理 |
|--------|------|-------------|
| `official` | url host ∈ {`api.anthropic.com`, `api.openai.com`}（可配置扩展） | `usage` **权威**，直接采纳，不核算、零开销 |
| `relay`（默认） | 其余所有上游 / 无法解析 host | `usage` 视为**未经验证的声明**，独立核算 + 交叉比对，偏差超容差报警 |

```toml
[[upstream]]
port    = 8788
url     = "https://relay.example.com"
trust   = "relay"        # 省略时按 host 派生；非官方 host → relay（fail-closed）

[billing_check]
enabled            = false   # 默认关：不开则零行为变化、零出站、零计算开销
tolerance_pct      = 15      # 偏差容差（默认值兼顾低误报与敏感度）
count_tokens_optin = false   # 独立开关：开启才向官方 api.anthropic.com 发 count_tokens 直连（信任边界级决策，默认关）
```

`trust` 在 config 加载期派生（请求期零开销），随 `provider_id` 同链路透传至 `RequestCtx.listener_trust`，作为「只对 relay 生效」的唯一可信来源。`trust` **不进 `audit_events` 表**（`provider_id` 已足够审计归因；usage 落独立 `usage.db`）。

### 15.2 `~/.sieve/usage.db` SQLite Schema

token 用量是**新数据域**，落独立 SQLite（`0600`，append-only），结构上独立于 `audit.db`，经 `request_id` 与第 6 节 `audit_events` 关联，**不污染审计模型**。复用第 6 节的 append-only 触发器 + `spawn_blocking` 写入模式。

```sql
PRAGMA user_version = 1;

CREATE TABLE usage_records (
  id                 INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp          INTEGER NOT NULL,        -- unix ms
  request_id         TEXT    NOT NULL,        -- 关联 audit_events（同次请求归因）
  provider_id        TEXT    NOT NULL,        -- 复用 listener 上游归因（哪个 listener 上游）
  trust_level        TEXT    NOT NULL CHECK (trust_level IN ('official','relay')),
  model              TEXT    NOT NULL,        -- 如 'claude-opus-4' / 'gpt-4o'
  direction          TEXT    NOT NULL CHECK (direction IN ('input','output')),
  independent_input  INTEGER,                 -- 本地独立计数：输入 token（tiktoken / 近似估算）
  independent_output INTEGER,                 -- 本地独立计数：输出 token
  relay_claimed      INTEGER,                 -- relay 声明的 usage（official 档与独立计数一致即权威值）
  expected_cost_usd  REAL,                    -- 独立计数 × 内置价表 = 应收成本
  deviation_pct      REAL,                    -- |relay_claimed - independent| / independent，超 tolerance_pct 报警
  is_estimate        INTEGER NOT NULL DEFAULT 0  -- 1 = 近似估算（Anthropic 无公开 tokenizer）；0 = tiktoken 精确
);

CREATE INDEX idx_usage_timestamp ON usage_records(timestamp);
CREATE INDEX idx_usage_request   ON usage_records(request_id);
CREATE INDEX idx_usage_provider  ON usage_records(provider_id);

CREATE TRIGGER usage_no_update BEFORE UPDATE ON usage_records
BEGIN SELECT RAISE(ABORT, 'usage_records is append-only'); END;

CREATE TRIGGER usage_no_delete BEFORE DELETE ON usage_records
BEGIN SELECT RAISE(ABORT, 'usage_records is append-only'); END;
```

### 15.3 独立计数记录结构（Rust）

核算挂在 pipeline **响应完成后**的观测节点（off the hot path，fire-and-forget，对齐第 9 节审计 fire-and-forget 模式），仅当 `trust == relay && billing_check.enabled` 时触发：

```rust
/// 单次请求的独立 token 核算结果，写入 usage_records
struct UsageRecord {
    request_id: String,        // 关联 audit_events
    provider_id: String,       // listener 上游归因
    trust_level: Trust,        // Official | Relay（只对 Relay 做核算与比对）
    model: String,             // 上游 model id
    direction: Direction,      // Input | Output
    independent_input: Option<u64>,   // OpenAI=tiktoken(o200k_base/cl100k_base 含框架开销)；Anthropic 输入默认近似估算
    independent_output: Option<u64>,  // OpenAI=tiktoken 计补全文本；Anthropic 输出仅近似估算
    relay_claimed: Option<u64>,       // relay 自报 usage（SSE message_delta.usage / 非流式 body usage）
    expected_cost_usd: Option<f64>,   // independent × 内置价表单价
    deviation_pct: Option<f64>,       // 超 tolerance_pct → StatusBar 报警 + 记录，不阻断流量
    is_estimate: bool,                // Anthropic 近似估算标 true，避免误读为精确账单
}
```

数据来源（**只读 / 只比对，绝不改写上游 usage**，不在协议层撒谎）：声明值来自 SSE `message_delta.usage`（`crates/sieve-core/src/sse/parser.rs`，`SseEvent::MessageDelta { usage }` 已结构性捕获）或非流式响应 body 的 `usage` 字段；累计全文 tokenize 在响应流结束（`parser.flush()` 后）或非流式 `collect()` 后进行。OpenAI 流式需请求侧注入 `stream_options.include_usage = true` 方有 usage。

### 15.4 内置价表结构

独立计数 × **官方公开单价**（本地内置价表，按 model + 方向查）= 应收成本，跟 relay 声明比对：

```rust
/// 本地内置价表（编译进二进制 / 随规则包更新），单位 USD per 1M tokens
struct ModelPrice {
    model: &'static str,       // 'claude-opus-4' / 'gpt-4o' / ...
    input_usd_per_mtok: f64,   // 输入单价
    output_usd_per_mtok: f64,  // 输出单价
}
// expected_cost = independent_input/1e6 * input_usd_per_mtok
//               + independent_output/1e6 * output_usd_per_mtok
```

> 价表内置 = 需随单价漂移维护（更新规则包 / 二进制）。`count_tokens` 直连为独立 opt-in 开关（默认关），开启才向官方 `api.anthropic.com` 发起一次 Sieve 主动出站换取权威输入数——属信任边界级决策，启用时 config / UI 必须显著警示。**所有统计严格本地，无任何出站上报路径。**

---

## 16. 相关文档

- [architecture.md](./architecture.md) —— 模块职责、性能预算、演进路径
- [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) —— manifest 协议详细规格（含更新通道与匿名 install 统计的设计取舍）
- [api-reference.md](../api/api-reference.md) —— 反向代理端点、管理 API、配置 schema
