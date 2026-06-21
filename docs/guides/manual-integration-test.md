# Sieve daemon · 手动联调 Checklist

> 上次更新：2026-05-05（增补 §14 sieve-updater 联调,域名落定 sieveai.dev）
> 范围：unix-style 改造 v2.x 5 项 TODO + sieve-updater 客户端闭环 的用户验证步骤
> 关联：[ADR-026](../design/ADR-026-port-based-listener-routing.md) / [ADR-028](../design/ADR-028-ipc-protocol-neutralization.md) / [SPEC-006](../specs/SPEC-006-update-and-telemetry.md) / PROGRESS.md

---

## ⚙️ 自动化状态（2026-06-18）

> **本清单大部分已被自动化覆盖**，无需逐项手动跑。一键 hermetic 验证（无真 API key / 无网络 / 无 GUI）：
> ```bash
> scripts/dogfood.sh        # 构建 + cargo e2e + smoke + updater 闭环，全过即 dogfood 就绪
> ```
> 自动化映射（详见 [SPEC-008 dogfood 自动化](../specs/SPEC-008-dogfood-automation.html)）：
> - **§1 基线** → `cargo fmt/clippy/test/deny/build`（CI `fmt`/`clippy`/`test`/`deny` job）
> - **§3-§5 multi-listener/协议错位/doctor** → `crates/sieve-cli/tests/{multi_agent_routing,content_type_matrix,doctor}.rs`
> - **§6-§9 audit/decisions CLI + 决策流** → `crates/sieve-cli/tests/dogfood_e2e.rs`（出站脱敏/入站拦截/no-client-policy 三策略/mock-GUI 决策流/audit 闭环）
> - **§14 sieve-updater 闭环** → `crates/sieve-updater/tests/updater_e2e.rs`（install-id/fetch→download→sha256→zstd 解压→原子落盘/失败模式/公钥 None skip），**已替代 §14.3 的 caddy+mkcert 手动 mock**
> - **透传/SSE/tool_use/脱敏黑盒** → `python3 scripts/smoke_test.py --mock-only`
> - **§13 跨仓一致性** → GUI 仓 `IPCSchemaV2FixtureTests.swift`（81 fixture 消费测试，防漂移红线）
>
> **仍需手动**（自动化收益低/依赖外部）：§3.3/§10 真 API key 实打、GUI 可视层（菜单栏/Toast/Settings/Onboarding）、真 Claude Code 流量触发 OUT-*/IN-CR-*。
>
> ⚠️ 自动化首轮抓出多个真 bug（见 PROGRESS.md 🚫 段 + lessons.md 2026-06-18）：zstd 字节序（已修）、headless CLI 嵌套 runtime panic（已修）、detection 审计未接线 + 6 类跨仓 schema 漂移（待排期）。

---

## 0. 文档目的

把 2026-05-05 完成的 13 commits（unix-style 改造）+ sieve-updater 客户端闭环转化为**可逐项勾选**的人工验证步骤。所有步骤跑完且勾选 → daemon 侧 dogfood 就绪。GUI 仓侧的联调（sieve-gui-macos）见该仓自己的 checklist，本文档不覆盖。服务端尚未实施,§14 用本地 mock 服务器即可验证客户端独立闭环。

**前置假设**：你在 macOS（Phase 1 唯一 Tier 1 平台）；本仓 clone 到 `~/src/sieve-suite/sieve`；已经装了 Rust toolchain（见 `rust-toolchain.toml`）+ `sqlite3` CLI + `zstd` CLI（`brew install zstd`,§14 用）+ `python3`（§14 mock server 用,系统自带）+ `caddy` + `mkcert`（§14.3 https 反代,`brew install caddy mkcert`）。

---

## 1. 基线验证（先跑这个，全过才进 §2 起的功能验证）

```bash
cd ~/src/sieve-suite/sieve
```

- [x] **fmt clean**：`cargo fmt --all -- --check` exit 0
- [x] **clippy 0 issues**：`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` exit 0
- [ ] **workspace 测试 760 passed / 7 ignored / 0 failed**：`cargo test --workspace --locked`（含 sieve-updater 35 个新测试）
- [ ] **deny 检查通过**：`cargo deny check`（如未装：`cargo install cargo-deny --locked`）
- [ ] **build 干净**：`cargo build --workspace --release --locked`,release 二进制大小约 9 MB
- [ ] **七个 crate 都在**：`ls crates/` 应有 `sieve-cli/ sieve-core/ sieve-hook/ sieve-ipc/ sieve-policy/ sieve-rules/ sieve-updater/`

