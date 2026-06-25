# Sieve 部署与运维指南

> Version: v2.0 — 2026-05-01
>
> **状态：早期预览（0.1.0-alpha）。**
> 用户规则（`~/.sieve/rules/user.toml`）+ 灰名单（`~/.sieve/decisions/`）+ audit schema（含 `caller_pid` / `caller_exe` / `provider_id`）+ GUI 多客户端 broadcast fan-out 均已就绪。

---

## 1. 安装前提

- 仅支持 macOS（Phase 1）；Linux / Windows 见下文 §2.2 / §2.3。
- 安装方式见 §2.1（一行命令优先：Homebrew / 自校验安装器 / cargo）。签名验证由安装器与 Homebrew **自动完成**；想亲手再验的看 §3（可选）。
- 通过 brew tap + GitHub Releases 公开分发。

---

## 2. 安装方式（macOS）

### 2.1 macOS

> **自校验是卖点，不是门槛。** 大多数 `curl … | sh` 要你盲信一段脚本；Sieve 的安装器在把任何东西落地前，先用 cosign / sigstore（keyless 签名 + Rekor 透明日志）校验自己的 release 产物，被篡改或来源不符就 **fail-closed 拒装**。一行命令，照样可验。

按从无摩擦到硬核的顺序，四条安装路径：

**① Homebrew（macOS 首选）**——brew 原生自动校验 sha256。

```bash
brew tap SieveAI-dev/sieve && brew install sieve   # CLI / daemon
brew install --cask sieve                          # GUI .app
```

**② 自校验一行安装器**——只装 `sieve` CLI / daemon 二进制（GUI 不走此路）。下载裸二进制 + 同名 `.sigstore.json` bundle，落地前自动校验（有 cosign 用 cosign 验签，无则回退对照 `SHA256SUMS` 校验 sha256 并明确警告），任一不符即 fail-closed 拒装。

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/SieveAI-dev/sieve/main/scripts/install.sh | bash
```

> 品牌短链 `sieveai.dev/install.sh` 即将提供，将代理此脚本。

**③ cargo install**——从源码构建。

```bash
cargo install --git https://github.com/SieveAI-dev/sieve sieve-cli   # 现可用
cargo install sieve                                                  # crates.io，Phase 2 起
```

**④ 手动（给偏执狂）**——从 [GitHub Releases](https://github.com/SieveAI-dev/sieve/releases) 下签名 `.dmg`（GUI 三件套：Native GUI App + 后台代理 + sieve-hook）或裸二进制，手动用 cosign 验签（完整命令见 §3.1）。

**GUI 安装后续步骤**（路径 ① cask 或路径 ④ .dmg）：

1. 双击挂载 .dmg，将 SieveGUI.app 拖入 `/Applications`（cask 已自动放置）
2. 首次启动 SieveGUI.app，按引导运行初始化：
   ```bash
   sieve setup
   ```
3. `setup` 自动完成以下操作（详见 [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md)）：
   - 改写 Claude Code `settings.json`，注册 `PreToolUse` hook（sieve-hook 二进制）
   - 写入 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` 到 shell 配置
   - 注册 `~/Library/LaunchAgents/tools.sieve.agent.plist`，launchd 接管后台代理
4. `setup` 完成后自动执行 `sieve doctor` 验证所有组件就位

> 验证已由安装器 / Homebrew 自动完成。想手动验的看 §3（可选），或用 `sieve doctor` 查看验证状态。

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
| `~/.sieve/decisions/*.json` | `0600` | atomic rename 写入，软链接禁止（no-follow symlink） |

> `upstream-routes.json`（OpenClaw 多 provider 路由，v1.5）如存在亦保存在 `~/.sieve/` 下，权限 `0600`。

---

## 3. 二进制签名验证（**可选 / 给偏执狂**）

> Sieve 把"可独立验证"作为产品定位的一部分。**用户不应仅凭信任安装 Sieve，而应能自己验证它。**
>
> 常规路径下验证**已由安装器 / Homebrew 自动完成**（见 §2.1，被篡改即 fail-closed 拒装），无需手动操作；`sieve doctor` 可查看验证状态。本节命令保留给想亲手再验一遍、或走手动 .dmg 路径（§2.1 路径 ④）的用户。

### 3.1 sigstore / cosign 验证（可选）

**走手动 .dmg 路径时必跑；其余路径已自动完成，可选复验。**

