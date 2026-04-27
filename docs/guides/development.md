# Sieve 开发指南

> **状态：Week 1 工程启动（2026-04-27）。**
> Cargo workspace 已就绪，透传 daemon 可运行，本文命令均为生效命令。

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

| 平台 | 支持级别 | 说明 |
|------|---------|------|
| macOS 14+（arm64 / x86_64） | **Tier 1** | doskey 主战场 |
| Ubuntu 22.04+（x86_64 / arm64） | Tier 1 | dogfood 验证 |
| Debian 12+ | Tier 1 | 同 Ubuntu |
| Windows 11 | **Tier 2，Phase 1 后期补充** | 暂不阻塞 GA |
| 其他 Linux 发行版 | 社区维护 | 仅二进制可用，issue 走 best-effort |

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
│   └── sieve-cli/                  # 主二进制 + 配置 + 弹窗 + 审计 SQLite + 管理 API
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

### crate 边界（[`.cursorrules` §3.3](../../.cursorrules)）

| crate | 职责 | 不允许做 |
|-------|------|---------|
| `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
| `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
| `sieve-cli` | 入口 / 配置 / 弹窗 / 审计 SQLite / 管理 API | 直接做规则匹配（必须经 `sieve-rules`） |

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

# 指定 target（macOS aarch64 / x86_64 / Linux musl）
cargo build --release --locked --target aarch64-apple-darwin -p sieve-cli
cargo build --release --locked --target x86_64-apple-darwin -p sieve-cli
cargo build --release --locked --target x86_64-unknown-linux-musl -p sieve-cli
```

### 3.3 测试 / Lint / 审计

**PR 必须全绿才能合并**（参见 §8）：

```bash
cargo fmt --all -- --check                                              # 格式
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  # 警告即错误
cargo test --workspace --locked                                         # 全部单元 + 集成
cargo deny check                                                        # 许可证 / 来源 / 重复依赖审计
```

### 3.4 启动 daemon（Week 1 透传）

```bash
# 1. 写最小 config
cat > /tmp/sieve.toml <<EOF
upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
EOF

# 2. 启动
RUST_LOG=info cargo run -p sieve-cli -- start --config /tmp/sieve.toml

# 3. 用 Claude Code 接入（另一个终端）
ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude -p "hello"
```

### 3.5 系统依赖（macOS）

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# vectorscan-rs 编译依赖（Week 2 起实际使用）
brew install cmake ninja pkg-config boost ragel
```

---

## 4. SSE Fuzz 测试（[PRD §9 硬约束 #5](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）

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

