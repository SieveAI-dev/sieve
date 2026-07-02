# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## 项目一句话

Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），夹在 AI 编码 agent（Claude Code / OpenClaw / Hermes / Codex CLI）和上游模型 API（Anthropic / OpenAI / 中转站）之间，对 crypto 开发者做双向安全检测，在不可逆动作（签名 / 转账 / 部署）前强制插入认知摩擦。

## 项目状态

**早期预览（0.1.0-alpha）**。

能力面：双向检测（出站脱敏自动改写不弹窗 + 入站 fail-closed 拦截 Critical 工具调用）、四路由内容类型对等覆盖（Anthropic/OpenAI × SSE/JSON）、出站 exfil 链与 canary 诱饵检测、WIF/xprv 私钥格式检测、配置化路由表 `[[upstream.routes]]`、ProviderCodec 协议分层、四家 agent 接入。Cargo workspace 8 个 crate；sigstore + 可复现构建 CI 在跑。daemon ↔ client 走 IPC（JSON-RPC over Unix socket，协议版本白名单仅 v2，权威源 SPEC-005）。源码在 `crates/` 下。

---

## Source of Truth 层级

文档冲突时按以下优先级裁决（高 → 低）：

1. **IPC 协议规格** — [docs/specs/SPEC-005-ipc-protocol.md](docs/specs/SPEC-005-ipc-protocol.md)（IPC wire schema 唯一权威源）+ 其余 [docs/specs/](docs/specs/) 功能规格
2. **架构 / 数据模型** — [docs/design/architecture.md](docs/design/architecture.md) + [docs/design/data-model.md](docs/design/data-model.md)
3. **API / 部署 / 开发指南** — `docs/api/` + `docs/guides/`
4. **README + .cursorrules** — 项目入口与代码规范

约束：

- 改 IPC 字段先改 SPEC-005，再落代码
- 术语首次出现先去 [docs/glossary.md](docs/glossary.md) 加条目，再在 SPEC 引用

---

## 不可放宽的硬约束（十六条 / .cursorrules §二）

任何 PR / 设计变更触碰以下任一条，**默认拒绝**，必须先和用户显式确认才能放宽：

1. **Rust 栈非选项** —— Go regexp 慢 1000 倍；hot loop 不允许引入非 Rust 二进制依赖
2. **绝不联网做 verifier** —— 任何外部 token / 签名 / 规则的远端校验都摧毁产品定位。**唯一允许的出站**：①上游 LLM API（用户主动调）②`updates.sieveai.dev` manifest（带 `?v=&os=&arch=&uid=&ch=` 5 字段,可关）③`cdn.sieveai.dev` 规则正文（更新通道协议）
3. **fail-closed High-Risk Tool Policy Gate** —— 签名 / shell / 敏感路径的 Critical 工具调用强制人工确认，**YOLO mode 不可关**
4. **BIP39 必须做 SHA-256 checksum 验证** —— 仅词表匹配不足以定级 Critical（Sieve 差异化点）
5. **SSE 边界处理 fuzz test 全覆盖** —— 半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流，**PR 不带 fuzz 不合并**
6. **自身供应链 sigstore + reproducible build + pinned deps** —— Tier 1（macOS / Linux）Week 1 起强制；Windows 为 Tier 2
7. **Critical 拦截 FP < 0.5%** —— 超过即用户禁用产品
8. **Critical 在所有版本（含降级模式）不可关闭** —— 产品安全承诺，不是用户偏好
9. **适配四家 agent：Claude Code / OpenClaw / Hermes / Codex CLI** —— UnifiedMessage 真实运行时支持 Anthropic Messages API + OpenAI Chat Completions 双协议；Codex CLI 走原生 PreToolUse hook（`~/.codex/hooks.json`）经 IPC judge_tool_call 取裁决，OpenClaw / Hermes 两家 hook 作 UX 层、安全不变量由网关 inbound_hold 兜底 fail-closed；其他协议（Gemini / Mistral 等）推 Phase 2
10. **GA 一次性公开 repo + 代码 + 文档** —— sigstore CI pipeline 照常跑通
11. **不在 Anthropic API 协议层撒谎** —— 不伪造 tool_use / stop_reason / id / usage / type；拦截发生时允许截 SSE 流注入 `sieve_blocked` event（Sieve 自报事件，不是冒充模型）；keep-alive comment 行 `: keep-alive\n\n` 不属于伪造
12. **不装本地 CA 做 MITM** —— Network Extension / 本地 CA 注入 / 系统 proxy 修改推 Phase 3 选购，Phase 1/2 不做
13. **出站脱敏不打断工作流** —— OUT-01~05/12 高频脱敏类必须自动脱敏 + 状态栏 5s 通知，不弹窗；每天弹几十次的产品没人用
14. **用户规则系统 fail-safe**（v2.0）—— 用户规则文件加载失败 / pattern 编译失败 / 安全 lint 拒绝 → daemon 必须正常启动 + 系统规则全功能；用户规则不能 override 或 suppress 系统 Critical；用户规则只能 High Ask/Warn/Mark，不能 Block / HookTerminal。详见 LayeredEngine 合并顺序
15. **行为序列检测保守起步 + 默认关闭**（v2.0）—— IN-SEQ-* 仅触发 StatusBar 通知，不引入新 Block 路径；默认关闭（feature flag `sequence_detection = false`），用户主动 opt-in。升级为 Block 类需充分样本积累 + FP < 0.5% + 新决策记录。
16. **所有入站能力必须经过 content-type 路由矩阵测试**（v2.0）—— 任何新增入站功能必须有集成测试覆盖 4 类组合（Anthropic SSE / Anthropic JSON / OpenAI SSE / OpenAI stream=false JSON）；新功能只挂 SSE 不挂 JSON 视为 P0 漏洞。

