---
name: 可疑样本提交
about: 提交在中转站 / LLM 输出中发现的可疑攻击样本（PRD §8.1 公开渠道）
title: "[SAMPLE] "
labels: sample, intel
assignees: doskey
---

## ⚠️ 提交前必读

**Sieve 不通过产品上传任何样本**（PRD §11.2）。你在这里提交的内容是**公开的**——会被 doskey 评估、公开讨论，并可能转化为 Sieve 规则进入下一版规则库。

### ❌ 严禁贴出

- 你的真实 API key / 私钥 / 助记词（即使你认为已经泄漏）
- 包含真实身份 / 企业信息的对话
- 完整的钱包地址（保留前 6 + 后 4 即可，如 `0x742d35...1234A`）
- 任何能识别到具体受害用户的信息

### ✅ 应该贴出

- 攻击者修改后的 tool_call / 地址 / 代码（脱敏后）
- 中转站入口 URL（如已知是攻击源）
- 攻击模式描述
- UCSB 论文 / Pink Drainer 等已公开攻击的复现样本

---

## 攻击类型

- [ ] 中转站修改 tool_call（PRD §1.2 第 1 句）
- [ ] 地址替换攻击（IN-CR-01）
- [ ] 危险工具调用（IN-CR-02 / IN-GEN-01-03：`rm -rf /` / `curl|sh` / `eval(base64)`）
- [ ] 签名钓鱼（IN-CR-05：EIP-712 数字化绕过 / typed data 滥用）
- [ ] 敏感路径访问（IN-CR-03：`~/.ssh/` / `~/.aws/` / `.keystore`）
- [ ] 持久化机制（IN-CR-04：crontab / launchd / systemd / shell rc）
- [ ] Markdown 图片 exfil（IN-GEN-04）
- [ ] Prompt injection（IN-GEN-05：`<|im_start|>` / `[INST]` / `Ignore previous`）
- [ ] Drainer 合约（Pink Drainer / Inferno / 其他）
- [ ] Solidity / Vyper 后门（合约层）
- [ ] npm / pip typosquat
- [ ] 其他:

## 复现环境

- AI 客户端: Claude Code / 自写 agent / 其他:
- 上游: Anthropic 官方 / 中转站名（如能公开）:
- 触发率: 必现 / 偶发（X/Y 次）/ 仅一次

## 攻击样本（脱敏后）

```
（贴样本，**严格遵循上方"应该贴出"和"严禁贴出"的清单**）
```

## 你的脱敏方法

请说明你是如何脱敏的，让 doskey 能评估样本可信度：



## 现有 Sieve 是否能捕获？

如果你已经在用 Sieve：

- 触发的 rule_id（如有）：
- 处置等级（Critical / High / Medium / Low）：
- 是否被 fail-closed 阻断：

如果**未被捕获**，这是最有价值的样本——会直接驱动新规则加入 PRD §5.2 Phase 2 表。

## 致谢偏好

- [ ] GitHub 用户名（默认）
- [ ] Twitter 用户名:
- [ ] 匿名（不致谢）

> 数据合作意向（PRD §13.2）：如果你来自 SlowMist / ScamSniffer / GoPlus / Chainabuse 等机构想做长期数据合作，请同时邮件 doskey.lee@gmail.com。
