# ADR-023: 进程上下文记录——caller_pid / caller_exe 字段 + macOS 系统 API 反查

## 状态

**已接受**

> 决策日期：2026-05-01
> 范围：Phase A（Week 5-8 ship），macOS only；cwd / ppid 字段推 v2.1
> 关联 PRD：[v2.0 §5.6](../prd/sieve-prd-v2.0.md)、[v2.0 §6.6](../prd/sieve-prd-v2.0.md)、[v2.0 §12 R-V20-06](../prd/sieve-prd-v2.0.md)
> 关联 review：[codex review PRD v2.0 §D1 / §D2 OQ-V20-02 关闭](../review/2026-05-01-codex-review-prd-v2.0.md)

---

## 背景

### 为什么要记录进程上下文

v1.5 audit.db 只记录"命中了哪条规则 + 发送了什么请求"，不知道是哪个进程发出的请求。这带来两个问题：

1. **归因盲区**：三家 agent（Claude Code / OpenClaw / Hermes）并行时，audit 无法区分"是 Claude Code 发的恶意 tool_use，还是 Hermes 的正常调用"，威胁分析困难；
2. **行为序列缺前置条件**：v2.0 Phase B 的行为序列窗口（§5.7）需要"发起请求的进程"来避免跨 agent 串会话；没有 caller_pid 就无法在序列分析里做进程隔离。

PRD v2.0 §5.6 决定 Phase A 在 audit.db 新增 `caller_pid` + `caller_exe` 两个字段，"仅记录，不分析"，给 Phase B 行为序列分析喂数据。

### macOS 进程反查 API 选型（OQ-V20-02 关闭）

获取 TCP 连接对端 PID 在 macOS 上有两个思路：

**方案 A：shell out `lsof -i`**
- 优点：一行命令，实现简单
- 缺点：每次调用 fork + exec `lsof`，耗时 50~200ms，严重影响 daemon 的 P99 延迟；`lsof` 在 macOS Sandbox / App Sandbox 场景权限不稳定；无法满足"反查耗时不超过 1ms"要求

**方案 B：macOS 系统 API `proc_pidinfo` + `proc_pidpath`**
- `proc_pidinfo(pid, PROC_PIDTCPINFO, ...)` 直接查 TCP 连接表，找到匹配的本地端口后得到 PID
- `proc_pidpath(pid, buf, len)` 从内核拿进程可执行文件路径（等价于 `/proc/<pid>/exe`）
- 耗时 < 1ms（无 fork/exec，直接系统调用）
- 无需额外权限，比 `lsof` 更符合 macOS Sandbox 约束

**决策**：采用方案 B（系统 API），OQ-V20-02 关闭（codex review §D2 建议，已在 PRD §14 Open Questions 标记为 Resolved）。

### cwd / ppid 字段为何推 v2.1

`proc_pidinfo` 的 `PROC_PIDVNODEPATHINFO` variant 可以拿 `cwd`，`PROC_PIDTBSDINFO` 可以拿 `ppid`。但这两个字段在 macOS 上会触发系统 entitlements 授权弹窗（`com.apple.security.temporary-exception.files.home-relative-path.read-only`），属于 macOS App Sandbox 限制。

Phase A 目标是"不引入部署摩擦"（PRD §5.6 注释明确说明）。引入 entitlements 授权弹窗会让 `sieve setup` 的安装体验变差——与"朋友 30 分钟内能装通"的 Week 5 验收指标冲突。

**决策**：Phase A 只反查 `caller_pid` + `caller_exe`，`caller_cwd` / `caller_ppid` 推 v2.1（待 macOS 权限模型稳定后评估）。

---

## 决策

### 1. audit schema 扩展（两个新字段）

`audit.db` 的 `events` 表新增：

```sql
ALTER TABLE events ADD COLUMN caller_pid  INTEGER;  -- TCP 连接对端 PID；反查失败为 NULL
ALTER TABLE events ADD COLUMN caller_exe  TEXT;     -- 进程可执行文件绝对路径；反查失败为 NULL
```

**字段约束**：
- `caller_pid` 和 `caller_exe` 均允许 NULL（反查失败不阻塞 daemon）
- `caller_exe` 存完整路径（如 `/Applications/Claude.app/Contents/MacOS/Claude`），不截断，不脱敏
- `caller_exe` 存本地路径，**不上传**（PRD §11 数据不上传，PRD §9 #2 不联网 verifier）

v2.1 再加 `caller_cwd TEXT` / `caller_ppid INTEGER` 字段（需 macOS entitlements 授权）。

### 2. macOS 反查实现——系统 API 优先

反查逻辑封装在 `crates/sieve-cli/src/process_context.rs`（新模块）：

```rust
pub struct CallerInfo {
    pub pid: i32,
    pub exe: Option<PathBuf>,
}

/// 从 TCP 连接的本地端口反查 caller 进程信息（macOS 实现）
/// 反查失败返回 None，不 panic，不阻塞
pub fn lookup_caller(local_port: u16) -> Option<CallerInfo>;
```

macOS 实现路径：
1. `proc_pidinfo(pid, PROC_PIDTCPINFO)` —— 枚举所有 TCP 连接，匹配本地端口，得到 PID
2. `proc_pidpath(pid, buf, PROC_PIDPATHINFO_MAXSIZE)` —— 根据 PID 拿可执行文件路径

