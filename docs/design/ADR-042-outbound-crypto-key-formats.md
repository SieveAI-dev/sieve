# ADR-042: 出站 crypto key 格式扩展——Bitcoin WIF + BIP-32 扩展私钥（带 Base58Check checksum）

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：`sieve-cli` 出站扫描 second-pass 接线 + 签名规则包新增 OUT-12 / OUT-13 格式族
> 关联：[ADR-016](./ADR-016-disposition-matrix-2d.md)（二维处置矩阵）、[ADR-024](./ADR-024-rules-engine-abstraction.md)（规则引擎抽象 / ScanRequest 路由上下文）、[ADR-025](./ADR-025-content-type-routing-matrix.md)（content-type 路由矩阵）、[ADR-034](./ADR-034-ga-key-gate.md)（GA 密钥 gate，签名规则包加载）、PRD v2.0 §9 #4 / #13 / #16

---

## 背景

### 现状：出站脱敏覆盖 OUT-01~11，缺两类 crypto 原生格式

Sieve 出站方向对常见密钥/凭据做脱敏（OUT-01~11），其中 BIP39 助记词（OUT-09）通过 `sieve-rules/src/bip39.rs` 的 SHA-256 checksum 验证定级，是产品的差异化点（PRD §9 #4：仅词表匹配不足以定级 Critical，必须 checksum 通过才动作）。

但出站规则集止于 OUT-11，**缺两类 crypto 钱包原生的私钥序列化格式**：

- **Bitcoin WIF（Wallet Import Format）**：单个私钥的 Base58Check 编码（主网 `5/K/L` 前缀），从任意 Bitcoin 钱包导出私钥即为此格式。
- **BIP-32 扩展私钥（xprv 家族）**：分层确定性钱包的扩展私钥序列化（`xprv`/`yprv`/`zprv` 主网、`tprv` 测试网等前缀），Base58Check 编码，泄露一个扩展私钥等价于泄露整棵派生子树的全部私钥。

### 触发的真实攻击场景

crypto 开发者在与 agent（Claude Code / OpenClaw / Hermes）协作时，常见以下出站泄露路径：

1. **粘贴钱包导出物求助**：开发者从钱包导出 WIF 私钥（"我这个地址签不动，帮我看看"），连同私钥一起贴进对话发往上游 LLM API。私钥一旦出本机进入第三方推理服务，等同失窃。
2. **调试 HD 钱包派生代码**：开发者把 `xprv...` 扩展私钥贴进对话让 agent 帮忙推导子地址。扩展私钥比单个私钥危害更大——它泄露的是整个派生分支。
3. **agent 读取 keystore 后回显**：agent 在工具调用中读取本地钱包文件，将解析出的私钥写进后续发往上游的请求体。

OUT-09（BIP39）已覆盖助记词这一形态，但**助记词与 WIF/xprv 是不同的序列化层级**——同一把私钥可能以助记词、WIF、或扩展私钥任一形式出现，三者需各自覆盖才不留缺口。

### 为什么不能只做"前缀 + 长度"正则

业界部分开源工具对 WIF/xprv 仅做"字符集 + 前缀 + 长度"的正则匹配，无校验和验证。这类做法误报面大：任何以 `xprv` 开头、长度落在区间内的 Base58 串都会命中，包括文档示例、占位符、测试向量、被截断的字符串。出站脱敏是高频自动改写路径（PRD §9 #13 不打断工作流），误报会破坏开发者粘贴的合法内容（如把一段无关的 Base58 数据改成占位符），直接侵蚀信任。

WIF 与 BIP-32 扩展私钥**都是 Base58Check 编码**——尾部 4 字节是 `SHA256(SHA256(payload))[..4]` 校验和。这给了我们与 BIP39 对齐的差异化打法：**只有校验和通过的串才动作**，把"看起来像私钥"收窄到"数学上是合法私钥序列化"，误报面大幅下降。

---

## 决策

