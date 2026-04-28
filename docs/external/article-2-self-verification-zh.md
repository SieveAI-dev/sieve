---
title: 自证清白——Sieve 自己怎么不变成新的供应链风险
author: doskey
draft: true
target_publish: 2026-W12（GA 后 1 周内发，承接文章 1）
target_channels: Twitter long-form / Mirror / doskey.dev
length: 约 3400 字
last_updated: 2026-04-29
---

# 自证清白——Sieve 自己怎么不变成新的供应链风险

你装 1Password 时凭什么相信它不卖你密码？

这个问题听起来很蠢——"它是知名公司，有口碑，有 SOC 2"。但换一个语境：你装一个**拦截你所有 LLM 流量**的本地代理，凭什么相信它不把你的 prompt、API key、助记词全都偷走？

Sieve 就是那个代理。我做了它。这篇文章想把同样的问题砸到 Sieve 自己头上。

---

## 一个 LLM 安全工具在供应链里的位置有多尴尬

Sieve 的工作原理是：夹在 Claude Code 和 Anthropic API 之间，所有出站 prompt 和入站响应都经过它。

这意味着如果 Sieve 自己被投毒，攻击者能拿到：

- 你所有的 prompt（包括粘贴进去的助记词、私钥）
- 你的 `ANTHROPIC_API_KEY`
- LLM 返回给你的所有工具调用参数

你装了一个"安全产品"，结果它成了最大的攻击面。这不是假设——这恰好是 2026 年 3 月 24 日发生在 LiteLLM 身上的事（文章 1 详述了这起投毒事件；LiteLLM 和 Sieve 在链路里的位置几乎完全相同）。

所以在聊"Sieve 能检测什么"之前，应该先聊清楚：**你有什么理由不把 Sieve 本身当成威胁？**

---

## "信任 Sieve"实际上是一个多层假设

当你说"我信任这个软件"，你其实在同时信任：

1. **代码**——源码做了它声称做的事
2. **编译链**——源码被正确编译成二进制，没有在编译期偷塞代码
3. **分发链**——你下载到的二进制就是编译出来的那个，没有在传输中被替换
4. **更新机制**——后续自动更新不会把恶意版本推进来
5. **公司治理**——维护者现在没有被收买，未来也不会

这五层里任何一层崩塌，前面四层的"信任"都白费。

历史告诉我们哪层最容易出问题：

**SolarWinds（2020）**：攻击者拿到 CI/CD 系统权限，在编译期注入恶意代码。用户拿到的是有签名的合法二进制——签名完全正确，但里面多了一个后门。这是第 2 层崩塌。

**xz-utils 后门（2024-03）**：攻击者用了近两年时间在开源社区积累声誉，成为 xz 项目的核心 maintainer，然后在一次"压缩性能优化"提交里悄悄注入后门代码。如果不是一个 PostgreSQL 工程师注意到 sshd 启动变慢并深挖，这个后门可能已经进入几乎所有 Linux 发行版。这是第 1 层崩塌，且攻击者完全绕过了"只信任知名 maintainer"的直觉。

**LiteLLM PyPI 投毒（2026-03）**：攻击者直接把恶意版本推到 PyPI 上。用户 `pip install litellm` 装到的是投毒版本。这是第 3 层崩塌。

**npm event-stream 后门（2018）**：原作者把维护权移交给一个新贡献者，后者在依赖树深处加入了针对特定比特币钱包的窃取代码。这是第 5 层崩塌：维护者换人了。

这四个案例的共同点：**传统的"开源可以审计"在长期、隐蔽的供应链攻击面前基本无效**。没有人会把每个 release 的源码和二进制全都读一遍。攻击者拼的就是人力不够。

---

## Sieve 怎么把"信任假设"砍成"可验证事实"

我没办法让你完全不用信任我。但我可以把"信任 doskey 这个人"这件事缩减到最小——剩下的部分，你自己可以验证。

具体来说，四个维度。

---

### 1. sigstore 二进制签名——验证分发链没有被篡改

