//! Bounded buffering and concurrency leases for proxy traffic.
use anyhow::{anyhow, Result};
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use http_body_util::BodyStream;
use std::{
    pin::Pin,
    sync::{Arc, LazyLock},
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const MAX_BODY_BYTES: usize = 16 << 20;
pub const BODY_TIMEOUT: Duration = Duration::from_secs(60);
pub static CONNECTIONS: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(128)));
pub static REQUESTS: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(32)));

/// Bound total bytes and elapsed time, including bodies without Content-Length.
pub async fn collect<B>(body: B, max: usize, deadline: Duration) -> Result<Bytes>
where
    B: http_body::Body<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    tokio::time::timeout(deadline, async move {
        let mut stream = Box::pin(BodyStream::new(body));
        let mut bytes = BytesMut::new();
        while let Some(frame) = stream.next().await {
            let frame = frame.map_err(|e| anyhow!("body read failed: {e}"))?;
            if let Some(data) = frame.data_ref() {
                if bytes.len().saturating_add(data.len()) > max {
                    return Err(anyhow!("body exceeds {max} bytes"));
                }
                bytes.extend_from_slice(data);
            }
        }
        Ok(bytes.freeze())
    })
    .await
    .map_err(|_| anyhow!("body read timed out"))?
}

/// Keep the request slot until the streaming response finishes or is dropped.
pub struct LeasedBody<B> {
    body: Pin<Box<B>>,
    lease: Option<OwnedSemaphorePermit>,
}
impl<B> LeasedBody<B> {
    pub fn new(body: B, lease: OwnedSemaphorePermit) -> Self {
        Self {
            body: Box::pin(body),
            lease: Some(lease),
        }
    }
}
impl<B: http_body::Body> http_body::Body for LeasedBody<B> {
    type Data = B::Data;
    type Error = B::Error;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        let frame = this.body.as_mut().poll_frame(cx);
        if matches!(frame, Poll::Ready(None) | Poll::Ready(Some(Err(_)))) {
            this.lease.take();
        }
        frame
    }
    fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }
    fn size_hint(&self) -> http_body::SizeHint {
        self.body.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::{Full, StreamBody};
    #[tokio::test]
    async fn rejects_oversize_and_stalled_bodies() {
        assert!(
            collect(Full::new(Bytes::from_static(b"12345")), 4, BODY_TIMEOUT)
                .await
                .is_err()
        );
        let pending =
            futures_util::stream::pending::<Result<http_body::Frame<Bytes>, std::io::Error>>();
        assert!(
            collect(StreamBody::new(pending), 4, Duration::from_millis(5))
                .await
                .is_err()
        );
        assert_eq!(
            collect(Full::new(Bytes::from_static(b"1234")), 4, BODY_TIMEOUT)
                .await
                .unwrap(),
            "1234"
        );
    }
    #[tokio::test]
    async fn response_drop_releases_capacity() {
        let slots = Arc::new(Semaphore::new(1));
        let lease = slots.clone().try_acquire_owned().unwrap();
        let body = LeasedBody::new(Full::new(Bytes::new()), lease);
        assert!(slots.clone().try_acquire_owned().is_err());
        drop(body);
        assert!(slots.try_acquire_owned().is_ok());
    }
}