任一项 fail → **不要继续**，先排查（看 PROGRESS.md / lessons.md）。

---

## 2. 旧 schema 向后兼容验证（不能破老用户）

ADR-026 引入 `[[upstream]]` 数组的同时保证旧 sieve.toml 仍可用。验证：

- [ ] 用旧 schema 启动 daemon：

  ```bash
  cat > /tmp/sieve-legacy.toml <<'EOF'
  upstream_url = "https://api.anthropic.com"
  port = 11453
  bind_addr = "127.0.0.1"
  EOF

  SIEVE_LOG=info cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml
  ```

- [ ] 启动日志含 `sieve daemon listener bound listen=127.0.0.1:11453 upstream_host=api.anthropic.com provider_id=anthropic protocol=Anthropic`（自动映射成单元素 vec）
- [ ] 不再有 `WARN ... legacy upstream_url/port are set`（因为只有旧字段，没有 `[[upstream]]` 冲突）
- [ ] Ctrl+C 优雅退出

---

## 3. Multi-listener（ADR-026 核心）

### 3.1 启动 + bind 全成功

- [ ] 写 multi-listener config：

  ```bash
  mkdir -p ~/.sieve
  cat > ~/.sieve/sieve.toml <<'EOF'
  bind_addr = "127.0.0.1"
  tls_verify_upstream = true

  [[upstream]]
  port = 11453
  url = "https://api.anthropic.com"
  provider_id = "anthropic"
  protocol = "anthropic"

  [[upstream]]
  port = 11454
  url = "https://api.deepseek.com/anthropic"
  provider_id = "deepseek"
  protocol = "anthropic"

  [[upstream]]
  port = 11455
  url = "https://api.openai.com"
  provider_id = "openai"
  protocol = "openai"
  EOF

  SIEVE_LOG=info cargo run -p sieve-cli -- start --config ~/.sieve/sieve.toml
  ```

- [ ] 启动日志含 3 条 `sieve daemon listener bound`，分别 port=11453 / 11454 / 11455
- [ ] `lsof -i -P | grep -E "sieve.*LISTEN"` 显示 3 个 `*.11453` / `*.11454` / `*.11455` 都在 LISTEN
- [ ] daemon 没崩，长期 running

### 3.2 端口冲突 fail-fast

- [ ] 写一份故意冲突的 config：

  ```bash
  cat > /tmp/sieve-dup.toml <<'EOF'
  bind_addr = "127.0.0.1"

  [[upstream]]
  port = 11453
  url = "https://api.anthropic.com"
  provider_id = "anthropic"

  [[upstream]]
  port = 11453
  url = "https://api.deepseek.com/anthropic"
  provider_id = "deepseek"
  EOF

  cargo run -p sieve-cli -- start --config /tmp/sieve-dup.toml
  ```

- [ ] daemon **拒绝启动**，stderr 含 `FATAL: duplicate listener port 11453`，exit code 非零
- [ ] PRD §9 #2 一致性：bind_addr 改成 `0.0.0.0` 同样 fail-fast

### 3.3 实际打两个上游（需要真 API key）

把 §3.1 的 daemon 跑起来，开两个 shell：

- [ ] **Shell A**（Anthropic 官方）：

  ```bash
  ANTHROPIC_BASE_URL=http://127.0.0.1:11453 \
  ANTHROPIC_AUTH_TOKEN=<your-anthropic-key> \
  claude --bare -p "ping"
  ```

  返回 Claude 正常响应（通过 sieve 透传上游）；daemon 日志看到 11453 listener 接到连接。

- [ ] **Shell B**（DeepSeek，验证 path prefix bug 修复 TODO-1）：

  ```bash
  ANTHROPIC_BASE_URL=http://127.0.0.1:11454 \
  ANTHROPIC_AUTH_TOKEN=<your-deepseek-key> \
  claude --bare -p "ping"
  ```

  daemon 实际转发到 `https://api.deepseek.com/anthropic/v1/messages`（不是 `/v1/messages` 404）。可以用 `tcpdump -i lo0 -A port 443 | grep messages` 抓包确认上游 path 含 `/anthropic`。

---

## 4. 协议错位 fail-closed（ADR-026 §决策 4）

daemon 跑着 §3.1 的 multi-listener config（11453=anthropic，11455=openai）。

- [ ] **Anthropic listener 收 OpenAI path → 400**：

  ```bash
  curl -i -X POST http://127.0.0.1:11453/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{}'
  ```

  - HTTP 状态：`400 Bad Request`
  - body JSON 含 `"type":"sieve_blocked"` + `"reason":"listener_protocol_mismatch"`
  - daemon 日志含 `WARN ... ADR-026 protocol mismatch: anthropic listener received openai`

