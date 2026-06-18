//! 阻塞式原始 HTTP client（用于驱动 daemon）。
//!
//! lift 自 `sieve-cli/tests/inbound_block.rs::fetch_response_body_raw` / `decode_chunked`。
//! 用 `std::net::TcpStream` + `Connection: close` 读到 EOF，绕过 hyper client 的
//! content-length 校验，能正确读取 sieve 注入 `sieve_blocked` 后的 chunked 响应。
//!
//! 在 `#[tokio::test]` 中调用时务必包 `tokio::task::spawn_blocking`，避免阻塞 executor
//! （否则 current_thread runtime 会死锁，详见源测试 `wait_for_http_ready` 注释）。

use bytes::Bytes;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// 最多重试次数（含首次）。覆盖 daemon 启动窗口/高并发下的瞬时连接重置。
const MAX_ATTEMPTS: usize = 4;

/// 阻塞发 HTTP POST，返回 `(status_code, response_headers, decoded_body)`。
///
/// - `base_url`：如 `http://127.0.0.1:1234`（取 `DaemonGuard::base_url()`）。
/// - `path`：如 `/v1/messages`。
/// - `headers`：额外请求头（`Host` / `Content-Length` / `Connection` 由本函数自动补；
///   传入的同名头会被自动头覆盖语义上忽略，建议只传业务头如 `content-type`）。
/// - `body`：请求体字节。
///
/// 响应 header 以小写 key 收进 `HashMap`；body 自动做 chunked 解码（plain body 原样返回）。
///
/// **可靠性**：daemon「端口已 LISTEN 但 accept loop 未就绪」的启动窗口、以及多测试并发
/// 高负载下，首个请求可能撞上 `ConnectionReset`/`ConnectionRefused`。本函数对这类**瞬时**
/// 失败（连接失败 / 写入失败 / 拿不到完整 status line）自动重试至多 [`MAX_ATTEMPTS`] 次
/// （线性退避），消除 e2e flake。业务级失败（如真返回 4xx/5xx）不重试，原样返回。
///
/// # Panics
///
/// `base_url` 解析失败时 panic。重试全部用尽仍拿不到响应时，返回 `status=0`（调用方断言会失败，
/// 比静默 panic 更易定位）。
#[must_use]
pub fn http_post(
    base_url: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, HashMap<String, String>, Vec<u8>) {
    let mut last = (0u16, HashMap::new(), Vec::new());
    for attempt in 1..=MAX_ATTEMPTS {
        match http_post_once(base_url, path, headers, body) {
            // 拿到有效 status line（含 4xx/5xx 业务响应）即视为成功，不重试。
            Ok(resp) if resp.0 != 0 => return resp,
            // status==0：响应不完整（多半是 reset），可重试。
            Ok(resp) => last = resp,
            // 连接 / 写入失败：瞬时错误，可重试。
            Err(_) => {}
        }
        if attempt < MAX_ATTEMPTS {
            std::thread::sleep(Duration::from_millis(50 * attempt as u64));
        }
    }
    last
}

/// 单次 HTTP POST 尝试。连接/写入失败返回 `Err`；能读到响应（哪怕被 reset 截断）返回 `Ok`。
fn http_post_once(
    base_url: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> std::io::Result<(u16, HashMap<String, String>, Vec<u8>)> {
    let (host, addr) = parse_base_url(base_url);

    let mut req = format!("POST {path} HTTP/1.1\r\n");
    req.push_str(&format!("Host: {host}\r\n"));
    for (k, v) in headers {
        // 跳过会与自动头冲突的项。
        let lk = k.to_ascii_lowercase();
        if lk == "host" || lk == "content-length" || lk == "connection" {
            continue;
        }
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    req.push_str("Connection: close\r\n\r\n");

    let mut raw_req = req.into_bytes();
    raw_req.extend_from_slice(body);

    let mut stream = TcpStream::connect(&addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.write_all(&raw_req)?;
    stream.flush()?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).ok(); // 容忍 connection reset（截断响应仍尽力解析）

    let (status, header_end) = parse_status_and_header_end(&raw);
    let header_block = &raw[..header_end.unwrap_or(raw.len())];
    let resp_headers = parse_headers(header_block);

    let raw_body: &[u8] = match header_end {
        Some(pos) => &raw[pos..],
        None => &[],
    };
    let body = decode_chunked(raw_body);
    Ok((status, resp_headers, body))
}

/// 发固定的流式（`stream:true`）POST `/v1/messages` 请求，返回 `(status, decoded_body)`。
///
/// lift 自 `inbound_block.rs::fetch_response_body_raw`（用于 SSE 入站拦截测试）。
///
/// # Panics
///
/// TCP 连接 / 写入失败时 panic。
#[must_use]
pub fn fetch_response_body_raw(base_url: &str) -> (u16, Bytes) {
    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, _headers, body) = http_post(
        base_url,
        "/v1/messages",
        &[("Content-Type", "application/json")],
        body_json.as_bytes(),
    );
    (status, Bytes::from(body))
}

/// 简单 chunked transfer encoding 解码器（不依赖第三方库）。
///
/// 格式：`<hex-len>\r\n<data>\r\n ... 0\r\n\r\n`。若输入不是有效 chunked（如 plain body），
/// 原样返回。lift 自 `inbound_block.rs::decode_chunked`。
#[must_use]
pub fn decode_chunked(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let Some(crlf_pos) = find_crlf(input, pos) else {
            result.extend_from_slice(input);
            return result;
        };
        let size_str = std::str::from_utf8(&input[pos..crlf_pos]).unwrap_or("0");
        let chunk_size = usize::from_str_radix(size_str.trim(), 16).unwrap_or(0);
        pos = crlf_pos + 2; // skip \r\n

        if chunk_size == 0 {
            break;
        }
        if pos + chunk_size > input.len() {
            result.extend_from_slice(&input[pos..]);
            break;
        }
        result.extend_from_slice(&input[pos..pos + chunk_size]);
        pos += chunk_size + 2; // skip data + \r\n
    }

    if result.is_empty() {
        result.extend_from_slice(input);
    }
    result
}

fn find_crlf(data: &[u8], start: usize) -> Option<usize> {
    (start..data.len().saturating_sub(1)).find(|&i| data[i] == b'\r' && data[i + 1] == b'\n')
}

/// 从 `http://host:port` 解析出 `(host_header, connect_addr)`，二者均为 `host:port`。
fn parse_base_url(base_url: &str) -> (String, String) {
    let stripped = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url);
    // 去掉可能存在的 path 部分。
    let host_port = stripped.split('/').next().unwrap_or(stripped).to_owned();
    (host_port.clone(), host_port)
}

/// 解析 status line 的状态码，并返回 body 起始偏移（`\r\n\r\n` 之后）。
fn parse_status_and_header_end(raw: &[u8]) -> (u16, Option<usize>) {
    let raw_str = String::from_utf8_lossy(raw);
    let status_code = raw_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);

    let sep = b"\r\n\r\n";
    let header_end = raw
        .windows(sep.len())
        .position(|w| w == sep)
        .map(|pos| pos + sep.len());
    (status_code, header_end)
}

/// 把 header 块（含 status line）解析为小写 key → value 的 `HashMap`。
fn parse_headers(header_block: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let text = String::from_utf8_lossy(header_block);
    for line in text.split("\r\n").skip(1) {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_ascii_lowercase(), v.trim().to_owned());
        }
    }
    map
}
