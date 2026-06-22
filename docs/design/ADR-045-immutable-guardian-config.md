# ADR-045: 不可变守护配置（自我保护）

## 状态

**Proposed**

> 决策日期：2026-06-22
> 范围：Sieve 全产品周期，所有版本；config / 系统 Critical 规则 / 用户规则 / hot-reload 全加载与求值路径
> 关联：[ADR-007](./ADR-007-fail-closed-critical-actions.md)（fail-closed 原则，本 ADR 为其姊妹篇）、[ADR-020](./ADR-020-user-rules-system.md)（用户规则系统）、[ADR-021](./ADR-021-tri-state-decision-and-graylist.md)（Critical 锁三道防线）、[ADR-024](./ADR-024-rule-engine-abstraction.md)（规则引擎抽象 / LayeredEngine 合并顺序）、[ADR-034](./ADR-034-ga-key-gate.md)（GA 密钥 gate）

---

## 背景

Sieve 的威胁模型把上游链路（模型 / 中转站 / prompt injection）整体视为不可信（[ADR-007](./ADR-007-fail-closed-critical-actions.md) §背景）。一旦上游被攻陷，agent 发出的 tool_use 就可能是攻击者控制的。**这引出一个比"拦截危险动作"更隐蔽的攻击面：与其触发拦截，不如先把守门人本身关掉。**

### 触发的真实攻击场景

被劫持的 agent 不直接发危险 tool_use，而是先尝试削弱 Sieve 自己——典型攻击路径包括：

1. **改配置文件关 Critical**：往 `config.toml` 写一个 `disable_critical = true` / `severity_overrides` 把签名规则降级，期望 daemon 静默接受；
2. **CLI flag 一键放行**：执行 `sieve start --yolo` / `--disable-critical` / `--no-prompt` 这类"全局放行"标志；
3. **用户规则提权**：写一条 `severity = "critical"` / `action = "block"` 的用户规则去影射或顶替系统规则；写一条 `id` 与系统 Critical 规则同名的用户规则，期望覆盖；
4. **allowlist 豁免系统 Critical**：在用户规则的 `allowlist_stopwords` 里填系统 Critical 规则 ID，期望把系统拦截"加白"；
5. **入站方向自动改写**：写一条 `direction = "inbound"` + `disposition = "auto_redact"` 的用户规则，妄图改写模型输出绕过入站检测（违反 PRD §9 #11）；
6. **符号链接攻击**：把 `~/.sieve/user-rules.toml` 或 decisions 目录做成指向攻击者控制文件的 symlink（`ln -s /tmp/evil.toml ...`），借加载流程读到恶意内容；
7. **未知字段夹带**：在配置里塞 Sieve 不认识的字段，赌解析器静默忽略而非报错，借此偷渡危险开关；
8. **dry-run 当后门**：把 daemon 切到 `dry_run` 模式，期望 fail-closed 拦截退化为"只记录不阻塞";
9. **hot-reload 注入**：在规则热替换窗口塞入削弱过的规则集；
10. **配置损坏降级**：故意写坏配置，赌 daemon "解析失败 → fallback 到不开 Critical"。

这十类尝试有一个共同点：**它们不是攻击被保护的资产，而是攻击 Sieve 这个守护进程本身的可信度。** 如果其中任何一条能把 Critical 拦截关掉或降级，[ADR-007](./ADR-007-fail-closed-critical-actions.md) 的 fail-closed 承诺就会在最关键的 0.001% 场景里悄悄失效，而用户毫不知情。

### 为什么需要这份 ADR

上述每一条的防御都已经分散落在多个 crate 里实现（config 层、`critical_lock`、用户规则 lint、文件加载层），且各有独立测试覆盖。但这些防御此前从未被收敛成一条**整体性的架构承诺**。本 ADR 把"守护配置对 agent 不可变"正式化为架构决策——它是 [ADR-007](./ADR-007-fail-closed-critical-actions.md)（Critical fail-closed 不可关）的姊妹篇：ADR-007 锁的是"动作不可放行"，ADR-045 锁的是"守门人不可被关停或削弱"。

本 ADR 为**追认型**：它陈述既有设计的整体不变量，并补一条端到端回归测试把这条不变量锁死，而非引入新功能。

---

## 决策

