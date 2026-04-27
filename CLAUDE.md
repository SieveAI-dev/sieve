# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## 项目一句话

Sieve 是一个完全本地运行的 LLM 流量代理（Rust 单二进制），夹在 Claude Code 和 Anthropic API 之间，对 crypto 开发者做双向安全检测，在不可逆动作（签名 / 转账 / 部署）前强制插入认知摩擦。

## 项目状态

**Week 1 工程启动（2026-04-27）**——Cargo workspace + 三 crate 骨架已建立，透传 daemon 可运行，sigstore CI pipeline 已就绪。规则匹配引擎 Week 2 起实现。源码在 `crates/` 下，文档仍是 source of truth，代码反映文档。

---

## Source of Truth 层级

文档冲突时按以下优先级裁决（高 → 低）：

1. **PRD v1.3** — [docs/prd/sieve-prd-v1.3.md](docs/prd/sieve-prd-v1.3.md)（已锁定执行，唯一权威源）
2. **ADR** — [docs/design/ADR-INDEX.md](docs/design/ADR-INDEX.md)（8 个已接受 + 候选清单 ADR-008/009/010）
3. **架构 / 数据模型** — [docs/design/architecture.md](docs/design/architecture.md) + [docs/design/data-model.md](docs/design/data-model.md)
4. **API / 部署 / 开发指南** — `docs/api/` + `docs/guides/`
5. **README + .cursorrules** — 项目入口与代码规范

约束：

- `docs/requirements/PRD-sieve.md` 是 v1.3 的薄指针，不复制全文
- `docs/prd/` 下 v1.0~v1.2 是历史归档，**永不修改**
- 术语首次出现先去 [docs/glossary.md](docs/glossary.md) 加条目，再在 PRD/ADR 引用

---

## 不可放宽的硬约束（PRD §9 十条 / .cursorrules §二）

任何 PR / 设计变更触碰以下任一条，**默认拒绝**，必须先和用户显式确认才能放宽：

1. **Rust 栈非选项** —— Go regexp 慢 1000 倍；hot loop 不允许引入非 Rust 二进制依赖
2. **绝不联网做 verifier** —— 任何外部 token / 签名 / 规则的远端校验都摧毁产品定位
3. **fail-closed High-Risk Tool Policy Gate** —— 签名 / shell / 敏感路径的 Critical 工具调用强制人工确认，**YOLO mode 不可关**
4. **BIP39 必须做 SHA-256 checksum 验证** —— 仅词表匹配不足以定级 Critical（Sieve 差异化点）
5. **SSE 边界处理 fuzz test 全覆盖** —— 半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流，**PR 不带 fuzz 不合并**
6. **自身供应链 sigstore + reproducible build + pinned deps** —— Tier 1（macOS / Linux）Week 1 起强制；Windows 见 [ADR-006 Tier 2](docs/design/ADR-006-sigstore-reproducible-build.md) 说明
7. **Critical 拦截 FP < 0.5%** —— 公理 12，超过即用户禁用产品
8. **Critical 在所有版本（含降级模式）不可关闭** —— 产品安全承诺，不是用户偏好
9. **Phase 1 仅适配 Claude Code** —— UnifiedMessage 接口预留但**不实现** OpenAI / Gemini / OpenRouter
10. **Week 12 GA 一次性公开 repo + 代码 + 文档** —— GA 前 repo 完全私有（见 ADR-011）；sigstore CI pipeline 照常跑通

---

## 三个 Crate（Week 1 落地后强制）

| crate | 职责 | 禁做 |
|------|------|-----|
| `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder | 任何 CLI / TUI / 配置加载 |
| `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 | 任何网络 IO |
| `sieve-cli` | 入口 / 配置 / 弹窗 / 审计日志（SQLite） | 直接做规则匹配 |

跨 crate 调用走显式 trait，避免互相 import 实现细节。详见 [.cursorrules §3.3](.cursorrules)。

---

## Rust 工具链与规范（开始写代码后强制）

全局 CLAUDE.md 只覆盖 Python/JS/Shell，Rust 规范全部在本文 + .cursorrules：

- **Lint**：CI 跑 `cargo fmt --check` + `cargo clippy --all-targets --all-features -- -D warnings`，警告即错误
- **unsafe**：禁用，除非完整 invariant 注释 + 单元测试覆盖；vectorscan FFI 边界封装在专门 crate 内部
- **错误类型**：库 crate 用 `thiserror`；**`anyhow` 仅允许在二进制层 `sieve-cli`**
- **async**：`tokio`，不混 async-std；阻塞 IO 走 `tokio::task::spawn_blocking`
- **panic**：禁 `.unwrap()` / `.expect()` 出现在 hot path 与请求处理路径；测试代码可放宽
- **注释**：只解释 why 和非显然的 invariant / trade-off；公开 API 必须 `///` rustdoc 关联到 PRD 章节或 ADR 编号
- **`#[allow(...)]`**：禁止兜底掩盖 lint，必须修；确需 allow 时附 SAFETY/TODO 注释

完整规范见 [.cursorrules §3](.cursorrules)。

---

## 提交前自检（沿用 .cursorrules §五）

