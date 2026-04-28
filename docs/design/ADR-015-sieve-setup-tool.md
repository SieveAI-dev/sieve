# ADR-015: sieve setup / doctor / uninstall 自动配置三件套

## 状态

**已接受**

> 决策日期：2026-04-28
> 范围：Phase 1 macOS 安装与卸载链路；Windows/Linux 不在本 ADR 范围
> 关联 PRD：[v1.4 §6.6、§10.1 Week 5](../prd/sieve-prd-v1.5.md)

---

## 背景

### 手动配置失败率高

v1.3 的接入方式要求用户：

1. 设置环境变量 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`（shell rc 文件 or IDE 配置）
2. 手动编辑 Claude Code `settings.json` 注册 `hooks.preToolUse`
3. 手动创建 launchd plist 并 `launchctl load`
4. 验证代理是否正常工作

每一步都有失败面：shell rc 文件改错导致登录失败、settings.json 语法错误导致 Claude Code 崩溃、launchd plist 权限问题导致守护进程无法启动。实测中，即使是开发者用户在无引导情况下的安装成功率也偏低。

### v1.4 动机

v1.4 §6.6 新增安装程序要求：提供 `sieve setup` 命令，在 .dmg 安装包的"安装后脚本"或用户首次运行时自动完成上述所有步骤，并提供 `sieve doctor` 验证链路、`sieve uninstall` 精确回滚。

### 为什么不直接修改 shell rc 文件

- `~/.zshrc` / `~/.bashrc` 是用户高度个人化的配置文件，Sieve 修改后与用户其他工具发生冲突（如 conda / rbenv / nvm PATH 设置顺序问题）难以排查；
- PATH 管理应由 .dmg 安装包的 `pkgbuild` post-install script 完成（`/usr/local/bin/sieve` 符号链接），不在 `sieve setup` 范围内；
- `.bashrc` / `.zshrc` 改动对用户几乎不可见，uninstall 容易遗漏；改 Claude Code settings.json + launchd plist 已经足够。

---

## 决策

### 1. 三件套职责

#### `sieve setup`

**做什么**：
1. 检测 Claude Code 安装路径（`~/.config/claude/` 或 `~/Library/Application Support/Claude/`）
2. 读取 Claude Code `settings.json`，注入 `hooks.preToolUse` 条目（`command: sieve-hook`，`onError: block`）
3. 写入 `~/.claude/settings.json` 的 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`（环境变量注入，或改 proxy 设置）
4. 生成 launchd plist：`~/Library/LaunchAgents/com.sieve.daemon.plist`，`ProgramArguments: [sieve start --config ~/.sieve/sieve.toml]`
5. `launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.sieve.daemon.plist`
6. 创建 `~/.sieve/` 目录（权限 `0700`）、`~/.sieve/pending/` 和 `~/.sieve/decisions/`（权限 `0700`）
7. 将所有改动记录到 `~/.sieve/setup.log`（追加格式，含时间戳 + 操作内容 + 原始备份路径）

**安全约束**：
- 修改任何用户文件前，先**打印将要改的内容 diff** 并要求用户输入 `y` 确认（不能跳过）
- 原始文件备份到 `~/.sieve/backup/<filename>.<timestamp>`，backup 路径写入 setup.log

```
$ sieve setup

Sieve Setup (v0.4.0)
────────────────────

Detected Claude Code at: ~/.config/claude/

Will make the following changes:

  [1] Add hook to ~/.config/claude/settings.json:
      +  "hooks": { "preToolUse": [{ "command": "sieve-hook", "onError": "block" }] }

  [2] Set proxy in ~/.claude/settings.json:
      +  "ANTHROPIC_BASE_URL": "http://127.0.0.1:11453"

  [3] Install LaunchAgent: ~/Library/LaunchAgents/com.sieve.daemon.plist

  [4] Create ~/.sieve/ directory (0700)

Proceed? [y/N]:
```

#### `sieve doctor`

**检查项**（按优先级）：
1. `sieve-hook` 二进制存在且可执行（`which sieve-hook`）
2. Claude Code settings.json 包含 hook 注册项（`onError: block`）
3. `ANTHROPIC_BASE_URL` 正确指向 `http://127.0.0.1:11453`
4. launchd 服务在运行（`launchctl list com.sieve.daemon` 返回 0）
5. 代理监听端口可达（`curl -s http://127.0.0.1:11453/health`）
6. **Canary secret 测试**：发送包含 `SIEVE_CANARY_TEST_KEY=sk-ant-` 格式 fake key 的请求，验证代理拦截并返回正确响应

