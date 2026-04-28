use std::path::Path;

use uuid::Uuid;

use crate::{error::PendingError, protocol::DecisionRequest};

/// 读取并验证 pending 文件。
///
/// 返回：
/// - `Ok(DecisionRequest)` — 文件存在、未过期、解析成功
/// - `Err(PendingError::NotFound)` — 文件不存在（fail-open）
/// - `Err(PendingError::Stale)` — created_at 超过 `stale_threshold_secs`（fail-closed）
/// - `Err(PendingError::ParseError)` — JSON 解析失败（fail-closed）
/// - `Err(PendingError::IoError)` — 其他 IO 错误
///
/// 关联：SPEC-001 §4.2（stale 检测）。
pub fn read_pending_checked(
    request_id: Uuid,
    base: &Path,
    stale_threshold_secs: i64,
) -> Result<DecisionRequest, PendingError> {
    let path = base.join("pending").join(format!("{request_id}.json"));

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(PendingError::NotFound);
        }
        Err(e) => return Err(PendingError::IoError(e.to_string())),
    };

    let req: DecisionRequest =
        serde_json::from_str(&content).map_err(|e| PendingError::ParseError(e.to_string()))?;

    // stale 检测：created_at 超过阈值视为过期，fail-closed。
    let age_secs = chrono::Utc::now()
        .signed_duration_since(req.created_at)
        .num_seconds();
    if age_secs > stale_threshold_secs {
        return Err(PendingError::Stale);
    }

    Ok(req)
}

/// 启发式扫目录结果。
pub struct ScanResult {
    /// 所有有效（未过期）的 pending 请求，按 created_at 升序排列。
    pub fresh: Vec<DecisionRequest>,
    /// 过期的 pending 文件路径（供调用方删除）。
    pub stale_paths: Vec<std::path::PathBuf>,
}

/// 扫描 `<base>/pending/` 目录，收集所有未过期的 pending 文件。
///
/// 用于 SIEVE_REQUEST_ID 未设置时的启发式匹配路径。
/// 按 created_at 升序排列，避免随机顺序引起非确定性行为。
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录）。
pub fn scan_pending_dir(base: &Path, stale_threshold_secs: i64) -> ScanResult {
    let pending_dir = base.join("pending");
    let mut fresh: Vec<DecisionRequest> = Vec::new();
    let mut stale_paths: Vec<std::path::PathBuf> = Vec::new();

    let entries = match std::fs::read_dir(&pending_dir) {
        Ok(e) => e,
        Err(_) => {
            // 目录不存在或无权读 → 视为空目录，fail-open。
            return ScanResult { fresh, stale_paths };
        }
    };

    let now = chrono::Utc::now();

    let decisions_dir = base.join("decisions");

    for entry in entries.flatten() {
        let path = entry.path();
        // 只处理 .json 文件。
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let req: DecisionRequest = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(_) => continue, // 解析失败的文件跳过。
        };

        // 已决策的 pending 跳过（避免重复弹窗）。
        // 若 decisions/<id>.json 已存在，说明该请求已被处理，不再加入 fresh/stale。
        // 关联：SPEC-001 §4.3（清理机制）。
        let decision_path = decisions_dir.join(format!("{}.json", req.request_id));
        if decision_path.exists() {
            continue;
        }

        let age_secs = now.signed_duration_since(req.created_at).num_seconds();
        if age_secs > stale_threshold_secs {
            stale_paths.push(path);
        } else {
            fresh.push(req);
        }
    }

    // 按 created_at 升序排列，保证确定性。
    fresh.sort_by_key(|r| r.created_at);

    ScanResult { fresh, stale_paths }
}
