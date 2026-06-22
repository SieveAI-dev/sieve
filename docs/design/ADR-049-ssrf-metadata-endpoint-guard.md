# ADR-049: SSRF / 元数据端点 / 本地管理 socket 防护

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：PreToolUse 执行层对「访问云元数据端点 / 集群与数据库管理端口 / 本地管理 socket」的工具调用做危险判定，定位为低优先、偏企业/云场景的广度补强，非 crypto 核心楔子
> 关联：[ADR-014](./ADR-014-dual-layer-defense.md)（双层防御 / Hook 类拦截路径）、[ADR-024](./ADR-024-rules-engine-abstraction.md)（规则引擎抽象 / 私有签名规则包热替换）、[ADR-034](./ADR-034-ga-key-gate.md)（规则包密钥 gate）、[ADR-025](./ADR-025-content-type-routing-matrix.md)（四路由矩阵——本 ADR 说明为何不适用）

---

## 背景

### 触发的真实攻击场景

agent（Claude Code / OpenClaw / Hermes）在企业或云开发环境里被诱导执行 shell 工具调用时，存在一类「向内打」的越权访问，典型链路：

1. **云元数据端点提权（SSRF 风格）**：模型被注入指令或被项目里的恶意 README 误导，执行 `curl http://169.254.169.254/...`（AWS/Azure link-local 元数据）、访问 `metadata.google.internal`（GCP）或 `100.100.100.200`（阿里云）。这些端点在云上会返回临时凭据 / 实例角色 token，一旦被 agent 拉到上下文里，等于把云身份泄露给后续任意一步操作。
2. **集群管理面直连**：`kubectl` 之外，直接对 Kubernetes API server 的 admin 端口（6443 / 8443）发请求，绕过正常 RBAC 审计路径。
3. **数据库管理端口直连**：对 PostgreSQL（5432）/ MySQL（3306）/ Redis（6379）/ MongoDB（27017）/ Elasticsearch（9200）的管理端口直连，做未授权读写或数据导出。
4. **本地管理 socket 越权**：访问 Docker daemon socket（等价于宿主机 root）、DBus 系统总线、SSH-agent socket（窃取已加载的私钥代理签名能力）。

这类操作的共同特征是：**命令本身看起来是普通网络/容器工具调用，但目标是「基础设施控制面 / 凭据面」**。它们不在 Sieve 出站脱敏（`OUT-*`）或入站地址替换（`IN-CR-01`）的覆盖范围内，属于 agent 执行边界上的横向/纵向越权。

### 为什么放在 PreToolUse 执行层

这类访问的危险性完全取决于「工具调用的目标」（URL / host / port / socket 路径），而不是 LLM 响应文本的内容。因此最合适的拦截点是 **agent 真正要执行命令之前**的 PreToolUse hook 判定层（与 ADR-014 的 Hook 类规则同一路径），而非 SSE/响应流检测。

### 范围声明（避免误读优先级）

本项是**广度补强**，主要服务企业/云开发场景，**不是 Sieve 的 crypto 核心差异化**。在路线优先级上低于出站密钥脱敏与入站地址替换。引入它的理由是：执行层已经有现成判定通道（ADR-014），加挂一组「危险网络目标 / socket 目标」检测的边际成本低，且能堵住一类清晰的越权面。具体目标清单与匹配方式由签名规则包提供（见「方案」）。

---

## 决策

> **在 PreToolUse 执行层（sieve-hook → daemon 判危）解析工具调用的网络/socket 目标，对云元数据端点、集群/数据库管理端口、本地管理 socket 的访问做危险判定；具体目标清单与处置等级由签名规则包定义，随更新通道分发。**

判定结果沿用现有处置矩阵（Block / Ask 等），命中即走 ADR-014 的 Hook 类拦截路径或 Critical 人工确认路径，不新增独立拦截机制。

---

## 硬约束逐条核对

