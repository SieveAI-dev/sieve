//! GUI peer 代码签名核验（F1-b，SPEC-005 §6.2.4）。
//!
//! macOS：经 `getsockopt(LOCAL_PEERTOKEN)` 取 Unix socket 对端进程的 audit token，
//! 交给 Security framework（`SecCodeCopyGuestWithAttributes` → `SecCodeCheckValidity`）
//! 核验对端是否满足配置的代码签名 requirement（`gui_peer_code_requirement`）。
//! 非 macOS：无核验能力，构造出的回调恒拒（fail-closed）。
//!
//! 该核验回答的是「连接对端是不是签名过的 Sieve GUI 二进制」。TouchID 在 wire 上
//! 零防伪价值（`LocalAuthentication` 只返回本地 bool，不签发可传递凭证），peer
//! 代码签名核验才是 GUI 应答通道防冒充的真防线。已知残余风险：挡不住 agent 调用
//! 合法签名的 `sieve` CLI 二进制——那条路径由 `resolve_decision` 的 A 方案门禁负责
//! （Critical 不开口子），两条防线合并见 SPEC-005 §11E 与 §6.2.4。

use std::sync::Arc;

/// 按配置构造注入给 `IpcServer` 的 peer 核验回调。
///
/// - `requirement`：macOS SecRequirement 语法，例如
///   `identifier "com.sieve.gui" and anchor apple generic and certificate leaf[subject.OU] = "TEAMID"`。
/// - 回调对每个连接懒执行一次（由 `sieve-ipc` 的 `PeerGate` 缓存）；核验失败/取 token
///   失败/平台不支持一律返回 `false`（fail-closed，Critical allow 会被静默改写 deny）。
///
/// # debug 构建测试信任缝（编译期剔除）
///
/// 仅 `cfg(debug_assertions)` 下识别环境变量 `SIEVE_TEST_GUI_PEER_VERDICT`
/// （`allow` / `deny`），用于集成测试模拟核验结果——测试环境造不出 Developer ID
/// 签名的 GUI 进程，正向路径只能注入。release 构建不含此代码路径，无运行时开关。
pub fn build_verifier(requirement: String) -> sieve_ipc::PeerVerifier {
    Arc::new(move |fd| {
        #[cfg(debug_assertions)]
        if let Ok(verdict) = std::env::var("SIEVE_TEST_GUI_PEER_VERDICT") {
            tracing::warn!(
                verdict,
                "SIEVE_TEST_GUI_PEER_VERDICT 生效（仅 debug 构建的测试信任缝）"
            );
            return verdict == "allow";
        }

        match verify_peer(fd, &requirement) {
            Ok(()) => {
                tracing::info!(fd, "GUI peer 代码签名核验通过");
                true
            }
            Err(e) => {
                tracing::warn!(fd, error = %e, "GUI peer 代码签名核验未通过（fail-closed）");
                false
            }
        }
    })
}

/// 核验失败原因。
#[derive(Debug)]
pub enum PeerVerifyError {
    /// 取对端 audit token 失败（getsockopt 错误）。
    PeerToken(std::io::Error),
    /// requirement 字符串不合法。
    BadRequirement(String),
    /// 对端进程不满足 requirement（决定性否定）。
    Rejected(String),
    /// 当前平台不支持（非 macOS）。
    #[cfg(not(target_os = "macos"))]
    Unsupported,
}

impl std::fmt::Display for PeerVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PeerToken(e) => write!(f, "getsockopt(LOCAL_PEERTOKEN) failed: {e}"),
            Self::BadRequirement(e) => write!(f, "invalid code requirement: {e}"),
            Self::Rejected(e) => write!(f, "peer failed code-signing check: {e}"),
            #[cfg(not(target_os = "macos"))]
            Self::Unsupported => write!(
                f,
                "peer code-signing verification unsupported on this platform"
            ),
        }
    }
}

/// 对连接 fd 的对端进程执行代码签名核验。
#[cfg(target_os = "macos")]
pub fn verify_peer(fd: std::os::unix::io::RawFd, requirement: &str) -> Result<(), PeerVerifyError> {
    macos::verify(fd, requirement)
}

#[cfg(not(target_os = "macos"))]
pub fn verify_peer(
    _fd: std::os::unix::io::RawFd,
    _requirement: &str,
) -> Result<(), PeerVerifyError> {
    Err(PeerVerifyError::Unsupported)
}

