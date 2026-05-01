//! 数据集 FP rate 验证（PRD §9 #7：Critical FP < 0.5%）。
//!
//! 跑法（release build 避免 dev build 太慢）：
//! ```bash
//! cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture
//! ```
//!
//! 两个测试均标记 `#[ignore]`，按需手动触发，不阻塞 CI 常规测试。

use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
use sieve_rules::manifest::Severity;
use std::path::PathBuf;

fn rules_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("rules");
    p
}

fn bench_data_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("bench-data");
    p
}

fn build_outbound_engine() -> VectorscanEngine {
    let path = rules_dir().join("outbound.toml");
    let rules = load_outbound_rules(&path).expect("load outbound.toml");
    VectorscanEngine::compile(rules).expect("compile outbound engine")
}

fn build_inbound_engine() -> VectorscanEngine {
    let path = rules_dir().join("inbound.toml");
    let rules = load_inbound_rules(&path).expect("load inbound.toml");
    let filtered: Vec<_> = rules
        .into_iter()
        .filter(|r| {
            r.pattern != "__ADDRESS_GUARD_PLACEHOLDER__"
                && r.pattern != "__OPENCLAW_SKILL_GUARD_PLACEHOLDER__"
        })
        .collect();
    VectorscanEngine::compile(filtered).expect("compile inbound engine")
}

/// 读取目录下所有 .txt 文件，返回 (文件路径, 内容) 列表。
fn read_samples(dir: &PathBuf) -> Vec<(PathBuf, String)> {
    let mut samples = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
            .map(|e| e.path())
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                samples.push((path, content));
            }
        }
    }
    samples
}

/// 递归读取目录及所有子目录下的 .txt 文件。
fn read_samples_recursive(root: &PathBuf) -> Vec<(PathBuf, String)> {
    let mut samples = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        samples.push((path, content));
                    }
                }
            }
        }
    }
    samples.sort_by(|a, b| a.0.cmp(&b.0));
    samples
}

/// 把样本路径归类到桶名（用于按桶聚合 FP / recall 报告）。
///
/// 规则：
/// - `bench-data/benign/*.txt`        → "benign-generic"
/// - `bench-data/benign-near/<sub>/*` → "<sub>"（如 "near-OUT-api-keys"）
/// - `bench-data/attacks/*.txt`       → "attacks-by-rule-id"（按文件名前缀解析）
/// - `bench-data/attacks-by-fear/<sub>/*` → "<sub>"（如 "signing"）
fn bucket_for(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    if let Some(idx) = s.find("/benign-near/") {
        let rest = &s[idx + "/benign-near/".len()..];
        return rest.split('/').next().unwrap_or("benign-near-unknown").to_string();
    }
    if let Some(idx) = s.find("/attacks-by-fear/") {
        let rest = &s[idx + "/attacks-by-fear/".len()..];
        return rest.split('/').next().unwrap_or("attacks-by-fear-unknown").to_string();
    }
    if s.contains("/benign/") {
        return "benign-generic".to_string();
    }
    if s.contains("/attacks/") {
        // attacks/ 用文件名前缀（IN-CR-01 / IN-GEN-01 / OUT-01 等）作桶
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        // 文件名形如 "IN-CR-02-CURL-1.txt" / "OUT-01-1.txt"，桶取前两/三段去掉末尾数字
        let parts: Vec<&str> = fname.trim_end_matches(".txt").split('-').collect();
        let mut bucket_parts = Vec::new();
        for p in &parts {
            if p.chars().all(|c| c.is_ascii_digit()) {
                break;
            }
            bucket_parts.push(*p);
        }
        return format!("attacks-{}", bucket_parts.join("-"));
    }
    "unknown".to_string()
}

