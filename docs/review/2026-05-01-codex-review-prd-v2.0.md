# Codex Review: Sieve PRD v2.0

> Reviewer: codex (gpt-5.5)
> 日期: 2026-05-01
> review 对象: docs/prd/sieve-prd-v2.0.md

## TL;DR

v1.5 -> v2.0 做 HIPS（Host-based Intrusion Prevention System，主机入侵防御系统）升级有商业价值，不是纯 over-engineering；价值来自用户规则、决策记忆和进程审计。问题是 PRD 把产品差异化、工程抽象、闭测实验和营销话术混成同一个 GA（General Availability，正式发布）承诺，风险被低估。建议 Phase A 做最小可交付，Phase B 行为序列降为 beta，不作为 GA 必达卖点。

## A. 战略层评估

**A1. HIPS 升级有价值，但不要按 CrowdStrike 级别承诺。**  
v2.0 最有价值的是用户规则、三态决策和进程上下文，能支持 $49/月的“本地最后一道闸”叙事。但 §2 对齐 Falcon / Defender ATP / OSSEC，会抬高用户预期。建议对外改成“LLM agent 场景的本地 HIPS MVP”，避免用户期待通用端点防护。

**A2. Phase A/B 拆分方向对，但 GA 风险仍低估。**  
§0 说 Phase A 不动 daemon 核心 pipeline，因此风险低；但 Week 5-7 同时做 policy、引擎抽象、灰名单、进程反查、规则编辑器、multi-agent 收尾，这不是低风险。v1.5.4 刚证明“平行入站路径”容易漏扫。建议 Phase A 作为 GA-blocking，Phase B 默认 beta。

**A3. 与硬约束 #1-#15 大体一致，但灰名单有硬冲突。**  
#14 可以成立，前提是用户规则失败只跳过用户层，系统规则失败仍 fail-closed。#15 只 StatusBar，符合 FP（False Positive，误报）红线。最大问题是 §5.4 允许 GUI “永久允许此次场景”，而 GUI 类包含 IN-CR-05 签名等 Critical；这等价于关闭 Critical，违反 PRD §9 #3/#8。必须改成：内置 Critical 只允许单次 allow，不允许永久 remember。

## B. 范围层评估

**B1. 瘦身后仍偏大，而且正文未同步。**  
§5.5.2 说只 ship 4 个 `sieve rules` 子命令，Week 5 仍写 8 个；§5.5.5 说 TUI（Text User Interface，终端文本界面）4 项能力，Week 6 又写匹配预览 / 正则解释器 / 示例库；§12 说新增 3 条风险，表里只有 2 条；changelog 仍写已撤回内容。先修一致性，否则范围会回弹。

**B2. 五项 HIPS 改造的 GA 优先级。**

- 用户规则：GA 可以做，但语义改成“用户自定义 High Ask / Warn / Mark”，不能升级为内置 Critical。
- 三态决策：GA 必须做，但 remember 只给非 Critical 或用户规则。
- 进程上下文：GA 做 caller_pid / caller_exe，cwd / ppid 推 v2.1。
- 规则引擎抽象：GA 只做最小 LayeredEngine；bench CI 和复杂 metrics 推后。
- 行为序列：推 v2.1，或 Week 9-12 仅 beta flag，不做 GA 阻塞项。

**B3. 规则编辑器 TUI MVP 仍过度。**  
“语法高亮 + schema 提示 + 实时 lint + 保存”已经是一个小产品。v2.0 更合理的是 `sieve rules edit` 打开 `$EDITOR`，保存后 lint、备份、原子替换、reload；ratatui 放 v2.1。

## C. 技术层评估

**C1. §5.5 用户规则安全约束不够保守。**  
现有 5 类禁止组合覆盖明显 override，但漏了过宽 allowlist、regex 资源耗尽、`user.toml` symlink / hardlink、灰名单伪造、reload 时序问题。建议补规则数量、pattern 长度、文件大小、编译耗时上限；要求 keywords；禁止 match-all；规则文件 0600、atomic rename、no-follow symlink；用户规则永远不能 suppress 系统 Critical。

**C2. §5.7 N=10 / TTL=5min 可以作为默认，但数据结构不够。**  
N=10 / TTL（Time To Live，存活时间）=5 分钟作为 MVP 合理。但 `ToolUseRecord` 只存 input hash 和 rule_hits，无法识别未命中单次规则的 `Read(.env)`、`curl POST`、`rm`。建议存隐私安全的结构化特征：tool_class、path_category、network_egress、persistence、cleanup、sensitive_file。

**C3. §6.3 LayeredEngine trait 不够通用。**  
`scan(&[u8]) -> Vec<MatchHit>` 对纯 vectorscan 足够，但 v2.0 需要 direction、protocol、content_kind、tool_name、source_agent、caller_exe。建议改为 `scan(ScanRequest) -> ScanReport`。LayeredEngine 必须定义合并顺序：系统 Critical 先执行，用户规则只能追加或触发 Ask。

