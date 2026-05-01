# Sieve 产品需求文档 v2.0

> **codename: Sieve**（产品正式名待定）
> 文档版本: v2.0 / 2026-05-01
> 文档主人: doskey
> 状态: HIPS 改造版（v1.5 multi-agent → v2.0 完整 HIPS），锁定执行
> 与 v1.5 差异: 5 项 HIPS 能力改造（可编程 policy / 三态 ask / 行为序列 / 规则引擎抽象 / 拦截引擎抽象），影响 §5 §6 §9 §10 章节
> 引擎复用: ✅ sieve-rules vectorscan + allowlist_stopwords 全文搜索 / 二维处置矩阵 / fail-closed critical_lock / 双层防御 ADR-014 / sub-agent X-Sieve-Origin 100% 复用 v1.5
> Phase 拆分: **Phase A（Week 5-8 ship）** + **Phase B（Week 9-12 ship MVP）**，GA 时间表 Week 12 不变

---

## 0. v1.5 → v2.0 修订说明

v1.5 在 2026-04-28 锁定 multi-agent 扩展（Claude Code + OpenClaw + Hermes 三家适配）后，doskey 在 2026-05-01 完成 v1.5.1 ~ v1.5.4 共 4 次规则集扩充（35 → 70 入站规则 + 1951 测试样本 + 修 Week 4 P0 非流式 JSON 入站绕过漏洞）。基于 [HIPS Readiness Assessment](../review/2026-05-01-hips-readiness-assessment.md) 评估结论 "**Sieve 当前是 HIPS 70%**"，v2.0 启动**完整 HIPS 改造**，目标 GA 时达到 **HIPS 90%**。

### 触发原因

[HIPS Readiness Assessment](../review/2026-05-01-hips-readiness-assessment.md) 列出 14 项经典 HIPS 标准对照，Sieve 满足 8 项 / 部分 4 项 / 不满足 2 项 = **70%**。距离合格 HIPS（CrowdStrike Falcon / Microsoft Defender ATP / OSSEC 级别）还差 **5 项关键能力**：

1. **可编程 policy 引擎**（不满足）：用户不能加自定义规则，所有规则项目内置 TOML
2. **三态决策 + 灰名单**（部分）：当前只有 allow/deny，缺持久化 ask 状态对接 GUI
3. **行为序列联动**（部分）：当前只看单次工具调用，无进程上下文 / 无序列窗口，逃逸面大
4. **规则引擎抽象**（部分）：vectorscan 引擎与 sieve-cli engine_adapter 耦合，难独立测试 / 压测
5. **拦截引擎抽象**（部分）：daemon 与 macOS / Claude Code hook 耦合，多平台难复用

5 项里 4 项 v1.5 里都"部分满足"（不是从零开始）—— v2.0 的工作是把"部分"做到"满足"。

### v1.5 → v2.0 改动汇总

| 改动 | 章节 | Phase | 影响范围 |
|------|------|-------|---------|
| 用户规则系统（仅 allow 豁免，不可加 Critical block） | §5.5 新增 | A (Week 5-8) | 中：新增 sieve-policy crate |
| 三态决策 ask（持久化灰名单，对接 GUI 接口）| §5.4 重写 | A (Week 5-8) | 小：扩展 IPC 协议 |
| 规则引擎抽象（独立 trait，可测试可压测）| §6.3 重写 | A (Week 5-8) | 中：crate 边界调整 |
| 进程上下文记录（caller_pid/exe/cwd 加 audit）| §5.6 新增 | A (Week 5-8) | 小：audit schema 加字段 |
| 行为序列窗口（InboundFilter 维护过去 N 次 tool_use 序列）| §5.7 新增 | B (Week 9-12 MVP) | 大：InboundFilter 重构 |
| 6 个 ADR（policy 加载 / 三态 ask / 序列窗口 / 进程上下文 / 规则引擎 trait / 灰名单存储） | §9 + ADR | A+B | 小：文档 |
| §9 硬约束新增 #14 #15 | §9 新增 | A | 小：约束声明 |
| Week 5-12 里程碑重写按 Phase A/B | §10 重写 | A+B | 中：roadmap |

### v2.0 没改的（明确说明）

- **§1 产品定位**：不变（"完全本地运行的 LLM 流量代理 + native GUI 守门人"）
- **§7 商业模式 / §8 数据飞轮 / §11 法律 / §12 风险登记**：不变（HIPS 改造不动定价 / 法律 / 数据合作）
- **§13 数据合作**：不变
- **PRD §9 硬约束第 1-13 条**：**全部保留**（详见 §9）—— v2.0 改造**绝不能违反任何一条**，特别是：
  - #1 Rust 栈非选项（用户规则也是 Rust + TOML 解析，不引入脚本语言运行时）
  - #2 绝不联网做 verifier（用户规则签名验证全本地）
  - #3 fail-closed Critical 不可关（用户规则不能 override 内置 Critical）
  - #11 不在协议层撒谎（行为序列产物用 sieve_blocked event 自报，不冒充 model）
  - #12 不装本地 CA 做 MITM（拦截引擎抽象仍走 reverse proxy 模式）

### Phase 拆分原则

| Phase | Week | 内容 | 工程影响 | GA 风险 |
|-------|------|------|---------|--------|
| **A** | Week 5-8 | 用户规则 + 三态 ask + 规则引擎抽象 + 进程上下文记录（仅记不分析） | 中（不动 daemon 核心 pipeline） | 低 |
| **B** | Week 9-12（与闭测并行）| 行为序列窗口 **beta flag 默认关闭**（3 条 IN-SEQ-* 启发式 + StatusBar 通知，闭测用户主动 opt-in）| 低-中（InboundFilter 加序列窗口，daemon.rs 不动，beta flag 控制启用）| 低 |

**GA 时间表 Week 12 不变**。Phase B 在闭测期间渐进 ship（"5% 用户灰度"做法），完整版本在 GA 后 v2.1 补完。

> **拦截引擎抽象（trait + macOS impl，预留 Linux/Windows）原列 v2.0 Phase B**，review 后认定为"工程整洁度需求，非 HIPS 标准核心"——v2.0 第一版只做 macOS，daemon.rs 当前形态就是事实上的 MacOSInterceptor，强行抽象成 trait 收益要 v2.1+ 多平台时才用得上，且 Phase B 与闭测并行重构 daemon.rs 风险高。**移到 v2.1 评估**（详见 §14 OQ-V20-07）。

---

## 1. 产品定位

### 1.1 一句话

**Sieve 是一个完全本地运行的 LLM agent HIPS MVP（Host-based Intrusion Prevention System for LLM agents），挂在 AI 编码 / 助手 agent 和上游模型之间做双向安全检测 + 进程行为联动。Phase 1 GA 适配三家：Claude Code（terminal coding agent）、OpenClaw（多通道消息网关）、Hermes Agent（multi-provider 编排器），服务 crypto 开发者、DeFi 重度用户、和把 LLM 接进生活流的 power user。**

> v1.5 → v2.0 关键词变化：**"代理"→"HIPS MVP"**，**"检测"→"检测 + 进程行为联动"**
>
> **关键澄清**（codex review #A1）：定位是 **"LLM agent 场景的本地 HIPS MVP"**，不是通用 endpoint HIPS（CrowdStrike Falcon / Microsoft Defender ATP / OSSEC 级别）。聚焦 LLM 流量的双边边界（agent ↔ model），不替代系统级 HIPS / EDR。营销话术不对齐通用 endpoint 厂商，避免抬高用户预期。

### 1.2 四句话核心叙事

不变（沿用 v1.5 §1.2）。

### 1.3 不是什么

沿用 v1.5 §1.3 全部条目，**新增**：

- **不是通用 HIPS**——Sieve 是 LLM 流量场景的 HIPS，不替代 macOS XProtect / Linux SELinux / Windows Defender。聚焦"AI agent 调用工具 / 模型回复" 这两个边界
- **不是规则市场 / app store**——用户可加私有规则但不接受社区上传规则到中心仓库（PRD §9 #2 不联网做 verifier 延伸）
- **不是 EDR**——不做 endpoint detection response，没有调查 / 取证 / 溯源能力（这些是 SOC 团队工具，与一人产品定位不符）

### 1.4 项目性质 + 法律实体