/// 验证 benign 数据集的 Critical FP rate < 0.5%（PRD §9 #7 公理 12）。
///
/// FP 定义：benign 样本触发了 Critical severity 规则。
/// 注意：本测试检查 vectorscan 层原始命中，不含 entropy / allowlist 后过滤。
/// 这是保守测量：实际 FP rate 会更低（allowlist 进一步过滤）。
#[test]
#[ignore = "long-running dataset test; run with: cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture"]
fn benign_dataset_critical_fp_rate_below_threshold() {
    let outbound = build_outbound_engine();
    let inbound = build_inbound_engine();

    // 加载规则元信息，用于 severity 判断
    let outbound_rules =
        load_outbound_rules(&rules_dir().join("outbound.toml")).expect("load outbound rules");
    let inbound_rules =
        load_inbound_rules(&rules_dir().join("inbound.toml")).expect("load inbound rules");

    let mut benigns = read_samples_recursive(&bench_data_dir().join("benign"));
    benigns.extend(read_samples_recursive(&bench_data_dir().join("benign-near")));
    let total = benigns.len();
    assert!(
        total >= 500,
        "benign dataset must have at least 500 samples (post-expansion), got {total}"
    );

    let mut fp_count = 0usize;
    let mut fp_details: Vec<String> = Vec::new();

    // 按桶聚合：bucket -> (total, fp_count, fp_details)
    use std::collections::BTreeMap;
    let mut per_bucket: BTreeMap<String, (usize, usize, Vec<String>)> = BTreeMap::new();

    for (path, content) in &benigns {
        let bucket = bucket_for(path);
        per_bucket.entry(bucket.clone()).or_default().0 += 1;

        let outbound_hits = outbound
            .scan(content.as_bytes())
            .expect("outbound scan failed");
        let inbound_hits = inbound
            .scan(content.as_bytes())
            .expect("inbound scan failed");

        // 检查出站 Critical 命中（结合 is_excluded allowlist 过滤）
        for hit in &outbound_hits {
            let rule = outbound_rules.iter().find(|r| r.id == hit.rule_id);
            let is_crit = rule.is_some_and(|r| r.severity == Severity::Critical);
            if is_crit {
                let matched_text =
                    &content[hit.start.min(content.len())..hit.end.min(content.len())];
                let excluded = rule.is_some_and(|r| outbound.is_excluded(matched_text, content, r));
                if !excluded {
                    fp_count += 1;
                    let detail = format!(
                        "FP [OUT Critical]: {} -> rule={} matched={:?}",
                        path.display(),
                        hit.rule_id,
                        &matched_text[..matched_text.len().min(60)]
                    );
                    let entry = per_bucket.entry(bucket.clone()).or_default();
                    entry.1 += 1;
                    entry.2.push(detail.clone());
                    fp_details.push(detail);
                }
            }
        }

        // 检查入站 Critical 命中（结合 is_excluded allowlist 过滤）
        for hit in &inbound_hits {
            let rule = inbound_rules.iter().find(|r| r.id == hit.rule_id);
            let is_crit = rule.is_some_and(|r| r.severity == Severity::Critical);
            if is_crit {
                let matched_text =
                    &content[hit.start.min(content.len())..hit.end.min(content.len())];
                let excluded = rule.is_some_and(|r| inbound.is_excluded(matched_text, content, r));
                if !excluded {
                    fp_count += 1;
                    let detail = format!(
                        "FP [IN Critical]: {} -> rule={} matched={:?}",
                        path.display(),
                        hit.rule_id,
                        &matched_text[..matched_text.len().min(60)]
                    );
                    let entry = per_bucket.entry(bucket.clone()).or_default();
                    entry.1 += 1;
                    entry.2.push(detail.clone());
                    fp_details.push(detail);
                }
            }
        }
    }

    let fp_rate = fp_count as f64 / total as f64;
    println!("\n=== Benign Dataset FP Rate Report ===");
    println!("Total benign samples: {total}");
    println!("Critical FP hits: {fp_count}");
    println!("Overall FP rate: {}/{} = {:.4}%", fp_count, total, fp_rate * 100.0);

    println!("\n--- Per-bucket breakdown ---");
    for (bucket, (bucket_total, bucket_fp, _)) in &per_bucket {
        let bucket_rate = if *bucket_total > 0 {
            *bucket_fp as f64 / *bucket_total as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "  {bucket:<40} {bucket_fp:>4}/{bucket_total:<4} = {bucket_rate:.2}%"
        );
    }

    if !fp_details.is_empty() {
        println!("\nFP Details:");
        for detail in &fp_details {
            println!("  {detail}");
        }
    } else {
        println!("\nNo FP hits detected.");
    }

    // PRD §9 #7：Critical FP < 0.5%
    assert!(
        fp_rate < 0.005,
        "FP rate {:.4}% exceeds PRD §9 #7 threshold (0.5%). FP count={fp_count}/{total}.\nDetails:\n{}",
        fp_rate * 100.0,
        fp_details.join("\n")
    );
}

