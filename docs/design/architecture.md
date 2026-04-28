# Sieve 整体架构（Phase 1）

> **状态**：设计阶段 / 锁定执行
> **文档版本**：v1.2 / 2026-04-28
> **依据 PRD**：[docs/prd/sieve-prd-v1.5.md](../prd/sieve-prd-v1.5.md)（当前权威源）；v1.4 归档见 [docs/prd/sieve-prd-v1.4.md](../prd/sieve-prd-v1.4.md)
> **范围**：Phase 1（12 周 GA），v1.5 起适配 Claude Code + OpenClaw + Hermes 三家 AI agent

---

## 1. 架构总览

### 1.1 部署拓扑（三件套架构，对应 PRD §6.1）

v1.4 引入三件套：Sieve 主代理（Rust 后台进程）、Native GUI App（独立仓库 `sieve-gui-macos`）、`sieve-hook`（独立 crate，Claude Code PreToolUse hook 入口）。

> **v1.5 multi-agent 扩展**：Sieve 主代理同时支持三家入口。v1.4 三件套架构不变，入口层扩展为：
> - **Claude Code**（ANTHROPIC_BASE_URL，沿用 v1.4 全部双层防御能力）
> - **OpenClaw**（多通道消息网关，改 daemon config 把所有 LLM provider base_url 指向 Sieve 11453 端口）
> - **Hermes Agent**（multi-provider 编排器，改每个 provider config 的 base_url；sub-agent 嵌套时 Hermes 主进程通过 `X-Sieve-Origin` header 传递调用链信息）
>
> 三家入口共用同一个 Sieve 主代理实例，Protocol Layer 按请求路径分发（`/v1/messages` → Anthropic 适配；`/v1/chat/completions` → OpenAI 适配，详见 §10.2）。

**v1.5 三入口拓扑图**（对应 PRD §6.1 重画架构图）：

```
┌────────────────────────────────────────────────────────────────────┐
│  入口 1：Claude Code（terminal coding agent）                       │
│        ↓ ANTHROPIC_BASE_URL=http://127.0.0.1:11453                 │
│        + PreToolUse hook → sieve-hook（双层防御，v1.4 不变）        │
├────────────────────────────────────────────────────────────────────┤
│  入口 2：OpenClaw daemon（多通道消息网关）                          │
│        ↓ 改 OpenClaw config → 所有 LLM provider base_url           │
│           指向 127.0.0.1:11453                                      │
│        + X-Sieve-Source-Channel header（WhatsApp/Slack/Telegram 等）│
│        ⚠ 无 PreToolUse hook 等价物 → Hook 类规则降级为 GUI hold     │
├────────────────────────────────────────────────────────────────────┤
│  入口 3：Hermes Agent（multi-provider 编排器）                      │
│        ↓ 改每个 provider config 的 base_url                        │
│        + Hermes 主进程自身调用：X-Sieve-Origin: hermes:<id>:0       │
│        + delegate 给 Claude Code / Codex CLI 时：                   │
│           X-Sieve-Origin: hermes-delegate-claude:<id>:1            │
│           （通过 ANTHROPIC_DEFAULT_HEADERS env var 注入子进程）      │
│        ⚠ 无 PreToolUse hook 等价物 → Hook 类规则降级为 GUI hold     │
└──────────────────────────────┬─────────────────────────────────────┘
                               ↓ HTTP（明文，仅本地回环）
┌──────────────────────────────────────────────────────────────────┐
│  Claude Code（仅入口 1 适用）                                     │
│    ↓ ANTHROPIC_BASE_URL=http://127.0.0.1:11453                   │
│    ↓ PreToolUse hook: sieve-hook check                           │
└──────────────┬───────────────────────────────┬───────────────────┘
               │ HTTP 请求/响应                  │ fork on PreToolUse
               ↓                                 ↓
┌──────────────────────────────┐   ┌─────────────────────────────┐
│  Sieve 主代理（Rust 后台进程）│   │  sieve-hook（独立二进制）   │
│  sieve-cli crate             │   │  sieve-hook crate           │
│                              │   │  启动 < 50ms，依赖最小化    │
│  Protocol Layer（v1.5 扩展） │   │  读 ~/.sieve/pending/       │
│   ├ anthropic.rs（v1.4）     │◄──│  写 ~/.sieve/decisions/     │
│   └ openai.rs（v1.5 新增）   │   │  TTY y/n → exit 0/1        │
│  Outbound Filter Pipeline    │   └─────────────────────────────┘
│  Upstream Forwarder          │
│  Inbound Filter Pipeline     │   IPC 通道 B（文件锁 + JSON）
│    ├ inbound_hook.rs         │
│    │  (Hook 类：写 pending)  │   IPC 通道 A（JSON-RPC Unix socket）
│    └ inbound_hold.rs         │◄──► ~/.sieve/ipc.sock
│       (GUI 类：hold SSE 流)  │
│                              │
│  IPC Channel Server          │
│  (sieve-ipc crate)           │
└──────────────────────────────┘
               ↕
┌──────────────────────────────┐
│  Native GUI App              │
│  sieve-gui-macos（独立仓库） │
│  SwiftUI macOS，常驻菜单栏   │
│  HIPS 弹窗 + 授权收集        │
│  v1.5：弹窗支持 source_agent │
│  图标 + origin_chain 调用链  │
└──────────────────────────────┘
               ↕
┌──────────────────────────────┐
│  上游 / Anthropic API        │
│  或 OpenAI 兼容 endpoint     │
│  或用户指定中转站             │
└──────────────────────────────┘
```