**C4. §5.4 灰名单 schema 不完整。**  
缺 fingerprint 规范、schema_version、rule_version、protocol、tool_name、source_agent、caller_exe、撤销路径、审计事件和权限要求。建议 v2.0 只允许非 Critical 永久 remember；灰名单写 audit.db；权限 0600；文件名用 hex digest。

## D. 风险层评估

**D1. R-V20-01/02 不完整。**  
建议补：R-V20-03 灰名单绕过 Critical；R-V20-04 content-type 路由回归导致 JSON / stream=false 漏扫；R-V20-05 用户 regex / TOML 资源耗尽；R-V20-06 PID（Process Identifier，进程标识符）反查错误；R-V20-07 Phase A 过载；R-V20-08 行为序列串会话。每条绑定测试或降级策略。

**D2. §14 Open Questions 哪些应立即回答。**  
OQ-V20-01 现在就定：IPC notify + 原子 reload，失败保留旧 user engine。OQ-V20-02 定为 macOS 系统 API，不 shell out `lsof -i`。OQ-V20-06 定为 GA 不做 GUI 编辑器，TUI 降级为 `$EDITOR` + lint。OQ-V20-07 已推 v2.1，不应继续 open。OQ-V20-03 可保留；OQ-V20-04/05 移到 Deferred。

## E. 与 v1.5.4 的一致性评估

**E1. 文档提到正确函数名，但没有把 content-type 路由变成不变量。**  
§6.2 写了 `forward_with_inbound_inspection` 和 `forward_with_openai_inbound_*`，方向正确；但 §5.5 用户规则、§5.4 灰名单、§5.7 序列窗口都没有明确要求同时覆盖 `text/event-stream` 和 `application/json`。v1.5.4 的教训是：新功能只要挂在 SSE parser 后，JSON 和 OpenAI `stream=false` 就会绕过。

**E2. 新功能可能重新引入 v1.5.4 漏洞模式。**  
最高风险三类：用户规则只接 SSE scan path，JSON 不跑 LayeredEngine；行为序列只在 SSE 聚合器更新，JSON helper 不更新序列；灰名单 fingerprint 对 SSE delta 和 JSON full body 计算不同。建议 §9 增加硬约束：所有入站能力必须经过 content-type 路由矩阵测试。

**E3. 必补测试矩阵。**  
每个新增能力至少覆盖 Anthropic / OpenAI、SSE / JSON、系统规则 / 用户规则。额外覆盖 OpenAI `stream=false` + AutoRedact。行为序列若保留，必须验证 JSON tool_use/tool_calls 进入同一窗口。

## 综合结论与建议

### 必须修改（Must）

- §5.4：禁止内置 Critical 永久 Remember；Critical 只允许单次 allow/deny。
- §5.5：澄清用户规则语义，允许 High Ask/Warn/Mark，但不能 override 或 suppress 系统 Critical。
- §5.5.3：补资源上限、文件权限、atomic reload、symlink 和宽泛 allowlist 防护。
- §6.3：把 `scan(&[u8])` 改成带上下文的 `ScanRequest`，定义 LayeredEngine 合并顺序。
- §5.7：ToolUseRecord 改存结构化安全特征。
- §9/§10：加入 content-type 路由回归测试矩阵。
- §10：删掉 Week 5/6 仍残留的 8 个子命令、匹配预览、正则解释器、示例库。
- §12：补灰名单绕过、路由回归、资源耗尽、进程归因错误、范围过载。
- §15 changelog：清理已撤回内容。

### 强烈建议（Should）

- Phase B 行为序列改为 beta / dogfood flag，不作为 Week 12 GA 承诺。
- `sieve rules edit` GA 采用 `$EDITOR` + lint + backup + reload，不做完整 ratatui 编辑器。
- 进程上下文 Phase A 只做 caller_pid / caller_exe。
- OQ-V20-01/02/06/07 立即关闭。
- HIPS 对外话术改成“LLM agent HIPS MVP”。
- 灰名单文件名改为 digest，内容带 schema_version / rule_version / fingerprint_version。
- 所有用户规则和灰名单变更写 audit.db。

### 可以考虑（Could）

- v2.1 再加匹配预览、正则解释器、示例库。
- v2.1 再做 GUI 规则编辑器。
- 行为序列样本足够后，再评估 High -> Block 升级 ADR。
- 用户规则分享 / import/export 推到付费高级版讨论。

### 明确不要做（Won't）

- v2.0 不做拦截引擎 trait / sieve-interceptor crate。
- v2.0 不做 AI 辅助生成规则。
- v2.0 不做规则市场或社区规则中心。
- v2.0 不允许任何用户配置关闭、永久绕过或降级内置 Critical。
