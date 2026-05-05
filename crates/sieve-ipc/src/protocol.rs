use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Notify（单向 daemon → GUI，v2.0）────────────────────────────────────────

/// 状态栏通知（单向 daemon → GUI），用于 IN-SEQ-* 序列检测 + 出站脱敏 + 其他不打断的提示。
///
/// JSON-RPC 2.0 method = `"sieve.notify_status_bar"`，fire-and-forget（无 id 字段）。
///
/// 关联：PRD v2.0 §5.7（行为序列 StatusBar 通知）+ §5.4.3（GUI 接口预留）+ ADR-013。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarNotify {
    /// 全局唯一通知 ID（UUIDv7，便于追踪 + 去重）。
    pub notify_id: Uuid,
    /// 创建时间（UTC，毫秒精度 + Z 后缀，SPEC-005 §4A）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub created_at: DateTime<Utc>,
    /// 通知类型枚举。
    pub kind: NotifyKind,
    /// 简短文案（GUI 状态栏显示，< 80 字符）。
    pub title: String,
    /// 详情（GUI 点击后展开，可选）。
    #[serde(default)]
    pub detail: Option<String>,
    /// 关联规则 ID（如 IN-SEQ-01-RECON-EXFIL / OUT-01-API-KEY）。
    #[serde(default)]
    pub rule_id: Option<String>,
    /// 自动消失秒数（0 = 不自动消失，用户手动关闭）。
    pub auto_dismiss_seconds: u32,
}

/// 状态栏通知类型。
///
/// 关联：PRD v2.0 §5.7.2（SequenceHit）+ §9 #13（OutboundRedacted）+ §9 #14（UserRules*）+ SPEC-005 §5.9（HookTerminal）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyKind {
    /// 行为序列检测命中（PRD §5.7.2）。
    SequenceHit,
    /// 出站自动脱敏（OUT-01~05/12，PRD §9 #13）。
    OutboundRedacted,
    /// 用户规则文件加载失败（PRD §9 #14 fail-safe，daemon 仍正常启动）。
    UserRulesLoadFailed,
    /// 用户规则 reload 成功（sieve rules edit 后 daemon 接收到 reload 通知并成功加载）。
    UserRulesReloaded,
    /// hook 终端处置路径被触发（SPEC-005 §5.9）。
    ///
    /// daemon 在 PreToolUse hook 终端路径决策完成后推送此通知，让 GUI 状态栏
    /// 显示"hook 已处理"事件（不打断工作流，5s 自动消失）。
    HookTerminal,
    /// 其他通用提示。
    Generic,
}

/// 用户规则重新加载请求（单向 sieve rules edit 命令 → daemon）。
///
/// JSON-RPC 2.0 method = `"sieve.reload_user_rules"`，fire-and-forget（无 id 字段）。
///
/// 关联：PRD v2.0 §5.5.5（编辑器关闭后 lint + atomic backup + IPC reload）+ ADR-013。
///
/// daemon 收到后：
/// 1. 重新读取 `~/.sieve/rules/user.toml`
/// 2. lint + UserEngine::compile（fail-safe，PRD §9 #14：失败保留旧引擎）
/// 3. atomic swap LayeredEngine 内的 user 字段
/// 4. 成功 → 推一条 `NotifyKind::UserRulesReloaded` StatusBarNotify
/// 5. 失败 → 推一条 `NotifyKind::UserRulesLoadFailed`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReloadUserRules {
    /// 触发 reload 的请求 ID（追踪用，可选）。
    #[serde(default)]
    pub trigger_id: Option<Uuid>,
}

// ── Multi-agent fields (v1.5) ────────────────────────────────────────────────

