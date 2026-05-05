# ADR-027: Network jail enforcement —— 防火墙层硬隔离 LLM 流量，bypass 不到

## 状态

**Proposed**

> 决策日期：2026-05-05
> 范围：Phase 3.x（v3.x post-GA opt-in 高级特性，**不影响 GA 时间表**）
> 关联 PRD：[v2.0 §1.2](../prd/sieve-prd-v2.0.md)、[v2.0 §9 #12](../prd/sieve-prd-v2.0.md)（不装本地 CA / 不 MITM 硬约束）、[v2.0 §11.2](../prd/sieve-prd-v2.0.md) 信任叙事

---

## 背景

### 现有威胁模型的 bypass 缺口

Sieve 当前架构靠 `ANTHROPIC_BASE_URL=http://127.0.0.1:11453` 让 agent 自愿把流量引入代理——这是 opt-in reverse proxy 模式，不是强制收口。三类常见 bypass 路径让这个模式出现结构性漏洞：

**路径 1：用户配错（善意 bypass）**

agent 环境变量漏配或错配时，流量直走 Anthropic / OpenAI，sieve 完全不知情。配置错误无声失败，审计日志出现无法解释的空窗，用户以为 sieve 在保护他们，实际上保护已失效。

**路径 2：第三方依赖偷调 LLM endpoint（无意 bypass）**

npm 包、二进制工具、agent 框架插件可以硬编码 `https://api.anthropic.com` 发请求，完全无视 `ANTHROPIC_BASE_URL`。随着 agentic 生态扩张，工具链里嵌套的 LLM 调用越来越多，sieve 对这些调用一无所知。

**路径 3：恶意 agent 主动绕路（主动 bypass）**

不在 PRD 威胁模型内，但威慑面缺口仍是产品弱点。被攻陷的 agent 可以在 tool_use 里直接调用 `curl https://api.anthropic.com/v1/messages`，绕开 sieve 代理层。

**当前 doctor 的盲点**：doctor 只在 setup 时刻验证一次配置是否正确，运行期发生的 bypass 完全发现不了。

### PRD §9 #12 硬约束不可破：透明 MITM 方案永久否决

显而易见的"强力"方案是透明拦截 + 解 TLS + 重新发上游（mitmproxy 模式）。PRD §9 #12 明确禁止，理由是多层面的：

1. **CA 注入要 sudo + 永久写 trust store**，破坏"普通用户态进程，一条命令卸载干净"的极简安装承诺；
2. **sieve 自身变成本地 CA = 高价值攻击面**，root cert 一旦被偷，整机所有 HTTPS 连接全线崩；
3. **部分 agent / SDK 做 cert pinning**，会直接拒连 sieve 自签证书，造成 hard failure，且无可见错误提示；
4. **「我们从不动你的 trust store」是 sieve 的核心信任叙事**（PRD §11.2 + ADR-006），装 CA 等于自废武功。

因此，mitmproxy 模式被永久否决，不作为候选方案。

### 「透明拦截但不解 TLS」方案也被否决

折中方案是：把 `*.anthropic.com:443` 重定向到 sieve，但不解密，只看 SNI + 加密字节流。问题是：

- **sieve 全部价值失效**：API key 扫描 / 地址替换 / tool_call 内容 / SSE 流过滤全部依赖 body 可见，SNI-only 拦截做不到
- **没有差异化**：`lsof / tcpdump / Little Snitch` 能做的事这个方案都能做，而且做得更好，sieve 没必要重复

### 第三条路：防火墙当 jail，不当 inspector

**结论先行**：把防火墙降级为「强制收口」而非「重定向 + 解密」。

规则极度简化：**除 sieve 自身外的任何进程，禁止直连 LLM endpoint:443**。

流量路径保持不变：`ANTHROPIC_BASE_URL=127.0.0.1:11453` → sieve 代理（明文 HTTP）→ sieve 扫描 → sieve 自己用 rustls 拨上游 443。

**不解 TLS、不装 CA、不动 trust store**，PRD §9 #12 不破。这是 iptables / pf 哲学的真正延伸：iptables 也不解密 TLS，它只决定包能不能走。

---

## 决策

### 1. 总体方案：基于 uid 的出站 egress filter

引入 `_sieve` 系统用户，daemon 以该用户身份运行；防火墙规则限制「LLM endpoint:443 只有 `_sieve` uid 能出站」。

**加入 jail 后的威胁矩阵变化**：

| 防御场景 | jail 之前 | jail 之后 |
|---------|----------|---------|
| 善意 agent 配错 base_url | 静默走原路，sieve 完全看不见 | **连不出去**，agent 立刻 ETIMEDOUT，doctor 引导用户修复 |
| 第三方 npm 包偷调 LLM | 完全绕过 sieve | 同上，被防火墙打死 |
| 恶意 agent 主动绕 sieve | 没辙 | 仍然没辙（恶意进程拿 sudo 关 pf）—— 但威慑面变大；不在 PRD 威胁模型内 |
| sieve 自身 crash | 其他进程流量正常 | **fail-closed，全部 LLM 流量打不出**——是 feature 不是 bug，符合 ADR-007 |

### 2. macOS 实现（pf）

`/etc/pf.anchors/com.sieve` 草案：

```pf
# Sieve network jail - macOS pf rules
# Managed by `sieve setup --jail`; do not edit manually.

table <llm_hosts> persist file "/etc/sieve/llm_hosts.txt"

# 阻断所有进程对 LLM endpoint:443 的直连
block out proto tcp from any to <llm_hosts> port 443

# 仅允许 _sieve uid 出站 → sieve 自身的 forwarder
pass out proto tcp from any to <llm_hosts> port 443 user _sieve
```

`/etc/pf.conf` 加 anchor 引用：

```pf
anchor "com.sieve"
load anchor "com.sieve" from "/etc/pf.anchors/com.sieve"
```

`/etc/sieve/llm_hosts.txt` 格式：每行一个 hostname / CIDR。pf 的 `table` 可以混合 hostname + IP，但 hostname 解析在 pf 重载时刻发生，需要 sieve daemon 定期 resolve + reload table（见 §5 hostname 列表治理）。

### 3. Linux 实现（nftables）

```nft
table inet sieve_jail {
    set llm_hosts {
        type ipv4_addr
        flags interval
        elements = { /* 由 sieve 周期性写入 */ }
    }
    chain output {
        type filter hook output priority 0;

        # 仅允许 _sieve uid 出站 LLM endpoint:443
        ip daddr @llm_hosts tcp dport 443 meta skuid != sieve drop
    }
}
```

与 pf 实现思路完全对称，靠 uid 区分 sieve 进程与其他进程。

### 4. CLI 子命令扩展

新增三个 jail 相关子命令（`sieve-cli` 扩展，ADR-015 工具链的延伸）：

```
sieve setup --jail        # 装 jail：建 _sieve 用户 + 写 pf/nft 规则 + reload + 改 launchd 跑 _sieve
sieve doctor --jail       # 体检 jail：规则在位 + active + hostname 列表 fresh + sieve 进程身份正确
sieve uninstall --jail    # 干净回滚：拆 pf/nft 规则 + reload + 改回原 launchd 用户 + 保留 _sieve 用户
```

`sieve setup --jail` 执行流程：

1. 二次确认提示（含 sudo 一次的解释 + 风险列表）
2. 创建 `_sieve` 系统用户（macOS `dscl` / Linux `useradd --system`）
3. 改 launchd plist 把 daemon 跑在 `_sieve` 用户下；reload plist
4. 写 anchor 文件 + reload pf/nft（sudo）
5. 启动后台 hostname resolve loop（sieve daemon 内置，5min 周期 resolve + reload table）
6. 跑一次 self-check：`curl https://api.anthropic.com` 在非 sieve 用户下应当 ETIMEDOUT

**`sieve uninstall --jail` 说明**：拆 pf/nft 规则 + reload + 改回原 launchd 用户，但**保留 `_sieve` 用户**（不删，避免破坏审计回溯）。

### 5. Hostname 列表治理

`llm_hosts.txt` 内置基础列表（随 sieve binary ship，走 sigstore 签名通道，ADR-006）：

```
api.anthropic.com
api.openai.com
api.deepseek.com
generativelanguage.googleapis.com
api.mistral.ai
api.groq.com
openrouter.ai
# ... 其他 ship 时已知的 LLM provider
```

用户自定义列表 `~/.sieve/extra-hosts.txt`（用于添加中转站等非标 endpoint）；sieve 启动时合并两份，去重后写入 `/etc/sieve/llm_hosts.txt`。

**漏掉的 host 退化为「跟现状一样」**（绕过去），不会比现在更糟——这是设计上的安全降级保证，避免 hostname 列表不完整导致硬 block 合法流量。

新 host 通过常规规则更新通道下发（sieve-rules 签名机制，PRD §11.3），与规则更新同路径，不单独建分发通道。

### 6. PRD §9 #12 兼容性声明

本 ADR **不破** PRD §9 #12。这是本 ADR 通过 review 的关键论点：

| §9 #12 禁止项 | jail 是否触犯 | 说明 |
|-------------|------------|------|
| 装本地 CA | 不装 | jail 不做任何 TLS 层操作 |
| Network Extension | 不用 NE | 使用标准 pf/nftables，无需 macOS Network Extension 框架 |
| 本地 CA 注入 | 不注入 | 防火墙规则纯粹是包过滤，不涉及 PKI |
| 系统 proxy 修改 | 不改 | 不修改 HTTP_PROXY / HTTPS_PROXY / 系统代理设置 |
| MITM 解 TLS | 不解 | sieve 从未看见上游 TLS session，只过滤 egress 包 |

唯一新增的系统级改动：

- 新增 `_sieve` 系统用户（标准 daemon 实践，与 `_mysql` / `_www` 等无异）
- pf anchor 文件（独立 anchor `com.sieve`，不污染系统默认规则集）
- launchd plist 用户身份变更（daemon 化部署的标准做法）

这些都是 daemon 化部署的标准做法，与「装 CA 信任锚」的信任语义完全不同。sieve 的信任叙事「我们从不动你的 trust store」在 jail 启用后仍然 100% 成立。

### 7. 默认关、opt-in、不阻塞 GA

**jail 默认不开，GA 不带，v3.x 才上。**

`sieve setup` 默认不开 jail；必须显式 `sieve setup --jail` 才提权安装。默认关的理由：

- **安装成功率**：一次性 sudo 会让 GA 安装成功率从预期 > 80% 掉到约 60%，Phase 1 不可接受
- **hostname 列表观察期**：dogfood 阶段需要观察 hostname 列表是否覆盖足，过早开放会产生 FP（合法流量被 block）
- **差异化卖点留存**：「Sieve Pro Mode：网络层硬隔离」作为 v3 「Sieve Pro Mode」的差异化卖点（PRD §11.5），在 GA 时发布会有更高营销价值

### 8. 边界声明（不在本 ADR 范围内）

以下方向**明确不做**或推后决策：

- **DNS 层 redirect / hostname 拦截** —— DoH 绕过 / hosts 文件污染引入新攻击面，明确不做
- **HTTP（非 TLS）的 LLM endpoint 处理** —— 极少见，目前靠 port 443 拦截已覆盖绝大多数场景，暂不专门处理
- **容器化部署（Docker / Lima）jail** —— 推 v3.1 单独 ADR，本 ADR 仅覆盖 host OS 部署
- **Windows jail（WFP）** —— 推 ADR-009 一起决策（Windows 服务部署形态）

---

## 影响

### 正面影响

1. **bypass 缺口闭合**：流量层面强制收口，配错 / 偷调 / 第三方组件绕不过去，审计完整性从「尽力而为」变为「jail 开启后 100%」
2. **fail-closed 在网络层兑现**：sieve crash 时 LLM 流量整体打不出去，符合 ADR-007 fail-closed 原则在网络层的延伸——「sieve 不在了，LLM 也不通了」是产品设计意图，不是副作用
3. **审计完整性**：jail 启用后，「sieve 没记录的 LLM 流量」= 「LLM 流量不存在」——审计可以 100% 信任
4. **差异化营销卖点**：「Sieve Pro Mode：网络层硬隔离」是 Little Snitch / Lulu / 企业 DLP 难以匹敌的窄面定位；防火墙只阻断不解密，信任叙事完整
5. **PRD §9 #12 不破**：不装 CA、不 MITM、不动 trust store，信任叙事完全保留

### 负面影响

1. **一次性 sudo**：`setup --jail` / `uninstall --jail` 时刻短暂提权（写 pf anchor、改 launchd 用户）；运行期 daemon 仍以 `_sieve` 非特权用户运行
2. **Hostname 列表维护负担**：新 provider 出现时要更新 ship 列表 + 走签名分发流程；漏掉的 host 退化为「跟现状一样」（不比现在差，但失去 jail 保护）
3. **错误体验变差**：agent 配错时报 ETIMEDOUT 而不是 401/404，对用户不友好；`sieve doctor --jail` 必须能一眼诊断「是 jail 拦了还是网络本身有问题」，否则用户骂街
4. **跨平台实现成本**：pf / nftables / 未来 WFP 各一份实现；macOS + Linux 各约 1-2 天工作量，后续 Windows 再算
5. **不绝对**：恶意进程拿 sudo 仍可关 pf；jail 是威慑 + 防呆，不是绝对边界（PRD 威胁模型本来就不覆盖 sudo 级别的主动攻击）

### 需要更新的文档

- `docs/design/architecture.md` —— 加 §部署形态：Pro Mode（jail-enabled）vs Standard Mode 对比
- `docs/design/ADR-INDEX.md` —— 加入本 ADR 条目（ADR-027）
- `docs/specs/SPEC-003-sieve-setup-tool.md` —— 加 §setup --jail / §doctor --jail / §uninstall --jail 子命令规格
- `docs/api/api-reference.md` —— §CLI 子命令加 jail 相关条目
- `docs/guides/deployment.md` —— Pro Mode 部署章节
- `SECURITY.md` —— 威胁模型更新（jail 闭合的 bypass 缺口 + 仍开放的 root 级别 bypass）
- `CHANGELOG.md` —— v3.x feature 条目
- 营销文案（v3 release 时）：landing page 加 Pro Mode 卖点

---

## 相关文档

- [PRD v2.0 §1.2](../prd/sieve-prd-v2.0.md) —— 完全本地不联网
- [PRD v2.0 §9 #12](../prd/sieve-prd-v2.0.md) —— 不装本地 CA、Network Extension / CA 注入 / 系统 proxy 修改推 Phase 3 选购
- [PRD v2.0 §11.2](../prd/sieve-prd-v2.0.md) —— 信任叙事：「我们从不动你的 trust store」
- [PRD v2.0 §11.5](../prd/sieve-prd-v2.0.md) —— 营销：Pro Mode 差异化卖点
- [ADR-006](./ADR-006-sigstore-reproducible-build.md) —— sigstore + reproducible build（hostname 列表分发走签名通道）
- [ADR-007](./ADR-007-fail-closed-critical-actions.md) —— fail-closed Critical actions（jail 把 fail-closed 延伸到网络层；ADR-007 §背景第一句「YOLO mode 下的不可逆动作一旦发生无法回滚」是本 ADR 网络层 fail-closed 的精神来源）
- [ADR-009](./ADR-009-windows-service-deployment.md)（候选）—— Windows 服务部署（jail Windows WFP 实现等待 ADR-009）
- [ADR-015](./ADR-015-sieve-setup-tool.md) —— sieve setup 工具（`--jail` 子命令是 ADR-015 工具链的延伸）
- [ADR-026](./ADR-026-port-based-listener-routing.md) —— Port-based listener routing（jail 按 LLM endpoint host 切片，sieve 按 listener port 切片，两侧对齐）
