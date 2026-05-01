# 公开攻击复现样本索引

生成日期：2026-05-01
方案：C（公开对抗复现）
来源说明：每条样本均有可追溯的公开 URL / 报告名 / CVE 编号，禁止虚构。

## 目录结构

| 子目录 | 描述 | 样本数 |
|--------|------|-------|
| rugpull-ai/ | AI agent rugpull / 钱包被掏空事件复现 | 9 |
| injection-pocs/ | 公开 prompt injection PoC 复现 | 10 |
| ctf-replays/ | CTF / 安全研究 PoC 复现 | 8 |
| owasp-llm-top10/ | OWASP LLM Top 10 各类（crypto 场景） | 9 |
| real-events/ | 真实历史事件 LLM 复述 | 10 |
| mcp-supply-chain/ | MCP 供应链攻击复现 | 9 |

总计：55 条

## 来源清单

### 真实事件来源
- Ledger Connect Kit 攻击（2023-12-14）：https://www.ledger.com/blog/security-incident-report
- SlowMist 分析：https://slowmist.medium.com/supply-chain-attack-on-ledger-connect-kit-analyzing-the-impact-and-preventive-measures-1005e39422fd
- Multichain 私钥泄露（2023-07-06）：https://www.certik.com/blog/multichain-collapse-the-private-key-leak-that-drained-usd125m
- Step Finance $40M 事件（2026-01-31）：https://www.bleepingcomputer.com/news/security/step-finance-says-compromised-execs-devices-led-to-40m-crypto-theft/
- Lazarus Operation 99（2025-01-09）：https://thehackernews.com/2025/01/lazarus-group-targets-web3-developers.html
- September 2025 NPM 供应链攻击：https://blockaid.io/blog/attack-report-the-september-2025-npm-supply-chain-attack

### MCP 漏洞来源
- CVE-2025-6514 mcp-remote RCE：https://jfrog.com/blog/2025-6514-critical-mcp-remote-rce-vulnerability/
- SmartLoader / Oura MCP 木马事件（2026-02）：https://thehackernews.com/2026/02/smartloader-attack-uses-trojanized-oura.html
- MCP 供应链攻击设计缺陷：https://www.securityweek.com/by-design-flaw-in-mcp-could-enable-widespread-ai-supply-chain-attacks/

### 学术研究 / PoC 来源
- arXiv 2503.16248 ElizaOS context manipulation：https://arxiv.org/abs/2503.16248
- Palo Alto Unit 42 野外间接 prompt injection：https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/
- OWASP LLM Top 10 2025：https://owasp.org/www-project-top-10-for-large-language-model-applications/
- Oasis Security Claude.ai prompt injection exfil：https://www.oasis.security/blog/claude-ai-prompt-injection-data-exfiltration-vulnerability
- Lasso Security Claude Code IDPI：https://www.lasso.security/blog/the-hidden-backdoor-in-claude-coding-assistant

### AI Agent 恶意路由来源
- Coinspeaker 恶意 LLM 路由攻击报告：https://www.coinspeaker.com/malicious-ai-agent-routers-steal-crypto-attack-vector/amp/
- KuCoin AI Trading Agent 漏洞 2026：https://www.kucoin.com/blog/en-ai-trading-agent-vulnerability-2026-how-a-45m-crypto-security-breach-exposed-protocol-risks
