# ADR-001: 选用 Rust 作为技术栈

## 状态

**已接受**（v1.4 锁定执行）

> 决策日期：2026-04-26
> 范围：Phase 1（12 周 GA）整套技术栈
> 关联 PRD：[v1.4 §6.3、§9.1](../prd/_archive/sieve-prd-v1.5.md)

---

## 背景

Sieve 是夹在 AI 编码 agent（Claude Code）和上游模型之间的 LLM 流量代理，承担**双向流式安全检测**。性能预算（[architecture.md §4](./architecture.md)）极其严苛：

- **整体 P99 添加延迟 < 20 ms**
- 普通流式 chunk +30–200 µs
- 工具调用边界 +5–15 ms
- 内存峰值 < 100 MB
- 二进制 < 20 MB 单文件
- 启动时间 < 500 ms

**热路径是多模式正则匹配**——出站 12 条规则（OUT-01~~12）+ 入站 10 条规则（IN-CR-01~~05、IN-GEN-01~05），每个 SSE chunk 都要走一遍。这对编译型语言和 SIMD 加速的依赖是结构性的。

候选语言对比：


| 候选       | 多模式正则吞吐                                     | 内存占用   | 启动时间        | 单二进制分发   | 单人维护成本 |
| -------- | ------------------------------------------- | ------ | ----------- | -------- | ------ |
| **Rust** | vectorscan-rs（SIMD），比 Go regexp 快 1000+ 倍   | 低，无 GC | < 100 ms 可达 | ✅        | 中（编译慢） |
| Go       | `regexp` 单线程慢、`regexp2` 也慢；hyperscan-go 维护差 | GC 抖动  | 快           | ✅        | 低      |
| Python   | 重写到 C 扩展才行；GIL + 启动慢                        | 极高     | > 1s        | ❌ 需打包器   | 低但跑不动  |
| Node.js  | regex 引擎慢；V8 内存大                            | 高      | > 200 ms    | ❌ pkg 不稳 | 中      |


Go 在 Cloudflare Pingora 之前是反向代理首选，但 Pingora 团队公开数据显示：**Rust 在 hyperscan/vectorscan 上的多模式扫描比 Go regexp 快 1000+ 倍**——对 Sieve 这种"每个 chunk 跑全部规则"的场景，这个差异是是否能达到 P99 < 20 ms 的关键。

Python 直接出局：GIL 导致 SSE 流式并发不可控、GC 暂停不可预测、`re` 引擎慢、单二进制分发体验差（PyInstaller 打出来 50MB+ 启动 1 秒+）。

## 决策

**选用 Rust 全栈**，具体技术选型如下（直接对应 PRD §6.3）：


| 用途                   | 选型                                     | 理由                                                                                            |
| -------------------- | -------------------------------------- | --------------------------------------------------------------------------------------------- |
| HTTP 服务 + 反向代理       | `hyper 1.x` + `tokio`                  | Cloudflare Pingora 同源生态；hyper 1.x 稳定 API；tokio 是事实标准                                          |
| TLS                  | `rustls`                               | 纯 Rust 实现，无 OpenSSL 依赖，便于单二进制 + 可复现构建（详见 [ADR-006](./ADR-006-sigstore-reproducible-build.md)） |
| 多模式正则                | `vectorscan-rs`                        | 比 Go regexp 快 1000+ 倍；Intel hyperscan 的开源 fork，生产级                                            |
| JSON 流式解析            | `serde_json` + 自研 partial parser       | serde_json + 自研 SSE / partial JSON 状态机；vectorscan SIMD 加速由 sieve-rules 负责                   |
| 客户端 HTTP（调上游）        | `hyper` + `hyper-rustls`               | 复用 hyper 服务端栈，避免引入第二个 HTTP 客户端                                                                |
| 配置                   | `serde` + `toml`                       | 标配，无需自研                                                                                       |
| SQLite               | `rusqlite`                             | 审计日志 + license key 本地存储                                                                       |
| 哈希 / 校验              | `sha2` + `crc32fast`                   | BIP39 SHA-256 校验、GitHub token CRC32                                                           |
| BIP39 / base58 / hex | `bip39` + `bs58` + `hex`               | 加密原语，避免自研出错                                                                                   |
| 字符串相似度               | `strsim`（Levenshtein）                  | AddressGuard 用                                                                                |
| 日志                   | `tracing` + `tracing-subscriber`       | 结构化、零成本抽象                                                                                     |
| 测试 / fuzz            | `cargo test` + `cargo fuzz`（libFuzzer） | SSE 边界 fuzz 必需（PRD §9.5）                                                                      |


