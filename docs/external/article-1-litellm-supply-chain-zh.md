---
title: 中转站揭黑——你用的 LLM 中转站可能在改你的 tool_call
author: doskey
draft: true
target_publish: 2026-W12（GA 同步发）
target_channels: Twitter long-form / Mirror / doskey.dev / Hacker News (English version)
length: 约 4200 字
last_updated: 2026-04-29
---

# 中转站揭黑：你用的 LLM 中转站可能在改你的 tool_call

> **English abstract for international readers:**
> On March 24, 2026, LiteLLM versions 1.82.7 and 1.82.8 were briefly published to PyPI with malicious code injected at the `__init__` stage. The attacker exploited trusted distribution infrastructure — not LiteLLM's own logic. If you proxy Anthropic or OpenAI API calls through any third-party relay, you have exactly the same attack surface. LLM traffic is not encrypted end-to-end from your IDE to the model. A tampered relay can silently rewrite the `tool_call` your AI agent produces — swapping an Ethereum address, injecting a malicious shell command, or dropping a safety check. Wallets don't see your prompts. LLM security products don't know what a seed phrase looks like. Sieve is a fully local LLM traffic proxy that inspects every byte in both directions, on your machine, before anything leaves or arrives.

---

## 2026-03-24，LiteLLM 投毒

2026 年 3 月 24 日，PyPI 上的 LiteLLM 1.82.7 和 1.82.8 带毒上架了。

攻击者在包的 `__init__.py` 里植入了代码，在导入阶段就会执行——不需要你主动调用任何函数。只要你的环境里装了这两个版本，启动时就已经中招。

影响范围：所有用 LiteLLM 做中转的服务——自建的 API 代理、公共 relay 服务、公司内部的 LLM 网关。在官方响应修复并发布 1.82.9 之前，用户毫无感知。

这不是 LiteLLM 的锅，是供应链的锅。问题在于：你信任了它所在的供应链。

这只是开始。

---

## 中转站的位置，决定了攻击面的大小

先说清楚什么是"中转站"。

广义上讲，任何夹在你的工具（Claude Code、Cursor、自写 agent）和 Anthropic / OpenAI 之间的东西都算：

- 自建的 LiteLLM 服务
- 公共 API relay（提供"便宜 Claude 额度"的那种）
- 公司内网的 AI Gateway
- 团队共享的 OpenRouter 账号代理

这些服务有一个共同点：**它们看得到明文流量**。

Anthropic 的 API 走 HTTPS，对外部网络是加密的。但 HTTPS 终止在你和中转站之间——中转站拿到的是完整明文请求和响应。你的 prompt、模型返回的代码、`tool_use` 调用里的参数，一字不差。

理论上它们只是透传。实际上，你有没有验证过？

---

## 历史不止一次

如果你觉得 LiteLLM 投毒是偶然，看几个前例：

**2024-12-02，@solana/web3.js 投毒**

攻击者入侵了 npm 发布账号，在 1.95.6 和 1.95.7 版本里注入了私钥窃取代码。这两个版本在 npm registry 上存活了约 5 小时。影响范围包括所有在这段时间内跑 `npm install` 的 Solana 开发环境。被盗资产估计在数万美元量级。

代码审查没有发现，因为恶意代码用了混淆。CI 没有发现，因为测试不跑安全扫描。用户没有发现，因为库的行为表面上没变。

**Pink Drainer EIP-712 绕过**

Pink Drainer 是一个以服务形式出售的 drainer 工具包（drainer：专为钓鱼站设计的智能合约，诱骗用户签署授权后自动掏空钱包）。其中有一个手法：EIP-712 结构化签名的 `verifyingContract` 字段不写 0x 地址，而是写十进制数字——比如 `996101...`。

MetaMask 在展示确认弹窗时，展示的是原始格式。用户看到一串数字，以为是金额或 nonce，直接签了。签完之后合约执行，资产转走。

这个攻击在 2024 年的钓鱼活动中造成了数百万美元损失（链上可查）。

**2026-04 UCSB + Fuzzland 论文**

arXiv:2604.08407，"Your Agent Is Mine"。研究者系统性地演示了如何通过在 LLM agent 的响应流里注入特定内容，劫持 agent 的工具调用序列。论文里有一节专门讨论了中转站作为攻击面的可能性。

三件事，三条路，都指向同一个结论：LLM 流量是攻击面，而且这个攻击面没有人在看守。

---

## 真实的攻击场景长什么样

我来描述一个具体场景，不是假设，而是技术上完全可行的：

你在用 Claude Code 写一个 DeFi 相关的脚本。需要从合约读取数据，然后发送转账。你告诉 Claude：

