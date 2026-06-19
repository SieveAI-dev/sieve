---
name: Bug 报告
about: 报告 Sieve 自身的功能缺陷（非安全漏洞 — 安全漏洞走 SECURITY.md）
title: "[BUG] "
labels: bug
assignees: doskey
---

> ⚠️ **如果是安全漏洞**（供应链 / fail-closed 失效 / 数据泄漏 / 检测绕过），请走 [SECURITY.md](../../SECURITY.md) email 渠道，**不要在这里公开**。

## 受影响版本

```
sieve --version    # 贴完整输出（含 SHA-256 + rules version）
```

- OS / arch:
- 安装方式: brew / 二进制 / 源码构建
- Claude Code 版本:

## 复现步骤

1.
2.
3.

## 期望行为



## 实际行为



## 相关日志

请贴 `~/.sieve/logs/sieve.log` 的相关片段。

> ⚠️ **不要贴原始 prompt 内容** —— 只贴 fingerprint / rule_id / 时间戳。Sieve 不存原文，你也不应该把原文贴到公开 issue（PRD §11.2）。

```
（脱敏后的日志）
```

## 截图（如适用）



## 提交前自检

- [ ] 已搜索现有 issue 没有重复
- [ ] 已验证二进制 sigstore 签名（[部署指南 §3](../../docs/guides/deployment.md#3-二进制签名验证必做)）
- [ ] **未在本 issue 粘贴任何原始 prompt 内容 / API key / 私钥 / 助记词 / 地址**
- [ ] 已读 `docs/guides/deployment.md` §12 FAQ 排除常见配置问题