---

## 八个 Crate

| crate | 职责 | 禁做 |
|------|------|-----|
| `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
| `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
| `sieve-cli` | 入口 / 配置 / 弹窗 / 审计日志（SQLite） | 直接做规则匹配 |
| `sieve-ipc` | IPC 协议（JSON-RPC over Unix socket + 文件锁 + pending/decisions 文件 IO） | 不参与请求处理 / 不依赖 sieve-core 业务逻辑 |
| `sieve-hook` | 极简 PreToolUse hook 二进制，启动时延 < 50ms，依赖只有 `serde_json` + `fd-lock` + `uuid` + `chrono` + `clap` | 不依赖 sieve-core / sieve-rules / sieve-ipc / tokio |
| `sieve-policy` | 用户规则加载（user.toml）/ lint（11 类约束）/ 与系统规则合并（LayeredEngine）/ 灰名单管理 / 规则编辑器 pipeline | 不直接做匹配（调 sieve-rules）；不直接处理网络 IO；不能 suppress 系统 Critical |
| `sieve-updater` | manifest 协议客户端 / install-id 生成 / 定时器 / ed25519 + sha256 校验 / 三个 env var 解析 | 不参与请求处理 / 不依赖 sieve-core 业务逻辑 / 不做规则文件原子替换（属于 sieve-rules） |
| `sieve-testing` | 集成测试 harness（DaemonGuard / mock upstream / e2e helper），`publish=false` | 不进生产二进制；仅供测试依赖 |

跨 crate 调用走显式 trait，避免互相 import 实现细节。详见 [.cursorrules §3.3](.cursorrules)。

> Native GUI App 在独立仓库 `sieve-gui-macos`，不在本 workspace。

---

## Rust 工具链与规范（开始写代码后强制）

全局 CLAUDE.md 只覆盖 Python/JS/Shell，Rust 规范全部在本文 + .cursorrules：

- **Lint**：CI 跑 `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings`（外加 sieve-cli 带 `sequence_detection` 特性单独一步），警告即错误
- **unsafe**：禁用，除非完整 invariant 注释 + 单元测试覆盖；vectorscan FFI 边界封装在专门 crate 内部
- **错误类型**：库 crate 用 `thiserror`；**`anyhow` 仅允许在二进制层 `sieve-cli`**
- **async**：`tokio`，不混 async-std；阻塞 IO 走 `tokio::task::spawn_blocking`
- **panic**：禁 `.unwrap()` / `.expect()` 出现在 hot path 与请求处理路径；测试代码可放宽
- **注释**：只解释 why 和非显然的 invariant / trade-off；公开 API 必须 `///` rustdoc 关联到相关 SPEC
- **`#[allow(...)]`**：禁止兜底掩盖 lint，必须修；确需 allow 时附 SAFETY/TODO 注释

完整规范见 [.cursorrules §3](.cursorrules)。

---

## 提交前自检（沿用 .cursorrules §五）

