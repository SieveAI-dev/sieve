//! 可热替换的系统规则引擎。
//!
//! # 为什么需要 SystemEngine
//!
//! 早期系统规则在 daemon 启动时一次性编译为固定的 [`VectorscanEngine`]，
//! 生命周期不可变（加载失败即 exit）。为支持「规则经签名包分发 + 运行时热替换 +
//! 无包时引擎仍可独立构建运行供审计」，把系统层也包成 [`arc_swap::ArcSwap`]，
//! 对称于 [`super::LayeredEngine`] 的 user 层热替换模式。规则包通过更新通道下发。
//!
//! 未加载有效规则时保留控制面，但扫描返回错误，上层必须拒绝未检测流量。
//! 热更新失败由调用方保留上一版引擎；成功更新时，扫描与命中元数据共用同一快照。

use super::{MatchEngine, MatchHit, ScanReport, ScanRequest, VectorscanEngine};
use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use arc_swap::ArcSwap;
use std::sync::Arc;

/// 可原子热替换的系统规则引擎。
///
/// 内部 `ArcSwap<Option<Arc<VectorscanEngine>>>`，对称于 [`super::LayeredEngine`] 的 user 层：
/// - `None`：无规则包（引擎独立运行正常态）= 空规则集（扫描返回错误）。
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
    /// scan 始终返回错误，daemon 拒绝需要检测的请求。上层应据 [`SystemEngine::has_rules`]
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
    /// `false` = 空集（扫描返回错误），上层据此提示用户「未加载规则包」。
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
        // 缺少系统规则时不能将“无法检测”当成“未命中”。
        match self.inner.load().as_ref().as_ref() {
            Some(e) => e.scan(input),
            None => Err(SieveRulesError::Engine(
                "system rules unavailable; refusing uninspected traffic".into(),
            )),
        }
    }

    fn scan_with_context(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport> {
        // 委托给当前持有的 VectorscanEngine；无包时返回空报告（rule_count = 0）。
        match self.inner.load().as_ref().as_ref() {
            Some(e) => e.scan_with_context(req),
            None => Err(SieveRulesError::Engine(
                "system rules unavailable; refusing uninspected traffic".into(),
            )),
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

    /// 空集扫描返回错误，规则数为零。
    #[test]
    fn empty_engine_refuses_uninspected_traffic() {
        let sys = SystemEngine::empty();
        assert!(!sys.has_rules(), "空集 has_rules 应为 false");
        assert_eq!(sys.rule_count(), 0);
        assert_eq!(sys.engine_name(), "system");
        assert!(sys.scan(b"sk-ant-api03-anything dangerous").is_err());
        assert!(sys.rules_snapshot().is_empty());
    }

    /// 上下文扫描也必须拒绝空规则。
    #[test]
    fn empty_engine_context_scan_refuses_traffic() {
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
        assert!(sys.scan_with_context(req).is_err());
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
        assert!(sys.scan(b"v1_pattern v2_pattern").is_err());

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
            sys.scan(b"hit v2_pattern now").is_err(),
            "卸包后必须拒绝扫描"
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

    /// 用户规则不能替代缺失的系统保护规则。
    #[test]
    fn user_rules_cannot_replace_missing_system_rules() {
        let sys = SystemEngine::empty();
        let user = VectorscanEngine::compile(vec![rule("MY-RULE", r"user_only", Severity::Medium)])
            .unwrap();
        let layered = LayeredEngine::new(sys, Some(user));
        assert!(layered.scan(b"user_only here").is_err());
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
                let _ = sys_read.scan(b"init_data swap_data");
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