每个 Sieve release 在 GitHub Actions 里自动用 [cosign](https://www.sigstore.dev/) 签名，并把签名记入 [Rekor 透明日志](https://rekor.sigstore.dev/)。

透明日志的意思是：一个公开的、append-only 的记录库，任何人都可以查。攻击者替换二进制的同时必须同时污染这个公开日志——而这个日志不由我控制。

你下载 Sieve 时，同时会拿到 `sieve.bundle`。验证方法：

```bash
# 安装 cosign（如果还没有）
brew install cosign

# 验证二进制签名
cosign verify-blob \
  --bundle sieve.bundle \
  --certificate-identity-regexp \
    "https://github.com/doskey-lee/sieve/.github/workflows/release.yml@refs/tags/v.+" \
  --certificate-oidc-issuer \
    "https://token.actions.githubusercontent.com" \
  ./sieve
```

输出 `Verified OK` 意味着：
- 这个二进制是由 GitHub Actions 在指定 workflow 里构建的
- 构建对应的 commit 在 Rekor 透明日志中有记录
- 签名所用的证书颁发给的是 GitHub 的 OIDC token，不是我手里的私钥（这一点很关键：就算 doskey 的本地开发机被黑，攻击者也没有签名 release 的能力）

这一步砍掉了**第 3 层（分发链篡改）**和部分**第 5 层（维护者账号被控）**的风险。

---

### 2. Reproducible Build——验证编译链没有偷塞代码

SolarWinds 的攻击方式是在 CI 里注入代码，让编译出来的二进制和源码不一样。对付这种攻击的方法只有一个：让任何人都能从源码自己编译，然后对比 hash。

Sieve 的 release 遵循 [Reproducible Builds](https://reproducible-builds.org/) 规范：相同 commit + 相同 toolchain → 字节完全相同的二进制。

具体措施：
- `Cargo.lock` 提交到仓库，依赖版本锁死
- `rust-toolchain.toml` 锁定 Rust 版本（当前 `1.87.0 stable`）
- `SOURCE_DATE_EPOCH` 设为 commit timestamp，消除构建时间戳的干扰
- `--remap-path-prefix` 去除开发者 home 路径污染

验证方法：

```bash
# 1. 拿到你想验证的版本的 commit hash（在 GitHub Release 页面有）
git clone https://github.com/doskey-lee/sieve.git
cd sieve
git checkout v0.x.y   # 替换成你要验证的版本

# 2. 用与 CI 相同的方式构建
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) \
  cargo build --release --locked \
  --target aarch64-apple-darwin -p sieve-cli

# 3. 对比 hash
shasum -a 256 target/aarch64-apple-darwin/release/sieve
# 把这个 hash 和 GitHub Release 里的 sieve.sha256 对比
```

如果两个 hash 相同：你手上的 Release binary 确实是从这份源码编译出来的，CI 编译期间没有偷塞代码。

这一步砍掉了**第 2 层（编译链注入）**的风险。

> 注意：macOS（aarch64 / x86_64）和 Linux（x86_64-musl）是 Tier 1，reproducible build 从 Week 1 起强制验证。Windows 是 Tier 2，Phase 1 暂不保证字节级复现（MSVC 时间戳问题，见 ADR-006）——但 sigstore 签名仍然覆盖 Windows。

---

### 3. 完全本地运行 + 不联网 verifier——验证 Sieve 不偷你的数据

Sieve 的网络 IO 只有两种：

1. **把你的 LLM 请求转发给上游**（你自己配置的 `upstream_url`，默认 `api.anthropic.com`）
2. **下载规则更新**（每周一次，见下一节）

除此之外没有任何出站连接。没有 telemetry，不收集 prompt，不上传审计日志（审计日志是本地 SQLite，在你的机器上）。

你不需要相信我说的这句话——你可以自己验：

```bash
# macOS：用 Little Snitch 或者直接 tcpdump
# 启动 Sieve，然后用 Claude Code 跑一个 prompt

# 方法 1：tcpdump（需要 root）
sudo tcpdump -i lo0 -n 'not host api.anthropic.com'
# 观察：除了 127.0.0.1（本机回环）和 api.anthropic.com，不应该有其他出站连接

# 方法 2：lsof 查看进程打开的连接
lsof -i -n -P | grep sieve
# 对比 Sieve 的 PID，只应该看到 127.0.0.1:11453（本地监听）
# 以及 api.anthropic.com 的出站连接（在你发请求时短暂出现）
```

这一步的关键不是"相信 Sieve 不联网"，而是**你可以自己抓包确认**。

这也是为什么 Sieve 必须本地运行——如果是 SaaS，你根本没有办法验证服务器端在做什么。本地运行是可验证性的前提条件。

---

### 4. 规则更新签名 + 透明日志——验证更新机制没有被劫持

规则更新是 Sieve 唯一主动发起的出站下载行为。如果攻击者控制了规则更新服务器，他们能不能通过推一个"恶意规则"来改变 Sieve 的行为？

答案是不能，原因是规则更新走 Ed25519 签名验证：

- 每次规则包发布时，用 Ed25519 私钥（doskey 持有，离线存储）签名
- Sieve 二进制里硬编码了对应的 Ed25519 公钥
- 下载后**必须验签通过**才加载，验签失败直接拒绝，**不降级执行**

同时，每次规则更新会：
- 发布 `rules-vN.changelog.md`（描述新增 / 修改 / 删除了哪些规则）
- 公示 `rules-vN.tar.zst.sha256`
- 把签名记入 Rekor 透明日志（与二进制用同一信任锚）

用户可以手动验证：

```bash
# 验证规则包签名（Sieve 自带子命令）
sieve verify-rules ./rules-v42.tar.zst ./rules-v42.sig

# 验证 hash
shasum -a 256 -c ./rules-v42.tar.zst.sha256

# 查看本次更新内容
cat ./rules-v42.changelog.md
```

规则更新日志是公开的（GitHub Release assets），你可以看到每条规则的具体变化，而不是一个"安全加固"的黑盒更新说明。

这一步砍掉了**第 4 层（更新机制被劫持）**的风险。

---

## 我们做不到的事——直说

上面四条说完，很容易给人"Sieve = 完全安全"的错觉。这不对。我来列几件 Sieve **做不到**的事：

**做不到：100% 消除对 doskey 个人的信任**

sigstore + reproducible build 把"信任 doskey"这件事缩减为"信任 doskey 没在 commit 里偷塞代码"。但最终源码还是我写的，规则还是我发布的。你仍然需要信任开源代码是你看到的那份代码。这不是 Sieve 独有的问题——所有开源软件都有这个局限。Sieve 做的是把可验证范围推到最大，但不声称消除了这个假设。

**做不到：防御 0day 和 APT**

Sieve 的检测基于规则集。针对 Sieve 专门设计的绕过、利用 LLM 解释差异的 prompt injection 变形、专门针对 Sieve 本身的 0day——这些我没有能力保证防御。Sieve 是一道闸，不是铜墙铁壁。

**做不到：阻止你主动把密钥发给 LLM**

如果你自己把助记词粘贴进 prompt，Sieve 会拦住并警告你。但如果你看了警告还是选择继续——这是你的决定，Sieve 尊重它并记录在审计日志里，但不会强制阻止。用户主权和安全护栏之间的平衡，我选的是：Critical 操作必须人工确认，但不做监狱看守。

**做不到：保证 Sieve 背后的香港法人主体永远不被收购 / 合规要求变化**

海外公司 + 加密支付双通道是当前架构，但商业实体有其生命周期。如果未来 Sieve 被收购或关闭，reproducible build 意味着你仍然可以从最后一个开源版本自己构建，sigstore 签名的历史记录仍然在 Rekor 里。这是"可验证"比"信任公司"更持久的地方。

---

## 为什么这件事在 LLM 工具链里尤其重要

Lakera Guard、LLM Guard、Prompt Security——这些产品大多是 SaaS。你的 prompt 经过他们的服务器。你没有办法验证他们的服务器在做什么。不是说他们在做坏事，但可验证性是零。

Sieve 的定位不一样：完全本地，开源核心引擎，用户持有验证能力。这是一个设计选择，不是"我们更诚实"。是因为可验证性本身就是 Sieve 的产品特性之一——对于 crypto 开发者来说，"你能自己验"比"我们承诺安全"值钱得多。

---

## 等 GA 后见

Sieve 目前仍在私有开发期（ADR-011），Week 12 GA 时一次性公开所有代码 + 文档 + 历史 sigstore 记录。

GA 后你能拿到：
- 完整源码（GitHub）
- 每个 release 的 `sieve.bundle`（cosign 验证用）
- 每个版本对应 commit 的 SHA-256 hash（reproducible build 验证用）
- 规则更新透明日志（所有历史版本）
- 一篇详细的"从零开始自己复现构建"教程

我会在这篇文章发出时同步把命令都测一遍。如果你 follow `doskey.dev`，GA 时会收到通知。

---

**本系列文章：**
- 文章 1：《你的 LLM 助手怎么成了黑客的中转站》——供应链攻击的攻击面
- 文章 2：本文——Sieve 怎么对自己动同一把刀
- 文章 3：《Drainer 复盘：一次真实的 LLM 辅助私钥窃取》

---

*doskey，2026 年 W12*

*注：本文中 GitHub 仓库 URL（`github.com/doskey-lee/sieve`）为占位符，GA 时替换为实际地址。*