不变（沿用 v1.5 §1.4）。

---

## 2. 市场判断与时间窗

不变（沿用 v1.5 §2）。

**v2.0 补充观察（HIPS 定位相关）**：

- **2026 年 LLM 安全产品分两派**：(1) 云端 SaaS 检测（Lakera Guard / Nightfall AI / Robust Intelligence）；(2) 本地 wrapper（Sieve / Lasso Code Security 等）。Sieve 在 (2) 派别里**唯一定位 HIPS** 的—— Lasso 是 SAST + 静态分析，不做运行时拦截
- **HIPS 定位让 Sieve 的"差异化故事"硬一档**：从"yet another LLM safety tool"→"first HIPS for AI coding agents"，营销话术更尖锐
- **风险**：HIPS 定位也意味着用户期望对齐到 CrowdStrike Falcon 级别，**不能再有 v1.5.4 这种 P0 漏洞**——产品级 HIPS 一次绕过就被卸载

---

## 3. 用户画像

不变（沿用 v1.5 §3）。

**v2.0 用户规则系统服务 P0/P1 客群中"想再严一档"的高级用户**——不专门定义 P2 客群（v2.1 起草 $99/月 Pro 套餐时再讨论）。

---

## 4. 核心用户场景

沿用 v1.5 §4 全部场景 A-F，**新增**：

### 4.7 场景 G：高级用户写自定义规则（v2.0 新增）

**用户**：DeFi 协议 dev，已经付费 $49/月 6 个月，对 Sieve 信任建立后想"再严一档"

**触发**：发现 Sieve 没拦"模型在解释 EIP-712 typed data 时把 `deadline` 字段值改为 0（permit 永不过期）"，但这是自己项目里典型的 phishing 模式，希望 Sieve 拦

**期望流程**：
1. `sieve rules edit` —— 调用 `$EDITOR` 打开 user.toml，关闭后自动 lint + backup + reload
2. 用户在编辑器里写 TOML 规则（编辑器提供 schema 提示 / 实时 lint / 匹配预览 / 正则解释器）
3. 本地 lint 通过后写入 **统一用户规则文件 `~/.sieve/rules/user.toml`**（不是分散文件，编辑器自动备份原文件）
4. daemon reload，命中后弹窗注明 "命中**你的规则** `MY-EIP712-DEADLINE-ZERO`（2026-05-15 添加）"

> **AI 辅助生成**（让 LLM 帮用户从自然语言描述生成规则草稿）**v2.0 不做**，留 v2.1 评估（详见 §14 OQ-V20-05）。v2.0 编辑器只支持手写 TOML + lint + 预览，不引入云端 LLM 依赖。

**验收**：
- 用户对照编辑器里的规则示例 + 实时 lint 提示，30 分钟内能加第一条规则（含读文档）
- 用户规则**不能影响系统规则的拦截覆盖**（哪怕 user.toml 损坏，daemon 仍正常启动 + 系统规则全功能）
- 用户规则 FP 高，可一键 `sieve rules disable MY-EIP712-DEADLINE-ZERO` 关闭单条规则
- `sieve rules export` 可导出 user.toml 备份；`sieve rules import <file>` 一键导入（v2.1 用例：分享给团队成员）

---

## 5. 功能需求

沿用 v1.5 §5.1 ~ §5.3 全部检测项 + 处置矩阵。**新增 / 重写以下章节**：

### 5.4 三态决策（v2.0 重写，原 §5.4.1 处置矩阵升级）

#### 5.4.1 三态决策模型

每条规则命中后，daemon 决定一个 **Decision**：

```
Decision := Allow | Deny | Ask
```

- **Allow**：通过，不打断
- **Deny**：阻断，对应 Action::Block
- **Ask**：等待用户决策，阻塞调用直到用户响应或超时

**v1.5 现状**：`Disposition` 枚举含 `GuiPopup` / `HookTerminal` 两种"等待决策"形态，但没明确建模 ask 状态——用户 allow 后立即过去，没有"永久允许"的灰名单概念。

**v2.0 三态扩展**：

| Decision | 持久化形态 | 触发场景 | Remember 是否允许 |
|----------|----------|---------|------------------|
| **Allow** | 1 次性（不持久） | 系统规则评估为非 Critical 或匹配 allowlist_stopwords | N/A |
| **Allow + Remember** | 写 `~/.sieve/decisions/<digest>.json` 灰名单 | 用户在 GUI 弹窗选"永久允许此次场景" | **仅非 Critical 系统规则 + 用户规则可 Remember** |
| **Deny** | 1 次性（不持久） | 系统规则评估为 Critical + 无用户决策 | N/A |
| **Ask (Hook 类)** | 走 sieve-hook 终端 y/n（不持久）| Hook 类规则命中（IN-CR-02~04）| 内置 Critical 不可 Remember |
| **Ask (GUI 类，非 Critical)** | 走 GUI 120s 弹窗（可选 Remember）| 用户规则 / IN-GEN-04 | ✅ |
| **Ask (GUI 类，Critical)** | 走 GUI 120s 弹窗（**禁用 Remember 选项**）| IN-CR-01 / IN-CR-05 等内置 Critical | ❌ |

#### 5.4.2 灰名单存储 schema（v2.0 新增，含 Critical 锁）

`~/.sieve/decisions/` 目录下每条灰名单一个 JSON 文件，文件名用 fingerprint hex digest（不直接暴露 rule_id）：

```json
{
  "schema_version": 1,
  "fingerprint_version": 1,
  "rule_id": "IN-GEN-04",
  "rule_version": "v1.5.4",
  "fingerprint": "sha256_64_hex_chars",
  "fingerprint_inputs": {
    "rule_id": "IN-GEN-04",
    "matched_canonical": "<canonical 形态：去空白、统一大小写后的命中片段>",
    "tool_name": "Bash",
    "protocol": "anthropic",
    "content_kind": "tool_use_input",
    "source_agent": "claude-code"
  },
  "decision": "allow",
  "expires_at": null,
  "added_at": 1745683210000,
  "added_by": "gui_user_decision",
  "context_hint": "用户输入的备注（GUI 表单）",
  "match_count_since": 0,
  "audit_event_id": "uuid-v4"
}
```

**Critical 锁约束**（PRD §9 #3 + #8 + #14 fail-closed 延伸）：
- daemon 在写灰名单前**必须验证 rule_id 不在 critical_lock.rs::FAIL_CLOSED_RULES**
- GUI 在内置 Critical 弹窗时**必须禁用 Remember checkbox**（IPC 协议 `request_decision.allow_remember=false`）
- 用户规则 + 非 Critical 系统规则 + IN-GEN-04 标记类规则**才允许** Remember
- 任何形式绕过 Critical 锁视为安全漏洞（与 v1.5.4 P0 同级）

**灰名单查询**：daemon 在规则命中后、Decision 决策前查 `decisions/<digest>.json`，存在则跳过弹窗直接 Allow。查询前重新计算 fingerprint 验证一致性，防止文件被人为编辑。

**灰名单文件权限**：`0600`（owner-only），目录 `0700`，atomic rename 写入，no-follow symlink。

**所有灰名单变更（add/remove/expire）必须写 audit.db**（kind=`grayllist_added` / `grayllist_removed`），含 fingerprint + rule_id + 用户决策时间。

#### 5.4.3 GUI 接口预留（v2.0 新增）

IPC 协议（ADR-013）扩展：
- `sieve.request_decision` JSON-RPC 消息加 `allow_remember: bool` 字段。**daemon 根据 critical_lock 计算该值**——内置 Critical 时强制 false，不让 GUI 决定。
- `sieve.decision_response` 加 `remember: bool` + `context_hint: Option<String>`。daemon 收到 `remember=true` 时**二次校验** rule_id 是否允许 Remember；不允许则忽略 remember 字段 + 写 audit ERROR。

GUI 端实现细节不在 PRD 范围（属 sieve-gui-macos 仓库），但**强约束**：内置 Critical 弹窗的 Remember checkbox 必须 disabled+灰显，并在 tooltip 解释"内置 Critical 规则保护核心安全场景，不允许永久绕过"。

### 5.5 用户规则系统（v2.0 新增）

#### 5.5.1 用户规则文件位置（v2.0 单文件 schema）