- [ ] **OpenAI listener 收 Anthropic path → 400**：

  ```bash
  curl -i -X POST http://127.0.0.1:11455/v1/messages \
    -H "Content-Type: application/json" \
    -d '{}'
  ```

  同上，但 `listener_protocol: "openai"` + `request_path: "/v1/messages"`。

- [ ] **健康检查 / 透传 path 不强制**（不在白名单）：

  ```bash
  curl -i http://127.0.0.1:11453/some-other-path
  ```

  daemon 透传给上游（502 / 404 等取决于上游）；不会被 protocol mismatch 拒。

- [ ] **X-Sieve-Provider header 不能 override listener 协议**：

  ```bash
  curl -i -X POST http://127.0.0.1:11453/v1/chat/completions \
    -H "X-Sieve-Provider: openai" \
    -d '{}'
  ```

  仍然 `400 listener_protocol_mismatch`（fail-closed 一致性，header routing 不能绕过 listener.protocol）。

---

## 5. doctor multi-listener 体检（ADR-026 Stage F）

daemon 跑着 §3.1 的 multi-listener config。

- [ ] 跑 doctor：

  ```bash
  cargo run -p sieve-cli -- doctor --agent claude
  ```

- [ ] 输出包含原 5 项 + 新增「ADR-026 multi-listener 全部端口可达（3 个 listener）」一项
- [ ] 全部 ✅
- [ ] **失败场景**：手动 kill 一个 listener 进程是不可能的（multi-listener 是 daemon 内部 spawn）。验证失败行为：把 sieve.toml 改成包含一个**未启动** daemon 的 port（比如临时关 daemon 后再跑 doctor），应看到「失败的 listener: port 11454 (deepseek)」类错误描述

---

## 6. sieve audit CLI（ADR-028 TODO-5）

让 daemon 跑一段时间产生一些 audit events（比如跑 §3.3 的实际 LLM 流量）。

### 6.1 tail

- [ ] **默认显示最近 20 条**：

  ```bash
  cargo run -p sieve-cli -- audit tail
  ```

- [ ] **jsonl 格式接 jq**：

  ```bash
  cargo run -p sieve-cli -- audit tail --format jsonl --limit 5 \
    | jq '{id, rule_id, severity, provider_id}'
  ```

  每行一个 JSON object，字段对齐 v3 schema。

- [ ] **--follow 流式跟踪**：

  ```bash
  cargo run -p sieve-cli -- audit tail -f --format jsonl
  ```

  另一 shell 触发新事件（比如打 LLM 流量 + 打错位请求），watch shell 应实时看到新行；Ctrl+C 优雅退出。

### 6.2 query

- [ ] **--since 时间过滤**：

  ```bash
  cargo run -p sieve-cli -- audit query --since 1h --format jsonl
  cargo run -p sieve-cli -- audit query --since 30m
  cargo run -p sieve-cli -- audit query --since 7d
  ```

- [ ] **--severity 过滤**：

  ```bash
  cargo run -p sieve-cli -- audit query --severity critical --format jsonl
  ```

- [ ] **--provider-id 过滤（v3 schema 新列）**：

  ```bash
  cargo run -p sieve-cli -- audit query --provider-id anthropic --format jsonl
  cargo run -p sieve-cli -- audit query --provider-id deepseek --format jsonl
  cargo run -p sieve-cli -- audit query --provider-id _system --format jsonl
  ```

  各应返回对应 listener / 系统级事件。

- [ ] **组合过滤**：

  ```bash
  cargo run -p sieve-cli -- audit query --since 1h --severity critical --provider-id anthropic
  ```

### 6.3 show

- [ ] **看单条详情**（取 tail 输出里某条 id）：

  ```bash
  cargo run -p sieve-cli -- audit show 42
  ```

- [ ] 输出含完整 raw_json 字段（如有）

---

## 7. 审计 provider_id 数据验证（ADR-026 Stage E）

daemon 跑过实际流量后：

- [ ] **SQLite 直接查 provider_id 分布**：

  ```bash
  sqlite3 ~/.sieve/audit.db \
    "SELECT provider_id, COUNT(*) FROM audit_events GROUP BY provider_id ORDER BY COUNT(*) DESC;"
  ```

  应见到分组：
  - `anthropic` / `deepseek` / `openai` 等（来自 listener.provider_id）
  - `_system`（daemon 系统级事件，control plane / oversize / UserRulesReloaded）
  - `unknown`（v2 老记录 migration 默认值，如有）