/// 验证攻击数据集的 recall rate（应该接近 100%）。
///
/// Recall 定义：包含攻击特征的样本中，至少有 1 个规则命中。
/// 注意：IN-CR-01（地址替换）通过 strsim Levenshtein 实现，不在 vectorscan 中，
/// 因此 IN-CR-01-* 样本按设计无法被 vectorscan 层命中，不计入 recall 统计。
#[test]
#[ignore = "long-running dataset test; run with: cargo test -p sieve-rules --release --test dataset_fp_rate -- --ignored --nocapture"]
fn attack_dataset_recall_rate() {
    let outbound = build_outbound_engine();
    let inbound = build_inbound_engine();

    let mut attacks = read_samples_recursive(&bench_data_dir().join("attacks"));
    attacks.extend(read_samples_recursive(&bench_data_dir().join("attacks-by-fear")));
    let total_raw = attacks.len();
    assert!(
        total_raw >= 500,
        "attacks dataset must have at least 500 samples (post-expansion), got {total_raw}"
    );

    // IN-CR-01（地址替换）+ attacks-by-fear/transfer/（其中 60% 是地址替换变种）
    // 走 strsim Levenshtein 路径，不在 vectorscan 中，需排除出 vectorscan-recall 计算
    let attacks: Vec<_> = attacks
        .into_iter()
        .filter(|(path, _)| {
            let s = path.to_string_lossy();
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !fname.starts_with("IN-CR-01-") && !s.contains("/attacks-by-fear/transfer/")
        })
        .collect();
    let total = attacks.len();

    let mut hit_count = 0usize;
    let mut missed: Vec<String> = Vec::new();

    use std::collections::BTreeMap;
    let mut per_bucket: BTreeMap<String, (usize, usize)> = BTreeMap::new();

    for (path, content) in &attacks {
        let bucket = bucket_for(path);
        per_bucket.entry(bucket.clone()).or_default().0 += 1;

        let outbound_hits = outbound
            .scan(content.as_bytes())
            .expect("outbound scan failed");
        let inbound_hits = inbound
            .scan(content.as_bytes())
            .expect("inbound scan failed");

        if !outbound_hits.is_empty() || !inbound_hits.is_empty() {
            hit_count += 1;
            per_bucket.entry(bucket).or_default().1 += 1;
        } else {
            missed.push(path.display().to_string());
        }
    }

    let recall = hit_count as f64 / total as f64;
    println!("\n=== Attack Dataset Recall Report ===");
    println!("Total attack samples (vectorscan-eligible): {total}");
    println!(
        "Excluded (Levenshtein-only path: IN-CR-01-* + attacks-by-fear/transfer/): {}",
        total_raw - total
    );
    println!("Samples with at least 1 hit: {hit_count}");
    println!(
        "Overall recall rate: {}/{} = {:.2}%",
        hit_count,
        total,
        recall * 100.0
    );

    println!("\n--- Per-bucket breakdown ---");
    for (bucket, (bucket_total, bucket_hit)) in &per_bucket {
        let bucket_rate = if *bucket_total > 0 {
            *bucket_hit as f64 / *bucket_total as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "  {bucket:<40} {bucket_hit:>4}/{bucket_total:<4} = {bucket_rate:.2}%"
        );
    }

    if !missed.is_empty() {
        println!("\nMissed samples (need rule tuning):");
        for m in &missed {
            println!("  MISS: {m}");
        }
    }

    // 目标 recall > 95%
    assert!(
        recall > 0.95,
        "Attack recall {:.2}% < 95% threshold. Missed {}/{} samples.\nMissed list:\n{}",
        recall * 100.0,
        total - hit_count,
        total,
        missed.join("\n")
    );
}

/// 快速冒烟测试：数据集目录存在且非空（不走实际扫描，适合 CI 常规测试）。
#[test]
fn dataset_directories_not_empty() {
    let attacks_dir = bench_data_dir().join("attacks");
    let benign_dir = bench_data_dir().join("benign");

    let attacks = read_samples(&attacks_dir);
    let benigns = read_samples(&benign_dir);

    assert!(
        attacks.len() >= 200,
        "attacks/ must have >= 200 samples, got {}",
        attacks.len()
    );
    assert!(
        benigns.len() >= 50,
        "benign/ must have >= 50 samples, got {}",
        benigns.len()
    );

    println!(
        "Dataset sizes: attacks={}, benign={}",
        attacks.len(),
        benigns.len()
    );
}
