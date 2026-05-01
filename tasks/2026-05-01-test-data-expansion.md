# 测试数据集扩充计划（2026-05-01）

## 背景

现状 `bench-data/`：226 条 attacks + 70 条 benign，跑出 0% Critical FP。这对工程门禁（PRD §9 #7）够用，对付费用户的"一周不误拦"信心不够。本次扩充直接面向"用户最怕的五件事"+ "看起来像攻击但合法"两条线。

不在本次范围：真实流量录制（B 方案，等真实用户上线后做）、公开攻击复现（C 方案，本批之后做）。

## 目标矩阵

| 类别 | 现有 | 新增目标 | 总计 |
|------|------|---------|------|
| benign generic | 70 | 0 | 70 |
| benign near-* (10 桶 × 100) | 0 | 1000 | 1000 |
| attacks 现有 | 226 | 0 | 226 |
| attacks-by-fear (5 桶 × 120) | 0 | 600 | 600 |
| **合计** | **296** | **1600** | **~1900** |

## 目录结构

```
bench-data/
├── benign/                              # 现有 70 条不动
├── benign-near/                         # 新增：与 attacks 对称，按"看起来像哪个规则"分桶
│   ├── near-OUT-api-keys/               # 看起来像 API key 但合法（教学/文档/示例）
│   ├── near-OUT-tokens/                 # 看起来像 JWT/Stripe/Slack/Discord token 但合法
│   ├── near-OUT-private-keys/           # 看起来像 PEM/SSH 私钥但合法（PEM 教学/SSH 配置文档）
│   ├── near-IN-CR-01-address/           # 看起来像地址替换但合法（多地址列表/同前缀对比）
│   ├── near-IN-CR-02-rce/               # 看起来像 RCE 但合法（curl/eval/nc 教学）
│   ├── near-IN-CR-03-secret-read/       # 看起来像敏感文件读取但合法（.env / .ssh / keystore 文档）
│   ├── near-IN-CR-04-persistence/       # 看起来像持久化但合法（cron/launchd/systemd 教学）
│   ├── near-IN-CR-05-crypto-addr/       # 看起来像 crypto 地址操作但合法
│   ├── near-IN-CR-06-misc/              # 其他 IN-CR-06 + IN-GEN-* 类
│   └── extra-generic-multilingual/      # 真实开发者口吻 + 中文/日/韩/西多语言扩充
├── attacks/                             # 现有 226 条不动
└── attacks-by-fear/                     # 新增：用户最怕的五件事
    ├── signing/                         # 钱包签名诱导（personal_sign / signTypedData / EIP-712 等）
    ├── transfer/                        # 转账（ERC-20 / native / approve+transferFrom / multi-call）
    ├── env-leak/                        # .env 外泄（cat / Read tool / docker env / CI secret）
    ├── private-key/                     # 私钥外泄（PEM / hex / WIF / ssh / Keystore / Keychain）
    └── shell-rce/                       # shell 命令注入（curl|sh / eval / base64-d|sh / os.system）
```

## 命名规则

- benign：`{桶名}-NNN.txt`（NNN 三位数）
- attacks：`fear-{主题}-NNN.txt`（不与现有 IN-CR-* 命名冲突）

## 内容多样性预算（每桶强制）

| 占比 | 内容形态 |
|------|---------|
| 60% | 真实公开内容风格（Solidity by Example / Foundry book / Hardhat docs / Ethereum SE 等的口吻） |
| 20% | 同一真实样本的变体改造（改地址/改语言/改长度） |
| 10% | **多语言**：中文 / 半中半英 / 日 / 韩 / 西（按真实用户构成） |
| 10% | **格式变种**：裸文本 / Markdown / 含 code fence / 含 SSE delta 边界 / 含 JSON tool_use |

## 验收标准

每个子代理完成后，主上下文跑：

```bash
cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture
```

- benign FP rate（含新桶）必须 < 0.5%（PRD §9 #7 硬约束）
- attacks recall（含新桶）必须 > 95%

如果某新桶 FP 高，**先调 allowlist** 而不是删测试样本（allowlist 是规则资产，删测试样本是逃避）。

## 子代理拆分（15 个，分 3 批）

### 第一批（5 个 attacks-by-fear，最优先）

| Agent | 输出目录 | 数量 |
|-------|---------|------|
| `attack-signing` | `bench-data/attacks-by-fear/signing/` | 120 |
| `attack-transfer` | `bench-data/attacks-by-fear/transfer/` | 120 |
| `attack-env-leak` | `bench-data/attacks-by-fear/env-leak/` | 120 |
| `attack-private-key` | `bench-data/attacks-by-fear/private-key/` | 120 |
| `attack-shell-rce` | `bench-data/attacks-by-fear/shell-rce/` | 120 |

### 第二批（5 个 benign near-OUT-*）

| Agent | 输出目录 | 数量 |
|-------|---------|------|
| `benign-near-out-api-keys` | `bench-data/benign-near/near-OUT-api-keys/` | 100 |
| `benign-near-out-tokens` | `bench-data/benign-near/near-OUT-tokens/` | 100 |
| `benign-near-out-private-keys` | `bench-data/benign-near/near-OUT-private-keys/` | 100 |
| `benign-near-in-cr-01-address` | `bench-data/benign-near/near-IN-CR-01-address/` | 100 |
| `benign-near-in-cr-02-rce` | `bench-data/benign-near/near-IN-CR-02-rce/` | 100 |

### 第三批（5 个 benign near-IN-* + 多语言）

| Agent | 输出目录 | 数量 |
|-------|---------|------|
| `benign-near-in-cr-03-secret-read` | `bench-data/benign-near/near-IN-CR-03-secret-read/` | 100 |
| `benign-near-in-cr-04-persistence` | `bench-data/benign-near/near-IN-CR-04-persistence/` | 100 |
| `benign-near-in-cr-05-crypto-addr` | `bench-data/benign-near/near-IN-CR-05-crypto-addr/` | 100 |
| `benign-near-in-cr-06-misc` | `bench-data/benign-near/near-IN-CR-06-misc/` | 100 |
| `benign-extra-multilingual` | `bench-data/benign-near/extra-generic-multilingual/` | 100 |

## 子代理共用约束

每个子代理必须：

1. **只在自己的目录写文件**，不动其他目录、不动源代码、不动规则文件
2. 读取 `crates/sieve-rules/rules/{inbound,outbound}.toml` 中对应规则的 pattern，理解触发条件
3. 写入 `.txt` 文件，每条 100-800 字节（避免太短没信息量、太长拉慢测试）
4. 文件命名连续编号 001-N
5. **benign 桶禁止包含真实可触发 Critical 的字符串**（含真实可解析私钥/真实 sk-ant-* 前缀完整长度等），用 `EXAMPLE_` / `[REDACTED]` / 显式截断
6. **attacks 桶必须包含真实能命中规则的特征**（即拷贝粘贴进 Claude Code 应该被拦）
7. 完成后输出一行汇总：`done: <dir> = <N> files, sample subjects: ...`

## 完成后由主上下文做的事

- 跑 `cargo test --release --test dataset_fp_rate -- --ignored --nocapture` 看新数据下的 FP/recall
- 如果有 FP，按桶定位是哪类合法场景被误伤，调 allowlist
- 写一份"扩充后测试报告"（按"用户最怕的五件事"维度组织数字，准备给营销文章用）
- C 方案（公开攻击复现）作为下一批任务规划
