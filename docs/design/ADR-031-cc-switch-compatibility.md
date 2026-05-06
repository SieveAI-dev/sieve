# ADR-031: 与 cc-switch 互操作策略（Sieve 作为 cc-switch 一档供应商）

## 状态
**Proposed**
> 决策日期：2026-05-06（草案，未通过）
> 范围：与第三方 Claude Code 配置管理器（cc-switch）的兼容路径选择
> 关联：[ADR-026](./ADR-026-port-based-listener-routing.md)（multi-listener 路由）/ [ADR-029](./ADR-029-free-first-defer-monetization.md)（装机量优先）/ [ADR-030](./ADR-030-update-telemetry-channel.md)（遥测 install UUID）/ [ADR-015](./ADR-015-sieve-setup-tool.md)（sieve setup）

## 背景

[cc-switch](https://github.com/farion1231/cc-switch)（farion1231/cc-switch，截至 2026-05 GitHub 49k stars，GitHub Trending 第一）是大陆开发者社区中最流行的 Claude Code / Codex / Gemini CLI / OpenCode / OpenClaw 供应商配置管理器。它是 Sieve 在「中文圈装机量」议题上必须正面应对的事实标准。

### cc-switch 关键事实（2026-05 调研）

- **技术栈**：Tauri 2.8 + React 18 + TS + Rust（与 `sieve-gui-macos` 同栈）
- **数据存储**：`~/.cc-switch/cc-switch.db`（SQLite SSOT）+ `~/.cc-switch/settings.json`（设备 UI 偏好）+ `~/.cc-switch/backups/`（保留最近 10 份，自动轮转）+ `~/.cc-switch/skills/`（软链接桥接）
- **切换机制**：UI 选定供应商后**原子写**（temp + rename）到 live 配置文件——对 Claude Code 即 `~/.claude/settings.json` 中的 `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN`；编辑当前供应商时反向从 live 文件回填（双向同步，防用户手改被静默覆盖）
- **「热切换」语义**：仅指对 Claude Code 改完无需重启 CLI，**不是真正的运行时路由**（Claude Code 自身在每次请求时读 env / settings.json）
- **生态资产**：50+ 预设供应商（AWS Bedrock / NVIDIA NIM / DeepSeek / Kimi / GLM / PackyCode / AICodeMirror / 硅基流动 / 胜算云 / Cubence 等）；统一 MCP/Skills/Prompts 面板；云同步（Dropbox/OneDrive/iCloud/WebDAV）；用量看板；会话历史浏览；本地代理（含格式转换、自动 failover、熔断）
- **无 CLI 版本**，仅桌面 GUI

### 为什么需要立 ADR

1. **生态位重叠的事实判断**：cc-switch 是「上游配置管理器」，Sieve 是「本地安全闸 + 多 listener 路由」（[ADR-026](./ADR-026-port-based-listener-routing.md)）。两者模型在用户视角上**有 30% 重叠**（都涉及切上游），如果不显式定义协作边界，会出现：
   - 用户同时装两者，两边都改 `~/.claude/settings.json`，互相覆盖（cc-switch 的双向同步会把 Sieve 写入的 base_url 误回填为「Sieve 是一个供应商」）
   - 用户在中文社区询问「cc-switch 和 Sieve 哪个更好」时，没有官方答案
   - cc-switch 的 50+ 预设拉力远超 Sieve（Sieve 当前 0 预设），新装机用户大概率先选 cc-switch 而错过 Sieve

2. **装机量战略对齐**（[ADR-029](./ADR-029-free-first-defer-monetization.md)）：Phase 1 唯一指标是装机量，敌对 cc-switch 没有任何收益，借力是更优解。

3. **不与 [ADR-027](./ADR-027-network-jail-enforcement.md) 网络隔离冲突**：cc-switch 自身是本地工具，不需要网络出站；Sieve 与 cc-switch 互操作不引入新的信任边界外联。

## 决策

### 决策 1：定位「互补而非竞争」，cc-switch 作为前端配置管理器，Sieve 作为后端安全闸

不与 cc-switch 在「切供应商」UX 上正面竞争。Sieve 接受 cc-switch 是大陆中文圈的事实入口，把自己定位成「在 cc-switch 选定的上游和 Claude Code 之间插一道本地安全闸」。

### 决策 2：路径 A 为 GA 前实施目标，路径 B 候选评估，路径 C 永久排除

| 路径 | 描述 | 工作量 | 决策 |
|------|------|--------|------|
| **A** | Sieve 作为 cc-switch 里的一种「特殊供应商」（base_url 指向 Sieve listener，真实凭据落 Sieve 配置） | 小（文档 + 一个供应商模板 PR） | **GA 前必做（P0）** |
| **B** | Sieve daemon 启动时读 `~/.cc-switch/cc-switch.db`，自动为每个 cc-switch provider 起 listener，订阅 active provider 变化 | 中（耦合 cc-switch schema，需跟踪迁移） | **候选，Phase 2 评估** |
| **C** | 推 cc-switch 维护者把 Sieve 嵌入为可选层 | 大（依赖对方节奏） | **永久排除**（不可控） |

### 决策 3：路径 A 落地约束

1. **Sieve listener 在 cc-switch 中以「Sieve Local Proxy」名义注册**，base_url 形如 `http://127.0.0.1:<sieve_listener_port>`，API key 字段填占位符（如 `sieve-local-proxy`，Sieve 不校验该字段）
2. **真实供应商凭据落在 Sieve daemon 配置（`config.toml` listener section）**，不进入 cc-switch 的 SQLite——避免 cc-switch 备份/云同步把上游 API key 上传到第三方网盘（Dropbox / OneDrive 等违反 [PRD §9 #2](../prd/sieve-prd-v2.0.md) 的精神边界）
3. **Sieve 提供 `sieve setup --emit-cc-switch-template`** 子命令，输出可粘贴到 cc-switch「自定义供应商」表单的 JSON 片段（[ADR-015](./ADR-015-sieve-setup-tool.md) 扩展）
4. **文档新增 `docs/guides/integration-cc-switch.md`**，明确：
   - 安装顺序（先 Sieve 后 cc-switch，或反之的差异）
   - cc-switch 的双向同步在 Sieve 介入后的行为（cc-switch 仍能读回 base_url=Sieve 端口，视为正常）
   - 卸载顺序与回滚步骤
5. **不向 cc-switch 仓库提交 PR 把 Sieve 加入官方预设列表**——避免被解读为「Sieve 依赖 cc-switch」或「Sieve 是 cc-switch 的子集」。仅在 Sieve 文档中提供「如何在 cc-switch 中手工添加 Sieve」指引

### 决策 4：抄 cc-switch 已验证的工程实践（与互操作无关，独立小项）

cc-switch 在配置管理上有几项已被 49k 用户验证的工程实践，Sieve daemon 配置层应吸收：

1. **原子写**（temp file + rename）——避免崩溃中途配置半写。Sieve 当前 daemon 配置写盘策略需对齐
2. **自动备份轮转**（保留 10 份）——用户手改 `config.toml` 出错时可回滚
3. **「双向同步」概念**——若 Sieve 未来要在 GUI 中编辑 listener 上游，UI 编辑路径必须从磁盘文件读取最新值再回写，不能用内存缓存覆盖

### 决策 5：50+ 预设供应商生态的应对

cc-switch 的核心拉力是 50+ 预设。Sieve 不复制这一资产，但**最小限度内置 10–15 个常见中转站的 listener 模板**（DeepSeek / Kimi / GLM / PackyCode / 硅基流动 / 胜算云 / Anthropic 官方 / OpenAI 官方等），降低单装 Sieve 用户的配置摩擦。模板形态为 `config.toml.example` 中的注释片段，不引入运行时供应商目录概念。

### 决策 6：传播策略

- 在 Sieve README 与 docs/guides/ 加一节「与 cc-switch 协作」，主动承认 cc-switch 的中文圈地位
- 中文圈传播话术：「Sieve 不是 cc-switch 的替代品，是它后面的安全闸」
- 不在中文社区公开比较两者优劣（[PRD §11.5.2 渠道分级](../prd/sieve-prd-v2.0.md)：境内不做 to-C 公开商业化营销）

## 影响

### 正面影响
- 借力 cc-switch 49k stars 的中文圈装机量基础，符合 [ADR-029](./ADR-029-free-first-defer-monetization.md) 装机量优先
- 避免与最大社区工具的正面冲突，省下 50+ 预设供应商生态的重复建设成本
- 抄到 cc-switch 已验证的原子写 / 备份轮转 / 双向同步三项工程实践
- 用户视角下「cc-switch 选供应商 + Sieve 本地审计」叙事自洽，两者合用比单用任何一方更安全

### 负面影响
- 在中文用户心智中可能被误读为「cc-switch 的插件」（需要文档话术持续纠正）
- 路径 A 依赖 cc-switch 的「自定义供应商」入口长期保留，若对方未来调整 schema，Sieve 模板需跟随
- 路径 B 若启动，会引入对 cc-switch SQLite schema 的隐式耦合，迁移风险需评估
- 与 cc-switch 的备份/云同步功能边界需要文档反复说明（避免用户把 Sieve 上游凭据塞进 cc-switch 数据库再同步到 Dropbox）

### 风险与缓解
- **风险**：cc-switch 推出自己的「安全审计」功能，与 Sieve 直接竞争。**缓解**：Sieve 的差异化护城河（crypto-native 检测、双向 SSE 解析、本地推理零云依赖）非短期可复制；继续按 PRD 节奏走
- **风险**：cc-switch 维护者要求 Sieve 不得使用「cc-switch」字样。**缓解**：所有文档使用「与 cc-switch 协作」中性表述，不用 cc-switch 商标 / logo

### 需要更新的文档（路径 A 启动时）
- 新增 `docs/guides/integration-cc-switch.md`
- [README.md](../../README.md)「常见问题 / 与其他工具协作」章节
- [docs/api/api-reference.md](../api/api-reference.md) `sieve setup --emit-cc-switch-template` 子命令规格
- [SPEC-003 sieve setup tool](../specs/SPEC-003-sieve-setup-tool.md) 加 emit-template 子命令
- [config.toml.example](../../sieve.toml.example) 加 10–15 个中转站 listener 模板（决策 5）
- CHANGELOG（[FEATURE]）

## 待澄清 / 阻塞 ADR 通过的事项

本 ADR 暂留 Proposed 状态，**实际通过前需要回答以下问题**：

1. **路径 A 的「特殊供应商」字段约定**：cc-switch 自定义供应商表单的具体字段集合（API key 是否必填、能否为空字符串）需实测。先在沙盒环境验证再决定占位符值
2. **cc-switch 备份是否会把自定义供应商也带上 Dropbox**：若是，Sieve listener 端口信息（虽不敏感）也会被云同步，需在文档中说明
3. **Phase 1 GA 前是否启动决策 5（10–15 个内置模板）**：取决于 dogfood 用户是否反馈「单装 Sieve 配置上游太麻烦」。若 dogfood 反馈不强烈，决策 5 推后到 Phase 2
4. **是否值得做路径 B**：等装机量数据出来后回看。若 cc-switch 用户中安装 Sieve 的比例 > 50%，路径 B 投入产出比成立；否则不做
5. **本 ADR 通过前不动代码**——只完成调研记录与决策框架，实际实施延后

## 相关文档
- [ADR-015: sieve setup / doctor / uninstall 自动配置三件套](./ADR-015-sieve-setup-tool.md)
- [ADR-026: Port-based listener routing](./ADR-026-port-based-listener-routing.md)
- [ADR-029: 装机量优先,延后商业化](./ADR-029-free-first-defer-monetization.md)
- [ADR-030: 更新通道复用为遥测信标](./ADR-030-update-telemetry-channel.md)
- 外部参考：[farion1231/cc-switch GitHub](https://github.com/farion1231/cc-switch) / [README_ZH.md](https://github.com/farion1231/cc-switch/blob/main/README_ZH.md)