```
┌──────────────────────────────────────────────────────────────────┐
│  Claude Code                                                      │
│    ↓ ANTHROPIC_BASE_URL=http://127.0.0.1:11453                   │
│    ↓ PreToolUse hook: sieve-hook check                           │
└──────────────┬───────────────────────────────┬───────────────────┘
               │ HTTP 请求/响应                  │ fork on PreToolUse
               ↓                                 ↓
┌──────────────────────────────┐   ┌─────────────────────────────┐
│  Sieve 主代理（Rust 后台进程）│   │  sieve-hook（独立二进制）   │
│  sieve-cli crate             │   │  sieve-hook crate           │
│                              │   │  启动 < 50ms，依赖最小化    │
│  Protocol Layer              │   │  读 ~/.sieve/pending/       │
│  Outbound Filter Pipeline    │◄──│  写 ~/.sieve/decisions/     │
│  Upstream Forwarder          │   │  TTY y/n → exit 0/1        │
│  Inbound Filter Pipeline     │   └─────────────────────────────┘
│    ├ inbound_hook.rs         │
│    │  (Hook 类：写 pending)  │   IPC 通道 B（文件锁 + JSON）
│    └ inbound_hold.rs         │
│       (GUI 类：hold SSE 流)  │
│                              │   IPC 通道 A（JSON-RPC Unix socket）
│  IPC Channel Server          │◄──► ~/.sieve/ipc.sock
│  (sieve-ipc crate)           │
└──────────────────────────────┘
               ↕
┌──────────────────────────────┐
│  Native GUI App              │
│  sieve-gui-macos（独立仓库） │
│  SwiftUI macOS，常驻菜单栏   │
│  HIPS 弹窗 + 授权收集        │
│  Preset 设置面板             │
└──────────────────────────────┘
               ↕
┌──────────────────────────────┐
│  上游 / api.anthropic.com    │
│  或用户指定中转站             │
└──────────────────────────────┘
```

> GUI App 代码在独立仓库 `sieve-gui-macos`（与本仓库同样私有至 Week 12 GA，见 [ADR-011](./ADR-011-private-until-ga.md)）。两仓库唯一硬约束是 IPC 协议版本号（`sieve-ipc` crate，详见 [ADR-013](./ADR-013-ipc-protocol.md)）。

### 1.2 数据流图（双向检测，含 IPC 分支）

```mermaid
flowchart LR
    Dev[开发者 / Claude Code]
    Hook[sieve-hook<br/>PreToolUse 拦截]
    GUI[Native GUI App<br/>sieve-gui-macos]

    subgraph Sieve [Sieve 主代理 127.0.0.1:11453]
        direction TB
        Proto[Protocol Layer<br/>Anthropic 适配]
        Out[Outbound Filter<br/>出站检测]
        Fwd[Upstream Forwarder<br/>reqwest + rustls]
        InHook[inbound_hook.rs<br/>Hook 类：写 pending 文件]
        InHold[inbound_hold.rs<br/>GUI 类：hold SSE 流]
        Agg[Tool Use Aggregator<br/>+ AddressGuard]
    end

    Up[上游 / api.anthropic.com]

    Dev -->|① HTTPS 请求| Proto
    Proto -->|② UnifiedMessage| Out
    Out -->|③ AutoRedact / GuiPopup / StatusBar| Fwd
    Fwd -->|④ 转发（可能脱敏的）prompt| Up
    Up -->|⑤ SSE 流式响应| Agg
    Agg -->|⑥ disposition 分流| InHook
    Agg -->|⑥ disposition 分流| InHold
    InHook -->|⑦a 写 pending 文件 + SSE 原样透传| Dev
    InHook -.->|IPC 通道 B（文件锁）| Hook
    Hook -->|exit 0/1| Dev
    InHold -->|⑦b hold SSE + keep-alive 25s| Dev
    InHold -.->|IPC 通道 A（Unix socket JSON-RPC）| GUI
    GUI -->|用户决策 allow/deny| InHold
    InHold -->|⑧ 继续 SSE 或 sieve_blocked event| Dev
```

