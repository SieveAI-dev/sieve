# Changelog

本文件记录 Sieve 所有显著变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

> 当前状态：**Phase A dogfood 进行中**（Week 3 入站 Crypto 钩子完成）。第一个公开版本（v0.1.0）将随 Week 12 GA 发布。
> 第一个公开版本（v0.1.0）将随 Week 12 GA 发布；详见 [PRD §10 12 周里程碑](../prd/sieve-prd-v1.3.md#10-12-周里程碑8-周-dogfood--4-周闭测)。
> v1 公开 API 在 Week 12 GA 后冻结，破坏性变更走 SemVer。冻结范围参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)。

---

## [v1.5.4-non-streaming-json-inbound-fix] - 2026-05-01

### 背景

Week 4 dogfood 实测发现的 P0 安全漏洞，标记"必须 Week 4 关闭"，至今未关。子代理修复时**顺手发现第二个隐蔽 bug**：OpenAI 路径 `stream=false` 分支原本直接 `forward_raw` 完全跳过入站检测——OpenAI 默认就是 stream=false，意味着 OpenAI 入站检测从来没生效过。

### Fixed [SECURITY]

- 🔴 **P0 漏洞 1（Anthropic 路径）**：daemon 当前只扫 `text/event-stream` SSE 流，`application/json` 非流式响应里的 `tool_use` **整体绕过所有入站规则**（IN-CR-02/03/04/05 / IN-GEN-* 全失效，含 v1.5.x 70 条新规则）。修复：`forward_with_inbound_inspection` 在收上游响应后按 Content-Type 路由，JSON 路径走新增 `handle_anthropic_json_inbound()` 解析 `content[]` → 提取 tool_use → 喂 `InboundFilter::on_tool_use_complete` → Critical 命中替换 body 为 `sieve_blocked` JSON

- 🔴 **P0 漏洞 2（OpenAI 路径，子代理顺手发现）**：`proxy_openai` 的 `stream=false` 分支原本直接 `forward_raw`，**跳过整个入站检测路径**——OpenAI 协议默认就是 stream=false，意味着 OpenAI 入站规则从来没生效过。AutoRedact 的 stream=false 分支同样问题。修复：改为调用 `forward_with_openai_inbound_inspection` 让 Content-Type 路由处理；新增 `handle_openai_json_inbound()` 解析 `choices[].message.tool_calls[]`

### Added

- 4 个新 helper 函数（`crates/sieve-cli/src/daemon.rs`）：
  - `handle_anthropic_json_inbound`：收 body → 解析 `AnthropicResponse.content[]` → tool_use 提取 → InboundFilter
  - `handle_openai_json_inbound`：同上但解析 OpenAI ChatCompletion 格式
  - `build_sieve_blocked_json_body` / `build_sieve_blocked_json_response`：构造非 SSE 格式的拦截响应（保持 PRD §9 #11 协议层诚实，body 内嵌 `sieve_blocked` 字段不冒充 model）
  - `passthrough_json_response`：未命中时原样转发（重建 headers）

- 2 条新集成测试（`crates/sieve-cli/tests/inbound_block.rs`）：
  - `anthropic_non_streaming_json_inbound_block`：mock 上游返回 `application/json` + `eth_signTransaction` tool_use → 验证 daemon 替换 body
  - `openai_non_streaming_json_inbound_block`：同款 OpenAI Chat Completions 格式 + `tool_calls` 数组
  - 新增 `spawn_mock_json_upstream` 测试 helper

### Changed

- `proxy_openai` `stream=false` 分支：`forward_raw` → `forward_with_openai_inbound_inspection`
- AutoRedact `stream=false` 同款修复

### 测试结果

| 指标 | 修复前 | 修复后 |
|------|------|------|
| Anthropic 非流式 JSON tool_use 拦截 | ❌ 完全绕过 | ✅ block |
| OpenAI 非流式 tool_calls 拦截 | ❌ 完全绕过（含 stream=false 默认场景）| ✅ block |
| inbound_block 集成测试 | 12 passed | **14 passed**（+2 新增）|
| dataset_fp_rate | 0% FP / 99.71% recall | **不变**（无回归）|
| cargo fmt --check / clippy -D warnings | pass | pass |

### Impact 评估

**这是 v1.5.x 系列发现的最严重漏洞。**Anthropic SDK 和 OpenAI SDK 的常见配置中，**非流式响应使用率 50%+**——这意味着 v1.5.x 累计加的 70 条入站规则在这些场景**实际拦截率为 0%**。本 patch 把"纸面 70 条规则"恢复到"实际 70 条规则"。

PRD §9 #7 实测数字（FP 0% / Recall 99.71%）此前**仅在 SSE 流模式有效**，本 patch 后才在所有响应模式生效。

---

## [v1.5.3-windows-powershell-pipe] - 2026-05-01

### 背景

v1.5.2 公开攻击复现报告里发现 CVE-2025-6514 Windows PowerShell 变种漏拦（mcp-001），已标记为"建议 Week 6 补"。本次 patch 直接补上，1 条规则一行一行测，10 分钟工作量。

### Added

- **`IN-CR-02-CURL-PIPE-WIN`**：`(?i)(curl|iwr|invoke-webrequest)(?:\s+\S+)+\s*\|\s*(powershell|pwsh)(\.exe)?` —— 覆盖 Windows `curl ... | powershell` / `iwr ... | pwsh` 形态。CVSS 9.6 / 437K+ 下载量影响（CVE-2025-6514 mcp-remote OS Command Injection RCE）。Critical + hook_terminal + 30s timeout default block。allowlist 7 条教学/审计豁免

### 测试结果

| 指标 | v1.5.2 终点 | v1.5.3 | 状态 |
|------|------|------|------|
| Critical FP | 0/1070 = 0% | **0/1070 = 0%** | ✅ |
| Attack Recall（合成） | 694/696 = 99.71% | **694/696 = 99.71%** | ✅（无回归） |
| Public Attack Replay | 51/55 = 92.7% | **52/55 = 94.5%** | ✅ |
| mcp-supply-chain 子桶 | 8/9 = 88.9% | **9/9 = 100%** | ✅ |

### Known limitations

剩 3 条公开复现漏拦全部为**接受盲区**（不计划修）：
- `owasp-003.txt`（OWASP LLM03 RAG 投毒）：无 payload 特征，需源头数据校验
- `real-003.txt` / `real-006.txt`：真实事件中的前端 UI 钓鱼 / 社工邮件，不在 LLM 流量范围

入站规则总数 69 → **70**。

---

## [v1.5.2-blind-spots-and-public-replay] - 2026-05-01

### 背景

v1.5.1 完成后剩 20 条 attack 漏拦记录在 `tasks/2026-05-01-rule-gaps.md`，本次清光（剩 2 条不可能 vectorscan 解决，已说明）。同时做方案 C：复现 55 条**公开发生过的真实攻击**作为信任基线 —— 合成数据再多也不如"已知 CVE / 已黑客攻击事件"有说服力，这是营销文章的最强弹药。

### Added

- **BIP39 入站 second-pass**（IN-CR-03-BIP39-INBOUND）：复用 outbound 已有的 `candidate_bip39_windows` + `verify_checksum`，在 `engine_adapter.rs::InboundAdapter::scan_text` 加同款代码块。命中 → Critical + `gui_popup` 60s 弹窗（模型诱导用户输入助记词的场景）。`__BIP39_SECOND_PASS_PLACEHOLDER__` 占位规则跳过 vectorscan 编译（参考 IN-CR-01 写法）

- **12 条新入站规则**（inbound.toml 57 → 69）：
  - **6 条边缘形态**：IN-CR-03-ENV-STANDALONE（`env\n`）、IN-CR-02-CURL-MULTILINE-SECRET（curl + `$VAR` 跨行）、IN-CR-02-PYTHON-OSPOPEN-CURL、IN-CR-02-NPM-PACKAGE-REDIRECT（package.json `npm:@attacker/pkg`）、IN-CR-02-NPM-POSTINSTALL-AUTO、IN-CR-03-PRINTENV pattern 修正（去掉末尾 `\b`）
  - **6 条 BIP39 社工/标签规则**（覆盖 checksum 不通过但有社工话术的场景）：IN-CR-03-BIP39-ENTER-PHRASE、IN-CR-03-BIP39-SEED-VAR、IN-CR-03-BIP39-COLON-LABEL、IN-CR-03-BIP39-TOOL-CMD、IN-CR-03-BIP39-MULTILINGUAL（High）、IN-CR-03-LEDGER-SEED-VERIFY

- **公开攻击复现数据集 55 条**（`bench-data/attacks-public-replay/`，每条带可追溯 URL）：
  - `rugpull-ai/` × 9：KuCoin AI Agent / ElizaOS arXiv 2503.16248 / Unit 42 IDPI / Step Finance $40M
  - `injection-pocs/` × 10：Oasis "Claudy Day" / Lasso Claude Code / PromptArmor / Docker MCP / Aikido PromptPwnd
  - `ctf-replays/` × 8：Awesome Prompt Injection / Open-Prompt-Injection / PromptMap / arXiv HTML AT bypass
  - `owasp-llm-top10/` × 9：LLM01-09 各一条 crypto 场景复现
  - `real-events/` × 10：Ledger 2023-12-14 / Multichain $126M / Lazarus Op99 / Sep 2025 NPM 攻击 / Step Finance
  - `mcp-supply-chain/` × 9：CVE-2025-6514（CVSS 9.6）/ SmartLoader / Oura 2026 / Anthropic MCP 设计缺陷

