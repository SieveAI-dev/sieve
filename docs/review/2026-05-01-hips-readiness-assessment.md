# Sieve HIPS Readiness Assessment

> 评估日期：2026-05-01
> 评估对象：v1.5.3 拦截引擎 + 规则引擎 + 测试方案
> 评估目的：判断当前实现与经典 HIPS（Host-based Intrusion Prevention System）标准的契合度，给后续改造提供 baseline
> 评估方法：对照成熟 HIPS（OSSEC / CrowdStrike Falcon / Microsoft Defender ATP / Kaspersky HIPS）的 14 项核心特征逐条比对

---

## TL;DR

**Sieve 当前是 HIPS 70%。**核心 HIPS 特征（fail-closed / 完全本地 / 实时拦截 / 不可绕过 / 审计完整 / 双层防御）已具备，但**还差 4 项关键能力**才算合格 HIPS：

1. **行为序列分析**（HIPS 看进程动作链，Sieve 现在只看单次工具调用）
2. **可编程 policy 引擎**（HIPS 用户写规则，Sieve 现在是项目内置 TOML）
3. **三态决策 + 灰名单**（HIPS 有"询问/学习"中间态，Sieve 只有 allow/block）
4. **进程上下文关联**（HIPS 知道是 chrome.exe 还是 ssh-agent 调的，Sieve 不知道调用方身份是哪个 agent 进程）

测试方案对**规则正确性**覆盖很好（1951 样本 / 0% FP / 99.71% recall / 4 个 fuzz target），但对**拦截引擎行为正确性**覆盖偏弱——双层防御的真实 hook 拦截链路（Claude Code → settings.json → sieve-hook → IPC pending → 用户 y/n）没有 e2e 测试，全部是 mock unit test。

---

## 1. HIPS 14 项核心特征对照表

| # | HIPS 标准特征 | Sieve 实现 | 评级 | 备注 |
|---|---|---|---|---|
| 1 | **完全本地，无云端验证** | ✅ PRD §9 #2 硬约束 | 满足 | 业内顶级 |
| 2 | **fail-closed 默认拒绝** | ✅ ADR-007 + critical_lock.rs | 满足 | YOLO mode 不可关 |
| 3 | **多层防御** | ⚠️ ADR-014 双层（SSE + hook）但同质化 | 部分 | 都是 pattern 层，缺行为层 |
| 4 | **实时拦截**（动作发生前阻止）| ✅ PreToolUse hook + GUI hold | 满足 | 业内顶级 |
| 5 | **三态决策**（allow / deny / **ask**）| ⚠️ 二态（GUI popup + hook 终端 y/n 算 ask 的弱版本）| 部分 | 缺持久化"灰名单" |
| 6 | **学习/适应** | ⚠️ `.sieveignore` 只学不调（不调 severity / 不动 pattern）| 部分 | 缺自适应 |
| 7 | **审计完整不可篡改** | ✅ SQLite append-only + BEFORE UPDATE/DELETE 触发器 | 满足 | data-model.md §审计 |
| 8 | **不可绕过**（root/admin 也不能关 Critical）| ✅ critical_lock.rs `FAIL_CLOSED_RULES` 编译期常量 | 满足 | YOLO mode 也覆盖 |
| 9 | **可编程 policy 引擎** | ❌ 内置 TOML，用户不能加规则 | **不满足** | 仅项目维护者改 |
| 10 | **沙箱/隔离** | ❌ 无（Phase 2 待定）| 不满足 | HIPS 经典差异点 |
| 11 | **行为序列关联**（看动作链，不只单次）| ⚠️ InboundFilter 有 SessionState 但仅做地址表 + tool_use 聚合 | 部分 | 不看跨工具/跨会话序列 |
| 12 | **签名验证调用方** | ✅ X-Sieve-Origin Ed25519（ADR-019）| 满足 | sub-agent 嵌套防伪造 |
| 13 | **实时通知 + 用户决策** | ✅ GUI hold 120s + hook 终端 y/n 30s | 满足 | 双通道齐备 |
| 14 | **状态持久化** | ✅ SQLite + `.sieveignore` + IPC pending/decisions 文件 | 满足 | data-model.md §3 |

