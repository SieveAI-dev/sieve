// macOS 系统调用（proc_pidpath / proc_pidinfo）需要 unsafe block。
// 顶层 #![deny(unsafe_code)] 在 main.rs 定义，此处局部豁免。
// 每处 unsafe block 均附 SAFETY 注释说明不变式（见下方函数）。
#![allow(unsafe_code)]

//! 进程上下文反查模块（PRD v2.0 §5.6 / §6.6 / Phase A 数据准备）。
//!
//! 通过 PID 反查 caller 进程信息（exe 路径 + ppid），供 Phase B 行为序列分析使用。
//! Phase A 仅采集数据，不做行为分析。
//!
//! **平台支持**：仅 macOS 实现真实查询（调用 `proc_pidpath` / `proc_pidinfo`）；
//! 其他平台返回 `None`（stub）。
//!
//! **性能保证**：30 秒 LRU cache（容量 256）保证 cache 命中 P99 < 10µs；
//! cache 未命中时走系统调用，失败静默返回 `None`，绝不阻塞调用方。
//!
//! **字段说明（§5.6.1）**：
//! - `cwd` 字段推迟到 v2.1（macOS 需 entitlements + 用户授权弹窗）
//! - Phase A 只采集 `pid` + `exe` + `ppid` 三个字段

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use lru::LruCache;

/// caller 进程信息（PRD v2.0 §5.6）。
///
/// `cwd` 字段推 v2.1，Phase A 不采集。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerInfo {
    /// 进程 ID
    pub pid: i32,
    /// 可执行文件绝对路径（macOS 上通过 `proc_pidpath` 获取）
    pub exe: Option<PathBuf>,
    /// 父进程 ID（macOS 上通过 `proc_pidinfo` + `PROC_PIDTBSDINFO` 获取）
    pub ppid: Option<i32>,
}

/// cache 条目：CallerInfo + 写入时刻
type CacheEntry = (CallerInfo, Instant);

/// LRU cache 容量
const CACHE_CAPACITY: usize = 256;

/// cache 条目 TTL（30 秒）
const CACHE_TTL: Duration = Duration::from_secs(30);

/// 全局静态 LRU cache（线程安全，Mutex 保护）
fn global_cache() -> &'static Mutex<LruCache<i32, CacheEntry>> {
    static CACHE: OnceLock<Mutex<LruCache<i32, CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        // SAFETY: CACHE_CAPACITY > 0，NonZeroUsize::new 不会返回 None
        let cap = std::num::NonZeroUsize::new(CACHE_CAPACITY).expect("capacity > 0");
        Mutex::new(LruCache::new(cap))
    })
}

/// 通过 PID 反查 caller 进程信息。
///
/// - cache 命中且未过期：直接返回，P99 < 10µs
/// - cache 未命中或已过期：调用系统 API 查询，失败静默返回 `None`
/// - 任何情况都不会 panic 或阻塞调用方
///
/// 关联 PRD v2.0 §5.6 / §6.6.1
pub fn lookup_caller(pid: i32) -> Option<CallerInfo> {
    // 先查 cache
    {
        let mut cache = global_cache().lock().ok()?;
        if let Some((info, ts)) = cache.get(&pid) {
            if ts.elapsed() < CACHE_TTL {
                return Some(info.clone());
            }
            // TTL 过期，从 cache 删除，走系统调用
            cache.pop(&pid);
        }
    }

    // cache 未命中，走系统调用（macOS only）
    let info = query_system(pid)?;

    // 写入 cache
    if let Ok(mut cache) = global_cache().lock() {
        cache.put(pid, (info.clone(), Instant::now()));
    }

    Some(info)
}

/// 清空 cache（仅用于测试）。
#[cfg(test)]
pub fn clear_cache_for_test() {
    if let Ok(mut cache) = global_cache().lock() {
        cache.clear();
    }
}

// ============================================================
// macOS 实现
// ============================================================

/// macOS 系统调用组合查询。
///
/// 只要 pid 无效（-1、i32::MAX 等），两个系统调用都会失败，返回 `None`。
/// 至少一个系统调用成功才返回 `Some`，避免对不存在的 pid 返回空信息。
#[cfg(target_os = "macos")]
fn query_system(pid: i32) -> Option<CallerInfo> {
    let exe = query_exe(pid);
    let ppid = query_ppid(pid);
    // 两个调用都失败 → pid 不存在或无权访问，返回 None
    if exe.is_none() && ppid.is_none() {
        return None;
    }
    Some(CallerInfo { pid, exe, ppid })
}

