# SPEC-009: 加密审计日志（`full` 档归档 + write-only logging）

> Version: v0.1 — 2026-06-19
> 状态：**Draft**（Phase 2 落地，代码实现后升 Frozen）
> 关联 ADR：[ADR-037](../design/ADR-037-encrypted-audit-log.md)（加密审计日志，唯一权威决策源）/ [ADR-003 amended](../design/ADR-003-local-only-no-cloud-verifier.md)（完全本地，数据层兑现）/ [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)（透明可验证信任叙事）/ [ADR-007](../design/ADR-007-fail-closed-critical-actions.md)（fail-closed 仅针对检测，不针对审计）/ [ADR-016](../design/ADR-016-disposition-matrix-2d.md)（脱敏路径）/ [ADR-023](../design/ADR-023-process-context-audit.md)（CallerContext）/ [ADR-026](../design/ADR-026-port-based-listener-routing.md)（provider_id 归因）
> 关联 PRD：v2.0 §9 #2、§11.2、§11.3
> 关联 SPEC：[SPEC-005](./SPEC-005-ipc-protocol.md)（`sieve.hello` 握手暴露 `audit_db_user_version`）/ [SPEC-010](./SPEC-010-overbilling-detection.md)（ADR-038 共用 `[[upstream]].trust` 与 fire-and-forget 观测模式，独立 spec）

---

## 0. 文档定位

**SPEC-009 是 ADR-037「加密审计日志（`full` 档）」的唯一工程级权威设计规格。**

本文描述 Sieve daemon 在 `[audit].level = "full"` 时，如何把**脱敏后**审计内容加密归档到本地，涵盖：

- 三档 logging level（`off` / `metadata` / `full`）的配置 schema 与默认语义
- `full` 档归档单元的写入状态机（脱敏后内容 → 随机 data key → AEAD → age recipient wrap → 追加）
- `age` 集成（recipient-only 加密、scrypt 口令保护 identity、密钥生成/轮换/解密 CLI）
- 防篡改哈希链（`prev_hash` + `seq`）的写入与验证算法及残余局限
- 保留期与按段轮换实现（归档上唯一允许的删除变更）
- 脱敏先于落盘红线的回归测试矩阵
- 加密失败 / 磁盘满 / 口令丢失的错误处理策略

**本 SPEC 不描述**：

- `metadata` 档现状（即当前已发布的 `audit_events` 表行为，零变化，见 [data-model.md §6](../design/data-model.md)）—— `full` 档只是**额外**存一份加密副本，不分叉数据模型
- 入站脱敏归档的内容来源问题（当前架构入站无脱敏能力，见 §3.5 残缺说明）
- ADR-038 的 token 核算 / 信任分级（独立 [SPEC-010](./SPEC-010-overbilling-detection.md)，仅 `[[upstream]].trust` 字段与 fire-and-forget 观测模式与本 SPEC 共享代码风格）

---

## 1. 背景与目标

### 1.1 三档 logging level

当前 `audit.db`（[data-model.md §6](../design/data-model.md)，代码实际表名 `audit_events`）只存 fingerprint + 最小元信息，**绝不存原始内容**——即使审计文件被读到，也只能拿到「此用户在此时刻触发过 OUT-09」。这是 [ADR-003](../design/ADR-003-local-only-no-cloud-verifier.md) 数据层兑现：本地足迹最小、从不留底流量。

但闭测期用户（尤其 doskey 自己）需要看到**脱敏后的实际内容片段**判断规则是否误伤、攻击是否真实。直接放开「存内容」会亲手建一个本地明文流量库——正是 Sieve 卖防护的那个漏洞。故扩展现有审计模型为三档：

| level | 落盘内容 | 实现 | 默认 |
|-------|---------|------|------|
| `off` | 什么都不留 | 既不写 `audit_events` 表，也不写归档 | — |
| `metadata` | 审计元数据（时间戳 / 方向 / 命中规则 / 类别 / 动作 / 用户处置 / 脱敏后片段或哈希） | **复用现有 `audit_events` 表**（[audit.rs](../../crates/sieve-cli/src/audit.rs) `CREATE_TABLE_DDL`），即当前行为 | ✅ **默认** |
| `full` | 全量内容归档，**只存脱敏后内容**，加密 + 保留期 | 新增 write-only logging 归档单元（本 SPEC §3~§6） | 关（opt-in + 显式警告） |

> **默认 `metadata` 而非 `off`**：`metadata` 是当前已发布行为（`audit_events` 现已在写），不是新能力；若改默认为 `off` 会静默丢失现有审计，属行为回退。`full` 是本 SPEC 真正的新增，完全 opt-in。

