# ADR-030: 更新通道复用为遥测信标 + Install UUID + 环境变量开关

## 状态

**Accepted**

> 决策日期：2026-05-05
> 范围：规则更新通道协议 + 匿名装机统计信标 + 用户可关闭开关
> 关联：[ADR-003 amended](./ADR-003-local-only-no-cloud-verifier.md)（网络边界，唯一允许的主动出站）/ [ADR-006](./ADR-006-sigstore-reproducible-build.md)（签名分发）/ [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)（manifest 协议与 updater 详细规格）

---

## 背景

Sieve 需要持续拉取规则更新——规则不更新就会过期失效。同时需要基础的匿名版本 / 平台分布统计，用于判断 deprecation 时机与灰度推送节奏。

为避免额外网络流量与独立心跳通道的复杂度，复用更新检查请求本身作为一次匿名信标，而非另开一条遥测通道。功能描述与可关闭开关对用户公开透明（见 §6 隐私声明）。

## 决策

### 1. 复用更新检查作为信标

规则更新检查请求本身既拉取规则，也作为一次匿名签到。**频率**：每天 4 次（每 6 小时一次，固定定时器），启动时立即查一次。服务端按 `日期 + uid` 去重计算装机统计，同一安装当天多次请求只计一次。

### 2. Install UUID

首次安装时生成 UUIDv4，纯随机，不掺任何设备 / 账号 / MAC / 序列号信息。

| OS | 持久化路径 |
|----|------|
| macOS | `~/Library/Caches/sieve/install-id` |
| Linux | `$XDG_CACHE_HOME/sieve/install-id`（默认 `~/.cache/sieve/`） |
| Windows | `%LOCALAPPDATA%\sieve\Cache\install-id` |

文件权限 `0600`；用户主动删除该文件 → 下次启动重新生成新 UUID（接受这一统计噪声）；永远只在本地生成，不上传任何账号 / 设备标识。首发 macOS only，跨平台路径统一封装为 `cache_dir() -> PathBuf` 按 `cfg!(target_os = ...)` 返回。

### 3. Manifest 协议 v0.1

```
GET https://updates.sieveai.dev/v1/manifest
  ?v=<client_version>          # 必选
  &os=<mac|linux|windows>      # 必选
  &arch=<x64|arm64>            # 必选
  &uid=<UUIDv4>                # 必选（除非 SIEVE_NO_TELEMETRY 设置）
  &ch=<stable|beta>            # 可选，默认 stable
```

仅 TLS 1.2+；不带 cookie / Auth header；User-Agent 仅 `sieve-updater/<v>`。`manifest` 接口走自有服务器（不挂 CDN），规则正文走 CDN 带 sha256 + ed25519 签名（[ADR-006](./ADR-006-sigstore-reproducible-build.md)）。响应含 `rules`（url / sha256 / size / signature）+ `client`（latest / min_supported）+ `next_check_after_seconds`（服务端动态调节频率）。完整 schema 见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

### 4. 服务端日志字段（匿名）

只存 `ts | uid | v | os | arch | ch | country(geoip)`。**不存原始 IP**（geoip 解析后丢弃或哈希后短期保留用于反滥用）、不存 User-Agent 详情、不存任何用户输入。

### 5. 三个环境变量开关（Unix-style，参考 NO_COLOR / NO_PROXY 惯例）

| 变量 | 作用 |
|------|------|
| `SIEVE_NO_UPDATE` | 完全跳过更新检查（不发请求，规则冻结，自然无信标） |
| `SIEVE_NO_TELEMETRY` | 仍发更新请求但省略 `uid` 字段（要规则更新但不参与统计） |
| `SIEVE_UPDATE_URL` | 覆盖默认更新源 URL（企业自托管镜像 / 私有内网） |

优先级：环境变量 > 配置文件 > 默认值。检测到 `SIEVE_NO_UPDATE` 时，启动日志必须打印 `update check disabled by SIEVE_NO_UPDATE`。

### 6. 隐私声明文案

> Sieve 每天 4 次连接更新服务器获取最新规则。请求会附带：Sieve 版本、操作系统、CPU 架构、一个本地随机生成的安装 ID（用于统计装机量，不绑定您的账号或设备）。Sieve 不上传 prompt、response、API key 或任何使用记录。可在设置中关闭装机统计（规则更新不受影响），也可通过 `SIEVE_NO_UPDATE=1` 完全禁用更新检查。

### 7. 配置 toml 等价开关

```toml
[update]
enabled = true              # 等价 SIEVE_NO_UPDATE
telemetry = true            # 等价 SIEVE_NO_TELEMETRY
url = "https://updates.sieveai.dev/v1/manifest"  # 等价 SIEVE_UPDATE_URL
check_interval_hours = 6
```

## 影响

### 正面影响
- 隐私边界清晰、可向用户言简意赅地解释；三个 env var 给企业部署、CI、隐私敏感用户留足出口。
- 规则正文走 CDN + 客户端签名校验，符合 [ADR-006](./ADR-006-sigstore-reproducible-build.md) Tier 1 要求。
- 每次请求带版本 / OS / 架构，后续 deprecation 与灰度推送都有数据基础。

### 负面影响
- manifest 接口不挂 CDN，流量全部打到自有服务器，需要基本的反滥用与限流。
- ed25519 签名密钥管理是新增运维负担（密钥泄露 = 规则分发被劫持的最大风险点）。
- 用户主动删除 `install-id` 文件会被算成新装机，装机统计存在小幅高估（可接受）。

### 需要更新的文档
- [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)：manifest 协议 + updater 模块详细设计（权威规格）
- [api-reference.md](../api/api-reference.md) §8 manifest 接口
- [development.md](../guides/development.md) 三个环境变量说明
- [deployment.md](../guides/deployment.md) 企业自托管镜像（`SIEVE_UPDATE_URL`）

## 相关文档
- [SPEC-006: 更新通道 + 装机遥测 manifest 协议](../specs/SPEC-006-update-and-telemetry.md)
- [ADR-003 amended](./ADR-003-local-only-no-cloud-verifier.md) / [ADR-006](./ADR-006-sigstore-reproducible-build.md)
