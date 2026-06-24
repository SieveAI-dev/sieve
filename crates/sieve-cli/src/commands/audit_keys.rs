//! `full` 档加密审计的密钥生命周期 CLI。
//!
//! 与只读的 `commands/audit.rs` 分离——本模块负责**写 + 加密**（密钥生成 / 轮换 /
//! 解密归档），不碰审计查询路径，保持模块语义边界清晰。
//!
//! 口令经环境变量 `SIEVE_AUDIT_PASSPHRASE` 提供（**不回显**，CI 友好）。daemon 永不
//! 接触私钥；私钥只在 ①keygen ②decrypt 两个时刻出现，且应离线保管。

use age::secrecy::SecretString;
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

use crate::audit_archive::{self, keys};

/// 从环境变量读口令（不回显）。缺失/空时给出明确指引。
fn read_passphrase() -> Result<SecretString> {
    let p = std::env::var("SIEVE_AUDIT_PASSPHRASE").map_err(|_| {
        anyhow!(
            "请通过环境变量 SIEVE_AUDIT_PASSPHRASE 提供口令（不回显）。\n\
             该口令保护 full 档私钥；口令丢失 = 归档永久不可读（by design）。"
        )
    })?;
    if p.is_empty() {
        return Err(anyhow!("SIEVE_AUDIT_PASSPHRASE 不能为空"));
    }
    Ok(SecretString::from(p))
}

fn default_identity_path(rotated: bool) -> Result<PathBuf> {
    let home = sieve_ipc::paths::sieve_home().context("获取 sieve home 失败")?;
    let name = if rotated {
        "audit-identity-rotated.age"
    } else {
        "audit-identity.age"
    };
    Ok(home.join(name))
}

/// 写文件并设 0600（Unix）。父目录确保存在（0700）。
fn write_protected_identity(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("创建目录 {} 失败", parent.display()))?;
    }
    std::fs::write(path, bytes).with_context(|| format!("写私钥文件 {} 失败", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// `sieve audit keygen` / `rotate-key`：生成密钥对，私钥经口令保护写离线文件，
/// 公钥打印供粘贴进 config。
pub fn keygen(out: Option<PathBuf>, force: bool, rotated: bool) -> Result<()> {
    let pass = read_passphrase()?;
    let out = match out {
        Some(p) => p,
        None => default_identity_path(rotated)?,
    };
    if out.exists() && !force {
        return Err(anyhow!(
            "私钥文件 {} 已存在。用 --force 覆盖——但这会使用旧密钥加密的归档【永久无法解密】。",
            out.display()
        ));
    }

    let kp = keys::generate(&pass)?;
    write_protected_identity(&out, &kp.protected_identity)?;

    println!("✅ 已生成 full 档加密审计密钥对（write-only logging，daemon 只持公钥）。");
    println!();
    println!("① 把公钥粘贴进 config.toml：");
    println!();
    println!("   [audit]");
    println!("   level = \"full\"");
    println!("   recipient = \"{}\"", kp.recipient);
    println!();
    println!("② 口令保护的私钥已写入（0600）：{}", out.display());
    println!(
        "   ⚠ 立即把它移出本机（密码管理器 / 离线介质）：daemon 只需公钥，\n\
         \x20    私钥留在本机会削弱「机器被攻陷也解不开历史归档」的防护。"
    );
    println!(
        "   ⚠⚠ 口令一旦丢失，私钥无法解锁，所有 full 档归档将【永久无法解密】——\n\
         \x20     这是设计使然，无法恢复。请立即备份口令到密码管理器。"
    );
    if rotated {
        println!();
        println!("↻ 轮换提示：用【旧】密钥加密的归档段仍需【旧】私钥解密，请妥善保留旧私钥文件。");
    }
    Ok(())
}

/// `sieve audit decrypt`：用口令解锁私钥，校验哈希链并解密归档段，逐条输出脱敏后内容。
pub fn decrypt(identity_path: PathBuf, segment: PathBuf) -> Result<()> {
    let pass = read_passphrase()?;
    let protected = std::fs::read(&identity_path)
        .with_context(|| format!("读私钥文件 {} 失败", identity_path.display()))?;
    let identity = keys::unlock_identity(&protected, &pass)?;

    let records = audit_archive::verify_and_decrypt(&segment, &identity)?;
    eprintln!(
        "✅ 哈希链校验通过，共 {} 条记录（{}）。",
        records.len(),
        segment.display()
    );
    for r in records {
        let obj = serde_json::json!({
            "seq": r.seq,
            "ts": r.ts,
            "content": String::from_utf8_lossy(&r.content),
        });
        println!("{obj}");
    }
    Ok(())
}