```
~/.sieve/rules/
├── user.toml                   # 统一用户规则文件（所有规则集中于此）
└── user.toml.bak.YYYYMMDD-HHMMSS  # 编辑前自动备份（最近 10 份）
```

> **设计理由**：v1 草案曾考虑 `user-001.toml` / `user-002.toml` 多文件方案，但实测用户分散文件易丢失上下文（"我哪条规则在哪个文件"）。v2.0 决定单文件 + 备份，规则之间可视化关联性更强、编辑器实现更简单。

#### 5.5.2 子命令清单（v2.0 Phase A）

| 命令 | 行为 |
|------|------|
| `sieve rules edit` | 调用 `$EDITOR`（fallback vim/nano）编辑 user.toml，关闭后 lint + atomic backup + reload；TUI / GUI 推 v2.1 |
| `sieve rules list` | 列出 user.toml 中所有规则 + 系统规则（合并视图，标注来源） |
| `sieve rules disable <id>` | 在 user.toml 给该规则加 `enabled = false`（不删除）|
| `sieve rules enable <id>` | 反向操作 |

> v2.1 评估增加：`lint`（独立校验）/ `import` / `export`（团队分享）/ `reload`（无 edit 上下文时手动触发）。Phase A MVP 只 ship 上述 4 个核心子命令，其他能力合并到 `edit` 内或推后

#### 5.5.2.1 用户规则与系统规则并存原则（v2.0 重写）

| 维度 | 系统规则 | 用户规则 |
|------|---------|---------|
| 来源 | `crates/sieve-rules/rules/{inbound,outbound}.toml`（编译进二进制）| `~/.sieve/rules/user.toml`（运行时加载） |
| **可表达 severity / action** | 全套（含 Critical/Block/HookTerminal）| **只能 High / Medium / Low + Ask / Warn / Mark**（不能 Critical / Block / HookTerminal）|
| 合并顺序 | **先评估，命中 Critical 立即 Block**（不给用户规则机会 suppress）| 系统规则未命中或全 Allow 后才评估；命中只能加 Ask 弹窗 / StatusBar 通知 |
| 灰名单 Remember | 仅非 Critical 允许（见 §5.4.2 Critical 锁）| 完全允许 |
| 加载失败时 | daemon 启动失败（CI gate）| 跳过该文件 + 写 ERROR 日志 + audit 事件 + GUI 状态栏通知，daemon 正常启动（PRD §9 #14）|
| 命中标识 | rule_id 直接（如 `IN-CR-02-CURL-PIPE`）| `user:<rule_id>`（前缀区分）|
| 通知文案 | "Sieve 系统规则 X 拦截"| "你的用户规则 X 触发（2026-05-15 添加）"|

**关键约束**（v2.0 强化）：
- 用户规则**不能 override 或 suppress 系统 Critical 命中** —— LayeredEngine 合并顺序保证（§6.3.1）
- 用户规则**不能与已有规则 ID 重复**（即使是禁用状态的规则）
- 用户规则**不能 reduce 系统规则的处置等级**（不能用 Allow 规则覆盖系统 Block）

#### 5.5.3 用户规则安全约束（v2.0 强约束 11 类）

为防止恶意 LLM 通过对话诱导用户加危险规则（如把 .env 加白），以及防止资源耗尽 / 文件系统攻击，用户规则**强约束 11 类**：

**A. 语义边界**（5 类，v2.0 初版已有）：

| 禁止组合 | 原因 |
|---------|------|
| `severity = "critical"` 或 `action = "block"` 或 `disposition = "hook_terminal"` | 用户规则不能阻断；只能 Ask / Warn / Mark / StatusBar |
| `pattern` 包含 `__` 前缀（系统占位符）| 防止用户绕开 second-pass 验证（BIP39 / address guard）|
| `id` 与系统规则冲突或与已有用户规则重复 | 防止 override / 重复加载 |
| `disposition = "auto_redact"` 用于入站方向 | 用户不能改写入站 model 输出（PRD §9 #11）|
| 任何 `allowlist_*` 字段试图豁免系统 Critical rule_id | LayeredEngine 验证：用户 allowlist 仅作用于自己的规则，不影响系统规则评估 |

**B. 资源上限**（3 类，v2.0 codex review 后补）：

| 限制 | 默认值 | 理由 |
|------|------|------|
| 单个 `pattern` 编译时间上限 | 100ms | 防止 ReDoS（Regex Denial of Service，正则拒绝服务）模式 |
| 单个 `pattern` 编译后 vectorscan db size 上限 | 1MB | 防止 alternation 爆炸 |
| user.toml 单文件大小上限 | 1MB | 防止恶意大文件 |
| user.toml 总规则条数上限 | 200 条 | 性能 + 用户认知边界 |
| 单个 `allowlist_stopwords` 字符串长度 ≥ 4 字节 | — | 防止超短停用词污染所有匹配 |
| `keywords` 字段必填且非空 | — | 强制启用 keywords 预过滤，避免 match-all pattern 拖慢扫描 |

**C. 文件系统安全**（3 类，v2.0 codex review 后补）：

| 约束 | 实施方式 |
|------|--------|
| 文件权限 `0600`（owner-only）| `sieve rules edit` 保存时强制设置 + daemon 启动时拒绝加载非 0600 文件 |
| 目录权限 `0700` | 同上 |
| **No-follow symlink**：`~/.sieve/rules/` 下任何符号链接拒绝加载 | 防止恶意 LLM 诱导用户 `ln -s /etc/passwd ~/.sieve/rules/user.toml` |
| **Atomic rename 写入**：先写 `user.toml.tmp` 再 rename | 防止编辑过程中 daemon reload 读到部分写入 |
| **TOCTOU 防护**：daemon reload 时持有文件锁 + 重新计算 inode/mtime | 防止 check-then-use 间隙文件被替换 |

**lint 时机**：
- `sieve rules edit` 保存前（拒绝违规写入）
- daemon 加载时（双重校验，防止用户绕过 edit 命令直接编辑文件）
- daemon reload 时（同上）

任何违规：写 audit ERROR + GUI 状态栏通知 + 该规则跳过加载，**绝不让 daemon 启动失败 / 系统规则跟着失效**（PRD §9 #14）。

#### 5.5.4 用户规则文件示例（user.toml）

```toml
# ~/.sieve/rules/user.toml
# v2.0 单文件 schema：所有用户规则集中于此

schema_version = 1
created_at = "2026-05-15T10:30:00Z"
updated_at = "2026-05-15T18:45:22Z"

[[rules]]
id = "MY-EIP712-PERMIT-DEADLINE-ZERO"
description = "EIP-712 permit deadline=0 永不过期（钓鱼模式）"
pattern = '''(?i)permit\s*\([^)]*\bdeadline\b\s*[:=]\s*0'''
severity = "high"           # 注意：用户规则禁止 critical+block 组合
action = "warn"
keywords = ["permit", "deadline"]
allowlist_stopwords = [
  "ERC-2612 specification",
  "interface definition",
]
disposition = "status_bar"  # 不打断
enabled = true
added_at = "2026-05-15T10:30:00Z"
added_by = "manual"         # "manual" | "imported"（v2.1 增加 "ai_assisted"）

[[rules]]
id = "MY-WALLET-DRAIN-FUNCTION-NAME"
description = "我项目里禁止出现的钓鱼合约函数名"
pattern = '''(?i)\b(claim|migrate|upgrade|verifyAndExecute)Wallet\b'''
severity = "medium"
action = "mark"
disposition = "status_bar"
enabled = true
added_at = "2026-05-16T09:12:00Z"
added_by = "manual"
```

#### 5.5.5 规则编辑器（v2.0 Phase A，调用 $EDITOR）

`sieve rules edit` **不实现独立 TUI 编辑器**，调用用户系统的 `$EDITOR`（fallback `vim` / `nano`）打开 `~/.sieve/rules/user.toml`，编辑器关闭后由 daemon 接手做 4 件事：

| 步骤 | 行为 |
|------|------|
| 1. **lint** | 完整执行 §5.5.3 全部 11 类约束校验 |
| 2. **backup** | 通过 lint 后，把原 `user.toml` rename 为 `user.toml.bak.YYYYMMDD-HHMMSS`（保留最近 10 份）|
| 3. **atomic write** | 把新内容写入 `user.toml.tmp` 再 rename 到 `user.toml`（atomic 不会读到部分写入）|
| 4. **reload** | IPC notify daemon 重新加载用户规则（失败保留旧 user engine，不影响系统规则）|