```bash
# macOS 示例
cosign verify-blob \
  --certificate-identity-regexp '^https://github.com/SieveAI-dev/sieve/\.github/workflows/release\.yml@refs/tags/v[0-9.]+$' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --bundle SieveGUI-<version>.dmg.sigstore \
  SieveGUI-<version>.dmg

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
SHA=$(shasum -a 256 SieveGUI-<version>.dmg | awk '{print $1}')
rekor-cli search --sha $SHA

# 或在浏览器搜索
open "https://search.sigstore.dev/?hash=$SHA"
```

任何对 Sieve 二进制的"重签名"会在 rekor 留痕，无法静默替换。

### 3.3 Reproducible Build 验证（强烈推荐，**macOS only**）

> Phase 1 仅支持 macOS（`aarch64-apple-darwin` / `x86_64-apple-darwin`）。Linux / Windows 推 Phase 2。

```bash
# 1. clone 当前 release tag
git clone https://github.com/SieveAI-dev/sieve.git --branch v0.1.0-alpha
cd sieve

# 2. 在干净环境内复构建（脚本待写于 GA 前）
./scripts/repro-build.sh macos-arm64
# 或 macos-amd64

# 3. 对比 SHA-256
shasum -a 256 target/repro/sieve-macos-arm64
shasum -a 256 ../SieveGUI-<version>.dmg

# 期望：两个 SHA-256 完全一致
```

> **GA build 密钥 gate**：GA release 二进制必须用 `cargo build --release --features ga_keys` 构建。该 feature 在编译期断言规则签名公钥（`sieve-updater::TRUSTED_PUBKEY`）与 X-Sieve-Origin 公钥（`sieve-ipc::SIEVE_ORIGIN_PUBLIC_KEY`）已填真实值——若仍是占位（`None` / 全零）则 **编译失败（E0080）**，杜绝 fail-open 验签进入 GA 二进制。alpha / dogfood build（默认）不启用，行为不变。

