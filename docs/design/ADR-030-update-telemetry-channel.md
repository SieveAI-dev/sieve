# ADR-030: 更新通道复用为遥测信标 + Install UUID + 三个环境变量开关

## 状态
**Accepted**
> 决策日期:2026-05-05
> 范围:规则更新通道设计 + 装机量/DAU/留存遥测方案
> 关联:[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)(**本 ADR 部分修订 ADR-003 的「绝对禁止 telemetry」反模式条款**,详见 ADR-003 §决策段 admonition) / [ADR-029](./ADR-029-free-first-defer-monetization.md)(免费优先,装机量为唯一指标) / [ADR-006](./ADR-006-sigstore-reproducible-build.md)(签名分发)
> 待落地项:域名 / ed25519 签名密钥管理 / 服务端实现(Cloudflare Workers vs 自托管)/ 发布通道是否引入 beta / sieve-updater crate 实现

## 背景

[ADR-029](./ADR-029-free-first-defer-monetization.md) 把装机量定为 GA 前唯一指标,需要可量化的 DAU / WAU / MAU / 留存数据,否则:

- 不知道产品是否在被使用
- 不知道哪些版本/平台占比,无法判断升级推送时机
- 后续任何商业化谈判都缺筹码

直接做单独遥测心跳通道有两个问题:

1. **用户容易禁用**:独立心跳意图明显,隐私敏感用户会一键关闭,数据失真
2. **不必要的网络流量**:Sieve 本身就需要持续拉取规则更新(规则不更新就失效),再加心跳是重复

## 决策

### 1. 复用更新检查作为遥测信标

Sieve 不开单独的遥测通道。规则更新检查请求本身既是功能(拉规则),也是遥测信标(每次请求 = 一次签到)。用户没有动机禁用更新——禁用了规则就过期失效。

**频率**:每天 4 次,即每 6 小时一次,固定定时器。
- 启动时立即查一次
- 之后按 6h 周期触发,即使内容无变化也照常发请求
- 服务端按 `日期 + uid` 去重算 DAU,同一安装当天多次请求只计一次

### 2. Install UUID 方案

**生成**:首次安装时生成 UUIDv4,纯随机,不掺任何设备/账号/MAC/序列号信息。

**持久化路径(按 OS)**:

| OS | 路径 |
|----|------|
| macOS | `~/Library/Caches/sieve/install-id` |
| Linux | `$XDG_CACHE_HOME/sieve/install-id`(默认 `~/.cache/sieve/`) |
| Windows | `%LOCALAPPDATA%\sieve\Cache\install-id` |

**约束**:
- 文件权限 `0600`
- 用户主动删除该文件 → 下次启动重新生成新 UUID,被视为新装机(接受这个统计噪声)
- 永远只在本地生成,不上传任何账号/邮箱/MAC/序列号

**首发实现**:macOS only(见 [ADR-029](./ADR-029-free-first-defer-monetization.md));Linux / Windows 路径在 Phase 2 跨平台时落地,建议封装统一的 `cache_dir() -> PathBuf` 函数按 `cfg!(target_os = ...)` 返回。

### 3. Manifest 协议 v0.1

**客户端请求**:

```
GET https://updates.sieve.app/v1/manifest
  ?v=<client_version>          # 必选,如 0.3.1
  &os=<mac|linux|windows>      # 必选
  &arch=<x64|arm64>            # 必选
  &uid=<UUIDv4>                # 必选(除非 SIEVE_NO_TELEMETRY 设置)
  &ch=<stable|beta>            # 可选,发布通道,默认 stable
```

- 仅 TLS 1.2+
- 不带 cookie / Auth header
- User-Agent 仅 `sieve-updater/<v>`,不暴露详细系统版本
- `manifest` 接口必须走自己的服务器(不挂 CDN),保证每次请求都能记日志
- 规则正文走 CDN,带 sha256 + ed25519 签名(防 CDN 被注入,见 [ADR-006](./ADR-006-sigstore-reproducible-build.md))

**服务端响应**:

```json
{
  "schema": 1,
  "rules": {
    "version": "2026.05.05.1",
    "url": "https://cdn.sieve.app/rules/2026.05.05.1.json.zst",
    "sha256": "ab12...",
    "size": 184320,
    "signature": "ed25519:..."
  },
  "client": {
    "latest": "0.4.0",
    "min_supported": "0.2.0",
    "deprecation_notice": null
  },
  "next_check_after_seconds": 21600
}
```

`next_check_after_seconds` 让服务端能动态调节频率(灰度发布、限流应急)。

### 4. 服务端日志字段

**只存这些字段**:`ts | uid | v | os | arch | ch | country(geoip)`

- **不存原始 IP**:geoip 解析后丢弃,或哈希后保留 ≤7 天用于反滥用,过期硬删
- 不存 User-Agent 详情、不存 Referer、不存任何用户输入
- DAU = `COUNT(DISTINCT uid) WHERE date = today`
- MAU / 留存曲线 / 版本分布 / 平台分布全部从这一张日志表算

### 5. 三个环境变量开关(Unix-style,参考 NO_COLOR / NO_PROXY 惯例)

任何非空值视为启用该开关,空或未设置 = 默认行为。优先级:**环境变量 > 配置文件 > 默认值**。

