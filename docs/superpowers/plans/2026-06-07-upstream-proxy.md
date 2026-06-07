# 上游转发代理支持 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 sieve daemon（及 updater）转发上游时可经配置的 HTTP CONNECT / SOCKS5 代理出网，解决受限网络下硬直连不可用的产品缺口。

**Architecture:** 在 `sieve-core` 新增 `ProxyConnector`（实现 `tower::Service<Uri>`，按 `ProxyConfig` 建立到 target 的 TCP 流：直连 / HTTP CONNECT 隧道 / SOCKS5）；把 `Forwarder` 内 `HttpsConnector<HttpConnector>` 的底层换成 `ProxyConnector`，TLS 仍由 hyper-rustls 在隧道之上做（端到端、不 MITM）。`sieve-cli` config 增加 `proxy` / `upstream.proxy` / `upstream.no_proxy` 字段并按优先级链解析；daemon 透传到 `Forwarder::new`；updater 复用同 connector。

**Tech Stack:** Rust / hyper-util legacy Client / hyper-rustls 0.27 / tokio / tokio-socks 0.5（新增）/ tower-service。

依据规格：`docs/specs/SPEC-007-upstream-proxy.md`。决策固化为 ADR-033（Task 10）。

---

## File Structure

| 文件 | 责任 | 动作 |
|---|---|---|
| `Cargo.toml`（workspace） | tokio-socks 版本声明 | 修改 |
| `crates/sieve-core/Cargo.toml` | tokio-socks 依赖 + feature | 修改 |
| `crates/sieve-core/src/forwarder/proxy.rs` | `ProxyConfig` + URL 解析 + `ProxyConnector`（Service）+ stream 包装 | 新建 |
| `crates/sieve-core/src/forwarder/mod.rs` | `Forwarder::new(url, proxy)` + client 类型改用 `ProxyConnector` | 修改 |
| `crates/sieve-cli/src/config.rs` | `UpstreamListener.proxy/no_proxy` + `Config.proxy` + 优先级解析方法 | 修改 |
| `crates/sieve-cli/src/daemon.rs` | listener_specs 构建透传 proxy 字符串到 `Forwarder::new` | 修改 |
| `crates/sieve-updater/src/manifest.rs` + `download.rs` | connector 复用 ProxyConnector | 修改 |
| `crates/sieve-core/tests/proxy_connector.rs` | mock HTTP-CONNECT / SOCKS5 代理 e2e | 新建 |
| `docs/design/ADR-033-upstream-proxy.md` + `ADR-INDEX.md` | 决策记录 | 新建/修改 |
| `docs/api/api-reference.md` + `guides/deployment.md` + `changelog/CHANGELOG.md` | 文档同步 | 修改 |

**约定**：sieve-core 的 proxy 能力门控在现有 `forwarder` feature 下（与 `Forwarder` 同 feature，见 `crates/sieve-core/Cargo.toml`）。

---

## Task 1: 新增 tokio-socks 依赖

**Files:**
- Modify: `Cargo.toml`（workspace `[workspace.dependencies]`）
- Modify: `crates/sieve-core/Cargo.toml`

- [ ] **Step 1: workspace 声明 tokio-socks**

在 `Cargo.toml` 的 `[workspace.dependencies]` 段（hyper-rustls 行附近）加：

```toml
tokio-socks = { version = "0.5", default-features = false }
```

- [ ] **Step 2: sieve-core 引入（门控在 forwarder feature）**

在 `crates/sieve-core/Cargo.toml` 的 `[dependencies]` 段加（与 hyper-rustls 同区，optional）：

```toml
tokio-socks = { workspace = true, optional = true }
```

并在 `[features]` 的 `forwarder` feature 依赖列表追加 `"dep:tokio-socks"`（与现有 `"dep:hyper-rustls"` 同列表）。

- [ ] **Step 3: 验证编译 + deny**

Run: `cargo build -p sieve-core --features forwarder 2>&1 | tail -5`
Expected: 编译通过（tokio-socks 拉取成功）

