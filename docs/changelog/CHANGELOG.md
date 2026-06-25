# Changelog

本文件记录 Sieve 面向用户的显著变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

> 公开 API 在首个稳定版发布后冻结，破坏性变更走 SemVer。冻结范围参见 [API 参考 - 接口冻结声明](../api/api-reference.md#接口冻结声明)。

---

## [Unreleased]

### Added

- **入站四路由对等覆盖**：所有入站检测统一覆盖 Anthropic 与 OpenAI 的流式（SSE）与非流式（JSON）四类响应路由，新增检测能力不会只挂流式而漏掉非流式。
- **可配置上游路由表**：网关新增配置化路由表，接入「协议兼容但请求路径不同的中转站」只需加一行路由配置、零代码改动；非标准请求路径可一行映射到对应 provider。
- **出站数据外泄链检测**：行为序列检测扩展为出站多步外泄链识别，覆盖读取密钥后打包外发、编码/加密后外发、跨 agent 拆分、写入剪贴板、写入公共产物等链型；引入密钥置信度分级（命中出站脱敏规则即升维高置信）。默认关闭、保守起步，命中仅状态栏通知，不引入新的阻断路径。
- **出站新增 Bitcoin WIF / BIP-32 扩展私钥（xprv）脱敏**：两种 Base58Check 编码的加密货币私钥格式，先格式粗筛产候选、再做校验和验证，仅校验和通过才动作，形似但校验和错误的串放行以压低误报。命中后自动改写请求、状态栏通知，不弹窗。
- **Canary 诱饵文件检测**：`sieve setup` 在敏感凭据/钱包目录布放警告型诱饵文件，工具调用读到诱饵即触发 Critical 阻断 + 人工确认；`sieve doctor` 新增诱饵自检（纯本地，不发网络），`sieve uninstall` 按记录回滚。
- **可选审计加密归档**：新增可选特性，审计归档只存脱敏后内容、采用仅追加的接收方加密——运行进程只持公钥无法解开历史归档，解密私钥离线保护，叠加哈希链防篡改与保留期自动清理。默认不编入主二进制。
- **可选本地用量与超额计费观测**：新增可选特性，对经过流量做独立 token 核算、与中转站声明的用量交叉比对，偏差超容差报警不阻断。用量统计严格本地留存、永不上传。默认 opt-in、默认不编入主二进制。
- **一行自校验安装链路（计划中）**：`curl|bash` 安装器、Homebrew formula/cask、`cargo install` 元数据就位——安装器下载二进制时自动校验签名或 SHA256，任一校验失败立即退出不安装。

### Changed

- **CLI 瘦身**：默认 `sieve` 二进制收窄到核心能力面（`start` / `decisions` / `rules` / `audit` 查询 / `setup` / `doctor` / `uninstall` / `version`）；token 用量观测与加密审计归档拆为可选特性，默认不编译，配置在任意特性组合下均向后兼容。
- **上游网关分层重构（行为等价）**：把上游差异收进可替换的边缘层，让接入协议兼容但路径不同的中转站零代码改动。本轮为纯结构重构，检测结论、脱敏输出、转发字节逐字节不变。
- **入站响应内容类型路由加固**：`Content-Type` 路由判定改为稳健的 media-type 匹配（大小写不敏感、容忍 `; charset=…` 参数与前导空格），消除非规范大小写绕过 JSON 路径入站检测的风险。

### Security

- **入站地址替换在非流式响应的覆盖闭合**：地址替换检测此前只覆盖流式响应，非流式 JSON 响应在 `stream=false` 时未被扫描；现已统一覆盖全部四类路由，非流式路径同样提取响应文本喂入检测并对待决路径 fail-closed 阻断。

### Fixed

- 修复 IPC 服务的 accept 循环遇瞬态错误即终止整个循环的问题：连接级瞬态错误立即重试、资源类错误退避后重试，控制面（决策弹窗 / hook / reload）不再因单次错误永久失效。
- 修复 daemon 与 GUI 之间多处通信字段不一致导致解码失败、连接中断的问题，并以协议规格为权威源逐个对齐。
- 补全出站脱敏、用户决策、入站 Critical 拦截路径的审计写入，`sieve audit query` 现可查到全部出/入站决策与拦截（不持久化任何明文 secret）。
- 修复 `sieve audit query --severity` 过滤因大小写不一致始终返回空的问题，改为大小写不敏感匹配并兼容历史数据。
- 修复 `sieve audit` / `sieve decisions` 命令的运行时崩溃，两命令恢复可用。
- 修复规则更新包解压因压缩格式魔数判断错误导致永远失败的问题，规则热更新通道恢复可用。
- 修复 release 可复现构建在多核并行下产出非确定 SHA-256 的问题：发布构建改为串行（`-j1`），消除 fat-LTO 符号合并顺序受并行编译完成顺序影响的不确定性，确保用户可自行复现并逐字节比对官方二进制哈希。

---

## [0.1.0-alpha]

首个早期预览版。Sieve 是一个完全本地运行的 LLM 流量安全代理（Rust 单二进制），夹在 AI 编码 agent 与上游模型 API 之间，对 crypto 开发者做双向检测。

### Added — 核心能力

- **出站脱敏**：检测并自动脱敏请求中的高熵密钥、加密货币私钥、BIP39 助记词（助记词做 SHA-256 校验和验证，仅词表匹配不足以定级）。命中后自动改写请求 body、状态栏通知，不弹窗、不打断工作流。
- **入站危险工具调用拦截**：对签名 / 转账 / 部署等不可逆 Critical 工具调用强制 fail-closed + 人工确认（GUI 弹窗或终端 y/N），所有版本、任何模式下不可关闭。
- **入站地址替换检测**：维护会话内出现过的 EVM 地址，对模型响应中的近似地址替换告警，提示地址替换攻击。
- **入站敏感路径与持久化机制检测**：覆盖 SSH / 云凭据 / 钱包 keystore / Keychain 等敏感路径读取，以及 shell rc / crontab / launchd / systemd 等后门埋点写入。
- **入站危险 shell 命令拦截**：覆盖下载执行、`curl|bash`、Windows PowerShell 管道等远程代码执行形态，以及无限授权签名、钱包连接劫持等签名风险。
- **行为序列检测（默认关闭）**：滑动窗口识别多步攻击链，保守起步，命中仅状态栏通知，不引入阻断路径，可由用户主动开启。

### Added — agent 接入

- **支持四家 AI 编码 agent**：Claude Code、OpenClaw、Hermes、Codex CLI，把上游基地址指向本地 daemon 即可接入。
- **统一双协议支持**：Anthropic Messages API 与 OpenAI Chat Completions 解析为统一中间表示，运行时同时支持两种协议。
- **`sieve setup` 一键配置**：自动检测并配置 agent 环境、注册 PreToolUse hook、写入基地址、加载后台守护；`--all-detected` 自动扫描已安装 agent。
- **`sieve doctor` 安装诊断** 与 **`sieve uninstall` 干净回滚**：均支持单 agent 或 `--all` 全量。
- **多 agent 调用链追踪**：sub-agent 嵌套调用携带签名的调用链元数据防伪造，调用链过深强制 fail-closed。

### Added — 上游与网关

- **多上游监听**：可同时绑定多个端口，每个端口显式声明上游协议并独立连接不同上游；协议与请求路径错位时 fail-closed 拒绝。只认单一基地址的 agent 通过指向不同端口切换上游，无须注入路由 header。
- **上游转发代理**：daemon 转发上游可经配置的 HTTP CONNECT / SOCKS5 代理出网，解决受限网络下硬直连不可用的问题；TLS 端到端到上游，代理只见密文、不做 MITM。规则更新复用同机制。

### Added — 规则分发与更新

- **签名规则包下发**：检测规则以签名规则包形式分发；开源引擎在无规则包时按空集 fail-safe 透传启动。
- **规则自动更新（可关闭）**：客户端定期检查并下载规则更新，下载经 TLS 校验签名与哈希后原子替换；可通过环境变量关闭更新检查、关闭附带的匿名标识、或指向自托管更新源。

### Added — 用户规则与决策

- **用户自定义规则**：通过 `sieve rules` 管理（`edit` / `list` / `disable` / `enable`），用户规则只能在 High 档做 Ask/Warn/Mark，加载或编译失败时 daemon 仍正常启动、系统规则全功能、且永不能覆盖或抑制系统 Critical。
- **headless 决策 CLI**：`sieve decisions` 让 daemon 在 GUI 不在线时仍可用，支持流式订阅待决事件、查看上下文、批准/阻断/告警；`sieve start --no-client-policy` 配置无 client 在线时的兜底策略。
- **灰名单**：用户主动放行的指纹可记忆免重复弹窗，Critical 永不可入灰名单。

### Added — 审计

- **可 unix-pipe 的审计查询**：`sieve audit` 提供 `tail` / `query` / `show`，输出 jsonl 方便接 jq / fluentd / vector，查询直接读本地审计库。
- **审计 logging 三档**：`off` / `metadata`（默认，与已发布行为一致）/ `full`（用户显式开启的加密 write-only 归档）。

### Architecture

- **三件套职责分层**：daemon 承担全部检测与决策；CLI 是 daemon 的入口与控制面；macOS 菜单栏 GUI 只做交互（人工确认、审计浏览、状态栏通知），本身不做任何检测。所有检测纯本地，绝不联网做远端校验。
- **daemon ↔ client IPC**：JSON-RPC over Unix socket，协议版本白名单仅 v2；daemon 主动推握手、决策请求与状态栏通知，支持多 client fan-out；所有 client 地位平等，不引入特权接口。wire schema 权威源为 [SPEC-005](../specs/SPEC-005-ipc-protocol.md)。
- **出站脱敏不打断工作流，入站 Critical fail-closed 不可关**：高频脱敏类自动改写 + 状态栏通知；Critical 工具调用强制人工确认，YOLO mode 也不可关。
- **不在上游协议层撒谎**：拦截时允许截流注入 Sieve 自报的阻断事件，但不伪造模型字段。
- **不装本地 CA 做 MITM**：TLS 端到端到上游。
- **供应链 sigstore 签名 + 可复现构建**：CI 强制门控。

### Security

- **加密审计 full 档抗运行时攻陷**：归档只存脱敏后内容，运行进程只持公钥无法解开历史归档，叠加哈希链防篡改；即便运行时被攻陷也读不出历史归档，整盘被拷拿到的也是密文。
- **超额计费的用量统计严格本地留存、永不出网。**

---
