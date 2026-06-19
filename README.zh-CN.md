# Sieve

[English](./README.md) | 中文

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](#安装phase-1-仅-macos)
[![Status: pre-GA beta](https://img.shields.io/badge/status-pre--GA%20beta-orange.svg)](#项目状态)

> **完全本地运行的 LLM 流量安全代理——不可逆动作前的最后一道闸。**

Sieve 是一个完全本地运行的 LLM 流量安全代理（Rust 单二进制），夹在你的 AI 编码 agent（Claude Code / Codex CLI / Cursor）和上游模型 API（Anthropic / OpenAI / 中转站）之间。它对双向流量做检测——出站脱敏（redacting secrets），入站拦截危险工具调用（fail-closed）——在不可逆动作（签名、转账、部署）之前强制插入一个认知摩擦的瞬间。专为 crypto-native 开发者打造。

```mermaid
flowchart LR
    A["AI agent<br/>(Claude Code / Codex / Cursor)"] <--> S["Sieve<br/>(本地 · 127.0.0.1)"]
    S <--> U["Anthropic / OpenAI / 中转站"]
    S -.-> D["出站：规则匹配 + 敏感信息脱敏<br/>入站：Critical 工具调用拦截 + 人工确认（HIPS）<br/>检测推理 100% 本地"]
```

---

## 为什么需要 Sieve

1. **上游不可信**——你路由经过的中转站可能改写你的 `tool_call`；官方 API 不会在私钥泄漏导致钱包被掏空时赔你钱。
2. **没人能替你兜底**——钱包安全产品永远看不到你的 prompt，LLM 安全产品不懂 crypto，DLP 不在你的工作流里。
3. **Sieve 是客户端的最后一道闸**——检测推理完全本地，字节流双向扫描，**永不上传你的 prompt、response、API key 或任何使用记录**。
4. **你不只是相信我们，你能验证我们**——源码公开、release 签名、可复现构建、透明的规则更新日志。

---

## 隐私

Sieve 每天 **4 次**连接更新服务器获取最新规则。每次请求仅附带 **5 个字段**：版本 / OS / CPU 架构 / 本地随机生成的安装 ID（install-id，不绑定任何账号或设备）/ 通道（channel）。它**永不上传 prompt、response、API key 或任何使用记录**。

- `SIEVE_NO_TELEMETRY=1` —— 关闭匿名安装统计（规则更新不受影响）。
- `SIEVE_NO_UPDATE=1` —— 完全禁用更新检查。

---

## 关键差异化（护城河）

1. **LLM 流量层的独占站位**——钱包安全产品看不到 prompt，DLP 不在工作流里。
2. **本地推理 + 边界明确的更新通道**——检测 100% 本地，零云依赖。
3. **Crypto 专项检测**——对照调研的 19 家 LLM/DLP 产品全无、9 家 AI Agent 安全工具全无此能力。
4. **双向检测 + fail-closed**——Critical 在任何模式下都不可关闭。

---

## Quick Start

> ⚠️ Sieve 当前处于 **GA 前闭测（pre-GA closed beta）**（见[项目状态](#项目状态)）。下方命令描述的是 GA 后的正式发布形态。

### 安装（Phase 1 仅 macOS）

> **Sieve 不提供 `curl ... | sh` 一键安装脚本。** 把远程脚本盲管进 shell 正是 Sieve 反对的攻击面——安全产品不能反着做自己所反对的事。

Sieve 通过**签名 `.dmg`** 经 [GitHub Releases](https://github.com/SieveAI-dev/sieve/releases) 分发：

1. 从 GitHub Releases 下载 `Sieve-<version>.dmg`。
2. 安装前**用 `cosign` 验证 `.dmg` 签名（必做）**。
3. 挂载，将 `Sieve.app` 拖入 `/Applications`，首次启动按引导运行 `sieve setup`。

Homebrew tap（`brew install sieve`）、Linux、Windows 均推后到 Phase 2。

### 接入你的 agent

```bash
# 一键配置 Claude Code
# （写 ANTHROPIC_BASE_URL + 注册 PreToolUse hook + 装 launchd plist）
sieve setup

# 体检
sieve doctor
```

`sieve setup` 内部做的事：

- 检测 Claude Code / Codex CLI / Cursor 是否安装；
- 把 `ANTHROPIC_BASE_URL=http://127.0.0.1:9119` 写入 `~/.claude/settings.json`；
- 注册 PreToolUse hook（双层防御）；
- 装 macOS launchd plist 让 daemon 开机自启。

### 验证拦截

```bash
# 让 Claude Code 输出一段「假」助记词（测试样本）。
# Sieve 应当截获并发起 HIPS 弹窗（GUI）或写 IPC pending file（CLI）。
sieve decisions watch   # GUI 不可用时用 CLI 接管决策
```

### 卸载

```bash
sieve uninstall   # 反向执行 setup 的全部步骤
```

---

## 配置

Sieve 读 `~/.sieve/config.toml`，可同时绑定多个上游 listener：

```toml
[[listener]]
name = "anthropic-official"
port = 9119
protocol = "anthropic"
upstream = "https://api.anthropic.com"
api_key = "${ANTHROPIC_API_KEY}"

[[listener]]
name = "openai-via-relay"
port = 9120
protocol = "openai"
upstream = "https://your-relay.example.com/v1"
api_key = "${RELAY_API_KEY}"

[detection]
sequence_detection = false   # 行为序列检测，GA 默认关闭

[telemetry]
# 默认开启匿名安装统计；SIEVE_NO_TELEMETRY=1 可全局关闭。
enabled = true
```

---

## 项目状态

仓库现已 **public**，处于 **GA 前闭测（pre-GA closed beta，仅邀请测试者）**。源码公开是为兑现信任叙事——*可验证，而非仅信任*。

质量基线：Critical 误报率（False Positive）**0.00%** / 攻击召回率（Attack Recall）**99.71%**；**clippy 0 warning**；含真实攻击复现样本的完整测试套件。

---

## 自证清白（[redacted]）

Sieve 用对待上游的同一套标准审视自己：

- **release 签名 + 可复现构建**——每个 release 都可被独立复现并验证。
- **pinned dependencies**——避免供应链事件。
- **源码公开**——拦截逻辑全部可审。
- **透明规则更新日志**——每次更新发布 changelog + 哈希，用户可独立验证。

---

## 技术栈

**Rust** + **hyper**（HTTP / 反向代理）+ **tokio**（async）+ **rustls**（TLS）+ **vectorscan-rs**（SIMD 多模式正则）+ **serde_json**（JSON 解析）。

macOS 原生 GUI 在独立仓库 [`SieveAI-dev/sieve-gui-macos`](https://github.com/SieveAI-dev/sieve-gui-macos)（SwiftUI + Combine，macOS 13+，Apple Silicon + Intel）。

---

## 定价

**Phase 1 完全免费。**

---

## 反馈

- **GitHub Issues** —— [`SieveAI-dev/sieve`](https://github.com/SieveAI-dev/sieve/issues)（公开样本提交也走这里）。
- **安全漏洞** —— 见 [SECURITY.md](./SECURITY.md)。
- **联系** —— doskey.lee@gmail.com

---

## License

- **代码**：[Apache License 2.0](./LICENSE)
- **文档**（`docs/` 下全部，以及 `README` / `CLAUDE.md` 等所有非源码 Markdown/配置）：[CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/)
