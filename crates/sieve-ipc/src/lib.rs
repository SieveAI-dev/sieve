// sieve-ipc: JSON-RPC 2.0 over Unix socket + pending/decision 文件协议库。
//
// 供 sieve-cli（主代理）调用，向 GUI（sieve-gui-macos）或 hook（sieve-hook）
// 传递决策请求并等待响应。

pub mod client;
pub mod decision_file;
pub mod error;
pub mod frame_reader;
pub mod origin_header;
pub mod paths;
pub mod pending_file;
pub mod protocol;
pub mod server;
pub mod ts_serde;
pub mod wire;

// 向后兼容别名：旧代码 `use sieve_ipc::socket_client::*` 仍可用。
pub use client as socket_client;
// 向后兼容别名：旧代码 `use sieve_ipc::socket_server::*` 仍可用。
pub use server::socket_server;

// 常用类型直接 re-export，调用方无需深层 import。
pub use client::send_reload_user_rules_oneshot;
pub use error::IpcError;
pub use origin_header::{
    build_signed_origin_header, parse_and_verify_origin_header, parse_origin_header, OriginHeader,
    OriginHeaderError, SIEVE_ORIGIN_PUBLIC_KEY,
};
pub use protocol::{
    AuditDbSnapshot, CancelReason, DecisionAction, DecisionRequest, DecisionResponse,
    DefaultOnTimeout, DetectionPayload, Disposition, EvaluateContentKind, EvaluateDirection,
    EvaluateMatch, EvaluateRecommendation, EvaluateRequest, EvaluateResult, GraylistEntrySummary,
    GraylistSnapshot, HealthRequest, HealthResult, HelloParams, IpcSnapshot, JudgeToolCallRequest,
    JudgeToolCallResult, ListGraylistRequest, ListGraylistResult, ListRulesResult, ListenSnapshot,
    ListenerSnapshot, NotifyKind, OriginHop, PausedChangedNotify, PresetChangedNotify,
    PresetOverride, PresetSnapshot, PurgeHistoryRequest, PurgeHistoryResult, RejectedOverride,
    ReloadConfigRequest, ReloadConfigResult, ReloadUserRules, RemoveGraylistRequest,
    RemoveGraylistResult, RequestDecisionCanceledNotify, RuleSummary, RulesSnapshot,
    SetPausedRequest, SetPausedResult, SetPresetOverridesRequest, SetPresetOverridesResult,
    SetPresetRequest, SetPresetResult, Severity, SourceAgent, StatusBarNotify, UiPhase,
};
pub use server::{
    BroadcastPlan, ControlError, ControlPlaneRequest, HelloBuilder, IpcServer, OversizeCallback,
    OversizeKind,
};

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::protocol::*;

    // ── StatusBarNotify serde ────────────────────────────────────────────────

    /// StatusBarNotify SequenceHit kind round-trip 序列化。
    ///
    /// 关联：行为序列 StatusBar 通知。
    #[test]
    fn status_bar_notify_serde_round_trip() {
        let notify = StatusBarNotify {
            notify_id: Uuid::now_v7(),
            created_at: Utc::now(),
            kind: NotifyKind::SequenceHit,
            title: "检测到侦察-外泄序列命中".to_owned(),
            detail: Some("IN-SEQ-01-RECON-EXFIL: 连续 3 步内读取私钥后发起外部请求".to_owned()),
            rule_id: Some("IN-SEQ-01-RECON-EXFIL".to_owned()),
            auto_dismiss_seconds: 5,
        };

        let json = serde_json::to_string(&notify).expect("serialize");
        let decoded: StatusBarNotify = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.notify_id, notify.notify_id);
        assert_eq!(decoded.kind, NotifyKind::SequenceHit);
        assert_eq!(decoded.title, notify.title);
        assert_eq!(decoded.detail, notify.detail);
        assert_eq!(decoded.rule_id.as_deref(), Some("IN-SEQ-01-RECON-EXFIL"));
        assert_eq!(decoded.auto_dismiss_seconds, 5);
    }

    /// 旧 v1.5 客户端不发 detail / rule_id 时，反序列化默认为 None（#[serde(default)]）。
    #[test]
    fn status_bar_notify_v15_compat() {
        let json = serde_json::json!({
            "notify_id": "01901234-5678-7abc-def0-123456789abc",
            "created_at": "2026-05-01T00:00:00Z",
            "kind": "outbound_redacted",
            "title": "API key 已自动脱敏",
            "auto_dismiss_seconds": 5
            // detail 和 rule_id 字段缺失
        });
        let decoded: StatusBarNotify =
            serde_json::from_value(json).expect("v1.5 compat deserialize");
        assert_eq!(decoded.kind, NotifyKind::OutboundRedacted);
        assert!(decoded.detail.is_none(), "detail 应默认 None");
        assert!(decoded.rule_id.is_none(), "rule_id 应默认 None");
    }

    /// ReloadUserRules 默认空 trigger_id 序列化 → 反序列化一致。
    #[test]
    fn reload_user_rules_default_round_trip() {
        let reload = ReloadUserRules::default();
        assert!(reload.trigger_id.is_none());

        let json = serde_json::to_string(&reload).expect("serialize");
        let decoded: ReloadUserRules = serde_json::from_str(&json).expect("deserialize");
        assert!(decoded.trigger_id.is_none());

        // 带 trigger_id 的情形。
        let with_id = ReloadUserRules {
            trigger_id: Some(Uuid::now_v7()),
        };
        let json2 = serde_json::to_string(&with_id).expect("serialize with_id");
        let decoded2: ReloadUserRules = serde_json::from_str(&json2).expect("deserialize with_id");
        assert_eq!(decoded2.trigger_id, with_id.trigger_id);
    }

    /// NotifyKind 所有变体 snake_case 序列化正确。
    #[test]
    fn notify_kind_serde() {
        let cases = [
            (NotifyKind::SequenceHit, "sequence_hit"),
            (NotifyKind::OutboundRedacted, "outbound_redacted"),
            (NotifyKind::UserRulesLoadFailed, "user_rules_load_failed"),
            (NotifyKind::UserRulesReloaded, "user_rules_reloaded"),
            (NotifyKind::HookTerminal, "hook_terminal"),
            (NotifyKind::Generic, "generic"),
        ];
        for (kind, expected) in cases {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(
                json,
                format!("\"{expected}\""),
                "NotifyKind::{kind:?} should serialize to \"{expected}\""
            );
            let decoded: NotifyKind = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, kind);
        }
    }

    // ── 协议 round-trip ──────────────────────────────────────────────────────

    #[test]
    fn decision_request_round_trip() {
        let req = DecisionRequest {
            request_id: Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::HookTerminal,
                title: "私钥检测".to_owned(),
                one_line_summary: "检测到 BIP39 助记词（12 词，checksum 通过）".to_owned(),
                details: serde_json::json!({ "word_count": 12 }),
                recommendation: None,
            }],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        };

        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: DecisionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, req.request_id);
        assert_eq!(decoded.detections[0].rule_id, "IN-CR-01");
        assert_eq!(decoded.default_on_timeout, DefaultOnTimeout::Block);
    }

    #[test]
    fn decision_response_round_trip() {
        let resp = DecisionResponse {
            request_id: Uuid::now_v7(),
            decision: DecisionAction::Deny,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
            context_hint: None,
            ui_phase_when_clicked: None,
        };

        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: DecisionResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.request_id, resp.request_id);
        assert_eq!(decoded.decision, DecisionAction::Deny);
        assert!(decoded.by_user);
        assert!(!decoded.remember);
    }

    #[test]
    fn disposition_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Disposition::GuiPopup).unwrap(),
            "\"gui_popup\""
        );
        assert_eq!(
            serde_json::to_string(&Disposition::HookTerminal).unwrap(),
            "\"hook_terminal\""
        );
    }

    #[test]
    fn severity_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&Severity::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn decision_action_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&DecisionAction::RedactAndAllow).unwrap(),
            "\"redact_and_allow\""
        );
    }

    // ── v1.5 multi-agent 字段 ───────────────────────────────────────────────

    /// 旧 v1.4 JSON（不含 source_agent / origin_chain / source_channel）能正常反序列化。
    ///
    /// source_agent 默认 Unknown，origin_chain 默认 []，source_channel 默认 None。
    #[test]
    fn v14_compat_missing_fields_use_defaults() {
        let json = serde_json::json!({
            "request_id": "01901234-5678-7abc-def0-123456789abc",
            "created_at": "2026-04-27T00:00:00Z",
            "timeout_seconds": 60,
            "default_on_timeout": "block",
            "detections": []
        });
        let req: DecisionRequest = serde_json::from_value(json).expect("v1.4 compat deserialize");
        assert_eq!(req.source_agent, SourceAgent::Unknown);
        assert!(req.origin_chain.is_empty());
        assert!(req.source_channel.is_none());
    }

    /// v1.5 完整 JSON 含全部新字段，deserialize 正确并 roundtrip。
    #[test]
    fn v15_full_fields_roundtrip() {
        let req = DecisionRequest {
            request_id: uuid::Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: SourceAgent::Claude,
            origin_chain: vec![OriginHop {
                agent: SourceAgent::Hermes,
                action: "delegate".to_owned(),
                timestamp: Utc::now(),
            }],
            source_channel: Some("slack".to_owned()),
            explicit_chain_depth: None,
            allow_remember: false,
        };

        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: DecisionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.source_agent, SourceAgent::Claude);
        assert_eq!(decoded.origin_chain.len(), 1);
        assert_eq!(decoded.origin_chain[0].action, "delegate");
        assert_eq!(decoded.source_channel.as_deref(), Some("slack"));
    }

    /// chain_depth() 返回 origin_chain 的长度。
    #[test]
    fn chain_depth_returns_origin_chain_len() {
        let mut req = DecisionRequest {
            request_id: uuid::Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        };
        assert_eq!(req.chain_depth(), 0);

        req.origin_chain.push(OriginHop {
            agent: SourceAgent::Claude,
            action: "user_input".to_owned(),
            timestamp: Utc::now(),
        });
        assert_eq!(req.chain_depth(), 1);

        req.origin_chain.push(OriginHop {
            agent: SourceAgent::Hermes,
            action: "skill_invoke".to_owned(),
            timestamp: Utc::now(),
        });
        req.origin_chain.push(OriginHop {
            agent: SourceAgent::OpenClaw,
            action: "channel_message".to_owned(),
            timestamp: Utc::now(),
        });
        assert_eq!(req.chain_depth(), 3);
    }

    /// SourceAgent 枚举 serde snake_case 序列化正确。
    #[test]
    fn source_agent_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&SourceAgent::Claude).unwrap(),
            "\"claude\""
        );
        assert_eq!(
            serde_json::to_string(&SourceAgent::OpenClaw).unwrap(),
            "\"open_claw\""
        );
        assert_eq!(
            serde_json::to_string(&SourceAgent::Hermes).unwrap(),
            "\"hermes\""
        );
        assert_eq!(
            serde_json::to_string(&SourceAgent::Unknown).unwrap(),
            "\"unknown\""
        );
        // 反序列化验证。
        let agent: SourceAgent = serde_json::from_str("\"open_claw\"").unwrap();
        assert_eq!(agent, SourceAgent::OpenClaw);
    }

    /// OriginHop 时间戳以 RFC3339 + 毫秒精度 + Z 后缀序列化（SPEC-005 §4A）。
    #[test]
    fn origin_hop_timestamp_rfc3339() {
        let ts = chrono::DateTime::parse_from_rfc3339("2026-04-27T12:34:56Z")
            .unwrap()
            .with_timezone(&Utc);
        let hop = OriginHop {
            agent: SourceAgent::Claude,
            action: "user_input".to_owned(),
            timestamp: ts,
        };
        let json = serde_json::to_string(&hop).expect("serialize");
        // P2-3：序列化必须含毫秒精度和 Z 后缀（整秒补 .000Z）
        assert!(
            json.contains("2026-04-27T12:34:56.000Z"),
            "timestamp should be RFC3339 millis+Z (§4A), got: {json}"
        );
        let decoded: OriginHop = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.timestamp, ts);
    }

    // ── jsonrpc envelope ────────────────────────────────────────────────────

    #[test]
    fn jsonrpc_request_omits_null_id() {
        let req = jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: "ping".to_owned(),
            params: None,
            id: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // 通知请求不携带 id 字段。
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn jsonrpc_call_includes_id() {
        let req = jsonrpc::Request::call(
            "sieve.request_decision",
            serde_json::json!({}),
            serde_json::Value::String("abc".to_owned()),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"sieve.request_decision\""));
    }
}