Run: `cargo deny check 2>&1 | tail -8`
Expected: advisories/licenses/bans/sources 全 ok（若 tokio-socks 引入新 license/dup，按 deny.toml 现状评估；如需放行在 deny.toml 记录）

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock crates/sieve-core/Cargo.toml
git commit -m "build(sieve-core): 引入 tokio-socks (SPEC-007 上游 SOCKS5 代理)"
```

---

## Task 2: ProxyConfig 类型 + URL 解析

纯逻辑，无网络，先 TDD。

**Files:**
- Create: `crates/sieve-core/src/forwarder/proxy.rs`
- Modify: `crates/sieve-core/src/forwarder/mod.rs`（在文件顶部加 `mod proxy; pub use proxy::ProxyConfig;`）

- [ ] **Step 1: 写失败测试**

在 `crates/sieve-core/src/forwarder/proxy.rs` 末尾（先建文件含 `#[cfg(test)]`）写：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_direct_when_none() {
        assert!(matches!(ProxyConfig::parse(None).unwrap(), ProxyConfig::Direct));
    }

    #[test]
    fn parse_socks5() {
        let c = ProxyConfig::parse(Some("socks5://127.0.0.1:6153")).unwrap();
        match c {
            ProxyConfig::Socks5 { addr, auth } => {
                assert_eq!(addr, "127.0.0.1:6153");
                assert!(auth.is_none());
            }
            _ => panic!("expected socks5"),
        }
    }

    #[test]
    fn parse_http_with_auth() {
        let c = ProxyConfig::parse(Some("http://user:pass@127.0.0.1:6152")).unwrap();
        match c {
            ProxyConfig::Http { addr, auth } => {
                assert_eq!(addr, "127.0.0.1:6152");
                assert_eq!(auth, Some(("user".into(), "pass".into())));
            }
            _ => panic!("expected http"),
        }
    }

    #[test]
    fn parse_rejects_unknown_scheme() {
        assert!(ProxyConfig::parse(Some("ftp://x:1")).is_err());
    }

    #[test]
    fn parse_rejects_missing_port() {
        assert!(ProxyConfig::parse(Some("socks5://127.0.0.1")).is_err());
    }
}
```

- [ ] **Step 2: 运行验证失败**

Run: `cargo test -p sieve-core --features forwarder proxy:: 2>&1 | tail -15`
Expected: 编译失败（`ProxyConfig` 未定义）

- [ ] **Step 3: 实现 ProxyConfig + parse**

在 `proxy.rs` 顶部（tests 之前）写：

```rust
//! 上游代理配置与 connector（SPEC-007）。
//!
//! ProxyConfig 描述「到 target 的 TCP 怎么建」：直连 / HTTP CONNECT / SOCKS5。
//! TLS 始终由上层 hyper-rustls 在隧道之上做——代理只见密文，不 MITM（PRD §9 #12）。

use crate::error::{SieveCoreError, SieveCoreResult};

/// 代理认证 (username, password)。
type ProxyAuth = (String, String);

/// 解析后的代理配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyConfig {
    /// 直连，不走代理。
    Direct,
    /// HTTP CONNECT 代理。`addr` = host:port。
    Http { addr: String, auth: Option<ProxyAuth> },
    /// SOCKS5 代理。`addr` = host:port。`socks5h` 与 `socks5` 都映射到此（远程 DNS 由 tokio-socks 处理 target 域名）。
    Socks5 { addr: String, auth: Option<ProxyAuth> },
}

impl ProxyConfig {
    /// 从 proxy URL 解析。`None` → Direct。
    ///
    /// 支持 `http://` / `socks5://` / `socks5h://`，可带 `user:pass@`。
    /// scheme 非法或缺端口时返回 [`SieveCoreError::Forwarder`]。
    pub fn parse(url: Option<&str>) -> SieveCoreResult<Self> {
        let Some(raw) = url.map(str::trim).filter(|s| !s.is_empty()) else {
            return Ok(ProxyConfig::Direct);
        };
        let uri: http::Uri = raw
            .parse()
            .map_err(|e| SieveCoreError::Forwarder(format!("invalid proxy url {raw:?}: {e}")))?;
        let scheme = uri
            .scheme_str()
            .ok_or_else(|| SieveCoreError::Forwarder(format!("proxy url missing scheme: {raw:?}")))?
            .to_ascii_lowercase();
        let authority = uri
            .authority()
            .ok_or_else(|| SieveCoreError::Forwarder(format!("proxy url missing host: {raw:?}")))?;
        if authority.port_u16().is_none() {
            return Err(SieveCoreError::Forwarder(format!(
                "proxy url must include port: {raw:?}"
            )));
        }
        // authority.host() 不含 userinfo；authority.as_str() 可能含 user:pass@host:port
        let host_port = format!(
            "{}:{}",
            authority.host(),
            authority.port_u16().expect("checked above")
        );
        let auth = parse_userinfo(authority.as_str());
        match scheme.as_str() {
            "http" => Ok(ProxyConfig::Http { addr: host_port, auth }),
            "socks5" | "socks5h" => Ok(ProxyConfig::Socks5 { addr: host_port, auth }),
            other => Err(SieveCoreError::Forwarder(format!(
                "unsupported proxy scheme {other:?} (want http/socks5/socks5h)"
            ))),
        }
    }
}

/// 从 `user:pass@host:port` 的 authority 串里抽 userinfo。无 `@` 返回 None。
fn parse_userinfo(authority: &str) -> Option<ProxyAuth> {
    let at = authority.rfind('@')?;
    let userinfo = &authority[..at];
    let (user, pass) = userinfo.split_once(':')?;
    Some((user.to_string(), pass.to_string()))
}
```

在 `crates/sieve-core/src/forwarder/mod.rs` 顶部（`use` 区之后）加：

```rust
mod proxy;
pub use proxy::ProxyConfig;
```

- [ ] **Step 4: 运行验证通过**

Run: `cargo test -p sieve-core --features forwarder proxy:: 2>&1 | tail -15`
Expected: 5 个 test 全 PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-core/src/forwarder/proxy.rs crates/sieve-core/src/forwarder/mod.rs
git commit -m "feat(sieve-core): ProxyConfig 解析 (SPEC-007)"
```