任何 SHA-256 差异 → **不要安装该二进制**，立即在 [GitHub Issues](https://github.com/SieveAI-dev/sieve/issues) 报告。

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

`11453`（默认本地代理端口）。

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

> Sieve 完全本地运行是产品承诺，暴露公网会**摧毁产品定位**。

---

## 6a. Multi-listener 部署

v2.x 起，sieve daemon 支持在单次启动中同时监听多个端口，每个端口绑定独立的上游（`[[upstream]]` 数组）。这使哑 client（Claude Code / OpenClaw / Hermes / Codex CLI 等只认一个 base_url env var）可以通过切换端口来切换上游 provider，无需注入任何 header。

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
| `port` | `u16` | 监听端口，必须 `127.0.0.1` 绑定，不可指向公网 |
| `url` | `String` | 真实上游地址，含 path prefix（如 `https://api.deepseek.com/anthropic`）|
| `provider_id` | `String` | 用于审计日志、IPC 事件标注 |
| `protocol` | `anthropic` \| `openai` | 显式声明协议，不再靠 path 猜；错位请求 fail-closed（400）|

**向后兼容**：旧 `upstream_url` + `port` 单字段格式仍可读，deserialize 时自动映射为单元素 `[[upstream]]`，行为不变。

### 6a.2 端口规划建议

- 默认起 `11453`，递增分配（`11454` / `11455` / ...）
- 避开常见冲突端口（`80` / `443` / `8080` / `5000` / `3000`）
- 同一 daemon 实例内端口必须唯一（启动时端口冲突 → 立即 `exit 1`，不进入 partial-start）
- 仅本地回环（`127.0.0.1`）—— `bind_addr` 任何非 `127.0.0.1` 值会触发 FATAL exit（完全本地运行硬约束）

### 6a.3 launchd plist 不需改动

daemon 内部完成多 `bind`，launchd plist 只需启动 `sieve start --config ~/.sieve/sieve.toml` 一次（无需多 socket activation）。`sieve setup` 写入的 plist 模板不变，无需手动调整。

### 6a.4 故障排查

| 症状 | 原因 | 处置 |
|------|------|------|
| daemon 启动 FATAL `duplicate listener port X` | `[[upstream]]` 数组里两项 `port` 相同 | 修 `sieve.toml`，每项 `port` 唯一 |
| daemon 启动 `bind listener port 11454: Address already in use` | 该端口被其他进程占用 | `lsof -i :11454` 排查；改用其他端口 |
| 任一 listener bind 失败 → daemon 整体退出 | fail-fast 行为 | by design——半启动状态会让 `sieve doctor` 输出混淆 |
| `sieve doctor` 报 `multi-listener 全部端口可达 ❌` | 某个 listener 当前不可连 | 检查 daemon 日志（`~/.sieve/logs/sieve.log`）看哪个 listener 失败；可能是端口冲突恢复中 |
| Claude Code 报 `ETIMEDOUT` 但 daemon 日志无新条目 | 客户端 `ANTHROPIC_BASE_URL` 端口写错（指向了 daemon 没绑的端口）| 核对 env var 端口；用 `lsof -i -P \| grep sieve` 确认 daemon 实际绑定的端口列表 |

### 6a.5 网络层硬隔离（计划中）

网络隔离（network jail enforcement）为计划中的 opt-in 能力，届时多 listener 会与 macOS pf / Linux nftables uid-based egress filter 配合，实现"非 sieve 进程无法直连 LLM endpoint"。本节只覆盖当前的 multi-listener，隔离部署见后续版本。

---

## 6b. 受限网络（Shadowrocket / Clash 等）部署

在「不挂代理连不上 LLM」的网络（规则代理 + 分流、非全局 TUN 透明代理——大量 crypto 开发者所处环境）下，sieve 默认对上游**硬直连**会第一跳即断。从 v2.x 起，daemon 转发上游与 updater 出站均可经配置的 **HTTP CONNECT / SOCKS5 代理**出网。详见 [SPEC-007](../specs/SPEC-007-upstream-proxy.md)。

> **TLS 端到端到上游，代理只见密文、不 MITM**（不解密、不装 CA，符合不装本地 CA 的硬约束）。代理仅是传输层隧道，出站目的地不变（仍仅上游 LLM / `sieveai.dev`），不破坏完全本地运行承诺。

### 6b.1 最简配置（全局代理）

`~/.sieve/sieve.toml`，把 `<你的端口>` 换成本地代理实际监听端口（Shadowrocket / Clash 的 SOCKS / HTTP 入口）：

```toml
# 全局兜底代理：所有 upstream 默认经它出网
proxy = "socks5://127.0.0.1:<你的 SOCKS 端口>"   # Clash 默认常见 7891；Shadowrocket 自定
# 或 HTTP CONNECT 代理：
# proxy = "http://127.0.0.1:<你的 HTTP 端口>"     # Clash 默认常见 7890

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
protocol = "anthropic"
# 继承全局 proxy
```

### 6b.2 每 upstream 单独配代理 / 强制直连

```toml
proxy = "socks5://127.0.0.1:7891"   # 全局兜底

[[upstream]]
port = 11453
url = "https://api.anthropic.com"
protocol = "anthropic"
proxy = "http://127.0.0.1:7890"     # 该 upstream 专属代理，覆盖全局

[[upstream]]
port = 11455
url = "http://127.0.0.1:8080"       # 本地中转站，无需走代理
protocol = "openai"
no_proxy = true                      # 显式直连，无视全局 proxy 与 env
```

**优先级链（高 → 低）**：`upstream.no_proxy`(直连) > `upstream.proxy` > 全局 `proxy` > env(`HTTPS_PROXY` 优先于 `ALL_PROXY`) > 直连。

### 6b.3 env 兜底（零配置）

不想改 config 时可直接用标准环境变量（config 的 `proxy` / `no_proxy` 优先于 env）：

```bash
export HTTPS_PROXY="socks5://127.0.0.1:7891"   # 优先于 ALL_PROXY
# 或
export ALL_PROXY="socks5://127.0.0.1:7891"
```

env 同时覆盖 daemon 上游转发与 sieve-updater 出站（manifest / 规则下载），受限网络下更新检查与匿名 install 统计一并可用（详见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)）。

### 6b.4 proxy URL 格式 & 注意事项

- scheme：`http://`（CONNECT）/ `socks5://` / `socks5h://`（`h` = 远程 DNS）；带认证用 `scheme://user:pass@host:port`
- 解析失败 → 启动期 config 校验 **fail-fast 报错**
- 代理连接失败（拒绝 / 超时 / 认证失败 / CONNECT 非 200 / SOCKS 握手失败）→ **明确报错，绝不静默回退直连**（防止你以为走代理实则直连）
- **隐私提示**：经**远程**代理时代理可见 SNI 目标 / 目标 IP（即「你在连 `api.anthropic.com`」），但**不可见** prompt / response / API key。**推荐使用可信本地代理出口（Shadowrocket / Clash）**，不要把目标元数据交给不可信远端。

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

