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
