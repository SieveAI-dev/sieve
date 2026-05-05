# SPEC-006: Sieve 更新通道与遥测协议

> Version: v0.1 — 2026-05-05
> Status: **Draft**（待 TODO-7~12 代码落地后升 Frozen）
> 关联 ADR：[ADR-030](../design/ADR-030-update-telemetry-channel.md)（更新通道遥测 · 核心决策）/ [ADR-029](../design/ADR-029-free-first-defer-monetization.md)（装机量优先）/ [ADR-003 amended](../design/ADR-003-local-only-no-cloud-verifier.md)（网络边界修订）/ [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)（签名分发）
> 关联 PRD：[sieve-prd-v2.0.md §9 硬约束 #2](../prd/sieve-prd-v2.0.md)

---

## 0. 文档定位

**SPEC-006 是 `sieve-updater` crate 实现的唯一权威设计规格。**

本文描述 Sieve 客户端如何拉取规则更新 + 兼作遥测信标，涵盖：

- Manifest 协议 wire format（v0.1）
- install-id 生成与持久化
- 三个环境变量优先级与启动 banner 行为
- 客户端实现流程（定时器 / 重试 / 降级）
- ed25519 + sha256 双重签名校验
- 失败处理策略
- 配置 `[update]` 段语义
- crate 边界约束

**本 SPEC 不描述**：
- 服务端实现（待 TODO-15 Cloudflare Workers 落地后独立规格）
- 规则文件格式（`sieve-rules` crate 负责）
- 规则热加载（属于 `sieve-policy` crate 职责）

---

## 1. 背景与设计原则

ADR-029 把装机量定为 GA 前唯一指标。ADR-030 决定复用规则更新请求作为遥测信标，而非开辟独立心跳通道：

- **用户没有动机禁用更新**——禁了规则就过期失效
- **零重复流量**——本来就要拉规则，附带 5 个匿名字段不增加任何额外网络访问
- **ADR-003 唯一允许例外**——更新通道是「绝对禁止独立 telemetry」反模式的唯一豁免，边界清晰可言简意赅地向用户解释

---

## 2. install-id 生成与持久化

### 2.1 生成策略

- **算法**：UUIDv4，纯随机（`uuid::Uuid::new_v4()`）
- **不掺入**：设备序列号 / MAC 地址 / 用户账号 / 邮箱 / 任何能识别个人或设备的信息
- **生成时机**：daemon 首次启动时，若持久化路径文件不存在则生成并写入

### 2.2 持久化路径

| OS | 路径 | 说明 |
|----|------|------|
| macOS | `~/Library/Caches/sieve/install-id` | Phase 1 首发 |
| Linux | `$XDG_CACHE_HOME/sieve/install-id`（默认 `~/.cache/sieve/install-id`） | Phase 2 跨平台时落地 |
| Windows | `%LOCALAPPDATA%\sieve\Cache\install-id` | Phase 2 跨平台时落地 |

### 2.3 `cache_dir()` 跨平台抽象

```rust
/// 返回 Sieve 缓存目录（含父目录创建）。
/// Phase 1 仅实现 macOS；Linux / Windows 分支预留占位以防编译失败。
pub fn cache_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        // ~/Library/Caches/sieve/
        dirs::cache_dir()
            .expect("cannot locate macOS cache dir")
            .join("sieve")
    }
    #[cfg(target_os = "linux")]
    {
        // $XDG_CACHE_HOME/sieve/ 或 ~/.cache/sieve/
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache"))
            .join("sieve")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::cache_dir()
            .expect("cannot locate Windows cache dir")
            .join("sieve")
    }
}

pub fn install_id_path() -> PathBuf {
    cache_dir().join("install-id")
}
```

### 2.4 文件权限与读写约束

- 父目录权限：`0700`（仅 owner 可读）
- 文件权限：`0600`
- 写入时：`fs::write` 后显式 `set_permissions(0o600)`
- 读取时：若文件不存在 → 新生成 UUIDv4 → 写入 → 返回
- 用户主动删除该文件 → 下次启动重新生成新 UUID，被视为新装机（接受统计噪声）

### 2.5 幂等性保证

```
启动时：
  if install_id_path().exists() {
      read → parse UUID → use
  } else {
      generate UUIDv4 → write 0600 → use
  }
```

**单元测试要求**：`install_id_is_idempotent`——多次调用返回同一 UUID；删除文件后调用返回新 UUID。

---

## 3. Manifest 协议 v0.1

### 3.1 客户端请求

```
GET https://updates.sieveai.dev/v1/manifest
  ?v=<client_version>          # 必选，语义版本 如 "0.3.1"
  &os=<mac|linux|windows>      # 必选，小写
  &arch=<x64|arm64>            # 必选
  &uid=<UUIDv4>                # 必选（除非 SIEVE_NO_TELEMETRY 设置，此时省略）
  &ch=<stable|beta>            # 可选，发布通道，默认 stable；服务端当前忽略 stable 以外的值
```

**传输层约束**：
- 仅 TLS 1.2+（`rustls` 默认已满足，拒绝降级）
- 不带 cookie、不带 `Authorization` header
- `User-Agent: sieve-updater/<v>`（仅客户端版本，不暴露系统版本详情）
- manifest 接口走自有服务器不挂 CDN（保证每次请求都能记日志，作为装机量信标）
- 规则正文走 CDN（`cdn.sieveai.dev`，带 sha256 + ed25519 签名）

**请求构造示例**：

```
GET https://updates.sieveai.dev/v1/manifest?v=0.3.1&os=mac&arch=arm64&uid=550e8400-e29b-41d4-a716-446655440000&ch=stable HTTP/1.1
Host: updates.sieveai.dev
User-Agent: sieve-updater/0.3.1
Accept: application/json
```

**`SIEVE_NO_TELEMETRY` 时省略 uid**：

```
GET https://updates.sieveai.dev/v1/manifest?v=0.3.1&os=mac&arch=arm64&ch=stable HTTP/1.1
```

### 3.2 服务端响应 JSON Schema

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

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `schema` | integer | 协议版本号，当前 `1`；客户端收到未知版本时 WARN + 跳过规则更新（不崩溃） |
| `rules.version` | string | 规则包版本标识，`YYYY.MM.DD.N` 格式 |
| `rules.url` | string | 规则包 CDN 下载地址（HTTPS） |
| `rules.sha256` | string | 规则包 64 位小写 hex sha256 |
| `rules.size` | integer | 规则包字节数（用于进度显示 + 大小合理性校验） |
| `rules.signature` | string | `"ed25519:"` 前缀 + 规则包内容的 ed25519 签名（base64url 编码） |
| `client.latest` | string | 最新客户端版本（语义版本） |
| `client.min_supported` | string | 最低支持版本；客户端版本低于此时打印 deprecation 警告 |
| `client.deprecation_notice` | string? | 可选弃用说明文本，非 null 时 log warn 打印 |
| `next_check_after_seconds` | integer | 服务端指定下次检查间隔（秒）；客户端用此值覆盖本地 `check_interval_hours` 默认值。灰度发布 / 限流应急时服务端可动态调节 |

**缺失字段处理**：

- `rules` 字段缺失 → 跳过规则更新（规则无变化），仍记为一次成功的 manifest 请求
- `client` 字段缺失 → 跳过版本告警，视为服务端可选功能未启用
- `next_check_after_seconds` 缺失 → 使用本地配置默认值（21600 秒）

### 3.3 规则包下载与原子替换流程（已实现，2026-05-05）

规则包下载在 manifest 请求成功后、当前规则版本 ≠ `rules.version` 时触发
（通过读取 `<cache_dir>/rules/latest_version.json` 判断当前版本）：

```
1.  读 <cache_dir>/rules/latest_version.json → current_version
    ├─[current_version == rules.version]──→ 跳过（已最新），仅 log debug
    └─[不同或文件不存在]──→ 触发下载

2.  download_rules(rules.url, MAX_RULES_SIZE=50MiB)
    ├─ TLS 1.2+（hyper-rustls，https_only）
    ├─ User-Agent: sieve-updater/<v>
    ├─ 逐帧累积 bytes，超出 50 MiB → ResponseTooLarge
    └─ 非 200 → Http error

3.  install_rules(payload, sha256, signature, version, dest_dir)
    ├─ a. verify_sha256(payload, rules.sha256)
    │     失败 → Sha256Mismatch → 无 tmp 文件残留
    ├─ b. verify_signature(payload, rules.signature)
    │     TRUSTED_PUBKEY = None → WARN + 跳过（不静默）
    │     公钥已配置 → ed25519-dalek verify；失败 → Ed25519Failed
    ├─ c. zstd 解压（以 \xFD\x2F\xB5\x28 magic 判断）
    │     非 zstd → fallback 直接当解压结果（测试友好）
    │     zstd 损坏 → DecompressFailed
    ├─ d. 写 <dest_dir>/.tmp-<version>.json（mode 0644，Unix）
    ├─ e. atomic rename .tmp → <dest_dir>/<version>.json
    ├─ f. 更新 <dest_dir>/current.json 符号链接（Unix）
    │     先 unlink 旧链接再 symlink 新文件
    │     Windows fallback：symlink 失败时 copy
    └─ g. 原子写 <dest_dir>/latest_version.json
          {"version":"<v>","installed_at":<unix_ts>,"sha256":"<v>"}
          （tmp + rename）

4.  成功 → tracing::info!(version, path, "rules installed")
    失败 → tracing::error! → 保留旧版规则继续运行，等下个 6h 周期

5.  download 失败时使用与 manifest fetch 相同的指数退避（1s/4s/16s × 3）
```

**staging 目录**：`<cache_dir>/rules/`（自动 mkdir 0700）。旧版本文件保留不删除（回滚预备，`sieve-rules` CLI 后续负责）。

**与 sieve-rules 热加载的边界**：本流程仅落盘 staging，**不触发热加载**。热加载由 `sieve-rules` 负责，触发入口为 `POST /_sieve/v1/rules/refresh`（见 [api-reference.md §2.2.5](../api/api-reference.md)），或 daemon 重启时读取 `current.json`。TODO：热加载接通后在此处补 cross-reference。

---

## 4. 客户端实现流程

### 4.1 启动序列

```
daemon::run() 启动时：
  1. 读取 / 生成 install-id（§2）
  2. 解析环境变量优先级（§6）
  3. 若 SIEVE_NO_UPDATE 已设置 → 打印 banner → 跳出，不启动 updater
  4. tokio::spawn updater_task()  ← 后台 task，不阻塞 daemon 启动
  5. daemon 正常处理请求

updater_task() 首次运行：
  1. 立即执行一次 manifest 检查（startup check）
  2. 进入 loop { sleep(interval); manifest_check() }
```

**关键约束**：updater_task 在 `daemon::run` spawn 后台运行，**不在 hot path**，任何 updater 失败不影响 daemon 主流程。

### 4.2 流程图（文字版）

```
daemon 启动
    │
    ├── 读/生成 install-id
    ├── 解析 env var（SIEVE_NO_UPDATE / SIEVE_NO_TELEMETRY / SIEVE_UPDATE_URL）
    │
    ├─[SIEVE_NO_UPDATE 已设]──→ 打印 banner ──→ 返回（不启动 updater）
    │
    └──→ spawn updater_task（tokio 后台 task）
              │
              └──→ 立即执行一次 manifest_check()
                        │
                        ├── 成功 → 规则更新（如有新版本） → 等待 next_check_after_seconds
                        └── 失败 → 指数退避重试（§4.3）
                                    │
                                    ├── 重试耗尽 → log error → 等待 6h 周期再试
                                    └── 任意重试成功 → 正常流程
```

### 4.3 失败重试策略（指数退避）

```
首次失败 → 等待 1s → 重试 #1
重试 #1 失败 → 等待 4s → 重试 #2
重试 #2 失败 → 等待 16s → 重试 #3（最终尝试）
重试 #3 失败 → log error（不 panic，不退出 daemon）→ 等待下个 6h 周期
```

| 参数 | 值 |
|------|-----|
| 最大重试次数 | 3 |
| 初始等待 | 1 秒 |
| 退避因子 | 4 |
| 重试耗尽行为 | log error → 等下个周期，不影响 daemon |

**实现**：

```rust
const RETRY_DELAYS_SECS: &[u64] = &[1, 4, 16];

async fn manifest_check_with_retry(ctx: &UpdaterCtx) -> Result<(), UpdaterError> {
    for (attempt, &delay) in RETRY_DELAYS_SECS.iter().enumerate() {
        match manifest_check(ctx).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                tracing::warn!("manifest check failed (attempt {}): {}", attempt + 1, e);
                if attempt < RETRY_DELAYS_SECS.len() - 1 {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }
            }
        }
    }
    Err(UpdaterError::RetriesExhausted)
}
```

### 4.4 定时器

```rust
let mut interval = tokio::time::interval(Duration::from_secs(ctx.check_interval_secs));
interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
loop {
    interval.tick().await;
    if let Err(e) = manifest_check_with_retry(&ctx).await {
        tracing::error!("updater: all retries exhausted: {}", e);
        // 不 panic，等下一个 interval tick
    }
    // 若服务端返回 next_check_after_seconds，更新 interval
    if let Some(secs) = ctx.next_check_override.take() {
        interval = tokio::time::interval(Duration::from_secs(secs));
    }
}
```

---

## 5. 签名校验

### 5.1 双重校验流程

```
规则包下载完成后：
  1. sha256 校验：SHA-256(file_bytes) == manifest.rules.sha256
     失败 → log error "sha256 mismatch" → 删除临时文件 → 中止
  2. ed25519 校验：verify(TRUSTED_PUBKEY, file_bytes, signature)
     失败 → log error "ed25519 signature invalid" → 删除临时文件 → 中止
  两项全通过 → 原子替换规则文件
```

### 5.2 公钥管理

```rust
/// 编译期硬编码的 ed25519 公钥（trusted pubkey）。
/// Phase 1 GA 前填入真实公钥（TODO-14 GCP KMS 落地后更新）。
/// 当前为 None 占位——签名校验在公钥为 None 时 WARN + 跳过（不静默通过）。
const TRUSTED_PUBKEY: Option<&[u8]> = None; // TODO-14: 替换为真实公钥字节
```

**公钥未配置时的行为**（开发阶段 / TODO-14 落地前）：

```
if TRUSTED_PUBKEY.is_none() {
    tracing::warn!(
        "updater: ed25519 public key not configured (TODO-14), \
         skipping signature verification — DO NOT ship to production"
    );
    // 跳过 ed25519 校验，仅做 sha256 校验
    // 绝不静默通过（此处必须有 log warn）
}
```

> **重要**：公钥未配置时 WARN + 跳过校验，绝不静默通过。GA 前必须填入真实公钥（TODO-14）。

### 5.3 与 ADR-006 的关系

ADR-006 描述的是 Sieve 二进制自身的 sigstore + reproducible build 签名体系。SPEC-006 的 ed25519 签名保护的是规则包分发，两者独立：

- 规则包签名：ed25519，trusted pubkey 编译期硬编码在 `sieve-updater` 二进制中
- 二进制签名：sigstore / cosign，通过 rekor 透明日志验证

**防 CDN 被注入**（ADR-030 §3 说明）：规则正文走 CDN，CDN 节点可能被攻击者注入恶意内容。ed25519 签名在客户端校验，即使 CDN 被完全攻陷也无法推送恶意规则。

---

## 6. 三个环境变量开关

### 6.1 优先级规则

```
优先级（高 → 低）：环境变量 > sieve.toml [update] 段 > 默认值
```

"任何非空值"视为启用该开关，空字符串或未设置 = 默认行为（Unix-style，参考 `NO_COLOR` 惯例）。

### 6.2 变量说明

| 变量 | 作用 | 默认 | 典型使用场景 |
|------|------|------|------------|
| `SIEVE_NO_UPDATE` | 完全跳过更新检查（不发任何请求，规则冻结，自然也无遥测） | 未设 | 离线 / 隔离网络 / 审计期 / CI 跑测试 |
| `SIEVE_NO_TELEMETRY` | 仍发更新请求但省略 `uid` 字段（仍能拿到规则更新，只是不参与装机统计） | 未设 | 隐私敏感场景 |
| `SIEVE_UPDATE_URL` | 覆盖默认更新源 URL | 未设（使用 `https://updates.sieveai.dev/v1/manifest`） | 企业自托管镜像 / 私有内网 / 本地 mock 测试 |

### 6.3 启动 Banner（`SIEVE_NO_UPDATE` 强制可见）

`SIEVE_NO_UPDATE` 设置时，daemon 启动日志**必须**打印：

```
update check disabled by SIEVE_NO_UPDATE
```

**规范**：
- 使用 `tracing::info!` 等级（确保默认日志等级下可见）
- 在 updater_task 启动前判断，检测到即打印
- 目的：防止用户忘了设过此变量却奇怪规则不更新

### 6.4 解析代码示例

```rust
pub struct UpdaterConfig {
    pub no_update: bool,        // SIEVE_NO_UPDATE != ""
    pub no_telemetry: bool,     // SIEVE_NO_TELEMETRY != ""
    pub update_url: String,     // SIEVE_UPDATE_URL 或 toml 或默认值
    pub check_interval_secs: u64,
}

impl UpdaterConfig {
    pub fn from_env_and_toml(toml: &UpdateToml) -> Self {
        let no_update = std::env::var("SIEVE_NO_UPDATE")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
            || !toml.enabled;

        let no_telemetry = std::env::var("SIEVE_NO_TELEMETRY")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
            || !toml.telemetry;

        let update_url = std::env::var("SIEVE_UPDATE_URL")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| toml.url.clone());

        let check_interval_secs = toml.check_interval_hours * 3600;

        Self { no_update, no_telemetry, update_url, check_interval_secs }
    }
}
```

---

## 7. 配置 `[update]` 段

### 7.1 `sieve.toml` 完整字段

```toml
[update]
enabled = true              # 等价 SIEVE_NO_UPDATE（false = 禁用更新检查）
telemetry = true            # 等价 SIEVE_NO_TELEMETRY（false = 省略 uid）
url = "https://updates.sieveai.dev/v1/manifest"  # 等价 SIEVE_UPDATE_URL
check_interval_hours = 6    # 定时检查间隔，默认 6h（= 每天 4 次）
                            # 注意：服务端 next_check_after_seconds 可动态覆盖此值
channel = "stable"          # 发布通道，当前只支持 "stable"；Phase 2 加 "beta"
```

### 7.2 字段语义

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `enabled` | bool | `true` | 等价 `SIEVE_NO_UPDATE`；false = 完全禁用更新检查 |
| `telemetry` | bool | `true` | 等价 `SIEVE_NO_TELEMETRY`；false = 省略 uid 字段 |
| `url` | string | `"https://updates.sieveai.dev/v1/manifest"` | 等价 `SIEVE_UPDATE_URL` |
| `check_interval_hours` | u16 | `6` | 检查间隔（小时），不建议用户修改；服务端 `next_check_after_seconds` 优先 |
| `channel` | string | `"stable"` | 发布通道；当前服务端忽略非 `stable` 值；Phase 2 加 `beta` 支持 |

### 7.3 优先级再确认

```
环境变量 > [update] toml 字段 > 默认值

例：
  SIEVE_NO_UPDATE=1 时，即使 toml 中 enabled=true，仍禁用更新
  SIEVE_UPDATE_URL=http://internal/v1/manifest 时，覆盖 toml.url
```

---

## 8. crate 边界

### 8.1 `sieve-updater` 独立 crate

| 属性 | 值 |
|------|-----|
| crate 名 | `sieve-updater` |
| 路径 | `crates/sieve-updater/` |
| 类型 | lib crate（供 `sieve-cli` 调用；GUI 仓未来可复用） |
| 主入口 | `pub async fn run_updater(config: UpdaterConfig) -> !` |

### 8.2 职责边界

**允许做**：
- manifest GET 请求（通过 `reqwest` 或 `hyper-rustls`）
- install-id 生成与读写（`cache_dir()` 抽象）
- 6h 定时器（`tokio::time::interval`）
- ed25519 签名校验（`ed25519-dalek` crate）
- sha256 校验（`sha2` crate）
- 三个环境变量解析
- `[update]` toml 段解析
- 规则包下载到临时文件 + 原子替换（只写文件，不解析规则）
- 失败重试（指数退避）

**禁止做**：
- 参与任何 HTTP 请求处理路径（hot path）
- 依赖 `sieve-core` 业务逻辑（仅允许依赖通用工具 crate）
- 依赖 `sieve-rules`（规则文件的原子替换属于 `sieve-rules`，updater 只负责下载）
- 依赖 `sieve-ipc`（updater 不通过 IPC 通道通知 GUI）
- 任何 panic 或 `unwrap()` 在非测试路径

### 8.3 依赖白名单

```toml
[dependencies]
tokio = { version = "1", features = ["rt", "time", "fs"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"
ed25519-dalek = "2"
tracing = "0.1"
thiserror = "1"
```

（`anyhow` 仅在 `sieve-cli` 层允许，updater 作为 lib crate 使用 `thiserror`）

---

## 9. 与其他模块的边界关系

### 9.1 与 ADR-003 amended 的关系

ADR-003 原条款「绝对禁止 telemetry 自动上报」被 ADR-030 部分修订。修订后：

- 独立心跳通道：**仍禁止**
- 更新通道附带匿名 install-id：**允许**（唯一例外，且可通过 `SIEVE_NO_TELEMETRY` 关闭 uid 字段）
- token verifier 联网验证：**永久禁止**，本 SPEC 不涉及

manifest 请求是「绝对禁止 telemetry」反模式的**唯一允许例外**，边界：

| 允许上传 | 禁止上传 |
|---------|---------|
| Sieve 版本号 | prompt / response 内容 |
| 操作系统类型 | API key |
| CPU 架构 | 用户账号 / 邮箱 |
| UUIDv4 install-id（可通过 SIEVE_NO_TELEMETRY 关闭） | MAC 地址 / 设备序列号 |
| 发布通道（stable/beta） | 任何能识别个人或设备的信息 |

### 9.2 与 ADR-006 的关系

ADR-006 的 Tier 1 要求（macOS / Linux sigstore + reproducible build）适用于 `sieve-updater` crate 编译进 `sieve` 主二进制的部分。规则包的 ed25519 签名是 ADR-006 规则分发签名链的客户端验证端。

### 9.3 与 `sieve-rules` 的关系

- `sieve-updater` 只负责：manifest 拉取 → 规则包下载 → 签名校验 → 写入临时文件
- `sieve-rules` 负责：规则文件原子替换 → 热加载到运行时（`arc-swap`）
- 两者通过文件系统路径解耦（updater 写入 `~/.sieve/rules/<version>.json.zst.tmp`，rules 执行原子 rename）

---

## 10. 失败模式与降级（已实现，2026-05-05）

### 10.1 网络错误

- DNS 解析失败 / 连接超时 → 指数退避重试（manifest：§4.3；规则下载：同样 1s/4s/16s × 3）
- TLS 握手失败 → 同上（hyper-rustls 强制 TLS 1.2+，不降级）
- 所有重试耗尽 → `tracing::error!` → 等下个 6h 周期
- **不 panic，不退出 daemon**

### 10.2 响应错误

- HTTP 非 200 → `UpdaterError::Http` → 纳入重试
- JSON 解析失败 → `UpdaterError::SerdeJson` → log warn + 跳过本次更新
- `schema` 版本未知 → log warn + 跳过规则更新，记为成功请求（遥测侧仍有价值）
- `rules` 字段缺失 → log debug "no rules update field" + 正常结束
- 规则包超 50 MiB → `UpdaterError::ResponseTooLarge` → log error + 等下个周期

### 10.3 签名校验失败

| 场景 | 行为 | 错误类型 |
|------|------|---------|
| sha256 不匹配 | log error + 无 .tmp 残留 + 保留旧规则 | `UpdaterError::Sha256Mismatch` |
| ed25519 无效（公钥已配置） | log error + 无 .tmp 残留 + 保留旧规则 | `UpdaterError::Ed25519Failed` |
| 公钥未配置（TRUSTED_PUBKEY = None） | log **warn** + 跳过 ed25519（仍做 sha256） | — |
| zstd 解压失败（损坏压缩包） | log error + 保留旧规则 | `UpdaterError::DecompressFailed` |

> **关键原则**：任何校验失败均 fail-closed，保留旧版规则继续运行，不弹窗不 panic。

### 10.4 与 sieve-rules 热加载的边界

`sieve-updater` 仅负责将规则包落盘到 `<cache_dir>/rules/` staging 目录，**不触发任何热加载**。热加载属于 `sieve-rules` 的职责：

- **触发入口**：`POST /_sieve/v1/rules/refresh`（见 [api-reference.md §2.2.5](../api/api-reference.md)）
- **重启加载**：daemon 重启时读取 `current.json` 符号链接
- **TODO**：热加载接通后，在 runner.rs `process_manifest` 成功路径补 IPC notify（目前有意省略）

### 10.5 文件系统错误

- `cache_dir()` 失败 → log error + rules 更新禁用（install-id 仍可用，遥测照常）
- `create_dir_secure(rules_dir)` 失败 → `UpdaterError::Io` → log error + 等下个周期
- install-id 写入失败 → 每次启动重新生成（不持久化），接受高估 DAU 的统计噪声

---

## 11. 服务端日志 Schema（参考，实现待 TODO-15）

服务端只存以下字段（详见 [data-model.md §X 服务端日志](../design/data-model.md)）：

```
ts (TIMESTAMP) | uid (UUID) | v (TEXT) | os (TEXT) | arch (TEXT) | ch (TEXT) | country (TEXT, geoip)
```

- 不存原始 IP：geoip 解析后丢弃（或哈希后保留 ≤7 天用于反滥用，过期硬删）
- DAU = `COUNT(DISTINCT uid) WHERE date = today`
- MAU / 留存曲线 / 版本分布 / 平台分布全从这一张日志表算

---

## 12. 测试覆盖矩阵

| 测试 ID | 场景 | 验收标准 |
|---------|------|---------|
| `install_id_is_idempotent` | 多次调用 `get_or_create_install_id()` | 返回同一 UUID |
| `install_id_regenerates_after_delete` | 删除 install-id 文件后调用 | 返回新 UUID，文件重新创建 |
| `env_no_update_disables_check` | `SIEVE_NO_UPDATE=1` | 不发任何 HTTP 请求，打印 banner |
| `env_no_telemetry_omits_uid` | `SIEVE_NO_TELEMETRY=1` | 请求 URL 无 `uid=` 参数 |
| `env_update_url_overrides_default` | `SIEVE_UPDATE_URL=http://localhost:8080/v1/manifest` | 请求打到 localhost |
| `manifest_missing_rules_field` | 服务端响应 `{"schema":1,"client":{...}}` 无 `rules` | 不更新规则，不报错 |
| `manifest_missing_next_check` | 服务端响应无 `next_check_after_seconds` | 使用默认 21600 秒 |
| `sha256_mismatch_aborts_update` | 下载完成后 sha256 不匹配 | 删除临时文件，保留旧规则 |
| `ed25519_invalid_aborts_update` | 签名无效 | 删除临时文件，保留旧规则，log error |
| `ed25519_pubkey_none_warns` | `TRUSTED_PUBKEY = None` | log warn，跳过 ed25519，继续 sha256 |
| `network_error_retries` | 前 2 次请求失败，第 3 次成功 | 成功完成更新 |
| `network_error_exhausted` | 全部 3 次重试失败 | log error，daemon 不退出，等下个周期 |
| `toml_update_section_parsed` | `[update] enabled=false` | 等价 `SIEVE_NO_UPDATE` |
| `env_overrides_toml` | `SIEVE_NO_UPDATE=1` 且 toml `enabled=true` | env var 胜出，禁用更新 |

---

## 13. 隐私声明文案（供 README / onboarding 使用）

> Sieve 每天 4 次连接更新服务器获取最新规则。请求会附带：Sieve 版本、操作系统、CPU 架构、一个本地随机生成的安装 ID（用于统计装机量，不绑定您的账号或设备）。Sieve 不上传 prompt、response、API key 或任何使用记录。可在设置中关闭装机统计（规则更新不受影响），也可通过环境变量 `SIEVE_NO_UPDATE=1` 完全禁用更新检查。

---

## 相关文档

- [ADR-030](../design/ADR-030-update-telemetry-channel.md) — 更新通道复用为遥测信标（核心决策源）
- [ADR-029](../design/ADR-029-free-first-defer-monetization.md) — 装机量优先，延后商业化
- [ADR-003 amended](../design/ADR-003-local-only-no-cloud-verifier.md) — 网络边界修订（唯一允许例外）
- [ADR-006](../design/ADR-006-sigstore-reproducible-build.md) — Sigstore 签名 + Reproducible Build
- [docs/api/api-reference.md §8](../api/api-reference.md) — manifest 接口 API 参考
- [docs/design/data-model.md §7](../design/data-model.md) — 服务端日志表 schema
- [docs/guides/development.md §X](../guides/development.md) — 三个环境变量开发者指南
- [docs/guides/deployment.md §X](../guides/deployment.md) — 企业自托管镜像部署
- [docs/specs/INDEX.md](INDEX.md) — SPEC 索引
