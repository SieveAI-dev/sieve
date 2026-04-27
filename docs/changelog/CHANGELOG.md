# Changelog

本文件记录 Sieve 所有显著变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

> 当前状态：**设计阶段（Pre-Code）**，尚未发布任何二进制版本。
> 第一个公开版本（v0.1.0）将随 Week 12 GA 发布；详见 [PRD §10 12 周里程碑](../prd/sieve-prd-v1.3.md#10-12-周里程碑8-周-dogfood--4-周闭测)。
> v1 公开 API 在 Week 12 GA 后冻结，破坏性变更走 SemVer。冻结范围参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)。

---

## [Unreleased](https://github.com/doskey/sieve/compare/v0.1.0...HEAD)

### Verified — 2026-04-27 Week 1 完成定义实跑

#### release.yml workflow_dispatch 首跑(run [24980079580](https://github.com/doskey/sieve/actions/runs/24980079580))
- **三 target reproducible build 双 SHA-256 一致 + cosign keyless OIDC 签名 + Rekor 上链 + cosign verify-blob 自验证**:
  - `aarch64-apple-darwin`: `af5c371f1a6531d2a8439425f9d90a5e339fca20a62825b8d895f29c6b883899`
  - `x86_64-apple-darwin`:  `47b729ee298f9dc1d5a3bd0a04f5f30b19983b7c87454b7358442514762164ea`
  - `x86_64-unknown-linux-gnu`: `bbe16fc2faf52a010dd3b3ae172599ec6b7ae9c8cd666c6046d06cfe265065fa`
- 已知遗留:`macos-universal` lipo 合并步骤路径修复(本 commit 含)。**universal binary 不影响 reproducible build pipeline 主路径验证**(三个独立 target 都已成功签名 + 上链)。
- ADR-006 §10 Week 1 hard gate **达成**。

