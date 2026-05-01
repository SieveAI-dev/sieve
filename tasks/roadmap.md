# Sieve 12 周里程碑 Roadmap

> Source of truth: [PRD v1.5 §10](../docs/prd/sieve-prd-v1.5.md#10-12-周里程碑8-周-dogfood--4-周闭测)（Week 6-8 已按 v1.5 multi-agent 扩展重写）
> 状态：**v2.0 + v2.1 代码 100% 落地（2026-05-01），进入 dogfood 准备阶段**。Phase A 全部代码任务完成，剩余 5 项非代码工作（见 tasks/status-2026-05-01.md）。
>
> 本文是 PRD §10 的执行视图，**任务粒度 / 验收标准** 跟随 PRD 同步更新；本文新增"依赖"列与"风险"列辅助调度。

---

## Phase A · dogfood（Week 1-8）

### Week 1 · 基础设施 + Anthropic 协议 + sigstore pipeline

**完成定义**（PRD §10.1 Week 1）：

- [ ] doskey 自己用 Claude Code，设 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`，所有日常操作正常 ← 等装 cargo 后实测
- [ ] sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 已跑通 ← 等首次 tag push 后验证

**任务清单**：

- [x] Rust workspace 骨架（sieve-core / sieve-rules / sieve-cli 三个 crate）
- [x] hyper + tokio + rustls 跑通透明转发 Anthropic Messages API
- [x] SSE 透传基础（不做规则匹配）
- [x] UnifiedMessage 内部 schema（Anthropic only，OpenAI/Gemini 接口预留）
- [x] sigstore 签名 pipeline（cosign + Rekor）
- [x] GitHub Actions reproducible build pipeline（双构建 SHA-256 比对）
- [x] cargo fmt / clippy / 基础 CI 跑通
- [~] ~~GitHub repo 公开~~ — 已被 [ADR-011](../docs/design/ADR-011-private-until-ga.md) 撤销；Week 12 GA 一次性公开
- [ ] 启动海外公司注册（香港首选，PRD §10.1 Week 7-8 Stripe 接入需要）← 用户行动项

**依赖**：无（这是起点）
**关键风险**：sigstore + reproducible 双构建首次跑通预算 2-3 天，超期需立刻识别
**关联 ADR**：ADR-001 / ADR-004 / ADR-006

---

### Week 2 · 出站 P0

**完成定义**（PRD §10.1 Week 2）：

- [x] paste .env 触发拦截（集成测试 outbound_block 覆盖 + smoke test 26/26）
- [ ] 标准 secret benchmark FP < 1%, Recall > 70%（本地测试集已达，完整 benchmark 留 Week 4）

**任务清单**：

- [x] vectorscan-rs 多模式正则集成
- [x] OUT-01~12 全部 P0 出站规则（API key / AWS / GitHub / JWT / 私钥 / 助记词等）
- [x] BIP39 SHA-256 checksum 验证（差异化点）
- [x] 占位符黑名单 + .sieveignore 学习型白名单
- [x] 单元测试覆盖 ≥ 80%（66 单元 + 20 集成 = 86 测试）
- [x] 出站拦截 UI / 脱敏工作流（被动 426 + body 含人类可读说明）
- [ ] **ADR-008 出站 Critical 状态码 dogfood 验证**：Week 2 实现已用 426，本周 dogfood 期间（2026-04-27 起）实测 Claude Code SDK 行为；若一周内无异常，升 Accepted（候选编号已在 [ADR-INDEX](../docs/design/ADR-INDEX.md) 登记）

**依赖**：Week 1 Rust 骨架 + API 转发
**关键风险**：BIP39 checksum 实现错误导致误报，需严格单测
**关联文档**：PRD §5.1（出站检测P0表）/ [api-reference.md §7.2](../docs/api/api-reference.md)（426 当前默认）

---

### Week 3 · 入站 Crypto 钩子

**完成定义**（PRD §10.1 Week 3）：

- [x] 复现 UCSB 论文 4 类攻击 PoC，Sieve 全部捕获（IN-CR-01/02/05 + IN-CR-04 warn）

**任务清单**：

- [x] SSE Parser + 流式处理框架
- [x] Tool Use Aggregator（完整工具调用重组）
- [x] IN-CR-01 地址替换检测（对话历史对比 + Levenshtein）
- [x] IN-CR-02 危险工具调用拦截（bash rm -rf / curl|sh / eval）
- [x] IN-CR-05 签名工具 fail-closed（signTransaction / signTypedData 全拦）
- [x] YOLO mode 不可关闭机制（运行时检查）
- [x] 大量 SSE 边界 fuzz test（cargo fuzz + AFL++ 双引擎，14 corpus seed）

**依赖**：Week 2 出站规则引擎基础
**关键风险**：SSE 流式边界处理复杂，半行 chunk 必须 fuzz 覆盖，否则规则绕过
**关联文档**：PRD §9 硬约束 #5

---

### Week 4 · 入站通用 + benchmark 数据集

**完成定义**（PRD §10.1 Week 4）：

- benchmark 数据集跑通，所有 Critical 规则达到 FP 阈值

**任务清单**：

- [x] IN-CR-03 敏感路径访问（10 条规则：SSH / AWS / GCP / Solana / Ethereum keystore / GPG / netrc / macOS Keychain / dotenv，含 allowlist；high warn 级别。Week 5 接 5s 倒计时弹窗）
- [x] IN-CR-04 持久化机制（9 条规则：shell rc / crontab / launchctl + LaunchAgents plist / systemctl + systemd unit / fish config / macOS Login Items；Critical block + fail-closed，全部进 `FAIL_CLOSED_RULES`，YOLO mode 不可关。附带 [BREAKING] 重命名旧 IN-CR-04 markdown exfil → IN-GEN-04）
- [x] sieve-rules manifest 字段扩展：`disposition` / `timeout_seconds` / `default_on_timeout`（处置矩阵二维字段已落地，详见 [ADR-012](../docs/design/ADR-012-disposition-matrix.md)）
- [x] sieve-ipc crate 骨架（IPC server Unix socket + GUI 通知协议，本周末完成）
- [x] sieve-hook crate 骨架（Claude Code PreToolUse hook 二进制，本周末完成）
- [ ] **【P0 必须 Week 4 关闭】非流式 JSON 响应里的 tool_use 入站检测**：当前 daemon 仅扫 `text/event-stream` SSE 流，非流式 `application/json` 响应里的 tool_use 整体绕过所有入站规则（IN-CR-02/03/04/05 / IN-GEN-* 全失效）。dogfood 实测发现，详见 [lessons.md](./lessons.md)。修复：daemon 按 response content-type 路由，JSON 路径解析 `AnthropicResponse.content[]` → 提取 tool_use → 走 `InboundFilter::on_tool_use_complete`，命中 fail-closed Critical 时把 body 替换为 `sieve_blocked` 等价 JSON。集成测试加非流式响应路径覆盖。
- IN-GEN-01~05 全部 P0 通用规则（shell 危险模式 / 远程脚本 / 编码执行 / Markdown exfil）
- ~~出站 OUT-01~05/12 自动脱敏路径~~（推到 Week 5）
- ~~入站 Hook 类不修改 SSE 流~~（推到 Week 5）
- CLI 弹窗 + 命令行确认交互（最简 stdout 版，Week 5 接 IPC + GUI）
- **benchmark 数据集构建**
  - 200-500 条合成攻击样本（UCSB 4 类 + drainer + Pink Drainer 数字化 + npm typosquat + curl|sh + eval base64）
  - 50-100 条真实 benign 会话回放（doskey 自己 Claude Code 日常工作录制）
  - canary 测试（假 BIP39 / 假地址 / 假 selector / honeypot 钱包）
  - 验证 Critical FP < 0.5%, High FP < 5%

**依赖**：Week 3 SSE + 工具调用处理
**关键风险**：benign 会话标注耗时，需提前准备。假 sample 生成需谨慎，防止反向泄露
**关联文档**：PRD §9 公理 12 / §10.1 Week 4 数据集具体化

---

### Week 5 🆕 · Native GUI App（独立仓库）+ sieve setup + IPC server + sieve-hook + 二维处置矩阵落地

**完成定义**（PRD §10.1 Week 5）：

- doskey 朋友 30 分钟内能 .dmg 安装 + 跑 setup + 看到拦截工作 ← .dmg 打包阻塞 R10-#3（GUI 独立仓库），核心引擎代码已全部完成

**任务清单**：

- [x] **sieve-ipc crate**（Week 4 末骨架已完成）
- [x] **sieve-hook crate**（Week 4 末骨架已完成）
- [x] **sieve-rules manifest 字段扩展**：`disposition` / `timeout_seconds` / `default_on_timeout`（Week 4 末已完成）
- [x] **sieve-core pipeline 重构**：
  - `outbound_redact`：命中出站规则时改写 body bytes（替换 secret 为 `[REDACTED]`），而非仅返 426
  - `inbound_hook`：Hook 类规则（IN-CR-02/03/04/05）不修改 SSE 流，通过 IPC 通知 GUI
  - `inbound_hold`：25s keep-alive comment 注入 + IPC 通知 GUI 等待审批（详见 [SPEC-002](../docs/specs/SPEC-002-inbound-hold.md)）
- [x] **sieve-cli 新子命令**：
  - `sieve setup`：改写 Claude Code `settings.json` 注册 `PreToolUse` hook + 写 `ANTHROPIC_BASE_URL` + 写 launchd plist（详见 [SPEC-003](../docs/specs/SPEC-003-setup-doctor-uninstall.md)）
  - `sieve doctor`：canary 拦截测试，验证 hook + daemon + IPC 全链路就位
  - `sieve uninstall`：按 `setup.log` 逐步回滚，dry-run / 确认 / 执行三阶段
  - `audit.rs`：接入 SQLite append-only 审计，schema 见 [data-model.md](../docs/design/data-model.md)
  - `daemon.rs`：删除对 Hook 类规则的 `sieve_blocked` SSE 注入（改由 IPC + GUI 处理）
- [x] **集成测试一次性按 v1.4 重写**：覆盖 IPC hold 流程 / setup 幂等性 / uninstall 回滚 / outbound_redact / 非流式 JSON 入站检测
- [ ] **GitHub Releases 自动化构建上传**（macOS only，`aarch64-apple-darwin` + `x86_64-apple-darwin`）← 等 GA 前 repo 公开后上传
- [ ] **三件套 .dmg 打包**：GUI App + 后台代理 + sieve-hook 合并成单 .dmg ← 阻塞 R10-#3 绝对路径 + GUI 独立仓库
- **GUI App（独立仓库 `sieve-gui-macos`，由 doskey 平行开发）**：
  - 状态栏常驻图标 + 审批弹窗
  - IPC 通道连接 sieve-ipc（[SPEC-001](../docs/specs/SPEC-001-ipc-protocol.md)）
  - 倒计时视觉（25s hold 剩余时间）
  - approve / reject / snooze 三个动作

**依赖**：Week 4 核心引擎稳定 + sieve-ipc / sieve-hook 骨架
**关键风险**：GUI App 独立仓库与 Rust 端 IPC 协议需严格对齐；集成测试重写工程量大，需优先排期
**关联文档**：PRD §10.1 Week 5 / SPEC-001~003 / ADR-013~016

---

### Week 6 🆕 · OpenAI 协议适配 + multi-agent setup（v1.5）

**完成定义**（PRD v1.5 §10.1 Week 6）：

- UnifiedMessage 双协议跑通（Anthropic + OpenAI 解析为同一中间表示）
- `sieve setup --agent claude|openclaw|hermes` 可执行，至少 Claude Code 路径全绿

**任务清单**：

1. [x] 新模块 `crates/sieve-core/src/protocol/openai.rs`：OpenAI Chat Completions 解析（参考 ADR-018）
2. [x] SSE Parser 适配 OpenAI delta 格式（无 event 头 + `[DONE]` 终止符）
3. [x] UnifiedMessage 双协议跑通（Anthropic + OpenAI 都能解析为同一中间表示）
4. [x] `sieve setup --agent claude|openclaw|hermes` 多 agent 参数 + `--all-detected` 自动扫描（参考 SPEC-004）
5. [x] IN-GEN-06 外部 channel prompt injection 规则定义 + vectorscan 编译
6. [x] IN-CR-06 OpenClaw 动态 skill 加载 fail-closed 规则定义
7. [x] IPC schema 加 `source_agent` / `origin_chain` / `source_channel` 字段（向后兼容，`#[serde(default)]`）
8. [x] X-Sieve-Origin HTTP header 协议落地（签名生成 + 验证，参考 ADR-019）

**依赖**：Week 5 三件套 .dmg + IPC + sieve-hook 完整
**关键风险**：OpenAI SSE 格式差异（尤其 delta 累积 + `[DONE]` 处理）需严格 fuzz 覆盖；IPC schema 向后兼容需测试旧格式能正常 deserialize
**关联文档**：PRD v1.5 §5.2 / ADR-018 / ADR-019 / SPEC-004

---

### v2.0 + v2.1 · HIPS 改造（2026-05-01，代码 100% 落地）

> 该段记录 PRD v2.0 / v2.1 新增工作，跨越 Week 5-8 执行窗口同步完成。

**Phase A 骨架（commit `cd0248d`）**：

- [x] 新增第 6 个 crate `sieve-policy`（策略引擎，独立于 sieve-rules 和 sieve-core）
- [x] `MatchEngine` trait + `ScanRequest` / `ScanReport` 抽象
- [x] `process_context` 模块（macOS `proc_pidinfo` + peer_addr 4-tuple 反查 PID）
- [x] audit v2 事件定义（7 个新 `AuditEvent` 变体）
- [x] IPC 三态字段（`HoldOutcome` 加 `remember` + `context_hint`）
- [x] `sieve rules` CLI 子命令（edit / list / disable / enable）

**Phase A 接入（commit `5021f0c`）**：

- [x] daemon 接入 `LayeredEngine`（向量引擎 + 用户规则引擎双层）
- [x] 三态决策（Allow / Deny / Remember，灰名单写入）
- [x] 出站灰名单自动脱敏路径（命中后 body bytes 改写 + 状态栏通知）
- [x] 4 类 content-type 矩阵路由（application/json / text/event-stream / 其他）
- [x] criterion benchmark 集成（sieve-rules bench CI）

**Phase B 骨架（commit `e68958b`）**：

- [x] `sequence` 模块（行为序列窗口，滑动 window + 事件计数）
- [x] IN-SEQ-01（快速部署序列）/ IN-SEQ-02（多签绕过序列）/ IN-SEQ-03（密钥访问 + 网络外传序列）
- [x] daemon 双路径接入（feature flag `sequence_detection`，默认关闭，等 dogfood 数据驱动评审）

**推迟清单 1（commit `af0b61a`）**：

- [x] 用户规则 `direction` 字段（inbound / outbound / both）
- [x] 序列 e2e 测试矩阵（6 个端到端测试）
- [x] criterion CI 集成（`cargo bench` 不报错）
- [x] `process_context` 4-tuple macOS 真实实现

**推迟清单 2（commit `05040cd`）**：

- [x] IPC 协议扩展（`StatusBarNotify` + `ReloadUserRules` 消息类型）
- [x] daemon 全接入：`AuditStore` 透传 + 灰名单全路径 + IN-SEQ-* IPC + hot-reload best-effort
- [x] `peer_addr_to_pid` 真实实现 + caller 透传 audit

**v2.1 工程项（commit `74e5e3a`）**：

- [x] `LayeredEngine` zero-downtime hot swap（`arc-swap` 原子替换）
- [x] `try_write_graylist` 失败路径 audit `ERROR` 记录
- [x] 多 GUI 客户端 broadcast 支持

**剩余 5 项非代码工作**：

- [ ] `vectorscan_rs::hs_database_size()` API（等外部 crate 升级）
- [ ] `origin_header.rs` GA 前真实 Ed25519 密钥（部署任务）
- [ ] OpenClaw `skill_install_guard` Week 7 实测（dogfood 数据驱动）
- [ ] 行为序列升级 Block 类的 ADR 评审（需 4 周 ≥ 50 序列样本 + FP < 0.5%）
- [ ] 行为序列 ML 分类器训练（需积累 dogfood 数据集）

---

### Week 7 · OpenClaw / Hermes 集成测试（v1.5）

**完成定义**（PRD v1.5 §10.1 Week 7）：

- 场景 E（OpenClaw 跨通道 injection）端到端跑通
- 场景 F（Hermes sub-agent 嵌套）端到端跑通

**任务清单**：

1. 装 OpenClaw daemon + 让它走 Sieve 代理（手动改 config，验证 PRD v1.5 Open Question #9）
2. 装 Hermes CLI + 让它走 Sieve（手动改 config，验证 PRD v1.5 Open Question #10）
3. 跑场景 E：OpenClaw 接 WhatsApp/Slack → 攻击 prompt → IN-GEN-06 触发 → GUI 弹窗 + 拒绝 → OpenClaw 收到 sieve_blocked 停止
4. 跑场景 F：Hermes delegate Claude Code → X-Sieve-Origin header → 下游识别 chain_depth=1 → 不二次弹窗
5. X-Sieve-Origin header 在 Hermes delegate Claude Code 时正确注入（验证 PRD v1.5 Open Question #11）

**依赖**：Week 6 OpenAI 协议适配 + IN-GEN-06/IN-CR-06 规则上线
**关键风险**：OpenClaw / Hermes 实际 config 格式未知（Open Question），需要 dogfood 期间人工调试
**关联文档**：PRD v1.5 §4.5 / §4.6 / US-18 / US-19

---

### Week 8 · 高强度 dogfood + Stripe 接入 + 三家扩展验证

**完成定义**（PRD v1.5 §10.1 Week 8）：

- doskey 用 Sieve 跑 1 周三家 agent，无 P0 / P1 bug
- Stripe 账号 + license key 系统上线

**任务清单**：

- doskey 自己用 OpenClaw 接 Telegram + Slack，实测 IN-GEN-06 触发与 false positive 率
- 用 Hermes delegate 给 Claude Code 测场景 F，收集调用链渲染反馈
- 刻意尝试 edge case（多终端 / 跨语言 / 网络延迟模拟）
- 每次 FP 都进 issue 列表 + 修复，调整 IN-GEN-06 命令式短语 pattern
- 规则库更新机制完整测试 + 第一次签名规则库下发测试
- 海外公司（香港/新加坡）注册完成
- Stripe 账号关联公司主体
- license key 生成 + 验证系统 + 14 天试用激活机制
- 降级模式（试用期后只读警告）实现
- **GUI App**（独立仓库）：倒计时视觉完善、合并多 issue 弹窗、设置界面 preset 切换（crypto / general / strict）

**依赖**：Week 7 集成测试跑通，公司注册已启动（Week 1）
**关键风险**：公司注册周期延误（4-6 周），必须 Week 1 启动以赶上 Week 8 Stripe；IN-GEN-06 FP 率需 dogfood 数据驱动调整
**关联文档**：PRD v1.5 §7.1 定价 / §11.5.1 法律实体

---

## Phase B · 闭测（Week 9-12）

### Week 9 · 闭测启动（画像精确化）

**完成定义**（PRD §10.2 Week 9）：

- 5-10 个精准闭测用户入场
- Discord 闭测频道建立，日常反馈机制就位

**闭测用户画像必须满足**（PRD §10.2 Week 9，v1.3 修订）：

- 高频 hackathon builder（ETHGlobal / Solana 常客，单 hackathon 写 10+ 合约）
- bug bounty hunter / 审计研究员（Code4rena / Sherlock / Immunefi 活跃）
- 小团队 protocol engineer（< 10 人，决策快，口碑传播力强）
- **不要找**：大企业 dev / 纯 web2 友人 / 纯 KOL

**任务清单**：

- Week 9 邀请 5 人（具体名单 TBD，见 PRD v1.5 §14 Open Questions）；邀请 1-2 个 OpenClaw / Hermes 重度用户
- Discord 闭测频道建立 + 权限配置
- 闭测 license key 分配（5 个独立 key）
- 每日反馈处理 SLA（24 小时内回复）
- bug 优先级标记系统

**依赖**：Week 8 Stripe + license 系统完成
**关键风险**：闭测用户选错方向会浪费 4 周时间，必须严格筛选
**关联文档**：PRD §3.1 P0 客群 / §10.2 Week 9 用户画像

---

### Week 10 · 闭测 + 内容准备

**完成定义**（PRD §10.2 Week 10）：

- 3 篇引爆文章初稿完成
- 闭测 bug 修复，二轮邀请准备

**任务清单**：

- 修闭测 Week 9 发现的 bug
- 内容 1 草稿：中转站揭黑（实测复刻 UCSB 方法论）
- 内容 2 草稿：**自证清白**（Sieve 怎么证明自己不是新的 LiteLLM）
- 内容 3 草稿：Pink Drainer 攻击复盘 + Sieve 防御
- 文章 2 核心叙事：sigstore 签名 + reproducible build + 透明规则日志
- 在线编辑 / 技术审校流程建立
- 邀请函 v2（给 Week 11 闭测扩大）

**依赖**：Week 9 闭测反馈 + 内容素材积累
**关键风险**：文章 2 是核心营销弹药，品质必须高；技术细节必须正确
**关联文档**：PRD §10.2 Week 10 / §1.2 第 4 句自证清白叙事

---

### Week 11 · 闭测扩大 + KOL 接洽

**完成定义**（PRD §10.2 Week 11）：

- 邀请 5-10 个新闭测用户（同样画像标准）
- KOL 接洽启动，数据合作优先

**任务清单**：

- 邀请 5 个新闭测用户（同样 builder / hunter / engineer 画像）
- 闭测 license key 管理（10 个总的，追踪使用情况）
- landing page 初稿（英文为主，中文次之）
- 视觉设计（logo / 色系 / 品牌指南）
- **KOL 接洽启动**（PRD §10.2 Week 11，数据合作优先）
  - Chaofan Shou (@Fried_rice) 主动接洽（UCSB 论文一作，顾问关系）
  - 慢雾 @evilcos 接洽（misttrack-skills 数据合作）
  - 数据合作洽谈（SlowMist / ScamSniffer / GoPlus）
  - 主动接洽 Peter Steinberger（OpenClaw）/ Nous Research 团队（Hermes）
- GitHub stars 和 issue 管理流程

**依赖**：Week 10 内容初稿 + 闭测稳定反馈
**关键风险**：KOL 接洽容易流于虚客套，需提前准备具体合作方案（参见 PRD §13.2）
**关联文档**：PRD §13.2 数据侧合作清单 / §13.3 内容合作清单

---

### Week 12 · GA 发布

> ⚠️ 关联 ADR-011：GA 之前 repo 完全私有，GA 当天首次公开；Week 9-11 闭测期间所有反馈走 Discord 私发，不暴露 repo。

**完成定义**（PRD §10.2 Week 12）：

- GA 第一周 GitHub stars > 200
- 试用注册 > 100
- 首批付费用户 ≥ 10

**任务清单**：

- 代码开源（MIT License）
- 二进制 sigstore 签名验证文档
- reproducible build 验证指南
- landing page 上线（英文 + 中文）
- 文章 1（中转站揭黑）发表（Twitter / Hacker News / Mirror）
- 文章 2（自证清白）发表（Twitter / Hacker News / Mirror + 中文版）
- 文章 3（Drainer 复盘）发表（Week 14，较文章 1/2 滞后）
- 14 天试用全面开放
- Stripe 收款上线 + 首次处理付款
- README 国际化（英文 + 中文）
- 社区链接（Discord / GitHub Discussions）
- 发 Show HN / Hacker News post
- 在 r/ethdev / r/cryptocurrency 发帖

**依赖**：Week 11 内容完稿 + KOL 反馈
**关键风险**：GA 发布节奏（文章 1+2 同步 vs 错开）影响传播，需与 KOL 协调
**关联文档**：PRD §10.2 Week 12 / §11.5.2 营销渠道分级

---

## Phase C · 维护（Week 13+）

每周稳定投入 5-10 小时：

- **每月一篇深度内容**（内容 3 + 后续系列）
- **用户反馈处理 + bug 修复**（优先级排序，每周处理清单）
- **规则库每周更新一次**（签名 + changelog）
- **季度大版本**（Phase 2 功能逐项上线）
  - 中文 PII 检测
  - 用户自定义规则 DSL
  - npm / pip typosquat
  - MCP 拦截 + scope-aware policy（Week 16-20）
- **第二个用户主动要 OpenClaw / Hermes / MCP 适配时再做**（PRD §6.1 Phase 1 约束）

---

## 跨周关键依赖图

```
Week 1 sigstore pipeline ─┐
                          ├─→ Week 12 GA 二진制可独立验证
Week 1 海外公司注册启动 ─→ Week 7-8 Stripe 接入 ──┘

Week 1 Rust 骨架
  ↓
Week 2 出站 P0 规则
  ↓
Week 3 入站 Crypto 钩子
  ↓
Week 4 入站通用 + benchmark
  ↓
Week 5 三件套 .dmg + IPC + setup
  ↓
Week 6 dogfood + 修 bug + GUI App 打磨
  ↓
Week 7-8 dogfood + Stripe
  ↓
Week 9 闭测启动
  ↓
Week 10 内容准备
  ↓
Week 11 KOL 接洽
  ↓
Week 12 GA 发布
  ↓
Week 13+ 慢节奏维护 + Phase 2
```

---

## Open Questions（与 PRD §14 同步）

- 正式产品名（Week 6-8 之间必须定，codename: Sieve）
- 法律实体注册地最终选择（香港 vs 新加坡 vs Stripe Atlas）
- Week 9 闭测邀请名单（5 个具体名字：builder + hunter + engineer）
- 加密收款实现方案（Stripe Crypto / Coinbase Commerce / 自部署）
- 降级模式 UI 过渡设计（试用期后只读警告但不显得被坑）

---

## 维护规则

- **每周一上午**（执行期开始后）：勾选完成项 + 写一段 Weekly Note
- **任何任务延期超 1 周**：必须在本文加 🚨 标记 + 缓解方案
- **PRD §10 修订时**：本文同步更新，时间戳标注
- **与 [tasks/lessons.md](./lessons.md) 配合**：每次 lessons 更新都应反思是否要调 roadmap
- **每周回顾**：验证关键依赖是否被阻断，风险是否上升

---

## 相关文档

- [PRD v1.4 §10](../docs/prd/sieve-prd-v1.5.md#10-12-周里程碑8-周-dogfood--4-周闭测)
- [PRD v1.4 完整版](../docs/prd/sieve-prd-v1.5.md)
- [Lessons](./lessons.md)
- [README](../README.md)
- [ADR 索引](../docs/design/ADR-INDEX.md)（待创建）
- [.cursorrules](../.cursorrules)