- [ ] **schema 版本**：

  ```bash
  sqlite3 ~/.sieve/audit.db "PRAGMA user_version;"
  ```

  返回 `3`（ADR-026 Stage E 升级）。

- [ ] **provider_id 列存在 + NOT NULL**：

  ```bash
  sqlite3 ~/.sieve/audit.db ".schema audit_events" | grep provider_id
  ```

  应看到 `provider_id TEXT NOT NULL DEFAULT 'unknown'`。

---

## 8. v2 → v3 schema migration（向后兼容老 audit.db）

如果你之前有 v2 schema 的 `~/.sieve/audit.db`（例如 dogfood 之前），验证 migration：

- [ ] **备份现有 audit.db**：

  ```bash
  cp ~/.sieve/audit.db /tmp/audit-pre-v3.db.bak
  ```

- [ ] **启动 daemon**：第一次启动会自动跑 v2→v3 migration（ALTER TABLE ADD COLUMN）

- [ ] **验证 user_version 升到 3 + 老数据保留 + provider_id 默认 'unknown'**：

  ```bash
  sqlite3 ~/.sieve/audit.db "PRAGMA user_version;"
  # → 3

  sqlite3 ~/.sieve/audit.db "SELECT COUNT(*) FROM audit_events;"
  # 应跟 backup 的 row count 一致（数据没丢）

  sqlite3 ~/.sieve/audit.db "SELECT provider_id, COUNT(*) FROM audit_events WHERE id <= (SELECT MAX(id)/2 FROM audit_events) GROUP BY provider_id;"
  # 老一半行的 provider_id 应是 'unknown'
  ```

- [ ] **migration 触发器仍 active**：

  ```bash
  sqlite3 ~/.sieve/audit.db "UPDATE audit_events SET rule_id='hacked' WHERE id=1;"
  # → 应报 "audit_events is append-only" error
  ```

---

## 9. sieve decisions headless CLI（ADR-028 TODO-4）

让 daemon 跑着，开两个 shell。

### 9.1 watch（订阅 pending events）

- [ ] **Shell A（订阅）**：

  ```bash
  cargo run -p sieve-cli -- decisions watch --format jsonl
  ```

  当前空（没有 pending decision）。

- [ ] **Shell B（触发一个 pending decision）**：可以通过打实际 LLM 流量 + 触发某条 GUI Ask 类规则。或者临时把某条规则的 disposition 改成 `gui_popup` 来触发。

  Shell A 应实时收到 jsonl event（每行含 request_id / rule_id / severity 等字段）。

### 9.2 show

- [ ] 取 watch 输出里的 request_id：

  ```bash
  cargo run -p sieve-cli -- decisions show <request-id>
  ```

  返回该 pending decision 的完整上下文（detection / origin / caller）。

### 9.3 resolve

- [ ] **批准**：

  ```bash
  cargo run -p sieve-cli -- decisions resolve <request-id> --approve --reason "合法 tool_use"
  ```

  daemon 应放行原 SSE 流；audit 应有对应 DecisionMade 事件（decision="allow"）。

- [ ] **拒绝**：

  ```bash
  cargo run -p sieve-cli -- decisions resolve <request-id> --block --reason "钓鱼地址替换"
  ```

  daemon 应注入 sieve_blocked event 截流。

- [ ] **warn 放行**：

  ```bash
  cargo run -p sieve-cli -- decisions resolve <request-id> --warn
  ```

### 9.4 --no-client-policy（GUI 不在线时的兜底）

如果 GUI 不在线（没有 client 连 IPC），daemon 应按 `--no-client-policy` flag 行为：

- [ ] **auto-block（默认）**：

  ```bash
  # 重启 daemon，确保 GUI 不在线
  pkill -f "sieve.*start" 2>/dev/null; sleep 1
  cargo run -p sieve-cli -- start --config ~/.sieve/sieve.toml --no-client-policy auto-block &

  # 触发一条非 Critical decision（应被自动 deny）
  ```

  audit 应有 DecisionMade 事件 decision="deny" + by_user=false。

- [ ] **auto-warn**：用 `--no-client-policy auto-warn` 重启，触发同样事件 → 自动 allow（标 warn）。

- [ ] **hold-and-fail-closed**：v1.x 行为，等超时后按 `default_on_timeout` 处置。

---

## 10. forwarder path prefix 修复（TODO-1）

DeepSeek Anthropic 兼容入口 `https://api.deepseek.com/anthropic` 是验证 TODO-1 的最直接 case。已经在 §3.3 Shell B 验证过。补充：

