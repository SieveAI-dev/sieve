# Doskey 必须亲自做的剩余工作

> 创建：2026-04-29
> 触发：W7-B + W10 文章 + W11 landing page 自动化推完后，剩下的事必须你亲自做
> 原则：每条都说明为什么子代理代不了你

---

## 一、Phase 1 GA 前必须做（Week 7-12，硬阻塞）

### 1. 海外公司注册（Week 1 启动 → Week 7-8 完成，已 4 周走起来）

**做什么**：香港有限公司或新加坡 Pte Ltd（推荐香港，按 ADR-005）
**为什么子代理做不了**：
- 需要本人身份证 / 护照
- 需要付公司注册费 + 秘书服务（HK ~$2K/年 / SG ~$3K/年）
- 银行开户必须本人亲自到场（HK 汇丰 / 渣打 / 星展）
**进度自查**：
- [ ] 选好哪个司法管辖区（HK / SG / Stripe Atlas Delaware）
- [ ] 注册资料提交
- [ ] 拿到 Certificate of Incorporation
- [ ] 开 corporate bank account
- [ ] 申请 BR (HK) / Operating License
**deadline**：Week 7-8 必须拿到执照，否则 Week 11-12 Stripe 接入卡死
**关联**：[ADR-005](../docs/design/ADR-005-overseas-legal-entity.md)

---

### 2. Stripe / Paddle 商户接入（Week 7-8）

**做什么**：注册 Stripe 商户 + 接入 Sieve 试用→付费转化流程
**为什么子代理做不了**：
- 必须用上面注册的海外公司主体
- KYC 审核需要本人提供 ID + utility bill
- 域名邮箱必须验证
**进度自查**：
- [ ] 海外公司主体就位（依赖事项 1）
- [ ] Stripe 账号创建 + KYC 通过
- [ ] 接入 Stripe Checkout 或 Stripe Billing
- [ ] webhook URL 配到 Sieve license server（待建）
- [ ] 测试 charge / refund / subscription 流程
**deadline**：Week 11 闭测扩大前

---

### 3. License 系统（Week 7-8）

**做什么**：Sieve 二进制如何验证用户付费 + 14 天试用怎么算
**为什么子代理做不完**：
- 需要后端服务（Cloudflare Workers / Fly.io / 自部署 VPS 都可以）
- 需要域名（license.sieve.<your-domain>）
- 需要持久化（D1 / Postgres）
- 涉及商业决策（offline 试用怎么算 / refund 政策 / 团队多机器怎么算）

**子代理可以帮做的部分**：
- [ ] license 服务后端代码（Rust + axum 或 TypeScript + Hono）
- [ ] sieve-cli 客户端验证逻辑（每次启动 ping license server）
- [ ] 离线宽限期实现（断网 N 天仍可用）

要让子代理做这部分，给我说一声。

---

### 4. 加密支付通道（Week 7-9）

**做什么**：Coinbase Commerce / Stripe Crypto / 自部署 ETH/USDC 收款
**为什么子代理做不了**：商业账号 KYC + 钱包私钥保管（你的资产）
**推荐**：先 Coinbase Commerce（USDC + ETH + BTC，托管，最简）；GA 6 个月后再考虑自部署
**进度自查**：
- [ ] Coinbase Commerce 账号
- [ ] Webhook 接 license server
- [ ] 测试一笔实际付款

---

### 5. 域名 + DNS

**做什么**：买域名（建议 sieve.dev / sieve.security / 类似），配 DNS / Email / SSL
**为什么子代理做不了**：需要付费 + 你的 Cloudflare/Namecheap 账号
**进度自查**：
- [ ] 主域名买好
- [ ] sieve.<domain> CNAME → landing page hosting
- [ ] license.sieve.<domain> CNAME → license server
- [ ] api.sieve.<domain> CNAME → 规则更新 endpoint
- [ ] support@sieve.<domain> 邮箱（Google Workspace / Migadu）

---

### 6. GA repo 公开 + 二进制开源（Week 12，**最严肃**）