/// 触发本次决策的上游 AI agent。
///
/// 关联：PRD v1.5 §6.5、ADR-019。
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
/// 关联：PRD v1.5 §4.6 场景 F、ADR-019。
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
/// 关联：ADR-014（双层防御）、SPEC-001。
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
/// Critical 规则强制使用 Block，不允许下游覆盖。关联：ADR-014 §fail-closed。
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
/// 关联：PRD §4 检测项分级、ADR-014。
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
/// 文件协议传输。关联：ADR-013 §3、SPEC-001 §3.1。
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
    /// 关联 PRD v1.5 §6.5、ADR-019。
    #[serde(default)]
    pub source_agent: SourceAgent,

    /// sub-agent 嵌套调用链。空 = 用户直接调（chain_depth=0）。
    ///
    /// 关联 PRD v1.5 §4.6、ADR-019。
    #[serde(default)]
    pub origin_chain: Vec<OriginHop>,

    /// OpenClaw 跨通道时的来源 channel（whatsapp / slack / etc）。
    ///
    /// 仅 OpenClaw 适配场景使用；其他 agent 为 None。
    /// 关联 PRD v1.5 §4.5 场景 E、IN-GEN-06。
    #[serde(default)]
    pub source_channel: Option<String>,

    /// `X-Sieve-Origin` header 中解析的真实嵌套深度（修 R7-#5）。
    ///
    /// `origin_chain` 只记录已知的 hop，中间层若无法重构则用占位符填充。
    /// 此字段直接保留 header 中的 `chain_depth` 数值，使 GUI/hook 能展示
    /// 真实嵌套层级，而不是受限于 `origin_chain.len()`。
    ///
    /// `None` 表示旧格式请求（v1.4 及以前），回退到 `origin_chain.len()`。
    /// 关联：ADR-019 §chain_depth 语义、PRD v1.5 §4.6。
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
    ///（PRD v2.0 §5.4.3）。
    ///
    /// 旧 v1.5 客户端不发此字段时，serde 默认为 `false`（保守 fail-safe：
    /// 老 GUI 即使能选 Remember 也会被 daemon 在 §5.4.2 二次校验里拒绝写入灰名单）。
    ///
    /// 关联：PRD v2.0 §5.4.2 灰名单 schema、§5.4.3 GUI 接口预留、PRD §9 #3 Critical fail-closed。
    #[serde(default)]
    pub allow_remember: bool,
}

impl DecisionRequest {
    /// 嵌套调用层数。
    ///
    /// 优先使用 `explicit_chain_depth`（来自 `X-Sieve-Origin` header 真实数值，修 R7-#5）；
    /// 旧格式请求（v1.4）回退到 `origin_chain.len()`。
    ///
    /// 0 = 用户直接调；≥2 强制 fail-closed GUI hold（ADR-019）；≥5 直接 426 拒绝。
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
/// 关联：SPEC-001 §3.3、ADR-014 §决策流程。
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
/// 关联：ADR-013 §3.4、SPEC-001 §3.3。
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
    /// 详见 PRD v2.0 §5.4.2「Critical 锁约束」三道防线。
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
    /// 关联：PRD v2.0 §5.4.2 灰名单 schema 中 context_hint 字段。
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

// ── v2.1 GUI 控制面方法（ADR-013 Supplement 2026-05-02）──────────────────────
//
// 所有方法走 JSON-RPC 2.0 over Unix socket（通道 A）。GUI → daemon 为请求/响应；
// daemon → GUI 为 fan-out notification。完整规格见 ADR-013 §S.1-S.4。

/// `sieve.set_paused` 请求参数。
///
/// `minutes ∈ [0, 60]`：0 = 立刻恢复；上限 60（防止"事实上的关闭"）。
/// Critical 锁规则不受暂停影响（[PRD v2.0 §9 #3 #8]）。
///
/// 关联：ADR-013 §S.4 / SPEC-002 §9.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPausedRequest {
    pub minutes: u32,
}

/// `sieve.set_paused` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPausedResult {
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；`paused=false` 时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// 受暂停影响的 disposition 集合（Critical 锁规则的 disposition 永不出现在此列表）。
    pub applies_to: Vec<String>,
}

/// `sieve.set_preset` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetRequest {
    /// `"strict"` | `"default"` | `"relaxed"` | `"custom"`。
    pub mode: String,
}

/// `sieve.set_preset` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub applied_at: DateTime<Utc>,
}

/// 单条 preset override（custom preset 下逐规则覆盖）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetOverride {
    pub timeout_seconds: u32,
    /// `"block"` | `"allow"` | `"redact"`。
    pub default_on_timeout: String,
}

/// `sieve.set_preset_overrides` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetPresetOverridesRequest {
    /// rule_id → override 映射。
    pub overrides: std::collections::HashMap<String, PresetOverride>,
}

/// 单条被拒绝的 override。
///
/// `reason ∈ { "critical_lock" | "unknown_rule" | "invalid_value" }`（ADR-013 §S.4）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedOverride {
    pub rule_id: String,
    pub reason: String,
}

/// `sieve.set_preset_overrides` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetOverridesResult {
    pub applied: Vec<String>,
    pub rejected: Vec<RejectedOverride>,
}

/// `sieve.reload_config` 请求参数（空对象）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReloadConfigRequest {}