- [ ] **抓包确认 path 前缀正确转发**（如 §3.3 提到的 tcpdump）
- [ ] **多层前缀验证**（如果有这种中转站）：

  ```toml
  [[upstream]]
  port = 11456
  url = "https://relay.example.com/api/v2"
  ```

  请求 `/v1/messages` 应转发到 `https://relay.example.com/api/v2/v1/messages`。

---

## 11. SPEC-005 协议中性化（ADR-028 TODO-3a）

- [ ] **wire 字段值未变**（向后兼容硬要求）：

  ```bash
  grep -E '"gui_popup"|"hook_terminal"' ~/.sieve/audit.db.queries 2>/dev/null
  ```

  daemon 仍用旧 disposition 枚举值（`gui_popup` 等），GUI 客户端兼容。

- [ ] **SPEC-005 文档已中性化**：

  ```bash
  grep -c "GUI 端\|client 端" docs/specs/SPEC-005-ipc-protocol.md
  ```

  「client 端」应远多于「GUI 端」。

---

## 12. sieve-ipc 模块化（ADR-028 TODO-3b）

仅 crate 内部重组，对外 API 100% 兼容。验证：

- [ ] **目录结构**：

  ```bash
  ls crates/sieve-ipc/src/protocol/  # envelope / decision / handshake / rules / audit / health / notify + README.md
  ls crates/sieve-ipc/src/server/    # mod.rs + socket_server.rs
  ls crates/sieve-ipc/src/client/    # mod.rs + connection.rs
  ```

- [ ] **`protocol/` 零 IO 依赖硬约束**：

  ```bash
  grep -rE "use tokio|use hyper|use fd_lock" crates/sieve-ipc/src/protocol/
  ```

  应当无任何匹配。

- [ ] **lib.rs 向后兼容别名**：

  ```bash
  grep -E "pub use client as socket_client|pub use server::socket_server" crates/sieve-ipc/src/lib.rs
  ```

  两条都应存在（旧 import 路径保持可用）。

---

## 13. 跨仓 follow-up（GUI 仓 sieve-gui-macos）

**不在本仓范围**，但你 dogfood 时会接触：

- [ ] GUI 仓 SPEC-002 / Swift 代码暂未读 `health.listeners` 数组（仍用 `listen` 单字段，向后兼容期内 daemon 仍发该字段）
- [ ] 启动 sieve-gui-macos：菜单栏出图标，连 `~/.sieve/ipc.sock`，能收到 daemon 推送的 sieve.hello / heartbeat
- [ ] HIPS 弹窗：能收到 decision request，能正常 approve / block / warn
- [ ] **GUI 仓需要一个独立 issue / PR** 把 health.listeners 数组接入显示——这是 follow-up，不阻塞 daemon ship

---

## 14. sieve-updater 客户端独立闭环（SPEC-006）

> 关联：[SPEC-006](../specs/SPEC-006-update-and-telemetry.md)
> **服务端尚未实施**。本节用本地 mock HTTP 服务器验证客户端能完整跑通 manifest → 下载 → 校验 → 原子落盘的闭环。

### 14.1 install-id 模块（首启幂等 + 删后重生）

- [ ] **首启生成 install-id**：

  ```bash
  rm -f ~/Library/Caches/sieve/install-id   # 清空旧的（如有）
  SIEVE_NO_UPDATE=1 cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml &
  DAEMON_PID=$!
  sleep 2
  kill $DAEMON_PID
  ```

  注：这里用 `SIEVE_NO_UPDATE=1` 启动只为触发 install-id 模块初始化（实际 install-id 加载发生在 updater task spawn 前;若发现 SIEVE_NO_UPDATE 路径下 install-id 不生成,见下方备选）。

- [ ] 备选：去掉 `SIEVE_NO_UPDATE`,让 updater task 真跑（会试连 updates.sieveai.dev,失败 log error 不影响 daemon）：

  ```bash
  cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml &
  DAEMON_PID=$!
  sleep 5     # 给 updater task 初始化时间
  kill $DAEMON_PID
  ```

- [ ] 检查文件存在 + 权限 0600：

  ```bash
  ls -la ~/Library/Caches/sieve/install-id
  # 期望：-rw-------  1 youruser  staff  36 May  5 12:34
  cat ~/Library/Caches/sieve/install-id
  # 期望：UUIDv4 字符串,例如 a1b2c3d4-e5f6-7890-1234-567890abcdef
  ```

