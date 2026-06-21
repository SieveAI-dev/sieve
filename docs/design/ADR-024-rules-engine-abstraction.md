# ADR-024: 规则引擎抽象——MatchEngine trait 重设计 + LayeredEngine 合并顺序

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase A（Week 5-8 ship）；`sieve-rules` crate trait 层 + 新增 `sieve-policy` crate
> 关联 PRD：v2.0 §6.3、v2.0 §5.5.2.1、v2.0 §9 #3

---

## 背景

### v1.5 规则引擎的现状与问题

v1.5 的 `MatchEngine` trait 定义于 `crates/sieve-rules/src/engine/mod.rs`，接口签名为：

```rust
fn scan(&self, bytes: &[u8]) -> SieveRulesResult<Vec<MatchHit>>;
fn last_scan_us(&self) -> u64;
```

这个接口在 v1.5 单一系统规则场景下够用，但 v2.0 引入用户规则（`sieve-policy` crate）后暴露三个根本性问题：

**问题 1：无路由上下文，无法识别生效路径**

v1.5.4 P0 修复（CHANGELOG v1.5.4）已证明：daemon 有多条分叉路径（Anthropic SSE / Anthropic JSON / OpenAI SSE / OpenAI stream=false），每条路径对应不同的 content-type 路由和解析逻辑。引擎收到 `&[u8]` 时完全不知道自己在哪条路径上，无法：
- 区分"入站规则应只看 tool_use_input"还是"出站规则看 request_body"
- 生成包含路由信息的 fingerprint（灰名单匹配的前置条件，PRD §5.4.2）
- 在 ScanReport 里注明"本次扫描在哪条 content-type 路径上命中"

**问题 2：性能指标 `last_scan_us` 暴露在 trait 上，设计错误**

`last_scan_us()` 是 mutable 状态方法——每次 `scan` 后更新，下次 `last_scan_us()` 调用读取。这在多线程并发 `scan` 场景下有数据竞争风险（需要额外锁），且语义不清晰（"哪次 scan 的耗时？"）。

**问题 3：用户规则引擎与系统规则引擎合并顺序无约束**

v1.5 只有系统规则引擎。v2.0 引入 `sieve-policy` 的用户规则引擎后，必须明确定义合并顺序：系统 Critical 命中时，用户规则能否 suppress？现有 trait 不提供这一保证，如果实现层顺序写错，可能让用户规则豁免系统 Critical（违反 PRD §9 #3 fail-closed）。

### v1 草案 `scan(&[u8])` 接口撤回

v2.0 规划初期曾讨论过"保持 `scan(&[u8])` 不变，在 caller 侧注入上下文"的方案。该方案的问题是：调用方需要重复注入上下文逻辑，LayeredEngine 的合并顺序无法在 trait 层强制约束。**本 ADR 正式撤回 v1 草案接口，改为 `scan(ScanRequest) -> ScanReport`。**

---

## 决策

### 1. ScanRequest：带上下文的扫描请求

```rust
/// v2.0 重设计：带完整路由上下文的扫描请求
/// 解决 v1.5 scan(&[u8]) 无法识别生效路径的问题
pub struct ScanRequest<'a> {
    pub bytes: &'a [u8],

    // 路由上下文（来自 v1.5.4 P0 修复的教训：必须传递路径信息）
    pub direction:    Direction,    // Inbound | Outbound
    pub protocol:     Protocol,     // Anthropic | OpenAI
    pub content_kind: ContentKind,  // SseEventDelta | JsonResponseBody | ToolUseInput | RequestBody

    // 业务上下文（用于 fingerprint 计算、灰名单查询、序列窗口）
    pub tool_name:    Option<&'a str>,   // "Bash" / "Read" / ... 仅 ToolUseInput 有效
    pub source_agent: Option<&'a str>,   // "claude-code" / "openclaw" / "hermes"
    pub caller_exe:   Option<&'a Path>,  // 进程上下文（§5.6，ADR-023）
}
```

