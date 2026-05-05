# SPEC-004: multi-agent setup 配置注入规格

> Version: v1.0 — 2026-04-28
> Status: Stable
> 关联 ADR：ADR-015（sieve setup 工具基础）、ADR-018（OpenAI 协议适配）、ADR-019（X-Sieve-Origin header 协议）
> 关联 PRD：v2.0 §6.6 §6.7 §10 Week 6（v1.5 引入）

---

## 1. 概述

### 1.1 设计原则

**引擎一样，配置不同**。Sieve 的检测引擎（sieve-rules 规则匹配、处置矩阵、GUI 弹窗、审计日志）在三家 agent 上 100% 复用。`sieve setup` 的 multi-agent 扩展只解决"不同 agent 的配置文件路径和字段格式不同"的问题。

- 代理层（sieve-core daemon）：对三家 agent 一视同仁，统一监听 `127.0.0.1:11453`
- 检测规则：不区分 agent，规则 ID 通用
- 配置注入：每家 agent 有独立的注入路径和字段，本 SPEC 分章详述

### 1.2 范围

本 SPEC 覆盖：
- `sieve setup --agent <name>` 的多 agent 参数扩展
- 三家 agent（Claude Code / OpenClaw / Hermes）各自的检测逻辑、配置注入步骤、回滚逻辑
- `sieve doctor --agent <name>` 的多 agent 检查项
- `setup.log` 的 multi-agent 扩展字段

**不覆盖**：OpenAI 协议解析细节（属 ADR-018 范围）、X-Sieve-Origin header 协议细节（属 ADR-019 范围）、检测规则算法（属 PRD §5.2 和各规则 SPEC 范围）。

### 1.3 三家 agent 特性对比

| 特性 | Claude Code | OpenClaw | Hermes |
|------|------------|---------|--------|
| 协议 | Anthropic Messages API | OpenAI Chat Completions（主） | OpenAI Chat Completions（主） |
| PreToolUse hook 等价物 | ✅ `hooks.PreToolUse` | ❌ 暂无 | ❌ 暂无 |
| Hook 类规则处置 | 终端 y/N 弹窗 | **降级为 GUI hold** | **降级为 GUI hold** |
| sub-agent 嵌套 | 不发起（被嵌套） | 不嵌套 | 嵌套 Claude Code 子进程 |
| config 文件格式 | JSON / JSONC | JSON（`~/.openclaw/openclaw.json`） | YAML（`~/.hermes/config.yaml`） |
| 配置注入方式 | env var + hook | provider router base_url | provider base_url |

---

## 2. 命令行接口

### 2.1 sieve setup

```bash
# 单 agent（沿用 SPEC-003）
sieve setup --agent claude

# 单独装某个 agent
sieve setup --agent openclaw
sieve setup --agent hermes

# 同时装多家（顺序处理：claude → openclaw → hermes）
sieve setup --agent claude --agent openclaw
sieve setup --agent claude --agent openclaw --agent hermes

# 自动检测系统已安装的 agent，逐个 dry-run + 用户确认
sieve setup --all-detected

# 全局标志（已有，适用 multi-agent）
sieve setup --agent openclaw --yes    # 跳过确认提示
```

`--agent` 参数值：`claude` | `openclaw` | `hermes`（大小写不敏感）。

传入未知值时输出：
```
未知 agent: "<value>"。支持的值：claude, openclaw, hermes
```
并 exit 2。

### 2.2 sieve doctor

```bash
sieve doctor                     # 检查所有已通过 setup 配置的 agent
sieve doctor --agent claude      # 只检查 Claude Code
sieve doctor --agent openclaw    # 只检查 OpenClaw
sieve doctor --agent hermes      # 只检查 Hermes
sieve doctor --json              # JSON 格式输出（全部 agent）
```

### 2.3 sieve uninstall

```bash
sieve uninstall --agent claude      # 只回滚 Claude Code 的改动
sieve uninstall --agent openclaw    # 只回滚 OpenClaw 的改动
sieve uninstall --agent hermes      # 只回滚 Hermes 的改动
sieve uninstall --all               # 移除所有 agent 适配（按 setup.log 逆序全部回滚）
```