- [ ] **首启幂等**（第二次启动复用同一 UUID）：

  ```bash
  UID_BEFORE=$(cat ~/Library/Caches/sieve/install-id)
  cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml &
  sleep 3
  kill %1
  UID_AFTER=$(cat ~/Library/Caches/sieve/install-id)
  [ "$UID_BEFORE" = "$UID_AFTER" ] && echo "OK: idempotent" || echo "FAIL"
  ```

- [ ] **删后重生**（用户主动删除 → 视为新装机,接受统计噪声）：

  ```bash
  rm ~/Library/Caches/sieve/install-id
  cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml &
  sleep 3
  kill %1
  ls ~/Library/Caches/sieve/install-id   # 应重新生成
  cat ~/Library/Caches/sieve/install-id  # 新 UUID,与 UID_BEFORE 不同
  ```

### 14.2 三个环境变量（SIEVE_NO_UPDATE / SIEVE_NO_TELEMETRY / SIEVE_UPDATE_URL）

- [ ] **SIEVE_NO_UPDATE banner 强制可见**（SPEC-006 §5）：

  ```bash
  SIEVE_NO_UPDATE=1 cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml 2>&1 | head -30 | grep -i "update check disabled"
  ```

  期望日志含一行：`update check disabled by SIEVE_NO_UPDATE`。**找不到这行视为 P0 bug**——用户忘了设过此变量却奇怪规则不更新的最大防护。

- [ ] **SIEVE_NO_UPDATE 不发任何更新请求**（不应连 updates.sieveai.dev）：

  ```bash
  # 用 sudo 不便时,只看日志中是否有 "starting updater task" 出现
  SIEVE_NO_UPDATE=1 cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml 2>&1 | grep "starting updater task"
  # 期望：无输出（updater task 没 spawn）
  ```

- [ ] **SIEVE_NO_TELEMETRY 仍发请求但无 uid**：见 §14.4 mock server 验证

- [ ] **SIEVE_UPDATE_URL 切到本地 mock**：见 §14.3

### 14.3 本地 mock 服务器（验证客户端独立闭环）

启个最简 mock manifest + CDN 服务器（同一 Python http.server 即可）。

- [ ] **准备 mock 工作目录**：

  ```bash
  MOCK_DIR=$(mktemp -d)
  cd "$MOCK_DIR"

  # 1. 生成假规则文件（任意内容均可,真实规则 schema 见 sieve-rules）
  echo '{"rules":[],"version":"2026.05.05.1","note":"test fixture"}' > rules-payload.json

  # 2. zstd 压缩
  zstd -19 -o rules.json.zst rules-payload.json

  # 3. 算 sha256（manifest 字段需要）
  RULES_SHA=$(shasum -a 256 rules.json.zst | awk '{print $1}')
  echo "RULES_SHA=$RULES_SHA"

  # 4. 写 manifest 响应（schema 见 SPEC-006 §3）
  mkdir -p v1
  cat > v1/manifest <<EOF
  {
    "schema": 1,
    "rules": {
      "version": "2026.05.05.1",
      "url": "http://127.0.0.1:8080/rules.json.zst",
      "sha256": "$RULES_SHA",
      "size": $(stat -f%z rules.json.zst),
      "signature": "ed25519:placeholder-not-verified-when-pubkey-is-None"
    },
    "client": {
      "latest": "0.1.0-alpha",
      "min_supported": "0.1.0-alpha",
      "deprecation_notice": null
    },
    "next_check_after_seconds": 21600
  }
  EOF

  # 5. 起 HTTP server
  python3 -m http.server 8080 &
  MOCK_PID=$!
  echo "MOCK_PID=$MOCK_PID  MOCK_DIR=$MOCK_DIR"
  ```

- [ ] **测试 mock 接口活着**：

  ```bash
  curl -s http://127.0.0.1:8080/v1/manifest | head -3
  curl -sI http://127.0.0.1:8080/rules.json.zst | head -3
  ```

> **注意**：客户端 `download.rs` 用 hyper-rustls **https_only**,默认拒绝纯 HTTP。本地 mock 验证需要要么 (a) 用 `mkcert` + nginx 反代 https 包一层,要么 (b) 临时跑 release 编译时移除 https_only 跑联调（不要 commit 这个改动）,要么 (c) 在 sieve-updater 加一个 dev-only feature flag `allow-http-mock`（默认 off）。**推荐 (a)**——一行 `mkcert` + caddy 反代 4 行配置就能起 https://localhost:8443。

参考 caddy 反代（5 秒起）：

