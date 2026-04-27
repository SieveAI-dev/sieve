# ADR-006: Sigstore 签名 + Reproducible Build + 透明日志（自证清白）

## 状态

**已接受**（v1.3 锁定执行）

> 决策日期：2026-04-26
> 范围：Sieve 二进制分发、规则更新、CI/CD 整套供应链
> 关联 PRD：[v1.3 §1.2 第 4 句、§9.6、§10.1 Week 1、§11.3](../prd/sieve-prd-v1.3.md)

---

## 背景

### 触发事件：LiteLLM 1.82.7/1.82.8 投毒（2026-03）

2026 年 3 月 24 日，LiteLLM PyPI 包 1.82.7 / 1.82.8 被攻击者植入恶意代码 —— **LLM 流量代理产品自身**成为供应链攻击载体。这是 Sieve 必须正面应对的现实：

> Sieve 自己也是中间层（位置和 LiteLLM 几乎一样），如果安装 Sieve 的二进制本身被投毒，用户的所有 prompt + 凭证 + 工具调用都会泄漏 —— **Sieve 会变成新的 LiteLLM 事件**。

### 用户视角的信任问题

PRD §1.2 第 4 句（v1.3 新增）把这个问题提到产品定位层面：

> **你不只是相信我们，你能验证我们**：开源核心引擎、sigstore 签名、可复现构建、透明规则更新日志——Sieve 自己被同一套标准审视，绝不成为新的供应链风险。

这不是工程细节，是**产品营销 talking point**。Sieve 的核心定位是"在不可信链路里做最后一道闸"，如果 Sieve 自己不能被验证为可信，整个产品定位崩塌。

### 业界标准

GitHub / Sigstore / SLSA 已经形成事实标准：

- **cosign + Rekor**：sigstore 提供的密钥轻量化签名 + 透明日志（公链式 append-only）；
- **Reproducible Build**：相同源码 + 相同环境 → 字节相同的二进制；用户可独立从源码构建出与 Release 字节相同的产物；
- **GitHub Actions OIDC**：CI 过程可签名，证明二进制确实由 GitHub Actions 在指定 commit 构建；
- **SLSA（Supply-chain Levels for Software Artifacts）**：Level 3 是个人项目可达到的合理目标。

参考实现：

- StepSecurity Harden-Runner（PRD §15.3 已列入必读项目）
- Cosign 官方示例
- Reproducible Builds project（reproducible-builds.org）

## 决策

### 1. 二进制签名

