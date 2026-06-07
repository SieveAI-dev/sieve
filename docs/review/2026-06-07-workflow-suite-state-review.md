# Sieve Suite 工程状态审查 — 2026-06-07

> 来源：11-agent workflow 全面审查（构建/测试地面真相 + 文档漂移 + 跨仓契约 + 安全态势 + 技术债 + 发布就绪）+ 主控全量测试实跑验证
> 范围：`sieve`（Rust daemon）/ `sieve-gui-macos`（Swift GUI）/ `dashboard`（mdBook）
> 审查时 git：sieve `016ba2e` · gui `ae0288f`（两仓工作区均干净，均 main 分支）
> 工具链：cargo 1.88(pinned) · swift 6.3.2 · xcodebuild · mdbook 0.5.3 · cargo-deny 0.19.4

---

## 总评

整体 **needs-attention → 经本次 P0 修复后 mostly-healthy**。构建 / clippy / fmt / deny 三仓全绿；GUI（134 测试）与 dashboard 健康。审查时 daemon 有 **13 个长期红测试**被文档谎报为「0 failed」，已于本次（2026-06-07）全部修复归零。最大剩余阻塞均为**非代码项**：125 项联调 checklist 0 勾选（dogfood 从未真跑）+ ADR-005 海外主体未落地级联卡死遥测/更新链（ADR-029 定的唯一 GA 指标）。

---

## 一、三仓构建/测试地面真相

| 子工程 | 构建 | 测试 | 判定 |
|---|---|---|---|
| **sieve** (Rust daemon, 7 crates, ~50.5k 行, 0.1.0-alpha) | build / clippy(0 warn) / fmt / deny 全绿（deny 有 10 条非阻塞 duplicate/license warning） | 审查时 **747 passed / 13 failed / 7 ignored** → 修复后 **760 / 0 / 7** | 审查时 broken → **已恢复 healthy** |
| **sieve-gui-macos** (Swift/SwiftUI, 56 文件) | swift build + xcodebuild BUILD SUCCEEDED，0 warning；SPM pin SQLite.swift@0.16.0 + Sparkle@2.9.1 | swift test **134 passed** | healthy（发布签名链未配） |
| **dashboard** (mdBook 聚合) | mdbook build exit 0，143 HTML 页，SUMMARY 140 链接 + 14 symlink 全解析、0 dangling | — | healthy（最干净） |

> 关键教训：构建 agent 因 `cargo test` 默认 fail-fast 在第 3 个 binary 后中止，只看到 2 个失败；主控用 `--no-fail-fast` 跑出真实 13 个。`| tail` 会用 tail 的退出码掩盖 cargo 失败码——验证测试基线必须 `--no-fail-fast` 且看汇总。

---

## 二、🔴 P0：daemon 13 个红测试（✅ 2026-06-07 已修复）

真相：PROGRESS / CLAUDE.md 长期写「760 passed / 0 failed」——760 实为测试**总数**被误作通过数；CHANGELOG L36 当时如实记了「747 passed / 13 failed」但无人跟进，随后进入 dogfood 冻结期。两簇根因性质不同：

### 簇 A — 产品 bug（9 个测试）：legacy/单-upstream OpenAI 路径被协议错位 400
- **根因**：`config.rs::resolved_upstreams()` 把 legacy `upstream_url` 硬编码映射成 `protocol: Protocol::Anthropic`，叠加 ADR-026 §决策4 协议错位检查（`daemon.rs:1357`）→ 任何 legacy/未声明协议配置发 OpenAI `/v1/chat/completions` 都被 fail-closed 400，违反 ADR-026 §决策1 向后兼容 + PRD §9 #16/#9 双协议硬约束。
- **修复方向**（产品负责人拍板）：`config::Protocol` 加 `#[default] Auto` 第三态。legacy 与省略 `protocol` 的 `[[upstream]]` → `Auto`，按请求 path 自适应、不做错位拒绝；**仅显式声明 anthropic/openai 才强制 fail-closed 错位**（安全契约对显式声明者完全保留）。`proxy_inner` 的 path 分发（`is_chat_completions_post → proxy_openai`）本就独立于 listener_protocol，无需改。
- **改动**：`config.rs`（enum + resolved_upstreams + 2 单元测试预期）、`daemon.rs`（2 处 match 补 Auto 分支 + 注释）、`sieve-ipc/protocol/health.rs`（ListenerSnapshot.protocol 文档）。**0 改业务/集成测试**。

### 簇 B — 测试 bug（4 个测试）：GUI popup Allow/RedactAndAllow 期望 200 实得 426
- **根因**：测试 mock `outbound_block.rs::mock_gui_respond_with_ready` 未跳过 SPEC-005 §3 引入的 `sieve.hello` 握手帧（有 params 无 request_id）→ 解析 request_id 崩溃 → mock 断连 → daemon `try_send` Closed → fallback Block → Deny → 假性 426。**产品代码正确**（真实 GUI 连接实测 200）。
- **修复**：mock 读取循环加 method 正过滤，只处理 `sieve.request_decision`，跳过 hello/heartbeat。连带修正 `outbound_gui_popup_deny_returns_426` 的假阳性（崩溃 fallback 恰好 = 426）。

### 验证
`cargo test --workspace --no-fail-fast` → **760 passed / 0 failed / 7 ignored**；`cargo fmt --all --check` 干净；`cargo clippy --workspace --all-targets -- -D warnings` 0 warning。下游文档（CHANGELOG/ADR-026/api-reference/data-model）同步更新。

---

## 三、文档漂移