不传 `--agent` 且不传 `--all` 时：

```
请指定 --agent <name> 或 --all。
```
并 exit 2。（**注**：SPEC-003 没有 `--agent` 参数的 uninstall；v1.5 起此为默认行为，无参数不再自动推断。）

---

## 3. 检测逻辑（detect_ai_agents 扩展）

`sieve setup --all-detected` 时，`detect_ai_agents()` 依次探测三家。`--agent <name>` 时只探测指定 agent，不探测其他。

### 3.1 Claude Code 检测（沿用 SPEC-003）

| 检查 | 条件 |
|------|------|
| 配置文件 | `~/.claude/settings.json` 存在 |
| 二进制 | `claude` 在 `$PATH`（`which claude` 返回 exit 0） |

两项同时满足 → detected。仅配置文件存在 → detected + 警告"未找到 claude 二进制，setup 继续但请确认 Claude Code 已安装"。

### 3.2 OpenClaw 检测

| 检查 | 条件 |
|------|------|
| 配置目录（主） | `~/.openclaw/` 目录存在 |
| 配置目录（备） | `~/Library/Application Support/openclaw/` 目录存在（macOS 可能路径） |
| 二进制 | `openclaw` 在 `$PATH`（通过 npm 全局安装） |
| daemon 状态 | `openclaw doctor` 命令返回 exit 0（TBD-03 已解决：doctor 为官方诊断命令，Week 8 验证 exit code 语义） |

**detected 条件**：配置目录存在 OR 二进制存在，任一满足即视为已安装。

daemon 未运行时：输出提示但不中止 setup（daemon 未运行时配置注入同样有效）。

**未检测到时**：
```
未找到 OpenClaw 安装（~/.openclaw/ 和 openclaw 二进制均未找到）。
跳过 OpenClaw 配置。如已安装，请先运行 openclaw 确认路径后重试。
```

### 3.3 Hermes 检测

| 检查 | 条件 |
|------|------|
| 配置文件（主） | `~/.hermes/config.yaml` 存在（TBD-02 已解决：YAML 格式） |
| 配置文件（备） | `~/.hermes/.env` 存在（仅存 API key，不含 base_url） |
| 二进制 | `hermes` 在 `$PATH` |
| provider 验证 | `hermes config check` 返回 exit 0（TBD-04 已解决：`hermes config providers list` 不存在，实际用 `hermes config check`） |

**detected 条件**：配置文件存在 OR 二进制存在，任一满足即视为已安装。

**未检测到时**：
```
未找到 Hermes 安装（~/.hermes/ 和 hermes 二进制均未找到）。
跳过 Hermes 配置。
```

---

## 4. 配置注入步骤（每家分开）

### 4.1 Claude Code（沿用 SPEC-003）

完整步骤见 SPEC-003 §3。本节只列 v1.5 无改动的摘要：

1. 修改 `~/.claude/settings.json`：
   - `env.ANTHROPIC_BASE_URL = "http://127.0.0.1:11453"`
   - `hooks.PreToolUse` 追加 `sieve-hook check`（幂等）
2. 写入 `~/Library/LaunchAgents/com.sieve.daemon.plist`
3. `launchctl load` 启动 daemon

### 4.2 OpenClaw 配置注入

#### 4.2.1 目标

把 OpenClaw 所有 LLM provider 的 `base_url` 改为 `http://127.0.0.1:11453`，使得 OpenClaw 发往上游 LLM 的流量经过 Sieve 代理。

OpenClaw 走 OpenAI Chat Completions 协议，Sieve 在接收到这类请求时走 `openai.rs` 协议适配层（依赖 ADR-018）。

#### 4.2.2 配置文件路径（TBD-01 已解决）