> "帮我写一个 ethers.js 脚本，从 Uniswap 读取我的 LP 仓位，然后转移到我的新地址 `0xPLACEHOLDER1234A`"

Claude Code 把这个请求发出去。请求经过你使用的中转站，转发到 Anthropic。

模型返回了代码。代码里有一行：

```js
const to = "0xPLACEHOLDER1234B"; // 注意：最后一个字符从 A 变成了 B
```

你眼睛扫了一眼地址，感觉对，因为前面的字符串是一样的。你运行了脚本。转账发出去了。

链上不可逆。

这需要中转站主动作恶吗？不一定。也可能是：
- 中转站用的某个依赖被投毒了
- 中转站的服务器被入侵了
- 中转站的某个员工在响应里动了手脚

从你的角度，你验证不了响应是原始的还是被改过的。模型签名这个概念目前不存在于 Anthropic 的 API 协议里——你收到的 SSE 流（Server-Sent Events，流式 API 响应格式），没有任何加密完整性保证。

中转站可以在字节流里改一个字符，你不会知道。

---

## 为什么现有工具都救不了你

这里我不是要黑任何产品，它们在各自的场景下都有价值。只是它们解决的不是这个问题。

**钱包扩展（MetaMask、Rabby 等）**

钱包在浏览器里，它看的是你最终提交到钱包的签名请求。在 Claude Code 的工作流里，签名调用在代码里，在脚本里，在 shell 里——钱包看不见这个层级的流量。更不用说你可能在 headless 环境或服务器上跑脚本。

**LLM 安全产品（Cloudflare AI Gateway、Lasso Security、Prompt Shield）**

这些产品主要做 prompt injection 检测、越狱防御、PII（个人信息）过滤。它们不懂 crypto：不会识别 BIP39 助记词（BIP39：一种用 12-24 个英文单词编码私钥的标准），不会检测 EVM 地址替换，不会解析 EIP-712 typed data 的 `verifyingContract` 字段。这不是它们的目标市场，不是它们的训练数据，也不是它们的检测规则。

**DLP（Data Loss Prevention，数据防泄漏）**

DLP 通常部署在企业网关或 CASB（云访问安全代理）层。个人开发者的 laptop 上没有，Cursor 的工作流里没有，Claude Code 的 hooks 里没有。即使有，DLP 规则通常针对身份证、银行卡号等合规场景，不覆盖 Solana private key 或 Ethereum keystore JSON。

**链上监控（Chainalysis、Nansen 等）**

交易已经发出来才能监控到。这时候你的钱已经不在了。事后告诉你"这是已知 drainer 地址"帮不上忙。

这是一个覆盖盲区：没有任何现有工具坐在 LLM 流量里，同时懂 crypto。

---

## Sieve：客户端最后一道闸

我做了 Sieve。一句话定位：**完全本地运行的 LLM 流量代理 + native GUI 守门人**。

夹在 Claude Code 和 Anthropic API 之间。在你的机器上。不上传任何数据。

三件核心的事：

### 1. 出站脱敏：帮你擦屁股，不打断你工作

你向 Claude Code paste 了整个 `.env` 文件来调试一个问题。`.env` 里有 Infura key、Ethereum 私钥、AWS access key。

Sieve 在字节流层面扫描，识别并替换这些内容，然后再把请求发出去。Claude 收到的是脱敏版。5 秒钟菜单栏通知，你不需要停下来。

BIP39 助记词不只做词表匹配——Sieve 做 SHA-256 校验位验证。只有校验位通过的助记词才触发拦截。这个细节让误报率显著低于简单的词表扫描。

### 2. 入站地址替换检测：1 字符差异也能抓住

Sieve 记录你 prompt 里出现的 EVM 地址。当模型返回代码时，把返回里的地址和你原始 prompt 里的地址逐一比较，用 Levenshtein 编辑距离（两个字符串之间的最小编辑操作数，1 = 差一个字符）量化差异。

差异存在但不是 0？打断 SSE 流，弹 GUI 窗口：

![场景 B 地址替换 GUI 弹窗](placeholder-screenshot-address-swap.png)

```
┌──────────────────────────────────────┐
│  ⚠  Sieve 检测到地址替换            │
│                                      │
│  prompt 中的地址:  0xPLACEHOLDER1234A │
│  模型返回的地址:  0xPLACEHOLDER1234B │
│  编辑距离: 1                         │
│                                      │
│  ⏰ 60 秒倒计时（超时默认拒绝）      │
│                                      │
│  [中止请求]       [我已核对，继续]   │
└──────────────────────────────────────┘
```