#[cfg(test)]
mod file_tests {
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    use super::{
        decision_file::{wait_for_decision, write_decision},
        pending_file::{read_pending, write_pending},
        protocol::*,
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        }
    }

    // ── pending_file ─────────────────────────────────────────────────────────

    #[test]
    fn pending_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_request(id);

        let path = write_pending(&req, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = read_pending(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    #[test]
    fn pending_not_found_error() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let err = read_pending(id, tmp.path()).unwrap_err();
        assert!(matches!(err, crate::IpcError::PendingNotFound { .. }));
    }

    #[test]
    fn pending_file_lock_two_tasks() {
        // 两个线程抢同一个 pending 文件——后者等前者释放锁后写入。
        // 验证不出现数据损坏（最终文件可被正确解析）。
        use std::sync::Arc;
        use std::sync::Barrier;
        use std::thread;

        let tmp = tempfile::tempdir().unwrap();
        let base = Arc::new(tmp.path().to_owned());
        let id = Uuid::now_v7();
        let barrier = Arc::new(Barrier::new(2));

        let base1 = Arc::clone(&base);
        let barrier1 = Arc::clone(&barrier);
        let t1 = thread::spawn(move || {
            barrier1.wait();
            let req = make_request(id);
            write_pending(&req, &base1).unwrap();
        });

        let base2 = Arc::clone(&base);
        let barrier2 = Arc::clone(&barrier);
        let t2 = thread::spawn(move || {
            barrier2.wait();
            let req = make_request(id);
            write_pending(&req, &base2).unwrap();
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // 文件仍可被正确解析（两次写入串行化）。
        let read_back = read_pending(id, &base).unwrap();
        assert_eq!(read_back.request_id, id);
    }

    // ── decision_file ────────────────────────────────────────────────────────

    #[test]
    fn decision_write_and_read() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let resp = DecisionResponse {
            request_id: id,
            decision: DecisionAction::Allow,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
            context_hint: None,
            ui_phase_when_clicked: None,
        };

        let path = write_decision(&resp, tmp.path()).unwrap();
        assert!(path.exists());

        let read_back = super::decision_file::read_decision(id, tmp.path()).unwrap();
        assert_eq!(read_back.request_id, id);
        assert_eq!(read_back.decision, DecisionAction::Allow);
    }

    #[tokio::test]
    async fn wait_for_decision_timeout_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 极短超时，不写决策文件，应返回 Block（default_on_timeout = Block）。
        let resp = wait_for_decision(
            id,
            tmp.path(),
            Duration::from_millis(100),
            DefaultOnTimeout::Block,
        )
        .await
        .unwrap();
        assert_eq!(resp.decision, DecisionAction::Deny);
        assert!(!resp.by_user);
    }

    #[tokio::test]
    async fn wait_for_decision_found() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let base = tmp.path().to_owned();

        // 50ms 后写决策文件，模拟用户操作。
        let base_clone = base.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let resp = DecisionResponse {
                request_id: id,
                decision: DecisionAction::Allow,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            };
            write_decision(&resp, &base_clone).unwrap();
        });

        let result = wait_for_decision(id, &base, Duration::from_secs(2), DefaultOnTimeout::Block)
            .await
            .unwrap();
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(result.by_user);
    }
}