- **新独立测试**：`public_replay_recall_rate`（dataset_fp_rate.rs）扫 attacks-public-replay/ 输出按子目录命中率，不硬性 assert（揭露盲区目的）

- 任务级文档：`tasks/2026-05-01-public-attack-replay-report.md`（完整漏拦分析 + 营销引语）

### Changed

- 总入站规则数 35 → 57 → **69 条**
- 测试数据集 296 → 1896 → **1951 条**（含公开复现 55）

### 测试结果

| 指标 | v1.5.1 终点 | v1.5.2 终点 | 阈值 | 状态 |
|------|------|------|------|------|
| Critical FP | 0/1070 = 0% | **0/1070 = 0%** | < 0.5% | ✅ |
| Attack Recall（合成数据集） | 676/696 = 97.13% | **694/696 = 99.71%** | > 95% | ✅ |
| Public Attack Replay（v1.5.2 新增） | — | **51/55 = 92.7%** | 揭露盲区，无硬阈值 | — |

按桶（合成数据集）：env-leak 100% / shell-rce 100% / signing 100% / private-key 98.33% / 现有 attacks 100% 无回归

按子目录（公开复现）：rugpull-ai 100% / injection-pocs 100% / ctf-replays 100% / owasp-llm-top10 88.9% / real-events 80% / mcp-supply-chain 88.9%

### Known limitations

- **2 条合成数据集仍漏拦**（vectorscan 能力边界，本 sprint 不修）：
  - `fear-privkey-087`：仅"请分享 WIF key"社工话术，无 pattern 特征（FP 风险太高）
  - `fear-privkey-096`：通过 `chrome.storage.local.get` JS API 读 MetaMask `KeyringController.vault`，需 JS 语义分析

- **4 条公开复现漏拦**：
  - 3 条接受盲区（RAG 投毒无 payload 特征 / 纯社工邮件不在 LLM 流量内）
  - **1 条建议下次补**：CVE-2025-6514 Windows `curl ... | powershell` 变种 → 加 `IN-CR-02-CURL-PIPE-WIN` 规则

### 营销可用引语（来自 public-attack-replay-report.md）

- **Ledger 2023 $600K Connect Kit 事件**："前端库已被供应链污染但 Sieve 的 IN-CR-05-ERC2612-PERMIT 在签名前弹窗确认" —— 来源：ledger.com/blog/security-incident-report
- **CVE-2025-6514（437K+ 下载量恶意 MCP）**："JSON 里藏 curl|bash 拿下开发机，Sieve 在执行前拦截" —— 来源：jfrog.com/blog/2025-6514
- **Lazarus Operation 99（Web3 开发者定向攻击）**："恶意编码挑战的 printenv / 读 Solana keypair / Python urlopen 三类操作分别被三条 IN-CR-03 规则覆盖" —— 来源：thehackernews.com/2025/01/lazarus-group-targets-web3-developers

---

## [v1.5.1-rule-expansion] - 2026-05-01

### 背景

PRD §9 #7 规定 Critical FP < 0.5%、Recall > 95% 是硬约束。Week 4 之前数据集只有 226 attacks + 70 benign，跑出 0% FP / 100% recall —— 数字漂亮但样本太少，对付费用户"一周不误拦"信心不够。本次扩充把 baseline 拉到付费用户决策级别。

### Added

- **测试数据集 296 → 1896 条**（+1600）：
  - `bench-data/attacks-by-fear/{signing,transfer,env-leak,private-key,shell-rce}/` × 120 = 600 条新攻击样本，按"用户最怕的五件事"组织（营销可直接引用）
  - `bench-data/benign-near/{near-OUT-api-keys, near-OUT-tokens, near-OUT-private-keys, near-IN-CR-01-address, near-IN-CR-02-rce, near-IN-CR-03-secret-read, near-IN-CR-04-persistence, near-IN-CR-05-crypto-addr, near-IN-CR-06-misc, extra-generic-multilingual}/` × 100 = 1000 条新 benign 样本，按"看起来像攻击但完全合法"对称分桶（教学/文档/Dockerfile/多语言）
  - 内容多样性预算：60% 真实 web3/dev 文档风格 + 20% 变体 + 10% 多语言（中/日/韩/西/法/德）+ 10% 格式变种（Markdown / JSON tool_use / SSE delta / Dockerfile）

- **22 条新入站规则**（`crates/sieve-rules/rules/inbound.toml`，35 → 57 条）：
  - **env-leak 桶**：IN-CR-03-GREP-CREDS-A/B（grep 扫敏感字段）、IN-CR-03-PRINTENV-CREDS（printenv/env dump）、IN-CR-03-DOCKER-ENV-DUMP（docker exec env）、IN-CR-03-GH-SECRET-LIST（CI secret 枚举）、IN-CR-03-CURL-EXFIL（外发 .env / API key）
  - **private-key 桶**：IN-CR-03-KEYCHAIN-FIND-PASS（macOS Keychain `-w` 导出）、IN-CR-03-METAMASK-VAULT（浏览器扩展 storage 路径）、IN-CR-03-WIN-DPAPI（Windows DPAPI 导出）、IN-CR-03-BITCOIN-DUMPPRIVKEY（`bitcoin-cli dumpprivkey`）、IN-CR-03-GPG-DIR 扩展（含 keystore 目录枚举）
  - **shell-rce 桶**：IN-CR-02-WGET-EXEC（`wget -O /tmp/x.sh && sh`）、IN-CR-02-PYTHON-RCE（`python -c "exec(__import__(...).read())"`）、IN-CR-02-BASE64-PIPE-SH（`base64 -d <<< ... | sh`）、IN-CR-02-MALICIOUS-REGISTRY（npm/pip 非官方 registry）；CURL-PIPE / WGET-PIPE / EVAL pattern 扩展支持 `sudo` 可选前缀
  - **signing 桶**：IN-CR-05-ERC2612-PERMIT（`permit(spender, deadline)` 无限授权签名）、IN-CR-05-WALLETCONNECT-URI（`wc:UUID@1|2` deeplink 签名劫持）

- **bench-data 测试递归读取 + 按桶聚合**（`crates/sieve-rules/tests/dataset_fp_rate.rs`）：
  - `read_samples_recursive` helper 支持子目录扫描
  - 按"桶"输出 per-bucket FP/recall 报告（FP 高时一眼定位是哪类合法场景误伤、recall 漏拦时定位规则盲区）
  - assertion 阈值升级：benign 至少 500 样本（原 50）、attacks 至少 500（原 200）

- 任务级文档：`tasks/2026-05-01-test-data-expansion.md`（计划）、`tasks/2026-05-01-test-data-expansion-report.md`（结果报告）、`tasks/2026-05-01-rule-gaps.md`（已知规则盲区清单，含 BIP39 入站 second-pass 等待后续 sprint）

### Changed

- **规则引擎 `is_excluded` 签名变更**（`crates/sieve-rules/src/engine/mod.rs`，`crates/sieve-cli/src/engine_adapter.rs`，`crates/sieve-rules/tests/inbound_rules.rs`、`tests/dataset_fp_rate.rs`）：
  - 新增 `full_context: &str` 参数，`allowlist_stopwords` 现在在**全文中搜索**而非只在 7-20 字节命中片段里
  - 这是核心机制升级 —— 让"教学语境词"（`the difference between` / `DO NOT RUN` / `compare to a suspicious case` / Dockerfile 安全前缀如 `/var/lib/apt/lists/`）能豁免短命中（如 `eval $`、`rm -rf /`、`systemctl enable`）
  - 调用方需同时传 matched_text + full_context；现有所有调用点已更新

- **22 条新规则 + 9 条现有规则补充 allowlist_stopwords**：教学场景豁免词（"the difference between"、"explain"、"DO NOT"、"NEVER"）、合法初始化（`ssh-agent -s` / `direnv hook` / `starship init` / `pyenv init` / `brew shellenv`）、Dockerfile 安全前缀（`apt/lists`、`var/cache`、`tmp`）、官方 registry 域名（`registry.npmjs.org` / `pypi.org`）等

### 测试结果（PRD §9 #7 验证）

| 指标 | 扩充前 | 扩充后 | 阈值 | 状态 |
|------|-------|-------|------|------|
| Critical FP rate | 0% (70 样本) | **0.00%** (0/1070 样本) | < 0.5% | ✅ 通过 |
| Attack recall rate | 100% (226 样本) | **97.13%** (676/696 样本) | > 95% | ✅ 通过 |

按桶细分：
- benign 11 桶全 0 FP（含 near-IN-CR-02-rce 100/100、near-IN-CR-04-persistence 100/100 等高风险桶）
- attacks-by-fear 4 桶 recall：signing 100% / shell-rce 97.5% / env-leak 97.5% / private-key 88.33%
- 现有 226 attacks 维持 100% recall（无回归）

### Known limitations

- **20 条仍漏拦**（已记录在 `tasks/2026-05-01-rule-gaps.md`）：
  - 14 条在 private-key 桶，全是 BIP39 助记词入站检测，需 second-pass 复用 outbound `validate_bip39`（vectorscan 不适合 alternation 超过 2048 词的 wordlist），推迟到下个 sprint
  - 6 条 shell/env 边缘形态（OS-level 编码绕过 / 内嵌脚本变种），下次迭代加 pattern

### 文档同步

