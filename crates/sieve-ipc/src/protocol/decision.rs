//! Decision（决策）相关 wire schema。
//!
//! 包含 DecisionRequest / DecisionResponse / DecisionAction 及配套枚举，
//! 对应 SPEC-005 §5（决策 RPC 流程）。
//!
//! **零 IO 约束**：本文件仅 import serde / chrono / uuid / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Multi-agent fields (v1.5) ────────────────────────────────────────────────

/// 触发本次决策的上游 AI agent。
///
/// 关联：X-Sieve-Origin header 协议。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAgent {
    /// Claude Code（Anthropic Messages API）
    Claude,
    /// OpenClaw（多通道消息网关，OpenAI 兼容协议为主）
    OpenClaw,
    /// Hermes Agent（multi-provider 编排器）
    Hermes,
    /// 未识别（fallback；header 缺失或格式错）
    #[default]
    Unknown,
}

/// 嵌套调用链中的一跳。
///
/// 关联：嵌套调用链场景。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginHop {
    /// 此跳的来源 agent。
    pub agent: SourceAgent,
    /// 此 hop 做了什么：user_input / delegate / skill_invoke / channel_message
    pub action: String,
    /// 此跳发生的时间（UTC，SPEC-005 §4A）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub timestamp: DateTime<Utc>,
}

// ── Enums ────────────────────────────────────────────────────────────────────

/// 检测结果的最终处置方式。
///
/// 与 sieve-rules 中的处置枚举镜像，IPC 层独立定义以避免循环依赖。
/// 关联：双层防御、SPEC-001。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// 自动脱敏——出站阶段替换敏感内容后放行，无需人工确认。
    AutoRedact,
    /// 弹出 GUI 窗口（sieve-gui-macos）请求用户确认。
    GuiPopup,
    /// 调用 PreToolUse hook（sieve-hook 二进制）在 TTY 请求用户确认。
    HookTerminal,
    /// 在状态栏静默提示，不打断流程。
    StatusBar,
}

/// 超时后的默认决策。
///
/// Critical 规则强制使用 Block，不允许下游覆盖。关联：fail-closed 双层防御。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    /// 脱敏后放行（适用于 AutoRedact 类型的超时回退）。
    Redact,
    /// 阻断——fail-closed，Critical 规则的强制回退策略。
    Block,
    /// 放行——仅适用于低优先级通知类规则。
    Allow,
}

/// 检测命中的严重等级。
///
/// 关联：检测项分级、双层防御。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// 最高级：签名、转账、部署等不可逆动作，强制人工确认，不可关闭。
    Critical,
    /// 高危：可逆但高风险操作。
    High,
    /// 中等：潜在风险，默认提示但可配置。
    Medium,
    /// 低危：信息提示。
    Low,
}

impl Severity {
    /// 严重度排序权重（越高越严重）：`Low=0 < Medium=1 < High=2 < Critical=3`。
    ///
    /// enum 声明顺序（`Critical` 在前）与严重度相反，故不能 `derive(Ord)`；
    /// 用显式权重供 `max_by_key` / 跨模块比较（决策授权门禁按 `max_severity` 判定，
    /// A 方案 headless 对 `Critical` 静默 deny）。
    pub fn rank(self) -> u8 {
        match self {
            Severity::Low => 0,
            Severity::Medium => 1,
            Severity::High => 2,
            Severity::Critical => 3,
        }
    }
}

// ── Detection payload ────────────────────────────────────────────────────────

/// 单条检测命中的 IPC 表示。
///
/// 去掉规则匹配内部细节（正则 / offset），只保留 GUI/hook 渲染所需字段。
/// 关联：SPEC-001 §3.2、SPEC-002 §2.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPayload {
    /// 规则 ID，例如 `IN-CR-01`。用于 hook 终端显示和日志关联。
    pub rule_id: String,
    /// 严重等级。
    pub severity: Severity,
    /// 处置方式。
    pub disposition: Disposition,
    /// 简短标题，在 GUI 标题栏或 hook 首行显示。
    pub title: String,
    /// 单行摘要，不超过 120 字符，用于 hook 终端和通知消息。
    pub one_line_summary: String,
    /// 扩展详情，结构由各规则自定义（GUI 侧渲染详细视图用）。
    pub details: serde_json::Value,
    /// daemon 对本 issue 的处置推荐（SPEC-005 §6.1.4）。
    ///
    /// 格式：`{ "decision": "deny"|"allow", "confidence": "high"|"medium"|"low", "reason": String }`。
    /// `None` 表示 daemon 对此 issue 无推荐（GUI 默认按钮为拒绝）。
    ///
    /// 旧版本不发此字段时 serde 默认 `None`（向后兼容）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<serde_json::Value>,
}

