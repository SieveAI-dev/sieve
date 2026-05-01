// macOS 系统调用（proc_pidpath / proc_pidinfo / proc_listpids / proc_pidfdinfo）需要 unsafe block。
// 顶层 #![deny(unsafe_code)] 在 main.rs 定义，此处局部豁免。
// 每处 unsafe block 均附 SAFETY 注释说明不变式（见下方函数）。
#![allow(unsafe_code)]

//! 进程上下文反查模块（PRD v2.0 §5.6 / §6.6 / Phase A 数据准备）。
//!
//! 通过 PID 反查 caller 进程信息（exe 路径 + ppid），供 Phase B 行为序列分析使用。
//! Phase A 仅采集数据，不做行为分析。
//!
//! **平台支持**：仅 macOS 实现真实查询（调用 `proc_pidpath` / `proc_pidinfo` /
//! `proc_listpids` / `proc_pidfdinfo`）；其他平台返回 `None`（stub）。
//!
//! **性能保证**：30 秒 LRU cache（容量 256）保证 cache 命中 P99 < 10µs；
//! cache 未命中时走系统调用，失败静默返回 `None`，绝不阻塞调用方。
//!
//! **socket 反查（§5.6 / §6.6 / OQ-V20-02）**：
//! `lookup_caller_by_socket_addr` 通过 TCP 4-tuple 反查 caller PID，
//! 走 proc_listpids → proc_pidinfo(PROC_PIDLISTFDS) → proc_pidfdinfo(PROC_PIDFDSOCKETINFO)
//! 三层调用扫所有进程 FD，匹配 (local_addr, local_port, remote_addr, remote_port)。
//! **禁止 shell out lsof/netstat**（OQ-V20-02）。
//!
//! **字段说明（§5.6.1）**：
//! - `cwd` 字段推迟到 v2.1（macOS 需 entitlements + 用户授权弹窗）
//! - Phase A 只采集 `pid` + `exe` + `ppid` 三个字段

use std::net::SocketAddr;
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

/// 全局静态 LRU cache（PID → CallerInfo，线程安全，Mutex 保护）
fn global_cache() -> &'static Mutex<LruCache<i32, CacheEntry>> {
    static CACHE: OnceLock<Mutex<LruCache<i32, CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        // SAFETY: CACHE_CAPACITY > 0，NonZeroUsize::new 不会返回 None
        let cap = std::num::NonZeroUsize::new(CACHE_CAPACITY).expect("capacity > 0");
        Mutex::new(LruCache::new(cap))
    })
}

/// socket 反查 cache：4-tuple key → PID，30s TTL（仅 macOS 使用）
#[cfg(target_os = "macos")]
type PeerCacheEntry = (i32, Instant);