> **命名**：中间档定名 `metadata`，**不叫 `decisions`**——`decisions` 已是既有术语（`~/.sieve/decisions/` 灰名单目录 + `sieve decisions` CLI + [ADR-021](../design/ADR-021-tri-state-decision-and-graylist.md)），复用会造冲突词。

### 1.2 威胁模型（必须逐条防住或诚实标注残余）

| # | 威胁 | `metadata` 档现状 | `full` 档目标 |
|---|------|------------------|--------------|
| 1 | 机器运行时被攻陷（live malware） | 安全（无内容） | **历史归档解不开**（机器无 identity 私钥）；残余：live malware 可截获正在流过的新明文（任何日志都防不住，不夸大） |
| 2 | 硬盘被偷 / 整盘镜像被拷走 | 安全（无内容） | **安全**（归档密文，无 identity 解不开） |
| 3 | `~/.sieve/` 被误同步云盘（iCloud / Dropbox）或误 `git add` | 安全（无内容） | **安全**（归档密文 + 段文件 0600） |

### 1.3 目标

1. `full` 档把脱敏后审计内容加密归档，威胁 1（运行时攻陷）也解不开历史归档
2. daemon **只持公钥（recipient）**，结构上不具备解密能力（write-only logging）
3. 私钥（identity）以口令保护，平时不在跑 daemon 的机器上
4. 哈希链保证历史归档「不可悄悄改写 / 删除 / 重排」
5. 保留期按段清理过期密文；除此之外归档 append-only
6. 整个能力可配置、默认全关；不开启则零行为变化、零新增出站

---

## 2. 配置 schema（`[audit]` 段）

### 2.1 TOML 形态

```toml
[audit]
# 三档 logging level；默认 metadata（= 当前已发布行为，零变化）
level = "metadata"           # off | metadata | full

# 以下字段仅 level = "full" 时生效；level != full 时被忽略（但仍校验合法性）
recipient = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"  # age X25519 公钥
archive_dir = "~/.sieve/audit-archive"   # 归档段目录，默认 <sieve_home>/audit-archive
retention_days = 30          # 0 = 永久保留；> 0 = 删除超期段
hash_chain = true            # full 档必做项，默认 true（见 §6）
rotation = "daily"           # 段轮换粒度：daily | size:<N>MiB（见 §5.1）
```

### 2.2 字段语义

| 字段 | 类型 | 默认 | 语义 |
|------|------|------|------|
| `level` | `AuditLevel` enum | `metadata` | `off` / `metadata` / `full`，`#[serde(rename_all = "snake_case")]` |
| `recipient` | `Option<String>` | `None` | age recipient 公钥（`age1...`）。`level = full` 时**必填**且必须是合法 age 公钥（否则 fail-fast，见 §2.3） |
| `archive_dir` | `Option<PathBuf>` | `<sieve_home>/audit-archive` | 归档段目录，daemon 启动时确保存在且 0700 |
| `retention_days` | `u32` | `30` | 段保留天数；`0` = 永久保留 |
| `hash_chain` | `bool` | `true` | 是否启用防篡改哈希链；`full` 档建议恒为 `true`（关闭会丢失历史防改写保证） |
| `rotation` | `String` | `"daily"` | 段轮换触发条件，见 §5.1 |

### 2.3 落点与 fail-fast 校验（实现约定）

照 [config.rs](../../crates/sieve-cli/src/config.rs) 既有 `UpdateConfig` 分段先例落地（这是仓内唯一已落地的「整段可省略」配置先例）：

```rust
// crates/sieve-cli/src/config.rs —— 放在 UpdateConfig 之后、Config struct 之前

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuditLevel {
    Off,
    #[default]
    Metadata,
    Full,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]   // 段级 default 让整个 [audit] 段可省略
pub struct AuditConfig {
    pub level: AuditLevel,
    pub recipient: Option<String>,
    pub archive_dir: Option<PathBuf>,
    pub retention_days: u32,
    pub hash_chain: bool,
    pub rotation: String,
}

impl Default for AuditConfig {
    // 手写而非 derive：retention_days=30 / hash_chain=true 是非零默认
    fn default() -> Self {
        Self {
            level: AuditLevel::Metadata,
            recipient: None,
            archive_dir: None,
            retention_days: 30,
            hash_chain: true,
            rotation: "daily".to_string(),
        }
    }
}
```

挂载到 `Config` struct（紧邻 `pub update: UpdateConfig` 之后）：

```rust
#[serde(default)]
pub audit: AuditConfig,
```

> ⚠ `Config` 手写了 `impl Default`（非 derive），新增字段必须同步在 `Config::default()` 里补 `audit: AuditConfig::default()`，否则编译失败。`Config` 整体只 derive `Deserialize` 不 derive `Serialize`，但子 struct 保留 `Serialize` 不受影响（与 `UpdateConfig` 一致）。