你不点"继续"，响应就不放行。60 秒没有动作，默认拒绝。

### 3. 签名调用 fail-closed：不点弹窗就不执行

当模型返回 `tool_use signTransaction` 或 `signTypedData` 调用时，Sieve 强制弹 GUI 窗口，展示完整的 typed data 参数，解析 `verifyingContract`，标注已知 drainer 特征。

120 秒倒计时，超时默认拒绝。没有"YOLO 模式可以跳过 Critical 检测"这个选项。

这不是用户偏好，这是产品安全承诺。

---

## 我不会成为新的供应链风险

你可能已经想到了：等等，Sieve 本身也夹在流量中间，它怎么保证自己不是新的问题？

这个问题我必须正面回答。

Sieve 的核心引擎在 Week 12 GA（全面发布）后开源，MIT 许可。你可以读代码，可以 audit，可以自己编译。

二进制发布使用 sigstore 签名 + reproducible build（可复现构建）。这意味着：你不只是信任我，你可以验证你下载的二进制和 GitHub 上的源代码 byte-for-byte 一致。任何篡改都会导致签名验证失败。

sigstore 是 Linux Foundation 主导的开源签名基础设施，已被 Python、npm、Maven 等主流生态广泛采用。这套机制是 Sieve 从第一周就强制跑通的，不是 GA 前最后才加的装饰。

规则更新走签名校验，不会静默修改。更新日志公开。

你用的中转站有这些吗？

---

## 怎么验证你现在的中转站是否可信

如果你现在用着第三方中转，有几件事可以做：

**最低限度：锁版本**

不要跑 `pip install litellm` 然后期待它永远是同一个东西。锁 hash。

```
# requirements.txt 或 pip-tools
litellm==1.82.9 --hash=sha256:你从官方发布页拿到的实际 hash
```

**稍好一些：pin 依赖 + CI 里跑 `pip-audit`**

`pip-audit` 会检查你的依赖树是否有已知 CVE（公开漏洞编号）。不能防 0day，但能抓住已知问题。

**真正的解决方向：把中转拿掉**

如果你直接调用 Anthropic 官方 API，中间没有第三方中转，攻击面就少了一层。代价是你不能用各家混合调度。

你能做到吗？今天可以，明天 agent 变复杂之后不一定。

---

## 这不是让你停止用 AI 工具

我写这篇文章不是为了制造恐慌，也不是说你今天用的工具都是不安全的。

大多数中转站是无辜的。LiteLLM 投毒事件里，LiteLLM 团队的响应速度是合理的。@solana/web3.js 事件的根本原因是账号安全，不是代码安全。

问题在于：你无法提前知道哪次是有问题的那次。

供应链攻击的特征是低概率、高损失、事前无感知。加密资产的特征是不可逆。这两个特征叠加，期望损失远超大多数人以为的风险。

你每天用 Claude Code 写 DeFi 相关的代码，你的工作流里有多少个"我信任它透传"的节点？

Sieve 做的是把这些节点里，最后一个放进 AI 生成结果之前的节点，交还给你控制。

---

## 试试看

Sieve 定价 $49/月，14 天免费试用，不需要信用卡。

- 官网 + 下载：sieve.doskey.dev（注：Week 12 GA 前仍为 waitlist，注册可以拿早期访问）
- 开源核心引擎：github.com/doskey/sieve（GA 后公开）
- Twitter/X：@doskey

如果你现在不需要，但认识做 DeFi 开发的朋友，把这篇文章转给他。

文章 2 我会写 Sieve 自己是怎么防供应链风险的，比"相信我们"更具体的机制。文章 3 是一次地址替换 drainer 攻击的全链路复盘。

---

## 参考资料

1. LiteLLM PyPI 供应链事件（2026-03-24）— GitHub Advisory / PyPI 安全团队公告
2. @solana/web3.js npm 投毒事件（2024-12-02）— Solana Foundation 官方声明：https://solana.com/news/2024-12-02-web3js-compromise
3. "Your Agent Is Mine: Compromising AI Agents through Tool API Attacks" — arXiv:2604.08407（UCSB + Fuzzland，2026-04）
4. Pink Drainer EIP-712 数字化绕过分析 — Revoke.cash 博客（2024）
5. sigstore 项目 — https://www.sigstore.dev/
6. pip-audit — https://pypi.org/project/pip-audit/
7. BIP39 规范（助记词 + 校验位） — https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
8. EIP-712 Typed Structured Data Hashing — https://eips.ethereum.org/EIPS/eip-712

---

*doskey，2026 年 4 月*

*我在 Hacker News / Twitter 上用同一个 ID。有技术细节想讨论，直接 @ 我。*
