OpenAI Codex v0.125.0 (research preview)
--------
workdir: /Users/doskey/src/sieve
model: gpt-5.5
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/doskey/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019dd3f2-ed3d-7b33-8de4-a72a1b35569c
--------
user
current changes
exec
/bin/zsh -lc 'git status --short --branch' in /Users/doskey/src/sieve
 succeeded in 0ms:
## main...origin/main
 M .github/workflows/ci.yml
 M .github/workflows/fuzz-nightly.yml
 M .github/workflows/release.yml
 M crates/sieve-cli/src/commands/doctor.rs
 M crates/sieve-cli/src/daemon.rs
 M crates/sieve-cli/src/main.rs
 M crates/sieve-hook/src/lib.rs
 M crates/sieve-hook/src/pending.rs
 M docs/specs/SPEC-003-sieve-setup-tool.md
 M tasks/known-issues-v1.4.md
?? crates/sieve-cli/tests/doctor.rs

exec
/bin/zsh -lc 'git diff --stat' in /Users/doskey/src/sieve
exec
/bin/zsh -lc 'git diff --cached --stat' in /Users/doskey/src/sieve
 succeeded in 0ms:
 .github/workflows/ci.yml                |  28 +++---
 .github/workflows/fuzz-nightly.yml      |   4 +
 .github/workflows/release.yml           |  19 +---
 crates/sieve-cli/src/commands/doctor.rs | 166 ++++++++++++++++++-------------
 crates/sieve-cli/src/daemon.rs          | 143 ++++++++++++++++++++++-----
 crates/sieve-cli/src/main.rs            |   6 +-
 crates/sieve-hook/src/lib.rs            | 168 ++++++++++++++++++++++++++++++--
 crates/sieve-hook/src/pending.rs        |  31 +++++-
 docs/specs/SPEC-003-sieve-setup-tool.md |  20 +++-
 tasks/known-issues-v1.4.md              |  92 ++++++-----------
 10 files changed, 480 insertions(+), 197 deletions(-)

 succeeded in 0ms:
exec
/bin/zsh -lc 'git diff -- .github/workflows/ci.yml .github/workflows/fuzz-nightly.yml .github/workflows/release.yml' in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
index f1c8b4c..3ba31b7 100644
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -1,5 +1,7 @@
 name: CI
 
+# Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux/Windows 推 Phase 2 后才恢复 matrix
+
 on:
   push:
     branches: [main]
@@ -14,7 +16,7 @@ env:
 jobs:
   fmt:
     name: rustfmt
-    runs-on: ubuntu-latest
+    runs-on: macos-14
     steps:
       - uses: actions/checkout@v4
       - name: 安装 toolchain（via rust-toolchain.toml）
@@ -23,42 +25,31 @@ jobs:
 
   clippy:
     name: clippy
-    runs-on: ubuntu-latest
+    runs-on: macos-14
     steps:
       - uses: actions/checkout@v4
       - run: rustup show
       - uses: Swatinem/rust-cache@v2
       - name: 安装构建依赖（vectorscan 需要 cmake/ninja/ragel）
         run: |
-          sudo apt-get update
-          sudo apt-get install -y cmake ninja-build pkg-config libssl-dev libboost-dev ragel
+          brew install cmake ninja pkg-config boost ragel
       - run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
 
   test:
-    name: test (${{ matrix.os }})
-    strategy:
-      fail-fast: false
-      matrix:
-        os: [ubuntu-latest, macos-14]
-    runs-on: ${{ matrix.os }}
+    name: test (macos-14)
+    runs-on: macos-14
     steps:
       - uses: actions/checkout@v4
       - run: rustup show
       - uses: Swatinem/rust-cache@v2
-      - name: 安装构建依赖（Linux）
-        if: runner.os == 'Linux'
-        run: |
-          sudo apt-get update
-          sudo apt-get install -y cmake ninja-build pkg-config libssl-dev libboost-dev ragel
       - name: 安装构建依赖（macOS）
-        if: runner.os == 'macOS'
         run: |
           brew install cmake ninja pkg-config boost ragel
       - run: cargo test --workspace --locked
 
   deny:
     name: cargo-deny
-    runs-on: ubuntu-latest
+    runs-on: macos-14
     steps:
       - uses: actions/checkout@v4
       # ADR-003: 检查出站依赖来源、license 合规、漏洞
@@ -68,6 +59,9 @@ jobs:
           arguments: --all-features
 
   fuzz-quick:
+    # cargo-fuzz 依赖 nightly + libfuzzer，在 Linux 上更稳定（macOS 支持有限）。
+    # fuzz 是 nightly 调研工具，不是发布支撑路径，保留 ubuntu-latest runner。
+    # Phase 1 仅 macOS 的约束不强制要求 fuzz runner 也切 macOS。
     name: fuzz-quick (cargo fuzz, 60s/target)
     runs-on: ubuntu-latest
     # 关 ASan 跑 fuzz：rust-fuzz/cargo-fuzz#404 已知问题。LLVM 20 的 ASan
diff --git a/.github/workflows/fuzz-nightly.yml b/.github/workflows/fuzz-nightly.yml
index ad7312e..1174f1b 100644
--- a/.github/workflows/fuzz-nightly.yml
+++ b/.github/workflows/fuzz-nightly.yml
@@ -1,5 +1,9 @@
 name: Fuzz Nightly (AFL++)
 
+# Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux/Windows 推 Phase 2 后才恢复 matrix
+# 例外：AFL++ fuzz 是 nightly 调研工具，不是发布支撑路径，保留 ubuntu-latest runner。
+# AFL++ 在 macOS 支持有限（部分 fork server 特性需要 Linux kernel），等 Phase 2 再评估迁移。
+
 # Week 3 暂禁(workflow_dispatch only)，Week 6+ 启用 schedule
 on:
   workflow_dispatch:
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index cea4d3c..194bf28 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -1,5 +1,7 @@
 name: Release
 
+# Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux/Windows 推 Phase 2 后才恢复 matrix
+
 on:
   # ADR-011: Tag-based release 在 Week 12 GA 时启用；GA 前仅用 workflow_dispatch 触发
   # 以减少 Rekor 透明日志记录 release 节奏（防止对手推断项目进度）
@@ -26,6 +28,7 @@ jobs:
   # ─────────────────────────────────────────────────────────────
   # Tier 1 平台：reproducible build + cosign 签名
   # ADR-006 §2: Tier 1 失败 → release 中止（hard gate）
+  # Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux target 推 Phase 2
   # ─────────────────────────────────────────────────────────────
   reproducible-build:
     name: Reproducible build (${{ matrix.target }})
@@ -39,27 +42,13 @@ jobs:
           - target: x86_64-apple-darwin
             os: macos-14
             artifact: sieve-x86_64-apple-darwin
-          # Week 1 Linux 用 gnu(glibc):Ubuntu musl-tools 不含 musl-g++,vectorscan(C++ 库)无法
-          # 编译。reproducibility 与 musl/gnu 选择无关(关键是 SOURCE_DATE_EPOCH + remap-prefix +
-          # 双构建 SHA-256)。Week 6 Tier 2 切回 musl 静态链接(用 cross-rs docker 镜像或预编译
-          # musl-cross)。见 docs/design/ADR-006-sigstore-reproducible-build.md。
-          - target: x86_64-unknown-linux-gnu
-            os: ubuntu-latest
-            artifact: sieve-x86_64-unknown-linux-gnu
     runs-on: ${{ matrix.os }}
     steps:
       - uses: actions/checkout@v4
         with:
           fetch-depth: 0  # 需要完整 git 历史以获取 commit timestamp
 
-      - name: 安装构建依赖（Linux）
-        if: runner.os == 'Linux'
-        run: |
-          sudo apt-get update
-          sudo apt-get install -y cmake ninja-build pkg-config libboost-dev ragel
-
       - name: 安装构建依赖（macOS）
-        if: runner.os == 'macOS'
         run: |
           brew install cmake ninja pkg-config boost ragel
 
@@ -212,7 +201,7 @@ jobs:
   publish:
     name: 发布 GitHub Release
     needs: [reproducible-build, macos-universal]
-    runs-on: ubuntu-latest
+    runs-on: macos-14
     if: startsWith(github.ref, 'refs/tags/v')
     steps:
       - uses: actions/checkout@v4

exec
/bin/zsh -lc 'git diff -- crates/sieve-cli/src/commands/doctor.rs crates/sieve-cli/src/daemon.rs crates/sieve-cli/src/main.rs crates/sieve-hook/src/lib.rs crates/sieve-hook/src/pending.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/crates/sieve-cli/src/commands/doctor.rs b/crates/sieve-cli/src/commands/doctor.rs
index ba3888d..b73e1e7 100644
--- a/crates/sieve-cli/src/commands/doctor.rs
+++ b/crates/sieve-cli/src/commands/doctor.rs
@@ -5,9 +5,24 @@
 //! 2. hooks.PreToolUse 是否含 sieve-hook check
 //! 3. daemon 是否在 :11453 监听（TCP 连接）
 //! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
-//! 5. canary 拦截测试（构造 OUT-01 已知字符串，验证 daemon 改写）
+//! 5. canary 本地引擎命中测试（OUT-01 规则 scan，不发真实网络请求）
 //!
 //! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
+//!
+//! # R4-#7 修复说明
+//!
+//! 原实现向 daemon 发 HTTP 请求，检查响应里**不含**原始 canary token。
+//! 该逻辑存在误报通过漏洞：daemon 未拦截（401/502 透传）时响应同样不含 canary token。
+//!
+//! 新实现改为**直接调用本地 sieve-rules 引擎**对 canary token 做 scan，
+//! 确认规则引擎确实命中 OUT-01，不依赖 daemon 是否在线。
+//! 同时独立检查 daemon TCP 监听（检查 3）。
+//! 输出明确区分「规则引擎命中」与「daemon 在线」两个状态。
+//!
+//! # R4-#8 修复说明
+//!
+//! 原实现任一检查失败仍返回 `Ok(())`，导致 CI 假绿灯。
+//! 新实现收集所有失败项，任一失败则返回 `Err`，含失败项名称列表。
 
 use anyhow::Result;
 
@@ -22,17 +37,21 @@
 #[cfg(target_os = "macos")]
 mod macos {
     use super::*;
-    use std::io::Write as IoWrite;
     use std::process::Command;
 
     /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
+    ///
+    /// # Errors
+    ///
+    /// 任一检查项失败时返回 `Err`，错误信息含失败项名称列表（R4-#8）。
     pub fn run() -> Result<()> {
         let home = std::env::var("HOME").unwrap_or_default();
         let settings_path = std::path::PathBuf::from(&home)
             .join(".claude")
             .join("settings.json");
 
-        let mut all_ok = true;
+        // 收集每项检查的结果 (label, passed)
+        let mut results: Vec<(&str, bool)> = Vec::new();
 
         // ── 检查 1: ANTHROPIC_BASE_URL
         let check1 = check_base_url(&settings_path);
@@ -40,7 +59,7 @@ pub fn run() -> Result<()> {
             "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
             check1,
         );
-        all_ok &= check1;
+        results.push(("ANTHROPIC_BASE_URL 配置", check1));
 
         // ── 检查 2: PreToolUse hook
         let check2 = check_hook_registered(&settings_path);
@@ -48,32 +67,48 @@ pub fn run() -> Result<()> {
             "settings.json: hooks.PreToolUse 含 sieve-hook check",
             check2,
         );
-        all_ok &= check2;
+        results.push(("PreToolUse hook 配置", check2));
 
         // ── 检查 3: daemon 监听 :11453
         let check3 = check_daemon_listening();
         print_check("daemon 在 127.0.0.1:11453 监听", check3);
-        all_ok &= check3;
+        results.push(("daemon 监听 :11453", check3));
 
         // ── 检查 4: launchd 状态
         let check4 = check_launchd();
         print_check("launchd com.sieve.daemon 已加载", check4);
-        all_ok &= check4;
-
-        // ── 检查 5: canary 拦截测试
-        let check5 = check_canary();
-        print_check("canary 拦截测试（OUT-01 脱敏）", check5);
-        all_ok &= check5;
+        results.push(("launchd 服务已加载", check4));
+
+        // ── 检查 5: canary 本地引擎命中测试（R4-#7 修复）
+        //
+        // 直接调用本地 sieve-rules 引擎扫描 canary token，
+        // 确认 OUT-01 规则确实命中。不发真实网络请求，不依赖 daemon 是否在线。
+        // 输出明确说明「仅验证规则引擎 + daemon listening；端到端验证需手动测」。
+        let check5 = check_canary_local_engine();
+        print_check(
+            "canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）",
+            check5,
+        );
+        results.push(("canary 规则引擎命中 OUT-01", check5));
 
-        // ── 汇总
+        // ── 汇总（R4-#8 修复）
         println!();
-        if all_ok {
+        let failures: Vec<&str> = results
+            .iter()
+            .filter_map(|(label, ok)| if *ok { None } else { Some(*label) })
+            .collect();
+
+        if failures.is_empty() {
             println!("✅ 所有检查通过，Sieve 运行正常。");
+            Ok(())
         } else {
             println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
+            Err(anyhow::anyhow!(
+                "{} 项检查失败：{}",
+                failures.len(),
+                failures.join("、")
+            ))
         }
-
-        Ok(())
     }
 
     fn print_check(label: &str, ok: bool) {
@@ -136,66 +171,63 @@ fn check_launchd() -> bool {
         stdout.contains("com.sieve.daemon")
     }
 
-    /// Canary 拦截测试：向 daemon 发送含 OUT-01 特征的请求，
-    /// 验证响应中已脱敏（不含原始 sk- token）。
+    /// Canary 本地规则引擎命中测试（R4-#7 修复）。
     ///
-    /// 注意：此测试仅在 daemon 运行时有意义；daemon 未运行时直接返回 false。
-    fn check_canary() -> bool {
-        use std::io::{Read, Write};
-        use std::net::TcpStream;
-        use std::time::Duration;
+    /// 构造一个**精确匹配 OUT-01 规则格式**的 canary token，
+    /// 直接调用 sieve-rules VectorscanEngine + 出站规则，验证至少 1 个 Detection 命中 OUT-01。
+    ///
+    /// 不发任何网络请求，不依赖 daemon 是否在线。
+    ///
+    /// # 为什么不发 HTTP 请求验证
+    ///
+    /// - daemon 不支持 runtime upstream override，无法将 canary 请求导向 fake upstream
+    /// - 向真实 upstream 发请求需要有效 API key，doctor 不应持有密钥
+    /// - 401/502 响应同样不含 canary token → 原逻辑误判通过（R4-#7 根本原因）
+    /// - 本地引擎 scan 已足以验证检测链路最关键的一环（规则编译 + pattern 匹配）
+    fn check_canary_local_engine() -> bool {
+        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
+        use sieve_rules::loader::load_outbound_rules;
+
+        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
+        // 在测试环境中，从 CARGO_MANIFEST_DIR 推断；生产环境从二进制同级目录推断。
+        let rules_candidates: Vec<std::path::PathBuf> = vec![
+            // 生产：~/.sieve/rules/outbound.toml
+            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
+                .join(".sieve")
+                .join("rules")
+                .join("outbound.toml"),
+            // 开发：workspace 相对路径（通过 SIEVE_RULES_PATH 覆盖）
+            std::path::PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
+        ];
+
+        let rules_path = rules_candidates
+            .into_iter()
+            .find(|p| !p.as_os_str().is_empty() && p.exists());
+
+        let Some(rules_path) = rules_path else {
+            // 规则文件不存在：canary 检查无法执行
+            return false;
+        };
 
-        // daemon 未运行直接 false
-        let Ok(mut stream) = TcpStream::connect_timeout(
-            &"127.0.0.1:11453".parse().unwrap(),
-            Duration::from_millis(500),
-        ) else {
+        let Ok(rules) = load_outbound_rules(&rules_path) else {
             return false;
         };
-        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
-
-        // 构造含已知 OUT-01 特征（sk-ant-api03-... 格式）的请求体
-        // 注意：这里使用测试用虚假 token，格式符合 OUT-01 模式
-        let canary_token = "sk-ant-api03-canary-test-aaaabbbbccccdddd-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX_AA";
-        let body = serde_json::json!({
-            "model": "claude-3-5-haiku-20241022",
-            "max_tokens": 1,
-            "messages": [{
-                "role": "user",
-                "content": format!("hello {canary_token}")
-            }]
-        })
-        .to_string();
-
-        let request = format!(
-            "POST /v1/messages HTTP/1.1\r\n\
-             Host: 127.0.0.1:11453\r\n\
-             Content-Type: application/json\r\n\
-             Content-Length: {}\r\n\
-             x-api-key: test\r\n\
-             anthropic-version: 2023-06-01\r\n\
-             Connection: close\r\n\
-             \r\n\
-             {}",
-            body.len(),
-            body
-        );
 
-        if stream.write_all(request.as_bytes()).is_err() {
+        let Ok(engine) = VectorscanEngine::compile(rules) else {
             return false;
-        }
+        };
 
-        let mut response = String::new();
-        let _ = stream.read_to_string(&mut response);
+        // 构造精确匹配 OUT-01 pattern `sk-ant-api03-[a-zA-Z0-9_\-]{93}AA` 的 canary token。
+        // body = "canaryDOCTOR" (12) + "test" (4) + 'a'*77 = 93 字符，后跟 "AA"。
+        // 整体格式符合真实 Anthropic API key 结构，确保 OUT-01 命中而非误判。
+        let canary_token = "sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA";
 
-        // 验证响应中不含原始 canary token（已被脱敏/拦截）
-        !response.contains(canary_token)
-    }
+        let Ok(hits) = engine.scan(canary_token.as_bytes()) else {
+            return false;
+        };
 
-    // 抑制 IoWrite 未使用警告
-    const _: fn() = || {
-        let _ = std::io::stdout().flush();
-    };
+        hits.iter().any(|h| h.rule_id == "OUT-01")
+    }
 }
 
 // ──────────────────────────────── 非 macOS stub ─────────────────────────────
diff --git a/crates/sieve-cli/src/daemon.rs b/crates/sieve-cli/src/daemon.rs
index fc6c54d..151b823 100644
--- a/crates/sieve-cli/src/daemon.rs
+++ b/crates/sieve-cli/src/daemon.rs
@@ -582,9 +582,18 @@ async fn forward_with_inbound_inspection(
                         return;
                     }
 
-                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
+                    // 2. Hook 类：写 pending 文件，失败时 fail-closed（不允许 fail-open）
                     for d in &hook_detections {
-                        write_hook_pending_silent(d);
+                        if let Err(e) = write_hook_pending_or_fail_closed(d) {
+                            tracing::error!(
+                                error = %e,
+                                rule = %d.rule_id,
+                                "Hook pending write failed; fail-closed: truncating SSE stream"
+                            );
+                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
+                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
+                            return;
+                        }
                     }
 
                     // 3. GUI 类：hold 流 + keep-alive + 等用户决策
@@ -726,8 +735,18 @@ async fn forward_with_inbound_inspection(
         let (blocking, hook_detections, flush_hold_detections) =
             classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
 
+        // flush 阶段 Hook 类同样 fail-closed：写失败即截流
         for d in &hook_detections {
-            write_hook_pending_silent(d);
+            if let Err(e) = write_hook_pending_or_fail_closed(d) {
+                tracing::error!(
+                    error = %e,
+                    rule = %d.rule_id,
+                    "Hook pending write failed (flush); fail-closed: truncating SSE stream"
+                );
+                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
+                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
+                return;
+            }
         }
 
         if !blocking.is_empty() {
@@ -843,20 +862,29 @@ fn classify_inbound_detections(
     (blocking, hook_detections, hold_detections)
 }
 
-/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
+/// 写 IPC pending 文件，失败时返回 `Err`（调用方负责 fail-closed）。
 ///
-/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
-/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
-fn write_hook_pending_silent(d: &sieve_core::Detection) {
-    use chrono::Utc;
+/// 旧函数 `write_hook_pending_silent` 只 warn 后继续，违反 fail-closed 原则。
+/// 新函数返回 `Result`，调用方在 `Err` 时必须注入 `sieve_blocked` 并截流。
+///
+/// 关联 PRD §9 #3（Critical 不可关）、ADR-014 §Hook 路径、SPEC-001 §3.1。
+fn write_hook_pending_or_fail_closed(
+    d: &sieve_core::Detection,
+) -> Result<(), sieve_ipc::error::IpcError> {
+    let sieve_home = sieve_ipc::paths::sieve_home()?;
+    write_hook_pending_to(d, &sieve_home)
+}
 
-    let sieve_home = match sieve_ipc::paths::sieve_home() {
-        Ok(h) => h,
-        Err(e) => {
-            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
-            return;
-        }
-    };
+/// 写 IPC pending 文件到指定 base 目录，失败时返回 `Err`。
+///
+/// 内部实现，分离出来方便测试注入临时路径，不依赖环境变量。
+///
+/// 关联 SPEC-001 §3.1、ADR-014 §Hook 路径。
+fn write_hook_pending_to(
+    d: &sieve_core::Detection,
+    sieve_home: &std::path::Path,
+) -> Result<(), sieve_ipc::error::IpcError> {
+    use chrono::Utc;
 
     let request_id = uuid::Uuid::new_v4();
     let ipc_req = sieve_ipc::DecisionRequest {
@@ -874,15 +902,15 @@ fn write_hook_pending_silent(d: &sieve_core::Detection) {
         }],
     };
 
-    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
-        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
-    } else {
-        tracing::info!(
-            rule = %d.rule_id,
-            request_id = %request_id,
-            "HookMark: pending file written, SSE stream continues"
-        );
-    }
+    sieve_ipc::pending_file::write_pending(&ipc_req, sieve_home)?;
+
+    tracing::info!(
+        rule = %d.rule_id,
+        request_id = %request_id,
+        "HookMark: pending file written, SSE stream continues"
+    );
+
+    Ok(())
 }
 
 /// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
@@ -1212,3 +1240,70 @@ fn apply_redacted_texts_to_request(
         extra: req.extra.clone(),
     })
 }
+
+// ─── 单元测试：Hook pending fail-closed ──────────────────────────────────────
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use sieve_core::detection::{Action, ContentSource, Detection, Severity};
+    use sieve_core::protocol::unified_message::ContentSpan;
+    use uuid::Uuid;
+
+    /// 构造最小化的 HookMark Detection，用于测试 write_hook_pending_to。
+    fn make_hook_detection() -> Detection {
+        Detection {
+            id: Uuid::new_v4(),
+            rule_id: "IN-CR-02".to_string(),
+            severity: Severity::Critical,
+            action: Action::HookMark,
+            source: ContentSource::InboundToolUseInput,
+            span: ContentSpan { start: 0, end: 10 },
+            evidence_truncated: "rm -rf /".to_string(),
+            fingerprint: "deadbeef01234567".to_string(),
+        }
+    }
+
+    /// happy path：base 目录可写 → 返回 Ok，pending 文件存在。
+    ///
+    /// 验证 HookMark 写成功后调用方可继续转发 SSE 流，不触发 fail-closed。
+    /// 关联 PRD §9 #3、SPEC-001 §3.1。
+    #[test]
+    fn hook_pending_write_happy_path() {
+        let tmp = tempfile::tempdir().expect("tempdir");
+        let d = make_hook_detection();
+
+        let result = write_hook_pending_to(&d, tmp.path());
+
+        assert!(result.is_ok(), "可写目录应返回 Ok，得到: {result:?}");
+
+        // 验证 pending 目录下有 .json 文件
+        let pending_dir = tmp.path().join("pending");
+        let entries: Vec<_> = std::fs::read_dir(&pending_dir)
+            .expect("pending dir should exist")
+            .filter_map(|e| e.ok())
+            .collect();
+        assert!(
+            !entries.is_empty(),
+            "pending 目录应有写入的 .json 文件，但为空"
+        );
+    }
+
+    /// fail-closed：base 指向不可写路径 → 返回 Err（调用方应注入 sieve_blocked 截流）。
+    ///
+    /// 确认 Hook pending 写失败必须返回 Err，禁止 fail-open。
+    /// 关联 PRD §9 #3 fail-closed 硬约束、ADR-007（fail-closed 语义）。
+    #[test]
+    fn hook_pending_write_fails_on_unwritable_base() {
+        // /dev/null 在 macOS/Linux 上是字符设备，不是目录，create_dir_all 必然失败
+        let unwritable = std::path::Path::new("/dev/null/nonexistent_sieve_home");
+        let d = make_hook_detection();
+
+        let result = write_hook_pending_to(&d, unwritable);
+
+        assert!(
+            result.is_err(),
+            "不可写 base 应返回 Err 以触发 fail-closed，但得到 Ok"
+        );
+    }
+}
diff --git a/crates/sieve-cli/src/main.rs b/crates/sieve-cli/src/main.rs
index e1bf315..4af94e3 100644
--- a/crates/sieve-cli/src/main.rs
+++ b/crates/sieve-cli/src/main.rs
@@ -136,7 +136,11 @@ async fn main() -> Result<()> {
             commands::setup::run(args)?;
         }
         Command::Doctor => {
-            commands::doctor::run()?;
+            // R4-#8：doctor 失败时返回非零 exit code，CI 脚本可捕获。
+            if let Err(e) = commands::doctor::run() {
+                eprintln!("sieve doctor: {e}");
+                std::process::exit(1);
+            }
         }
         Command::Uninstall(args) => {
             commands::uninstall::run(args)?;
diff --git a/crates/sieve-hook/src/lib.rs b/crates/sieve-hook/src/lib.rs
index 6d69ce0..8d344a9 100644
--- a/crates/sieve-hook/src/lib.rs
+++ b/crates/sieve-hook/src/lib.rs
@@ -58,17 +58,31 @@ pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
 
 /// 启发式运行逻辑：无 request_id 时扫目录。
 ///
-/// 优先级 3（SPEC-001 §4.3）：
-/// - 零 fresh pending → fail-open（exit 0）
-/// - stale 文件 → 删除 + warn + fail-open（exit 0）
-/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
+/// 优先级 3（SPEC-001 §4.3），决策表（P1-R3-#6 修复后）：
+/// - fresh=[] && stale=[] && corrupt=[] → fail-open（exit 0）：Sieve 未标记任何请求
+/// - corrupt 非空 → fail-closed（exit 1）：无法确认 Sieve 判定，保守拒绝
+/// - fresh 非空（corrupt=[]） → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
+/// - fresh=[] && stale 非空（corrupt=[]） → 删 stale + fail-open（exit 0）
 ///   多 pending 时用户一次决策广播给所有 request_id。
 ///
 /// 返回进程退出码：0 = 允许，1 = 拒绝。
-/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
+/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）；known-issues-v1.4.md §P1-R3-#6。
 pub fn run_check_heuristic(base: &Path) -> i32 {
     let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);
 
+    // 损坏文件优先检查：只要有损坏文件，立即 fail-closed，不管 fresh 有没有。
+    // 因为损坏文件可能对应本次工具调用的 Sieve 拦截标记，无法安全放行。
+    // 关联：P1-R3-#6（corrupt → fail-open 漏洞修复）。
+    if !scan.corrupt_paths.is_empty() {
+        for corrupt_path in &scan.corrupt_paths {
+            eprintln!(
+                "sieve-hook: pending file {} corrupt, refusing tool call to be safe",
+                corrupt_path.display()
+            );
+        }
+        return 1;
+    }
+
     // 删除 stale 文件 + 打 warning。
     for stale_path in &scan.stale_paths {
         eprintln!(
@@ -79,7 +93,7 @@ pub fn run_check_heuristic(base: &Path) -> i32 {
     }
 
     if scan.fresh.is_empty() {
-        // 零 pending：Sieve 代理未标记任何请求，fail-open。
+        // 零 pending（corrupt=[]，stale 已清理）：Sieve 代理未标记任何请求，fail-open。
         return 0;
     }
 
@@ -538,4 +552,146 @@ fn full_lifecycle_no_repeat_popup() {
             "second scan should return empty stale_paths"
         );
     }
+
+    // ════════════════════════════════════════════════════════════════════════
+    // P1-R3-#6 修复：corrupt pending 文件 fail-closed 的 7 个新测试
+    // ════════════════════════════════════════════════════════════════════════
+
+    fn write_corrupt_pending(base: &Path, filename: &str) {
+        let dir = base.join("pending");
+        std::fs::create_dir_all(&dir).unwrap();
+        std::fs::write(dir.join(filename), b"not valid json{{{").unwrap();
+    }
+
+    // 测试 13：scan 包含 corrupt 文件 → corrupt_paths 非空，fresh/stale 不变
+    #[test]
+    fn scan_corrupt_json_goes_to_corrupt_paths() {
+        let tmp = tempfile::tempdir().unwrap();
+        write_corrupt_pending(tmp.path(), "corrupt-A.json");
+
+        let result = pending::scan_pending_dir(tmp.path(), 600);
+        assert_eq!(
+            result.corrupt_paths.len(),
+            1,
+            "corrupt json file should appear in corrupt_paths"
+        );
+        assert!(
+            result.fresh.is_empty(),
+            "corrupt file should not appear in fresh"
+        );
+        assert!(
+            result.stale_paths.is_empty(),
+            "corrupt file should not appear in stale_paths"
+        );
+        let corrupt_name = result.corrupt_paths[0]
+            .file_name()
+            .unwrap()
+            .to_str()
+            .unwrap();
+        assert_eq!(corrupt_name, "corrupt-A.json");
+    }
+
+    // 测试 14：scan IO 错误（chmod 000 文件）→ 算 corrupt（unix only）
+    #[cfg(unix)]
+    #[test]
+    fn scan_io_error_goes_to_corrupt_paths() {
+        use std::os::unix::fs::PermissionsExt;
+
+        let tmp = tempfile::tempdir().unwrap();
+        let pending_dir = tmp.path().join("pending");
+        std::fs::create_dir_all(&pending_dir).unwrap();
+        let file_path = pending_dir.join("unreadable.json");
+        std::fs::write(&file_path, b"{}").unwrap();
+        // 移除读权限，使 read_to_string 失败。
+        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000)).unwrap();
+
+        let result = pending::scan_pending_dir(tmp.path(), 600);
+
+        // 恢复权限（tempdir drop 时需要能清理）。
+        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644)).unwrap();
+
+        assert_eq!(
+            result.corrupt_paths.len(),
+            1,
+            "IO-unreadable file should appear in corrupt_paths"
+        );
+        assert!(result.fresh.is_empty());
+        assert!(result.stale_paths.is_empty());
+    }
+
+    // 测试 15：scan 全 corrupt → fresh=[]，corrupt_paths 非空
+    #[test]
+    fn scan_all_corrupt_yields_empty_fresh() {
+        let tmp = tempfile::tempdir().unwrap();
+        write_corrupt_pending(tmp.path(), "bad1.json");
+        write_corrupt_pending(tmp.path(), "bad2.json");
+
+        let result = pending::scan_pending_dir(tmp.path(), 600);
+        assert!(
+            result.fresh.is_empty(),
+            "all-corrupt scan should yield empty fresh"
+        );
+        assert_eq!(
+            result.corrupt_paths.len(),
+            2,
+            "all corrupt files should appear in corrupt_paths"
+        );
+    }
+
+    // 测试 16：run_check_heuristic 全 corrupt → exit 1（fail-closed）
+    #[test]
+    fn heuristic_all_corrupt_returns_1() {
+        let tmp = tempfile::tempdir().unwrap();
+        write_corrupt_pending(tmp.path(), "evil.json");
+
+        let code = super::run_check_heuristic(tmp.path());
+        assert_eq!(code, 1, "all-corrupt pending should fail-closed (exit 1)");
+    }
+
+    // 测试 17：run_check_heuristic 混合 fresh + corrupt → exit 1（保守 fail-closed）
+    #[test]
+    fn heuristic_mixed_fresh_and_corrupt_returns_1() {
+        let tmp = tempfile::tempdir().unwrap();
+        // 写一个合法 fresh pending（Allow）。
+        let id = Uuid::now_v7();
+        write_pending_json(
+            tmp.path(),
+            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
+        );
+        // 再写一个损坏文件。
+        write_corrupt_pending(tmp.path(), "corrupt.json");
+
+        let code = super::run_check_heuristic(tmp.path());
+        assert_eq!(
+            code, 1,
+            "mixed fresh+corrupt should fail-closed (exit 1), corrupt wins"
+        );
+    }
+
+    // 测试 18：run_check_heuristic 仅 stale → 删 stale + exit 0（保持原行为）
+    #[test]
+    fn heuristic_only_stale_deletes_and_returns_0() {
+        let tmp = tempfile::tempdir().unwrap();
+        let id = Uuid::now_v7();
+        let stale_time = Utc::now() - Duration::minutes(11);
+        let req = make_req(id, DefaultOnTimeout::Block, stale_time);
+        write_pending_json(tmp.path(), &req);
+
+        let pending_file = tmp.path().join("pending").join(format!("{id}.json"));
+        let code = super::run_check_heuristic(tmp.path());
+        assert_eq!(code, 0, "stale-only should fail-open (exit 0)");
+        assert!(
+            !pending_file.exists(),
+            "stale file should be deleted after heuristic run"
+        );
+    }
+
+    // 测试 19：run_check_heuristic 全空 → exit 0（fail-open 默认）
+    #[test]
+    fn heuristic_completely_empty_returns_0() {
+        let tmp = tempfile::tempdir().unwrap();
+        // pending 目录不存在，完全空白。
+        let code = super::run_check_heuristic(tmp.path());
+        assert_eq!(code, 0, "completely empty state should fail-open (exit 0)");
+    }
 }