新增 **OUT-12（Bitcoin WIF）** + **OUT-13（BIP-32 扩展私钥 xprv 家族）** 两类出站脱敏格式族，二者均为 Base58Check 编码，**必须在出站 second-pass 做 Base58Check 校验和验证**（对齐 BIP39 的 checksum 差异化打法），仅校验和通过的命中才动作；处置 `disposition=auto_redact`（自动改写 body + 状态栏 5s 通知，不弹窗）；具体格式定义（前缀集、长度边界、字符集）由签名规则包提供，随更新通道分发。

---

## 硬约束逐条核对

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed High-Risk Tool Policy Gate**（#3） | ✔ 不冲突 | OUT-12/13 是出站脱敏（自动改写），不走入站 Critical 工具门；fail-closed 语义针对入站不可逆动作，本 ADR 不触碰该路径 |
| **Critical 在所有版本不可关闭**（#8） | ✔ 不冲突 | OUT-12/13 处置为 `auto_redact`（非 Critical/Block），不进 `critical_lock.rs` 的 fail-closed 名单；不改变任何现有 Critical 项的可关闭性 |
| **BIP39 必须 SHA-256 checksum**（#4） | ✔ 对齐并延伸 | 本 ADR 把"必须 checksum"原则延伸到 WIF/xprv：二者 Base58Check 尾部校验和（双 SHA-256 前 4 字节）必须验证通过才动作，与 OUT-09 的 `verify_checksum` 同一设计哲学 |
| **绝不联网做 verifier**（#2） | ✔ 满足 | Base58Check 校验和是纯本地算术（双 SHA-256），无任何远端调用；不查链上余额、不验证私钥是否对应活跃地址 |
| **不在 API 协议层撒谎 / 不伪造 tool_use**（#11） | ✔ 满足 | 出站脱敏仅把请求体中的私钥子串替换为占位符后照常转发；不伪造 tool_use / stop_reason / id / usage，不构造 Sieve 自报事件之外的任何字段 |
| **不装本地 CA 做 MITM**（#12） | ✔ 满足 | 复用既有出站转发路径，agent 主动把 `ANTHROPIC_BASE_URL` 指向本地 daemon，无证书注入 |
| **出站脱敏自动改写不弹窗**（#13） | ✔ 满足 | `disposition=auto_redact` → `Action::Redact`，在 body 层改写后转发，不返回 426、不弹 HIPS 窗，状态栏 5s 通知；与 OUT-01~05 同路径 |
| **四路由 content-type 矩阵**（#16） | ✔ 适用 | 出站脱敏改写须对 Anthropic SSE / Anthropic JSON / OpenAI SSE / OpenAI stream=false JSON 四类请求体对等生效；详见验收标准 |

---

## 方案

### 1. 接线点：出站 second-pass 校验和验证

WIF/xprv 的格式族 pattern（前缀集 + 字符集 + 长度边界）由签名规则包提供，走 `sieve-rules` 的 vectorscan 主扫描产出候选命中。但 vectorscan 不做校验和——与 BIP39 完全同构，需在 `OutboundAdapter::scan_text`（`crates/sieve-cli/src/engine_adapter.rs:376`）内追加 **Base58Check second-pass**：

- 现有 BIP39 second-pass 在 `engine_adapter.rs:463-495`：分词 → `candidate_bip39_windows` → 对每窗口 `verify_checksum`，仅通过者产出 Detection。
- 新增对称逻辑：对 vectorscan 在 OUT-12/13 上的候选命中（或独立扫描出的候选子串）做 Base58Check 解码 + 尾 4 字节校验和比对，**仅校验和通过的命中才进 `detections`**；校验和失败的候选直接丢弃（视为误报）。
- 校验和验证函数落在 `sieve-rules`（与 `bip39.rs` 同 crate、同位置语义），保持"引擎层不联网、纯本地算术"的边界（ADR-024：`sieve-rules` 禁网络 IO）。