输出格式：
```
$ sieve doctor

  ✓ sieve-hook binary found at /usr/local/bin/sieve-hook
  ✓ Claude Code hook registered (onError: block)
  ✓ ANTHROPIC_BASE_URL configured
  ✓ Daemon running (pid 12345)
  ✓ Proxy reachable on :11453
  ✓ Canary detection working

All checks passed.
```

#### `sieve uninstall`

**默认 dry-run**：不加 `--confirm` flag 时只打印将要删除/回滚的内容，不实际执行。

**回滚逻辑**：读 `~/.sieve/setup.log`，按倒序精确回滚每一项改动（恢复备份文件）。

**不删除**：`~/.sieve/audit.db`（审计日志，用户数据）、`~/.sieve/sieve.toml`（用户配置），除非加 `--purge` flag。

### 2. macOS only（`#[cfg(target_os = "macos")]`）

setup / doctor / uninstall 子命令用严格 cfg 限制：

```rust
#[cfg(target_os = "macos")]
pub mod setup;
#[cfg(target_os = "macos")]
pub mod doctor;
#[cfg(target_os = "macos")]
pub mod uninstall;
```

非 macOS 运行时，CLI 层在 dispatch 阶段报友好错误：

```
error: 'sieve setup' is currently only supported on macOS.
       Linux and Windows support is planned for Phase 2.
       See https://docs.sieve.dev/installation for manual setup instructions.
```

**不**用 `#[cfg(not(target_os = "macos"))]` 空实现——编译期直接排除，防止非 macOS 二进制意外包含 macOS 特有系统调用。

### 3. 操作幂等性

多次运行 `sieve setup`：
- 已配置的项目显示 `(already configured, skipping)`；
- 不重复备份（检查 backup 目录已存在同名备份则跳过）；
- launchd plist 已加载时先 unload 再重新 load（原子性）。

### 4. 不污染 shell rc 文件

绝不写入 `~/.zshrc`、`~/.bashrc`、`~/.bash_profile`、`~/.zprofile`、`~/.profile`。PATH 管理由 .dmg 的 post-install script 负责（`/usr/local/bin/sieve` + `/usr/local/bin/sieve-hook` 符号链接），与本 ADR 解耦。

---

## 影响

### 正面影响

1. **安装转化率**：一键 setup，无手动编辑 JSON 风险，预期安装成功率 > 95%；
2. **可诊断性**：doctor 命令给客服提供标准输出，用户复制粘贴即可定位问题；
3. **干净卸载**：setup.log 驱动的精确回滚，无残留；
4. **安全改动展示**：强制 diff 预览 + 确认，用户完全了解 Sieve 改了哪些文件，减少信任摩擦。

### 负面影响

1. **macOS launchd 适配**：launchd plist 有 macOS 版本差异（macOS 12+ 和 macOS 15+ 的 bootstrap 命令略有不同）；需要运行时检测 `sw_vers`；
2. **Claude Code 版本差异**：settings.json 路径和 hook 配置 schema 在不同 Claude Code 版本可能不同；doctor 命令需要检测 Claude Code 版本并适配；
3. **权限提升**：launchctl load 不需要 sudo（用户 LaunchAgent），但写 `/usr/local/bin/` 需要 sudo（由 .dmg 安装包处理，不在 setup 命令范围）；
4. **onError: block 依赖**：这是 Claude Code hook 机制提供的能力，未来 Claude Code 改动 hook API 时需要同步更新。

### 需要更新的文档

- `docs/guides/deployment.md` §2.1、§4 —— macOS 安装改为 .dmg + sieve setup 流程
- `docs/guides/development.md` §3.4 —— 开发环境启动 daemon 改为 sieve setup
- `docs/api/api-reference.md` §4 —— 环境变量说明更新（sieve setup 自动配置）

---

## 相关文档

- [PRD-sieve v1.4 §6.6](../prd/sieve-prd-v1.5.md) —— 安装程序需求
- [ADR-013](./ADR-013-ipc-protocol.md) —— `~/.sieve/` 目录结构（setup 负责创建）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— hook 注册 `onError: block` 的必要性
- [ADR-012](./ADR-012-native-gui-app-phase1.md) —— .dmg 安装包（PATH 管理的载体）
- [architecture.md](./architecture.md) —— sieve-cli 子命令结构