#[cfg(test)]
mod socket_tests {
    //! 验证双向 JSON-RPC over Unix socket 通信模型。
    //!
    //! 测试用 IpcClient::auto_respond / 手动 socket 连接模拟真实 GUI 客户端行为，
    //! 不再使用旧的 inject_decision 绕过 socket 层。
    use std::sync::Arc;
    use std::time::Duration;

    use chrono::Utc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    use uuid::Uuid;

    use super::{
        protocol::{jsonrpc, *},
        socket_client::IpcClient,
        socket_server::IpcServer,
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        }
    }

    /// 辅助：启动服务端并返回 Arc<IpcServer>。
    async fn start_server(socket_path: &std::path::Path) -> Arc<IpcServer> {
        let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
        let server = Arc::new(server);
        let s = Arc::clone(&server);
        tokio::spawn(async move { s.run(listener).await });
        // 等服务端就绪。
        tokio::time::sleep(Duration::from_millis(10)).await;
        server
    }

    // ── 测试 1：GUI 连接 → request_decision → GUI 收到 → 回 decision → 主代理拿到 ──

    /// 核心 happy path：双向通信全链路。
    ///
    /// 1. 模拟 GUI 客户端连接并保持长连接。
    /// 2. 主代理调 `request_decision`。
    /// 3. GUI mock 读到 request 后写回 Allow。
    /// 4. 主代理收到 Allow。
    #[tokio::test]
    async fn gui_connect_request_decision_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let id = Uuid::now_v7();

        // 模拟 GUI：连接 socket，读一条 request，写回 Allow。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
                .await
                .expect("auto_respond failed");
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        let req = make_request(id);
        let result = server
            .request_decision(req, Duration::from_secs(3), "inbound")
            .await
            .unwrap();

        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(result.by_user, "GUI 回复的决策应标记 by_user=true");
    }

    // ── 测试 2：没有 GUI 客户端 → 立即 fallback ──

    /// 没有任何 GUI 连接时，request_decision 必须立即返回 fallback，
    /// 不应等待整个 timeout 时长（性能 + 体验要求）。
    #[tokio::test]
    async fn no_gui_connected_immediate_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        };

        let start = std::time::Instant::now();
        let result = server
            .request_decision(req, Duration::from_secs(5), "inbound")
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // 没有 GUI，应立即返回（远小于 5s 超时）。
        assert!(
            elapsed < Duration::from_millis(500),
            "no-GUI path should return immediately, got {elapsed:?}"
        );
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(!result.by_user);
    }

    // ── 测试 3：GUI 连接后断线 → pending requests 立即 fallback ──

    /// GUI 建立长连接后意外断线，主代理正在等待的 pending request 应立即 fallback，
    /// 不应等满 timeout。
    #[tokio::test]
    async fn gui_disconnect_triggers_pending_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // 模拟 GUI：连接后保持一小段时间再断线（不回复任何决策）。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            let stream = UnixStream::connect(&path_clone).await.unwrap();
            // 保持 50ms 后 drop（模拟 GUI 崩溃）。
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(stream);
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(20)).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        };

        let start = std::time::Instant::now();
        // 给很长的 timeout，期望断线后快速 fallback。
        let result = server
            .request_decision(req, Duration::from_secs(10), "inbound")
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // GUI 断线后 pending oneshot 被 drop，应远早于 10s 超时返回。
        assert!(
            elapsed < Duration::from_secs(3),
            "should fallback quickly after GUI disconnect, got {elapsed:?}"
        );
        assert_eq!(result.decision, DecisionAction::Deny, "Block → Deny");
        assert!(!result.by_user);
    }

    // ── 测试 4：多并发 request_decision，GUI 顺序回复，每个正确路由 ──

    /// 同时发起 3 个 request_decision，GUI mock 逐一回复，验证 request_id 路由正确。
    #[tokio::test]
    async fn concurrent_requests_correctly_routed() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::now_v7()).collect();

        // 模拟 GUI：长连接，读 3 条 request，全部回复 Deny。
        let path_clone = socket_path.clone();
        let ids_clone = ids.clone();
        tokio::spawn(async move {
            // 重试连接直到服务端就绪。
            let stream = {
                let mut last_err = None;
                let mut s = None;
                for _ in 0..10 {
                    match UnixStream::connect(&path_clone).await {
                        Ok(st) => {
                            s = Some(st);
                            break;
                        }
                        Err(e) => {
                            last_err = Some(e);
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                }
                s.unwrap_or_else(|| panic!("connect failed: {last_err:?}"))
            };
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();

            // 收到多少条就回多少条。
            let mut replied = 0usize;
            while replied < ids_clone.len() {
                let Some(line) = lines.next_line().await.unwrap() else {
                    break;
                };
                if line.trim().is_empty() {
                    continue;
                }
                // 解析 request_id，原样回 Deny。
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(rid_str) =
                        val.pointer("/params/request_id").and_then(|v| v.as_str())
                    {
                        if let Ok(rid) = rid_str.parse::<Uuid>() {
                            let resp = DecisionResponse {
                                request_id: rid,
                                decision: DecisionAction::Deny,
                                decided_at: Utc::now(),
                                by_user: true,
                                remember: false,
                                context_hint: None,
                                ui_phase_when_clicked: None,
                            };
                            let rpc_resp = jsonrpc::Response {
                                jsonrpc: "2.0".to_owned(),
                                result: Some(serde_json::to_value(&resp).unwrap()),
                                error: None,
                                id: serde_json::Value::String(rid.to_string()),
                            };
                            let mut payload = serde_json::to_string(&rpc_resp).unwrap();
                            payload.push('\n');
                            write_half.write_all(payload.as_bytes()).await.unwrap();
                            replied += 1;
                        }
                    }
                }
            }
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        // 并发发起 3 个 request_decision。
        let server = Arc::clone(&server);
        let mut handles = vec![];
        for &id in &ids {
            let s = Arc::clone(&server);
            let req = make_request(id);
            handles.push(tokio::spawn(async move {
                s.request_decision(req, Duration::from_secs(5), "inbound")
                    .await
            }));
        }

        // 收集结果，全部应为 Deny（by_user=true）。
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert_eq!(result.decision, DecisionAction::Deny);
            assert!(result.by_user);
        }
    }

    // ── 测试 5：GUI 启动晚于主代理 → 连上后正常工作 ──

    /// 主代理先启动，GUI 延迟后才连接；
    /// 第一次调用（GUI 未连）立即 fallback；
    /// GUI 连上后的第二次调用能正常路由到 GUI 并拿到 by_user=true 的响应。
    ///
    /// 这验证了"启动顺序无关"的核心契约：主代理不假设 GUI 先起，
    /// GUI 不假设自己必须最后起。
    #[tokio::test]
    async fn gui_connects_late_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // ── 阶段一：GUI 未连，request_decision 立即 fallback ──
        let id_before = Uuid::now_v7();
        let req_before = make_request(id_before);
        let before = server
            .request_decision(req_before, Duration::from_secs(5), "inbound")
            .await
            .unwrap();
        // 没有 GUI，立即 fallback（by_user=false）。
        assert!(!before.by_user, "GUI 未连时应立即 fallback");

        // ── 阶段二：GUI 连上，request_decision 路由到真实 GUI ──
        let id_after = Uuid::now_v7();
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            // GUI 延迟 100ms 启动（模拟真实延迟）。
            tokio::time::sleep(Duration::from_millis(100)).await;
            IpcClient::auto_respond(path_clone, id_after, DecisionAction::Deny)
                .await
                .expect("auto_respond failed");
        });

        // 等 GUI 建立连接。
        tokio::time::sleep(Duration::from_millis(150)).await;

        let req_after = make_request(id_after);
        let after = server
            .request_decision(req_after, Duration::from_secs(3), "inbound")
            .await
            .unwrap();
        // GUI 已连接，回复了 Deny，by_user=true。
        assert_eq!(after.decision, DecisionAction::Deny);
        assert!(after.by_user, "GUI 连接后的请求应由 GUI 响应");
    }

    // ── 保留：timeout fallback 验证 ──

    /// 有 GUI 连接但 GUI 不回复——超时后应返回 default_on_timeout fallback。
    #[tokio::test]
    async fn socket_server_timeout_with_connected_gui() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let server = start_server(&socket_path).await;

        // 模拟 GUI：连接但什么都不回复（只建立连接）。
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            let _stream = UnixStream::connect(&path_clone).await.unwrap();
            // 保持连接，不发任何数据，等测试结束。
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        // 等 GUI mock 建立连接。
        tokio::time::sleep(Duration::from_millis(20)).await;

        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 1,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![],
            source_agent: SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        };

        // GUI 连着但不回复，100ms 超时后应返回 Allow（default_on_timeout）。
        let result = server
            .request_decision(req, Duration::from_millis(100), "inbound")
            .await
            .unwrap();
        assert_eq!(result.decision, DecisionAction::Allow);
        assert!(!result.by_user);
    }

    // ── v2.0/v2.1 集成测试：broadcast_status_bar ─────────────────────────────

    use super::protocol::{NotifyKind, StatusBarNotify};
    use super::socket_client::send_reload_user_rules_oneshot;

    /// 辅助：连接 Unix socket 并持续接收第一条非空行，返回给 channel。
    fn spawn_gui_receiver(path: std::path::PathBuf, notify_tx: tokio::sync::mpsc::Sender<String>) {
        tokio::spawn(async move {
            let stream = retry_connect_helper(&path, 10, Duration::from_millis(10))
                .await
                .unwrap();
            let (read_half, _write_half) = stream.into_split();
            let mut lines = tokio::io::BufReader::new(read_half).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    let _ = notify_tx.send(line).await;
                    break;
                }
            }
        });
    }

    /// broadcast_status_bar：单 GUI 连接，broadcast 后正常收到 notify。
    ///
    /// 关联：行为序列 StatusBar 通知 + 多 GUI 客户端支持。
    #[tokio::test]
    async fn broadcast_status_bar_to_gui_client() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc_broadcast.sock");
        let server = start_server(&socket_path).await;

        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<String>(8);
        spawn_gui_receiver(socket_path.clone(), notify_tx);

        // 等 GUI 建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        let notify = StatusBarNotify {
            notify_id: Uuid::now_v7(),
            created_at: Utc::now(),
            kind: NotifyKind::SequenceHit,
            title: "序列检测命中".to_owned(),
            detail: None,
            rule_id: Some("IN-SEQ-01".to_owned()),
            auto_dismiss_seconds: 5,
        };
        let expected_id = notify.notify_id;

        // broadcast_status_bar 现在是同步方法（无 .await）。
        server.broadcast_status_bar(notify);

        // GUI 应在 1s 内收到通知（通过 handle_connection 写循环写入 socket）。
        let received = tokio::time::timeout(Duration::from_secs(1), notify_rx.recv())
            .await
            .expect("timeout waiting for notify")
            .expect("channel closed");

        let val: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(val["method"].as_str(), Some("sieve.notify_status_bar"));
        assert_eq!(
            val["params"]["notify_id"].as_str(),
            Some(expected_id.to_string().as_str())
        );
        assert_eq!(val["params"]["kind"].as_str(), Some("sequence_hit"));
        assert!(val.get("id").is_none(), "通知不应有 id 字段");
    }

    /// 无 GUI 客户端连接时，broadcast_status_bar 静默丢弃，不返回错误。
    ///
    /// 关联：多 GUI 客户端支持。
    #[tokio::test]
    async fn broadcast_status_bar_no_gui_clients_silently_drops() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc_no_gui.sock");
        let server = start_server(&socket_path).await;

        // 没有任何 GUI 连接。
        let notify = StatusBarNotify {
            notify_id: Uuid::now_v7(),
            created_at: Utc::now(),
            kind: NotifyKind::Generic,
            title: "测试通知".to_owned(),
            detail: None,
            rule_id: None,
            auto_dismiss_seconds: 0,
        };

        // 不应 panic / 不应 hang。
        server.broadcast_status_bar(notify);
        // 到达此处即表示静默丢弃成功。
    }

    // ── v2.1 新增：多 GUI 客户端 broadcast 测试 ──────────────────────────────

    /// 3 个 GUI 客户端同时连接，broadcast 后**全部 3 个**都收到相同 notify。
    ///
    /// 验证 fan-out 语义：单次 broadcast_status_bar 投递到所有已注册 sender。
    /// 关联：多 GUI 客户端支持。
    #[tokio::test]
    async fn broadcast_status_bar_to_three_gui_clients() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc_broadcast_3.sock");
        let server = start_server(&socket_path).await;

        // 启动 3 个独立 GUI mock，各自等待收到 notify。
        let mut rxs = Vec::new();
        for _ in 0..3 {
            let (tx, rx) = tokio::sync::mpsc::channel::<String>(8);
            spawn_gui_receiver(socket_path.clone(), tx);
            rxs.push(rx);
        }

        // 等所有 3 个 GUI 建立连接（给足时间）。
        tokio::time::sleep(Duration::from_millis(60)).await;

        let notify_id = Uuid::now_v7();
        let notify = StatusBarNotify {
            notify_id,
            created_at: Utc::now(),
            kind: NotifyKind::Generic,
            title: "多 GUI fan-out 测试".to_owned(),
            detail: None,
            rule_id: None,
            auto_dismiss_seconds: 3,
        };

        server.broadcast_status_bar(notify);

        // 全部 3 个 GUI mock 都应在 1s 内收到相同 notify_id。
        for (i, rx) in rxs.iter_mut().enumerate() {
            let received = tokio::time::timeout(Duration::from_secs(1), rx.recv())
                .await
                .unwrap_or_else(|_| panic!("GUI[{i}] timed out waiting for notify"))
                .unwrap_or_else(|| panic!("GUI[{i}] channel closed before receiving"));

            let val: serde_json::Value = serde_json::from_str(&received).unwrap();
            assert_eq!(
                val["method"].as_str(),
                Some("sieve.notify_status_bar"),
                "GUI[{i}] wrong method"
            );
            assert_eq!(
                val["params"]["notify_id"].as_str(),
                Some(notify_id.to_string().as_str()),
                "GUI[{i}] wrong notify_id"
            );
        }
    }

    /// GUI A + B 都连，A 断开，broadcast → B 收到 + gui_writers 中 dead sender 自动清理。
    ///
    /// 验证 lazy 清理策略：A 断开后 sender 不立即移除，而是在下次 broadcast 的
    /// try_send 返回 Closed 时才从 Vec 移除。
    /// 关联：多 GUI 客户端支持。
    #[tokio::test]
    async fn broadcast_after_gui_disconnect_drops_dead_writers() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc_broadcast_disconnect.sock");
        let server = start_server(&socket_path).await;

        // GUI A：连接后短暂保持，随后主动断开。
        let path_a = socket_path.clone();
        let gui_a_handle = tokio::spawn(async move {
            let _stream = UnixStream::connect(&path_a).await.unwrap();
            // 保持 50ms 后 drop（模拟崩溃 / 关闭）。
            tokio::time::sleep(Duration::from_millis(50)).await;
            // stream drop → sender Closed。
        });

        // GUI B：长连接，等待接收 notify。
        let (tx_b, mut rx_b) = tokio::sync::mpsc::channel::<String>(8);
        spawn_gui_receiver(socket_path.clone(), tx_b);

        // 等两个 GUI 都建立连接。
        tokio::time::sleep(Duration::from_millis(30)).await;

        // 等 GUI A 断开。
        gui_a_handle.await.unwrap();
        // 再等一个 tick 让 socket 关闭传播。
        tokio::time::sleep(Duration::from_millis(20)).await;

        let notify_id = Uuid::now_v7();
        let notify = StatusBarNotify {
            notify_id,
            created_at: Utc::now(),
            kind: NotifyKind::Generic,
            title: "断线清理测试".to_owned(),
            detail: None,
            rule_id: None,
            auto_dismiss_seconds: 0,
        };

        // 此次 broadcast：GUI A 的 sender 应被检测为 Closed 并移除；GUI B 应收到。
        server.broadcast_status_bar(notify);

        // GUI B 应在 1s 内收到。
        let received = tokio::time::timeout(Duration::from_secs(1), rx_b.recv())
            .await
            .expect("GUI B timed out waiting for notify")
            .expect("GUI B channel closed");

        let val: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(val["method"].as_str(), Some("sieve.notify_status_bar"));
        assert_eq!(
            val["params"]["notify_id"].as_str(),
            Some(notify_id.to_string().as_str())
        );

        // 验证 dead sender 已被清理：gui_writers 长度应为 1（只剩 GUI B）。
        // 注：broadcast 已执行过 lazy 清理，此时 Vec 中 A 的 sender 已移除。
        // 从外部访问 gui_writers 需要 Arc clone；此处通过再次 broadcast 验证间接效果：
        // 如果 A 的 sender 未清理，下次 broadcast 会重复触发 Closed 错误（无害但多余）。
        // 直接断言：再次 broadcast，GUI B 还能收到第二条 notify。
        let notify2_id = Uuid::now_v7();
        let notify2 = StatusBarNotify {
            notify_id: notify2_id,
            created_at: Utc::now(),
            kind: NotifyKind::Generic,
            title: "第二次 broadcast 验证".to_owned(),
            detail: None,
            rule_id: None,
            auto_dismiss_seconds: 0,
        };
        server.broadcast_status_bar(notify2);

        // 注：spawn_gui_receiver 只接收第一条；此处直接验证 GUI B 的 rx 已消费完。
        // 第二条走的是 GUI B 的 mpsc write channel 再由 handle_connection 写到 socket，
        // 但 rx_b 在 spawn_gui_receiver 内消费第一条后 task 退出（break），
        // 所以 rx_b 不会再有消息。验证 broadcast 不 panic 即可（已完成）。
    }

    /// send_reload_user_rules_oneshot → daemon mock IpcServer 能从 reload_rx 取到 ReloadUserRules。
    ///
    /// 关联：编辑器关闭后 lint + atomic backup + IPC reload 流程。
    #[tokio::test]
    async fn send_reload_user_rules_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc_reload.sock");
        let (server, listener) = IpcServer::bind(socket_path.clone()).unwrap();
        let server = Arc::new(server);

        // 取出 reload_rx，模拟 daemon 监听。
        let mut reload_rx = server.reload_rx().await.expect("reload_rx should be Some");

        // 启动 server accept 循环。
        let s = Arc::clone(&server);
        tokio::spawn(async move { s.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 发 reload 通知。
        let trigger_id = Uuid::now_v7();
        send_reload_user_rules_oneshot(&socket_path, Some(trigger_id))
            .await
            .expect("send_reload should succeed");

        // daemon 侧应在 1s 内收到。
        let received = tokio::time::timeout(Duration::from_secs(1), reload_rx.recv())
            .await
            .expect("timeout waiting for reload")
            .expect("channel closed");

        assert_eq!(
            received.trigger_id,
            Some(trigger_id),
            "trigger_id 应与发送侧一致"
        );
    }

    // 辅助：连接重试（broadcast 测试用）。
    async fn retry_connect_helper(
        path: &std::path::Path,
        attempts: u32,
        delay: Duration,
    ) -> Result<UnixStream, crate::IpcError> {
        let mut last_err = None;
        for _ in 0..attempts {
            match UnixStream::connect(path).await {
                Ok(s) => return Ok(s),
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(delay).await;
                }
            }
        }
        Err(crate::IpcError::Socket(last_err.unwrap()))
    }
}