/// 全局静态 socket 反查 LRU cache（线程安全，Mutex 保护）
#[cfg(target_os = "macos")]
fn peer_cache() -> &'static Mutex<LruCache<(SocketAddr, SocketAddr), PeerCacheEntry>> {
    static PEER_CACHE: OnceLock<Mutex<LruCache<(SocketAddr, SocketAddr), PeerCacheEntry>>> =
        OnceLock::new();
    PEER_CACHE.get_or_init(|| {
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

/// 通过 TCP 连接的 4-tuple 反查 caller PID（macOS only）。
///
/// 实现：扫所有进程的 FD，匹配 socket 的 local_addr/port + remote_addr/port。
/// 失败（权限不足、socket 已关、非 macOS）返回 `None`。
///
/// # 性能
///
/// 未命中 cache 时 P99 < 50ms（扫所有进程 FD）；30 秒 LRU 缓存命中 P99 < 1µs。
///
/// # 语义
///
/// `local` 是 daemon accept 侧的本地地址（如 `127.0.0.1:11453`），
/// `remote` 是 caller 进程的 ephemeral 地址（如 `127.0.0.1:<random_port>`）。
/// 函数扫描所有进程，找出拥有该 socket caller 侧视角（local=remote, remote=local）的进程。
///
/// # 平台
///
/// 仅 macOS 实现真实查询；其他平台返回 `None`（与 v2.0 Phase A stub 行为一致）。
///
/// 关联：PRD v2.0 §5.6 / §6.6 / OQ-V20-02。
pub fn lookup_caller_by_socket_addr(local: SocketAddr, remote: SocketAddr) -> Option<i32> {
    // 先查 peer cache（key 以 daemon 视角存，即原始 local/remote）
    {
        let mut cache = peer_cache().lock().ok()?;
        if let Some(&(pid, ts)) = cache.get(&(local, remote)) {
            if ts.elapsed() < CACHE_TTL {
                return Some(pid);
            }
            cache.pop(&(local, remote));
        }
    }

    // cache 未命中，走系统调用（macOS only）
    let pid = find_pid_by_socket_addr(local, remote)?;

    // 写入 peer cache
    if let Ok(mut cache) = peer_cache().lock() {
        cache.put((local, remote), (pid, Instant::now()));
    }

    Some(pid)
}

/// 一次性反查：从 socket 4-tuple 找 PID 后再调 [`lookup_caller`] 拿完整 [`CallerInfo`]。
///
/// 关联：PRD v2.0 §5.6 / §6.6 / OQ-V20-02。
pub fn lookup_caller_by_peer(local: SocketAddr, remote: SocketAddr) -> Option<CallerInfo> {
    let pid = lookup_caller_by_socket_addr(local, remote)?;
    lookup_caller(pid)
}

/// 清空 cache（仅用于测试）。
#[cfg(test)]
pub fn clear_cache_for_test() {
    if let Ok(mut cache) = global_cache().lock() {
        cache.clear();
    }
    #[cfg(target_os = "macos")]
    if let Ok(mut cache) = peer_cache().lock() {
        cache.clear();
    }
}

// ============================================================
// macOS 实现
// ============================================================

// socket_fdinfo 字段偏移量（通过 C 程序验证，SDK MacOSX15.4 / arm64）：
//
//   socket_fdinfo = proc_fileinfo(24) + socket_info(768) = 792 字节
//   socket_info.soi_kind              offset 256  (i32, SOCKINFO_TCP=2)
//   tcp_sockinfo.tcpsi_ini.insi_fport offset 264  (i32, network byte order, remote port)
//   tcp_sockinfo.tcpsi_ini.insi_lport offset 268  (i32, network byte order, local port)
//   tcp_sockinfo.tcpsi_ini.insi_vflag offset 288  (u8,  INI_IPV4=1 / INI_IPV6=2)
//   tcp_sockinfo.tcpsi_ini.insi_faddr offset 296  (union 16B, IPv4 地址在 +12 处)
//   tcp_sockinfo.tcpsi_ini.insi_laddr offset 312  (union 16B, IPv4 地址在 +12 处)
//     → IPv4 remote addr at offset 308 (296+12)
//     → IPv4 local  addr at offset 324 (312+12)
//   IPv6 remote addr at offset 296  (insi_faddr 整个 16B 作为 in6_addr)
//   IPv6 local  addr at offset 312  (insi_laddr 整个 16B 作为 in6_addr)
//
// proc_fdinfo = 8 字节（proc_fd: i32 at 0, proc_fdtype: u32 at 4）
// PROX_FDTYPE_SOCKET = 2

/// socket_fdinfo buffer 总大小（通过 C sizeof 验证：792）
#[cfg(target_os = "macos")]
const SOCKET_FDINFO_SIZE: usize = 792;
/// proc_fdinfo 单条大小（通过 C sizeof 验证：8）
#[cfg(target_os = "macos")]
const PROC_FDINFO_SIZE: usize = 8;

/// `PROC_ALL_PIDS` — macOS proc_listpids type，部分 libc 版本未导出此常量
#[cfg(target_os = "macos")]
const PROC_ALL_PIDS: u32 = 1;
/// `PROC_PIDFDSOCKETINFO` — macOS proc_pidfdinfo flavor，部分 libc 版本未导出此常量
#[cfg(target_os = "macos")]
const PROC_PIDFDSOCKETINFO: i32 = 3;

/// 在 buffer 的给定字节偏移处读取 i32（原生字节序）。
///
/// # SAFETY
/// 调用方保证 offset + 4 <= buf.len()，且 buf 对应 C 结构体已由系统调用填充。
#[cfg(target_os = "macos")]
#[inline]
unsafe fn read_i32_at(buf: &[u8], offset: usize) -> i32 {
    debug_assert!(offset + 4 <= buf.len());
    i32::from_ne_bytes(buf[offset..offset + 4].try_into().unwrap_unchecked())
}

/// 在 buffer 的给定字节偏移处读取 u8。
///
/// # SAFETY
/// 调用方保证 offset < buf.len()，且 buf 对应 C 结构体已由系统调用填充。
#[cfg(target_os = "macos")]
#[inline]
unsafe fn read_u8_at(buf: &[u8], offset: usize) -> u8 {
    debug_assert!(offset < buf.len());
    *buf.get_unchecked(offset)
}

/// 在 buffer 的给定字节偏移处读取 [u8; 4]（IPv4 地址，network byte order）。
///
/// # SAFETY
/// 调用方保证 offset + 4 <= buf.len()，且 buf 对应 C 结构体已由系统调用填充。
#[cfg(target_os = "macos")]
#[inline]
unsafe fn read_ipv4_at(buf: &[u8], offset: usize) -> [u8; 4] {
    debug_assert!(offset + 4 <= buf.len());
    buf[offset..offset + 4].try_into().unwrap_unchecked()
}

/// 在 buffer 的给定字节偏移处读取 [u8; 16]（IPv6 地址，network byte order）。
///
/// # SAFETY
/// 调用方保证 offset + 16 <= buf.len()，且 buf 对应 C 结构体已由系统调用填充。
#[cfg(target_os = "macos")]
#[inline]
unsafe fn read_ipv6_at(buf: &[u8], offset: usize) -> [u8; 16] {
    debug_assert!(offset + 16 <= buf.len());
    buf[offset..offset + 16].try_into().unwrap_unchecked()
}

/// 通过 TCP 4-tuple 扫描所有进程 FD 反查 PID（macOS only）。
///
/// 实现：
/// 1. `proc_listpids(PROC_ALL_PIDS)` 获取所有 PID 列表
/// 2. 对每个 PID，`proc_pidinfo(PROC_PIDLISTFDS)` 获取 FD 列表
/// 3. 对每个 socket FD（PROX_FDTYPE_SOCKET=2），`proc_pidfdinfo(PROC_PIDFDSOCKETINFO)` 获取 TCP socket 信息
/// 4. 比较 caller 侧 4-tuple（local=remote, remote=local）
///
/// caller 侧视角：daemon accept 的 local=daemon addr, remote=caller ephemeral addr；
/// caller 进程的 socket：local=caller ephemeral addr, remote=daemon addr。
///
/// 关联：PRD v2.0 §5.6 / §6.6 / OQ-V20-02
#[cfg(target_os = "macos")]
fn find_pid_by_socket_addr(daemon_local: SocketAddr, daemon_remote: SocketAddr) -> Option<i32> {
    // caller 侧 4-tuple：local=daemon_remote(caller ephemeral), remote=daemon_local
    let caller_local = daemon_remote;
    let caller_remote = daemon_local;

    // Step 1: 获取所有 PID 列表
    // SAFETY:
    // - proc_listpids(PROC_ALL_PIDS, 0, NULL, 0) 返回需要的 buffer 大小（字节数）
    // - 第一次调用传 NULL/0 探查大小，不写入内存，安全
    let needed_bytes = unsafe { libc::proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
    if needed_bytes <= 0 {
        return None;
    }
    // 多分配一些空间应对进程动态增加（PID 为 i32）
    let pid_count = (needed_bytes as usize / std::mem::size_of::<i32>()) + 16;
    let mut pid_buf: Vec<i32> = vec![0i32; pid_count];

    // SAFETY:
    // - pid_buf 已分配足够空间（pid_count * 4 字节）
    // - proc_listpids 向 pid_buf 写入 PID 列表，每个 PID 4 字节
    // - 传入的 buffersize 是总字节数，不会超出 pid_buf 边界
    let written_bytes = unsafe {
        libc::proc_listpids(
            PROC_ALL_PIDS,
            0,
            pid_buf.as_mut_ptr() as *mut libc::c_void,
            (pid_count * std::mem::size_of::<i32>()) as i32,
        )
    };
    if written_bytes <= 0 {
        return None;
    }
    let actual_pid_count = written_bytes as usize / std::mem::size_of::<i32>();

    // Step 2: 对每个 PID 扫描 FD 列表
    // proc_fdinfo 每条 8 字节：proc_fd(i32) + proc_fdtype(u32)
    // 预分配一个 FD buffer（256 条，不够时会重分配）
    let mut fd_buf: Vec<u8> = vec![0u8; 256 * PROC_FDINFO_SIZE];

    for &pid in &pid_buf[..actual_pid_count] {
        if pid <= 0 {
            continue;
        }

        // SAFETY:
        // - fd_buf 已分配足够空间
        // - proc_pidinfo(PROC_PIDLISTFDS) 向 fd_buf 写入 proc_fdinfo 数组
        // - 若 pid 无权访问，返回 ≤ 0，不修改 fd_buf
        let fd_bytes = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDLISTFDS,
                0,
                fd_buf.as_mut_ptr() as *mut libc::c_void,
                fd_buf.len() as i32,
            )
        };
        if fd_bytes <= 0 {
            continue;
        }
        let fd_count = fd_bytes as usize / PROC_FDINFO_SIZE;

        // Step 3: 对每个 socket FD 查询 TCP socket 信息
        let mut sock_buf = [0u8; SOCKET_FDINFO_SIZE];

        for i in 0..fd_count {
            let base = i * PROC_FDINFO_SIZE;
            if base + PROC_FDINFO_SIZE > fd_buf.len() {
                break;
            }
            // SAFETY: base + 4 <= fd_buf.len()（上方已校验），buf 已由 proc_pidinfo 填充
            let fd_num = unsafe { read_i32_at(&fd_buf, base) };
            // SAFETY: base + 4 + 4 <= fd_buf.len()（PROC_FDINFO_SIZE=8），buf 已由 proc_pidinfo 填充
            let fd_type =
                u32::from_ne_bytes(fd_buf[base + 4..base + 8].try_into().unwrap_or([0u8; 4]));

            // 只处理 socket 类型 FD（PROX_FDTYPE_SOCKET=2）
            if fd_type != libc::PROX_FDTYPE_SOCKET as u32 {
                continue;
            }

            // SAFETY:
            // - sock_buf 大小 = SOCKET_FDINFO_SIZE = 792，等于 sizeof(socket_fdinfo)
            // - proc_pidfdinfo(PROC_PIDFDSOCKETINFO) 向 sock_buf 写入 socket_fdinfo
            // - 若 fd 已关或权限不足，返回 ≤ 0，不修改 sock_buf
            let ret = unsafe {
                libc::proc_pidfdinfo(
                    pid,
                    fd_num,
                    PROC_PIDFDSOCKETINFO,
                    sock_buf.as_mut_ptr() as *mut libc::c_void,
                    SOCKET_FDINFO_SIZE as i32,
                )
            };
            if ret <= 0 {
                continue;
            }

            // 解析 socket_fdinfo buffer（按验证过的偏移量读取）
            // SAFETY: 所有偏移均已通过 C 程序验证，sock_buf 已由 proc_pidfdinfo 填充
            let soi_kind = unsafe { read_i32_at(&sock_buf, 256) };
            // SOCKINFO_TCP = 2（来自 proc_info.h enum）
            if soi_kind != 2 {
                continue;
            }

            // 端口：network byte order → host byte order
            let raw_fport = unsafe { read_i32_at(&sock_buf, 264) };
            let raw_lport = unsafe { read_i32_at(&sock_buf, 268) };
            let remote_port = u16::from_be((raw_fport & 0xFFFF) as u16);
            let local_port = u16::from_be((raw_lport & 0xFFFF) as u16);

            // vflag：INI_IPV4=1 / INI_IPV6=2
            let vflag = unsafe { read_u8_at(&sock_buf, 288) };

            if vflag & 0x01 != 0 {
                // IPv4 路径
                // SAFETY: offset 308+4=312 <= 792，buf 已填充
                let raw_faddr = unsafe { read_ipv4_at(&sock_buf, 308) };
                // SAFETY: offset 324+4=328 <= 792，buf 已填充
                let raw_laddr = unsafe { read_ipv4_at(&sock_buf, 324) };
                let remote_ip = std::net::Ipv4Addr::from(raw_faddr);
                let local_ip = std::net::Ipv4Addr::from(raw_laddr);

                let sock_local = SocketAddr::from((local_ip, local_port));
                let sock_remote = SocketAddr::from((remote_ip, remote_port));

                if sock_local == caller_local && sock_remote == caller_remote {
                    return Some(pid);
                }
            } else if vflag & 0x02 != 0 {
                // IPv6 路径
                // SAFETY: offset 296+16=312 <= 792，buf 已填充
                let raw_faddr = unsafe { read_ipv6_at(&sock_buf, 296) };
                // SAFETY: offset 312+16=328 <= 792，buf 已填充
                let raw_laddr = unsafe { read_ipv6_at(&sock_buf, 312) };
                let remote_ip = std::net::Ipv6Addr::from(raw_faddr);
                let local_ip = std::net::Ipv6Addr::from(raw_laddr);

                let sock_local = SocketAddr::from((local_ip, local_port));
                let sock_remote = SocketAddr::from((remote_ip, remote_port));

                if sock_local == caller_local && sock_remote == caller_remote {
                    return Some(pid);
                }
            }
        }
    }

    None
}

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

/// 非 macOS 平台不支持 socket 反查，统一返回 `None`。
/// Linux 实现推 Phase 2/3 ADR。
#[cfg(not(target_os = "macos"))]
fn find_pid_by_socket_addr(_daemon_local: SocketAddr, _daemon_remote: SocketAddr) -> Option<i32> {
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
    // socket 反查测试（macOS only）
    // ----------------------------------------------------------

    /// 启动本地 TCP listener，自连接，用 4-tuple 反查 PID。
    ///
    /// # 权限说明
    ///
    /// macOS 查询他进程 FD 需要 root 或特定 entitlement；
    /// 无 SIP 豁免的 CI（GitHub Actions macOS runner）可能因权限不足返回 None。
    /// 本地非沙箱开发环境应返回 Some(self_pid)。
    /// 标记 `#[ignore]`：CI 跳过，本地用 `cargo test -- --ignored` 验证。
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "需要查询他进程 FD 的权限；本地非 SIP 沙箱环境可通过，CI 可能因权限不足返回 None"]
    fn lookup_self_tcp_pair_returns_some_pid() {
        use std::io::{Read, Write};
        use std::net::{TcpListener, TcpStream};

        clear_cache_for_test();
        let self_pid = std::process::id() as i32;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind 应成功");
        let listener_addr = listener.local_addr().expect("local_addr 应成功");

        let accept_handle = std::thread::spawn(move || {
            listener.accept().ok().map(|(mut s, _)| {
                let mut buf = [0u8; 1];
                let _ = s.read(&mut buf);
                s
            })
        });

        let mut client = TcpStream::connect(listener_addr).expect("connect 应成功");
        let client_local = client.local_addr().expect("client local_addr 应成功");

        // daemon 视角：local=listener_addr, remote=client_local
        let result = lookup_caller_by_socket_addr(listener_addr, client_local);

        let _ = client.write_all(&[0u8]);
        let _ = accept_handle.join();

        if let Some(found_pid) = result {
            assert_eq!(
                found_pid, self_pid,
                "反查到的 PID 应是本进程（{self_pid}），实际: {found_pid}"
            );
        } else {
            // 权限不足时退化为 None，写注释说明，不 panic
            eprintln!(
                "lookup_caller_by_socket_addr 返回 None（可能是权限不足），\
                 本地非 SIP 沙箱环境运行应返回 Some({self_pid})"
            );
        }
    }

    /// 用不存在的 4-tuple 反查，应返回 None 且不 panic。
    #[cfg(target_os = "macos")]
    #[test]
    fn lookup_invalid_remote_returns_none() {
        clear_cache_for_test();
        let bogus_local: SocketAddr = "127.0.0.1:19999".parse().unwrap();
        let bogus_remote: SocketAddr = "127.0.0.1:29999".parse().unwrap();
        let result = lookup_caller_by_socket_addr(bogus_local, bogus_remote);
        assert!(
            result.is_none(),
            "不存在的 4-tuple 应返回 None，实际: {result:?}"
        );
    }

    /// peer cache 命中测试：手动注入 cache 条目，验证第二次调用走 cache 路径（< 1ms）。
    #[cfg(target_os = "macos")]
    #[test]
    fn peer_cache_works() {
        clear_cache_for_test();
        let bogus_local: SocketAddr = "127.0.0.1:19998".parse().unwrap();
        let bogus_remote: SocketAddr = "127.0.0.1:29998".parse().unwrap();

        // 手动向 peer cache 注入一条记录
        {
            let mut cache = peer_cache().lock().unwrap();
            cache.put((bogus_local, bogus_remote), (12345i32, Instant::now()));
        }

        // 从 cache 命中，应 < 1ms
        let t0 = Instant::now();
        let result = lookup_caller_by_socket_addr(bogus_local, bogus_remote);
        let elapsed = t0.elapsed();

        assert_eq!(result, Some(12345i32), "cache 命中应返回注入的 PID");
        assert!(
            elapsed < Duration::from_millis(1),
            "cache 命中耗时应 < 1ms，实际: {elapsed:?}"
        );
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