/// `sieve.reload_config` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadConfigResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub reloaded_at: DateTime<Utc>,
    pub system_rules_count: u32,
    pub user_rules_count: u32,
    /// 用户规则 lint 错误清单（仅警告，不阻断）。
    pub user_rules_errors: Vec<String>,
}

/// `sieve.health` 请求参数（空对象）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthRequest {}

/// preset 快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSnapshot {
    pub mode: String,
    pub overrides: std::collections::HashMap<String, PresetOverride>,
}

/// 监听地址快照（health 子结构）。
///
/// **ADR-026 后语义变化**：daemon 可同时绑定多个端口（[`HealthResult::listeners`]
/// 数组每项一个）。本结构仍代表单个 listener，但 [`HealthResult::listen`] 字段
/// 退化为 `listeners[0]` 的别名，仅保留向后兼容（旧 GUI 客户端读取）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenSnapshot {
    pub addr: String,
    pub port: u16,
}

/// 单 listener 完整快照（ADR-026 §决策 6 + Stage F）。
///
/// 比 [`ListenSnapshot`] 多带 `provider_id` + `protocol` 元信息，新版 GUI 客户端
/// 通过 [`HealthResult::listeners`] 数组消费，列出 daemon 所有 listener 及其上游身份。
///
/// 协议版本不 bump（v2 内向后兼容扩展）：旧客户端忽略本数组，新客户端读取它。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerSnapshot {
    /// 监听地址（强制 `127.0.0.1`，PRD §9 #2）。
    pub addr: String,
    /// 监听端口（multi-listener 各自唯一）。
    pub port: u16,
    /// 上游身份标识（来自 `sieve.toml [[upstream]] provider_id`，留空时从 URL host 派生）。
    pub provider_id: String,
    /// 协议声明（`"anthropic"` | `"openai"`）。
    /// 错位请求会被 daemon fail-closed 400（ADR-026 §决策 4）。
    pub protocol: String,
}

/// audit.db 快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDbSnapshot {
    pub path: String,
    pub size_bytes: u64,
    pub schema_version: u32,
    pub events_total: u64,
    pub events_today: u64,
}

/// 规则集快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesSnapshot {
    pub system_count: u32,
    pub user_count: u32,
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub last_reload: Option<DateTime<Utc>>,
}

/// 灰名单快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistSnapshot {
    pub active_count: u32,
}

/// IPC 状态快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSnapshot {
    pub connected_clients: u32,
    pub total_decisions_inflight: u32,
}

/// `sieve.health` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub daemon_version: String,
    pub protocol_version: String,
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub preset: PresetSnapshot,
    /// 当前是否处于暂停状态（SPEC-005 §9.5）。
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；`paused=false` 时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// **已废弃，向后兼容保留**：等价于 `listeners[0]`。
    /// ADR-026 多 listener 后单一 listen 字段语义不再唯一；新客户端应读 `listeners` 数组。
    pub listen: ListenSnapshot,
    /// 多 listener 完整快照（ADR-026 §决策 6 + Stage F）。
    ///
    /// daemon 同时绑定的所有端口及其元信息（含 provider_id / protocol）。
    /// `#[serde(default)]` 保证旧 daemon 不发本字段时新客户端拿到空 vec 不崩。
    /// 旧客户端忽略本字段、读 `listen` 单值即可继续工作。
    #[serde(default)]
    pub listeners: Vec<ListenerSnapshot>,
    pub audit_db: AuditDbSnapshot,
    pub rules: RulesSnapshot,
    pub graylist: GraylistSnapshot,
    pub ipc: IpcSnapshot,
}

/// evaluate sandbox 的内容种类。
///
/// 跟 `sieve_rules::ContentKind` 对应，IPC 层独立定义避免循环依赖。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluateContentKind {
    RawText,
    ToolUseInput,
    ModelResponse,
}

/// evaluate sandbox 的方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluateDirection {
    Outbound,
    Inbound,
}

/// `sieve.evaluate` 请求参数。
///
/// payload 上限 64KB（daemon 端校验），超过返回 -32003 payload_too_large。
/// 关联：ADR-013 §S.4。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRequest {
    pub direction: EvaluateDirection,
    pub content_kind: EvaluateContentKind,
    /// 触发此次 evaluate 的上游 agent（SPEC-005 §5.7）。
    ///
    /// 旧版本发来 `"claude-code"` 字符串时，serde 无法匹配任何 SourceAgent 变体，
    /// 因为 `SourceAgent` 使用 snake_case（`"claude"` / `"open_claw"` 等）。
    /// 为保持向后兼容，字段类型维持为 `SourceAgent`；旧 GUI 应更新为 `"claude"`。
    #[serde(default)]
    pub source_agent: SourceAgent,
    pub payload: String,
}

