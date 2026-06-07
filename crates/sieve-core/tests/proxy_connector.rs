//! ProxyConnector 集成测试（SPEC-007）。
//!
//! 覆盖 Direct + HTTP CONNECT（含失败不静默回退）。SOCKS5 路径靠
//! `ProxyConfig::parse` 单测（见 forwarder/proxy.rs）+ 真实 Shadowrocket/Clash
//! dogfood 验证——mock SOCKS5 server 成本高，不纳入自动测试。
#![cfg(feature = "forwarder")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sieve_core::forwarder::{ProxyConfig, ProxyConnector};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower_service::Service;

/// 起一个只 accept 不读写的 TCP listener，返回其 addr。
async fn spawn_listener() -> std::net::SocketAddr {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            if l.accept().await.is_err() {
                break;
            }
        }
    });
    addr
}

#[tokio::test]
async fn direct_connects_to_target() {
    let target = spawn_listener().await;
    let mut conn = ProxyConnector::new(ProxyConfig::Direct);
    let uri: http::Uri = format!("http://{target}").parse().unwrap();
    assert!(conn.call(uri).await.is_ok(), "direct connect should succeed");
}

/// mock HTTP CONNECT 代理：读 CONNECT 请求头 → 标记 → 回 200。
async fn spawn_http_connect_proxy(saw_connect: Arc<AtomicBool>) -> std::net::SocketAddr {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut client, _)) = l.accept().await else {
                break;
            };
            let saw = saw_connect.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut byte = [0u8; 1];
                while !buf.ends_with(b"\r\n\r\n") {
                    if client.read_exact(&mut byte).await.is_err() {
                        return;
                    }
                    buf.push(byte[0]);
                }
                if buf.starts_with(b"CONNECT ") {
                    saw.store(true, Ordering::SeqCst);
                }
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .await;
                // 握手验证即可，保持连接片刻避免 RST。
                let mut sink = [0u8; 64];
                let _ = client.read(&mut sink).await;
            });
        }
    });
    addr
}

#[tokio::test]
async fn http_connect_sends_connect_and_succeeds() {
    let saw = Arc::new(AtomicBool::new(false));
    let proxy = spawn_http_connect_proxy(saw.clone()).await;
    let mut conn =
        ProxyConnector::new(ProxyConfig::parse(Some(&format!("http://{proxy}"))).unwrap());
    let uri: http::Uri = "https://api.anthropic.com".parse().unwrap();
    let r = conn.call(uri).await;
    assert!(
        r.is_ok(),
        "CONNECT tunnel should succeed: {:?}",
        r.err().map(|e| e.to_string())
    );
    assert!(
        saw.load(Ordering::SeqCst),
        "proxy should have received CONNECT request"
    );
}

#[tokio::test]
async fn http_connect_propagates_proxy_failure_no_fallback() {
    // 代理端口几乎不可能有监听 → connector 必须报错，绝不静默回退直连。
    let mut conn = ProxyConnector::new(ProxyConfig::parse(Some("http://127.0.0.1:1")).unwrap());
    let uri: http::Uri = "https://api.anthropic.com".parse().unwrap();
    assert!(
        conn.call(uri).await.is_err(),
        "proxy connection failure must surface as error, not silent direct"
    );
}