- **PROGRESS 约 1 个月未更新**（5-12，与 git HEAD 同日）：是「冻结待 dogfood」非「代码动了文档没跟」，里程碑与 git log 逐条对得上。已于本次更新。
- **测试数三处打架**：PROGRESS「760 passed」/ CLAUDE.md「725/0」/ CHANGELOG「747/13」——已统一为真实的 760/0/7（本次）。
- **跨仓 SPEC-005 pin 失真**：GUI `upstream-references.md` pin 钉在 `2e38e44`（该 commit 不碰 SPEC-005），真正引入 listeners[] 的是 `7108a45`。运行期契约两侧一致（GUI 解码超前实现），仅元数据失真。→ **本次修复中**（回填 `7108a45`）。
- **SPEC §14 fixture 防漂移机制名存实亡**：daemon 权威 fixture `sieve.health/response.full.json` 不含 listeners[]（但 `health.rs:104` 已序列化）；SPEC §14.2 强制的 GUI 端 `Fixtures/v2/` + `IPCSchemaV2FixtureTests.swift` 不存在。→ **待办**。
- **~39 处文档悬空引用**：glossary.md/DOCS-STANDARD.md 的 `../prd/`/`../design/` 前缀错；PRD/ADR-022/CHANGELOG 指向已归档的 hips-readiness；ADR-026/027/028 互引旧文件名；ADR-017 无说明跳号；CLAUDE.md「21 个 ADR」过期。→ **本次修复中**。

---

## 四、跨仓 SPEC-005 契约

核心 wire 契约两侧**完全一致**（protocol_version=v2、错误码段位 -32100~、方法名、decision_response required 字段、listeners[] 解码三档兼容），运行期无错位风险。漂移集中在治理层：pin 失真（见上）+ fixture 机制未落地（见上）。

---

## 五、安全态势

- ✅ **隐私承诺与代码完全一致**：遥测只发 5 字段（v/os/arch/uid/ch），不含 prompt/key/使用记录；`SIEVE_NO_TELEMETRY`/`SIEVE_NO_UPDATE` 真实生效；日志/audit.db 不落原始 body（raw_json 生产代码零写入）；fail-closed Critical 在暂停/失联/无 client 下确实强制拒绝（ADR-007）。
- 🟠 **规则包 Ed25519 验签当前 fail-OPEN**：`signature.rs:18 TRUSTED_PUBKEY=None` → skip+warn，仅靠同源 manifest 的 SHA-256。CDN/TLS 沦陷可绕过。与 `SECURITY.md:76` 承诺漂移。**alpha 可接受，GA 硬阻塞**。同类：`origin_header.rs` 公钥全零占位。建议加 release-channel 编译期断言（占位密钥则编译失败）。
- 🟢 次要：IN-CR-03 未列入 FAIL_CLOSED_RULES（运行时由 hook 路径独立兜底，无实际 fail-open），属注释与实现一致性瑕疵。

---

## 六、Rust 技术债

- ✅ panic 纪律极好：50.5k 行仅 14 处生产 unwrap/expect，sieve-core `#![forbid(unsafe_code)]`，零 FIXME/HACK。
- 🟡 请求热路径 4 处 `.expect`（`daemon.rs:1426/1566/2043/2075`）+ IPC 路径 3 处 `lock().expect("poisoned")`（`socket_server.rs:478/587/666`）——当前由守卫保证不可达，但违反 CLAUDE.md「请求路径禁 expect」，重构易触发 panic=DoS。建议前者改 enum 让非法状态不可表达，后者改 `into_inner()` 毒化恢复。
- 🟡 `daemon.rs` 5069 行 / `proxy_inner` ~794 行，Anthropic/OpenAI 双路径镜像重复（`forward_with_*` / `handle_*_json_inbound`）——正是簇 A 回归发生的高风险区。建议 GA 后拆分。
- 🟡 `process_context.rs` 硬编码 macOS socket_fdinfo ABI 偏移，前向兼容脆性（建议 offset_of!/bindgen）。

---

## 七、发布阻塞与优先级

| 优先级 | 类型 | 阻塞项 | 状态 |
|---|---|---|---|
| P0 | dev | daemon 13 个红测试 | ✅ 本次已修 |
| P0 | user | 联调 checklist 125 项 0 勾选（dogfood 未跑） | ⏳ 待用户 |
| P1 | user | ADR-005 海外主体未落地（4-6 周日历）→ 级联卡死域名/密钥/服务端 → 卡死 ADR-029 唯一 GA 指标「装机量」 | ⏳ 待用户 |
| P1 | decision | 三决策：ed25519 密钥托管(荐 GCP KMS)/服务端(荐 Cloudflare Workers+D1)/发布通道(荐 stable) | ⏳ 待拍板 |
| P1 | user/dev | GUI 发布签名链：Sparkle EdDSA 真实密钥 + Apple Notarization Team ID（脚本已就绪，缺凭证） | ⏳ 待用户 |
| P1 | dev | Ed25519 规则包验签 fail-open（GA 前闭环 + 编译期断言） | 待办 |
| P2 | dev | 热路径 expect 重构 / daemon.rs 拆分 / SPEC §14 fixture 机制落地 | 待办 |
| P2 | docs | SPEC-005 pin 回填 + ~39 处悬空引用 | ⏳ 本次修复中 |

---

## 八、本次会话处理进展（2026-06-07）

- ✅ P0 daemon 13 个红测试全部修复，760/0/7 验证通过，fmt/clippy 全绿
- ✅ 测试数谎报纠正（PROGRESS + CLAUDE.md）
- ✅ 下游文档同步（CHANGELOG/ADR-026/api-reference/data-model）
- ⏳ SPEC-005 pin 回填（GUI 仓 upstream-references.md）
- ⏳ sieve/docs ~39 处悬空引用修复
- ⏭ 待用户：dogfood 联调 / 海外主体 / 三决策 / GUI 签名凭证