```bash
brew install caddy mkcert
mkcert -install
cd "$MOCK_DIR"
mkcert localhost 127.0.0.1
cat > Caddyfile <<EOF
localhost:8443 {
  tls localhost+1.pem localhost+1-key.pem
  reverse_proxy http://127.0.0.1:8080
}
EOF
caddy run --config Caddyfile &
CADDY_PID=$!
```

之后客户端用 `SIEVE_UPDATE_URL=https://localhost:8443/v1/manifest`。

### 14.4 完整闭环（fetch → download → 校验 → 原子落盘）

- [ ] **清空 staging area**（保证测试干净）：

  ```bash
  rm -rf ~/Library/Caches/sieve/rules
  ls ~/Library/Caches/sieve/    # 应只有 install-id 和可能的 rules（已删）
  ```

- [ ] **跑客户端,指向 mock**：

  ```bash
  SIEVE_UPDATE_URL=https://localhost:8443/v1/manifest \
  RUST_LOG=sieve_updater=debug,info \
  cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml 2>&1 | tee /tmp/sieve-updater.log &
  sleep 8     # 启动立即查一次,5-8 秒内应完成 download+install
  kill %1
  ```

- [ ] **检查日志**：

  ```bash
  grep -E "starting updater task|fetch_manifest|download_rules|install_rules|rules installed" /tmp/sieve-updater.log
  ```

  期望按顺序出现：
  - `starting updater task target=https://localhost:8443/v1/manifest telemetry=true`
  - `fetch_manifest ok version=2026.05.05.1`
  - `download_rules ok size=<bytes>`
  - `install_rules ok version=2026.05.05.1`（含 sha256 校验通过 + ed25519 跳过 WARN + zstd 解压成功）
  - `rules installed version=2026.05.05.1 path=...`

- [ ] **检查 staging 落盘**：

  ```bash
  ls -la ~/Library/Caches/sieve/rules/
  # 期望：
  #   2026.05.05.1.json     ← 解压后内容
  #   current.json          ← symlink 指向上面
  #   latest_version.json   ← 元信息

  cat ~/Library/Caches/sieve/rules/latest_version.json
  # 期望：{"version":"2026.05.05.1","installed_at":<ts>,"sha256":"<hex>"}

  readlink ~/Library/Caches/sieve/rules/current.json
  # 期望：2026.05.05.1.json

  cat ~/Library/Caches/sieve/rules/current.json
  # 期望与 mock 的 rules-payload.json 一致
  ```

- [ ] **二次启动同版本应跳过下载**（version 比对生效）：

  ```bash
  cargo run -p sieve-cli -- start --config /tmp/sieve-legacy.toml 2>&1 | tee /tmp/sieve-updater-2.log &
  sleep 5
  kill %1
  grep -E "fetch_manifest|download_rules|install_rules" /tmp/sieve-updater-2.log
  # 期望：fetch_manifest 出现,download_rules / install_rules 不应出现
  ```

### 14.5 失败模式（client 应优雅降级,不影响 daemon）

- [ ] **sha256 不匹配 → 安装失败但 daemon 继续运行**：

  改 `MOCK_DIR/v1/manifest` 把 `sha256` 字段改成假值,重启客户端,日志应有 `Sha256Mismatch`,临时文件应被清理（`ls ~/Library/Caches/sieve/rules/.tmp-*` 应无文件）,daemon 主进程不退出。

- [ ] **manifest 接口 500 → 重试 3 次后跳过**：

  停掉 caddy（`kill $CADDY_PID`）模拟服务端 down。客户端日志应有指数退避 `1s/4s/16s` 重试 3 次后 `RetryExhausted`,daemon 继续跑（后续每 6h 再尝试）。

- [ ] **解压失败 → DecompressFailed**：

  把 `rules.json.zst` 替换为非 zst 字节（`echo "not zstd" > rules.json.zst`）+ 改 manifest 的 sha256。日志应有 `DecompressFailed`。

### 14.6 公钥 None 占位的 WARN 强制可见（SPEC-006 §安全）

- [ ] 在 §14.4 完整闭环日志里查找：

  ```bash
  grep -i "trusted pubkey not configured" /tmp/sieve-updater.log
  ```

  期望含一行：`ed25519 trusted pubkey not configured, skipping signature verification`。**找不到这行视为 P0 bug**——若公钥占位被人误改成跳过 WARN,会导致供应链攻击防线失效（TODO-14 GCP KMS 落地后填真值,届时这行 WARN 消失）。

### 14.7 清理

```bash
kill $MOCK_PID $CADDY_PID 2>/dev/null
rm -rf "$MOCK_DIR"
```

