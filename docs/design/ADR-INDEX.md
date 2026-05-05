# Sieve ADR 索引

> Architecture Decision Records 索引与编号规则。
> ADR 平铺在 `docs/design/ADR-NNN-*.md`（项目级共识，不引入 adr/ 子目录）。

---

## 编号规则

- 三位数递增编号 `ADR-NNN-描述.md`，**不跳号、不复用**
- 决策被推翻：写新 ADR + 旧 ADR 状态改为 **Superseded by ADR-NNN**，**不删除旧文件**
- 决策提案中：状态 **Proposed**；接受后改 **Accepted**；废弃改 **Deprecated**

---

## 当前 ADR（按编号）


| 编号                                                        | 标题                                                     | 状态       | 决策日期       | 关联 PRD                |
| --------------------------------------------------------- | ------------------------------------------------------ | -------- | ---------- | --------------------- |
| [ADR-001](./ADR-001-rust-tech-stack.md)                   | 选用 Rust 作为技术栈                                          | Accepted | 2026-04-26 | §6.3、§9.1             |
| [ADR-002](./ADR-002-rule-engine-only-phase1.md)           | Phase 1 纯规则引擎，不引入本地 ML 模型                              | Accepted | 2026-04-26 | §6.2                  |
| [ADR-003](./ADR-003-local-only-no-cloud-verifier.md)      | 完全本地运行，绝不联网做 token verifier                            | Accepted | 2026-04-26 | §1.2、§9.2、§11.2       |
| [ADR-004](./ADR-004-anthropic-first-unified-interface.md) | Phase 1 只适配 Anthropic Messages API，UnifiedMessage 接口预留 | Accepted | 2026-04-26 | §6.1、§9.9             |
| [ADR-005](./ADR-005-overseas-legal-entity.md)             | [redacted]作为收款与营销载体                                        | Accepted | 2026-04-26 | §1.4、§11.5            |
| [ADR-006](./ADR-006-sigstore-reproducible-build.md)       | Sigstore 签名 + Reproducible Build + 透明日志                | Accepted | 2026-04-26 | §1.2、§9.6、§10.1、§11.3 |
| [ADR-007](./ADR-007-fail-closed-critical-actions.md)      | Critical 等级 fail-closed 强制确认，YOLO mode 不可关闭            | Accepted | 2026-04-26 | §5.3、§9.3、§9.8、§11.2  |
| [ADR-011](./ADR-011-private-until-ga.md)                  | Week 12 GA 前 repo 完全私有，不做任何 public 暴露                 | Accepted | 2026-04-27 | §9 #10、§10.1、§11.3     |
| [ADR-012](./ADR-012-native-gui-app-phase1.md)             | Phase 1 必做 Native GUI App（macOS SwiftUI 独立进程）           | Accepted | 2026-04-28 | §6.4、§6.6、§10.1 Week 5 |
| [ADR-013](./ADR-013-ipc-protocol.md)                      | IPC 协议：JSON-RPC over Unix socket + 文件锁 JSON 文件          | Accepted | 2026-04-28 | §6.5、§10.1 Week 3+5    |
| [ADR-014](./ADR-014-dual-layer-defense.md)                | 双层防御：Sieve 代理（SSE 层）+ Claude Code PreToolUse Hook     | Accepted | 2026-04-28 | §6.7、§9 第 11 条         |
| [ADR-015](./ADR-015-sieve-setup-tool.md)                  | sieve setup / doctor / uninstall 自动配置三件套（macOS only）   | Accepted | 2026-04-28 | §6.6、§10.1 Week 5      |
| [ADR-016](./ADR-016-disposition-matrix-2d.md)             | 处置矩阵从一维四级升级为二维（出站/入站 × 严重度）                        | Accepted | 2026-04-28 | §5.3、§5.4、§9 第 13 条   |
| [ADR-018](./ADR-018-openai-protocol-adaptation.md)        | sieve-core 新增 OpenAI Chat Completions 协议适配层，UnifiedMessage 真实支持双协议 | Accepted | 2026-04-28 | §6.1、§6.3、§9 第 9 条    |
| [ADR-019](./ADR-019-x-sieve-origin-header.md)             | X-Sieve-Origin HTTP header 协议——sub-agent 嵌套调用链元数据传递与双重弹窗去重 | Accepted | 2026-04-28 | §4.6、§6.5               |
| [ADR-020](./ADR-020-user-rules-system.md)                 | 用户规则系统（单文件 user.toml + 11 类安全约束 + $EDITOR + 4 子命令）             | Accepted | 2026-05-01 | v2.0 §5.5、§9 #14       |
| [ADR-021](./ADR-021-tri-state-decision-and-graylist.md)   | 三态决策（Allow / Deny / Ask）+ 灰名单 schema + Critical 锁三道防线        | Accepted | 2026-05-01 | v2.0 §5.4、§9 #3 延伸     |
| [ADR-022](./ADR-022-behavior-sequence-window.md)          | 行为序列联动窗口（结构化特征 / N=10/TTL=5min / IN-SEQ-* / GA 默认关闭）        | Accepted | 2026-05-01 | v2.0 §5.7、§9 #15       |
| [ADR-023](./ADR-023-process-context-audit.md)             | 进程上下文记录（caller_pid + caller_exe / proc_pidinfo / LRU cache）  | Accepted | 2026-05-01 | v2.0 §5.6              |
| [ADR-024](./ADR-024-rules-engine-abstraction.md)          | 规则引擎抽象（MatchEngine::scan(ScanRequest) + LayeredEngine 合并顺序）  | Accepted | 2026-05-01 | v2.0 §6.3、§5.5.2.1     |
| [ADR-025](./ADR-025-content-type-routing-matrix.md)       | content-type 路由矩阵（v1.5.4 P0 教训永久化，4 类组合集成测试硬约束）           | Accepted | 2026-05-01 | v2.0 §9 #16、§5.7.4     |
| [ADR-026](./ADR-026-port-based-listener-routing.md)       | Port-based listener routing —— 多上游 listener + path prefix 修复            | Accepted | 2026-05-05 | v2.0 §6.1、§6.5、§9 #2、§9 #16 |
| [ADR-027](./ADR-027-network-jail-enforcement.md)          | Network jail enforcement —— 防火墙层硬隔离 LLM 流量（v3.x post-GA opt-in） | Proposed | 2026-05-05 | v2.0 §1.2、§9 #12、§11.2、§11.5 |
| [ADR-028](./ADR-028-ipc-protocol-neutralization.md)       | IPC 协议中性化 —— 去 GUI 假设 + sieve-ipc 内部模块化 + headless decision path | Accepted | 2026-05-05 | v2.0 §6.5、§6.6        |