diff --git a/crates/sieve-hook/src/pending.rs b/crates/sieve-hook/src/pending.rs
index 5f60116..de03e8a 100644
--- a/crates/sieve-hook/src/pending.rs
+++ b/crates/sieve-hook/src/pending.rs
@@ -49,6 +49,12 @@ pub struct ScanResult {
     pub fresh: Vec<DecisionRequest>,
     /// 过期的 pending 文件路径（供调用方删除）。
     pub stale_paths: Vec<std::path::PathBuf>,
+    /// 损坏的 pending 文件路径（IO 读取失败或 JSON 解析失败）。
+    ///
+    /// 调用方收到非空 corrupt_paths 时必须 fail-closed（exit 1），
+    /// 因为无法确定 Sieve 对这些请求的判定结果。
+    /// 关联：known-issues-v1.4.md §P1-R3-#6（fail-open 漏洞修复）。
+    pub corrupt_paths: Vec<std::path::PathBuf>,
 }
 
 /// 扫描 `<base>/pending/` 目录，收集所有未过期的 pending 文件。
@@ -61,12 +67,17 @@ pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
     let pending_dir = base.join("pending");
     let mut fresh: Vec<DecisionRequest> = Vec::new();
     let mut stale_paths: Vec<std::path::PathBuf> = Vec::new();
+    let mut corrupt_paths: Vec<std::path::PathBuf> = Vec::new();
 
     let entries = match std::fs::read_dir(&pending_dir) {
         Ok(e) => e,
         Err(_) => {
             // 目录不存在或无权读 → 视为空目录，fail-open。
-            return ScanResult { fresh, stale_paths };
+            return ScanResult {
+                fresh,
+                stale_paths,
+                corrupt_paths,
+            };
         }
     };
 
@@ -82,11 +93,19 @@ pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
         }
         let content = match std::fs::read_to_string(&path) {
             Ok(c) => c,
-            Err(_) => continue,
+            Err(_) => {
+                // IO 读取失败 → 算损坏，不 skip（P1-R3-#6）。
+                corrupt_paths.push(path);
+                continue;
+            }
         };
         let req: DecisionRequest = match serde_json::from_str(&content) {
             Ok(r) => r,
-            Err(_) => continue, // 解析失败的文件跳过。
+            Err(_) => {
+                // JSON 解析失败 → 算损坏，不 skip（P1-R3-#6）。
+                corrupt_paths.push(path);
+                continue;
+            }
         };
 
         // 已决策的 pending 跳过（避免重复弹窗）。
@@ -108,5 +127,9 @@ pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
     // 按 created_at 升序排列，保证确定性。
     fresh.sort_by_key(|r| r.created_at);
 
-    ScanResult { fresh, stale_paths }
+    ScanResult {
+        fresh,
+        stale_paths,
+        corrupt_paths,
+    }
 }

exec
/bin/zsh -lc 'cat crates/sieve-cli/tests/doctor.rs' in /Users/doskey/src/sieve
 succeeded in 0ms:
//! `sieve doctor` 集成测试（R4-#7 + R4-#8 修复验证）。
//!
//! 仅 macOS 编译运行（`#[cfg(target_os = "macos")]`）。
//!
//! 测试矩阵：
//! - R4-#7-T1: canary token 确实命中本地引擎 OUT-01
//! - R4-#7-T2: daemon 未在线 → canary 检查不误判通过（SIEVE_RULES_PATH 指向无效路径）
//! - R4-#8-T1: 任一检查失败 → run() 返回 Err，含失败项名
//! - R4-#8-T2: sieve doctor 命令 exit code 非零（受限 HOME，检查必然失败）

#![cfg(target_os = "macos")]

use std::path::PathBuf;

/// 找到 workspace 下的 outbound.toml 路径。
fn outbound_rules_path() -> PathBuf {
    // 集成测试运行时 CARGO_MANIFEST_DIR = crates/sieve-cli
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent() // crates/
        .unwrap()
        .join("sieve-rules")
        .join("rules")
        .join("outbound.toml")
}

// ─────────────────────────────────────────────────────────────────
// R4-#7-T1: canary token 真命中本地 OUT-01 规则
// ─────────────────────────────────────────────────────────────────

/// 直接用 sieve-rules 引擎 scan canary token，验证命中 OUT-01。
///
/// 这是对 doctor::check_canary_local_engine 核心逻辑的镜像测试：
/// 确认我们选的 canary token 在 outbound.toml 规则下确实命中 OUT-01。
#[test]
fn canary_token_hits_out01_in_local_engine() {
    use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
    use sieve_rules::loader::load_outbound_rules;

    let rules_path = outbound_rules_path();
    assert!(
        rules_path.exists(),
        "outbound.toml 未找到：{}",
        rules_path.display()
    );

    let rules = load_outbound_rules(&rules_path).expect("加载 outbound.toml 失败");
    let engine = VectorscanEngine::compile(rules).expect("VectorscanEngine 编译失败");

    // 与 doctor::check_canary_local_engine 使用完全相同的 canary token
    // OUT-01 pattern: sk-ant-api03-[a-zA-Z0-9_\-]{93}AA
    // 拆分：前缀 "sk-ant-api03-" + "canaryDOCTOR" (12) + "test" (4) + 'a'*77 (77) = 93 + "AA"
    let canary_token = "sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA";

    let hits = engine.scan(canary_token.as_bytes()).expect("scan 失败");

    let out01_hits: Vec<_> = hits.iter().filter(|h| h.rule_id == "OUT-01").collect();
    assert!(
        !out01_hits.is_empty(),
        "canary token 应命中 OUT-01，实际命中规则: {:?}",
        hits.iter().map(|h| &h.rule_id).collect::<Vec<_>>()
    );
}

// ─────────────────────────────────────────────────────────────────
// R4-#7-T2: 规则文件不存在 → canary 检查失败而非误判通过
// ─────────────────────────────────────────────────────────────────

/// 当 SIEVE_RULES_PATH 指向不存在路径、HOME 也没有 ~/.sieve/rules/outbound.toml 时，
/// check_canary_local_engine（通过 doctor::run 间接调用）应失败而非误判通过。
///
/// 验证方法：在隔离 HOME（无规则文件）下调用 doctor::run，
/// 期望返回 Err（因为多项检查失败，包括 canary）。
#[test]
fn canary_check_fails_when_rules_file_missing() {
    use std::sync::Mutex;
    use tempfile::tempdir;

    // env var 修改需要串行（避免并发测试污染）
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    let _guard = ENV_LOCK.lock().unwrap();

    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();

    // 建 .claude/ 但不放 settings.json，也不放 ~/.sieve/rules/outbound.toml
    std::fs::create_dir_all(fake_home.join(".claude")).unwrap();

    let orig_home = std::env::var("HOME").unwrap_or_default();
    let orig_rules = std::env::var("SIEVE_RULES_PATH").unwrap_or_default();

    // SAFETY: 单线程测试，Mutex 保证串行访问
    unsafe {
        std::env::set_var("HOME", fake_home.to_str().unwrap());
        // 清空 SIEVE_RULES_PATH，确保规则文件找不到
        std::env::set_var("SIEVE_RULES_PATH", "");
    }

    let result = sieve_cli_doctor::run_check_canary_local_engine_via_test_hook();

    // 恢复环境变量
    unsafe {
        std::env::set_var("HOME", &orig_home);
        if orig_rules.is_empty() {
            std::env::remove_var("SIEVE_RULES_PATH");
        } else {
            std::env::set_var("SIEVE_RULES_PATH", &orig_rules);
        }
    }

    // 规则文件不存在时 canary 检查应返回 false
    assert!(!result, "规则文件不存在时 canary 检查不应误判为 true");
}

// ─────────────────────────────────────────────────────────────────
// R4-#8-T1: 任一检查失败 → run() 返回 Err，含失败项名称
// ─────────────────────────────────────────────────────────────────

/// 在受限环境下（无 settings.json / daemon 未启动）调用 doctor::run，
/// 验证返回 Err，且错误信息含失败项名称。
#[test]
fn doctor_run_returns_err_when_checks_fail() {
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    let _guard = ENV_LOCK.lock().unwrap();

    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();
    std::fs::create_dir_all(fake_home.join(".claude")).unwrap();

    let orig_home = std::env::var("HOME").unwrap_or_default();
    let orig_rules = std::env::var("SIEVE_RULES_PATH").unwrap_or_default();

    // SAFETY: 单线程测试，Mutex 保证串行访问
    unsafe {
        std::env::set_var("HOME", fake_home.to_str().unwrap());
        std::env::set_var("SIEVE_RULES_PATH", "");
    }

    // doctor::run() 是 pub，通过 sieve-cli 的 lib 接口调用
    // 注：sieve-cli 是 binary crate，集成测试通过 helper 模块访问
    let result = sieve_cli_doctor::run_doctor();

    unsafe {
        std::env::set_var("HOME", &orig_home);
        if orig_rules.is_empty() {
            std::env::remove_var("SIEVE_RULES_PATH");
        } else {
            std::env::set_var("SIEVE_RULES_PATH", &orig_rules);
        }
    }

    assert!(
        result.is_err(),
        "任一检查失败时 doctor::run 应返回 Err，实际返回 Ok"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("检查失败"),
        "Err 信息应含「检查失败」，实际：{err_msg}"
    );
}

// ─────────────────────────────────────────────────────────────────
// R4-#8-T2: `sieve doctor` exit code 非零（受限 HOME）
// ─────────────────────────────────────────────────────────────────

/// 在受限环境下运行 `sieve doctor` 子进程，验证 exit code != 0。
///
/// 注意：此测试需要 `sieve` 二进制已编译（cargo test 时自动保证）。
/// 通过设置 HOME 到空目录确保所有检查失败。
#[test]
fn sieve_doctor_exits_nonzero_when_checks_fail() {
    use tempfile::tempdir;

    // 找到 cargo 构建的 sieve 二进制
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("sieve");

    if !target_dir.exists() {
        // 二进制未构建，跳过而非 panic
        eprintln!("跳过 sieve_doctor_exits_nonzero_when_checks_fail：sieve 二进制未找到");
        return;
    }

    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();
    std::fs::create_dir_all(fake_home.join(".claude")).unwrap();

    let status = std::process::Command::new(&target_dir)
        .arg("doctor")
        .env("HOME", fake_home.to_str().unwrap())
        .env("SIEVE_RULES_PATH", "")
        // 不继承 SIEVE_LOG，避免噪音
        .env_remove("SIEVE_LOG")
        .status()
        .expect("运行 sieve doctor 失败");

    assert!(
        !status.success(),
        "受限 HOME 下 sieve doctor 应以非零 exit code 退出，实际：{status}"
    );
}

// ─────────────────────────────────────────────────────────────────
// 测试 helper：通过内联模块访问 doctor 内部逻辑
// ─────────────────────────────────────────────────────────────────

/// 由于 sieve-cli 是 binary crate（没有 lib.rs），集成测试无法直接导入内部函数。
/// 这里通过将核心逻辑提取为独立模块并在测试中重新实现来验证行为。
mod sieve_cli_doctor {
    use anyhow::Result;

    /// 镜像 doctor::check_canary_local_engine 逻辑，供测试调用。
    pub fn run_check_canary_local_engine_via_test_hook() -> bool {
        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
        use sieve_rules::loader::load_outbound_rules;
        use std::path::PathBuf;

        let rules_candidates: Vec<PathBuf> = vec![
            PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".sieve")
                .join("rules")
                .join("outbound.toml"),
            PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
        ];

        let rules_path = rules_candidates
            .into_iter()
            .find(|p| !p.as_os_str().is_empty() && p.exists());

        let Some(rules_path) = rules_path else {
            return false;
        };

        let Ok(rules) = load_outbound_rules(&rules_path) else {
            return false;
        };

        let Ok(engine) = VectorscanEngine::compile(rules) else {
            return false;
        };

        let canary_token = "sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA";

        let Ok(hits) = engine.scan(canary_token.as_bytes()) else {
            return false;
        };

        hits.iter().any(|h| h.rule_id == "OUT-01")
    }

    /// 镜像 doctor::run 的核心逻辑，使用可控的 HOME 环境。
    ///
    /// 简化版本：只验证 settings.json 存在 + canary 本地引擎命中，
    /// 不调用 launchctl（避免系统依赖）。
    pub fn run_doctor() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_default();
        let settings_path = std::path::PathBuf::from(&home)
            .join(".claude")
            .join("settings.json");

        let mut results: Vec<(&'static str, bool)> = Vec::new();

        // 检查 settings.json 存在（简化：文件不存在 → false）
        let check1 = settings_path.exists();
        results.push(("ANTHROPIC_BASE_URL 配置", check1));

        // canary 本地引擎检查
        let check5 = run_check_canary_local_engine_via_test_hook();
        results.push(("canary 规则引擎命中 OUT-01", check5));

        let failures: Vec<&str> = results
            .iter()
            .filter_map(|(label, ok)| if *ok { None } else { Some(*label) })
            .collect();

        if failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "{} 项检查失败：{}",
                failures.len(),
                failures.join("、")
            ))
        }
    }
}

exec
/bin/zsh -lc 'git diff -- docs/specs/SPEC-003-sieve-setup-tool.md tasks/known-issues-v1.4.md' in /Users/doskey/src/sieve
 succeeded in 0ms:
diff --git a/docs/specs/SPEC-003-sieve-setup-tool.md b/docs/specs/SPEC-003-sieve-setup-tool.md
index c061b1b..e831959 100644
--- a/docs/specs/SPEC-003-sieve-setup-tool.md
+++ b/docs/specs/SPEC-003-sieve-setup-tool.md
@@ -340,7 +340,25 @@ Sieve 已卸载。
 
 ---
 
-## 7. 未决事项（TBD）
+## 7. 补充说明：Canary 检测实现变更（R4-#7 修复，2026-04-27）
+
+§4.2 "Canary 检测"原定向 `127.0.0.1:11453` 发 HTTP 请求，验证响应不含原始 token。
+该方案有致命缺陷：daemon 透传时拿到 401/502，响应同样不含 canary token，导致误判通过。
+
+**当前实现改为本地引擎 scan 方案**（已落地）：
+
+- 直接调用 `sieve_rules::engine::VectorscanEngine::compile(outbound_rules).scan(canary_token)`
+- canary token 精确匹配 OUT-01 pattern（`sk-ant-api03-[a-zA-Z0-9_\-]{93}AA`）
+- 不发任何网络请求，不依赖 daemon 是否在线
+- 输出标注：「canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）」
+- 规则文件路径优先读 `~/.sieve/rules/outbound.toml`，可通过 `SIEVE_RULES_PATH` env var 覆盖
+
+**限制说明**：本地 scan 验证了规则编译 + pattern 命中，但不验证 daemon 是否真的拦截了转发请求。
+端到端验证（daemon 实际改写 body）需要手动测试或后续引入 fake upstream。
+
+---
+
+## 8. 未决事项（TBD）
 
 | 编号 | 问题 | 选项 |
 |------|------|------|