**lint 失败处理**：保留原 `user.toml` 不动，stderr 打印违规清单（带行号 + 原因），用户重跑 `sieve rules edit` 即可。

**Phase A MVP 优势**（codex review 后从 ratatui TUI 简化）：
- 不引入 ratatui 依赖（编译体积 + 跨终端兼容性问题）
- 用户用熟悉的编辑器（vim/nano/code）
- 工程量从 "TUI MVP（5-7 天）" 降到 "shell out + lint pipeline（1-2 天）"
- daemon-side reload 逻辑不变，只是 edit 入口换形态

> v2.1 评估增加：匹配预览（嵌迷你 vectorscan）/ 正则解释器（PCRE → 中文）/ 内置示例库 / TUI / GUI 编辑器。v2.0 用户从 `docs/guides/user-rules.md` 文档里看示例，自己复制改
>
> **AI 辅助生成不在 v2.0 范围**（v2.1 评估，详见 §14 OQ-V20-05）。v2.0 编辑器**完全离线**，不依赖任何云端服务。

### 5.6 进程上下文记录（v2.0 Phase A 新增）

#### 5.6.1 audit schema 扩展

`audit.db` events 表加 2 个字段：

```sql
ALTER TABLE events ADD COLUMN caller_pid INTEGER;
ALTER TABLE events ADD COLUMN caller_exe TEXT;       -- macOS proc_pidpath / Linux /proc/<pid>/exe
```

> v2.1 评估增加：`caller_cwd`（macOS 需 entitlements + 用户授权弹安全提示）+ `caller_ppid`（行为序列分析的进程树重建）。Phase A MVP 不引入这两个字段，避免部署摩擦

#### 5.6.2 数据来源

daemon 接受 HTTP 请求时，从 TCP 连接的 PID（macOS `lsof -i` / `proc_pidinfo` API）反查 caller 进程信息。

**仅记录，不分析**（Phase A）—— 给 v2.0 Phase B 行为序列分析喂数据。

#### 5.6.3 隐私 / 性能约束

- caller_exe 用本地文件路径（不上传，PRD §11 数据不上传）
- 反查失败（权限不足等）→ 字段为 NULL，daemon 不阻塞
- 反查耗时不能超过 1ms（用 LRU cache 缓存 PID → 进程信息映射）

### 5.7 行为序列联动（v2.0 Phase B beta 功能，**默认关闭**）

> **2026-05-01 codex review 后调整**：行为序列从"GA 必达卖点"降级为 **beta flag 默认关闭**。GA 时通过 `[features] sequence_detection = false` 默认关闭，闭测用户 + dogfood 用户主动 opt-in。GA 营销不承诺行为序列，按 dogfood 数据决定 v2.1 是否升级为默认开启。这样降低 Phase B 范围风险 + Week 12 GA 风险。

#### 5.7.1 序列窗口模型（结构化特征版，codex review 后重写）

`InboundFilter::SessionState` 加滑动窗口（默认 N=10 / TTL=5 分钟）：

```rust
struct ToolUseSequence {
    window: VecDeque<ToolUseRecord>,
    max_size: usize,                  // 默认 10
    expires_after_ms: u64,            // 默认 300_000（5 分钟）
}

/// 隐私安全的结构化特征（不存原始 input，用于序列模式匹配）
struct ToolUseRecord {
    timestamp_ms: u64,
    tool_name: String,                // "Bash" / "Read" / "Write" / "Edit"
    tool_class: ToolClass,            // 枚举：Shell | FileRead | FileWrite | Network | Other
    path_category: Option<PathCategory>,  // SensitiveSecret | Wallet | DotEnv | Code | Tmp | Other
    network_egress: bool,             // tool_input 含 curl/wget/POST 外发
    persistence_mech: bool,           // tool_input 含 systemctl/launchctl/crontab 等持久化
    cleanup_mech: bool,               // tool_input 含 rm / shred / history -c
    sensitive_file_hint: bool,        // tool_input 含 .env / id_rsa / keystore / mnemonic 等敏感关键词
    rule_hits: Vec<String>,           // 此次单次检测命中的规则 ID（可能为空）
}
```

**为什么改结构化特征**（v1 草案存 input hash）：
- 序列模式（如 `Read(.env)` → `curl POST`）需要识别**未命中单次规则的良性外观操作**——hash 无法识别 path/网络/持久化等特征
- 隐私 / 性能：仍不存原始 input，只存编译期定义的枚举特征
- 升级路径：v2.1 加 ML 分类器时直接用这些结构化特征做训练集

#### 5.7.2 启发式 kill chain 检测（v2.0 Phase B beta，3 条）

预设 3 个经典 LLM-against-crypto kill chain pattern，序列窗口里命中即触发 IN-SEQ-* 类规则（severity = High，disposition = StatusBar，**不阻断只通知**）：

| 规则 ID | 模式（基于结构化特征）| 描述 |
|---------|---------------------|------|
| `IN-SEQ-01-RECON-EXFIL` | `tool_class=FileRead + path_category=SensitiveSecret` 之后 `network_egress=true` | 读敏感文件 + 外发请求 |
| `IN-SEQ-02-CLEANUP-AFTER-ATTACK` | `tool_class=Shell + network_egress=true` 之后 `cleanup_mech=true` | 执行远程脚本后立即删痕迹 |
| `IN-SEQ-03-PERSISTENCE-CHAIN` | 3 次 `persistence_mech=true` 跨不同 tool 调用 | 多机制持久化（systemctl + launchctl + crontab）|

**v2.0 GA 仅 ship 这 3 个 + 框架**（且默认关闭）——更多模式 v2.1 按真实 dogfood 数据补。

#### 5.7.3 与 v1.5 单次检测的关系

行为序列**不替代**单次检测：

- 单次检测命中 Critical → 立即 Block（不等序列）
- 单次检测命中 High/Medium → 加入序列 + 单次按规则 disposition 处理（StatusBar / Mark）
- 序列触发 IN-SEQ-* → StatusBar 通知（"过去 5 分钟内有可疑动作链"）

序列检测**不引入新 Block 路径**（PRD §9 #15 硬约束）。

#### 5.7.4 双路径不变量（v2.0 codex review #E2 强化）

序列窗口更新点**必须同时覆盖** SSE 流路径 + 非流式 JSON 路径（v1.5.4 P0 教训）：
- `forward_with_inbound_inspection` SSE 聚合器命中 tool_use → 更新序列
- `handle_anthropic_json_inbound` JSON helper 解析 tool_use → **同样更新序列**
- `forward_with_openai_inbound_inspection` 两路径同款
- 集成测试矩阵覆盖：Anthropic SSE + JSON / OpenAI SSE + stream=false JSON 4 类组合都进入同一序列窗口

不满足该不变量视为 P0 漏洞（与 v1.5.4 同级）。

---

## 6. 技术架构

### 6.1 架构总览（v2.0 重写）

v1.5 五个 crate（sieve-core / sieve-rules / sieve-cli / sieve-ipc / sieve-hook）保留，**v2.0 新增 2 个 crate**（Phase A 加 1 个，Phase B 加 1 个）：

| crate | 职责 | 何时落地 | 禁做 |
|------|------|---------|-----|
| `sieve-core` | Pipeline / SSE Parser / UnifiedMessage / Forwarder / **SessionState 序列窗口（v2.0 Phase B）** | v1.5 已有，v2.0 扩展 | CLI / TUI / 配置加载 |
| `sieve-rules` | 规则定义 / vectorscan 编译 / 匹配引擎 / Ed25519 验证 / **trait 化（v2.0 Phase A）** | v1.5 已有，v2.0 重构 | 任何网络 IO |
| `sieve-cli` | 入口 / 配置 / 弹窗 / 审计日志（SQLite）/ **进程上下文反查（v2.0 Phase A）** | v1.5 已有 | 直接做规则匹配 |
| `sieve-ipc` | IPC 协议 / 文件锁 / pending/decisions 文件 IO / **三态决策 ask + 灰名单 schema（v2.0 Phase A）** | v1.5 已有，v2.0 扩展 | 不参与请求处理 |
| `sieve-hook` | 极简 PreToolUse hook 二进制 | v1.5 已有 | 不依赖 sieve-core / sieve-rules |
| **`sieve-policy`** | **用户规则加载 / lint / 与系统规则合并 / 灰名单管理 / 规则编辑器** | **v2.0 Phase A** | **不直接做匹配（调 sieve-rules）** |

