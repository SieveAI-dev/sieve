# Sieve 12 周里程碑 Roadmap

> Source of truth: [PRD v1.3 §10](../docs/prd/sieve-prd-v1.3.md#10-12-周里程碑8-周-dogfood--4-周闭测)
> 状态：**Week 1 工程启动（2026-04-27）**。Week 1 核心工程任务已完成，等首次 tag push 验证 sigstore pipeline。
>
> 本文是 PRD §10 的执行视图，**任务粒度 / 验收标准** 跟随 PRD 同步更新；本文新增"依赖"列与"风险"列辅助调度。

---

## Phase A · dogfood（Week 1-8）

### Week 1 · 基础设施 + Anthropic 协议 + sigstore pipeline

**完成定义**（PRD §10.1 Week 1）：

- [ ] doskey 自己用 Claude Code，设 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453`，所有日常操作正常 ← 等装 cargo 后实测
- [ ] sigstore 签名 pipeline + GitHub Actions reproducible build pipeline 已跑通 ← 等首次 tag push 后验证

**任务清单**：

- [x] Rust workspace 骨架（sieve-core / sieve-rules / sieve-cli 三个 crate）
- [x] hyper + tokio + rustls 跑通透明转发 Anthropic Messages API
- [x] SSE 透传基础（不做规则匹配）
- [x] UnifiedMessage 内部 schema（Anthropic only，OpenAI/Gemini 接口预留）
- [x] sigstore 签名 pipeline（cosign + Rekor）
- [x] GitHub Actions reproducible build pipeline（双构建 SHA-256 比对）
- [x] cargo fmt / clippy / 基础 CI 跑通
- [~] ~~GitHub repo 公开~~ — 已被 [ADR-011](../docs/design/ADR-011-private-until-ga.md) 撤销；Week 12 GA 一次性公开
- [ ] 启动海外公司注册（香港首选，PRD §10.1 Week 7-8 Stripe 接入需要）← 用户行动项

**依赖**：无（这是起点）
**关键风险**：sigstore + reproducible 双构建首次跑通预算 2-3 天，超期需立刻识别
**关联 ADR**：ADR-001 / ADR-004 / ADR-006

---

### Week 2 · 出站 P0

**完成定义**（PRD §10.1 Week 2）：

- [x] paste .env 触发拦截（集成测试 outbound_block 覆盖 + smoke test 26/26）
- [ ] 标准 secret benchmark FP < 1%, Recall > 70%（本地测试集已达，完整 benchmark 留 Week 4）

**任务清单**：

- [x] vectorscan-rs 多模式正则集成
- [x] OUT-01~12 全部 P0 出站规则（API key / AWS / GitHub / JWT / 私钥 / 助记词等）
- [x] BIP39 SHA-256 checksum 验证（差异化点）
- [x] 占位符黑名单 + .sieveignore 学习型白名单
- [x] 单元测试覆盖 ≥ 80%（66 单元 + 20 集成 = 86 测试）
- [x] 出站拦截 UI / 脱敏工作流（被动 426 + body 含人类可读说明）
- [ ] **ADR-008 出站 Critical 状态码 dogfood 验证**：Week 2 实现已用 426，本周 dogfood 期间（2026-04-27 起）实测 Claude Code SDK 行为；若一周内无异常，升 Accepted（候选编号已在 [ADR-INDEX](../docs/design/ADR-INDEX.md) 登记）

**依赖**：Week 1 Rust 骨架 + API 转发
**关键风险**：BIP39 checksum 实现错误导致误报，需严格单测
**关联文档**：PRD §5.1（出站检测P0表）/ [api-reference.md §7.2](../docs/api/api-reference.md)（426 当前默认）

---

### Week 3 · 入站 Crypto 钩子

**完成定义**（PRD §10.1 Week 3）：

- 复现 UCSB 论文 4 类攻击 PoC，Sieve 全部捕获

**任务清单**：

- SSE Parser + 流式处理框架
- Tool Use Aggregator（完整工具调用重组）
- IN-CR-01 地址替换检测（对话历史对比 + Levenshtein）
- IN-CR-02 危险工具调用拦截（bash rm -rf / curl|sh / eval）
- IN-CR-05 签名工具 fail-closed（signTransaction / signTypedData 全拦）
- YOLO mode 不可关闭机制
- 大量 SSE 边界 fuzz test（跨 chunk / C0 控制字符 / 提前断流）

**依赖**：Week 2 出站规则引擎基础
**关键风险**：SSE 流式边界处理复杂，半行 chunk 必须 fuzz 覆盖，否则规则绕过
**关联文档**：PRD §9 硬约束 #5

---

### Week 4 · 入站通用 + benchmark 数据集

**完成定义**（PRD §10.1 Week 4）：

- benchmark 数据集跑通，所有 Critical 规则达到 FP 阈值

**任务清单**：

- IN-CR-03 敏感路径访问（~/.ssh / ~/.aws / .env / keystore）
- IN-CR-04 持久化机制（crontab / launchd / systemd / .bashrc）
- IN-GEN-01~05 全部 P0 通用规则（shell 危险模式 / 远程脚本 / 编码执行 / Markdown exfil）
- 处置矩阵完整实现（Critical block / High warn 5s / Medium 标记）
- CLI 弹窗 + 命令行确认交互
- **benchmark 数据集构建**
  - 200-500 条合成攻击样本（UCSB 4 类 + drainer + Pink Drainer 数字化 + npm typosquat + curl|sh + eval base64）
  - 50-100 条真实 benign 会话回放（doskey 自己 Claude Code 日常工作录制）
  - canary 测试（假 BIP39 / 假地址 / 假 selector / honeypot 钱包）
  - 验证 Critical FP < 0.5%, High FP < 5%

**依赖**：Week 3 SSE + 工具调用处理
**关键风险**：benign 会话标注耗时，需提前准备。假 sample 生成需谨慎，防止反向泄露
**关联文档**：PRD §9 公理 12 / §10.1 Week 4 数据集具体化

---

### Week 5 · 打磨 + 配置 + 文档

**完成定义**（PRD §10.1 Week 5）：

- doskey 朋友 30 分钟内能 brew install + 配好

**任务清单**：

- 完整配置系统（config.toml + 环境变量覆盖）
- 日志和审计输出（本地 SQLite append-only）
- 完整用户文档（Claude Code 接入教程 + FAQ）
- License 验证机制 + 14 天试用机制
- 规则库脱敏配置 + 空规则集容错
- brew tap 仓库初始化
- GitHub Releases 自动化构建上传

**依赖**：Week 4 核心引擎稳定
**关键风险**：配置系统过度设计，需把握简洁度
**关联文档**：PRD §11.3 开源策略

---

### Week 6 · doskey 自用 + 修 bug（Windows / Linux 二进制 Tier 2）

**完成定义**（PRD §10.1 Week 6）：

- doskey 自己一周无 P0 bug，FP 触发次数 < 5 次
- Windows / Linux 二进制编译完成

**任务清单**：

- doskey 100% 工作时间用 Sieve
- 收集所有 false positive，加 .sieveignore 默认条目
- 性能 benchmark 验证 P99 < 20ms
- Windows 二进制编译 + 本地测试
- Linux 二进制编译 + 虚拟机测试
- 跨平台路径处理规则（windows 盘符 / UNC 路径）
- 二进制签名（cosign sigstore）

**依赖**：Week 5 配置 + 文档完整
**关键风险**：Windows Defender SmartScreen 新二进制阻挡，需提前规划
**关联文档**：PRD §6.4 性能预算

---

### Week 7-8 · 高强度 dogfood + Stripe 接入

**完成定义**（PRD §10.1 Week 7-8）：

- doskey 用 Sieve 跑 2 周，无 P0 / P1 bug
- Stripe 账号 + license key 系统上线

**任务清单**：

- 刻意尝试 edge case（多终端 / 跨语言 / 网络延迟模拟）
- 每次 FP 都进 issue 列表 + 修复
- 规则库更新机制完整测试
- 第一次签名规则库下发测试
- 海外公司（香港/新加坡）注册完成
- Stripe 账号关联公司主体
- license key 生成 + 验证系统
- 14 天试用激活机制
- 降级模式（试用期后只读警告）实现

**依赖**：Week 6 二进制稳定，公司注册已启动（Week 1）
**关键风险**：公司注册周期延误（4-6 周），必须 Week 1 启动以赶上 Week 7-8 Stripe
**关联文档**：PRD §7.1 定价 / §11.5.1 法律实体

---

## Phase B · 闭测（Week 9-12）

### Week 9 · 闭测启动（画像精确化）

**完成定义**（PRD §10.2 Week 9）：

- 5-10 个精准闭测用户入场
- Discord 闭测频道建立，日常反馈机制就位

**闭测用户画像必须满足**（PRD §10.2 Week 9，v1.3 修订）：

- 高频 hackathon builder（ETHGlobal / Solana 常客，单 hackathon 写 10+ 合约）
- bug bounty hunter / 审计研究员（Code4rena / Sherlock / Immunefi 活跃）
- 小团队 protocol engineer（< 10 人，决策快，口碑传播力强）
- **不要找**：大企业 dev / 纯 web2 友人 / 纯 KOL

**任务清单**：

- Week 9 邀请 5 人（具体名单 TBD，见 PRD §14 Open Questions）
- Discord 闭测频道建立 + 权限配置
- 闭测 license key 分配（5 个独立 key）
- 每日反馈处理 SLA（24 小时内回复）
- bug 优先级标记系统

**依赖**：Week 8 Stripe + license 系统完成
**关键风险**：闭测用户选错方向会浪费 4 周时间，必须严格筛选
**关联文档**：PRD §3.1 P0 客群 / §10.2 Week 9 用户画像

---

### Week 10 · 闭测 + 内容准备

**完成定义**（PRD §10.2 Week 10）：

- 3 篇引爆文章初稿完成
- 闭测 bug 修复，二轮邀请准备

**任务清单**：

- 修闭测 Week 9 发现的 bug
- 内容 1 草稿：中转站揭黑（实测复刻 UCSB 方法论）
- 内容 2 草稿：**自证清白**（Sieve 怎么证明自己不是新的 LiteLLM）
- 内容 3 草稿：Pink Drainer 攻击复盘 + Sieve 防御
- 文章 2 核心叙事：sigstore 签名 + reproducible build + 透明规则日志
- 在线编辑 / 技术审校流程建立
- 邀请函 v2（给 Week 11 闭测扩大）

**依赖**：Week 9 闭测反馈 + 内容素材积累
**关键风险**：文章 2 是核心营销弹药，品质必须高；技术细节必须正确
**关联文档**：PRD §10.2 Week 10 / §1.2 第 4 句自证清白叙事

---

### Week 11 · 闭测扩大 + KOL 接洽

**完成定义**（PRD §10.2 Week 11）：

- 邀请 5-10 个新闭测用户（同样画像标准）
- KOL 接洽启动，数据合作优先

**任务清单**：

- 邀请 5 个新闭测用户（同样 builder / hunter / engineer 画像）
- 闭测 license key 管理（10 个总的，追踪使用情况）
- landing page 初稿（英文为主，中文次之）
- 视觉设计（logo / 色系 / 品牌指南）
- **KOL 接洽启动**（PRD §10.2 Week 11，数据合作优先）
  - Chaofan Shou (@Fried_rice) 主动接洽（UCSB 论文一作，顾问关系）
  - 慢雾 @evilcos 接洽（misttrack-skills 数据合作）
  - 数据合作洽谈（SlowMist / ScamSniffer / GoPlus）
- GitHub stars 和 issue 管理流程

**依赖**：Week 10 内容初稿 + 闭测稳定反馈
**关键风险**：KOL 接洽容易流于虚客套，需提前准备具体合作方案（参见 PRD §13.2）
**关联文档**：PRD §13.2 数据侧合作清单 / §13.3 内容合作清单

---

### Week 12 · GA 发布

> ⚠️ 关联 ADR-011：GA 之前 repo 完全私有，GA 当天首次公开；Week 9-11 闭测期间所有反馈走 Discord 私发，不暴露 repo。

**完成定义**（PRD §10.2 Week 12）：

- GA 第一周 GitHub stars > 200
- 试用注册 > 100
- 首批付费用户 ≥ 10

**任务清单**：

- 代码开源（MIT License）
- 二进制 sigstore 签名验证文档
- reproducible build 验证指南
- landing page 上线（英文 + 中文）
- 文章 1（中转站揭黑）发表（Twitter / Hacker News / Mirror）
- 文章 2（自证清白）发表（Twitter / Hacker News / Mirror + 中文版）
- 文章 3（Drainer 复盘）发表（Week 14，较文章 1/2 滞后）
- 14 天试用全面开放
- Stripe 收款上线 + 首次处理付款
- README 国际化（英文 + 中文）
- 社区链接（Discord / GitHub Discussions）
- 发 Show HN / Hacker News post
- 在 r/ethdev / r/cryptocurrency 发帖

**依赖**：Week 11 内容完稿 + KOL 反馈
**关键风险**：GA 发布节奏（文章 1+2 同步 vs 错开）影响传播，需与 KOL 协调
**关联文档**：PRD §10.2 Week 12 / §11.5.2 营销渠道分级

---

## Phase C · 维护（Week 13+）

每周稳定投入 5-10 小时：

- **每月一篇深度内容**（内容 3 + 后续系列）
- **用户反馈处理 + bug 修复**（优先级排序，每周处理清单）
- **规则库每周更新一次**（签名 + changelog）
- **季度大版本**（Phase 2 功能逐项上线）
  - 中文 PII 检测
  - 用户自定义规则 DSL
  - npm / pip typosquat
  - MCP 拦截 + scope-aware policy（Week 16-20）
- **第二个用户主动要 OpenClaw / Hermes / MCP 适配时再做**（PRD §6.1 Phase 1 约束）

---

## 跨周关键依赖图

```
Week 1 sigstore pipeline ─┐
                          ├─→ Week 12 GA 二진制可独立验证
Week 1 海外公司注册启动 ─→ Week 7-8 Stripe 接入 ──┘

Week 1 Rust 骨架
  ↓
Week 2 出站 P0 规则
  ↓
Week 3 入站 Crypto 钩子
  ↓
Week 4 入站通用 + benchmark
  ↓
Week 5 配置 + 文档
  ↓
Week 6 二进制 + Windows Tier 2
  ↓
Week 7-8 dogfood + Stripe
  ↓
Week 9 闭测启动
  ↓
Week 10 内容准备
  ↓
Week 11 KOL 接洽
  ↓
Week 12 GA 发布
  ↓
Week 13+ 慢节奏维护 + Phase 2
```

---

## Open Questions（与 PRD §14 同步）

- 正式产品名（Week 6-8 之间必须定，codename: Sieve）
- 法律实体注册地最终选择（香港 vs 新加坡 vs Stripe Atlas）
- Week 9 闭测邀请名单（5 个具体名字：builder + hunter + engineer）
- 加密收款实现方案（Stripe Crypto / Coinbase Commerce / 自部署）
- 降级模式 UI 过渡设计（试用期后只读警告但不显得被坑）

---

## 维护规则

- **每周一上午**（执行期开始后）：勾选完成项 + 写一段 Weekly Note
- **任何任务延期超 1 周**：必须在本文加 🚨 标记 + 缓解方案
- **PRD §10 修订时**：本文同步更新，时间戳标注
- **与 [tasks/lessons.md](./lessons.md) 配合**：每次 lessons 更新都应反思是否要调 roadmap
- **每周回顾**：验证关键依赖是否被阻断，风险是否上升

---

## 相关文档

- [PRD v1.3 §10](../docs/prd/sieve-prd-v1.3.md#10-12-周里程碑8-周-dogfood--4-周闭测)
- [PRD v1.3 完整版](../docs/prd/sieve-prd-v1.3.md)
- [Lessons](./lessons.md)
- [README](../README.md)
- [ADR 索引](../docs/design/ADR-INDEX.md)（待创建）
- [.cursorrules](../.cursorrules)