diff --git a/tasks/known-issues-v1.4.md b/tasks/known-issues-v1.4.md
index 817c360..793ef16 100644
--- a/tasks/known-issues-v1.4.md
+++ b/tasks/known-issues-v1.4.md
@@ -44,23 +44,9 @@
 
 ---
 
-### P1-R3-#2：Hook pending 写入失败时 fail-open 🚨
+### ~~P1-R3-#2：Hook pending 写入失败时 fail-open~~（已修复）
 
-**位置**：`crates/sieve-cli/src/daemon.rs:877-879`
-
-**症状**：
-- IN-CR-02 / IN-CR-04 等 `hook_terminal` 命中后，daemon 调用 `write_hook_pending_silent()` 写 `~/.sieve/pending/<id>.json`
-- 写失败（磁盘满 / 权限错 / IO 异常）→ 当前代码**只 warn 后继续转发 SSE**
-- 静态注册的 `sieve-hook check` 启发式扫目录找不到 pending → fail-open exit 0
-- **危险工具调用没有任何拦截点**，违反 §9 fail-closed 硬约束
-
-**修法**：
-- pending 写失败时**走 fail-closed 路径**：注入 `sieve_blocked` SSE event + 关流
-- 或者 daemon 在写失败后改 disposition 为 GuiPopup 路径，强制 hold + GUI 处理
-
-**影响**：违反 PRD §9 第 3 条 + ADR-007 fail-closed 原则。攻击场景罕见但严重——攻击者诱发磁盘满即可绕过
-
-**等待依赖**：无；可独立修复，但需要审计日志同步告警
+**修复**：`write_hook_pending_silent` 改为 `write_hook_pending_or_fail_closed`（返回 `Result`）；写失败时注入 `sieve_blocked` SSE event 并截流。提取 `write_hook_pending_to(d, base)` 供单元测试注入路径，新增 2 个测试全部通过。
 
 ---
 
@@ -84,26 +70,25 @@
 
 ---
 
-### P1-R3-#6：sieve-hook 启发式扫遇坏 pending fail-open 🚨
+### ~~P1-R3-#6：sieve-hook 启发式扫遇坏 pending fail-open~~ **[Fixed]**
 
-**位置**：`crates/sieve-hook/src/pending.rs:83-90`
+**位置**：`crates/sieve-hook/src/pending.rs`（已修复）
 
-**症状**：
-- 启发式扫描 `~/.sieve/pending/` 时如果某个文件写到一半 / 损坏 / 读权限异常 → 当前代码直接 `continue` skip
+**症状**（已修复）：
+- 启发式扫描 `~/.sieve/pending/` 时如果某个文件写到一半 / 损坏 / 读权限异常 → 旧代码直接 `continue` skip
 - 如果**所有** fresh pending 都是损坏的 → fresh=[] → exit 0 fail-open
 - 应该被 HookTerminal 拦截的工具调用被放行
 
-**修法**：
-- 解析失败的 pending **不能 skip**，应当：
-  - 记 stale_paths（让后续逻辑处理）
-  - 或直接按 fail-closed 处理：发现解析失败立即 exit 1，让 Claude Code 拒绝
-- 启发式扫描的语义需要重新审视：
-  - 当前假设"扫不到 = 没有 Sieve 标记 = 让 Claude Code 通过"
-  - 但"扫到坏文件 = 不知道 Sieve 怎么判 = 应该保守 fail-closed"
+**修复方案**：
+- `ScanResult` 新增 `corrupt_paths: Vec<PathBuf>` 字段
+- IO 读取失败或 JSON 解析失败 → 加入 `corrupt_paths`（不再 skip）
+- `run_check_heuristic` 新决策表：
+  - `corrupt_paths` 非空 → 立即 fail-closed（exit 1），打 stderr 提示
+  - `fresh` 非空（corrupt=[]）→ 正常弹窗流程
+  - 全空 / 仅 stale → 原有行为不变
+- 新增 7 个单元/集成测试覆盖 corrupt 路径
 
-**影响**：违反 fail-closed；攻击者可以人工损坏 pending 绕过
-
-**等待依赖**：无；改起来很简单，但是要决定"扫到 0 个有效 pending 时" 的行为语义
+**影响**：违反 fail-closed 漏洞已消除
 
 ---
 
@@ -127,45 +112,28 @@
 
 ---
 
-### P2-R4-#7：doctor canary 检查太弱，会误报通过
+### ~~P2-R4-#7：doctor canary 检查太弱，会误报通过~~ ✅ Fixed
 
-**位置**：`crates/sieve-cli/src/commands/doctor.rs:191-192`
+**位置**：`crates/sieve-cli/src/commands/doctor.rs`
 
-**症状**：
-- doctor 只检查响应里**不含**原始 canary token
-- 如果 daemon 只是把请求透传到 Anthropic 后拿到 401/502，响应也不会包含 canary token → doctor 误判"脱敏正常"
-- 如果 canary token 本身没命中 OUT-01（拼写错 / 格式不匹配），同样会误判通过
-- 用户以为 setup 验证通过，实际拦截链路根本没工作
-
-**修法**：
-- 校验**真的命中**了本地拦截/脱敏路径——构造一个明确匹配 OUT-01 规则的 token + 验证 upstream 收到的是 redacted body
-- 或者用 fake upstream（local stub）拦请求 + 验证收到的 body 已经被改写
-- doctor 输出明确区分"未走代理"vs"走了代理但没拦截"
-
-**影响**：v1.5 §6.6 / SPEC-003 §doctor 的核心承诺失效，"装上即可用"无法验证
-
-**等待依赖**：无；可独立修
+**修复方案**：采用本地引擎直接 scan 方案（方案4）。
+- 废弃原 HTTP 请求验证（401/502 透传误判根本原因）
+- `check_canary_local_engine()`：直接调用 `VectorscanEngine::compile(outbound_rules).scan(canary_token)`
+- canary token 精确匹配 OUT-01 pattern（`sk-ant-api03-[a-zA-Z0-9_\-]{93}AA`）
+- 输出明确标注「仅验证规则引擎 + daemon listening；端到端验证需手动测」
+- 新增集成测试 `tests/doctor.rs::canary_token_hits_out01_in_local_engine`（T1）+ `canary_check_fails_when_rules_file_missing`（T2）
 
 ---
 
-### P2-R4-#8：doctor 失败时仍返回 Ok，CI 脚本无法捕获
-
-**位置**：`crates/sieve-cli/src/commands/doctor.rs:70-76`
-
-**症状**：
-- 任一检查项失败时 `doctor::run()` 仍返回 `Ok(())`
-- `sieve doctor` 在 CI / 脚本里以 exit 0 成功退出
-- `sieve setup` 中的 `doctor::run()?` 无法捕获安装失败
-- daemon 没启动 / 规则路径无效 / launchd 异常 → setup 显示成功但实际不可用
-
-**修法**：
-- 任一检查项失败时 `doctor::run()` 返回 `Err`（含失败项汇总）
-- main.rs 把 doctor 的 Err 映射为 exit code != 0
-- setup 自动调 doctor 时如果 doctor Err → 触发自动回滚
+### ~~P2-R4-#8：doctor 失败时仍返回 Ok，CI 脚本无法捕获~~ ✅ Fixed
 
-**影响**：违反 SPEC-003 doctor 设计承诺；CI 集成时假绿灯
+**位置**：`crates/sieve-cli/src/commands/doctor.rs` + `src/main.rs`
 
-**等待依赖**：无；改起来很简单
+**修复方案**：
+- `run()` 收集所有失败项到 `Vec<(&str, bool)>`，任一失败返回 `Err("N 项检查失败：...")`
+- `main.rs` `Command::Doctor` 分支：`if let Err(e) = run() { eprintln!(...); std::process::exit(1); }`
+- 新增集成测试 `tests/doctor.rs::doctor_run_returns_err_when_checks_fail`（T1）+ `sieve_doctor_exits_nonzero_when_checks_fail`（T2，子进程验证 exit code 非零）
+- setup 调用路径 `doctor::run()?` 已可正确捕获 Err（setup 回滚由 F-B1 子代理负责）
 
 ---
 

exec
/bin/zsh -lc 'ls -la && find . -maxdepth 3 -name Cargo.toml -print | sort' in /Users/doskey/src/sieve
 succeeded in 0ms:
total 304
drwxr-xr-x  25 doskey  staff    800 Apr 28 16:29 .
drwxr-xr-x@ 20 doskey  staff    640 Apr 27 11:22 ..
-rw-r--r--@  1 doskey  staff   6148 Apr 27 11:23 .DS_Store
drwxr-xr-x@  3 doskey  staff     96 Apr 27 13:12 .cargo
drwxr-xr-x@  4 doskey  staff    128 Apr 27 19:16 .claude
-rw-r--r--@  1 doskey  staff   8283 Apr 28 15:28 .cursorrules
drwxr-xr-x@ 16 doskey  staff    512 Apr 28 16:31 .git
-rw-r--r--@  1 doskey  staff   5473 Apr 27 12:59 .gitattributes
drwxr-xr-x@  6 doskey  staff    192 Apr 27 13:16 .github
-rw-r--r--@  1 doskey  staff   5876 Apr 27 12:58 .gitignore
-rw-r--r--@  1 doskey  staff  12499 Apr 28 15:28 CLAUDE.md
-rw-r--r--@  1 doskey  staff  64212 Apr 28 10:32 Cargo.lock
-rw-r--r--@  1 doskey  staff   1443 Apr 28 10:32 Cargo.toml
-rw-r--r--@  1 doskey  staff   3864 Apr 27 13:54 LICENSE
-rw-r--r--@  1 doskey  staff   8306 Apr 28 15:23 README.md
-rw-r--r--@  1 doskey  staff   5609 Apr 28 15:24 SECURITY.md
drwxr-xr-x@  7 doskey  staff    224 Apr 28 09:02 crates
-rw-r--r--@  1 doskey  staff    984 Apr 27 17:10 deny.toml
drwxr-xr-x  13 doskey  staff    416 Apr 28 15:27 docs
drwxr-xr-x@  6 doskey  staff    192 Apr 27 17:54 fuzz
drwxr-xr-x@  4 doskey  staff    128 Apr 27 17:10 fuzz_afl
-rw-r--r--@  1 doskey  staff    188 Apr 27 13:12 rust-toolchain.toml
drwxr-xr-x@  3 doskey  staff     96 Apr 27 16:59 scripts
drwxr-xr-x@  8 doskey  staff    256 Apr 27 19:04 target
drwxr-xr-x@  6 doskey  staff    192 Apr 28 19:54 tasks
./Cargo.toml
./crates/sieve-cli/Cargo.toml
./crates/sieve-core/Cargo.toml
./crates/sieve-hook/Cargo.toml
./crates/sieve-ipc/Cargo.toml
./crates/sieve-rules/Cargo.toml
./fuzz/Cargo.toml
./fuzz_afl/Cargo.toml

exec
/bin/zsh -lc "cat Cargo.toml && printf '\\n--- sieve-cli ---\\n' && cat crates/sieve-cli/Cargo.toml && printf '\\n--- sieve-hook ---\\n' && cat crates/sieve-hook/Cargo.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
[workspace]
resolver = "2"
members = [
    "crates/sieve-core",
    "crates/sieve-rules",
    "crates/sieve-cli",
    "crates/sieve-ipc",
    "crates/sieve-hook",
    "fuzz",
]
exclude = [
    "fuzz_afl",  # AFL crate 与 cargo fuzz 编译器要求不同，workspace 隔离
]

[workspace.package]
version = "0.1.0-alpha"
edition = "2021"
rust-version = "1.87"
license = "MIT"
repository = "https://github.com/<owner>/sieve"
authors = ["doskey <doskey.lee@gmail.com>"]

[workspace.dependencies]
# 用于跨 crate 共享版本(各 crate Cargo.toml 用 .workspace = true 引用)
tokio = { version = "1", default-features = false }
hyper = { version = "1", default-features = false }
hyper-util = { version = "0.1", default-features = false }
hyper-rustls = { version = "0.27", default-features = false }
rustls = { version = "0.23", default-features = false }
webpki-roots = "0.26"
http = "1"
http-body = "1"
http-body-util = "0.1"
bytes = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", default-features = false, features = ["serde", "clock"] }
fd-lock = "4"
clap = { version = "4", features = ["derive"] }

[profile.release]
codegen-units = 1
lto = "fat"
strip = "symbols"
panic = "abort"
debug = false
opt-level = 3

[profile.release-with-debug]
inherits = "release"
debug = true
strip = "none"

--- sieve-cli ---
[package]
name = "sieve-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve CLI daemon: hyper proxy server, config, audit log"
publish = false

[[bin]]
name = "sieve"
path = "src/main.rs"

[dependencies]
sieve-core = { path = "../sieve-core" }
sieve-rules = { path = "../sieve-rules" }
sieve-ipc = { path = "../sieve-ipc" }
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { workspace = true }

tokio = { workspace = true, features = ["full"] }
hyper = { workspace = true, features = ["http1", "http2", "server"] }
hyper-util = { workspace = true, features = ["tokio", "server-auto", "server-graceful", "service"] }
http = { workspace = true }
http-body = { workspace = true }
http-body-util = { workspace = true }
bytes = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }

clap = { version = "4", features = ["derive"] }
anyhow = "1"
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
uuid = { version = "1", features = ["v4"] }
serde_json = { workspace = true }
tokio-stream = { version = "0.1", features = ["sync"] }
futures-util = "0.3"

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }
hyper = { workspace = true, features = ["http1", "server", "client"] }
hyper-util = { workspace = true, features = ["tokio", "client-legacy", "http1", "service", "server"] }
http-body-util = { workspace = true }
http = { workspace = true }
bytes = { workspace = true }
sieve-core = { path = "../sieve-core" }
sieve-ipc = { path = "../sieve-ipc" }
anyhow = "1"
tempfile = "3"
serde_json = { workspace = true }
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }

--- sieve-hook ---
[package]
name = "sieve-hook"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Sieve PreToolUse hook: TTY confirmation for Critical detections (macOS, Phase 1)"
publish = false

# Phase 1 macOS only。
[target.'cfg(target_os = "macos")'.dependencies]

[[bin]]
name = "sieve-hook"
path = "src/main.rs"

[lib]
name = "sieve_hook_lib"
path = "src/lib.rs"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
fd-lock = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
clap = { workspace = true }

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"

[dev-dependencies.criterion]
version = "0.5"
features = ["html_reports"]

[[bench]]
name = "hook_startup"
harness = false

exec
/bin/zsh -lc "python3 - <<'PY'
s='sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA'
body=s[len('sk-ant-api03-'):-2]
print(len(s), len(body), len(s)-len('sk-ant-api03-')-2)
print(body)
PY" in /Users/doskey/src/sieve
 succeeded in 0ms:
108 93 93
canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

exec
/bin/zsh -lc "grep -n \"OUT-01\\|sk-ant\" -n crates/sieve-rules/rules/outbound.toml && sed -n '1,120p' crates/sieve-rules/rules/outbound.toml" in /Users/doskey/src/sieve
 succeeded in 0ms:
1:# OUT-01~12 出站 P0 规则集
10:# OUT-01: Anthropic API Key
11:# 格式: sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" 结尾
14:id = "OUT-01"
15:description = "Anthropic API key (sk-ant-api03-...)"
16:pattern = 'sk-ant-api03-[a-zA-Z0-9_\-]{93}AA'
20:keywords = ["sk-ant-api03"]
21:allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
# OUT-01~12 出站 P0 规则集
#
# 关联 PRD §5.1 出站检测目标。
# 来源: gitleaks/gitleaks config/gitleaks.toml (MIT License) + Sieve 自写
#
# 重要约束: 所有 pattern 必须为 vectorscan 兼容 PCRE 子集。
# 禁止使用: lookahead(?=...) / lookbehind(?<=...) / 反向引用\1 / 原子组(?>...) / \C \R \K

# ---------------------------------------------------------------------------
# OUT-01: Anthropic API Key
# 格式: sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" 结尾
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-01"
description = "Anthropic API key (sk-ant-api03-...)"
pattern = 'sk-ant-api03-[a-zA-Z0-9_\-]{93}AA'
severity = "critical"
action = "block"
entropy_min = 4.5
keywords = ["sk-ant-api03"]
allowlist_regexes = ['sk-ant-api03-[xX]{5,}']
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-02: OpenAI API Key
# 新格式: sk-proj-/sk-svcacct-/sk-admin- + 58~200 char + T3BlbkFJ + 58~200 char
# 旧格式: sk- + 20 alnum + T3BlbkFJ + 20 alnum
# 两种格式用 alternation 合并，无 lookahead
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-02"
description = "OpenAI API key (sk-... legacy + sk-proj-/sk-svcacct-/sk-admin-)"
pattern = 'sk-(?:proj|svcacct|admin)-[A-Za-z0-9_\-]{58,65}T3BlbkFJ[A-Za-z0-9_\-]{58,65}|sk-[a-zA-Z0-9]{20}T3BlbkFJ[a-zA-Z0-9]{20}'
severity = "critical"
action = "block"
entropy_min = 4.5
keywords = ["T3BlbkFJ"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-03: AWS Access Key ID
# 前缀: A3T[A-Z0-9] / AKIA / ASIA / ABIA / ACCA，后跟 16 个 base32 字符
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-03"
description = "AWS Access Key ID (AKIA / ASIA / ABIA / ACCA / A3T)"
pattern = '(?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16}'
severity = "critical"
action = "block"
entropy_min = 3.0
keywords = ["AKIA", "ASIA", "ABIA", "ACCA"]
allowlist_regexes = []
allowlist_stopwords = ["AKIAIOSFODNN7EXAMPLE"]  # AWS 官方文档示例 key
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-04: GitHub Personal Access Token
# 格式: ghp_/gho_/ghu_/ghs_/ghr_ + 36 个 alnum 字符
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-04"
description = "GitHub PAT (ghp_/gho_/ghu_/ghs_/ghr_)"
pattern = 'gh[pousr]_[0-9a-zA-Z]{36}'
severity = "critical"
action = "block"
entropy_min = 4.0
keywords = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-05: Google Cloud API Key
# 格式: AIza + 35 个 alnum/_/-
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-05"
description = "Google Cloud API Key (AIza...)"
pattern = 'AIza[0-9A-Za-z_\-]{35}'
severity = "high"
action = "block"
entropy_min = 4.0
keywords = ["AIza"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "auto_redact"
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-06: JWT Token
# 格式: eyJ... (header.payload.signature，三段均为 base64url)
# 不用 lookahead，直接匹配三段结构
# ---------------------------------------------------------------------------
[[rules]]
id = "OUT-06"
description = "JWT Token (eyJ...)"
pattern = 'ey[A-Za-z0-9_\-]{16,}\.ey[A-Za-z0-9_\/\-]{16,}\.[A-Za-z0-9_\/\-]{10,}'
severity = "high"
action = "block"
entropy_min = 3.5
keywords = ["eyJ"]
allowlist_regexes = []
allowlist_stopwords = []
disposition = "gui_popup"
timeout_seconds = 15
default_on_timeout = "redact"

# ---------------------------------------------------------------------------
# OUT-07: PEM Private Key Header
# 覆盖: RSA / EC / DSA / PKCS#8 / generic PRIVATE KEY 头部
# 注意: 不包含 OPENSSH（由 OUT-10 专项覆盖）
# ---------------------------------------------------------------------------
[[rules]]

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-rules/src/engine.rs && sed -n '1,220p' crates/sieve-rules/src/loader.rs" in /Users/doskey/src/sieve
 exited 1 in 0ms:
sed: crates/sieve-rules/src/engine.rs: No such file or directory

exec
/bin/zsh -lc 'find crates/sieve-rules/src -maxdepth 2 -type f -print' in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-rules/src/error.rs
crates/sieve-rules/src/wordlist.rs
crates/sieve-rules/src/critical_lock.rs
crates/sieve-rules/src/lib.rs
crates/sieve-rules/src/manifest.rs
crates/sieve-rules/src/placeholder.rs
crates/sieve-rules/src/bip39.rs
crates/sieve-rules/src/ed25519.rs
crates/sieve-rules/src/loader.rs
crates/sieve-rules/src/engine/mod.rs

exec
/bin/zsh -lc "sed -n '1,320p' crates/sieve-rules/src/engine/mod.rs && printf '\\n--- loader ---\\n' && sed -n '1,260p' crates/sieve-rules/src/loader.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! Vectorscan 多模式正则引擎（关联 ADR-001 / ADR-002 / PRD §6.4）。
//!
//! Phase 1 用 block mode（出站请求一次性扫描）；Week 3 起 stream mode 处理 SSE 流式。
//!
//! # 生命周期设计说明
//!
//! `vectorscan_rs::BlockScanner<'db>` 借用 `&'db BlockDatabase`，无法与 db 同存于同一 struct
//! 而不引入 unsafe self-referential pattern。鉴于 `lib.rs` 已有 `#![deny(unsafe_code)]`，
//! 本实现选择每次 `scan()` 调用时从 `BlockDatabase` 创建 `BlockScanner`（alloc scratch）。
//! scratch 分配代价远小于实际扫描代价，在 P99 < 20ms 目标下可接受。
//! Week 3 如需优化，可改为 `thread_local!` scratch 复用方案（仍无 unsafe）。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use crate::placeholder::is_placeholder;
use std::collections::HashMap;
use vectorscan_rs::{BlockDatabase, BlockScanner, Flag, Pattern, Scan};

/// 一次匹配的位置信息。
#[derive(Debug, Clone)]
pub struct MatchHit {
    /// 命中的规则 ID（如 OUT-01）。
    pub rule_id: String,
    /// 命中位置在输入字节流的起始偏移（闭区间，需 SOM_LEFTMOST flag）。
    pub start: usize,
    /// 命中位置的结束偏移（开区间）。
    pub end: usize,
}

/// 多模式匹配引擎 trait。
pub trait MatchEngine: Send + Sync {
    /// 对输入字节流执行多模式匹配，返回所有命中。
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>>;
}

/// Vectorscan 多模式正则引擎。
///
/// 编译后的 `BlockDatabase` 线程安全（`Send + Sync`）；扫描时按需创建 `BlockScanner`（含 scratch）。
pub struct VectorscanEngine {
    db: BlockDatabase,
    rules: HashMap<u32, RuleEntry>,
}

impl VectorscanEngine {
    /// 编译规则集为 vectorscan database。
    ///
    /// 每条规则的 `pattern` 编译为带 `SOM_LEFTMOST` flag（精确报告 start offset）。
    pub fn compile(rules: Vec<RuleEntry>) -> SieveRulesResult<Self> {
        let patterns: Vec<Pattern> = rules
            .iter()
            .enumerate()
            .map(|(i, r)| {
                Pattern::new(
                    r.pattern.as_bytes().to_vec(),
                    Flag::SOM_LEFTMOST,
                    Some(i as u32),
                )
            })
            .collect();

        let db = BlockDatabase::new(patterns)
            .map_err(|e| SieveRulesError::Engine(format!("compile vectorscan db: {e}")))?;

        let rules_map: HashMap<u32, RuleEntry> = rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| (i as u32, r))
            .collect();

        Ok(Self {
            db,
            rules: rules_map,
        })
    }

    /// 获取规则元信息（用于上层组装 Detection）。
    pub fn rule_meta(&self, pattern_id: u32) -> Option<&RuleEntry> {
        self.rules.get(&pattern_id)
    }

    /// 候选文本是否被 placeholder / per-rule allowlist 排除。
    pub fn is_excluded(&self, candidate: &str, rule: &RuleEntry) -> bool {
        // 全局 placeholder 黑名单
        if is_placeholder(candidate) {
            return true;
        }
        // per-rule allowlist regexes
        for r in &rule.allowlist_regexes {
            if let Ok(re) = regex::Regex::new(r) {
                if re.is_match(candidate) {
                    return true;
                }
            }
        }
        // per-rule allowlist stopwords
        for sw in &rule.allowlist_stopwords {
            if candidate.contains(sw.as_str()) {
                return true;
            }
        }
        false
    }
}

impl MatchEngine for VectorscanEngine {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        // 每次 scan 创建新 scanner（alloc scratch）。
        // 参见模块文档中关于生命周期设计的说明。
        let mut scanner = BlockScanner::new(&self.db)
            .map_err(|e| SieveRulesError::Engine(format!("create scanner: {e}")))?;

        // vectorscan 对带量词的 pattern（`{m,n}` / `(?:..)*` 等）会在每个合法 end
        // 位置都触发回调。例如 `\.env\b(?:\.[a-z]+)*` 在 `.env.example` 上会从
        // start=0 emit end=4,6,7,...,12 多次。下游 allowlist 只能看到 matched_text，
        // 短 match（仅 `.env`）拿不到完整文件名上下文，会绕过 `\.env\.example` 白名单。
        //
        // 此处按 (rule_id, start) 保留**最长** end，给上层 longest-match 语义。
        // 关联：IN-CR-03-DOTENV / IN-CR-03-SSH-DIR allowlist 正确性。
        let mut by_key: HashMap<(String, usize), MatchHit> = HashMap::new();
        scanner
            .scan(input, |id, from, to, _flags| {
                let rule_id = self
                    .rules
                    .get(&id)
                    .map(|r| r.id.clone())
                    .unwrap_or_default();
                let key = (rule_id.clone(), from as usize);
                by_key
                    .entry(key)
                    .and_modify(|existing| {
                        if (to as usize) > existing.end {
                            existing.end = to as usize;
                        }
                    })
                    .or_insert(MatchHit {
                        rule_id,
                        start: from as usize,
                        end: to as usize,
                    });
                Scan::Continue
            })
            .map_err(|e| SieveRulesError::Engine(format!("scan failed: {e}")))?;

        // 输出排序保证测试与下游处理的确定性
        let mut hits: Vec<MatchHit> = by_key.into_values().collect();
        hits.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| a.rule_id.cmp(&b.rule_id))
        });
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Action, Severity};

    fn rule(id: &str, pattern: &str, severity: Severity) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action: Action::Block,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: crate::manifest::DefaultOnTimeout::Block,
        }
    }

    #[test]
    fn compile_and_scan_simple() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"say hello world").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-TEST");
        assert_eq!(hits[0].start, 4);
        assert_eq!(hits[0].end, 9);
    }

    #[test]
    fn no_match_returns_empty() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"goodbye world").unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn multiple_patterns_match() {
        let rules = vec![
            rule("OUT-A", r"foo", Severity::High),
            rule("OUT-B", r"bar", Severity::Low),
        ];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"foobar").unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn is_excluded_placeholder() {
        let rules = vec![rule("OUT-01", r"sk-ant-api03", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("sk-ant-api03-XXXXXXXX", rule_entry));
        assert!(!engine.is_excluded("sk-ant-api03-real-mixed-content-xyz", rule_entry));
    }

    #[test]
    fn allowlist_stopword_excludes() {
        let mut r = rule("OUT-01", r"secret", Severity::High);
        r.allowlist_stopwords = vec!["example".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("my example secret", rule_entry));
        assert!(!engine.is_excluded("my real secret", rule_entry));
    }

    #[test]
    fn allowlist_regex_excludes() {
        let mut r = rule("OUT-01", r"private_key", Severity::High);
        r.allowlist_regexes = vec![r"(?i)test".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("test_private_key", rule_entry));
        assert!(!engine.is_excluded("prod_private_key", rule_entry));
    }

    /// vectorscan 对带量词的 pattern 会触发多个 endpoint 回调；引擎必须保留最长 end，
    /// 否则 allowlist 看不到完整 matched_text 会漏过短 match。关联 IN-CR-03-DOTENV。
    #[test]
    fn longest_match_per_start_dedup() {
        let rules = vec![rule("TEST-DOTENV", r"\.env\b(?:\.[a-z]+)*", Severity::High)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"read .env.example").unwrap();
        // 期望：仅 1 个 hit，匹配整段 `.env.example`（end=17），而非短 `.env`（end=9）
        let dotenv_hits: Vec<_> = hits.iter().filter(|h| h.rule_id == "TEST-DOTENV").collect();
        assert_eq!(
            dotenv_hits.len(),
            1,
            "expected single longest-match per start, got: {hits:?}"
        );
        assert_eq!(dotenv_hits[0].start, 5);
        assert_eq!(
            dotenv_hits[0].end, 17,
            "should keep longest end (.env.example = 12 chars), got {hits:?}"
        );
    }
}

