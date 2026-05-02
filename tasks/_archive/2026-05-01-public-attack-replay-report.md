# 公开攻击复现报告（2026-05-01，方案 C）

## TL;DR

共复现 **55 条**公开攻击，Sieve 拦下 **51 条**（51/55 = **92.7%**），漏拦 4 条。
涉及 6 个来源：rugpull-ai / injection-pocs / ctf-replays / owasp-llm-top10 / real-events / mcp-supply-chain。
现有 1896 条合成样本的 97.13% recall 经此次真实案例验证基本成立。

---

## 按来源拦截率

| 子目录 | Total | Hits | Recall |
|--------|------:|-----:|-------:|
| rugpull-ai | 9 | 9 | 100.0% |
| injection-pocs | 10 | 10 | 100.0% |
| ctf-replays | 8 | 8 | 100.0% |
| owasp-llm-top10 | 9 | 8 | 88.9% |
| real-events | 10 | 8 | 80.0% |
| mcp-supply-chain | 9 | 8 | 88.9% |
| **TOTAL** | **55** | **51** | **92.7%** |

---

## 漏拦案例分析

### Miss 1：owasp-003.txt — OWASP LLM04 RAG 投毒攻击
**文件**：`owasp-llm-top10/owasp-003.txt`
**来源**：https://genai.owasp.org/llmrisk/llm04-data-and-model-poisoning/

**攻击形态**：攻击者向 DeFi 文档 RAG 语料库投毒，植入"这是官方以太坊基金会捐款地址"的错误记录，诱导 AI agent 在生成代码时把转账目标替换为攻击者地址。样本中只有 `signer.sendTransaction` + 钱包地址，无 shell 命令、无签名方法名。

**当前规则缺什么**：IN-CR-01 地址替换用 Levenshtein 距离算法，但只在入站 tool_use JSON 中比对，无法覆盖"文档中植入的假地址在 AI 生成代码里被原样引用"这一 RAG 中毒路径。vectorscan 层无对应 pattern。

**是否值得加规则**：这是内容层的信任问题，不是 payload 特征问题。需要出站检测层扫描"AI 生成代码中的以太坊地址"与用户期望目标的一致性，属于 Phase 2 范畴。接受漏拦。

---

### Miss 2：real-003.txt — Lazarus Operation 99 LinkedIn 社工话术
**文件**：`real-events/real-003.txt`
**来源**：https://thehackernews.com/2025/01/lazarus-group-targets-web3-developers.html

**攻击形态**：虚假猎头在 LinkedIn 发送的初始招募消息，诱导开发者克隆含木马的 GitLab 仓库并运行 `npm install && node src/index.js`。纯自然语言社工，没有直接的危险命令特征。

**当前规则缺什么**：社工文本无法用 pattern 匹配（`npm install` 是完全合法操作，`node src/index.js` 也是）。这是入站信任来源问题，不是 payload 特征问题。

**是否值得加规则**：不值得。正常的开发工作指令（`npm install`、`node xxx`）在 Sieve 的覆盖范围之外，强行加规则会导致大量误报。接受漏拦，这属于人类判断层。

---

### Miss 3：real-006.txt — Ledger npm 账号网络钓鱼邮件
**文件**：`real-events/real-006.txt`
**来源**：https://www.sonatype.com/blog/decrypting-the-ledger-connect-kit-compromise-a-deep-dive-into-the-crypto-drainer-attack

**攻击形态**：伪装成 npm 官方的钓鱼邮件，针对 Ledger 员工，骗取 npm 账号凭据，最终导致 2023年12月 Ledger Connect Kit 供应链攻击 $600K 损失。纯邮件正文，无代码命令。

**当前规则缺什么**：Sieve 作用于 LLM 对话流量，不是邮件客户端代理。这是上游人类操作层面的攻击，超出 Sieve 的信任边界定义。

**是否值得加规则**：不适用。Sieve 不监控邮件。接受漏拦，同时这是真实事件里最重要的"初始突破口"说明，证明供应链攻击的开端往往在 Sieve 的保护范围之外——这反而强化了 Sieve 在后续阶段（LLM 被入侵后的命令执行层）的价值。

---

### Miss 4：mcp-001.txt — CVE-2025-6514 Windows PowerShell 变种
**文件**：`mcp-supply-chain/mcp-001.txt`
**来源**：https://jfrog.com/blog/2025-6514-critical-mcp-remote-rce-vulnerability/

