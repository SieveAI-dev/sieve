# Sieve 术语表

> 项目文档中频繁出现的专业术语统一定义。所有 ADR / PRD / 设计文档涉及的术语首次出现时应在 glossary 中可查。
>
> **维护原则**：术语定义变更必须同步 PRD / ADR / 相关文档。新增术语在对应字母段插入，并检查 PRD / ADR 是否需要更新。

---

## A. 产品定位

### Sieve

Sieve（筛子）是完全本地运行的 LLM 流量代理，夹在 Claude Code 等 AI 编码 agent 和 Anthropic API 之间，对 crypto 开发者做双向安全检测。核心作用：在不可逆动作（签名、转账、部署）前强制插入认知摩擦，防止私钥泄漏、地址替换、危险工具调用导致的资产损失。v1.4 起引入 native GUI 守门人，提供 HIPS 弹窗 + 双层防御架构。

### HIPS（Host-based Intrusion Prevention System，主机入侵防御系统）

v2.0 PRD §0 启动改造的目标定位。Sieve 自我定位为与 CrowdStrike Falcon / Microsoft Defender ATP / OSSEC 同级别的主机入侵防御系统，在 AI agent 工具调用层面提供行为级拦截能力，而非仅做规则匹配。v2.0 落地后 HIPS 核心特性（行为序列检测、进程上下文反查、灰名单持久化）达标 90%。

### 自证清白 / [redacted]

Sieve 核心信任叙事的第四句话：产品不只要求用户信任，而是通过开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志等机制，让用户能够独立验证产品的安全性和诚实性。这是相对于 LiteLLM 供应链事件的反思——Sieve 自己不能成为新的风险源。

### 完整功能期 / Trial

新用户从首次安装 Sieve 开始的完整功能使用期。期满后进入降级模式。

### 降级模式 / Degraded Mode

完整功能期结束后用户所处的状态。Critical 仍 fail-closed 不可关闭，其他级别只读警告——即便在降级模式下，核心安全防线（Critical 拦截）始终保持。

### P0 / P1 客群

**P0 客群**：Crypto-native AI 重度开发者，日用 Claude Code ≥ 4 小时。**P1 客群**：智能合约开发者、bug bounty hunter、审计师。

---

## B. 技术架构层

### UnifiedMessage

Sieve 内部的统一消息表示格式，封装了来自 Anthropic Messages API 的 message 结构（文本、tool_use、tool_result），以及 Sieve 的扩展字段（规则匹配结果、风险等级、处置决策）。允许检测引擎以统一的数据结构处理出站和入站流量。详见 PRD §6.1。

### Outbound Filter Pipeline

Sieve 检测链中的上行阶段：用户 prompt 和交互内容从 Claude Code 向 Anthropic API 发送时经过的规则引擎。主要检测敏感信息泄漏（私钥、API key、BIP39 助记词）。

### Inbound Filter Pipeline

Sieve 检测链中的下行阶段：Anthropic API 的 response（包括 tool_use 结构）返回给 Claude Code 时经过的规则引擎。主要检测地址替换攻击、危险工具调用、签名钓鱼。

### Tool Use Aggregator

将多个 tool_use 对象（bash、write_file 等）聚合并进行批量检测的模块。支持跨工具调用序列的上下文分析（如"先 write_file 再 bash 执行"的危险组合）。

### AddressGuard

Sieve 的地址替换攻击防御模块。通过对比对话历史中用户指定的地址和模型输出中的地址，检测 1-3 字符偏差，标记为地址替换攻击。

### vectorscan / vectorscan-rs

Intel 开源的 SIMD 多模式正则引擎及其 Rust binding。Sieve 用于高效匹配大量规则（私钥前缀、BIP39 词表等）。相比 Go regexp，性能快 1000 倍，是 Sieve 采用 Rust 栈的核心原因。详见 PRD §6.3 和 [ADR-001](design/ADR-001-rust-tech-stack.md)。

### sonic-rs

Rust 版 SIMD JSON 解析器，用于高效解析和修改大型 tool_use JSON 结构（如 signTypedData 数据）。

> **当前状态（2026-05-01）**：v2.0 起未实际引入；JSON 解析全部走 serde_json。

### partial-json-parser

部分 JSON 解析库，用于处理 SSE 流中被分割的 JSON 数据包（流式推理时 tool_use 可能跨越多个事件）。

### SSE / Server-Sent Events

HTTP 的流式响应协议。Anthropic API 用 SSE 推送消息块（message_start、content_block_start、content_block_delta、message_stop）。Sieve 必须处理 SSE 边界的不规则性（可能在任意字节分割）。详见 [.cursorrules §2.5](../.cursorrules)。