跨 crate 调用走显式 trait，避免互相 import 实现细节。

### 6.2 v2.0 整体架构图

```
                    ┌────────────────────────────────────┐
                    │ AI Agent（Claude / OpenClaw / Hermes） │
                    └──────────────────┬─────────────────┘
                                       │ HTTP / SSE
                                       ▼
                    ┌────────────────────────────────────┐
                    │   sieve-cli daemon (macOS)          │
                    │   forward_with_inbound_inspection   │
                    │   forward_with_openai_inbound_*     │
                    │   v2.0 Phase A 加进程上下文反查      │
                    └──────────────────┬─────────────────┘
                                       │
                                       ▼
                    ┌────────────────────────────────────┐
                    │   sieve-core pipeline                │
                    │   ├── outbound (脱敏 / 拦截)         │
                    │   ├── inbound (检测 / 双层防御)      │
                    │   └── ToolUseSequence (Phase B 新)   │
                    └──────┬───────────────────┬─────────┘
                           │ trait MatchEngine │
                           ▼                   ▼
                    ┌──────────────┐    ┌──────────────────┐
                    │ sieve-rules  │    │  sieve-policy   │
                    │  系统规则     │    │  (Phase A 新)    │
                    │  Vectorscan  │◄───┤  用户规则加载    │
                    │  + Allowlist │    │  灰名单 / 编辑器  │
                    └──────────────┘    └──────────────────┘
                           ▲                   ▲
                           │                   │
                           └───┬───────────────┘
                               │
                               ▼
                    ┌────────────────────────────────────┐
                    │   sieve-cli engine_adapter          │
                    │   ├── BIP39 second-pass             │
                    │   ├── Address Guard                 │
                    │   ├── 进程上下文反查 (Phase A 新)    │
                    │   └── audit (SQLite append-only)    │
                    └──────────────────┬─────────────────┘
                                       │ IPC JSON-RPC + 文件锁
                                       ▼
                    ┌────────────────────────────────────┐
                    │  sieve-ipc + sieve-hook + GUI App   │
                    │  - GUI hold + 三态决策（v2.0 新）   │
                    │  - Hook 终端 y/n（v1.5 已有）        │
                    └────────────────────────────────────┘
```

> **拦截引擎抽象层移到 v2.1**：v2.0 daemon.rs 保持 v1.5 形态（事实上的 MacOSInterceptor），不引入 `sieve-interceptor` crate。v2.1 真要 Linux / Windows 时再做抽象（详见 §14 OQ-V20-07）

### 6.3 规则引擎抽象（v2.0 Phase A 新增）

#### 6.3.1 trait 设计

`sieve-rules/src/engine/mod.rs` 现有 `MatchEngine` trait（v1.5 已有），v2.0 扩展：

```rust
/// v2.0 新增：扫描请求上下文（codex review #C3 后从 &[u8] 升级）
pub struct ScanRequest<'a> {
    pub bytes: &'a [u8],

    // 路由上下文（v1.5.4 P0 教训：必须传递 protocol + content_kind 让规则知道自己在哪条路径生效）
    pub direction: Direction,                // Inbound | Outbound
    pub protocol: Protocol,                  // Anthropic | OpenAI
    pub content_kind: ContentKind,           // SseEventDelta | JsonResponseBody | ToolUseInput | RequestBody

    // 业务上下文（用于 fingerprint 计算、序列窗口、灰名单查询）
    pub tool_name: Option<&'a str>,          // "Bash" / "Read" / ... 仅 ToolUseInput 有效
    pub source_agent: Option<&'a str>,       // "claude-code" / "openclaw" / "hermes"
    pub caller_exe: Option<&'a Path>,        // 进程上下文（§5.6）
}

pub struct ScanReport {
    pub hits: Vec<MatchHit>,
    pub elapsed_us: u64,                     // 本次扫描耗时（取代 trait 上的 last_scan_us）
    pub engine_name: &'static str,
    pub rule_count: usize,
}

pub trait MatchEngine: Send + Sync {
    /// 单次扫描（v2.0 重构：带上下文）
    fn scan(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport>;

    /// 批量扫描（用于压力测试 / 数据集回归）
    fn scan_batch(&self, reqs: &[ScanRequest<'_>]) -> SieveRulesResult<Vec<ScanReport>> {
        reqs.iter().map(|r| self.scan(r.clone())).collect()
    }

    /// 引擎元信息（启动时一次性查询）
    fn engine_name(&self) -> &str;
    fn rule_count(&self) -> usize;
    fn compiled_pattern_size_bytes(&self) -> usize;
}

/// v2.0 新增：合并系统规则 + 用户规则
pub struct LayeredEngine<S, U>
where
    S: MatchEngine,  // 系统规则引擎
    U: MatchEngine,  // 用户规则引擎（None 表示用户规则未加载）
{
    system: S,
    user: Option<U>,
}

impl<S, U> MatchEngine for LayeredEngine<S, U> {
    fn scan(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport> {
        // 第一层：系统规则全量扫
        let mut report = self.system.scan(req.clone())?;

        // 第二层：如果系统命中 Critical，立即返回，不让用户规则有机会
        // suppress（PRD §9 #3 + v2.0 用户规则不能 override 系统 Critical）
        if report.hits.iter().any(|h| is_system_critical(&h.rule_id)) {
            return Ok(report);
        }

        // 否则追加用户规则命中（用户规则只能加 Ask / Warn / Mark，不能 Block）
        if let Some(user) = &self.user {
            let mut user_report = user.scan(req)?;
            // 用户规则的 hits 加 "user:" 前缀防止与系统冲突
            for h in user_report.hits.iter_mut() {
                if !h.rule_id.starts_with("user:") {
                    h.rule_id = format!("user:{}", h.rule_id);
                }
            }
            report.hits.extend(user_report.hits);
        }
        Ok(report)
    }
    // ... 其他方法委托
}
```

#### 6.3.1 LayeredEngine 合并顺序（codex review #C3 强约束）

| 优先级 | 评估顺序 | 行为 |
|--------|---------|------|
| 1 | 系统规则 Critical 命中 | **立即 Block / Ask / HookTerminal**，不评估用户规则 |
| 2 | 系统规则非 Critical 命中（High / Medium / Low）| 收集 hits，继续评估用户规则 |
| 3 | 用户规则命中 | 追加到 hits，**只能加 Ask / Warn / Mark / StatusBar** |
| 4 | 灰名单查询 | 见 §5.4.2，仅对非 Critical 系统规则 + 用户规则有效 |

**绝不允许**：
- 用户规则的 `allowlist_*` 字段豁免系统规则命中（lint 阶段强制拦截）
- 用户规则使 daemon 跳过系统规则评估
- LayeredEngine 实现把第 2/3 步顺序倒过来（系统规则永远先行）

#### 6.3.2 独立测试 + 压力测试

新增 `crates/sieve-rules/benches/`（已有 framework）下：

| Benchmark | 目标 |
|-----------|------|
| `scan_single_rule.rs` | 单条规则扫描 baseline（v1.5 已有，扩展 P50/P99/P99.9 报告） |
| `scan_70_rules.rs` | 70 条系统规则全量扫描，5KB 输入，P99 < 1ms |
| `scan_with_user_rules.rs` | 70 系统 + 30 用户规则扫描，验证 LayeredEngine overhead < 20% |

CI 加 `cargo bench --no-default-features --features ci-bench` job，P99 退化 > 10% 失败。

### 6.4 拦截引擎抽象（v2.0 不做，推 v2.1）

> **2026-05-01 review 决策**：v2.0 daemon.rs 保持 v1.5 形态。理由：
> - v2.0 第一版只做 macOS，daemon.rs 当前形态就是事实上的 MacOSInterceptor
> - 抽象成 trait 收益要 v2.1+ 多平台时才用得上，提前抽象的设计很可能不准
> - daemon.rs 140KB 大重构，Phase B 与闭测并行重构风险高（v1.5.4 刚修好的 P0 稳定性可能丢失）
> - HIPS 14 项标准里没有"拦截引擎可抽象"——是工程整洁度需求，不是产品价值
>
> v2.1 Linux / Windows 真要落地时（按 v1.5 PRD §9 第 9 条 Phase 2 触发条件），那时再设计 trait + 重构 daemon.rs。详见 §14 OQ-V20-07。

