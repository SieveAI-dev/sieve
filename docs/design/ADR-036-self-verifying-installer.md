# ADR-036: 自校验安装器（一行安装，校验自动化，手动验签下沉为可选）

## 状态
**Accepted**
> 决策日期：2026-06-19
> 范围：安装/分发的**用户接触面**——一行安装器（curl|bash）+ Homebrew + cargo install；把"用户手动 cosign 验签"从安装主路径移除并自动化。**不触及**签名/可复现构建本身（[ADR-006](./ADR-006-sigstore-reproducible-build.md)）。
> 关联：[ADR-006](./ADR-006-sigstore-reproducible-build.md)（sigstore 签名 + reproducible build）/ [ADR-034](./ADR-034-ga-key-gate.md)（GA 密钥 gate）/ [ADR-015](./ADR-015-sieve-setup-tool.md)（`sieve setup`）
> 来源：2026-06-19 安装方式简化任务
> 关联 PRD：v2.0 §9 #2（绝不联网做 verifier）、§9 #6（供应链 sigstore + reproducible build + pinned deps）

## 背景

GA 前的安装路径（README + deployment.md）是：从 GitHub Releases 下签名 `.dmg` / 二进制，**要求用户先用 cosign 手动验签才能装**。README 还有一段明确拒绝 `curl … | sh` 的说教：

> Sieve does not provide a `curl ... | sh` one-line installer. Blindly piping a remote script into a shell is exactly the attack surface Sieve exists to oppose …

这把两件本应分开的事混为一谈了：

- **用户手动 cosign 那一步** = 纯摩擦 + 喊口号。现实里几乎没人会真去敲那条 `cosign verify-blob`，它只剩"给用户设了道门槛"这一个效果，把想用的人挡在门外。
- **给产物签名、校验产物这件事本身** = 这是 Sieve 唯一像样的差异化（可验证，而非只能信我）。这个必须保留。

现状产物（release.yml）已经具备自动校验所需的一切：裸二进制 + 每个的 `<artifact>.sigstore.json`（cosign keyless OIDC bundle，自包含签名 + 证书 + Rekor 条目）+ `SHA256SUMS`。校验所需材料都在，缺的只是"把校验从用户的家庭作业变成安装器替他做"。

错误的二选一是"硬核手动验签" vs "裸 `curl|sh` 盲信"。正确解是中间那条：**安装做到一行，但安装器在落地任何二进制前自动校验签名，校验不过就 fail-closed 拒装。**

## 决策

1. **新增自校验 `curl|bash` 安装器**（`scripts/install.sh`）：
   - `set -euo pipefail`；curl `--proto '=https' --tlsv1.2 -fsSL`；脚本托管在自己控制的 HTTPS 源（GitHub raw / GA 后 `sieveai.dev/install.sh`）。
   - 流程：检测平台 → 下载二进制 + `.sigstore.json`（+ `SHA256SUMS`）→ **自动校验**：有 `cosign` 用 sigstore bundle 验签（最强，bundle 自包含证书 + Rekor SET，不强依赖联网）；无 `cosign` 回退对照 `SHA256SUMS` 的 sha256 + **明确警告其只防传输损坏、不证明来源**并引导装 cosign → **任一校验失败立即退出、不安装（fail-closed）** → 装 `~/.local/bin` → 提示 `sieve setup`。
   - 只装 daemon/CLI 二进制；**GUI 不走 `curl|sh`**（继续签名 `.dmg` / brew cask）。
   - 脚本本身可读、注释充分——它就是"可验证而非盲信"这一价值观的现场演示。
2. **Homebrew 为 macOS 首选**：formula（CLI）+ cask（GUI），带 sha256，brew 原生自动校验。源文件置于 `packaging/homebrew/`，发布时复制到独立 tap `SieveAI-dev/homebrew-sieve`。
3. **cargo install**：补全 metadata；当前 `cargo install --git …/sieve sieve-cli` 即可（不受 `publish=false` 影响）；crates.io 的 `cargo install sieve` 标注 Phase 2（需 workspace 全部 crate 发版）。
4. **手动 cosign 验签从主路径下沉为可选**（"给偏执狂的完整验证"）：文档不再要求"装前必须验签"；改为"验证已由安装器自动完成；想手动验的看完整验证小节，或用 `sieve doctor` 查看验证状态"。