--- loader ---
//! 从 toml 加载出站规则集。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct OutboundRulesFile {
    rules: Vec<RuleEntry>,
}

/// 从 toml 文件加载出站规则。
pub fn load_outbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| SieveRulesError::Manifest(format!("read {}: {e}", path.display())))?;
    let f: OutboundRulesFile = toml::from_str(&s)
        .map_err(|e| SieveRulesError::Manifest(format!("parse {}: {e}", path.display())))?;
    Ok(f.rules)
}

/// 加载入站规则集（toml schema 与出站一致）。
pub fn load_inbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    load_outbound_rules(path) // schema 同，直接复用
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_minimal_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-TEST"
description = "test rule"
pattern = "hello"
severity = "critical"
action = "block"
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "OUT-TEST");
    }

    #[test]
    fn loads_rule_with_optional_fields() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-02"
description = "with optional fields"
pattern = "secret"
severity = "high"
action = "warn"
entropy_min = 3.5
keywords = ["secret", "key"]
allowlist_regexes = ["(?i)example"]
allowlist_stopwords = ["test"]
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].entropy_min, Some(3.5));
        assert_eq!(rules[0].keywords.len(), 2);
        assert_eq!(rules[0].allowlist_regexes.len(), 1);
        assert_eq!(rules[0].allowlist_stopwords.len(), 1);
    }

    #[test]
    fn returns_error_on_missing_file() {
        let result = load_outbound_rules(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_on_invalid_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "not valid toml [[[").unwrap();
        let result = load_outbound_rules(f.path());
        assert!(result.is_err());
    }
}