### 6.5 行为序列窗口（v2.0 Phase B MVP 新增）

详见 §5.7。实现位置 `crates/sieve-core/src/pipeline/inbound.rs::SessionState::sequence_window`。

### 6.6 进程上下文反查（v2.0 Phase A 新增）

#### 6.6.1 macOS 实现

```rust
// crates/sieve-cli/src/process_context.rs
pub struct CallerInfo {
    pub pid: i32,
    pub exe: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub ppid: Option<i32>,
}

pub fn lookup_caller(socket_addr: SocketAddr) -> Option<CallerInfo> {
    // macOS: proc_pidinfo + proc_pidpath
    // Linux: /proc/<pid>/exe + /proc/<pid>/cwd（v2.1 加）
    // Windows: GetExtendedTcpTable + QueryFullProcessImageName（v2.1 加）
}
```

#### 6.6.2 LRU Cache

```rust
type CallerCache = LruCache<i32, (CallerInfo, Instant)>;
const CACHE_TTL: Duration = Duration::from_secs(30);
```

PID → CallerInfo 映射缓存 30 秒（同一调用方多次请求复用），失效后重查。反查失败不阻塞 daemon。

### 6.7 沿用 v1.5 章节

§6.1 整体架构图（v1.5 三件套）/ §6.4 Native GUI App / §6.5 IPC 协议 schema / §6.6 sieve setup / §6.7 双层防御 ADR-014：**全部沿用**，v2.0 在其上叠加 §6.1~§6.6。

---

## 7. 商业模式与定价

不变（沿用 v1.5 §7）。

**v2.0 仅做产品能力升级，不调价**——$49/月 Pro 套餐覆盖用户规则 + 三态 ask + 行为序列。

> P2 客群的 "$99/月 Advanced" 套餐 v2.1 引入，GA 不在范围。

---

## 8. 数据飞轮与威胁情报

不变（沿用 v1.5 §8）。

**v2.0 新增本地数据维度**（不上传）：

- **用户规则使用情况**：用户加了多少条规则、命中分布、disable 频率（用于 v2.1 调整 lint 规则严格度）
- **进程上下文样本**：caller_pid/exe/cwd 在 audit.db 累积，本地分析可观测"哪个 agent 进程问题最多"
- **行为序列触发频率**：IN-SEQ-* 触发率 / FP 反馈，喂 v2.1 ML 分类器（如果触发条件满足）

**所有数据本地，不上传 doskey 服务器**（PRD §11 不变）。

---

## 9. 工程上必须做对的硬约束（v2.0 新增 #14 #15）

**沿用 v1.5 §9 第 1-13 条全部**（详见 v1.5 PRD），新增：

### #14：用户规则系统的 fail-safe（v2.0 新增）

**约束**：用户规则文件加载失败 / pattern 编译失败 / 安全 lint 拒绝 → daemon 必须**正常启动 + 系统规则全功能**，仅在错误日志 + audit.db 写 ERROR 事件 + GUI 状态栏通知用户"用户规则 X 加载失败"。

**绝不允许**：用户规则错误导致 daemon 启动失败 / 系统规则跟着失效 / 拦截能力下降。

**测试要求**：集成测试 `crates/sieve-cli/tests/user_rules_corruption.rs` 覆盖 5 类破坏（语法错误 / pattern 编译失败 / lint 违规 / 文件权限错 / 磁盘空间满）。

### #15：行为序列检测的保守起步 + beta 默认关闭（v2.0 新增）

**约束**：行为序列检测（IN-SEQ-*）：

1. **Phase B 仅触发 StatusBar 通知**，不引入新 Block 路径
2. **GA 默认关闭**（`[features] sequence_detection = false`）—— 闭测用户 / dogfood 用户主动 opt-in（codex review #A2 后调整）
3. GA 营销话术**不承诺**行为序列检测，按 dogfood 数据决定 v2.1 是否升级为默认开启

**绝不允许**：
- 序列检测 FP 导致用户合法操作被阻断
- GA 把行为序列作为必达卖点
- 序列窗口更新只覆盖 SSE 路径不覆盖 JSON 路径（v1.5.4 P0 教训，详见 §5.7.4 双路径不变量）

**升级触发条件**：v2.1 起，IN-SEQ-* 升级为 Block 类需满足：
1. 真实付费用户连续 4 周 ≥ 50 个序列样本数据收集
2. 该序列模式 FP rate < 0.5%（与 PRD §9 #7 公理 12 对齐）
3. 写新 ADR 评审

### #16：所有入站能力必须经过 content-type 路由矩阵测试（v2.0 新增，源自 v1.5.4 P0 教训）

**背景**：v1.5.4 修了两个 P0 漏洞：
- Anthropic 非流式 `application/json` 响应里的 tool_use 绕过所有入站规则
- OpenAI `stream=false` 分支跳过入站检测路径（OpenAI 协议默认 stream=false，意味着入站规则从未生效）

**约束**：v2.0 任何新增入站功能（用户规则 / 三态决策灰名单 / 行为序列窗口 / 进程上下文反查）**必须有集成测试覆盖以下 4 类组合**：

| 协议 | 响应模式 | 测试要求 |
|------|---------|---------|
| Anthropic | text/event-stream（SSE 流）| ✅ |
| Anthropic | application/json（非流式）| ✅ |
| OpenAI | text/event-stream（stream=true）| ✅ |
| OpenAI | application/json（stream=false 默认）| ✅ |

**绝不允许**：
- 新功能只挂在 SSE parser 后 → JSON 路径自动绕过（v1.5.4 漏洞模式）
- 集成测试只覆盖 Anthropic 流模式 → 其他 3 类组合静默失效
- daemon 重构（如 §6.4 v2.1 拦截引擎抽象时）破坏 content-type 路由

**实施**：
- v2.0 起所有 `crates/sieve-cli/tests/inbound_block.rs` 新增 test case 必须按 4 类矩阵交叉
- CI gate：测试覆盖率工具检测每个新规则 ID 至少在这 4 类组合里被覆盖一次
- PR 描述模板加 "content-type 路由矩阵覆盖确认" 勾选项

> v2.0 暂不引入 AI 辅助生成规则相关硬约束（功能不在 v2.0 范围）。v2.1 启动该功能时，对应客户端边界约束在新 ADR 中评审 + PRD §9 加 #17。

---

## 10. 12 周里程碑（v2.0 调整）

> 沿用 v1.5 §10 Week 1-4 全部里程碑（已完成 + v1.5.1~v1.5.4 patch）。**Week 5-12 重写按 Phase A/B**。

### Week 5 · v2.0 Phase A 启动 + Week 5 v1.4 关键路径收尾

- [ ] **sieve-policy crate 骨架**：用户规则加载（统一 user.toml）+ lint + LayeredEngine（trait 抽象）
- [ ] **sieve-rules engine trait 扩展**：MatchEngine 加 batch / metrics 接口（向后兼容）
- [ ] **进程上下文反查模块**：crates/sieve-cli/src/process_context.rs + LRU cache + macOS impl（仅 caller_pid + caller_exe 字段）
- [ ] **audit schema migration**：events 表加 caller_pid + caller_exe 字段（v2 schema 升级，cwd/ppid 推 v2.1）
- [ ] **`sieve rules` 4 个核心子命令**：edit（调用 $EDITOR + lint pipeline）/ list / disable / enable
- [ ] Week 5 v1.4 收尾：sieve-core pipeline 重构 / 出站脱敏 / IPC 完整化（沿用 v1.5）

### Week 6 · v2.0 Phase A 完成 + multi-agent setup（沿用 v1.5）