> **Week 7 调研结论**：配置文件为 `~/.openclaw/openclaw.json`（JSON 格式，非 TOML）。
> provider 字段路径：`models.providers.<id>.baseUrl`（camelCase）。
> 依据：openclaw/openclaw GitHub docs/concepts/model-providers.md。
> Week 8 dogfood 时最终验证。

**探测路径**（按优先级顺序）：

```
1. 环境变量 OPENCLAW_CONFIG（若存在则直接用）
2. ~/.openclaw/openclaw.json（主路径，文档明确）
3. ~/Library/Application Support/openclaw/openclaw.json（macOS Application Support 备用）
4. ~/.openclaw/config.json（旧版兼容）
```

#### 4.2.3 注入步骤

```
1. 读取 config.toml，解析 TOML
2. 找到 provider 路由表（TBD-01：字段路径待确认）
3. 对每个 provider entry：
   a. 备份原 base_url（写入 setup.log）
   b. 设置 base_url = "http://127.0.0.1:11453"
4. 写回 config.toml（保留其他字段不变）
```

**幂等性**：若某 provider 的 base_url 已是 `http://127.0.0.1:11453`，跳过不重复写。

**不注入的内容**：
- 不修改 OpenClaw 的 hook / skill 注册配置（暂无 PreToolUse 等价物）
- 不修改 OpenClaw 的 channel 配置（WhatsApp / Slack 接入等）
- 不修改 OpenClaw 的认证信息（API key 等）

#### 4.2.4 X-Sieve-Source-Channel header 要求

OpenClaw 须在向 Sieve 的请求中携带 `X-Sieve-Source-Channel` header（标明消息来源 channel，如 `whatsapp` / `slack` / `telegram`），供 IN-GEN-06 规则使用。

当前状态（TBD-05 已解决）：
- OpenClaw **支持** `models.providers.<id>.headers` 字段注入自定义 HTTP header（依据：model-providers.md）
- setup 时静态写入 `X-Sieve-Source-Channel: "openclaw"` 到每个 provider 的 headers
- **限制**：值为静态字符串，无法动态反映 WhatsApp/Slack 具体子 channel
- IN-GEN-06 获得来源类型（openclaw），channel 粒度信号留 Phase 1 后期给 OpenClaw 提 PR
- Week 8 dogfood 时确认 headers 字段是否随 HTTP 请求转发到 Sieve

#### 4.2.5 降级说明

OpenClaw 没有 PreToolUse hook 等价物。Hook 类规则（IN-CR-02 ~ IN-CR-04、IN-GEN-01 ~ IN-GEN-03）在 OpenClaw 上**降级为 GUI hold**：每次匹配到这些规则时弹 GUI 弹窗确认，而非终端 y/N 提示。Critical 类规则（IN-CR-01、IN-CR-05、IN-CR-06、IN-GEN-06）始终 GUI hold，无论 agent 类型。

#### 4.2.6 Header routing 与 Port routing 分工（ADR-026）

v2.x 起，sieve 同时支持两种 multi-provider 路由机制。两者并存、不冲突，适用场景不同：

| 场景 | 路由方式 | 实现位置 | 适用客户端 |
|------|---------|---------|-----------|
| 哑 client（Claude Code / Codex CLI / Cursor 等只认 single `ANTHROPIC_BASE_URL` 的 agent） | **Port routing**（ADR-026，一等公民） | listener 维度，每个 port 绑定独立 forwarder | 客户端无需注入任何 header，靠不同 port 区分上游 |
| Smart router agent（OpenClaw / Hermes 这种自己当 LLM 网关的） | **Header routing**（`X-Sieve-Provider`，本节原有方案）| 应用层，daemon 查 `~/.sieve/upstream-routes.json` 选 forwarder | 客户端在请求里注入 `X-Sieve-Provider: <id>`，daemon 在同一 port 内按 header 选上游 |
| 同一 client 同会话切多家 | sieve 不解决 | — | 用 LiteLLM / OpenRouter 当 sieve 上游 |

**关键规则——两种 routing 并存时的优先级与安全约束**：