**编译目标**：

- macOS：`x86_64-apple-darwin` + `aarch64-apple-darwin`（universal binary）
- Linux：`x86_64-unknown-linux-musl`（静态链接，无 glibc 依赖）
- Windows：`x86_64-pc-windows-msvc`（次要，Phase 1 仅二进制可用）

---

## 影响

### 正面影响

1. **性能达标**：vectorscan SIMD + serde_json 提供高性能处理，是 P99 < 20 ms 唯一能算得过来账的栈；
2. **内存安全 + 并发安全**：作为代理产品，OOM 或并发数据竞争意味着用户全停 ——Rust 的所有权系统在编译期排除了这两类灾难；
3. **单二进制分发**：`rustls` + `musl` 让 Linux 二进制完全静态链接；macOS universal binary 一份就够；< 20 MB 二进制大小可达；
4. **生态对齐**：Cloudflare Pingora、StepSecurity Harden-Runner、Deno、Datadog Rust agent 都是同生态——Sieve 跟随主流而非创新；
5. **可复现构建友好**：Rust 编译器对 `SOURCE_DATE_EPOCH` 支持成熟，`cargo` 的 lockfile 严格——这是 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 的物质基础；
6. **AI 协作好**：Claude Code 写 Rust 的质量在 2026 年明显高于 2024 年，编译器反馈 + 类型系统对 AI 协作非常友好。

### 负面影响

1. **编译慢**：clean build 在 M2 Pro 上需要 3–8 分钟；增量编译可达可接受范围（10–30s），但首次 CI 构建是痛点 —— 用 `sccache` + GitHub Actions cache 缓解；
2. **vectorscan 编译复杂度**：vectorscan 需要 clang + cmake + boost，CI 配置不像 pure Rust crate 那么直接；macOS / Linux / Windows 三平台各自有坑，必须 Week 1 就把 CI 跑通（不能拖到 Week 6 才发现 Windows 编译失败）；
3. **招人面窄**：但本项目**单人无所谓**——doskey 一个人 + Claude Code，没有招人压力，反而是 Rust 的"只能他自己读代码"的特性可以阻止社区贡献者引入低质量 PR；
4. **学习曲线**：lifetime / async / pin 这些概念对单人开发者是真实成本，但 doskey 已有 Rust 经验，且 Claude Code 在写 Rust 时的能力已经超过大部分新手；
5. **依赖管理风险**：`cargo` 不像 npm 那样脆弱，但 LiteLLM 事件提示了供应链风险——必须配合 [ADR-006](./ADR-006-sigstore-reproducible-build.md) 的 pinned dependencies 和签名验证。

### 需要更新的文档

- `docs/guides/development.md`（待编写）—— 写明依赖列表、本地编译指南、vectorscan 编译注意事项
- `docs/guides/deployment.md`（待编写）—— 写明三平台二进制打包流程、musl 静态链接配置、universal binary 合并步骤
- [architecture.md](./architecture.md) §2 —— 模块依赖列已对齐本 ADR
- [data-model.md](./data-model.md) —— 内部数据结构已用 Rust 伪代码示例
- `docs/changelog/CHANGELOG.md`（待编写）—— Phase 1 启动后记录技术栈选定

---

## 相关文档

- [PRD-sieve v1.4 §6.3](../prd/_archive/sieve-prd-v1.5.md) —— Rust 技术栈表
- [PRD-sieve v1.4 §9.1](../prd/_archive/sieve-prd-v1.5.md) —— 工程硬约束第 1 条："Rust 栈非选项"
- [architecture.md](./architecture.md) —— Phase 1 整体架构与性能预算
- [ADR-002](./ADR-002-rule-engine-only-phase1.md) —— Phase 1 纯规则引擎（依赖 vectorscan）
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— Sigstore + Reproducible Build（依赖 Rust 工具链）