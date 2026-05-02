# SPEC-003: sieve setup 工具行为规格

> Version: v1.0 — 2026-04-28
> Status: Stable
> 关联 ADR：ADR-015（sieve setup 自动配置工具）
> 关联 PRD：v2.0 §6.6、§10.1 Week 5（v1.4 引入）

---

## 1. 目标

定义 `sieve setup` / `sieve doctor` / `sieve uninstall` 三个子命令的完整行为，包括：
- 系统状态探测逻辑
- 文件修改内容与路径
- 备份与回滚机制
- 错误恢复路径
- 平台限制

**前置条件**：用户已通过 `.dmg` 安装 Sieve，`sieve` 可执行文件在 `$PATH` 中。

---

## 2. 平台约束

**Phase 1 仅支持 macOS**。非 macOS 系统运行任意子命令时，立即输出：

```
sieve setup is macOS only in Phase 1. Linux and Windows support is planned for Phase 2.
```

并以 exit code 2 退出。

实现：`#[cfg(not(target_os = "macos"))]` 的 `fn check_platform()` 在子命令入口处调用（不用 compile-time cfg 排除，以便 CI 多平台编译通过，见 todo.md Q5 推荐 A）。

---

## 3. sieve setup

### 3.1 步骤概述

```
1. check_platform()
2. detect_ai_agents()          → 探测已安装的 AI 代理
3. build_change_plan()         → 计算将做的所有变更
4. print_dry_run_diff()        → 打印 diff 风格预览
5. prompt_user_confirm()       → 等待用户输入 y 确认
6. backup_files()              → 备份原文件
7. apply_changes()             → 执行所有变更
8. write_setup_log()           → 写 setup.log
9. print_success_summary()     → 打印完成摘要
```

任何步骤失败 → 调用 `rollback_changes()`（见 §3.8）。

### 3.2 步骤 2：detect_ai_agents

探测以下路径，判断用户安装了哪些 AI 代理：

| AI 代理 | settings.json 路径 | 检测条件 |
|--------|-------------------|---------|
| Claude Code | `~/.claude/settings.json` | 文件存在 |
| Cursor | `~/.cursor/settings.json` 或 `~/.config/Cursor/User/settings.json` | 文件存在 |

**Phase 1 只修改 Claude Code settings.json**。探测到 Cursor 时，输出提示：
```
检测到 Cursor，Cursor 支持计划在 Phase 2 加入，当前跳过。
```

若未探测到任何 AI 代理，输出：
```
未找到 Claude Code settings.json（~/.claude/settings.json）。
请先安装 Claude Code，然后重新运行 sieve setup。
```
并 exit 1。

### 3.3 步骤 3：build_change_plan

计算将做的变更，返回有序变更列表：

```
变更列表：
  1. 修改 ~/.claude/settings.json
     - 添加 env.ANTHROPIC_BASE_URL = "http://127.0.0.1:11453"
     - 添加 hooks.PreToolUse[0] = { matcher: ".*", hooks: [{ type: "command", command: "sieve-hook check" }] }
  2. 写入 ~/Library/LaunchAgents/com.sieve.daemon.plist
  3. 运行 launchctl load ~/Library/LaunchAgents/com.sieve.daemon.plist
```

### 3.4 步骤 4：print_dry_run_diff

以 diff 风格输出变更预览（绿色 + 号表示新增，红色 - 号表示删除）：

```
将对 ~/.claude/settings.json 做以下改动：
  ...（现有内容节选）
+ "env": {
+   "ANTHROPIC_BASE_URL": "http://127.0.0.1:11453"
+ },
+ "hooks": {
+   "PreToolUse": [
+     {
+       "matcher": ".*",
+       "hooks": [{ "type": "command", "command": "sieve-hook check" }]
+     }
+   ]
+ }

将新建 ~/Library/LaunchAgents/com.sieve.daemon.plist（内容见下）：
  ... （plist XML 节选）
```

### 3.5 步骤 5：prompt_user_confirm

```
以上变更将修改 2 个文件。继续？[y/N]
```

- 等待 stdin 输入，不设超时
- 用户输入 `y` / `Y` 继续
- 用户输入其他任何字符（包括 Enter）→ 中止，输出 `已取消，未做任何修改`，exit 0
- `--yes` / `-y` flag 跳过此步骤（用于脚本化安装）

### 3.6 步骤 6：backup_files

在 `~/.sieve/backups/<timestamp>/` 下备份将被修改的文件：

```
~/.sieve/backups/
└── 2026-04-28T03-14-15Z/
    └── claude_settings.json    # ~/.claude/settings.json 原文件的完整副本
```

- timestamp 格式：`YYYY-MM-DDTHH-MM-SSZ`（ISO 8601，`:` 替换为 `-`，兼容文件名）
- 若 `~/.claude/settings.json` 不存在（全新安装），记录备份路径为 `(none)`，回滚时删除新建文件

### 3.7 步骤 7：apply_changes

#### 3.7.1 修改 Claude Code settings.json

settings.json 是 JSONC（JSON with Comments）格式。处理策略：

