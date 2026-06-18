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

**状态**：✅ 已修复（v1.5.4，2026-05-01，commit 14153e2）

**修复实现**（详见 [CHANGELOG v1.5.4](../docs/changelog/CHANGELOG.md#v154-non-streaming-json-inbound-fix---2026-05-01)）：
- `daemon.rs::forward_with_inbound_inspection` 加 Content-Type 路由
- 新增 `handle_anthropic_json_inbound` 解析 `content[]` → tool_use → InboundFilter
- 新增 `handle_openai_json_inbound` 解析 `choices[].message.tool_calls[]`
- 新增 `build_sieve_blocked_json_body` 构造拦截响应（保持 PRD §9 #11 协议层诚实）

**子代理顺手发现的第二个隐蔽 bug**：`proxy_openai` 的 `stream=false` 分支原本直接 `forward_raw` 完全跳过入站检测——OpenAI 协议默认就是 stream=false，意味着 OpenAI 入站规则**从未生效过**。本 patch 一并修复。

**测试验证**：新增 2 条集成测试 `anthropic_non_streaming_json_inbound_block` + `openai_non_streaming_json_inbound_block`，inbound_block 14/14 通过；dataset_fp_rate FP 0% / Recall 99.71% 不变（无回归）。

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
- [PRD §6.3 学习型白名单](../docs/prd/_archive/sieve-prd-v1.3.md)
- [ADR-008 出站 426 状态码](../docs/design/ADR-008-outbound-426-status.md)
- 当前 426 实现见 `crates/sieve-cli/src/engine_adapter.rs` OutboundAdapter
- Roadmap Week 5 任务清单需补充："出站拦截 UX 优化（426 body 文案 + 加白引导）"

**状态**：Open（Week 5 处理）

---

### 2026-04-28 · [工具调度] 大型架构翻转单次"按规格实现"无法收口，必须接受多轮 review + 边界止损

**触发**：v1.4 同步——把 v1.3 的一维处置矩阵翻转为二维 + IPC + 双层防御 + GUI 独立仓库 + sieve setup 工具。派 5 文档 + 6 代码子代理并行实现，cargo fmt/clippy/test 全绿，但 codex review 三轮抓出 21 个问题：

| 轮次 | 发现 | 修 | 残留 |
|------|------|------|------|
| R1 | 9 | 9 | 0 |
| R2 | 6（修 R1 引入）| 6 | 0 |
| R3 | 6（修 R2 暴露）| 0（止损）| 6 |

**错误模式**：
1. **"绿灯不等于正确"**：测试断言验旧行为，新行为没人写测试 → 关键回归（如 disposition 路由短路、GUI hold 自连自）通过测试但实际功能挂掉
2. **"修 bug 暴露下层"**：每轮修都让上层问题暴露下层缺口；R3 的 6 个问题（如 setup 不部署规则文件 / IN-CR-01 disposition 不生效 / RedactAndAllow 漏脱敏）都是 R2 修 disposition 路由后才"显现"——它们一直存在，只是之前路由短路把它们盖住了
3. **"端到端依赖未落地组件"**：v1.4 的 GUI App 在独立仓库 sieve-gui-macos 还没建，所有 GUI 路径只能 mock，mock 又引入更多假设——真正修复必须等 GUI 端落地

**防范规则**：
1. **大型架构翻转必须用 codex review（或同等独立审查）三轮以上**——单轮不够，要做"修 R1 → review R2 → 修 R2 → review R3"循环直到收口
2. **每轮 codex review 后立即评估"是否在无限循环"**：如果新发现 ≥ 上一轮修复数 ≈ 30%，止损归档已知问题，等依赖组件就位
3. **test 绿不是"可以 commit"的充分条件**——还要看 codex review 的"实质功能验证"。每个 P1 问题都要至少跑通一次 mock 端到端
4. **架构骨架值得保留**：止损不等于回滚。doskey 已经在 `tasks/known-issues-v1.4.md` 登记 6 个待修问题 + 触发回归条件（GUI MVP / .dmg 打包 / 任何用户 dogfood / 下次 v1.4 增量 review）
5. **派子代理时给"四要素"还不够**：codex 抓的问题大半是"子代理按规格实现但规格本身不完整"——以后写 spec 时要预演 fail-closed 路径的所有触发场景（IO 失败 / 文件损坏 / GUI 不存在 / 解析失败），spec 不能只描述 happy path
6. **flaky test 当 P0 修**：sieve-ipc 环境变量并发竞态测试在 R1 修中两次"附带修复"都没修干净，第三次才用全局 mutex 真正搞定。flaky test 是 review 信号噪声的最大源头

**关联**：
- [tasks/known-issues-v1.4.md](./known-issues-v1.4.md) 6 个待修问题登记 + 回归触发条件
- 三轮 codex review log: `/tmp/codex-review-v1.4.log` / `-r2.log` / `-r3.log`
- [tasks/todo.md](./todo.md) v1.4 同步执行计划
- 回滚基线：git tag `pre-v1.4-refactor` (commit 743e681)

**状态**：Open（等 GUI App 落地或下次 review 触发）

---

### 2026-06-11 · [架构决策] 跨仓 wire 契约的 v1→v2 重命名只改一端，且 fixture 防漂移测试名存实亡，漂移隐藏数月

**情境**：补落地 SPEC-005 §14.2「GUI 消费 daemon 权威 fixture 副本」的测试（`IPCSchemaV2FixtureTests`）后，**第一次跑就抓出 preset mode 跨仓漂移**：SPEC-005 §5.6 把 v1 preset `"default"` 重命名为 `"standard"`、要求「daemon 与各端同步替换」，但只有 GUI 改了（`Preset` enum 用 `standard`），daemon 侧（`config::Preset::Default` variant + `daemon_control_plane` String 字面量 + setup 模板）从未替换——daemon 一直推 `"default"` → GUI 解码 `Preset` enum 失败 → disconnected。这直接卡死真机 dogfood（GUI 连不上 daemon），却因无跨仓一致性测试隐藏数月。

**错误模式**：
- SPEC 写了「必须同步替换」的契约，但只有写 SPEC 的人和改 GUI 的人执行，daemon 漏改、无机器强制
- SPEC §14 要求的防漂移测试从未真正落地：`schema_v2_fixtures.rs` 全文只单向反序列化、从未实现 §14.1 双向稳定，daemon fixture 缺 `listeners[]` 数月无人发现；GUI §14.2 的 `Fixtures/v2` + 测试根本不存在，用内联手写 JSON 测解码（「名存实亡」，2026-06-07 审查原话）
- 内联手写 JSON 给「已覆盖」的假象，但手写 JSON 与 daemon 真实输出可以各写各的、永不对账

**根因分析**：
- 跨仓契约变更（重命名/加字段）天然有「N 端同步」成本，缺机器强制时必漏某端
- §14.1「fixture 是序列化事实」只有做双向稳定（`to_value == fixture`）才成立；只做单向反序列化（能 parse 就算过）根本不校验 fixture 是否等于真实输出
- 覆盖率错觉：有「health 解码测试」≠ 有「health 与 daemon 输出一致测试」

**防范规则**：
- **跨仓 wire 契约必须有「一端权威产物 + 另一端消费同一产物」的对账测试**：daemon 权威 fixture 做双向稳定（序列化输出 == fixture），对端消费同一份 fixture 副本解码——两侧共用同一 JSON = 无漂移空间。禁止两端各写各的 fixture/内联 JSON
- **SPEC 规定的测试机制要验证「真落地」而非「有同名文件」**：§14.1 双向稳定缺失数月就因没人检查 `schema_v2_fixtures.rs` 是否真做了 `to_value` 比较
- **enum wire 值改名 = breaking change，必须全仓 grep 所有端**：config enum variant、运行时 String 字面量、setup 模板、文档枚举列表、对端 enum——一处不漏，改完跑跨仓一致性测试
- **`format!("{:?}", enum).to_lowercase()` 生成 wire 值是脆性反模式**（`daemon.rs:773`）：wire 值应走 serde 序列化，不依赖 Debug = variant 名的巧合。记为后续技术债

**关联**：
- 修复 commit：sieve `ae20fd3`（preset default→standard）/ `2c51c96`（fixture 双向稳定）/ GUI `b3075b0`（IPCSchemaV2FixtureTests 消费副本）
- SPEC-005 §5.6（preset_mode v2 重命名）/ §14.1–14.2（fixture 防漂移）
- [2026-06-07 工程审查](../docs/review/2026-06-07-workflow-suite-state-review.md) §三/§四（fixture 防漂移名存实亡 + 跨仓契约漂移）

**状态**：Fixed（daemon preset 改 standard + 兼容 `alias="default"`；双向稳定 + GUI 消费副本测试已落地；`format!("{:?}")` 脆性记为后续技术债）

---

### 2026-06-18 · [安全] smoke_test.py 出站断言滞后于 disposition 改动（OUT-01 426→auto_redact），无 key 不跑遂隐藏

**情境**：做 dogfood 自动化、给 `scripts/smoke_test.py` 加 `--mock-only`（本地 mock 上游去真 API 依赖）时，`test_outbound_block_fake_key` 断言「OUT-01 fake key → HTTP 426 + sieve_blocked」实跑得到 401。
**错误模式**：OUT-01 的 disposition 早已从 `block` 改为 `auto_redact`（ADR-016 二维处置矩阵 + PRD v1.4 §6.1 fix #2：disposition 优先于 action=block）——daemon 现在**脱敏后转发**而非 426 拦截。Rust 集成测试 `outbound_block.rs::fake_anthropic_key_auto_redacted_and_forwarded` 当时同步改对了，但 Python smoke 测试没改，断言变成错的。
**根因分析**：smoke 测试默认要真 `ANTHROPIC_API_KEY` + 真网络才跑全套，进 dogfood 冻结期后基本没人拿真 key 跑过，错误断言静默存活数月——和 [跨仓 fixture 漂移](#2026-06-11--架构决策跨仓-wire-契约的-v1v2-重命名只改一端且-fixture-防漂移测试名存实亡漂移隐藏数月) 同一根因模式：**没法自动跑的验证 = 必然漂移的验证**。
**防范规则**：① 任何处置等级/disposition 变化必须同时改 Rust 集成测试 **和** Python smoke 测试（CLAUDE.md 文档触发表已要求，扩展到测试断言）；② 测试断言不得依赖「真 key + 真网络」做唯一执行条件——必须有 hermetic mock 路径让 CI 每次都跑（本次 `--mock-only` 已落地）；③ 出站脱敏类规则的 smoke 断言应验「转发到上游的 body 已脱敏」（mock 捕获 body 比对原文），而非验状态码。
**关联**：`scripts/smoke_test.py`（`test_outbound_redact_fake_key`）/ `crates/sieve-rules/rules/outbound.toml` OUT-01 / ADR-016 / PRD v1.4 §6.1 / dogfood 自动化 Epic（PROGRESS.md P0.3）
**状态**：Fixed（smoke 断言改为 auto_redact + mock body 脱敏校验，`--mock-only` 29/29 通过）

### 2026-06-18 · [安全] sieve-updater ZSTD_MAGIC 字节序反置，真实规则包永不解压（dogfood 自动化抓出）

**情境**：写 updater 闭环 hermetic e2e（`tests/updater_e2e.rs`，§14.4），断言「安装的文件内容 = 解压后的规则 JSON」时失败——磁盘上是 zstd **压缩**字节而非解压内容。
**错误模式**：`install.rs::ZSTD_MAGIC = [0xFD,0x2F,0xB5,0x28]` 字节序反了。zstd 帧 magic 是 `0xFD2FB528`，磁盘小端存储为 `28 B5 2F FD`（RFC 8878 §3.1.1）。`data[..4] == ZSTD_MAGIC` 对**任何真 zstd 流永远为假** → `decompress_zstd` 永远走「当原始字节」fallback → 规则包被原样（压缩态）写盘 → sieve-rules 加载必失败。**整个 ADR-030 规则热更新功能在生产中是坏的**。
**根因分析**：① 常量写错没人核对真实 magic；② 现有 4 个 zstd 相关单测全部假阳性掩盖——要么用明文走 fallback、要么用**同一个错误常量**当 payload 前缀（自洽于 bug）、要么只断言「文件存在 / 版本号」从不断言**解压后内容**。「测试通过」≠「功能正确」。
**防范规则**：① 涉及编解码/序列化的代码，单测必须断言**端到端内容回环**（encode→install→read 出来 == 原始），不能只查文件存在；② 魔数/协议常量必须对照规范原文核对字节序，最好用 hex/注释标明来源（RFC/版本）；③ 不要用「被测代码自己的常量」构造测试输入——会和 bug 自洽；④ 这类 bug 唯有真实 payload 端到端 e2e 能抓，印证 dogfood 自动化的价值。
**关联**：`crates/sieve-updater/src/install.rs`（ZSTD_MAGIC + happy_path 回环断言）/ `tests/updater_e2e.rs` / ADR-030 / SPEC-006 §3.3 / CHANGELOG
**状态**：Fixed（常量改 `[0x28,0xB5,0x2F,0xFD]` + happy_path 补解压内容断言 + e2e 闭环；updater 39 单测 + 8 e2e 全绿）

### 2026-06-18 · [安全] headless dogfood 自动化首轮抓出 4 类 daemon bug（含 2 个 P0）+ 6 类跨仓 schema 漂移

**情境**：落地 headless dogfood e2e（`dogfood_e2e.rs` 11 测试）+ 跨仓 fixture 一致性测试（GUI 仓 81 fixture）后，**首轮就批量抓出真 bug**——印证「没法自动跑的验证 = 必然漂移/腐烂的验证」。
**抓出的 bug**：
- **P0-A `sieve audit` / `sieve decisions` CLI 完全跑不起来**：`main()` 是 `#[tokio::main]`，而两命令的 sync `run()` 内又 `Builder::new_current_thread().block_on()` → "Cannot start a runtime from within a runtime" panic（exit 134）。**headless「CLI 当决策客户端」策略的命门**。单测只测内层 async `run_async`、绕过 sync wrapper，故隐藏。**已修**（`run` 改 async 委托，`main` 直接 `.await`）。
- **P0-B detection 审计全未接线**：`OutboundRedacted` / `DecisionMade` / `InboundDecisionResolved` / `InboundHookMarked` / `StatusBarNotified` 等 AuditEvent 变体只在 audit.rs **测试模块**构造，daemon 热路径**零 append 调用**——出/入站/决策结果全不写 audit。`sieve audit query` 即便修好也查不到核心流量。**未修（大产品改动，需热路径全面接 audit）**，Phase D-1 测试已锚定。
- **C/D 跨仓 wire schema 漂移 ×6（多个致命）**：`sieve.hello` preset:"default"（GUI 无此 enum）/ `preset_changed` 发 `mode` 缺 `preset` / `paused_changed` 缺必填 `source` / `notify_status_bar` 整体字段错位 / `purge_history` purged_at 类型(ms数 vs ISO串) / `evaluate.would_recommendation` 类型。与 2026-06-11 preset 漂移**同根因**（只改一端），但漂移点更多更深。**未修（需定 canonical 端 + 跨仓改）**，GUI fixture 测试用 `#[expect(throws:)]` 钉死现状当红线。
- 小：harness `with_no_client_policy` 误写 toml（daemon 只从 CLI flag 读 + `deny_unknown_fields`）→ 启动失败（**已修**：改传 CLI flag）；`wait_for_ipc` 探测连接残留污染 `connected_clients`（daemon lazy 清理 gui_writers，未修，harness 已注明无 client 路径勿调）。
**根因分析**：这些路径从未被自动化跑过——headless CLI、跨仓协议、审计落库全靠「人会去手动验」，进 dogfood 冻结期后无人验 → 腐烂。和 [zstd 字节序](#2026-06-18--安全sieve-updater-zstd_magic-字节序反置真实规则包永不解压dogfood-自动化抓出) / [跨仓 fixture 漂移](#2026-06-11--架构决策跨仓-wire-契约的-v1v2-重命名只改一端且-fixture-防漂移测试名存实亡漂移隐藏数月) 同一模式。
**防范规则**：① 任何「打算让用户手动验」的路径都要先问「能不能自动验」——能就别留给人（全局 CLAUDE.md 已有，强化）；② CLI 入口（sync wrapper 包 async）必须有「实跑子进程」的集成测试，不能只测内层 async；③ 跨仓 wire 契约必须有双向 fixture 一致性测试入两仓 CI，任一端漂移即 block PR；④ 审计/落库类副作用必须有「真触发→查得到」的 e2e，不能只测 schema/构造。
**关联**：`crates/sieve-cli/src/commands/{audit,decisions}.rs` + `main.rs`（P0-A 修复）/ `crates/sieve-cli/tests/dogfood_e2e.rs`（Phase D 正向断言 P0-B）/ GUI 仓 `IPCSchemaV2FixtureTests.swift`（C/D 翻转）/ `crates/sieve-testing/src/daemon.rs`（harness 修复）
**状态**：Fixed（2026-06-18 同日全修）。P0-A 嵌套 runtime + harness 已修；**P0-B 审计接线**：`gated_request_decision` 写 `DecisionMade`（所有 gui_popup 决策 + no-client-policy）、出站脱敏（Anthropic+OpenAI）写 `OutboundRedacted`，`sieve audit query` 现查得到核心流量（窄路径 hook-mark/status-bar/direct-block 暂留，当前规则集不触发 direct-block）；**6 类 schema 漂移全修**：daemon 侧 D4(paused_changed 补 source)/D6(purge_history purged_at i64→ISO)，GUI 侧 D3(删 preset)/D5(重写 EventNotifyParams)/D7(would_recommendation→对象)，D1/D2 陈旧 fixture 校正，两仓 fixture 字节对齐 + 断言翻转。

### 2026-06-18 · [工具调度] Explore agent 把「陈旧 fixture」误当「live 协议漂移」，差点改坏本来通的协议

**情境**：修 6 类跨仓 schema 漂移时，两个 Explore agent 都基于「daemon fixture = daemon live 输出」下结论，建议把 daemon `hello.preset` 等多处 `String`→`Preset enum`、改 wire 输出。
**错误模式**：daemon `hello`/`set_preset`/`preset_changed` 的 preset/mode 用 `format!("{:?}", cfg.preset).to_lowercase()` —— **Debug 变体名**（`Standard`→"standard"），**永远不发 "default"**。fixture 里的 `"default"` 是 2026-06-11 改名前的**陈旧静态测试数据**，daemon 侧字段是 `String` 收任意值故反序列化通过、掩盖了陈旧。若按 agent 建议改 daemon，会把**本来 live 已通的 hello 协议改坏**。真正要改的 daemon 代码只有 D4(补 source)/D6(purged_at 类型)——agent 误判要改 6 处。
**根因分析**：① Explore agent 读 fixture 当 ground truth，没核对「struct serde 属性 + 构造点实际值」推断 live 序列化；② 跨仓 fixture 测试只单向反序列化、不双向稳定（serialize 回比），陈旧 fixture 静默存活。
**防范规则**：① 跨仓 wire 修复，canonical 以 **SPEC + 真实 struct serde 序列化**为准，不以 fixture 为准（fixture 可能陈旧）；② 区分「陈旧 fixture」（仅测试数据坏，live 通）vs「live 漂移」（struct 真发了不兼容形态）——前者只改 fixture，后者改代码，**误判方向会改坏好协议**；③ fixture 防漂移测试应做**双向稳定**（serialize↔deserialize 比对），单向反序列化挡不住陈旧；④ agent 的精密结论必须主上下文用真实代码核验，尤其涉及改 wire 协议。
**关联**：`crates/sieve-cli/src/daemon.rs`（preset Debug 序列化）/ `config.rs` Preset enum / SPEC-005 §5.6/§10.2/§11B / `crates/sieve-ipc/tests/schema_v2_fixtures.rs`（建议补双向稳定）
**状态**：Fixed（按 SPEC + live 序列化判定 canonical，仅改真偏离的 D4/D6 + 陈旧 fixture）

## 分类索引

### 架构决策
- [2026-06-11 · 跨仓 wire 契约 v1→v2 重命名只改一端，fixture 防漂移名存实亡](#2026-06-11--架构决策-跨仓-wire-契约的-v1v2-重命名只改一端且-fixture-防漂移测试名存实亡漂移隐藏数月)

### 规则误报
（暂无）

### 性能
（暂无）

### 工具调度
- [2026-04-27 · 卡壳超过 30 分钟必须先搜索官方 issue tracker](#2026-04-27--工具调度-卡壳超过-30-分钟必须先搜索官方-issue-tracker)
- [2026-04-27 · claude `-p` headless 默认走 OAuth 直连，需 `--bare` 才走 ANTHROPIC_BASE_URL](#2026-04-27--工具调度-claude--p-headless-默认走-oauth-直连需---bare-才走-anthropic_base_url)
- [2026-04-28 · 大型架构翻转单次"按规格实现"无法收口，必须接受多轮 review + 边界止损](#2026-04-28--工具调度-大型架构翻转单次按规格实现无法收口必须接受多轮-review--边界止损)

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
- [PRD v1.3](../docs/prd/_archive/sieve-prd-v1.3.md)
- [.cursorrules](../.cursorrules)
- [全局 CLAUDE.md](~/.claude/CLAUDE.md)

---

## 执行期回顾清单（Phase A 结束时填）

Week 8 结束时回顾：
- [ ] 有多少条 lessons 被记录？
- [ ] 其中 Critical（影响进度）有多少条？
- [ ] 有多少条已经修复（Fixed status）？
- [ ] 有多少条反向指导了 Phase B 计划调整？

