# ADR-020: 用户规则系统——单文件 user.toml + $EDITOR + fail-safe 加载

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase A (Week 5-8)，`sieve-policy` crate 落地
> 关联 PRD：v2.0 §5.5、§9 #14

---

## 背景

### 触发原因

v2.0 HIPS 改造的核心差异点之一是"可编程 policy 引擎"。v1.5 所有规则均内置 TOML、编译进二进制，用户无法添加私有检测逻辑，这在 v2.0 需要解决。

典型用户需求（场景 G，PRD v2.0 §4.7）：DeFi 协议 dev 想拦截"EIP-712 permit deadline=0 永不过期"这类项目特有 phishing 模式，但 Sieve 系统规则不可能覆盖每个用户的业务语义。

### 两个被否决的草案

**草案 v1（多文件方案）**：用户在 `~/.sieve/rules/` 下创建 `user-001.toml` / `user-002.toml` 等多个文件，daemon 启动时遍历加载。被否决原因：用户分散管理多个规则文件时上下文割裂（"我哪条规则在哪个文件"），规则之间关联性差，编辑器实现复杂。

**草案 v2（ratatui TUI 编辑器）**：内置 ratatui TUI，提供语法高亮 + schema 提示 + 实时 lint + 正则解释器。被否决原因：工程量等价于一个小产品（5-7 天），引入 ratatui 依赖增加编译体积、跨终端兼容性问题；用户用熟悉的 vim/code 效率更高；等量工程力放在 LayeredEngine + 安全约束收益更大。

### 核心约束

用户规则必须满足两条根本约束，两者缺一不可：

1. **安全性**：用户规则不能削弱系统 Critical 防护（PRD §9 #3/#8/#14）
2. **隔离性**：用户规则加载失败不影响 daemon 启动和系统规则工作（PRD §9 #14）

---

## 决策

### 1. 单文件 schema（否定多文件方案）

用户规则统一存于 `~/.sieve/rules/user.toml`，编辑前自动备份（最近 10 份）：

```
~/.sieve/rules/
├── user.toml                        # 统一用户规则文件
└── user.toml.bak.YYYYMMDD-HHMMSS   # 编辑前自动备份
```

文件顶层包含 `schema_version`、`created_at`、`updated_at` 三个元字段 + `[[rules]]` 数组。每条规则必填 `id`、`pattern`、`severity`、`action`、`keywords`（keywords 非空强制，启用预过滤性能优化）。

### 2. $EDITOR 方案（否定 ratatui TUI）

`sieve rules edit` 子命令调用 `$EDITOR`（fallback 顺序：`$VISUAL` → `$EDITOR` → `vim` → `nano`）打开 user.toml。编辑器退出后由 daemon 接手执行 4 步流水线：

| 步骤 | 行为 | 失败策略 |
|------|------|---------|
| 1. **lint** | 执行全部 11 类安全约束校验（§5.5.3） | lint 失败：保留原文件不变，stderr 打印违规清单（带行号 + 原因） |
| 2. **backup** | lint 通过后把原 `user.toml` rename 为 `user.toml.bak.YYYYMMDD-HHMMSS` | 保留最近 10 份，超出自动删除最旧份 |
| 3. **atomic write** | 写 `user.toml.tmp` 再 rename 到 `user.toml`（防止 daemon reload 读半写状态） | rename 失败：回滚 backup，通知用户 |
| 4. **reload** | IPC notify daemon 重新加载用户规则 | 失败：保留旧 user engine，不影响系统规则 |

### 3. 4 个核心子命令（MVP）

| 子命令 | 行为 |
|--------|------|
| `sieve rules edit` | 调用 `$EDITOR` + lint + backup + atomic write + reload |
| `sieve rules list` | 列出 user.toml 中所有规则 + 系统规则合并视图，标注来源（`[system]` / `[user]`）|
| `sieve rules disable <id>` | 在 user.toml 给该规则加 `enabled = false`（不删除规则条目）|
| `sieve rules enable <id>` | 反向操作，把 `enabled = false` 改为 `true` |

v2.1 评估追加：`lint`（独立校验）/ `import` / `export`（团队分享）/ `reload`（无 edit 上下文时手动触发）。

### 4. 用户规则能力边界（fail-safe 的来源）

用户规则**只能表达** High / Medium / Low × Ask / Warn / Mark / StatusBar 的处置组合，**明确禁止**以下任何组合：

| 禁止项 | 原因 |
|--------|------|
| `severity = "critical"` | 用户不能声明 Critical 等级规则 |
| `action = "block"` | 用户规则不能硬阻断（Block 只属于系统规则） |
| `disposition = "hook_terminal"` | HookTerminal 属于系统双层防御机制（ADR-014），用户无权调用 |
| `pattern` 含 `__` 前缀占位符 | 防止用户绕开 BIP39 / address guard 二次验证 |
| `id` 与系统规则或现有用户规则重复 | 防止 override / 重复加载 |
| `disposition = "auto_redact"` 用于入站方向 | 用户不能改写入站 model 输出（PRD §9 #11） |
| `allowlist_*` 字段试图豁免系统 Critical rule_id | 用户 allowlist 仅作用于自己的规则，lint 阶段强制拦截 |