- [ ] **三态决策 ask 后端实现**：IPC 协议扩展 `allow_remember`（daemon 计算，内置 Critical 强制 false）+ `decision_response.remember`（daemon 二次校验 critical_lock）
- [ ] **灰名单存储**：`~/.sieve/decisions/` 目录读写（文件名 hex digest，0600 权限，atomic rename）+ Critical 锁验证
- [ ] **`sieve rules edit` 实现**：shell out 到 `$EDITOR` + 关闭后 lint（11 类约束）+ atomic backup/rename + IPC reload
- [ ] **content-type 路由矩阵集成测试**（PRD §9 #16）：用户规则 / 灰名单 4 类组合（Anthropic SSE + JSON / OpenAI SSE + stream=false JSON）覆盖
- [ ] **用户规则 e2e 测试**：corruption 5 类 + 系统规则不退化保证 + symlink 攻击测试
- [ ] **规则引擎压测 baseline**：scan_70_rules.rs / scan_with_user_rules.rs benchmark（不做 scan_stress）
- [ ] OpenAI 协议适配 + multi-agent setup（沿用 v1.5 Week 6）

### Week 7 · v2.0 Phase A 收尾 + OpenClaw / Hermes 集成（沿用 v1.5）

- [ ] **Phase A 完整 ship**：用户规则 / 三态 ask / 规则引擎抽象 / 进程上下文记录 全部上线
- [ ] **CHANGELOG v2.0-alpha-A**：标 v2.0 Phase A 完成
- [ ] OpenClaw / Hermes 集成测试（沿用 v1.5 Week 7）

### Week 8 · v2.0 Phase B 启动（beta 默认关闭）+ 高强度 dogfood + Stripe 接入（沿用 v1.5）

- [ ] **InboundFilter 序列窗口骨架**：`ToolUseSequence` + `ToolUseRecord` 结构化特征字段（tool_class / path_category / network_egress / persistence_mech / cleanup_mech / sensitive_file_hint）+ 滑动窗口（默认 N=10 / TTL=5min）
- [ ] **`[features] sequence_detection = false` 默认关闭**（PRD §9 #15）
- [ ] **§5.7.4 双路径不变量实施**：SSE 路径 + JSON 路径 helper 同时更新序列窗口（PRD §9 #16）
- [ ] 高强度 dogfood 三家 agent（沿用 v1.5 Week 8）+ Stripe 接入

### Week 9 · v2.0 Phase B + 闭测启动

- [ ] **3 条 IN-SEQ-* 启发式实现**：RECON-EXFIL / CLEANUP-AFTER-ATTACK / PERSISTENCE-CHAIN（基于结构化特征匹配）
- [ ] **行为序列 e2e 测试矩阵**（PRD §9 #16）：mock 攻击序列在 4 类 content-type 组合下都触发 IN-SEQ-*
- [ ] 闭测启动（沿用 v1.5 Week 9）

### Week 10 · v2.0 Phase B 灰度 + 闭测 + 内容准备

- [ ] **闭测用户主动 opt-in 行为序列**（不是灰度，是用户配置 `sequence_detection = true`）
- [ ] 内容准备（沿用 v1.5 Week 10）+ **HIPS MVP 营销弹药**（参见 v1.5.x CHANGELOG 引语，话术不对齐 CrowdStrike 级别）

### Week 11 · v2.0 闭测扩大 + KOL 接洽

- [ ] **行为序列 opt-in 用户数据收集**（按 PRD §9 #15 升级触发条件评估 v2.1 是否默认开启）
- [ ] **HIPS readiness re-assessment**：闭测后再做一次评估，目标 85%+（不强承诺 90%+）
- [ ] KOL 接洽（沿用 v1.5 Week 11）

### Week 12 · v2.0 GA 发布

- [ ] **公开 repo + 三件套 .dmg + sigstore 签名 + landing page**
- [ ] **"LLM agent HIPS MVP" 定位营销文章 ship**：3 篇文章（v1.5.x CHANGELOG 引语 + v2.0 HIPS MVP 升级故事，不对齐通用 endpoint 厂商）
- [ ] v2.0 ship features：用户规则 + `$EDITOR` 规则编辑 + 三态 ask（含 Critical 锁）+ 进程上下文记录 + **行为序列 beta 默认关闭**

### Week 13+ · v2.1 滚动迭代

- [ ] 行为序列 Phase B 完整版（更多 IN-SEQ-* 模式 + 升级 Block 类的评估）
- [ ] **拦截引擎抽象 + 多平台**（按 v1.5 PRD §9 第 9 条 Phase 2 触发条件，Linux / Windows）
- [ ] 规则编辑器 GUI 版本（sieve-gui-macos 集成）
- [ ] AI 辅助生成规则（详见 §14 OQ-V20-05）
- [ ] 进程上下文 cwd / ppid 字段补完（详见 §5.6.1 注释）
- [ ] P2 客群 Advanced 套餐 $99/月

---

## 11. 法律与合规边界

不变（沿用 v1.5 §11）。

**v2.0 补充**：用户规则文件本地存储，**绝不上传** doskey 服务器或第三方（与 PRD §11 数据不上传一致）。

---

## 12. 风险登记册（v2.0 新增 8 条，含 codex review 后补 6 条）

沿用 v1.5 §12，新增：

| 风险 | 影响 | 应对 |
|------|------|------|
| **R-V20-01：用户规则被恶意 LLM 诱导加白** | 高（用户规则把 .env 加白等场景） | §5.5.3 安全约束 11 类 + lint 双重校验 + LayeredEngine 强制不允许豁免系统规则 |
| **R-V20-02：行为序列 FP 失控** | 中（用户合法操作被通知打扰）| §9 #15 保守起步 + 仅 StatusBar 不 Block + GA 默认关闭 |
| **R-V20-03：灰名单绕过内置 Critical**（用户在 GUI 选 "永久允许" 而 daemon 没拦） | **极高**（等价于关闭 Critical，违反 ADR-007 fail-closed）| §5.4.2 Critical 锁三道防线：(1) IPC `allow_remember=false` daemon 计算（不让 GUI 决定）；(2) daemon 收到 `remember=true` 二次校验 critical_lock；(3) GUI checkbox 必须 disabled+灰显内置 Critical |
| **R-V20-04：content-type 路由回归**（v2.0 新功能只挂 SSE 不挂 JSON，重新引入 v1.5.4 P0 漏洞模式）| **极高**（70 条入站规则 + 用户规则 + 灰名单 + 序列窗口在 JSON / OpenAI 默认 stream=false 路径全瞎）| §9 #16 硬约束 + Week 6/9 集成测试矩阵 4 类组合覆盖 + PR 模板勾选项 |
| **R-V20-05：用户规则 regex / TOML 资源耗尽**（ReDoS / alternation 爆炸 / 1MB+ 大文件 / 200 条规则上限）| 高（daemon 启动慢 / 内存爆 / 单次 scan 卡死）| §5.5.3 资源上限 6 类 + lint 阶段拒绝 + daemon 加载阶段双重校验 |
| **R-V20-06：进程上下文反查错误或隐私惊吓**（PID 反查到错的 exe / 用户被 macOS 弹出权限提示）| 中（audit 数据污染 / 部署摩擦）| LRU cache 30s + 反查失败不阻塞 daemon + 仅记 caller_pid/exe（cwd/ppid 推 v2.1 避免 entitlements 摩擦）|
| **R-V20-07：Phase A 范围过载**（Week 5-7 同时做 policy + 引擎抽象 + 灰名单 + 进程反查 + 编辑器 + multi-agent）| 高（Week 12 GA 时间表风险）| codex review 后瘦身：编辑器降级 $EDITOR / 行为序列降 beta / 拦截引擎抽象推 v2.1 / OQ 立即关闭减少不确定性 |
| **R-V20-08：行为序列串会话**（不同 daemon 会话的 tool_use 错误进入同一序列窗口，导致跨用户 FP）| 中（窗口归属错误 / 隐私越界） | SessionState 按连接 ID 隔离 + 每会话独立 ToolUseSequence 实例 + 测试覆盖跨会话隔离 |

---

## 13. 与 doskey 其他业务的咬合 + 数据合作

不变（沿用 v1.5 §13）。

---

## 14. Open Questions（v2.0 新增）

沿用 v1.5 §14 全部 Open Questions，新增（codex review 后立即关闭 4 条）：

**已决策（不再 Open）**：

