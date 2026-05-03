# Sieve daemon · 进度

> 上次更新：2026-05-03
> 当前阶段：**SPEC-005 v2 代码全部就绪 — 等用户手动联调**

## 当前阶段一句话

SPEC-005 v2.0 frozen + v2.0+ 兼容扩展（list_rules / purge_history）双侧契约全部实现并对齐；
P0 5/5 ✅ · P1 10/10 ✅ · P2 5/6 ✅（P2 仅余 fixtures polish）+ 善后全部 ✅；
e2e 集成测试 harness 6 场景，cargo workspace 696 passed 全绿。
GUI 仓库（sieve-gui-macos）同步完成 swift test 127 passed + xcodebuild SUCCEEDED。
代码侧任务清零，等用户手动联调反馈。

---

## ✅ 主里程碑

### 2026-05-03 v2.0+ 兼容扩展 + 业务层完整化
- SPEC-005 §11A sieve.list_rules + §11B sieve.purge_history（不 bump version）
- daemon 实现两个新 method
- recommendation 字段 daemon 业务层真实注入
- fixtures 81 条（17+2 method × 3 档）

### 2026-05-03 e2e 测试 + 业务层 polish
- 端到端集成测试 harness（6 场景：握手 / heartbeat / 单 issue / merged / 重连丢 inflight / set_paused 串行化）
- pre-existing flake canary_token_hits_out01 修复
- audit oversize callback 注入

### 2026-05-02..03 P1-5 wire DTO 拆分（最大改造）
- 内部 DecisionRequest 与 wire DTO 分离
- 单 issue 平铺 / 多 issue merged + issues[]
- created_at → received_at_daemon
- origin_request_id 真实透传

### 2026-05-02 P0 全部 + P1 大部分
- 帧读取 FrameReader + memchr（移除无界 BufReader::lines）
- sieve.hello + sieve.heartbeat + socket 0600
- 字段对齐：paused_until / origin_request_id / HealthResult 拆分 / NotifyKind / parse_error / fan-out 串行化 / write timeout / pending-leak

### 2026-05-02 协议骨架（4 组双侧同步对）
- 协议版本号 v1 → v2
- 方法名 sieve.* 前缀
- 错误码段位 -32100~99
- decision_response.result required 字段

详细 commit 列表见 git log。

---

## 🚧 进行中
（无 — 代码侧任务清零）

---

## ⏭ 下一步（等用户联调反馈）

### 用户介入项
- 真实 dogfood：启 daemon + GUI 跑 HIPS / Settings / History 流程
- 反馈 bug 或 UX 调整

### 已知小尾巴（不阻塞联调）
- direction 字段在 sieve-core/pipeline 某孤立路径未被完整测试覆盖
- list_rules daemon 端从 LayeredEngine 取规则的实际列表完整性需联调验证
- purge_history daemon 端实际 SQLite delete events 行为需联调验证

### 发布前（Phase 1C，等联调通过后）
- Tier 1 sigstore reproducible build 跑通 release artifact 流程
- GA 准备：Week 12 一次性公开 repo

---

## 🚫 阻塞 / 等决策
（无）

---

## 完成定义（DoD，每项任务通用）

- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 全过
- 涉及 SSE / 规则 / 工具调用判定的改动 → 对应 fuzz / 单元测试已加
- PRD §9 十六条硬约束未被绕过
- CHANGELOG 已更新（依赖升级 / 行为变更 / 检测项 ID 变化必记）
- 关联文档（requirements / design / api / SPEC）已同步
- **本文件已勾选 + 移项至「已完成」**

详见 `.cursorrules §五` + 项目根 `CLAUDE.md`。
