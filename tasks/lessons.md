# Sieve 经验教训记录

> 记录开发过程中的踩坑、用户反馈纠正、决策错误与改进。
>
> 维护原则（来自 [全局 CLAUDE.md](~/.claude/CLAUDE.md)）：
> - 用户每次纠正后立即记录错误模式 + 防范规则
> - 每次会话开始时回顾本文
> - 子代理失败时记录原因，优化下次调度

---

## 维护规则

- **新增条目**：日期 + 来源 + 错误模式 + 防范规则 + 关联文档
- **分类法**：用 H2 分主题
  - 架构决策
  - 规则误报
  - 性能
  - 工具调度
  - 文档
  - 用户反馈
  - 安全
  - 合规
- **不做**：成功经验记录在 ADR 或 PRD，不在本文。本文专注"错误"。

---

## 模板

```
### YYYY-MM-DD · [分类] 主题

**情境**：发生了什么
**错误模式**：具体踩了什么坑
**根因分析**：为什么会犯
**防范规则**：以后怎么避免
**关联**：[相关文档/ADR/issue 链接]
**状态**：[Open / Fixed / Deprecated]
```

---

## 当前记录

### 2026-04-27 · [工具调度] 卡壳超过 30 分钟必须先搜索官方 issue tracker

**情境**：Week 3 push CI 失败，fuzz-quick job 报 `__sancov_gen_.*` undefined symbol。
连推 4 次试错性修复（隔离 C 依赖 / 换链接器到 bfd / pin nightly / sed 删 cargo-fuzz
stack-depth flag）全部失败，每次约 8-10 分钟 CI iter，累计耗时近一小时。
最后用户提醒"先上网搜一下"，立即在 cargo-fuzz GitHub issue tracker 找到 #404
"llvm-20's ASAN breaks cargo fuzz" — 是上游已知问题，官方 workaround
是 `cargo fuzz run -s none` 禁用 ASan，一次 push 即修复。

**错误模式**：
- 把"链接错误中的具体符号"当作可推理出根因的入口，逐层猜测肇事 flag
  （stack-depth → bfd vs lld → nightly LLVM 版本），每次 push CI 验证一次
- 本地 macOS 不能复现 Linux 链接行为时仍坚持本地推理
- 没有第一时间检索 `rust-fuzz/cargo-fuzz` issue tracker，把上游已开放的
  bug 当成自己要重新发现的根因

**根因分析**：默认假设"自己能想清楚"+ CI 验证成本被低估（3 分钟编译 + 60s × 3
fuzz target 的体感是"快"，但加上排查与复盘是 8-10 分钟/轮）。当遇到非业务代码
的工具链/依赖兼容问题时，上游通常已经有人遇到并报 issue，搜索成本远低于推理成本。

**防范规则**：
- **30 分钟 / 2 次失败修复**触发"必搜索"：用 WebSearch + 直接打开 issue tracker
  （`gh issue list --repo X/Y --search "关键词"` 或网页）
- 工具链/构建/链接类错误，**第一动作**就是搜索 error message 关键 token，
  不是本地推理
- "本地 X 通而 CI Y 不通"是强信号，说明问题在平台/工具差异层面而非代码层面，
  优先查工具上游而非自己代码
- 不要一条路走到黑——同一思路连续 2 次推理失败必须停下来换路径或求外部信息