fail-fast 校验加进 `check_safety_invariants(&self) -> Result<(), String>`（纯函数，可单测；自动经 `enforce_safety_invariants()` 获得 `exit(1)` 语义；唯一调用点 `main.rs` config 加载之后、`AuditStore::init` 之前）：

```rust
// 只有 full 档才强校验 recipient
if self.audit.level == AuditLevel::Full {
    match self.audit.recipient.as_deref() {
        None => return Err(
            "[audit] level=full 必须配置 recipient（age 公钥）。\
             先跑 `sieve audit keygen` 生成密钥对。".to_string()),
        Some(s) if s.parse::<age::x25519::Recipient>().is_err() => return Err(format!(
            "[audit] recipient 不是合法 age 公钥（期望 age1... 形式）：{s}")),
        Some(_) => {}
    }
    // archive_dir 可写性在 AuditStore/ArchiveWriter init 时校验（涉及 IO，不放纯函数）
}
```

---

## 3. 归档写入状态机

### 3.1 ArchiveWriter 是独立单元，不复用 AuditStore

`full` 档归档单元（命名 `ArchiveWriter`）**与 `AuditStore` 并列**，不塞进 `audit_events` 表。理由：

- `AuditStore` 是 append-only SQLite + `BEFORE UPDATE/DELETE` 触发器模型；归档是 age 加密段文件 + 哈希链 + 保留期删除（§5 唯一允许删除），两种存储语义冲突——硬塞会破坏 append-only 不变量
- 加密归档需要 recipient 公钥 + AEAD + `seq`/`prev_hash` 链，与 SQLite 行模型正交

**所有权与透传**：`ArchiveWriter` 在 [main.rs](../../crates/sieve-cli/src/main.rs) 构造 `AuditStore` 旁边构造（仅注入 recipient 公钥，无 identity，结构上不可解密），以 `Arc<ArchiveWriter>` 透传进 `RequestCtx`（[daemon.rs](../../crates/sieve-cli/src/daemon.rs) `RequestCtx` struct），与现有 `Arc<AuditStore>` 并列。

### 3.2 红线：脱敏后内容来源（决策 2 兑现）

**归档单元的输入只能是脱敏后字节，绝不是 pipeline 入口的原始 body。**

出站数据流时序（Anthropic，OpenAI 对称）：

```
extract_text_content()           → 原始 texts: Vec<(usize, String)>      ❌ 含 secret，绝不归档
   ↓ detection scan → redact_hits: Vec<RedactHit>
redact_segments(&texts, &hits)   → seg_result.texts: Vec<String>         ✅ 已脱敏，含 [REDACTED:rule_id] 占位符
   ↓
OutboundRedacted audit append    → raw_json=None（脱敏事件不持久化原文，现状基线）
   ↓
apply_redacted_texts_to_request  → new_body_bytes（脱敏后，serde_json 重序列化）
   ↓  ★ 归档 hook 挂这里：new_body_bytes 已得、JSON 已验证、forward 之前
forward_with_inbound_inspection  → 转发上游
```

归档 hook 消费 `seg_result.texts`（或 `new_body_bytes`），**严禁**消费 `extract_text_content()` 的原始 `texts` 或原始 `body_bytes`。`redact_segments` / `redact_body_bytes`（[outbound_redact.rs](../../crates/sieve-core/src/pipeline/outbound_redact.rs)）均为纯函数、无 IO、无写盘，其返回值是归档的唯一合法输入。

> 现状基线：`OutboundRedacted` audit 写入时 `raw_json=None`（daemon「脱敏事件不持久化原文，含 secret」），即 `metadata` 档本就「纯元数据零内容」。`full` 档「存脱敏后内容」是新增能力，hook 取脱敏后产物。

### 3.3 hook 复用 fire-and-forget 模式（off hot path）

归档写入沿用现有审计 fire-and-forget 模式（绝不阻塞请求热路径，PRD §9 性能预算 P99 < 20ms）：

```rust
// daemon.rs 出站 AutoRedact 分支内，new_body_bytes 构造后、forward 前
if let Some(archive) = ctx.archive.as_ref() {       // 仅 full 档 Some
    let archive = Arc::clone(archive);
    let event_id = /* audit_events.id，由 AuditStore::append 回传或预分配 */;
    let payload = seg_result.texts.clone();          // 脱敏后文本段
    let provider_id = ctx.listener_provider_id.clone();
    tokio::spawn(async move {                         // fire-and-forget，不阻塞 forward
        if let Err(e) = archive.append(event_id, &payload, &provider_id).await {
            tracing::warn!(error = %e, "audit archive append failed");  // 只 warn，绝不传播
        }
    });
}
```