exec
/bin/zsh -lc "grep -R \"SIEVE_RULES_PATH\\|outbound.toml\\|rules\" -n crates/sieve-cli/src crates/sieve-rules/src | head -80" in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-cli/src/config.rs:5://! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
crates/sieve-cli/src/config.rs:6://! Week 3 新增：`inbound_rules_path`（入站规则路径）。
crates/sieve-cli/src/config.rs:61:    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
crates/sieve-cli/src/config.rs:63:    pub rules_path: Option<PathBuf>,
crates/sieve-cli/src/config.rs:76:    /// 入站规则 toml 路径（Week 3，默认 `crates/sieve-rules/rules/inbound.toml`）。
crates/sieve-cli/src/config.rs:78:    pub inbound_rules_path: Option<PathBuf>,
crates/sieve-cli/src/config.rs:174:            rules_path: None,
crates/sieve-cli/src/config.rs:177:            inbound_rules_path: None,
crates/sieve-cli/src/config.rs:233:    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
crates/sieve-cli/src/config.rs:234:    pub fn resolved_rules_path(&self) -> PathBuf {
crates/sieve-cli/src/config.rs:235:        if let Some(p) = &self.rules_path {
crates/sieve-cli/src/config.rs:238:        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
crates/sieve-cli/src/config.rs:241:    /// 解析入站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/inbound.toml`（相对 cwd）。
crates/sieve-cli/src/config.rs:242:    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
crates/sieve-cli/src/config.rs:243:        if let Some(p) = &self.inbound_rules_path {
crates/sieve-cli/src/config.rs:246:        PathBuf::from("crates/sieve-rules/rules/inbound.toml")
crates/sieve-cli/src/config.rs:353:    fn parse_dry_run_and_rules_path() {
crates/sieve-cli/src/config.rs:358:            rules_path = "/x.toml"
crates/sieve-cli/src/config.rs:362:        assert_eq!(c.rules_path.as_ref().unwrap().to_str().unwrap(), "/x.toml");
crates/sieve-cli/src/config.rs:366:    fn resolved_rules_path_explicit() {
crates/sieve-cli/src/config.rs:368:            rules_path: Some(PathBuf::from("/custom/rules.toml")),
crates/sieve-cli/src/config.rs:371:        assert_eq!(c.resolved_rules_path(), PathBuf::from("/custom/rules.toml"));
crates/sieve-cli/src/config.rs:375:    fn resolved_rules_path_fallback() {
crates/sieve-cli/src/config.rs:377:        let p = c.resolved_rules_path();
crates/sieve-cli/src/config.rs:379:            p.ends_with("outbound.toml"),
crates/sieve-cli/src/config.rs:380:            "fallback should end with outbound.toml, got {:?}",
crates/sieve-cli/src/daemon.rs:270:                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
crates/sieve-cli/src/daemon.rs:399:                rules = %seg_result.redacted_summary,
crates/sieve-cli/src/daemon.rs:841:                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
crates/sieve-cli/src/main.rs:31:use sieve_rules::engine::VectorscanEngine;
crates/sieve-cli/src/main.rs:32:use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
crates/sieve-cli/src/main.rs:61:            let rules_path = cfg.resolved_rules_path();
crates/sieve-cli/src/main.rs:62:            tracing::info!(path = %rules_path.display(), "loading outbound rules");
crates/sieve-cli/src/main.rs:63:            let rules = load_outbound_rules(&rules_path).with_context(|| {
crates/sieve-cli/src/main.rs:65:                    "failed to load outbound rules from {}; \
crates/sieve-cli/src/main.rs:66:                     set rules_path in sieve.toml or ensure the default path exists",
crates/sieve-cli/src/main.rs:67:                    rules_path.display()
crates/sieve-cli/src/main.rs:70:            tracing::info!(count = rules.len(), "outbound rules loaded");
crates/sieve-cli/src/main.rs:73:            let engine = VectorscanEngine::compile(rules.clone())
crates/sieve-cli/src/main.rs:75:            let adapter = OutboundAdapter::new(Arc::new(engine), rules);
crates/sieve-cli/src/main.rs:93:            let inbound_rules_path = cfg.resolved_inbound_rules_path();
crates/sieve-cli/src/main.rs:94:            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
crates/sieve-cli/src/main.rs:95:            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
crates/sieve-cli/src/main.rs:97:                    "failed to load inbound rules from {}; \
crates/sieve-cli/src/main.rs:98:                         set inbound_rules_path in sieve.toml or ensure the default path exists",
crates/sieve-cli/src/main.rs:99:                    inbound_rules_path.display()
crates/sieve-cli/src/main.rs:104:            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
crates/sieve-cli/src/main.rs:109:                count = vectorscan_rules.len(),
crates/sieve-cli/src/main.rs:110:                placeholders = placeholder_rules.len(),
crates/sieve-cli/src/main.rs:111:                "inbound rules partitioned"
crates/sieve-cli/src/main.rs:115:            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
crates/sieve-cli/src/main.rs:119:                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);
crates/sieve-cli/src/main.rs:166:             NOT blocked. Fail-closed rules (IN-CR-01/02/05/IN-GEN-01/03/OUT-01~12) \
crates/sieve-cli/src/commands/doctor.rs:17://! 新实现改为**直接调用本地 sieve-rules 引擎**对 canary token 做 scan，
crates/sieve-cli/src/commands/doctor.rs:84:        // 直接调用本地 sieve-rules 引擎扫描 canary token，
crates/sieve-cli/src/commands/doctor.rs:177:    /// 直接调用 sieve-rules VectorscanEngine + 出站规则，验证至少 1 个 Detection 命中 OUT-01。
crates/sieve-cli/src/commands/doctor.rs:188:        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
crates/sieve-cli/src/commands/doctor.rs:189:        use sieve_rules::loader::load_outbound_rules;
crates/sieve-cli/src/commands/doctor.rs:191:        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
crates/sieve-cli/src/commands/doctor.rs:193:        let rules_candidates: Vec<std::path::PathBuf> = vec![
crates/sieve-cli/src/commands/doctor.rs:194:            // 生产：~/.sieve/rules/outbound.toml
crates/sieve-cli/src/commands/doctor.rs:197:                .join("rules")
crates/sieve-cli/src/commands/doctor.rs:198:                .join("outbound.toml"),
crates/sieve-cli/src/commands/doctor.rs:199:            // 开发：workspace 相对路径（通过 SIEVE_RULES_PATH 覆盖）
crates/sieve-cli/src/commands/doctor.rs:200:            std::path::PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
crates/sieve-cli/src/commands/doctor.rs:203:        let rules_path = rules_candidates
crates/sieve-cli/src/commands/doctor.rs:207:        let Some(rules_path) = rules_path else {
crates/sieve-cli/src/commands/doctor.rs:212:        let Ok(rules) = load_outbound_rules(&rules_path) else {
crates/sieve-cli/src/commands/doctor.rs:216:        let Ok(engine) = VectorscanEngine::compile(rules) else {
crates/sieve-cli/src/commands/setup.rs:508:        let rules_path = sieve_home.join("rules").join("outbound.toml");
crates/sieve-cli/src/commands/setup.rs:509:        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
crates/sieve-cli/src/commands/setup.rs:535:rules_path = "{rules_path}"
crates/sieve-cli/src/commands/setup.rs:538:inbound_rules_path = "{inbound_rules_path}"
crates/sieve-cli/src/commands/setup.rs:553:            rules_path = rules_path.display(),
crates/sieve-cli/src/commands/setup.rs:554:            inbound_rules_path = inbound_rules_path.display(),
crates/sieve-cli/src/commands/setup.rs:705:            content.contains(&format!("rules_path = \"{}", dir.path().display())),
crates/sieve-cli/src/commands/setup.rs:706:            "rules_path 必须是绝对路径: {content}"
crates/sieve-cli/src/commands/setup.rs:729:        assert!(cfg.rules_path.is_some(), "rules_path 应有绝对路径");
crates/sieve-cli/src/engine_adapter.rs:1://! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` /
crates/sieve-cli/src/engine_adapter.rs:4://! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，

exec
/bin/zsh -lc "sed -n '460,580p' crates/sieve-cli/src/commands/setup.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
                .unwrap_or_default()
                .join(sieve_toml_path)
        };

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.sieve.daemon</string>
  <key>ProgramArguments</key>
  <array>
    <string>{bin}</string>
    <string>start</string>
    <string>--config</string>
    <string>{config}</string>
  </array>
  <key>WorkingDirectory</key>
  <string>{work_dir}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{err}</string>
</dict>
</plist>
"#,
            bin = sieve_bin.display(),
            config = config_abs.display(),
            work_dir = sieve_home.display(),
            log = log_path.display(),
            err = err_path.display(),
        ))
    }

    /// 构建默认 sieve.toml 内容（所有路径使用绝对路径）。
    ///
    /// 生成的内容与 [`crate::config::Config`] 的扁平字段完全匹配（`deny_unknown_fields`），
    /// 可直接被 `toml::from_str::<Config>()` 反序列化而不报错。
    pub(super) fn build_default_sieve_toml(sieve_toml_path: &Path) -> Result<String> {
        let sieve_home = sieve_toml_path
            .parent()
            .ok_or_else(|| anyhow!("sieve.toml 路径无父目录"))?;
        let rules_path = sieve_home.join("rules").join("outbound.toml");
        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
        let audit_db = sieve_home.join("audit.db");
        let ipc_socket = sieve_home.join("ipc.sock");
        let pending_dir = sieve_home.join("pending");
        let decisions_dir = sieve_home.join("decisions");
        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| sieve_home.to_path_buf());
        let launchd_plist = home
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");

        Ok(format!(
            r#"# sieve.toml — 由 `sieve setup` 自动生成，所有路径为绝对路径
# 修改后需重启 daemon：launchctl kickstart -k gui/$(id -u)/com.sieve.daemon

upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
tls_verify_upstream = true
dry_run = false
preset = "default"
gui_socket_enabled = false

# 出站规则文件路径（绝对路径，launchd 从 / 启动时不依赖 cwd）
rules_path = "{rules_path}"

# 入站规则文件路径
inbound_rules_path = "{inbound_rules_path}"

# 审计日志数据库路径（绝对路径）
audit_db_path = "{audit_db}"

# IPC Unix socket 路径
ipc_socket_path = "{ipc_socket}"

# 待决策 / 已决策文件目录
pending_dir = "{pending_dir}"
decisions_dir = "{decisions_dir}"

# launchd plist 路径（macOS）
launchd_plist_path = "{launchd_plist}"
"#,
            rules_path = rules_path.display(),
            inbound_rules_path = inbound_rules_path.display(),
            audit_db = audit_db.display(),
            ipc_socket = ipc_socket.display(),
            pending_dir = pending_dir.display(),
            decisions_dir = decisions_dir.display(),
            launchd_plist = launchd_plist.display(),
        ))
    }

    /// 简单去除 `// ...` 行注释（不处理字符串内的 `//`，够用于 settings.json）。
    pub(super) fn strip_json_comments(s: &str) -> String {
        s.lines()
            .map(|line| {
                // 找到不在引号内的 `//`
                let mut in_string = false;
                let mut escaped = false;
                let mut comment_start = None;
                let chars: Vec<char> = line.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if escaped {
                        escaped = false;
                    } else if chars[i] == '\\' && in_string {
                        escaped = true;
                    } else if chars[i] == '"' {
                        in_string = !in_string;
                    } else if !in_string

exec
/bin/zsh -lc "grep -R \"copy.*outbound\\|rules/outbound\\|outbound.toml\" -n crates/sieve-cli/src/commands/setup.rs crates/sieve-cli/src | head -100" in /Users/doskey/src/sieve
 succeeded in 0ms:
crates/sieve-cli/src/commands/setup.rs:508:        let rules_path = sieve_home.join("rules").join("outbound.toml");
crates/sieve-cli/src/config.rs:61:    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
crates/sieve-cli/src/config.rs:233:    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
crates/sieve-cli/src/config.rs:238:        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
crates/sieve-cli/src/config.rs:379:            p.ends_with("outbound.toml"),
crates/sieve-cli/src/config.rs:380:            "fallback should end with outbound.toml, got {:?}",
crates/sieve-cli/src/commands/doctor.rs:191:        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
crates/sieve-cli/src/commands/doctor.rs:194:            // 生产：~/.sieve/rules/outbound.toml
crates/sieve-cli/src/commands/doctor.rs:198:                .join("outbound.toml"),
crates/sieve-cli/src/commands/setup.rs:508:        let rules_path = sieve_home.join("rules").join("outbound.toml");

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-cli/src/commands/setup.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup）。
//!
//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
//!
//! 步骤：
//! 1. 检测 `~/.claude/settings.json` 是否存在
//! 2. 计算 diff（ANTHROPIC_BASE_URL + PreToolUse hook + launchd plist）
//! 3. dry-run 打印 diff，非 --yes 等待用户确认
//! 4. 备份原文件到 `~/.sieve/backups/<RFC3339>/`
//! 5. 写 `~/.sieve/sieve.toml`（默认配置，绝对路径）
//! 6. 修改 settings.json（解析失败则 abort，不写任何内容）
//! 7. 写 launchd plist（命令包含 `--config <abs_path>/sieve.toml`）+ `launchctl load -w`
//! 8. 写 setup.log（JSON Lines，含 created_new 字段）
//! 9. 自动调用 doctor 验证
//!
//! 错误恢复：任意步骤失败 → 反向回滚已做改动。

use crate::cli::SetupArgs;
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::commands::doctor;
    use anyhow::{anyhow, bail, Context};
    use chrono::Utc;
    use serde_json::Value;
    use std::fs;
    use std::io::{self, Write as IoWrite};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// setup.log 每行的结构（JSON Lines）。
    ///
    /// `created_new`：true 表示 setup 前该文件不存在，由 setup 新建；
    /// uninstall 时 `created_new=true` 的文件直接删除，`false` 的从备份恢复。
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SetupLogEntry {
        pub timestamp: String,
        pub action: String,
        pub path: Option<String>,
        pub detail: Option<String>,
        /// setup 前该文件是否不存在（新建 vs 覆盖）。
        #[serde(default)]
        pub created_new: bool,
    }

    impl SetupLogEntry {
        pub(super) fn new(action: impl Into<String>) -> Self {
            Self {
                timestamp: Utc::now().to_rfc3339(),
                action: action.into(),
                path: None,
                detail: None,
                created_new: false,
            }
        }

        pub(super) fn with_path(mut self, path: impl Into<String>) -> Self {
            self.path = Some(path.into());
            self
        }

        pub(super) fn with_detail(mut self, detail: impl Into<String>) -> Self {
            self.detail = Some(detail.into());
            self
        }

        pub(super) fn with_created_new(mut self, created_new: bool) -> Self {
            self.created_new = created_new;
            self
        }
    }

    /// setup 执行上下文，用于错误时反向回滚。
    struct SetupContext {
        backup_dir: PathBuf,
        /// 已写入的文件路径，错误时按逆序恢复。
        written_files: Vec<PathBuf>,
        /// 已执行的 launchctl load，错误时需要 unload。
        launchd_loaded: Option<PathBuf>,
    }

    impl SetupContext {
        fn new(backup_dir: PathBuf) -> Self {
            Self {
                backup_dir,
                written_files: Vec::new(),
                launchd_loaded: None,
            }
        }

        /// 回滚所有已做改动（从备份目录恢复）。
        fn rollback(&self) {
            eprintln!("[sieve setup] 回滚已做改动…");

            if let Some(plist) = &self.launchd_loaded {
                let _ = Command::new("launchctl")
                    .args(["unload", &plist.to_string_lossy()])
                    .status();
                eprintln!("  ↩ launchctl unload {}", plist.display());
            }

            for path in self.written_files.iter().rev() {
                // 计算备份中的相对路径：去掉 HOME 前缀
                let home = std::env::var("HOME").unwrap_or_default();
                let rel = path.strip_prefix(&home).unwrap_or(path.as_path());
                let backup_src = self.backup_dir.join(rel);
                if backup_src.exists() {
                    if let Err(e) = fs::copy(&backup_src, path) {
                        eprintln!("  ✗ 恢复 {} 失败: {}", path.display(), e);
                    } else {
                        eprintln!("  ↩ 恢复 {}", path.display());
                    }
                } else {
                    // 备份不存在说明是新建的，直接删除
                    let _ = fs::remove_file(path);
                    eprintln!("  ↩ 删除新建文件 {}", path.display());
                }
            }
        }
    }

    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
    pub fn run(args: SetupArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);

        let settings_path = home_path.join(".claude").join("settings.json");
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
        let backup_dir = sieve_home.join("backups").join(&backup_ts);
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let setup_log_path = sieve_home.join("setup.log");

        // ── 1. 读取现有 settings.json（允许不存在；解析失败则 abort，不覆盖用户文件）
        let settings_existed_before = settings_path.exists();
        let existing_settings: Value = if settings_existed_before {
            let raw =
                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
            // Strip JSON 注释（简单处理：删除 // 行注释）
            let stripped = strip_json_comments(&raw);
            serde_json::from_str(&stripped).map_err(|e| {
                anyhow!(
                    "无法解析 ~/.claude/settings.json：{}。\n\
                     请用 JSON 校验工具修复后重试。setup 已 abort，未做任何改动。",
                    e
                )
            })?
        } else {
            serde_json::json!({})
        };
        // sieve.toml 将写入 ~/.sieve/sieve.toml（绝对路径）
        let sieve_toml_path = sieve_home.join("sieve.toml");

        // ── 2. 计算 diff
        let sieve_url = "http://127.0.0.1:11453";
        let hook_entry = serde_json::json!({
            "matcher": ".*",
            "hooks": [{"type": "command", "command": "sieve-hook check"}]
        });

        let current_base_url = existing_settings
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|v| v.as_str())
            .unwrap_or("<未设置>");
        let has_hook = existing_settings
            .pointer("/hooks/PreToolUse")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        let plist_content = build_plist_content(&sieve_toml_path)?;

        // ── 3. 打印 diff
        println!("=== sieve setup diff ===");
        println!(
            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
            current_base_url, sieve_url
        );
        if has_hook {
            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
        } else {
            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
        }
        if sieve_toml_path.exists() {
            println!(
                "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
                sieve_toml_path.display()
            );
        } else {
            println!("[sieve.toml] 新建 {}", sieve_toml_path.display());
        }
        println!(
            "[launchd] 写入 {} (含 --config {})",
            plist_path.display(),
            sieve_toml_path.display()
        );
        println!("[launchd] 执行 launchctl load -w");
        println!("========================");

        // ── 4. dry-run 直接返回

exec
/bin/zsh -lc "sed -n '220,460p' crates/sieve-cli/src/commands/setup.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
        // ── 4. dry-run 直接返回
        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 5. 等待用户确认
        if !args.yes {
            print!("继续执行以上操作？[y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("已取消。");
                return Ok(());
            }
        }

        // ── 6. 备份
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
        let mut ctx = SetupContext::new(backup_dir.clone());

        let result = do_setup(
            &mut ctx,
            &home_path,
            &settings_path,
            &plist_path,
            &sieve_toml_path,
            &setup_log_path,
            &backup_dir,
            existing_settings,
            settings_existed_before,
            sieve_url,
            hook_entry,
            plist_content,
        );

        if let Err(ref e) = result {
            eprintln!("[sieve setup] 失败: {e}");
            ctx.rollback();
            return result;
        }

        // ── 9. 自动跑 doctor 验证
        println!("\n[sieve setup] 正在验证安装…");
        doctor::run()?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn do_setup(
        ctx: &mut SetupContext,
        home_path: &Path,
        settings_path: &Path,
        plist_path: &Path,
        sieve_toml_path: &Path,
        setup_log_path: &Path,
        backup_dir: &Path,
        mut existing_settings: Value,
        settings_existed_before: bool,
        sieve_url: &str,
        hook_entry: Value,
        plist_content: String,
    ) -> Result<()> {
        // 备份 settings.json（仅在文件已存在时）
        if settings_existed_before {
            let rel = settings_path
                .strip_prefix(home_path)
                .unwrap_or(settings_path);
            let backup_dest = backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
        }

        // 修改 settings.json
        {
            let env = existing_settings
                .get_mut("env")
                .and_then(|v| v.as_object_mut())
                .map(|obj| {
                    obj.insert(
                        "ANTHROPIC_BASE_URL".to_string(),
                        serde_json::json!(sieve_url),
                    );
                })
                .is_some();
            if !env {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "env".to_string(),
                    serde_json::json!({"ANTHROPIC_BASE_URL": sieve_url}),
                );
            }

            // 追加 PreToolUse hook（幂等：已存在则跳过）
            let hooks_obj = existing_settings
                .get_mut("hooks")
                .and_then(|v| v.as_object_mut());
            if let Some(hooks) = hooks_obj {
                let pre_tool = hooks
                    .entry("PreToolUse")
                    .or_insert_with(|| serde_json::json!([]));
                if let Some(arr) = pre_tool.as_array_mut() {
                    let already = arr.iter().any(|item| {
                        item.pointer("/hooks/0/command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.contains("sieve-hook"))
                            .unwrap_or(false)
                    });
                    if !already {
                        arr.push(hook_entry);
                    }
                }
            } else {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "hooks".to_string(),
                    serde_json::json!({"PreToolUse": [hook_entry]}),
                );
            }

            // 确保父目录存在
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json_str = serde_json::to_string_pretty(&existing_settings)?;
            fs::write(settings_path, json_str.as_bytes()).context("写入 settings.json 失败")?;
            ctx.written_files.push(settings_path.to_path_buf());
            println!("[setup] ✅ settings.json 已更新");
        }

        // 写 ~/.sieve/sieve.toml（绝对路径配置，供 launchd plist 引用）
        let sieve_toml_existed_before = sieve_toml_path.exists();
        {
            if sieve_toml_existed_before {
                // 备份已有 sieve.toml
                let rel = sieve_toml_path
                    .strip_prefix(home_path)
                    .unwrap_or(sieve_toml_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
            }
            if let Some(parent) = sieve_toml_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_content = build_default_sieve_toml(sieve_toml_path)?;
            fs::write(sieve_toml_path, toml_content.as_bytes()).context("写入 sieve.toml 失败")?;
            ctx.written_files.push(sieve_toml_path.to_path_buf());
            println!("[setup] ✅ sieve.toml 写入 {}", sieve_toml_path.display());
        }

        // 写 launchd plist
        {
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 备份已有 plist
            if plist_path.exists() {
                let rel = plist_path.strip_prefix(home_path).unwrap_or(plist_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
            }
            fs::write(plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
            ctx.written_files.push(plist_path.to_path_buf());
            println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());
        }

        // launchctl load
        {
            let status = Command::new("launchctl")
                .args(["load", "-w", &plist_path.to_string_lossy()])
                .status()
                .context("执行 launchctl load 失败")?;
            if !status.success() {
                bail!("launchctl load 返回非零: {:?}", status.code());
            }
            ctx.launchd_loaded = Some(plist_path.to_path_buf());
            println!("[setup] ✅ launchd 服务已加载");
        }

        // 写 setup.log（含 created_new 字段，供 uninstall 精确还原）
        {
            let entries: Vec<SetupLogEntry> = vec![
                SetupLogEntry::new("setup_complete")
                    .with_detail(format!("backup_dir={}", backup_dir.display())),
                SetupLogEntry::new("settings_updated")
                    .with_path(settings_path.to_string_lossy().to_string())
                    .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse")
                    .with_created_new(!settings_existed_before),
                SetupLogEntry::new("sieve_toml_written")
                    .with_path(sieve_toml_path.to_string_lossy().to_string())
                    .with_created_new(!sieve_toml_existed_before),
                SetupLogEntry::new("launchd_loaded")
                    .with_path(plist_path.to_string_lossy().to_string()),
            ];
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(setup_log_path)
                .context("打开 setup.log 失败")?;
            for entry in &entries {
                let line = serde_json::to_string(entry)? + "\n";
                file.write_all(line.as_bytes())?;
            }
            println!("[setup] ✅ setup.log 写入 {}", setup_log_path.display());
        }

        Ok(())
    }

    /// 构建 launchd plist 内容（使用当前 sieve 二进制路径 + 绝对路径 --config）。
    ///
    /// plist 中 ProgramArguments 必须使用绝对路径，且 --config 指向绝对配置文件，
    /// 否则 launchd 从根目录启动时找不到相对路径规则文件，daemon 会立即退出。
    /// WorkingDirectory 兜底设置为 sieve_home（~/.sieve）。
    pub(super) fn build_plist_content(sieve_toml_path: &Path) -> Result<String> {
        let sieve_bin = std::env::current_exe().context("获取当前二进制路径失败")?;
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let log_path = sieve_home.join("daemon.log");
        let err_path = sieve_home.join("daemon.err");
        // config 路径必须是绝对路径
        let config_abs = if sieve_toml_path.is_absolute() {
            sieve_toml_path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_default()

exec
/bin/zsh -lc 'cargo test --workspace --locked' in /Users/doskey/src/sieve
 exited 101 in 1682ms:

running 35 tests
test commands::setup::tests::bad_json_parse_returns_error_not_empty_object ... ok
test commands::setup::tests::setup_log_entry_created_new_serializes_correctly ... ok
test commands::setup::tests::plist_contains_absolute_config_flag ... ok
test config::tests::audit_db_path_falls_back_to_default ... ok
test config::tests::audit_db_path_explicit_field_wins ... ok
test commands::setup::tests::default_sieve_toml_has_absolute_paths ... ok
test config::tests::audit_db_path_falls_back_to_log_path ... ok
test config::tests::defaults_are_sane ... ok
test commands::setup::tests::default_sieve_toml_parses_as_config ... ok
test config::tests::listen_addr_parses ... ok
test config::tests::parse_full_toml ... ok
test config::tests::parse_dry_run_and_rules_path ... ok
test config::tests::parse_minimal_toml ... ok
test config::tests::resolved_rules_path_explicit ... ok
test config::tests::resolved_rules_path_fallback ... ok
test daemon::tests::hook_pending_write_fails_on_unwritable_base ... ok
test config::tests::unknown_field_rejected ... ok
test config::tests::resolved_sieveignore_path_explicit ... ok
test commands::uninstall::tests::uninstall_toml_created_new_true_deletes_file ... ok
test engine_adapter::tests::map_action_warn_becomes_hook_mark ... ok
test commands::uninstall::tests::uninstall_created_new_true_deletes_file ... ok
test engine_adapter::tests::redact_evidence_short ... ok
test engine_adapter::tests::redact_evidence_long ... ok
test commands::uninstall::tests::uninstall_created_new_false_removes_sieve_entries_only ... ok
test commands::uninstall::tests::uninstall_toml_created_new_false_restores_from_backup ... ok
test daemon::tests::hook_pending_write_happy_path ... ok
test engine_adapter::tests::scan_no_match_returns_empty ... ok
test audit::tests::decision_event_stores_decision_field ... ok
test audit::tests::update_trigger_blocks ... ok
test audit::tests::write_and_read_events ... ok
test engine_adapter::tests::disposition_hook_terminal_beats_enforce_action ... ok
test engine_adapter::tests::span_offset_applied ... ok
test engine_adapter::tests::disposition_auto_redact_beats_enforce_action ... ok
test engine_adapter::tests::scan_detects_pattern ... ok
test engine_adapter::tests::disposition_gui_popup_beats_enforce_action ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audit_append_only.rs (target/debug/deps/audit_append_only-7963e579460992e0)

running 3 tests
test update_is_rejected_by_trigger ... ok
test delete_is_rejected_by_trigger ... ok
test write_3_events_and_read_back ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/doctor.rs (target/debug/deps/doctor-e98e76573dad036c)

running 4 tests
test canary_check_fails_when_rules_file_missing ... ok
test doctor_run_returns_err_when_checks_fail ... ok
test canary_token_hits_out01_in_local_engine ... ok
  ❌ settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453
  ❌ settings.json: hooks.PreToolUse 含 sieve-hook check
  ❌ daemon 在 127.0.0.1:11453 监听
  ❌ launchd com.sieve.daemon 已加载
  ❌ canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）

❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。
sieve doctor: 5 项检查失败：ANTHROPIC_BASE_URL 配置、PreToolUse hook 配置、daemon 监听 :11453、launchd 服务已加载、canary 规则引擎命中 OUT-01
test sieve_doctor_exits_nonzero_when_checks_fail ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.49s

     Running tests/inbound_block.rs (target/debug/deps/inbound_block-ebd9dd42f9aab019)

running 10 tests
test address_substitution_from_prompt_seed_blocks ... FAILED
test benign_response_passes_through_unchanged ... FAILED
test ucsb_attack_4_markdown_exfil_failclosed_without_gui ... FAILED
test ucsb_attack_2_dangerous_shell_hookmark_passthrough ... FAILED
test malformed_tool_use_partial_json_blocks ... FAILED
test in_cr_04_persistence_shell_rc_hookmark_passthrough ... FAILED
test ucsb_attack_3_signing_tool_blocked ... FAILED
test ucsb_attack_1_address_substitution_blocked ... FAILED
test in_cr_03_sensitive_path_warn_passes_through ... FAILED
test unterminated_final_event_still_blocks_critical ... FAILED

failures:

---- address_substitution_from_prompt_seed_blocks stdout ----

thread 'address_substitution_from_prompt_seed_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- benign_response_passes_through_unchanged stdout ----

thread 'benign_response_passes_through_unchanged' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_4_markdown_exfil_failclosed_without_gui stdout ----

thread 'ucsb_attack_4_markdown_exfil_failclosed_without_gui' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_2_dangerous_shell_hookmark_passthrough stdout ----

thread 'ucsb_attack_2_dangerous_shell_hookmark_passthrough' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- malformed_tool_use_partial_json_blocks stdout ----

thread 'malformed_tool_use_partial_json_blocks' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_04_persistence_shell_rc_hookmark_passthrough stdout ----

thread 'in_cr_04_persistence_shell_rc_hookmark_passthrough' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_3_signing_tool_blocked stdout ----

thread 'ucsb_attack_3_signing_tool_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- ucsb_attack_1_address_substitution_blocked stdout ----

thread 'ucsb_attack_1_address_substitution_blocked' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- in_cr_03_sensitive_path_warn_passes_through stdout ----

thread 'in_cr_03_sensitive_path_warn_passes_through' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }

---- unterminated_final_event_still_blocks_critical stdout ----

thread 'unterminated_final_event_still_blocks_critical' panicked at crates/sieve-cli/tests/inbound_block.rs:93:59:
called `Result::unwrap()` on an `Err` value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    address_substitution_from_prompt_seed_blocks
    benign_response_passes_through_unchanged
    in_cr_03_sensitive_path_warn_passes_through
    in_cr_04_persistence_shell_rc_hookmark_passthrough
    malformed_tool_use_partial_json_blocks
    ucsb_attack_1_address_substitution_blocked
    ucsb_attack_2_dangerous_shell_hookmark_passthrough
    ucsb_attack_3_signing_tool_blocked
    ucsb_attack_4_markdown_exfil_failclosed_without_gui
    unterminated_final_event_still_blocks_critical

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p sieve-cli --test inbound_block`

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/tests/inbound_block.rs | sed -n '70,110p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
    70	    Bytes::from(s)
    71	}
    72	
    73	/// mock 上游 StreamBody 类型（size_hint unknown → hyper 用 chunked transfer，不加 content-length）。
    74	type MockBody = StreamBody<tokio_stream::Once<Result<Frame<Bytes>, Infallible>>>;
    75	
    76	/// 把 Bytes 包成 StreamBody（无 exact size_hint）。
    77	///
    78	/// hyper 对 `Full<Bytes>` 会自动加 content-length；StreamBody unknown size 时用 chunked。
    79	/// sieve 透传 content-length 到客户端，注入 sieve_blocked 后实际 body 超出长度，HTTP 协议错误。
    80	fn bytes_to_chunked_body(data: Bytes) -> MockBody {
    81	    let stream = tokio_stream::once(Ok::<_, Infallible>(Frame::data(data)));
    82	    StreamBody::new(stream)
    83	}
    84	
    85	/// 在 :0 端口启动 plain-HTTP mock 上游（chunked transfer），返回 (addr, shutdown sender)。
    86	///
    87	/// responder 返回 (status, body_bytes)；Content-Type 固定为 `text/event-stream`。
    88	async fn spawn_mock_sse_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
    89	where
    90	    F: Fn(Request<Bytes>) -> Fut + Clone + Send + Sync + 'static,
    91	    Fut: std::future::Future<Output = (hyper::StatusCode, Bytes)> + Send,
    92	{
    93	    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    94	    let addr = listener.local_addr().unwrap();
    95	    let (tx, mut rx) = oneshot::channel::<()>();
    96	
    97	    tokio::spawn(async move {
    98	        loop {
    99	            tokio::select! {
   100	                _ = &mut rx => break,
   101	                accept = listener.accept() => {
   102	                    let Ok((stream, _)) = accept else { continue };
   103	                    let io = TokioIo::new(stream);
   104	                    let r = responder.clone();
   105	                    tokio::spawn(async move {
   106	                        let svc = service_fn(move |req: Request<Incoming>| {
   107	                            let r = r.clone();
   108	                            async move {
   109	                                let (parts, body) = req.into_parts();
   110	                                let bytes = body

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '85,110p' && nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '240,260p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
    85	        // 确认 OUT-01 规则确实命中。不发真实网络请求，不依赖 daemon 是否在线。
    86	        // 输出明确说明「仅验证规则引擎 + daemon listening；端到端验证需手动测」。
    87	        let check5 = check_canary_local_engine();
    88	        print_check(
    89	            "canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）",
    90	            check5,
    91	        );
    92	        results.push(("canary 规则引擎命中 OUT-01", check5));
    93	
    94	        // ── 汇总（R4-#8 修复）
    95	        println!();
    96	        let failures: Vec<&str> = results
    97	            .iter()
    98	            .filter_map(|(label, ok)| if *ok { None } else { Some(*label) })
    99	            .collect();
   100	
   101	        if failures.is_empty() {
   102	            println!("✅ 所有检查通过，Sieve 运行正常。");
   103	            Ok(())
   104	        } else {
   105	            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
   106	            Err(anyhow::anyhow!(
   107	                "{} 项检查失败：{}",
   108	                failures.len(),
   109	                failures.join("、")
   110	            ))
   240	            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
   241	        let mut ctx = SetupContext::new(backup_dir.clone());
   242	
   243	        let result = do_setup(
   244	            &mut ctx,
   245	            &home_path,
   246	            &settings_path,
   247	            &plist_path,
   248	            &sieve_toml_path,
   249	            &setup_log_path,
   250	            &backup_dir,
   251	            existing_settings,
   252	            settings_existed_before,
   253	            sieve_url,
   254	            hook_entry,
   255	            plist_content,
   256	        );
   257	
   258	        if let Err(ref e) = result {
   259	            eprintln!("[sieve setup] 失败: {e}");
   260	            ctx.rollback();

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/setup.rs | sed -n '258,272p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   258	        if let Err(ref e) = result {
   259	            eprintln!("[sieve setup] 失败: {e}");
   260	            ctx.rollback();
   261	            return result;
   262	        }
   263	
   264	        // ── 9. 自动跑 doctor 验证
   265	        println!("\n[sieve setup] 正在验证安装…");
   266	        doctor::run()?;
   267	
   268	        Ok(())
   269	    }
   270	
   271	    #[allow(clippy::too_many_arguments)]
   272	    fn do_setup(

exec
/bin/zsh -lc "grep -n \"fn build_sieve_blocked_sse\\|build_sieve_blocked_sse\" -n crates/sieve-cli/src/daemon.rs && sed -n '520,760p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
554:                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
580:                        let blocked_payload = build_sieve_blocked_sse(&blocking);
593:                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
687:                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
695:                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
707:                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
746:                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
757:            let blocked_payload = build_sieve_blocked_sse(&blocking);
774:            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
927:fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    tokio::spawn(async move {
        let mut parser = SseParser::new();
        let mut aggregator = Aggregator::new();

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        while let Some(frame_result) = stream.next().await {
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
                    let events = match parser.push_chunk(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // 收集本批 events 的 detections，按 action 分组处理
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                    );

                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
                    // 关联：ADR-014 §双层防御、PRD §9 #3。

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，失败时 fail-closed（不允许 fail-open）
                    for d in &hook_detections {
                        if let Err(e) = write_hook_pending_or_fail_closed(d) {
                            tracing::error!(
                                error = %e,
                                rule = %d.rule_id,
                                "Hook pending write failed; fail-closed: truncating SSE stream"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            // keep-alive channel：daemon 把心跳写入 SSE 流
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。

                            // 启动 keep-alive 转发 task
                            let ka_fwd_handle = tokio::spawn(async move {
                                while let Some(ka_bytes) = ka_rx.recv().await {
                                    if tx_ka
                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });

                            // 构造 IPC 请求
                            use chrono::Utc;
                            let request_id = uuid::Uuid::new_v4();
                            let timeout_seconds = hold_detections
                                .iter()
                                .find_map(|d| {
                                    if let Action::HoldForDecision {
                                        timeout_seconds, ..
                                    } = d.action
                                    {
                                        Some(timeout_seconds)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(60);

                            let ipc_detections = hold_detections
                                .iter()
                                .map(|d| sieve_ipc::protocol::DetectionPayload {
                                    rule_id: d.rule_id.clone(),
                                    severity: map_severity_to_ipc(d.severity),
                                    disposition: sieve_ipc::Disposition::GuiPopup,
                                    title: format!("检测命中：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                })
                                .collect();

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
                                    // 然后继续转发后续 SSE。
                                    if tx
                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    // 修 R2-#3：用户拒绝时不发触发帧，直接注入 sieve_blocked 并关流。
                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            // IPC 未初始化：fail-closed，阻断
                            tracing::warn!(
                                "GuiPopup detection but IPC server not initialized; fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 无 blocking / hold：透传原始 frame
                    if tx
                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "upstream body error: {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
        let flushed = parser.flush();
        let (blocking, hook_detections, flush_hold_detections) =
            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);

        // flush 阶段 Hook 类同样 fail-closed：写失败即截流
        for d in &hook_detections {
            if let Err(e) = write_hook_pending_or_fail_closed(d) {
                tracing::error!(
                    error = %e,
                    rule = %d.rule_id,
                    "Hook pending write failed (flush); fail-closed: truncating SSE stream"
                );
                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                return;
            }
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
            }
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

exec
/bin/zsh -lc "sed -n '900,980p' crates/sieve-cli/src/daemon.rs && sed -n '805,862p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
            one_line_summary: d.evidence_truncated.clone(),
            details: serde_json::json!({}),
        }],
    };

    sieve_ipc::pending_file::write_pending(&ipc_req, sieve_home)?;

    tracing::info!(
        rule = %d.rule_id,
        request_id = %request_id,
        "HookMark: pending file written, SSE stream continues"
    );

    Ok(())
}

/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
    match s {
        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
        sieve_core::Severity::High => sieve_ipc::Severity::High,
        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
    }
}

/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
    let payload = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "detections": detections.iter().map(|d| serde_json::json!({
            "rule_id": d.rule_id,
            "severity": d.severity,
            "fingerprint": d.fingerprint,
        })).collect::<Vec<_>>(),
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
}

/// 用已收集的 body bytes 重新构造请求并转发。
async fn forward_raw(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();

    for evt in events {
        match inbound_filter.observe_event(evt) {
            Ok(hits) => all_hits.extend(hits),
            Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
        }
        match aggregator.process(evt) {
            Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
                Ok(hits) => all_hits.extend(hits),
                Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
            },
            Ok(None) => {}
            Err(sieve_core::tool_use_aggregator::AggregatorError::MalformedToolUse {
                ref tool_id,
                ref error,
            }) => {
                tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
                all_hits.push(build_malformed_tool_use_detection(tool_id));
            }
            Err(e) => {
                tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
            }
        }
    }

    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();

    for d in all_hits {
        match &d.action {
            Action::Block => {
                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
                if d.severity == sieve_core::Severity::Critical
                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
                {
                    blocking.push(d);
                }
                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
            }
            Action::HookMark => {
                // Hook 类：写 pending 文件，SSE 流继续转发
                hook_detections.push(d);
            }
            Action::HoldForDecision { .. } => {
                // GUI 类：hold 流等决策
                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
                hold_detections.push(d);
            }
            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
            }
        }
    }

    (blocking, hook_detections, hold_detections)

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-ipc/src/pending_file.rs && sed -n '1,160p' crates/sieve-ipc/src/paths.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
use std::path::{Path, PathBuf};

use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{ensure_dirs, pending_dir},
    protocol::DecisionRequest,
};

/// 将 [`DecisionRequest`] 写入 `<base>/pending/<request_id>.json`。
///
/// 写入前用 fd-lock 对目标文件加独占写锁，防止并发写入同一 request_id（极少见
/// 但理论可行）。文件以 pretty JSON 格式写入，方便调试和 hook 侧直接读取。
///
/// 关联：SPEC-001 §3.1（pending 文件写入规约）、ADR-013 §4（文件协议备用路径）。
pub fn write_pending(req: &DecisionRequest, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let dir = pending_dir(base);
    let path = dir.join(format!("{}.json", req.request_id));

    // 打开（或创建）文件，然后加独占写锁再写内容。
    // 使用 std::fs::OpenOptions 而非 std::fs::write，以便 fd-lock 持有文件描述符。
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    let mut lock = RwLock::new(file);
    {
        let mut guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;
        let json = serde_json::to_string_pretty(req)?;
        use std::io::Write;
        guard.write_all(json.as_bytes())?;
    }

    Ok(path)
}

/// 读取并解析 `<base>/pending/<request_id>.json`。
///
/// 返回：
/// - `Ok(DecisionRequest)` 成功
/// - `Err(IpcError::PendingNotFound)` 文件不存在
/// - `Err(IpcError::Json)` 解析失败
pub fn read_pending(request_id: Uuid, base: &Path) -> Result<DecisionRequest, IpcError> {
    let path = pending_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let req: DecisionRequest = serde_json::from_str(&content)?;
    Ok(req)
}
use std::path::PathBuf;

use crate::error::IpcError;

/// 计算 sieve home 目录。
///
/// 优先级：`$SIEVE_HOME` 环境变量 > `$HOME/.sieve`。
/// $HOME 缺失时返回 [`IpcError::HomeNotFound`]。
///
/// 关联：SPEC-001 §2.1（目录结构）。
pub fn sieve_home() -> Result<PathBuf, IpcError> {
    if let Ok(val) = std::env::var("SIEVE_HOME") {
        return Ok(PathBuf::from(val));
    }
    let home = std::env::var("HOME").map_err(|_| IpcError::HomeNotFound)?;
    Ok(PathBuf::from(home).join(".sieve"))
}

/// `<sieve_home>/pending/` 目录，存放主代理写入的待决策文件。
pub fn pending_dir(base: &std::path::Path) -> PathBuf {
    base.join("pending")
}

/// `<sieve_home>/decisions/` 目录，存放 hook/GUI 写入的决策文件。
pub fn decisions_dir(base: &std::path::Path) -> PathBuf {
    base.join("decisions")
}

/// `<sieve_home>/locks/` 目录，存放文件锁占位符。
pub fn locks_dir(base: &std::path::Path) -> PathBuf {
    base.join("locks")
}

/// `<sieve_home>/ipc.sock` Unix socket 路径（主代理监听，GUI 连接）。
pub fn ipc_socket_path(base: &std::path::Path) -> PathBuf {
    base.join("ipc.sock")
}

/// 确保所有子目录存在，不存在时递归创建。
///
/// 幂等——多次调用安全。
pub fn ensure_dirs(base: &std::path::Path) -> Result<(), IpcError> {
    for dir in [pending_dir(base), decisions_dir(base), locks_dir(base)] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

exec
/bin/zsh -lc "sed -n '1,280p' .github/workflows/release.yml" in /Users/doskey/src/sieve
 succeeded in 0ms:
name: Release

# Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux/Windows 推 Phase 2 后才恢复 matrix

on:
  # ADR-011: Tag-based release 在 Week 12 GA 时启用；GA 前仅用 workflow_dispatch 触发
  # 以减少 Rekor 透明日志记录 release 节奏（防止对手推断项目进度）
  # TODO(Week 12 GA): 取消注释以下三行，启用 tag-based release
  # push:
  #   tags:
  #     - "v*"
  workflow_dispatch:
    inputs:
      tag:
        description: "要构建的 tag（例如 v0.1.0-alpha）"
        required: true

# ADR-006: id-token: write 用于 cosign keyless OIDC 签名
# contents: write 用于上传 release assets
permissions:
  contents: write
  id-token: write

env:
  CARGO_TERM_COLOR: always

jobs:
  # ─────────────────────────────────────────────────────────────
  # Tier 1 平台：reproducible build + cosign 签名
  # ADR-006 §2: Tier 1 失败 → release 中止（hard gate）
  # Phase 1 仅 macOS（PRD v1.4 §6.6 + ADR-012）；Linux target 推 Phase 2
  # ─────────────────────────────────────────────────────────────
  reproducible-build:
    name: Reproducible build (${{ matrix.target }})
    strategy:
      fail-fast: true  # Tier 1 任意失败立即中止
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-14
            artifact: sieve-aarch64-apple-darwin
          - target: x86_64-apple-darwin
            os: macos-14
            artifact: sieve-x86_64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # 需要完整 git 历史以获取 commit timestamp

      - name: 安装构建依赖（macOS）
        run: |
          brew install cmake ninja pkg-config boost ragel

      - name: 安装 toolchain + 目标平台
        run: |
          rustup show
          rustup target add ${{ matrix.target }}

      # ADR-006 §2: SOURCE_DATE_EPOCH = commit timestamp，消除构建时间污染
      - name: 设置 SOURCE_DATE_EPOCH
        run: echo "SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)" >> $GITHUB_ENV

      # ADR-006 §2: --remap-path-prefix 去除开发者路径污染
      - name: 第一次构建
        env:
          SOURCE_DATE_EPOCH: ${{ env.SOURCE_DATE_EPOCH }}
          RUSTFLAGS: "-D warnings --remap-path-prefix=$HOME=/build --remap-path-prefix=$GITHUB_WORKSPACE=/src"
        run: |
          cargo build --release --locked --target ${{ matrix.target }} -p sieve-cli
          cp target/${{ matrix.target }}/release/sieve sieve-build1
          shasum -a 256 sieve-build1 | tee sha256-build1.txt

      # ADR-006 §2: 清理后重建，SHA-256 必须一致才算 reproducible
      - name: 清理并第二次构建
        env:
          SOURCE_DATE_EPOCH: ${{ env.SOURCE_DATE_EPOCH }}
          RUSTFLAGS: "-D warnings --remap-path-prefix=$HOME=/build --remap-path-prefix=$GITHUB_WORKSPACE=/src"
        run: |
          cargo clean
          cargo build --release --locked --target ${{ matrix.target }} -p sieve-cli
          cp target/${{ matrix.target }}/release/sieve sieve-build2
          shasum -a 256 sieve-build2 | tee sha256-build2.txt

      # ADR-006 hard gate：哈希不一致则 release 中止
      - name: SHA-256 比对（必须一致）
        run: |
          H1=$(awk '{print $1}' sha256-build1.txt)
          H2=$(awk '{print $1}' sha256-build2.txt)
          echo "Build 1 hash: $H1"
          echo "Build 2 hash: $H2"
          if [ "$H1" != "$H2" ]; then
            echo "::error::Reproducible build FAILED for ${{ matrix.target }}. Hashes differ."
            echo "::error::H1=$H1"
            echo "::error::H2=$H2"
            exit 1
          fi
          echo "::notice::Reproducible build PASS: ${{ matrix.target }} — $H1"

      - name: 暂存最终二进制
        run: |
          mkdir -p dist
          cp sieve-build1 dist/${{ matrix.artifact }}
          chmod +x dist/${{ matrix.artifact }}

      # ADR-006 §1: cosign keyless OIDC 签名，bundle 格式（含 Rekor 日志条目）
      - name: 安装 cosign
        uses: sigstore/cosign-installer@v3

      - name: cosign sign-blob（keyless OIDC）
        env:
          COSIGN_EXPERIMENTAL: "1"
        run: |
          cosign sign-blob --yes \
            --oidc-issuer=https://token.actions.githubusercontent.com \
            --bundle dist/${{ matrix.artifact }}.sigstore.json \
            dist/${{ matrix.artifact }}

      # 自验证：确认签名立即可用，防止 bundle 格式异常
      - name: cosign verify-blob（自验证）
        run: |
          REPO="${{ github.repository }}"
          REF="${{ github.ref }}"
          cosign verify-blob \
            --bundle dist/${{ matrix.artifact }}.sigstore.json \
            --certificate-identity-regexp "https://github.com/${REPO}/.github/workflows/release.yml@${REF}" \
            --certificate-oidc-issuer https://token.actions.githubusercontent.com \
            dist/${{ matrix.artifact }}

      - name: 上传 artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            dist/${{ matrix.artifact }}
            dist/${{ matrix.artifact }}.sigstore.json
            sha256-build1.txt
          retention-days: 30

  # ─────────────────────────────────────────────────────────────
  # macOS universal binary（lipo aarch64 + x86_64）
  # ─────────────────────────────────────────────────────────────
  macos-universal:
    name: 构建 macOS universal binary
    needs: reproducible-build
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4

      - uses: actions/download-artifact@v4
        with:
          pattern: sieve-*-apple-darwin
          path: artifacts
          merge-multiple: false

      - name: lipo 合并 universal
        run: |
          mkdir -p dist
          # upload-artifact@v4 保留源目录结构,二进制实际位于 artifacts/<name>/dist/<name>。
          # 用 find 查找,不依赖路径前缀的具体形态。
          AARCH64_BIN=$(find artifacts -type f -name 'sieve-aarch64-apple-darwin' ! -name '*.json' | head -1)
          X86_64_BIN=$(find artifacts -type f -name 'sieve-x86_64-apple-darwin' ! -name '*.json' | head -1)
          if [ -z "$AARCH64_BIN" ] || [ -z "$X86_64_BIN" ]; then
            echo "::error::找不到 macOS 单架构产物"
            ls -laR artifacts
            exit 1
          fi
          echo "::notice::aarch64 binary: $AARCH64_BIN"
          echo "::notice::x86_64 binary:  $X86_64_BIN"
          lipo -create -output dist/sieve-macos-universal "$AARCH64_BIN" "$X86_64_BIN"
          chmod +x dist/sieve-macos-universal
          shasum -a 256 dist/sieve-macos-universal | tee sha256-universal.txt
          echo "::notice::macOS universal binary: $(awk '{print $1}' sha256-universal.txt)"

      - name: 安装 cosign
        uses: sigstore/cosign-installer@v3

      # universal binary 单独签名（lipo 合并后哈希不同于各架构产物）
      - name: cosign sign-blob（universal）
        env:
          COSIGN_EXPERIMENTAL: "1"
        run: |
          cosign sign-blob --yes \
            --oidc-issuer=https://token.actions.githubusercontent.com \
            --bundle dist/sieve-macos-universal.sigstore.json \
            dist/sieve-macos-universal

      - name: 上传 universal artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sieve-macos-universal
          path: |
            dist/sieve-macos-universal
            dist/sieve-macos-universal.sigstore.json
            sha256-universal.txt

  # ─────────────────────────────────────────────────────────────
  # 发布 GitHub Release
  # 只在 tag push 时运行（workflow_dispatch 跳过发布）
  # ─────────────────────────────────────────────────────────────
  publish:
    name: 发布 GitHub Release
    needs: [reproducible-build, macos-universal]
    runs-on: macos-14
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4

      - uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: 整理 release 文件 + 生成 SHA256SUMS
        run: |
          mkdir -p release
          # 收集所有二进制和签名文件
          find artifacts -type f \( \
            -name 'sieve-*' \
            -o -name '*.sigstore.json' \
            -o -name 'sha256-universal.txt' \
          \) -exec cp {} release/ \;
          # 生成聚合 SHA256SUMS（仅二进制，不含 .json / .txt）
          cd release
          shasum -a 256 sieve-* > SHA256SUMS
          cat SHA256SUMS

      - name: 创建 GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: release/*
          generate_release_notes: true
          fail_on_unmatched_files: true

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-cli/src/commands/doctor.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor）。
//!
//! 5 项检查：
//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
//! 2. hooks.PreToolUse 是否含 sieve-hook check
//! 3. daemon 是否在 :11453 监听（TCP 连接）
//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
//! 5. canary 本地引擎命中测试（OUT-01 规则 scan，不发真实网络请求）
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
//!
//! # R4-#7 修复说明
//!
//! 原实现向 daemon 发 HTTP 请求，检查响应里**不含**原始 canary token。
//! 该逻辑存在误报通过漏洞：daemon 未拦截（401/502 透传）时响应同样不含 canary token。
//!
//! 新实现改为**直接调用本地 sieve-rules 引擎**对 canary token 做 scan，
//! 确认规则引擎确实命中 OUT-01，不依赖 daemon 是否在线。
//! 同时独立检查 daemon TCP 监听（检查 3）。
//! 输出明确区分「规则引擎命中」与「daemon 在线」两个状态。
//!
//! # R4-#8 修复说明
//!
//! 原实现任一检查失败仍返回 `Ok(())`，导致 CI 假绿灯。
//! 新实现收集所有失败项，任一失败则返回 `Err`，含失败项名称列表。

use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::process::Command;

    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
    ///
    /// # Errors
    ///
    /// 任一检查项失败时返回 `Err`，错误信息含失败项名称列表（R4-#8）。
    pub fn run() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_default();
        let settings_path = std::path::PathBuf::from(&home)
            .join(".claude")
            .join("settings.json");

        // 收集每项检查的结果 (label, passed)
        let mut results: Vec<(&str, bool)> = Vec::new();

        // ── 检查 1: ANTHROPIC_BASE_URL
        let check1 = check_base_url(&settings_path);
        print_check(
            "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
            check1,
        );
        results.push(("ANTHROPIC_BASE_URL 配置", check1));

        // ── 检查 2: PreToolUse hook
        let check2 = check_hook_registered(&settings_path);
        print_check(
            "settings.json: hooks.PreToolUse 含 sieve-hook check",
            check2,
        );
        results.push(("PreToolUse hook 配置", check2));

        // ── 检查 3: daemon 监听 :11453
        let check3 = check_daemon_listening();
        print_check("daemon 在 127.0.0.1:11453 监听", check3);
        results.push(("daemon 监听 :11453", check3));

        // ── 检查 4: launchd 状态
        let check4 = check_launchd();
        print_check("launchd com.sieve.daemon 已加载", check4);
        results.push(("launchd 服务已加载", check4));

        // ── 检查 5: canary 本地引擎命中测试（R4-#7 修复）
        //
        // 直接调用本地 sieve-rules 引擎扫描 canary token，
        // 确认 OUT-01 规则确实命中。不发真实网络请求，不依赖 daemon 是否在线。
        // 输出明确说明「仅验证规则引擎 + daemon listening；端到端验证需手动测」。
        let check5 = check_canary_local_engine();
        print_check(
            "canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）",
            check5,
        );
        results.push(("canary 规则引擎命中 OUT-01", check5));

        // ── 汇总（R4-#8 修复）
        println!();
        let failures: Vec<&str> = results
            .iter()
            .filter_map(|(label, ok)| if *ok { None } else { Some(*label) })
            .collect();

        if failures.is_empty() {
            println!("✅ 所有检查通过，Sieve 运行正常。");
            Ok(())
        } else {
            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
            Err(anyhow::anyhow!(
                "{} 项检查失败：{}",
                failures.len(),
                failures.join("、")
            ))
        }
    }

    fn print_check(label: &str, ok: bool) {
        let icon = if ok { "✅" } else { "❌" };
        println!("  {} {}", icon, label);
    }

    /// 检查 settings.json 中 ANTHROPIC_BASE_URL。
    fn check_base_url(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|x| x.as_str())
            .map(|s| s == "http://127.0.0.1:11453")
            .unwrap_or(false)
    }

    /// 检查 PreToolUse hook 是否含 sieve-hook check。
    fn check_hook_registered(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/hooks/PreToolUse")
            .and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    /// 尝试 TCP 连接 127.0.0.1:11453，成功则 daemon 在监听。
    fn check_daemon_listening() -> bool {
        use std::net::TcpStream;
        use std::time::Duration;
        TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
    }

    /// 检查 launchctl list 是否含 com.sieve.daemon。
    fn check_launchd() -> bool {
        let Ok(output) = Command::new("launchctl").arg("list").output() else {
            return false;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("com.sieve.daemon")
    }

    /// Canary 本地规则引擎命中测试（R4-#7 修复）。
    ///
    /// 构造一个**精确匹配 OUT-01 规则格式**的 canary token，
    /// 直接调用 sieve-rules VectorscanEngine + 出站规则，验证至少 1 个 Detection 命中 OUT-01。
    ///
    /// 不发任何网络请求，不依赖 daemon 是否在线。
    ///
    /// # 为什么不发 HTTP 请求验证
    ///
    /// - daemon 不支持 runtime upstream override，无法将 canary 请求导向 fake upstream
    /// - 向真实 upstream 发请求需要有效 API key，doctor 不应持有密钥
    /// - 401/502 响应同样不含 canary token → 原逻辑误判通过（R4-#7 根本原因）
    /// - 本地引擎 scan 已足以验证检测链路最关键的一环（规则编译 + pattern 匹配）
    fn check_canary_local_engine() -> bool {
        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
        use sieve_rules::loader::load_outbound_rules;

        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
        // 在测试环境中，从 CARGO_MANIFEST_DIR 推断；生产环境从二进制同级目录推断。
        let rules_candidates: Vec<std::path::PathBuf> = vec![
            // 生产：~/.sieve/rules/outbound.toml
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".sieve")
                .join("rules")
                .join("outbound.toml"),
            // 开发：workspace 相对路径（通过 SIEVE_RULES_PATH 覆盖）
            std::path::PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
        ];

        let rules_path = rules_candidates
            .into_iter()
            .find(|p| !p.as_os_str().is_empty() && p.exists());

        let Some(rules_path) = rules_path else {
            // 规则文件不存在：canary 检查无法执行
            return false;
        };

        let Ok(rules) = load_outbound_rules(&rules_path) else {
            return false;
        };

        let Ok(engine) = VectorscanEngine::compile(rules) else {
            return false;
        };

        // 构造精确匹配 OUT-01 pattern `sk-ant-api03-[a-zA-Z0-9_\-]{93}AA` 的 canary token。
        // body = "canaryDOCTOR" (12) + "test" (4) + 'a'*77 = 93 字符，后跟 "AA"。
        // 整体格式符合真实 Anthropic API key 结构，确保 OUT-01 命中而非误判。
        let canary_token = "sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA";

        let Ok(hits) = engine.scan(canary_token.as_bytes()) else {
            return false;
        };

        hits.iter().any(|h| h.rule_id == "OUT-01")
    }
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve doctor` 非 macOS 占位实现。
    pub fn run() -> Result<()> {
        anyhow::bail!(
            "sieve doctor is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! 透传 daemon（架构图节点 ①③⑤⑧）。
//!
//! Week 2：POST /v1/messages body 收集 → 出站规则扫描 → Critical 命中时返回 426；
//! 非 messages 路径 / 解析失败 / 无命中 → 流式透传（Week 1 行为保持不变）。
//!
//! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
//!
//! Week 4（v1.4）：
//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
//!
//! 关联：PRD v1.4 §6.1 §6.7 / ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use futures_util::StreamExt as _;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use sieve_core::detection::Action;
use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
use sieve_core::sse::parser::SseParser;
use sieve_core::tool_use_aggregator::Aggregator;
use sieve_core::Forwarder;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::Config;

/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

/// 启动 daemon，永久阻塞直到进程收到信号。
///
/// `filter` 是出站规则引擎包装；`inbound_engine` + `inbound_sieveignore` 用于每连接构造
/// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
/// `cfg.dry_run` 决定是否实际拦截。
///
/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
///
/// # Errors
/// bind 端口失败或 Forwarder 初始化失败时返回错误。
pub async fn run(
    cfg: Config,
    filter: Arc<OutboundFilter>,
    inbound_engine: Arc<dyn InboundEngine>,
    inbound_sieveignore: Arc<HashSet<String>>,
) -> Result<()> {
    let listen = cfg.listen_addr()?;
    let dry_run = cfg.dry_run;
    let forwarder =
        Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);

    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
        Ok(home) => {
            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
                Ok((server, listener)) => {
                    let server = Arc::new(server);
                    let srv_clone = Arc::clone(&server);
                    tokio::spawn(async move {
                        srv_clone.run(listener).await;
                    });
                    tracing::info!(socket = %socket_path.display(), "IPC server started");
                    Some(server)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
            None
        }
    };

    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind {}", listen))?;

    tracing::info!(
        listen = %listen,
        upstream = %cfg.upstream_url,
        dry_run = dry_run,
        "sieve daemon started"
    );

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "accept failed");
                continue;
            }
        };

        let forwarder = forwarder.clone();
        let filter = filter.clone();
        let inbound_engine = inbound_engine.clone();
        let inbound_sieveignore = inbound_sieveignore.clone();
        let ipc_server = ipc_server.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                let flt = filter.clone();
                // 每连接独立 InboundFilter（&mut self trait 要求）
                let ib_filter =
                    InboundFilter::new(inbound_engine.clone(), inbound_sieveignore.clone());
                let ipc = ipc_server.clone();
                async move { proxy(f, flt, ib_filter, dry_run, ipc, req).await }
            });

            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                tracing::debug!(peer = %peer, error = %e, "connection closed with error");
            }
        });
    }
}

/// 请求入口：捕获 `proxy_inner` 的所有错误，转换为 502 Bad Gateway 响应。
async fn proxy(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(forwarder, filter, inbound_filter, dry_run, ipc, req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "proxy failed");
            let body = format!("sieve proxy error: {e}");
            let resp = Response::builder()
                .status(http::StatusCode::BAD_GATEWAY)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(string_body(body))
                .unwrap_or_else(|_| Response::new(empty_body()));
            Ok(resp)
        }
    }
}

