# SPEC-001: sieve-hook 文件 IPC 协议

> Version: v1.0 — 2026-04-28
> 关联 ADR：ADR-013（IPC 协议）、ADR-014（双层防御）
> 关联 PRD：v1.4 §6.5、§6.7

---

## 1. 目标

定义 Sieve 主代理（`sieve-core` pipeline）与 `sieve-hook`（Claude Code PreToolUse hook）之间的文件 IPC 协议。该协议仅用于 Hook 类规则（IN-CR-02~04、IN-GEN-01~03）；GUI 弹窗类（IN-CR-01、IN-CR-05、IN-GEN-04、OUT-06~10）使用 Unix socket JSON-RPC（见 §6.5 架构图）。

**设计约束**：
- `sieve-hook` 是被 Claude Code 每次 PreToolUse 都 fork 的进程，启动时延必须 < 50ms
- `sieve-hook` 禁止依赖 `sieve-core` / `sieve-rules` / vectorscan；只允许 `serde_json` + `fd-lock`
- 文件 IPC 不依赖 daemon 在线——主代理写文件，hook 读文件，两者无持久连接

---

## 2. 目录结构

```
~/.sieve/
├── pending/
│   └── <request_id>.json       # 主代理写，sieve-hook 读
├── decisions/
│   └── <request_id>.json       # sieve-hook 写，主代理读
└── locks/
    └── <request_id>.lock       # fd-lock 文件锁（advisory lock）
```

- **`pending/`**：主代理检测到 Hook 类规则命中后写入。写入完成后主代理等待对应 decisions 文件出现（inotify / kqueue / 轮询，最长 `timeout_seconds`）。
- **`decisions/`**：`sieve-hook` 用户决策后写入，主代理读取并继续处理。
- **`locks/`**：fd-lock 防止主代理和 hook 同时读写同一请求文件。写入方在打开文件前先拿写锁，读取方拿读锁。

### 2.1 request_id 格式

使用 **UUIDv7**（128-bit，前 48-bit 为毫秒时间戳，按时间自然排序）。

```
格式：xxxxxxxx-xxxx-7xxx-yxxx-xxxxxxxxxxxx
示例：018f2c3a-1b2c-7d3e-a4f5-6c7d8e9f0a1b
```

选择 UUIDv7 的理由：
1. 按时间排序——便于 `pending/` 目录按最新文件查找
2. 无需中心化计数器
3. 标准库 `uuid` crate v1.7+ 支持

---

## 3. pending JSON Schema

主代理写入 `~/.sieve/pending/<request_id>.json`。

```jsonc
{
  "request_id": "018f2c3a-1b2c-7d3e-a4f5-6c7d8e9f0a1b",  // UUIDv7，string
  "rule_id": "IN-CR-02",                                   // string，检测项 ID
  "severity": "critical",                                   // "critical" | "high" | "medium"
  "disposition": "hook_terminal",                           // 固定为 "hook_terminal"（此协议只处理 Hook 类）
  "title": "危险工具调用：bash rm -rf",                     // string，≤ 80 字符
  "one_line_summary": "模型请求执行 `rm -rf /tmp/data`",   // string，≤ 120 字符，展示给用户
  "tool_name": "bash",                                      // string，Claude Code tool_use name 字段原文
  "tool_args_preview": "rm -rf /tmp/data",                  // string，≤ 200 字符，参数摘要（敏感内容已脱敏）
  "created_at_rfc3339": "2026-04-28T03:14:15Z",            // RFC 3339，主代理写入时间
  "timeout_seconds": 30,                                    // uint32，规则超时（见 PRD §5.4.2）
  "default_on_timeout": "deny"                              // "deny" | "allow"（Critical 永远是 "deny"）
}
```

**字段约束**：