- `docs/design/architecture.md` §5 误报率预算章节加入实测数字
- `docs/guides/development.md` §3.3 补 `cargo test --test dataset_fp_rate -- --ignored` 命令 + bench-data 目录结构说明
- `docs/requirements/user-stories.md` 加 US-21（"用户最怕的五件事"覆盖率验收）

---

## [v1.5-multi-agent] - 2026-04-28

### Architecture（multi-agent 扩展）

- **Phase 1 范围扩展**：Claude Code → Claude Code + OpenClaw + Hermes 三家适配（PRD §9 第 9 条重写）
- **UnifiedMessage 真实运行时支持双协议**：Anthropic Messages API + OpenAI Chat Completions 均解析为同一中间表示，不再仅预留接口（关联 ADR-018）
- **X-Sieve-Origin HTTP header 协议**：sub-agent 嵌套调用追踪，Ed25519 签名防伪造，关联 ADR-019
- **Hook 类规则在 OpenClaw / Hermes 上降级为 GUI hold**：三家 hook 机制不同，最低公分母统一走 GUI hold；Critical fail-closed 规则在三家全保留

### Added

- 2 个新 ADR：ADR-018（OpenAI 协议适配 / OpenAI Chat Completions SSE delta 格式解析）、ADR-019（X-Sieve-Origin header / sub-agent 嵌套调用追踪协议）
- 1 个新 SPEC：SPEC-004（multi-agent setup 配置注入 / `sieve setup --agent` 多 agent 参数）
- **IN-GEN-06**：外部 channel prompt injection 检测（命令式短语 + 来源不可信 channel → Critical GUI hold 60 秒默认拒绝）
- **IN-CR-06**：OpenClaw 动态 skill 加载 fail-closed（Critical block，YOLO mode 不可关）
- `sieve setup --agent claude|openclaw|hermes` 多 agent 参数 + `--all-detected` 自动扫描
- `sieve doctor --all` 验证三家 agent 配置全绿
- `sieve uninstall --all` 一键清理所有 agent 适配（按 `setup.log` 逆序回滚）
- IPC schema 新增字段：`source_agent` / `origin_chain` / `source_channel`（`#[serde(default)]` 向后兼容）
- glossary 新增 8 条 multi-agent 术语：OpenClaw / Hermes / X-Sieve-Origin / chain_depth / origin_chain / source_channel / multi-agent 调用链 / sub-agent 决策传递
- 用户故事新增 US-18（OpenClaw 跨通道 injection 防御）/ US-19（Hermes sub-agent 嵌套决策传递）/ US-20（multi-agent 一键安装）

### Changed [BREAKING]

- **PRD §9 第 9 条重写**（v1.4 → v1.5）：从"Phase 1 仅适配 Claude Code，UnifiedMessage 接口预留"扩展为"Phase 1 GA 适配三家：Claude Code / OpenClaw / Hermes，UnifiedMessage 真实运行时支持 Anthropic + OpenAI 双协议；其他协议（Gemini / Mistral 等）推 Phase 2"
- roadmap Week 6-8 重写：Week 6 = OpenAI 协议适配 + multi-agent setup（8 条任务）；Week 7 = OpenClaw/Hermes 集成测试（5 条任务）；Week 8 = 高强度 dogfood 扩到三家 + Stripe 接入

### Hardened

- `chain_depth ≥ 2` 强制 fail-closed GUI hold，不传递上游决策
- `chain_depth ≥ 5` 直接返回 426，不弹窗（防无限递归调用链）
- X-Sieve-Origin header Ed25519 签名，防伪造注入

### Migration（v1.4 → v1.5）

- v1.4 引擎完全复用：sieve-rules / dispatch / IPC 基础结构 / GUI 弹窗逻辑不变
- v1.4 用户升级 v1.5 不需要重新跑 `sieve setup`（除非要加 OpenClaw / Hermes 适配）
- 老 `sieve.toml` 加新 agent 字段后向后兼容（`#[serde(default)]` 处理缺失字段）
- IPC schema 新增字段向后兼容：旧 sieve-hook 读新 schema 时新字段用默认值，不报错

---

## [v1.4-architecture] - 2026-04-28

### Architecture [BREAKING]

- **处置矩阵二维化**：从一维四级（Critical / High / Medium / Low）→ 二维（出站/入站 × 严重度），规则 manifest 新增 `disposition` / `default_on_timeout` 字段（关联 ADR-016）
- **出站自动脱敏路径**：OUT-01~05/12 高频脱敏类不再弹窗，命中后自动改写请求 body + 状态栏 5s 通知，不打断工作流（关联 PRD §9 第 13 条）
- **HIPS 弹窗架构 + 25s keep-alive comment hold 流**：GUI 弹窗类（IN-CR-01/05）hold SSE 流期间每 25 秒发 `: keep-alive\n\n`，避免 Claude Code HTTP 超时；用户中止时截流注入优雅 error event（关联 SPEC-002）
- **Native GUI App 提到 Phase 1 必做**：macOS SwiftUI 独立进程，独立 git 仓库 `sieve-gui-macos`（不在本 workspace）（关联 ADR-012）
- **双层防御**：Sieve 代理（出站脱敏 + GUI hold 流）+ Claude Code PreToolUse hook（Hook 类阻断）；`sieve-hook` 作为独立 crate 加入 workspace（关联 ADR-014）

### Added

- 新 crate `sieve-ipc`：JSON-RPC over Unix socket + 文件锁 IPC 库（pending / decisions 文件协议，关联 ADR-013）
- 新 crate `sieve-hook`：极简 PreToolUse hook 二进制，依赖仅 `serde_json` + `fd-lock` + `uuid` + `chrono` + `clap`，启动时延 4~5ms（关联 SPEC-001）
- 5 个新 ADR：ADR-012（Native GUI App）、ADR-013（IPC 协议）、ADR-014（双层防御）、ADR-015（sieve setup 工具）、ADR-016（处置矩阵二维化）
- 3 个新 SPEC：SPEC-001（sieve-hook 协议）、SPEC-002（HIPS 弹窗行为）、SPEC-003（sieve setup 工具）
- `sieve-rules` manifest 新字段：`disposition`（AutoRedact / GuiPopup / HookTerminal / StatusBar）、`timeout_seconds`、`default_on_timeout`
- `critical_lock` 新常量：`HOOK_RULES`（25 条 Hook 类规则）、`GUI_RULES`（11 条 GUI 弹窗类规则）；`FAIL_CLOSED_RULES` 扩展为 24 条 Critical 全集

### Changed [BREAKING]

- ADR-007 Week 3 落地的"SSE 流 `sieve_blocked` 截流"对 Hook 类（IN-CR-02~04 / IN-GEN-01~03）的实现改由 `sieve-hook` 在 PreToolUse 阶段完成；fail-closed 原则不变，实现路径变（ADR-014 supersede ADR-007 中 Hook 类相关部分）
- Phase 1 仅 macOS：Linux / Windows 推 Phase 2；sigstore CI 保留多平台编译，`sieve setup` 命令在非 macOS 报友好错误

### Hardened — 新增 PRD §9 硬约束第 11-13 条

- **第 11 条（不在 Anthropic API 协议层撒谎）**：不伪造 tool_use / stop_reason / id / usage / type；拦截发生时允许截 SSE 流注入 `sieve_blocked` event（Sieve 自报事件，不是冒充模型）；keep-alive comment 行 `: keep-alive\n\n` 不属于伪造
- **第 12 条（不装本地 CA 做 MITM）**：Network Extension / 本地 CA 注入 / 系统 proxy 修改均推 Phase 3 选购，Phase 1/2 不做
- **第 13 条（出站脱敏不打断工作流）**：OUT-01~05/12 高频类必须自动脱敏 + 状态栏 5s 通知，不弹窗；每天弹几十次的产品没人用

### Removed / Deprecated

- 撤销 `architecture.md §6` 中的"❌ 桌面 GUI App（Electron / Tauri）"决策（被 ADR-012 翻转）
- 撤销 `deployment.md §2.2 §2.3` Linux/Windows 安装详细步骤（推 Phase 2，保留占位段）

### Migration

- Critical 规则永远 fail-closed，不允许通过任何 preset 关闭
- 旧 `sieve.toml` 加新字段后向后兼容（`#[serde(default)]` 处理缺失字段），但旧 sieve binary 读新 toml 会因 `unknown_fields` 失败
- `pre-v1.4-refactor` git tag 标记重构基线，回滚：`git checkout pre-v1.4-refactor`

---

## [Unreleased](https://github.com/doskey/sieve/compare/v0.1.0...HEAD)

### Known Issues — Week 4 dogfood 实测发现

#### 🔴 P0：入站检测仅覆盖流式 SSE，非流式 JSON 响应里的 tool_use 完全绕过
- 当前 `daemon::forward_with_inbound_inspection` 把 response body 喂给 `SseParser` +
  `Aggregator` + `InboundFilter`，假定响应是 `text/event-stream` 字节流
- Anthropic Messages API 同样支持非流式调用：客户端不传 `stream:true`（或显式
  `stream:false`）时，响应是单个 `application/json` body。Sieve 当前对这种响应
  原样透传，**所有入站规则失效**——IN-CR-02 / IN-CR-03 / IN-CR-04 / IN-CR-05 /
  IN-GEN-* 都被绕过
- 攻击者只需让 SDK 发非流式请求，就能让模型在 tool_use 里写 `>> ~/.bashrc` /
  `eth_signTransaction` / `rm -rf /` 而 Sieve 完全看不到。PRD §5.2「入站是 Sieve
  真正的护城河」语境下属严重产品级缺陷