### ANTHROPIC_BASE_URL

用户接入 Sieve 的环境变量。将其设为 `http://127.0.0.1:11453`（Sieve 本地监听地址），Claude Code 会将所有 API 请求路由到 Sieve，由 Sieve 转发至真实 Anthropic API。

### sieve-core / sieve-rules / sieve-cli

Sieve 的三个主要 crate：
- **sieve-core**：核心检测引擎（UnifiedMessage、Pipeline、规则执行）
- **sieve-rules**：规则定义、加载、版本管理
- **sieve-cli**：二进制入口点、配置解析、审计日志（SQLite）、UI 弹窗

详见 [.cursorrules §3.3](../.cursorrules)。

### MatchEngine trait

sieve-rules v2.0 引入的核心抽象接口，定义引擎的标准化扫描 API：`scan(text) -> Vec<Detection>`、`scan_with_context(ScanRequest) -> ScanReport`、`engine_name() -> &str`、`rule_count() -> usize`、`compiled_pattern_size_bytes() -> usize`。VectorscanEngine 和用户规则引擎均实现此 trait，通过 LayeredEngine 统一调度。详见 PRD §6.3.1。

### ScanRequest / ScanReport

v2.0 引擎 trait 的上下文数据结构（PRD §6.3.1）。`ScanRequest` 携带扫描上下文字段：`direction`（Outbound/Inbound）、`protocol`（Anthropic/OpenAI）、`content_kind`（RawText/ToolInput/ToolResult）、`tool_name`（可选）、`source_agent`（可选）、`caller_exe`（进程上下文反查结果，可选）。`ScanReport` 返回命中的 `Detection` 列表及引擎耗时统计。两者均为 `scan_with_context` 接口的参数和返回值，替代旧版裸字符串扫描。

### LayeredEngine

sieve-rules v2.0 引入的分层规则引擎包装器，将系统规则引擎（VectorscanEngine）和用户规则引擎组合为统一调度链：**强制系统规则先行**，命中 `critical_lock::FAIL_CLOSED_RULES` 的规则立即返回，不评估用户规则。用户规则引擎字段使用 `ArcSwap<Option<Arc<U>>>` 实现 lock-free 读 + atomic swap，支持 zero-downtime 热替换（见"arc-swap 热替换"条目）。详见 [CLAUDE.md §五个 Crate](../CLAUDE.md)。

### arc-swap 热替换（v2.1 新增）