- 任何 listener 上的请求都可以携带 `X-Sieve-Provider` header → header routing **优先于** listener 默认 forwarder
- listener 协议错位 fail-closed（ADR-026 §决策 4）**不能被 `X-Sieve-Provider` header 绕过**：header routing 只能在当前 listener 的 protocol 约束范围内切换上游，无法改变 protocol 校验逻辑（fail-closed 一致性不变）

参见 [ADR-026 §决策 6](../design/ADR-026-port-based-listener-routing.md)、[ADR-019](../design/ADR-019-x-sieve-origin-header.md)。

### 4.3 Hermes 配置注入

#### 4.3.1 目标

把 Hermes 每个已配置 provider 的 `base_url` 改为 `http://127.0.0.1:11453`，覆盖所有 Hermes 发出的 LLM 流量。

Hermes 嵌套启动 Claude Code 子进程时，子进程的 `ANTHROPIC_BASE_URL` 已由 Claude Code 自身的 settings.json 配置（见 §4.1），流量自然经过 Sieve，无需额外注入。

#### 4.3.2 配置文件路径（TBD-02 已解决）

> **Week 7 调研结论**：配置文件为 `~/.hermes/config.yaml`（**YAML 格式，非 TOML**）。
> 备用 `~/.hermes/.env` 仅存 API key，不包含 base_url，不支持 setup 写入。
> 依据：hermes-agent.nousresearch.com/docs/user-guide/configuration + cli-config.yaml.example。
> Week 8 dogfood 时最终验证。

**探测路径**（按优先级顺序）：

```
1. 环境变量 HERMES_CONFIG（若存在则直接用）
2. ~/.hermes/config.yaml（主路径，YAML 格式）
3. ~/.hermes/config.toml（旧版兼容，TOML 格式，若存在 serde_yaml 仍可读）
4. ~/.hermes/.env（仅用于检测安装，apply 时 bail 并给出友好提示）
```

若仅找到 `.env`，setup 输出提示并退出：
```
Hermes 仅找到 .env 文件，不支持 base_url 注入。
请先运行 hermes config edit 创建 config.yaml，
或手动将 model.base_url 设为 http://127.0.0.1:11453。
```

#### 4.3.3 注入步骤

```
1. 读取 ~/.hermes/config.yaml（YAML 格式；TBD-02 已解决）
2. 直接解析文件，不依赖 hermes config providers list（TBD-04 已解决：该命令不存在）
3. 修改顶层 model.base_url = "http://127.0.0.1:11453"
4. 修改 delegation.base_url = "http://127.0.0.1:11453"（TBD-06 降级，若 delegation 字段存在）
5. 写回 config.yaml
```

**不注入的内容**：
- 不修改 Hermes 的 orchestration 规则（由 Hermes 自主决定 delegate 给哪个 agent）
- 不修改 Hermes 的 API key 等认证信息
- 不修改 Hermes 的 sub-agent 调用命令（`claude` / `codex` 等）

#### 4.3.4 X-Sieve-Origin header 注入（sub-agent 嵌套）

当 Hermes 的 provider 配置 base_url 已指向 Sieve，Hermes 自身的 LLM 调用会带上默认 header。

Hermes delegate 给 Claude Code 子进程时，嵌套关系须通过以下方式传递给 Sieve：

**TBD-06 已解决（降级）**：

调研结论：Hermes delegation 子进程**不**自动继承父进程环境变量（文档明确 delegation 使用 config.yaml 中的 delegation section 配置，不透传父进程 env）。`ANTHROPIC_DEFAULT_HEADERS` 注入方案**不可行**。

**降级实现**：

```yaml
# setup 时写入 ~/.hermes/config.yaml
delegation:
  base_url: "http://127.0.0.1:11453"  # 子进程流量经过 Sieve
```

- Hermes 委托 Claude Code 子进程时，`delegation.base_url` 指向 Sieve，子进程 LLM 请求经过 Sieve
- X-Sieve-Origin header 在 Phase 1 后期由 Sieve daemon 端根据请求特征推断调用链
- PRD §6.7 场景 F（完整调用链显示）推迟到 Phase 1 后期实现