// ── Request / Response ───────────────────────────────────────────────────────

/// 主代理 → GUI / Hook 的决策请求。
///
/// JSON-RPC 2.0 method = `"request_decision"`，通过 Unix socket 或 pending
/// 文件协议传输。关联：SPEC-001 §3.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    /// 全局唯一请求 ID（UUIDv7，含时间戳，便于排序和 stale 检测）。
    pub request_id: Uuid,
    /// 请求创建时间（UTC，SPEC-005 §4A）。hook 侧用于 stale 检测（> 10 分钟视为过期）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub created_at: DateTime<Utc>,
    /// 用户响应超时时长（秒）。范围 30–120，由规则配置决定。
    pub timeout_seconds: u32,
    /// 超时后的默认决策。Critical 规则此字段服务端强制为 `Block`。
    pub default_on_timeout: DefaultOnTimeout,
    /// 本次请求触发的所有检测命中列表（可多条）。
    pub detections: Vec<DetectionPayload>,

    // v1.5 新增字段（serde default 保证 v1.4 旧请求依然可解析）
    /// 触发此次决策的 agent。默认 Unknown（v1.4 旧请求）。
    ///
    /// 关联：X-Sieve-Origin header 协议。
    #[serde(default)]
    pub source_agent: SourceAgent,

    /// sub-agent 嵌套调用链。空 = 用户直接调（chain_depth=0）。
    ///
    /// 关联：嵌套调用链。
    #[serde(default)]
    pub origin_chain: Vec<OriginHop>,

    /// OpenClaw 跨通道时的来源 channel（whatsapp / slack / etc）。
    ///
    /// 仅 OpenClaw 适配场景使用；其他 agent 为 None。
    /// 关联：跨通道来源场景、IN-GEN-06。
    #[serde(default)]
    pub source_channel: Option<String>,

    /// `X-Sieve-Origin` header 中解析的真实嵌套深度（修 R7-#5）。
    ///
    /// `origin_chain` 只记录已知的 hop，中间层若无法重构则用占位符填充。
    /// 此字段直接保留 header 中的 `chain_depth` 数值，使 GUI/hook 能展示
    /// 真实嵌套层级，而不是受限于 `origin_chain.len()`。
    ///
    /// `None` 表示旧格式请求（v1.4 及以前），回退到 `origin_chain.len()`。
    /// 关联：chain_depth 语义、嵌套调用链。
    #[serde(default)]
    pub explicit_chain_depth: Option<usize>,

    // v2.0 新增字段（serde default 保证 v1.5 旧客户端请求依然可解析）
    /// 用户是否被允许选择「记住此决策」（写入灰名单）。
    ///
    /// **daemon 必须根据规则计算后传入**：内置 Critical（即
    /// `sieve_rules::critical_lock::is_critical_locked` 返 true 的规则）
    /// 必须强制 `false`，不让 GUI 端有机会让用户选 Remember；
    /// 非 Critical 系统规则与用户规则可以为 `true`。
    ///
    /// GUI 端若收到 `false`，必须把 Remember checkbox disabled + 灰显，
    /// 并在 tooltip 解释"内置 Critical 规则保护核心安全场景，不允许永久绕过"
    ///（GUI 接口预留）。
    ///
    /// 旧 v1.5 客户端不发此字段时，serde 默认为 `false`（保守 fail-safe：
    /// 老 GUI 即使能选 Remember 也会被 daemon 在 §5.4.2 二次校验里拒绝写入灰名单）。
    ///
    /// 关联：灰名单 schema、GUI 接口预留、Critical fail-closed。
    #[serde(default)]
    pub allow_remember: bool,
}

