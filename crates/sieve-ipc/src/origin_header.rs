// X-Sieve-Origin HTTP header 解析、签名验证与构造。
//
// X-Sieve-Origin header 协议定义。
//
// Header 格式：
//   无签名：`<source_agent>:<request_id>:<chain_depth>`
//   有签名：`<source_agent>:<request_id>:<chain_depth>:<base64_ed25519_sig>`
//
// 签名对象为 `<source_agent>:<request_id>:<chain_depth>` 整体字符串。
// Phase 1 GA 前签名可选；GA 后强制。

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::protocol::SourceAgent;

// ── 公钥常量 ─────────────────────────────────────────────────────────────────

/// Sieve 主代理签发 X-Sieve-Origin header 使用的 Ed25519 公钥（原始 32 字节）。
///
/// 这是 X-Sieve-Origin 验签的长期信任根：对应私钥签发
/// header，客户端用本公钥 fail-closed 验证。
pub const SIEVE_ORIGIN_PUBLIC_KEY: &[u8; 32] = &[
    0x17, 0xb8, 0xa9, 0xfd, 0x14, 0xa8, 0x50, 0x6e, 0x32, 0xa4, 0xe4, 0x48, 0x74, 0xa1, 0xf5, 0x06,
    0x81, 0x97, 0xc2, 0x99, 0xff, 0x24, 0xf5, 0xf7, 0xc2, 0xa9, 0x43, 0x56, 0xb2, 0x66, 0x67, 0x1b,
];

