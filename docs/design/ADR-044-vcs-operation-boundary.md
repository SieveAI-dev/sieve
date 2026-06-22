# ADR-044: Sieve VCS 操作边界——不拦截 .git、不扫描 staged diff、不做文件系统级扫描

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：Sieve 全产品周期的能力边界声明（否决型）；明确 Sieve **不**承担 VCS / 文件系统级 secret 扫描职责
> 关联：[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（完全本地，能力物理边界）、[ADR-014](./ADR-014-dual-layer-defense.md)（双层防御职责划分）、[ADR-024](./ADR-024-rules-engine-abstraction.md)（规则引擎抽象）

---

## 背景

### 触发的真实场景

crypto 开发者最常见的 secret 泄露事故之一：把含私钥 / 助记词 / API key 的 `.env`、keystore JSON、配置文件**误 commit 进 git**，随后 push 到公开仓库，几分钟内被自动化扫描机器人扒走并清空钱包。这是真实、高频、后果不可逆的攻击面。

因此曾出现一个看似自然的扩展提案：**让 Sieve 拦截 `git commit`，扫描 `git diff --cached`（staged diff）里的硬编码密钥**，在提交前阻断。逻辑似乎成立——Sieve 本来就在做 secret 检测（出站脱敏 OUT-* 已有成熟的密钥模式匹配 + BIP39 SHA-256 checksum + Base58Check 校验能力），把同一套检测器接到 git staged 内容上，看起来是低成本的能力复用。

### 为什么必须停下来画边界

这个提案的诱惑在于"检测器已经有了，顺手接一下"，但它会把 Sieve 从一个**职责清晰的流量 + 工具层代理**，悄悄拖成一个**文件系统 / VCS 扫描器**。一旦开了这个口子，下一步就是"那也扫一下 working tree 吧"、"那也 watch 文件改动吧"、"那也扫 untracked 文件吧"——典型的 scope creep。本 ADR 的目的就是**在它复发之前把边界钉死**，并给出更合理的替代方案。

Sieve 的产品定位（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md) / [ADR-014](./ADR-014-dual-layer-defense.md)）有两条清晰的物理边界：

1. **流量代理层**：监控 agent 发往上游 LLM API 的内容（出站脱敏密钥/助记词），以及上游回流的内容（入站危险检测）。检测入口是字节流 / 解析后的文本与事件（见「方案」节真实接线点），**不是文件、不是仓库状态**。
2. **tool 级 PreToolUse hook 层**：在 agent 即将执行一次工具调用（shell 命令 / 文件写 / 工具参数）之前拦截。判定输入是 `{tool_name, tool_input}` 这种**单次工具调用的结构化意图**，不是 `.git` 目录状态、不是 diff、不是文件系统快照。

`git commit` 的 staged diff 既不流经 LLM API（它是本地 git 内部状态），也不是 agent 发起的「一次工具调用参数」可天然覆盖的对象——要拿到它必须主动去读 `.git/index` 与 working tree，这是一个**全新的、文件系统/VCS 级的数据源**，根本性地超出上述两条边界。

---

## 决策

**Sieve 不拦截任何 `.git` 操作、不扫描 staged diff、不做文件系统级或仓库级扫描；Sieve 的职责边界严格限定为「流量代理（监控发往 LLM API 的内容）+ tool 级 PreToolUse hook（拦截单次工具调用意图）」两层，secret 检测只发生在这两层流经的数据上。**

推荐替代（OS 原生、零 Sieve 延迟）：**git pre-commit hook + CI 阶段的专用 secret 扫描 job**。

---

## 硬约束逐条核对

逐条核对 PRD §9 / `.cursorrules §二` 中相关的不可放宽约束：