> **fail 策略红线**：归档写失败（age 加密失败 / 磁盘满 / 权限错）**只 warn 不阻断 forward**。审计可靠性问题绝不能变成可用性事故——[ADR-007](../design/ADR-007-fail-closed-critical-actions.md) fail-closed 针对检测，不针对审计。详见 §8。

### 3.4 单个归档单元加密封装（hybrid 加密 + AEAD）

每个归档单元（一次脱敏事件的脱敏后内容）独立加密。封装采用 **age 原生 hybrid 加密**（X25519 + ChaCha20-Poly1305），不手搓密码学：

```
归档单元写入流程（ArchiveWriter::append）：

  payload (脱敏后内容，UTF-8 JSON / 文本段)
    │
    │ ① 构造归档记录头（明文元数据，参与哈希链）
    ▼
  ArchiveRecordHeader {
      seq: u64,              // 段内单调递增
      event_id: i64,        // 关联 audit_events.id
      provider_id: String,  // ADR-026 归因
      timestamp_rfc3339: String,
      key_id: String,       // recipient 公钥指纹（轮换定位，§5.2）
      prev_hash: [u8; 32],  // 上一条记录密文的 SHA-256（§6）
  }
    │
    │ ② age 加密 payload（recipient-only）
    │    内部：随机 data key → ChaCha20-Poly1305 AEAD 加密 payload
    │           → X25519 用 recipient 公钥 wrap data key
    ▼
  ciphertext: Vec<u8> = age::encrypt(recipient, payload)   // daemon 无 identity，结构上不可逆
    │
    │ ③ 算本记录哈希（含密文，链接下一条）
    ▼
  row_hash = SHA-256(seq || event_id || prev_hash || ciphertext)
    │
    │ ④ 追加到当前段文件（length-prefixed framing）
    ▼
  segment file += encode(header, ciphertext, row_hash)
```

> **为什么是 hybrid 不是非对称直接加密**：非对称只用来 wrap 一个随机对称 data key，实际内容用 AEAD（XChaCha20-Poly1305 / AES-256-GCM 同级）加密大块数据——这正是 `age` 的内部模型（X25519 + ChaCha20-Poly1305）。daemon 端只配 recipient 公钥（`age1...`），**无 identity，结构上不具备解密能力**。

### 3.5 入站归档的架构残缺（诚实标注）

ADR-037 决策 2 提到「入站经 redaction map 替换后的内容」，**但当前架构入站无脱敏能力**：

- 入站 `HoldOutcome::RedactAndAllow` 在 daemon 中与 `Allow` 合并、**原样放行原始 frame**，无任何替换
- `classify_inbound_detections` 中 `Action::Redact` 被明确跳过（注释「入站脱敏暂不实现」）
- `sieve-policy` lint `InboundAutoRedactForbidden` 明令禁止 `direction=inbound + auto_redact`

**结论**：`full` 档入站归档**没有「替换后内容」可存**——入站要么原样放行（含原始内容，不可归档，否则违反脱敏红线）要么被 Deny 截流注入 `sieve_blocked`。**本 SPEC v0.1 范围内，`full` 档归档仅覆盖出站脱敏后内容**；入站 full 档归档需先建入站脱敏能力（当前架构空白），登记为未来方向，不在本次落地。

---

## 4. age 集成与密钥生命周期

### 4.1 依赖引入

`age` 加在 `crates/sieve-cli/Cargo.toml [dependencies]`（审计独有，照 `rusqlite` 本地声明风格，不进 `[workspace.dependencies]`）：

```toml
age = "0.11"        # MIT OR Apache-2.0，在 deny.toml allow 白名单内
sha2 = "0.10"       # 哈希链需；sieve-cli 当前未直接依赖，core/policy/rules/updater 已用 0.10
```

> **cargo-deny 风险点**（引入后必跑 `cargo deny check`）：
> - `age` 拉入 `x25519-dalek` → `curve25519-dalek`；仓内 `ed25519-dalek=2` 已依赖一个 `curve25519-dalek`（Cargo.lock 当前仅 1 个版本）。**版本不对齐会触发 `multiple-versions = warn`**（非致命但污染 CI），应对齐。
> - `age` 传递依赖 `secrecy` / `chacha20poly1305` / `scrypt` 系（RustCrypto，MIT/Apache）大概率全过白名单，实测确认无 BSL/GPL 类落网。

### 4.2 keygen / rotate-key / decrypt CLI 流程

