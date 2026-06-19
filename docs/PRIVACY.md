# Sieve Privacy & Telemetry

> Version: v1.0 — 2026-06-19

Sieve 是一个**完全本地运行**的 LLM 流量安全代理。它从不上传你的 prompt、响应、API key、
使用记录、设备序列号或任何账号信息。本文件说明 Sieve **唯一**的出站网络行为，以及如何关闭它。

## Sieve 会联网做什么

| 出站 | 目的 | 频率 | 可关闭 |
|------|------|------|--------|
| 你主动调用的上游 LLM API | 代理你的请求（产品核心功能） | 按你的使用 | 不调用即不发生 |
| `updates.sieveai.dev/v1/manifest` | 检查规则/版本更新（规则不更新会失效） | 每 6 小时一次 | `SIEVE_NO_UPDATE=1` |
| `cdn.sieveai.dev` | 下载新规则正文 | 有更新时 | 同上 |

## 更新检查携带哪些数据

更新检查请求携带最小化、匿名的查询参数：

- `v` 当前版本 · `os` 操作系统 · `arch` CPU 架构 · `ch` 发布通道（stable）
- `uid` 一个本地随机生成的安装标识（UUIDv4）

**关于 `uid`：**
- 首次运行时本地生成，纯随机，**不含**任何设备指纹 / MAC / 序列号 / 账号 / 邮箱信息
- 存于 `<cache_dir>/install-id`，文件权限 `0600`
- 删除该文件即生成新的（重新计为一次新安装）
- 仅用于估算活跃安装数，帮助判断何时推送规则更新

## 如何关闭

```bash
# 关闭安装标识上报（更新检查仍进行，但不带 uid）
export SIEVE_NO_TELEMETRY=1

# 或完全关闭更新检查（规则将不再自动更新）
export SIEVE_NO_UPDATE=1
```

也可在配置文件中设置 `update.telemetry = false` 或 `update.enabled = false`。

## 我们的承诺

Sieve 的核心是**可验证**：开源核心引擎、sigstore 签名、可复现构建。
你不必相信这份文档——你可以自己抓包验证上述出站行为，或读源码（`crates/sieve-updater/`）。
如发现任何与本文件不符的网络行为，请按 SECURITY.md 报告，我们视为安全事件处理。
