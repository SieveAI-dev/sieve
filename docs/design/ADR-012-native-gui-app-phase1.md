# ADR-012: Phase 1 必做 Native GUI App（macOS SwiftUI 独立进程）

## 状态

**已接受**

> 决策日期：2026-04-28
> 范围：Phase 1 macOS GUI 架构；Windows/Linux GUI 推 Phase 2

---

## 背景

早期架构决策（architecture.md §6）明确标注：

> ❌ 桌面 GUI App（Electron / Tauri）—— Phase 1 不做

这一决策在 v1.3 时代成立：GUI 是"做减法"的产物，Phase 1 优先跑通检测引擎。

v1.4 版本引入了 **HIPS 弹窗**需求：
- IN-CR-01（地址替换）/ IN-CR-05（签名工具）/ IN-GEN-04（markdown exfil）这类 **GUI 类规则**，需要在检测命中后向用户呈现完整上下文（替换前后地址对比、typed data 详情）并收集授权决策；
- 终端 y/n（sieve-hook 模式）只适合 Hook 类规则的低上下文决策；GUI 弹窗提供 120 秒读透明数据的空间，是差异化核心；
- 安装程序（.dmg）、菜单栏常驻状态、设置面板 preset 切换，都需要 native app 作为载体。

过去排除 Electron / Tauri 的理由（引入 Chromium 运行时、二进制膨胀、无法统一 macOS 系统权限）**仍然成立**。排除的是"跨平台 webview 方案"，不是"native app"本身。

### 架构选项对比

| 方案 | 安装包载体 | 弹窗体验 | 与代理进程解耦 | Phase 1 可行 |
|------|-----------|---------|--------------|-------------|
| SwiftUI 独立进程（macOS native） | .dmg / Notariziation | 系统级弹窗、无 webview | 独立进程，崩溃不影响代理 | ✅ |
| Electron | 250MB+ 运行时 | webview | 独立进程 | ❌ 太重 |
| Tauri | 较轻 | webview | 独立进程 | ❌ 跨语言维护成本 |
| 终端 TUI（单一进程内）| 无需 | ASCII 弹窗 | 同进程，崩溃连带 | ❌ 体验差 |
| stdout mock（占位）| 无需 | 无视觉 | 独立进程 | 仅 Week 3-4 过渡 |

---

## 决策

### 1. Phase 1 必做 macOS SwiftUI 独立 App

**撤销** architecture.md §6 原有"❌ 桌面 GUI App（Electron / Tauri）"的否决——该否决仅针对 webview 方案，不适用于 native SwiftUI。

v1.4 Week 5 里程碑：GUI App 完成 HIPS 弹窗主流程（地址替换 / 签名 / markdown exfil 弹窗，含授权/拒绝/倒计时 3 段视觉）。

### 2. GUI 在独立 git 仓库 `sieve-gui-macos`

Rust 仓库（本仓库）**不出现任何 Swift 代码、Xcode 工程文件或 .swift 文件**。

理由：
- Rust CI（cargo clippy / fuzz / bench）与 Xcode CI（swift build / xcodebuild test）完全解耦，避免交叉依赖卡流水线；
- monorepo 没有协作同步价值，反而拖慢 Rust 的 clean build 时间；
- `sieve-gui-macos` 仓库遵循与本仓库相同的发布前管理原则。

### 3. GUI 进程独立于代理进程

GUI App 崩溃 **不影响** `sieve-cli` 守护进程的检测能力。失联时的降级行为由 IPC 协议超时策略决定（见 ADR-013 / ADR-014）——超时触发 `default_on_timeout`（通常为 Block），实现 fail-closed。

### 4. GUI 职责边界

| 职责 | 属于 GUI App | 属于 Rust 代理 |
|------|-------------|---------------|
| .dmg 安装包 + macOS Notarization | ✅ | ❌ |
| HIPS 弹窗渲染（地址对比、typed data、倒计时） | ✅ | ❌ |
| 菜单栏常驻图标（状态显示） | ✅ | ❌ |
| Preset 设置面板（Minimal / Standard / Paranoid） | ✅ | ❌ |
| 检测逻辑（规则匹配、SSE 解析） | ❌ | ✅ |
| 审计日志写入（SQLite）| ❌ | ✅ |
| IPC server（Unix socket JSON-RPC） | ❌ | ✅ |

### 5. 跨仓库协调机制

- **IPC 协议版本号**（`v1` 起）是两仓库之间唯一硬约束；版本不匹配时代理侧报错友好提示用户升级 GUI；
- IPC schema 变更必须同步更新 SPEC-001（sieve-hook-protocol）和 SPEC-002（hips-popup-behavior），两仓库各自引用；
- GUI App 版本遵循 semver，与 Rust 侧 major 版本对齐（SPEC 版本号决定兼容性）。

### 6. Phase 1 只做 macOS

Windows / Linux GUI 推 Phase 2，触发条件与 ADR-004 §3 第二适配器相同（有真实用户主动要求时）。

---

## 影响

### 正面影响

1. **HIPS 弹窗差异化**：完整 typed data 展示 + 倒计时 3 段视觉 + 授权/拒绝，是 Lakera / LLM Guard 不会做的 native 体验；
2. **安装包规范化**：.dmg + code signing + Notarization 是 macOS 生态标准，降低"不明来源应用"警告；
3. **进程解耦**：GUI 崩溃不影响检测，代理侧 fail-closed 仍然生效；
4. **CI 解耦**：Rust CI 不被 Xcode 依赖污染，构建速度保持可控。

### 负面影响

1. **跨仓库协调成本**：IPC schema 改动需两仓库同步发版；缓解：SPEC 先写，protocol version 握手严格执行；
2. **SwiftUI 学习曲线**：macOS app 开发有上手成本（Xcode / codesign / Notarization 流程）；建议提前做纸面练习；
3. **Windows / Linux 用户等待**：Phase 2 触发前无 GUI，只有 sieve-hook 终端弹窗；README 提前说明；
4. **macOS Notarization 时间**：第一次走 Apple 审核通道约 1-3 天；提前建好 Apple Developer 账号和证书。

### 需要更新的文档

- `docs/design/architecture.md` §6 —— 删除"❌ 桌面 GUI App（Electron / Tauri）"条目，改为"✅ macOS SwiftUI 独立进程（sieve-gui-macos 仓库）"
- `docs/guides/deployment.md` §2.1 —— macOS 安装改为 .dmg 流程
- `docs/guides/development.md` §2 仓库结构 —— 加"GUI 代码在独立仓库 sieve-gui-macos"说明
---

## 相关文档

- [ADR-013](./ADR-013-ipc-protocol.md) —— IPC 协议（JSON-RPC Unix socket + 文件锁）
- [ADR-014](./ADR-014-dual-layer-defense.md) —— 双层防御（GUI 类规则 hold 流 + GUI 弹窗）
- [ADR-015](./ADR-015-sieve-setup-tool.md) —— sieve setup 自动配置（含 .dmg 安装链路）
- [architecture.md](./architecture.md) —— Phase 1 整体架构