**Sieve 的安全关键配置与系统 Critical 规则对 agent 不可变——任何经由配置文件、用户规则、CLI flag、hot-reload 的削弱尝试都在加载期或求值期被结构性拒绝，而非依赖运行时检查或用户警惕。**

"结构性拒绝"的含义：削弱路径要么在解析阶段无法表达（类型系统 / `deny_unknown_fields` 直接拒绝未知字段），要么在加载阶段被显式 lint 拒绝并 fail-safe 降级（用户规则被丢弃但 daemon 正常启动），要么在求值阶段被强制覆盖（系统 Critical 的 action 一律收敛为 Block）。三层任一独立成立即可阻断攻击，不依赖单点。

---

## 硬约束逐条核对

| 约束 | 结论 | 理由 |
|------|------|------|
| **fail-closed** | ✔ | 配置解析失败时 daemon `exit(1)` 拒绝启动（`config.rs:580` `enforce_safety_invariants`），不存在"配置坏 → 退化为不拦"的路径；本 ADR 把这一行为收敛成不变量 |
| **Critical 所有版本不可关**（PRD §9 #8） | ✔ | `critical_lock::enforce_action`（`critical_lock.rs:115`）对 fail-closed 规则一律返回 `Action::Block`，与 license / 模式 / 配置正交；本 ADR 锁死该路径的回归免疫 |
| **BIP39 必须 SHA-256 checksum**（PRD §9 #4） | ✔（不削弱） | 本 ADR 不触碰检测逻辑，BIP39 checksum 验证（`sieve-rules/src/bip39.rs`）保持原状；守护配置不可变恰恰保证 agent 无法把带 checksum 的 Critical 规则降级 |
| **绝不联网做 verifier**（PRD §9 #2） | ✔ | 全部防御在本地加载/求值期完成，零网络 IO；不向任何远端校验配置或规则是否被篡改 |
| **不在 API 协议层撒谎 / 不伪造 tool_use**（PRD §9 #11） | ✔ | 本 ADR 只拒绝削弱性配置，不改写任何上下游协议字段；用户规则 lint A-5 额外禁止入站方向 `auto_redact`，防止用户规则伪造模型输出（`lint.rs` A-5） |
| **不装本地 CA 做 MITM**（PRD §9 #12） | ✔ | 不涉及任何 CA / Network Extension / 系统 proxy 行为 |
| **出站脱敏自动改写不弹窗**（PRD §9 #13） | ✔（不冲突） | 本 ADR 不改出站脱敏路径；用户规则 lint 对出站方向 `auto_redact` 合法放行（仅入站方向禁止），脱敏自动改写语义完整保留 |
| **四路由矩阵**（PRD §9 #16 / ADR-025） | ✘（不适用） | 本 ADR 是 config / 规则加载层的自我保护，不新增任何入站检测能力，不触碰 SSE/JSON 解析路径，故 content-type 四路由矩阵不适用；回归测试仍覆盖出站 + 入站两个方向的配置攻击 |

---

## 方案

防御分布在四个结构层，每层独立成立（接线点均为公开代码，引规划核实结论）：

### 1. config 层：未知字段拒绝 + 启动期不变量

- **`#[serde(deny_unknown_fields)]`**（`crates/sieve-cli/src/config.rs:87` 等多处）：任何 Sieve 不认识的字段（含偷渡的 `disable_critical` 类危险开关）触发解析错误，**不静默忽略**。对应攻击路径 #1 / #7。
- **`Config::enforce_safety_invariants`**（`config.rs:580`）：启动期调用 `check_safety_invariants`（`config.rs:520`），违规则打印 `FATAL` 并 `std::process::exit(1)`；不存在"配置坏 → 半启动 / 不开 Critical"的 fallback。对应攻击路径 #10。`dry_run` 仅打印降级 WARN，**不削弱 fail-closed 的求值**（见 §3）。对应攻击路径 #8。

### 2. 求值层：系统 Critical 强制 Block