#[cfg(target_os = "macos")]
mod macos {
    #![allow(unsafe_code)] // getsockopt FFI；与 process_context.rs 同口径

    use super::PeerVerifyError;
    use security_framework::os::macos::code_signing::{
        Flags, GuestAttributes, SecCode, SecRequirement,
    };
    use std::os::unix::io::RawFd;

    /// `sys/un.h`：level `SOL_LOCAL`；libc crate 未导出，按系统头文件定义。
    const SOL_LOCAL: libc::c_int = 0;
    /// `sys/un.h`：`LOCAL_PEERTOKEN` —— 取对端进程 audit token（含 pid + pidversion，
    /// 比 `LOCAL_PEERPID` 抗 pid 复用竞态）。
    const LOCAL_PEERTOKEN: libc::c_int = 0x006;

    /// `mach/audit.h` 的 `audit_token_t`：8 × u32（32 字节）。
    type AuditToken = [u32; 8];

    fn peer_audit_token(fd: RawFd) -> Result<AuditToken, PeerVerifyError> {
        let mut token: AuditToken = [0; 8];
        let mut len = std::mem::size_of::<AuditToken>() as libc::socklen_t;
        // SAFETY：fd 由调用方保证在连接存活期内有效；token 是符合 C 布局的定长栈缓冲，
        // len 初始化为缓冲区大小，内核只在 len 范围内写入。
        let rc = unsafe {
            libc::getsockopt(
                fd,
                SOL_LOCAL,
                LOCAL_PEERTOKEN,
                token.as_mut_ptr().cast::<libc::c_void>(),
                &mut len,
            )
        };
        if rc != 0 {
            return Err(PeerVerifyError::PeerToken(std::io::Error::last_os_error()));
        }
        if len as usize != std::mem::size_of::<AuditToken>() {
            return Err(PeerVerifyError::PeerToken(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unexpected audit token length: {len}"),
            )));
        }
        Ok(token)
    }

    pub fn verify(fd: RawFd, requirement: &str) -> Result<(), PeerVerifyError> {
        let token = peer_audit_token(fd)?;

        let req: SecRequirement = requirement
            .parse()
            .map_err(|e| PeerVerifyError::BadRequirement(format!("{e}")))?;

        // audit_token_t → 字节序列（native endian，与内核写入布局一致）
        let mut token_bytes = [0u8; 32];
        for (i, word) in token.iter().enumerate() {
            token_bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_ne_bytes());
        }
        // token_data 活到 copy_guest 调用之后（CFMutableDictionary 标准回调亦会 retain）
        let token_data = core_foundation::data::CFData::from_buffer(&token_bytes);
        let mut attrs = GuestAttributes::new();
        {
            use core_foundation::base::TCFType;
            attrs.set_audit_token(token_data.as_concrete_TypeRef());
        }

        let code = SecCode::copy_guest_with_attribues(None, &attrs, Flags::NONE)
            .map_err(|e| PeerVerifyError::Rejected(format!("copy guest: {e}")))?;
        code.check_validity(Flags::NONE, &req)
            .map_err(|e| PeerVerifyError::Rejected(format!("check validity: {e}")))?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// 决定性负例（真 FFI）：本测试进程经 socketpair 充当对端——cargo test 二进制
        /// 只有 ad-hoc 签名，绝不满足 Sieve GUI requirement → 必须 Rejected。
        #[test]
        fn adhoc_test_process_fails_gui_requirement() {
            let mut fds = [0; 2];
            // SAFETY：socketpair 输出两个已连接的 UDS fd，写入定长数组。
            let rc =
                unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
            assert_eq!(rc, 0);
            let requirement = r#"identifier "com.sieve.gui" and anchor apple generic"#;
            let result = verify(fds[0], requirement);
            // SAFETY：关闭本测试打开的 fd。
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
            match result {
                Err(PeerVerifyError::Rejected(_)) => {}
                other => panic!("期望 Rejected，实际 {other:?}"),
            }
        }

        /// audit token 可取（socketpair 对端 = 本进程）。
        #[test]
        fn peer_audit_token_readable_via_socketpair() {
            let mut fds = [0; 2];
            // SAFETY：同上。
            let rc =
                unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
            assert_eq!(rc, 0);
            let token = peer_audit_token(fds[0]);
            // SAFETY：关闭本测试打开的 fd。
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
            let token = token.expect("socketpair 对端 audit token 应可取");
            // audit_token_t 第 6 槽（index 5）为 pid
            assert_eq!(token[5], std::process::id());
        }

        /// requirement 语法错误 → BadRequirement（不 panic）。
        #[test]
        fn malformed_requirement_reports_bad_requirement() {
            let mut fds = [0; 2];
            // SAFETY：同上。
            let rc =
                unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
            assert_eq!(rc, 0);
            let result = verify(fds[0], "this is (((not a requirement");
            // SAFETY：关闭本测试打开的 fd。
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
            match result {
                Err(PeerVerifyError::BadRequirement(_)) => {}
                other => panic!("期望 BadRequirement，实际 {other:?}"),
            }
        }
    }
}

