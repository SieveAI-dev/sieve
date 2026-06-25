# Changelog

本项目所有重要变更记录于此。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Security

- **入站四路由对等：非流式 JSON 路径补写 hook pending 文件。** 此前 `handle_json_inbound`
  对 `HookMark`（hook_terminal 处置，如 IN-CR-02 危险 shell / IN-CR-03 敏感文件访问）命中
  既不注入 `sieve_blocked` 也不写 IPC pending 文件，导致非流式 JSON 响应里的危险工具调用对
  依赖 pending 文件的 PreToolUse hook（如 Claude Code `sieve-hook check`）双层防御全失效。
  现与 SSE 路径一致：`HookMark` 命中写 pending 文件供 hook 拦截，写失败 fail-closed 升级阻断。
  守护工程硬约束「所有入站能力必须经过 content-type 路由矩阵测试」（`.cursorrules §二 #16`）。
- **修复工具输入扫描的引号转义绕过。** `check_tool_use` 此前仅扫 `serde_json::to_string(&input)`，
  序列化把字符串值里的 `"` 转义成 `\"`，使依赖引号的 Critical pattern（如 IN-CR-02-EVAL
  `eval\s*["(\$]` 匹配 `eval "$(...)"`）被转义符破坏而绕过。现改为递归扫描反转义后的原始
  字符串值（保留序列化扫描作兜底，按 fingerprint 去重），关闭该 Critical 规则绕过。

### Changed

- **[BREAKING] BIP39 助记词出站处置：`Block`（426 硬阻断）→ `auto_redact`（200 脱敏转发）。**
  对齐工程硬约束「出站脱敏不打断工作流」（`.cursorrules §二 #13`）：检出 checksum 合法的助记词
  后静默改写为占位符 + 转发上游，而非硬阻断中断 crypto 开发者工作流。明文助记词在转发前
  已脱敏，安全不变量不变。同时修正脱敏 span 为助记词窗口本身（此前为整段输入，会过度脱敏
  毁掉用户原文）。
- **BIP39 出站检测 `rule_id`：`OUT-09` → `OUT-14`。** 消除与 outbound 规则集中 Slack Token
  规则（`OUT-09`）的编号撞号——此前代码构建的 BIP39 检测硬编码 `OUT-09`，污染审计 / 指纹 /
  fail-closed 注册表。

### Added

- **CI 新增 `detection-regression` job：在规则包可用时跑红队 + 四路由回归。** 检测规则通过
  签名规则包下发，不内置于源码树；红队 / 四路由集成测试在规则包缺失时优雅 SKIP（覆盖未
  真正生效）。新 job 在规则包可用时（`SIEVE_RULES_PACK_B64` secret）落地规则后运行回归，
  使四路由对等 / 出站脱敏处置等覆盖真实生效；secret 缺失时优雅跳过。

### Fixed

- **`redteam_inbound` 测试断言更正为 pending-file 机制。** 此前断言危险 shell tool_use 应注入
  `sieve_blocked`，与 hook_terminal 处置的实际语义（写 pending 文件、由 hook 在 PreToolUse 拦截、
  daemon 不截流）矛盾（系 disposition 优先重构后遗漏同步）。现断言四路由均写出 pending 文件，
  正确守护双层防御的四路由对等。
