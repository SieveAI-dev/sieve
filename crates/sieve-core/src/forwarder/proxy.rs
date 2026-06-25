//! 上游代理配置与 connector（SPEC-007）。
//!
//! `ProxyConfig` 描述「到 target 的 TCP 怎么建」：直连 / HTTP CONNECT / SOCKS5。
//! TLS 始终由上层 hyper-rustls 在隧道之上做——代理只见密文，不 MITM。
//! 代理连接失败时返回明确错误，**绝不静默回退直连**（SPEC-007 §6）。

use crate::error::{SieveCoreError, SieveCoreResult};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use base64::Engine as _;
use hyper_util::client::legacy::connect::{Connected, Connection};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tower_service::Service;

/// 代理认证 (username, password)。
type ProxyAuth = (String, String);

/// 解析后的上游代理配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyConfig {
    /// 直连，不走代理。
    Direct,
    /// HTTP CONNECT 代理。
    Http {
        /// 代理 host:port。
        addr: String,
        /// 可选 Basic 认证 (user, pass)。
        auth: Option<ProxyAuth>,
    },
    /// SOCKS5 代理（`socks5` / `socks5h` 都映射到此，target 域名在代理侧解析）。
    Socks5 {
        /// 代理 host:port。
        addr: String,
        /// 可选用户名/密码认证。
        auth: Option<ProxyAuth>,
    },
}

impl ProxyConfig {
    /// 从 proxy URL 解析。`None` / 空串 → [`ProxyConfig::Direct`]。
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
        let port = authority.port_u16().ok_or_else(|| {
            SieveCoreError::Forwarder(format!("proxy url must include port: {raw:?}"))
        })?;
        let host_port = format!("{}:{}", authority.host(), port);
        let auth = parse_userinfo(authority.as_str());
        match scheme.as_str() {
            "http" => Ok(ProxyConfig::Http {
                addr: host_port,
                auth,
            }),
            "socks5" | "socks5h" => Ok(ProxyConfig::Socks5 {
                addr: host_port,
                auth,
            }),
            other => Err(SieveCoreError::Forwarder(format!(
                "unsupported proxy scheme {other:?} (want http/socks5/socks5h)"
            ))),
        }
    }
}

/// 从 `user:pass@host:port` 的 authority 串抽 userinfo。无 `@` 返回 None。
fn parse_userinfo(authority: &str) -> Option<ProxyAuth> {
    let at = authority.rfind('@')?;
    let (user, pass) = authority[..at].split_once(':')?;
    Some((user.to_string(), pass.to_string()))
}

/// connector 产出的统一底层流（明文 TCP / SOCKS5 隧道）。TLS 由上层做。
pub enum MaybeProxyStream {
    /// 直连或 HTTP CONNECT 隧道后的明文 TCP 流。
    Tcp(TcpStream),
    /// SOCKS5 隧道流（Box 化避免 large_enum_variant：Socks5Stream 比 TcpStream 大）。
    Socks(Box<tokio_socks::tcp::Socks5Stream<TcpStream>>),
}

