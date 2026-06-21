//! 可热替换的系统规则引擎。
//!
//! # 为什么需要 SystemEngine
//!
//! 早期系统规则在 daemon 启动时一次性编译为固定的 [`VectorscanEngine`]，
//! 生命周期不可变（加载失败即 exit）。为支持「规则经签名包分发 + 运行时热替换 +
//! 无包时引擎仍可独立构建运行供审计」，把系统层也包成 [`arc_swap::ArcSwap`]，
//! 对称于 [`super::LayeredEngine`] 的 user 层热替换模式。规则包通过更新通道下发。
//!
//! # 两种「无规则」语义
//!
//! - **无规则包**（引擎独立运行的正常状态）：`inner = None` → 空规则集 fail-safe，
//!   scan 返回空，透传不检测（上层须醒目告知用户「未加载规则包」）。
//! - **包存在但验签 / sha256 失败**：由 `sieve-updater::install` 在安装阶段拒绝，
//!   不会走到 `swap_system`；daemon 启动时若 current.json 验证失败则保持空集（fail-safe），
//!   而非用篡改规则。这两条决策不在本引擎，本引擎只负责「持有当前可用规则集」。

use super::{MatchEngine, MatchHit, ScanReport, ScanRequest, VectorscanEngine};
use crate::error::SieveRulesResult;
use crate::manifest::RuleEntry;
use arc_swap::ArcSwap;
use std::sync::Arc;

/// 可原子热替换的系统规则引擎。
///
/// 内部 `ArcSwap<Option<Arc<VectorscanEngine>>>`，对称于 [`super::LayeredEngine`] 的 user 层：
/// - `None`：无规则包（引擎独立运行正常态）= 空规则集 fail-safe（透传不检测）。
/// - `Some(Arc<VectorscanEngine>)`：已装签名规则包，正常检测。
///
/// # Hot Swap（reload 链）
///
/// - scan 路径（hot path）调用 `ArcSwap::load()` 取快照，零锁零开销（lock-free read）。
/// - swap 路径 [`SystemEngine::swap_system`] 调用 `ArcSwap::store()` 原子写入新指针。
/// - 正在进行中的 scan 持有旧 `Arc<VectorscanEngine>` 快照，结束后自动释放（引用计数归零）。
///
/// daemon 启动从 updater 缓存目录的 `current.json` 加载；
/// updater 装完新签名包后发 IPC `sieve.reload_rules` → daemon 调 `swap_system`。
pub struct SystemEngine {
    inner: ArcSwap<Option<Arc<VectorscanEngine>>>,
}

impl SystemEngine {
    /// 以给定 [`VectorscanEngine`] 构造（`Some`）或空集（`None`）。
    pub fn new(engine: Option<VectorscanEngine>) -> Self {
        Self {
            inner: ArcSwap::from(Arc::new(engine.map(Arc::new))),
        }
    }

    /// 空规则集（fail-safe）：无规则包时的默认状态（引擎可独立运行供审计）。
    ///
    /// scan 始终返回空命中，daemon 透传不检测。上层应据 [`SystemEngine::has_rules`]
    /// 为 `false` 时向用户醒目告警「未加载规则包」。
    pub fn empty() -> Self {
        Self::new(None)
    }

    /// 原子热替换系统规则引擎（reload 链调用）。
    ///
    /// 调用完成后所有后续 [`MatchEngine::scan`] 立即使用新引擎；已在进行中的 scan 持有旧
    /// `Arc` 快照，完成后旧引擎自动释放。传入 `None` 退化为空集 fail-safe（等同 [`SystemEngine::empty`]）。
    pub fn swap_system(&self, engine: Option<VectorscanEngine>) {
        self.inner.store(Arc::new(engine.map(Arc::new)));
    }

    /// 当前是否已加载签名规则包。
    ///
    /// `false` = 空集 fail-safe（透传不检测），上层据此提示用户「未加载规则包」。
    pub fn has_rules(&self) -> bool {
        self.inner.load().is_some()
    }

    /// 系统规则快照（SPEC-005 §11A `sieve.list_rules` 用），无包时返回空 `Vec`。
    pub fn rules_snapshot(&self) -> Vec<RuleEntry> {
        self.inner
            .load()
            .as_ref()
            .as_ref()
            .map(|e| e.rules_snapshot())
            .unwrap_or_default()
    }
}