---

## Task 3: ProxyConnector — 直连 + SOCKS5

connector 是核心。先做 Direct + Socks5（HTTP CONNECT 在 Task 4，便于隔离）。输出统一 stream 类型。

**Files:**
- Modify: `crates/sieve-core/src/forwarder/proxy.rs`
- Test: `crates/sieve-core/tests/proxy_connector.rs`（新建）

- [ ] **Step 1: 写失败集成测试（SOCKS5 + 直连）**

新建 `crates/sieve-core/tests/proxy_connector.rs`：

```rust
//! ProxyConnector 集成测试：用 mock 代理验证 connector 能建到 target 的 TCP 流。
//! 不做 TLS（直接验 connector 输出的明文 TCP 隧道到 mock target）。
#![cfg(feature = "forwarder")]

use sieve_core::forwarder::{ProxyConfig, ProxyConnector};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower_service::Service;

/// 起一个 echo TCP 服务，返回其 addr。
async fn spawn_echo() -> std::net::SocketAddr {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut s, _)) = l.accept().await else { break };
            tokio::spawn(async move {
                let mut buf = [0u8; 64];
                if let Ok(n) = s.read(&mut buf).await {
                    let _ = s.write_all(&buf[..n]).await;
                }
            });
        }
    });
    addr
}

#[tokio::test]
async fn direct_connects_to_target() {
    let target = spawn_echo().await;
    let mut conn = ProxyConnector::new(ProxyConfig::Direct);
    let uri: http::Uri = format!("http://{target}").parse().unwrap();
    let mut stream = conn.call(uri).await.expect("direct connect");
    stream.write_all(b"ping").await.unwrap();
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"ping");
}
```

> SOCKS5 的 mock 代理较重，放 Task 4 与 HTTP CONNECT 一起做 e2e；本 task 先用 Direct 驱动出 `ProxyConnector` 骨架与 stream 类型。

- [ ] **Step 2: 运行验证失败**

Run: `cargo test -p sieve-core --features forwarder --test proxy_connector direct_connects 2>&1 | tail -15`
Expected: 编译失败（`ProxyConnector` 未定义）

- [ ] **Step 3: 实现 ProxyConnector（Direct + Socks5）+ 统一 stream**

在 `proxy.rs` 追加（tests mod 之前）：

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper_util::client::legacy::connect::{Connected, Connection};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// connector 产出的统一 TCP 流（直连 / 隧道后的明文流）。
/// 包成 TokioIo<MaybeProxyStream> 供 hyper 使用；并实现 Connection。
pub enum MaybeProxyStream {
    Tcp(TcpStream),
    Socks(tokio_socks::tcp::Socks5Stream<TcpStream>),
}

