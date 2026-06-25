//! Critical 规则强制 fail-closed 的运行时分类注册表（双层防御 + 处置矩阵）。
//!
//! ## 语义说明
//!
//! 规则的 fail-closed / hook / gui 分类**不再硬编码 ID 名单**，而是下沉为规则自身字段
//! （[`RuleEntry::effective_fail_closed`] / [`RuleEntry::effective_disposition`]），
//! 由引擎在加载规则时调 [`register_rules`] 构建进程级运行时注册表。本模块只保留**机制**：
//!
//! - fail-closed：不可关闭、不可永久白名单、dry-run 仍 enforce 的规则集合（所有 Critical + 显式标注的少数高危规则）。Hook 类的 fail-closed 由 sieve-hook 侧实现，代理侧同样不允许绕过。
//! - hook（disposition=HookTerminal）：命中后写 IPC pending file，由 sieve-hook 在 PreToolUse 阶段拦截。
//! - gui（disposition=GuiPopup）：命中后 hold SSE 流并通过 IPC 弹出 GUI 等待决策。
//!
//! 注册表为 accumulate 语义（[`register_rules`] 只增不删）：多方向引擎分别注册其规则；
//! 热替换装入新规则后旧 fail-closed ID 仍保留——这只会**过度保护**（已删规则不会再命中），
//! 对安全无害。无规则包时注册表为空，`is_fail_closed` 恒 `false`（与空集 fail-safe 一致）。
//!
//! 变更分类逻辑需走架构决策评审（fail-closed 语义 + disposition 矩阵）。

use crate::manifest::{Action, Disposition, RuleEntry};
use arc_swap::ArcSwap;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

/// 规则运行时分类注册表（fail-closed / hook / gui 三类 ID 集合）。
///
/// 由 [`register_rules`] 从已加载规则的字段派生，替代历史硬编码 ID 名单。
/// 纯数据结构，可独立构建与测试（不触全局状态）。
#[derive(Debug, Clone, Default)]
pub struct RuleClassRegistry {
    /// fail-closed 规则 ID。
    pub fail_closed: HashSet<String>,
    /// disposition=HookTerminal 规则 ID。
    pub hook: HashSet<String>,
    /// disposition=GuiPopup 规则 ID。
    pub gui: HashSet<String>,
}

impl RuleClassRegistry {
    /// 从规则集派生分类（accumulate 进 `self`）。
    pub fn extend_from_rules<'a>(&mut self, rules: impl IntoIterator<Item = &'a RuleEntry>) {
        for rule in rules {
            if rule.effective_fail_closed() {
                self.fail_closed.insert(rule.id.clone());
            }
            match rule.effective_disposition() {
                Disposition::HookTerminal => {
                    self.hook.insert(rule.id.clone());
                }
                Disposition::GuiPopup => {
                    self.gui.insert(rule.id.clone());
                }
                _ => {}
            }
        }
    }

    /// 从规则集构建一个新注册表。
    pub fn from_rules<'a>(rules: impl IntoIterator<Item = &'a RuleEntry>) -> Self {
        let mut r = Self::default();
        r.extend_from_rules(rules);
        r
    }
}

/// 进程级全局注册表（lock-free read via [`ArcSwap`]，初始为空）。
fn registry() -> &'static ArcSwap<RuleClassRegistry> {
    static R: OnceLock<ArcSwap<RuleClassRegistry>> = OnceLock::new();
    R.get_or_init(|| ArcSwap::from_pointee(RuleClassRegistry::default()))
}

/// 将一批规则的分类注册进全局注册表（accumulate，只增不删）。
///
/// 引擎在 [`crate::engine::VectorscanEngine::compile`] 编译规则时调用；
/// daemon 启动 / 热替换装入新规则后调用即可让 `is_fail_closed` 等查询生效。
///
/// 用 [`ArcSwap::rcu`]（compare-and-retry）做原子的 read-modify-write：并发注册不会丢更新
/// （读路径 `is_fail_closed` 等仍是 lock-free）。入参先收集成快照以支持 rcu 闭包重试。
pub fn register_rules<'a>(rules: impl IntoIterator<Item = &'a RuleEntry>) {
    let incoming: Vec<&RuleEntry> = rules.into_iter().collect();
    registry().rcu(|cur| {
        let mut next = RuleClassRegistry::clone(cur);
        next.extend_from_rules(incoming.iter().copied());
        next
    });
}

/// 用给定注册表整体替换全局注册表（测试 / 显式安装用）。
pub fn install_registry(reg: RuleClassRegistry) {
    registry().store(Arc::new(reg));
}

/// 检查给定 rule_id 是否 fail-closed（查运行时注册表）。
pub fn is_fail_closed(rule_id: &str) -> bool {
    registry().load().fail_closed.contains(rule_id)
}

/// 返回当前 fail-closed 规则 ID 集合的快照（克隆）。
///
/// 供需要枚举系统 Critical ID 的场景（如用户规则 lint 防影射系统规则）。
pub fn fail_closed_snapshot() -> HashSet<String> {
    registry().load().fail_closed.clone()
}

