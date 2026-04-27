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
| [ADR-005](./ADR-005-overseas-legal-entity.md)             | 海外公司主体作为收款与营销载体                                        | Accepted | 2026-04-26 | §1.4、§11.5            |
| [ADR-006](./ADR-006-sigstore-reproducible-build.md)       | Sigstore 签名 + Reproducible Build + 透明日志                | Accepted | 2026-04-26 | §1.2、§9.6、§10.1、§11.3 |
| [ADR-007](./ADR-007-fail-closed-critical-actions.md)      | Critical 等级 fail-closed 强制确认，YOLO mode 不可关闭            | Accepted | 2026-04-26 | §5.3、§9.3、§9.8、§11.2  |


---

## 候选 / 计划中 ADR


| 候选编号    | 主题                                                     | 触发文档                      | 优先级 | 计划周次     |
| ------- | ------------------------------------------------------ | ------------------------- | --- | -------- |
| ADR-008 | Critical 出站状态码选择（426 vs 451 vs 自定义）                    | api-reference.md §7.2     | P1  | Week 2-3 |
| ADR-009 | Windows 服务部署形态（sc.exe NT Service 选择）                   | guides/deployment.md §5.4 | P2  | Week 6+  |
| ADR-010 | 加密支付双通道实现路径（Stripe Crypto vs Coinbase Commerce vs 自部署） | ADR-005 §3                | P2  | Week 7+  |

### 候选 ADR 倾向决策（doskey 已签确）

- **ADR-008**：**维持 `426 Upgrade Required`**（确认日期 2026-04-27）。现有 [api-reference.md §7.2](../api/api-reference.md) 即此方案。Week 2 dogfood 阶段实测 Claude Code SDK 行为后正式落 ADR；如 SDK 表现异常（自动重试 / 错误信息丢失等）再切换备选方案。验证项已加入 [tasks/roadmap.md](../../tasks/roadmap.md) Week 2 任务清单。
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
> 关联 PRD：[v1.3 §X](../prd/sieve-prd-v1.3.md)

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
- [PRD v1.3](../prd/sieve-prd-v1.3.md)
- [API 参考](../api/api-reference.md)
- [部署指南](../guides/deployment.md)