- **`critical_lock::enforce_action`**（`crates/sieve-rules/src/critical_lock.rs:115`）：对任一命中规则，若 `is_fail_closed(rule_id)`（`critical_lock.rs:93`，查运行时 `RuleClassRegistry`）为真，**无条件返回 `Action::Block`**，忽略请求的 action。
- 注册表的 `default_on_timeout` 对 Critical 类锁定为 `Block`（`critical_lock.rs:148`），与 [ADR-007](./ADR-007-fail-closed-critical-actions.md) v1.4 补充段"`default_on_timeout` 对 Critical 规则只允许 Block"一致。
- 这一层是 CLI flag 攻击（#2）的终点：即便存在某个放行标志（实际不存在，见 [ADR-007](./ADR-007-fail-closed-critical-actions.md) §决策 2），求值期的强制覆盖也会把 Critical 收敛回 Block。

### 3. 用户规则 lint 层：fail-safe 拒绝（PRD §9 #14）

`crates/sieve-policy/src/lint.rs` 在用户规则加载期逐条 lint，命中即拒绝该条（daemon 仍正常启动 + 系统规则全功能，符合 PRD §9 #14 fail-safe）：

- **A-1**（`lint.rs:115`）：禁止用户规则声明 `severity = critical` / `action = block` / `disposition = hook_terminal`——用户规则只能 High Ask/Warn/Mark，不能 Block/HookTerminal。对应攻击路径 #3。
- **A-4**：用户规则 `id` 与系统 Critical rule_id 冲突即拒绝（`lint.rs`，比对 `critical_lock::fail_closed_snapshot()`）。对应攻击路径 #3。
- **A-5**：`direction = inbound`（或 `both`）+ `disposition = auto_redact` 拒绝——用户不能改写模型输出（PRD §9 #11）。对应攻击路径 #5。
- **A-6**：`allowlist_stopwords` 含系统 Critical rule_id 即拒绝——用户 allowlist 不能把系统拦截加白。对应攻击路径 #4。

合并顺序由 LayeredEngine 保证"系统规则先行、用户规则后行"（`crates/sieve-policy/src/engine.rs:16`），且用户规则在合并时 `fail_closed` 被强制置 `Some(false)`（`engine.rs:155`），即便 lint 漏网也无法在求值期 fail-close。这是对 lint 层的兜底。

### 4. 文件加载层：no-follow symlink 拒绝

- 用户规则文件 / 目录加载拒绝 symlink（`crates/sieve-policy/src/loader.rs:126-145`，`PolicyError::SymlinkRejected`，PRD §5.5.3-C）。
- decisions / 灰名单目录同样 no-follow（`crates/sieve-policy/src/graylist.rs:138-145,185-189` 等）。对应攻击路径 #6。

### 5. 签名规则包优先级链（hot-reload 安全）

系统 Critical 规则集经签名规则包分发并由规则引擎热替换（[ADR-024](./ADR-024-rule-engine-abstraction.md) `LayeredEngine` / `SystemEngine`，[ADR-034](./ADR-034-ga-key-gate.md) 密钥 gate）。热替换走 §2 的 `RuleClassRegistry` 求值路径，agent 无法在 hot-reload 窗口注入削弱过的系统规则集（签名校验 + 求值期强制 Block 双重兜底）。对应攻击路径 #9。具体检测规则定义由签名规则包提供，随更新通道分发，本 ADR 不涉及规则内容。

---

## 分步实施

每步可独立 ship + 独立测试。本 ADR 的核心交付是回归测试（防御本身已落地）。

### Step 1：补"agent 写恶意配置"端到端回归测试（核心）

新建一个集成测试，覆盖背景列出的全部攻击路径，每条独立断言被结构性拒绝：

- 配置层：未知危险字段 → 解析失败；非 loopback `bind_addr` → `check_safety_invariants` 返回 `Err`；坏配置 → 不存在不开 Critical 的 fallback 路径。
- 求值层：构造一个 fail-closed 规则，请求 action = Allow，断言 `enforce_action` 返回 `Block`；断言 `default_on_timeout` 为 Block。
- 用户规则层：A-1 / A-4 / A-5 / A-6 各构造一条恶意规则，断言对应 `LintKind` 违规且系统规则不受影响、daemon 不崩。
- 文件层：构造 symlink user-rules / decisions 目录，断言 `SymlinkRejected`。
- 合并层：构造一条试图 fail-close 的用户规则，断言合并后 `fail_closed = Some(false)`。

验收：上述 ~10 条攻击路径全部在测试中红→绿，作为回归免疫基线。

### Step 2：在 ADR-INDEX 登记 ADR-045，并交叉补链 ADR-007

