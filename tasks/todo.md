# IN-CR-03 敏感路径访问 · 实施计划

> Week 4 第一项。关联 PRD §5.2 入站检测目标 / [roadmap Week 4](./roadmap.md)。
> 创建：2026-04-27

---

## 范围

入站规则：当模型生成的工具调用（`Read` / `Bash` / `Edit` / `Glob` 等）入参中
出现敏感路径关键词时，触发 **High severity warn 5s**（PRD §5.2，非 Critical block——
合法用例存在，给用户判断空间）。

**复用既有架构**：IN-CR-02 已通过 `engine_adapter.rs:check_tool_use()` 把
`tool.input` 序列化成 JSON 后喂给 vectorscan。新规则只需加 toml 条目，
扫描器无需改动。

## 不做

- 不动 Forwarder / SSE Parser / Tool Use Aggregator
- 不实现 IN-CR-04（持久化机制）—— 下一项
- 不动出站规则、不动 IN-CR-01/02/05 已有规则
- 不引入新 crate / 新依赖

## 检测项清单（8 条 IN-CR-03-*）

| 子规则 ID | 路径模式 | severity | action | 关键 allowlist |
|---|---|---|---|---|
| IN-CR-03-SSH-PRIVATE | `id_rsa` / `id_ed25519` / `id_ecdsa` / `id_dsa` 文件名 | high | warn | `*.pub`（公钥） |
| IN-CR-03-SSH-DIR | `~/.ssh/...` 路径 | high | warn | `known_hosts` / `authorized_keys` / `config` / `environment` |
| IN-CR-03-AWS-CREDS | `~/.aws/credentials` 或 `.aws/credentials` | high | warn | （`.aws/config` 不拦：仅 region/profile） |
| IN-CR-03-DOTENV | `.env` / `.env.local` / `.env.production` 等 | high | warn | `.env.example` / `.env.template` / `.env.sample` / `.env.test` / `.env.dist` / `.env.ci` |
| IN-CR-03-ETH-KEYSTORE | geth keystore 文件名 `UTC--<ts>--<40hex>` | high | warn | — |
| IN-CR-03-GPG-DIR | `~/.gnupg/...` | high | warn | — |
| IN-CR-03-NETRC | `.netrc` | high | warn | — |
| IN-CR-03-MACOS-KEYCHAIN | `login.keychain-db` / `System.keychain` | high | warn | — |

**Vectorscan PCRE 子集合规检查**：所有 pattern 仅用 `(?:...)` / `(?i)` /
字符类 / 量词，无 lookahead / lookbehind / 反向引用 / 原子组。

## 文件清单

- 改：`crates/sieve-rules/rules/inbound.toml`（追加 8 条规则）
- 改：`crates/sieve-rules/tests/inbound_rules.rs`（每条规则正例 + allowlist 反例）
- 改：`crates/sieve-cli/tests/inbound_block.rs`（端到端：SSE 流中 `Read` 工具
  调用入参为 `~/.ssh/id_rsa` → 期望 1 条 IN-CR-03-* warn detection）
- 不动：所有 src/*.rs（架构已就绪，仅加规则）

## 验收标准

- [ ] `cargo build --workspace --locked` 通过
- [ ] `cargo fmt --all -- --check` 干净
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 干净
- [ ] `cargo test --workspace --locked` 全绿（**新增 ≥ 16 个测试**：8 正 + 8 allowlist 反）
- [ ] 端到端集成测试新增 1 例（inbound_block.rs）
- [ ] vectorscan 编译规则集 0 错误（`InboundFilter::new` 启动不 panic）

## 风险

1. **pattern 兼容性**：geth keystore `UTC--` 时间戳含 `T:` 字符，分隔符要测真机数据
   → 用真实 keystore 文件名样本验证
2. **DOTENV FP**：`.env` 是 vendored doc 极常见词。allowlist 必须覆盖主流
   `.env.*` 后缀，否则项目里的 `*.env.example` 文档检索会持续告警
3. **High warn 5s 的运行时实现**：当前 codebase 是否已实现 5s 倒计时弹窗？
   如未实现则本次仅产出 detection 记录，不阻塞流量；倒计时弹窗在
   Week 5 配置/CLI 工作流中补完。**先确认现状再决定是否需要本次扩展**。

## 步骤

1. [x] 验证当前 `Action::Warn` 在 inbound 流中的实际行为
   → **结论**：High severity 仅 `tracing::warn!` 日志，不阻流量；5s 倒计时弹窗
   留 Week 5。本次仅产出 detection 记录能力，符合切分。
2. [x] 写 10 条规则到 inbound.toml（实施时补 GCP / Solana 完整覆盖 US-07）
3. [x] 写单元测试（rules 层，新增 18 个）
4. [x] 写集成测试（cli 层，1 个端到端 warn-passes-through）
5. [x] 全量 `cargo test --workspace --locked`（176/176）+ `cargo clippy -D warnings` + `cargo fmt --check` 全绿
6. [x] 文档同步：CHANGELOG `[Added]` + roadmap Week 4 勾选项
7. [ ] 提交 commit

## 实施期发现

- **Engine bug**：vectorscan 对带量词 pattern 触发多 endpoint 回调，allowlist 失效。
  在引擎层加 longest-match-per-start dedup 修复，副带改善 OUT-06 / OUT-08。
- **NETRC pattern 错**：`\b\.netrc` 前置 `\b` 在 `~/.netrc` 不命中（`/`+`.` 都是
  非 word 字符无 boundary）。改为仅尾部 `\b`。
- **markdown exfil ID 错位**：现 `id = "IN-CR-04"` 实际是 markdown 通用 exfil，
  应归 IN-GEN-*。本次未改，留 IN-CR-04 持久化 PR 一起 `[BREAKING]` 重命名。
- **UX 未做**：5s 倒计时弹窗 + .sieveignore 加白引导，Week 5 处理（已记 lessons）。

## 完成后

开 IN-CR-04 持久化机制（同套架构，crontab / launchd / systemd / shell rc，
**Critical block** 级别——US-07 写入 = 后门埋点，比读敏感路径风险更高）。
顺便重命名当前 `IN-CR-04` markdown rule → `IN-GEN-04`。