**选用**：[Sigstore cosign](https://www.sigstore.dev/) `sign-blob` + Rekor 透明日志。

具体流程：

1. GitHub Actions 在 release 时调用 `cosign sign-blob --bundle sieve.bundle ./sieve-binary`；
2. 使用 GitHub OIDC token（`id-token: write` 权限）—— **无需管理私钥**；
3. 签名 + 证书 + Rekor 日志条目都打包到 `sieve.bundle`；
4. 用户下载二进制时，同时下载 `sieve.bundle`，可用 `cosign verify-blob` 验证；
5. 验证内容：
   - 签名合法
   - 证书 issuer 是 GitHub Actions OIDC
   - 证书 subject 是 `https://github.com/<owner>/sieve/.github/workflows/release.yml@refs/tags/<version>`
   - Rekor 日志中存在对应记录

**用户可独立验证**：

```bash
cosign verify-blob \
  --bundle sieve.bundle \
  --certificate-identity-regexp "https://github.com/<owner>/sieve/.github/workflows/release.yml@refs/tags/v.+" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  ./sieve-binary
```

### 2. Reproducible Build

**目标**：相同 source commit + 相同 toolchain → 任何人构建出 byte-identical 二进制。

具体实现：

| 措施 | 实现 |
|------|------|
| **依赖锁定** | `Cargo.lock` 提交到仓库；`vendor/` 目录可选（用 `cargo vendor`） |
| **Toolchain 锁定** | `rust-toolchain.toml` 锁定 Rust 版本（如 `channel = "1.82.0"`） |
| **vectorscan 编译锁定** | submodule 钉到具体 commit + 编译 flags 在 `build.rs` 显式声明 |
| **`SOURCE_DATE_EPOCH`** | 设为 commit timestamp，消除 build time 注入 |
| **去除路径污染** | `--remap-path-prefix=$HOME=/build`，避免开发者 home 路径出现在 binary |
| **Strip 符号确定性** | `strip` 在 macOS / Linux 各自走标准化流程 |
| **静态链接** | Linux 用 `x86_64-unknown-linux-musl`；macOS 用 universal binary（合并步骤可复现） |

CI 矩阵 + 平台分级：

| 平台 Tier | 平台 | sigstore 签名 | reproducible build | 上线时间 |
|----------|------|--------------|-------------------|---------|
| **Tier 1** | `macos-latest`（主战场，PRD §10.1 Week 6） | ✅ Week 1 跑通 | ✅ Week 1 双构建比对 SHA-256 | Week 1 |
| **Tier 1** | `ubuntu-latest`（musl 静态链接） | ✅ Week 1 跑通 | ✅ Week 1 双构建比对 SHA-256 | Week 1 |
| **Tier 2** | `windows-latest`（次要平台，PRD §10.1 Week 6） | ✅ Week 6 跑通 | ⚠️ Phase 2 攻坚（MSVC 时间戳问题，见下文负面影响） | Week 6 签名 / Phase 2 复现 |

- Tier 1 失败则 release 中止 —— 这是 hard gate；
- Tier 2 在 Week 6+ 与"Windows 二进制可用"承诺一起上线，签名走通即可发布，reproducible build 在 Phase 2 单独立项攻坚。

### 3. 透明规则更新日志

每次规则库发布（每周一次，PRD §8.3）：

| 内容 | 实现 |
|------|------|
| **Changelog** | `rules-vN.changelog.md` 列出新增 / 修改 / 删除的规则 ID + 简述 |
| **SHA-256 哈希** | `rules-vN.tar.zst.sha256` 公示 |
| **Ed25519 签名** | `rules-vN.sig`（详见 [data-model.md §7](./data-model.md)） |
| **GitHub Release 公开** | 所有上述文件作为 GitHub Release asset，可独立下载验证 |
| **Rekor 透明日志条目** | 用 cosign 把规则包签名同时记入 Rekor，与二进制使用同一信任锚 |

**用户可独立验证**：

```bash
# 1. 验证签名
sieve verify-rules ./rules-v42.tar.zst ./rules-v42.sig

# 2. 验证 hash
sha256sum -c ./rules-v42.tar.zst.sha256

# 3. 查看 changelog
cat ./rules-v42.changelog.md
```

### 4. Week 1 必须跑通 Pipeline

PRD §10.1 Week 1 已修订（v1.3）：**sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 必须 Week 1 跑通**——这是 PRD §1.2 第 4 句的物质基础。

**完成定义**：

- Week 1 结束前，repo 已有 release workflow，对一个 hello-world 二进制成功执行：cosign sign + rekor 上链 + reproducible build 验证（双构建 SHA-256 一致）；
- 这条不能拖到 Week 11–12 GA 前才补——**因为它是营销叙事的物质基础**，不是工程"最后一公里"。

### 5. 营销 Talking Point

这条决策不只是工程实现，更是营销弹药。具体使用方式（PRD §10.2 Week 10 文章 2）：

- **文章 2 标题**：《自证清白：Sieve 怎么证明自己不是新的 LiteLLM》
- **核心论点**：用户用 `cosign verify-blob` 自己验，用 `tcpdump` 自己抓包验"不联网 verifier"，用 reproducible build 自己从源码构出 byte-identical binary
- **战略意义**：Sieve 的差异化不只是检测能力，还有**可验证性**。这是 Lakera / LLM Guard / 其它中间层产品做不到的（他们是 SaaS / 闭源）
- 后续所有营销围绕这个 talking point 展开

---

## 影响

### 正面影响

1. **抗供应链攻击**：用户独立验证签名 + 哈希 + 透明日志，攻击者要篡改二进制必须同时控制 GitHub OIDC + Rekor 公链 + 用户本地验证流程，门槛极高；
2. **抗法律纠纷**：未来如果有用户声称"Sieve 偷我的 prompt"，doskey 可以指向开源代码 + 可复现构建 + 抓包验证，提供反证；
3. **营销 talking point**：与 PRD §1.2 第 4 句直接绑定；文章 2 战略地位与文章 1（中转站揭黑）并列；
4. **个人 IP 加分**：doskey 转型"AI × Crypto 安全研究者"过程中，"做了一个用 sigstore + reproducible build 的产品"是强信号；
5. **抗内部失误**：如果 doskey 自己不小心把 build 环境污染了，CI 双构建比对会立即报错——这是对 doskey 自己的兜底；
6. **与 [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) 形成可验证性双支柱**：ADR-003 让用户能验证"不上传"，ADR-006 让用户能验证"二进制就是源码编出来的"。

### 负面影响

1. **CI 时间翻倍**：reproducible build 双构建意味着每次 release CI 时间从 ~10 min 翻到 ~25 min；这是合理代价；
2. **vectorscan 复现复杂度**：vectorscan 是 C++ 项目，编译可复现性比 pure Rust 难度高一个量级；Week 1 必须验证 vectorscan 在 macOS / Linux / Windows 三平台都能复现，否则技术栈选型可能要回炉重来（虽然 [ADR-001](./ADR-001-rust-tech-stack.md) 已锁定，但 ADR-006 是必须解决的硬约束）；
3. **Windows 复现最难（与 PRD 理想的暂时偏离）**：MSVC 编译有时间戳注入 + 动态链接 CRT，Phase 1 Windows 仅作"二进制 + sigstore 签名可用"承诺，**reproducible build 推到 Phase 2 单独立项**。这是与 PRD §9 第 6 条全平台理想的暂时偏离，**Tier 1（macOS / Linux）的硬约束不动**——核心营销 talking point 仍成立（主战场用户可独立复现）。下次 PRD 修订时需在 §9.6 同步加 Tier 1 / Tier 2 标注；
4. **学习曲线**：cosign + Rekor + SLSA + GitHub OIDC 的串联调试 Week 1 会占用至少 2–3 天；预算内；
5. **签名密钥轮换**：sigstore 用 OIDC 短期证书，无需密钥轮换；规则签名的 Ed25519 long-lived key 需要轮换计划——Phase 1 不做，假设单 key 至少用满 Phase 1（12 周 + 6 个月维护）；密钥泄漏的回退方案：发新版本二进制硬编码新公钥 + 强制升级。

### 需要更新的文档

- [PRD-sieve v1.3 §1.2 第 4 句](../prd/sieve-prd-v1.3.md) —— 已对齐"自证清白"叙事
- [PRD-sieve v1.3 §9.6](../prd/sieve-prd-v1.3.md) —— 工程硬约束第 6 条
- [PRD-sieve v1.3 §10.1 Week 1](../prd/sieve-prd-v1.3.md) —— Week 1 sigstore + reproducible build 必须跑通
- [PRD-sieve v1.3 §11.3](../prd/sieve-prd-v1.3.md) —— 开源策略 + 透明更新日志
- [data-model.md §7](./data-model.md) —— 规则签名文件格式
- `docs/guides/development.md`（待编写）—— 写明本地复现构建步骤
- `docs/guides/deployment.md`（待编写）—— 写明用户如何独立验证 sigstore 签名
- `docs/changelog/CHANGELOG.md`（待编写）—— 每次 release 记录构建哈希 + Rekor 链接

---

## 相关文档

- [PRD-sieve v1.3 §1.2 第 4 句](../prd/sieve-prd-v1.3.md) —— 自证清白核心叙事
- [PRD-sieve v1.3 §9.6](../prd/sieve-prd-v1.3.md) —— 工程硬约束
- [PRD-sieve v1.3 §10.1 Week 1](../prd/sieve-prd-v1.3.md) —— Pipeline 必须 Week 1 跑通
- [PRD-sieve v1.3 §10.2 Week 10](../prd/sieve-prd-v1.3.md) —— 文章 2 营销战略
- [PRD-sieve v1.3 §11.3](../prd/sieve-prd-v1.3.md) —— 开源策略
- [PRD-sieve v1.3 §15.3](../prd/sieve-prd-v1.3.md) —— Sigstore + Reproducible Builds 必读项目
- [ADR-001](./ADR-001-rust-tech-stack.md) —— Rust 技术栈是 reproducible build 的基础
- [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) —— 可验证性双支柱
- [data-model.md §7](./data-model.md) —— 规则签名文件格式