---

## 15. 完成定义（DoD）

- [ ] §1 基线全过（含 sieve-updater 35 测试,workspace 760 passed）
- [ ] §2 旧 schema 兼容
- [ ] §3 multi-listener bind + 实际两个上游各能跑
- [ ] §4 协议错位 fail-closed（4 个子 case 全过）
- [ ] §5 doctor 多 listener 体检
- [ ] §6 sieve audit tail / query / show（jsonl 接 jq）
- [ ] §7 audit provider_id 分布合理
- [ ] §8 v2→v3 migration（如有 v2 老 db）
- [ ] §9 sieve decisions watch / show / resolve + --no-client-policy 三种策略
- [ ] §10 forwarder path prefix（DeepSeek 等）
- [ ] §11 SPEC-005 中性化（文档级别）
- [ ] §12 sieve-ipc 模块化（结构级别）
- [ ] §13 GUI 仓接入正常（向后兼容期内）
- [ ] §14.1 install-id 首启 + 幂等 + 删后重生
- [ ] §14.2 三个 env var（SIEVE_NO_UPDATE banner 必可见 / SIEVE_NO_TELEMETRY / SIEVE_UPDATE_URL）
- [ ] §14.3 本地 mock + caddy https 反代起得来
- [ ] §14.4 完整闭环（fetch→download→sha256→ed25519 skip WARN→zstd→tmp+rename+symlink+latest_version.json）
- [ ] §14.5 三种失败模式不击穿 daemon（sha256 mismatch / 服务端 down 重试耗尽 / 解压失败）
- [ ] §14.6 公钥 None 占位 WARN 强制可见

**任一项 fail**：在 tasks/lessons.md 记一条 lesson，回报到主上下文准备修复。

**全过**：unix-style 改造 v2.x + sieve-updater 客户端闭环 联调通过 → 进 dogfood 阶段（服务端实现与 dogfood 并行启动）。

---

## 附录 A：常用排错命令

```bash
# daemon 进程 + 端口
ps aux | grep "sieve.*start" | grep -v grep
lsof -i -P | grep -E "sieve|11453|11454|11455"

# IPC socket 状态
ls -la ~/.sieve/ipc.sock
file ~/.sieve/ipc.sock      # 应为 socket
nc -U ~/.sieve/ipc.sock     # 手动连 IPC（按 Ctrl+C 退出）

# audit.db 大小 + schema 版本
ls -la ~/.sieve/audit.db
sqlite3 ~/.sieve/audit.db "PRAGMA user_version;"

# 强制清空 dogfood 数据（重新开始联调）
rm -f ~/.sieve/audit.db ~/.sieve/ipc.sock
rm -rf ~/.sieve/pending ~/.sieve/decisions

# 强制清空 sieve-updater 状态（§14 重跑）
rm -rf ~/Library/Caches/sieve/    # install-id + rules/ staging 一起清

# 看 updater 当前 staging 状态
ls -la ~/Library/Caches/sieve/rules/ 2>/dev/null
cat ~/Library/Caches/sieve/rules/latest_version.json 2>/dev/null

# 三个 env var 一次性 export（联调 §14 用）
export SIEVE_UPDATE_URL=https://localhost:8443/v1/manifest
unset SIEVE_NO_UPDATE SIEVE_NO_TELEMETRY     # 默认放开
```

## 附录 B：关联文档

- [ADR-026 Port-based listener routing](../design/ADR-026-port-based-listener-routing.md)
- [ADR-028 IPC 协议中性化](../design/ADR-028-ipc-protocol-neutralization.md)
- [SPEC-006 manifest 协议更新通道与 install 统计设计](../specs/SPEC-006-update-and-telemetry.md)
- [SPEC-003 sieve setup tool](../specs/SPEC-003-sieve-setup-tool.md) §4.2b doctor multi-listener
- [SPEC-004 multi-agent setup](../specs/SPEC-004-multi-agent-setup.md) §4.2.6 header vs port routing
- [SPEC-006 manifest 协议 + sieve-updater 客户端](../specs/SPEC-006-update-and-telemetry.md)
- [deployment.md §6a Multi-listener 部署](deployment.md#6a-multi-listener-部署) / §13 企业自托管镜像
- [development.md §3.4a Multi-listener 配置](development.md#34a-multi-listener-配置) / §13 三个 env var
- [api-reference.md §6.4a sieve decisions / §6.4b sieve audit](../api/api-reference.md#64a-sieve-decisions-cli) / §8 manifest 接口
- tasks/PROGRESS.md 用户验证清单段
