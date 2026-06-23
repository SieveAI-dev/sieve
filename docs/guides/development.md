# Sieve 开发指南

> Version: v2.1 — 2026-05-07
>
> **状态：v2.0 + v2.1 代码全量落地。**
> 五 crate 骨架完整，IPC + hook 路径跑通，行为序列检测（IN-SEQ-*）以 feature flag 形式 opt-in，用户规则 CLI 可用，benchmark CI gate 已接入 `.github/workflows/bench.yml`。

---

## 1. 前置依赖

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | **1.87.0 stable**（已锁定在 `rust-toolchain.toml`,推荐 [`rustup`](https://rustup.rs/) 管理） | 主语言 |
| `cargo` | 同 Rust 版本 | 构建 / 依赖管理 |
| `clang` | 16+ | `vectorscan-rs` 编译需要（C/C++ 后端） |
| `cmake` | 3.20+ | vectorscan 子构建 |
| `ninja` | 1.10+ | vectorscan 子构建 |
| `pkg-config` | 0.29+ | 系统库探测 |
| `boost` | 1.74+（headers） | vectorscan 依赖 |
| `ragel` | 6.10+ | vectorscan 解析器生成 |
| `git` | 2.40+ | 版本控制 |
| `cosign` | 2.x | 二进制签名验证（参见 [部署指南](deployment.md)） |
| `pnpm` | 9.x（可选） | 若有 web landing page 子项目 |

**一键安装系统依赖**：

```bash
# macOS
brew install cmake ninja pkg-config boost ragel

# Ubuntu / Debian
sudo apt-get install -y cmake ninja-build pkg-config libboost-dev ragel
```

**操作系统支持矩阵（Phase 1）**：

| OS | Tier | 状态 | 备注 |
|----|------|------|------|
| macOS 13+（Apple Silicon + Intel） | Tier 1 | 全功能 | Phase 1 唯一支持 |
| Linux / Windows | Phase 2 | 不支持 | Phase 2 计划，时间未定 |

---

## 2. 仓库结构

```
sieve/
├── Cargo.toml                      # workspace root
├── rust-toolchain.toml             # 钉死 toolchain，CI / 本地一致
├── deny.toml                       # cargo-deny 配置
├── crates/
│   ├── sieve-core/                 # 协议层 + SSE Parser + UnifiedMessage + Forwarder
│   ├── sieve-rules/                # 规则定义 + vectorscan 编译 + 匹配引擎 + Ed25519 验证
│   ├── sieve-cli/                  # 主二进制 + 配置 + 弹窗 + 审计 SQLite + 管理 API
│   ├── sieve-ipc/                  # IPC server：Unix socket + GUI 通知协议
│   └── sieve-hook/                 # Claude Code PreToolUse hook 二进制
├── rules/                          # 内置规则定义（toml/yaml）
│   ├── outbound/                   # OUT-01~12
│   └── inbound/                    # IN-CR-* / IN-GEN-*
├── tests/
│   ├── integration/                # 端到端：起 sieve 进程 + 模拟上游 + 实测 SSE
│   └── benches/                    # criterion benchmark
├── fuzz/                           # cargo-fuzz targets（独立 crate）
│   ├── Cargo.toml
│   └── fuzz_targets/
│       ├── sse_parser.rs
│       ├── tool_use_aggregator.rs
│       └── address_extract.rs
├── docs/                           # 当前文档目录
└── scripts/
    ├── repro-build.sh              # 可复现构建脚本
    └── verify-cosign.sh            # 验证脚本
```

> Native GUI App（macOS 状态栏 + 审批弹窗）在独立仓库 **`sieve-gui-macos`**，本仓库不构建 GUI，不包含 Swift 代码。两仓库通过 sieve-ipc 的 IPC 协议契约协调（详见 [SPEC-001](../specs/SPEC-001-sieve-hook-protocol.md)）。

### crate 边界（[`.cursorrules` §3.3](../../.cursorrules)）

| crate | 职责 | 不允许做 |
|-------|------|---------|
| `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
| `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
| `sieve-cli` | 入口 / 配置 / 弹窗 / 审计 SQLite / 管理 API | 直接做规则匹配（必须经 `sieve-rules`） |
| `sieve-ipc` | Unix socket IPC server + GUI 通知协议（pending/decision 目录扫描） | 任何规则匹配逻辑 |
| `sieve-hook` | Claude Code PreToolUse hook 二进制（`#[cfg(target_os = "macos")]` 严格限定） | 直接访问 sieve-rules / sieve-core 内部实现 |

跨 crate 调用走显式 trait，避免互相 import 实现细节。

---

## 3. 构建命令

### 3.1 一次性环境校验

```bash
rustup show                                    # 确认 toolchain == 1.87.0 stable（锁定在 rust-toolchain.toml）
cargo --version
clang --version
```

### 3.2 日常构建

```bash
# 默认（开发用，debug build）
cargo build --workspace --locked

# Release（reproducible build，本地用）
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) cargo build --release --workspace --locked

# 指定 target（macOS aarch64 / x86_64，Phase 1 仅支持 macOS）
cargo build --release --locked --target aarch64-apple-darwin -p sieve-cli
cargo build --release --locked --target x86_64-apple-darwin -p sieve-cli
```

### 3.3 测试 / Lint / 审计

**PR 必须全绿才能合并**（参见 §8）：

```bash
cargo fmt --all -- --check                                              # 格式
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  # 警告即错误
cargo test --workspace --locked                                         # 全部单元 + 集成
cargo deny check                                                        # 许可证 / 来源 / 重复依赖审计
```

**PRD §9 #7 数据集回归**（不阻塞 PR 但合并前必须人工跑过，详见 [CHANGELOG v1.5.1](../changelog/CHANGELOG.md#v151-rule-expansion---2026-05-01)）：

```bash
# 跑 1896 样本 baseline（标 #[ignore]，不在 cargo test 默认范围）
# 数据集结构：
#   crates/sieve-rules/bench-data/
#     ├── attacks/                     # 现有 226 条按规则 ID 分类
#     ├── attacks-by-fear/             # 新增 600 条按"用户最怕五件事"分类
#     │   ├── signing/  transfer/  env-leak/  private-key/  shell-rce/  (各 120 条)
#     ├── benign/                      # 现有 70 条 generic Q&A
#     └── benign-near/                 # 新增 1000 条"看起来像攻击但合法"按规则 ID 对称分桶
#         ├── near-OUT-api-keys/       near-IN-CR-02-rce/      ...  (10 桶 × 100 条)
cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture --test-threads=1

# 阈值（assertion 内嵌）：
#   benign Critical FP rate < 0.5%（PRD §9 #7 硬约束）
#   attacks recall rate > 95%
# 输出按桶聚合，FP 高时按桶定位是哪类合法场景误伤；recall 漏时按桶定位规则盲区
```

> 数据集扩充 / 新规则 PR：必须跑这条命令，把 per-bucket 报告贴在 PR description 里。

**v2.0 新增集成测试**（默认模式 + `sequence_detection` feature on 两套；具体计数随版本浮动，**以最新 status / CI artifact 为准**，不在文档里硬编码）：

```bash
# 行为序列检测端到端（v2.0 §5.3，需 opt-in feature）
cargo test -p sieve-cli --features sieve-cli/sequence_detection --test sequence_window_e2e

# 灰名单集成（v2.0 §5.4.2）
cargo test -p sieve-cli --test graylist_integration   # 14 个用例

# SSE 流损坏边界（corruption.rs，v2.0 SSE hardening）
cargo test -p sieve-core --test corruption            # 12 个用例

# Content-Type 矩阵（v1.5 OpenAI 协议适配验证）
cargo test -p sieve-core --test content_type_matrix   # 6 个用例

# feature flag 开启后全量测试（643 用例）
cargo test --workspace --features sieve-cli/sequence_detection --locked
```

**用户规则 CLI（v2.0 §5.5.2）**：

```bash
# 启动 $EDITOR（fallback vim/nano），编辑 ~/.sieve/rules/user.toml
# 保存后自动 lint pipeline + atomic backup + IPC notify reload
sieve rules edit

# 列出用户规则 + 系统规则总数
sieve rules list

# 禁用指定规则（toml 序列化 + atomic rename）
sieve rules disable <rule-id>

# 重新启用
sieve rules enable <rule-id>
```

### 3.4 启动 daemon（开发模式）

```bash
# 1. 写最小 config（旧 schema，仍兼容）
cat > /tmp/sieve.toml <<EOF
upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
EOF

# 2. 启动
SIEVE_LOG=info cargo run -p sieve-cli -- start --config /tmp/sieve.toml

# 3. 用 Claude Code 接入（另一个终端）
#    重要：--bare 强制走 ANTHROPIC_API_KEY 路径，否则 Claude Max OAuth 会绕开
#    ANTHROPIC_BASE_URL 直连 claude.ai 后端，daemon 收不到流量。
ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude --bare -p "hello"

# 4. 验证流量过代理：daemon 日志应显示 detection 或 INBOUND/OUTBOUND BLOCKED；
#    若 daemon 日志为空但 claude 仍正常返回，说明请求绕过代理（典型：忘了 --bare）。
```

#### 3.4a Multi-listener 配置（ADR-026 推荐）

配置 daemon 同时监听多个端口，每个 port 独立连接不同的真实上游：

```bash
# 多 listener config
cat > /tmp/sieve.toml <<EOF
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
EOF

# 启动（任一 listener bind 失败 → fail-fast 整体退出）
SIEVE_LOG=info cargo run -p sieve-cli -- start --config /tmp/sieve.toml

# 切换不同上游：改 ANTHROPIC_BASE_URL 端口即可
ANTHROPIC_BASE_URL=http://127.0.0.1:11453 ANTHROPIC_AUTH_TOKEN=<anthropic-key> claude --bare
ANTHROPIC_BASE_URL=http://127.0.0.1:11454 ANTHROPIC_AUTH_TOKEN=<deepseek-key>  claude --bare

# 协议错位测试（fail-closed）
curl -X POST http://127.0.0.1:11453/v1/chat/completions   # Anthropic listener 收 OpenAI path
# → 400 + {"type":"sieve_blocked","reason":"listener_protocol_mismatch", ...}
```

详见 [ADR-026 §决策 1-4](../design/ADR-026-port-based-listener-routing.md)。

> daemon 日志环境变量是 `SIEVE_LOG`（不是 `RUST_LOG`），格式 `<crate>=<level>,fallback`，
> 例如 `SIEVE_LOG=sieve_cli=debug,info`。

**生产模式**（Week 5 起推荐）：运行 `sieve setup` 自动完成 Claude Code settings.json 注册 hook + 写 `ANTHROPIC_BASE_URL` + 注册 launchd，详见 [deployment.md §2.1](deployment.md#21-macos-安装)。

### 3.5 dev 模式跑 IPC + hook

Week 5 后，本地联调 IPC 通道与 hook 的标准流程：

```bash
# 1. 起 daemon（含 IPC server）
SIEVE_LOG=sieve_cli=debug,info cargo run -p sieve-cli -- start --config /tmp/sieve.toml

# 2. 起 mock GUI（模拟 sieve-gui-macos 的 IPC 连接，待 sieve-ipc crate 完成后可用）
cargo run -p sieve-ipc --example mock_gui
# 注：mock_gui example 尚未实现时，标注"待 Week 5 C3/C4 子代理实现"

# 3. 触发 hook 单测（模拟 Claude Code PreToolUse 调用）
cargo run -p sieve-hook -- check --request-id <UUID> --sieve-home /tmp/sieve-test
```

> `sieve-hook` 与 `sieve-ipc` 均用 `#[cfg(target_os = "macos")]` 严格限定，**不在 Linux / Windows 编译**。

### 3.6 系统依赖（macOS）

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# vectorscan-rs 编译依赖（Week 2 起实际使用）
brew install cmake ninja pkg-config boost ragel
```

---

## 4. SSE Fuzz 测试（PRD §9 硬约束 #5）

> **PR 不带 fuzz 不合并。**SSE 边界处理决定了 Sieve 能否真正拦住注入；任何涉及 `sse_parser` / `tool_use_aggregator` 的改动必须补 fuzz 用例。

### 4.1 必须覆盖的 6 类边界

参见 `crates/sieve-core/src/sse/README.md`（待写）：

1. **半行 chunk**：`event:` 与 `data:` 跨多个 TCP read 到达
2. **跨 chunk 的 `\n\n` 分隔符**：第一个 `\n` 在前一 chunk，第二个在后一 chunk
3. **嵌入 C0 控制字符**：`\x00`～`\x1F` 出现在 `data:` body 内
4. **多 event 粘包**：单次 read 返回 N 个完整 event + 半个不完整 event
5. **提前断流**：上游 RST / FIN 在 event 中间到达
6. **畸形 JSON**：`data:` body 在多 chunk 拼接后才完整成 JSON，期间不能误判

### 4.2 Fuzz 命令

```bash
# smoke test（CI 跑 60s）
cargo fuzz run sse_parser -- -max_total_time=60

# 本地深度跑（建议 ≥ 10 分钟）
cargo fuzz run sse_parser -- -max_total_time=600

# 列出现有 target
cargo fuzz list

# 重放历史 corpus 验证回归
cargo fuzz run sse_parser fuzz/corpus/sse_parser/
```

### 4.3 Fuzz 失败处理

- 失败 corpus 自动落到 `fuzz/artifacts/sse_parser/crash-<hash>`
- **必须**：将 crash 输入压缩后加入 `fuzz/corpus/regression/` + 写单元测试复现 + 修复后保留 corpus 永久回归

---

## 5. 性能 Benchmark

### 5.1 P99 < 20ms 是 CI gate

PRD §6.4 性能预算：

| 操作 | 目标延迟 |
|------|---------|
| 普通 token 流式 chunk | +30-200 µs |
| 工具调用边界完整检查 | +5-15 ms |
| 整体 P99 添加延迟 | **< 20 ms** |
| 内存峰值 | < 100 MB |
| 二进制大小 | < 20 MB 单文件 |
| 启动时间 | < 500 ms |

CI 用 [`criterion`](https://github.com/bheisler/criterion.rs) 跑核心 benchmark：
- **自动门禁**：bench mean 退化 > 10% → CI block 合并（`.github/workflows/bench.yml`，criterion mean regression gate；P99 不在 CI 自动门禁中，因 macOS runner 噪声大不稳定）。
- **真 P99 验证**：本地或 release 候选构建跑 `cargo bench --bench latency` 后用 criterion 报告查看 p99 行；P99 > 20ms 视为退化（PRD §6.4），由 reviewer 在 PR 中确认。

### 5.2 命令

```bash
cargo bench --bench latency                # 主 latency benchmark
cargo bench --bench latency -- --save-baseline main   # 保 baseline
cargo bench --bench latency -- --baseline main        # 与 baseline 对比

# 单个 benchmark
cargo bench --bench latency -- sse_parse_chunk

# v2.0 新增 benchmark target（PRD §6.3.2）
cargo bench -p sieve-rules --bench scan_70_rules        # 70 条规则全量扫描
cargo bench -p sieve-rules --bench scan_with_user_rules # 含用户自定义规则时的扫描性能
```

### 5.3 CI benchmark gate（v2.0 新增）

`.github/workflows/bench.yml` 在 macOS runner 上自动运行 benchmark 并与 main 分支 baseline 对比：

- 任何 bench mean 退化 > 10% → CI 失败，PR 无法合并（即上文所说的 mean regression gate；P99 退化由 reviewer 手工 smoke 确认）
- 对比脚本：`scripts/bench_compare.sh`
- 本地手动复现：

```bash
# 保 main baseline
git checkout main
cargo bench -p sieve-rules --bench scan_70_rules -- --save-baseline main

# 回切分支，与 baseline 对比
git checkout <your-branch>
cargo bench -p sieve-rules --bench scan_70_rules -- --baseline main
```

### 5.4 数据集

性能 benchmark 必须用 §7 描述的真实 benign 会话回放（50-100 条），而**不是**人造均匀负载。

---

## 6. 检测规则编写指南

每条新规则的 PR **必须**包含以下 6 项材料；缺一项 reviewer 直接打回。

### 6.1 规则 ID 命名规范

| 前缀 | 含义 | 示例 |
|------|------|------|
| `OUT-XX` | 出站规则（用户 → 模型） | `OUT-01` OpenAI key |
| `IN-CR-XX` | 入站 Crypto 钩子 | `IN-CR-05` 签名工具 fail-closed |
| `IN-GEN-XX` | 入站通用 | `IN-GEN-02` 远程脚本执行 |
| `IN-MCP-XX` | 入站 MCP 拦截（Phase 2） | `IN-MCP-01` MCP allowlist |

ID 在所有版本不复用，废弃后保留占位（`<deprecated since vX.Y>`）。

### 6.2 PR 必备材料

1. **Severity + FP 上限承诺** —— 引用 PRD §6.5 误报率预算；如想申报 Critical，须明确 `< 0.5%` 或更严格
2. **≥ 5 条 benign 测试用例** —— 真实场景采样，**不接受人造**（如 README 中常见的 `0xDEADBEEF`）
3. **≥ 5 条 malicious 测试用例** —— 至少 2 条来自公开 incident（链上、安全播报、UCSB 论文等），**附 source URL**
4. **误报治理策略** —— 占位符黑名单关键字、上下文关键词、entropy 阈值依据
5. **回滚方案** —— 规则上线后 FP 失控如何快速 disable：通常通过 `severity_overrides` 降级或下一个签名规则包覆盖
6. **基准跑分** —— 在新规则下 §7.1 benchmark 数据集的 FP / Recall 数据

### 6.3 命中算法层级（强烈建议）

按以下顺序短路，**最便宜的过滤器先跑**：

1. **前缀字符串匹配**（μs 级，vectorscan SIMD）
2. **正则 / 词表匹配**
3. **校验位 / entropy / SHA-256**（如 BIP39 SHA-256 校验，PRD §9 硬约束 #4）
4. **上下文关键词 + 占位符黑名单**

**禁止**：把 SHA-256 / 大量 regex 放在第 1 层，会爆 P99。

---

## 7. `.sieveignore` 工作流

### 7.1 本地白名单**永远不上传**

- `.sieveignore` 保存在 `~/.sieve/.sieveignore`（用户级），**不在仓库 root**
- 仓库 `.gitignore` 默认忽略 `.sieveignore` / `*.sieveignore`
- 提交 PR 前确认：`git status` 不应出现 `.sieveignore`

### 7.2 fingerprint 格式

```
<rule_id>:<sha256_prefix_8_hex>
```

例：`OUT-09:7a3b9c1d`

完整定义见 `docs/design/data-model.md`（待写）。

### 7.3 添加 / 移除

通过本地管理 API（参见 [API 参考 §2.2.4](../api/api-reference.md#224-白名单管理-sieveignore)）或 CLI：

```bash
sieve sieveignore add OUT-09:7a3b9c1d --comment "test mnemonic from unit-test fixture"
sieve sieveignore list
sieve sieveignore remove OUT-09:7a3b9c1d
```

---

## 8. 配置 / 调试

### Sieve 配置（`sieve.toml`）

Week 2 起支持的字段，Week 5 后新增 IPC / 部署相关字段（与 [data-model.md §5](../design/data-model.md) 对齐）：

```toml
upstream_url = "https://api.anthropic.com"      # 上游 LLM API
port = 11453                                     # 本地代理端口
bind_addr = "127.0.0.1"                         # 强制 loopback（ADR-003）
log_path = "/path/to/audit.db"                  # 审计 SQLite，可选（默认 ~/.sieve/audit.db）
tls_verify_upstream = true                      # 上游 TLS 校验（测试场景可关）
rules_path = "/path/to/outbound.toml"           # 出站规则集，可选（默认 crates/sieve-rules/rules/outbound.toml）
sieveignore_path = "/path/to/.sieveignore"      # fingerprint 白名单，可选（默认 ~/.sieve/sieveignore）
dry_run = false                                 # true=仅记录命中不拦截，false=Critical 命中返 426

# Week 5 新增字段（IPC + 部署，macOS only）
ipc_socket_path = "/tmp/sieve.sock"             # IPC Unix socket，GUI App 连接此路径
pending_dir = "~/.sieve/pending"                # inbound hold 写入待审批请求的目录
decisions_dir = "~/.sieve/decisions"            # GUI 写回审批决定的目录（approved/rejected/expired）
preset = "crypto"                               # 规则预设：crypto / general / strict
launchd_plist_path = "~/Library/LaunchAgents/tools.sieve.agent.plist"  # sieve setup 写入的 plist 路径
gui_socket_enabled = true                       # 是否启用 GUI IPC 通知通道（false=降级为 CLI 文本弹窗）
```

### `.sieveignore`（fingerprint 白名单）

每行一个 16 字符 hex fingerprint，`#` 开头注释，空行忽略。指纹算法：

```
fingerprint = SHA-256("{rule_id}:{normalized_content}")[..8].hex()
normalized_content = content.trim()  // 长串截断到前 32 字节
```

触发规则后，从 `sieve` 日志中复制 `fingerprint` 字段添加到此文件即可豁免。Phase 1 不热加载，变更需重启 daemon。

### `--dry-run`（临时关闭实际拦截）

```bash
sieve start --config sieve.toml --dry-run
# 命中只 tracing::warn! 记录，不返 426，继续转发上游
# CLI 出现 --dry-run 即覆盖 config.dry_run=true（无法从 CLI 显式关闭）
```

或在开发调试时直接加 flag：

```bash
RUST_LOG=info cargo run -p sieve-cli -- start --config sieve.toml --dry-run
```

### Benchmark（性能预算 PRD §6.4）

```bash
cargo bench -p sieve-rules                        # 全部 bench target
cargo bench -p sieve-rules --bench scan_70_rules  # 70 条规则扫描（v2.0 新增）
```

当前覆盖 vectorscan block mode 不同 buffer size（1KB / 100KB / 1MB）+ v2.0 新增 70 条 / 用户规则两个 target；CI gate（`.github/workflows/bench.yml`）macOS runner 上 mean 退化 > 10% 失败，详见 §5.3。

---

## 8a. Feature Flag 矩阵（v2.0）

> PRD §9 #15：行为序列检测（IN-SEQ-*）GA 默认关闭，闭测 / dogfood 用户主动 opt-in。

`sieve-cli` 的 Cargo feature 默认全部关闭（`default = []`）。下表为可选特性，按需 opt-in 编译；
默认二进制只含核心命令面，不拉入特性专属的重依赖。

| Feature | Cargo flag | 默认 | 含义 | 仅启用时编译的依赖 |
|---------|-----------|------|------|------------------|
| `sequence_detection` | `--features sieve-cli/sequence_detection` | **OFF** | InboundFilter 启用 `SessionState::ToolUseSequence`；激活 3 条 IN-SEQ-* 启发式规则 | —— |
| `usage` | `--features usage` | **OFF** | `sieve usage` 子命令 + daemon 端本地用量核算路径 | `tiktoken-rs` |
| `audit-crypto` | `--features audit-crypto` | **OFF** | `sieve audit keygen/rotate-key/decrypt` 子命令 + daemon 端 `[audit].level = "full"` 加密归档 | `age` / `sha2` / `base64` |

**默认二进制命令面**：`start` / `version` / `setup` / `doctor` / `uninstall` / `rules` / `decisions` / `audit`（仅 `tail` / `query` / `show`）。
默认**不含** `usage` 子命令、不含 `audit keygen/rotate-key/decrypt`；`audit tail/query/show` 始终可用。
未编入 `audit-crypto` 时，配置 `[audit].level = "full"` 会优雅降级为 `metadata` 档（只写元数据，不加密归档）。

**构建 / 测试**：

```bash
# 默认构建（不含 tiktoken-rs / age 等特性依赖）
cargo build -p sieve-cli --bins

# 全特性构建（含 usage + audit-crypto）
cargo build -p sieve-cli --bins --features "usage audit-crypto"

# opt-in 行为序列检测构建
cargo build --features sieve-cli/sequence_detection

# 含 feature 的测试（643 用例 vs 默认 633 用例）
cargo test --features sieve-cli/sequence_detection

# 仅序列检测端到端
cargo test -p sieve-cli --features sieve-cli/sequence_detection --test sequence_window_e2e
```

> clippy 在默认与全特性两套配置下均须 `-D warnings` 干净：
> `cargo clippy -p sieve-cli --all-targets --features "usage audit-crypto" --locked -- -D warnings`。

**运行时启用**（dogfood 用）：在 `~/.sieve/config.toml` 中：

```toml
[features]
sequence_detection = true
```

重启 daemon 后生效；不需要重新编译二进制（daemon 在运行时读取此 flag，但 IN-SEQ-* 规则仍需二进制编译时启用 feature）。

> 注意：GA 发布的标准二进制**不含** `sequence_detection`，闭测版本单独编译提供。

### 用户规则热加载开发工作流

编辑用户规则时，无需重启 daemon（PRD §5.5.2）：

1. `sieve rules edit` → 调 `$EDITOR` 打开 `~/.sieve/rules/user.toml`
2. 保存退出 → 自动 lint pipeline 校验
3. lint 通过 → 自动 atomic backup（保留最近 10 份，命名 `user.toml.bak.YYYYMMDD-HHMMSS`）→ atomic rename 原子替换
4. atomic rename 后 → IPC notify daemon 执行 hot swap（zero-downtime，不中断现有连接）
5. lint 失败 → 原文件不变 + stderr 打印违规清单，用户修改后重跑 `sieve rules edit`

```bash
# 验证 hot reload 是否生效（reload 后 events 表会出现 user_rules_reloaded 记录）
sieve events --since 1m | grep user_rules_reloaded
```

---

## 9. Fuzz 测试（PRD §9 #5 硬约束，Week 3 起双引擎就绪）

### 9.1 cargo fuzz（libFuzzer）

```bash
# 安装 nightly toolchain（libFuzzer 需要 sanitizer）
rustup install nightly
cargo install cargo-fuzz --locked

# 列出所有 target
cargo +nightly fuzz list
# 输出：sse_parser / tool_use_aggregator / inbound_filter

# 跑 60 秒（CI 模式）
cargo +nightly fuzz run sse_parser -- -max_total_time=60

# 持续跑（本地深度测试）
cargo +nightly fuzz run sse_parser

# 最小化 crash 输入
cargo +nightly fuzz tmin sse_parser fuzz/artifacts/sse_parser/crash-xxx

# 覆盖率（LLVM 报告）
cargo +nightly fuzz coverage sse_parser
```

### 9.2 AFL++（afl crate）

仅在专用 fuzz worker / docker 镜像中跑：

```bash
cargo install cargo-afl --locked
cd fuzz_afl
cargo afl build --bin sse_parser_afl
mkdir -p afl_out
cargo afl fuzz -i ../fuzz/corpus/sse_parser -o afl_out target/debug/sse_parser_afl
```

### 9.3 Corpus 共享

cargo fuzz 与 AFL++ 共享 `fuzz/corpus/<target>/` 目录（都是字节文件）。AFL++ `afl_out/queue/` 中发现的新输入定期 rsync 回 corpus 供 libFuzzer 复用。

### 9.4 必须覆盖的 5 类边界

参见 `crates/sieve-core/src/sse/parser.rs`：
1. **半行 chunk**：event: 与 data: 跨多个 TCP read
2. **跨 chunk 分隔符**：`\n\n` 切成两个 chunk
3. **C0 控制字符**：0x00-0x1F 嵌入 data: body
4. **多 event 粘包**：单 chunk 含 N 个完整 events
5. **提前断流**：连接在 event 中途关闭

---

## 10. 本地 dogfood 流程

每个开发者**必须** 100% 时间用 Sieve 工作：

```bash
cargo build --release
./target/release/sieve --config dev.toml      # 前台跑，方便看日志

# 另一个终端
export ANTHROPIC_BASE_URL=http://127.0.0.1:11453
export ANTHROPIC_AUTH_TOKEN=<your-real-key>
claude   # 或任何使用 Anthropic API 的工具
```

Dogfood 守则：

- **每次 false positive 触发都开 issue**，复现步骤 + 关联 fingerprint + 同步加 fuzz / 单元测试
- **禁止**：用 `severity_overrides` 把 Critical 降级当作"修 FP"——必须改规则
- 每周末汇总 FP 计数，超阈值（详见 PRD §6.5）必须当周修

---

## 11. 提交 PR 流程

### 11.1 沟通节奏

```
1. 先开 issue：描述问题 / 方案 / 代码改动范围
   ↓
2. 等维护者回复确认方向（避免无效 PR）
   ↓
3. fork → branch（命名建议 `feat/<scope>` / `fix/<scope>` / `rule/<id>`）
   ↓
4. 写代码 + 测试 + 文档
   ↓
5. 提 PR（关联 issue 编号，带 §9.2 自检清单）
   ↓
6. CI 全绿 → reviewer 走查 → 合并
```

### 11.2 PR 自检清单（提交前必跑）

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
cargo fuzz run sse_parser -- -max_total_time=60   # 只在 SSE 相关 PR
cargo bench --bench latency -- --baseline main    # 只在性能相关 PR
```

PR 描述中**强制**附上：

- 关联 issue
- 改动摘要（**为什么** > 什么）
- §11.3 文档同步清单
- 是否触及 PRD §9 硬约束（默认不允许绕过）

### 11.3 文档同步要求

参考 [`.cursorrules` §1.5 文档化触发条件](../../.cursorrules) 和用户级文档规则中的"修改功能 Checklist"：

- [ ] 阅读现有需求文档，确认变更范围
- [ ] 阅读现有设计文档，评估影响
- [ ] 更新所有涉及的文档（用 `rg` 搜索关联引用）
- [ ] 在 [`CHANGELOG.md`](../changelog/CHANGELOG.md) 记录变更**原因**（不是 what，是 why）
- [ ] 检查 [README.md](../../README.md) 描述是否仍然准确
- [ ] 修改 API 时必须同步 [`docs/api/api-reference.md`](../api/api-reference.md)
- [ ] 架构变更必须有 `docs/design/ADR-*.md`

### 11.4 CI Gate 清单

GitHub Actions（`.github/workflows/ci.yml`）强制：

- [ ] `cargo fmt --check` 通过
- [ ] `cargo clippy -D warnings` 通过
- [ ] `cargo test --workspace` 通过
- [ ] `cargo audit` 无 P0 / P1 漏洞
- [ ] `cargo deny check` 通过（许可证 + 重复依赖）
- [ ] SSE fuzz smoke 60s 不 crash
- [ ] criterion benchmark P99 < 20ms（与 main baseline 对比）
- [ ] `cosign verify-blob` 验证发布 artifact 签名（仅 release PR）
- [ ] reproducible build 复算 SHA-256 与 GitHub Actions 产物一致

### 11.5 Reviewer

当前维护团队规模较小，review 资源有限。请在 PR 描述中详尽列出测试场景，降低 review cost。

### 11.6 安全披露

**不要在公开 issue 公布 0day**。流程：

1. 邮件到 `security@<域名待定>`，**附 PGP 公钥加密**（指纹待定，发布于 README）
2. 等待维护者回复确认（48h 内）
3. 协商修复时间窗与公开披露日期
4. 修复合并后再写 advisory（GitHub Security Advisory）

---

## 12. 文档同步要求（再强调）

> **改代码前必须先读 `@docs/` 中相关文档**（[`.cursorrules` §1.1](../../.cursorrules)）。

### 修改功能 Checklist

- [ ] 阅读现有需求文档，确认变更范围
- [ ] 阅读现有设计文档，评估影响
- [ ] 更新所有涉及的文档（使用搜索确认）
- [ ] 在 `CHANGELOG.md` 记录变更原因
- [ ] 检查 README 描述是否仍然准确

### 新增功能 Checklist

- [ ] 创建 `docs/requirements/PRD-<功能名>.md`
- [ ] 创建 `docs/design/<功能名>-design.md` 或 ADR
- [ ] 更新 `README.md` 功能列表
- [ ] 更新 `docs/api/api-reference.md`（如有 API 变更）
- [ ] 更新 `docs/changelog/CHANGELOG.md`

---

## 13. 更新通道环境变量（ADR-030）

> 三个环境变量控制更新检查 / 遥测行为，适用于开发者本地测试与 CI 场景。优先级：**env var > sieve.toml [update] > 默认值**。完整协议规格见 [SPEC-006](../specs/SPEC-006-update-and-telemetry.md)。

### 13.1 变量速查表

| 变量 | 作用 | 默认 | 典型使用场景 |
|------|------|------|------------|
| `SIEVE_NO_UPDATE` | 完全跳过更新检查（不发任何请求，规则冻结，无遥测） | 未设（默认开启） | CI 测试 / 离线开发 / 审计期 / 避免遥测污染 |
| `SIEVE_NO_TELEMETRY` | 仍发更新请求但省略 `uid` 字段 | 未设（默认附带 uid） | 隐私敏感场景 / 想要规则更新但不参与统计 |
| `SIEVE_UPDATE_URL` | 覆盖默认 manifest URL | 未设（使用 `https://updates.sieveai.dev/v1/manifest`） | 企业自托管镜像 / 本地 mock 服务器测试 |

任何非空值视为启用该开关（unix-style，参考 `NO_COLOR` 惯例）。

### 13.2 使用示例

**CI 测试中禁用更新检查（推荐）**：

```bash
# 避免 CI 遥测污染 + 防止更新检查影响测试隔离性
SIEVE_NO_UPDATE=1 cargo run -p sieve-cli -- start --config /tmp/sieve.toml
```

**本地 mock 服务器测试**：

```bash
# 起一个本地 mock manifest 服务器（mockito / wiremock 等）
SIEVE_UPDATE_URL=http://localhost:8080/v1/manifest cargo run -p sieve-cli -- start --config /tmp/sieve.toml
```

**仅关闭遥测，保留规则更新**：

```bash
SIEVE_NO_TELEMETRY=1 cargo run -p sieve-cli -- start --config /tmp/sieve.toml
```

### 13.3 Banner 行为（`SIEVE_NO_UPDATE` 强制可见）

检测到 `SIEVE_NO_UPDATE` 时，daemon 启动日志**必须**打印：

```
update check disabled by SIEVE_NO_UPDATE
```

这一行是强制的：防止开发者忘了在 shell profile 里设过此变量，后来奇怪为什么规则不更新。

### 13.4 sieve.toml 等价配置

```toml
[update]
enabled = true              # 等价 SIEVE_NO_UPDATE（false = 禁用）
telemetry = true            # 等价 SIEVE_NO_TELEMETRY（false = 省略 uid）
url = "https://updates.sieveai.dev/v1/manifest"  # 等价 SIEVE_UPDATE_URL
check_interval_hours = 6    # 定时检查间隔，env var 无等价物
channel = "stable"          # 发布通道，Phase 2 加 beta
```

env var 优先级始终高于 toml：同时设置时 env var 胜出。

---

## 14. 测试隔离：SIEVE_HOME

> 适用于编写集成测试时避免污染真实用户数据（`~/.sieve/audit.db` / `~/.sieve/ipc.sock` 等）。

`config.rs::sieve_home()` 优先读 `SIEVE_HOME` env var（优先级高于 `$HOME/.sieve`），`audit_db_path` / `sieveignore` 路径全部走该函数。集成测试 spawn daemon 时**必须**同时注入以下三个环境变量：

```bash
# 测试隔离标准模式（在 spawn helper 中注入）
SIEVE_HOME=/tmp/sieve-test-<uuid>   # 隔离所有文件 I/O 到 tempdir
SIEVE_NO_UPDATE=1                   # 禁止向 updates.sieveai.dev 发送 manifest 请求
SIEVE_NO_TELEMETRY=1                # 禁止发送 uid（防止遥测污染统计数据）
```

**IPC socket 无法 bind 的测试场景**：不要依赖「不存在的路径让 UnixListener::bind 失败」——`AuditStore::init` 会自动创建父目录，绕过这一假设。正确做法是在 tempdir 中预先创建 `ipc.sock` 为**目录**（而非文件），使 `UnixListener::bind` 触发 `EISDIR` 返回 `IpcServer = None`。

```rust
// 正确：预占 ipc.sock 为目录 → EISDIR
let ipc_sock = sieve_home.join("ipc.sock");
std::fs::create_dir_all(&ipc_sock)?;

// 错误：不存在路径会被 AuditStore::init 自动创建父目录
```

注意：`#[tokio::test]` current_thread runtime 上不要使用 `std::io::Read::read` 阻塞 mock upstream 的 accept loop——会阻断 daemon 等待上游响应，测试永远不完成。改用 HTTP-level probe 时已实测此陷阱，相关 trade-off 注释见 `crates/sieve-cli/tests/outbound_block.rs`。

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- Cursor 项目规则：[../../.cursorrules](../../.cursorrules)
- 当前活动 PRD 入口（指针）：../requirements/PRD-sieve.md（v2.0 已锁定执行）
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 部署指南：[deployment.md](deployment.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)
- 架构文档：`../design/architecture.md`（待写）
- 数据模型：`../design/data-model.md`（待写）

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。开发流程任何变更必须同步更新本文与 [CHANGELOG](../changelog/CHANGELOG.md)。
