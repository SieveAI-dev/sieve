//! 帧读取器：`read_buf` + `memchr` 手动扫描换行，替代无界 `BufReader::lines()`。
//!
//! 实现 SPEC-005 §1.3.1 规范性帧接收算法（双方 MUST 遵守）：
//! - 单帧（含尾部 `\n`）> 1 MiB → 关连接（`FrameError::OversizeFrame`）
//! - partial remainder > 1 MiB 且无 newline → 关连接（`FrameError::OversizeRemainder`）
//! - 解析失败 / EOF → 返回 `Ok(None)`，**不**关连接（§1.3.1 第 5 条）
//! - 多帧粘包正确处理（内循环先消费所有完整帧）
//!
//! 禁止在 debug/trace 日志中记录 `frame_buf` 内容（§1.3.1 第 4 条）。

use bytes::BytesMut;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

/// 单帧上限：1 MiB（SPEC-005 §1.1）。
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// 每次从 socket 读取的最大字节数（chunk size，不影响帧上限）。
const READ_CHUNK_SIZE: usize = 64 * 1024;

/// 帧读取错误（触发关连接的条件，SPEC-005 §1.3.1）。
#[derive(Debug, Error)]
pub enum FrameError {
    /// 单帧（含尾部 `\n`）超过 1 MiB，接收方 MUST 关闭连接。
    #[error("oversize frame: {size_bytes} bytes exceeds {MAX_FRAME_BYTES}")]
    OversizeFrame {
        /// 帧字节数（含 `\n`）。
        size_bytes: usize,
    },
    /// partial remainder（无 `\n`）超过 1 MiB，接收方 MUST 关闭连接。
    #[error("oversize remainder: {size_bytes} bytes exceeds {MAX_FRAME_BYTES}")]
    OversizeRemainder {
        /// remainder 字节数。
        size_bytes: usize,
    },
    /// 底层 IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// SPEC-005 §1.3.1 帧读取器。
///
/// 封装可增长 `BytesMut` 缓冲区，跨多次 `read_buf` 调用保留 remainder。
/// 使用 `memchr::memchr(b'\n', &buf)` 找帧边界（SIMD 加速）。
///
/// # 使用方式
///
/// ```ignore
/// let mut reader = FrameReader::new(read_half);
/// loop {
///     match reader.read_frame().await {
///         Ok(Some(frame)) => { /* frame 已去尾 \n，UTF-8 bytes */ }
///         Ok(None) => break, // EOF 或解析失败，不关连接
///         Err(e) => {
///             // OversizeFrame / OversizeRemainder → 必须关连接 + audit
///             return Err(e);
///         }
///     }
/// }
/// ```
pub struct FrameReader<R> {
    reader: R,
    buf: BytesMut,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    /// 创建新的 `FrameReader`，初始缓冲区容量为 `READ_CHUNK_SIZE`。
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: BytesMut::with_capacity(READ_CHUNK_SIZE),
        }
    }

    /// 读取下一帧（不含尾部 `\n`）。
    ///
    /// - 返回 `Ok(Some(bytes))`：一个完整帧。
    /// - 返回 `Ok(None)`：EOF（对端关闭连接）——上层应正常断开，不视为错误。
    /// - 返回 `Err(FrameError::OversizeFrame | OversizeRemainder)`：超限，
    ///   调用方 MUST 关闭连接 + 写 audit `ipc_oversize_frame`。
    ///
    /// 实现严格遵守 §1.3.1 算法顺序：
    /// 1. 先循环消费已有缓冲中所有完整帧（内层 while）。
    /// 2. 无完整帧时调用 `read_buf` 追加新数据。
    /// 3. 判 remainder > MAX_FRAME_BYTES。
    pub async fn read_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        loop {
            // 1) 先在已有缓冲中循环扫描完整帧（多帧粘包场景）。
            if let Some(frame) = self.try_consume_frame()? {
                return Ok(Some(frame));
            }

            // 2) 缓冲中无完整帧，尝试从 socket 读更多数据。
            let n = self.reader.read_buf(&mut self.buf).await?;
            if n == 0 {
                // EOF：对端关闭连接，正常返回 None。
                return Ok(None);
            }

            // 3) 追加数据后，再次检查完整帧（下一次循环处理）。
            // 检查 remainder 超限（先看完整帧，再看 remainder——§1.3.1 MUST 约束 2）。
            // 这里先不判，让 try_consume_frame 在下一轮循环扫描完整帧后再判。
        }
    }

    /// 尝试从内部缓冲区取出一个完整帧（含 §1.3.1 oversize 检查）。
    ///
    /// - `Ok(Some(frame))`：找到完整帧，缓冲区已推进（remainder 保留）。
    /// - `Ok(None)`：无 `\n`，同时检查 remainder 超限（Err）或继续等待（None）。
    /// - `Err(FrameError::OversizeFrame)`：帧长超 1 MiB。
    /// - `Err(FrameError::OversizeRemainder)`：无 `\n` 且 buf > 1 MiB。
    fn try_consume_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        match memchr::memchr(b'\n', &self.buf) {
            Some(idx) => {
                let frame_len = idx + 1; // 含 \n
                if frame_len > MAX_FRAME_BYTES {
                    return Err(FrameError::OversizeFrame {
                        size_bytes: frame_len,
                    });
                }
                // 切走完整帧，保留 remainder。
                let frame = self.buf.split_to(frame_len);
                // 返回不含尾部 \n 的帧内容。
                Ok(Some(frame[..idx].to_vec()))
            }
            None => {
                // 无完整帧：检查 remainder 是否已超限（§1.3.1 步骤 2）。
                if self.buf.len() > MAX_FRAME_BYTES {
                    return Err(FrameError::OversizeRemainder {
                        size_bytes: self.buf.len(),
                    });
                }
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 辅助：把 bytes 包成 AsyncRead ──────────────────────────────────────────

    fn reader_from_bytes(data: &[u8]) -> impl AsyncRead + Unpin + '_ {
        tokio_test::io::Builder::new().read(data).build()
    }

    // ── 用例 1：单帧 < 1 MiB ──────────────────────────────────────────────────

    /// §1.3.1 正常路径：一个完整帧（< 1 MiB）应返回 Ok(Some(bytes))。
    #[tokio::test]
    async fn single_frame_under_limit() {
        let data = b"{\"jsonrpc\":\"2.0\",\"method\":\"sieve.hello\"}\n";
        let mut reader = FrameReader::new(reader_from_bytes(data));
        let frame = reader.read_frame().await.expect("read_frame");
        assert!(frame.is_some(), "应返回 Some(frame)");
        let bytes = frame.unwrap();
        assert_eq!(bytes, b"{\"jsonrpc\":\"2.0\",\"method\":\"sieve.hello\"}");
    }

    // ── 用例 2：单帧恰好 1 MiB ─────────────────────────────────────────────────

    /// §1.3.1：单帧恰好 1 MiB（含 \n）= 允许上限，应成功返回。
    #[tokio::test]
    async fn single_frame_exactly_at_limit() {
        // payload = MAX_FRAME_BYTES - 1 字节 + '\n' = MAX_FRAME_BYTES 总计。
        let mut data = vec![b'x'; MAX_FRAME_BYTES - 1];
        data.push(b'\n');
        let mut reader = FrameReader::new(reader_from_bytes(&data));
        let frame = reader.read_frame().await.expect("read_frame");
        assert!(frame.is_some(), "1 MiB 帧应允许");
        assert_eq!(frame.unwrap().len(), MAX_FRAME_BYTES - 1);
    }

    // ── 用例 3：单帧 > 1 MiB → OversizeFrame ─────────────────────────────────

    /// §1.3.1：单帧（含 \n）超过 1 MiB → 必须返回 OversizeFrame 错误。
    #[tokio::test]
    async fn single_frame_over_limit_returns_oversize_error() {
        // payload = MAX_FRAME_BYTES 字节 + '\n' = MAX_FRAME_BYTES + 1 总计。
        let mut data = vec![b'x'; MAX_FRAME_BYTES];
        data.push(b'\n');
        let mut reader = FrameReader::new(reader_from_bytes(&data));
        let result = reader.read_frame().await;
        assert!(
            matches!(result, Err(FrameError::OversizeFrame { .. })),
            "超限帧应返回 OversizeFrame，实际: {result:?}"
        );
    }

    // ── 用例 4：多帧粘包（一次 read_buf 含 2 个完整帧）────────────────────────

    /// §1.3.1：多帧粘包场景——一次 read_buf 拿到 2 个完整帧，两帧都要能读出。
    #[tokio::test]
    async fn multiple_frames_in_one_read() {
        let data = b"{\"method\":\"a\"}\n{\"method\":\"b\"}\n";
        let mut reader = FrameReader::new(reader_from_bytes(data));

        let frame1 = reader
            .read_frame()
            .await
            .expect("frame1 io")
            .expect("frame1 some");
        assert_eq!(frame1, b"{\"method\":\"a\"}");

        let frame2 = reader
            .read_frame()
            .await
            .expect("frame2 io")
            .expect("frame2 some");
        assert_eq!(frame2, b"{\"method\":\"b\"}");

        // EOF
        let eof = reader.read_frame().await.expect("eof io");
        assert!(eof.is_none(), "应返回 None on EOF");
    }

    // ── 用例 5：半帧 + EOF → Ok(None) 不关连接 ────────────────────────────────

    /// §1.3.1 第 5 条：解析失败 / EOF 时 MUST 仅记录元信息，**不**关连接。
    /// 此处验证半帧（无 \n）+ 流结束时返回 Ok(None) 而非 Err。
    #[tokio::test]
    async fn partial_frame_followed_by_eof_returns_none() {
        // 半帧：无 \n，EOF。
        let data = b"{\"method\":\"incomplete\"";
        let mut reader = FrameReader::new(reader_from_bytes(data));
        let result = reader.read_frame().await.expect("should not be io error");
        assert!(
            result.is_none(),
            "半帧 + EOF 应返回 Ok(None)，不关连接; got {result:?}"
        );
    }
}