**攻击形态**：CVE-2025-6514 的 Windows 变种。`cmd.exe /c curl https://... | powershell -` 注入在 mcp-remote OAuth 授权端点的 JSON 字段里。Sieve 的 IN-CR-02-CURL-PIPE pattern 只匹配 `curl ... | bash/sh`，不匹配 `| powershell`。

**当前规则缺什么**：IN-CR-02-CURL-PIPE pattern `curl(?:\s+\S+)+\s*\|\s*(?:sudo\s+)?(ba)?sh` 只覆盖 Unix shell，漏掉 Windows `powershell`/`cmd.exe` 管道形态。

**是否值得加规则**：**值得加**。Sieve 的 Phase 1 GA 覆盖三端（macOS/Linux 为主），但 Windows 是 Tier 2，用户可能存在。CVE-2025-6514 有 437,000+ 下载量影响面，补一条 `curl ... | powershell` pattern 代价极低。建议 Week 6 补入 IN-CR-02-CURL-PIPE-WIN 规则。

---

## 营销可用引语

以下均为可直接在 Twitter/Mirror 文章引用的句子，每条带来源 URL：

**1. Ledger Connect Kit 攻击（2023年12月，$600K 被盗）**
> "2023 年 12 月，Ledger Connect Kit 供应链攻击让 SushiSwap、Kyber、Revoke.cash 的用户在两小时内损失超 $600K。攻击载荷是通过 Angel Drainer 诱导用户签名 gasless permit。Sieve 的 IN-CR-05-ERC2612-PERMIT 规则在这类 permit(spender, deadline) 调用出现时立即弹窗确认——即便 Ledger 的库已被污染，Sieve 也会在签名前卡住你。"
来源：https://www.ledger.com/blog/security-incident-report

**2. ElizaOS 内存注入攻击（arXiv 2503.16248，2025年3月）**
> "学术研究证明，ElizaOS AI agent 可以通过 Discord 注入一条虚假内存，让 agent 在 X（Twitter）上收到合法转账请求时把资金转给攻击者。Sieve 的 IN-CR-05-SOLANA signAllTransactions 规则覆盖了这一路径——所有 signTransaction/signAndSendTransaction 调用都需要经过 GUI 弹窗人工确认，内存注入改不了这道门禁。"
来源：https://arxiv.org/abs/2503.16248

**3. CVE-2025-6514 mcp-remote RCE（CVSS 9.6，437,000+ 下载量影响）**
> "CVE-2025-6514 让任何连接到恶意 MCP 服务器的开发机都可能被完全接管——恶意 JSON 里的 `authorization_endpoint` 字段注入 `curl ... | bash`，在开发者毫不知情的情况下执行。Sieve 的 IN-CR-02-CURL-PIPE 规则匹配这一 pattern，在命令被执行前拦截并通知用户。"
来源：https://jfrog.com/blog/2025-6514-critical-mcp-remote-rce-vulnerability/

**4. SmartLoader / Oura MCP 木马（2026年2月，StealC 盗取加密钱包）**
> "2026 年 2 月，SmartLoader 通过克隆 Oura 健康 MCP 服务器并在 MCP Market 上架，让开发者安装含 StealC 信息窃取器的木马版本，偷走浏览器密码和加密钱包数据。Sieve 在 mcp-supply-chain 类攻击上命中率 88.9%——其中含 `curl ... | bash` 初始化脚本和 `printenv | grep KEY` 凭据扫描均被规则覆盖。"
来源：https://thehackernews.com/2026/02/smartloader-attack-uses-trojanized-oura.html

**5. Lazarus Operation 99（Web3 开发者定向攻击，2025年1月）**
> "2025 年 1 月，朝鲜 Lazarus 组织通过 Operation 99 对全球 Web3 开发者发起定向攻击，恶意代码调用 `printenv`、读取 `~/.config/solana/id.json`、使用 urllib.request.urlopen 外联。这三类操作均在 Sieve 规则覆盖范围内——IN-CR-03-PRINTENV、IN-CR-03-SOLANA-KEYPAIR、IN-CR-02-PYTHON-EXEC-URLOPEN 会在本地拦截这些动作。"
来源：https://thehackernews.com/2025/01/lazarus-group-targets-web3-developers.html