- `ADR-INDEX.md` 新增 ADR-045 条目（已接受清单 +1）。
- 在 [ADR-007](./ADR-007-fail-closed-critical-actions.md) 末尾补一句交叉引用："守护配置不可变（ADR-045）是 fail-closed 原则在 config / 规则加载层的姊妹承诺"。

### Step 3：CI 守护（沿用 ADR-007 既有 gate）

确认 CI 既有的"任何 PR 引入 `--disable-critical` / `--yolo` / `--no-prompt` 类 flag 即 hard-fail"门禁覆盖本 ADR；若缺失则补一条静态扫描，把 Step 1 的回归测试纳入 `cargo nextest` 默认 profile。

---

## 验收标准

1. **回归测试全绿**：Step 1 的端到端测试覆盖 ≥ 10 条攻击路径，全部断言结构性拒绝；纳入 workspace 默认测试集，CI 强制通过。
2. **求值不变量**：对任意 fail-closed 规则 ID，`enforce_action(id, Allow)` / `enforce_action(id, Warn)` 均返回 `Block`（property 风格断言）。
3. **fail-safe 不变量**：注入任意一条违反 A-1/A-4/A-5/A-6 的用户规则后，daemon 正常启动且系统规则全功能（断言系统 Critical 仍命中并 Block）。
4. **四路由（不适用）**：本 ADR 不新增入站检测能力，content-type 四路由矩阵（ADR-025）不适用；回归测试以"出站方向配置攻击 + 入站方向配置攻击"两个方向替代覆盖，确保两个方向的削弱尝试都被拒。
5. **红队 bypass 用例全断**（对照规划红队清单）：
   - `disable_critical = true` 偷渡 → 解析失败（deny_unknown_fields）。
   - 用户规则 `severity = critical` / `action = block` → A-1 拒绝。
   - 用户规则 `id` 撞系统 Critical → A-4 拒绝。
   - `allowlist_stopwords` 填系统 Critical ID → A-6 拒绝。
   - 入站 `auto_redact` 用户规则 → A-5 拒绝。
   - symlink 替换 user-rules / decisions → `SymlinkRejected`。
   - `--yolo` / `--disable-critical` flag → 不存在该 flag（CI hard-fail）；即便存在，求值期强制 Block 兜底。
   - `dry_run` 当后门 → fail-closed 求值不受 `dry_run` 影响。

---

## 风险 / 已知 bypass / 误报面

### 残余攻击面（不在 Sieve 进程内可解的边界）

- **进程外攻击**：拥有对 daemon 同等权限的本地攻击者（同用户、可 `kill` daemon / 替换二进制 / 改 launchd plist）超出本 ADR 的进程内防御范围——这属于操作系统层的代码完整性与文件权限职责（二进制完整性见 ADR-006 签名 + 可复现构建）。本 ADR 防的是**agent 经由 Sieve 的合法配置/规则接口**做的削弱，不是物理替换守护进程。Sieve 在自己的层做到结构性不可变即兑现承诺。
- **守门人不在路径上**：若用户主动卸载 Sieve 或不把流量指向 daemon，Sieve 自然不生效——这是用户主动选择，不属本 ADR 防御范畴。

### 已知 bypass（设计上接受的边界）

- 用户规则层的 A-1~A-6 是"拒绝 + fail-safe 降级"语义：恶意用户规则被丢弃，但合法用户规则仍加载。攻击者无法借此削弱系统规则，但也无法用用户规则强化系统 Critical（这是 PRD §9 #14 的有意取舍：用户规则只能加检测、不能改系统 Critical 的等级与处置）。

### 误报面

- 本 ADR 是结构性拒绝逻辑，不引入新的检测匹配，**不产生检测层 FP**。唯一面向用户的体验成本是：合法用户若误写一条 `severity = critical` 的用户规则会被 lint 拒绝并打印明确 violation 消息——这是预期行为而非误报，消息已指明 PRD §5.5.3-A 依据，用户改回 High/Warn/Mark 即可。
- `deny_unknown_fields` 对配置拼写错误较严格：用户写错字段名会导致启动失败而非静默忽略。这是 fail-closed 的有意设计（宁可启动失败也不静默偷渡危险开关），错误消息已指明拒绝的字段。
