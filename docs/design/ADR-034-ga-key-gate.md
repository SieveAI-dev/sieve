# ADR-034: GA 编译期密钥 gate（占位公钥 fail-closed）

## 状态
**Accepted**
> 决策日期：2026-06-11
> 范围：sieve-updater 规则签名公钥 + sieve-ipc X-Sieve-Origin 公钥的「就位」编译期强制
> 关联：[ADR-006](./ADR-006-sigstore-reproducible-build.md)（sigstore + 签名分发）/ [ADR-019](./ADR-019-x-sieve-origin-header.md)（X-Sieve-Origin header）/ updater 签名校验（详见内部记录）/ `SECURITY.md`
> 来源：2026-06-07 工程状态审查 §5 安全态势（标记「GA 硬阻塞」）
> 关联 PRD：v2.0 §9 #3（fail-closed）、§9 #6（供应链 pinned deps）

## 背景

两处 Ed25519 公钥当前是**占位**，验签 fail-OPEN：

- `crates/sieve-updater/src/signature.rs`：`TRUSTED_PUBKEY: Option<[u8; 32]> = None` → `verify_signature` skip+warn（仅靠同源 manifest 的 SHA-256 兜底）。
- `crates/sieve-ipc/src/origin_header.rs`：`SIEVE_ORIGIN_PUBLIC_KEY: &[u8; 32] = &[0u8; 32]`（全零）。

alpha 阶段可接受——规则包仍有同源 SHA-256，X-Sieve-Origin 验签 GA 前可选（ADR-019）。但 `SECURITY.md` 对外承诺「规则包 Ed25519 签名 + fail-closed 验证」。若占位公钥随 GA 二进制发布，CDN / TLS 任一沦陷即可绕过规则签名——**信任根失效**。审查（§5）将其标为「alpha 可接受，GA 硬阻塞」。

真实公钥依赖 [ADR-006](./ADR-006-sigstore-reproducible-build.md) follow-up（签名基建：GCP KMS 等，见 PROGRESS TODO-14）尚未落地，是**运维项**。问题在于：不能靠「人记得 GA 前填公钥」——这正是最容易在发布日遗漏、且后果最严重的一类 checklist 项。需要一个**机器强制**的 gate。

## 决策

引入 cargo feature **`ga_keys`** 作为 GA release build 标志。启用时，占位公钥触发**编译失败**：

- `sieve-updater`：`#[cfg(feature = "ga_keys")] const _: () = assert!(TRUSTED_PUBKEY.is_some(), …);`
- `sieve-ipc`：`#[cfg(feature = "ga_keys")] const _: () = { const fn is_all_zeros(&[u8;32])->bool {…}; assert!(!is_all_zeros(SIEVE_ORIGIN_PUBLIC_KEY), …); };`
- `sieve-cli`（顶层 binary）：`ga_keys = ["sieve-updater/ga_keys", "sieve-ipc/ga_keys"]` 传递。

行为矩阵：

| build | feature | 占位公钥 | 结果 |
|---|---|---|---|
| **alpha**（开发 / CI / dogfood） | 无 `ga_keys`（default） | 仍占位 | 编译通过；验签 skip+warn（**逐字节同现状**） |
| **GA release** | `--features ga_keys` | 仍占位 | **编译失败 E0080**，无法出包 |
| **GA release** | `--features ga_keys` | 已填真公钥 | 编译通过；验签 fail-closed 生效 |

GA release pipeline 须 `cargo build --release --features ga_keys`；占位则 E0080 失败。

### 设计要点
- **编译期 const assert**（Rust 1.57+ const panic）：零运行时开销、零新依赖（不引 `trybuild`，符合 PRD §9 #6 pinned deps）。
- gate 是**「就位」强制**而非「正确性校验」——只保证公钥非占位，不校验公钥是否对应真实签名私钥（那是发布流程 + ADR-006 KMS 的职责）。
- 数组全零判定用手写 `const fn`（`[u8; 32]` 的 `PartialEq` 非 const，不能直接 `!=` 比较）。

## 硬约束分析（必须逐条成立才接受）

- **PRD §9 #3（fail-closed Critical 不可关）**：本决策**加强** fail-closed——GA 二进制不允许 fail-open 验签出厂。✔ 收紧，非放宽。
- **PRD §9 #6（供应链 pinned deps）**：零新依赖，纯 const assert。✔
- **不改运行时行为**：alpha build 与现状逐字节一致，无任何运行时分支变化。✔

## 影响

### 正面影响
- GA 二进制**不可能**带占位公钥 fail-open 出厂，机器强制兑现 `SECURITY.md`。
- 把「GA 前填公钥」从人肉 checklist 项变成编译期硬 gate，消除发布日遗漏风险。
- alpha 开发 / CI / dogfood 零影响（默认 build 不启 feature，编译时间 + 二进制大小 0 变化）。

### 负面影响
- GA release pipeline 必须在两个公钥常量填真值后才能编译——这是**有意的强制**，非缺陷。
- gate 仅保证「非占位」，**不验证公钥正确性**（需发布流程 + ADR-006 KMS 配合；真公钥与真私钥的对应关系由签名基建保证）。

### 需要更新的文档
- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行（已完成）
- `SECURITY.md`：注明 GA build 经 `ga_keys` 编译期 gate 强制真实公钥（已完成）
- [docs/guides/deployment.md](../guides/deployment.md)：reproducible build 章节加 GA build `--features ga_keys` + 验证步骤（已完成）
- [docs/changelog/CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` Security 条目（已完成）
- tasks/PROGRESS.md：TODO-14 关联本 gate 已就位（已完成）

## 相关文档
- [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](./ADR-006-sigstore-reproducible-build.md)
- [ADR-019: X-Sieve-Origin header 协议](./ADR-019-x-sieve-origin-header.md)
- [SECURITY.md](../../SECURITY.md)