| 约束 | 判定 | 理由 |
|------|------|------|
| **fail-closed High-Risk Tool Policy Gate** | ✔ 不受影响 | 本决策是「不新增一个数据源」，不触碰 Critical 工具调用的 fail-closed 路径；现有 PreToolUse hook 的 fail-closed 语义（解析失败 / 超时 / IPC 失联均拒绝执行）完全保留，见 [ADR-014 §4](./ADR-014-dual-layer-defense.md) |
| **Critical 在所有版本（含降级模式）不可关闭** | ✔ 不受影响 | 本决策不删除、不弱化任何现有 Critical 检测项；它只是声明「不增加 VCS 这条检测路径」，与 Critical 不可关无交集 |
| **BIP39 必须做 SHA-256 checksum 验证** | ✔ 保留且不外借 | BIP39 checksum 验证（`candidate_bip39_windows` + `verify_checksum`，见「方案」）仍只服务于出站脱敏路径；本决策恰恰是**拒绝**把这套检测器接到 git staged 内容上，避免在一个被它越过的位置消耗这套差异化能力 |
| **绝不联网做 verifier** | ✔ 一致 | 推荐替代（pre-commit hook + CI 扫描）是 OS 原生 / CI 原生机制，与 Sieve 进程无关，更不引入任何 Sieve 的远端校验；不增加任何出站 host（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 的三 host 边界不变） |
| **不在 API 协议层撒谎 / 不伪造 tool_use** | ✔ 一致 | 本决策不在 SSE / JSON 协议流中注入或伪造任何字段；它不引入任何新的协议层操作 |
| **不装本地 CA 做 MITM** | ✔ 一致 | 本决策不引入 Network Extension / 本地 CA / 系统 proxy 修改；它是「不做某事」，方向与该约束完全同向 |
| **出站脱敏自动改写不弹窗** | ✔ 不受影响 | 现有出站高频脱敏（OUT-* 自动改写 + 状态栏通知）的 UX 不变；本决策不向出站路径增加任何弹窗或拦截动作 |
| **四路由矩阵（content-type 路由）是否适用** | ✘ 不适用 | 四路由矩阵（Anthropic SSE/JSON + OpenAI SSE/JSON，[ADR-025](./ADR-025-content-type-routing-matrix.md)）约束的是**入站检测**必须四类对等覆盖。本决策**不新增任何入站检测能力**，反而是声明不引入 VCS 这条**非 LLM-API 流量**的检测路径，因此 content-type 路由矩阵在此处无对象可约束，标记不适用 |

核对结论：本决策与「完全本地」「双层防御职责划分」定位**完全一致**，且**不引入文件系统 / VCS 级扫描**这一新数据源，方向与所有相关硬约束同向。

---

## 方案

本节是**否决型决策**，方案即「明确不接哪些接线点 + 现有两层边界的真实接线点」。

### Sieve 的检测数据源仅有以下两类（真实接线点）

1. **流量层文本 / 事件**（出站脱敏 + 入站检测的唯一入口）：
   - 出站扫描：`sieve-cli/src/engine_adapter.rs` 的 `OutboundAdapter::scan_text`（L376 起）——输入是**发往 LLM API 的请求体文本**。
   - 入站扫描：`InboundAdapter::scan_text`（L203 起）+ `sieve-core/src/pipeline/inbound.rs` 的 `scan_assistant_text`（L253）/ `observe_event`（L312）——输入是**上游回流的响应文本 / SSE 事件**。
   - BIP39 checksum 校验：`sieve-rules/src/bip39.rs`（`candidate_bip39_windows` + `verify_checksum`）——只在上述流量文本上运行。

2. **单次工具调用意图**（PreToolUse hook 的唯一入口）：
   - `sieve-hook/src/main.rs` 的 hook 入口读 stdin 的 `{tool_name, tool_input}`（`check` / `codex` 子命令），经 IPC 交 daemon 的工具调用判定（`inbound.rs` 的 `check_tool_use`，L36）。
   - 判定对象是**这一次工具调用的结构化参数**，hook 进程依赖 `serde_json` + `fd-lock` + `uuid` 等极简依赖（启动时延 < 50ms），**刻意不依赖任何文件系统遍历 / git 库 / 仓库状态读取能力**。

### 明确不接的接线点

- **不**新增任何读取 `.git/index` / `.git/HEAD` / working tree / `git diff --cached` 的代码路径。
- **不**在 `engine_adapter` 或任何 pipeline 节点引入「扫描本地文件 / 目录」的入口。
- **不**为 `sieve-hook` 增加 VCS 感知能力（即便某工具调用恰好是 `git commit`，hook 也只看这次调用的 `tool_input` 字符串本身，**不主动去解析 staged diff**）。

> 注意一个容易混淆的边界：若 agent 通过工具调用执行的命令**本身**把私钥明文写进了命令参数（如 `echo <privkey> >> .env`），那命中的是现有出站/工具层的密钥模式检测，与「扫描 git staged 内容」是两回事——前者是 Sieve 边界内的流量/工具数据，后者需要主动读取仓库状态，属本 ADR 否决范围。具体检测规则定义由签名规则包提供，随更新通道分发（[ADR-024](./ADR-024-rules-engine-abstraction.md) / [ADR-034](./ADR-034-ga-key-gate.md)）。

### 推荐替代方案

| 需求 | 推荐机制 | 为什么比 Sieve 拦 git 更优 |
|------|---------|--------------------------|
| 提交前拦硬编码密钥 | git **pre-commit hook**（OS / git 原生）| 运行在 git 自身的提交生命周期里，是这类问题的标准解；零 Sieve 进程延迟，离线可用 |
| 推送 / 合并前兜底扫描 | **CI 阶段的专用 secret 扫描 job** | 在不可信开发机之外的独立环境运行，覆盖整仓历史而非单次 diff；是行业常见做法 |

业界常见做法即「pre-commit 本地快速门 + CI 全量兜底」两道防线，二者均不依赖 Sieve 在请求路径上的实时性，反而避免了把多 MB diff 塞进流量代理热路径带来的延迟与 UX 问题。

---

## 分步实施

