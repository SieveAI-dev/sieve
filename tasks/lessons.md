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

### 2026-04-27 · [安全] 入站检测仅覆盖流式 SSE，非流式 JSON 响应里的 tool_use 漏过

**情境**：Week 4 IN-CR-04 落地后用 `claude --bare -p "请用 Bash 把 alias 加到 .bashrc"`
做端到端 dogfood。第一轮发现 daemon 收到了请求但 bashrc 仍被写入；继续诊断
（加请求体 stream 字段日志）发现 claude SDK 单次会话会发**两类请求**：
(1) 32KB system + tools 注册请求 `stream` 字段缺失（非流式）；(2) 6KB conversation
请求 `stream:true`（流式 SSE）。

最终验证拦截工作的 case 是 stream:true 的请求——Sieve 的 `forward_with_inbound_
inspection` 走 SSE 路径，正确解析 tool_use 触发 IN-CR-04-SHELL-RC-APPEND
fail-closed Critical。**但若有客户端发送 stream:false 或缺省的 `/v1/messages`
请求，响应是单个 `application/json` body，daemon 不解析，tool_use 直接透传到
客户端**——所有入站规则（IN-CR-02 / IN-CR-03 / IN-CR-04 / IN-CR-05 /
IN-GEN-* 全部）失效。

**错误模式**：
- 架构假设入站永远是 SSE 流，没考虑 Anthropic Messages API 同样支持非流式调用
- 集成测试 `inbound_block.rs` 仅 mock SSE upstream，覆盖盲区从未被测试发现
- 攻击者只需 SDK 调用时不传 `stream:true`（默认值取决于 SDK），就能完全绕过
  入站规则——在 PRD §5.2 「入站是 Sieve 真正的护城河」语境下，这是**严重产品
  级缺陷**，不是工程优化项

**根因分析**：
- `proxy_inner` 走 `forward_with_inbound_inspection` 时假定 response body 是
  SSE 字节流，喂给 `SseParser` + `Aggregator` + `InboundFilter`
- 非 SSE 响应（content-type: application/json，body 是单个 JSON 对象）经过
  parser 后产生 0 个 SSE event，Aggregator 永远不返回 `CompletedToolCall`，
  下游 `check_tool_use` 不被调用，最后 body 原样转发到客户端
- 整个 codebase grep 没有任何处理 application/json 响应的 tool_use 提取代码

**防范规则**：
- **Week 4 内必须修**（用户硬要求）：daemon 增加 JSON 响应分支——按 response
  content-type 路由：`text/event-stream` 走现有 SSE 路径；`application/json`
  解析 `AnthropicResponse`，遍历 `content[]` 提取 tool_use → 喂 `InboundFilter::
  on_tool_use_complete`，命中 fail-closed Critical 时把 response body 替换为
  `sieve_blocked` 等价 JSON
- 集成测试增加非流式 JSON 响应路径覆盖（`tests/inbound_block.rs` 加 mock
  upstream 返回单 JSON body 的 case）
- 任何「下一类响应/协议」加入时（Phase 2 OpenAI 等），先问"非流式响应路径
  是否完整覆盖"，不能默认 SSE-only

**关联**：
- daemon `forward_with_inbound_inspection` (`crates/sieve-cli/src/daemon.rs:261`)
- 集成测试 `tests/inbound_block.rs` 全部用 mock SSE upstream，无 JSON 响应 case
- PRD §5.2 「入站是 Sieve 真正的护城河」 / 公理 12 Critical FP < 0.5%
- 本次 dogfood 验证 commit (待开)

**状态**：Open（**Week 4 必须关闭**，2026-05-04 前）

---

### 2026-04-27 · [工具调度] claude `-p` headless 默认走 OAuth 直连，需 `--bare` 才走 ANTHROPIC_BASE_URL