/// GA 编译期密钥 gate。
///
/// 启用 `ga_keys` feature（GA release build）时，若 [`SIEVE_ORIGIN_PUBLIC_KEY`]
/// 仍为全零占位，则**编译失败**——阻止 X-Sieve-Origin 验签 fail-open 进入 GA
/// 二进制。alpha build（默认无此 feature）行为不变。关联 SECURITY.md。
#[cfg(feature = "ga_keys")]
const _: () = {
    const fn is_all_zeros(key: &[u8; 32]) -> bool {
        let mut i = 0;
        while i < 32 {
            if key[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
    assert!(
        !is_all_zeros(SIEVE_ORIGIN_PUBLIC_KEY),
        "ga_keys feature enabled but SIEVE_ORIGIN_PUBLIC_KEY is all zeros — GA build must embed a real Ed25519 public key"
    );
};

// ── 错误类型 ─────────────────────────────────────────────────────────────────

/// X-Sieve-Origin header 解析 / 验证错误。
///
/// Header 格式规范。
#[derive(Debug, thiserror::Error)]
pub enum OriginHeaderError {
    /// header 值格式不合法（必须是 3 或 4 个冒号分隔字段）。
    #[error("X-Sieve-Origin format invalid: expected `<agent>:<request_id>:<depth>` got `{0}`")]
    InvalidFormat(String),

    /// `source_agent` 字段不是已知枚举值。
    #[error("X-Sieve-Origin source_agent unknown: `{0}`")]
    UnknownAgent(String),

    /// `request_id` 字段不是合法 UUID。
    #[error("X-Sieve-Origin request_id is not a valid UUID: `{0}`")]
    InvalidRequestId(String),

    /// `chain_depth` 字段不是合法 usize。
    #[error("X-Sieve-Origin chain_depth is not a number: `{0}`")]
    InvalidChainDepth(String),

    /// `chain_depth` ≥ 5，直接拒绝（攻击防御门限）。
    ///
    /// chain_depth 语义、fail-closed 防御。
    #[error("X-Sieve-Origin chain_depth too deep ({0} >= 5): nested call rejected")]
    ChainTooDeep(usize),

    /// Ed25519 签名验证失败。
    #[error("X-Sieve-Origin signature invalid (Ed25519 verify failed)")]
    SignatureInvalid,

    /// 调用了需要签名的接口，但 header 中不含签名字段。
    ///
    /// Phase 1 GA 后强制要求签名；GA 前该错误在 `parse_and_verify_origin_header` 中触发。
    #[error("X-Sieve-Origin signature missing (required after GA)")]
    SignatureMissing,
}

// ── 解析后的结构 ──────────────────────────────────────────────────────────────

/// 解析后的 X-Sieve-Origin header 字段。
///
/// Header 格式规范。
#[derive(Debug, Clone)]
pub struct OriginHeader {
    /// 触发调用链的源 agent。
    pub source_agent: SourceAgent,
    /// 调用链根请求 ID（所有嵌套层共享同一个）。
    pub request_id: uuid::Uuid,
    /// 当前嵌套层级深度（0 = 用户直接调 agent）。
    pub chain_depth: usize,
    /// Ed25519 签名原始字节（如有）。
    ///
    /// Phase 1 GA 前可选；GA 后 `parse_and_verify_origin_header` 强制要求。
    pub signature: Option<Vec<u8>>,
}

// ── source_agent 字符串映射 ───────────────────────────────────────────────────

/// 将 `source_agent` 字段字符串解析为 [`SourceAgent`] 枚举。
///
/// v1.5 第一版只支持单一 agent 编码（`-delegate-` 复合形式留 v1.6，见 SPEC-002）。
fn parse_source_agent(s: &str) -> Result<SourceAgent, OriginHeaderError> {
    match s {
        "claude" => Ok(SourceAgent::Claude),
        "open_claw" => Ok(SourceAgent::OpenClaw),
        "hermes" => Ok(SourceAgent::Hermes),
        "unknown" => Ok(SourceAgent::Unknown),
        other => Err(OriginHeaderError::UnknownAgent(other.to_owned())),
    }
}

/// 将 [`SourceAgent`] 枚举序列化为 header 字段字符串。
fn source_agent_to_str(agent: SourceAgent) -> &'static str {
    match agent {
        SourceAgent::Claude => "claude",
        SourceAgent::OpenClaw => "open_claw",
        SourceAgent::Hermes => "hermes",
        SourceAgent::Unknown => "unknown",
    }
}

// ── 核心实现 ──────────────────────────────────────────────────────────────────

/// 解析 X-Sieve-Origin header 值（不验签）。
///
/// 接受 3 字段（无签名）或 4 字段（含签名）格式：
/// - `<agent>:<request_id>:<depth>`
/// - `<agent>:<request_id>:<depth>:<base64_sig>`
///
/// Header 格式规范。
///
/// # Errors
///
/// 返回 [`OriginHeaderError`] 的对应变体：
/// - 字段数不足 → [`OriginHeaderError::InvalidFormat`]
/// - agent 不可识别 → [`OriginHeaderError::UnknownAgent`]
/// - request_id 非法 → [`OriginHeaderError::InvalidRequestId`]
/// - chain_depth 非数字 → [`OriginHeaderError::InvalidChainDepth`]
/// - chain_depth ≥ 5 → [`OriginHeaderError::ChainTooDeep`]
pub fn parse_origin_header(value: &str) -> Result<OriginHeader, OriginHeaderError> {
    // 最多分为 4 部分：agent, request_id, depth, [base64_sig]
    // 用 splitn(4, ':') 避免签名中的 base64 '=' 被误切。
    let parts: Vec<&str> = value.splitn(4, ':').collect();
    if parts.len() < 3 {
        return Err(OriginHeaderError::InvalidFormat(value.to_owned()));
    }

    let source_agent = parse_source_agent(parts[0])?;

    let request_id = uuid::Uuid::parse_str(parts[1])
        .map_err(|_| OriginHeaderError::InvalidRequestId(parts[1].to_owned()))?;

    let chain_depth: usize = parts[2]
        .parse()
        .map_err(|_| OriginHeaderError::InvalidChainDepth(parts[2].to_owned()))?;

    if chain_depth >= 5 {
        return Err(OriginHeaderError::ChainTooDeep(chain_depth));
    }

    let signature = if parts.len() == 4 {
        let bytes = B64
            .decode(parts[3])
            .map_err(|_| OriginHeaderError::SignatureInvalid)?;
        Some(bytes)
    } else {
        None
    };

    Ok(OriginHeader {
        source_agent,
        request_id,
        chain_depth,
        signature,
    })
}

/// 解析并验签 X-Sieve-Origin header。
///
/// `verifying_key` 是 Sieve 主代理的 Ed25519 公钥原始 32 字节。
/// 使用 [`SIEVE_ORIGIN_PUBLIC_KEY`] 作为默认值时，GA 前请勿在生产中调用此函数。
///
/// Phase 1 GA 前行为：签名缺失时返回 [`OriginHeaderError::SignatureMissing`]。
///
/// 签名验证。
///
/// # Errors
///
/// 在 [`parse_origin_header`] 错误基础上，额外返回：
/// - 签名缺失 → [`OriginHeaderError::SignatureMissing`]
/// - 签名验证失败 → [`OriginHeaderError::SignatureInvalid`]
pub fn parse_and_verify_origin_header(
    value: &str,
    verifying_key: &[u8; 32],
) -> Result<OriginHeader, OriginHeaderError> {
    let header = parse_origin_header(value)?;

    let sig_bytes = header
        .signature
        .as_deref()
        .ok_or(OriginHeaderError::SignatureMissing)?;

    // 构造待验签消息：`<agent>:<request_id>:<depth>`
    let message = format!(
        "{}:{}:{}",
        source_agent_to_str(header.source_agent),
        header.request_id,
        header.chain_depth
    );

    let vk =
        VerifyingKey::from_bytes(verifying_key).map_err(|_| OriginHeaderError::SignatureInvalid)?;

    let sig_array: &[u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| OriginHeaderError::SignatureInvalid)?;
    let signature = Signature::from_bytes(sig_array);

    vk.verify(message.as_bytes(), &signature)
        .map_err(|_| OriginHeaderError::SignatureInvalid)?;

    Ok(header)
}

/// 构造带签名的 X-Sieve-Origin header 值（Sieve 主代理在发起 sub-agent 请求时调用）。
///
/// 签名覆盖 `<agent>:<request_id>:<depth>` 字符串，防止攻击者伪造 header 绕过弹窗去重。
///
/// 签名验证。
pub fn build_signed_origin_header(
    source_agent: SourceAgent,
    request_id: uuid::Uuid,
    chain_depth: usize,
    signing_key: &SigningKey,
) -> String {
    let message = format!(
        "{}:{}:{}",
        source_agent_to_str(source_agent),
        request_id,
        chain_depth
    );
    let sig: Signature = signing_key.sign(message.as_bytes());
    let sig_b64 = B64.encode(sig.to_bytes());
    format!("{message}:{sig_b64}")
}

// ── 单元测试 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    use super::*;
    use crate::protocol::SourceAgent;

    const TEST_UUID: &str = "01901234-5678-7abc-def0-123456789abc";

    // 1. 解析合法 header（chain_depth=0）
    #[test]
    fn parse_valid_header_depth_zero() {
        let value = format!("claude:{TEST_UUID}:0");
        let h = parse_origin_header(&value).expect("should parse");
        assert_eq!(h.source_agent, SourceAgent::Claude);
        assert_eq!(h.request_id.to_string(), TEST_UUID);
        assert_eq!(h.chain_depth, 0);
        assert!(h.signature.is_none());
    }

    // 2. 解析合法 header（chain_depth=1）
    #[test]
    fn parse_valid_header_depth_one() {
        let value = format!("hermes:{TEST_UUID}:1");
        let h = parse_origin_header(&value).expect("should parse");
        assert_eq!(h.source_agent, SourceAgent::Hermes);
        assert_eq!(h.chain_depth, 1);
    }

    // 3. 格式错误（缺冒号）
    #[test]
    fn parse_invalid_format_missing_colon() {
        let err = parse_origin_header("claude_no_colons").unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::InvalidFormat(_)),
            "expected InvalidFormat, got: {err}"
        );
    }

    // 4. 未知 agent
    #[test]
    fn parse_unknown_agent() {
        let value = format!("xyz:{TEST_UUID}:0");
        let err = parse_origin_header(&value).unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::UnknownAgent(_)),
            "expected UnknownAgent, got: {err}"
        );
    }

    // 5. 非法 UUID
    #[test]
    fn parse_invalid_uuid() {
        let err = parse_origin_header("claude:notuuid:0").unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::InvalidRequestId(_)),
            "expected InvalidRequestId, got: {err}"
        );
    }

    // 6. chain_depth 非数字
    #[test]
    fn parse_invalid_chain_depth_not_number() {
        let value = format!("claude:{TEST_UUID}:abc");
        let err = parse_origin_header(&value).unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::InvalidChainDepth(_)),
            "expected InvalidChainDepth, got: {err}"
        );
    }

    // 7. chain_depth=5 → ChainTooDeep
    #[test]
    fn parse_chain_too_deep() {
        let value = format!("claude:{TEST_UUID}:5");
        let err = parse_origin_header(&value).unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::ChainTooDeep(5)),
            "expected ChainTooDeep(5), got: {err}"
        );
    }

    // 8. 签名 roundtrip：build → parse_and_verify 成功
    #[test]
    fn signature_roundtrip() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key: [u8; 32] = signing_key.verifying_key().to_bytes();
        let request_id = uuid::Uuid::parse_str(TEST_UUID).unwrap();

        let header_value =
            build_signed_origin_header(SourceAgent::Claude, request_id, 1, &signing_key);

        let h = parse_and_verify_origin_header(&header_value, &verifying_key)
            .expect("roundtrip should succeed");
        assert_eq!(h.source_agent, SourceAgent::Claude);
        assert_eq!(h.request_id, request_id);
        assert_eq!(h.chain_depth, 1);
        assert!(h.signature.is_some());
    }

    // 9. 签名错误（手动改 base64 后缀）
    #[test]
    fn signature_invalid() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key: [u8; 32] = signing_key.verifying_key().to_bytes();
        let request_id = uuid::Uuid::parse_str(TEST_UUID).unwrap();

        let mut header_value =
            build_signed_origin_header(SourceAgent::Hermes, request_id, 0, &signing_key);

        // 截掉最后一个字符再拼一个不同字符，使签名损坏。
        let last = header_value.pop().unwrap();
        header_value.push(if last == 'A' { 'B' } else { 'A' });

        let err = parse_and_verify_origin_header(&header_value, &verifying_key).unwrap_err();
        assert!(
            matches!(
                err,
                OriginHeaderError::SignatureInvalid | OriginHeaderError::ChainTooDeep(_)
            ),
            "expected SignatureInvalid (or parse-level error), got: {err}"
        );
    }

    // 10. 签名缺失但调用了 verify 接口 → SignatureMissing
    #[test]
    fn signature_missing_returns_error() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key: [u8; 32] = signing_key.verifying_key().to_bytes();

        // 构造一个没有签名的 header 值。
        let value = format!("claude:{TEST_UUID}:0");

        let err = parse_and_verify_origin_header(&value, &verifying_key).unwrap_err();
        assert!(
            matches!(err, OriginHeaderError::SignatureMissing),
            "expected SignatureMissing, got: {err}"
        );
    }

    // 11. 信任根已嵌入真公钥——`SIEVE_ORIGIN_PUBLIC_KEY` 非全零占位，
    // X-Sieve-Origin 验签 fail-closed 生效。守护：改回全零会让此测试红；`ga_keys`
    // 启用时文件顶部 const 断言在编译期同样保证非全零。
    #[test]
    fn origin_trust_root_is_embedded() {
        assert_ne!(
            SIEVE_ORIGIN_PUBLIC_KEY, &[0u8; 32],
            "X-Sieve-Origin trust root must embed a real key, not all-zeros placeholder"
        );
    }
}