#### 端到端 dogfood 验证(PRD §10.1 Week 1 第 1 完成定义)
- doskey 在真机用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude` 启动 Claude Code v2.1.119(Opus 4.7),非流式聊天测试通过(2026-04-27 14:35 时点)。
- e2e smoke test 脚本(`scripts/smoke_test.py`)真机自验 21/21 通过:401 字节级透传 / 4xx 错误码 / 8KB body / 20 路并发 / 真 key 200 / SSE 流式 / tool_use partial_json。

### Added — Week 1 (2026-04-27 启动)

#### 工程骨架
- Cargo workspace + 三 crate（`sieve-core` / `sieve-rules` / `sieve-cli`），关联 .cursorrules §3.3
- `rust-toolchain.toml` 锁定 1.87.0，targets: aarch64-apple-darwin / x86_64-apple-darwin / x86_64-unknown-linux-musl
- `.cargo/config.toml` reproducible build flags（`--remap-path-prefix` + musl 静态链接），关联 ADR-006
- `deny.toml` cargo-deny 策略（出站 host 白名单 + 许可证白名单 + advisories yanked deny），关联 ADR-003

#### sieve-core（透传层）
- `UnifiedMessage` schema（Anthropic-only 实现，UpstreamProvider::Relay variant 预留），关联 PRD §6.1 / ADR-004
- `AnthropicRequest` 解析（serde 子集，#[serde(flatten)] 兼容未识别字段）
- `Forwarder`：hyper 1.x + hyper-rustls 0.27 + rustls 0.23 + aws-lc-rs provider + webpki-roots，ALPN h2+http/1.1
- SSE passthrough（Week 1 字节透传，Week 3 切到 parser）
- `PipelineNode` trait 占位（Week 2 起 OutboundFilter / InboundFilter 实现）
- 错误类型：`thiserror`，**禁止 anyhow**（.cursorrules §3.2）

#### sieve-rules（占位骨架）
- `MatchEngine` trait + `MatchHit` 数据结构占位（Week 2 起 vectorscan 实现）
- `RulesManifest` schema（rules-vN.manifest.json），关联 data-model.md
- `Ed25519 Verifier`（规则包验签占位，Week 5 起做实际下发）
- `vectorscan-rs 0.0.6` 依赖加入（用于三平台编译验证），关联 ADR-001
- `ed25519-dalek 2.x` 依赖加入

#### sieve-cli（daemon）
- `sieve start --config <path>`：hyper 1.x server 监听 127.0.0.1:11453，反向代理到 api.anthropic.com
- 配置加载：TOML，bind_addr **强制 127.0.0.1** 校验（非 loopback → exit(1)），关联 ADR-003 / PRD §9 #2
- 透传逻辑：headers 剥 Host 重写，body 通过 hyper `Incoming` 流式 chunk-by-chunk（SSE 字节级零缓冲）
- `serde(deny_unknown_fields)`：任何未知配置字段直接拒绝
- `audit` 模块占位（Week 4 接入 SQLite append-only）
- tracing-subscriber 日志（`SIEVE_LOG` 环境变量控制等级）
- **未引入 --disable-critical / --yolo flag**，关联 ADR-007

#### CI / CD（关键 - ADR-006 hard gate）
- `.github/workflows/ci.yml`：fmt / clippy / test（ubuntu + macos-14 矩阵）/ cargo-deny
- `.github/workflows/release.yml`：tag v* 触发，矩阵覆盖三 target，**双构建 SHA-256 比对**，**cosign keyless OIDC 签名**（`id-token: write`），Rekor 透明日志上链，sigstore bundle 上传到 GitHub Release
- macOS universal binary（lipo 合 aarch64 + x86_64）

### Changed — 2026-04-27 PRD §9 #10 修订

- **撤销 "Day 1 GitHub repo 公开 README + 架构文档" 承诺**，见 [ADR-011](../design/ADR-011-private-until-ga.md)
- 新策略：Week 12 GA 时一次性公开 repo + 代码（MIT）+ 文档 + sigstore 验证流程
- 影响范围：repo 保持 private 至 Week 12；Week 1-11 release.yml 不绑定 tag（改为 workflow_dispatch），减少 Rekor 透明日志痕迹
- ADR-006 sigstore + reproducible build CI **不受影响**，GA 前照常跑通；只是不做 public Rekor 验证演示
- 营销弹药 GA 当天集中释放（文章 1+2+3 同步）

### Pending（Week 2 起）
- vectorscan-rs 实际规则编译与 OUT-01~12 出站 P0 规则
- BIP39 SHA-256 checksum 验证（差异化点，PRD §9 #4）
- SSE Parser 完整实现 + fuzz corpus（PRD §9 #5）
- 入站 Crypto 钩子（IN-CR-01~05）

### Known Issues
- 本地需安装 Rust toolchain（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- vectorscan-rs 编译需要系统包 `boost` + `ragel`（macOS：`brew install boost ragel`；Linux：`apt-get install libboost-dev ragel`）
- ADR-008 出站 Critical 状态码（候选，Week 2 dogfood 实测后落 ADR）
- ADR-005 [redacted] Week 1 启动（非工程任务，doskey 跟进）

---

> 以下为 PRD v1.3 设计阶段计划，**尚未实现**。任何条目在实际编码、测试、签名验证完成前不视为已交付。

### 计划中（Phase A dogfood, Week 1-8）

#### 新增

- **W1 基础设施 + Anthropic 协议**
  - Rust 项目骨架（`sieve-core` / `sieve-rules` / `sieve-cli` workspace）
  - `hyper` + `tokio` + `rustls` HTTP 反向代理跑通
  - 透明转发 Anthropic Messages API（`POST /v1/messages` 含 SSE，`POST /v1/messages/count_tokens`，`GET /v1/models`）
  - `UnifiedMessage` 内部 schema（仅 Anthropic 实现，其他 provider 接口预留，[PRD §9 #9](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - ~~GitHub repo 公开~~ — 已被 [ADR-011](../design/ADR-011-private-until-ga.md) 撤销，repo 保持私有至 Week 12 GA
  - **🚨 sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 必须 W1 跑通** —— [PRD §1.2 第 4 句](../prd/sieve-prd-v1.3.md#12-四句话核心叙事v13-加第-4-句) 自证清白叙事的物质基础
- **W2 出站 P0 规则（OUT-01 ~ OUT-12）**
  - OUT-01 OpenAI / Anthropic API key（前缀 + entropy + 占位符黑名单，FP < 0.1%）
  - OUT-02 AWS Access Key（`AKIA[0-9A-Z]{16}` + 排除官方示例，FP < 0.1%）
  - OUT-03 GitHub Token（前缀 + CRC32 校验，FP < 0.05%）
  - OUT-04 JWT（三段 base64 + header 解码验证，FP < 0.5%）
  - OUT-05 RSA / Ed25519 / SSH 私钥（PEM 头精确匹配，FP < 0.01%）
  - OUT-06 Ethereum 私钥（regex + entropy + 上下文，FP < 1%，**只能 High，不上 Critical**）
  - OUT-07 Bitcoin WIF（base58 + 双 SHA-256 校验位，FP < 0.001%）
  - OUT-08 Solana 私钥（base58 88 字符或 hex 64 字节，FP < 1%）
  - **OUT-09 BIP39 助记词 + SHA-256 checksum 验证**（差异化点，FP < 0.05%；[PRD §9 #4](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - OUT-10 Keystore JSON（Web3 Secret Storage v3 schema，FP < 0.01%）
  - OUT-11 .env 文件特征（多行 KEY=VALUE 密度阈值，FP < 5%，仅 Medium）
  - OUT-12 数据库连接串（URI scheme + 用户名密码字段，FP < 0.5%）
  - 占位符黑名单 + `.sieveignore` 学习型白名单
  - 单元测试覆盖 ≥ 80%
- **W3 入站 Crypto 钩子**
  - SSE Parser + `tool_use` Aggregator
  - **IN-CR-01 地址替换检测**（对话历史 `0x[a-fA-F0-9]{40}` 比对：相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄）
  - **IN-CR-05 签名工具 fail-closed**（`eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 全部强制弹窗，YOLO mode 不可关闭，[PRD §9 #3](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - **大量 SSE fuzz test** 覆盖 PRD §9 #5 列出的 6 类边界
- **W4 入站通用 + 危险 tool call + benchmark 数据集**
  - IN-CR-02 危险工具调用（`bash` 含 `rm -rf` / `curl..|sh` / `eval(base64..)` / `sudo`）
  - IN-CR-03 敏感路径访问（`~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/`）
  - IN-CR-04 持久化机制（`crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc`）
  - IN-GEN-01 危险 shell 模式（`rm -rf /`、fork bomb、`> /dev/sda`、`dd if=/dev/zero`）
  - IN-GEN-02 远程脚本执行（`curl X | sh`、`wget X | bash`、`bash <(curl X)`）
  - IN-GEN-03 编码后执行（`eval(base64.b64decode(...))`、`exec(__import__('os')...)`）
  - IN-GEN-04 Markdown 图片 exfil（`![](http://X.com/?Y=Z)` + 域名不在白名单）
  - IN-GEN-05 Prompt injection 反向（`<|im_start|>`、`[INST]`、`### System:`、`Ignore previous`）
  - 处置矩阵完整实现（Critical / High / Medium / Low → HTTP 行为映射，参见 [API 参考 §5](../api/api-reference.md#5-处置矩阵--http-行为)）
  - CLI 弹窗 + 命令行确认（fail-closed，超时按拒绝，参见 [API 参考 §6](../api/api-reference.md#6-cli-退出码--弹窗确认协议)）
  - **Benchmark 数据集**（[PRD v1.3 §10.1 W4 修订](../prd/sieve-prd-v1.3.md#101-phase-adogfood-阶段week-1-8)）：
    - 200-500 条合成攻击样本（UCSB 4 类攻击 + drainer 模式 + Pink Drainer 数字化绕过 + npm typosquat + `curl|sh` + eval base64）
    - 50-100 条真实 benign 会话回放（doskey 自己日常 Claude Code 工作录制）
    - canary 测试（假 BIP39、假地址、假 selector、假 .env，使用 honeypot 钱包私钥）
    - 目标：Critical FP < 0.5%，High FP < 5%
- **W5 配置系统 + 试用期 + brew tap**
  - 完整 `config.toml` schema（参见 [API 参考 §3](../api/api-reference.md#3-配置文件-schema-sieveconfigtoml)）
  - 本地 SQLite append-only 审计日志（仅 fingerprint + 元信息，**不存原文**）
  - License 验证 + 试用期机制（**本地 Ed25519 验证，不联网 verify**，[PRD §9 #2](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - brew tap (`doskey/sieve`) + GitHub Releases 发布流水线
  - 本地管理 API（参见 [API 参考 §2](../api/api-reference.md#2-sieve-本地管理-api)）
- **W6 doskey 自用 + 修 bug**
  - doskey 100% 时间用 Sieve 工作
  - 性能 benchmark 验证 P99 < 20ms（[PRD §6.4](../prd/sieve-prd-v1.3.md#64-性能预算)）
  - macOS / Linux / Windows 二进制（macOS arm64 + Linux x86_64 为 Tier 1）
  - 收集 false positive，加 `.sieveignore` 默认条目
- **W7-W8 高强度 dogfood**
  - 第一次签名规则库下发测试（Ed25519 验证 fail-closed）
  - Stripe 接入 + license key 系统（**海外公司账号**，参见 [PRD §11.5.1](../prd/sieve-prd-v1.3.md#1151-公司主体与收款)）
  - 完成定义：doskey 用 Sieve 跑 2 周，无 P0 / P1 bug

### 计划中（Phase B 闭测, Week 9-12）

#### 新增

- **W9 闭测启动**
  - 5 人闭测白名单（[PRD v1.3 §10.2 W9 修订](../prd/sieve-prd-v1.3.md#102-phase-b闭测阶段week-9-12)）：
    - 高频 hackathon builder（ETHGlobal / Solana / 各 L2 hackathon 常客）
    - bug bounty hunter / 审计研究员（Code4rena / Sherlock / Immunefi 活跃用户）
    - 小团队 protocol engineer（< 10 人 protocol team）
  - **不邀请**：大企业开发者、纯 web2 友人、纯 KOL
  - 专属 license key
  - Discord 闭测频道
  - 每天处理反馈
- **W10 闭测 + 内容准备**
  - 修闭测 bug
  - 起草 3 篇引爆文章：
    1. 中转站揭黑（实测复刻 UCSB 论文方法论）
    2. **🆕 自证清白：Sieve 怎么证明自己不是新的 LiteLLM**（[PRD v1.3 §10.2 W10 修订](../prd/sieve-prd-v1.3.md#102-phase-b闭测阶段week-9-12)，把 §1.2 第 4 句讲透，后续所有营销围绕此 talking point）
    3. Pink Drainer 攻击复盘 + Sieve 怎么防
- **W11 闭测扩大 + 数据合作接洽**
  - 闭测扩到 10 人（同样画像标准）
  - landing page（英文为主，中文次之）
  - **数据合作优先于内容合作**（[PRD §13.2](../prd/sieve-prd-v1.3.md#132-数据侧合作清单v13-新增)）：
    - 第一目标：Chaofan Shou (@Fried_rice) 顾问关系
    - 第二目标：慢雾 @evilcos misttrack-skills 数据合作
- **W12 GA 发布（v0.1.0）**
  - **代码开源（MIT）**（[PRD §11.3](../prd/sieve-prd-v1.3.md#113-开源策略)）
  - 二进制 cosign 签名验证 + reproducible build 验证步骤公开（参见 [部署指南 §3](../guides/deployment.md#3-二进制签名验证必做)）
  - landing page 上线
  - 文章 1 + 2 同步发（中转站揭黑 + 自证清白）
  - 试用期全面开放
  - Stripe 收款上线（**[redacted]**）
  - **冻结 v1 公开 API**（参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)）
  - 完成定义：GA 第一周 GitHub stars > 200，试用注册 > 100，首批付费用户 ≥ 10

### 暂不做（明确推迟到 Phase 2）

- 中文 PII（身份证 / 银行卡 / 统一信用代码）
- 内网域名 / 内部代号（用户自定义规则）
- 长代码块识别 + Copyright 提示
- 自定义规则 DSL
- npm / pip typosquat 检测
- Markdown 链接钓鱼
- Unicode 攻击防御（NFC + 控制字符黑名单）
- Calldata 静态解码（4byte 离线 SQLite）
- ERC20 危险 approve（`approve(MAX)` / `setApprovalForAll`）
- EIP-2612 / EIP-7702 滥用
- Drainer 黑名单（Chainabuse + ScamSniffer 集成）
- 协议白名单
- Solidity 后门检测（Slither）
- **MCP 拦截 IN-MCP-01~03**（[PRD v1.3 §5.2 修订](../prd/sieve-prd-v1.3.md#52-入站检测sieve-真正的护城河)，Phase 2 Week 16-20）
- 桌面 App / VS Code 插件
- OpenAI / Gemini / OpenRouter 协议适配（**第二个用户主动要才做**，[PRD §9 #9](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）

---

## PRD 文档版本演进

> Sieve 项目尚未发布二进制版本，但产品需求文档已迭代 4 版。每版日期 + 一句话差异：

### [PRD v1.0](../prd/sieve-prd-v1.0.md) - 2026-04-26

- 工程启动前 PRD，团队 SaaS 视角，覆盖完整商业计划
- 一句话：双向检测的本地 LLM 流量代理，服务 crypto 开发者，反对中转站不可信
- 状态：**已废弃**，被 v1.1 收敛

### [PRD v1.1](../prd/sieve-prd-v1.1.md) - 2026-04-26

- 个人项目版：从 v1.0 砍掉一半范围，定位从"独角兽备选"改为"个人产品 + 现金流 + IP 入口"
- 关键改动：
  - 定价收敛为 3 档（Free / $19 Pro / $99 Crypto）
  - MVP 范围砍 50%，只做出站 secret + 危险 tool call + 地址替换 + 签名拦截
  - 三 agent 适配（Claude Code / OpenClaw / Hermes）用统一本地代理
  - sigstore + reproducible build 提到 Phase 1 必交付物
  - 桌面 App、VS Code 插件、Slither、中文 PII 全部推到 Phase 2
  - 节奏：6-8 周冲 MVP + 慢节奏维护
- 状态：**已废弃**，被 v1.2 第一性原理重写覆盖

### [PRD v1.2](../prd/sieve-prd-v1.2.md) - 2026-04-26

- 第一性原理修订版：用 12 条公理重新推导每个决策
- 关键改动：
  - 定价收敛到 **单一 [redacted]/月**（年付 [redacted]0），降级模式只读警告（公理 11 / 12）
  - 公理 7：Phase 1 **只做 Claude Code**，OpenClaw / Hermes 推迟到第二个用户主动要时
  - 12 周冲 GA（8 周 dogfood + 4 周闭测）
  - 处置矩阵：Critical 阻断 + High 警告 + Medium 标记 + Low 静默
  - "Sieve 的本质不是 LLM 安全产品，是在不可逆动作前插入认知摩擦的保险工具"
- 状态：**被 v1.3 取代**

### [PRD v1.3](../prd/sieve-prd-v1.3.md) - 2026-04-26（**当前活动版本**）

第一性原理 + 合规边界修订版，**锁定执行**。在 v1.2 基础上吸收 GPT-5.5 review 的 8 条改动：


| #   | 改动                                                                               | 章节         |
| --- | -------------------------------------------------------------------------------- | ---------- |
| 1   | **新增中国大陆合规边界**（v1.2 完全漏掉的硬约束）                                                    | §11.5 整章新增 |
| 2   | **"自证清白"从工程细节提到产品定位** —— sigstore + 透明日志做成营销 talking point                       | §1.2 第 4 句 |
| 3   | **数据标注稀缺性论证** —— 单人团队最稀缺资源不是算力，是持续标注高质量数据的能力                                     | §6.2       |
| 4   | **Benchmark 数据集大小具体化** —— 200-500 攻击样本 + 50-100 benign 会话                        | §10.1 W4   |
| 5   | **闭测画像精确化** —— hackathon builder + 审计研究员 + 小团队 protocol engineer                 | §10.2 W9   |
| 6   | **数据侧伙伴接洽清单** —— SlowMist / ScamSniffer / GoPlus / Chainabuse / Sourcify / Forta | §13.2      |
| 7   | **MCP 拦截放进 Phase 2** —— Claude Code 真实威胁面（IN-MCP-01~03）                          | §5.2       |
| 8   | **用户教育成本作为风险登记** + [redacted]周期延误风险                                                  | §12        |


附加改动：

- §1.4 法律实体明确：[redacted]（首选香港，次选新加坡）
- §3.1 P0 客群地理分布：海外为主
- §3.3 不服务客群补充：中国大陆境内公开 to-C 商业化
- §10.1 W1 sigstore + reproducible build pipeline 必须本周跑通
- §10.2 W10 文章 2 改为"自证清白"主题
- §10.2 W11 KOL 接洽：数据合作优先于内容合作
- §11.3 透明更新日志加入开源策略
- §15.4 监管参考资料

---

## 文档结构变更

### [unreleased](https://github.com/doskey/sieve/compare/v0.1.0...HEAD) - 2026-04-27

#### 新增

- 文档结构初始化：
  - `docs/api/api-reference.md` —— API 参考首版（反向代理 / 本地管理 API / 配置 schema / 环境变量 / 处置矩阵 / 错误码）
  - `docs/guides/development.md` —— 开发指南首版（构建、测试、SSE fuzz、benchmark、规则编写、PR 流程）
  - `docs/guides/deployment.md` —— 部署与运维指南首版（安装、签名验证、服务运行、升级回滚、FAQ）
  - `docs/changelog/CHANGELOG.md` —— 本文件
- 所有文档反映 [PRD v1.3](../prd/sieve-prd-v1.3.md) 设计意图，**未实现任何代码**

#### 文档审查与一致性修复（2026-04-27）

全量审查 docs/ 文档对 PRD v1.3 的一致性，输出关键冲突清单并修复。

**修复（关键冲突）**：

- [ADR-005](../design/ADR-005-overseas-legal-entity.md) —— 移除未授权的 BVI / Cayman [redacted]行（PRD §1.4 仅锁定香港 / 新加坡 / [redacted]）
- [ADR-006](../design/ADR-006-sigstore-reproducible-build.md) —— 显式标注 Tier 1（macOS / Linux）/ Tier 2（Windows）平台分级；承认 Windows reproducible build 推到 Phase 2 是与 PRD §9 #6 全平台理想的暂时偏离，需 PRD 下次修订同步
- [deployment.md §2.3](../guides/deployment.md) —— Windows 部署描述加 Tier 2 标识（Week 6+ 才出二进制 + 签名，reproducible 不承诺）
- [deployment.md §11.1](../guides/deployment.md) —— 补 license 离线过期 → 降级模式自动转换流程
- [api-reference.md §1.6](../api/api-reference.md) —— 补完整入站 Critical SSE 序列（`sieve_block` → buffer 暂停 → `sieve_resume` / `sieve_terminate`）+ buffer 上限 + `event_id` 关联
- [api-reference.md §2.2.3](../api/api-reference.md) —— 补 `user_decision` 字段值域定义（`null` / approve / deny / timeout / interrupted）
- [api-reference.md §2.2.4](../api/api-reference.md) —— 明确 fingerprint = `<rule_id>:<sha256_prefix_8_hex>` 长度规范
- [api-reference.md / deployment.md] 多处过时的"待写"链接修正为已存在文档的真实链接
- 删除 CHANGELOG.md 末尾空白占位符（"模板段落" / "链接"）

**新增文档**：

- [docs/glossary.md](../glossary.md) —— 项目术语表（54 条术语，覆盖产品 / 架构 / 检测 / 安全 / 协议 / 运营 / 合规 7 个主题）
- [docs/design/ADR-INDEX.md](../design/ADR-INDEX.md) —— ADR 索引 + 编号规则 + 候选 ADR 列表（ADR-008 候选 Critical 状态码 / ADR-009 候选 Windows 服务 / ADR-010 候选加密支付通道）
- [tasks/roadmap.md](../../tasks/roadmap.md) —— 12 周里程碑可勾选执行清单 + 跨周依赖图
- [tasks/lessons.md](../../tasks/lessons.md) —— 经验教训记录骨架

**倾向决策（doskey 确认 2026-04-27）**：

- **ADR-008（候选）出站 Critical 状态码维持 `426 Upgrade Required`**——api-reference.md §7.2 现有方案。Week 2 dogfood 阶段实测 Claude Code SDK 行为后正式落 ADR；如 SDK 表现异常（自动重试 / 错误信息丢失等）再切换备选方案。已在 [tasks/roadmap.md](../../tasks/roadmap.md) Week 2 任务清单加入验证项。

#### Git 仓库脚手架（2026-04-27）

为内部 GitHub repo 基础设施（GA 前私有；[ADR-011](../design/ADR-011-private-until-ga.md) 规定 Week 12 GA 时公开）准备完整的 git 治理文件：

- **新增** `.gitignore` —— Rust + macOS / Linux / Windows + Sieve 特定（`.sieveignore` / `audit.db` / `*.sigstore` / 临时文档）。**Cargo.lock 不入忽略名单**（reproducible build 要求入库，[ADR-006](../design/ADR-006-sigstore-reproducible-build.md)）
- **新增** `.gitattributes` —— 强制 LF 行尾（reproducible build 跨平台一致性）+ GitHub linguist 语言识别（docs / prd / research 标记 vendored / documentation）+ 二进制文件标记
- **新增** `SECURITY.md` —— 安全漏洞报告流程（email doskey.lee@gmail.com 临时渠道，security@sieve.tools 待 Week 6-8 商标定后启用）+ 24h/7d/30d 响应 SLA + 自身供应链承诺 + 不在范围清单
- **新增** `LICENSE` —— 双轨许可说明：文档 **CC BY-NC-SA 4.0** / 代码 **MIT**（均在 Week 12 GA 时同步公开；[ADR-011](../design/ADR-011-private-until-ga.md)）
- **新增** `.github/ISSUE_TEMPLATE/` —— bug_report / feature_request / **suspicious_sample**（[PRD §8.1](../prd/sieve-prd-v1.3.md#81-简化版) 用户公开提交可疑样本走这里）+ config.yml（指引安全漏洞走 SECURITY.md，紧急资产损失走 email）
- **新增** `.github/PULL_REQUEST_TEMPLATE.md` —— 对齐 [.cursorrules §五](../../.cursorrules) 自检清单 + PRD §9 硬约束验证 + 检测项变更模板 + Breaking Changes 流程
- **新增** `.github/dependabot.yml` —— Cargo 周更（仅 patch / minor，major 走人工评估，对齐 [PRD §9 #6](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束) pinned dependencies）+ GitHub Actions 周更 + 关键依赖分组（tokio-stack / simd-stack / crypto-stack）

仓库尚未 `git init`。doskey 完成审阅后可执行：
```bash
cd /Users/doskey/src/sieve
git init
git add -A
git commit -m "chore: initial commit, Pre-Code design phase docs"
# 创建 GitHub repo 后：
git remote add origin <github-url>
git push -u origin main
```

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 术语表：[../glossary.md](../glossary.md)
- ADR 索引：[../design/ADR-INDEX.md](../design/ADR-INDEX.md)
- Roadmap：[../../tasks/roadmap.md](../../tasks/roadmap.md)

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。
> 任何依赖升级、行为变更、检测项 ID 增删必须在本文记录（`[.cursorrules` §1.5](../../.cursorrules)）。