本 ADR 是边界声明，实施 = **固化边界 + 加防回归护栏**，每步可独立 ship + 独立验证：

1. **文档固化**（本步即 ship）：将本 ADR 收入 `ADR-INDEX.md`，状态 Proposed → Accepted 后，作为「Sieve 不做 VCS / 文件系统扫描」的权威引用源。验证：INDEX 链接可达，编号不跳号。
2. **代码护栏注释**：在 `sieve-hook/src/main.rs` 的 hook 入口与 `engine_adapter.rs` 的 scan 入口附近，加简短注释指明「检测数据源仅限流量文本 / 单次工具调用意图，不读取仓库或文件系统状态，理由见 ADR-044」。验证：注释存在，不引入任何新依赖（`cargo tree -p sieve-hook` 依赖集不变）。
3. **依赖面回归断言**：确认 `sieve-hook` 与 `sieve-core` 的依赖清单中**不含** git 库 / 文件遍历库（如 `git2` / `walkdir` 等）。验证：`cargo deny check` + 人工核对 `Cargo.toml` 依赖未新增此类项。
4. **替代方案文档化**：在部署 / 开发指南中加一段「Sieve 不替代 pre-commit secret 扫描；推荐配置 pre-commit hook + CI secret 扫描 job」。验证：guides 文档含该段，README 链接可达。

每步互不依赖，可分批合并。

---

## 验收标准

- **边界声明可引用**：任何后续 PR 提出「让 Sieve 扫 git diff / working tree / 文件系统」时，可直接以本 ADR 为否决依据；新增此类能力必须先写新 ADR 推翻本 ADR（状态改 Superseded），不得静默实现。
- **依赖面零增长**：`sieve-hook` / `sieve-core` 不引入任何 git / 文件系统遍历依赖；`cargo deny check` 绿。
- **现有能力零回归**：出站脱敏（OUT-*）、入站检测、BIP39 checksum、PreToolUse hook fail-closed 行为全部不变；workspace 测试套件不因本 ADR 出现新增 / 删除检测项。
- **四路由 content_type_matrix**：✘ 不适用——本决策不新增入站检测能力，无新规则 ID 需进入 `content_type_matrix.rs` 的四类 test case；CI 路由覆盖门（[ADR-025](./ADR-025-content-type-routing-matrix.md)）无新对象需校验。
- **红队 / scope 复发用例**（验证边界确实被守住）：
  1. agent 执行 `git commit` 含一个 staged 的 `.env`（内含密钥）→ **预期 Sieve 不主动拦截该 commit**（不读 staged diff），这是设计行为而非缺陷；私钥泄露的兜底由 pre-commit / CI 承担。
  2. agent 通过工具调用执行 `echo <privkey> >> config && git add config` → 命中的是命令参数中的密钥模式（现有流量/工具层检测），与「扫 git staged」无关，验证两条路径未被混淆。
  3. 任何试图在 hook / pipeline 中调用 `git`、读取 `.git/` 或遍历目录的代码改动 → 应在 review / 依赖断言阶段被本 ADR 拦下。

---

## 风险 / 已知 bypass / 误报面

### 已知未覆盖面（设计取舍，非缺陷）

- **误 commit 密钥不被 Sieve 拦**：如果开发者把私钥 commit 进 git，Sieve 不会在 git 层面阻止——这正是本决策划出的边界。**缓解**：推荐的 pre-commit hook + CI secret 扫描覆盖此场景，且这两道防线本就是该问题的行业标准解，覆盖度（整仓历史扫描）优于单次 diff 内联扫描。

### 为什么不在 Sieve 内补这个口（已知 bypass 论证）

- **进程内同源 bypass**：即便 Sieve 去扫 staged diff，扫描逻辑仍运行在 agent 可触及的本地进程环境内，理论上可被绕过；而 Sieve 在请求/工具边界上的拦截之所以有价值，正因为它卡在 agent 与上游之间的**流量必经之路**上。把检测移到 git 本地状态会丢失这个结构性优势。
- **延迟 / UX 悬崖**：commit 体量可达多 MB（含二进制 / 大文件 diff），把它塞进流量代理热路径做内联扫描会引入显著延迟，违背「出站脱敏不打断工作流」的 UX 原则方向。git 自身的 pre-commit hook 没有这个约束。

### 误报面

- 本决策**不引入任何新检测**，因此**不增加任何误报面**——这是它相对「Sieve 扫 git diff」方案的一个直接收益（后者会把仓库里的历史样本数据、测试 fixture、文档示例密钥等全部纳入扫描，误报面急剧扩大）。

### 边界复发风险（本 ADR 的核心防御对象）

- 最大风险是**未来再次出现「检测器现成、顺手接一下」的 scope creep 提案**。防御手段即本 ADR 本身 + 依赖面回归断言：任何引入 VCS / 文件系统数据源的改动都必须显式推翻本 ADR，使决策可见、可审计。