LayeredEngine 中用户规则引擎的热更新机制。`LayeredEngine.user` 字段类型为 `ArcSwap<Option<Arc<U>>>`，scan hot path 通过 `load()` 零开销读取当前引擎快照（无锁），`sieve.reload_user_rules` 信号触发时通过 `store()` 原子替换新编译引擎。实现 lock-free 读 + atomic swap 的 zero-downtime 热替换，基于 [arc-swap](https://crates.io/crates/arc-swap) crate。

### HoldOutcome

sieve-core 入站 hold 路径的决策结果枚举，由 `sieve.decision_response` 解析后生成。三个变体：`Allow { remember: bool, context_hint: Option<String> }`（用户允许，携带灰名单信息）、`RedactAndAllow { redaction_map: Vec<(String, String)>, remember: bool, context_hint: Option<String> }`（允许但替换敏感内容）、`Deny { reason: String }`（用户拒绝或超时 fail-closed）。Pipeline 根据 HoldOutcome 决定后续 SSE 流的处理方式。

---

## C. 检测概念层

### AddressGuard

IN-CR-01 的实现模块，使用 strsim crate 的 Levenshtein 距离检测一字符近邻地址替换攻击。触发条件：candidate 与会话历史中 addresses_seen 内某地址长度相等且 Levenshtein distance ∈[1,3]。Phase 1 仅覆盖 ETH 地址（0x 前缀 40 字符十六进制），BTC 地址 Week 4 加入。

### 三态决策（Tri-state Decision）

v2.0 重写处置矩阵引入的决策模型（PRD §5.4.1）：`Decision := Allow | Deny | Ask`。Ask 触发 GUI 弹窗（GuiPopup）或 hook 终端（HookTerminal），用户最终产生 Allow 或 Deny；daemon 在 Ask 路径上等待用户决策，持有 SSE 流不发送后续 chunk。对应 ADR-021 三态决策与灰名单设计。

### 灰名单（Graylist）

用户在 GUI 弹窗选择"允许"并勾选 Remember 后，daemon 将该次决策的 fingerprint + context_hint 写入 `~/.sieve/decisions/<sha256_64_hex>.json`（文件权限 0600，目录权限 0700，atomic rename，no-follow symlink，所有变更写 audit.db）（PRD §5.4.2）。后续相同 fingerprint 的请求命中灰名单时直接放行（GraylistHit 审计事件），无弹窗。与 .sieveignore 白名单的区别：灰名单带 context_hint 和 remember 来源追踪；白名单基于 fingerprint 前缀手动管理。

### Critical 锁三道防线

v2.0 防止内置 Critical 规则被"永久允许"绕过的三层校验机制（PRD §5.4.3）：(1) **IPC 层**：daemon 端 `is_critical_locked(rule_id)` 计算 `allow_remember=false` 传入 DecisionRequest，GUI 收到 false 必须 disabled+灰显 Remember checkbox；(2) **存储层**：`graylist::add_entry()` 内部二次校验 rule_id 是否允许 Remember，不允许则忽略写入并记录 `GraylistCriticalRejected` 审计事件；(3) **GUI 层**：Remember checkbox 对内置 Critical 规则始终 disabled+灰显，tooltip 解释原因，防止 GUI 实现 bug 绕过前两层。

### fingerprint（灰名单指纹）

灰名单文件名及 lookup 键，格式为 sha256 64 位小写 hex，输入 = `rule_id + matched_canonical + tool_name + protocol + content_kind + source_agent` 组合的 SHA-256 摘要（与 .sieveignore 的 `rule_id:sha256_prefix_8_hex` 格式不同）。每次 scan 命中时重新计算 fingerprint，查找 `~/.sieve/decisions/<fingerprint>.json` 是否存在，校验文件内容防篡改。详见 PRD §5.4.2。

### 用户规则（User Rules）

存储在 `~/.sieve/rules/user.toml` 的用户自定义检测规则，由 `sieve rules edit` 管理。`severity` 仅允许 `high`/`medium`/`low`（禁止 `critical`），`action` 仅允许 `warn`/`mark`/`ask`（禁止 `block`/`hook_terminal`）。通过 LayeredEngine 与系统规则并存，系统规则优先执行；命中 critical_lock 时跳过用户规则评估。支持 `enabled: bool` 字段和 `direction` 字段动态控制生效范围。

### direction 字段

用户规则的方向字段，类型 `RuleDirection := Outbound | Inbound | Both`（默认 `Both`，兼容旧版 user.toml 无此字段的情况）。LayeredEngine 在调用用户规则引擎前按 direction 过滤，确保出站规则只参与出站扫描、入站规则只参与入站扫描，避免误报。

### 行为序列窗口（Behavior Sequence Window）

sieve-core SessionState 中新增的 `ToolUseSequence` 数据结构（feature `sequence_detection` 默认 off，PRD §9 #15），记录滑动窗口内最近 N 次（默认 N=10）工具调用记录，窗口时效 TTL=5 分钟。每次工具调用产生一条 ToolUseRecord 写入窗口，IN-SEQ-* 规则在 `InboundFilter` 聚合完整 tool_use 后对窗口历史进行模式匹配。

### ToolUseRecord 结构化特征

行为序列窗口中每条工具调用记录的特征向量，用于 IN-SEQ-* kill chain 模式匹配。字段：`tool_class`（Shell/FileRead/FileWrite/Network/Other）× `path_category`（SensitiveSecret/Wallet/DotEnv/Code/Tmp/Other）+ 4 个布尔位：`network_egress`（是否发起网络请求）、`persistence_mech`（是否写持久化配置）、`cleanup_mech`（是否执行清理/痕迹消除）、`sensitive_file_hint`（是否涉及敏感路径）。

### IN-SEQ-* 启发式 kill chain

v2.0 新增的 3 条行为序列检测规则，`severity=High`，处置为 `StatusBar` 仅通知不阻断（PRD §5.7.2）：
- **`IN-SEQ-01-RECON-EXFIL`**：FileRead+SensitiveSecret 后跟 network_egress——疑似文件读取后外渗
- **`IN-SEQ-02-CLEANUP-AFTER-ATTACK`**：Shell+network_egress 后跟 cleanup_mech——疑似攻击后痕迹清理
- **`IN-SEQ-03-PERSISTENCE-CHAIN`**：3 次以上 persistence_mech 分布在不同 tool_name——疑似持久化后门链

命中时通过 `StatusBarNotify { kind: SequenceHit }` 广播状态栏通知，写 `SequenceHit` 审计事件，不 hold SSE 流。

### 双路径不变量（Dual-Path Invariant）

PRD §5.7.4 + §9 #16 的工程约束：SSE 流路径（content_block_delta 流式处理）和 JSON 路径（非流式整体解析）**必须同时更新**行为序列窗口 `ToolUseSequence`。任何涉及工具调用检测的新功能必须经过 4 类 content-type 矩阵测试（SSE 流 × Anthropic / SSE 流 × OpenAI / JSON × Anthropic / JSON × OpenAI），确保两条路径行为一致。

### 进程上下文反查（Process Context Lookup）

daemon accept loop 在新连接建立时，通过 macOS `proc_listpids` + `proc_pidfdinfo` 接口反查连接方的 PID，再通过 `proc_pidpath` 获取可执行文件路径（caller_exe）。结果存入 30s TTL 的 LRU cache，写入 audit.db 的 `caller_pid` + `caller_exe` 列（v2 audit schema 新增列）。非 macOS 平台或反查失败时字段为 NULL。

### NotifyKind 状态栏通知类型

IPC `sieve.notify_status_bar` 消息中的通知类型枚举，用于 GUI 展示不同样式的状态栏通知：`SequenceHit`（行为序列命中）、`OutboundRedacted`（出站脱敏）、`UserRulesLoadFailed`（用户规则加载失败）、`UserRulesReloaded`（用户规则热加载成功）、`Generic`（通用临时信息）。详见 [api-reference.md §6.3.1](./api/api-reference.md#631-sievenotify_status_bar-daemon--gui-单向)。

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

Sieve 的核心防守原则：当风险等级为 Critical 时，即使用户启用 Claude Code 的"YOLO mode"（跳过工具确认），Sieve 也**必须**强制人工确认，不允许自动通过。这是公理 8 的硬约束。详见 PRD §9 #3 和 [ADR-007](design/ADR-007-fail-closed-critical-actions.md)。

### Critical / High / Medium / Low

Sieve 的四级风险分类。**Critical**：可立即导致资产损失的风险（私钥泄漏、地址替换、危险工具执行）；**High**：需要人工判断的风险；**Medium / Low**：警告或通知。各级别的 FP 上限见 PRD §5.3。

### YOLO mode

Claude Code 中用户可启用的跳过工具确认的模式。Sieve 对 Critical 风险 fail-closed，即 YOLO mode 也无法绕过。

### .sieveignore

学习型白名单文件，位于用户工作目录（如 `~/.sieve/` 或项目根）。用户可记录已审核的规则匹配项，Sieve 后续相同触发条件下自动跳过警告。用于误报治理。

### fingerprint

白名单中的规则标识符，格式为 `rule_id:sha256_prefix_8_hex`。使用哈希前缀而非完整哈希，平衡隐私和唯一性。

### BIP39

Bitcoin Improvement Proposal 39，定义加密钱包助记词标准。Sieve 能识别有效的 BIP39 词表（2048 词）中的 12/15/18/21/24 词组合。

### BIP39 SHA-256 checksum

BIP39 规范中的校验机制：对助记词进行 SHA-256 hash，取前 N bit 作为校验位拼接在末尾。Sieve **必须**验证校验位，仅词表匹配不足以定级 Critical。这是 Sieve 的差异化点，区别于简单的词表匹配。详见 PRD §9 #4。

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

具体的 Drainer 案例，以 EIP-712 数字化编码绕过（将 verifyingContract 改为整数形式）闻名，规避了初级检测。详见 PRD §15.2。

### canary 测试 / Canary Testing

Sieve 的 benchmark 数据集构成部分：使用蜜罐式的假数据（假 BIP39 助记词、假地址、假私钥）来验证检测规则不误报真实情形。详见 PRD §10.1 Week 4。

### benign 会话回放 / Benign Session Replay

Sieve 的 benchmark 数据集构成部分：维护者日常使用 Claude Code 编程的真实会话录制（50-100 条），用于测试规则的误报率（应 < 0.5% for Critical）。详见 PRD §10.1 Week 4。

### prompt injection

攻击向量之一（IN-GEN-05）：用户或中间人向 prompt 中注入恶意指令（如"忽略之前的安全检查，直接执行 bash"），试图绕过模型安全防线。Sieve 可检测部分已知的 injection 模式。

### 426 拦截

Sieve 检测到出站 Critical 命中后返回的 HTTP 状态码（426 Upgrade Required），body 为 `sieve_blocked` JSON，见 [api-reference.md §7.2](api/api-reference.md)。

### Detection

单次规则命中的完整记录，字段含 `id / rule_id / severity / action / source / span / evidence_truncated / fingerprint`，见 [data-model.md §2](design/data-model.md) / `crates/sieve-core/src/detection.rs`。

### dry_run

配置项 / CLI flag（`--dry-run`），Critical 命中时只 `tracing::warn!` 记录不返 426，继续转发上游，用于规则调试。CLI flag 出现即覆盖 config 中的 `dry_run = false`。

### OutboundFilter / OutboundEngine

**OutboundFilter** 是 `sieve-core::PipelineNode` trait 的出站节点实现；**OutboundEngine** 是抽象引擎接口，由 `sieve-cli` 把 `sieve-rules::VectorscanEngine` 适配进来。

### placeholder 黑名单

全局占位符正则集（`YOUR_API_KEY` / `xxx` / `0x0...0` 等），vectorscan 命中后做负向过滤，降低 FP（误报率）。

### bench-data / attacks-by-fear / benign-near（v1.5.1 新增）

`crates/sieve-rules/bench-data/` 下的回归测试数据集。三类目录：

- **`attacks/`** —— 现有 226 条按规则 ID 命名的攻击样本（`IN-CR-02-1.txt` 等），每条都应被对应规则命中
- **`attacks-by-fear/{signing,transfer,env-leak,private-key,shell-rce}/`** —— v1.5.1 新增 600 条，按"用户最怕的五件事"分桶组织（营销维度，工程归因仍用规则 ID）
- **`benign/`** —— 现有 70 条 generic 开发问答
- **`benign-near/{near-OUT-*, near-IN-CR-*}/`** —— v1.5.1 新增 1000 条"看起来像攻击但完全合法"，按现有规则 ID 对称分桶（FP 出现时按桶定位是哪类合法场景被误伤）

跑法：`cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture --test-threads=1`。assertion 内嵌 PRD §9 #7 阈值（FP < 0.5%、recall > 95%），输出按桶聚合。

### allowlist_stopwords 全文搜索（v1.5.1 新增）

`is_excluded(matched_text, full_context, rule)` 在 `allowlist_stopwords` 命中时**在完整上下文中搜索停用词**而非仅在命中片段里。让短命中（`eval $`、`rm -rf /`、`systemctl enable`）能识别教学/合法场景：教学短语（`the difference between` / `DO NOT RUN`）、合法 shell 初始化（`direnv hook` / `starship init`）、Dockerfile 安全前缀、官方 registry 域名等。

---

## D. 安全 & 合规层

### sigstore / cosign / Rekor

开源供应链安全工具链：
- **sigstore**：无密钥签名平台，基于 OIDC 而非私钥
- **cosign**：容器镜像和二进制签名工具
- **Rekor**：透明签名日志，所有签名都被公开记录

Sieve 的二进制和规则库都用 sigstore 签名，让用户能独立验证来源。详见 PRD §15.3 和 [ADR-006](design/ADR-006-sigstore-reproducible-build.md)。

### Reproducible Build

确定性构建：相同源代码、相同编译工具链、相同配置，两次编译得到**完全相同的二进制**（包括时间戳、哈希）。Sieve 采用 Reproducible Build，用户可验证下载的二进制确实来自公开代码。详见 [ADR-006](design/ADR-006-sigstore-reproducible-build.md)。

### SLSA

Supply-chain Levels for Software Artifacts，Google 主导的供应链安全评分标准。Sieve 瞄准 SLSA Level 3（签名 + 构建日志 + 源代码认证）。详见 [ADR-006](design/ADR-006-sigstore-reproducible-build.md)。

### pinned dependencies

在 Cargo.lock 中明确锁定所有依赖的版本，禁止浮动版本号。防止依赖在后续构建时悄悄升级为恶意版本（如 LiteLLM 事件）。详见 PRD §9 #6。

### Ed25519

Sieve 用于规则库签名的公钥密码算法。相比 RSA，Ed25519 更小、更快、密钥导出确定性。

### LiteLLM 投毒事件

2026-03-24，Python LiteLLM 库的 PyPI 包被投毒，版本 1.82.7/1.82.8 包含恶意代码，窃取用户 LLM API key。这个事件证明了"上游不可信"的真实性，也驱动了 Sieve 对自身供应链的严格要求。详见 PRD §2.1 和 [ADR-006](design/ADR-006-sigstore-reproducible-build.md)。

### logging level 三档（off / metadata / full）

`[audit].level` 配置项的三档审计落盘粒度（ADR-037 引入）：`off`（什么都不留，不写 `audit.db` 不写归档）/ `metadata`（**默认**，= 当前已发布行为，复用 `events` 表存脱敏后元数据：时间戳、方向、命中规则、动作、用户处置）/ `full`（**opt-in + 显式警告，默认关**，额外存一份脱敏后完整内容的加密归档，带保留期 + 哈希链）。中间档命名为 `metadata` 而非 `decisions`，是为避开既有术语冲突——`decisions` 已用于 `~/.sieve/decisions/` 灰名单目录、`sieve decisions` headless CLI 与 [ADR-021](design/ADR-021-tri-state-decision-and-graylist.md) 三态决策模型。`full` 档元数据仍照常写 `events` 表，不分叉数据模型。详见 [ADR-037](design/ADR-037-encrypted-audit-log.md)。

### write-only logging（只写日志）

ADR-037 `full` 档加密归档的核心模型：daemon **只持有 age 公钥（recipient）**，结构上**只能加密追加、不具备解密能力**；私钥（identity）平时离线（密码管理器 / 另一台机器 / 离线介质），仅在审计时出现。收益是即便机器运行时被攻陷（live malware），攻击者也**解不开历史归档**——机器上根本没有私钥。残余暴露面只是 malware 可截获正在流过的新明文流量，任何日志设计都防不住，ADR 不夸大。加密为 hybrid 混合（每段随机对称数据密钥做 AEAD，公钥包裹数据密钥），优先用 `age`（X25519 + ChaCha20-Poly1305）不手搓密码学。详见 [ADR-037](design/ADR-037-encrypted-audit-log.md)。

### 归档单元 / archive segment

ADR-037 `full` 档加密归档的存储与保留单位。每个归档段是一份独立的 age 密文文件，输入**只能**是脱敏后内容（出站 `redact_body_bytes()` 返回值 / 入站经 redaction map 替换后的内容），绝不是 pipeline 入口的原始 body——呼应「脱敏先于任何字节落盘」红线。保留期 `retention_days = N` 按段（mtime / 段日期）整段删除，删除是 `full` 档归档上**唯一允许的变更**（区别于 `events` 表 append-only），每次清理写一条 `metadata` 审计事件。密钥轮换时新段用新 recipient，旧段保持旧 recipient（段头记 key id）。详见 [ADR-037](design/ADR-037-encrypted-audit-log.md)。

### recipient（age 公钥）/ identity（age 私钥）

age 非对称加密的密钥对，`sieve audit keygen` 生成 X25519 密钥对。**recipient**（`age1...` 公钥）写入 `config.toml [audit].recipient`，daemon 用它加密归档，是 write-only logging 中 daemon 唯一持有的密钥。**identity**（私钥）以口令保护（age 原生 scrypt KDF）输出，**强制移出本机**，daemon 不留存，仅在另一台 / 离线机器执行 `sieve audit decrypt --identity <file>` 审计时使用。口令丢失则 identity 不可解锁、**归档永久不可读（by design）**，UI 必须最显眼方式警示。详见 [ADR-037](design/ADR-037-encrypted-audit-log.md)。

### 哈希链（审计防篡改 / Hash Chain）

ADR-037 `full` 档归档的防篡改机制（**已裁定必做**），独立于加密：加密保证「读不到」，哈希链保证「不被悄悄改写」。每条归档记录含 `prev_hash`（上一条记录密文的 SHA-256）+ 单调递增 `seq`，中间任何删改 / 重排会断链，截断尾部留下 `seq` 缺口，呼应 [ADR-006](design/ADR-006-sigstore-reproducible-build.md) Rekor 透明日志的本地同构。**残余局限（诚实写明）**：哈希链**挡不住「末尾追加伪造」**——被攻陷的 daemon 持公钥可继续合法追加并续上链；它保证的是历史不可悄悄改写 / 删除 / 重排。Phase 1 不引入外部锚点，尾部截断可检出缺口但无法区分「合法新写入未完成」与「恶意截断」。详见 [ADR-037](design/ADR-037-encrypted-audit-log.md)。

---

## E. 协议 & 上游层

### Anthropic Messages API

OpenAI-style 的 LLM API 协议，支持流式推理、工具调用、multimodal 输入。Sieve 完全代理这个 API 的请求和响应。

### Claude Code

Anthropic 官方 AI 编码助手，Phase 1 的唯一支持客户端。用户通过设置 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` 让 Claude Code 的请求路由到 Sieve。

### 中转站 / API relay / 代理

第三方服务，位于用户和 Anthropic API 之间，转发和可能修改请求/响应。Sieve 的核心问题定位正是"中转站不可信"。Sieve 本身也是中转站，但以"本地运行、开源可验证"来重新定义信任模型。

### MCP / Model Context Protocol

Anthropic 的工具协议，允许 LLM 通过标准接口调用外部工具（不仅是 bash/write_file，还有数据库查询、文件搜索等）。Phase 2 中 Sieve 计划支持 MCP server 的安全检测。详见 PRD §5.2。

### MCP server

实现 MCP 协议的服务端，提供工具定义和执行。Sieve 的 IN-MCP-01~03 规则针对 MCP server 的工具调用进行检测。

### OpenClaw

Peter Steinberger 开发的本地多通道消息网关 + 个人 AI 助手，支持接入 WhatsApp / Slack / Telegram 等外部 channel 并路由给 LLM。Sieve v1.5 起 Phase 1 适配：通过改写 OpenClaw config 把所有 LLM provider base_url 指向 Sieve 11453 端口。详见 PRD v1.5 §6.6 和 [ADR-019](design/ADR-019-x-sieve-origin-header.md)。

### Hermes Agent

Nous Research 自我改进的 multi-provider AI agent 编排器，支持动态 delegate 给 Claude Code / Codex CLI 等子 agent，使用 Hermes Function Calling 标准（`<tools>` / `<tool_call>` schema）。Sieve v1.5 起 Phase 1 适配：通过改写 Hermes provider config 的 base_url + 注入 `X-Sieve-Origin` header 追踪调用链。详见 PRD v1.5 §4.6。

### multi-agent 适配

Sieve v1.5 新增能力：同时挡在多家 AI agent（Claude Code + OpenClaw + Hermes）与上游 LLM 之间做双向安全检测。依赖协议适配层（Anthropic Messages API + OpenAI Chat Completions 兼容，详见 [ADR-018](design/ADR-018-openai-protocol-adaptation.md)）。引擎 100% 复用 v1.4，差异仅在协议适配 + 配置注入 + 2 条新检测项。

### X-Sieve-Origin header

v1.5 引入的 HTTP header 协议，用于 sub-agent 嵌套调用的 origin 追踪。格式：`X-Sieve-Origin: <agent_id>:<session_id>:<chain_depth>`。当 `chain_depth ≥ 2`（两层以上嵌套），强制走 GUI hold 并展示调用链信息，防止嵌套决策被低层 agent 绕过。详见 [ADR-019](design/ADR-019-x-sieve-origin-header.md)。

### 跨通道 prompt injection

v1.5 新增检测项 IN-GEN-06 的核心威胁：OpenClaw 接入的外部 channel（WhatsApp / Slack / Telegram 等）可能注入恶意 prompt，试图通过 OpenClaw 转发路径绕过用户认知，让 LLM 执行资产操作。Sieve 检测 channel 消息中的命令式短语模式（如"忽略之前的指令"/ "立即转账"等）。

### Hook 类规则降级

OpenClaw / Hermes 没有 Claude Code PreToolUse hook 等价物时，原本走"hook 终端弹窗"的 Hook 类规则（IN-CR-02/04/05 等）降级为"GUI hold"。降级不破坏 fail-closed 承诺（GUI hold 同样 100% 阻断），只是 UX 退步（用户需从 GUI 而非终端确认）。Phase 1 后期计划给 OpenClaw / Hermes 提 PR 引入 PreToolUse 等价拦截点。

### PreToolUse hook 等价物

泛指 AI agent 在执行工具调用前的拦截点，供 Sieve sieve-hook 写入 pending 文件实现双层防御。目前只 Claude Code 提供（通过 `settings.json` hook 注册）；OpenClaw / Hermes 暂无，Phase 1 后期提 PR 推动适配。详见 [ADR-014](design/ADR-014-dual-layer-defense.md)。

### sub-agent 嵌套调用

Hermes delegate 给 Claude Code / Codex CLI 的两层调用模式。Hermes 主进程发起第一层请求（`chain_depth=0`），其 delegate 的子 agent 发起第二层请求（`chain_depth=1`）。Sieve 通过 `X-Sieve-Origin` header 识别调用链深度，`chain_depth ≥ 2` 时强制 GUI hold 并警告用户。

### 超额计费检测 / overbilling detection

ADR-038 引入的能力：Sieve 作为夹在 agent 与上游之间的代理，**两个方向的原始字节都有**，对经过的 LLM 流量做独立 token 核算，跟中转站（relay）声明的 `usage` 字段交叉比对，偏差超容差报警。目标不是「逐 token 精确对账」而是「超额计费异常检测」——抓系统性虚报（如乘 1.5 = 多报 50%，远高于 tokenizer 噪声 ±5~10%），不追求逐字节相等。检出仅 StatusBar 报警 + 写本地 usage 记录，**不阻断流量**（计费监督，非安全拦截，不引入新 Block 路径）。`[billing_check].enabled` **默认 false**，开启则零行为变化、零新增出站、零计算开销。详见 [ADR-038](design/ADR-038-overbilling-detection.md)。

### 信任分级（official / relay）

ADR-038 对上游的信任标记，在 `[[upstream]]` 上**按 host 自动派生**、可显式 `trust` 覆盖：**`official`（官方直连）**——host ∈ {`api.anthropic.com`, `api.openai.com`}（可配置扩展），`usage` 视为**权威**直接采纳、不必自己算、零核算开销；**`relay`（经中转）**——其余所有上游，`usage` 视为**未经验证的声明**，必须独立核算 + 交叉比对，偏差超容差报警。保守默认：无法判定时按 `relay` 处理（fail-closed 倾向——把可信当不可信只多算一次，把不可信当可信会漏掉欺诈）。复用 [ADR-026](design/ADR-026-port-based-listener-routing.md) 的 `provider_id` 做审计归因。详见 [ADR-038](design/ADR-038-overbilling-detection.md)。

### 独立 token 核算（Independent Token Accounting）

ADR-038 对抗不可信 relay 的核心手段：永远优先权威信源，只在对抗 `relay` 时才自己算，且自己算也不手搓 tokenizer。独立计数 × **官方公开单价**（本地内置价表，按 model + 信任级查）= 应收成本，跟 relay 声明比对，偏差 = `|relay_claim - independent_count| / independent_count` 超 `tolerance_pct`（**默认 15%**）即报警。挂在 pipeline 响应完成后的观测节点、**off the hot path**（fire-and-forget 不阻塞转发）。流式时累计完整文本再 tokenize（复用 SSE Aggregator）。统计落本地 `~/.sieve/usage.db`（0600，append-only），**严格本地、永不上传**——呼应 [SPEC-006](specs/SPEC-006-update-and-telemetry.md) never-upload 承诺，即使聚合产品分析也禁止。详见 [ADR-038](design/ADR-038-overbilling-detection.md)。

### tiktoken（o200k_base / cl100k_base）

OpenAI 的 BPE tokenizer 及其编码表，ADR-038 用 `tiktoken-rs` crate（不手搓 BPE）对 OpenAI 侧做独立 token 计数：GPT-4o 及更新模型用 `o200k_base` 编码，老模型用 `cl100k_base`，并把 chat 消息每条/每轮的框架开销（per-message / per-reply tokens）算进去，接近精确。vocab 文件随二进制分发或首启缓存（评估打包进二进制避免运行时下载，呼应 PRD §9 #2）。Anthropic 侧因 Claude 无公开 tokenizer，输入默认本地近似估算、输出只能近似估算（明确标为估算）。详见 [ADR-038](design/ADR-038-overbilling-detection.md)。

### count_tokens 直连

Anthropic 官方 `POST /v1/messages/count_tokens` 端点，ADR-038 拿它作为 Anthropic 输入侧的**权威独立信源**跟 relay 比对（直连官方、绕过 relay、请求体为脱敏后内容、有独立 rate limit 不按 token 计费）。作为独立开关 **`count_tokens_optin` 默认 false**，仅用户显式开启才生效。**注意 §9 #2 张力**：它字面上正是「Sieve 主动发起、非用户直接触发的、远端 token 校验」出站——已裁定路径 (C)：**默认不联网**（Anthropic 输入默认仅本地近似估算，§9 #2 一字不破），仅在用户 opt-in 时打官方 endpoint，且 config / UI 必须显著警示「这会向 `api.anthropic.com` 发起一次 Sieve 主动出站」；文档把它与「联网做 verifier（Sieve 自营云后端）」严格区分。详见 [ADR-038](design/ADR-038-overbilling-detection.md)。

---

## F. 项目运营 & 节奏层

### 12 周里程碑

Sieve 从开始开发到 GA（通用版）发布的目标周期。分为三个 phase：Phase A dogfood (Week 1-8) + Phase B 闭测 (Week 9-12) + Phase C 维护 (Week 13+)。详见 PRD §10。

### Phase A dogfood

第一个 8 周，使用 Sieve 完成日常工作（Claude Code 编程），收集真实误报，积累 benchmark 数据集。目标：连续一周无 P0/P1 bug。详见 PRD §10.1。

### Phase B 闭测 / Closed Beta

第二个 4 周（Week 9-12），邀请 5-10 个精准的 crypto 开发者（hackathon builder、bug bounty hunter、protocol engineer）参与测试，收集反馈和修复 bug。详见 PRD §10.2。

### Phase C 维护 / Maintenance

Week 13 之后的长期维护模式。每月发布一篇深度内容，规则库每周更新一次，季度发大版本（Phase 2 功能逐项上线）。

### hackathon builder

闭测画像之一：ETHGlobal、Solana、L2 自主 hackathon 的常客。特点是时间紧、必用 AI、单个 hackathon 可能写 10+ 合约。是 Sieve 的理想用户。

---

## 维护规则

**新增术语**：在对应字母段按字母顺序插入，并同步检查 PRD / ADR 是否需要更新引用。

**删除术语**：仅当术语在所有文档中不再被引用时考虑删除。

**更新定义**：每次术语定义变更，必须通知相关文档（PRD / ADR / 设计文档）做联动修改。

**别名 / 缩写**：用斜体表示（如 *MCP server* 是 Model Context Protocol server 的缩写），并链接到主条目。