**后续**：Phase 1 后期可给 Hermes 提 PR 支持 delegation.env_vars 字段，不阻塞 GA。

#### 4.3.5 降级说明

同 §4.2.5，OpenClaw / Hermes 均无 PreToolUse hook 等价物，Hook 类规则降级为 GUI hold。

---

## 5. setup.log 多 agent 扩展

### 5.1 新增字段

每条 log entry 在 SPEC-003 §3.8 的字段基础上，增加：

```jsonl
{"ts":"...","event":"setup_start","version":"0.1.0","agents":["claude","openclaw"]}
{"ts":"...","event":"backup","agent":"claude","path":"~/.claude/settings.json","backup":"~/.sieve/backups/.../claude_settings.json"}
{"ts":"...","event":"modified","agent":"claude","path":"~/.claude/settings.json","change_type":"json_merge","fields_changed":["env.ANTHROPIC_BASE_URL","hooks.PreToolUse"]}
{"ts":"...","event":"backup","agent":"openclaw","path":"~/.openclaw/config.toml","backup":"~/.sieve/backups/.../openclaw_config.toml"}
{"ts":"...","event":"modified","agent":"openclaw","path":"~/.openclaw/config.toml","change_type":"toml_merge","fields_changed":["providers.anthropic.base_url","providers.openai.base_url"]}
{"ts":"...","event":"created","agent":"claude","path":"~/Library/LaunchAgents/com.sieve.daemon.plist","change_type":"new_file"}
{"ts":"...","event":"launchctl_load","agent":"claude","plist":"~/Library/LaunchAgents/com.sieve.daemon.plist","result":"ok"}
{"ts":"...","event":"setup_done","version":"0.1.0","agents":["claude","openclaw"]}
```

**新增字段说明**：
- `agent`：每条 entry 标记归属 agent（`claude` / `openclaw` / `hermes`）。daemon 相关 entry（plist / launchctl）归属 `claude`，因为 daemon 只装一次
- `agents`：`setup_start` 和 `setup_done` 记录本次 setup 覆盖的 agent 列表
- `fields_changed`：记录实际改动的字段路径（用于精确回滚）

### 5.2 uninstall 的 agent 过滤

`sieve uninstall --agent <name>` 时，反向遍历 setup.log 时只处理 `agent == <name>` 的 entry，跳过其他 agent 的 entry。

`sieve uninstall --all` 时，处理所有 entry（沿用 SPEC-003 §5.2 行为）。

**边界情况**：daemon plist 是共享资源（三家 agent 共用一个 daemon）。仅当 `uninstall --all` 或最后一家 agent 被 uninstall 时，才 unload + 删除 plist。判断方式：检查 setup.log 中是否还有其他 agent 的有效 `modified` entry 未被 uninstall。

---

## 6. sieve doctor 多 agent 检查

### 6.1 Claude Code（沿用 SPEC-003 §4）

| 检查项 | 通过条件 |
|--------|---------|
| ANTHROPIC_BASE_URL | settings.json `env.ANTHROPIC_BASE_URL == "http://127.0.0.1:11453"` |
| PreToolUse hook | `hooks.PreToolUse` 数组含 `sieve-hook check` |
| daemon 监听 | TCP connect `127.0.0.1:11453` 成功（2s 超时） |
| launchd 状态 | `launchctl list com.sieve.daemon` exit 0 |
| Canary | 发送含 `sk-ant-test-XXX` 的请求，响应 body 含 `[REDACTED]` |

### 6.2 OpenClaw

| 检查项 | 通过条件 |
|--------|---------|
| daemon 监听 | 同 Claude Code —— TCP connect `127.0.0.1:11453` 成功 |
| provider 配置正确 | 解析 `~/.openclaw/config.toml`，所有 provider 的 base_url 均为 `http://127.0.0.1:11453` |
| Canary（OpenAI 协议） | 发送 OpenAI Chat Completions 格式的含 `sk-test-XXX` 请求，响应 body 含 `[REDACTED]` |
| X-Sieve-Source-Channel 注入 | setup 写入 `models.providers.<id>.headers.X-Sieve-Source-Channel = "openclaw"`；doctor 输出提醒 Week 8 dogfood 验证实际转发（TBD-05 已解决，Week 8 最终确认） |