- **修复进度**：roadmap Week 4 加入硬阻塞项，2026-05-04 前必须关闭。修复路径：
  daemon 按 response content-type 路由，JSON 分支解析 `AnthropicResponse.content[]`
  → 提取 tool_use → 走 `InboundFilter::on_tool_use_complete`；fail-closed Critical
  时把 body 替换为 `sieve_blocked` 等价 JSON。集成测试加 mock 非流式 upstream
  覆盖
- 详见 [tasks/lessons.md「入站检测仅覆盖流式 SSE」](../../tasks/lessons.md) /
  [tasks/roadmap.md Week 4](../../tasks/roadmap.md)

#### claude `-p` headless 默认走 OAuth 直连（dogfood 操作记录）
- Claude Max OAuth 优先级高于 `ANTHROPIC_BASE_URL`，非 `--bare` 模式 claude CLI
  会忽略代理直连 claude.ai 后端
- 必须用 `claude --bare -p` 强制走 `ANTHROPIC_API_KEY` auth 路径才会经过代理
- 不影响 Sieve 代码，只影响 dogfood 流程；development.md 待补「dogfood / 调试」段
- 详见 [tasks/lessons.md](../../tasks/lessons.md)

### [BREAKING] — Week 4 (2026-04-27)

#### rule ID 重命名：旧 `IN-CR-04` markdown exfil → `IN-GEN-04`
- 原 Week 3 落地的 markdown 图片 exfil 规则错置于 `IN-CR-*`（Crypto 钩子）命名空间。
  按 PRD §5.2，`IN-CR-04` 应是持久化机制；markdown 通用 exfil 归 `IN-GEN-*`
- 行为不变：仍是 high warn / 不入 fail-closed 名单 / 不阻断流量
- **fingerprint 失效**：fingerprint = `sha256(rule_id || matched_text)`，rule_id 改名
  → `~/.sieve/sieveignore` 中以旧 IN-CR-04:* 开头的条目自动失效。Week 1 末仅 doskey 一人
  dogfood，无外部影响

### Added — Week 4 持久化机制（IN-CR-04 全套）

#### 入站持久化机制检测（IN-CR-04，PRD §5.2 / Roadmap Week 4 / US-07）
- 9 条 IN-CR-04-* 子规则，全部 **Critical block + fail-closed**（YOLO mode 不可关，进
  `FAIL_CLOSED_RULES` 名单），覆盖主流后门埋点路径：
  - `IN-CR-04-SHELL-RC-APPEND`：`>>` / `tee -a` 写 `.bashrc` / `.zshrc` / `.bash_profile` /
    `.zprofile` / `.profile` / `.bash_aliases` / `.kshrc` 等
  - `IN-CR-04-CRONTAB`：`crontab -e` / `<` / `-r`（编辑 / install / 删除；`-l` 仅查看不命中）
  - `IN-CR-04-CRON-D-WRITE`：写 `/etc/cron.{d,daily,hourly,monthly,weekly,allow,deny}/`
  - `IN-CR-04-LAUNCHCTL`：`launchctl load` / `bootstrap` / `enable` / `kickstart` / `asuser`
  - `IN-CR-04-LAUNCH-AGENT-PLIST`：写 `~/Library/LaunchAgents/*.plist` 或
    `/Library/LaunchDaemons/*.plist`（要求 `>` / `cp` / `mv` / `tee` / `cat >` 写意图前缀）
  - `IN-CR-04-SYSTEMCTL-ENABLE`：`systemctl enable` / `--user enable` / `start` /
    `daemon-reload`（不拦 `disable` / `stop` / `status`）
  - `IN-CR-04-SYSTEMD-UNIT-WRITE`：写 `/etc/systemd/system/*.{service,timer,socket}` 或
    `~/.config/systemd/user/*.{service,timer,socket}`
  - `IN-CR-04-FISH-CONFIG`：`>>` / `tee -a` 写 `~/.config/fish/config.fish` / `conf.d/*.fish`
  - `IN-CR-04-LOGIN-ITEMS`：macOS `defaults write LoginItems` 或
    `osascript ... login items`
- 设计原则：pattern 锚定 Bash 命令的"写意图"（重定向 / tee / crontab 编辑 / launchctl
  load / systemctl enable 等），不拦读路径——避免与 IN-CR-03 read=High 处置冲突
- 已知 gap：Edit/Write 类工具直接写持久化文件（无 Bash 重定向）不被 IN-CR-04 直接命中；
  但配套启用动作（launchctl load / systemctl enable）仍会触发，多步攻击链至少一处被截断
- ADR-007 §"Week 4 落地范围"已记录 traceability，无需新 ADR

#### critical_lock 扩展（sieve-rules）
- `FAIL_CLOSED_RULES` 加 9 条 IN-CR-04-* 子规则
- 新增单测 `in_cr_04_persistence_fail_closed` 验证 9 条全部 fail-closed
- `enforce_action` 单测加 IN-CR-04-CRONTAB 验证 manifest action=Warn 仍强制 Block

#### 测试
- sieve-rules 新增 14 个单测（9 IN-CR-04 正例 + 4 关键 benign 反例 + 1 unrelated benign）
  + 1 critical_lock 单测
- sieve-cli 新增 1 个端到端集成测试（`in_cr_04_persistence_shell_rc_blocked`）：
  tool_use Bash command `>> ~/.bashrc` → SSE 注入 sieve_blocked 含 IN-CR-04-SHELL-RC-APPEND
- 全 workspace 192/192 测试通过（174 → 192，+18，零回归）

### Added — Week 4 进行中 (2026-04-27 起)

#### 入站敏感路径检测（IN-CR-03，PRD §5.2 / Roadmap Week 4）
- 10 条 IN-CR-03-* 子规则，全部 high warn（非 fail-closed Critical，给用户判断空间），完整覆盖 US-07 验收清单（SSH / AWS / GCP / Solana / Ethereum keystore）：
  - `IN-CR-03-SSH-PRIVATE`：SSH 私钥文件名（id_rsa / id_ed25519 / id_ecdsa / id_dsa），allowlist `*.pub`
  - `IN-CR-03-SSH-DIR`：`~/.ssh/...`，allowlist `known_hosts` / `authorized_keys` / `config` / `environment`
  - `IN-CR-03-AWS-CREDS`：`~/.aws/credentials`（不拦 `~/.aws/config`）
  - `IN-CR-03-DOTENV`：`.env` / `.env.local` / `.env.production` 等，allowlist `.env.example` / `.template` / `.sample` / `.dist` / `.test` / `.ci`
  - `IN-CR-03-ETH-KEYSTORE`：geth keystore 文件名 `UTC--<timestamp>--<40hex>`
  - `IN-CR-03-GPG-DIR`：`~/.gnupg/...`
  - `IN-CR-03-NETRC`：`.netrc` 凭据文件
  - `IN-CR-03-MACOS-KEYCHAIN`：`login.keychain-db` / `System.keychain`
  - `IN-CR-03-GCP-CREDS`：`~/.config/gcloud/application_default_credentials.json` / `legacy_credentials/`
  - `IN-CR-03-SOLANA-KEYPAIR`：`~/.config/solana/*.json`（CLI 默认 keypair 位置）
- 复用 IN-CR-02 已有 `engine_adapter::check_tool_use` 通道——`tool.input` JSON 序列化后喂给 vectorscan，无需新增扫描器
- daemon 当前仅记录 detection 到日志（`tracing::warn!`），不阻塞流量；5s 倒计时弹窗 UI 留 Week 5

#### Engine: longest-match-per-start dedup（sieve-rules::engine）
- `VectorscanEngine::scan` 对带量词（`{m,n}` / `(?:..)*`）的 pattern 在多个 endpoint 触发的回调，按 `(rule_id, start)` 去重保留最长 end
- 修复 IN-CR-03-DOTENV / SSH-DIR allowlist 失效问题（短 match `.env` 拿不到完整文件名上下文，绕过 `\.env\.example` 白名单）
- 输出排序确定（`(start, rule_id)` 字典序），下游处理与测试可重现
- 副作用：OUT-06 JWT / OUT-08 Stripe 等贪婪 pattern 现在每个起点仅返回最长 match，detection 数量减少但语义不变

#### 测试
- sieve-rules 新增 16 个单元测试（IN-CR-03 8 正例 + 6 allowlist 反例 + 2 通用 benign）+ 1 个引擎 dedup 测试
- sieve-cli 新增 1 个端到端集成测试（`in_cr_03_sensitive_path_warn_passes_through`）
- 全 workspace 174/174 测试通过（Week 3 → Week 4：158 → 174，+16 新增 0 回归）



#### 入站 SSE 流式处理（sieve-core）
- `SseParser`：增量解析器，push_chunk + flush 接口，**无缓冲整流**
- 5 类边界全覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流
- `Aggregator`：Tool Use partial_json 跨 SSE event 聚合，content_block_stop 后 deserialize
- `StreamingPipelineNode` trait：observe_event / on_tool_use_complete / on_message_stop
- `InboundFilter` impl：入站文本扫描 + tool_use 检查 + .sieveignore 过滤
- `InboundEngine` trait（由 sieve-cli engine_adapter 桥接到 VectorscanEngine）
- `AddressGuard`：IN-CR-01 地址替换检测，strsim Levenshtein，distance ∈[1,3] 且 len 相等触发
- ToolUseBlock 加 `span: Option<ContentSpan>` 字段