/// 单条 evaluate 命中。
///
/// **敏感数据保护**：critical_lock 规则命中时，`matched_pattern_summary` 仅含规则类型摘要
/// （如 "BIP39 with checksum match"），daemon 端**禁止**回填 matched_canonical 或原 payload 片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateMatch {
    pub rule_id: String,
    /// `"system"` | `"user"`。
    pub rule_kind: String,
    /// `"critical"` | `"high"` | `"medium"` | `"low"`。
    pub severity: String,
    pub disposition: Disposition,
    pub matched_pattern_summary: String,
    pub fields_triggered: Vec<String>,
    /// `"allow"` | `"deny"` | `"redact_and_allow"`，daemon 模拟决策。
    pub would_decision: String,
    #[serde(default)]
    pub would_recommendation: Option<EvaluateRecommendation>,
}

/// evaluate 的 daemon 推荐结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRecommendation {
    pub decision: String,
    /// `"high"` | `"medium"` | `"low"`。
    pub confidence: String,
    pub reason: String,
}

/// `sieve.evaluate` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub evaluated_at: DateTime<Utc>,
    pub matches: Vec<EvaluateMatch>,
    /// 未命中的规则 ID 抽样（不保证完整列表）。
    #[serde(default)]
    pub no_match: Vec<String>,
}

/// `sieve.list_graylist` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListGraylistRequest {
    /// 分页大小（None = 默认 50）。
    #[serde(default)]
    pub limit: Option<u32>,
    /// 分页游标（None = 第一页）。
    #[serde(default)]
    pub cursor: Option<String>,
}

/// 灰名单条目摘要（去敏感字段）。
///
/// **隐私保护**：daemon 返回时**不**包含 `fingerprint_inputs.matched_canonical`，避免 GUI
/// 间接拿到敏感片段。完整 inputs 查看路径推 v2.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistEntrySummary {
    pub fingerprint: String,
    pub rule_id: String,
    /// `"system"` | `"user"`。
    pub rule_kind: String,
    /// unix ms。
    pub added_at: i64,
    pub added_by: String,
    #[serde(default)]
    pub context_hint: Option<String>,
    pub match_count_since: u64,
    /// unix ms（None = 永不过期）。
    #[serde(default)]
    pub expires_at: Option<i64>,
}

/// `sieve.list_graylist` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGraylistResult {
    pub entries: Vec<GraylistEntrySummary>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// `sieve.remove_graylist` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveGraylistRequest {
    pub fingerprint: String,
}

/// `sieve.remove_graylist` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveGraylistResult {
    pub removed: bool,
    pub audit_event_id: String,
}

// ── v2.0+ 兼容扩展方法 ───────────────────────────────────────────────────────

/// `sieve.list_rules` 响应（SPEC-005 §11A，Since v2.0 兼容扩展）。
///
/// 旧版本 daemon 不实现此方法，GUI 端应在收到 `-32601 method_not_found` 时降级
/// （禁用规则总览 UI，不崩溃）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesResult {
    /// 当前已加载的全部规则快照（系统规则 + 用户规则合并）。
    pub rules: Vec<RuleSummary>,
}

/// 单条规则摘要（SPEC-005 §11A `RuleSummary` 字段表）。
///
/// 11 个字段对应 SPEC 字段表，GUI 端可直接用于渲染 Detection 规则总览 Table。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    /// 规则唯一标识（如 `"IN-CR-01"`；用户规则含 `"user:"` 前缀）。
    pub rule_id: String,
    /// UI 显示标题（fallback 为 rule_id，因系统 RuleEntry 只有 description 无独立 title）。
    pub title: String,
    /// 严重等级：`"low"` / `"medium"` / `"high"` / `"critical"`。
    pub severity: String,
    /// 流量方向：`"inbound"` / `"outbound"`。
    pub direction: String,
    /// 处置形式：`"gui_popup"` / `"auto_redact"` / `"status_bar"` / `"hook_terminal"`。
    pub disposition: String,
    /// 超时后默认处置（仅 `disposition == "gui_popup"` 时有意义）。
    #[serde(default)]
    pub default_on_timeout: Option<String>,
    /// 弹窗自动确认超时秒数（仅 `disposition == "gui_popup"` 时有意义）。
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
    /// `true` 时 GUI 端禁止编辑此规则（Critical 级系统规则强制为 true）。
    pub critical_lock: bool,
    /// 规则是否启用。
    pub enabled: bool,
    /// 规则来源：`"system"` / `"user"`。
    pub rule_kind: String,
    /// 规则描述/备注，可能为 `null`。
    #[serde(default)]
    pub description: Option<String>,
}

