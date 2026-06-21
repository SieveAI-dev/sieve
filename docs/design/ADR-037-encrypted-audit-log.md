# ADR-037: 加密审计日志（可选 `full` 日志档位 + write-only logging）

## 状态
**Accepted**
> 决策日期：2026-06-19
> 范围：sieve daemon 落盘审计行为，新增三档 logging level（`off` / `metadata` / `full`）；`full` 档引入 write-only logging（非对称混合加密）+ 保留期 + 防篡改哈希链
> 关联：[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（完全本地，数据层兑现）/ [ADR-006](./ADR-006-sigstore-reproducible-build.md)（签名 + 防篡改信任叙事）/ [ADR-007](./ADR-007-fail-closed-critical-actions.md)（fail-closed）/ [ADR-016](./ADR-016-disposition-matrix-2d.md)（脱敏路径）/ [ADR-021](./ADR-021-tri-state-decision-and-graylist.md)（决策 / 灰名单模型）/ [ADR-023](./ADR-023-process-context-audit.md)（CallerContext）/ [SPEC-009](../specs/SPEC-009-encrypted-audit-log.md)（工程级详细设计，Phase 2 落地）
> 关联 PRD：v2.0 §9 #2、§11.2、§11.3

## 背景

Sieve 脱敏出站、拦截入站。用户有权审计「它到底擦了什么、拦了什么」——这是信任叙事的一部分，也是 dogfood 调试 FP 的刚需。但 Sieve 的立身之本是「本地足迹最小、从不留底你的流量」（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 数据层兑现）：当前 `audit.db`（[data-model.md §6](./data-model.md)）只存 fingerprint + 最小元信息，**绝不存原始内容**，即使审计文件被读到也只能拿到「此用户在此时刻触发过 OUT-09」。

矛盾在于：部分用户（尤其闭测期深度使用者）需要看到**脱敏后的实际内容片段**来判断规则是否误伤、攻击是否真实。直接放开「存内容」会亲手建一个本地明文流量库——正是 Sieve 卖防护的那个漏洞（LiteLLM 投毒事件的同构风险）。因此任何「存内容」的能力必须按严格威胁模型设计：

**威胁模型（必须逐条防住或诚实标注残余）**：
1. 机器运行时被攻陷（live malware）
2. 硬盘被偷 / 整盘镜像被拷走
3. `~/.sieve/` 目录被误同步到云盘（iCloud / Dropbox）或被误 `git add`

现状 `audit.db` 只有元信息，对 2/3 已经安全（拿到也无内容）。但一旦引入「存脱敏后内容」的 `full` 档，落盘的内容必须加密，且加密模型要让威胁 1（运行时攻陷）也无法解开历史归档。

## 决策

### 1. 三档 logging level（扩展现有审计模型，不另起平行模型）

在 `[audit]` 配置段引入 `level`，默认值保持当前行为：

| level | 落盘内容 | 实现 |
|-------|---------|------|
| `off` | 什么都不留 | 不写 `audit.db`，不写归档 |
| `metadata`（**默认**） | 审计元数据：时间戳、方向、命中规则、类别、动作（脱敏/拦截）、用户处置（放行/拒绝）、**脱敏后**的一小段上下文片段或哈希 | **复用现有 `events` 表**（[data-model.md §6.2](./data-model.md)），即当前行为；`evidence_meta` 已是脱敏后元信息 |
| `full`（**opt-in + 显式警告**） | 全量内容归档，**只存脱敏后内容**，加密、带保留期 | 新增 write-only logging 归档（见下） |

> **命名（已裁定 2026-06-19）**：中间档定名 `metadata`，**不叫 `decisions`**——`decisions` 已是既有术语（`~/.sieve/decisions/` 灰名单目录 + `sieve decisions` headless CLI + [ADR-021](./ADR-021-tri-state-decision-and-graylist.md) 三态决策模型），复用会造冲突词，违背 glossary 维护原则。

> **配置与默认（已裁定 2026-06-19）**：本 ADR 的**新增能力**（`full` 加密归档）**默认关闭**，完全 opt-in。`level` 默认值保持 `metadata`——这是当前已发布行为（`audit.db` 现已在写），**不是新能力**；若改默认为 `off` 会静默丢失现有审计，属行为回退，故默认维持 `metadata`。`off` / `full` 均由用户显式配置切换。

`metadata` 档 = 现状，零行为变化；本 ADR 真正的新增是 `full` 档。`full` 档的元数据**仍照常写 `events` 表**，归档只是额外存一份「脱敏后完整内容」的加密副本，用 `events.id` 关联，**不分叉数据模型**。

### 2. 不可违背的红线：脱敏先于任何字节落盘

**脱敏必须在内存里、在任何字节碰硬盘之前完成。脱敏前的明文密钥（API key / 助记词 / 私钥）永远不许落盘，无论是否加密。`full` 档归档的永远是脱敏后内容。**

工程兑现（**2026-06-19 据代码勘察校正**）：出站脱敏生产路径实际调用 `redact_segments()`（`crates/sieve-core/src/pipeline/outbound_redact.rs:167`，返回 `SegmentRedactResult.texts` 脱敏后文本段；`redact_body_bytes():60` 仅 fuzz/单测保留，被 `redact_segments` 内部按段调用）。归档单元的输入**只能**是脱敏后产物——`seg_result.texts` 或经 `apply_redacted_texts_to_request()` 写回的 `new_body_bytes`（`daemon.rs:2135` Anthropic / `:2691` OpenAI，均在 `forward` 之前、无中间写盘）——**绝不是** `daemon.rs:1732` 的原始 `texts` 或入口 body。归档 hook 挂在 `new_body` 构造后、`forward` 前，复用现有 `OutboundRedacted` fire-and-forget（`:2107-2120`，当前 `raw_json=None`）同位置但消费脱敏后内容。

**入站归档范围（2026-06-19 据代码勘察修正）**：当前架构**不存在入站脱敏**——入站 `RedactAndAllow` 与 `Allow` 合并、原样放行原始内容（`daemon.rs:3060-3068`），`sieve-policy` lint `InboundAutoRedactForbidden`（`corruption.rs:319+`）明令禁止入站 auto-redact。因此 `full` 档**入站无「脱敏后内容」可存**。Phase 2 范围：**`full` 档仅归档出站脱敏后内容**（红线最相关、且脱敏确实发生的方向——用户密钥泄漏在出站）；入站 full 归档**推迟**到入站脱敏能力具备时再做（架构空白，登记备查），不在本次为它新建入站脱敏。

此红线必须有回归测试守护（Phase 2 必带）：构造含明文 `sk-ant-...` / BIP39 助记词的出站请求，开 `full` 档，断言归档密文解密后**不含任何原始密钥明文**，且 pipeline 全程无「原始 body 写盘」调用。

### 3. `full` 档加密：write-only logging（非对称混合）

**用 write-only logging 模型——这是用非对称的真正理由**：daemon **只持有公钥（recipient）**，只能加密追加，**没有解密能力**；**私钥（identity）只在审计时需要**，平时不该出现在跑 daemon 的机器上（放密码管理器 / 另一台机器 / 离线介质）。

收益：即使机器运行时被攻陷（威胁 1），攻击者也**解不开历史归档**——机器上根本没有私钥。残余暴露面只是「live malware 可截获正在流过的新明文流量」，这是任何日志设计都防不住的，本 ADR **不夸大、不假装能防**。

**加密为 hybrid（混合），不用非对称直接加密大块数据**：
- 每个归档单元随机生成对称数据密钥（data key）
- 用 AEAD（XChaCha20-Poly1305 或 AES-256-GCM）加密内容
- 用公钥包裹（wrap）数据密钥

**优先用 `age`（`age` / `rage` crate），不手搓密码学**：`age` 本质就是「加密给某个 recipient 公钥」的混合加密（X25519 + ChaCha20-Poly1305），原生支持 recipient-only 加密，正是 write-only logging 的标准答案。daemon 端只配置 recipient 公钥（`age1...`），无 identity，**结构上不具备解密能力**。

> **私钥口令保护（已裁定 2026-06-19：采纳 age 原生）**：用户输入的口令是用来**保护私钥**的，不是当加密密钥用，收益是 **daemon 正常运行不需要口令**（只用公钥），口令只在 ①初始生成密钥对 ②审计解密 两个时刻出现。`age` 原生的口令保护 identity 用 **scrypt** KDF（同为内存硬 KDF）——**直接采纳，不自研 Argon2id 包裹**。本特性重点是「加密挡住 malware 拿到出站/入站全量内容」，KDF 算法选择非重点；零自研密码学符合「不手搓」原则。

### 4. 防篡改 = 独立于加密的另一回事，审计日志两者都要（哈希链，**已裁定保留 2026-06-19**）

加密保证「读不到」，**不保证「不被改」**。注意因为 daemon 持有公钥（加密能力），被攻陷的 daemon **能往归档末尾塞伪造条目**，磁盘文件也能被截断 / 删改。

**建议加一层哈希链**：每条归档记录含 `prev_hash`（上一条记录密文的 SHA-256）+ 单调递增 `seq`。中间任何删改 / 重排会断链；截断尾部会留下 `seq` 缺口。这呼应 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 的「透明、可独立验证」信任叙事（Rekor 透明日志的本地同构）。

**残余局限（诚实写明）**：哈希链**挡不住「末尾追加伪造」**（攻陷的 daemon 持公钥可继续合法追加，并续上链）。它保证的是**历史不可悄悄改写 / 删除 / 重排**。尾部截断仅在与「外部锚点」（如另存的 head 指针 / 周期 checkpoint）比对时才可检出——Phase 1 不引入外部锚点，故尾部截断**可检出缺口但无法区分「合法新写入未完成」与「恶意截断」**。

> **已裁定保留（2026-06-19）**：哈希链虽增加实现与验证复杂度，但「历史防改写」收益值得，纳入 `full` 档必做项。

### 5. 密钥对生成 UX / 保留期 / 轮换

- **生成**：`sieve audit keygen` → 生成 X25519 密钥对；recipient 公钥写入 `config.toml [audit].recipient`；identity 私钥**以口令保护**（age scrypt 或 Argon2id 包裹，见决策 3）输出，并**强制提示用户把它移出本机**（密码管理器 / 离线介质），daemon 不留存 identity。
- **保留期**：`retention_days = N`，daemon 周期扫描归档段（archive segment），删除 mtime / 段日期超过 N 天的**整段密文文件**。删除是 `full` 档归档上**唯一允许的变更**（区别于 `events` 表 append-only），每次清理写一条 `metadata` 审计事件记录「删了哪些段」。`0` = 永久保留。
- **轮换**：`sieve audit rotate-key` → 生成新密钥对，新归档段用新 recipient；旧段保持用旧 recipient 加密（审计旧段需对应旧 identity）。`signer_pubkey_id` 式的 key id 记入段头，便于审计时定位。
- **解密（审计时，应在另一台/离线机器执行）**：`sieve audit decrypt --identity <file>`，口令解锁 identity → 解密段 → 校验哈希链 → 输出脱敏后内容。

### 6. 口令丢失 = 归档永久不可读（by design）

口令丢失则 identity 不可解锁，**归档永久不可读，这是设计使然**，不是 bug。本 ADR 写明此性质；后续 UI（`sieve audit keygen` 输出 + GUI 开启 `full` 档的确认框）**必须用最显眼方式**把这句话怼给用户，否则会收到「忘了密码求恢复」而无法恢复。

### 硬约束分析（必须逐条成立才接受）

- **PRD §9 #2（绝不联网做 verifier）**：本特性**纯本地落盘**，不新增任何出站。✔ 不违反。
- **PRD §11.2 / §11.3（数据本地化 / 绝不上传内容）**：归档严格本地，从不上传；`metadata` 默认档行为不变。✔ 强化而非削弱。
- **[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)（本地兑现）**：`full` 档是「用户对自己机器上数据的可选审计」，write-only logging + 离线私钥使其**比明文日志更安全**，强化 ADR-003 叙事。✔
- **脱敏红线（决策 2）**：归档只消费脱敏后字节，回归测试守护。✔ 这是接受本 ADR 的前提条件，不成立则整个特性不上。

## 影响

### 正面影响
- 用户可审计「擦了什么 / 拦了什么」的实际（脱敏后）内容，满足知情权 + dogfood 调试 FP 刚需。
- write-only logging 让「机器运行时被攻陷」也解不开历史归档——把审计能力做成**信任叙事的强化点**而非漏洞。
- 默认 `metadata` 档零行为变化，`off` 给极端隐私偏好用户，`full` 完全 opt-in + 显式警告，三档覆盖光谱。
- 哈希链（若保留）让历史归档「不可悄悄改写」，呼应 ADR-006 透明可验证。

### 负面影响
- `full` 档引入密钥管理 UX 负担（生成 / 离线保管 / 轮换 / 口令丢失不可恢复），需 UI 显著警示。
- 新增依赖 `age`/`rage`（+ 可能 `argon2`），过 cargo-deny；`age` 供应链需 pin + 评估（PRD §9 #6）。
- 哈希链（若保留）增加归档写入与审计验证复杂度，且对「末尾追加伪造」无效（已诚实标注）。
- 残余暴露面：live malware 可截获正在流过的新明文——任何日志设计都防不住，文档不得夸大。

### 需要更新的文档（derivation rule）
- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行（P0）
- [data-model.md](./data-model.md)：新增「§N 加密审计归档（`full` 档）」——归档单元 schema、段文件命名、哈希链记录结构、`[audit]` 配置字段表；`events` 表关联 `archive_ref`（P0）
- [api-reference.md](../api/api-reference.md) §3 config schema：新增 `[audit]` 段（`level` / `recipient` / `archive_dir` / `retention_days` / `hash_chain` / `rotation`）；§2 新增 `sieve audit keygen` / `rotate-key` / `decrypt` 子命令（P1）
- [glossary.md](./glossary.md)：新增术语 `write-only logging` / `归档单元 / archive segment` / `recipient（age 公钥）` / `identity（age 私钥）` / logging level 三档（P0）
- 新建 [SPEC-009-encrypted-audit-log.md](../specs/SPEC-009-encrypted-audit-log.md)：归档状态机、AEAD 封装格式、哈希链验证算法、密钥生命周期、脱敏红线回归测试矩阵 + [specs/INDEX.md](../specs/INDEX.md) 加行（P0，Phase 2）
- [deployment.md](../guides/deployment.md)：`full` 档启用指南 + 密钥离线保管最佳实践 + 口令丢失不可恢复警示（P1）
- [CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Added + Security 条目（P1）
- **回归测试硬约束**：「脱敏前明文绝不落盘」回归测试（Phase 2 必带，否则特性不合并）

## 相关文档
- [SPEC-009: 加密审计日志](../specs/SPEC-009-encrypted-audit-log.md)（Phase 2 落地）
- [ADR-003: 完全本地运行](./ADR-003-local-only-no-cloud-verifier.md)
- [ADR-006: Sigstore + 可复现构建 + 透明日志](./ADR-006-sigstore-reproducible-build.md)
- [ADR-021: 三态决策 + 灰名单](./ADR-021-tri-state-decision-and-graylist.md)
- [ADR-023: 进程上下文审计](./ADR-023-process-context-audit.md)
- [data-model.md §6: 本地审计日志 SQLite Schema](./data-model.md)