三个动作作为新变体加进 [cli.rs](../../crates/sieve-cli/src/cli.rs) `AuditCommand` enum，或拆为独立 `commands/keys.rs`（推荐，避免污染 `commands/audit.rs` 只读语义；clap 自动 kebab：`RotateKey` → `rotate-key`）。所有 async 子命令遵循 `run → run_async` 委托模式，**禁止内部 `block_on` 自建 runtime**（嵌套 runtime panic / exit 134，CHANGELOG 已记）。

#### `sieve audit keygen`

```
sieve audit keygen [--out <identity-file>] [--yes]

1. 生成 X25519 密钥对（age::x25519::Identity::generate）
2. recipient 公钥（age1...）→ 提示写入 config.toml [audit].recipient（不自动改 config，回显让用户粘贴）
3. identity 私钥用 scrypt 口令保护（age 原生口令加密，内存硬 KDF）：
     - 交互读口令（二次确认）
     - 写 <identity-file>（默认 ~/.sieve/audit-identity.age），Unix 0600，目录 0700
     - 私钥【绝不打印 stdout】（避免落 shell history / 终端 scrollback）
4. 覆盖已有 identity 文件前走 confirm_or_abort [y/N] 二次确认
5. 输出后用最显眼方式打印不可逆警告：
     ⚠ 警告：口令一旦丢失，identity 永久无法解锁，全部 full 档归档将【永久不可读】。
       请立即把 identity 文件 + 口令移出本机（密码管理器 / 离线介质）。daemon 不留存 identity。
```

> **scrypt 而非自研 Argon2id**：用户口令用来**保护私钥**，不是当加密密钥用——收益是 daemon 正常运行不需要口令（只用公钥），口令只在「生成密钥」「审计解密」两个时刻出现。`age` 原生口令保护 identity 用 scrypt（内存硬 KDF），**直接采纳，零自研密码学**。

#### `sieve audit rotate-key`

```
sieve audit rotate-key [--out <new-identity-file>] [--yes]

1. 生成新 X25519 密钥对
2. 新归档段用新 recipient（key_id = 新公钥指纹，记入段头 §5.2）
3. 旧段保持旧 recipient 加密不变（审计旧段需对应旧 identity）
4. 提示更新 config.toml [audit].recipient 为新公钥
5. 同 keygen 的口令保护 + 不可逆警告
```

#### `sieve audit decrypt`（应在另一台 / 离线机器执行）

```
sieve audit decrypt --identity <file> [--segment <seg-file>] [--out <plaintext-dir>]

1. 交互读口令 → scrypt 解锁 identity
2. 按 key_id 匹配段（多 identity 时定位对应旧 recipient 的段）
3. 逐记录 age 解密 ciphertext → 脱敏后内容
4. 校验哈希链（§6）：seq 连续 + prev_hash 链合法
5. 输出脱敏后内容（jsonl / pretty，复用 OutputFormat），链断裂时显著报警
```

> **decrypt 不走 AuditStore / audit_db_path**——它读 age 加密段 + 离线 identity，与现有 `sieve audit query/tail/show` 只读 SQLite 查询语义完全不同、并列存在。daemon 机器无 identity，无法 decrypt（write-only logging 兑现）。

### 4.3 密钥管理摘要表

| 阶段 | daemon 持有 | identity 位置 | 口令出现时刻 |
|------|------------|--------------|-------------|
| 正常运行 | recipient 公钥（config） | **不在本机** | 从不 |
| keygen | — | 用户保管（密码管理器 / 离线） | 生成时（保护私钥）|
| 审计 decrypt | — | 另一台 / 离线机器 | 解密时（解锁私钥）|

---

## 5. 保留期与轮换实现

### 5.1 段轮换（rotation）

归档按**段文件（archive segment）**组织，轮换触发：

| `rotation` 值 | 触发 | 段文件命名 |
|--------------|------|-----------|
| `daily`（默认） | 跨自然日（本地时区）开新段 | `audit-archive/seg-YYYY-MM-DD.age` |
| `size:<N>MiB` | 当前段 ≥ N MiB 开新段 | `audit-archive/seg-<ISO8601>-<nnn>.age` |

每段是 length-prefixed 记录的追加文件（§3.4 framing）。段内 `seq` 从 0 单调递增；跨段时哈希链可选续接（v0.1 哈希链以段为单位独立，跨段不强制续链，简化轮换——残余局限见 §6.3）。

### 5.2 段头（segment header）

每个段文件首部写一次明文段头（不加密，仅元数据）：

```rust
struct SegmentHeader {
    format_version: u16,     // 归档格式版本，独立于 SQLite schema version
    key_id: String,          // 本段使用的 recipient 公钥指纹（rotate-key 后定位用）
    created_rfc3339: String,
    rotation: String,        // 复刻 config，便于离线 decrypt 还原上下文
}
```