| 字段 | 类型 | 约束 |
|------|------|------|
| `request_id` | string | 必须是合法 UUIDv7 |
| `rule_id` | string | 必须在规则 manifest 中存在 |
| `severity` | string enum | `critical` / `high` / `medium` |
| `disposition` | string | 此协议中固定为 `hook_terminal` |
| `title` | string | 1–80 字符 |
| `one_line_summary` | string | 1–120 字符 |
| `tool_name` | string | 非空 |
| `tool_args_preview` | string | 0–200 字符；主代理负责截断并脱敏 |
| `created_at_rfc3339` | string | 合法 RFC 3339 时间 |
| `timeout_seconds` | uint32 | 1–3600 |
| `default_on_timeout` | string enum | `deny` / `allow` |

**不变量**：severity=critical 时 `default_on_timeout` 必须为 `deny`。主代理在写入前检查，不合规则直接 panic（fail-closed）。

---

## 4. decisions JSON Schema

`sieve-hook` 写入 `~/.sieve/decisions/<request_id>.json`。

```jsonc
{
  "request_id": "018f2c3a-1b2c-7d3e-a4f5-6c7d8e9f0a1b",  // 与 pending 对应
  "decision": "deny",                                       // "allow" | "deny"
  "decided_at_rfc3339": "2026-04-28T03:14:28Z",           // RFC 3339，sieve-hook 写入时间
  "by": "user",                                            // "user" | "timeout"
  "remember": false                                         // bool，是否加入 .sieveignore 白名单
}
```

**字段约束**：

| 字段 | 类型 | 约束 |
|------|------|------|
| `request_id` | string | 必须与 pending 文件名一致 |
| `decision` | string enum | `allow` / `deny` |
| `decided_at_rfc3339` | string | 合法 RFC 3339 |
| `by` | string enum | `user` / `timeout` |
| `remember` | bool | Critical 规则强制为 `false`，hook 端写入前检查 |

**硬约束（Critical 不可绕过）**：
- 当 pending 文件中 `default_on_timeout = deny` 且 `by = timeout` 时，decision 必须写 `deny`
- Critical 规则 `remember` 永远写 `false`，即使用户在 TTY 中选择记住也忽略

---

## 5. sieve-hook 调用约定

### 5.1 Claude Code 注册方式

`sieve setup` 在 `~/.claude/settings.json` 写入：

```jsonc
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "sieve-hook check"
          }
        ]
      }
    ]
  }
}
```

Claude Code 在每次执行 tool 之前 fork `sieve-hook check`，并通过环境变量传入上下文。

### 5.2 环境变量

| 变量 | 来源 | 含义 |
|------|------|------|
| `SIEVE_REQUEST_ID` | Sieve 主代理（未来版本注入）| 当前请求的 UUIDv7 |
| `CLAUDE_TOOL_NAME` | Claude Code（标准 PreToolUse 环境）| 即将执行的 tool 名称 |
| `CLAUDE_TOOL_INPUT` | Claude Code | tool 参数 JSON（部分实现可能不暴露） |

**当 `SIEVE_REQUEST_ID` 不可用时的降级**：`sieve-hook` 扫描 `~/.sieve/pending/` 目录，取最新（按 UUIDv7 时间前缀排序）且未决策的文件，匹配 `CLAUDE_TOOL_NAME`。若多个文件均匹配，取最新一条。若无匹配，执行 §6 "pending 文件不存在"的路径。

### 5.3 TTY 弹窗行为

```
┌─────────────────────────────────────────────────┐
│ ⚠  Sieve 拦截：危险工具调用                     │
│ 规则：IN-CR-02 · 危险 bash                      │
│ 操作：rm -rf /tmp/data                          │
│                                                   │
│ 允许此次执行？[y/N]  30s 后自动拒绝              │
└─────────────────────────────────────────────────┘
```

- 默认答案：`N`（大写表示默认）
- 倒计时由 hook 自己维护（`Instant::now()` 轮询），不依赖代理时钟
- 用户按 `y` + Enter → `decision=allow`
- 用户按 `n` / Enter / Ctrl+C / 超时 → `decision=deny`
- 倒计时格式：每秒刷新同一行 `30s 后自动拒绝` → `29s 后自动拒绝` → …

### 5.4 exit code 语义

| exit code | 含义 | Claude Code 行为 |
|-----------|------|-----------------|
| `0` | allow（放行）| 继续执行 tool |
| `1` | deny（拒绝）| 取消 tool 执行，向用户报告被拦截 |