impl DecisionRequest {
    /// 嵌套调用层数。
    ///
    /// 优先使用 `explicit_chain_depth`（来自 `X-Sieve-Origin` header 真实数值，修 R7-#5）；
    /// 旧格式请求（v1.4）回退到 `origin_chain.len()`。
    ///
    /// 0 = 用户直接调；≥2 强制 fail-closed GUI hold；≥5 直接 426 拒绝。
    pub fn chain_depth(&self) -> usize {
        self.explicit_chain_depth.unwrap_or(self.origin_chain.len())
    }
}

/// GUI 弹窗倒计时阶段（SPEC-005 §5.10）。
///
/// 记录用户点击决策按钮时弹窗所处的视觉紧迫阶段，便于后续审计分析
/// "用户是在蓝色冷静期还是红色最后期限点的允许"。
///
/// - `blue`：倒计时充裕（> 2/3 剩余），低紧迫感；
/// - `orange`：倒计时过半但未进入最后阶段；
/// - `red`：倒计时最后阶段（< 1/3 剩余），高紧迫感。
///
/// 当弹窗 disposition ≠ `gui_popup` 时（如自动超时决策）此字段为 `null`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPhase {
    /// 倒计时充裕阶段，低紧迫感。
    Blue,
    /// 倒计时过半阶段，中等紧迫感。
    Orange,
    /// 倒计时最后阶段，高紧迫感。
    Red,
}

/// 用户或超时产生的决策动作。
///
/// 关联：SPEC-001 §3.3、双层防御决策流程。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    /// 用户允许：GUI 类继续转发原始 SSE，Hook 类返回 exit 0。
    Allow,
    /// 用户拒绝：GUI 类截流注入 `sieve_blocked` event，Hook 类返回 exit 1。
    Deny,
    /// 仅出站脱敏类：按规则 redact 占位符替换后转发。
    RedactAndAllow,
}

/// GUI / Hook → 主代理的决策响应。
///
/// 写入 `<sieve_home>/decisions/<request_id>.json` 或通过 socket 返回。
/// 关联：SPEC-001 §3.3。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    /// 对应的请求 ID，用于主代理侧匹配 oneshot channel。
    pub request_id: Uuid,
    /// 决策动作。
    pub decision: DecisionAction,
    /// 决策时间（UTC，SPEC-005 §4A）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub decided_at: DateTime<Utc>,
    /// `true` 表示用户主动操作，`false` 表示超时默认。
    pub by_user: bool,
    /// 是否记住此次决策（同规则 + 同 tool 不再询问）。
    ///
    /// Critical severity 的决策此字段服务端强制写 `false`，即使用户请求记住也拒绝。
    ///
    /// **v2.0 校验路径**：daemon 收到 remember=true 时必须二次校验：
    /// 1. 对应 DecisionRequest 的 `allow_remember` 必须为 true
    /// 2. 命中规则必须不在 `sieve_rules::critical_lock::FAIL_CLOSED_RULES`
    ///
    /// 任一不满足 → 忽略 remember + 写 audit ERROR 事件 +（可选）GUI 状态栏告警。
    /// 详见「Critical 锁约束」三道防线。
    #[serde(default)]
    pub remember: bool,

    // v2.0 新增字段
    /// 用户在 GUI 弹窗里输入的备注（可选）。
    ///
    /// 写入灰名单 entry 的 `context_hint` 字段，便于将来用户回看时知道
    /// "我当时为啥允许这个"。daemon 不解读，仅透传 + 持久化。
    ///
    /// 旧 v1.5 客户端不发此字段时，serde 默认为 `None`。
    ///
    /// 关联：灰名单 schema 中 context_hint 字段。
    #[serde(default)]
    pub context_hint: Option<String>,

    // v2.1 新增字段
    /// 用户点击决策按钮时弹窗所处的倒计时阶段（SPEC-005 §5.10）。
    ///
    /// - `"blue"` / `"orange"` / `"red"`：三个紧迫度阶段；
    /// - `null`：disposition ≠ gui_popup 时（如超时自动决策）不适用。
    ///
    /// 旧 GUI 客户端不发此字段时，serde 默认为 `None`（向后兼容）。
    #[serde(default)]
    pub ui_phase_when_clicked: Option<UiPhase>,
}

/// 多 issue 合并请求的整体决策标签（SPEC-005 §6.2.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergedDecisionAction {
    AllDeny,
    AllAllow,
    Partial,
}