#### 入站规则集（sieve-rules/rules/inbound.toml）
- IN-CR-01 地址替换（占位，实际由 AddressGuard strsim 实现）
- IN-CR-02 危险 shell 命令（rm -rf / curl|sh / wget|sh / eval / nc 反弹 / dd 擦盘）
- IN-CR-04 markdown exfil（候选，warn）
- IN-CR-05 签名工具白名单（EVM eth_signTransaction 等 / Solana signTransaction 等 / Bitcoin signRawTransaction 等）
- IN-GEN-01 markdown javascript: URI（critical block）
- IN-GEN-02 inline HTML img（warn）
- IN-GEN-03 bash -c 任意执行（critical block）
- 共 13 条规则，vectorscan PCRE 子集兼容

#### Critical fail-closed 强制（sieve-rules::critical_lock）
- `FAIL_CLOSED_RULES` 常量（出站 OUT-01~12 全部 + 入站 11 条 critical 规则）
- `is_fail_closed(rule_id)` + `enforce_action(rule_id, requested) -> Action`
- 运行时检查：在 OutboundAdapter / InboundAdapter scan 时调用，即使 manifest action 写 allow / mark 也强制为 Block

#### Daemon 入站集成（sieve-cli）
- `daemon::run` 新签名：`(cfg, OutboundFilter, Arc<dyn InboundEngine>, Arc<HashSet<String>>)`
- `forward_with_inbound_inspection`：SSE tee 透传 + 同步 SseParser/Aggregator/InboundFilter，Critical 命中**注入 sieve_blocked event 然后关 channel**（用户拍板：截流策略）
- **剥掉响应 content-length 头**强制 chunked transfer（否则注入 sieve_blocked 时 body 长度对不上 content-length 会被客户端截断）
- `engine_adapter::InboundAdapter` 实现 `sieve_core::InboundEngine`
- main.rs 启动加载 inbound rules，partition __ADDRESS_GUARD_PLACEHOLDER__ 占位规则不传 vectorscan
- `audit_yolo_disabled`：运行时检查 config 字段（deny_unknown_fields 已防御 + 此函数双保险 tracing）

#### Fuzz 双引擎（关联 PRD §9 #5 硬约束）
- `fuzz/`（libFuzzer + cargo fuzz）：3 个 fuzz_target（sse_parser / tool_use_aggregator / inbound_filter）
- `fuzz_afl/`（AFL++ + afl crate）：3 个对应 target
- `fuzz/corpus/sse_parser/` 14 个 seed：正常流 / partial_json 跨 chunk / ping 穿插 / error 中途 / 半行 chunk / C0 控制 / 多 event 粘包 / 提前 EOF / 未知 event / 8KB partial_json / UCSB 4 类攻击 PoC
- `sieve_core::fuzz_helpers` 模块（双引擎共享函数体）
- ci.yml 加 fuzz-quick job：每 push 跑 60s/target 总 3 分钟
- fuzz-nightly.yml 骨架（workflow_dispatch only，Week 6+ 启用 schedule）

#### Bug 修复（Week 2 遗留）
- 出站 dry_run + Critical bug：Week 2 实现允许 dry_run + Critical 时仍 forward 到上游，违反 ADR-007 fail-closed。**Week 3 修复**：fail-closed 名单中的规则在 dry_run 下仍返 426
- 入站截流时 sieve_blocked event 注入与 content-length 冲突：剥掉响应 content-length 强制 chunked

#### 测试与验证
- 单元 + 集成测试 138 个全过（增量：sieve-core 25→56 / sieve-rules 29→50+12 / sieve-cli 15+3+5→15+5+3+5）
- Python smoke 29/29（原 26 + 入站 benign 透传 1 + 真 key 测试 2）
- UCSB 4 类攻击 PoC 测试（`tests/inbound_block.rs` 5 项）全部 ✓
- release 二进制 9.1 MB（Week 2 9.0 → +0.1 MB，strsim + 入站规则）
- cargo bench 编译通过（Week 4 实测性能）
- cargo deny licenses bans sources 全过（加 NCSA for libfuzzer-sys + MIT for fuzz crates）

### Pending（Week 4 起）
- 完整 secret benchmark 数据集（200-500 攻击样本 + 50-100 benign 真实会话回放）
- IN-GEN-04~05 完整规则
- 主动 macOS / TUI 弹窗（Action::WarnConfirm 实现）
- BIP39 Phase 2 multi-language wordlist（目前仅英文）

### Known Issues
- 入站截流时 Claude Code SDK 收到不完整 SSE 序列（text 已发但 message_stop 未发），依赖 SDK 容错；真实 SDK 行为需 dogfood 验证
- IN-CR-04 markdown exfil 当前 warn，Week 4 评估升级 critical 触发条件
- IN-CR-01 阈值 `distance ∈[1,3] 且 len 相等`，Phase 1 仅覆盖 ETH 地址，BTC 地址 Week 4 加
- AFL++ nightly 仅 sse_parser_afl，其他两个 target 待 Week 6+ 补

---

### Added — Week 2 (2026-04-27 完成)

#### 出站规则引擎(sieve-rules)
- `VectorscanEngine`：vectorscan-rs 0.0.6 多模式正则，Block mode + SOM_LEFTMOST 报告精确字节偏移
- `MatchEngine` trait + `MatchHit` 数据结构（关联 ADR-001 / ADR-002）
- `bip39 module`：SHA-256 checksum 验证（PRD §9 #4 差异化点），内嵌 BIP39 官方英文 2048 词 wordlist（MIT）
- `placeholder module`：全局占位符黑名单（YOUR_API_KEY / xxx / 0x0...0 等）
- `loader::load_outbound_rules(toml_path)`：从 toml 加载规则集
- `RuleEntry` 扩展字段：`entropy_min` / `keywords` / `allowlist_regexes` / `allowlist_stopwords`（gitleaks 风格）
- criterion micro benchmark 骨架（`benches/scan_bench.rs`，Week 4 接入完整 secret benchmark）

#### 出站规则集(sieve-rules/rules/outbound.toml)
- OUT-01~12 全部 P0 规则上线（gitleaks MIT 风格 pattern）：
  - OUT-01 Anthropic API key / OUT-02 OpenAI API key（legacy + proj 新格式）
  - OUT-03 AWS Access Key / OUT-04 GitHub PAT / OUT-05 GCP API key
  - OUT-06 JWT / OUT-07 PEM Private Key / OUT-08 Stripe Live Key
  - OUT-09 Slack Token / OUT-10 OpenSSH Private Key / OUT-11 Discord Bot Token
  - OUT-12 BIP39 助记词（占位 pattern，运行时由 bip39 module 二次 checksum 验证）
- 12 条规则 positive + negative case 集成测试全部通过（`tests/outbound_rules.rs`）

#### Detection 数据模型(sieve-core)
- `Detection { id: Uuid, rule_id, severity, action, source, span, evidence_truncated, fingerprint }`
- `Severity { Low, Medium, High, Critical }`
- `Action { Block, Redact, WarnConfirm, MarkOnly, SilentLog }`
- `ContentSource { OutboundUserText, OutboundSystemText, InboundAssistantText, InboundToolUseInput }`
- `fingerprint(rule_id, content) -> 16 hex`（SHA-256 前 8 bytes，关联 docs/design/data-model.md §155-161）
- `PipelineNode::process()` 签名升级：返回 `Vec<Detection>`
- `OutboundFilter`（impl PipelineNode，从 sieveignore 过滤）+ `OutboundEngine` trait（由 sieve-cli engine_adapter 桥接）
- `AnthropicRequest::extract_text_content()`：从 messages + system 提取所有文本内容

#### Daemon 集成(sieve-cli)
- `Config` 加 `rules_path` / `sieveignore_path` / `dry_run` 字段
- `Command::Start` 加 `--dry-run` flag（覆盖 config.dry_run）
- `engine_adapter::OutboundAdapter`：把 sieve_rules::VectorscanEngine 适配到 sieve_core::OutboundEngine trait
- daemon::proxy_inner：POST /v1/messages 走 collect → 解析 AnthropicRequest → extract_text → OutboundFilter::process → Critical 命中且非 dry_run 返 426 JSON；其他路径继续流式透传（保 Week 1 字节级一致）
- 426 JSON schema：`{ type: "sieve_blocked", blocked_at, detections[], guidance: { zh, en } }`
- fail-closed 启动：规则加载失败 / vectorscan 编译失败 → 直接退出，不 fallback 无规则模式（ADR-007）

#### 测试与验证
- 单元测试：sieve-core 25 / sieve-rules 26 / sieve-cli 15 = **66 单元测试**
- 集成测试：`outbound_rules` 12 / `proxy_passthrough` 5 / `outbound_block` 3 = **20 集成测试**
- e2e smoke（`scripts/smoke_test.py`）26 项断言通过（原 21 + 新 4 项 426 拦截 + 1 项 benign 透传）
- `cargo deny check licenses bans sources` 全过
- release 二进制 9.0 MB（< 22MB 预算）

### Pending（Week 3 起）
- 完整 SSE Parser + fuzz corpus（PRD §9 #5）
- 入站规则 IN-CR-01~05（地址替换 / 危险工具调用 / 签名工具）
- ADR-008 出站 Critical 状态码 dogfood 验证（Week 2 期间真实 dogfood 后正式落 ADR）