立场转变：从"**不提供** `curl|sh`"改为"提供一个**会自校验**的 `curl|sh`"。一个天天教别人"别盲目 pipe 脚本进 shell"的安全工具，自己的安装器就该是反例的反例——一行命令，校验照样发生。

## 硬约束分析（必须逐条成立才接受）

- **PRD §9 #6（供应链 sigstore + reproducible build + pinned deps）**：签名机制、可复现构建、Rekor 透明日志**一字不动**（ADR-006 原样保留）。本决策只改"谁来跑校验"——从用户改为安装器。✔ 不削弱。
- **fail-closed**：安装器校验失败 = 立即退出、不安装。安全姿态不降级，反而把 fail-closed 从"依赖用户自觉"变成"安装器强制"。✔ 收紧，非放宽。
- **PRD §9 #2（绝不联网做 verifier）**：该条针对**运行时**对 token/签名/规则做**远端**校验。安装是一次性动作，且校验在**本地**完成（cosign 本地验 sigstore bundle，bundle 自包含证书 + Rekor SET）；下载产物是安装的固有动作，非"联网 verifier"。✔ 不触碰。

## 影响

### 正面影响
- 安装从"下载 → 手动敲 cosign → 拖拽"压成一行；摩擦大幅下降，上手门槛显著降低。
- 校验从"几乎没人做的家庭作业"变成"安装器每次强制做"——实际被校验的产物比例从 ≈0 升到 100%。
- "自校验安装器"成为可对外宣传的卖点，强化"可验证而非盲信"的信任叙事，而非被 HN/竞品拿"安全工具却让人盲信安装器"来锤。
- 多渠道（brew / curl|bash / cargo）覆盖不同人群，且每条都带校验（brew sha256 / 安装器 cosign / cargo 源码构建）。

### 负面影响
- 无 cosign 时的 sha256 兜底，真实性弱于 cosign（`SHA256SUMS` 本身未签名）——已用显式警告 + 引导装 cosign 缓解，且 fail-closed 仍在。强校验路径（cosign）始终可用。
- `curl|bash` 仍有"信任脚本源"的固有面：用户得信任拉脚本的 HTTPS 源（GitHub raw / sieveai.dev）。缓解：脚本短、可读、可先 `curl` 下来审，且脚本校验的是它**下载的产物**而非要你盲信产物。
- Homebrew formula/cask 的 sha256 当前是 pre-GA 占位（全零，故意 fail-closed），首个 release 后须填真值（见 `packaging/homebrew/README.md`）。
- pre-GA 暂无 GitHub release，安装器/brew 需首个 release 才能实跑（脚本与 formula 已就绪）。

### 取代 / 修订
- **取代** README.md / README.zh-CN.md 中"Sieve does not provide a `curl … | sh` …"的说教段——立场已转为"提供自校验安装器"。
- deployment.md 的手动 cosign 验签从"required/必做"降为"optional/可选"小节（命令原文保留）。

### 需要更新的文档
- 新建本 ADR + [ADR-INDEX](./ADR-INDEX.md) 加行
- README.md / README.zh-CN.md 安装段重排（一行命令打头，手动验签下沉）
- [docs/guides/deployment.md](../guides/deployment.md) 安装/验证段重排
- [docs/changelog/CHANGELOG.md](../changelog/CHANGELOG.md) `[Unreleased]` 加 Added 条目
- tasks/PROGRESS.md（进度真实源在内部仓）勾选对应项

## 相关文档
- [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](./ADR-006-sigstore-reproducible-build.md)
- [ADR-034: GA 编译期密钥 gate](./ADR-034-ga-key-gate.md)
- [ADR-015: sieve setup 工具](./ADR-015-sieve-setup-tool.md)
- `scripts/install.sh`、`packaging/homebrew/`
