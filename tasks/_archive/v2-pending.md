---
name: v2.0/v2.1 剩余未完成工作
description: 代码侧 100% 完成后剩余的 5 项非代码工作 + 商业化阻塞索引
type: backlog
created: 2026-05-02
---

# v2.0/v2.1 剩余未完成工作

> 上次 push：`7dfdb3b` docs: v2.0 + v2.1 代码 100% 落地后下游文档全量同步（2026-05-01）
> 工作树：clean，本地与 origin/main 同步
> 范围：仅列**代码侧 100% 落地后剩余的工作**。商业化 17 项见 [doskey-todo.md](./doskey-todo.md)。

---

## 一、外部依赖（等上游 crate 发新版）

### TODO-EXT-1：vectorscan_rs `hs_database_size()` API 上限校验

| 字段 | 值 |
|---|---|
| 类型 | 外部依赖等待 |
| 触发 | vectorscan_rs 0.0.7+ 暴露 `hs_database_size()` 函数 |
| 工作 | `crates/sieve-policy/src/lint.rs::lint_pattern_resource_limits` 加 1MB db size 上限校验（PRD §5.5.3-B） |
| 当前形态 | 用编译时间 100ms 作 db size 的代理指标 |
| 关联 | [PRD §5.5.3-B](../docs/prd/sieve-prd-v2.0.md#553-用户规则安全约束) / `crates/sieve-policy/src/lint.rs:286-287` TODO 注释 |
| 阻塞 | 上游 vectorscan_rs crate 维护节奏（无 ETA） |
| 行动 | 每季度一次 `cargo outdated -p vectorscan_rs` 看版本 → 上游 PR 暴露 API 后再做 |

---

## 二、部署/运维任务（GA 前 doskey 自己做）

### TODO-DEPLOY-1：origin_header.rs GA 前替换真实 Ed25519 密钥

| 字段 | 值 |
|---|---|
| 类型 | 部署任务 |
| 触发 | Week 12 GA 前 |
| 工作 | 生成真实 Ed25519 密钥对 → `keys/origin_pubkey.ed25519` 替换占位 → 私钥保管在 1Password / hardware key |
| 当前形态 | `crates/sieve-ipc/src/origin_header.rs:23` 占位 TODO 注释（ADR-019） |
| 关联 | [ADR-019 X-Sieve-Origin header](../docs/design/ADR-019-x-sieve-origin-header.md) |
| 阻塞 | doskey 决定密钥保管方案（HSM / yubikey / 1Password）+ 生成时机 |
| 行动 | Week 11 闭测扩大期间生成 → Week 12 GA 一并 ship |

---

## 三、Dogfood 数据触发（doskey 用 Sieve 跑出数据后才能动）

### TODO-DOGFOOD-1：OpenClaw `skill_install_guard` Week 7 实测

| 字段 | 值 |
|---|---|
| 类型 | dogfood 实测 |
| 触发 | doskey 装上 OpenClaw → 跑实际 skill install 流量 → 抓 HTTP request body |
| 工作 | 4 处 `# TODO（Week 7）` 注释逐项落实： |
| | 1. `SKILL_INSTALL_PATH_PATTERNS` 候选路径列表 → 实测真实 endpoint 替换 |
| | 2. `body_looks_like_skill_manifest` 判定 → 实测 manifest schema 后改严格字段匹配 |
| | 3. `extract_manifest_summary` → 补充 `permissions` 列表解析 + 风险评分 |
| | 4. `check_openclaw_skill_install` → 接 source URL 黑名单查询 |
| 当前形态 | 占位逻辑已写好（覆盖常见路径模式 + 通用 manifest 字段判定）；文件 `crates/sieve-core/src/skill_install_guard.rs` |
| 关联 | [PRD v1.5 §4.6](../docs/prd/sieve-prd-v1.5.md#46) / [ADR-016](../docs/design/ADR-016-disposition-matrix-2d.md) / [SPEC-004](../docs/specs/SPEC-004-multi-agent-setup.md) §10 TBD-01 |
| 阻塞 | 装 OpenClaw + 跑流量样本 |
| 行动 | Week 7 集成测试期间装 OpenClaw → 抓 5+ 个真实 skill install request 样本 → 替换占位 |

### TODO-DOGFOOD-2：行为序列升级 Block 类的 ADR 评审

| 字段 | 值 |
|---|---|
| 类型 | 产品决策（dogfood 数据触发） |
| 触发条件（PRD §9 #15）| 真实付费用户连续 4 周 ≥ 50 个序列样本 + 该模式 FP rate < 0.5% |
| 工作 | 写新 ADR 评审升级 IN-SEQ-* 从 StatusBar 通知到 Block 阻断 → CHANGELOG `[BREAKING]` |
| 当前形态 | feature `sequence_detection` 默认 OFF（GA 不承诺）；3 条 IN-SEQ-* 仅 StatusBar |
| 关联 | [PRD §9 #15](../docs/prd/sieve-prd-v2.0.md#15行为序列检测的保守起步--beta-默认关闭v20-新增) / [ADR-022](../docs/design/ADR-022-behavior-sequence-window.md) |
| 阻塞 | 闭测/GA 后真实数据收集（最早 Week 12 GA + 4 周 = Week 16 起评估） |
| 行动 | GA 后每周看 `audit.db` 中 `kind = "sequence_hit"` 事件统计 → 满足条件后写 ADR-026 |

### TODO-DOGFOOD-3：行为序列 ML 分类器训练

| 字段 | 值 |
|---|---|
| 类型 | ML 训练（dogfood 数据集触发） |
| 触发条件 | dogfood + 闭测累积 ≥ 500 条标注序列样本（kill chain 正例 + 良性序列负例） |
| 工作 | 1. 从 `audit.db` 导出 `SequenceHit` + 无命中良性序列 → 用 `ToolUseRecord` 结构化特征做训练集 |
| | 2. 训练简单分类器（XGBoost / 浅层 MLP）→ 输出"序列恶意度 0~1"分数 |
| | 3. daemon 加 `inbound_filter.detect_sequence_hits_with_ml()` 路径（feature gated） |
| 当前形态 | `ToolUseRecord` 字段已按 ML 升级路径设计（隐私安全的结构化特征，PRD §5.7.1） |
| 关联 | [PRD §5.7.1](../docs/prd/sieve-prd-v2.0.md#571-序列窗口模型结构化特征版codex-review-后重写) / [ADR-022](../docs/design/ADR-022-behavior-sequence-window.md) |
| 阻塞 | 数据集积累（最早 Week 16+） |
| 行动 | GA 后等数据积累 → 评估是否值得引入 ML（也可能数据显示 3 条启发式已足够，跳过 ML） |

---

## 四、商业化阻塞（doskey 一人自己推）

详见 [tasks/doskey-todo.md](./doskey-todo.md) 17 项。代码侧零阻塞。摘要：

1. 海外公司注册（HK / SG / Stripe Atlas）
2. Stripe / Coinbase Commerce 商户接入
3. License 后端（域名 + 部署 + DB）
4. 域名 + DNS + 邮箱
5. Apple Developer Program + .dmg 公证
6. KOL 接洽（Chaofan / 慢雾 / Yu Feng）
7. Discord 闭测频道
8. ... 共 17 项

---

## 五、决策快查表

| 问"还能不能 ship？" | 答 |
|---|---|
| 当前代码能不能跑通完整流程？ | ✅ 可以。本地 dogfood 起 daemon + Claude Code 即可 |
| GA Week 12 是否阻塞在代码侧？ | ❌ 不阻塞。所有 PRD §5.4-§5.7 + §6.3-§6.6 + §9 #14-#16 已 ship |
| GA 前必须 doskey 做的部分？ | DEPLOY-1 真实密钥 + 商业化 17 项（独立追踪） |
| GA 后才能做的代码项？ | DOGFOOD-1/2/3（需真实数据触发） |
| 等上游的代码项？ | EXT-1 vectorscan_rs API |

---

## 六、最近一次代码状态（参考）

> 详见 [status-2026-05-01.md](./status-2026-05-01.md)。

- commit：`7dfdb3b`（origin/main HEAD）
- 测试：default 633 passed / feature on 644 passed / 1 failed（doctor 竞态，单跑通过）/ 5 ignored
- fmt + clippy --workspace --all-targets --all-features：clean
- 6 crate workspace（sieve-core / sieve-rules / sieve-cli / sieve-ipc / sieve-hook / sieve-policy）
- ~25,000 行 Rust 代码
