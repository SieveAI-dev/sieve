//! OUT-01~12 出站规则集成测试。
//!
//! 关联 PRD §5.1 出站检测目标。
//!
//! 每条规则覆盖:
//!   - `positive` case: 满足 pattern 结构的 fake key，引擎**应**命中。
//!   - `negative` case: 明显不满足 pattern 的输入（过短 / 前缀错误 / 不含必要锚点），引擎**不应**命中。
//!
//! OUT-12 BIP39 占位 pattern 被过滤掉，不在本测试验证范围；
//! BIP39 的 SHA-256 checksum + 词数验证由 `sieve_rules::bip39` 单独测试。
//!
//! **依赖**: `rules-engine-agent` 实现 `sieve_rules::engine::VectorscanEngine`
//! 及 `sieve_rules::loader::load_outbound_rules`。
//! 在对应实现合并前本文件不参与 `cargo test`（编译不通过）。
//!
//! 验证命令（实现合并后）:
//! ```bash
//! cargo test -p sieve-rules --test outbound_rules --locked
//! ```

use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::loader::load_outbound_rules;
use std::path::PathBuf;

/// 返回 outbound.toml 的绝对路径（相对于 crate manifest 目录）。
fn rules_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("rules");
    p.push("outbound.toml");
    p
}

/// 加载并编译规则集，过滤掉 OUT-12 BIP39 占位 pattern。
///
/// OUT-12 的 pattern 为 `__BIP39_PREFILTER_PLACEHOLDER__`，运行时由
/// `sieve_rules::bip39::bip39_pattern_from_wordlist()` 动态替换，
/// 此处测试不验证该规则。
fn build_engine() -> VectorscanEngine {
    let rules = load_outbound_rules(&rules_path()).expect("load outbound.toml failed");
    let filtered: Vec<_> = rules
        .into_iter()
        .filter(|r| r.pattern != "__BIP39_PREFILTER_PLACEHOLDER__")
        .collect();
    VectorscanEngine::compile(filtered).expect("VectorscanEngine compile failed")
}

/// 断言 `rule_id` 在 `text` 中**应**命中。
fn assert_hit(engine: &VectorscanEngine, rule_id: &str, text: &str) {
    let hits = engine.scan(text.as_bytes()).expect("scan failed");
    let matched: Vec<_> = hits.iter().filter(|h| h.rule_id == rule_id).collect();
    assert!(
        !matched.is_empty(),
        "rule {rule_id} should match, but got no hit.\ninput: {text}"
    );
}

/// 断言 `rule_id` 在 `text` 中**不应**命中。
fn assert_no_hit(engine: &VectorscanEngine, rule_id: &str, text: &str) {
    let hits = engine.scan(text.as_bytes()).expect("scan failed");
    let matched: Vec<_> = hits.iter().filter(|h| h.rule_id == rule_id).collect();
    assert!(
        matched.is_empty(),
        "rule {rule_id} should NOT match, but got {} hit(s).\ninput: {text}",
        matched.len()
    );
}

// ---------------------------------------------------------------------------
// OUT-01: Anthropic API Key
// ---------------------------------------------------------------------------
#[test]
fn out_01_anthropic_key() {
    let e = build_engine();

    // positive: 前缀 sk-ant-api03- + 93 个合法字符 + "AA" 结尾（共 108 字符）
    // 用 "abcdefghij" 重复 10 次,take 93 保证够长
    let body: String = "abcdefghij".repeat(10).chars().take(93).collect();
    let key = format!("sk-ant-api03-{body}AA");
    assert_eq!(key.len(), "sk-ant-api03-".len() + 93 + 2);
    assert_hit(&e, "OUT-01", &key);

    // negative: 前缀正确但长度远不足（只有 5 个字符 + AA）
    assert_no_hit(&e, "OUT-01", "sk-ant-api03-xxxxxAA");

    // negative: 前缀完全不同
    assert_no_hit(&e, "OUT-01", "sk-other-key-12345");
}

// ---------------------------------------------------------------------------
// OUT-02: OpenAI API Key — 旧格式（sk- + 20 alnum + T3BlbkFJ + 20 alnum）
// ---------------------------------------------------------------------------
#[test]
fn out_02_openai_key_legacy() {
    let e = build_engine();

    // positive: 旧格式 — sk- + 20 + T3BlbkFJ + 20
    let key = "sk-aBcDeFgHiJkLmNoPqRsTT3BlbkFJaBcDeFgHiJkLmNoPqRsT";
    assert_eq!(key.len(), "sk-".len() + 20 + "T3BlbkFJ".len() + 20);
    assert_hit(&e, "OUT-02", key);

    // negative: 缺少 T3BlbkFJ 魔术串
    assert_no_hit(&e, "OUT-02", "sk-aBcDeFgHiJkLmNoPqRsTaBcDeFgHiJkLmNoPqRsT");
}