### 2. 处置路由：disposition=auto_redact → Action::Redact

OUT-12/13 在签名规则包中显式标 `disposition=auto_redact`。命中后经 `map_action_by_disposition`（`engine_adapter.rs:134`）映射为 `Action::Redact { placeholder }`。规则引擎抽象保证 disposition 优先于 enforce_action（`engine_adapter.rs:404-411`），因此即便未来某 ID 被误列入 fail-closed 名单，显式 `auto_redact` 仍走 Redact 而非 Block——不会把高频脱敏类升级成打断工作流的拦截。

### 3. 四路由对等改写

`Action::Redact` 的 body 改写在 daemon 出站转发路径分四类落地（`crates/sieve-cli/src/daemon.rs`）：

| 路由 | 改写接线点（现有） |
|------|------|
| M-1 Anthropic SSE | `daemon.rs:2215`（AutoRedact 脱敏 body bytes 后转发，不返回 426） |
| M-2 Anthropic JSON | `daemon.rs:2504`（文本段层脱敏，重新序列化 JSON 后转发） |
| M-3 / M-4 OpenAI SSE / stream=false JSON | `daemon.rs:3082`（命中 Redact 的 secret 转发前脱敏）+ `daemon.rs:3147,3161`（stream 与 stream=false 分支） |

OUT-12/13 复用同一 `Action::Redact` 路径，无需新增改写逻辑——但**必须有四路由测试证明候选私钥在四类请求体中都被改写**（出站方向的 #16 等价义务）。

### 4. 规则包加载与分发

OUT-12/13 的格式族定义随签名规则包分发，通过更新通道下发并经签名校验加载（ADR-034 GA 密钥 gate）。引擎侧只新增 Base58Check second-pass 接线，规则定义变更不需要改 daemon 二进制。

---

## 分步实施

每步可独立 ship + 独立测试。

**步骤 1：sieve-rules 加 Base58Check 校验和验证函数**
- 在 `sieve-rules` 新增 Base58Check 解码 + 尾 4 字节双 SHA-256 校验和比对函数，签名风格对齐 `bip39::verify_checksum`（输入候选串，返回 bool）。
- 独立测试：已知合法 WIF / xprv 测试向量返回 true；篡改任一字符、截断、错误前缀返回 false；非 Base58 字符返回 false。不依赖 daemon 启动。

**步骤 2：OutboundAdapter::scan_text 追加 second-pass**
- 在 `engine_adapter.rs:376` 的 scan_text 内，对 OUT-12/13 候选命中追加校验和 second-pass，仅通过者产出 Detection。
- 独立测试：构造含合法 WIF / xprv 的文本 → 产出 Detection；含"形似但校验和错误"的串 → 不产出。

**步骤 3：签名规则包加入 OUT-12/13 格式族**
- 规则包新增两条 `disposition=auto_redact` 的格式族定义（前缀集 / 字符集 / 长度边界）。
- 独立测试：规则包加载后 `list_rules` 含 OUT-12/13；disposition 解析为 `auto_redact`。

**步骤 4：四路由对等改写测试**
- 在 content-type 路由矩阵测试中，为 OUT-12 与 OUT-13 各加四个 test case（M-1~M-4），断言私钥子串在四类请求体中均被替换为占位符。
- 独立测试：`content_type_matrix` 通过；`check-routing-coverage.sh` CI gate 对两个新 ID 各计四路由命中。

**步骤 5：文档同步**
- user-stories + architecture（出站检测项表）+ api-reference（OUT-* 列表）+ CHANGELOG（新增检测项，P0）。

---

## 验收标准

### 功能验收

1. **校验和门**：合法 WIF（`5/K/L` 前缀）与合法 BIP-32 扩展私钥（`xprv` 家族）在出站文本中被检出并改写为占位符；校验和不通过的形似串不产出 Detection、不被改写。
2. **处置等级**：OUT-12/13 命中走 `Action::Redact`（auto_redact），不返回 426、不弹 HIPS 窗，触发状态栏 5s 通知。
3. **不联网**：检出全程无任何出站网络调用（除上游 LLM API 转发本身）。