/// 红队决定性用例（F1-b 验收条件）+ 测试信任缝行为。
///
/// 全链路真核验：本测试进程（cargo test 二进制，ad-hoc 签名）连 socket 冒充 GUI
/// 批 Critical——getsockopt(LOCAL_PEERTOKEN) 与 SecCode 核验全部真跑，不经任何 seam。
#[cfg(all(test, target_os = "macos"))]
mod redteam_tests {
    use super::build_verifier;
    use chrono::Utc;
    use sieve_ipc::{
        socket_client::IpcClient, socket_server::IpcServer, DecisionAction, DecisionRequest,
        DefaultOnTimeout, DetectionPayload, Disposition, Severity, SourceAgent,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    fn critical_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-05".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::GuiPopup,
                title: "签名工具调用".to_owned(),
                one_line_summary: "检测到签名工具调用".to_owned(),
                details: serde_json::json!({}),
                recommendation: None,
            }],
            source_agent: SourceAgent::Claude,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        }
    }

    /// 决定性负例：非 GUI 本机进程连 socket 尝试批准 Critical → 必须失败（落地为 deny）。
    #[serial_test::serial]
    #[tokio::test]
    async fn impostor_local_process_cannot_approve_critical() {
        assert!(
            std::env::var("SIEVE_TEST_GUI_PEER_VERDICT").is_err(),
            "决定性用例必须走真核验，不允许 seam 生效"
        );

        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let (server, listener) = IpcServer::bind(socket_path.clone()).unwrap();
        let server = Arc::new(server);
        // 真 verifier：requirement 指向签名的 Sieve GUI——本测试进程绝不满足。
        server.set_peer_verifier(build_verifier(
            r#"identifier "com.sieve.gui" and anchor apple generic"#.to_owned(),
        ));
        let s = Arc::clone(&server);
        tokio::spawn(async move { s.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 冒充 GUI：连接 + 对 Critical 请求回 Allow。
        let id = Uuid::now_v7();
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
                .await
                .expect("auto_respond failed");
        });
        tokio::time::sleep(Duration::from_millis(30)).await;

        let result = server
            .request_decision(
                critical_request(id),
                Duration::from_secs(3),
                "inbound",
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            result.decision,
            DecisionAction::Deny,
            "红队决定性用例：冒充进程的 Critical allow 必须被真核验拦下改写为 deny"
        );
    }

    /// 测试信任缝（仅 debug 构建）：SIEVE_TEST_GUI_PEER_VERDICT=allow 模拟核验通过。
    /// release 构建不含该代码路径（cfg(debug_assertions) 编译期剔除）。
    #[serial_test::serial]
    #[tokio::test]
    async fn debug_seam_allow_verdict_lets_critical_pass() {
        std::env::set_var("SIEVE_TEST_GUI_PEER_VERDICT", "allow");

        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        let (server, listener) = IpcServer::bind(socket_path.clone()).unwrap();
        let server = Arc::new(server);
        server.set_peer_verifier(build_verifier(
            r#"identifier "com.sieve.gui" and anchor apple generic"#.to_owned(),
        ));
        let s = Arc::clone(&server);
        tokio::spawn(async move { s.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let path_clone = socket_path.clone();
        tokio::spawn(async move {
            IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
                .await
                .expect("auto_respond failed");
        });
        tokio::time::sleep(Duration::from_millis(30)).await;

        let result = server
            .request_decision(
                critical_request(id),
                Duration::from_secs(3),
                "inbound",
                None,
            )
            .await
            .unwrap();
        std::env::remove_var("SIEVE_TEST_GUI_PEER_VERDICT");

        assert_eq!(
            result.decision,
            DecisionAction::Allow,
            "debug seam 模拟核验通过时 Critical allow 应放行（正向路径接线验证）"
        );
    }
}