### 5.3 保留期清理（唯一允许的归档变更）

`retention_days = N`（`N > 0`）时，daemon 周期扫描 `archive_dir`，删除**段日期 / mtime 超过 N 天的整段密文文件**：

```
ArchiveWriter::purge_expired(retention_days):
  for seg in archive_dir.segments():
      if now - seg.date > retention_days:
          fs::remove_file(seg.path)           // 整段删除，不改段内记录
          # 写一条 metadata 审计事件记录「删了哪个段」（落 audit_events 表）
  return purged_segments
```

约束：

- **删除是 `full` 档归档上唯一允许的变更**（区别于 `audit_events` 表 append-only）。归档段文件本身无 SQLite 触发器保护——按段整体删除，绝不删段内单条记录（否则断哈希链且无意义）。
- `retention_days = 0` = 永久保留，不删。
- 每次清理写一条 `metadata` 档审计事件（记「删了哪些段 + 段日期范围」），保留删除可审计性。
- 清理触发点：daemon 周期任务（复用现有定时器或独立 interval），off hot path。

> 对比 `audit_events` 表的 `purge_history`（`delete_all_events` 临时 DROP 触发器再删）——归档段是文件不是表，无触发器，直接 `fs::remove_file`，但同样要写 metadata 审计留痕。

---

## 6. 防篡改哈希链

### 6.1 为什么加密之外还要哈希链

加密保证「读不到」，**不保证「不被改」**。被攻陷的 daemon 持有公钥（加密能力），**能往归档末尾塞伪造条目**；磁盘段文件也能被截断 / 删改。哈希链让中间任何删改 / 重排断链、尾部截断留缺口。呼应 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)「透明、可独立验证」（Rekor 透明日志本地同构）。

### 6.2 链结构与写入

每条归档记录含 `prev_hash`（上一条记录密文的 SHA-256）+ 段内单调递增 `seq`：

```rust
// 写入侧（ArchiveWriter::append，已持 Mutex 串行化，是算链的天然位置）
fn compute_row_hash(seq: u64, event_id: i64, prev_hash: &[u8; 32], ciphertext: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(seq.to_le_bytes());
    h.update(event_id.to_le_bytes());
    h.update(prev_hash);
    h.update(ciphertext);            // 链接密文，非明文（明文从不在链计算中出现）
    h.finalize().into()
}

// 段内第一条 prev_hash = [0u8; 32]（genesis）
// 第 n 条 prev_hash = 第 n-1 条的 row_hash
```

> ⚠ 哈希链计算的输入是**密文**，绝不是脱敏后明文 payload——保证哈希链验证过程本身也不需要明文，离线 decrypt 校验链时无需先解密即可验链完整性（解密与验链可解耦）。

### 6.3 验证算法（`sieve audit decrypt` 内 + 独立 `verify`）

```
verify_hash_chain(segment) -> Result<ChainStatus>:
    prev = [0u8; 32]
    expected_seq = 0
    for record in segment.records():          // 不需解密，只读密文 + 头
        # ① seq 连续性
        if record.seq != expected_seq:
            return Err(GapDetected { at: expected_seq, found: record.seq })
        # ② prev_hash 链接性
        if record.prev_hash != prev:
            return Err(ChainBroken { at: record.seq })
        # ③ 重算 row_hash 比对（防密文被改）
        recomputed = compute_row_hash(record.seq, record.event_id, &record.prev_hash, &record.ciphertext)
        if recomputed != record.row_hash:
            return Err(CiphertextTampered { at: record.seq })
        prev = record.row_hash
        expected_seq += 1
    return Ok(Intact { count: expected_seq })
```

### 6.4 残余局限（诚实写明，不夸大）

| 攻击 | 哈希链能否检出 | 说明 |
|------|--------------|------|
| 中间记录删除 / 改写 / 重排 | ✅ 检出 | 断链（`prev_hash` 不匹配）或 `seq` 缺口 |
| 密文被篡改 | ✅ 检出 | `row_hash` 重算不匹配 |
| **末尾追加伪造** | ❌ **挡不住** | 攻陷的 daemon 持公钥，可继续合法追加并续上链——任何 write-only logging 都防不住，本 SPEC 不假装能防 |
| 尾部截断（删末尾若干条） | ⚠ 仅检出缺口 | `seq` 留缺口可见，但 v0.1 无外部锚点（另存 head 指针 / 周期 checkpoint），**无法区分「合法新写入未完成」与「恶意截断」** |

> Phase 1 不引入外部锚点。哈希链保证的是**历史不可悄悄改写 / 删除 / 重排**，不是「绝对完整性」。尾部截断检出需未来引入周期 checkpoint（另存 head `seq` + `row_hash` 到独立位置）才能升级，登记为未来方向。