`sieve-hook` 在写完 decisions 文件后，按 decisions.decision 决定退出码（`allow → 0`，`deny → 1`）。

---

## 6. 错误处理

| 场景 | sieve-hook 行为 | 依据 |
|------|----------------|------|
| pending 文件不存在 | exit 0（放行）| 主代理判定为通过，hook 无需拦截 |
| pending 文件存在但 stale（`created_at` > 当前时间 10 分钟）| exit 1（fail-closed）| 过期请求不可信 |
| pending 文件 JSON 解析失败 | exit 1（fail-closed）| 数据损坏，不放行 |
| 文件锁拿不到 | 等待 100ms 重试，3 次失败后 exit 1 | 并发保护 |
| decisions 文件写入失败 | 记录 stderr，exit 1 | 写决策失败视为拒绝 |
| TTY 不可用（非交互模式）| 按 `default_on_timeout` 决定 | headless 环境降级 |

**stale 判定**：`now() - created_at > 600s`。容忍系统时钟偏差 ±5s，实际阈值为 605s。

---

## 7. 启动时延预算

目标：**端到端（fork → TTY 出现）< 50ms**。

约束与基准：
- 禁止在启动路径依赖 `sieve-core` / `sieve-rules` / vectorscan / rusqlite
- 依赖仅 `serde_json` + `fd-lock`（两个无 C 依赖的纯 Rust crate）
- 文件读取：`pending/` 目录扫描限制 ≤ 100 个文件，超过则只取最新 100 个
- benchmark 指令：`cargo bench -p sieve-hook --bench startup_latency`（Week 5 落地时补）

---

## 8. Critical 不可绕过的硬约束

以下约束在 `sieve-hook` 代码中以 `assert!` / 编译期常量强制：

1. `default_on_timeout = deny` 的请求，超时路径写 `decision = deny`，无任何例外分支
2. `remember = true` 写入 decisions 文件前检查：若 pending 中 severity = critical，强制改写为 `remember = false`
3. 超时检测不依赖 sleep——使用 `Instant::now()` + 非阻塞 stdin 读取（`crossterm` 或 `termios`）
4. exit code 与 decision 文件写入必须原子对齐：decisions 文件写成功后才 exit；若写失败则 exit 1

---

## 9. 文件生命周期

- **pending 文件**：hook 写完 `decisions/<id>.json` 后**立即尝试删除** `pending/<id>.json`（`write_decision` 内部执行）；删除失败只打 warning，不影响决策流程。主代理超时后也负责清理残留的 pending 文件。
  - 主动清理原因：避免 sieve-hook 下次 PreToolUse 启发式扫目录时重复处理已决策的 pending，导致反复弹窗。
- **decisions 文件**：主代理读取后负责删除
- **lock 文件**：fd-lock 使用 O_CREAT 打开，文件描述符关闭时锁自动释放，文件本身可以保留（下次重新用）
- **清理策略**：`sieve doctor` 检查并清理 stale（> 1 小时）的 pending / decisions / lock 文件
- **scan 去重**：`scan_pending_dir()` 在判断 fresh/stale 前先检查 `decisions/<id>.json` 是否存在；存在则跳过该 pending（既不进 fresh 也不进 stale）。双重保障：即使 pending 文件未被及时删除，也不会重复弹窗。

---

## 10. 未决事项（TBD）

| 编号 | 问题 | 选项 |
|------|------|------|
| TBD-1 | `SIEVE_REQUEST_ID` 环境变量注入机制 | A. 主代理通过 Unix socket 给 Claude Code 发信号注入；B. 主代理写 `~/.sieve/current_request_id`（单文件，最新请求覆盖写）；当前 Phase 1 暂用 B |
| TBD-2 | sieve-hook 是否加密 pending 文件内容（防本地侧信道）| A. 不加密（Phase 1，本地文件权限 0600 已足够）；B. ChaCha20 加密（Phase 2）；当前选 A |
| TBD-3 | 多个并发 Hook 类请求的合并弹窗 | 当前：每个请求独立弹一个 TTY 提示；Phase 2 考虑队列合并 |