/// 多 issue 响应中的单项决策。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerIssueDecision {
    pub issue_id: String,
    pub decision: DecisionAction,
    pub remember: bool,
    #[serde(default)]
    pub context_hint: Option<String>,
}

/// GUI 对 merged request_decision 的响应（SPEC-005 §6.2.2）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedDecisionResponse {
    pub request_id: Uuid,
    pub merged_decision: MergedDecisionAction,
    pub per_issue: Vec<PerIssueDecision>,
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub decided_at: DateTime<Utc>,
    #[serde(default)]
    pub by_user: bool,
    #[serde(default)]
    pub ui_phase_when_clicked: Option<UiPhase>,
}

impl MergedDecisionResponse {
    /// 兼容当前 pipeline 的整体动作：
    /// - 全拒绝/全允许直接映射；
    /// - partial 采用 RedactAndAllow，保守脱敏所有 hold detection 后继续转发。
    pub fn into_aggregate(self) -> DecisionResponse {
        // fail-closed 聚合：任何 per_issue 被拒 → 整批 Deny，绝不让被拒项降级。旧实现把
        // Partial 一律映射 RedactAndAllow，而入站把 RedactAndAllow 当 Allow → 多 issue 入站
        // 弹窗里用户拒掉 Critical(IN-CR-*)、放行 non-critical 时，被拒的 Critical 会被静默
        // 降级并转发危险 tool call（违反核心安全不变量：Critical 决策任何情况不得降级；且是
        // 「解码失败→超时→Block→Deny」旧行为的安全回归）。
        // into_aggregate 不知方向，但「any Deny → Deny」对入站（拦截危险调用）与出站（拒绝即
        // 拦截不泄露）都 fail-closed。仅当无任何 Deny（混合 Allow + RedactAndAllow）时才保守
        // RedactAndAllow（出站脱敏转发 / 入站无危险项放行）。这也兜底 GUI 误报 merged_decision
        // 标签（如 AllAllow 却夹带 Deny per_issue）——信任更保守的 per_issue Deny。
        let any_deny = self
            .per_issue
            .iter()
            .any(|issue| issue.decision == DecisionAction::Deny);
        let decision = if any_deny {
            DecisionAction::Deny
        } else {
            match self.merged_decision {
                MergedDecisionAction::AllDeny => DecisionAction::Deny,
                MergedDecisionAction::AllAllow => DecisionAction::Allow,
                MergedDecisionAction::Partial => DecisionAction::RedactAndAllow,
            }
        };
        // 聚合为 Deny 时绝不 remember（不记住拒绝；避免含 Critical 的批次因某个 non-critical
        // 勾选 remember 而误持久化）；其余按 per_issue 的 Allow+remember 折叠。
        let remember = decision != DecisionAction::Deny
            && self
                .per_issue
                .iter()
                .any(|issue| issue.decision == DecisionAction::Allow && issue.remember);
        let context_hint = self
            .per_issue
            .iter()
            .find_map(|issue| issue.context_hint.clone());
        DecisionResponse {
            request_id: self.request_id,
            decision,
            decided_at: self.decided_at,
            by_user: self.by_user,
            remember,
            context_hint,
            ui_phase_when_clicked: self.ui_phase_when_clicked,
        }
    }
}

/// `sieve.request_decision_canceled` 取消原因。
///
/// 关联：SPEC-002 §9.3 / §9.4。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    /// daemon 侧倒计时已到。
    Timeout,
    /// 上游连接断开。
    UpstreamDisconnected,
    /// 灰名单或重复抑制策略命中后提前决策。
    DuplicateSuppressed,
    /// daemon 正在关停。
    DaemonShutdown,
    /// 多 GUI 场景下另一 GUI 已答复。
    ResolvedByPeer,
}

/// `sieve.request_decision_canceled` 通知（daemon → GUI fan-out）。
///
/// GUI 收到后必须按 SPEC-002 §9.3 处理：未弹窗则移除排队，已弹窗则关闭并显示浮层。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDecisionCanceledNotify {
    pub request_id: Uuid,
    pub reason: CancelReason,
    /// daemon 已应用的 default_on_timeout 结果。
    pub auto_decision: DecisionAction,
}