---

## 7. 红线回归测试矩阵（脱敏先于落盘）

**ADR-037 接受前提：脱敏前明文绝不落盘，回归测试守护，不带测试不合并。**

| 用例 | 构造 | 断言 |
|------|------|------|
| 出站 sk-ant 不落盘 | 含明文 `sk-ant-api03-xxxx` 的请求 → 开 `full` 档 | ① 归档密文 age decrypt 后**不含**原始 `sk-ant-api03-xxxx`（只含 `[REDACTED:OUT-01]`）；② pipeline 全程无「原始 body 写盘」调用 |
| 出站 BIP39 不落盘 | 含 BIP39 助记词（24 词 + SHA-256 checksum 合法）的请求 → 开 `full` 档 | 解密后无任何助记词原文，只有脱敏占位符 |
| 归档输入来源 | mock `ArchiveWriter::append` | 输入只来自 `seg_result.texts` / `new_body_bytes`，**绝不**来自 `extract_text_content()` 原始 texts |
| `metadata` 档零内容 | 开 `metadata` 档触发 OutboundRedacted | `audit_events.raw_json IS NULL`，无归档段产生 |
| `off` 档零落盘 | 开 `off` 档触发命中 | 既无 `audit_events` 行也无归档段 |
| daemon 无解密能力 | 仅配 recipient（无 identity）的 daemon | 进程内无 identity；归档段无 identity 无法解密（用错误口令 / 无 identity 解密必失败）|
| 哈希链断裂检出 | 手动改一条密文 / 删中间一条 | `verify_hash_chain` 返回 `CiphertextTampered` / `ChainBroken` / `GapDetected` |
| 加密失败不阻断 forward | mock age 加密返回 Err（如磁盘满） | forward 正常完成（只 warn），请求不失败 |
| 内容类型路由全覆盖 | Anthropic SSE / Anthropic JSON / OpenAI SSE / OpenAI JSON 四类（PRD §9 #16） | 四类出站脱敏归档 hook 全挂（出站脱敏侧；入站见 §3.5 残缺）|

> 红线测试是**合并前置硬约束**：构造含明文 `sk-ant` / BIP39 的请求，开 `full` 档，断言密文解密后无原始密钥明文 + 全程无原始 body 写盘调用——任一断言失败则特性不合并。

---

## 8. 错误处理 / fail 策略

| 故障 | 处置 | 理由 |
|------|------|------|
| age 加密失败 | `tracing::warn!` 记录，**不阻断 forward**，丢弃本次归档 | 审计可靠性问题绝不变可用性事故；[ADR-007](../design/ADR-007-fail-closed-critical-actions.md) fail-closed 针对检测不针对审计 |
| 磁盘满 / 段文件写失败 | 同上，warn + 丢弃本次归档；后续重试自然恢复 | fire-and-forget 不传播到请求路径 |
| `archive_dir` 不可写 | **启动期** fail-fast（`AuditStore`/`ArchiveWriter::init` 校验，IO 类不放纯函数）；运行期写失败降级为 warn | 启动期硬错暴露配置问题；运行期不拖垮请求 |
| `recipient` 非法 age 公钥 | **启动期** fail-fast（`check_safety_invariants` Err → exit(1)）| 配置错误必须早暴露，见 §2.3 |
| 口令丢失 | **identity 永久无法解锁，归档永久不可读，by design** | 见下 |
| 哈希链验证失败（decrypt 时） | 显著报警（不静默），输出已验证部分 + 断点位置 | 审计需知道历史是否被动过 |

> **错误处理硬约束**：age 加密失败 / 计数失败的错误处理**必须走 `Result` 不能 panic**（release profile `panic=abort` + 请求处理路径禁 `unwrap`/`expect`）。审计写入在请求处理路径上，归档失败只 warn。

### 8.1 口令丢失 = 永久不可读（by design）

口令丢失则 identity 不可解锁，**归档永久不可读，这是设计使然，不是 bug**。本 SPEC 写明此性质；UI 必须用最显眼方式怼给用户：

- `sieve audit keygen` 输出末尾的不可逆警告（§4.2）
- GUI 开启 `full` 档的确认框
- `deployment.md` 密钥离线保管最佳实践章

否则会收到「忘了密码求恢复」而无法恢复。这是「不夸大、不假装能防」原则的对偶——既不假装能防 live malware 截获新明文，也不假装能恢复丢失口令。

---

## 9. 变更 / 决策记录

本 SPEC 落地 [ADR-037](../design/ADR-037-encrypted-audit-log.md)，关键裁定（2026-06-19）逐条对应：

