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
（暂无）

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