### Known Issues（Week 2）
- 426 时间戳 Phase 1 用 UNIX epoch 秒（简化），Week 4 引入 chrono 改完整 RFC3339
- BIP39 OUT-12 vectorscan 预筛 pattern 占位符 `__BIP39_PREFILTER_PLACEHOLDER__`，运行时由 bip39 module 动态生成 alternation；集成测试中过滤掉（由 bip39 module 单测覆盖）
- 完整标准 secret benchmark 自建样本数据集留 Week 4（PRD §10.1 原计划）

---

### Verified — 2026-04-27 Week 1 完成定义实跑

#### release.yml workflow_dispatch 首跑(run [24980079580](https://github.com/doskey/sieve/actions/runs/24980079580))
- **三 target reproducible build 双 SHA-256 一致 + cosign keyless OIDC 签名 + Rekor 上链 + cosign verify-blob 自验证**:
  - `aarch64-apple-darwin`: `af5c371f1a6531d2a8439425f9d90a5e339fca20a62825b8d895f29c6b883899`
  - `x86_64-apple-darwin`:  `47b729ee298f9dc1d5a3bd0a04f5f30b19983b7c87454b7358442514762164ea`
  - `x86_64-unknown-linux-gnu`: `bbe16fc2faf52a010dd3b3ae172599ec6b7ae9c8cd666c6046d06cfe265065fa`
- 已知遗留:`macos-universal` lipo 合并步骤路径修复(本 commit 含)。**universal binary 不影响 reproducible build pipeline 主路径验证**(三个独立 target 都已成功签名 + 上链)。
- ADR-006 §10 Week 1 hard gate **达成**。

