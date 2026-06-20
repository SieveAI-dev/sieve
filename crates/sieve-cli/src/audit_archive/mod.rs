//! 加密审计归档（`full` 档，ADR-037 / SPEC-009）。
//!
//! **write-only logging**：daemon 只持 age **recipient 公钥**，对每条归档记录做混合加密
//! （`age::encrypt` = 随机 file key + ChaCha20-Poly1305 内容加密 + X25519 wrap 给公钥）。
//! 机器运行时被攻陷也**解不开历史归档**——本机无私钥。
//!
//! **红线（ADR-037 决策 2）**：归档单元只消费**脱敏后内容**（出站 `seg_result.texts` /
//! `new_body`）。本模块的 [`ArchiveWriter::append`] 把收到的字节**原样加密**——它无法判断
//! 输入是否脱敏，红线靠**调用点只传脱敏后内容**兑现（daemon hook + 回归测试守护）。
//!
//! **防篡改（ADR-037 决策 4）**：每条记录含 `prev_hash`（上条 hash）+ 单调 `seq`，构成
//! 段内哈希链——中间删改/重排/截断会断链。残余局限：挡不住「末尾追加伪造」（持公钥的
//! daemon 可续链），只保证历史不可悄悄改写。
//!
//! **保留期（ADR-037 决策 5）**：超期段整段删除（归档上唯一允许的变更）。
//!
//! ## 接通状态（已全部接线，2026-06-20）
//!
//! - `keys`（生成/解锁）+ [`verify_and_decrypt`] → `sieve audit keygen/rotate-key/decrypt` CLI。
//! - [`ArchiveWriter`] 写入路径 → daemon 出站脱敏后 fire-and-forget 归档（`daemon::build_archive_writer`
//!   按 `audit.level = full` 构造；`daemon::archive_redacted_outbound` 在 Anthropic / OpenAI
//!   两条出站脱敏路径的脱敏后 `new_body` 上 hook，红线只存脱敏后内容）。
//! - `purge_expired` → daemon 启动时按保留期清理超期段。

pub mod keys;

use age::x25519::{Identity, Recipient};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::config::ArchiveRotation;

/// 哈希链 genesis（段首记录的 prev_hash）。
const GENESIS_HASH: [u8; 32] = [0u8; 32];

/// 单条归档记录（JSONL 一行）。`ct` 是 age 加密的脱敏后内容（base64）。
#[derive(Debug, Serialize, Deserialize)]
struct ArchiveRecord {
    /// 段内单调递增序号（从 0 起）。
    seq: u64,
    /// 写入时间（RFC3339）。
    ts: String,
    /// 上一条记录的 hash（hex，64 字符）；段首为全零。
    prev_hash: String,
    /// 本条 hash = SHA256(seq_be || prev_hash || ct_bytes)（hex）；hash_chain 关时为全零。
    hash: String,
    /// age 加密后的脱敏内容（base64）。
    ct: String,
}

/// 解密并校验后的一条归档记录（供 `sieve audit decrypt`）。
#[derive(Debug)]
pub struct DecryptedRecord {
    pub seq: u64,
    pub ts: String,
    /// 脱敏后明文内容（解密结果）。
    pub content: Vec<u8>,
}

/// 哈希链头状态（按段维护，串行化 append）。
struct ChainHead {
    segment: String,
    prev_hash: [u8; 32],
    next_seq: u64,
}

/// 加密归档写入器（daemon 持有，**只含公钥，结构上不可解密**）。
pub struct ArchiveWriter {
    recipient: Recipient,
    dir: PathBuf,
    rotation: ArchiveRotation,
    hash_chain: bool,
    head: Mutex<ChainHead>,
}