// ---------------------------------------------------------------------------
// OUT-02 新格式（sk-proj- + 58 + T3BlbkFJ + 58）
// ---------------------------------------------------------------------------
#[test]
fn out_02_openai_key_proj_format() {
    let e = build_engine();

    // positive: 新格式 sk-proj-
    let long_body: String = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ012345"
        .chars()
        .take(58)
        .collect();
    let key = format!("sk-proj-{long_body}T3BlbkFJ{long_body}");
    assert_hit(&e, "OUT-02", &key);

    // negative: 前缀不在允许列表中
    assert_no_hit(
        &e,
        "OUT-02",
        &format!("sk-foo-{long_body}T3BlbkFJ{long_body}"),
    );
}

// ---------------------------------------------------------------------------
// OUT-03: AWS Access Key ID
// ---------------------------------------------------------------------------
#[test]
fn out_03_aws_access_key() {
    let e = build_engine();

    // positive AKIA: 16 个 base32 uppercase
    assert_hit(&e, "OUT-03", "AKIAIOSFODNN7AAAAAAA");

    // positive ASIA (临时凭证前缀)
    // pattern 后缀是 [A-Z2-7]{16},不能用 0/1/8/9
    assert_hit(&e, "OUT-03", "ASIAABCDEFGHIJKL2345");

    // positive ABIA
    assert_hit(&e, "OUT-03", "ABIAABCDEFGHIJ234567");

    // positive ACCA
    assert_hit(&e, "OUT-03", "ACCAABCDEFGHIJ234567");

    // negative: 小写前缀，vectorscan 大小写敏感（无 (?i) 标志）
    assert_no_hit(&e, "OUT-03", "akiaiosfodnn7aaaaaaa");

    // negative: 长度不足（只有 10 个字符在前缀后）
    assert_no_hit(&e, "OUT-03", "AKIASHORT1234");
}

// ---------------------------------------------------------------------------
// OUT-04: GitHub PAT
// ---------------------------------------------------------------------------
#[test]
fn out_04_github_pat() {
    let e = build_engine();

    // positive ghp_ (GitHub Personal Access Token)
    assert_hit(&e, "OUT-04", "ghp_abcdefghijklmnopqrstuvwxyz0123456789");

    // positive ghs_ (GitHub Apps token)
    assert_hit(&e, "OUT-04", "ghs_abcdefghijklmnopqrstuvwxyz0123456789");

    // positive gho_ (OAuth token)
    assert_hit(&e, "OUT-04", "gho_abcdefghijklmnopqrstuvwxyz0123456789");

    // positive ghu_ (user-to-server token)
    assert_hit(&e, "OUT-04", "ghu_abcdefghijklmnopqrstuvwxyz0123456789");

    // positive ghr_ (refresh token)
    assert_hit(&e, "OUT-04", "ghr_abcdefghijklmnopqrstuvwxyz0123456789");

    // negative: 前缀不符合 gh[pousr]_ 格式
    assert_no_hit(&e, "OUT-04", "ghx_abcdefghijklmnopqrstuvwxyz0123456789");

    // negative: 长度不足（只有 10 个字符）
    assert_no_hit(&e, "OUT-04", "ghp_shortkey");
}

// ---------------------------------------------------------------------------
// OUT-05: Google Cloud API Key
// ---------------------------------------------------------------------------
#[test]
fn out_05_gcp_api_key() {
    let e = build_engine();

    // positive: AIza + 35 个合法字符
    assert_hit(&e, "OUT-05", "AIzaSyABCDEFghijklmnopqrstuvwxyz01234567");

    // negative: 前缀不是 AIza
    assert_no_hit(&e, "OUT-05", "AIxx1234567890abcdefghijklmnopqrstuv");

    // negative: 长度不足（AIza + 10）
    assert_no_hit(&e, "OUT-05", "AIzaShortKey");
}

