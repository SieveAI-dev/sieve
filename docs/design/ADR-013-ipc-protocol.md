# ADR-013: IPC 协议（JSON-RPC over Unix socket + 文件锁 JSON 文件）

## 状态

**已接受**

> 决策日期：2026-04-28
> 范围：Phase 1 代理进程 ↔ GUI App 及代理进程 ↔ sieve-hook 两条 IPC 通道
> 关联 PRD：[v1.4 §6.5、§10.1 Week 3 + Week 5](../prd/sieve-prd-v1.5.md)

---

## 背景

Sieve Phase 1 引入了三个需要跨进程通信的组件：

1. **`sieve-cli` 守护进程**（Rust）——持续运行的 SSE 代理 + 检测引擎
2. **`sieve-gui-macos` GUI App**（Swift，独立仓库）——HIPS 弹窗渲染 + 用户授权收集
3. **`sieve-hook`**（Rust，独立 crate）——Claude Code PreToolUse hook，每次工具调用时 fork

三者之间的通信需求不同：

| 通信方向 | 频率 | 延迟预算 | 内容 |
|---------|------|---------|-----|
| 代理 → GUI App | 低（仅 GUI 类规则命中时）| 500 ms 内 GUI 弹出 | 检测上下文（地址对比 / typed data）+ 授权请求 |
| GUI App → 代理 | 低（用户决策后一次性）| 用户操作延迟 | allow / deny |
| 代理 → sieve-hook | 高（每次 PreToolUse 前）| 代理写文件 < 5 ms | pending 标记 + 规则上下文 |
| sieve-hook → 代理 | 高（用户决策后一次性）| hook exit 前 < 5 ms | allow / deny |

这两类通信需求差异极大，**不宜用同一种机制实现**。

### 为什么代理 ↔ GUI App 用 Unix socket + JSON-RPC

- GUI App 是常驻进程，socket 连接在会话内保持，避免每次握手开销；
- JSON-RPC 2.0 是现有 IDE/服务端生态通用协议，减少自定义协议的解析 bug；
- Unix socket 路径固定（`~/.sieve/ipc.sock`），不占用 TCP 端口，无防火墙干扰；
- 比命名管道（macOS named pipe = FIFO）更健壮，支持双向双工。

### 为什么代理 ↔ sieve-hook 用文件锁 + JSON 文件

- sieve-hook 是 **每次 PreToolUse 都 fork 的进程**，启动时延 < 50 ms 是硬约束（见 ADR 任务规格 Q2）；
- fork 出来的 hook 进程生命周期极短（几秒），建立 socket 连接并握手需要 tokio，引入异步运行时后启动时延膨胀 > 200 ms；
- 文件 IO + fd-lock 是 `sieve-hook` 依赖只有 `serde_json` + `fd-lock` 的物质基础；
- pending file 还可在 hook 进程崩溃时做幂等性保护：代理侧超时后按 `default_on_timeout` 决策，不会挂起。

---

## 决策

### 1. 双 IPC 通道

#### 通道 A：代理 ↔ GUI App（JSON-RPC 2.0 over Unix socket）

**路径**：`~/.sieve/ipc.sock`

**协议**：JSON-RPC 2.0（无 batch，单请求单响应 + 服务端主动 notify）

**版本握手**：连接建立后服务端（代理）发 `sieve.hello` 通知，包含 `protocol_version: "v1"`；GUI App 如不认识该版本则断开并提示用户升级。

**核心消息**：
- `sieve.request_decision`（代理 → GUI）：请求用户对检测命中做授权决策
- `sieve.decision_response`（GUI → 代理）：用户决策结果（allow / deny）
- `sieve.event_notify`（代理 → GUI）：非阻塞通知（菜单栏状态更新等）

**request_decision 负载示意**（完整 schema 见 SPEC-002）：
```json
{
  "jsonrpc": "2.0",
  "method": "sieve.request_decision",
  "params": {
    "request_id": "<uuid>",
    "rule_id": "IN-CR-05",
    "disposition": "GuiPopup",
    "timeout_seconds": 120,
    "default_on_timeout": "Block",
    "context": { ... }
  },
  "id": "<request_id>"
}
```

#### 通道 B：代理 ↔ sieve-hook（文件锁 + JSON 文件）

**目录结构**：
- `~/.sieve/pending/<request_id>.json` —— 代理写，hook 读
- `~/.sieve/decisions/<request_id>.json` —— hook 写，代理读（inotify/kqueue 监听）