---

## 候选 / 计划中 ADR


| 候选编号    | 主题                                                     | 触发文档                      | 优先级 | 计划周次     |
| ------- | ------------------------------------------------------ | ------------------------- | --- | -------- |
| ADR-008 | Critical 出站状态码（426）+ 入站 SSE event 注入 — Week 2-3 dogfood 期间实测后升 Accepted | api-reference.md §7.2 + §7.3 | P0  | Week 2-3 dogfood 期间 |
| ADR-009 | Windows 服务部署形态（sc.exe NT Service 选择）                   | guides/deployment.md §5.4 | P2  | Week 6+  |
| ADR-010 | [redacted]实现路径（Stripe Crypto vs Coinbase Commerce vs 自部署） | ADR-005 §3                | P2  | Week 7+  |

### 候选 ADR 倾向决策（doskey 已签确）

- **ADR-008**：**维持 `426 Upgrade Required`**（确认日期 2026-04-27）。出站用 426，入站用 sieve_blocked SSE event 注入（Week 3 落地）。Week 2-3 dogfood 期间实测 Claude Code SDK 行为；dogfood 无异常后正式升 Accepted。如 SDK 表现异常（自动重试 / 错误信息丢失等）再切换备选方案。
- **ADR-009**：待定。Week 6+ Windows 二进制 Tier 2 上线时评估。
- **ADR-010**：初步方向 = Stripe + Coinbase Commerce 双通道（[ADR-005 §3](./ADR-005-overseas-legal-entity.md)）。Week 7+ 公司主体落地后正式立项。


---

## 维护规则

- 新增 ADR：在表格按编号顺序插入；同步更新 README.md 文档导航
- 状态变化：仅更新本索引，**不修改 ADR 内的"已接受"标注**（用 Superseded by 链接代替）
- ADR 之间互相引用：用相对路径 `./ADR-NNN-*.md`

---

## ADR 模板

最简模板（完整模板见 project CLAUDE.md 全局规范）：

```markdown
# ADR-NNN: 标题

## 状态
**Proposed | Accepted | Deprecated | Superseded by ADR-XXX**
> 决策日期：YYYY-MM-DD
> 范围：...
> 关联 PRD：[v2.0 §X](../prd/sieve-prd-v2.0.md)

## 背景
...

## 决策
...

## 影响
### 正面影响
...
### 负面影响
...
### 需要更新的文档
- ...

## 相关文档
- ...
```

---

## 相关文档

- [架构](./architecture.md)
- [数据模型](./data-model.md)
- [PRD v2.0](../prd/sieve-prd-v2.0.md)
- [API 参考](../api/api-reference.md)
- [部署指南](../guides/deployment.md)