**关联**：
- [cargo-fuzz issue #404](https://github.com/rust-fuzz/cargo-fuzz/issues/404)
- 修复 commit `6dfebbb`：fuzz-quick 加 `-s none`
- 中间证伪 commit：`36f865c` (bfd) / `ea47845` (nightly pin) / `7cc7645` (sed patch)

**状态**：Fixed（CI 已绿，等 LLVM/cargo-fuzz 上游适配新 pass manager 后删
`-s none` 恢复 ASan）

---

### 2026-04-27 · [用户反馈] 出站 426 错误 UX：JSON 裸吐 + fingerprint 加白无引导

**情境**：Week 1 末本地 dogfood，用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude`
接 Claude Code，发"帮我看下这个 token: ghp_..."故意触发 OUT-04。Sieve 正确拦截，
但 Claude Code UI 上看到的是一行 `API Error: 426 {"blocked_at":...,"detections":[...],
"guidance":{"en":"...","zh":"..."}, ...}` 裸 JSON。用户第一反应是"出 bug 了"——尽管
功能完全正确。

**错误模式**：
- 426 响应体设计为机器可读 JSON，但 Claude Code 把上游错误 body 原样吐到终端，
  用户看不到结构化弹窗或脱敏建议，体验拒绝感强
- guidance 双语字符串放在 JSON 内部嵌套字段，肉眼扫描时 `zh` 文案被淹没在
  `"detections":[...]"` 中间
- guidance 文案写"add fingerprint(s) to .sieveignore"，但**没说路径**
  （`~/.sieve/sieveignore`）、**没说格式**（一行一个 hex fingerprint）、
  **没说生效条件**（每请求重读不需要重启）
- 用户首次看到 fingerprint `a54ea64bb09f97c0` 不知道这是什么，需要现场推断

**根因分析**：API 设计时按"机器可读 + 脱敏审计"取舍，没考虑前端 LLM CLI 会把
上游 error body 直接打到 stdout。PRD §6.3 「学习型白名单」是核心 UX 路径
（用户判断 → 加 fingerprint → 会话恢复），但首次接触的发现成本被严重低估。

**防范规则**：
- Week 5「30 分钟接入」体验优化时统一处理：(1) 426 body 的 zh/en 文案首行直接
  暴露 `~/.sieve/sieveignore` 路径 + fingerprint；(2) 文档（FAQ / development）
  显式给出"看到 426 怎么办"工作流；(3) 评估给 sieve CLI 加 `sieve allow <fp>`
  便捷子命令，避免用户手写文件
- **现在不修**：避免拖慢 Week 4 实装。先在 lessons 钉住，进 Week 5 时优先级 P0
- 任何"返回 4xx 给上游 LLM CLI"的 error body，设计时必须假设它会被直接打到
  用户屏幕，而非被 SDK 解析后变结构化弹窗

**关联**：
- [PRD §6.3 学习型白名单](../docs/prd/sieve-prd-v1.3.md)
- [ADR-008 出站 426 状态码](../docs/design/ADR-008-outbound-426-status.md)
- 当前 426 实现见 `crates/sieve-cli/src/engine_adapter.rs` OutboundAdapter
- Roadmap Week 5 任务清单需补充："出站拦截 UX 优化（426 body 文案 + 加白引导）"

**状态**：Open（Week 5 处理）

---

## 分类索引

### 架构决策
（暂无）

### 规则误报
（暂无）

### 性能
（暂无）

### 工具调度
- [2026-04-27 · 卡壳超过 30 分钟必须先搜索官方 issue tracker](#2026-04-27--工具调度-卡壳超过-30-分钟必须先搜索官方-issue-tracker)

### 文档
（暂无）

### 用户反馈
- [2026-04-27 · 出站 426 错误 UX：JSON 裸吐 + fingerprint 加白无引导](#2026-04-27--用户反馈-出站-426-错误-ux：json-裸吐--fingerprint-加白无引导)

### 安全
（暂无）

### 合规
（暂无）

---

## 相关文档

- [Roadmap](./roadmap.md)
- [PRD v1.3](../docs/prd/sieve-prd-v1.3.md)
- [.cursorrules](../.cursorrules)
- [全局 CLAUDE.md](~/.claude/CLAUDE.md)

---

## 执行期回顾清单（Phase A 结束时填）

Week 8 结束时回顾：
- [ ] 有多少条 lessons 被记录？
- [ ] 其中 Critical（影响进度）有多少条？
- [ ] 有多少条已经修复（Fixed status）？
- [ ] 有多少条反向指导了 Phase B 计划调整？