/// 检查给定 rule_id 是否为 HookTerminal 处置规则（查运行时注册表）。
pub fn is_hook_rule(rule_id: &str) -> bool {
    registry().load().hook.contains(rule_id)
}

/// 检查给定 rule_id 是否为 GuiPopup 处置规则（查运行时注册表）。
pub fn is_gui_rule(rule_id: &str) -> bool {
    registry().load().gui.contains(rule_id)
}

/// 强制覆盖 action：fail-closed 规则一律返回 Block。
pub fn enforce_action(rule_id: &str, requested: Action) -> Action {
    if is_fail_closed(rule_id) {
        Action::Block
    } else {
        requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{DefaultOnTimeout, Severity};

    /// 构造测试规则；`fc` = fail_closed 显式值（None 走 severity 推断）。
    fn rule(
        id: &str,
        severity: Severity,
        disp: Option<Disposition>,
        fc: Option<bool>,
    ) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            severity,
            action: Action::Block,
            pattern: "x".into(),
            description: id.into(),
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: disp,
            fail_closed: fc,
            timeout_seconds: None,
            default_on_timeout: DefaultOnTimeout::Block,
        }
    }

    // ---- RuleClassRegistry 纯逻辑（不触全局，确定性）----

    /// Critical 规则推断 fail-closed；非 Critical 默认非 fail-closed。
    #[test]
    fn registry_infers_fail_closed_from_severity() {
        let rules = [
            rule("CRIT", Severity::Critical, None, None),
            rule("HIGH", Severity::High, None, None),
            rule("LOW", Severity::Low, None, None),
        ];
        let reg = RuleClassRegistry::from_rules(&rules);
        assert!(
            reg.fail_closed.contains("CRIT"),
            "Critical 应推断 fail-closed"
        );
        assert!(!reg.fail_closed.contains("HIGH"));
        assert!(!reg.fail_closed.contains("LOW"));
    }

    /// 显式 fail_closed 覆盖 severity 推断（高危但 fail-closed，如 OUT-09 / IN-GEN-06）。
    #[test]
    fn registry_explicit_fail_closed_overrides_severity() {
        let rules = [
            rule("HIGH-FC", Severity::High, None, Some(true)),
            rule("CRIT-NOFC", Severity::Critical, None, Some(false)),
        ];
        let reg = RuleClassRegistry::from_rules(&rules);
        assert!(
            reg.fail_closed.contains("HIGH-FC"),
            "显式 true 应 fail-closed"
        );
        assert!(
            !reg.fail_closed.contains("CRIT-NOFC"),
            "显式 false 应覆盖 Critical 推断"
        );
    }

    /// disposition 派生 hook / gui 分类。
    #[test]
    fn registry_derives_hook_and_gui_from_disposition() {
        let rules = [
            rule(
                "HOOK-R",
                Severity::Critical,
                Some(Disposition::HookTerminal),
                None,
            ),
            rule(
                "GUI-R",
                Severity::Critical,
                Some(Disposition::GuiPopup),
                None,
            ),
            rule("BAR-R", Severity::Low, Some(Disposition::StatusBar), None),
        ];
        let reg = RuleClassRegistry::from_rules(&rules);
        assert!(reg.hook.contains("HOOK-R"));
        assert!(!reg.gui.contains("HOOK-R"));
        assert!(reg.gui.contains("GUI-R"));
        assert!(!reg.hook.contains("GUI-R"));
        // status_bar 不进 hook/gui
        assert!(!reg.hook.contains("BAR-R") && !reg.gui.contains("BAR-R"));
    }

    /// extend_from_rules 累积多批规则（对称多方向引擎注册）。
    #[test]
    fn registry_extend_accumulates() {
        let mut reg = RuleClassRegistry::default();
        reg.extend_from_rules(&[rule("OUT", Severity::Critical, None, None)]);
        reg.extend_from_rules(&[rule("IN", Severity::Critical, None, None)]);
        assert!(reg.fail_closed.contains("OUT") && reg.fail_closed.contains("IN"));
    }

    // ---- 全局注册表 + enforce_action（用 install_registry 安装已知集合）----

    /// register_rules + is_fail_closed / enforce_action 联动。
    ///
    /// 用唯一前缀 ID 避免与同二进制其他测试经 compile 注册的真实规则 ID 冲突；
    /// 用 accumulate 的 register_rules（不用 REPLACE 语义的 install_registry，避免并行测试
    /// 互相清空全局注册表），断言覆盖 enforce_action 的 fail-closed 强制语义。
    #[test]
    fn global_register_and_enforce() {
        register_rules(&[rule(
            "CLTEST-FC-CRITICAL",
            Severity::Critical,
            Some(Disposition::GuiPopup),
            None,
        )]);

        assert!(is_fail_closed("CLTEST-FC-CRITICAL"));
        assert!(is_gui_rule("CLTEST-FC-CRITICAL"));
        // fail-closed 规则的 action 一律强制 Block
        assert_eq!(
            enforce_action("CLTEST-FC-CRITICAL", Action::Allow),
            Action::Block
        );
        // 未注册规则保持请求 action
        assert_eq!(
            enforce_action("CLTEST-DEFINITELY-ABSENT-9999", Action::Mark),
            Action::Mark
        );
    }
}
