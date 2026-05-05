# Sieve 部署与运维指南

> Version: v2.0 — 2026-05-01
>
> **状态：v2.0 + v2.1 代码落地，Phase A dogfood 进行中。**
> 用户规则（`~/.sieve/rules/user.toml`）+ 灰名单（`~/.sieve/decisions/`）+ audit v2 schema（含 `caller_pid` / `caller_exe`）已全部落地。GUI 多客户端支持（v2.1 broadcast fan-out）已就绪。对外仍不接受外部安装，Week 12 GA 后公开。

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

5. `setup` 自动完成以下操作（详见 [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）：
   - 改写 Claude Code `settings.json`，注册 `PreToolUse` hook（sieve-hook 二进制）
   - 写入 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` 到 shell 配置
   - 注册 `~/Library/LaunchAgents/tools.sieve.agent.plist`，launchd 接管后台代理

6. `setup` 完成后自动执行 `sieve doctor` 验证所有组件就位

> **Sieve 不提供 `curl ... | sh` 一键安装脚本。**
> 远程脚本盲跑是 [PRD §9](../prd/sieve-prd-v2.0.md#9-工程上必须做对的硬约束) 反对的攻击面，自己不能反着做。

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
~/.sieve/                                              # 0700
├── config.toml                                        # 主配置，参见 docs/api/api-reference.md §3
├── audit.db                                           # SQLite append-only 审计库（v2 schema）
├── .sieveignore                                       # 本地白名单（不上传）
├── rules/                                             # 规则目录
│   ├── <已签名系统规则包解压文件>                        # 内置规则
│   ├── user.toml                                      # 用户自定义规则（0600，v2.0 §5.5）
│   └── user.toml.bak.YYYYMMDD-HHMMSS                 # 编辑备份，保留最近 10 份（0600）
├── decisions/                                         # 灰名单条目目录（v2.0 §5.4.2）
│   └── <sha256_64_hex>.json                           # 灰名单 entry（0600，atomic rename 写入）
├── ipc.sock                                           # Unix domain socket（IPC，v1.4）
├── keys/
│   └── sieve-rules.pub                                # Ed25519 公钥（Phase 1 内置在二进制 + 落盘）
├── bin/
│   └── sieve.prev                                     # 上一版二进制（用于回滚）
└── logs/
    └── sieve.log                                      # 文本日志，按天 rotate
```

**文件权限**：

| 路径 | 权限 | 说明 |
|------|------|------|
| `~/.sieve/` | `0700` | 目录对外不可读 |
| `~/.sieve/rules/user.toml` | `0600` | daemon 启动时校验，权限不对拒绝加载 |
| `~/.sieve/decisions/*.json` | `0600` | atomic rename 写入，软链接禁止（no-follow symlink，PRD §5.5.3-C） |

> `upstream-routes.json`（OpenClaw 多 provider 路由，v1.5）如存在亦保存在 `~/.sieve/` 下，权限 `0600`。

---

## 3. 二进制签名验证（**必做**）

> Sieve 把"自证清白"作为产品定位的一部分（[PRD §1.2 第 4 句](../prd/sieve-prd-v2.0.md#12-四句话核心叙事v13-加第-4-句)）。**用户不应仅凭信任安装 Sieve，而应能自己验证它。**

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
- `hooks.PreToolUse`：注册 sieve-hook 二进制路径（详见 [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）
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

`11453`（[PRD §6.1](../prd/sieve-prd-v2.0.md#61-phase-1-单-agent-架构只-claude-code)）。

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

> Sieve 完全本地运行是产品承诺（[PRD §1.1](../prd/sieve-prd-v2.0.md#11-一句话) / [§9 #2](../prd/sieve-prd-v2.0.md#9-工程上必须做对的硬约束)），暴露公网会**摧毁产品定位**。

---

## 6a. Multi-listener 部署（ADR-026）

v2.x 起，sieve daemon 支持在单次启动中同时监听多个端口，每个端口绑定独立的上游（`[[upstream]]` 数组）。这使哑 client（Claude Code / Codex CLI / Cursor 等只认 `ANTHROPIC_BASE_URL` 一个 env var）可以通过切换端口来切换上游 provider，无需注入任何 header。

### 6a.1 配置示例

`~/.sieve/sieve.toml`：

```toml
bind_addr = "127.0.0.1"
tls_verify_upstream = true

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
provider_id = "anthropic"
protocol = "anthropic"

[[upstream]]
port = 11454
url = "https://api.deepseek.com/anthropic"
provider_id = "deepseek"
protocol = "anthropic"

[[upstream]]
port = 11455
url = "https://api.openai.com"
provider_id = "openai"
protocol = "openai"
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `port` | `u16` | 监听端口，必须 `127.0.0.1` 绑定，不可指向公网（[ADR-003](../design/ADR-003-local-only-binding.md)） |
| `url` | `String` | 真实上游地址，含 path prefix（如 `https://api.deepseek.com/anthropic`）|
| `provider_id` | `String` | 用于审计日志、IPC 事件标注 |
| `protocol` | `anthropic` \| `openai` | 显式声明协议，不再靠 path 猜；错位请求 fail-closed（400）|

**向后兼容**：旧 `upstream_url` + `port` 单字段格式仍可读，deserialize 时自动映射为单元素 `[[upstream]]`，行为不变。

### 6a.2 端口规划建议

- 默认起 `11453`，递增分配（`11454` / `11455` / ...）
- 避开常见冲突端口（`80` / `443` / `8080` / `5000` / `3000`）
- 同一 daemon 实例内端口必须唯一（启动时端口冲突 → 立即 `exit 1`，不进入 partial-start）
- 仅本地回环（`127.0.0.1`）—— `bind_addr` 任何非 `127.0.0.1` 值会触发 FATAL exit（[ADR-003](../design/ADR-003-local-only-binding.md) / [PRD §9 #2](../prd/sieve-prd-v2.0.md)）

### 6a.3 launchd plist 不需改动

daemon 内部完成多 `bind`，launchd plist 只需启动 `sieve start --config ~/.sieve/sieve.toml` 一次（无需多 socket activation）。`sieve setup` 写入的 plist 模板不变，无需手动调整。

### 6a.4 故障排查

| 症状 | 原因 | 处置 |
|------|------|------|
| daemon 启动 FATAL `duplicate listener port X` | `[[upstream]]` 数组里两项 `port` 相同 | 修 `sieve.toml`，每项 `port` 唯一 |
| daemon 启动 `bind listener port 11454: Address already in use` | 该端口被其他进程占用 | `lsof -i :11454` 排查；改用其他端口 |
| 任一 listener bind 失败 → daemon 整体退出 | fail-fast 行为（[ADR-026 §决策 3](../design/ADR-026-port-based-listener-routing.md)）| by design——半启动状态会让 `sieve doctor` 输出混淆 |
| `sieve doctor` 报 `ADR-026 multi-listener 全部端口可达 ❌` | 某个 listener 当前不可连 | 检查 daemon 日志（`~/.sieve/logs/sieve.log`）看哪个 listener 失败；可能是端口冲突恢复中 |
| Claude Code 报 `ETIMEDOUT` 但 daemon 日志无新条目 | 客户端 `ANTHROPIC_BASE_URL` 端口写错（指向了 daemon 没绑的端口）| 核对 env var 端口；用 `lsof -i -P \| grep sieve` 确认 daemon 实际绑定的端口列表 |

### 6a.5 Pro Mode（v3.x，ADR-027 网络层硬隔离）

[ADR-027](../design/ADR-027-network-jail-enforcement.md) network jail enforcement 计划在 v3.x post-GA opt-in 上线，届时多 listener 会与 macOS pf / Linux nftables uid-based egress filter 配合，实现"非 sieve 进程无法直连 LLM endpoint"。本节只覆盖 v2.x multi-listener，jail 部署见未来版本。

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

- 仅存 fingerprint + 元信息，**绝不存原始 prompt 内容**（[PRD §11.3](../prd/sieve-prd-v2.0.md#113-开源策略) / API 参考 §2.2.3）
- schema 详见 [data-model.md](../design/data-model.md)

### 7.3 查询 CLI

```bash
sieve events --since 1h                    # 最近 1 小时
sieve events --since 2026-04-25T00:00 --severity critical
sieve events --rule OUT-09 --limit 50
```

底层等价于 `GET /_sieve/v1/events`（参见 [API 参考 §2.2.3](../api/api-reference.md#223-审计事件查询)）。

---

## 7a. 用户规则部署（v2.0 §5.5）

### 7a.1 首次初始化

用户无需手动创建目录。首次运行 `sieve rules edit` 时 daemon 自动：

1. 创建 `~/.sieve/rules/`（权限 `0700`）
2. 生成默认模板 `~/.sieve/rules/user.toml`（权限 `0600`）
3. 调 `$EDITOR`（fallback 顺序：`$EDITOR` → `vim` → `nano`）打开模板

### 7a.2 编辑流程

```bash
sieve rules edit
```

保存退出后，自动执行：

1. **lint pipeline**：解析 TOML + 验证规则 ID 格式 + severity 合法性检查
2. lint 通过 → **atomic backup**：将原文件复制为 `user.toml.bak.YYYYMMDD-HHMMSS`（保留最近 10 份，超出自动清理最旧）
3. **atomic rename**：写临时文件后 `rename()` 原子替换，避免部分写入
4. **IPC notify reload**：通知运行中的 daemon 执行 hot swap（zero-downtime，不中断现有连接）

### 7a.3 失败回滚

- lint 失败：原文件**不变**，stderr 打印全部违规清单，用户修改后重跑 `sieve rules edit`
- 不触发备份，不发送 IPC notify

### 7a.4 规则管理 CLI

```bash
sieve rules list                  # 合并展示用户规则数 + 系统规则数
sieve rules disable <rule-id>     # 写 disabled = true，atomic rename，IPC notify
sieve rules enable <rule-id>      # 反向
```

---

## 7b. 灰名单部署（v2.0 §5.4.2）

### 7b.1 条目写入流程

GUI 弹窗用户选 "记住此次决策" → `sieve-gui-macos` 通过 IPC 通知 daemon → daemon 调 `graylist::add_entry` → 写 `~/.sieve/decisions/<digest>.json`（权限 `0600`，atomic rename 写入）。

`digest` 为 SHA-256 64 位 hex，基于 `{rule_id}:{normalized_content}` 计算。

### 7b.2 Critical 锁三道防线

即使用户选择"记住"，Critical 级别规则命中时有三道防线阻止进入灰名单：

1. **daemon 侧**：`graylist::add_entry` 对 Critical severity 规则直接返回 `CriticalRejected` 错误，不写文件
2. **IPC 协议层**：`graylist_critical_rejected` event 写入 audit.db，记录被拒绝的尝试
3. **GUI 侧**：收到 `graylist_critical_rejected` 响应后，不再显示"记住"选项

这三道防线确保 PRD §9 #8 约束（Critical 在所有版本不可关闭）。

### 7b.3 手动清空灰名单

```bash
# 清空所有灰名单条目（重启 daemon 后缓存失效）
rm -rf ~/.sieve/decisions/
```

> 清空后 daemon 无需重启即可感知（下次查询时目录不存在视为空）。

---

## 7c. Audit Schema Migration（v2.0 §5.6.1）

### 7c.1 自动迁移

v1 → v2 schema migration 在 daemon 首次以 v2.0+ 二进制启动时**自动触发**，无需用户操作：

```
-- v2 新增列
ALTER TABLE events ADD COLUMN caller_pid INTEGER;   -- NULL = 来源未知
ALTER TABLE events ADD COLUMN caller_exe TEXT;       -- NULL = 来源未知
```

migration 完成后，append-only 触发器（禁止 UPDATE / DELETE）在新 schema 下**仍然生效**。

### 7c.2 v2 新增 AuditEvent 变体

v2.0 新增以下 7 个 `kind` 值，运维时可按此过滤：

| kind | 含义 |
|------|------|
| `decision_made` | 用户在弹窗做出审批决策（approved / rejected） |
| `graylist_added` | 灰名单条目写入成功 |
| `graylist_critical_rejected` | Critical 规则拒绝写入灰名单 |
| `graylist_add_failed` | 灰名单写入失败（IO error 等） |
| `graylist_hit` | 请求命中已有灰名单条目（自动放行） |
| `sequence_hit` | IN-SEQ-* 行为序列规则命中 |
| `user_rules_reloaded` | 用户规则 hot reload 成功 |
| `user_rules_load_failed` | 用户规则加载失败（lint 通过但 IO 错误） |

### 7c.3 查询样例

```sql
SELECT kind, rule_id, severity, caller_pid, caller_exe, raw_json
FROM events
WHERE kind IN ('decision_made', 'graylist_added', 'graylist_critical_rejected',
               'graylist_add_failed', 'graylist_hit', 'sequence_hit',
               'user_rules_reloaded', 'user_rules_load_failed')
ORDER BY at DESC LIMIT 50;
```

等价 CLI：

```bash
sieve events --since 24h | grep -E 'graylist|sequence_hit|user_rules'
```

---

## 7d. GUI 多客户端（v2.1）

v2.1 支持 `sieve-gui-macos` 多实例并发连接 IPC（例如多个状态栏实例或调试用 mock_gui）：

- daemon 维护 broadcast channel，所有已连接 GUI 客户端均收到通知
- 断开连接的 sender 在下次广播时 lazy 清理（`dead sender` 自动移除，不影响其他客户端）
- 开发调试时可同时跑真实 GUI + `cargo run -p sieve-ipc --example mock_gui` 验证通知路径

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

推荐使用 `sieve uninstall`，按 `setup.log` 逐步回滚（详见 [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）：

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
export SIEVE_NO_UPDATE=1
sieve --config ~/.sieve/config.toml
```

或在 config 中（ADR-030 §7）：

```toml
[update]
enabled = false
```

> **说明**：旧字段名 `SIEVE_DISABLE_RULES_UPDATE` / `[rules_update]` 已被 ADR-030 替换为统一的 `SIEVE_NO_UPDATE` / `[update]` 段。详见 §13 自托管镜像 + manifest 接口 + [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

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

- **$49 / 月**（[PRD §7.1](../prd/sieve-prd-v2.0.md#71-单一定价)）
- **年付 $490**（省 2 个月）
- 支付通道：Stripe + 加密支付（USDC / USDT）双通道（[PRD §11.5.1](../prd/sieve-prd-v2.0.md#1151-公司主体与收款)）

### 11.3 降级模式（试用结束未付费）

[PRD §7.1](../prd/sieve-prd-v2.0.md#71-单一定价) 描述的"只读警告"模式：

- ✅ **Critical 仍然阻断**（产品安全承诺，[PRD §9 #8](../prd/sieve-prd-v2.0.md#9-工程上必须做对的硬约束)）
- ⚠ High / Medium / Low 仅记录，不弹窗、不警告
- 不停止运行 —— 不让用户因为没付费而失去基本保护

### 11.4 License 验证流程

- **本地 Ed25519 公钥** 验证 license key 签名 → **不联网 verify**（[PRD §9 #2](../prd/sieve-prd-v2.0.md#9-工程上必须做对的硬约束)）
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

### 12.6 sieve doctor 新检查项（v2.0）

`sieve doctor` 在原有检查基础上新增以下项：

```bash
sieve doctor
```

v2.0 新增检查：

| 检查项 | 期望结果 | 失败原因 |
|--------|---------|---------|
| `user.toml 存在` | `~/.sieve/rules/user.toml` 存在 | 首次未运行 `sieve rules edit` |
| `user.toml 权限 0600` | `stat ~/.sieve/rules/user.toml` 显示 `-rw-------` | 权限被意外修改 |
| `user.toml lint 通过` | 规则文件可解析且字段合法 | 手动编辑引入语法错误 |
| `decisions/ 权限 0700` | 目录权限正确 | 权限被意外修改 |
| `audit.db schema v2` | events 表含 caller_pid / caller_exe 列 | migration 未执行（旧 daemon 遗留） |

任何检查失败时，`sieve doctor` 输出修复建议：

```bash
# 示例：user.toml 权限修复
chmod 0600 ~/.sieve/rules/user.toml

# 示例：手动触发 audit migration（重启 daemon 自动执行）
launchctl unload ~/Library/LaunchAgents/tools.sieve.agent.plist
launchctl load ~/Library/LaunchAgents/tools.sieve.agent.plist
```

---

## 13. 企业自托管镜像 / 私有更新源（ADR-030）

> 适合企业内网部署、离线环境（air-gapped）、隐私敏感机构使用。完整协议见 [SPEC-006 §3](../specs/SPEC-006-update-and-telemetry.md)。

### 13.1 `SIEVE_UPDATE_URL` 用法

将 manifest 请求指向企业内网服务器：

```bash
# 运行时设置
export SIEVE_UPDATE_URL=https://updates.internal.corp/sieve/v1/manifest
sieve start --config ~/.sieve/sieve.toml

# 或写入 sieve.toml（env var 优先级更高）
```

```toml
[update]
url = "https://updates.internal.corp/sieve/v1/manifest"
```

### 13.2 自建服务端要求

自托管 manifest 服务端必须满足：

- 响应 JSON 格式符合 [SPEC-006 §3.2 schema](../specs/SPEC-006-update-and-telemetry.md)（`schema` / `rules` / `client` / `next_check_after_seconds` 字段）
- 支持 TLS 1.2+（客户端强制 TLS，HTTP 连接会被拒绝）
- 不需要 `Authorization` header（Sieve 客户端不发认证信息）
- 规则包（`rules.url`）可与 manifest 服务在同一服务器，也可放企业 CDN

**最简 mock 响应示例**（用于本地开发测试）：

```json
{
  "schema": 1,
  "rules": {
    "version": "2026.01.01.0",
    "url": "https://cdn.internal.corp/sieve/rules/2026.01.01.0.json.zst",
    "sha256": "aaabbbccc...",
    "size": 184320,
    "signature": "ed25519:..."
  },
  "client": {
    "latest": "0.3.1",
    "min_supported": "0.2.0",
    "deprecation_notice": null
  },
  "next_check_after_seconds": 21600
}
```

### 13.3 离线环境（air-gapped）

完全禁用更新检查，规则永久冻结在安装时的版本：

```bash
# 方式一：环境变量（推荐，即时生效）
export SIEVE_NO_UPDATE=1
sieve start --config ~/.sieve/sieve.toml
# 启动日志会打印：update check disabled by SIEVE_NO_UPDATE

# 方式二：写入 sieve.toml
```

```toml
[update]
enabled = false     # 等价 SIEVE_NO_UPDATE
```

> 注意：离线模式下规则不更新，规则包会逐渐过期。建议定期通过安全渠道分发最新规则包到 `~/.sieve/rules/`，并验证 ed25519 签名后原子替换。

### 13.4 隐私敏感场景（只想要规则更新，不参与装机统计）

```bash
export SIEVE_NO_TELEMETRY=1
sieve start --config ~/.sieve/sieve.toml
```

或写入 `sieve.toml`：

```toml
[update]
telemetry = false   # 省略 uid 字段，仍发更新请求
```

**效果**：

- 仍能拉取最新规则（规则更新不受影响）
- manifest 请求 URL 中省略 `uid=` 参数
- 服务端无法统计该安装 ID 的 DAU

### 13.5 优先级汇总

```
SIEVE_NO_UPDATE=1       → 完全不发请求（最高权限）
SIEVE_NO_TELEMETRY=1    → 发请求但省略 uid
SIEVE_UPDATE_URL=<url>  → 覆盖默认 manifest URL
↑ 以上 env var 均优先于 sieve.toml [update] 段
```

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v2.0.md](../prd/sieve-prd-v2.0.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[development.md](development.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)
- ADR-006 sigstore + reproducible build：[../design/ADR-006-sigstore-reproducible-build.md](../design/ADR-006-sigstore-reproducible-build.md)
- ADR-030 更新通道遥测：[../design/ADR-030-update-telemetry-channel.md](../design/ADR-030-update-telemetry-channel.md)
- SPEC-006 manifest 协议规格：[../specs/SPEC-006-update-and-telemetry.md](../specs/SPEC-006-update-and-telemetry.md)
- 数据模型：[../design/data-model.md](../design/data-model.md)
- 术语表：[../glossary.md](../glossary.md)

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。部署 / 运维流程任何变更必须同步更新本文与 [CHANGELOG](../changelog/CHANGELOG.md)。