| 约束 | 核对 | 理由 |
|---|---|---|
| **fail-closed** | ✔ | 完全复用既有判危链路：`sieve-hook` 任何错误路径（解析失败 / daemon 不可达 / 超时 / 判危失败）均 fail-closed（`main.rs:147-148` codex 路径 exit 2；`daemon_control_plane.rs:1122` judge 失败 → deny）。本 ADR 只新增规则，不改 fail-closed 语义 |
| **Critical 在所有版本（含降级模式）不可关闭** | ✔ | 若签名规则包把某目标（如云元数据端点 / Docker socket）定为 Critical，则继承系统 Critical 不可关属性（`critical_lock.rs` 强制 Block）。用户规则不能 suppress（`sieve-policy` lint 禁 allowlist 命中系统 Critical）。本 ADR 不引入任何「可关掉 Critical」的开关 |
| **BIP39 必须做 SHA-256 checksum 验证** | ✘（不适用） | 本 ADR 不触碰 BIP39 / 助记词检测，出站 checksum 验证逻辑（`sieve-rules/src/bip39.rs`）原样不动 |
| **绝不联网做 verifier** | ✔ | 目标判定为纯本地匹配（命令字符串 → 目标 host/port/socket），不对任何外部端点发请求做「确认」。规则正文经既有更新通道分发（ADR-024 / ADR-034），不构成运行时联网校验 |
| **不在 API 协议层撒谎 / 不伪造 tool_use** | ✔ | 走 PreToolUse hook（execution 边界），完全不改 SSE/JSON 响应流，不注入/伪造 `tool_use` / `stop_reason` / `id` / `usage`。拦截表现为 agent 拒绝执行该工具调用，符合 ADR-014 §1 协议层硬约束 |
| **不装本地 CA 做 MITM** | ✔ | 检测对象是 agent 自己要发出的命令的目标，靠 PreToolUse hook 在执行前判定，**不需要解密任何流量**，不安装 CA、不做 Network Extension、不改系统 proxy |
| **出站脱敏自动改写不弹窗** | ✘（不适用） | 本 ADR 属入站/执行层危险拦截，不是出站脱敏类（`OUT-*`）；不涉及「自动改写 body + 状态栏通知」路径 |
| **四路由 content-type 矩阵** | ✘（不适用，见下） | 见「验收标准 · 四路由说明」 |

---

## 方案

接线点全部为既有通道，本 ADR 不新增 crate、不新增拦截路径，仅扩展规则集与判危输入解析：

1. **判危入口（hook → daemon）**：agent 触发 PreToolUse hook → `sieve-hook` 读工具调用 JSON → `codex_ipc::judge_tool_call`（`sieve-hook/src/codex_ipc.rs:32`，方法名 `sieve.judge_tool_call`，`main.rs:181`）→ daemon `handle_judge_tool_call`（`sieve-cli/src/daemon_control_plane.rs:1018`，与真实入站路径同款检测引擎，见 `:165`）。

2. **工具调用检测 trait**：daemon 侧判危复用入站引擎的 `InboundEngine::check_tool_use`（`sieve-core/src/pipeline/inbound.rs:36-40`），以 `ContentSource::InboundToolUseInput`（`inbound.rs:332`）为来源。本 ADR 的新规则在此 trait 实现内被求值——即对 `CompletedToolCall` 的命令文本解析出目标 host/port/socket 并比对规则集。

3. **规则集来源**：目标清单（元数据端点 IP/域名、管理端口号、socket 路径）与各目标的处置等级（Block / Ask）作为签名规则包条目，由 `sieve-rules` 的引擎加载（`SystemEngine` / `LayeredEngine`，`sieve-rules/src/engine/mod.rs:22,165`，支持热替换）。**具体检测规则定义由签名规则包提供，随更新通道分发**（ADR-024 引擎抽象 + ADR-034 密钥 gate）；公开文档不内联精确清单与端口数值。

4. **处置分流**：命中后按规则 disposition 走既有矩阵——Block/Critical 类经 ADR-014 Hook 类路径让 agent 拒绝执行；如定为需人工确认则走 Critical 人工确认。本 ADR 不引入新 disposition 语义。

> 命令目标解析需要从工具调用参数（shell 命令字符串 / 工具 input）中抽出网络目标与 socket 路径。该解析逻辑落在 `check_tool_use` 的实现侧，作为后续 SPEC 的工程规格细化（解析规则、归一化、绕过收敛）；本 ADR 只锁定「在哪判、判什么类」。

---

## 分步实施

每步可独立 ship、独立测试，互不阻塞：