**pending 文件格式**（完整 schema 见 SPEC-001）：
```json
{
  "schema_version": "v1",
  "request_id": "<uuid>",
  "rule_id": "IN-CR-02",
  "disposition": "HookTerminal",
  "timeout_seconds": 30,
  "default_on_timeout": "Block",
  "context": { ... }
}
```

**decision 文件格式**：
```json
{
  "schema_version": "v1",
  "request_id": "<uuid>",
  "decision": "allow" | "deny",
  "decided_at": "<iso8601>"
}
```

**文件锁语义**：代理用 `fd-lock` 写 pending 文件时持有 exclusive lock；hook 读时持有 shared lock，防止读到半写文件。

### 2. 代理侧并发管理

代理侧维护 `HashMap<request_id, oneshot::Sender<Decision>>`，接收 GUI decision response 或检测到 decision 文件后发送到等待的 Future。

超时计时器与 oneshot 共同 `select!`——先到先得：
- 用户决策到：发送用户决策，清理 pending/decision 文件；
- 超时到：发送 `default_on_timeout`，写 `~/.sieve/ipc.log` 记录超时事件，清理 pending 文件。

### 3. 超时与默认值

超时秒数和超时默认动作来自规则 manifest 的 `timeout_seconds` 和 `default_on_timeout` 字段（见 ADR-016）。代理侧不硬编码超时值，保持规则驱动。

| 场景 | 典型 timeout_seconds | default_on_timeout |
|------|--------------------|--------------------|
| IN-CR-05 签名 GUI 弹窗 | 120 | Block |
| IN-CR-01 地址替换 GUI 弹窗 | 60 | Block |
| IN-CR-02 Hook 终端 | 30 | Block |
| IN-GEN-04 markdown exfil GUI | 60 | Block |

### 4. 协议版本管理

- 协议版本号从 `v1` 起，向后不兼容时递增；
- GUI App 只接受自己已知的版本，拒绝未知版本时弹窗提示用户升级；
- sieve-hook schema_version 相同机制——hook 读到未知 schema_version 时 exit 1（fail-closed）。

### 5. 安全边界

- Unix socket 权限 `0600`（仅 owner 读写），防止同机其他进程接入；
- pending/decisions 目录权限 `0700`；
- JSON 文件不包含私钥或签名数据——context 只包含工具调用参数摘要，完整数据留在代理内存。

---

## 影响

### 正面影响

1. **sieve-hook 启动 < 50 ms**：文件 IO 路径不引入 tokio，二进制最小化；
2. **进程解耦**：GUI 崩溃时 pending 文件超时后 fail-closed，代理不挂起；
3. **协议可进化**：version 字段保证向后不兼容升级有明确握手；
4. **可观测**：pending / decisions 目录可用于调试和审计（doctor 命令可直接检查）。

### 负面影响

1. **两套 IPC 机制维护成本**：socket 和文件锁各需独立错误处理；缓解：SPEC-001 和 SPEC-002 严格规范 schema，减少实现歧义；
2. **文件系统依赖**：`~/.sieve/` 目录权限异常时会影响 hook 通道；doctor 命令检查此目录；
3. **kqueue/inotify 差异**：macOS 用 kqueue 监听 decisions 文件，Linux Phase 2 换 inotify；Phase 1 macOS only 暂不处理；
4. **request_id 碰撞**：UUID v4 碰撞概率极低，但 pending 目录未清理时同名文件会被覆盖；代理侧写前检查文件不存在，存在则报错（防御性编程）。

### 需要更新的文档

- `docs/specs/SPEC-001-sieve-hook-protocol.md`（新建）—— pending/decisions 文件 schema 完整规范
- `docs/specs/SPEC-002-hips-popup-behavior.md`（新建）—— GUI 弹窗触发条件、倒计时、多 issue 合并
- `docs/design/data-model.md` §5 —— 配置 schema 加 `ipc_socket_path`
- `docs/api/api-reference.md` §6 —— hook exit code 表 + GUI decision response 格式

---

## 相关文档

- [PRD-sieve v1.4 §6.5](../prd/sieve-prd-v1.5.md) —— IPC 协议选型
- [ADR-012](./ADR-012-native-gui-app-phase1.md) —— GUI App 独立仓库（IPC 是两仓库协调契约）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（哪些规则走哪条 IPC 通道）
- [ADR-015](./ADR-015-sieve-setup-tool.md) —— sieve setup 负责创建 `~/.sieve/` 目录权限
- [architecture.md](./architecture.md) —— 整体架构图 IPC 通道
- [data-model.md](./data-model.md) —— IPC 相关配置字段