| ADR-037 决策 | 本 SPEC 章节 | 实现要点 |
|-------------|-------------|---------|
| 1. 三档 logging level（默认 `metadata`） | §1.1 / §2 | `AuditLevel` enum，`metadata` 默认 = 现状零变化 |
| 2. 脱敏先于落盘红线 | §3.2 / §7 | hook 消费 `seg_result.texts`，回归测试守护 |
| 3. write-only logging（age 混合加密） | §3.4 / §4 | daemon 只持 recipient，无 identity 结构上不可解密 |
| 私钥口令保护采纳 age 原生 scrypt | §4.2 | 零自研密码学，口令保护私钥非当加密密钥 |
| 4. 哈希链（已裁定保留） | §6 | `prev_hash` + `seq`，链密文非明文 |
| 5. 密钥生成 / 轮换 / 解密 / 保留期 | §4.2 / §5 | keygen/rotate-key/decrypt CLI + 按段保留期 |
| 6. 口令丢失永久不可读 | §8.1 | by design，UI 最显眼警示 |

### 9.1 与代码 ground-truth 的偏差修正

- **表名**：代码实际表名 `audit_events`（data-model.md §6.2 文档写 `events`，已过时）。`full` 档不改此表，仅 `archive_ref` 关联（见下）。
- **schema 版本**：代码 `CURRENT_SCHEMA_VERSION=3`（data-model.md 写 v2，落后一版）。若 `audit_events` 需加 `archive_ref TEXT` 列关联归档段，仿 v2→v3 迁移加 v3→v4 分支并 bump 到 4；SPEC-005 `sieve.hello` 的 `audit_db_user_version` 字段会看到新版本号，需确认向后兼容。
- **入站脱敏**：ADR-037 决策 2 提及「入站经 redaction map 替换后内容」，但代码入站无脱敏能力（§3.5），v0.1 归档仅覆盖出站。
- **`redact_body_bytes` vs `redact_segments`**：ADR-037 §45 点名 `redact_body_bytes()`，但生产出站路径实际调 `redact_segments()`（`redact_body_bytes` 仅被其内部按段调用 / fuzz 保留）。归档真正消费的是 `seg_result.texts` / `new_body_bytes`。

### 9.2 文档同步清单（实现时）

- [data-model.md](../design/data-model.md)：新增「§N 加密审计归档（`full` 档）」——归档单元 schema、段文件命名、哈希链记录结构、`[audit]` 配置字段表；`audit_events` 关联 `archive_ref`；修正表名 / schema 版本漂移（P0）
- [api-reference.md](../api/api-reference.md) §3 config：新增 `[audit]` 段（`level` / `recipient` / `archive_dir` / `retention_days` / `hash_chain` / `rotation`）；§2 新增 `sieve audit keygen` / `rotate-key` / `decrypt` 子命令（P1）
- [glossary.md](../design/glossary.md)：新增 `write-only logging` / `归档单元 / archive segment` / `recipient（age 公钥）` / `identity（age 私钥）` / logging level 三档（P0）
- [deployment.md](../guides/deployment.md)：`full` 档启用指南 + 密钥离线保管最佳实践 + 口令丢失不可恢复警示（P1）
- [CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Added + Security 条目（P1）
- [specs/INDEX.md](./INDEX.md)：加 SPEC-009 行（P0）
- **回归测试硬约束**：§7 脱敏先于落盘回归测试（Phase 2 必带，否则特性不合并）

---

## 相关文档

- [ADR-037: 加密审计日志（`full` 档 + write-only logging）](../design/ADR-037-encrypted-audit-log.md)（唯一权威决策源）
- [ADR-003: 完全本地运行，绝不联网做 verifier](../design/ADR-003-local-only-no-cloud-verifier.md)
- [ADR-006: Sigstore + 可复现构建 + 透明日志](../design/ADR-006-sigstore-reproducible-build.md)
- [ADR-007: fail-closed Critical actions](../design/ADR-007-fail-closed-critical-actions.md)
- [ADR-021: 三态决策 + 灰名单](../design/ADR-021-tri-state-decision-and-graylist.md)
- [ADR-023: 进程上下文审计](../design/ADR-023-process-context-audit.md)
- [ADR-026: Port-based listener routing（provider_id）](../design/ADR-026-port-based-listener-routing.md)
- [data-model.md §6: 本地审计日志 SQLite Schema](../design/data-model.md)
- [SPEC-005: IPC 协议（`audit_db_user_version` 握手）](./SPEC-005-ipc-protocol.md)
- [SPEC-010: 超额计费检测（共用 `[[upstream]].trust` / fire-and-forget 观测）](./SPEC-010-overbilling-detection.md)