1. **目标解析 + 规则比对最小集（元数据端点）**：先只覆盖云元数据端点一类（最高危、特征最清晰）。在 `check_tool_use` 实现里加目标解析 + 规则比对；规则条目入签名规则包。验证：命中元数据端点的工具调用走 Block 路径，agent 拒绝执行。
2. **扩管理端口（集群 + 数据库）**：在已建好的解析框架上追加 K8s admin / DB admin 端口规则条目。验证：命中端口走对应处置等级。
3. **扩本地管理 socket（Docker / DBus / SSH-agent）**：追加 socket 路径目标解析与规则条目。验证：访问对应 socket 走 Block 路径。
4. **绕过收敛 + 红队回归**：把「已知 bypass」用例（见风险节）纳入红队测试集与回归套件，逐项收敛解析归一化。

每步独立可发布：规则包条目热替换即可上线（ADR-024），解析框架向后兼容空规则集（无规则时不命中、不报错）。

---

## 验收标准

### 功能验收

- 第 1 步后：构造命中云元数据端点的工具调用，daemon 判危返回 Block 类处置，agent 侧拒绝执行；构造无害网络访问不误命中。
- 第 2/3 步后：管理端口、本地管理 socket 各类目标均能命中并走规则定义的处置等级。
- fail-closed 回归：daemon 不可达 / 判危超时时，`sieve-hook` 走 fail-closed（codex 路径 exit 2 / judge 失败 deny），不放行。

### 四路由 content-type 矩阵说明（不适用）

本 ADR 检测的是 **agent 工具调用的执行目标**，拦截点是 PreToolUse hook 的 `judge_tool_call` 通道（execution 边界），**不解析 LLM 响应流**。ADR-025 的四路由矩阵（M-1 Anthropic SSE / M-2 Anthropic JSON / M-3 OpenAI SSE / M-4 OpenAI JSON）约束的是**响应侧（入站）文本/事件检测必须在四种 content-type 路径上对等覆盖**；本 ADR 不在任何响应路径上新增检测，因此 `content_type_matrix` **不适用**。

> 对等的覆盖维度在本 ADR 是「支持的 agent hook 通道」而非「响应 content-type」——即同一组目标规则在所有走 `judge_tool_call` 的 client（Claude Code / 经对应 hook 的 OpenClaw / Hermes）上一致生效；验收以判危链路的单测/集成测试覆盖，而非四路由矩阵。

### 红队 bypass 用例（纳入回归）

构造下列变形，验证检测不被绕过（具体期望命中由规则包定义）：

- 元数据端点的等价写法变形（IP 的不同进制/补零形式、十进制整数形式、域名大小写、尾随路径/查询）。
- 管理端口经不同工具触达（直接 socket 工具、不同 client 字节流封装）。
- socket 路径的相对/符号链接/绝对形式差异。
- 多目标拼接在一条复合命令里（管道/分号/逻辑连接）。

---

## 风险 / 已知 bypass / 误报面

### 已知 bypass（需后续解析归一化收敛）

1. **目标编码绕过**：元数据端点 IP 可用十进制整数、八进制、补零、混合进制等价写法表达；域名可加尾点或大小写变形。解析层需做地址归一化，否则纯字面匹配会被绕过。第 4 步专门收敛。
2. **间接触达**：通过本地端口转发、代理跳板、先写脚本再执行（dropper 模式）等方式间接命中目标，单条命令字面里看不到目标。本 ADR 只覆盖「命令里能解析出目标」的直接情形；间接链路属其他执行层检测范畴，不在本 ADR 承诺内。
3. **非 hook client**：若 agent 不经任何 PreToolUse hook 执行命令，则本检测不生效——这与 ADR-014 的整体前提一致（执行层依赖 hook 注册），不是本 ADR 独有缺口。

### 误报面

- 合法运维场景确实会访问数据库/集群管理端口（正常 `kubectl`、本地开发连数据库）。为此 DB/集群类目标的默认处置宜偏向「需确认」而非一律硬 Block，由规则包按目标分级设置；元数据端点与高危 socket（Docker/SSH-agent）特征更尖锐，误报面更小，可定更严等级。
- 误报具体阈值与分级随规则包迭代调整，公开文档不固化数值。

### 范围风险

本项偏企业/云场景，与 crypto 核心楔子正交。实施时不得挤占核心检测的性能预算与维护注意力；解析逻辑必须收敛在 `check_tool_use` 实现内，不污染出站脱敏与入站地址替换路径。