impl Default for SystemEngine {
    /// 默认空集 fail-safe（等同 [`SystemEngine::empty`]）。
    fn default() -> Self {
        Self::empty()
    }
}

impl MatchEngine for SystemEngine {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        // 空集 fail-safe：无规则包时返回空命中（透传不检测）。
        match self.inner.load().as_ref().as_ref() {
            Some(e) => e.scan(input),
            None => Ok(Vec::new()),
        }
    }

    fn scan_with_context(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport> {
        // 委托给当前持有的 VectorscanEngine；无包时返回空报告（rule_count = 0）。
        match self.inner.load().as_ref().as_ref() {
            Some(e) => e.scan_with_context(req),
            None => Ok(ScanReport {
                hits: Vec::new(),
                elapsed_us: 0,
                engine_name: self.engine_name().to_string(),
                rule_count: 0,
            }),
        }
    }

    fn engine_name(&self) -> &str {
        "system"
    }

    fn rule_count(&self) -> usize {
        self.inner
            .load()
            .as_ref()
            .as_ref()
            .map(|e| e.rule_count())
            .unwrap_or(0)
    }

    fn compiled_pattern_size_bytes(&self) -> usize {
        self.inner
            .load()
            .as_ref()
            .as_ref()
            .map(|e| e.compiled_pattern_size_bytes())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LayeredEngine;
    use crate::manifest::{Action, DefaultOnTimeout, Severity};

    fn rule(id: &str, pattern: &str, severity: Severity) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action: Action::Block,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            fail_closed: None,
            timeout_seconds: None,
            default_on_timeout: DefaultOnTimeout::Block,
        }
    }

    fn veng(id: &str, pattern: &str) -> VectorscanEngine {
        VectorscanEngine::compile(vec![rule(id, pattern, Severity::Critical)]).unwrap()
    }

    /// 指定严重度的单规则引擎（用于区分 fail-closed / 非 fail-closed 系统命中）。
    fn veng_sev(id: &str, pattern: &str, severity: Severity) -> VectorscanEngine {
        VectorscanEngine::compile(vec![rule(id, pattern, severity)]).unwrap()
    }

    /// 空集 fail-safe：无规则包时 scan 返回空，rule_count = 0，has_rules = false。
    #[test]
    fn empty_engine_is_fail_safe_passthrough() {
        let sys = SystemEngine::empty();
        assert!(!sys.has_rules(), "空集 has_rules 应为 false");
        assert_eq!(sys.rule_count(), 0);
        assert_eq!(sys.engine_name(), "system");
        let hits = sys.scan(b"sk-ant-api03-anything dangerous").unwrap();
        assert!(hits.is_empty(), "空集应透传不检测，返回空命中: {hits:?}");
        assert!(sys.rules_snapshot().is_empty());
    }

    /// 空集 scan_with_context 返回空报告（rule_count = 0），不 panic。
    #[test]
    fn empty_engine_scan_with_context_empty_report() {
        let sys = SystemEngine::empty();
        let req = ScanRequest {
            bytes: b"anything",
            direction: super::super::Direction::Outbound,
            protocol: super::super::Protocol::Anthropic,
            content_kind: super::super::ContentKind::RequestBody,
            tool_name: None,
            source_agent: None,
            caller_exe: None,
        };
        let report = sys.scan_with_context(req).unwrap();
        assert!(report.hits.is_empty());
        assert_eq!(report.rule_count, 0);
        assert_eq!(report.engine_name, "system");
    }

    /// Default impl 等同 empty。
    #[test]
    fn default_is_empty() {
        let sys = SystemEngine::default();
        assert!(!sys.has_rules());
    }

    /// 装包后正常检测：has_rules = true，命中规则。
    #[test]
    fn loaded_engine_detects() {
        let sys = SystemEngine::new(Some(veng("OUT-01", r"secret_key")));
        assert!(sys.has_rules());
        assert_eq!(sys.rule_count(), 1);
        let hits = sys.scan(b"leaking secret_key here").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-01");
        assert_eq!(sys.rules_snapshot().len(), 1);
    }

    /// swap_system 原子热替换：空 → 装包 → 换包 → 卸包，scan 立即看到新状态。
    #[test]
    fn swap_system_hot_replaces() {
        // 初始空集
        let sys = SystemEngine::empty();
        assert!(sys.scan(b"v1_pattern v2_pattern").unwrap().is_empty());

        // swap 装入 v1
        sys.swap_system(Some(veng("SYS-V1", r"v1_pattern")));
        assert!(sys.has_rules());
        let h1 = sys.scan(b"hit v1_pattern now").unwrap();
        assert!(
            h1.iter().any(|h| h.rule_id == "SYS-V1"),
            "v1 应命中: {h1:?}"
        );

        // swap 换到 v2，v1 不再命中
        sys.swap_system(Some(veng("SYS-V2", r"v2_pattern")));
        let h2 = sys.scan(b"hit v2_pattern now").unwrap();
        assert!(
            h2.iter().any(|h| h.rule_id == "SYS-V2"),
            "v2 应命中: {h2:?}"
        );
        let h2_on_v1 = sys.scan(b"hit v1_pattern now").unwrap();
        assert!(
            !h2_on_v1.iter().any(|h| h.rule_id == "SYS-V1"),
            "换包后 v1 不应命中: {h2_on_v1:?}"
        );

        // swap None 卸包 → 回到空集 fail-safe
        sys.swap_system(None);
        assert!(!sys.has_rules());
        assert!(
            sys.scan(b"hit v2_pattern now").unwrap().is_empty(),
            "卸包后应回到空集透传"
        );
    }

    /// SystemEngine 满足 MatchEngine bound，可作 LayeredEngine 的 S 参数（阶段 C 前置验证）。
    ///
    /// 同时验证空系统层不阻断 user 层：系统空集时 user 规则仍正常评估。
    #[test]
    fn usable_as_layered_system_layer() {
        // SYS-A 用 High 严重度（非 fail-closed），命中后不短路，继续合并 user 命中。
        let sys = SystemEngine::new(Some(veng_sev("SYS-A", r"system_hit", Severity::High)));
        let user = VectorscanEngine::compile(vec![rule("MY-RULE", r"user_hit", Severity::Medium)])
            .unwrap();
        let layered = LayeredEngine::new(sys, Some(user));

        // 系统规则命中（SYS-A 非 fail-closed）→ 合并 user 命中
        let hits = layered.scan(b"system_hit and user_hit").unwrap();
        assert!(
            hits.iter().any(|h| h.rule_id == "SYS-A"),
            "系统层应命中: {hits:?}"
        );
        assert!(
            hits.iter().any(|h| h.rule_id == "MY-RULE"),
            "用户层应合并: {hits:?}"
        );
    }

    /// 空系统层 + LayeredEngine：系统无包时 user 规则仍生效（fail-safe 不误伤 user 层）。
    #[test]
    fn empty_system_layer_still_evaluates_user() {
        let sys = SystemEngine::empty();
        let user = VectorscanEngine::compile(vec![rule("MY-RULE", r"user_only", Severity::Medium)])
            .unwrap();
        let layered = LayeredEngine::new(sys, Some(user));
        let hits = layered.scan(b"user_only here").unwrap();
        assert_eq!(hits.len(), 1, "系统空集时 user 规则应正常命中: {hits:?}");
        assert_eq!(hits[0].rule_id, "MY-RULE");
    }

    /// swap_system 期间并发 scan 不阻塞、不 panic（ArcSwap lock-free 保证，对称 user 层测试）。
    #[test]
    fn swap_does_not_block_concurrent_reads() {
        use std::thread;

        let sys = Arc::new(SystemEngine::new(Some(veng("SYS-INIT", r"init_data"))));
        let sys_read = Arc::clone(&sys);
        let sys_swap = Arc::clone(&sys);

        let reader = thread::spawn(move || {
            for _ in 0..200 {
                let _ = sys_read.scan(b"init_data swap_data").unwrap();
            }
        });

        let swapper = thread::spawn(move || {
            for i in 0..10u32 {
                if i % 2 == 0 {
                    sys_swap.swap_system(Some(veng("SYS-SWAP", r"swap_data")));
                } else {
                    sys_swap.swap_system(None);
                }
            }
        });

        reader.join().expect("reader 线程不应 panic");
        swapper.join().expect("swapper 线程不应 panic");
    }
}