**设计要点**：
- `direction` + `protocol` + `content_kind` 三元组唯一标识请求所在的 content-type 路由分支——与 PRD §9 #16 的 4 类测试矩阵对应
- `caller_exe` 来自 ADR-023 进程上下文记录，允许 None（反查失败不阻塞）
- 生命周期参数 `'a` 避免无谓 clone，bytes + str 借用 caller 的 buffer

### 2. ScanReport：内联性能指标，废弃 last_scan_us trait 方法

```rust
/// v2.0 重设计：扫描结果内联指标，废弃 last_scan_us trait 方法
pub struct ScanReport {
    pub hits:        Vec<MatchHit>,
    pub elapsed_us:  u64,           // 本次扫描耗时（与请求绑定，无多线程竞争）
    pub engine_name: &'static str,  // 引擎标识（"system-vectorscan" / "user-vectorscan"）
    pub rule_count:  usize,         // 本次评估的规则数（便于 bench 回归分析）
}
```

`elapsed_us` 随每次 `scan` 返回，由 caller 决定如何使用（写 audit / 打 log / 传 benchmark）。废弃 `last_scan_us()` trait 方法，解决多线程竞争问题。

### 3. MatchEngine trait 重设计

```rust
pub trait MatchEngine: Send + Sync {
    /// 单次扫描（v2.0 重设计：带上下文）
    fn scan(&self, req: &ScanRequest<'_>) -> SieveRulesResult<ScanReport>;

    /// 批量扫描（用于压力测试 / 数据集回归）
    fn scan_batch(&self, reqs: &[ScanRequest<'_>]) -> SieveRulesResult<Vec<ScanReport>> {
        reqs.iter().map(|r| self.scan(r)).collect()
    }

    /// 引擎元信息（启动时一次性查询，不在 hot path）
    fn engine_name(&self) -> &'static str;
    fn rule_count(&self) -> usize;
    fn compiled_pattern_size_bytes(&self) -> usize;
}
```

`scan_batch` 提供默认实现（顺序 scan），引擎可 override 为并行版本；`engine_name` 等元信息方法不在请求处理 hot path 上，允许字符串分配。

### 4. LayeredEngine：合并顺序强约束

`LayeredEngine<S: MatchEngine, U: MatchEngine>` 实现 `MatchEngine`，内部封装系统规则引擎（S）+ 可选的用户规则引擎（U）：

| 优先级 | 评估顺序 | 行为 | 不变量 |
|--------|---------|------|-------|
| 1 | 系统规则全量扫描 | 收集所有 hits | 用户规则**不参与**这一步 |
| 2 | 系统规则 Critical 命中判断 | **立即返回 ScanReport**，跳过用户规则 | 用户规则**永远不能** suppress 系统 Critical |
| 3 | 系统规则非 Critical 命中 | 收集 hits，**继续评估用户规则** | 顺序不可倒转 |
| 4 | 用户规则扫描（仅第 2 步未触发时）| 追加 hits，加 `user:` 前缀 | 用户规则只能加 Ask/Warn/Mark |
| 5 | 灰名单查询 | 仅对非 Critical 系统规则 + 用户规则有效 | Critical 命中时灰名单被跳过（PRD §5.4.2 Critical 锁）|

**工程保证**：
- LayeredEngine 的 `scan()` 实现对步骤 2 有显式 early return，代码路径在 `is_system_critical()` 后分叉
- `is_system_critical(rule_id)` 查询 `critical_lock.rs::FAIL_CLOSED_RULES`（ADR-007 已有实现），不做重复判断
- 用户规则引擎为 `Option<U>`，`None` 表示用户规则未加载或加载失败（daemon 正常运行，系统规则全功能）

### 5. 用户规则 allowlist_* 字段的 lint 约束

用户规则的 `allowlist_stopwords` / `allowlist_patterns` 字段由 `sieve-policy` 的 lint 在加载时验证：**不能包含任何系统 Critical rule_id**。