// ---------------------------------------------------------------------------
// OUT-06: JWT Token
// ---------------------------------------------------------------------------
#[test]
fn out_06_jwt() {
    let e = build_engine();

    // positive: 标准 JWT（header.payload.signature 三段 base64url）
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\
               .eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIn0\
               .SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    assert_hit(&e, "OUT-06", jwt);

    // negative: 只有一段（无点分隔）
    assert_no_hit(&e, "OUT-06", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

    // negative: 缺少第三段（只有两段）
    assert_no_hit(&e, "OUT-06", "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0");
}

// ---------------------------------------------------------------------------
// OUT-07: PEM Private Key
// ---------------------------------------------------------------------------
#[test]
fn out_07_pem_key() {
    let e = build_engine();

    // positive: PKCS#8 generic
    assert_hit(&e, "OUT-07", "-----BEGIN PRIVATE KEY-----");

    // positive: RSA
    assert_hit(&e, "OUT-07", "-----BEGIN RSA PRIVATE KEY-----");

    // positive: EC
    assert_hit(&e, "OUT-07", "-----BEGIN EC PRIVATE KEY-----");

    // positive: DSA
    assert_hit(&e, "OUT-07", "-----BEGIN DSA PRIVATE KEY-----");

    // positive: ENCRYPTED PRIVATE KEY
    assert_hit(&e, "OUT-07", "-----BEGIN ENCRYPTED PRIVATE KEY-----");

    // negative: 这是公钥，不是私钥
    assert_no_hit(&e, "OUT-07", "-----BEGIN PUBLIC KEY-----");

    // negative: BEGIN 后没有 PRIVATE KEY 字样
    assert_no_hit(&e, "OUT-07", "-----BEGIN CERTIFICATE-----");
}

// ---------------------------------------------------------------------------
// OUT-08: Stripe Live Key
// ---------------------------------------------------------------------------
#[test]
fn out_08_stripe_live() {
    let e = build_engine();

    // positive: 密钥
    assert_hit(&e, "OUT-08", "sk_live_abcdefghijklmnopqrstuvwxyz");

    // positive: 发布密钥
    assert_hit(&e, "OUT-08", "pk_live_abcdefghijklmnopqrstuvwxyz");

    // positive: 受限密钥
    assert_hit(&e, "OUT-08", "rk_live_abcdefghijklmnopqrstuvwxyz");

    // negative: test 环境 key（_test_ 不在 pattern 中，但此处验证 _live_ 才匹配）
    assert_no_hit(&e, "OUT-08", "sk_test_abcdefghijklmnopqrstuvwxyz");

    // negative: 前缀不是 sk/pk/rk
    assert_no_hit(&e, "OUT-08", "mk_live_abcdefghijklmnopqrstuvwxyz");
}

// ---------------------------------------------------------------------------
// OUT-09: Slack Token
// ---------------------------------------------------------------------------
#[test]
fn out_09_slack_token() {
    let e = build_engine();

    // positive: xoxb (Bot token)
    assert_hit(&e, "OUT-09", "xoxb-1234567890-1234567890-AbCdEfGhIjKlMnOp");

    // positive: xoxp (User token)
    assert_hit(&e, "OUT-09", "xoxp-1234567890-1234567890-AbCdEfGhIjKlMnOp");

    // positive: xoxa (App-level token)
    assert_hit(&e, "OUT-09", "xoxa-1234567890-1234567890-AbCdEfGhIjKlMnOp");

    // positive: xoxs (Workspace token)
    assert_hit(&e, "OUT-09", "xoxs-1234567890-1234567890-AbCdEfGhIjKlMnOp");

    // negative: 前缀字母不在 [bpas] 中
    assert_no_hit(&e, "OUT-09", "xoxz-1234567890-1234567890-AbCdEf");

    // negative: 后缀太短（少于 10 个字符）
    assert_no_hit(&e, "OUT-09", "xoxb-short");
}

// ---------------------------------------------------------------------------
// OUT-10: OpenSSH Private Key
// ---------------------------------------------------------------------------
#[test]
fn out_10_openssh_key() {
    let e = build_engine();

    // positive: 精确匹配 OpenSSH 头部
    assert_hit(&e, "OUT-10", "-----BEGIN OPENSSH PRIVATE KEY-----");

    // negative: 少了 OPENSSH 标识（属于 OUT-07 范畴）
    assert_no_hit(&e, "OUT-10", "-----BEGIN PRIVATE KEY-----");

    // negative: 拼写错误
    assert_no_hit(&e, "OUT-10", "-----BEGIN OPENSSH PUBLIC KEY-----");
}

// ---------------------------------------------------------------------------
// OUT-11: Discord Bot Token
// ---------------------------------------------------------------------------
#[test]
fn out_11_discord_token() {
    let e = build_engine();

    // positive: 符合 24~28.6.27~38 base64url 格式
    // 第一段 26 字符，第二段 6 字符，第三段 27 字符
    assert_hit(
        &e,
        "OUT-11",
        "MTIzNDU2Nzg5MDEyMzQ1Njc4.AbCdEf.xyzabcdefghijklmnopqrstuvwx",
    );

    // positive: 最短有效格式（24.6.27）— 第一段需要至少 24 字符
    assert_hit(
        &e,
        "OUT-11",
        "MTIzNDU2Nzg5MDEyMzQ1Njc8.AbCdEf.xyzabcdefghijklmnopqrstuvwx",
    );

    // negative: 第一段太短（只有 10 字符）
    assert_no_hit(
        &e,
        "OUT-11",
        "MTIzNDU2Nz.AbCdEf.xyzabcdefghijklmnopqrstuvwx",
    );

    // negative: 第二段太长（7 字符而非 6）
    assert_no_hit(
        &e,
        "OUT-11",
        "MTIzNDU2Nzg5MDEyMzQ1Njc4.AbCdEfG.xyzabcdefghijklmnopqrstuvwx",
    );

    // negative: 缺少点分隔
    assert_no_hit(&e, "OUT-11", "MTIzNDU2Nzg5MDEyMzQ1Njc4AbCdEfxyzabcdefghij");
}