- 仅存 fingerprint + 元信息，**绝不存原始 prompt 内容**（数据本地化，参见 [API 参考 §2.2.3](../api/api-reference.md#223-审计事件查询)）
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

这三道防线确保 Critical 在所有版本不可关闭的安全承诺。

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

## 7e. 加密审计日志（`full` 档）部署指南

> 适用对象：需要审计「Sieve 到底擦了什么、拦了什么」的实际（**脱敏后**）内容片段的用户——尤其用于排查规则误伤（FP）。工程级设计见 [SPEC-009](../specs/SPEC-009-encrypted-audit-log.md)（Phase 2 落地）。

> ⚠️ **口令丢失 = 归档永久不可读，这是设计使然，不是 bug。** Sieve 跑 daemon 的机器上**只存公钥**、永不持有私钥；解密归档只能靠你离线保管的 identity 私钥 + 口令。**口令一旦丢失，所有历史归档无人能解，包括你自己、包括 Sieve 维护者。** 这是 write-only logging 模型的安全前提（连机器被攻陷也解不开历史归档），不是可修复的缺陷。开启 `full` 档前请读完本节最后一句。

### 7e.1 三档 logging level

`[audit].level` 控制落盘行为，**默认 `metadata`（即现状，零行为变化）**：

| level | 落盘内容 | 默认 |
|-------|---------|------|
| `off` | 什么都不留（不写 `audit.db`、不写归档） | 否 |
| `metadata` | 审计元数据：时间戳、方向、命中规则、类别、动作、用户处置、**脱敏后**上下文片段或哈希（即当前 `events` 表行为） | **是** |
| `full` | 在 `metadata` 基础上，额外存一份「**脱敏后**完整内容」的加密归档，带保留期 + 哈希链 | 否（**opt-in + 显式警告**） |

> `full` 档的元数据**仍照常写 `events` 表**，归档只是额外的加密副本，用 `events.id` 关联。`metadata` 与 `full` 共存，不分叉数据模型。

> **红线（不可违背）**：脱敏永远先于任何字节落盘。归档存的**永远是脱敏后内容**，脱敏前的明文密钥（API key / 助记词 / 私钥）无论是否加密都绝不落盘。

### 7e.2 启用 `full` 档（三步）

**Step 1：生成密钥对**

```bash
sieve audit keygen
```

`keygen` 生成一对 age（X25519）密钥：

- **recipient 公钥**（`age1...`）——给 daemon 加密用，可公开
- **identity 私钥**——审计解密用，**以口令保护**（age 原生 scrypt KDF），**必须移出本机**

`keygen` 会在终端最显眼处打印「口令丢失 = 归档永久不可读」警示，并提示你立即把 identity 私钥转移到离线保管（见 §7e.3）。

**Step 2：把 recipient 公钥写入 config**

`~/.sieve/sieve.toml`：

```toml
[audit]
level = "full"
recipient = "age1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
archive_dir = "~/.sieve/audit-archive"   # 归档段文件目录，默认即此
retention_days = 90                       # 保留期，0 = 永久保留
hash_chain = true                         # 防篡改哈希链，建议保持开启
```

daemon **只持 recipient 公钥**——结构上不具备解密能力。即使机器运行时被攻陷，攻击者也解不开历史归档（机器上根本没有私钥）。

**Step 3：把 identity 私钥移出本机**

```bash
# 确认 identity 已安全转移到离线保管后，从本机删除（见 §7e.3 保管最佳实践）
rm ~/path/to/identity.key   # daemon 不需要它，本机不该留
```

> daemon **不留存 identity**。本机只需 recipient 公钥（在 config 里）即可正常运行 `full` 档归档。

### 7e.3 私钥离线保管最佳实践

identity 私钥是解开全部历史归档的**唯一钥匙**，必须离开跑 daemon 的这台机器。按可靠性从高到低：

| 方式 | 说明 |
|------|------|
| **密码管理器** | 1Password / Bitwarden 等，存 identity 私钥 + 口令（口令与私钥分项存或分开记） |
| **另一台机器** | 转移到不跑 daemon 的审计机；解密也在那台机器执行 |
| **离线介质** | 加密 U 盘 / 纸质抄写存保险柜，适合极端隔离需求 |

**保管红线**：

- ❌ **不要**把 identity 私钥留在跑 daemon 的本机（违背 write-only logging 模型，机器被攻陷即全军覆没）
- ❌ **不要**把口令和私钥存在同一个会被一起泄露的地方
- ✅ 解密审计（§7e.5）应在**另一台 / 离线机器**执行，不在生产机解密

### 7e.4 归档目录：绝不同步到云盘 / 绝不 `git add`

`~/.sieve/audit-archive/`（即 `[audit].archive_dir`）存的是加密归档段文件。虽然内容已加密，但仍属于本地审计资产，**不应离开本机进入任何同步 / 版本控制系统**：

- ❌ **绝不**把 `~/.sieve/`（含 `audit-archive/`）放进 iCloud / Dropbox / OneDrive 等云盘同步目录
- ❌ **绝不** `git add ~/.sieve/audit-archive/`（或把 `~/.sieve` 纳入任何 git 仓库）
- ✅ 如确需把 `~/.sieve` 软链接到别处，确认目标路径**不在**任何同步 / 版本控制范围内

> 这正是威胁模型 第 3 条「目录被误同步到云盘或被误 `git add`」要防的暴露面。归档加密能挡住「文件被读到」，但不该主动把它送出本机。

### 7e.5 保留期与轮换

**保留期（`retention_days`）**：

- daemon 周期扫描归档段，删除超过 `N` 天的**整段密文文件**；每次清理写一条 `metadata` 审计事件记录「删了哪些段」
- 删除是 `full` 档归档上**唯一允许的变更**（区别于 `events` 表的 append-only）
- `retention_days = 0` = 永久保留（不自动清理）

**密钥轮换**：

```bash
sieve audit rotate-key
```

- 生成**新**密钥对，新归档段改用新 recipient 加密
- **旧段保持用旧 recipient 加密**——审计旧段仍需对应的**旧 identity 私钥**，因此**旧私钥也要继续妥善保管**，不能轮换后就丢
- 段头记录 key id，审计时自动定位该段对应哪把 identity

### 7e.6 审计解密（应在另一台 / 离线机器执行）

```bash
sieve audit decrypt --identity <identity-file>
```

流程：口令解锁 identity → 解密归档段 → 校验哈希链（`hash_chain = true` 时）→ 输出脱敏后内容。

> 哈希链保证历史归档**不可悄悄改写 / 删除 / 重排**（呼应透明可验证叙事）。诚实标注的残余局限：它**挡不住「末尾追加伪造」**（持公钥的 daemon 可继续合法追加），保证的是历史不可被篡改。

### 7e.7 关闭 `full` 档 / 回到默认

```toml
[audit]
level = "metadata"   # 回到默认行为；或 "off" 完全不留
```

切回 `metadata` / `off` 后 daemon 不再写新归档，但**已有归档段不会自动删除**（仍受 `retention_days` 约束或需手动清理）。手动清理已加密归档：

```bash
# 确认无后续审计 / 合规需要后，删除归档目录（不可恢复）
rm -rf ~/.sieve/audit-archive/
```

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

或在 config 中（详见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)）：

