# Sieve 安全策略

> Sieve 是 LLM 流量层的安全代理（[PRD §1.1](docs/prd/sieve-prd-v2.0.md#11-一句话)），自身被攻陷会摧毁产品定位。
> "自证清白"是产品安全承诺的核心叙事（[PRD §1.2 第 4 句](docs/prd/sieve-prd-v2.0.md#12-四句话核心叙事v13-加第-4-句) + [ADR-006](docs/design/ADR-006-sigstore-reproducible-build.md)）。
>
> **请不要在 GitHub Issues 公开报告 Sieve 自身的安全漏洞。** 走下方私有渠道。

---

## 报告渠道

| 渠道 | 说明 | 适用阶段 |
|------|------|---------|
| **Email** | doskey.lee@gmail.com | Phase 1 GA 之前唯一渠道 |
| Email | security@sieveai.dev | GA 后启用（域名 sieveai.dev 已确定 2026-05-05；DNS / MX 注册待 ADR-005 [redacted]） |
| PGP | TBD（Week 6-8 GA 前公布公钥指纹） | GA 后启用 |

请在邮件标题加前缀 `[SIEVE-SECURITY]`，内容包含：

- **受影响版本**：`sieve --version` 输出（含二进制 SHA-256 + rules version）
- **平台**：OS / arch / 安装方式（brew / 二进制 / 源码构建）
- **漏洞类型**（任一）：
  - 供应链（二进制 / 规则包 / 依赖被篡改）
  - 数据泄漏（prompt 上传 / 远端 verifier）
  - fail-closed 失效（YOLO mode 绕过 / Critical 拦截被关闭）
  - 检测绕过（已知攻击模式未触发）
  - 配置层校验缺失（如 `bind_address = 0.0.0.0` 未被拒绝）
  - 拒绝服务（崩溃 / 内存爆炸 / 死锁）
- **复现步骤**（最小化用例）
- **影响评估**：可能导致的资产损失风险等级
- **致谢偏好**：GitHub 用户名 / Twitter / 匿名

---

## 响应时间承诺

| 阶段 | SLA |
|------|-----|
| 邮件确认收到 | **24 小时内** |
| 初步评估（严重程度 + 复现验证） | **7 天内** |
| 修复或缓解（按严重程度） | Critical: 7 天 / High: 30 天 / Medium: 90 天 |
| 公开 advisory | 修复发布后 30 天内 |

> doskey 是[redacted]（[PRD §1.4](docs/prd/sieve-prd-v2.0.md#14-项目性质--法律实体v13-修订)），上述 SLA 已考虑 1 人响应能力。如涉及**当前正在被利用 + 资产损失风险**，请在邮件标题加 `[URGENT]`，doskey 会优先响应。

---

## 责任披露原则

- 修复发布前请勿公开（包括会议演讲 / 博客 / Twitter / 漏洞数据库）
- 修复发布同时致谢报告者（除非要求匿名）
- 涉及 Sieve 用户资产损失风险时，立即推送强制升级 + 透明日志（[ADR-006 §3](docs/design/ADR-006-sigstore-reproducible-build.md)）公示事件
- doskey 不提供 bounty 现金奖励（[redacted]无预算），但对重大发现会在 GA 后的 advisory + 文章中署名致谢

---

## 不在范围

以下不构成 Sieve 安全漏洞：

- **用户配置错误**：如试图把 `[server].bind_address` 改成 `0.0.0.0`（配置层会拒绝启动）
- **中转站 / 上游 API 漏洞**：不在 Sieve 责任范围（这恰恰是 Sieve 检测的对象，[PRD §1.2 第 1 句](docs/prd/sieve-prd-v2.0.md#12-四句话核心叙事v13-加第-4-句)）
- **钱包 / 浏览器扩展钓鱼**：Sieve 是认知摩擦层，不是钱包安全产品
- **检测规则的误报 / 漏报**（除非违反 [PRD §6.5 误报率预算](docs/prd/sieve-prd-v2.0.md#65-误报率预算)）：误报治理走 [`.sieveignore`](docs/api/api-reference.md) 与 GitHub Issue 普通流程
- **使用过期 / 未签名的二进制**：用户责任在安装时验证 [sigstore 签名](docs/guides/deployment.md#3-二进制签名验证必做)

---

## 自身供应链承诺

详见 [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](docs/design/ADR-006-sigstore-reproducible-build.md)：

- 所有 release 二进制 **sigstore 签名 + Rekor 透明日志**（cosign verify-blob 可验证）
- **Tier 1（macOS / Linux）reproducible build** 双构建 SHA-256 必须一致才能 release
- **Tier 2（Windows）** 提供二进制 + sigstore 签名，reproducible build 推到 Phase 2
- 规则包 **Ed25519 签名 + fail-closed 验证**（签名失败 → 沿用上一份已验证规则）。alpha build 公钥占位、暂为 fail-open（skip+warn，靠同源 SHA-256 兜底）；**GA build 经 `ga_keys` 编译期 gate 强制真实公钥就位**，占位则编译失败（[ADR-034](docs/design/ADR-034-ga-key-gate.md)）
- **pinned dependencies**：`Cargo.lock` 入库 + Dependabot 周更（major 升级单独评估）

供应链审计建议：

```bash
# 1. 验证二进制签名
cosign verify-blob \
  --certificate-identity-regexp '^https://github.com/SieveAI-dev/sieve/' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --bundle sieve.sigstore \
  ./sieve

# 2. 自己复现构建对比 SHA-256（Tier 1）
git clone https://github.com/SieveAI-dev/sieve.git --branch v0.1.0
./scripts/repro-build.sh linux-amd64
sha256sum target/repro/sieve-linux-amd64
sha256sum ./sieve   # 必须一致

# 3. 抓包验证"不联网 verifier"（ADR-003）
sudo tcpdump -i any -nn host '!api.anthropic.com and !your-relay.com'
sieve --config ~/.sieve/config.toml
# 期望：除上游 API 外无任何外发流量
```

---

## 历史 Advisories

> Week 12 GA 前 advisories 不分配正式 `SIEVE-YYYY-NNN` 编号，记录在 CHANGELOG。GA 后启用正式编号。

### Pre-GA P0：非流式 JSON 入站检测绕过（已修复 2026-05-01）

- **影响版本**：v1.5.0 ~ v1.5.3（v1.5.x 70 条入站规则）
- **影响范围**：
  - **漏洞 1（Anthropic）**：`application/json` 非流式响应里的 `tool_use` 绕过所有入站规则（IN-CR-02/03/04/05 / IN-GEN-* 全失效）
  - **漏洞 2（OpenAI）**：`proxy_openai` `stream=false` 分支跳过入站检测；OpenAI 协议**默认 stream=false**，意味着 OpenAI 入站规则**从未生效过**
- **严重程度**：P0（PRD §5.2 "入站是 Sieve 真正的护城河"语境下属严重产品级缺陷）
- **修复**：v1.5.4 commit `14153e2`，详见 [CHANGELOG](docs/changelog/CHANGELOG.md#v154-non-streaming-json-inbound-fix---2026-05-01)
- **修复验证**：2 条新集成测试 + dataset_fp_rate 0% FP / 99.71% Recall 无回归
- **披露状态**：Pre-GA 期间 dogfood 内部测试中发现并修复，发现与修复均在公开发布前完成，无外部用户受影响

---

## 相关文档

- [ADR-003: 完全本地运行，绝不联网做 token verifier](docs/design/ADR-003-local-only-no-cloud-verifier.md)
- [ADR-006: Sigstore 签名 + Reproducible Build + 透明日志](docs/design/ADR-006-sigstore-reproducible-build.md)
- [ADR-007: Critical 等级 fail-closed 强制确认](docs/design/ADR-007-fail-closed-critical-actions.md)
- [PRD §9 工程硬约束](docs/prd/sieve-prd-v2.0.md#9-工程上必须做对的硬约束)
- [PRD §11 法律与合规边界](docs/prd/sieve-prd-v2.0.md#11-法律与合规边界)
- [部署指南 §3 二进制签名验证](docs/guides/deployment.md#3-二进制签名验证必做)
