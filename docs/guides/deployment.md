# Sieve 部署与运维指南

> **状态：设计阶段（Pre-Code）。**
> Sieve 尚未发布任何二进制版本。本文反映 Week 12 GA 后的目标交付形态（参见 [PRD §10 12 周里程碑](../prd/sieve-prd-v1.5.md#10-12-周里程碑8-周-dogfood--4-周闭测)）。Phase A dogfood 期间仅 doskey 自用 + Phase B 闭测白名单（5-10 人）使用，**不接受外部安装**。

---

## 1. 当前状态


| 阶段              | 时间          | 安装途径                           |
| --------------- | ----------- | ------------------------------ |
| Phase A dogfood | Week 1-8    | 仅 doskey 自构建运行                 |
| Phase B 闭测      | Week 9-12   | 闭测白名单专属 license + 私有二进制        |
| **GA**          | **Week 12** | brew tap + GitHub Releases（公开） |
| Phase 2+        | Week 13+    | 慢节奏维护，季度大版本                    |


> 在 Phase A / B 期间任何"外部用户"询问安装请回复"等 Week 12 GA 公开发布"，**不要发私有二进制**给非闭测名单。

---

## 2. 安装方式（Phase 1 GA 后）

### 2.1 macOS

Phase 1 GA 交付形态：**三件套 .dmg**（Native GUI App + 后台代理 + sieve-hook）。

**安装步骤**：

1. 从 [GitHub Releases](https://github.com/doskey/sieve/releases) 下载 `Sieve-<version>.dmg`

2. cosign 验证 .dmg 签名（**必做**）：
   ```bash
   cosign verify-blob \
     --certificate-identity-regexp '^https://github.com/doskey/sieve/\.github/workflows/release\.yml@refs/tags/v[0-9.]+$' \
     --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
     --bundle Sieve-<version>.dmg.sigstore \
     Sieve-<version>.dmg
   # 期望输出：Verified OK
   ```

3. 双击挂载 .dmg，将 Sieve.app 拖入 `/Applications`

4. 首次启动 Sieve.app，按引导运行初始化：
   ```bash
   sieve setup
   ```

5. `setup` 自动完成以下操作（详见 [SPEC-003](../specs/SPEC-003-setup-doctor-uninstall.md)）：
   - 改写 Claude Code `settings.json`，注册 `PreToolUse` hook（sieve-hook 二进制）
   - 写入 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` 到 shell 配置
   - 注册 `~/Library/LaunchAgents/tools.sieve.agent.plist`，launchd 接管后台代理

6. `setup` 完成后自动执行 `sieve doctor` 验证所有组件就位

> **Sieve 不提供 `curl ... | sh` 一键安装脚本。**
> 远程脚本盲跑是 [PRD §9](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束) 反对的攻击面，自己不能反着做。

> Homebrew tap（`brew install sieve`）推 Phase 2，当前不可用。

### 2.2 Linux

Phase 1 不支持 Linux。Phase 2 计划：Linux GUI App 与代理同源构建，sieve-hook 行为不变。Linux 用户暂时无法使用 Sieve。

### 2.3 Windows

Phase 1 不支持 Windows。Phase 2 计划。

### 2.4 配置目录

Phase 1 仅 macOS：

| OS | 路径 |
|----|------|
| macOS | `~/.sieve/` |


子目录 / 文件：

```
~/.sieve/
├── config.toml              # 主配置，参见 docs/api/api-reference.md §3
├── audit.db                 # SQLite append-only 审计库
├── .sieveignore             # 本地白名单（不上传）
├── rules/                   # 已签名规则包解压目录
├── keys/
│   └── sieve-rules.pub      # Ed25519 公钥（Phase 1 内置在二进制 + 落盘）
├── bin/
│   └── sieve.prev           # 上一版二进制（用于回滚）
└── logs/
    └── sieve.log            # 文本日志，按天 rotate
```

---

## 3. 二进制签名验证（**必做**）

> Sieve 把"自证清白"作为产品定位的一部分（[PRD §1.2 第 4 句](../prd/sieve-prd-v1.5.md#12-四句话核心叙事v13-加第-4-句)）。**用户不应仅凭信任安装 Sieve，而应能自己验证它。**

### 3.1 sigstore / cosign 验证

**首次安装必跑。**

```bash
# macOS 示例
cosign verify-blob \
  --certificate-identity-regexp '^https://github.com/doskey/sieve/\.github/workflows/release\.yml@refs/tags/v[0-9.]+$' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --bundle Sieve-<version>.dmg.sigstore \
  Sieve-<version>.dmg

# 期望输出
# Verified OK
```

要点：

- `**--certificate-identity-regexp**`：限制签名只能来自 Sieve 仓库的 release workflow，**任何 fork / 私有 build 都通不过**
- `**--certificate-oidc-issuer`**：限制 OIDC 颁发者为 GitHub Actions
- 不要用 `--insecure-ignore-*` 绕过验证

### 3.2 sigstore transparency log 查询（rekor）

每次签名都会写入公开透明日志 [rekor](https://search.sigstore.dev/)：

```bash
# 用 .dmg SHA-256 查询所有签名记录
SHA=$(shasum -a 256 Sieve-<version>.dmg | awk '{print $1}')
rekor-cli search --sha $SHA

# 或在浏览器搜索
open "https://search.sigstore.dev/?hash=$SHA"
```

任何对 Sieve 二进制的"重签名"会在 rekor 留痕，无法静默替换。

### 3.3 Reproducible Build 验证（强烈推荐，**macOS only**）

> Phase 1 仅支持 macOS（`aarch64-apple-darwin` / `x86_64-apple-darwin`）。Linux / Windows 推 Phase 2。

```bash
# 1. clone 当前 release tag
git clone https://github.com/doskey/sieve.git --branch v0.1.0
cd sieve

# 2. 在干净环境内复构建（脚本待写于 GA 前）
./scripts/repro-build.sh macos-arm64
# 或 macos-amd64

# 3. 对比 SHA-256
shasum -a 256 target/repro/sieve-macos-arm64
shasum -a 256 ../Sieve-<version>.dmg

# 期望：两个 SHA-256 完全一致
```

实现细节见 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)。任何 SHA-256 差异 → **不要安装该二进制**，立即在 [GitHub Issues](https://github.com/doskey/sieve/issues) 报告。

---

## 4. 配置 Claude Code 接入

**推荐方案**：`sieve setup` 自动处理，用户无需手动操作。

`setup` 改写 Claude Code `~/.claude/settings.json`，插入：
- `hooks.PreToolUse`：注册 sieve-hook 二进制路径（详见 [SPEC-003](../specs/SPEC-003-setup-doctor-uninstall.md)）
- `env.ANTHROPIC_BASE_URL`：写入 `http://127.0.0.1:11453`

**备用方案（开发者 dogfood）**：手动 export 仍有效：

```bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:11453
export ANTHROPIC_AUTH_TOKEN=<your-real-anthropic-or-router-key>
```

> 注意：手动方案不会注册 `PreToolUse` hook，意味着 Hook 类规则（IN-CR-02/03/04/05）的 fail-closed 拦截**不会触发**。仅推荐 dogfood / 调试用，**不推荐生产**。

> Sieve 是中间层。原来的 Anthropic 官方 key / 中转站 key 仍然要给，Sieve 只做扫描后透传。

API 参见 [API 参考 §1](../api/api-reference.md#1-反向代理端点对-claude-code)。

---

## 5. 服务运行模式

### 5.1 前台调试

```bash
sieve start --config ~/.sieve/config.toml
# Ctrl-C 退出
```

适合：首次安装验证 / 开发调试 / 编写规则。

### 5.2 macOS launchd（生产模式）

plist 内容由 `sieve setup` 自动写入 `~/Library/LaunchAgents/tools.sieve.agent.plist`，用户无需手写。

手动管理（调试 / 重载时可用）：

```bash
# 查看状态
launchctl list | grep sieve

# 重启
launchctl unload ~/Library/LaunchAgents/tools.sieve.agent.plist
launchctl load ~/Library/LaunchAgents/tools.sieve.agent.plist
```

日志：
```bash
tail -f ~/.sieve/logs/launchd.out.log
tail -f ~/.sieve/logs/launchd.err.log
```

### 5.3 sigstore CI（macOS only）

v1.4 起 CI 仅跑 macOS target（`aarch64-apple-darwin` + `x86_64-apple-darwin`）。Linux / Windows target 推 Phase 2 时再加。实际 CI 配置见 `.github/workflows/`，本节仅说明策略。

---

## 6. 端口冲突处理

### 6.1 默认端口

`11453`（[PRD §6.1](../prd/sieve-prd-v1.5.md#61-phase-1-单-agent-架构只-claude-code)）。

### 6.2 端口被占用时

```bash
# 1. 查谁在用
lsof -iTCP:11453 -sTCP:LISTEN
# 或
ss -ltnp | grep 11453

# 2. 改 config.toml
[server]
port = 21453

# 3. 同步改 ANTHROPIC_BASE_URL
export ANTHROPIC_BASE_URL=http://127.0.0.1:21453
```

### 6.3 绑定地址原则（**永远不能改**）

- ✅ `127.0.0.1`（默认）
- ✅ `::1`（Phase 2 IPv6 支持）
- ❌ `0.0.0.0` —— Sieve 启动时 schema 校验会拒绝
- ❌ 任何公网 IP / LAN IP —— 同上

> Sieve 完全本地运行是产品承诺（[PRD §1.1](../prd/sieve-prd-v1.5.md#11-一句话) / [§9 #2](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)），暴露公网会**摧毁产品定位**。

---

## 7. 日志 & 审计

### 7.1 文本日志


| 路径                                                  | 内容                       | 滚动策略              |
| --------------------------------------------------- | ------------------------ | ----------------- |
| `~/.sieve/logs/sieve.log`                           | 进程级日志（启动、配置加载、规则刷新、上游连接） | 按天 rotate，保留 14 天 |
| `~/.sieve/logs/launchd.out.log` / `launchd.err.log` | macOS launchd 标准输出 / 错误  | 由 launchd 接管      |


通过环境变量 / config 调级别：

```bash
SIEVE_LOG_LEVEL=debug sieve --config ~/.sieve/config.toml
# 或写 config.toml: [storage].log_level = "debug"
```

### 7.2 审计 SQLite

`~/.sieve/audit.db`（**append-only**）：

- 仅存 fingerprint + 元信息，**绝不存原始 prompt 内容**（[PRD §11.3](../prd/sieve-prd-v1.5.md#113-开源策略) / API 参考 §2.2.3）
- schema 详见 [data-model.md](../design/data-model.md)

### 7.3 查询 CLI

```bash
sieve events --since 1h                    # 最近 1 小时
sieve events --since 2026-04-25T00:00 --severity critical
sieve events --rule OUT-09 --limit 50
```

底层等价于 `GET /_sieve/v1/events`（参见 [API 参考 §2.2.3](../api/api-reference.md#223-审计事件查询)）。

---

## 8. 升级 / 回滚

### 8.1 升级

```bash
# macOS：下载新版 .dmg → 重跑 §3.1 cosign 验证 → 替换前先停服务
launchctl unload ~/Library/LaunchAgents/tools.sieve.agent.plist
# 双击新版 .dmg，拖入 Applications 覆盖
launchctl load ~/Library/LaunchAgents/tools.sieve.agent.plist

# Homebrew（Phase 2 可用时）
# brew update && brew upgrade sieve
```

### 8.2 回滚

Sieve 升级时自动把上一版二进制保留到 `~/.sieve/bin/sieve.prev`。配置 `[server].binary_fallback = true` 后可一键 fallback：

```bash
sieve self-rollback              # CLI 子命令，等价于：
# 1. 停服务
# 2. cp ~/.sieve/bin/sieve.prev /usr/local/bin/sieve
# 3. 启服务
```

> **回滚前必停服务**，否则 SQLite WAL 可能不一致。

### 8.3 升级前 checklist

- 已停服务（参见 §8.1）
- 已验证新版本 cosign 签名
- 已读 [CHANGELOG](../changelog/CHANGELOG.md) 确认无破坏性变更
- 当前 `~/.sieve/.sieveignore` 已备份（防误删）

---

## 9. 卸载

推荐使用 `sieve uninstall`，按 `setup.log` 逐步回滚（详见 [SPEC-003](../specs/SPEC-003-setup-doctor-uninstall.md)）：

```bash
# Step 1：dry-run 预览将要撤销的操作
sieve uninstall --dry-run

# Step 2：确认后执行
sieve uninstall

# sieve uninstall 自动完成：
# - 从 Claude Code settings.json 移除 PreToolUse hook 注册
# - 移除 ANTHROPIC_BASE_URL 环境变量注入
# - unload + 删除 launchd plist
# - 删除 /usr/local/bin/sieve（或安装路径）
# - 不自动删除 ~/.sieve/（含审计日志），提示用户手动处理
```

审计日志（`~/.sieve/audit.db`）是用户本地资产，`sieve uninstall` **不自动删除**，需手动处理：

```bash
# 备份
mv ~/.sieve ~/.sieve.bak.$(date +%Y%m%d)
# 或彻底删除（不可恢复）
# rm -rf ~/.sieve
```

> 删除前确认无后续合规 / 复盘需要。

---

## 10. 离线运行

适合空气墙环境 / 出差不联网 / 极度多疑场景。

```bash
export SIEVE_DISABLE_RULES_UPDATE=1
sieve --config ~/.sieve/config.toml
```

或在 config 中：

```toml
[rules_update]
enabled = false
```

特性：

- 仅依赖**打包时内置**规则（最近一次签名规则包，编译进二进制）
- 启动时不发任何网络请求（除转发用户实际 API 调用）
- 启动横幅提示当前内置规则版本 + 发布日期，供用户判断是否过期

---

## 11. License 激活、试用与降级

### 11.1 14 天试用

- 首次启动注册邮箱（在 CLI 弹窗输入），后端发 license key 邮件
- 试用期内全功能开放
- 无网络环境下按 `[license].offline_grace_days`（默认 30 天）使用缓存的最近一次成功验证
- 离线超过 `offline_grace_days` 时，license 视为过期 → 自动进入 §11.3 降级模式（**Critical 仍阻断**，High/Medium/Low 仅记录）

### 11.2 正式版定价

- **$49 / 月**（[PRD §7.1](../prd/sieve-prd-v1.5.md#71-单一定价)）
- **年付 $490**（省 2 个月）
- 支付通道：Stripe + 加密支付（USDC / USDT）双通道（[PRD §11.5.1](../prd/sieve-prd-v1.5.md#1151-公司主体与收款)）

### 11.3 降级模式（试用结束未付费）

[PRD §7.1](../prd/sieve-prd-v1.5.md#71-单一定价) 描述的"只读警告"模式：

- ✅ **Critical 仍然阻断**（产品安全承诺，[PRD §9 #8](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)）
- ⚠ High / Medium / Low 仅记录，不弹窗、不警告
- 不停止运行 —— 不让用户因为没付费而失去基本保护

### 11.4 License 验证流程

- **本地 Ed25519 公钥** 验证 license key 签名 → **不联网 verify**（[PRD §9 #2](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)）
- 公钥内置在二进制 + 落盘 `~/.sieve/keys/`
- License 包含：邮箱、签发时间、过期时间、计划等级（trial / paid_monthly / paid_yearly）

---

## 12. 故障排查（FAQ）

### 12.1 Claude Code 连不上 Sieve

```bash
# 1. 确认 Sieve 在跑
curl http://127.0.0.1:11453/_sieve/v1/healthz
# 期望：{"status":"ok","uptime_s":...}

# 2. 确认环境变量
echo $ANTHROPIC_BASE_URL    # 应为 http://127.0.0.1:11453
echo $ANTHROPIC_AUTH_TOKEN  # 应非空（透传给上游用）

# 3. 确认端口未被防火墙拦截
# macOS：默认不会拦截本地回环
# Linux：`sudo iptables -L INPUT | grep 11453`

# 4. 确认进程没在 0.0.0.0 上跑（应只 127.0.0.1）
ss -ltnp | grep sieve
```

### 12.2 SSE 流卡住

```bash
# 1. 上游可达性
curl -I https://api.anthropic.com/v1/models    # 或你配置的 upstream.url

# 2. 看 Sieve 日志
tail -f ~/.sieve/logs/sieve.log

# 3. 临时升日志级别
SIEVE_LOG_LEVEL=debug sieve --config ~/.sieve/config.toml

# 4. 确认 upstream.timeout_ms 是否设得过短
```

### 12.3 误报触发太多

1. **不要直接用 `severity_overrides` 关 Critical** —— PRD §9 #8 禁止
2. 优先加 `.sieveignore`：
  ```bash
   sieve sieveignore add <fingerprint> --comment "false positive: <场景>"
  ```
3. 对 Medium / High 规则可在 `[detection.severity_overrides]` 降级
4. 报 issue（带 fingerprint + 上下文，**不要贴原始 prompt**），doskey 会评估规则改进

### 12.4 license 验证失败

```bash
# 1. 确认 key 完整性（无换行、无空格）
echo -n "$SIEVE_LICENSE_KEY" | wc -c

# 2. 检查系统时间（license 有过期时间，时间偏差大会过期）
date -u
# 与 https://time.is/UTC 对比

# 3. 重新拷贝 key（来源邮件可能有换行符）

# 4. 离线宽限期是否耗尽
sieve license info
```

### 12.5 如何完全离线

参见 [§10 离线运行](#10-离线运行)。

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[development.md](development.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)
- ADR-006 sigstore + reproducible build：[../design/ADR-006-sigstore-reproducible-build.md](../design/ADR-006-sigstore-reproducible-build.md)
- 数据模型：[../design/data-model.md](../design/data-model.md)
- 术语表：[../glossary.md](../glossary.md)

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。部署 / 运维流程任何变更必须同步更新本文与 [CHANGELOG](../changelog/CHANGELOG.md)。