#### 端到端 dogfood 验证(PRD §10.1 Week 1 第 1 完成定义)
- doskey 在真机用 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453 claude` 启动 Claude Code v2.1.119(Opus 4.7),非流式聊天测试通过(2026-04-27 14:35 时点)。
- e2e smoke test 脚本(`scripts/smoke_test.py`)真机自验 21/21 通过:401 字节级透传 / 4xx 错误码 / 8KB body / 20 路并发 / 真 key 200 / SSE 流式 / tool_use partial_json。

### Added — Week 1 (2026-04-27 启动)

#### 工程骨架
- Cargo workspace + 三 crate（`sieve-core` / `sieve-rules` / `sieve-cli`），关联 .cursorrules §3.3
- `rust-toolchain.toml` 锁定 1.87.0，targets: aarch64-apple-darwin / x86_64-apple-darwin / x86_64-unknown-linux-musl
- `.cargo/config.toml` reproducible build flags（`--remap-path-prefix` + musl 静态链接），关联 ADR-006
- `deny.toml` cargo-deny 策略（出站 host 白名单 + 许可证白名单 + advisories yanked deny），关联 ADR-003

#### sieve-core（透传层）
- `UnifiedMessage` schema（Anthropic-only 实现，UpstreamProvider::Relay variant 预留），关联 PRD §6.1 / ADR-004
- `AnthropicRequest` 解析（serde 子集，#[serde(flatten)] 兼容未识别字段）
- `Forwarder`：hyper 1.x + hyper-rustls 0.27 + rustls 0.23 + aws-lc-rs provider + webpki-roots，ALPN h2+http/1.1
- SSE passthrough（Week 1 字节透传，Week 3 切到 parser）
- `PipelineNode` trait 占位（Week 2 起 OutboundFilter / InboundFilter 实现）
- 错误类型：`thiserror`，**禁止 anyhow**（.cursorrules §3.2）

#### sieve-rules（占位骨架）
- `MatchEngine` trait + `MatchHit` 数据结构占位（Week 2 起 vectorscan 实现）
- `RulesManifest` schema（rules-vN.manifest.json），关联 data-model.md
- `Ed25519 Verifier`（规则包验签占位，Week 5 起做实际下发）
- `vectorscan-rs 0.0.6` 依赖加入（用于三平台编译验证），关联 ADR-001
- `ed25519-dalek 2.x` 依赖加入

#### sieve-cli（daemon）
- `sieve start --config <path>`：hyper 1.x server 监听 127.0.0.1:11453，反向代理到 api.anthropic.com
- 配置加载：TOML，bind_addr **强制 127.0.0.1** 校验（非 loopback → exit(1)），关联 ADR-003 / PRD §9 #2
- 透传逻辑：headers 剥 Host 重写，body 通过 hyper `Incoming` 流式 chunk-by-chunk（SSE 字节级零缓冲）
- `serde(deny_unknown_fields)`：任何未知配置字段直接拒绝
- `audit` 模块占位（Week 4 接入 SQLite append-only）
- tracing-subscriber 日志（`SIEVE_LOG` 环境变量控制等级）
- **未引入 --disable-critical / --yolo flag**，关联 ADR-007

#### CI / CD（关键 - ADR-006 hard gate）
- `.github/workflows/ci.yml`：fmt / clippy / test（ubuntu + macos-14 矩阵）/ cargo-deny
- `.github/workflows/release.yml`：tag v* 触发，矩阵覆盖三 target，**双构建 SHA-256 比对**，**cosign keyless OIDC 签名**（`id-token: write`），Rekor 透明日志上链，sigstore bundle 上传到 GitHub Release
- macOS universal binary（lipo 合 aarch64 + x86_64）

### Changed — 2026-04-27 PRD §9 #10 修订

- **撤销 "Day 1 GitHub repo 公开 README + 架构文档" 承诺**，见 [ADR-011](../design/ADR-011-private-until-ga.md)
- 新策略：Week 12 GA 时一次性公开 repo + 代码（MIT）+ 文档 + sigstore 验证流程
- 影响范围：repo 保持 private 至 Week 12；Week 1-11 release.yml 不绑定 tag（改为 workflow_dispatch），减少 Rekor 透明日志痕迹
- ADR-006 sigstore + reproducible build CI **不受影响**，GA 前照常跑通；只是不做 public Rekor 验证演示
- 营销弹药 GA 当天集中释放（文章 1+2+3 同步）

### Pending（Week 3 起）
- SSE Parser 完整实现 + fuzz corpus（PRD §9 #5）
- 入站 Crypto 钩子（IN-CR-01~05）

### Known Issues
- 本地需安装 Rust toolchain（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- vectorscan-rs 编译需要系统包 `boost` + `ragel`（macOS：`brew install boost ragel`；Linux：`apt-get install libboost-dev ragel`）
- ADR-008 出站 Critical 状态码（候选，Week 2 dogfood 实测后落 ADR）
- ADR-005 [redacted] Week 1 启动（非工程任务，doskey 跟进）

---

> 以下为 PRD v1.3 设计阶段计划，**尚未实现**。任何条目在实际编码、测试、签名验证完成前不视为已交付。

### 计划中（Phase A dogfood, Week 1-8）

#### 新增

- **W1 基础设施 + Anthropic 协议**
  - Rust 项目骨架（`sieve-core` / `sieve-rules` / `sieve-cli` workspace）
  - `hyper` + `tokio` + `rustls` HTTP 反向代理跑通
  - 透明转发 Anthropic Messages API（`POST /v1/messages` 含 SSE，`POST /v1/messages/count_tokens`，`GET /v1/models`）
  - `UnifiedMessage` 内部 schema（仅 Anthropic 实现，其他 provider 接口预留，[PRD §9 #9](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - ~~GitHub repo 公开~~ — 已被 [ADR-011](../design/ADR-011-private-until-ga.md) 撤销，repo 保持私有至 Week 12 GA
  - **🚨 sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 必须 W1 跑通** —— [PRD §1.2 第 4 句](../prd/sieve-prd-v1.3.md#12-四句话核心叙事v13-加第-4-句) 自证清白叙事的物质基础
- **W2 出站 P0 规则（OUT-01 ~ OUT-12）**
  - OUT-01 OpenAI / Anthropic API key（前缀 + entropy + 占位符黑名单，FP < 0.1%）
  - OUT-02 AWS Access Key（`AKIA[0-9A-Z]{16}` + 排除官方示例，FP < 0.1%）
  - OUT-03 GitHub Token（前缀 + CRC32 校验，FP < 0.05%）
  - OUT-04 JWT（三段 base64 + header 解码验证，FP < 0.5%）
  - OUT-05 RSA / Ed25519 / SSH 私钥（PEM 头精确匹配，FP < 0.01%）
  - OUT-06 Ethereum 私钥（regex + entropy + 上下文，FP < 1%，**只能 High，不上 Critical**）
  - OUT-07 Bitcoin WIF（base58 + 双 SHA-256 校验位，FP < 0.001%）
  - OUT-08 Solana 私钥（base58 88 字符或 hex 64 字节，FP < 1%）
  - **OUT-09 BIP39 助记词 + SHA-256 checksum 验证**（差异化点，FP < 0.05%；[PRD §9 #4](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - OUT-10 Keystore JSON（Web3 Secret Storage v3 schema，FP < 0.01%）
  - OUT-11 .env 文件特征（多行 KEY=VALUE 密度阈值，FP < 5%，仅 Medium）
  - OUT-12 数据库连接串（URI scheme + 用户名密码字段，FP < 0.5%）
  - 占位符黑名单 + `.sieveignore` 学习型白名单
  - 单元测试覆盖 ≥ 80%
- **W3 入站 Crypto 钩子**
  - SSE Parser + `tool_use` Aggregator
  - **IN-CR-01 地址替换检测**（对话历史 `0x[a-fA-F0-9]{40}` 比对：相同放行 / 前 N 后 M 匹配标红 / Levenshtein ≤ 4 标黄）
  - **IN-CR-05 签名工具 fail-closed**（`eth_sendTransaction` / `signTransaction` / `signMessage` / `signTypedData` 全部强制弹窗，YOLO mode 不可关闭，[PRD §9 #3](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - **大量 SSE fuzz test** 覆盖 PRD §9 #5 列出的 6 类边界
- **W4 入站通用 + 危险 tool call + benchmark 数据集**
  - IN-CR-02 危险工具调用（`bash` 含 `rm -rf` / `curl..|sh` / `eval(base64..)` / `sudo`）
  - IN-CR-03 敏感路径访问（`~/.ssh/`、`~/.aws/`、`/etc/shadow`、`.env`、`*.keystore`、`~/.config/solana/`）
  - IN-CR-04 持久化机制（`crontab`、`launchd`、`systemd`、`.bashrc`、`.zshrc`）
  - IN-GEN-01 危险 shell 模式（`rm -rf /`、fork bomb、`> /dev/sda`、`dd if=/dev/zero`）
  - IN-GEN-02 远程脚本执行（`curl X | sh`、`wget X | bash`、`bash <(curl X)`）
  - IN-GEN-03 编码后执行（`eval(base64.b64decode(...))`、`exec(__import__('os')...)`）
  - IN-GEN-04 Markdown 图片 exfil（`![](http://X.com/?Y=Z)` + 域名不在白名单）
  - IN-GEN-05 Prompt injection 反向（`<|im_start|>`、`[INST]`、`### System:`、`Ignore previous`）
  - 处置矩阵完整实现（Critical / High / Medium / Low → HTTP 行为映射，参见 [API 参考 §5](../api/api-reference.md#5-处置矩阵--http-行为)）
  - CLI 弹窗 + 命令行确认（fail-closed，超时按拒绝，参见 [API 参考 §6](../api/api-reference.md#6-cli-退出码--弹窗确认协议)）
  - **Benchmark 数据集**（[PRD v1.3 §10.1 W4 修订](../prd/sieve-prd-v1.3.md#101-phase-adogfood-阶段week-1-8)）：
    - 200-500 条合成攻击样本（UCSB 4 类攻击 + drainer 模式 + Pink Drainer 数字化绕过 + npm typosquat + `curl|sh` + eval base64）
    - 50-100 条真实 benign 会话回放（doskey 自己日常 Claude Code 工作录制）
    - canary 测试（假 BIP39、假地址、假 selector、假 .env，使用 honeypot 钱包私钥）
    - 目标：Critical FP < 0.5%，High FP < 5%
- **W5 配置系统 + 试用期 + brew tap**
  - 完整 `config.toml` schema（参见 [API 参考 §3](../api/api-reference.md#3-配置文件-schema-sieveconfigtoml)）
  - 本地 SQLite append-only 审计日志（仅 fingerprint + 元信息，**不存原文**）
  - License 验证 + 试用期机制（**本地 Ed25519 验证，不联网 verify**，[PRD §9 #2](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）
  - brew tap (`doskey/sieve`) + GitHub Releases 发布流水线
  - 本地管理 API（参见 [API 参考 §2](../api/api-reference.md#2-sieve-本地管理-api)）
- **W6 doskey 自用 + 修 bug**
  - doskey 100% 时间用 Sieve 工作
  - 性能 benchmark 验证 P99 < 20ms（[PRD §6.4](../prd/sieve-prd-v1.3.md#64-性能预算)）
  - macOS / Linux / Windows 二进制（macOS arm64 + Linux x86_64 为 Tier 1）
  - 收集 false positive，加 `.sieveignore` 默认条目
- **W7-W8 高强度 dogfood**
  - 第一次签名规则库下发测试（Ed25519 验证 fail-closed）
  - Stripe 接入 + license key 系统（**海外公司账号**，参见 [PRD §11.5.1](../prd/sieve-prd-v1.3.md#1151-公司主体与收款)）
  - 完成定义：doskey 用 Sieve 跑 2 周，无 P0 / P1 bug

### 计划中（Phase B 闭测, Week 9-12）

#### 新增

- **W9 闭测启动**
  - 5 人闭测白名单（[PRD v1.3 §10.2 W9 修订](../prd/sieve-prd-v1.3.md#102-phase-b闭测阶段week-9-12)）：
    - 高频 hackathon builder（ETHGlobal / Solana / 各 L2 hackathon 常客）
    - bug bounty hunter / 审计研究员（Code4rena / Sherlock / Immunefi 活跃用户）
    - 小团队 protocol engineer（< 10 人 protocol team）
  - **不邀请**：大企业开发者、纯 web2 友人、纯 KOL
  - 专属 license key
  - Discord 闭测频道
  - 每天处理反馈
- **W10 闭测 + 内容准备**
  - 修闭测 bug
  - 起草 3 篇引爆文章：
    1. 中转站揭黑（实测复刻 UCSB 论文方法论）
    2. **🆕 自证清白：Sieve 怎么证明自己不是新的 LiteLLM**（[PRD v1.3 §10.2 W10 修订](../prd/sieve-prd-v1.3.md#102-phase-b闭测阶段week-9-12)，把 §1.2 第 4 句讲透，后续所有营销围绕此 talking point）
    3. Pink Drainer 攻击复盘 + Sieve 怎么防
- **W11 闭测扩大 + 数据合作接洽**
  - 闭测扩到 10 人（同样画像标准）
  - landing page（英文为主，中文次之）
  - **数据合作优先于内容合作**（[PRD §13.2](../prd/sieve-prd-v1.3.md#132-数据侧合作清单v13-新增)）：
    - 第一目标：Chaofan Shou (@Fried_rice) 顾问关系
    - 第二目标：慢雾 @evilcos misttrack-skills 数据合作
- **W12 GA 发布（v0.1.0）**
  - **代码开源（MIT）**（[PRD §11.3](../prd/sieve-prd-v1.3.md#113-开源策略)）
  - 二进制 cosign 签名验证 + reproducible build 验证步骤公开（参见 [部署指南 §3](../guides/deployment.md#3-二进制签名验证必做)）
  - landing page 上线
  - 文章 1 + 2 同步发（中转站揭黑 + 自证清白）
  - 试用期全面开放
  - Stripe 收款上线（**[redacted]**）
  - **冻结 v1 公开 API**（参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)）
  - 完成定义：GA 第一周 GitHub stars > 200，试用注册 > 100，首批付费用户 ≥ 10

### 暂不做（明确推迟到 Phase 2）

- 中文 PII（身份证 / 银行卡 / 统一信用代码）
- 内网域名 / 内部代号（用户自定义规则）
- 长代码块识别 + Copyright 提示
- 自定义规则 DSL
- npm / pip typosquat 检测
- Markdown 链接钓鱼
- Unicode 攻击防御（NFC + 控制字符黑名单）
- Calldata 静态解码（4byte 离线 SQLite）
- ERC20 危险 approve（`approve(MAX)` / `setApprovalForAll`）
- EIP-2612 / EIP-7702 滥用
- Drainer 黑名单（Chainabuse + ScamSniffer 集成）
- 协议白名单
- Solidity 后门检测（Slither）
- **MCP 拦截 IN-MCP-01~03**（[PRD v1.3 §5.2 修订](../prd/sieve-prd-v1.3.md#52-入站检测sieve-真正的护城河)，Phase 2 Week 16-20）
- 桌面 App / VS Code 插件
- OpenAI / Gemini / OpenRouter 协议适配（**第二个用户主动要才做**，[PRD §9 #9](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束)）

---

## PRD 文档版本演进

> Sieve 项目尚未发布二进制版本，但产品需求文档已迭代 4 版。每版日期 + 一句话差异：

### [PRD v1.0](../prd/sieve-prd-v1.0.md) - 2026-04-26

- 工程启动前 PRD，团队 SaaS 视角，覆盖完整商业计划
- 一句话：双向检测的本地 LLM 流量代理，服务 crypto 开发者，反对中转站不可信
- 状态：**已废弃**，被 v1.1 收敛

### [PRD v1.1](../prd/sieve-prd-v1.1.md) - 2026-04-26

- 个人项目版：从 v1.0 砍掉一半范围，定位从"独角兽备选"改为"个人产品 + 现金流 + IP 入口"
- 关键改动：
  - 定价收敛为 3 档（Free / $19 Pro / $99 Crypto）
  - MVP 范围砍 50%，只做出站 secret + 危险 tool call + 地址替换 + 签名拦截
  - 三 agent 适配（Claude Code / OpenClaw / Hermes）用统一本地代理
  - sigstore + reproducible build 提到 Phase 1 必交付物
  - 桌面 App、VS Code 插件、Slither、中文 PII 全部推到 Phase 2
  - 节奏：6-8 周冲 MVP + 慢节奏维护
- 状态：**已废弃**，被 v1.2 第一性原理重写覆盖

### [PRD v1.2](../prd/sieve-prd-v1.2.md) - 2026-04-26

- 第一性原理修订版：用 12 条公理重新推导每个决策
- 关键改动：
  - 定价收敛到 **单一 [redacted]/月**（年付 [redacted]0），降级模式只读警告（公理 11 / 12）
  - 公理 7：Phase 1 **只做 Claude Code**，OpenClaw / Hermes 推迟到第二个用户主动要时
  - 12 周冲 GA（8 周 dogfood + 4 周闭测）
  - 处置矩阵：Critical 阻断 + High 警告 + Medium 标记 + Low 静默
  - "Sieve 的本质不是 LLM 安全产品，是在不可逆动作前插入认知摩擦的保险工具"
- 状态：**被 v1.3 取代**

### [PRD v1.3](../prd/sieve-prd-v1.3.md) - 2026-04-26（**当前活动版本**）

第一性原理 + 合规边界修订版，**锁定执行**。在 v1.2 基础上吸收 GPT-5.5 review 的 8 条改动：


| #   | 改动                                                                               | 章节         |
| --- | -------------------------------------------------------------------------------- | ---------- |
| 1   | **新增中国大陆合规边界**（v1.2 完全漏掉的硬约束）                                                    | §11.5 整章新增 |
| 2   | **"自证清白"从工程细节提到产品定位** —— sigstore + 透明日志做成营销 talking point                       | §1.2 第 4 句 |
| 3   | **数据标注稀缺性论证** —— 单人团队最稀缺资源不是算力，是持续标注高质量数据的能力                                     | §6.2       |
| 4   | **Benchmark 数据集大小具体化** —— 200-500 攻击样本 + 50-100 benign 会话                        | §10.1 W4   |
| 5   | **闭测画像精确化** —— hackathon builder + 审计研究员 + 小团队 protocol engineer                 | §10.2 W9   |
| 6   | **数据侧伙伴接洽清单** —— SlowMist / ScamSniffer / GoPlus / Chainabuse / Sourcify / Forta | §13.2      |
| 7   | **MCP 拦截放进 Phase 2** —— Claude Code 真实威胁面（IN-MCP-01~03）                          | §5.2       |
| 8   | **用户教育成本作为风险登记** + [redacted]周期延误风险                                                  | §12        |


附加改动：

- §1.4 法律实体明确：[redacted]（首选香港，次选新加坡）
- §3.1 P0 客群地理分布：海外为主
- §3.3 不服务客群补充：中国大陆境内公开 to-C 商业化
- §10.1 W1 sigstore + reproducible build pipeline 必须本周跑通
- §10.2 W10 文章 2 改为"自证清白"主题
- §10.2 W11 KOL 接洽：数据合作优先于内容合作
- §11.3 透明更新日志加入开源策略
- §15.4 监管参考资料

---

## 文档结构变更

### [unreleased](https://github.com/doskey/sieve/compare/v0.1.0...HEAD) - 2026-04-27

#### 新增

- 文档结构初始化：
  - `docs/api/api-reference.md` —— API 参考首版（反向代理 / 本地管理 API / 配置 schema / 环境变量 / 处置矩阵 / 错误码）
  - `docs/guides/development.md` —— 开发指南首版（构建、测试、SSE fuzz、benchmark、规则编写、PR 流程）
  - `docs/guides/deployment.md` —— 部署与运维指南首版（安装、签名验证、服务运行、升级回滚、FAQ）
  - `docs/changelog/CHANGELOG.md` —— 本文件
- 所有文档反映 [PRD v1.3](../prd/sieve-prd-v1.3.md) 设计意图，**未实现任何代码**

#### 文档审查与一致性修复（2026-04-27）

全量审查 docs/ 文档对 PRD v1.3 的一致性，输出关键冲突清单并修复。

**修复（关键冲突）**：

- [ADR-005](../design/ADR-005-overseas-legal-entity.md) —— 移除未授权的 BVI / Cayman [redacted]行（PRD §1.4 仅锁定香港 / 新加坡 / [redacted]）
- [ADR-006](../design/ADR-006-sigstore-reproducible-build.md) —— 显式标注 Tier 1（macOS / Linux）/ Tier 2（Windows）平台分级；承认 Windows reproducible build 推到 Phase 2 是与 PRD §9 #6 全平台理想的暂时偏离，需 PRD 下次修订同步
- [deployment.md §2.3](../guides/deployment.md) —— Windows 部署描述加 Tier 2 标识（Week 6+ 才出二进制 + 签名，reproducible 不承诺）
- [deployment.md §11.1](../guides/deployment.md) —— 补 license 离线过期 → 降级模式自动转换流程
- [api-reference.md §1.6](../api/api-reference.md) —— 补完整入站 Critical SSE 序列（`sieve_block` → buffer 暂停 → `sieve_resume` / `sieve_terminate`）+ buffer 上限 + `event_id` 关联
- [api-reference.md §2.2.3](../api/api-reference.md) —— 补 `user_decision` 字段值域定义（`null` / approve / deny / timeout / interrupted）
- [api-reference.md §2.2.4](../api/api-reference.md) —— 明确 fingerprint = `<rule_id>:<sha256_prefix_8_hex>` 长度规范
- [api-reference.md / deployment.md] 多处过时的"待写"链接修正为已存在文档的真实链接
- 删除 CHANGELOG.md 末尾空白占位符（"模板段落" / "链接"）

**新增文档**：

- [docs/glossary.md](../glossary.md) —— 项目术语表（54 条术语，覆盖产品 / 架构 / 检测 / 安全 / 协议 / 运营 / 合规 7 个主题）
- [docs/design/ADR-INDEX.md](../design/ADR-INDEX.md) —— ADR 索引 + 编号规则 + 候选 ADR 列表（ADR-008 候选 Critical 状态码 / ADR-009 候选 Windows 服务 / ADR-010 候选加密支付通道）
- [tasks/roadmap.md](../../tasks/roadmap.md) —— 12 周里程碑可勾选执行清单 + 跨周依赖图
- [tasks/lessons.md](../../tasks/lessons.md) —— 经验教训记录骨架

**倾向决策（doskey 确认 2026-04-27）**：

- **ADR-008（候选）出站 Critical 状态码维持 `426 Upgrade Required`**——api-reference.md §7.2 现有方案。Week 2 dogfood 阶段实测 Claude Code SDK 行为后正式落 ADR；如 SDK 表现异常（自动重试 / 错误信息丢失等）再切换备选方案。已在 [tasks/roadmap.md](../../tasks/roadmap.md) Week 2 任务清单加入验证项。

#### Git 仓库脚手架（2026-04-27）

为内部 GitHub repo 基础设施（GA 前私有；[ADR-011](../design/ADR-011-private-until-ga.md) 规定 Week 12 GA 时公开）准备完整的 git 治理文件：

- **新增** `.gitignore` —— Rust + macOS / Linux / Windows + Sieve 特定（`.sieveignore` / `audit.db` / `*.sigstore` / 临时文档）。**Cargo.lock 不入忽略名单**（reproducible build 要求入库，[ADR-006](../design/ADR-006-sigstore-reproducible-build.md)）
- **新增** `.gitattributes` —— 强制 LF 行尾（reproducible build 跨平台一致性）+ GitHub linguist 语言识别（docs / prd / research 标记 vendored / documentation）+ 二进制文件标记
- **新增** `SECURITY.md` —— 安全漏洞报告流程（email doskey.lee@gmail.com 临时渠道，security@sieve.tools 待 Week 6-8 商标定后启用）+ 24h/7d/30d 响应 SLA + 自身供应链承诺 + 不在范围清单
- **新增** `LICENSE` —— 双轨许可说明：文档 **CC BY-NC-SA 4.0** / 代码 **MIT**（均在 Week 12 GA 时同步公开；[ADR-011](../design/ADR-011-private-until-ga.md)）
- **新增** `.github/ISSUE_TEMPLATE/` —— bug_report / feature_request / **suspicious_sample**（[PRD §8.1](../prd/sieve-prd-v1.3.md#81-简化版) 用户公开提交可疑样本走这里）+ config.yml（指引安全漏洞走 SECURITY.md，紧急资产损失走 email）
- **新增** `.github/PULL_REQUEST_TEMPLATE.md` —— 对齐 [.cursorrules §五](../../.cursorrules) 自检清单 + PRD §9 硬约束验证 + 检测项变更模板 + Breaking Changes 流程
- **新增** `.github/dependabot.yml` —— Cargo 周更（仅 patch / minor，major 走人工评估，对齐 [PRD §9 #6](../prd/sieve-prd-v1.3.md#9-工程上必须做对的硬约束) pinned dependencies）+ GitHub Actions 周更 + 关键依赖分组（tokio-stack / simd-stack / crypto-stack）

仓库尚未 `git init`。doskey 完成审阅后可执行：
```bash
cd /Users/doskey/src/sieve
git init
git add -A
git commit -m "chore: initial commit, Pre-Code design phase docs"
# 创建 GitHub repo 后：
git remote add origin <github-url>
git push -u origin main
```

---

## 相关文档

- 项目入口：[../../README.md](../../README.md)
- 当前活动 PRD：[../prd/sieve-prd-v1.3.md](../prd/sieve-prd-v1.3.md)
- API 参考：[../api/api-reference.md](../api/api-reference.md)
- 开发指南：[../guides/development.md](../guides/development.md)
- 部署指南：[../guides/deployment.md](../guides/deployment.md)
- 术语表：[../glossary.md](../glossary.md)
- ADR 索引：[../design/ADR-INDEX.md](../design/ADR-INDEX.md)
- Roadmap：[../../tasks/roadmap.md](../../tasks/roadmap.md)

---

> 本文档遵循 [Sieve 文档规则](../../.cursorrules)。
> 任何依赖升级、行为变更、检测项 ID 增删必须在本文记录（`[.cursorrules` §1.5](../../.cursorrules)）。