| 变量 | 作用 | 典型场景 |
|------|------|----------|
| `SIEVE_NO_UPDATE` | 完全跳过更新检查(不发请求,规则冻结,自然也无遥测) | 离线/隔离网络/审计期/CI 跑测试 |
| `SIEVE_NO_TELEMETRY` | 仍发更新请求但省略 `uid` 字段 | 想要规则更新但不参与统计 |
| `SIEVE_UPDATE_URL` | 覆盖默认更新源 URL | 企业自托管镜像/私有内网/自托管测试 |

**强制可见性**:检测到 `SIEVE_NO_UPDATE` 时,启动日志或 banner 必须打印一行 `update check disabled by SIEVE_NO_UPDATE`,避免用户忘了设过此变量却奇怪规则不更新。

### 6. 隐私声明文案(写入首次启动 onboarding + README + 隐私政策页)

> Sieve 每天 4 次连接更新服务器获取最新规则。请求会附带:Sieve 版本、操作系统、CPU 架构、一个本地随机生成的安装 ID(用于统计装机量,不绑定您的账号或设备)。Sieve 不上传 prompt、response、API key 或任何使用记录。可在设置中关闭装机统计(规则更新不受影响),也可通过环境变量 `SIEVE_NO_UPDATE=1` 完全禁用更新检查。

### 7. 配置 toml 等价开关

`sieve.toml` 加 `[update]` 段(GA 前可只接 env var,toml 字段在 Phase 2 落地):

```toml
[update]
enabled = true              # 等价 SIEVE_NO_UPDATE
telemetry = true            # 等价 SIEVE_NO_TELEMETRY
url = "https://updates.sieve.app/v1/manifest"  # 等价 SIEVE_UPDATE_URL
check_interval_hours = 6    # 6h × 4 = 4 次/天,不暴露给用户改
```

环境变量优先级始终高于配置文件。

## 已排除的方案

- **单独心跳通道**:用户容易禁用,隐私感知差
- **仅按 IP+UA hash 去重**:算不了留存(明天的同一台机认不出来),NAT 大公司被合并
- **上传任何能识别个人或设备的字段**(账号 / MAC / 序列号 / 真实姓名)
- **manifest 挂 CDN**:边缘节点会吃掉绝大多数请求,服务端看不到信号
- **将更新和遥测做成两个独立请求**:用户可以禁用前者(规则失效),保留后者无意义;或反之

## 影响

### 正面影响
- 装机量数据有可量化口径,支持 [ADR-029](./ADR-029-free-first-defer-monetization.md) 的「装机量为唯一指标」决策
- 用户隐私边界清晰、可言简意赅地解释
- 三个 env var 给企业部署、CI、隐私敏感用户留足出口
- 规则正文走 CDN,签名校验在客户端,符合 [ADR-006](./ADR-006-sigstore-reproducible-build.md) Tier 1 要求
- 每次请求带版本/OS/架构,后续 deprecation 与灰度推送都有数据基础

### 负面影响
- manifest 接口不能挂 CDN,流量全部打到自有服务器,需要做基本的反滥用与限流
- ed25519 签名密钥管理是新增运维负担(密钥泄露 = 规则分发被劫持的最大风险点)
- 用户主动删除 `install-id` 文件会被算成新装机,DAU 与「真实独立装机数」存在小幅高估(可接受)
- 跨平台路径解析逻辑需要在落地时统一封装,首发 macOS 时就要把 `cache_dir()` 抽象做对

### 需要更新的文档
- 新建 SPEC-006 落地 manifest 协议 + 客户端 updater 模块详细设计(含 6h 定时器 / install-id 生成 / env var 解析 / 签名校验 / 失败重试策略)
- [docs/api/api-reference.md](../api/api-reference.md) 加 §X manifest 接口章节
- [docs/guides/development.md](../guides/development.md) 加 SIEVE_NO_UPDATE / SIEVE_NO_TELEMETRY / SIEVE_UPDATE_URL 三个环境变量说明
- [docs/guides/deployment.md](../guides/deployment.md) 加企业自托管镜像章节(SIEVE_UPDATE_URL 用法)
- [docs/design/data-model.md](./data-model.md) 加服务端日志表 schema(若服务端代码进本仓)
- README.md 加隐私声明文案
- CHANGELOG `[Unreleased]` 加 manifest 协议 + 三个 env var 条目

## 待决项(GA 前必须落地)

1. **域名**:`updates.sieve.app` / `cdn.sieve.app` 是占位,实际域名待 [ADR-005](./ADR-005-overseas-legal-entity.md) 海外主体确定后注册
2. **ed25519 签名密钥管理**:HSM / 单独 build 机 / 1Password Secrets / GCP KMS,选一种并写入 [ADR-006](./ADR-006-sigstore-reproducible-build.md) follow-up
3. **`ch` 通道策略**:首发是否直接上 beta 通道?还是 stable 一条道?推荐先 stable 单通道,Phase 2 再加 beta
4. **服务端实现**:Cloudflare Workers + KV / D1 vs 自托管 Go 或 Rust?影响后续日志聚合方案。倾向 Cloudflare Workers(零运维,Manifest 接口天然反 DDoS)
5. **客户端 crate 归属**:新增 `sieve-updater` crate 还是放进 `sieve-cli`?倾向新增独立 crate,职责清晰,且 GUI 仓后续可复用同一份 updater 二进制

## 相关文档
- [ADR-029: 装机量优先,延后商业化](./ADR-029-free-first-defer-monetization.md)
- [ADR-005: 海外公司主体作为收款与营销载体](./ADR-005-overseas-legal-entity.md)
- [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](./ADR-006-sigstore-reproducible-build.md)
- [ADR-011: Week 12 GA 前 repo 完全私有](./ADR-011-private-until-ga.md)
