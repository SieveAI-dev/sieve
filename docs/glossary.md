# Sieve 术语表

> 项目文档中频繁出现的专业术语统一定义。所有 ADR / PRD / 设计文档涉及的术语首次出现时应在 glossary 中可查。
>
> **维护原则**：术语定义变更必须同步 PRD / ADR / 相关文档。新增术语在对应字母段插入，并检查 PRD / ADR 是否需要更新。

---

## A. 产品定位与商业

### Sieve

Sieve（筛子）是完全本地运行的 LLM 流量代理，夹在 Claude Code 等 AI 编码 agent 和 Anthropic API 之间，对 crypto 开发者做双向安全检测。核心作用：在不可逆动作（签名、转账、部署）前强制插入认知摩擦，防止私钥泄漏、地址替换、危险工具调用导致的资产损失。详见 [PRD §1.1](../prd/sieve-prd-v1.3.md#11-一句话)。

### 自证清白 / [redacted]

Sieve 核心商业叙事的第四句话：产品不只要求用户信任，而是通过开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志等机制，让用户能够独立验证产品的安全性和诚实性。这是相对于 LiteLLM 供应链事件的反思——Sieve 自己不能成为新的风险源。详见 [PRD §1.2](../prd/sieve-prd-v1.3.md#12-四句话核心叙事v13-加第-4-句)。

### doskey

项目主体，Sieve 的个人创作者兼主要维护者。产品以 doskey 个人品牌背书，定位为[redacted]。

### 试用期 / Trial

新用户从首次安装 Sieve 开始的完全功能使用期限，$0 计费。试用结束后，用户可选择 [redacted]/月订阅或进入降级模式。详见 [PRD §7.1](../prd/sieve-prd-v1.3.md#71-单一定价)。

### 降级模式 / Degraded Mode

试用期结束且未付费的用户所处的状态。此时 Sieve 仅发出只读警告，不再强制拦截 Critical 级别威胁。这是产品的"宽松"态度，避免用户感觉被坑。详见 [PRD §7.1](../prd/sieve-prd-v1.3.md#71-单一定价)。

### P0 / P1 客群

**P0 客群**：Crypto-native AI 重度开发者，日用 Claude Code ≥ 4 小时，持有 crypto 资产，付费意愿强。**P1 客群**：智能合约开发者、bug bounty hunter、审计师，工作涉及单笔 $100K+ 价值的代码。详见 [PRD §3](../prd/sieve-prd-v1.3.md#3-用户画像)。

---

## B. 技术架构层

### UnifiedMessage

Sieve 内部的统一消息表示格式，封装了来自 Anthropic Messages API 的 message 结构（文本、tool_use、tool_result），以及 Sieve 的扩展字段（规则匹配结果、风险等级、处置决策）。允许检测引擎以统一的数据结构处理出站和入站流量。详见 [PRD §6.1](../prd/sieve-prd-v1.3.md)。

### Outbound Filter Pipeline

Sieve 检测链中的上行阶段：用户 prompt 和交互内容从 Claude Code 向 Anthropic API 发送时经过的规则引擎。主要检测敏感信息泄漏（私钥、API key、BIP39 助记词）。

### Inbound Filter Pipeline

Sieve 检测链中的下行阶段：Anthropic API 的 response（包括 tool_use 结构）返回给 Claude Code 时经过的规则引擎。主要检测地址替换攻击、危险工具调用、签名钓鱼。

### Tool Use Aggregator

将多个 tool_use 对象（bash、write_file 等）聚合并进行批量检测的模块。支持跨工具调用序列的上下文分析（如"先 write_file 再 bash 执行"的危险组合）。

### AddressGuard

Sieve 的地址替换攻击防御模块。通过对比对话历史中用户指定的地址和模型输出中的地址，检测 1-3 字符偏差，标记为地址替换攻击。

### vectorscan / vectorscan-rs

Intel 开源的 SIMD 多模式正则引擎及其 Rust binding。Sieve 用于高效匹配大量规则（私钥前缀、BIP39 词表等）。相比 Go regexp，性能快 1000 倍，是 Sieve 采用 Rust 栈的核心原因。详见 [PRD §6.3](../prd/sieve-prd-v1.3.md) 和 [ADR-001](../design/ADR-001-rust-tech-stack.md)。

### sonic-rs

Rust 版 SIMD JSON 解析器，用于高效解析和修改大型 tool_use JSON 结构（如 signTypedData 数据）。

### partial-json-parser

部分 JSON 解析库，用于处理 SSE 流中被分割的 JSON 数据包（流式推理时 tool_use 可能跨越多个事件）。

### SSE / Server-Sent Events

HTTP 的流式响应协议。Anthropic API 用 SSE 推送消息块（message_start、content_block_start、content_block_delta、message_stop）。Sieve 必须处理 SSE 边界的不规则性（可能在任意字节分割）。详见 [.cursorrules §2.5](../.cursorrules)。

### ANTHROPIC_BASE_URL

用户接入 Sieve 的环境变量。将其设为 `http://localhost:8734`（Sieve 本地监听地址），Claude Code 会将所有 API 请求路由到 Sieve，由 Sieve 转发至真实 Anthropic API。

### sieve-core / sieve-rules / sieve-cli

Sieve 的三个主要 crate：
- **sieve-core**：核心检测引擎（UnifiedMessage、Pipeline、规则执行）
- **sieve-rules**：规则定义、加载、版本管理
- **sieve-cli**：二进制入口点、配置解析、Stripe License 验证、UI 弹窗

详见 [.cursorrules §3.3](../.cursorrules)。

---

## C. 检测概念层

### AddressGuard

IN-CR-01 的实现模块，使用 strsim crate 的 Levenshtein 距离检测一字符近邻地址替换攻击。触发条件：candidate 与会话历史中 addresses_seen 内某地址长度相等且 Levenshtein distance ∈[1,3]。Phase 1 仅覆盖 ETH 地址（0x 前缀 40 字符十六进制），BTC 地址 Week 4 加入。

### Aggregator

Tool Use partial_json 跨 SSE event 聚合器（`sieve_core::sse::aggregator`）。负责将流式 SSE 中分片的 tool_use JSON 输入拼接完整，在 content_block_stop event 后 deserialize 为 `ToolUseBlock`，供 `InboundFilter` 做规则检测。

### critical_lock

`sieve_rules::critical_lock` 模块，存放 `FAIL_CLOSED_RULES` 常量（出站 OUT-01~12 全部 + 入站 11 条 critical 规则）及 `enforce_action(rule_id, requested) -> Action` 函数。运行时在 OutboundAdapter / InboundAdapter scan 时调用，即使 manifest action 写 allow / mark 也强制返回 `Action::Block`，且不受 `dry_run` 配置影响。

### sieve_blocked event

入站截流时注入的 SSE event（Week 3 落地）。Sieve 检测到入站 Critical 命中后，在 SSE 流中注入此 event 然后关闭连接。数据格式含 `type: "sieve_blocked"` / `blocked_at`（unix epoch）/ `detections[]`（rule_id + severity + fingerprint）/ `guidance`（zh + en）。关联 [api-reference.md §7.3](./api/api-reference.md)。

### SseParser

增量 SSE 解析器（`sieve_core::sse::parser::SseParser`），提供 `push_chunk` + `flush` 接口，无缓冲整流。Week 3 实现，覆盖 5 类边界：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流。所有 SSE 相关 PR 必须附 fuzz 覆盖（PRD §9 #5）。

### tool_use

Anthropic Messages API 中代表模型工具调用的结构。包含 tool name（bash、write_file 等）、input（JSON 对象）、id。Sieve 的入站检测主要围绕 tool_use 的合法性和危险性判断。

### fail-closed

Sieve 的核心防守原则：当风险等级为 Critical 时，即使用户启用 Claude Code 的"YOLO mode"（跳过工具确认），Sieve 也**必须**强制人工确认，不允许自动通过。这是公理 8 的硬约束。详见 [PRD §9 #3](../prd/sieve-prd-v1.3.md) 和 [ADR-007](../design/ADR-007-fail-closed-critical-actions.md)。

### Critical / High / Medium / Low

Sieve 的四级风险分类。**Critical**：可立即导致资产损失的风险（私钥泄漏、地址替换、危险工具执行）；**High**：需要人工判断的风险；**Medium / Low**：警告或通知。各级别的 FP 上限见 [PRD §5.3](../prd/sieve-prd-v1.3.md)。

### YOLO mode

Claude Code 中用户可启用的跳过工具确认的模式。Sieve 对 Critical 风险 fail-closed，即 YOLO mode 也无法绕过。

### .sieveignore

学习型白名单文件，位于用户工作目录（如 `~/.sieve/` 或项目根）。用户可记录已审核的规则匹配项，Sieve 后续相同触发条件下自动跳过警告。用于误报治理。

### fingerprint

白名单中的规则标识符，格式为 `rule_id:sha256_prefix_8_hex`。使用哈希前缀而非完整哈希，平衡隐私和唯一性。

### BIP39

Bitcoin Improvement Proposal 39，定义加密钱包助记词标准。Sieve 能识别有效的 BIP39 词表（2048 词）中的 12/15/18/21/24 词组合。

### BIP39 SHA-256 checksum

BIP39 规范中的校验机制：对助记词进行 SHA-256 hash，取前 N bit 作为校验位拼接在末尾。Sieve **必须**验证校验位，仅词表匹配不足以定级 Critical。这是 Sieve 的差异化点，区别于简单的词表匹配。详见 [PRD §9 #4](../prd/sieve-prd-v1.3.md)。

### 地址替换攻击 / Address Substitution

攻击方式之一（IN-CR-01）：中转站或中间人修改用户 prompt 中的目标地址，将资金转向攻击者账户。Sieve 通过对比对话历史检测此类攻击。

### 敏感路径访问 / Sensitive Path Access

攻击方式之一（IN-CR-03）：模型返回的 tool_use 试图读取本地敏感凭据文件，常见路径包括 SSH 私钥（`~/.ssh/id_rsa` 等）、AWS 凭据（`~/.aws/credentials`）、GCP ADC、Solana CLI keypair、geth keystore、`.netrc` / GPG / macOS Keychain / dotenv 等。Sieve 通过扫描 `tool_use.input` JSON 序列化结果，命中触发 **High 警告**（区别于持久化机制 IN-CR-04 的 Critical 拦截，因合法用例较多需用户判断）。10 条子规则均含 allowlist（如 `*.pub` 公钥 / `~/.ssh/known_hosts` / `.env.example`）防止误报。Week 4 落地，5s 倒计时弹窗 UI 待 Week 5 接入。

### 持久化机制 / Persistence Mechanism

攻击方式之一（IN-CR-04）：模型生成的 Bash / 工具调用试图把恶意代码写进系统启动配置，让其下次开机/开 terminal/cron 周期自动执行——典型的"清理一下机器"借口下偷偷埋的后门。Sieve 9 条 IN-CR-04-* 子规则覆盖：shell rc 文件（`>>` / `tee -a` 写 `.bashrc` / `.zshrc` 等）、crontab（`-e` / `<` / `-r` 不含 `-l`）、`/etc/cron.{d,daily,...}/`、macOS launchctl + LaunchAgents/LaunchDaemons plist、Linux systemd 单元 + `systemctl enable`、fish shell 配置、macOS Login Items（`defaults write LoginItems` / `osascript`）。处置 **Critical block + fail-closed**（YOLO mode 不可关），写持久化文件 = 后门埋点级别。pattern 锚定 Bash"写意图"（重定向 / tee / 启用命令）避免与 IN-CR-03 read=High 冲突。详见 [ADR-007 §"Week 4 落地范围"](./design/ADR-007-fail-closed-critical-actions.md)。

### 签名钓鱼 / Signature Phishing

攻击方式之一（IN-CR-05）：诱导用户对恶意内容（如 drainer 合约的 Permit 调用）签名。Sieve 对 EVM / Solana / Bitcoin 的签名相关 RPC 方法（`eth_signTransaction` / `personal_sign` / `signTypedData_v4` / `signAndSendTransaction` 等）强制 fail-closed Critical 拦截，YOLO mode 不可关。检测维度还包括 verifyingContract 数字化绕过与已知 drainer 特征。

### Drainer

钱包资产抽取攻击合约。通过精心构造的 Permit 签名、信息授权等机制，一次签名可清空钱包资产。是 crypto 用户面临的最严重威胁之一。

### Pink Drainer

具体的 Drainer 案例，以 EIP-712 数字化编码绕过（将 verifyingContract 改为整数形式）闻名，规避了初级检测。详见 [PRD §15.2](../prd/sieve-prd-v1.3.md#152-关键事件)。

### canary 测试 / Canary Testing

Sieve 的 benchmark 数据集构成部分：使用蜜罐式的假数据（假 BIP39 助记词、假地址、假私钥）来验证检测规则不误报真实情形。详见 [PRD §10.1 Week 4](../prd/sieve-prd-v1.3.md)。

### benign 会话回放 / Benign Session Replay

Sieve 的 benchmark 数据集构成部分：doskey 自己日常使用 Claude Code 编程的真实会话录制（50-100 条），用于测试规则的误报率（应 < 0.5% for Critical）。详见 [PRD §10.1 Week 4](../prd/sieve-prd-v1.3.md)。

### prompt injection

攻击向量之一（IN-GEN-05）：用户或中间人向 prompt 中注入恶意指令（如"忽略之前的安全检查，直接执行 bash"），试图绕过模型安全防线。Sieve 可检测部分已知的 injection 模式。

### 426 拦截

Sieve 检测到出站 Critical 命中后返回的 HTTP 状态码（426 Upgrade Required），body 为 `sieve_blocked` JSON，见 [api-reference.md §7.2](../api/api-reference.md)。

### Detection

单次规则命中的完整记录，字段含 `id / rule_id / severity / action / source / span / evidence_truncated / fingerprint`，见 [data-model.md §2](./data-model.md) / `crates/sieve-core/src/detection.rs`。

### dry_run

配置项 / CLI flag（`--dry-run`），Critical 命中时只 `tracing::warn!` 记录不返 426，继续转发上游，用于规则调试。CLI flag 出现即覆盖 config 中的 `dry_run = false`。

### OutboundFilter / OutboundEngine

**OutboundFilter** 是 `sieve-core::PipelineNode` trait 的出站节点实现；**OutboundEngine** 是抽象引擎接口，由 `sieve-cli` 把 `sieve-rules::VectorscanEngine` 适配进来。

### placeholder 黑名单

全局占位符正则集（`YOUR_API_KEY` / `xxx` / `0x0...0` 等），vectorscan 命中后做负向过滤，降低 FP（误报率）。

---

## D. 安全 & 合规层

### sigstore / cosign / Rekor

开源供应链安全工具链：
- **sigstore**：无密钥签名平台，基于 OIDC 而非私钥
- **cosign**：容器镜像和二进制签名工具
- **Rekor**：透明签名日志，所有签名都被公开记录

Sieve 的二进制和规则库都用 sigstore 签名，让用户能独立验证来源。详见 [PRD §15.3](../prd/sieve-prd-v1.3.md#153-必读项目) 和 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)。

### Reproducible Build

确定性构建：相同源代码、相同编译工具链、相同配置，两次编译得到**完全相同的二进制**（包括时间戳、哈希）。Sieve 采用 Reproducible Build，用户可验证下载的二进制确实来自公开代码。详见 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)。

### SLSA

Supply-chain Levels for Software Artifacts，Google 主导的供应链安全评分标准。Sieve 瞄准 SLSA Level 3（签名 + 构建日志 + 源代码认证）。详见 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)。

### pinned dependencies

在 Cargo.lock 中明确锁定所有依赖的版本，禁止浮动版本号。防止依赖在后续构建时悄悄升级为恶意版本（如 LiteLLM 事件）。详见 [PRD §9 #6](../prd/sieve-prd-v1.3.md)。

### Ed25519

Sieve 用于规则库签名的公钥密码算法。相比 RSA，Ed25519 更小、更快、密钥导出确定性。

### LiteLLM 投毒事件

2026-03-24，Python LiteLLM 库的 PyPI 包被投毒，版本 1.82.7/1.82.8 包含恶意代码，窃取用户 LLM API key。这个事件证明了"上游不可信"的真实性，也驱动了 Sieve 对自身供应链的严格要求。详见 [PRD §2.1](../prd/sieve-prd-v1.3.md) 和 [ADR-006](../design/ADR-006-sigstore-reproducible-build.md)。

---

## E. 协议 & 上游层

### Anthropic Messages API

OpenAI-style 的 LLM API 协议，支持流式推理、工具调用、multimodal 输入。Sieve 完全代理这个 API 的请求和响应。

### Claude Code

Anthropic 官方 AI 编码助手，Phase 1 的唯一支持客户端。用户通过设置 `ANTHROPIC_BASE_URL=http://localhost:8734` 让 Claude Code 的请求路由到 Sieve。

### 中转站 / API relay / 代理

第三方服务，位于用户和 Anthropic API 之间，转发和可能修改请求/响应。Sieve 的核心问题定位正是"中转站不可信"。Sieve 本身也是中转站，但以"本地运行、开源可验证"来重新定义信任模型。

### MCP / Model Context Protocol

Anthropic 的工具协议，允许 LLM 通过标准接口调用外部工具（不仅是 bash/write_file，还有数据库查询、文件搜索等）。Phase 2 中 Sieve 计划支持 MCP server 的安全检测。详见 [PRD §5.2](../prd/sieve-prd-v1.3.md)。

### MCP server

实现 MCP 协议的服务端，提供工具定义和执行。Sieve 的 IN-MCP-01~03 规则针对 MCP server 的工具调用进行检测。

---

## F. 项目运营 & 节奏层

### 12 周里程碑

Sieve 从开始开发到 GA（通用版）发布的目标周期。分为三个 phase：Phase A dogfood (Week 1-8) + Phase B 闭测 (Week 9-12) + Phase C 维护 (Week 13+)。详见 [PRD §10](../prd/sieve-prd-v1.3.md#10-执行里程碑与-roadmap)。

### Phase A dogfood

第一个 8 周，doskey 自己 100% 时间用 Sieve 完成日常工作（Claude Code 编程），收集真实误报，积累 benchmark 数据集。目标：doskey 一周无 P0/P1 bug。详见 [PRD §10.1](../prd/sieve-prd-v1.3.md)。

### Phase B 闭测 / Closed Beta

第二个 4 周（Week 9-12），邀请 5-10 个精准的 crypto 开发者（hackathon builder、bug bounty hunter、protocol engineer）参与测试，收集反馈和修复 bug。详见 [PRD §10.2](../prd/sieve-prd-v1.3.md)。

### Phase C 维护 / Maintenance

Week 13 之后的长期维护模式。doskey 每周稳定投入 5-10 小时，每月发布一篇深度内容，规则库每周更新一次，季度发大版本（Phase 2 功能逐项上线）。

### hackathon builder

闭测画像之一：ETHGlobal、Solana、L2 自主 hackathon 的常客。特点是时间紧、必用 AI、单个 hackathon 可能写 10+ 合约。是 Sieve 的理想用户。详见 [PRD §10.2 Week 9](../prd/sieve-prd-v1.3.md)。

---

## G. 法律与合规层

### Tier 1 / Tier 2 平台

根据市场容量和 Sieve 聚焦程度的分类。**Tier 1**：macOS / Linux（主战场，Sieve 将长期优化）；**Tier 2**：Windows（支持但非主要推广对象）。

### [redacted]

Sieve 的法律实体必须[redacted]，不能是中国大陆个人或个体户。首选[redacted]或[redacted] Ltd。理由：Stripe 账户要求、中国大陆 crypto 合规监管、doskey [redacted]。详见 [PRD §11.5.1](../prd/sieve-prd-v1.3.md#1151-公司主体与收款)。

### [redacted]

Sieve 的首选法律实体。注册快（2-4 周）、成本低（3,000-5,000 HKD）、银行账户对中国国籍友好、可衔接 Stripe Asia。

### [redacted]

美国 Stripe 提供的一站式公司注册服务（Delaware LLC）。Sieve 的第三选项，优点是 Stripe 集成丝滑，缺点是银行账户对中国国籍不友好。

### [redacted]

部分司法区的税务优惠，本地公司从海外来源获得的收入可享受减税或免税待遇（如新加坡的部分产业豁免）。详见 [ADR-005](../design/ADR-005-overseas-legal-entity.md)。

### 个人信息保护法 / PIPL

《中华人民共和国个人信息保护法》，对处理个人信息的企业设置高要求。Sieve 完全本地运行不上传 prompt，理论上不触发 PIPL，但明确这点很关键。详见 [PRD §11.5.3](../prd/sieve-prd-v1.3.md#1153-数据本地化)。

---

## 维护规则

**新增术语**：在对应字母段按字母顺序插入，并同步检查 PRD / ADR 是否需要更新引用。

**删除术语**：仅当术语在所有文档中不再被引用时考虑删除。

**更新定义**：每次术语定义变更，必须通知相关文档（PRD / ADR / 设计文档）做联动修改。

**别名 / 缩写**：用斜体表示（如 *MCP server* 是 Model Context Protocol server 的缩写），并链接到主条目。