/// 核心代理逻辑。
///
/// - POST /v1/messages → collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测
/// - 其他路径 → 流式透传（Week 1 行为）
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>> {
    let (parts, body) = req.into_parts();
    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    let is_messages_post = method == http::Method::POST && path == "/v1/messages";

    if is_messages_post {
        // 1. collect 完整 body（出站扫描需要全文）
        let collected = body
            .collect()
            .await
            .map_err(|e| anyhow!("collect body: {e}"))?;
        let body_bytes = collected.to_bytes();

        // 2. 解析 AnthropicRequest；解析失败则直接透传（上游会返回 400）
        let anthropic_req: sieve_core::protocol::anthropic::AnthropicRequest =
            match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("non-anthropic body, passing through: {e}");
                    return forward_raw(forwarder, parts, body_bytes).await;
                }
            };

        // 3. 提取文本段 → 逐段扫描
        let texts = anthropic_req.extract_text_content();
        let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

        for (offset, text) in &texts {
            use sieve_core::pipeline::PipelineNode;
            use sieve_core::protocol::unified_message::{
                ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
            };
            use std::time::SystemTime;

            let mut msg = sieve_core::UnifiedMessage {
                role: sieve_core::Role::User,
                content_blocks: vec![ContentBlock::Text {
                    text: text.clone(),
                    span: Some(ContentSpan {
                        start: *offset,
                        end: *offset + text.len(),
                    }),
                }],

exec
/bin/zsh -lc "sed -n '1,220p' crates/sieve-hook/src/lib.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
// main.rs 通过 use sieve_hook_lib::* 复用这些定义。

pub mod decision;
pub mod error;
pub mod pending;
pub mod protocol;

use std::path::Path;
use uuid::Uuid;

use decision::{write_decision, DecisionOutcome};
use error::PendingError;
use pending::{read_pending_checked, scan_pending_dir};

const STALE_THRESHOLD_SECS: i64 = 600;

/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
///
/// pending 文件不存在 → exit 0（fail-open）
/// pending 文件存在但已过期 → exit 1（fail-closed）
/// JSON 解析失败 → exit 1（fail-closed）
/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Err(PendingError::NotFound) => 0,
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            1
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            1
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            1
        }
        Ok(req) => {
            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
            let outcome = match req.default_on_timeout {
                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                _ => DecisionOutcome::Deny,
            };
            if let Err(e) = write_decision(request_id, &outcome, base) {
                eprintln!("sieve-hook: failed to write decision: {e}");
            }
            match outcome {
                DecisionOutcome::Allow => 0,
                DecisionOutcome::Deny => 1,
            }
        }
    }
}

/// 启发式运行逻辑：无 request_id 时扫目录。
///
/// 优先级 3（SPEC-001 §4.3），决策表（P1-R3-#6 修复后）：
/// - fresh=[] && stale=[] && corrupt=[] → fail-open（exit 0）：Sieve 未标记任何请求
/// - corrupt 非空 → fail-closed（exit 1）：无法确认 Sieve 判定，保守拒绝
/// - fresh 非空（corrupt=[]） → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
/// - fresh=[] && stale 非空（corrupt=[]） → 删 stale + fail-open（exit 0）
///   多 pending 时用户一次决策广播给所有 request_id。
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）；known-issues-v1.4.md §P1-R3-#6。
pub fn run_check_heuristic(base: &Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 损坏文件优先检查：只要有损坏文件，立即 fail-closed，不管 fresh 有没有。
    // 因为损坏文件可能对应本次工具调用的 Sieve 拦截标记，无法安全放行。
    // 关联：P1-R3-#6（corrupt → fail-open 漏洞修复）。
    if !scan.corrupt_paths.is_empty() {
        for corrupt_path in &scan.corrupt_paths {
            eprintln!(
                "sieve-hook: pending file {} corrupt, refusing tool call to be safe",
                corrupt_path.display()
            );
        }
        return 1;
    }

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending（corrupt=[]，stale 已清理）：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    // （非 TTY 路径：直接按策略决定，不弹提示。）
    let outcome = decide_outcome_for_requests(&scan.fresh);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
    for req in reqs {
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            _ => return DecisionOutcome::Deny,
        }
    }
    DecisionOutcome::Allow
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::path::Path;
    use uuid::Uuid;

    use crate::decision::{self, DecisionOutcome};
    use crate::pending;
    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};

    fn write_pending_json(base: &Path, req: &DecisionRequest) {
        let dir = base.join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        let json = serde_json::to_string_pretty(req).unwrap();
        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
    }

    fn make_req(
        id: Uuid,
        dot: DefaultOnTimeout,
        created_at: chrono::DateTime<Utc>,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at,
            timeout_seconds: 30,
            default_on_timeout: dot,
            detections: vec![],
        }
    }

    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────

    #[test]
    fn pending_not_found_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
    }

    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────

    #[test]
    fn pending_stale_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
    }

    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────

    #[test]
    fn pending_parse_error_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let dir = tmp.path().join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        // 写入非法 JSON。
        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
    }

    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────

    #[test]
    fn pending_allow_on_timeout_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
    }

    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────

    #[test]
    fn pending_block_on_timeout_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());

exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/commands/doctor.rs | sed -n '188,210p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   188	        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
   189	        use sieve_rules::loader::load_outbound_rules;
   190	
   191	        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
   192	        // 在测试环境中，从 CARGO_MANIFEST_DIR 推断；生产环境从二进制同级目录推断。
   193	        let rules_candidates: Vec<std::path::PathBuf> = vec![
   194	            // 生产：~/.sieve/rules/outbound.toml
   195	            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
   196	                .join(".sieve")
   197	                .join("rules")
   198	                .join("outbound.toml"),
   199	            // 开发：workspace 相对路径（通过 SIEVE_RULES_PATH 覆盖）
   200	            std::path::PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
   201	        ];
   202	
   203	        let rules_path = rules_candidates
   204	            .into_iter()
   205	            .find(|p| !p.as_os_str().is_empty() && p.exists());
   206	
   207	        let Some(rules_path) = rules_path else {
   208	            // 规则文件不存在：canary 检查无法执行
   209	            return false;
   210	        };

exec
/bin/zsh -lc 'cargo test -p sieve-hook --locked' in /Users/doskey/src/sieve
 succeeded in 166ms:
test tests::full_lifecycle_no_repeat_popup ... ok
test tests::pending_allow_on_timeout_returns_0 ... ok
test tests::heuristic_single_pending_block ... ok
test tests::pending_stale_returns_1 ... ok
test tests::env_request_id_takes_priority_over_heuristic ... ok
test tests::pending_parse_error_returns_1 ... ok
test tests::pending_block_on_timeout_returns_1 ... ok
test tests::scan_all_corrupt_yields_empty_fresh ... ok
test tests::scan_corrupt_json_goes_to_corrupt_paths ... ok
test tests::scan_io_error_goes_to_corrupt_paths ... ok
test tests::heuristic_multi_pending_all_allow ... ok
test tests::heuristic_multi_pending_all_decisions_written ... ok
test tests::scan_returns_undecided_pending ... ok
test tests::scan_skips_already_decided_pending ... ok
test tests::write_decision_removes_pending_file ... ok
test tests::write_decision_tolerates_pending_delete_failure ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/main.rs (target/debug/deps/sieve_hook-0a5b35fbe9e30564)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sieve_hook_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exec
/bin/zsh -lc "nl -ba crates/sieve-cli/src/daemon.rs | sed -n '570,600p;730,750p;862,910p'" in /Users/doskey/src/sieve
 succeeded in 0ms:
   570	                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
   571	                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
   572	                    // 关联：ADR-014 §双层防御、PRD §9 #3。
   573	
   574	                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
   575	                    if !blocking.is_empty() {
   576	                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
   577	                        for d in &blocking {
   578	                            tracing::warn!(rule = %d.rule_id, "inbound detection");
   579	                        }
   580	                        let blocked_payload = build_sieve_blocked_sse(&blocking);
   581	                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   582	                        return;
   583	                    }
   584	
   585	                    // 2. Hook 类：写 pending 文件，失败时 fail-closed（不允许 fail-open）
   586	                    for d in &hook_detections {
   587	                        if let Err(e) = write_hook_pending_or_fail_closed(d) {
   588	                            tracing::error!(
   589	                                error = %e,
   590	                                rule = %d.rule_id,
   591	                                "Hook pending write failed; fail-closed: truncating SSE stream"
   592	                            );
   593	                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
   594	                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   595	                            return;
   596	                        }
   597	                    }
   598	
   599	                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
   600	                    if !hold_detections.is_empty() {
   730	            }
   731	        }
   732	
   733	        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
   734	        let flushed = parser.flush();
   735	        let (blocking, hook_detections, flush_hold_detections) =
   736	            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);
   737	
   738	        // flush 阶段 Hook 类同样 fail-closed：写失败即截流
   739	        for d in &hook_detections {
   740	            if let Err(e) = write_hook_pending_or_fail_closed(d) {
   741	                tracing::error!(
   742	                    error = %e,
   743	                    rule = %d.rule_id,
   744	                    "Hook pending write failed (flush); fail-closed: truncating SSE stream"
   745	                );
   746	                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
   747	                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
   748	                return;
   749	            }
   750	        }
   862	    (blocking, hook_detections, hold_detections)
   863	}
   864	
   865	/// 写 IPC pending 文件，失败时返回 `Err`（调用方负责 fail-closed）。
   866	///
   867	/// 旧函数 `write_hook_pending_silent` 只 warn 后继续，违反 fail-closed 原则。
   868	/// 新函数返回 `Result`，调用方在 `Err` 时必须注入 `sieve_blocked` 并截流。
   869	///
   870	/// 关联 PRD §9 #3（Critical 不可关）、ADR-014 §Hook 路径、SPEC-001 §3.1。
   871	fn write_hook_pending_or_fail_closed(
   872	    d: &sieve_core::Detection,
   873	) -> Result<(), sieve_ipc::error::IpcError> {
   874	    let sieve_home = sieve_ipc::paths::sieve_home()?;
   875	    write_hook_pending_to(d, &sieve_home)
   876	}
   877	
   878	/// 写 IPC pending 文件到指定 base 目录，失败时返回 `Err`。
   879	///
   880	/// 内部实现，分离出来方便测试注入临时路径，不依赖环境变量。
   881	///
   882	/// 关联 SPEC-001 §3.1、ADR-014 §Hook 路径。
   883	fn write_hook_pending_to(
   884	    d: &sieve_core::Detection,
   885	    sieve_home: &std::path::Path,
   886	) -> Result<(), sieve_ipc::error::IpcError> {
   887	    use chrono::Utc;
   888	
   889	    let request_id = uuid::Uuid::new_v4();
   890	    let ipc_req = sieve_ipc::DecisionRequest {
   891	        request_id,
   892	        created_at: Utc::now(),
   893	        timeout_seconds: 60,
   894	        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
   895	        detections: vec![sieve_ipc::protocol::DetectionPayload {
   896	            rule_id: d.rule_id.clone(),
   897	            severity: map_severity_to_ipc(d.severity),
   898	            disposition: sieve_ipc::Disposition::HookTerminal,
   899	            title: format!("检测命中：{}", d.rule_id),
   900	            one_line_summary: d.evidence_truncated.clone(),
   901	            details: serde_json::json!({}),
   902	        }],
   903	    };
   904	
   905	    sieve_ipc::pending_file::write_pending(&ipc_req, sieve_home)?;
   906	
   907	    tracing::info!(
   908	        rule = %d.rule_id,
   909	        request_id = %request_id,
   910	        "HookMark: pending file written, SSE stream continues"

exec
/bin/zsh -lc "find . -maxdepth 3 -name clippy.toml -o -name .clippy.toml -o -path './.cargo/*' -type f -print -exec sed -n '1,160p' {} \\;" in /Users/doskey/src/sieve
 succeeded in 0ms:
./.cargo/config.toml
# Reproducible build flags(ADR-006)
# SOURCE_DATE_EPOCH 由 CI 设置,本地开发不强制
[build]
rustflags = [
    "--remap-path-prefix=/Users/doskey=/build",
    "--remap-path-prefix=/home/runner=/build",
    "--remap-path-prefix=/Users/runner=/build",
]

# Linux musl 静态链接
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]

# 网络访问限制(辅助 cargo deny,不强制)
[net]
git-fetch-with-cli = true

exec
/bin/zsh -lc 'cargo clippy --workspace --all-targets --all-features --locked -- -D warnings' in /Users/doskey/src/sieve
 succeeded in 136ms:
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s

exec
/bin/zsh -lc "grep -R \"~/.sieve/rules\\|rules_path\\|outbound.toml\" -n README.md docs crates | head -120" in /Users/doskey/src/sieve
 succeeded in 0ms:
docs/design/ADR-007-fail-closed-critical-actions.md:78:**Week 2 落地**：`sieve-cli` 集成 `OutboundFilter` 时，`rules_path` 加载失败或 `VectorscanEngine` 编译失败均调用 `process::exit(1)`，**不降级为无规则运行**。实现见 `crates/sieve-cli/src/main.rs` 启动序列。
docs/design/data-model.md:238:| `rules_path` | string | `"~/.sieve/rules-v{N}.tar.zst"` | 规则文件路径 |
docs/design/data-model.md:281:rules_path = "~/.sieve/rules"
docs/design/data-model.md:412:### 7.4 规则条目 schema（`outbound.toml` / `inbound.toml`）
docs/design/data-model.md:417:# crates/sieve-rules/rules/outbound.toml schema
docs/changelog/CHANGELOG.md:283:#### 出站规则集(sieve-rules/rules/outbound.toml)
docs/changelog/CHANGELOG.md:303:- `Config` 加 `rules_path` / `sieveignore_path` / `dry_run` 字段
docs/specs/SPEC-003-sieve-setup-tool.md:354:- 规则文件路径优先读 `~/.sieve/rules/outbound.toml`，可通过 `SIEVE_RULES_PATH` env var 覆盖
docs/guides/development.md:333:rules_path = "/path/to/outbound.toml"           # 出站规则集，可选（默认 crates/sieve-rules/rules/outbound.toml）
docs/review/2026-04-28-codex-review-v1.4-r3.md:39: M crates/sieve-rules/rules/outbound.toml
docs/review/2026-04-28-codex-review-v1.4-r3.md:106: crates/sieve-rules/rules/outbound.toml             |  26 +
docs/review/2026-04-28-codex-review-v1.4-r3.md:597: //! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
docs/review/2026-04-28-codex-review-v1.4-r3.md:598: //! Week 3 新增：`inbound_rules_path`（入站规则路径）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:632:     pub inbound_rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r3.md:708:             inbound_rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r3.md:2222:test config::tests::parse_dry_run_and_rules_path ... ok
docs/review/2026-04-28-codex-review-v1.4-r3.md:2225:test config::tests::resolved_rules_path_explicit ... ok
docs/review/2026-04-28-codex-review-v1.4-r3.md:2226:test config::tests::resolved_rules_path_fallback ... ok
docs/review/2026-04-28-codex-review-v1.4-r3.md:5034:/bin/zsh -lc "sed -n '1,260p' crates/sieve-rules/rules/outbound.toml && sed -n '1,260p' crates/sieve-rules/rules/inbound.toml" in /Users/doskey/src/sieve
docs/review/2026-04-28-codex-review-v1.4-r3.md:6832:+    let path = rules_path();
docs/review/2026-04-28-codex-review-v1.4-r3.md:6989:+        p.push("outbound.toml");
docs/review/2026-04-28-codex-review-v1.4-r3.md:6992:+    let rules = load_outbound_rules(&path).expect("load outbound.toml failed");
docs/review/2026-04-28-codex-review-v1.4-r3.md:7063://! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
docs/review/2026-04-28-codex-review-v1.4-r3.md:7064://! Week 3 新增：`inbound_rules_path`（入站规则路径）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:7119:    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:7121:    pub rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r3.md:7136:    pub inbound_rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r3.md:7232:            rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r3.md:7235:            inbound_rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r3.md:7291:    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:7292:    pub fn resolved_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r3.md:7293:        if let Some(p) = &self.rules_path {
docs/review/2026-04-28-codex-review-v1.4-r3.md:7296:        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
docs/review/2026-04-28-codex-review-v1.4-r3.md:7300:    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r3.md:7301:        if let Some(p) = &self.inbound_rules_path {
docs/review/2026-04-28-codex-review-v1.4-r3.md:8770:   508	        let rules_path = sieve_home.join("rules").join("outbound.toml");
docs/review/2026-04-28-codex-review-v1.4-r3.md:8771:   509	        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
docs/review/2026-04-28-codex-review-v1.4-r3.md:8802:   535	rules_path = "{rules_path}"
docs/review/2026-04-28-codex-review-v1.4-r3.md:8805:   538	inbound_rules_path = "{inbound_rules_path}"
docs/review/2026-04-28-codex-review-v1.4-r3.md:8820:   553	            rules_path = rules_path.display(),
docs/review/2026-04-28-codex-review-v1.4-r3.md:8821:   554	            inbound_rules_path = inbound_rules_path.display(),
docs/review/2026-04-28-codex-review-v1.4-r3.md:8972:   705	            content.contains(&format!("rules_path = \"{}", dir.path().display())),
docs/review/2026-04-28-codex-review-v1.4-r3.md:8973:   706	            "rules_path 必须是绝对路径: {content}"
docs/review/2026-04-28-codex-review-v1.4-r3.md:8996:   729	        assert!(cfg.rules_path.is_some(), "rules_path 应有绝对路径");
docs/review/2026-04-28-codex-review-v1.4-r3.md:9001:/bin/zsh -lc "sed -n '1,180p' crates/sieve-cli/src/daemon.rs && rg \"resolved_rules_path|load_outbound_rules|load_inbound_rules\" -n crates/sieve-cli/src/main.rs crates/sieve-cli/src/daemon.rs" in /Users/doskey/src/sieve
docs/review/2026-04-28-codex-review-v1.4-r3.md:9184:crates/sieve-cli/src/main.rs:61:            let rules_path = cfg.resolved_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r3.md:9185:crates/sieve-cli/src/main.rs:63:            let rules = load_outbound_rules(&rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r3.md:9186:crates/sieve-cli/src/main.rs:95:            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r3.md:9212:            let rules_path = cfg.resolved_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r3.md:9213:            tracing::info!(path = %rules_path.display(), "loading outbound rules");
docs/review/2026-04-28-codex-review-v1.4-r3.md:9214:            let rules = load_outbound_rules(&rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r3.md:9217:                     set rules_path in sieve.toml or ensure the default path exists",
docs/review/2026-04-28-codex-review-v1.4-r3.md:9218:                    rules_path.display()
docs/review/2026-04-28-codex-review-v1.4-r3.md:9244:            let inbound_rules_path = cfg.resolved_inbound_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r3.md:9245:            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
docs/review/2026-04-28-codex-review-v1.4-r3.md:9246:            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r3.md:9249:                         set inbound_rules_path in sieve.toml or ensure the default path exists",
docs/review/2026-04-28-codex-review-v1.4-r3.md:9250:                    inbound_rules_path.display()
docs/review/2026-04-28-codex-review-v1.4-r3.md:12039: /// 写临时 sieve.toml，其中 rules_path 为绝对路径（避免 cwd 歧义）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:12054:     let rules = outbound_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r3.md:12624:   233	    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
docs/review/2026-04-28-codex-review-v1.4-r3.md:12625:   234	    pub fn resolved_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r3.md:12626:   235	        if let Some(p) = &self.rules_path {
docs/review/2026-04-28-codex-review-v1.4-r3.md:12629:   238	        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
docs/review/2026-04-28-codex-review-v1.4-r3.md:12633:   242	    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r3.md:12634:   243	        if let Some(p) = &self.inbound_rules_path {
docs/review/2026-04-28-codex-review-v1.4-r3.md:12744:   353	    fn parse_dry_run_and_rules_path() {
docs/review/2026-04-28-codex-review-v1.4-r3.md:12749:   358	            rules_path = "/x.toml"
docs/review/2026-04-28-codex-review-v1.4-r3.md:13989:            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r3.md:13992:                         set inbound_rules_path in sieve.toml or ensure the default path exists",
docs/review/2026-04-28-codex-review-v1.4-r3.md:13993:                    inbound_rules_path.display()
docs/review/2026-04-28-codex-review-v1.4-r3.md:14042:  在 fresh `sieve setup` 后，生成的配置把 `rules_path`/`inbound_rules_path` 指到 `~/.sieve/rules/*.toml`，但 setup 流程没有创建该目录或复制内置规则；launchd 启动的 `sieve start --config ~/.sieve/sieve.toml` 会在加载规则时报错退出，导致一键安装不可用。
docs/review/2026-04-28-codex-review-v1.4-r3.md:14065:  在 fresh `sieve setup` 后，生成的配置把 `rules_path`/`inbound_rules_path` 指到 `~/.sieve/rules/*.toml`，但 setup 流程没有创建该目录或复制内置规则；launchd 启动的 `sieve start --config ~/.sieve/sieve.toml` 会在加载规则时报错退出，导致一键安装不可用。
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:16:- **文件**：`crates/sieve-rules/rules/outbound.toml:179`
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:20:# crates/sieve-rules/rules/outbound.toml:188
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:319:- **文件**：`crates/sieve-rules/rules/outbound.toml:25`
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:323:# crates/sieve-rules/rules/outbound.toml:60
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:327:# crates/sieve-rules/rules/outbound.toml:91
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:433:- **问题**：默认回退到 `crates/sieve-rules/rules/outbound.toml`，用户通过 brew/GitHub Release 启动时通常没有这个相对路径。
docs/review/2026-04-27-codex-week-1-3-implementation-review.md:434:- **建议修复**：Week 4/5 将内置规则 `include_str!` 到二进制，外部 `rules_path` 只作为覆盖。
docs/review/2026-04-28-codex-review-v1.4-r2.md:37: M crates/sieve-rules/rules/outbound.toml
docs/review/2026-04-28-codex-review-v1.4-r2.md:98: crates/sieve-rules/rules/outbound.toml             |  26 +
docs/review/2026-04-28-codex-review-v1.4-r2.md:270: //! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
docs/review/2026-04-28-codex-review-v1.4-r2.md:271: //! Week 3 新增：`inbound_rules_path`（入站规则路径）。
docs/review/2026-04-28-codex-review-v1.4-r2.md:305:     pub inbound_rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r2.md:382:             inbound_rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r2.md:4943:test config::tests::resolved_rules_path_fallback ... ok
docs/review/2026-04-28-codex-review-v1.4-r2.md:4944:test config::tests::parse_dry_run_and_rules_path ... ok
docs/review/2026-04-28-codex-review-v1.4-r2.md:4945:test config::tests::resolved_rules_path_explicit ... ok
docs/review/2026-04-28-codex-review-v1.4-r2.md:7028:/bin/zsh -lc 'git diff -- crates/sieve-rules/src/manifest.rs crates/sieve-rules/src/critical_lock.rs crates/sieve-rules/src/engine/mod.rs crates/sieve-rules/rules/outbound.toml crates/sieve-rules/rules/inbound.toml' in /Users/doskey/src/sieve
docs/review/2026-04-28-codex-review-v1.4-r2.md:7361:diff --git a/crates/sieve-rules/rules/outbound.toml b/crates/sieve-rules/rules/outbound.toml
docs/review/2026-04-28-codex-review-v1.4-r2.md:7363:--- a/crates/sieve-rules/rules/outbound.toml
docs/review/2026-04-28-codex-review-v1.4-r2.md:7364:+++ b/crates/sieve-rules/rules/outbound.toml
docs/review/2026-04-28-codex-review-v1.4-r2.md:7954:/bin/zsh -lc 'grep -n "''^id = '"\\\"OUT\" -A8 crates/sieve-rules/rules/outbound.toml" in /Users/doskey/src/sieve
docs/review/2026-04-28-codex-review-v1.4-r2.md:10946:    61	            let rules_path = cfg.resolved_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r2.md:10947:    62	            tracing::info!(path = %rules_path.display(), "loading outbound rules");
docs/review/2026-04-28-codex-review-v1.4-r2.md:10948:    63	            let rules = load_outbound_rules(&rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r2.md:10951:    66	                     set rules_path in sieve.toml or ensure the default path exists",
docs/review/2026-04-28-codex-review-v1.4-r2.md:10952:    67	                    rules_path.display()
docs/review/2026-04-28-codex-review-v1.4-r2.md:10978:    93	            let inbound_rules_path = cfg.resolved_inbound_rules_path();
docs/review/2026-04-28-codex-review-v1.4-r2.md:10979:    94	            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
docs/review/2026-04-28-codex-review-v1.4-r2.md:10980:    95	            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
docs/review/2026-04-28-codex-review-v1.4-r2.md:10983:    98	                         set inbound_rules_path in sieve.toml or ensure the default path exists",
docs/review/2026-04-28-codex-review-v1.4-r2.md:10984:    99	                    inbound_rules_path.display()
docs/review/2026-04-28-codex-review-v1.4-r2.md:11070:     5	//! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
docs/review/2026-04-28-codex-review-v1.4-r2.md:11071:     6	//! Week 3 新增：`inbound_rules_path`（入站规则路径）。
docs/review/2026-04-28-codex-review-v1.4-r2.md:11126:    61	    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
docs/review/2026-04-28-codex-review-v1.4-r2.md:11128:    63	    pub rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r2.md:11143:    78	    pub inbound_rules_path: Option<PathBuf>,
docs/review/2026-04-28-codex-review-v1.4-r2.md:11240:   175	            rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r2.md:11243:   178	            inbound_rules_path: None,
docs/review/2026-04-28-codex-review-v1.4-r2.md:11299:   234	    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
docs/review/2026-04-28-codex-review-v1.4-r2.md:11300:   235	    pub fn resolved_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r2.md:11301:   236	        if let Some(p) = &self.rules_path {
docs/review/2026-04-28-codex-review-v1.4-r2.md:11304:   239	        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
docs/review/2026-04-28-codex-review-v1.4-r2.md:11308:   243	    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
docs/review/2026-04-28-codex-review-v1.4-r2.md:11309:   244	        if let Some(p) = &self.inbound_rules_path {
docs/review/2026-04-28-codex-review-v1.4-r2.md:11420:   350	    fn parse_dry_run_and_rules_path() {
docs/review/2026-04-28-codex-review-v1.4-r2.md:11425:   355	            rules_path = "/x.toml"
docs/review/2026-04-28-codex-review-v1.4-r2.md:11429:   359	        assert_eq!(c.rules_path.as_ref().unwrap().to_str().unwrap(), "/x.toml");

exec
/bin/zsh -lc "sed -n '1,260p' crates/sieve-cli/src/engine_adapter.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` /
//! `sieve_core::InboundEngine` trait。
//!
//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，
//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
//!
//! 关联 ADR-002 / PRD §5.1 / Week 2 出站 / Week 3 入站拦截集成。

use sieve_core::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use sieve_core::error::SieveCoreResult;
use sieve_core::pipeline::inbound::InboundEngine;
use sieve_core::pipeline::outbound::OutboundEngine;
use sieve_core::protocol::unified_message::ContentSpan;
use sieve_core::tool_use_aggregator::CompletedToolCall;
use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// `VectorscanEngine` 包装，实现 `sieve_core::OutboundEngine`。
///
/// 内部持有规则反查表（`rule_id → RuleEntry`），用于从 `MatchHit` 取真实 severity/action。
pub struct OutboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表，用于从 MatchHit 映射元数据。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl OutboundAdapter {
    /// 构造 adapter。
    ///
    /// `rules` 与 `VectorscanEngine::compile` 传入的规则集一致，用于构建反查表。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

/// 把 `sieve_rules::Severity` 映射为 `sieve_core::Severity`。
fn map_severity(r: RulesSeverity) -> Severity {
    match r {
        RulesSeverity::Low => Severity::Low,
        RulesSeverity::Medium => Severity::Medium,
        RulesSeverity::High => Severity::High,
        RulesSeverity::Critical => Severity::Critical,
    }
}

/// 根据 `RuleEntry.disposition` 和 `RulesAction` 映射为 `sieve_core::Action`。
///
/// v1.4 重构：优先按 `effective_disposition()` 路由，`RulesAction` 作为兜底。
///
/// | Disposition       | Action                                       |
/// |-------------------|----------------------------------------------|
/// | AutoRedact        | `Redact { placeholder }`                     |
/// | GuiPopup          | `HoldForDecision { request_id, timeout_s }`  |
/// | HookTerminal      | `HookMark`                                   |
/// | StatusBar         | `MarkOnly`                                   |
///
/// `timeout_seconds` / `default_on_timeout` 取自 `RuleEntry`，不再硬编码 5。
///
/// 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
fn map_action_by_disposition(
    disposition: sieve_rules::manifest::Disposition,
    _rule_action: RulesAction,
    rule_id: &str,
    timeout_seconds: u32,
) -> Action {
    use sieve_rules::manifest::Disposition;
    match disposition {
        Disposition::AutoRedact => Action::Redact {
            placeholder: format!("[REDACTED:{rule_id}]"),
        },
        Disposition::GuiPopup => Action::HoldForDecision {
            request_id: uuid::Uuid::new_v4(),
            timeout_seconds,
        },
        Disposition::HookTerminal => Action::HookMark,
        Disposition::StatusBar => Action::MarkOnly,
    }
}

/// 旧接口：仅用 `RulesAction` 映射（兜底，无 disposition 信息时使用）。
///
/// `Warn` → `HookMark`（v1.4 后 Warn 一律走 HookTerminal 路径）。
///
/// 注：修 #2 后生产路径不再调用此函数（disposition 优先），
/// 保留用于单元测试验证 Warn → HookMark 的语义不变。
#[allow(dead_code)]
fn map_action(r: RulesAction) -> Action {
    match r {
        RulesAction::Block => Action::Block,
        RulesAction::Warn => Action::HookMark,
        RulesAction::Mark => Action::MarkOnly,
        RulesAction::Allow => Action::SilentLog,
    }
}

/// 截断并脱敏证据片段（用于 `Detection.evidence_truncated`）。
///
/// 超过 8 字符时，保留前 4 + `***` + 后 4，防止原始密钥写入审计日志。
fn redact_evidence(matched: &str) -> String {
    let chars: Vec<char> = matched.chars().collect();
    let len = chars.len();
    if len <= 8 {
        "*".repeat(len)
    } else {
        let head: String = chars[..4].iter().collect();
        let tail: String = chars[len - 4..].iter().collect();
        format!("{head}***{tail}")
    }
}

/// `VectorscanEngine` 包装，实现 `sieve_core::InboundEngine`。
///
/// 与 [`OutboundAdapter`] 共用辅助函数（`map_severity` / `map_action` / `redact_evidence`），
/// 额外在工具调用检查中调用 `sieve_rules::critical_lock::enforce_action` 保证 fail-closed。
pub struct InboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl InboundAdapter {
    /// 构造 adapter。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

impl InboundEngine for InboundAdapter {
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);

            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复，入站侧）。
            //
            // 规则显式写了 disposition 时直接路由；
            // disposition=None 且 fail-closed 时才强制 Block。
            // 这确保 IN-CR-02（hook_terminal）/ IN-CR-05（gui_popup）即使在 fail-closed
            // 名单里也能走正确的 HookMark / HoldForDecision 路径（不被截成 Block）。
            //
            // 关联：ADR-016（二维处置矩阵）、ADR-014（双层防御）、PRD v1.4 §5.4。
            let action = if let Some(r) = rule {
                if let Some(disp) = r.disposition {
                    // 显式 disposition：直接路由，不经过 enforce_action
                    let timeout = r.timeout_seconds.unwrap_or(60);
                    map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                } else {
                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
                    let enforced =
                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                    if enforced == RulesAction::Block {
                        Action::Block
                    } else {
                        let disp = r.effective_disposition();
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                    }
                }
            } else {
                // 规则表中找不到：fail-closed Block
                Action::Block
            };

            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_offset + hit.start,
                    end: body_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }
        Ok(detections)
    }

    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();
        // 1. 工具名扫描（IN-CR-05 签名工具）
        hits.extend(self.scan_text(&tool.name, source, 0)?);
        // 2. 工具输入序列化扫描（IN-CR-02 危险 shell 等）
        if let Ok(input_str) = serde_json::to_string(&tool.input) {
            hits.extend(self.scan_text(&input_str, source, 0)?);
        }
        Ok(hits)
    }
}

impl OutboundEngine for OutboundAdapter {
    /// 扫描文本，返回已过滤（per-rule allowlist）的命中列表，并执行 BIP39 second-pass。
    ///
    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
    ///
    /// BIP39 second-pass（PRD §9 #4）：vectorscan 之后独立扫描。
    /// 先提取全部在词表的连续词窗口，再做 SHA-256 checksum 验证，
    /// **仅 checksum 通过才生成 Critical Detection**。
    /// 词表命中但 checksum 失败的窗口**不得**定级 Critical（差异化要求）。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_byte_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            // per-rule allowlist 过滤
            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());

exec
/bin/zsh -lc "sed -n '260,560p' crates/sieve-cli/src/engine_adapter.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);
            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复）。
            //
            // 规则显式写了 disposition 时，**直接按 disposition 路由**——
            // 这确保 OUT-01（auto_redact）即使在 fail-closed 名单里也走 Redact 而非 Block。
            // 只有 disposition=None（旧规则 / 无显式配置）且 fail-closed 时，才走 Block。
            //
            // 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
            let action = rule
                .map(|r| {
                    if let Some(disp) = r.disposition {
                        // 显式 disposition：直接路由，不经过 enforce_action
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                    } else {
                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
                        let enforced =
                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                        if enforced == RulesAction::Block {
                            Action::Block
                        } else {
                            let disp = r.effective_disposition();
                            let timeout = r.timeout_seconds.unwrap_or(60);
                            map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                        }
                    }
                })
                .unwrap_or(Action::Block);
            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_byte_offset + hit.start,
                    end: body_byte_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }

        // BIP39 second-pass（关联 PRD §9 #4 差异化点）
        // vectorscan 不覆盖 BIP39，此处独立扫描：
        // 1. 按空白分词，提取全在词表的连续窗口
        // 2. 对每个窗口做 SHA-256 checksum 验证
        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
        let wl = sieve_rules::wordlist::wordlist_index();
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
        for window in candidates {
            if sieve_rules::bip39::verify_checksum(&window, wl) {
                let window_text = window.join(" ");
                let evidence_truncated = redact_evidence(&window_text);
                let fp = fingerprint("OUT-09", &window_text);
                detections.push(Detection {
                    id: Uuid::new_v4(),
                    rule_id: "OUT-09".to_string(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    // span 为整个输入范围的近似（无精确字节偏移）
                    span: ContentSpan {
                        start: body_byte_offset,
                        end: body_byte_offset + input.len(),
                    },
                    evidence_truncated,
                    fingerprint: fp,
                });
                // 同一文本只需报一次（找到一个有效助记词即触发拦截）
                break;
            }
        }

        Ok(detections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_rules::engine::VectorscanEngine;
    use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};

    fn make_rule(
        id: &str,
        pattern: &str,
        severity: RulesSeverity,
        action: RulesAction,
    ) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
        }
    }

    #[test]
    fn scan_detects_pattern() {
        let rules = vec![make_rule(
            "OUT-TEST",
            r"secret",
            RulesSeverity::Critical,
            RulesAction::Block,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        let hits = adapter
            .scan_text("my secret key", ContentSource::OutboundUserText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-TEST");
        assert_eq!(hits[0].severity, Severity::Critical);
        assert!(matches!(hits[0].action, Action::Block));
    }

    #[test]
    fn scan_no_match_returns_empty() {
        let rules = vec![make_rule(
            "OUT-TEST",
            r"secret",
            RulesSeverity::High,
            RulesAction::Warn,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        let hits = adapter
            .scan_text(
                "nothing suspicious here",
                ContentSource::OutboundUserText,
                0,
            )
            .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn map_action_warn_becomes_hook_mark() {
        // v1.4：Warn 一律走 HookTerminal 路径（HookMark action）
        let a = map_action(RulesAction::Warn);
        assert!(matches!(a, Action::HookMark));
    }

    #[test]
    fn redact_evidence_short() {
        let r = redact_evidence("abc");
        assert_eq!(r, "***");
    }

    #[test]
    fn redact_evidence_long() {
        let r = redact_evidence("1234567890abcdef");
        assert!(r.starts_with("1234"));
        assert!(r.ends_with("cdef"));
        assert!(r.contains("***"));
    }

    #[test]
    fn span_offset_applied() {
        let rules = vec![make_rule(
            "OUT-OFF",
            r"hello",
            RulesSeverity::Low,
            RulesAction::Mark,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        // offset=100, text starts at byte 0 within "say hello", pattern at 4..9
        let hits = adapter
            .scan_text("say hello", ContentSource::OutboundSystemText, 100)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].span.start, 104); // 100 + 4
        assert_eq!(hits[0].span.end, 109); // 100 + 9
    }

    // ── 修 #2 回归：disposition 优先于 enforce_action ──────────────────────────

    /// disposition=auto_redact 即使 action=block（fail-closed 名单）也走 Redact 路径。
    ///
    /// 修 #2（路由短路修复）：OUT-01 等 AutoRedact 规则在 fail-closed 名单里，
    /// 旧代码 enforce_action 会把 action 强制变 Block，跳过 disposition 路由。
    /// 修复后：显式 disposition 优先，OUT-01 必须走 Action::Redact 而非 Action::Block。
    #[test]
    fn disposition_auto_redact_beats_enforce_action() {
        let mut rule = make_rule(
            "OUT-01", // 在 fail-closed 名单里
            r"sk-ant",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::AutoRedact);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text("my sk-ant-key here", ContentSource::OutboundUserText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-01");
        // 关键断言：应该是 Redact，不是 Block
        assert!(
            matches!(hits[0].action, Action::Redact { .. }),
            "disposition=auto_redact 应走 Redact 路径，实际: {:?}",
            hits[0].action
        );
    }

    /// disposition=hook_terminal 即使在 fail-closed 名单里也走 HookMark 路径。
    ///
    /// 修 #2 回归：IN-CR-02 等 HookTerminal 规则不应被 enforce_action 截成 Block。
    #[test]
    fn disposition_hook_terminal_beats_enforce_action() {
        let mut rule = make_rule(
            "IN-CR-02", // 在 fail-closed 名单里
            r"rm -rf",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::HookTerminal);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "IN-CR-02");
        // 关键断言：应该是 HookMark，不是 Block
        assert!(
            matches!(hits[0].action, Action::HookMark),
            "disposition=hook_terminal 应走 HookMark 路径，实际: {:?}",
            hits[0].action
        );
    }

    /// disposition=gui_popup 即使在 fail-closed 名单里也走 HoldForDecision 路径。
    #[test]
    fn disposition_gui_popup_beats_enforce_action() {
        let mut rule = make_rule(
            "IN-CR-05-EVM", // 在 fail-closed 名单里
            r"eth_signTypedData",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::GuiPopup);
        rule.timeout_seconds = Some(60);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text(
                "call eth_signTypedData method",
                ContentSource::InboundAssistantText,
                0,
            )
            .unwrap();
        assert_eq!(hits.len(), 1);
        // 关键断言：应该是 HoldForDecision，不是 Block
        assert!(
            matches!(hits[0].action, Action::HoldForDecision { .. }),
            "disposition=gui_popup 应走 HoldForDecision 路径，实际: {:?}",
            hits[0].action
        );
    }
}

exec
/bin/zsh -lc "sed -n '1,180p' crates/sieve-cli/src/cli.rs && sed -n '1,120p' crates/sieve-cli/src/main.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
//! 命令行解析（clap）。
//!
//! 设计约束（ADR-007）：**禁止任何 --disable-critical / --yolo flag**。
//! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
//! 不暴露给 CLI。
//!
//! Week 5 新增（ADR-015 / SPEC-003）：`setup` / `doctor` / `uninstall` 子命令，
//! 仅 macOS Phase 1 支持；非 macOS 编译进友好错误 stub。

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Sieve LLM 流量代理命令行入口（PRD §6.1）。
#[derive(Debug, Parser)]
#[command(name = "sieve", version, about = "Sieve LLM traffic proxy")]
pub struct Cli {
    /// 子命令。
    #[command(subcommand)]
    pub command: Command,
}

/// 顶层子命令枚举。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// 启动 daemon（Week 2：出站规则拦截 + 透传）。
    Start {
        /// config.toml 路径；文件不存在时使用内置默认值。
        #[arg(short, long, default_value = "sieve.toml")]
        config: PathBuf,

        /// 仅记录命中，不实际拦截（覆盖 config.dry_run 为 true）。
        ///
        /// flag 出现即为 true；不出现时沿用 config.toml 中的 dry_run 值。
        /// 禁止添加 --no-dry-run 等关闭安全机制的 flag（ADR-007）。
        #[arg(long)]
        dry_run: bool,
    },
    /// 打印版本号并退出。
    Version,
    /// 自动配置 Claude Code 环境（仅 macOS Phase 1）。
    ///
    /// 修改 `~/.claude/settings.json`，注册 launchd plist，写审计 setup 日志。
    /// 关联：ADR-015 / SPEC-003 §setup。
    Setup(SetupArgs),
    /// 诊断 Sieve 安装状态（仅 macOS Phase 1）。
    ///
    /// 检查 settings.json / hook / daemon / launchd / canary 共 5 项。
    /// 关联：ADR-015 / SPEC-003 §doctor。
    Doctor,
    /// 干净回滚 setup 的所有改动（仅 macOS Phase 1）。
    ///
    /// 从备份目录恢复原文件，卸载 launchd plist。
    /// 关联：ADR-015 / SPEC-003 §uninstall。
    Uninstall(UninstallArgs),
}

/// `sieve setup` 参数（ADR-015 / SPEC-003 §setup）。
#[derive(clap::Args, Debug)]
pub struct SetupArgs {
    /// 不实际改文件，仅打印 diff（dry-run 模式）。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行（CI / 自动化用；仍打印 diff）。
    #[arg(long)]
    pub yes: bool,
}

/// `sieve uninstall` 参数（ADR-015 / SPEC-003 §uninstall）。
#[derive(clap::Args, Debug)]
pub struct UninstallArgs {
    /// 不实际改文件，仅打印将恢复的内容。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行。
    #[arg(long)]
    pub yes: bool,
}
//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
//!
//! 子命令：
//! - `sieve start [--config <path>] [--dry-run]`：启动 daemon
//! - `sieve version`：打印版本号
//! - `sieve setup [--dry-run] [--yes]`：自动配置 Claude Code（仅 macOS，ADR-015）
//! - `sieve doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `sieve uninstall [--dry-run] [--yes]`：回滚 setup 改动（仅 macOS）

// unsafe_code 在生产代码中禁止（等效 forbid），测试代码通过 #[allow(unsafe_code)] 豁免
// 以支持 Rust 1.80+ 的 std::env::set_var 必须用 unsafe {} 的要求。
#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

mod audit;
mod cli;
mod commands;
mod config;
mod daemon;
mod engine_adapter;

use audit::AuditStore;
use cli::{Cli, Command};
use engine_adapter::{InboundAdapter, OutboundAdapter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_rules::engine::VectorscanEngine;
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            config: cfg_path,
            dry_run: cli_dry_run,
        } => {
            let mut cfg = config::Config::load(&cfg_path)
                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;

            // CLI --dry-run 出现（true）时覆盖 config 中的值；
            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
            if cli_dry_run {
                cfg.dry_run = true;
            }

            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)

            let audit_path = cfg.audit_db_path()?;
            let _audit = AuditStore::init(&audit_path)
                .with_context(|| format!("init audit store at {}", audit_path.display()))?;

            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
            let rules_path = cfg.resolved_rules_path();
            tracing::info!(path = %rules_path.display(), "loading outbound rules");
            let rules = load_outbound_rules(&rules_path).with_context(|| {
                format!(
                    "failed to load outbound rules from {}; \
                     set rules_path in sieve.toml or ensure the default path exists",
                    rules_path.display()
                )
            })?;
            tracing::info!(count = rules.len(), "outbound rules loaded");

            // 编译出站 vectorscan db（fail-closed）
            let engine = VectorscanEngine::compile(rules.clone())
                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
            let adapter = OutboundAdapter::new(Arc::new(engine), rules);

            // 加载 .sieveignore（出站 + 入站共用同一份）
            let sieveignore_path = cfg.resolved_sieveignore_path();
            let sieveignore = load_sieveignore(&sieveignore_path);
            tracing::info!(
                path = %sieveignore_path.display(),
                entries = sieveignore.len(),
                "sieveignore loaded"
            );
            let sieveignore_arc = Arc::new(sieveignore);

            let filter = Arc::new(OutboundFilter::new(
                Arc::new(adapter),
                Arc::clone(&sieveignore_arc),
            ));

            // 加载入站规则（fail-closed，ADR-007）
            let inbound_rules_path = cfg.resolved_inbound_rules_path();
            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
                format!(
                    "failed to load inbound rules from {}; \
                         set inbound_rules_path in sieve.toml or ensure the default path exists",
                    inbound_rules_path.display()
                )
            })?;

            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
                .iter()
                .cloned()
                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
            tracing::info!(
                count = vectorscan_rules.len(),
                placeholders = placeholder_rules.len(),
                "inbound rules partitioned"
            );

            // 编译入站 vectorscan db（独立实例，fail-closed）
            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;
            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
            let inbound_adapter =
                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);


exec
/bin/zsh -lc "sed -n '1,160p' crates/sieve-hook/src/pending.rs" in /Users/doskey/src/sieve
 succeeded in 0ms:
use std::path::Path;

use uuid::Uuid;

use crate::{error::PendingError, protocol::DecisionRequest};

/// 读取并验证 pending 文件。
///
/// 返回：
/// - `Ok(DecisionRequest)` — 文件存在、未过期、解析成功
/// - `Err(PendingError::NotFound)` — 文件不存在（fail-open）
/// - `Err(PendingError::Stale)` — created_at 超过 `stale_threshold_secs`（fail-closed）
/// - `Err(PendingError::ParseError)` — JSON 解析失败（fail-closed）
/// - `Err(PendingError::IoError)` — 其他 IO 错误
///
/// 关联：SPEC-001 §4.2（stale 检测）。
pub fn read_pending_checked(
    request_id: Uuid,
    base: &Path,
    stale_threshold_secs: i64,
) -> Result<DecisionRequest, PendingError> {
    let path = base.join("pending").join(format!("{request_id}.json"));

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(PendingError::NotFound);
        }
        Err(e) => return Err(PendingError::IoError(e.to_string())),
    };

    let req: DecisionRequest =
        serde_json::from_str(&content).map_err(|e| PendingError::ParseError(e.to_string()))?;

    // stale 检测：created_at 超过阈值视为过期，fail-closed。
    let age_secs = chrono::Utc::now()
        .signed_duration_since(req.created_at)
        .num_seconds();
    if age_secs > stale_threshold_secs {
        return Err(PendingError::Stale);
    }

    Ok(req)
}

/// 启发式扫目录结果。
pub struct ScanResult {
    /// 所有有效（未过期）的 pending 请求，按 created_at 升序排列。
    pub fresh: Vec<DecisionRequest>,
    /// 过期的 pending 文件路径（供调用方删除）。
    pub stale_paths: Vec<std::path::PathBuf>,
    /// 损坏的 pending 文件路径（IO 读取失败或 JSON 解析失败）。
    ///
    /// 调用方收到非空 corrupt_paths 时必须 fail-closed（exit 1），
    /// 因为无法确定 Sieve 对这些请求的判定结果。
    /// 关联：known-issues-v1.4.md §P1-R3-#6（fail-open 漏洞修复）。
    pub corrupt_paths: Vec<std::path::PathBuf>,
}

/// 扫描 `<base>/pending/` 目录，收集所有未过期的 pending 文件。
///
/// 用于 SIEVE_REQUEST_ID 未设置时的启发式匹配路径。
/// 按 created_at 升序排列，避免随机顺序引起非确定性行为。
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录）。
pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
    let pending_dir = base.join("pending");
    let mut fresh: Vec<DecisionRequest> = Vec::new();
    let mut stale_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut corrupt_paths: Vec<std::path::PathBuf> = Vec::new();

    let entries = match std::fs::read_dir(&pending_dir) {
        Ok(e) => e,
        Err(_) => {
            // 目录不存在或无权读 → 视为空目录，fail-open。
            return ScanResult {
                fresh,
                stale_paths,
                corrupt_paths,
            };
        }
    };

    let now = chrono::Utc::now();

    let decisions_dir = base.join("decisions");

    for entry in entries.flatten() {
        let path = entry.path();
        // 只处理 .json 文件。
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                // IO 读取失败 → 算损坏，不 skip（P1-R3-#6）。
                corrupt_paths.push(path);
                continue;
            }
        };
        let req: DecisionRequest = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(_) => {
                // JSON 解析失败 → 算损坏，不 skip（P1-R3-#6）。
                corrupt_paths.push(path);
                continue;
            }
        };

        // 已决策的 pending 跳过（避免重复弹窗）。
        // 若 decisions/<id>.json 已存在，说明该请求已被处理，不再加入 fresh/stale。
        // 关联：SPEC-001 §4.3（清理机制）。
        let decision_path = decisions_dir.join(format!("{}.json", req.request_id));
        if decision_path.exists() {
            continue;
        }

        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
        if age_secs > stale_threshold_secs {
            stale_paths.push(path);
        } else {
            fresh.push(req);
        }
    }

    // 按 created_at 升序排列，保证确定性。
    fresh.sort_by_key(|r| r.created_at);

    ScanResult {
        fresh,
        stale_paths,
        corrupt_paths,
    }
}

codex
补丁让 doctor 失败变成非零是合理方向，但当前会使 setup 在验证失败时留下半配置环境；canary 规则路径也可能验证错误规则集。这些会破坏安装/诊断流程的可靠性。

Full review comments:

- [P1] 在 setup 的 doctor 失败时回滚改动 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:105-110
  当 `sieve setup` 执行完 `do_setup` 后调用 `doctor::run()?` 时，这个新增的 `Err` 会直接从 `setup::run` 返回，但回滚逻辑只包住了 `do_setup` 的 `result`；只要 daemon 尚未监听、launchd 未加载或 canary 缺规则，用户的 `settings.json` 已被改成 `127.0.0.1` 且 plist 可能已加载，却不会恢复，导致安装失败后环境处于半配置状态。请在 setup 的 doctor 失败分支也调用 `ctx.rollback()`，或避免从这里向 setup 传播未处理的 Err。

- [P2] 优先使用实际规则覆盖路径做 canary 扫描 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:193-200
  当用户通过 `SIEVE_HOME` 安装到非默认目录，或在开发/CI 中设置 `SIEVE_RULES_PATH` 覆盖规则时，这个候选列表仍先用 `$HOME/.sieve/rules/outbound.toml`，且完全不看 `SIEVE_HOME`/`sieve.toml` 的 `rules_path`。因此 `sieve doctor` 会扫描错误规则文件，可能对旧规则误报通过，或在有效安装上失败；应复用实际配置路径，并让显式覆盖优先。
2026-04-28T12:02:32.768562Z ERROR codex_core::session: failed to record rollout items: thread 019dd3f2-ed74-7942-9570-4f4787e229b0 not found
2026-04-28T12:02:32.805157Z ERROR codex_core::session: failed to record rollout items: thread 019dd3f2-ed3d-7b33-8de4-a72a1b35569c not found
补丁让 doctor 失败变成非零是合理方向，但当前会使 setup 在验证失败时留下半配置环境；canary 规则路径也可能验证错误规则集。这些会破坏安装/诊断流程的可靠性。

Full review comments:

- [P1] 在 setup 的 doctor 失败时回滚改动 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:105-110
  当 `sieve setup` 执行完 `do_setup` 后调用 `doctor::run()?` 时，这个新增的 `Err` 会直接从 `setup::run` 返回，但回滚逻辑只包住了 `do_setup` 的 `result`；只要 daemon 尚未监听、launchd 未加载或 canary 缺规则，用户的 `settings.json` 已被改成 `127.0.0.1` 且 plist 可能已加载，却不会恢复，导致安装失败后环境处于半配置状态。请在 setup 的 doctor 失败分支也调用 `ctx.rollback()`，或避免从这里向 setup 传播未处理的 Err。

- [P2] 优先使用实际规则覆盖路径做 canary 扫描 — /Users/doskey/src/sieve/crates/sieve-cli/src/commands/doctor.rs:193-200
  当用户通过 `SIEVE_HOME` 安装到非默认目录，或在开发/CI 中设置 `SIEVE_RULES_PATH` 覆盖规则时，这个候选列表仍先用 `$HOME/.sieve/rules/outbound.toml`，且完全不看 `SIEVE_HOME`/`sieve.toml` 的 `rules_path`。因此 `sieve doctor` 会扫描错误规则文件，可能对旧规则误报通过，或在有效安装上失败；应复用实际配置路径，并让显式覆盖优先。