- ~~**OQ-V20-01**：用户规则 reload 机制~~ → **决策：IPC notify + atomic reload，失败保留旧 user engine**（与系统规则隔离，不影响系统规则）
- ~~**OQ-V20-02**：进程上下文反查 macOS API 选型~~ → **决策：macOS 走系统 API（`proc_pidinfo` + `proc_pidpath`），不 shell out `lsof -i`**（避免命令调用开销 + sandbox 权限）
- ~~**OQ-V20-06**：规则编辑器 TUI vs GUI 优先级~~ → **决策：v2.0 不做独立编辑器**，`sieve rules edit` shell out 到 `$EDITOR` + lint pipeline（codex review #B3 + Should #2）。GUI 编辑器推 v2.1
- ~~**OQ-V20-07**：拦截引擎抽象时机~~ → **决策：v2.0 完全不做**，daemon.rs 保持 v1.5 形态。v2.1 真有 Linux/Windows 需求时再做（详见 §6.4）

**保留 Open**：

- **OQ-V20-03**：行为序列窗口大小默认值 N=10 / TTL=5 分钟是否合适 —— 影响 IN-SEQ-* recall / FP 平衡，Week 9 实现后真实 dogfood 数据调
- **OQ-V20-04**：拦截引擎 trait 是否要支持"插入 hook 让用户在请求 / 响应间做自定义动作"（HIPS 经典做法但增加复杂度）—— 推 v2.1 评估
- **OQ-V20-05**：**AI 辅助生成规则功能（v2.0 已明确不做，v2.1 评估）** —— 让 LLM 帮用户从自然语言描述生成 TOML 草稿。设计要点：(a) 完全可选 fallback 离线手写；(b) 走用户自己的 API key 不经过 Sieve 服务器；(c) 生成结果仍走本地 lint；(d) 编辑器告知用户隐私边界 "prompt 将发送到你配置的 LLM 服务商，勿包含真实私钥"；(e) Sieve 出站 OUT-* 规则仍作用于该 LLM 调用（套娃保护）。v2.1 启动前需新写 ADR 评审 + PRD §9 加 #17 客户端边界约束

---

## 15. 关键参考资料

沿用 v1.5 §15，新增：

- [HIPS Readiness Assessment 2026-05-01](../review/2026-05-01-hips-readiness-assessment.md) —— v2.0 启动依据
- [CHANGELOG v1.5.4](../changelog/CHANGELOG.md#v154-non-streaming-json-inbound-fix---2026-05-01) —— v1.5 → v2.0 启动前最后一个 patch
- HIPS 经典实现参考：CrowdStrike Falcon / Microsoft Defender ATP / OSSEC
- 用户规则系统参考：Wireshark display filters / Snort 规则 / Suricata 规则

---

## 文档结束

**v2.0 决策定稿日期**：2026-05-01
**v2.0 GA 目标日期**：Week 12（与 v1.5 时间表一致）
**v2.0 状态**：锁定执行，2026-05-02 起进入 Week 5 Phase A 实施

---

## v1.5 → v2.0 changelog

| 日期 | 改动 | 章节 |
|------|------|------|
| 2026-05-01 | v2.0 初版：HIPS 改造 5 项能力 + Phase A/B 拆分 + GA Week 12 不变 | 全文 |
| 2026-05-01 | §0 修订说明：触发原因（HIPS Assessment 70%）+ 改动汇总 + 不改的部分 + Phase 拆分原则 | §0 |
| 2026-05-01 | §1.1 一句话：从"代理"升级到"HIPS" | §1.1 |
| 2026-05-01 | §3 新增 P2 客群（高级 power user / 安全工程师，v2.1 套餐）| §3 |
| 2026-05-01 | §4.7 新增场景 G（高级用户写自定义规则）| §4 |
| 2026-05-01 | §5.4 重写：三态决策模型 + 灰名单 schema + GUI 接口预留 | §5.4 |
| 2026-05-01 | §5.5 新增：用户规则系统（位置 / 优先级 / 安全约束 / 用例）| §5.5 |
| 2026-05-01 | §5.6 新增：进程上下文记录（audit schema 扩展 / 数据来源 / 隐私性能约束）| §5.6 |
| 2026-05-01 | §5.7 新增：行为序列联动（窗口模型 / 启发式 IN-SEQ-* / 与单次检测关系）| §5.7 |
| 2026-05-01 | §6.1 新增 sieve-policy + sieve-interceptor 两个 crate | §6.1 |
| 2026-05-01 | §6.2 重画整体架构图 | §6.2 |
| 2026-05-01 | §6.3 新增规则引擎抽象（MatchEngine trait 扩展 / LayeredEngine / benchmarks）| §6.3 |
| 2026-05-01 | §6.4 新增拦截引擎抽象（Interceptor trait / MacOSInterceptor MVP / 多平台预留）| §6.4 |
| 2026-05-01 | §6.6 新增进程上下文反查（macOS 实现 / LRU Cache）| §6.6 |
| 2026-05-01 | §9 新增 #14（用户规则 fail-safe）+ #15（行为序列保守起步）| §9 |
| 2026-05-01 | §10 重写 Week 5-12 按 Phase A/B 拆分 | §10 |
| 2026-05-01 | §12 新增 R-V20-01/02/03 三条 v2.0 改造风险 | §12 |
| 2026-05-01 | §14 新增 OQ-V20-01/02/03/04 四条 Open Questions | §14 |
| 2026-05-01 | §15 新增 HIPS Assessment + CHANGELOG v1.5.4 + HIPS 经典实现参考 | §15 |
| 2026-05-01 | review feedback #1：场景 G 改为规则编辑器路径（手写）；用户规则统一 user.toml（不分散文件）；新增 §5.5.5 规则编辑器；§10 Week 5/6 子任务相应更新 | §4.7 §5.5.1 §5.5.2 §5.5.4 §5.5.5 §10 |
| 2026-05-01 | review feedback #2：v2.0 暂不做 AI 辅助生成（§5.5.6 / §9 #16 / §12 R-V20-04/05 移除）；规则编辑器仅手写；功能降级到 §14 OQ-V20-05 v2.1 评估 | §4.7 §5.5.1 §5.5.4 §5.5.5 §9 §10 §12 §14 |
| 2026-05-01 | review feedback #3（自查）范围瘦身：(1) §6.4 拦截引擎抽象整章移到 v2.1（OQ-V20-07）+ §6.1 删 sieve-interceptor crate + §6.2 架构图删拦截层 + §10 Week 8/9 删任务 + §12 删 R-V20-03；(2) §5.5.2 子命令 8→4；(3) §5.5.5 编辑器能力 7→4；(4) §5.6 audit 字段 4→2；(5) §6.3 benchmark 4→3；(6) §3 P2 客群简化为一句话。Phase B 工作量减 50%，GA 风险 30-40%→10-15% | §0 §3 §5.5.2 §5.5.5 §5.6.1 §6.1 §6.2 §6.3.2 §6.4 §10 §12 §14 |
| 2026-05-01 | **review feedback #4（codex review，9 Must + 3 Should）**：(1) §5.4 灰名单加 Critical 锁三道防线；(2) §5.5 用户规则只能 High Ask/Warn/Mark，不能 override 系统 Critical；(3) §5.5.3 安全约束 5→11 类（资源上限 / 文件权限 / atomic rename / no-follow symlink / TOCTOU 防护 / keywords 必填）；(4) §6.3 `scan(&[u8])` 改 `scan(ScanRequest) -> ScanReport` + LayeredEngine 合并顺序强约束；(5) §5.7 ToolUseRecord 改结构化安全特征；(6) §9 新增 #16 content-type 路由矩阵硬约束 + §5.7.4 双路径不变量；(7) §10 Week 5/6 删除残留 8 子命令 / 匹配预览 / 正则解释器 / 示例库；(8) §12 风险 2→8 条；(9) §15 changelog 清理。**Should**：行为序列降 beta 默认关闭；规则编辑器从 ratatui TUI 改 `$EDITOR` + lint pipeline；§1.1 话术 "HIPS" → "LLM agent HIPS MVP"。OQ-V20-01/02/06/07 立即关闭。详见 [docs/review/2026-05-01-codex-review-prd-v2.0.md](../review/2026-05-01-codex-review-prd-v2.0.md) | §1.1 §5.4 §5.5 §5.5.3 §5.5.5 §5.7 §6.3 §9 §10 §12 §14 §15 |