**做什么**：把 GitHub repo 从 private 翻为 public（按 ADR-011）
**为什么子代理不能代你做**：这是不可逆动作 + 全世界看的事件
**强制 checklist**（按 ADR-006 + ADR-011）：
- [ ] 所有 secret 检查（grep 仓库历史 `sk-` / `eyJ` / `ghp_` / 邮箱 / 内部 IP）
- [ ] git history 审计（确认没 leak 过任何东西）
- [ ] sigstore CI pipeline 在 main 上每次 push 都签
- [ ] reproducible build 验证（你自己跑一遍 → hash 跟 GitHub Release 一致）
- [ ] LICENSE 文件确认 MIT
- [ ] SECURITY.md 含 vuln disclosure 流程
- [ ] CODEOWNERS 配你自己
- [ ] 第一篇文章（中转站揭黑）发布**之前**确认 repo 公开

---

### 7. 二进制 .dmg 打包 + 公证（Week 11-12）

**做什么**：把 sieve-cli + sieve-hook + GUI App（独立仓库 sieve-gui-macos 你单独做）打包成 .dmg
**为什么子代理代不了**：
- macOS notarization 需要 Apple Developer Program 账号（$99/年）
- 需要你的 Apple ID + 2FA
- 公证流程需要 Apple ID 验证
**进度自查**：
- [ ] Apple Developer Program 加入
- [ ] Developer ID Application certificate
- [ ] xcrun altool / notarytool 打公证
- [ ] cosign sign .dmg + 上传到 Rekor

**子代理可以帮做的部分**：
- [ ] release.yml workflow 加 .dmg 打包步骤（已部分 stub）
- [ ] sieve setup 修复 R3-#1（部署内置规则到 ~/.sieve/rules/）—— 但前提是 .dmg 结构定下来

---

## 二、闭测 / KOL（Week 9-11，软阻塞）

### 8. Discord 闭测频道

**做什么**：起一个 Discord server，邀请 5 个海外 crypto dev 闭测
**为什么子代理代不了**：人脉 + 邀请 + 每天看反馈
**目标用户**：
- 2 个 hackathon builder（DEFCON / ETHGlobal 圈）
- 2 个智能合约审计研究员
- 1 个 OpenClaw / Hermes 重度用户

### 9. KOL 接洽

**目标**（PRD §13.3）：
- **Chaofan Shou** (@Fried_rice) UCSB+Fuzzland —— 优先（他是论文作者，最 relevant）
- **慢雾 @evilcos** —— 中文圈影响力 + 数据合作
- **Yu Feng** UCSB 教授 —— 通过 Chaofan 间接

**为什么子代理代不了**：私信 + 关系建立 + 真人对话
**doskey 应该准备的**：
- 一句话 pitch
- demo 视频（30 秒）
- 文章 1+2+3 草稿（W10 子代理在写）

### 10. 内容渠道运营

**Twitter @doskey** + **Mirror.xyz** + **Hacker News** + **doskey.dev 个人博客**：
- 文章 1（W12 GA 同步发）
- 文章 2（GA 后 1 周）
- 文章 3（GA 后 2 周）
- 之后每月 1 篇深度（W13+ 慢节奏）

子代理写了草稿（`docs/external/article-{1,2,3}-*.md`），你润色 + 发。

---

## 三、个人时间投入

### 11. dogfood（Week 6-12 持续）

**做什么**：你自己 100% 时间用 Sieve 工作（PRD §10.1 Week 6-8）
**为什么必须你做**：
- 真实工作流（写 ERC20 转账脚本 / 审计 / 跟进 bug bounty）
- 收集 false positive
- 主观 UX 反馈（弹窗烦不烦 / 倒计时合理不）
**自动化能给的**：
- 把当前 sieve-cli + sieve-hook + sieve setup 搞通能跑（已基本做到）
- 你把 OpenClaw / Hermes 装上让 setup 跑通后，回报 SPEC-004 §10 6 个 TBD 的实测结果

### 12. 反馈闭环

**做什么**：每天处理闭测用户反馈（Week 9-12）
**最容易暴露的真问题**：
- known-issues-v1.4.md 列的 R3-#1 (.dmg 规则部署) / R3-#3 (RedactAndAllow 漏脱敏) / R3-#5 (IN-CR-01 disposition) 等
- doctor / setup 真实环境的边界 case
- GUI App 在独立仓库 (sieve-gui-macos) 的 IPC 协议握手 bug