1. 读取文件原始字节
2. 用宽容的 JSON 解析器（strip 注释后解析）得到 JSON 值
3. 在内存中添加 / 合并字段（不删除现有字段）
4. 序列化为标准 JSON（pretty-print，2 空格缩进）写回
5. **注意**：此操作会丢失原有注释——这是已知的 trade-off，在步骤 4 预览时告知用户

**冲突处理**：

| 字段 | 已存在时 |
|------|---------|
| `env.ANTHROPIC_BASE_URL` | 覆盖写（打印警告，显示旧值和新值）|
| `hooks.PreToolUse` | 在数组末尾追加（不删除现有 hook）；若已有 `sieve-hook check` 条目则跳过 |

**幂等性**：多次运行 `sieve setup` 不产生重复 hook 条目。

#### 3.7.2 写 launchd plist

写入 `~/Library/LaunchAgents/com.sieve.daemon.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.sieve.daemon</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/sieve</string>
    <string>start</string>
    <string>--config</string>
    <string>/Users/$USER/.sieve/sieve.toml</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/Users/$USER/.sieve/daemon.log</string>
  <key>StandardErrorPath</key>
  <string>/Users/$USER/.sieve/daemon.err</string>
</dict>
</plist>
```

注意：`$USER` 在写入时替换为实际用户名，不写字面量 `$USER`。

#### 3.7.3 launchctl load

```bash
launchctl load ~/Library/LaunchAgents/com.sieve.daemon.plist
```

若 plist 已加载（`launchctl list | grep com.sieve.daemon` 有输出），先 unload 再 load。

### 3.8 步骤 8：write_setup_log

写入 `~/.sieve/setup.log`，每次 `sieve setup` 追加（不覆盖）：

每次安装写一个 JSON Lines 块，每行一个 entry：

```jsonl
{"ts":"2026-04-28T03:14:15Z","event":"setup_start","version":"0.1.0"}
{"ts":"2026-04-28T03:14:15Z","event":"backup","path":"~/.claude/settings.json","backup":"~/.sieve/backups/2026-04-28T03-14-15Z/claude_settings.json"}
{"ts":"2026-04-28T03:14:16Z","event":"modified","path":"~/.claude/settings.json","change_type":"json_merge"}
{"ts":"2026-04-28T03:14:16Z","event":"created","path":"~/Library/LaunchAgents/com.sieve.daemon.plist","change_type":"new_file"}
{"ts":"2026-04-28T03:14:16Z","event":"launchctl_load","plist":"~/Library/LaunchAgents/com.sieve.daemon.plist","result":"ok"}
{"ts":"2026-04-28T03:14:16Z","event":"setup_done","version":"0.1.0"}
```

`uninstall` 通过反向遍历此文件执行回滚（见 §5）。

### 3.9 错误回滚（apply_changes 任意步骤失败）

```
1. 输出 "安装失败（步骤 N），正在回滚..."
2. 按 setup.log 已记录的 entry 反向遍历
3. 对 "backup" entry：从备份路径恢复原文件（或删除新建文件）
4. 对 "launchctl_load" entry：launchctl unload
5. 输出 "回滚完成，所有变更已还原"
6. exit 1
```

---

## 4. sieve doctor

### 4.1 检查项列表

按顺序执行，输出每项 ✓ / ✗：

```
[✓] ANTHROPIC_BASE_URL 已配置
    Claude Code settings.json 中 env.ANTHROPIC_BASE_URL = "http://127.0.0.1:11453"

[✓] PreToolUse hook 已注册
    sieve-hook check 已在 hooks.PreToolUse 列表中

[✓] Sieve daemon 在监听
    :11453 端口有 TCP LISTEN 状态

[✓] launchd 服务运行中
    com.sieve.daemon 状态：running (PID 12345)

[✓] Canary 检测通过
    发送已知触发词 → daemon 正确返回脱敏后的内容
```

### 4.2 各检查项实现细节

| 检查项 | 通过条件 | 失败时提示 |
|-------|---------|----------|
| ANTHROPIC_BASE_URL | `settings.json` 解析后 `env.ANTHROPIC_BASE_URL == "http://127.0.0.1:11453"` | `请运行 sieve setup 重新配置` |
| PreToolUse hook | `hooks.PreToolUse` 数组中存在 `command == "sieve-hook check"` 的条目 | `请运行 sieve setup 重新配置` |
| daemon 监听 | TCP connect `127.0.0.1:11453` 成功（2 秒超时）| `daemon 未运行，尝试：launchctl start com.sieve.daemon` |
| launchd 状态 | `launchctl list com.sieve.daemon` 退出码 = 0 | `尝试：launchctl load ~/Library/LaunchAgents/com.sieve.daemon.plist` |
| Canary 检测 | 发送含 `sk-ant-` 前缀的测试 API key 到 `http://127.0.0.1:11453/v1/messages`（模拟出站）；响应 body 中该 key 被替换为 `[REDACTED]` | `检测失败，规则引擎可能未正常工作` |

**Canary 检测使用的测试数据**：
```
test key: sk-ant-test-XXXXXXXXXXXXXXXXXXXXXXXXXXXX（固定字符串，不是真 key）
期望响应：包含 [REDACTED] 替换后的内容
```