impl ArchiveWriter {
    /// 构造写入器：解析 recipient 公钥，确保归档目录存在（0700）。
    ///
    /// `recipient` 来自 `config.toml [audit].recipient`；只读公钥，无解密能力。
    pub fn new(
        recipient: &str,
        dir: PathBuf,
        rotation: ArchiveRotation,
        hash_chain: bool,
    ) -> Result<Self> {
        let recipient = keys::parse_recipient(recipient)?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("创建归档目录 {} 失败", dir.display()))?;
        restrict_dir_permissions(&dir);
        Ok(Self {
            recipient,
            dir,
            rotation,
            hash_chain,
            head: Mutex::new(ChainHead {
                segment: String::new(),
                prev_hash: GENESIS_HASH,
                next_seq: 0,
            }),
        })
    }

    /// 追加一条归档记录。**`content` 必须是脱敏后内容**（红线，调用点保证）。
    ///
    /// 同步阻塞写盘——调用方应在 `spawn_blocking` / fire-and-forget 中调用（off hot path）。
    /// 失败只应被调用方 warn，绝不阻断 forward（审计可靠性问题不得变为可用性事故）。
    pub fn append(&self, content: &[u8]) -> Result<()> {
        let segment = self.segment_name();
        let seg_path = self.dir.join(&segment);

        let mut head = self
            .head
            .lock()
            .map_err(|e| anyhow!("archive head mutex poisoned: {e}"))?;

        // 段切换（或首次写入）：从已存在段文件末尾恢复链头（支持 daemon 重启续链）。
        if head.segment != segment {
            let (prev_hash, next_seq) = load_chain_head(&seg_path)?;
            head.segment = segment.clone();
            head.prev_hash = prev_hash;
            head.next_seq = next_seq;
        }

        // 混合加密脱敏后内容（age：随机 file key + AEAD + 公钥 wrap）。
        let ciphertext = age::encrypt(&self.recipient, content)
            .map_err(|e| anyhow!("age 加密归档记录失败: {e}"))?;
        let ct_b64 = B64.encode(&ciphertext);

        let seq = head.next_seq;
        let prev_hash = head.prev_hash;
        let hash = if self.hash_chain {
            compute_hash(seq, &prev_hash, ct_b64.as_bytes())
        } else {
            GENESIS_HASH
        };

        let record = ArchiveRecord {
            seq,
            ts: Utc::now().to_rfc3339(),
            prev_hash: hex_encode(&prev_hash),
            hash: hex_encode(&hash),
            ct: ct_b64,
        };
        let mut line = serde_json::to_string(&record).context("序列化归档记录失败")?;
        line.push('\n');

        append_line_0600(&seg_path, line.as_bytes())
            .with_context(|| format!("写归档段 {} 失败", seg_path.display()))?;

        head.prev_hash = hash;
        head.next_seq = seq + 1;
        Ok(())
    }

    /// 按 rotation 计算当前段文件名（基于 UTC 当前时间）。
    fn segment_name(&self) -> String {
        let now = Utc::now();
        let tag = match self.rotation {
            ArchiveRotation::Daily => now.format("%Y-%m-%d").to_string(),
            ArchiveRotation::Weekly => now.format("%G-W%V").to_string(),
            ArchiveRotation::Monthly => now.format("%Y-%m").to_string(),
        };
        format!("archive-{tag}.jsonl")
    }

    /// 清理超期段（ADR-037 决策 5）：删除 mtime 早于 `retention_days` 的整段文件。
    /// `retention_days == 0` 表示永久保留（no-op）。返回被删除的段文件名。
    pub fn purge_expired(&self, retention_days: u32) -> Result<Vec<String>> {
        if retention_days == 0 {
            return Ok(Vec::new());
        }
        let cutoff = std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(
                retention_days as u64 * 24 * 3600,
            ))
            .ok_or_else(|| anyhow!("retention cutoff 计算溢出"))?;

        let mut deleted = Vec::new();
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(e) => e,
            Err(_) => return Ok(deleted), // 目录不存在 → 无段可清
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_segment = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("archive-") && n.ends_with(".jsonl"))
                .unwrap_or(false);
            if !is_segment {
                continue;
            }
            let expired = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|mtime| mtime < cutoff)
                .unwrap_or(false);
            if expired && std::fs::remove_file(&path).is_ok() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    deleted.push(name.to_string());
                }
            }
        }
        Ok(deleted)
    }
}

/// 校验段文件的哈希链完整性并用 identity 解密所有记录（`sieve audit decrypt`）。
///
/// 任何链断裂（hash 不匹配 / prev 不连续 / seq 跳号）立即返回错误（历史被改写）。
pub fn verify_and_decrypt(seg_path: &Path, identity: &Identity) -> Result<Vec<DecryptedRecord>> {
    let data = std::fs::read_to_string(seg_path)
        .with_context(|| format!("读归档段 {} 失败", seg_path.display()))?;
    let mut out = Vec::new();
    let mut expected_prev = GENESIS_HASH;
    let mut expected_seq = 0u64;

    for (lineno, line) in data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let rec: ArchiveRecord = serde_json::from_str(line)
            .with_context(|| format!("解析归档记录失败（第 {} 行）", lineno + 1))?;

        if rec.seq != expected_seq {
            return Err(anyhow!(
                "哈希链断裂：seq 跳号（期望 {expected_seq}，实际 {}）——历史被截断或重排",
                rec.seq
            ));
        }
        let prev = hex_decode(&rec.prev_hash)?;
        if prev != expected_prev {
            return Err(anyhow!(
                "哈希链断裂：第 {} 条 prev_hash 不连续——历史被改写",
                rec.seq
            ));
        }
        let recomputed = compute_hash(rec.seq, &prev, rec.ct.as_bytes());
        if hex_encode(&recomputed) != rec.hash {
            return Err(anyhow!(
                "哈希链断裂：第 {} 条 hash 不匹配——记录被篡改",
                rec.seq
            ));
        }

        let ciphertext = B64.decode(&rec.ct).context("base64 解码归档密文失败")?;
        let content = age::decrypt(identity, &ciphertext)
            .map_err(|e| anyhow!("age 解密第 {} 条失败（identity 不匹配？）: {e}", rec.seq))?;

        out.push(DecryptedRecord {
            seq: rec.seq,
            ts: rec.ts,
            content,
        });
        expected_prev = recomputed;
        expected_seq += 1;
    }
    Ok(out)
}