调用点：daemon 接受新的 HTTP 连接时（`accept()` 后），非阻塞调用 `lookup_caller`，失败 → 字段为 NULL，继续处理请求。

### 3. LRU cache（PID → CallerInfo，TTL 30s）

同一 agent 进程会在短时间内发出大量 HTTP 请求。每次都走系统 API 反查耗时虽然 < 1ms，但仍有 syscall 开销。使用 LRU cache 减少重复查询：

```rust
type CallerCache = LruCache<i32, (CallerInfo, Instant)>;
const CACHE_TTL: Duration = Duration::from_secs(30);
const CACHE_MAX_ENTRIES: usize = 64;  // 同时跑 64 个不同 PID 已足够
```

**cache 策略**：
- key = PID；value = (CallerInfo, 查询时间戳)
- 命中 cache 且 TTL 未过期 → 直接返回缓存值，跳过系统调用
- TTL 过期（进程可能已被替换为同 PID 新进程）→ 重新查询
- cache 满 → LRU 淘汰最久未访问项

**TTL 30s 理由**：macOS PID 复用周期通常 > 30s（进程启动到退出最短也有几秒），30s TTL 在正确性和性能之间平衡合理。

### 4. 反查失败不阻塞 daemon

以下情况反查失败，`caller_pid` / `caller_exe` 均为 NULL，daemon 正常继续：
- 权限不足（Sandbox 限制）
- 进程已退出（PID 反查到中途进程退出）
- 系统 API 返回错误码
- cache 查询耗时 > 1ms（超时保护，走 try 模式不 block）

**绝不允许**：反查失败导致 daemon panic / 返回 HTTP 5xx / 丢弃请求。

### 5. 隐私 / 安全约束

- `caller_exe` 是本地文件路径，**不上传**（PRD §11 数据不上传原则）
- `audit.db` 文件权限 `0600`（daemon 已有约束，新字段继承）
- 反查结果不用于任何网络请求 / 远端校验（PRD §9 #2 不联网 verifier）
- Phase A 不分析 `caller_exe` 内容（只记录路径），分析是 Phase B 行为序列的事

### 6. v2.1 升级路径

| 字段 | Phase A（v2.0）| v2.1（待 macOS 权限评估）|
|------|-------------|----------------------|
| `caller_pid` | ✅ 实现 | 不变 |
| `caller_exe` | ✅ 实现（`proc_pidpath`）| 不变 |
| `caller_cwd` | ❌ 推后 | 需 `PROC_PIDVNODEPATHINFO` + entitlements |
| `caller_ppid` | ❌ 推后 | 需 `PROC_PIDTBSDINFO` + entitlements |
| Linux `/proc/<pid>/exe` | ❌ v1.5 macOS only | v2.1 多平台扩展时实现 |
| Windows `QueryFullProcessImageName` | ❌ v1.5 macOS only | v2.1 多平台扩展时实现 |

---

## 影响

### 正面影响

1. **归因可追溯**：audit.db 记录 `caller_exe`，可区分 Claude Code / OpenClaw / Hermes 发出的请求，威胁分析能力提升；
2. **行为序列前置条件满足**：Phase B 的 `ToolUseSequence` 可按 PID 隔离不同 agent 的序列窗口，避免跨 agent 串会话（PRD §12 R-V20-08 应对）；
3. **低开销**：系统 API + LRU cache，正常路径耗时 < 1ms，满足性能约束；
4. **部署零摩擦**：不引入 entitlements 授权弹窗，`sieve setup` 安装体验不受影响。

### 负面影响

1. **macOS 限定**：Linux / Windows 下 `lookup_caller` 返回 `None`，`caller_pid` / `caller_exe` 字段为 NULL——Phase A 只做 macOS，多平台 v2.1 补；
2. **PID 复用误查风险**：进程退出后 PID 被快速复用，cache 里残留旧 CallerInfo——TTL 30s 缓解，但不能完全排除边界情况（audit 标注 WARNING 字段表明不确定性）；
3. **cwd / ppid 空缺**：行为序列的进程树分析（攻击者进程 fork 子进程）在 v2.0 无法做，依赖 v2.1 补完；
4. **schema migration**：`ALTER TABLE` 需要 daemon 启动时执行迁移，需测试旧 audit.db 升级场景。

### 需要更新的文档

- `docs/design/data-model.md` —— audit schema §events 表加 `caller_pid` / `caller_exe` 字段说明
- `docs/guides/development.md` —— 新增 `process_context.rs` 模块说明 + macOS API 权限约束
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目（ADR-023）

---

## 相关文档

- [PRD v2.0 §5.6 进程上下文记录](../prd/sieve-prd-v2.0.md)
- [PRD v2.0 §6.6 进程上下文反查](../prd/sieve-prd-v2.0.md)
- [PRD v2.0 §12 R-V20-06 进程归因错误风险](../prd/sieve-prd-v2.0.md)
- [PRD v2.0 §9 #2 不联网做 verifier](../prd/sieve-prd-v2.0.md)
- [codex review v2.0 §D2 / OQ-V20-02](../review/2026-05-01-codex-review-prd-v2.0.md)
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed 不阻塞上下文（反查失败不能阻塞请求处理）
- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议（audit 事件写入路径）