### 6.3 Hermes

| 检查项 | 通过条件 |
|--------|---------|
| hermes CLI 可用 | `hermes --version` exit 0 |
| provider 配置正确 | 解析 Hermes 配置文件（TBD-02），所有 provider base_url 均为 `http://127.0.0.1:11453` |
| Canary | 同 OpenClaw Canary |
| X-Sieve-Origin header 注入 | TBD-06 降级：ANTHROPIC_DEFAULT_HEADERS 不可行；doctor 验证 delegation.base_url = 127.0.0.1:11453；子进程 origin chain 推断在 Phase 1 后期实现 |

### 6.4 doctor 输出格式

```
sieve doctor --agent openclaw

[✓] Sieve daemon 在监听
    :11453 端口有 TCP LISTEN 状态

[✓] OpenClaw provider 配置正确
    ~/.openclaw/config.toml 中 2 个 provider 均指向 127.0.0.1:11453

[✓] Canary 检测通过（OpenAI 协议）
    发送测试 key → 正确返回 [REDACTED]

[✗] X-Sieve-Source-Channel 透传未确认
    需实测 OpenClaw 是否支持自定义 header 注入（见 TBD-05）
```

各 agent doctor 独立退出码，`--agent` 指定后只检查该 agent，exit 0 / 1。

---

## 7. 错误处理与回滚

### 7.1 单 agent setup 失败

某 agent 的 `apply_changes()` 失败时：

```
1. 输出 "openclaw 配置注入失败（步骤 N），正在回滚..."
2. 按 setup.log 中本次写入的 openclaw entry 逆序回滚
3. 已成功的其他 agent 改动**不回滚**（有各自的 backup，可单独 uninstall）
4. 输出回滚结果 + 已成功 agent 列表
5. exit 1
```

### 7.2 --all-detected 部分失败

```
sieve setup --all-detected
→ claude: ✓ 成功
→ openclaw: ✗ 失败（config 文件解析错误）
→ hermes: ✓ 成功

openclaw 配置注入失败，已回滚 openclaw 的改动。
claude 和 hermes 配置已保留。
如需重试 openclaw：sieve setup --agent openclaw
```

### 7.3 检测不到 agent

`sieve setup --all-detected` 时，某 agent 未检测到：跳过 + 友好提示（§3.2 / §3.3 的提示文案），继续处理下一个 agent。

### 7.4 config 文件路径 TBD 时的降级

若 OpenClaw / Hermes 的配置文件在所有已知路径均未找到，setup 输出：

```
未找到 <agent> 配置文件（已尝试以下路径）：
  - ~/.openclaw/config.toml
  - ~/Library/Application Support/openclaw/config.toml

请手动配置，或等待 Week 6 实测后更新 sieve。
```
并 exit 1（仅该 agent 失败）。

---

## 8. 平台约束

**仅 macOS**，与 SPEC-003 §2 相同。

非 macOS 时输出：
```
sieve setup is macOS only in Phase 1. Linux and Windows support is planned for Phase 2.
```
exit 2。

实现方式与 SPEC-003 §2 相同（`check_platform()` 运行时检查，不用 compile-time cfg）。

---

## 9. 不做的事

| 排除项 | 说明 |
|--------|------|
| 修改 OpenClaw / Hermes 的二进制 | 只通过标准配置文件接口注入 |
| 持有 OpenClaw / Hermes 源码副本 | 不分发第三方 binary |
| 替用户决定 Hermes 走哪个 provider | provider 选择是 Hermes 的职责，Sieve 只改 base_url |
| 实现 OpenClaw pre_skill_invoke hook 等价物 | Phase 1 后期给 OpenClaw 提 PR，不阻塞 GA |
| 实现 Hermes hook 等价物 | 同上，Phase 1 后期给 Nous Research 提 PR |
| 多 provider 并行注入 | setup 顺序处理（claude → openclaw → hermes），不并行，防止共享 plist 冲突 |
| `sieve setup` 无参数时的默认行为改变 | 现有行为：无参数时提示使用 `--agent <name>` 或 `--all-detected`；不改变为隐式装所有 |