**满足 8 项 / 部分 4 项 / 不满足 2 项 = 70%。**

---

## 2. 拦截引擎评估（pipeline + IPC + hook）

### 2.1 架构概览

```
SSE 流（Anthropic / OpenAI 协议）
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ daemon.rs::forward_with_inbound_inspection (Anthropic)  │
│ daemon.rs::forward_with_openai_inbound_inspection (OAI) │
└─────────────────────────────────────────────────────────┘
    │
    ▼  text/event-stream 路径（已实现）
┌──────────────────────────────────────┐
│ SseParser → ToolUseAggregator        │
│   → InboundFilter::on_tool_use_complete│
│   → engine_adapter::scan_text         │
│   → vectorscan + BIP39 second-pass    │
└──────────────────────────────────────┘
    │
    ▼ Detection.disposition 路由（dispatch_impl）
    ├── AutoRedact   → outbound_redact 改写 body
    ├── HookTerminal → inbound_hook 写 ~/.sieve/pending/<id>.json + SSE 透传
    │                  └→ Claude Code PreToolUse → sieve-hook 终端 y/n
    ├── GuiPopup     → inbound_hold hold SSE 流 + IPC notify GUI + keep-alive
    │                  └→ GUI App 弹窗 120s → decision_response → 继续/截流
    └── StatusBar    → 通知不打断
    
    ▼ application/json 路径（❌ 当前漏洞，P0 修复中）
    所有入站规则全部绕过！v1.5.x 70 条规则在此路径全失效
```

### 2.2 优势

- **协议层诚实**（PRD §9 #11）：截流允许、注入 `sieve_blocked` event 允许，**禁止伪造 model 字段**——这是产品承诺级别的约束，比多数 HIPS 还严
- **双层防御**（ADR-014）：Hook 类规则在 PreToolUse 边界拦截（不污染 Claude Code 上下文），GUI 类规则在 SSE 传输阶段 hold（保留完整可视化 context），分场景使用合适的拦截点
- **Fail-closed 全路径**：超时 / 进程崩溃 / IPC 失败 / GUI 失联 全有 fail-closed 路径（ADR-014 §4 表格）
- **Critical 不可绕过**：`FAIL_CLOSED_RULES` 是编译期常量，YOLO mode 也覆盖，运行时检查
- **SessionState 跨调用**：InboundFilter 维护会话状态（地址表 / 多 tool_use 聚合），不是无状态的单次匹配

### 2.3 不足