impl AsyncRead for MaybeProxyStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            MaybeProxyStream::Socks(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}
impl AsyncWrite for MaybeProxyStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, b: &[u8]) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_write(cx, b),
            MaybeProxyStream::Socks(s) => Pin::new(s).poll_write(cx, b),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_flush(cx),
            MaybeProxyStream::Socks(s) => Pin::new(s).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            MaybeProxyStream::Socks(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
impl Connection for TokioIo<MaybeProxyStream> {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

/// 按 ProxyConfig 建立到 target 的 TCP 流的 connector。实现 tower Service<Uri>。
#[derive(Clone)]
pub struct ProxyConnector {
    cfg: ProxyConfig,
}

impl ProxyConnector {
    pub fn new(cfg: ProxyConfig) -> Self {
        Self { cfg }
    }
}

/// 从 target Uri 抽 host:port（默认端口按 scheme：https=443, http=80）。
fn target_host_port(uri: &http::Uri) -> SieveCoreResult<(String, u16)> {
    let host = uri
        .host()
        .ok_or_else(|| SieveCoreError::Forwarder(format!("target uri missing host: {uri}")))?
        .to_string();
    let port = uri.port_u16().unwrap_or(match uri.scheme_str() {
        Some("http") => 80,
        _ => 443,
    });
    Ok((host, port))
}

impl Service<http::Uri> for ProxyConnector {
    type Response = TokioIo<MaybeProxyStream>;
    type Error = SieveCoreError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: http::Uri) -> Self::Future {
        let cfg = self.cfg.clone();
        Box::pin(async move {
            let (host, port) = target_host_port(&dst)?;
            let stream = match cfg {
                ProxyConfig::Direct => {
                    let s = TcpStream::connect((host.as_str(), port))
                        .await
                        .map_err(|e| SieveCoreError::Forwarder(format!("direct connect failed: {e}")))?;
                    MaybeProxyStream::Tcp(s)
                }
                ProxyConfig::Socks5 { addr, auth } => {
                    let s = match auth {
                        Some((u, p)) => tokio_socks::tcp::Socks5Stream::connect_with_password(
                            addr.as_str(), (host.as_str(), port), &u, &p,
                        ).await,
                        None => tokio_socks::tcp::Socks5Stream::connect(
                            addr.as_str(), (host.as_str(), port),
                        ).await,
                    }
                    .map_err(|e| SieveCoreError::Forwarder(format!("socks5 connect failed: {e}")))?;
                    MaybeProxyStream::Socks(s)
                }
                ProxyConfig::Http { addr, auth } => {
                    let s = http_connect(&addr, &host, port, auth.as_ref()).await?;
                    MaybeProxyStream::Tcp(s)
                }
            };
            Ok(TokioIo::new(stream))
        })
    }
}
```

> `http_connect` 在 Task 4 实现；本 task 先放一个 `todo!()` 占位让 Direct/Socks5 路径编译——**不**，为避免占位，本 task 直接把 `http_connect` 的最小实现一并写入（见 Task 4 Step 3 的函数体），这样 Http 分支也可编译。若拆分执行，Task 4 仅补测试。

把 Task 4 Step 3 的 `http_connect` 函数体此处一并粘入 `proxy.rs`。

需要 `tower-service` 依赖：在 `crates/sieve-core/Cargo.toml` 加 `tower-service = { version = "0.3", optional = true }` 并入 `forwarder` feature（hyper-util 已传递依赖 tower-service，但显式声明更稳）。

- [ ] **Step 4: 运行验证通过**

Run: `cargo test -p sieve-core --features forwarder --test proxy_connector direct_connects 2>&1 | tail -15`
Expected: `direct_connects_to_target` PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-core/src/forwarder/proxy.rs crates/sieve-core/tests/proxy_connector.rs crates/sieve-core/Cargo.toml
git commit -m "feat(sieve-core): ProxyConnector 直连+SOCKS5 (SPEC-007)"
```

---

## Task 4: HTTP CONNECT 隧道 + 双代理 e2e 测试

**Files:**
- Modify: `crates/sieve-core/src/forwarder/proxy.rs`（`http_connect` 若 Task 3 已粘入则仅校验）
- Modify: `crates/sieve-core/tests/proxy_connector.rs`

- [ ] **Step 1: 写失败测试（mock HTTP CONNECT 代理）**

在 `proxy_connector.rs` 追加。mock CONNECT 代理：accept → 读到 `\r\n\r\n` → 回 `HTTP/1.1 200 Connection Established\r\n\r\n` → 然后把后续字节转发到真实 target（双向 copy）。

```rust
async fn spawn_http_connect_proxy(target: std::net::SocketAddr) -> std::net::SocketAddr {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut client, _)) = l.accept().await else { break };
            tokio::spawn(async move {
                // 读 CONNECT 请求头直到 \r\n\r\n
                let mut buf = Vec::new();
                let mut byte = [0u8; 1];
                while !buf.ends_with(b"\r\n\r\n") {
                    if client.read_exact(&mut byte).await.is_err() { return; }
                    buf.push(byte[0]);
                }
                assert!(buf.starts_with(b"CONNECT "), "expected CONNECT, got {:?}", String::from_utf8_lossy(&buf));
                client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await.unwrap();
                // 隧道：双向转发到 target
                let mut up = TcpStream::connect(target).await.unwrap();
                let _ = tokio::io::copy_bidirectional(&mut client, &mut up).await;
            });
        }
    });
    addr
}

#[tokio::test]
async fn http_connect_tunnels_to_target() {
    let target = spawn_echo().await;
    let proxy = spawn_http_connect_proxy(target).await;
    let mut conn = ProxyConnector::new(
        ProxyConfig::parse(Some(&format!("http://{proxy}"))).unwrap()
    );
    let uri: http::Uri = format!("http://{target}").parse().unwrap();
    let mut stream = conn.call(uri).await.expect("connect via http proxy");
    stream.write_all(b"pong").await.unwrap();
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"pong");
}

#[tokio::test]
async fn http_connect_propagates_proxy_failure() {
    // 代理拒绝（端口无监听）→ connector 返回错误，不静默直连
    let bad_proxy = "127.0.0.1:1"; // 几乎不可能有监听
    let mut conn = ProxyConnector::new(
        ProxyConfig::parse(Some(&format!("http://{bad_proxy}"))).unwrap()
    );
    let uri: http::Uri = "http://127.0.0.1:9/".parse().unwrap();
    assert!(conn.call(uri).await.is_err());
}
```

- [ ] **Step 2: 运行验证失败**

Run: `cargo test -p sieve-core --features forwarder --test proxy_connector http_connect 2>&1 | tail -20`
Expected: 若 Task 3 已粘入 `http_connect`，测试应可编译但需确认逻辑；若未粘入则编译失败（函数缺失）

- [ ] **Step 3: 实现 http_connect**

在 `proxy.rs` 追加（若 Task 3 已粘入则核对一致）：

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use base64::Engine as _;

/// 经 HTTP 代理建 CONNECT 隧道到 target，返回隧道 TcpStream（明文，TLS 由上层做）。
async fn http_connect(
    proxy_addr: &str,
    target_host: &str,
    target_port: u16,
    auth: Option<&ProxyAuth>,
) -> SieveCoreResult<TcpStream> {
    let mut s = TcpStream::connect(proxy_addr)
        .await
        .map_err(|e| SieveCoreError::Forwarder(format!("connect to proxy {proxy_addr} failed: {e}")))?;
    let mut req = format!(
        "CONNECT {target_host}:{target_port} HTTP/1.1\r\nHost: {target_host}:{target_port}\r\n"
    );
    if let Some((u, p)) = auth {
        let token = base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"));
        req.push_str(&format!("Proxy-Authorization: Basic {token}\r\n"));
    }
    req.push_str("\r\n");
    s.write_all(req.as_bytes())
        .await
        .map_err(|e| SieveCoreError::Forwarder(format!("write CONNECT failed: {e}")))?;

    // 读响应头直到 \r\n\r\n
    let mut buf = Vec::with_capacity(128);
    let mut byte = [0u8; 1];
    loop {
        let n = s.read(&mut byte).await
            .map_err(|e| SieveCoreError::Forwarder(format!("read CONNECT resp failed: {e}")))?;
        if n == 0 {
            return Err(SieveCoreError::Forwarder("proxy closed during CONNECT".into()));
        }
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") { break; }
        if buf.len() > 8192 {
            return Err(SieveCoreError::Forwarder("CONNECT response header too large".into()));
        }
    }
    let head = String::from_utf8_lossy(&buf);
    let status_line = head.lines().next().unwrap_or_default();
    // 形如 "HTTP/1.1 200 Connection Established"
    let ok = status_line.split_whitespace().nth(1) == Some("200");
    if !ok {
        return Err(SieveCoreError::Forwarder(format!(
            "proxy CONNECT rejected: {status_line:?}"
        )));
    }
    Ok(s)
}
```

加依赖 `base64`：`crates/sieve-core/Cargo.toml` 加 `base64 = { version = "0.22", optional = true }` 入 `forwarder` feature（若 workspace 已有 base64 则用 workspace）。

- [ ] **Step 4: 运行验证通过**

Run: `cargo test -p sieve-core --features forwarder --test proxy_connector 2>&1 | tail -20`
Expected: `direct_connects_to_target` / `http_connect_tunnels_to_target` / `http_connect_propagates_proxy_failure` 全 PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-core/src/forwarder/proxy.rs crates/sieve-core/tests/proxy_connector.rs crates/sieve-core/Cargo.toml
git commit -m "feat(sieve-core): HTTP CONNECT 隧道 + 代理 e2e 测试 (SPEC-007)"
```

---

## Task 5: Forwarder::new 接入 ProxyConnector

**Files:**
- Modify: `crates/sieve-core/src/forwarder/mod.rs`

- [ ] **Step 1: 写失败测试**

在 `mod.rs` 的 `#[cfg(test)] mod tests`（若无则新建）加：

```rust
#[test]
fn forwarder_new_accepts_proxy_arg() {
    // Direct（None）应与旧行为一致
    let f = Forwarder::new("https://api.anthropic.com", ProxyConfig::Direct);
    assert!(f.is_ok());
    // SOCKS5 配置也能构造（不实际连接）
    let p = ProxyConfig::parse(Some("socks5://127.0.0.1:6153")).unwrap();
    assert!(Forwarder::new("https://api.anthropic.com", p).is_ok());
}
```

- [ ] **Step 2: 运行验证失败**

Run: `cargo test -p sieve-core --features forwarder forwarder_new_accepts_proxy 2>&1 | tail -15`
Expected: 编译失败（`Forwarder::new` 只接受 1 个参数）

- [ ] **Step 3: 改造 Forwarder**

在 `mod.rs`：

1. 改 client 字段类型（第 32-35 行）为：

```rust
    client: Client<hyper_rustls::HttpsConnector<proxy::ProxyConnector>, BoxBody>,
```

2. 改 `new` 签名与 connector 构建（第 57、85-92 行）：

```rust
    pub fn new(upstream_url: &str, proxy: ProxyConfig) -> SieveCoreResult<Self> {
        install_crypto_provider();
        // ... host/scheme/path_prefix 解析保持不变 ...

        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .wrap_connector(proxy::ProxyConnector::new(proxy));

        let client = Client::builder(TokioExecutor::new()).build::<_, BoxBody>(connector);
        // ... Ok(Self { ... }) 不变 ...
    }
```

> 关键：`.build()` 改为 `.wrap_connector(ProxyConnector::new(proxy))`。`wrap_connector` 要求底层 connector 实现 `Service<Uri, Response: Read+Write+Connection+Unpin+Send>`——由 Task 3 的 impl 满足。若编译报 trait bound 不满足，按报错补 `Send + 'static` 等约束到 `ProxyConnector` 与 `MaybeProxyStream`。

- [ ] **Step 4: 运行验证通过**

Run: `cargo test -p sieve-core --features forwarder forwarder_new_accepts_proxy 2>&1 | tail -15`
Expected: PASS

Run: `cargo test -p sieve-core --features forwarder 2>&1 | tail -5`
Expected: sieve-core 全绿（原有 forwarder 测试不回归）

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-core/src/forwarder/mod.rs
git commit -m "feat(sieve-core): Forwarder::new 接入 ProxyConnector (SPEC-007)"
```

---

## Task 6: config schema — proxy 字段 + 优先级解析

**Files:**
- Modify: `crates/sieve-cli/src/config.rs`

- [ ] **Step 1: 写失败测试**

在 `config.rs` 的 `#[cfg(test)] mod tests` 加：

```rust
#[test]
fn upstream_proxy_overrides_global() {
    let toml_str = r#"
        proxy = "socks5://127.0.0.1:1"
        [[upstream]]
        port = 11453
        url = "https://api.anthropic.com"
        proxy = "http://127.0.0.1:2"
        [[upstream]]
        port = 11454
        url = "http://127.0.0.1:8080"
        no_proxy = true
    "#;
    let c: Config = toml::from_str(toml_str).unwrap();
    let ups = c.resolved_upstreams();
    // upstream[0]: 自身 proxy 覆盖全局
    assert_eq!(c.effective_proxy(&ups[0]), Some("http://127.0.0.1:2".to_string()));
    // upstream[1]: no_proxy → 直连(None)，无视全局
    assert_eq!(c.effective_proxy(&ups[1]), None);
}

#[test]
fn global_proxy_applies_when_upstream_unset() {
    let toml_str = r#"
        proxy = "socks5://127.0.0.1:6153"
        [[upstream]]
        port = 11453
        url = "https://api.anthropic.com"
    "#;
    let c: Config = toml::from_str(toml_str).unwrap();
    let ups = c.resolved_upstreams();
    assert_eq!(c.effective_proxy(&ups[0]), Some("socks5://127.0.0.1:6153".to_string()));
}
```

- [ ] **Step 2: 运行验证失败**

Run: `cargo test -p sieve-cli upstream_proxy_overrides_global global_proxy_applies 2>&1 | tail -15`
Expected: 编译失败（字段/方法缺失）

- [ ] **Step 3: 加字段 + effective_proxy**

1. `UpstreamListener`（约第 59-76 行）加两个字段：

```rust
    /// 该 listener 专属上游代理 URL（覆盖全局 [`Config::proxy`]）。
    /// 形如 `socks5://127.0.0.1:6153` / `http://127.0.0.1:6152`（SPEC-007）。
    #[serde(default)]
    pub proxy: Option<String>,

    /// 显式直连，无视全局 proxy 与 env（优先级最高）。
    #[serde(default)]
    pub no_proxy: bool,
```

2. `Config` struct（约第 149 行）加全局字段：

```rust
    /// 全局兜底上游代理 URL（可选）。每个 upstream 未设 proxy 且未 no_proxy 时继承。
    /// 也可由 env `ALL_PROXY` / `HTTPS_PROXY` 兜底（config 优先）。SPEC-007。
    #[serde(default)]
    pub proxy: Option<String>,
```

> 注意 `Config` 与 `UpstreamListener` 都有 `#[serde(deny_unknown_fields)]`，新字段加 `#[serde(default)]` 保证旧配置兼容。`resolved_upstreams()`（约 416-426）的 legacy 映射构造 `UpstreamListener` 时补 `proxy: None, no_proxy: false`。

3. `Config` 的 `impl` 加方法：

```rust
    /// 计算某 upstream 的有效代理 URL。优先级：
    /// no_proxy > upstream.proxy > 全局 proxy > env(ALL_PROXY/HTTPS_PROXY) > 直连(None)。
    pub fn effective_proxy(&self, up: &UpstreamListener) -> Option<String> {
        if up.no_proxy {
            return None;
        }
        if let Some(p) = up.proxy.as_ref().filter(|s| !s.is_empty()) {
            return Some(p.clone());
        }
        if let Some(p) = self.proxy.as_ref().filter(|s| !s.is_empty()) {
            return Some(p.clone());
        }
        // env 兜底：HTTPS_PROXY 优先于 ALL_PROXY（scheme-specific 优先）
        for key in ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"] {
            if let Some(v) = std::env::var_os(key) {
                let v = v.to_string_lossy().trim().to_string();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
        None
    }
```

更新需要同步的现有测试：`resolved_upstreams` 相关测试若用结构体字面量构造 `UpstreamListener`，补 `proxy: None, no_proxy: false`（grep `UpstreamListener {` 定位，编译器会报缺字段）。

- [ ] **Step 4: 运行验证通过**

Run: `cargo test -p sieve-cli upstream_proxy global_proxy 2>&1 | tail -15`
Expected: 2 个 test PASS

Run: `cargo test -p sieve-cli --lib config:: 2>&1 | tail -5`
Expected: config 模块全绿

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-cli/src/config.rs
git commit -m "feat(config): proxy/upstream.proxy/no_proxy 字段 + 优先级解析 (SPEC-007)"
```

---

## Task 7: daemon 透传 proxy 到 Forwarder

**Files:**
- Modify: `crates/sieve-cli/src/daemon.rs`

- [ ] **Step 1: 改 listener_specs 构建（约第 648-665 行）**

把 `Forwarder::new(&u.url)` 改为带 proxy：

```rust
            let proxy = ProxyConfig::parse(cfg.effective_proxy(u).as_deref())
                .map_err(|e| anyhow::anyhow!("invalid proxy for upstream {}: {e}", u.url))?;
            let f = Arc::new(Forwarder::new(&u.url, proxy).map_err(|e| {
                // ... 保留原 error 包装 ...
            })?);
```

文件顶部 `use` 加 `use sieve_core::forwarder::ProxyConfig;`（或 `sieve_core::ProxyConfig`，按 re-export 路径）。

- [ ] **Step 2: 改另一处 Forwarder::new（约第 709 行）**

该处（fallback/单 upstream 路径）同样补 proxy 参数。若该路径对应某个 upstream，用 `cfg.effective_proxy(...)`；若是无 upstream 上下文的兜底，用 `ProxyConfig::Direct`。按上下文读 705-715 行确定，保持与 Step 1 一致语义。

- [ ] **Step 3: 编译 + 全量测试**

Run: `cargo build -p sieve-cli 2>&1 | tail -5`
Expected: 编译通过

Run: `cargo test -p sieve-cli 2>&1 | tail -8`
Expected: sieve-cli 全绿（含改 config 后的现有集成测试；content_type_matrix 等用 legacy 配置 → effective_proxy=None → Direct，行为不变）

- [ ] **Step 4: Commit**

```bash
git add crates/sieve-cli/src/daemon.rs
git commit -m "feat(daemon): listener 按 effective_proxy 透传到 Forwarder (SPEC-007)"
```

---

## Task 8: updater 复用 ProxyConnector

**Files:**
- Modify: `crates/sieve-updater/src/manifest.rs`（约 83-93）
- Modify: `crates/sieve-updater/src/download.rs`（约 18-28）
- Modify: `crates/sieve-updater/Cargo.toml`（依赖 sieve-core forwarder feature 暴露的 ProxyConnector/ProxyConfig）

- [ ] **Step 1: 暴露 ProxyConnector 供 updater 用**

确认 `sieve_core::forwarder::{ProxyConfig, ProxyConnector, MaybeProxyStream}` 为 `pub`（Task 3 已 pub ProxyConnector/MaybeProxyStream；mod.rs re-export）。`crates/sieve-updater/Cargo.toml` 确保依赖 `sieve-core`（启用 `forwarder` feature）。

- [ ] **Step 2: manifest.rs / download.rs connector 改造**

两处把：

```rust
    let tls = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()  // 或现有 https_or_http，保持原样
        .enable_http1()
        .build();
    let client = Client::builder(TokioExecutor::new()).build(tls);
```

改为（client 类型签名同步改为 `HttpsConnector<ProxyConnector>`）：

```rust
    let proxy = sieve_core::forwarder::ProxyConfig::parse(proxy_url.as_deref())?; // proxy_url 见 Step 3
    let tls = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .wrap_connector(sieve_core::forwarder::ProxyConnector::new(proxy));
    let client = Client::builder(TokioExecutor::new()).build(tls);
```

`download.rs:18` / `manifest.rs:83` 的 client 类型别名同步改为 `HttpsConnector<ProxyConnector>`。

- [ ] **Step 3: proxy_url 来源**

updater 入口（runner / UpdaterConfig）增加可选 `proxy: Option<String>`，由 daemon 启动 updater 时传入 `cfg.proxy.clone()`（全局），并在 updater 内对 None 时回退 env（复用与 config 同样的 HTTPS_PROXY/ALL_PROXY 探测，或直接传 `ProxyConfig::parse` 接受 env——为 DRY，updater 用 daemon 传入的全局 proxy 字符串；env 兜底由 daemon 侧 `effective_proxy`-style 逻辑统一计算后传入）。grep `UpdaterConfig` 定位结构，加字段并在 `crates/sieve-cli/src/daemon.rs` 启动 updater 处传 `cfg.proxy`。

- [ ] **Step 4: 测试 + 验证**

Run: `cargo test -p sieve-updater 2>&1 | tail -5`
Expected: updater 全绿（原 35 测试不回归；新增 connector 构造测试可选）

- [ ] **Step 5: Commit**

```bash
git add crates/sieve-updater/ crates/sieve-cli/src/daemon.rs
git commit -m "feat(updater): manifest/download 复用 ProxyConnector (SPEC-007)"
```

---

## Task 9: 全量回归 + clippy + fmt

**Files:** 无（验证）

- [ ] **Step 1: 全量测试**

Run: `cargo test --workspace --no-fail-fast 2>&1 | grep -E "test result|FAILED"`
Expected: 0 failed（基线 760 + 新增 proxy 测试）

- [ ] **Step 2: clippy + fmt**

Run: `cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 0 warning（注意自定义 Service/Future 的 clippy；如 `type_complexity` 报警，按需 `#[allow]` 加注释或抽 type alias）

Run: `cargo fmt --all --check`
Expected: 干净

- [ ] **Step 3: cargo-deny**

Run: `cargo deny check 2>&1 | tail -8`
Expected: 4 道闸 ok（tokio-socks/base64 的 license/dup 若触发，deny.toml 记录评估）

- [ ] **Step 4: Commit（如有 fmt/clippy 修正）**

```bash
git add -A && git commit -m "chore: SPEC-007 proxy fmt/clippy 收尾"
```

---

## Task 10: ADR-033 + 文档同步

**Files:**
- Create: `docs/design/ADR-033-upstream-proxy.md`
- Modify: `docs/design/ADR-INDEX.md`、`docs/api/api-reference.md`、`docs/guides/deployment.md`、`docs/changelog/CHANGELOG.md`
- Modify: `docs/specs/SPEC-007-upstream-proxy.md`（状态 Draft → Stable）

- [ ] **Step 1: 写 ADR-033**

新建 `docs/design/ADR-033-upstream-proxy.md`，含：状态 Accepted / 决策（支持 HTTP CONNECT + SOCKS5 上游代理，按 scheme 自选，每 upstream + 全局 + env）/ 硬约束分析（照搬 SPEC-007 §9：不违反 PRD §9 #2 出站目的地不变、#12 不 MITM、与 ADR-027 区分）/ 后果（受限网络可用 + 隐私提示用可信代理）。关联 SPEC-007。

- [ ] **Step 2: ADR-INDEX 加行**

`docs/design/ADR-INDEX.md` 主表加：
```
| [ADR-033](./ADR-033-upstream-proxy.md) | 上游转发代理支持（HTTP CONNECT + SOCKS5） | Accepted | 2026-06-07 | v2.0 §6.1、§9 #2 |
```
并把 `CLAUDE.md` Source of Truth 段 ADR 计数 25 → 26。

- [ ] **Step 3: api-reference + deployment + CHANGELOG**

- `docs/api/api-reference.md` §3 config：新增 `proxy` / `[[upstream]].proxy` / `no_proxy` 字段说明 + 优先级链 + proxy URL 格式。
- `docs/guides/deployment.md`：新增「受限网络（Shadowrocket/Clash）」章——示例 `proxy = "socks5://127.0.0.1:<socks 端口>"`，提示用可信本地代理。
- `docs/changelog/CHANGELOG.md` `[Unreleased]` 加 `### Added` 上游代理支持条目。

- [ ] **Step 4: SPEC-007 状态更新**

`docs/specs/SPEC-007-upstream-proxy.md` 第一行 `状态：Draft` → `Stable`；`docs/specs/INDEX.md` SPEC-007 行状态 Draft → Stable。

- [ ] **Step 5: Commit**

```bash
git add docs/ CLAUDE.md
git commit -m "docs: ADR-033 上游代理 + api-reference/deployment/CHANGELOG 同步 (SPEC-007)"
```

---

## Task 11: dogfood 冒烟（真实 Shadowrocket）

**Files:** 无（手动验证，可选）

- [ ] **Step 1: 配置真实代理冒烟**

在本地 sieve.toml 写 `proxy = "socks5://127.0.0.1:<你的 Shadowrocket SOCKS 端口>"`，起 daemon，用 curl 经 sieve 端口发一个最小 `/v1/messages` 请求（或真实 agent），确认上游返回 200（不再因网络直连失败）。

Run（示例）: `RUST_LOG=info cargo run -p sieve-cli -- start --config sieve.toml`
Expected: 日志显示请求经代理转发成功，无 "direct connect failed"。

- [ ] **Step 2: 记录结果到 PROGRESS**

把冒烟结果（成功/问题）记入 `tasks/PROGRESS.md`，并把 integration-test-checklist 的网络相关项标记可跑。

---

## Self-Review 结果

- **Spec 覆盖**：SPEC-007 §2 配置→Task 6；§3 URL→Task 2；§4 架构→Task 3/5；§5 协议→Task 3/4；§6 错误不回退→Task 4（failure 测试）；§7 updater→Task 8；§8 测试矩阵→Task 2/3/4/6/9；§9 决策→Task 10；§10 文档→Task 10。全覆盖。
- **类型一致**：`ProxyConfig`（Direct/Http/Socks5）、`ProxyConnector::new`、`Forwarder::new(url, proxy)`、`Config::effective_proxy(&UpstreamListener)` 跨 task 命名一致。
- **已知风险**：Task 3/5 自定义 hyper connector 的 trait bound（`Connection`/`Read`/`Write`/`Send+'static`）以 cargo 报错驱动微调——已在对应 step 标注。tokio-socks `Socks5Stream<TcpStream>` 实现 AsyncRead/Write，满足包装。
