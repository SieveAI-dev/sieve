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
    /// 创建时间（UTC）。
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
/// 关联：PRD v2.0 §5.7.2（SequenceHit）+ §9 #13（OutboundRedacted）+ §9 #14（UserRules*）。
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
    /// 此跳发生的时间（UTC）。
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
    /// 请求创建时间（UTC）。hook 侧用于 stale 检测（> 10 分钟视为过期）。
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
    /// 决策时间（UTC）。
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
    /// 暂停截止时间（UTC）；`paused=false` 时为 None。
    #[serde(default)]
    pub until: Option<DateTime<Utc>>,
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
    pub reloaded_at: DateTime<Utc>,
    pub system_rules_count: usize,
    pub user_rules_count: usize,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenSnapshot {
    pub addr: String,
    pub port: u16,
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
    pub system_count: usize,
    pub user_count: usize,
    pub last_reload: Option<DateTime<Utc>>,
}

/// 灰名单快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistSnapshot {
    pub active_count: usize,
}

/// IPC 状态快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSnapshot {
    pub connected_clients: usize,
    pub total_decisions_inflight: usize,
}

/// `sieve.health` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub daemon_version: String,
    pub protocol_version: String,
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub preset: PresetSnapshot,
    /// 暂停截止时间；未暂停时为 None。
    #[serde(default)]
    pub paused: Option<DateTime<Utc>>,
    pub listen: ListenSnapshot,
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
    /// `"claude-code"` | `"openclaw"` | `"hermes"` | `"unknown"`。
    pub source_agent: String,
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

// ── v2.1 daemon → GUI notifications ──────────────────────────────────────────

/// `sieve.preset_changed` 通知（daemon → GUI fan-out）。
///
/// 关联：ADR-013 §S.3 / SPEC-002 §9.2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetChangedNotify {
    pub mode: String,
    /// 仅 `mode == "custom"` 时有意义；其他模式可为空 map。
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, PresetOverride>,
    pub changed_at: DateTime<Utc>,
    /// `"cli"` | `"gui"` | `"config_reload"`。
    pub source: String,
}

/// `sieve.paused_changed` 通知（daemon → GUI fan-out）。
///
/// `applies_to` **永远不包含** Critical 锁规则的 disposition——暂停不影响内置 Critical 拦截。
/// 关联：ADR-013 §S.3 / SPEC-002 §9.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedChangedNotify {
    pub paused: bool,
    /// 暂停截止时间（UTC）；未暂停时为 None。
    #[serde(default)]
    pub until: Option<DateTime<Utc>>,
    /// `"user_request"` | `"auto_resumed"` | `"daemon_restart"`。
    pub reason: String,
    pub applies_to: Vec<String>,
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
