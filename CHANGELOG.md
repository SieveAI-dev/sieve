# Changelog

本项目所有重要变更记录于此。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Security

- **audit.db 脱敏契约锁定（GUI 审查 D-1）。** 核实 daemon 写 audit.db 时**不持久化任何命中
  原文 / 密钥 / 助记词 / 地址（含前缀片段）**：`raw_json` 列仅存已序列化的 `AuditEvent` 元数据
  （`rule_id` / `severity` / `request_id` / `caller`），命中证据不进入 `AuditEvent`，出站脱敏类
  恒 `NULL`，全 crate 零明文写入路径。在 `docs/design/data-model.md §6.3` 锁为规范性安全不变量，
  并禁止后续新增任何含原文/前缀的列（`evidence` / `evidence_meta` / `matched_text`）。
- **修正 `data-model.md §6.2` 审计表 schema 漂移（GUI 审查 D-1）。** 本节此前误写表名为 `events`
  且含 `evidence_meta`（存密钥前缀）列，与实现（`audit.rs` 的 `audit_events` 表 + `raw_json`）及
  §14 归档段不一致。现对齐到真实 `audit_events` schema。注：GUI 曾按旧 schema 读
  `events.evidence_meta`，需跨仓对齐到本节真实 schema（GUI 仓侧改动，不在本 PR）。
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
- **`sieve.hello` 握手新增 `paused_until` 字段（GUI 审查 D-5）。** nullable `Timestamp`
  （§4A 毫秒 + Z），与 `paused` 配对：client 握手即可正确进入暂停态并显示「恢复至…」，无需
  等首条 `sieve.paused_changed` 才补齐 until。v2.x 向后兼容扩展（字段新增 + `#[serde(default)]`，
  旧 client 忽略未知字段），**不** bump `protocol_version`。SPEC-005 §3.2 同步。

### Fixed

- **暂停态握手丢弃截止时间修复（GUI 审查 D-5）。** `handle_connection` 发 `sieve.hello` 时此前
  只读 `paused` 布尔、丢弃 `paused_until` 时间值，导致 client 握手进入暂停态却拿不到 until →
  状态降级、菜单栏假装正常（违反「菜单栏状态以握手为准，不假装健康」）。现 daemon 取过期过滤
  后的 until 快照填入 hello，`paused` 由其 `is_some()` 派生，二者天然一致。含 schema 双向稳定
  测试 + 暂停态握手 wire 回归测试（`hello_carries_paused_until_when_paused`）。
- **`timeout_seconds` 越界钳制（GUI 审查 D-3）。** `IpcServer::request_decision`（所有
  `request_decision` 发送的唯一 choke point）在 wire 序列化前把 `timeout_seconds` 钳到 SPEC-005
  §6.1.1 区间 `[30,120]`（越界 `0` / 合并取最小后 `<30` / `>120` 一律钳边界 + warn，不拒绝以免
  中断决策流 fail-open）。修复 GUI 收到 `0` 时倒计时首 tick 即归零的隐患。含单元测试。
- **CI fuzz-quick 构建修复：fuzz 改用 `--target x86_64-unknown-linux-gnu`。** 此前 cargo-fuzz
  默认编 musl target，需 musl rust-std **和** musl C++ 编译器（libfuzzer-sys C++ 运行时），
  `--profile minimal` 两者皆缺 → 构建失败（`E0463` / 找不到 `x86_64-linux-musl-g++`），fuzz
  **从未真正运行**（CI 覆盖虚化）。改用 gnu target（ubuntu host，std + g++ 现成）后 fuzz 可
  真正运行并提供覆盖（仍 continue-on-error）。
- **HIPS 超时决策回传幂等性核实（GUI 审查 D-3）。** 核实 daemon `decision_response` 处理已按
  `request_id` 幂等（`pending.remove`）：GUI 倒计时 fail-closed 回传的 `deny, by_user=false` 与
  daemon 自身 oneshot 兜底超时不会对同一请求双重处置。在 SPEC-005 §6.2.1 注 ③ 锁定此契约。

### Changed (契约锁定 / 文档，GUI 审查 D-2 / D-3 / D-4)

- **SPEC-005 §4A `JsonRpcId` 锁定为 String（D-2）。** 此前写「String 或 Int」有歧义；现明确
  daemon **MUST NOT** 发 Number id，所有 request `id` 恒为 `Uuid` 格式 String（daemon 实现已符合，
  仅锁契约防回归 + 防 GUI 误判 response）。
- **SPEC-005 §6.1.1 锁定单 issue `rule_id` 恒非空（D-4）。** daemon 单 issue `request_decision`
  必发非空 `rule_id`（源自 `Detection.rule_id` 白名单，6 处构造点已核实无空串路径）；client 应断言
  非空，空串视为 `-32602`，杜绝"无命中详情仍可批准"。
- **SPEC-005 §6.1.1 / §6.2.1 锁定 `timeout_seconds` 区间与超时回传幂等契约（D-3）。**