### 4.3 doctor 退出码

- 所有项通过：exit 0
- 任意项失败：exit 1
- `--json` flag：输出 JSON 格式结果（供脚本解析）

---

## 5. sieve uninstall

### 5.1 步骤概述

```
1. check_platform()
2. read_setup_log()              → 解析 ~/.sieve/setup.log，找最近一次 setup_done
3. build_rollback_plan()         → 计算将回滚的变更
4. print_dry_run_diff()          → 打印 diff 风格预览（默认 dry-run）
5. prompt_user_confirm()         → 等待用户输入 y 确认
6. execute_rollback()            → 执行回滚
7. append_uninstall_log()        → 记录卸载到 setup.log
8. print_success_summary()       → 打印提示
```

### 5.2 回滚操作

按 setup.log 中最近一次完整安装（`setup_start` → `setup_done`）的 entry 逆序执行：

| setup.log event | uninstall 操作 |
|----------------|---------------|
| `launchctl_load` | `launchctl unload` + 删除 plist 文件 |
| `created` | 删除该文件（new_file 类型）|
| `modified` | 从 backup 路径恢复原文件 |
| `backup` (none) | 删除安装后新建的文件 |

### 5.3 uninstall 完成后的提示

```
Sieve 已卸载。

以下目录未自动删除（含审计日志，请手动确认后删除）：
  ~/.sieve/

如需彻底清除：
  rm -rf ~/.sieve
```

**设计决策**：默认不删 `~/.sieve/`，避免误删审计日志。用户明确 `sieve uninstall --purge` 时才删除（TBD-1）。

### 5.4 setup.log 不存在时的处理

若 `~/.sieve/setup.log` 不存在或无法解析，输出：

```
未找到安装记录（~/.sieve/setup.log）。
尝试手动清理：
  - 从 ~/.claude/settings.json 移除 env.ANTHROPIC_BASE_URL 和 sieve-hook hook
  - launchctl unload ~/Library/LaunchAgents/com.sieve.daemon.plist
  - rm ~/Library/LaunchAgents/com.sieve.daemon.plist
```
并 exit 1。

---

## 6. 不做的事

以下操作明确排除在 Phase 1 范围之外：

| 排除项 | 原因 |
|--------|------|
| 修改 `~/.zshrc` / `~/.bashrc` | PATH 由 `.dmg` 安装包处理，setup 不做 shell 配置 |
| 安装本地 CA 证书 | Sieve 不做 TLS 拦截，无需 CA（PRD §9 第 12 条：不装本地 CA）|
| 修改系统代理设置（network proxy）| 使用 `ANTHROPIC_BASE_URL` 环境变量定向，不劫持系统流量 |
| 需要 `sudo` 权限的任何操作 | 所有文件写在用户目录下（`~/.claude/`、`~/Library/LaunchAgents/`、`~/.sieve/`），均无需 root |
| 修改 Cursor / Windsurf / Zed 配置 | Phase 1 仅 Claude Code；其他 agent 支持留给 Phase 2 |

---

## 7. 补充说明：Canary 检测实现变更（R4-#7 修复，2026-04-27）

§4.2 "Canary 检测"原定向 `127.0.0.1:11453` 发 HTTP 请求，验证响应不含原始 token。
该方案有致命缺陷：daemon 透传时拿到 401/502，响应同样不含 canary token，导致误判通过。

**当前实现改为本地引擎 scan 方案**（已落地）：

- 直接调用 `sieve_rules::engine::VectorscanEngine::compile(outbound_rules).scan(canary_token)`
- canary token 精确匹配 OUT-01 pattern（`sk-ant-api03-[a-zA-Z0-9_\-]{93}AA`）
- 不发任何网络请求，不依赖 daemon 是否在线
- 输出标注：「canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）」
- 规则文件路径优先读 `~/.sieve/rules/outbound.toml`，可通过 `SIEVE_RULES_PATH` env var 覆盖

**限制说明**：本地 scan 验证了规则编译 + pattern 命中，但不验证 daemon 是否真的拦截了转发请求。
端到端验证（daemon 实际改写 body）需要手动测试或后续引入 fake upstream。

---

## 8. 未决事项（TBD）

| 编号 | 问题 | 选项 |
|------|------|------|
| TBD-1 | `sieve uninstall --purge` 是否实现（删除 ~/.sieve/）| A. Phase 1 实现，二次确认后删；B. Phase 1 不实现，输出 rm 命令让用户手动执行；当前推荐 B（更安全）|
| TBD-2 | settings.json 注释丢失的处理方式 | A. 接受（当前方案）；B. 用 jsonc-parser crate 保留注释位置；Phase 1 选 A，Phase 2 评估 B |
| TBD-3 | setup 是否支持多用户（system-wide 安装）| Phase 1 仅支持当前用户，system-wide 推 Phase 2 |
| TBD-4 | `sieve setup --config <path>` 指定自定义 sieve.toml 路径 | Phase 1 硬编码 `~/.sieve/sieve.toml`；Phase 2 加参数 |