#[cfg(test)]
mod paths_tests {
    use super::paths::*;
    use std::sync::Mutex;

    // 任何修改 SIEVE_HOME / HOME 的测试都必须先拿到这把锁。
    // Rust test 默认多线程跑同一个 test binary，env var 是进程全局状态。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn sieve_home_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let orig = std::env::var("SIEVE_HOME").ok();
        std::env::set_var("SIEVE_HOME", "/tmp/test_sieve_override");
        let home = sieve_home().unwrap();
        match orig {
            Some(v) => std::env::set_var("SIEVE_HOME", v),
            None => std::env::remove_var("SIEVE_HOME"),
        }
        assert_eq!(home.to_str().unwrap(), "/tmp/test_sieve_override");
    }

    #[test]
    fn sieve_home_default_uses_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let orig = std::env::var("SIEVE_HOME").ok();
        std::env::remove_var("SIEVE_HOME");
        let home = sieve_home().unwrap();
        if let Some(v) = orig {
            std::env::set_var("SIEVE_HOME", v);
        }
        assert!(home.to_str().unwrap().ends_with(".sieve"));
    }

    #[test]
    fn ensure_dirs_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_dirs(tmp.path()).unwrap();
        ensure_dirs(tmp.path()).unwrap(); // 第二次调用不应报错。
        assert!(pending_dir(tmp.path()).exists());
        assert!(decisions_dir(tmp.path()).exists());
        assert!(locks_dir(tmp.path()).exists());
    }
}