### 四路由 content_type_matrix（PRD §9 #16 出站等价义务）

| 用例 ID | 路由 | 期望 |
|---------|------|------|
| OUT-12-M1 | Anthropic SSE 请求体含 WIF | body 改写为占位符后转发 |
| OUT-12-M2 | Anthropic JSON 请求体含 WIF | JSON 文本段脱敏后重序列化转发 |
| OUT-12-M3 | OpenAI SSE 请求体含 WIF | body 改写后转发 |
| OUT-12-M4 | OpenAI stream=false JSON 含 WIF | JSON 脱敏后转发 |
| OUT-13-M1~M4 | 同上四路由，请求体含 xprv | 同上 |

任一路由未改写 = P0 漏洞（v1.5.4 / ADR-025 教训：只挂 SSE 不挂 JSON 视为漏洞）。

### 红队 bypass 用例

| 用例 | 期望 |
|------|------|
| 合法 WIF 但前后拼接随机字符（边界探测） | 校验和验证应锚定完整候选串，拼接破坏校验和则不误改写无关字符 |
| 形似 xprv 前缀 + 随机 Base58 填充至合法长度 | 校验和不通过 → 不动作（验证降 FP 的核心用例） |
| 测试网前缀（`tprv` 等）合法扩展私钥 | 命中并改写（测试网私钥同样是敏感凭据） |
| 私钥跨多个文本段拆分 | 记录已知 bypass（见下），不在本 ADR 承诺覆盖 |
| BIP39 助记词与 WIF 同时出现 | OUT-09 与 OUT-12 各自独立命中，互不抑制 |

---

## 风险 / 已知 bypass / 误报面

### 已知 bypass

- **跨段拆分**：私钥被人为拆成多个文本段（如分两条消息、或在工具调用 input 与正文间拆开）可绕过单段扫描。出站脱敏按文本段扫描，不做跨段重组——与现有 OUT-* 同一限制，本 ADR 不承诺覆盖。
- **变体编码**：私钥经 base64 / hex 二次编码后传输不命中 Base58Check 格式族。编码即外泄前兆属另一主题，不在本 ADR 范围。
- **非标准前缀的派生格式**：某些钱包使用非标准 SLIP-0132 前缀的扩展私钥序列化，若规则包前缀集未收录则不命中；前缀集随规则包迭代扩展。

### 误报面

- Base58Check 校验和把误报压到极低——随机 Base58 串通过 4 字节校验和的概率约 1/2³²。残余误报主要来自**真实的 Base58Check 编码但非私钥**（如某些链的地址或其他 Base58Check 对象恰好落在前缀/长度区间内）。通过前缀集精确锚定（仅私钥序列化前缀）控制此面。
- 误报后果可控：auto_redact 只改写命中子串为占位符，不阻断请求；即便偶发误改写，开发者从状态栏通知可见并可在用户规则侧调整。仍须确保 OUT-12/13 的 FP 不侵蚀 PRD §9 #7 的 Critical FP 预算——但 OUT-12/13 非 Critical，独立计量。

### 风险

- **second-pass 性能**：Base58Check 解码 + 双 SHA-256 是 O(候选数) 的额外开销。候选数受 vectorscan 前置过滤约束（仅对命中格式族的子串做验证），不对全文逐字符跑校验和；需在 bench 中确认对出站 P99 的增量在预算内。
- **规则包与引擎版本错配**：若规则包下发 OUT-12/13 但引擎尚无 Base58Check second-pass，命中将退化为"无校验和的格式族匹配"（误报面回到正则级）。须保证 second-pass 接线（步骤 1-2）先于规则包格式族（步骤 3）上线，或在 second-pass 缺失时对这两个 ID 不动作。