/// `sieve.purge_history` 请求参数（SPEC-005 §11B，Since v2.0 兼容扩展）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeHistoryRequest {
    /// GUI 端 Touch ID 通过的时刻（UTC，Unix ms）；用于审计，不作为幂等 key。
    pub confirmed_at: i64,
}

/// `sieve.purge_history` 响应（SPEC-005 §11B）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeHistoryResult {
    /// daemon 实际执行删除完成的时刻（UTC，Unix ms）。
    pub purged_at: i64,
    /// 本次删除的 audit event 行数；`0` 表示历史本就为空，视为成功。
    pub rows_deleted: u64,
}

// ── v2.1 daemon → GUI notifications ──────────────────────────────────────────

/// `sieve.hello` 握手通知参数（daemon → GUI，每次连接的第一条出站消息）。
///
/// 关联：SPEC-005 §3（握手协议）。
/// GUI 收到后应校验 `protocol_version == "v2"`，不兼容时关闭连接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloParams {
    /// IPC 协议版本，当前固定为 `"v2"`。
    pub protocol_version: String,
    /// daemon 二进制版本（来自 Cargo.toml）。
    pub daemon_version: String,
    /// 当前是否处于暂停状态。
    pub paused: bool,
    /// 当前生效的 preset 名称（如 `"default"` / `"paranoid"` / `"custom"`）。
    pub preset: String,
    /// daemon 已运行秒数（启动时刻到连接时刻）。
    pub uptime_seconds: u64,
    /// audit.db 的 PRAGMA user_version（schema 版本）。
    pub audit_db_user_version: u32,
    /// daemon 本次启动时生成的唯一 UUID（整生命周期不变）。
    pub daemon_boot_id: Uuid,
}

/// `sieve.preset_changed` 通知（daemon → GUI fan-out）。
///
/// 关联：ADR-013 §S.3 / SPEC-002 §9.2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetChangedNotify {
    pub mode: String,
    /// 仅 `mode == "custom"` 时有意义；其他模式可为空 map。
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, PresetOverride>,
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub changed_at: DateTime<Utc>,
    /// `"cli"` | `"gui"` | `"config_reload"`。
    pub source: String,
    /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。
    ///
    /// GUI 触发的 mutating request → 填对应 request id；CLI/daemon 自身触发 → `None`。
    /// GUI 端可据此识别"本地回声"（自己发的 set_preset 导致的广播，无需弹窗）。
    #[serde(default)]
    pub origin_request_id: Option<Uuid>,
}

/// `sieve.paused_changed` 通知（daemon → GUI fan-out）。
///
/// `applies_to` **永远不包含** Critical 锁规则的 disposition——暂停不影响内置 Critical 拦截。
/// 关联：ADR-013 §S.3 / SPEC-002 §9.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedChangedNotify {
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；未暂停时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// `"user_request"` | `"auto_resumed"` | `"daemon_restart"`。
    pub reason: String,
    pub applies_to: Vec<String>,
    /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。
    ///
    /// GUI 触发的 mutating request → 填对应 request id；CLI/daemon 自身触发 → `None`。
    /// GUI 端可据此识别"本地回声"，避免重复展示操作确认。
    #[serde(default)]
    pub origin_request_id: Option<Uuid>,
}

/// `sieve.request_decision_canceled` 取消原因。
///
/// 关联：ADR-013 §S.3 / SPEC-002 §9.3 / §9.4。
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

// ── JSON-RPC 2.0 envelope ────────────────────────────────────────────────────

/// JSON-RPC 2.0 协议封装。
///
/// 手写实现以避免引入大型 jsonrpc crate 依赖。关联：ADR-013 §2（传输协议选型）。
pub mod jsonrpc {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// JSON-RPC 2.0 请求（通知或有 id 的调用）。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub jsonrpc: String,
        pub method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub params: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<Value>,
    }

    impl Request {
        /// 构造一个有 id 的调用请求。
        pub fn call(method: impl Into<String>, params: Value, id: Value) -> Self {
            Self {
                jsonrpc: "2.0".to_owned(),
                method: method.into(),
                params: Some(params),
                id: Some(id),
            }
        }
    }

    /// JSON-RPC 2.0 成功响应。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<ErrorObject>,
        pub id: Value,
    }

    /// JSON-RPC 2.0 错误对象。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ErrorObject {
        pub code: i64,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
    }
}