- [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` 通过
- [ ] 涉及 SSE / 规则 / 工具调用判定的改动有对应 fuzz / 单元测试
- [ ] PRD §9 十条硬约束未被绕过
- [ ] CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- [ ] 关联文档（requirements / design / api）已同步
- [ ] 临时文档（`_temp-` / `_draft-`）已清理或归档

---

## 文档更新工作流（项目特有派生关系）

任何代码 / 设计改动后必须按下表更新文档（详细规则见 [PRD-sieve.md "上游变更触发的下游更新清单"](docs/requirements/PRD-sieve.md)）：

| 改动类型 | 必须更新 | 标记 |
|---------|---------|------|
| 新增 / 删除检测项（OUT-* / IN-CR-* / IN-GEN-*） | user-stories + architecture + api-reference + CHANGELOG | P0 |
| 检测项 FP 上限调整 | architecture（误报率预算章）+ CHANGELOG | P0 |
| 检测项处置等级变化（如 Medium → Critical） | user-stories（验收标准）+ ADR + CHANGELOG `[BREAKING]` | P0 |
| 新增上游协议适配（Phase 2 OpenAI 等） | architecture + 新 ADR + api-reference + development | P0 |
| Pipeline 节点增删 / 顺序调整 | architecture（架构图）+ CHANGELOG | P0 |
| 性能预算调整（P99 / 内存 / 二进制大小） | architecture §性能预算 + development（benchmark 命令） | P0 |
| crate 边界变化 | .cursorrules §3.3 + architecture | P0 |
| 定价 / 试用 / 降级模式 | README + user-stories US-12 + CHANGELOG | P1 |
| 法律实体 / 渠道策略 | README + ADR-005 + CHANGELOG | P1 |
| 工程硬约束变化（PRD §9 十条） | PRD-sieve 版本演进表 + ADR + .cursorrules §二 + CHANGELOG `[BREAKING]` | P0 |
| 配置 / `config.toml` schema 变化 | api-reference §3 + deployment + CHANGELOG | P1 |
| 依赖升级（vectorscan / rustls 等） | CHANGELOG | P2 |

无需文档化：纯代码格式化、注释优化、测试补充（无功能变更）。

---

## 何时写 ADR / SPEC

**写 ADR**——决策影响以下任一项时（编号见 [ADR-INDEX](docs/design/ADR-INDEX.md)）：

- 技术栈 / 关键依赖（如换掉 vectorscan）
- 协议层适配（如 Phase 2 接 OpenAI）
- 检测项处置等级变化（Medium → Critical 等）
- 信任边界（如新增任何上游 verify、改 fail-closed 行为）
- 商业 / 法律主体变化

候选 ADR 已登记在 INDEX：ADR-008（426 状态码）、ADR-009（Windows 服务）、ADR-010（加密支付通道）。

**写 SPEC**（`docs/specs/` 当前为空，按需新建）——具体检测算法落地需要工程级规格时（如 BIP39 SHA-256 状态机、地址替换 Levenshtein 算法、SSE 流式 vectorscan 状态机）。Phase 2 功能 SPEC 暂不写——不为想象用户写代码。

---

## 12 周里程碑

执行视图见 [tasks/roadmap.md](tasks/roadmap.md)，每周完成定义跟 PRD §10 同步，本文不重复。

**Week 1 关键路径**（必须并行启动，否则 12 周里程碑会延期）：

1. Rust workspace 骨架（三个 crate）+ hyper/tokio/rustls 跑通透明转发 Anthropic Messages API
2. UnifiedMessage 内部 schema（Anthropic only，三家接口预留）
3. **sigstore + reproducible build pipeline 必须本周跑通**（[ADR-006 §4](docs/design/ADR-006-sigstore-reproducible-build.md)，Tier 1 hard gate；这是 PRD §1.2 第 4 句"自证清白"的物质基础）
4. **[redacted]启动**（[ADR-005](docs/design/ADR-005-overseas-legal-entity.md)，[redacted]才能拿到执照，Week 7-8 Stripe 接入需要）
5. ~~GitHub repo 公开~~（被 ADR-011 撤销；repo 保持私有至 Week 12 GA）

---

## 常用命令

Week 1 起 Cargo workspace 已就绪，日常命令：

```bash
# 构建
cargo build --workspace --locked

# Lint（CI 强制通过）
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo deny check

# 测试
cargo test --workspace --locked
cargo test -p sieve-core --locked   # 跑单个测试

# 启动透传 daemon
RUST_LOG=info cargo run -p sieve-cli -- start --config sieve.toml

# Dry-run 模式（仅记录命中，不拦截）
RUST_LOG=info cargo run -p sieve-cli -- start --config sieve.toml --dry-run

# Reproducible build（本地复现）
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) cargo build --release --locked

# Fuzz（Week 3 起，SSE Parser 加入后）
cargo +nightly fuzz run sse_parser

# Benchmark（Week 4 起，验证 P99 < 20ms）
cargo bench -p sieve-rules
```

详细命令清单与系统依赖见 [docs/guides/development.md](docs/guides/development.md)。

---

## [redacted]约束

- **doskey 一人 + Claude Code**，不要假设有团队 / DevRel / 销售 / 数据标注师
- **[redacted] + Stripe + [redacted]**，[redacted]（[ADR-005](docs/design/ADR-005-overseas-legal-entity.md)）
- **[redacted] to-C 公开商业化营销**——Twitter / Hacker News / Mirror 是主战场，微信 / 小红书 / 知乎 / B 站不规划（[PRD §11.5.2 渠道分级](docs/prd/sieve-prd-v1.3.md#1152-营销渠道分级)）
- 18 个月 [redacted] ≥ [redacted]，24 个月 [redacted]——**不追独角兽，不融资，不招人**

---

## 全局规则继承

用户级 [~/.claude/CLAUDE.md](~/.claude/CLAUDE.md) 已设定通用规则：沟通风格、工作流（规划 / 验证 / Bug 修复 / 经验沉淀）、子代理调度策略、Python/JS/Shell 规范、Git 提交规范（含禁止 AI 签名）、文档管理 v2.0 + DOCS-STANDARD 引用、RTK 命令代理。

**本文只补 Sieve 项目特有约束，不重复全局内容。** 如冲突，按 .cursorrules 与 PRD 优先；若 PRD 明确允许放宽全局规则的某条，必须在 ADR 中记录。