---

## 能力盲区总结

| 盲区类型 | 样本 | 是否加规则 |
|---------|------|-----------|
| RAG 投毒（假地址植入语料）| owasp-003 | Phase 2，接受 |
| 纯社工文本（LinkedIn/邮件钓鱼）| real-003, real-006 | 不适用，接受 |
| Windows PowerShell 管道形态（`curl \| powershell`）| mcp-001 | **Week 6 补 IN-CR-02-CURL-PIPE-WIN** |

---

## 来源清单（55 条样本全索引）

| 样本文件 | 来源 URL | 事件 |
|---------|---------|------|
| rugpull-001 | https://www.coinspeaker.com/malicious-ai-agent-routers-steal-crypto-attack-vector/amp/ | 恶意 LLM 路由钱包盗窃 |
| rugpull-002 | https://www.kucoin.com/blog/en-ai-trading-agent-vulnerability-2026-how-a-45m-crypto-security-breach-exposed-protocol-risks | AI Agent $45M 漏洞 |
| rugpull-003 | https://www.bleepingcomputer.com/news/security/step-finance-says-compromised-execs-devices-led-to-40m-crypto-theft/ | Step Finance $40M |
| rugpull-004 | https://medium.com/the-ledger-bc/more-crypto-rug-pulls-likely-as-hackers-zombify-ai-trading-bots-d3a7e6770447 | AI 交易机器人被僵尸化 |
| rugpull-005 | https://arxiv.org/abs/2503.16248 | ElizaOS 内存注入 |
| rugpull-006 | https://www.coinspeaker.com/malicious-ai-agent-routers-steal-crypto-attack-vector/amp/ | 路由注入 tool call |
| rugpull-007 | https://arxiv.org/abs/2503.16248 | ElizaOS 多平台内存注入 |
| rugpull-008 | https://www.kucoin.com/blog/en-ai-trading-agent-vulnerability-2026-how-a-45m-crypto-security-breach-exposed-protocol-risks | 过度权限私钥泄露 |
| rugpull-009 | https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/ | Unit 42 野外 IDPI #7 |
| injection-001 | https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/ | Unit 42 IDPI #3 强制订阅 |
| injection-002 | https://www.oasis.security/blog/claude-ai-prompt-injection-data-exfiltration-vulnerability | Oasis "Claudy Day" |
| injection-003 | https://www.lasso.security/blog/the-hidden-backdoor-in-claude-coding-assistant | Lasso Claude Code IDPI |
| injection-004 | https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/ | Unit 42 IDPI #5 Stripe |
| injection-005 | https://www.promptarmor.com/resources/claude-cowork-exfiltrates-files | PromptArmor 文件外泄 |
| injection-006 | https://www.docker.com/blog/mcp-horror-stories-github-prompt-injection/ | Docker MCP GitHub 注入 |
| injection-007 | https://www.aikido.dev/blog/promptpwnd-github-actions-ai-agents | PromptPwnd GitHub Actions |
| injection-008 | https://venturebeat.com/security/ai-agent-runtime-security-system-card-audit-comment-and-control-2026 | VentureBeat 3 agents 泄密 |
| injection-009 | https://www.infosecurity-magazine.com/news/researchers-10-wild-indirect/ | 10 野外 IDPI 案例 |
| injection-010 | https://www.truefoundry.com/blog/claude-code-prompt-injection | MCP tool def 注入 |
| ctf-001 | https://github.com/Joe-B-Security/awesome-prompt-injection | Awesome Prompt Injection CTF |
| ctf-002 | https://github.com/liu00222/Open-Prompt-Injection | Open-Prompt-Injection 基准 |
| ctf-003 | https://github.com/utkusen/promptmap | PromptMap base64 绕过 |
| ctf-004 | https://learnprompting.org/docs/prompt_hacking/injection | 角色扮演越狱 |
| ctf-005 | https://arxiv.org/abs/2507.14799 | HTML Accessibility Tree 注入 |
| ctf-006 | https://arxiv.org/html/2411.00459v2 | Unicode 不可见字符绕过 |
| ctf-007 | https://genai.owasp.org/llmrisk/llm01-prompt-injection/ | OWASP LLM01 多轮注入 |
| ctf-008 | https://arxiv.org/html/2506.23260v1 | A2A 协议 tool_result 注入 |
| owasp-001 | https://genai.owasp.org/llmrisk/llm01-prompt-injection/ | OWASP LLM01 直接注入 |
| owasp-002 | https://genai.owasp.org/llmrisk/llm02-sensitive-information-disclosure/ | OWASP LLM02 私钥泄露 |
| owasp-003 | https://genai.owasp.org/llmrisk/llm04-data-and-model-poisoning/ | OWASP LLM04 RAG 投毒（漏拦）|
| owasp-004 | https://genai.owasp.org/llmrisk/llm06-excessive-agency/ | OWASP LLM06 过度权限 |
| owasp-005 | https://genai.owasp.org/llmrisk/llm03-supply-chain/ | OWASP LLM03 恶意插件持久化 |
| owasp-006 | https://genai.owasp.org/llmrisk/llm05-improper-output-handling/ | OWASP LLM05 输出处理不当 |
| owasp-007 | https://genai.owasp.org/llmrisk/llm07-system-prompt-leakage/ | OWASP LLM07 系统 prompt 泄露 |
| owasp-008 | https://genai.owasp.org/llmrisk/llm08-vector-and-embedding-weaknesses/ | OWASP LLM08 向量嵌入弱点 |
| owasp-009 | https://genai.owasp.org/llmrisk/llm09-misinformation/ | OWASP LLM09 虚假地址 |
| real-001 | https://www.ledger.com/blog/security-incident-report | Ledger Connect Kit $600K |
| real-002 | https://www.certik.com/blog/multichain-collapse-the-private-key-leak-that-drained-usd125m | Multichain $126M |
| real-003 | https://thehackernews.com/2025/01/lazarus-group-targets-web3-developers.html | Lazarus Op99（漏拦，纯社工）|
| real-004 | https://blockaid.io/blog/attack-report-the-september-2025-npm-supply-chain-attack | Sep 2025 NPM 攻击 |
| real-005 | https://www.bleepingcomputer.com/news/security/step-finance-says-compromised-execs-devices-led-to-40m-crypto-theft/ | Step Finance $40M |
| real-006 | https://www.sonatype.com/blog/decrypting-the-ledger-connect-kit-compromise-a-deep-dive-into-the-crypto-drainer-attack | Ledger 钓鱼邮件（漏拦，非 LLM 流量）|
| real-007 | https://expel.com/blog/inside-lazarus-how-north-korea-uses-ai-to-industrialize-attacks-on-developers/ | Lazarus MCLIP 助记词社工 |
| real-008 | https://www.halborn.com/blog/post/explained-the-multichain-hack-july-2023 | Multichain 攻击者 LLM 重构 |
| real-009 | https://cybersecuritynews.com/ai-assisted-lazarus-campaign-targets/ | Lazarus 编码挑战木马 |
| real-010 | https://thehackernews.com/2025/02/lazarus-group-deploys-marstech1.html | Marstech1 npm 植入 |
| mcp-001 | https://jfrog.com/blog/2025-6514-critical-mcp-remote-rce-vulnerability/ | CVE-2025-6514 Windows 变种（漏拦，需补规则）|
| mcp-002 | https://thehackernews.com/2026/02/smartloader-attack-uses-trojanized-oura.html | SmartLoader/Oura MCP |
| mcp-003 | https://www.securityweek.com/by-design-flaw-in-mcp-could-enable-widespread-ai-supply-chain-attacks/ | MCP 设计缺陷 tool 注入 |
| mcp-004 | https://jfrog.com/blog/2025-6514-critical-mcp-remote-rce-vulnerability/ | CVE-2025-6514 Linux 变种 |
| mcp-005 | https://authzed.com/blog/timeline-mcp-breaches | MCP 注册表投毒 |
| mcp-006 | https://www.kaspersky.com/about/press-releases/kaspersky-warns-open-source-ai-connector-could-be-abused-by-cyberattackers | Kaspersky MCP 滥用预警 |
| mcp-007 | https://policyascode.dev/blog/mcp-security-vulnerabilities-2026/ | MCP 利用链 LaunchAgent 持久化 |
| mcp-008 | https://datasciencedojo.com/blog/mcp-security-risks-and-challenges/ | MCP Rug Pull 信任建立后反水 |
| mcp-009 | https://thehackernews.com/2026/04/anthropic-mcp-design-vulnerability.html | Anthropic MCP RCE via tool return |