```toml
[update]
enabled = false
```

> **说明**：旧字段名 `SIEVE_DISABLE_RULES_UPDATE` / `[rules_update]` 已统一为 `SIEVE_NO_UPDATE` / `[update]` 段。详见 §13 自托管镜像 + manifest 接口 + [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

特性：

- 仅依赖**打包时内置**规则（最近一次签名规则包，编译进二进制）
- 启动时不发任何网络请求（除转发用户实际 API 调用）
- 启动横幅提示当前内置规则版本 + 发布日期，供用户判断是否过期

---

## 11. 运行模式与降级

Phase 1 完全免费，无需 license、无试用流程，安装后即开放全部检测能力。

### 11.1 降级模式

在某些异常条件下（如本地配置不完整），Sieve 进入"只读警告"模式：

- ✅ **Critical 仍然阻断**（产品安全承诺，任何状态不变）
- ⚠ High / Medium / Low 仅记录，不弹窗、不警告
- 不停止运行 —— 任何情况下都不让用户失去基本保护

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

1. **不要直接用 `severity_overrides` 关 Critical** —— Critical 不可关闭，禁止
2. 优先加 `.sieveignore`：
  ```bash
   sieve sieveignore add <fingerprint> --comment "false positive: <场景>"
  ```
3. 对 Medium / High 规则可在 `[detection.severity_overrides]` 降级
4. 报 issue（带 fingerprint + 上下文，**不要贴原始 prompt**），维护者会评估规则改进

### 12.4 如何完全离线

参见 [§10 离线运行](#10-离线运行)。

### 12.5 sieve doctor 新检查项（v2.0）

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

## 13. 企业自托管镜像 / 私有更新源

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

### 13.4 隐私敏感场景（只想要规则更新，不发送匿名统计字段）

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
- 请求中不再包含任何安装标识符

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
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[development.md](development.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)
- 架构文档：[../design/architecture.md](../design/architecture.md)
- SPEC-006 manifest 协议规格：[../specs/SPEC-006-update-and-telemetry.md](../specs/SPEC-006-update-and-telemetry.md)
- 数据模型：[../design/data-model.md](../design/data-model.md)
- 术语表：[../glossary.md](../glossary.md)

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。部署 / 运维流程任何变更必须同步更新本文与 [CHANGELOG](../changelog/CHANGELOG.md)。