impl AsyncRead for MaybeProxyStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            MaybeProxyStream::Socks(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for MaybeProxyStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        b: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_write(cx, b),
            MaybeProxyStream::Socks(s) => Pin::new(s.as_mut()).poll_write(cx, b),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_flush(cx),
            MaybeProxyStream::Socks(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeProxyStream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            MaybeProxyStream::Socks(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}

/// 本地 newtype 包装 `TokioIo`，以便实现 hyper [`Connection`]（绕开孤儿规则：
/// 不能为外部类型 `TokioIo<_>` 直接 impl 外部 trait `Connection`）。
pub struct ProxyStream(TokioIo<MaybeProxyStream>);

impl hyper::rt::Read for ProxyStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl hyper::rt::Write for ProxyStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl Connection for ProxyStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

/// 按 [`ProxyConfig`] 建立到 target 的 TCP 流的 connector。实现 tower [`Service<Uri>`]，
/// 作为 hyper-rustls `HttpsConnector` 的底层 connector（TLS 在其上做）。
#[derive(Clone)]
pub struct ProxyConnector {
    cfg: ProxyConfig,
}

impl ProxyConnector {
    /// 用给定代理配置构造 connector。
    pub fn new(cfg: ProxyConfig) -> Self {
        Self { cfg }
    }
}

/// 从 target Uri 抽 host:port（默认端口按 scheme：http=80，其余=443）。
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
    type Response = ProxyStream;
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
                        .map_err(|e| {
                            SieveCoreError::Forwarder(format!("direct connect failed: {e}"))
                        })?;
                    MaybeProxyStream::Tcp(s)
                }
                ProxyConfig::Socks5 { addr, auth } => {
                    let s = match auth {
                        Some((u, p)) => {
                            tokio_socks::tcp::Socks5Stream::connect_with_password(
                                addr.as_str(),
                                (host.as_str(), port),
                                &u,
                                &p,
                            )
                            .await
                        }
                        None => {
                            tokio_socks::tcp::Socks5Stream::connect(
                                addr.as_str(),
                                (host.as_str(), port),
                            )
                            .await
                        }
                    }
                    .map_err(|e| {
                        SieveCoreError::Forwarder(format!("socks5 connect failed: {e}"))
                    })?;
                    MaybeProxyStream::Socks(Box::new(s))
                }
                ProxyConfig::Http { addr, auth } => {
                    let s = http_connect(&addr, &host, port, auth.as_ref()).await?;
                    MaybeProxyStream::Tcp(s)
                }
            };
            Ok(ProxyStream(TokioIo::new(stream)))
        })
    }
}

/// 经 HTTP 代理建 CONNECT 隧道到 target，返回隧道明文 TcpStream（TLS 由上层做）。
async fn http_connect(
    proxy_addr: &str,
    target_host: &str,
    target_port: u16,
    auth: Option<&ProxyAuth>,
) -> SieveCoreResult<TcpStream> {
    let mut s = TcpStream::connect(proxy_addr).await.map_err(|e| {
        SieveCoreError::Forwarder(format!("connect to proxy {proxy_addr} failed: {e}"))
    })?;
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

    // 读响应头直到 \r\n\r\n。
    let mut buf = Vec::with_capacity(128);
    let mut byte = [0u8; 1];
    loop {
        let n = s
            .read(&mut byte)
            .await
            .map_err(|e| SieveCoreError::Forwarder(format!("read CONNECT resp failed: {e}")))?;
        if n == 0 {
            return Err(SieveCoreError::Forwarder(
                "proxy closed connection during CONNECT".into(),
            ));
        }
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") {
            break;
        }
        if buf.len() > 8192 {
            return Err(SieveCoreError::Forwarder(
                "CONNECT response header too large".into(),
            ));
        }
    }
    let head = String::from_utf8_lossy(&buf);
    let status_line = head.lines().next().unwrap_or_default();
    // 形如 "HTTP/1.1 200 Connection Established"
    if status_line.split_whitespace().nth(1) != Some("200") {
        return Err(SieveCoreError::Forwarder(format!(
            "proxy CONNECT rejected: {status_line:?}"
        )));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_direct_when_none() {
        assert!(matches!(
            ProxyConfig::parse(None).unwrap(),
            ProxyConfig::Direct
        ));
        assert!(matches!(
            ProxyConfig::parse(Some("")).unwrap(),
            ProxyConfig::Direct
        ));
    }

    #[test]
    fn parse_socks5() {
        match ProxyConfig::parse(Some("socks5://127.0.0.1:6153")).unwrap() {
            ProxyConfig::Socks5 { addr, auth } => {
                assert_eq!(addr, "127.0.0.1:6153");
                assert!(auth.is_none());
            }
            other => panic!("expected socks5, got {other:?}"),
        }
    }

    #[test]
    fn parse_http_with_auth() {
        match ProxyConfig::parse(Some("http://user:pass@127.0.0.1:6152")).unwrap() {
            ProxyConfig::Http { addr, auth } => {
                assert_eq!(addr, "127.0.0.1:6152");
                assert_eq!(auth, Some(("user".into(), "pass".into())));
            }
            other => panic!("expected http, got {other:?}"),
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