LayeredEngine 合并顺序（ADR-020 强约束，对应 PRD v2.0 §6.3.1）：系统规则 Critical 命中时立即返回，不给用户规则机会 suppress；用户规则在系统规则全部 Allow 后才追加，且仅能追加 Ask/Warn/Mark 结果，不能修改已有 hits 的 action。

### 5. 11 类安全约束（lint + daemon 双重校验）

**A. 语义边界**（5 类）：第 4 节已全部列出的禁止组合。

**B. 资源上限**（3 类）：

| 限制 | 默认值 |
|------|------|
| 单个 `pattern` 编译时间上限 | 100ms（防 ReDoS） |
| 单个 `pattern` 编译后 vectorscan db size 上限 | 1MB（防 alternation 爆炸） |
| user.toml 单文件大小上限 | 1MB |
| user.toml 总规则条数上限 | 200 条 |
| `allowlist_stopwords` 单字符串长度下限 | 4 字节（防超短停用词污染所有匹配） |
| `keywords` 字段必填且非空 | 强制启用 keywords 预过滤，避免 match-all pattern 拖慢扫描 |

**C. 文件系统安全**（3 类）：

| 约束 | 实施方式 |
|------|--------|
| 文件权限 `0600`（owner-only） | `sieve rules edit` 保存时强制设置；daemon 加载时拒绝非 0600 文件 |
| 目录权限 `0700` | 同上 |
| No-follow symlink | `~/.sieve/rules/` 下任何符号链接拒绝加载（防恶意 LLM 诱导 `ln -s /etc/passwd user.toml`） |
| Atomic rename 写入 | 先写 `user.toml.tmp` 再 rename（防 daemon reload 读半写状态） |
| TOCTOU 防护 | daemon reload 时持有文件锁 + 重新计算 inode/mtime（防 check-then-use 间隙文件替换） |

违反上述任何约束：写 audit ERROR + GUI 状态栏通知 + 该规则跳过加载，**daemon 正常启动 + 系统规则全功能**。

### 6. Fail-safe 加载（PRD §9 #14 的工程实现）

PRD §9 #14 硬约束：用户规则文件加载失败不得影响 daemon 启动和系统规则功能。工程实现：

- `sieve-policy` crate 独立于 `sieve-rules`；系统规则编译进二进制，用户规则运行时加载
- daemon 启动时先建立系统 `VectorscanEngine`（失败则 `process::exit(1)`，沿用 ADR-007 §2 fail-closed），再尝试加载 `user.toml`（失败仅写 ERROR + GUI 通知，daemon 继续用 `LayeredEngine { system, user: None }` 形态）
- IPC reload 时失败保留旧 user engine，不影响系统规则
- 测试覆盖 5 类破坏场景：语法错误 / pattern 编译失败 / lint 违规 / 文件权限错 / 磁盘空间满

---

## 影响

### 正面影响

1. **用户差异化防护**：DeFi 开发者能在 30 分钟内加第一条项目特有规则（验收标准 §4.7）；
2. **系统规则完全隔离**：用户规则任何形式的失败都不能拖累系统 Critical 防护，PRD §9 #14 工程落地；
3. **工程量可控**：$EDITOR 方案比 ratatui TUI 节省约 5-6 天，Phase A 工期可控；
4. **LayeredEngine 可测试**：trait 化后系统规则 + 用户规则两层可独立 unit test + benchmark，满足 HIPS 规则引擎抽象标准；
5. **可编程差异化检测**：用户规则提供"可编程 HIPS"能力，让用户在系统规则之外按自身场景"再严一档"。

### 负面影响

1. **无实时 lint 提示**：$EDITOR 方案无法在用户输入时实时提示语法错误，只在保存后 lint；引导用户看 `docs/guides/user-rules.md` 里的示例，自己复制改；
2. **v2.1 依赖**：匹配预览 / 正则解释器 / 示例库 / GUI 编辑器全部推 v2.1，高级用户第一版体验有限；
3. **新 crate 维护**：`sieve-policy` 新增 crate 边界需要维护 trait 接口，fuzz test 覆盖范围扩大。

### 需要更新的文档

- `docs/design/architecture.md` —— 加 `sieve-policy` crate 职责 + LayeredEngine 数据流
- `docs/api/api-reference.md` —— 加 `sieve rules` 子命令 CLI 参考
- `docs/guides/user-rules.md`（新建）—— 用户规则写法 + 安全约束示例 + FAQ
- `CLAUDE.md` 五个 Crate 表格 —— 补 `sieve-policy` 行

---

## 相关文档

- PRD v2.0 §5.5 —— 用户规则系统完整需求
- PRD v2.0 §4.7 —— 场景 G：高级用户写自定义规则（验收标准）
- PRD v2.0 §9 #14 —— 用户规则 fail-safe 硬约束
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（用户规则不能 override 的根基）
- [ADR-021](./ADR-021-tri-state-decision-and-graylist.md) —— 三态决策 + 灰名单（用户规则的 Remember 权限在此定义）
- [architecture.md](./architecture.md) —— Pipeline 模块与 sieve-policy 关系
- [data-model.md](./data-model.md) —— 处置矩阵编码、LayeredEngine 合并顺序