---

## 四、决策类（你拍板，我推进）

### 13. 域名 + 产品名

**当前状态**：codename "Sieve"（PRD v1.4 文件头）
**Open Question**（PRD v1.4 §14 第 1 条）：正式产品名 Week 6-8 之间必须定
**候选思路**：保留 Sieve / 改用 doskey 个人品牌 / 全新名

### 14. 价格弹性

**当前 PRD §7.1**：$49/月单价
**可考虑**：
- $29 / $49 / $99 三档？（基础 / 专业 / 团队）
- 年付 vs 月付折扣？
- 团队版 / Enterprise 版（PRD 说"等 5+ 客户主动问再说"，先保留）

### 15. 国内合规边界

**当前 PRD §11.4**：海外公司主体 + 不在国内做 to-C 商业化
**Open Question**：知乎 / B 站 / 即刻 是否完全不发？还是只发研究内容（如 UCSB 论文翻译）？

---

## 五、监控类（GA 后持续）

### 16. supply chain 监控

**做什么**：每周 grep PyPI / npm 新发布的包，看有没有针对 Sieve / 类似产品的 typosquat / 投毒
**自动化能给的**：写个 GitHub Actions cron，结果发 Telegram / 邮件
**为什么登记**：触发了你才能反应

### 17. 竞品动向

**做什么**：每月看 GoPlus AgentGuard / Blockaid / Lasso 是否升级
**PRD §12 风险**：竞品升级是高概率风险

---

## 六、不做的事（明确登记）

防止"feature creep"，PRD §1.3 + §9 明确不做的：

- ❌ 中转站
- ❌ LLM Gateway
- ❌ 钱包
- ❌ 审计公司
- ❌ 云 SaaS
- ❌ 反恶意软件 / OS 级拦截 / 本地 CA / MITM
- ❌ 在 API 协议层撒谎（不伪造 tool_use）
- ❌ Phase 1 适配 Gemini / Mistral / Cursor / VS Code 插件
- ❌ Linux / Windows GUI（推 Phase 2）
- ❌ 团队版 / Enterprise（等 5+ 客户问）
- ❌ 融资 / 招人 / 企业销售 / ads / 转售用户数据

---

## 七、本仓库已自动化推完的清单

供你 review 已经做了什么：

✅ Week 1-3：Rust workspace + 协议骨架 + 出站规则 + 入站 Crypto 钩子
✅ Week 4：IN-CR-02~04 + IN-GEN-01~05 + 二维矩阵 + sieve-hook + benchmark 数据集 (FP 0.0000% / recall 100% / P99 24~104µs)
✅ Week 5：Native GUI（独立仓库 B1 由你做）+ sieve setup + IPC + sieve-hook 二进制
✅ Week 6 (v1.5)：OpenAI 协议 + multi-agent setup + IN-GEN-06/IN-CR-06
✅ Week 7-B：OpenClaw / Hermes 6 个 TBD 实现（基于公开调研，Week 8 dogfood 验证）
✅ Week 10 内容草稿：3 篇引爆文章（你润色 + 发）
✅ Week 11 landing page 第一版（你换 placeholder + 配 DNS + deploy）

剩 6 条 known-issues（v1.4 R3-#1/#3/#5/#4 + R8-#2-Inbound + R9-#2）等真实 dogfood / GUI 真做后回头修。

---

## 八、合并节奏建议

不要等所有事都做完才发 GA，按这个顺序：

```
Week 7-8: 海外公司 + Stripe + License + 域名（4 件并行）
Week 9: Discord 闭测频道 + 邀请 5 个用户
Week 10: 文章 1 润色 + Mirror 发首版
Week 11: KOL 接洽 + landing page 上线 + 闭测扩到 10 人
Week 12 GA 当天:
  - repo public
  - 文章 1+2 同步发（中英文）
  - landing page 完整版
  - .dmg 公证 + 签名版
  - Stripe 上线收款
  - sieve doctor 必须能在新装机器上一键跑通
```

修复 known-issues 推到 GA 后第一周（Week 13）。