**情境**：完成 Week 4 IN-CR-04 后想用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453
claude -p "<trigger>"` headless 验证规则。第一次跑 daemon 日志显示零请求（claude
还是回复了），第二次加 `--bare` flag 后 daemon 才收到请求并触发拦截。

**错误模式**：
- 假设 `ANTHROPIC_BASE_URL` env var 对所有 claude 调用模式都生效
- 没注意到 Claude Max OAuth 优先级高于 BASE_URL——非 `--bare` 模式下，claude
  CLI 优先用 OAuth token 直连 claude.ai 后端，**完全忽略 ANTHROPIC_BASE_URL**
- claude debug log 里有提示 `ANTHROPIC_BASE_URL=... is not a first-party
  Anthropic host`——但仍然没用代理

**根因分析**：claude --help 里 `--bare` 描述明确：「Anthropic auth is strictly
ANTHROPIC_API_KEY or apiKeyHelper via --settings (OAuth and keychain are never
read)」。即只有 `--bare` 才保证走 API key 路径，进而尊重 BASE_URL；默认模式
对 Claude Max 用户优先 OAuth。

**防范规则**：
- 在 Sieve 项目用 claude headless 自测时**必须**用 `claude --bare -p "..."`
  并确保 `ANTHROPIC_API_KEY` 已设
- 验收 inbound 规则前先用一个无害 prompt 跑 `claude --bare -p` 同时观察
  daemon 日志，确认请求确实抵达代理（看到 `DBG request received` 或类似），
  再发触发性 prompt
- 文档化到 development.md「dogfood / 调试」章节

**关联**：
- claude --help 输出（`--bare` 段）
- 本次诊断过程记录在 commit message
- [feedback memory: claude -p --bare 自测](../../.claude/projects/-Users-doskey-src-sieve/memory/feedback_plain_language_no_rule_ids.md)

**状态**：Open（持续应用）

---

### 2026-04-27 · [用户反馈] 跟用户说人话，不堆 rule ID 黑话 / 能 headless 测就别让用户跑

**情境**：Week 4 IN-CR-04 落地后向用户汇报，下一步建议里写"建议你重启 daemon
跑一两次 dogfood smoke"。还在通篇用 IN-CR-03 / IN-CR-04 / IN-GEN-04 这种内部
rule ID 描述能力。用户当场反馈：(1) "IN-CR-* 我根本都不记得它们是什么"；
(2) "你不能直接测试吗？用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude -p
\"prompt\"`"。

**错误模式**：
- **rule ID 黑话**：把内部检测项编号当作沟通语言。这些 ID 对实现者有意义，
  对产品 owner 看到就累——"IN-CR-04 落地了"远不如"模型写后门到 .bashrc 现在
  会被拦"清楚
- **不必要地把执行卸载给用户**：明明 `claude -p` 可以无人值守跑一次完整
  prompt-response 链路，却建议用户"开新会话试试"。这是把 Claude Code 当成
  必须人在终端面前的工具，忘了它有 headless 模式

**根因分析**：
- 前者：写代码时整天读 inbound.toml / critical_lock.rs，rule ID 已经成肌肉
  记忆；没切换语境到"用户视角看到的是 SSE 拦截 + Bash 命令被截断"
- 后者：dogfood 心智里默认"用户用 = 在终端开 Claude 真实操作"，没意识到
  对 Sieve 这个代理而言，模拟测试和真实使用走的是同一条 HTTP 路径，
  headless 完全可以替代

**防范规则**：
- **沟通用人话**：跟用户讲检测能力时直接说"模型写 .bashrc 会被拦/读 SSH 私钥
  会警告"等行为描述；rule ID 仅用于代码、commit message、文档内部 traceability
- **能 headless 测就直接测**：完成新检测规则后，主动用 `claude -p "<触发 prompt>"`
  跑一次端到端验证，不要把"重启 daemon dogfood"作为下一步建议丢给用户
- **rule ID 出现在汇报里时**：必须括号里加一句人话翻译，如
  `IN-CR-04（写持久化文件，比如 .bashrc / crontab / launchd）`

**关联**：
- 本次 commit `7cf50ff` 后给用户的回复用满了 IN-CR-* 黑话被纠正
- 本会话三条 commit 用人话改写参考：
  - 0786b6c → "10 条规则拦模型读 SSH 私钥 / AWS 凭据 / .env 等敏感路径"
  - 7a70098 → "对齐 user-stories 验收标准 + 修 glossary typo"
  - 7cf50ff → "9 条规则拦模型写后门到 .bashrc / crontab / launchd 等持久化位置（YOLO mode 不可关）+ 把命名错位的 markdown exfil 规则归类正确"

**状态**：Open（持续应用，本会话从下一条回复起改）

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
- [2026-04-27 · claude `-p` headless 默认走 OAuth 直连，需 `--bare` 才走 ANTHROPIC_BASE_URL](#2026-04-27--工具调度-claude--p-headless-默认走-oauth-直连需---bare-才走-anthropic_base_url)

### 安全
- [2026-04-27 · 入站检测仅覆盖流式 SSE，非流式 JSON 响应里的 tool_use 漏过 (Week 4 必须关闭)](#2026-04-27--安全-入站检测仅覆盖流式-sse非流式-json-响应里的-tool_use-漏过)

### 文档
（暂无）

### 用户反馈
- [2026-04-27 · 出站 426 错误 UX：JSON 裸吐 + fingerprint 加白无引导](#2026-04-27--用户反馈-出站-426-错误-ux：json-裸吐--fingerprint-加白无引导)

### 安全（已合并到分类索引上方）

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