- ✅ **P0 漏洞已修**（v1.5.4，2026-05-01 commit `14153e2`）：非流式 `application/json` 响应里的 tool_use 此前完全绕过入站规则；同时修复隐蔽 bug：OpenAI `stream=false` 分支跳过入站检测（OpenAI 协议默认 stream=false，意味着 OpenAI 入站规则**从未生效过**）。详见 [SECURITY.md 历史 Advisories](../../SECURITY.md#历史-advisories)
- ⚠️ **进程上下文丢失**：Sieve 当前不知道这次 tool_use 是哪个调用链发出的（X-Sieve-Origin 只到 sub-agent 层级，没到操作系统进程层级）。HIPS 级别需要知道"是 chrome.exe 还是 ssh-agent 调的"
- ⚠️ **行为序列不分析**：InboundFilter 有 SessionState 但只做地址表 + tool_use 聚合，**不分析跨工具序列**。如"模型先 cat .env → 再 curl POST → 再 rm -rf 痕迹"这种典型 kill chain，每步都看似无害，但序列是攻击。Sieve 当前每步独立判断
- ⚠️ **OpenAI 路径与 Anthropic 路径不对称**：两条几乎平行的代码路径（`forward_with_inbound_inspection` vs `forward_with_openai_inbound_inspection`），任何新检测要双倍维护成本
- ⚠️ **沙箱缺失**：可疑命令没有"先在沙箱跑一遍看效果"的机制；HIPS 经典做法（Cuckoo / dynamic analysis）Phase 1 完全没做

---

## 3. 规则引擎评估（vectorscan + allowlist + critical_lock）

### 3.1 架构概览

```
┌─────────────────────────────────────────┐
│ inbound.toml / outbound.toml (TOML)     │
│   - 70 入站规则 + 11 出站规则            │
│   - 每条带 severity / disposition /      │
│     keywords / allowlist_regexes /       │
│     allowlist_stopwords (v1.5.1 升级)    │
└─────────────────────────────────────────┘
    │ loader.rs 解析
    ▼
┌─────────────────────────────────────────┐
│ VectorscanEngine::compile()             │
│   - BlockDatabase 多模式正则编译         │
│   - SOM_LEFTMOST flag 精确 start offset  │
│   - HashMap<u32, RuleEntry> 元信息查询   │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ scan(input) → Vec<MatchHit>             │
│ is_excluded(matched, full_context, rule)│
│   - placeholder 黑名单                   │
│   - allowlist_regexes (匹配 candidate)   │
│   - allowlist_stopwords (全文搜索) ← v1.5.1 │
└─────────────────────────────────────────┘
    │
    ▼ 二阶段（vectorscan 之外）
┌─────────────────────────────────────────┐
│ BIP39 second-pass (engine_adapter.rs)   │
│   - candidate_bip39_windows + checksum  │
│ Address Guard (sieve-cli)               │
│   - strsim Levenshtein 距离             │
│ Skill Install Guard (IN-CR-06)          │
└─────────────────────────────────────────┘
```

### 3.2 优势

- **vectorscan 多模式同时匹配**：业内最快的开源多模式正则引擎，70 条规则 P99 < 1ms
- **二维处置矩阵**（ADR-016）：`disposition` × `severity` × `default_on_timeout` 三元组让规则元数据驱动行为，不需要在 daemon 写硬编码 if-else
- **allowlist_stopwords 全文搜索**（v1.5.1 核心升级）：让短命中能识别教学/合法语境（`eval $` / `rm -rf /` / `systemctl enable`），是 FP 从 5.7% 降到 0% 的关键
- **BIP39 second-pass**：vectorscan 不适合 2048 词 alternation，单独走 SHA-256 checksum 验证，是 PRD §9 #4 的差异化点
- **placeholder 黑名单**：`YOUR_API_KEY` / `xxx` / `0x0...0` 全局过滤，对抗示例代码 FP
- **TOML schema 向后兼容**：新加 `disposition` / `timeout_seconds` 字段时旧规则不报错（`#[serde(default)]`）

### 3.3 不足

- ❌ **不可编程**：用户不能加自定义规则。HIPS 用户写规则是基本能力（OSSEC `<rule>` XML / Falcon CrowdStrike CQL / Defender XDR Custom Detection KQL），Sieve 只能等项目方加
- ⚠️ **vectorscan PCRE 子集限制**：禁 lookahead / lookbehind / 反向引用 / 原子组，导致很多"正向匹配 X 但排除 Y"的复杂逻辑只能拆成多条规则 + allowlist。比如 `IN-CR-02-MALICIOUS-REGISTRY` 想匹配"非官方 registry"只能用 allowlist 豁免 npmjs.org / pypi.org
- ⚠️ **规则与代码耦合**：`critical_lock.rs::FAIL_CLOSED_RULES` 是编译期常量，加新 Critical 规则必须同时改这个常量；`HOOK_RULES` / `GUI_RULES` 同样硬编码。HIPS 应该让 disposition 字段单一来源（rule.disposition），而不是规则文件 + Rust 常量两边维护
- ⚠️ **无规则版本化 / 热更新**：规则编译进二进制（`embedded_rules.rs`），改规则要重发版本。HIPS 标准做法是规则签名 + 在线/离线更新（CrowdStrike 每天 N 次 channel file 推送）
- ⚠️ **无规则灰度 / A/B**：新规则上线没有"先 5% 用户跑 7 天看 FP"机制；vectorscan 编译失败也会让整个 daemon 起不来（一损俱损）
- ⚠️ **regex 缺乏语义**：`grep ANTHROPIC_API_KEY` 是 IN-CR-03-GREP-CREDS-A 命中，但 Sieve 不知道这是"开发者在 debug 自己的项目"还是"恶意脚本扫家目录"——HIPS 要看上下文（调用方进程、cwd、之前 5 个动作）

---

## 4. 测试方案评估

### 4.1 测试矩阵

| 类别 | 数量 | 文件 | 评级 |
|------|------|------|------|
| 单元测试 | 164 个 `#[test]` | crates/{sieve-cli,sieve-rules,sieve-core}/tests/, src/**/tests | ✅ 充分 |
| 数据集回归 | 1951 样本 / 5 桶 | bench-data/{attacks,attacks-by-fear,benign,benign-near,attacks-public-replay} | ✅ 业内顶级 |
| FP/Recall 阈值 assertion | 0% FP / 99.71% recall | dataset_fp_rate.rs | ✅ |
| 公开攻击复现 | 55 条带 URL | attacks-public-replay/ × 6 子目录 | ✅ 营销价值 |
| Fuzz target | 4 个 | fuzz/fuzz_targets/ (inbound_filter / sse_parser / sse_parser_openai / tool_use_aggregator) | ⚠️ 偏少 |
| 集成测试 | inbound_block 54K + outbound_block 49K + multi_agent_setup 53K + doctor 34K | crates/sieve-cli/tests/ | ✅ 大 |
| E2E（真实链路） | 仅 proxy_passthrough 17K | crates/sieve-cli/tests/proxy_passthrough.rs | ❌ 不够 |

### 4.2 优势

- **数据集分桶设计**：按"看起来像攻击但合法"（benign-near/）+ "用户最怕五件事"（attacks-by-fear/）双向对称分桶，FP 高时定位规则盲区，recall 漏拦时定位生成"假攻击"
- **多语言/多格式覆盖**：每桶 10% 多语言（中/日/韩/西/法/德）+ 10% 格式变种（Markdown / JSON tool_use / SSE delta / Dockerfile）
- **公开攻击复现**：55 条全部带可追溯 URL，Sieve 是少数有"已知 CVE 复现拦截率"的产品
- **SSE 边界 fuzz**：cargo-fuzz + AFL++ 双引擎覆盖（PRD §9 #5），半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流五类边界
- **Append-only 审计测试**：BEFORE UPDATE/DELETE 触发器单测覆盖，保证 audit.db 不可篡改

### 4.3 不足

- 🔴 **缺真正的 E2E 测试**：双层防御的真实拦截链路（`Claude Code → settings.json hook → sieve-hook 进程 → IPC pending → 用户 y/n → exit code → Claude Code 拒绝执行`）**完全没有 e2e 测试**——这条路径任何环节坏了都意味着 Hook 类规则全部失效，但当前 CI 不验证它
- 🔴 **缺非流式 JSON 路径覆盖**（P0 修复中正在补）
- ⚠️ **Fuzz target 偏少**：4 个 target 只覆盖 SSE parser + tool_use aggregator + inbound filter；BIP39 second-pass、address guard、规则编译、IPC 协议 都没 fuzz
- ⚠️ **缺 sigstore 链路测试**：reproducible build 是 PRD §9 #6 硬约束，但 CI 上的"两次构建 SHA-256 比对"是手动跑（development.md §4.2）
- ⚠️ **缺真实流量回放测试**：当前数据集是合成 + 公开复现，没有"录一周真实开发者用 Claude Code 流量回放给 Sieve 看 FP"的机制（B 方案，等真实用户上线）
- ⚠️ **缺多 agent 嵌套调用 fuzz**：multi_agent_setup.rs 有大量集成测试（53K），但是配置注入测试，不测**运行时 sub-agent X-Sieve-Origin 链 fuzz**（恶意 agent 伪造 chain_depth 攻击 Sieve 自己的场景）
- ⚠️ **缺性能回归 baseline**：`cargo bench` 在 development.md 列出但没在 CI 强制（PRD 要 P99 < 20ms 是 CI gate，但当前不强制）

---

## 5. 改造建议（向 HIPS 演进）

### 5.1 短期（Week 5-6，不破坏 GA 节奏）

**P0 必做**：
1. ~~关 P0 漏洞：非流式 JSON 入站检测~~（✅ 2026-05-01 v1.5.4 完成，含 OpenAI stream=false 隐蔽 bug 一并修复）
2. **加 e2e 测试**：双层防御链路（Claude Code → hook → sieve-hook → IPC → 用户决策）端到端 1 条 happy path + 1 条 fail-closed
3. **Fuzz 扩展**：BIP39 second-pass + address guard + IPC 协议各加 1 个 fuzz target
4. **disposition 单一来源**：把 `critical_lock.rs::HOOK_RULES` / `GUI_RULES` 移除，改从 `rule.effective_disposition()` 单一来源拿——这是 ADR-014 + ADR-016 的设计意图，当前是历史遗留

**P1 推荐**：
5. **进程上下文记录**：在审计日志里记 `caller_pid` / `caller_exe` / `caller_cwd`（HIPS 必备）
6. **简单行为序列**：InboundFilter 加"过去 N 次 tool_use 序列"窗口，标注序列里有没有 cat/curl/rm 经典 kill chain（做 Mark 不 Block，先收数据）

### 5.2 中期（Week 7-12 GA）

7. **三态决策**：在 GUI hold 弹窗加"允许这次 / 永久允许 / 拒绝"三选项，"永久允许"写到 `.sieveignore` 灰名单（HIPS 标准做法）
8. **可编程规则**：暴露 `~/.sieve/custom-rules.toml`，用户可加规则但不能动 Critical 内置（这是 HIPS 与 Sieve 当前最大差距）
9. **规则签名 + 热更新**：内置规则用 sigstore 签名，daemon 启动时验证；新增 `sieve update-rules` 命令拉远端签名规则包（PRD 没禁这个，本地 verify 不算"联网做 verifier"）
10. **真实流量回放**：dogfood 录的脱敏流量做 baseline，CI 上每周跑一次 vs 历史数据看是否有新 FP

### 5.3 长期（Phase 2，触发条件后）

11. **沙箱执行**：可疑 Bash 命令先在 firejail / Docker 沙箱跑一遍记录效果，再决定真执行（HIPS 经典差异点）
12. **行为序列 ML 分类器**：满 4 周 High FP > 5% + 真实付费用户 ≥ 10 反馈才启动（Phase 2 已在 roadmap）
13. **跨平台 HIPS 标准对接**：MITRE ATT&CK Framework 映射、SIGMA 规则导入（让 SOC 团队可用）
14. **Network Extension MITM**（已在 PRD §9 #12 否决，长期可重新评估）

---

## 6. 改造优先级矩阵

| 紧迫度 ↓ \ 收益 → | 高收益 | 中收益 | 低收益 |
|------|------|------|------|
| **🔴 立即** | ~~P0 漏洞~~（✅ v1.5.4 关闭）/ e2e 测试 / disposition 单一来源 | 进程上下文记录 | — |
| **🟠 Week 5-6** | 行为序列窗口 / Fuzz 扩展 | 三态决策 | — |
| **🟡 GA 前** | 可编程规则 / 规则签名 + 热更新 | 真实流量回放 | — |
| **🟢 Phase 2** | 沙箱执行 / 行为序列 ML | MITRE ATT&CK 映射 | SIGMA 导入 |

---

## 7. 改造时不应该破坏的承诺

如果用户要把项目改造成 HIPS，**以下 7 条 PRD §9 硬约束 + ADR 决策不能动**：

1. **Rust 栈非选项**（PRD §9 #1）：HIPS 改造时仍然不能引入非 Rust 二进制依赖到 hot path
2. **绝不联网做 verifier**（PRD §9 #2）：规则签名验证 / 沙箱判断 / ML 推理全部本地
3. **fail-closed Critical 不可关**（PRD §9 #3 / ADR-007）：HIPS 用户加规则可以放宽自己的，但内置 Critical 不允许 override
4. **不在协议层撒谎**（PRD §9 #11 / ADR-014）：HIPS 拦截通知用 `sieve_blocked` event 自报，不冒充 model 字段
5. **不装本地 CA 做 MITM**（PRD §9 #12）：HIPS 改造时仍然走 reverse proxy 模式，不动 Network Extension（除非 Phase 3）
6. **本地优先，不上传任何用户数据**（PRD §11）：行为序列 / 进程上下文 / 沙箱日志全留本地
7. **审计 append-only 不可篡改**（ADR-007）：HIPS 加新审计字段可以，但 SQLite 触发器拒绝 UPDATE/DELETE 不能改

---

## 8. 结论

**Sieve 当前是 HIPS 70%**，比"LLM 流量代理"定位高一档（多数 LLM 安全产品在 30-50%）。

距离合格 HIPS 还差 4 项关键能力：**行为序列分析 / 可编程 policy / 三态决策 + 灰名单 / 进程上下文关联**。这 4 项里，前 3 项可以在 Week 5-12 GA 前补完（不破坏 PRD 硬约束），第 4 项跨平台进程信息抓取在 Phase 2 做更合理。

**测试方案对规则正确性是业内顶级**（1951 样本 / 5 桶 / 0% FP / 99.71% recall + 公开攻击复现 + 4 fuzz target），但**对拦截引擎链路正确性偏弱**（双层防御 e2e 链路完全没测，真实 Claude Code → hook → sieve-hook → IPC 链路靠开发者手动 dogfood 验证）。这是 HIPS 改造的第一个该补的洞，比加新规则更重要。

~~**P0 漏洞（非流式 JSON 入站绕过）必须先关**，否则 v1.5.x 的 70 条规则在常见生产路径上等于纸面数字。~~ → ✅ **v1.5.4 已关闭**（2026-05-01）。

---

## 附录：关键代码路径与行号索引

| 模块 | 文件 | 关键 API |
|------|------|---------|
| 入站 pipeline | `crates/sieve-core/src/pipeline/inbound.rs` | `InboundFilter::on_tool_use_complete`, `SessionState` |
| 双层 dispatch | `crates/sieve-core/src/pipeline/mod.rs` | `dispatch_impl::dispatch` |
| Hook 类拦截 | `crates/sieve-core/src/pipeline/inbound_hook.rs` | 写 IPC pending file |
| GUI 类拦截 | `crates/sieve-core/src/pipeline/inbound_hold.rs` | hold SSE + 25s keep-alive |
| 出站脱敏 | `crates/sieve-core/src/pipeline/outbound_redact.rs` | `redact_body_bytes` |
| 规则引擎 | `crates/sieve-rules/src/engine/mod.rs` | `VectorscanEngine`, `is_excluded` (v1.5.1 全文搜索) |
| 规则元数据 | `crates/sieve-rules/src/manifest.rs` | `RuleEntry`, `Disposition`, `Severity`, `Action` |
| Critical 锁 | `crates/sieve-rules/src/critical_lock.rs` | `FAIL_CLOSED_RULES`, `HOOK_RULES`, `GUI_RULES` |
| BIP39 引擎 | `crates/sieve-rules/src/bip39.rs` | `candidate_bip39_windows`, `verify_checksum` |
| Daemon 入口 | `crates/sieve-cli/src/daemon.rs:1451` | `forward_with_inbound_inspection` |
| OpenAI 入口 | `crates/sieve-cli/src/daemon.rs:1784` | `forward_with_openai_inbound_inspection` |
| Engine 适配 | `crates/sieve-cli/src/engine_adapter.rs` | `InboundAdapter`, `OutboundAdapter` |
| 审计 | `crates/sieve-cli/src/audit.rs` | `AuditEvent`, SQLite append-only |

**关键 ADR**：
- [ADR-007](../design/ADR-007-fail-closed-critical-actions.md)：fail-closed 原则
- [ADR-014](../design/ADR-014-dual-layer-defense.md)：双层防御（SSE + Hook）
- [ADR-016](../design/ADR-016-disposition-matrix-2d.md)：二维处置矩阵