/// 从已存在段文件末尾恢复链头：返回 `(last_hash, next_seq)`。文件不存在 → genesis。
fn load_chain_head(seg_path: &Path) -> Result<([u8; 32], u64)> {
    if !seg_path.exists() {
        return Ok((GENESIS_HASH, 0));
    }
    let data = std::fs::read_to_string(seg_path)
        .with_context(|| format!("读归档段 {} 失败", seg_path.display()))?;
    let last = data.lines().filter(|l| !l.trim().is_empty()).next_back();
    match last {
        None => Ok((GENESIS_HASH, 0)),
        Some(line) => {
            let rec: ArchiveRecord = serde_json::from_str(line).context("解析归档段末行失败")?;
            Ok((hex_decode(&rec.hash)?, rec.seq + 1))
        }
    }
}

/// hash = SHA256(seq_be || prev_hash || ct_bytes)。
fn compute_hash(seq: u64, prev_hash: &[u8; 32], ct: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(seq.to_be_bytes());
    h.update(prev_hash);
    h.update(ct);
    h.finalize().into()
}

/// 追加一行到段文件，确保文件 0600（Unix）。
fn append_line_0600(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(bytes)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_dir_permissions(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(dir) {
        let mut perms = meta.permissions();
        perms.set_mode(0o700);
        let _ = std::fs::set_permissions(dir, perms);
    }
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_dir: &Path) {}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Result<[u8; 32]> {
    if s.len() != 64 {
        return Err(anyhow!("hash hex 长度应为 64，实际 {}", s.len()));
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| anyhow!("hash hex 解析失败: {e}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use tempfile::tempdir;

    fn fresh_writer(dir: PathBuf, hash_chain: bool) -> (ArchiveWriter, Identity) {
        let identity = Identity::generate();
        let recipient = identity.to_public().to_string();
        let w = ArchiveWriter::new(&recipient, dir, ArchiveRotation::Daily, hash_chain)
            .expect("build writer");
        (w, identity)
    }

    #[test]
    fn roundtrip_encrypt_then_decrypt() {
        let dir = tempdir().unwrap();
        let (w, id) = fresh_writer(dir.path().to_path_buf(), true);
        w.append(b"[REDACTED:OUT-01] redacted prompt body").unwrap();
        w.append(b"second redacted record").unwrap();

        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let recs = verify_and_decrypt(&seg, &id).expect("verify+decrypt");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].content, b"[REDACTED:OUT-01] redacted prompt body");
        assert_eq!(recs[1].content, b"second redacted record");
    }

    // ── 红线：归档段文件里看不到任何明文内容（只有密文）──────────────────────
    #[test]
    fn on_disk_segment_is_ciphertext_only_no_plaintext() {
        let dir = tempdir().unwrap();
        let (w, _id) = fresh_writer(dir.path().to_path_buf(), true);
        // 模拟「脱敏后内容」——真实流程绝不会把明文密钥传进来，但即便传了，盘上也是密文。
        let redacted = b"user asked about [REDACTED:PRIVATE-KEY] and [REDACTED:BIP39]";
        w.append(redacted).unwrap();

        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let raw = std::fs::read(&seg).unwrap();
        // 段文件中不得出现脱敏后内容的明文片段（已被 age 加密）。
        assert!(
            !contains_subslice(&raw, b"[REDACTED:PRIVATE-KEY]"),
            "归档段不得含明文内容，必须是密文"
        );
        assert!(
            !contains_subslice(&raw, b"user asked about"),
            "归档段不得含明文内容，必须是密文"
        );
    }

    // ── 哈希链：篡改任一记录必须被检出 ──────────────────────────────────────
    #[test]
    fn hash_chain_detects_tampering() {
        let dir = tempdir().unwrap();
        let (w, id) = fresh_writer(dir.path().to_path_buf(), true);
        w.append(b"record-A").unwrap();
        w.append(b"record-B").unwrap();
        w.append(b"record-C").unwrap();

        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        // 正常情况下校验通过。
        verify_and_decrypt(&seg, &id).expect("clean chain verifies");

        // 篡改中间记录的密文（替换为另一条 recipient 的合法密文，但 hash 不变）。
        let data = std::fs::read_to_string(&seg).unwrap();
        let mut lines: Vec<String> = data.lines().map(|s| s.to_string()).collect();
        let mut rec: ArchiveRecord = serde_json::from_str(&lines[1]).unwrap();
        rec.ct = B64.encode(b"forged-ciphertext"); // 改密文但不更新 hash
        lines[1] = serde_json::to_string(&rec).unwrap();
        std::fs::write(&seg, lines.join("\n") + "\n").unwrap();

        let err = verify_and_decrypt(&seg, &id).expect_err("篡改必须被检出");
        assert!(
            err.to_string().contains("hash 不匹配") || err.to_string().contains("断裂"),
            "篡改应触发哈希链断裂，实际: {err}"
        );
    }

    #[test]
    fn hash_chain_detects_truncation() {
        let dir = tempdir().unwrap();
        let (w, id) = fresh_writer(dir.path().to_path_buf(), true);
        for i in 0..4 {
            w.append(format!("rec-{i}").as_bytes()).unwrap();
        }
        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        // 删掉中间一条 → seq 跳号 → 断链。
        let data = std::fs::read_to_string(&seg).unwrap();
        let lines: Vec<&str> = data.lines().collect();
        let kept = format!("{}\n{}\n{}\n", lines[0], lines[1], lines[3]);
        std::fs::write(&seg, kept).unwrap();
        assert!(
            verify_and_decrypt(&seg, &id).is_err(),
            "删中间记录应被检出（seq 跳号断链）"
        );
    }

    #[test]
    fn chain_resumes_across_writer_restart() {
        let dir = tempdir().unwrap();
        let recipient_id = Identity::generate();
        let recipient = recipient_id.to_public().to_string();
        // 第一个 writer 写两条。
        {
            let w = ArchiveWriter::new(
                &recipient,
                dir.path().to_path_buf(),
                ArchiveRotation::Daily,
                true,
            )
            .unwrap();
            w.append(b"r0").unwrap();
            w.append(b"r1").unwrap();
        }
        // 第二个 writer（模拟 daemon 重启）续写，链应连续。
        {
            let w = ArchiveWriter::new(
                &recipient,
                dir.path().to_path_buf(),
                ArchiveRotation::Daily,
                true,
            )
            .unwrap();
            w.append(b"r2").unwrap();
        }
        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let recs = verify_and_decrypt(&seg, &recipient_id).expect("重启后链应连续可验证");
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[2].seq, 2);
    }

    #[test]
    fn purge_with_zero_retention_is_noop() {
        let dir = tempdir().unwrap();
        let (w, _id) = fresh_writer(dir.path().to_path_buf(), true);
        w.append(b"x").unwrap();
        assert!(w.purge_expired(0).unwrap().is_empty());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn wrong_identity_cannot_decrypt() {
        let dir = tempdir().unwrap();
        let (w, _id) = fresh_writer(dir.path().to_path_buf(), true);
        w.append(b"secret-redacted").unwrap();
        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let wrong = Identity::generate();
        assert!(
            verify_and_decrypt(&seg, &wrong).is_err(),
            "错误 identity 不能解密（write-only logging）"
        );
    }

    #[test]
    fn keys_module_integration_recipient_from_keygen_works() {
        // keygen 产出的 recipient 能被 ArchiveWriter 接受。
        let pass = SecretString::from("p".to_owned());
        let kp = keys::generate(&pass).unwrap();
        let dir = tempdir().unwrap();
        let w = ArchiveWriter::new(
            &kp.recipient,
            dir.path().to_path_buf(),
            ArchiveRotation::Daily,
            true,
        )
        .expect("keygen recipient 应被接受");
        w.append(b"redacted").unwrap();
        // 用解锁的 identity 解密。
        let id = keys::unlock_identity(&kp.protected_identity, &pass).unwrap();
        let seg = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let recs = verify_and_decrypt(&seg, &id).unwrap();
        assert_eq!(recs[0].content, b"redacted");
    }

    fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }
}