- [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` 通过
- [ ] 涉及 SSE / 规则 / 工具调用判定的改动有对应 fuzz / 单元测试
- [ ] 十六条工程硬约束（.cursorrules §二）未被绕过
- [ ] CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- [ ] 关联文档（requirements / design / api）已同步
- [ ] 临时文档（`_temp-` / `_draft-`）已清理或归档

---

## 文档更新工作流（项目特有派生关系）

任何代码 / 设计改动后必须按下表更新文档：

| 改动类型 | 必须更新 | 标记 |
|---------|---------|------|
| 新增 / 删除检测项（OUT-* / IN-CR-* / IN-GEN-*） | architecture + api-reference + CHANGELOG | P0 |
| 检测项 FP 上限调整 | architecture（误报率预算章）+ CHANGELOG | P0 |
| 检测项处置等级变化（如 Medium → Critical） | architecture + CHANGELOG `[BREAKING]` | P0 |
| 新增上游协议适配（Phase 2 等） | architecture + api-reference + development | P0 |
| Pipeline 节点增删 / 顺序调整 | architecture（架构图）+ CHANGELOG | P0 |
| 性能预算调整（P99 / 内存 / 二进制大小） | architecture §性能预算 + development（benchmark 命令） | P0 |
| crate 边界变化 | .cursorrules §3.3 + architecture | P0 |
| 工程硬约束变化（十六条） | .cursorrules §二 + CHANGELOG `[BREAKING]` | P0 |
| 配置 / `config.toml` schema 变化 | api-reference §3 + deployment + CHANGELOG | P1 |
| 依赖升级（vectorscan / rustls 等） | CHANGELOG | P2 |

无需文档化：纯代码格式化、注释优化、测试补充（无功能变更）。

---

## 何时写 SPEC

**写 SPEC**（[docs/specs/](docs/specs/)）——具体检测算法或协议落地需要工程级规格时（如 BIP39 SHA-256 状态机、地址替换近似算法、SSE 流式 vectorscan 状态机、multi-agent setup 配置注入、IPC wire schema）。Phase 2 功能 SPEC 暂不写——不为想象用户写代码。

> 公开仓不含架构决策记录文件、也不引用其编号；需说明架构时用自包含的功能描述或指向公开 SPEC。

---

## 关键能力路径（已落地）

- **透明转发 + UnifiedMessage**：hyper/tokio/rustls 跑通 Anthropic Messages API + OpenAI Chat Completions 双协议透明转发，内部统一 schema。
- **sigstore + 可复现构建 pipeline**（Tier 1 hard gate）。
- **`sieve-ipc` + `sieve-hook`**：IPC server JSON-RPC + 文件锁协议，hook 启动时延 < 50ms。
- **`sieve setup` 一键安装**：检测 agent → 注册 PreToolUse hook + 写 base_url → 加载 launchd plist（[SPEC-003](docs/specs/SPEC-003-sieve-setup-tool.md)）。
- **出站自动脱敏路径**：高频脱敏类命中后改写 body bytes，不弹窗，状态栏 5s 通知。
- **入站双层防御**：Hook 类写 IPC pending file 不修改 SSE 流；GUI 类 hold 流 + keep-alive comment + 用户确认后处置。

---

## 常用命令

Week 1 起 Cargo workspace 已就绪，日常命令：

```bash
# 构建
cargo build --workspace --locked

# Lint（CI 强制通过；clippy 分两步，避免 --all-features 触发编译期密钥 gate）
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p sieve-cli --all-targets --features sequence_detection --locked -- -D warnings
cargo deny check

# 测试（nextest profile ci 带 retries 抗并发 flake；doctest 单列，nextest 不覆盖）
cargo nextest run --workspace --locked --profile ci
cargo test --workspace --locked --doc
cargo test -p sieve-core --locked   # 跑单个测试

# 四路由覆盖门禁（check-routing-matrix job；纯静态扫描，无需构建）
bash scripts/check_routing_coverage.sh

# Hermetic e2e + FP/recall 数据集门（一键 dogfood）
bash scripts/dogfood.sh

# 启动透传 daemon（日志走 SIEVE_LOG）
cargo run -p sieve-cli -- start --config sieve.toml

# Dry-run 模式（仅记录命中，不拦截）
cargo run -p sieve-cli -- start --config sieve.toml --dry-run

# Reproducible build（本地复现，入口脚本含 -j1 串行硬约束，不能并行）
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) ./scripts/repro-build.sh macos-arm64

# Fuzz（SSE 边界全覆盖，PR 不带 fuzz 不合并；-s none 避 cargo-fuzz#404 sanitizer 冲突，CI 四 target 全跑）
cargo +nightly fuzz run -s none sse_parser -- -max_total_time=60
cargo +nightly fuzz run -s none sse_parser_openai -- -max_total_time=60
cargo +nightly fuzz run -s none tool_use_aggregator -- -max_total_time=60
cargo +nightly fuzz run -s none inbound_filter -- -max_total_time=60

# Benchmark（Week 4 起，验证 P99 < 20ms）
cargo bench -p sieve-rules

# 可选特性门（默认关，CLI 瘦身）：usage / audit-crypto 默认不编入主二进制
cargo build -p sieve-cli --bins                          # 默认能力面：start / decisions / rules / audit(查询) / setup / doctor / uninstall / version
cargo build -p sieve-cli --features usage --bins         # +token 用量观测（usage 子命令）
cargo build -p sieve-cli --features audit-crypto --bins  # +加密审计归档（audit keygen / rotate-key / decrypt）
# 红队回归是开发 / CI 工具，已移出主二进制：用 cargo test 而非 sieve 子命令
cargo test -p sieve-cli --test redteam_inbound --test redteam_outbound --locked
```

详细命令清单与系统依赖见 [docs/guides/development.md](docs/guides/development.md)。

---

## 全局规则继承

用户级 [~/.claude/CLAUDE.md](~/.claude/CLAUDE.md) 已设定通用规则：沟通风格、工作流（规划 / 验证 / Bug 修复 / 经验沉淀）、子代理调度策略、Python/JS/Shell 规范、Git 提交规范（含禁止 AI 签名）、文档管理 v2.0 + DOCS-STANDARD 引用。

**本文只补 Sieve 项目特有约束，不重复全局内容。** 如冲突，按 .cursorrules 优先。