> 关键性质：**所有检测纯本地**，没有任何分支会把 prompt 发到 Anthropic 以外的 host（[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）。Hook 类规则代理侧不修改 SSE 流（[ADR-014](./ADR-014-dual-layer-defense.md)）；GUI 类规则 hold 流期间每 25 秒发送 SSE comment `: keep-alive\n\n` 防超时。

---

## 2. 模块职责矩阵

### 2.1 本仓库 crate

| 模块（crate）                  | 职责                                                                                                   | 输入                        | 输出                                              | 关键依赖                                                    |
| ---------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------- | ----------------------------------------------- | ------------------------------------------------------- |
| **Protocol Layer**<br/>(`sieve-core`) | 解析 Anthropic Messages API 请求/响应；将原始 JSON 映射到 `UnifiedMessage`；接口预留 OpenAI/OpenRouter（不实现） | 原始 HTTP/JSON 字节流 | `UnifiedMessage` 结构 | `hyper 1.x`、`tokio`、`sonic-rs` |
| **Outbound Filter Pipeline**<br/>(`sieve-rules`) | 对 outbound `UnifiedMessage` 执行 OUT-01~12 规则；产出 `Detection` 列表；按处置矩阵路由到 AutoRedact / GuiPopup / StatusBar | `UnifiedMessage` | `(UnifiedMessage_可能脱敏, Vec<Detection>, Disposition)` | `vectorscan-rs`、`bip39`、`bs58`、`hex`、`sha2`、`crc32fast` |
| **Upstream Forwarder**<br/>(`sieve-core`) | 将（可能脱敏的）请求转发到上游；保持 SSE 长连接、TLS 终结、超时与重试 | 已检测的 outbound 请求 | 上游 SSE 字节流 | `reqwest`、`rustls`、`tokio` |
| **SSE Parser**<br/>(`sieve-core`) | 流式切分 `event:` / `data:` 行；处理半行 chunk、跨 chunk 分隔、C0 控制字符、多 event 粘包、提前断流 | 上游字节流 | 完整 SSE event 序列 | 自研 + `bytes` |
| **Tool Use Aggregator**<br/>(`sieve-core`) | 聚合 `tool_use` block 直到 JSON 完整（partial-json-parser）；在工具调用边界触发 disposition 分流 | SSE event 序列 | 完整 `tool_use` 对象 + `Disposition` | 自研 partial JSON parser |
| **inbound_hook.rs**<br/>(`sieve-core`) | Hook 类规则命中后写 `~/.sieve/pending/<id>.json`；**不修改 SSE 流**，原样透传给 Claude Code | `tool_use` 对象 + `HookTerminal` disposition | pending 文件 + 透传 SSE | `sieve-ipc`（IPC 通道 B） |
| **inbound_hold.rs**<br/>(`sieve-core`) | GUI 类规则命中后 hold SSE 流；每 25s 发 keep-alive comment；经 IPC 通道 A 等待用户决策；用户拒绝时注入 `sieve_blocked` event | `tool_use` 对象 + `GuiPopup` disposition | hold 住的 SSE + keep-alive / sieve_blocked event | `sieve-ipc`（IPC 通道 A） |
| **AddressGuard**<br/>(`sieve-rules`) | 维护本会话所有出现过的 `0x[a-fA-F0-9]{40}`；对模型新输出地址做：完全相同放行 / Levenshtein ≤ 4 标黄/触发 IN-CR-01 | 会话历史地址集合 + 新地址 | `Detection`（含相似度证据） | `strsim`（Levenshtein）、`hashbrown` |
| **sieve-ipc**<br/>（独立 crate） | IPC 协议库：Unix socket JSON-RPC server（通道 A）、pending/decisions 文件 IO（通道 B）、`~/.sieve/` 目录管理 | `DecisionRequest` / `DecisionResponse` | IPC 消息收发 + 文件读写 | `tokio`、`serde_json`、`fd-lock`、`uuid` |
| **sieve-hook**<br/>（独立 crate，独立二进制） | Claude Code PreToolUse hook 入口；启动 < 50ms；读 pending 文件；TTY y/n 倒计时；写 decisions 文件；exit 0/1 | `~/.sieve/pending/` 目录 | `~/.sieve/decisions/` 文件 + exit code | `serde_json`、`fd-lock`（最小依赖，禁止引入 vectorscan / rusqlite） |
| **sieve-cli**<br/>（入口 crate） | 入口 / 配置加载 / `sieve setup` / `sieve doctor` / `sieve uninstall`（macOS only）/ 审计日志（SQLite）/ launchd 守护 | CLI args + `config.toml` | 启动 daemon / 管理命令输出 | `anyhow`（仅此 crate 允许）、`rusqlite`、`clap` |
| **协议适配层 `protocol/openai.rs`**<br/>（`sieve-core`，**v1.5 新增**） | 解析 OpenAI Chat Completions API 请求/响应；将 delta / function_call / tool_calls 映射到 `UnifiedMessage`；处理 `data: [DONE]` 流结束标记 | 原始 HTTP/JSON 字节流（OpenAI 协议格式） | `UnifiedMessage`（与 anthropic.rs 输出一致，下游 Filter Pipeline 无感知） | `serde`、`sonic-rs`（与 anthropic.rs 共用） |

> **关联决策**：协议适配层设计见 [ADR-018](./ADR-018-openai-protocol-adaptation.md)（OpenAI 协议适配，由子代理 G2 撰写）。

> **Native GUI App**（SwiftUI，常驻菜单栏、HIPS 弹窗、Preset 设置面板）在独立仓库 **`sieve-gui-macos`**，不在本 workspace。两仓库的协调契约是 `sieve-ipc` crate 中 IPC 协议版本（`v1` 起），详见 [ADR-012](./ADR-012-native-gui-app-phase1.md) + [ADR-013](./ADR-013-ipc-protocol.md)。

> **共用依赖**：配置（`serde` + `toml`）、SQLite 审计日志（`rusqlite`，仅 `sieve-cli`）、license 验证（Ed25519 + JWT-like，详见 [data-model.md](./data-model.md) §8）。

---

## 3. 关键技术决策一览


| ID      | 决策                                        | 摘要                                                                                   | 链接                                                        |
| ------- | ----------------------------------------- | ------------------------------------------------------------------------------------ | --------------------------------------------------------- |
| ADR-001 | 选用 Rust 作为技术栈                             | hyper + tokio + rustls + vectorscan-rs + sonic-rs；Go regexp 慢 1000+ 倍，Python GIL 不可控 | [ADR-001](./ADR-001-rust-tech-stack.md)                   |
| ADR-002 | Phase 1 纯规则引擎，不引入本地 ML 模型                 | 三个独立论证：结构化优先 / 误报敏感 / 单人团队数据标注稀缺                                                     | [ADR-002](./ADR-002-rule-engine-only-phase1.md)           |
| ADR-003 | 完全本地运行，绝不联网 verifier                      | 不上传 prompt、不上传 fingerprint、不做云端 token 校验                                             | [ADR-003](./ADR-003-local-only-no-cloud-verifier.md)      |
| ADR-004 | Phase 1 只适配 Anthropic，UnifiedMessage 接口预留 | 公理 7：不为想象用户写代码；第二适配等真实付费用户主动提                                                        | [ADR-004](./ADR-004-anthropic-first-unified-interface.md) |
| ADR-005 | [redacted]作为收款与营销载体                           | 香港 → 新加坡 → [redacted]；[redacted]                                                      | [ADR-005](./ADR-005-overseas-legal-entity.md)             |
| ADR-006 | Sigstore 签名 + Reproducible Build + 透明日志   | 自证清白是产品定位（PRD §1.2 第 4 句），不只是工程实现                                                    | [ADR-006](./ADR-006-sigstore-reproducible-build.md)       |
| ADR-007 | Critical 等级 fail-closed，YOLO mode 不可关闭    | 签名工具调用 / rm -rf / curl|sh / eval(base64) 永远强制确认                                      | [ADR-007](./ADR-007-fail-closed-critical-actions.md)      |


---

## 4. 性能预算


| 操作                | 目标延迟        |
| ----------------- | ----------- |
| 普通 token 流式 chunk | +30–200 µs  |
| 工具调用边界完整检查        | +5–15 ms    |
| 整体 P99 添加延迟       | **< 20 ms** |
| 内存峰值              | < 100 MB    |
| 二进制大小             | < 20 MB 单文件 |
| 启动时间              | < 500 ms    |


**说明**：

- 普通 chunk（30–200 µs）走 vectorscan stream mode + 简单 entropy 计算，必须在用户感知阈值之下；
- 工具调用边界（5–15 ms）允许更重的检查（partial JSON 重组、AddressGuard 历史比对、多模式联合规则），因为这是不可逆动作前的最后一道闸；
- P99 < 20 ms 是面向 Claude Code 长会话的硬约束，超出意味着用户感知到"代理变慢了"，会触发卸载；
- 内存 100 MB 上限确保普通 dev 笔记本（16 GB RAM 是基线）在重度多窗口场景下 Sieve 不挤占其他进程；
- 二进制 < 20 MB + 启动 < 500 ms 是分发体验线，要确保 `.dmg` 安装后立即可用；
- IPC 往返（主代理 → GUI → 主代理，不含用户决策时间）：< 50 ms；
- `sieve-hook` 启动时延（fork → TTY 出现）：< 50 ms（依赖最小化，实测 4–5 ms）；
- GUI 类规则 hold 流期间 keep-alive comment 间隔：**25 s**；
- IN-CR-05（签名工具）最长 hold 时长：**120 s**（超时 fail-closed）。

参考：[PRD §6.4](../prd/sieve-prd-v1.5.md)。

---

## 5. 误报率预算


| 检测类型     | Critical 拦截 FP 上限                  | High Warn FP 上限 |
| -------- | ---------------------------------- | --------------- |
| OUT-*    | < 0.5%（单条 Critical 各自上限见 PRD §5.1） | < 5%            |
| IN-CR-*  | < 0.5%                             | < 3%            |
| IN-GEN-* | N/A（全部 High 及以下）                   | < 10%           |


> **公理 12**：**Critical FP > 0.5% → 用户禁用产品**。这是硬约束，不是工程优化项。任何 Critical 规则在 dogfood / 闭测期间触发 FP 即被回滚或降级到 High。

参考：[PRD §6.5](../prd/sieve-prd-v1.5.md)。

---

## 6. 部署形态（v1.4 三件套）

Phase 1 部署形态为 **macOS .dmg 安装包**，包含三件套：Native GUI App + Rust 后台代理 + `sieve-hook` 命令行。分发渠道与系统集成方式：

| 维度 | 选型 |
|------|------|
| 分发 | macOS `.dmg` 安装包（含 GUI App + 后台代理 + `sieve-hook`，带 sigstore 签名 + Notarization） |
| 配置 | `~/.sieve/config.toml` + 环境变量覆盖 |
| 安装引导 | `.dmg` 安装后运行 `sieve setup`：自动写入 Claude Code `settings.json` hook 注册项（`onError: block`）、配置 `ANTHROPIC_BASE_URL`、生成 launchd plist |
| 守护 | macOS：`launchd` user agent（`~/Library/LaunchAgents/com.sieve.daemon.plist`）；`sieve setup` 自动注册开机自启 |
| 客户端接入 | `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`（`sieve setup` 自动配置，详见 [ADR-015](./ADR-015-sieve-setup-tool.md)） |
| PreToolUse hook | `sieve-hook check`（`sieve setup` 写入 Claude Code hook 注册，`onError: block` 保证 fail-closed） |
| IPC | 通道 A：`~/.sieve/ipc.sock`（Unix socket JSON-RPC，代理 ↔ GUI）；通道 B：`~/.sieve/pending/` + `~/.sieve/decisions/`（文件锁 JSON，代理 ↔ sieve-hook） |
| 可观测 | 本地 SQLite 审计日志（`~/.sieve/audit.db`，append-only）+ `sieve doctor` 全链路自检 |

**v1.4 显式不做**：

- ✅ macOS SwiftUI 独立进程（`sieve-gui-macos` 仓库）—— **已撤销**原 v1.3 中"❌ 桌面 GUI App（Electron / Tauri）"的否决；该否决只针对 webview 方案，不适用 native SwiftUI（详见 [ADR-012](./ADR-012-native-gui-app-phase1.md)）
- ❌ 操作系统级 Network Extension / 本地 CA 注入 / MITM（推 Phase 3，[PRD §6.8 + §9 第 12 条](../prd/sieve-prd-v1.5.md)）
- ❌ Linux / Windows 平台支持（推 Phase 2，触发条件见 [ADR-004 §3](./ADR-004-anthropic-first-unified-interface.md)）
- ❌ VS Code 插件 / 浏览器扩展
- ❌ 修改 `~/.zshrc` / `~/.bashrc`（PATH 由 .dmg 安装包的 post-install script 处理，`sieve setup` 不写 shell rc，详见 [ADR-015](./ADR-015-sieve-setup-tool.md)）

**v1.5 新增（multi-agent setup）**：

- ✅ multi-agent 配置注入（v1.5 起）：`sieve setup --agent claude|openclaw|hermes`，支持多 `--agent` 参数同时配置，以及 `sieve setup --all-detected`（自动检测系统已装的 agent，逐个 dry-run + 确认）。详见 [SPEC-004](../specs/SPEC-004-multi-agent-setup.md)。
- 三家 agent 各自配置注入路径：

| Agent | 配置目标 | 注入字段 |
|-------|---------|---------|
| Claude Code | `~/.claude/settings.json` | `env.ANTHROPIC_BASE_URL` + `hooks.PreToolUse` |
| OpenClaw | `~/.openclaw/config.toml` | provider router 表里所有 `base_url` |
| Hermes | `~/.hermes/config.toml` 或 `.env` | 每个 provider 的 `base_url` |

---

## 7. Phase 2 演进路径（触发条件，不是路线图）

下面四件事**只在条件触发时启动**，不进入 12 周里程碑：


| Phase 2 能力           | 触发条件                                                           |
| -------------------- | -------------------------------------------------------------- |
| 二阶段轻量 ML 分类器         | 用户实际 High FP 持续 4 周 > 5%，**且**至少 10 个真实付费用户主动反馈"误报太多"          |
| MCP 拦截（IN-MCP-01~03） | Week 16–20 启动；前提是 Phase 1 GA + 至少 1 个闭测用户在 dogfood 中触发过 MCP 调用 |
| 协议白名单 + Drainer 黑名单  | Phase 2 数据合作落地后（慢雾 misttrack-skills / ScamSniffer Pro 接通）      |
| OpenClaw / Hermes 适配 | **第二个真实付费用户**主动要求时（公理 7：不为想象用户写代码）                             |


> 这是"不做承诺，只做触发器"的原则——Phase 2 路线图的灵活性决定了[redacted]在 12 周后能否快速响应真实用户。

---

## 7.5 已知架构限制（Week 4 dogfood 实测发现，**Week 4 内必须关闭**）

### 入站检测仅覆盖流式 SSE 响应

**现状**：`Inbound Filter Pipeline` 实现 `forward_with_inbound_inspection` 假定 upstream
response body 是 `text/event-stream` 字节流，喂给 `SseParser` + `Aggregator` 才能解析
出 tool_use。对 `application/json` 单体响应（Anthropic Messages API 不传 `stream:true`
时的默认格式）整个 body 原样透传，**所有入站规则全部失效**——IN-CR-02 / IN-CR-03 /
IN-CR-04 / IN-CR-05 / IN-GEN-* 一律被绕过。

**风险等级**：P0。攻击面在 PRD §5.2「入站是 Sieve 真正的护城河」语境下属严重产品级
缺陷——攻击者只要让 SDK 发非流式请求，就能让模型在 tool_use 里写 `>> ~/.bashrc` /
`eth_signTransaction` 等而 Sieve 完全看不到。**违反公理 12（Critical FP < 0.5%）的
镜像约束：Critical 漏报率不可控**。

**修复路径**（Week 4 内必须落地）：
1. `daemon::proxy_inner` 在 forward 完上游后，按 `response.headers["content-type"]`
   分流：`text/event-stream` → 现有 SSE 路径；`application/json` → 新增 JSON 路径
2. JSON 路径 collect 完整 body，反序列化为 `AnthropicResponse`，遍历 `content[]`
   提取 `tool_use` 块，手工构造 `CompletedToolCall` 喂 `InboundFilter::on_tool_use_complete`
3. 命中 fail-closed Critical 时把 body 替换为 `sieve_blocked` 等价 JSON 错误结构
   （HTTP 200 + `{"type":"error",...,"sieve_blocked":...}`），同时更新 content-length
4. 容量上限参考既有 SSE channel cap，单 message body 上限 ~8MB
5. 集成测试 `inbound_block.rs` 加 mock 非流式 upstream case 强制覆盖

**v1.4 双层防御后的修复路径细化**：

- **Hook 类（IN-CR-02~04、IN-GEN-01~03）**：这些规则由 `sieve-hook` 在 Claude Code PreToolUse 阶段拦截，不依赖 SSE 流处理路径。非流式 JSON bypass 对 Hook 类规则**不再适用**——即使代理层看不到非流式 body，只要 Claude Code 发起 PreToolUse，sieve-hook 仍会读 pending 文件并拦截。上述修复路径主要针对 pending 文件写入仍依赖代理检测的场景。
- **GUI 类（IN-CR-01/05、IN-GEN-04）**：这些规则仍依赖代理的 SSE 流处理（hold 流 + IPC 通知 GUI）；非流式 JSON bypass 对 GUI 类仍然有效，修复路径 1-5 必须完整落地。

**关联**：[CHANGELOG Known Issues §Week 4](../changelog/CHANGELOG.md) /
[tasks/lessons.md 「入站检测仅覆盖流式 SSE」](../../tasks/lessons.md) /
[roadmap Week 4](../../tasks/roadmap.md)

---

## 8. 不在 Phase 1 范围

为防范围蔓延，以下能力**显式标记为不在 Phase 1**：

- ✅ ~~OpenAI / OpenRouter / Hermes / OpenClaw 协议适配（接口预留，不实现）~~（**v1.5 撤销此限制**：OpenAI 协议 + OpenClaw / Hermes 适配已纳入 Phase 1，Week 6-7 实现，见 §10.2 + [ADR-018](./ADR-018-openai-protocol-adaptation.md)）
- ❌ 本地 ML 模型 / ONNX / 任何分类器（见 [ADR-002](./ADR-002-rule-engine-only-phase1.md)）
- ❌ VS Code 插件 / 浏览器扩展（推 Phase 2）
- ❌ Cursor 适配（推 Phase 2）
- ❌ 操作系统级 Network Extension / 本地 CA 注入 / MITM（推 Phase 3，PRD §6.8 + §9 第 12 条）
- ❌ Linux / Windows 平台支持（推 Phase 2）
- ❌ Gemini / Mistral / Cohere / Ollama 等其他 LLM 协议（推 Phase 2；Phase 1 仅实现 Anthropic + OpenAI 两种上游协议）
- ❌ 修改 `~/.zshrc` / `~/.bashrc`（PATH 由 .dmg 安装包处理，详见 [ADR-015](./ADR-015-sieve-setup-tool.md)）
- ❌ 企业团队功能（多用户、SSO、审批工作流、SOC2）
- ❌ 云同步（配置 / 规则 / 审计日志全部本地，[ADR-003](./ADR-003-local-only-no-cloud-verifier.md)）
- ❌ 中文 PII / 内网域名 / 自定义规则 DSL（PRD §5.1 Phase 2）
- ❌ npm / pip typosquat、Markdown 链接钓鱼、Unicode 攻击、Calldata 解码、ERC20 危险 approve、Drainer 黑名单（PRD §5.2 Phase 2）
- ❌ [redacted] / [redacted] / [redacted]（PRD §7.3）
- ❌ 给 OpenClaw / Hermes 提 PR 实现 hook 等价物（Phase 1 后期目标，不阻塞 GA；降级 GUI hold 已 100% fail-closed）

---

## 10. Multi-Agent 扩展架构（v1.5 新增）

> 本章对应 PRD v1.5 §6.1~6.7，描述三家 AI agent 适配的工程架构细节。v1.4 章节（§1~§9）保持不变，本章独立增补。

### 10.1 三家 agent 的拓扑差异

| 维度 | Claude Code | OpenClaw | Hermes Agent |
|------|-------------|----------|--------------|
| 上游 LLM 协议 | Anthropic Messages API | OpenAI Chat Completions（多 provider） | OpenAI Chat Completions（多 provider） |
| Sieve 接入方式 | `ANTHROPIC_BASE_URL` env var | daemon config `base_url` 字段 | provider config `base_url` 字段 |
| Hook 机制 | ✅ PreToolUse hook → `sieve-hook`（双层防御） | ❌ 无等价物 | ❌ 无等价物 |
| Hook 类规则处置 | 终端 y/N 弹窗（IPC 通道 B） | **降级为 GUI hold** | **降级为 GUI hold** |
| Critical 规则处置 | GUI hold（IPC 通道 A，不可关闭） | GUI hold（不可关闭） | GUI hold（不可关闭） |
| X-Sieve-Origin 注入方 | `sieve setup` 写 Claude Code settings | 用户手动 or `sieve setup --agent openclaw` | Hermes 自身通过 `ANTHROPIC_DEFAULT_HEADERS` 注入子进程 |
| sub-agent 嵌套 | 不适用（Claude Code 是叶节点） | 不适用 | ✅ Hermes delegate 给 Claude Code / Codex，chain_depth ≥ 1 |
| 双层防御退化场景 | 无退化 | Hook 类退化为 GUI hold（UX 退步，安全不退化） | 同 OpenClaw；Phase 1 后期提 PR 升级 |

> 关键约束：**Critical 规则在三家 agent 上都不可关闭**（PRD §9 第 3 + 8 + 11 条），降级只影响 Hook 类规则（High 及以下）的 UX 路径。

### 10.2 协议适配层

v1.5 Protocol Layer 新增 `openai.rs`，与 `anthropic.rs` 并列：

```
crates/sieve-core/src/protocol/
├── mod.rs          # 按请求路径分发（/v1/messages → anthropic, /v1/chat/completions → openai）
├── anthropic.rs    # v1.4 已实现
└── openai.rs       # v1.5 新增（详见 ADR-018）
```

**路径分发规则**：

| 请求路径 | 协议适配器 | 适用 agent |
|---------|-----------|-----------|
| `POST /v1/messages` | `anthropic.rs` | Claude Code |
| `POST /v1/messages/count_tokens` | `anthropic.rs` | Claude Code |
| `POST /v1/chat/completions` | `openai.rs` | OpenClaw / Hermes |
| 其他 `/v1/...` | 501 Not Implemented | — |

**UnifiedMessage 中间表示**：两个协议适配器输出相同的 `UnifiedMessage` schema，下游 Filter Pipeline 无感知。SSE 格式差异由适配器内部消化：
- Anthropic：`event: content_block_delta` + `data: {...}` 结构
- OpenAI：`data: {"choices":[{"delta":{...}}]}` + `data: [DONE]` 结束标记

关联决策：[ADR-018](./ADR-018-openai-protocol-adaptation.md)（OpenAI 协议适配，包含 UnifiedMessage 映射细节）。

### 10.3 X-Sieve-Origin header 协议

用于解决 Hermes sub-agent 嵌套调用时的双重弹窗问题（PRD §4.6 场景 F）。

**header 格式**：`X-Sieve-Origin: <source_agent>:<request_id>:<chain_depth>`

**示例**：
- `X-Sieve-Origin: claude:abc-123:0`（用户直接调 Claude Code）
- `X-Sieve-Origin: hermes:def-456:0`（用户直接调 Hermes）
- `X-Sieve-Origin: hermes-delegate-claude:def-456:1`（Hermes 转给 Claude Code，chain_depth=1）

**关键语义**：
- `chain_depth ≥ 2`：强制 GUI hold，嵌套深度过大的调用属于可疑行为
- `chain_depth ≥ 5`：直接返回 426（调用链过深，拒绝处理）
- 已有父层 allow 记录（同 `request_id`）且 `chain_depth > 0`：子层弹窗去重，不再重复询问用户

**注入机制**：Hermes 启动 Claude Code 子进程时，通过 `ANTHROPIC_DEFAULT_HEADERS` 环境变量自动注入，用户无需手动配置。

**防伪造**：header 携带 Ed25519 签名（公钥由 `sieve setup` 预置，私钥由 Sieve 主代理持有）；伪造或无效签名的 header 视同无 header 处理（chain_depth=0，不降级）。

关联决策：[ADR-019](./ADR-019-x-sieve-origin-header.md)（X-Sieve-Origin header 协议完整规格）。

### 10.4 Hook 类规则降级机制

OpenClaw / Hermes 没有 PreToolUse hook 等价物，Hook 类规则（IN-CR-02~04、IN-GEN-01~03）在这两家 agent 上的处置路径从"终端 y/N 弹窗"降级为"GUI hold"：

| 规则类型 | Claude Code 处置 | OpenClaw / Hermes 降级处置 |
|---------|----------------|--------------------------|
| Hook 类 Critical（IN-CR-02~04） | sieve-hook 终端 y/N，30s 超时 fail-closed | GUI hold，30s 超时 fail-closed |
| Hook 类 High（IN-GEN-01~03） | sieve-hook 终端 y/N | GUI hold |
| GUI 类 Critical（IN-CR-01/05/06） | GUI hold（v1.4 不变） | GUI hold（相同路径） |

**降级影响说明**：
- 安全性不退化：Critical 仍然 fail-closed，只是弹窗从终端切到 GUI
- UX 退步：OpenClaw / Hermes 用户每次工具调用都会跳 GUI 弹窗，比 Claude Code 用户（终端内决策）更打断流
- 缓解措施：用户可在 GUI Preset 面板选择"Trust this agent's tool boundary"，但 **Critical 类规则仍强制 GUI hold**（PRD §9 第 3 + 8 + 11 条不可放宽）

Phase 1 后期目标（不阻塞 GA）：给 OpenClaw / Hermes 提 PR 实现 `pre_skill_invoke` hook 等价物，升级为双层防御。

### 10.5 Multi-Agent 配置注入

`sieve setup --agent` 参数新增三家 agent 的配置注入能力，详细规格见 [SPEC-004](../specs/SPEC-004-multi-agent-setup.md)。

**关键行为约束**：
- 与 v1.4 `sieve setup`（无 `--agent` 参数，默认配置 Claude Code）向后兼容
- 每次配置修改前打印 diff + 要求 `y` 确认；原始文件备份到 `~/.sieve/backups/`
- `sieve setup --all-detected` 自动检测系统已安装的 agent（扫描 `~/.openclaw/`、`~/.hermes/` 等路径），逐个 dry-run 展示将要修改的字段
- `sieve doctor --agent openclaw` 支持单独诊断某家 agent 的接入状态
- 退出码：`0` 全部成功 / `1` 至少一个 agent 配置失败但已回滚 / `2` 部分失败且回滚也失败（紧急状态，stderr 输出需要手动清理的步骤）

---

## 11. 相关文档

- [PRD-sieve v1.5](../prd/sieve-prd-v1.5.md)
- [data-model.md](./data-model.md) —— UnifiedMessage / Detection / 配置 / 审计日志 schema
- [ADR-001](./ADR-001-rust-tech-stack.md) —— Rust 技术栈
- [ADR-002](./ADR-002-rule-engine-only-phase1.md) —— Phase 1 纯规则引擎
- [ADR-003](./ADR-003-local-only-no-cloud-verifier.md) —— 完全本地，零云依赖
- [ADR-004](./ADR-004-anthropic-first-unified-interface.md) —— Anthropic-first，UnifiedMessage 接口预留
- [ADR-005](./ADR-005-overseas-legal-entity.md) —— [redacted]
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— Sigstore + Reproducible Build
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— Critical fail-closed
- [ADR-012](./ADR-012-native-gui-app-phase1.md) —— Native GUI App（macOS SwiftUI，独立仓库）
- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议（双通道：Unix socket + 文件锁）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（Hook 类 + GUI 类分流）
- [ADR-015](./ADR-015-sieve-setup-tool.md) —— sieve setup / doctor / uninstall
- [ADR-016](./ADR-016-disposition-matrix-2d.md) —— 处置矩阵二维化
- [SPEC-001](../specs/SPEC-001-sieve-hook-protocol.md) —— sieve-hook 文件 IPC 协议
- [SPEC-002](../specs/SPEC-002-hips-popup-behavior.md) —— HIPS 弹窗行为规范
- [SPEC-003](../specs/SPEC-003-sieve-setup-tool.md) —— sieve setup 详细规格
- [SPEC-004](../specs/SPEC-004-multi-agent-setup.md) —— multi-agent 配置注入规格（v1.5 新增）
- [ADR-018](./ADR-018-openai-protocol-adaptation.md) —— OpenAI 协议适配（v1.5 新增）
- [ADR-019](./ADR-019-x-sieve-origin-header.md) —— X-Sieve-Origin header 协议（v1.5 新增）
- `docs/api/api-reference.md` —— 反向代理 API 适配细节 + 环境变量 + 配置 schema（含 v1.5 X-Sieve-Origin §7）