[PRD §6.4](../prd/sieve-prd-v1.3.md#64-性能预算) 性能预算：

| 操作 | 目标延迟 |
|------|---------|
| 普通 token 流式 chunk | +30-200 µs |
| 工具调用边界完整检查 | +5-15 ms |
| 整体 P99 添加延迟 | **< 20 ms** |
| 内存峰值 | < 100 MB |
| 二进制大小 | < 20 MB 单文件 |
| 启动时间 | < 500 ms |

CI 用 [`criterion`](https://github.com/bheisler/criterion.rs) 跑核心 benchmark，**任何 PR 让 P99 超过 20ms → 自动 block 合并**。

### 5.2 命令

```bash
cargo bench --bench latency                # 主 latency benchmark
cargo bench --bench latency -- --save-baseline main   # 保 baseline
cargo bench --bench latency -- --baseline main        # 与 baseline 对比

# 单个 benchmark
cargo bench --bench latency -- sse_parse_chunk
```

### 5.3 数据集

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

1. **Severity + FP 上限承诺** —— 引用 [PRD §6.5 误报率预算](../prd/sieve-prd-v1.3.md#65-误报率预算)；如想申报 Critical，须明确 `< 0.5%` 或更严格
2. **≥ 5 条 benign 测试用例** —— 真实场景采样，**不接受人造**（如 README 中常见的 `0xDEADBEEF`）
3. **≥ 5 条 malicious 测试用例** —— 至少 2 条来自公开 incident（链上、安全播报、UCSB 论文等），**附 source URL**
4. **误报治理策略** —— 占位符黑名单关键字、上下文关键词、entropy 阈值依据
5. **回滚方案** —— 规则上线后 FP 失控如何快速 disable：通常通过 `severity_overrides` 降级或下一个签名规则包覆盖
6. **基准跑分** —— 在新规则下 §7.1 benchmark 数据集的 FP / Recall 数据

### 6.3 命中算法层级（强烈建议）

按以下顺序短路，**最便宜的过滤器先跑**：

1. **前缀字符串匹配**（μs 级，vectorscan SIMD）
2. **正则 / 词表匹配**
3. **校验位 / entropy / SHA-256**（如 [BIP39 SHA-256 校验](../prd/sieve-prd-v1.3.md#5-功能需求)，PRD §9 硬约束 #4）
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

## 8. 本地 dogfood 流程

每个开发者（Phase 1 = doskey）**必须** 100% 时间用 Sieve 工作：

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
- 每周末汇总 FP 计数，超阈值（详见 [PRD §6.5](../prd/sieve-prd-v1.3.md#65-误报率预算)）必须当周修

---

## 9. 提交 PR 流程

### 9.1 沟通节奏

```
1. 先开 issue：描述问题 / 方案 / 代码改动范围
   ↓
2. 等 doskey 回复确认方向（避免无效 PR）
   ↓
3. fork → branch（命名建议 `feat/<scope>` / `fix/<scope>` / `rule/<id>`）
   ↓
4. 写代码 + 测试 + 文档
   ↓
5. 提 PR（关联 issue 编号，带 §9.2 自检清单）
   ↓
6. CI 全绿 → reviewer 走查 → 合并
```

### 9.2 PR 自检清单（提交前必跑）

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
- §9.3 文档同步清单
- 是否触及 [PRD §9 硬约束](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)（默认不允许绕过）

### 9.3 文档同步要求

参考 [`.cursorrules` §1.5 文档化触发条件](../../.cursorrules) 和用户级文档规则中的"修改功能 Checklist"：

- [ ] 阅读现有需求文档，确认变更范围
- [ ] 阅读现有设计文档，评估影响
- [ ] 更新所有涉及的文档（用 `rg` 搜索关联引用）
- [ ] 在 [`CHANGELOG.md`](../changelog/CHANGELOG.md) 记录变更**原因**（不是 what，是 why）
- [ ] 检查 [README.md](../../README.md) 描述是否仍然准确
- [ ] 修改 API 时必须同步 [`docs/api/api-reference.md`](../api/api-reference.md)
- [ ] 架构变更必须有 `docs/design/ADR-*.md`

### 9.4 CI Gate 清单

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

### 9.5 Reviewer

**Phase 1 唯一 reviewer：doskey。**单人 review 不是优势，是不得已。请在 PR 描述中详尽列出测试场景，降低 review cost。

### 9.6 安全披露

**不要在公开 issue 公布 0day**。流程：

1. 邮件到 `security@<域名待定>`（GA 前用 `doskey-sieve@<待定>`），**附 PGP 公钥加密**（指纹待定，发布于 README）
2. 等待 doskey 回复确认（48h 内）
3. 协商修复时间窗与公开披露日期
4. 修复合并后再写 advisory（GitHub Security Advisory）

---

## 10. 文档同步要求（再强调）

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

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- Cursor 项目规则：[../../.cursorrules](../../.cursorrules)
- 当前活动 PRD：[../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 部署指南：[deployment.md](deployment.md)
- 变更日志：[../changelog/CHANGELOG.md](../changelog/CHANGELOG.md)
- 架构文档：`../design/architecture.md`（待写）
- 数据模型：`../design/data-model.md`（待写）

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。开发流程任何变更必须同步更新本文与 [CHANGELOG](../changelog/CHANGELOG.md)。