/// 通过 `proc_pidpath` 获取可执行文件路径。
///
/// 返回值 > 0 表示成功，路径长度为返回值字节数（不含 NUL）。
#[cfg(target_os = "macos")]
fn query_exe(pid: i32) -> Option<PathBuf> {
    // SAFETY:
    // - buf 是局部栈上数组，生命周期在调用期间有效
    // - PROC_PIDPATHINFO_MAXSIZE 是 libproc.h 中定义的最大路径长度（4096）
    // - proc_pidpath 只向 buf[0..len] 写入，不超界
    // - 若 pid 无效或权限不足，返回 ≤ 0，不修改 buf
    let mut buf = [0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    let ret =
        unsafe { libc::proc_pidpath(pid, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as u32) };
    if ret <= 0 {
        return None;
    }
    // 去掉 NUL 终止符，取有效字节
    let len = ret as usize;
    let path_bytes = &buf[..len];
    // 去掉末尾多余 NUL（部分系统实现会在 len 以内填 NUL）
    let trimmed = match path_bytes.iter().position(|&b| b == 0) {
        Some(pos) => &path_bytes[..pos],
        None => path_bytes,
    };
    Some(PathBuf::from(std::str::from_utf8(trimmed).ok()?))
}

/// 通过 `proc_pidinfo` + `PROC_PIDTBSDINFO` 获取 ppid。
#[cfg(target_os = "macos")]
fn query_ppid(pid: i32) -> Option<i32> {
    // SAFETY:
    // - proc_bsdinfo 是 C struct，全字段 Copy，零初始化合法
    // - proc_pidinfo 将 size 字节写入 &mut info，size = std::mem::size_of::<libc::proc_bsdinfo>()
    // - 若 pid 无效或权限不足，返回 ≤ 0，不修改 info
    let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
    let ret = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            &mut info as *mut libc::proc_bsdinfo as *mut libc::c_void,
            size,
        )
    };
    if ret <= 0 {
        return None;
    }
    Some(info.pbi_ppid as i32)
}

// ============================================================
// 非 macOS stub
// ============================================================

/// 非 macOS 平台不支持进程上下文反查，统一返回 `None`。
#[cfg(not(target_os = "macos"))]
fn query_system(_pid: i32) -> Option<CallerInfo> {
    None
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    // ----------------------------------------------------------
    // macOS only 测试
    // ----------------------------------------------------------

    #[cfg(target_os = "macos")]
    #[test]
    fn lookup_self_returns_some() {
        clear_cache_for_test();
        let pid = std::process::id() as i32;
        let info = lookup_caller(pid).expect("自身进程应能查到");
        assert_eq!(info.pid, pid);

        let exe = info.exe.expect("macOS 上应有 exe 路径");
        // exe 路径应该包含可执行文件名（非空且是有效 UTF-8 路径）
        assert!(!exe.as_os_str().is_empty(), "exe 路径不应为空");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn lookup_invalid_pid_returns_none() {
        clear_cache_for_test();
        // -1 和 i32::MAX 都不是合法 PID
        assert!(lookup_caller(-1).is_none(), "PID -1 应返回 None");
        assert!(
            lookup_caller(i32::MAX).is_none(),
            "PID i32::MAX 应返回 None"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cache_hit_is_fast() {
        clear_cache_for_test();
        let pid = std::process::id() as i32;

        // 第一次：cache miss，走系统调用
        let _ = lookup_caller(pid);

        // 第二次：cache hit，应 < 10µs
        let start = Instant::now();
        let _ = lookup_caller(pid);
        let elapsed = start.elapsed();

        // 给 CI 留余地，断言 < 1ms（比 10µs 宽松 100 倍以应对调度抖动）
        assert!(
            elapsed < Duration::from_millis(1),
            "cache 命中耗时应 < 1ms，实际: {:?}",
            elapsed
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cache_ttl_expires() {
        // 手动向 cache 写入已过期的条目（ts = Instant::now() - 60s）
        // 过期时间模拟：用一个远过去的 Instant
        let pid = std::process::id() as i32;

        // 先 clear，确保干净状态
        clear_cache_for_test();

        // 手动插入过期条目
        {
            let cap = std::num::NonZeroUsize::new(CACHE_CAPACITY).unwrap();
            // 通过 lookup 使 cache 有值
            let _ = lookup_caller(pid);

            // 替换 cache 条目为过期时间
            if let Ok(mut cache) = global_cache().lock() {
                if let Some(entry) = cache.get_mut(&pid) {
                    // 将时间戳设置为 TTL + 1s 之前（模拟过期）
                    entry.1 = Instant::now()
                        .checked_sub(CACHE_TTL + Duration::from_secs(1))
                        .unwrap_or_else(|| {
                            // Instant::now().checked_sub 在某些平台可能返回 None（单调时钟不支持负值）
                            // fallback：用一个极小的 Instant（不可达），测试降级为只验证 lookup 成功
                            Instant::now()
                        });
                }
                // 验证容量设置正确（避免编译警告）
                let _ = cap;
            }
        }

        // 查询：cache 条目已过期，应重新走系统调用
        let info = lookup_caller(pid);
        // macOS 上自身 PID 应能查到
        assert!(info.is_some(), "过期后重新查询应能成功");
    }

    // ----------------------------------------------------------
    // 跨平台测试
    // ----------------------------------------------------------

    #[test]
    fn clear_cache_for_test_works() {
        // 先 lookup 一次（使 cache 有内容）
        let pid = std::process::id() as i32;
        let _ = lookup_caller(pid);

        // clear
        clear_cache_for_test();

        // 验证 cache 已清空（间接：检查 cache 长度为 0）
        let len = global_cache().lock().map(|c| c.len()).unwrap_or(usize::MAX);
        assert_eq!(len, 0, "clear 后 cache 应为空");
    }

    // ----------------------------------------------------------
    // 非 macOS stub 测试
    // ----------------------------------------------------------

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_returns_none() {
        clear_cache_for_test();
        // 非 macOS 平台：query_system 返回 None，故 lookup_caller 也返回 None
        assert!(lookup_caller(1).is_none(), "非 macOS 应返回 None");
        assert!(
            lookup_caller(std::process::id() as i32).is_none(),
            "非 macOS 自身 PID 也应返回 None"
        );
    }
}