实施方式：
- lint 阶段枚举 `allowlist_*` 内容，对每项调用 `is_system_critical()` 检查
- 违规 → 写 audit ERROR + GUI 状态栏通知 + 该规则跳过加载（PRD §9 #14）
- 目的：即使 lint 通过，LayeredEngine 的合并顺序也保证用户 allowlist 不影响系统规则评估——两道防线

### 6. 与 PRD §9 #3 fail-closed 的关系

本 ADR 是 ADR-007 fail-closed 原则在用户规则系统引入后的工程落地：

- ADR-007 建立"Critical 规则不可关闭"的产品承诺
- 本 ADR 建立"LayeredEngine 合并顺序保证用户规则永远不能 suppress 系统 Critical"的工程不变量
- 两者共同构成 PRD §9 #3 在 v2.0 的完整实现

### 7. 独立测试与压力测试

新增 benchmark（`crates/sieve-rules/benches/`）：

| Benchmark 文件 | 目标 |
|---------------|------|
| `scan_70_rules.rs` | 70 条系统规则全量扫描，5KB 输入，P99 < 1ms |
| `scan_with_user_rules.rs` | 70 系统 + 30 用户规则（LayeredEngine），overhead < 20% |

CI 加 `cargo bench --no-default-features --features ci-bench` job，P99 退化 > 10% 失败（PRD §6.3.2）。

---

## 影响

### 正面影响

1. **路由感知**：`ScanRequest` 携带 `direction` / `protocol` / `content_kind`，引擎扫描结果与 v1.5.4 P0 修复的 content-type 路由强绑定，新功能接入时不会静默跳过 JSON / stream=false 路径；
2. **合并顺序不变量**：LayeredEngine 在 trait 层强制系统规则先行，用户规则永远不能 suppress Critical——PRD §9 #3 在工程层真正成立；
3. **可独立测试**：`MatchEngine` trait 可 mock，系统规则引擎 / 用户规则引擎 / LayeredEngine 各自可以独立单测 + bench，不依赖 daemon 启动；
4. **性能指标清晰**：`ScanReport.elapsed_us` 随请求返回，多线程无竞争，benchmark 数据可靠。

### 负面影响

1. **v1.5 接口不兼容**：`scan(&[u8])` → `scan(&ScanRequest)` 是 breaking change，`sieve-cli` 的 `engine_adapter.rs` 调用点全部需要同步更新（Phase A Week 5 必须做）；
2. **ScanRequest 构造负担**：每次调用前 caller 需要填充 `direction` / `protocol` / `content_kind`，代码量略增——但这是必要成本（v1.5.4 P0 的教训就是不传路由上下文导致漏洞）；
3. **`last_scan_us` 废弃**：若有外部消费 `last_scan_us()` 的代码（当前只有内部 bench），需同步迁移到 `ScanReport.elapsed_us`。

### 需要更新的文档

- `docs/design/architecture.md` —— §规则引擎 crate 边界，加 `ScanRequest` / `ScanReport` / `LayeredEngine` 模块关系图
- `docs/design/data-model.md` —— 加 `ScanRequest` / `ScanReport` schema 说明
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目（ADR-024）
- `.cursorrules §3.3` —— crate 边界说明加 `sieve-policy` 调 `sieve-rules` 的 trait 调用约束

---

## 相关文档

- PRD v2.0 §6.3 规则引擎抽象
- PRD v2.0 §5.5.2.1 用户规则与系统规则并存原则
- PRD v2.0 §9 #3 fail-closed High-Risk Tool Policy Gate
- [CHANGELOG v1.5.4](../changelog/CHANGELOG.md) —— content-type 路由 P0 修复，ScanRequest 上下文字段的直接来源
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 原则（LayeredEngine 不变量的上层依据）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（LayeredEngine 是代理层规则引擎的合并抽象）
- [ADR-023](./ADR-023-process-context-audit.md) —— 进程上下文（`ScanRequest.caller_exe` 来源）