---

## 10. 未决事项（TBD）

> **Week 7 调研更新（2026-04-27）**：TBD-01 ~ TBD-06 已通过公开文档调研填上实现，不再阻塞开发。
> Week 8 dogfood 时做最终验证，若有偏差立即更新 SPEC。

| 编号 | 问题 | Week 7 调研结论 | fallback / 降级 | Week 8 dogfood 验证 |
|------|------|----------------|----------------|---------------------|
| ~~**TBD-01**~~ | ~~OpenClaw config 文件路径和 provider 字段结构~~ | **已解决**：`~/.openclaw/openclaw.json`（JSON 格式），字段路径 `models.providers.<id>.baseUrl`（camelCase）。依据：openclaw/openclaw docs/concepts/model-providers.md | 若主路径未找到，依次尝试 Application Support + config.json | ✅ Week 8 验证实际路径 |
| ~~**TBD-02**~~ | ~~Hermes provider 配置位置（TOML / .env / keychain）~~ | **已解决**：`~/.hermes/config.yaml`（YAML 格式，非 TOML），备用 `~/.hermes/.env`（仅存 API key，不含 base_url）。依据：hermes-agent.nousresearch.com/docs/user-guide/configuration | .env 文件时给出友好提示 + bail！让用户手动创建 config.yaml | ✅ Week 8 验证文件格式 |
| ~~**TBD-03**~~ | ~~OpenClaw daemon 状态检查命令~~ | **已解决**：`openclaw doctor` 命令存在（AGENTS.md 明确记录为"Rebrand/migration/config warnings"诊断命令）。`openclaw status` 存在但面向 chat session 内部使用 | 二进制未找到时跳过检查，输出 None | ✅ Week 8 确认 doctor exit code 语义 |
| ~~**TBD-04**~~ | ~~Hermes provider 列表查询命令~~ | **已解决（降级）**：`hermes config providers list` 命令**不存在**。实际命令为 `hermes config`（查看）和 `hermes config check`（验证）。依据：hermes-agent.nousresearch.com/docs/user-guide/configuration | 用 `hermes config check` 替代；未找到时跳过，daemon_running = None | ✅ Week 8 确认 config check exit code |
| ~~**TBD-05**~~ | ~~OpenClaw 是否支持注入自定义 HTTP header~~ | **已解决（部分）**：OpenClaw 支持 `models.providers.<id>.headers` 字段注入自定义 HTTP header。`X-Sieve-Source-Channel: "openclaw"` 静态写入配置。**限制**：值为静态字符串，无法动态反映 WhatsApp/Slack 具体 channel | IN-GEN-06 获得 channel 来源信号（openclaw），无法区分子 channel；可接受 | ✅ Week 8 确认 OpenClaw 是否在转发请求时保留 headers 字段 |
| ~~**TBD-06**~~ | ~~Hermes 是否透传 `ANTHROPIC_DEFAULT_HEADERS`~~ | **已解决（降级）**：Hermes delegation 子进程**不**继承父进程环境变量。`ANTHROPIC_DEFAULT_HEADERS` 注入不可行。**降级方案**：`delegation.base_url` 指向 Sieve，子进程流量经过 Sieve。X-Sieve-Origin header 在 Phase 1 后期由 Sieve daemon 端推断 | PRD §6.7 场景 F（完整调用链显示）退化为 Phase 1 后期实现 | ✅ Week 8 确认 delegation.base_url 对所有子进程生效 |
| **TBD-07** | `sieve uninstall --purge` 是否适用于 multi-agent 场景（沿用 SPEC-003 TBD-1） | 倾向 Phase 1 不实现 | — | Phase 2 评估 |
