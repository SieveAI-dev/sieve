//! JSON-RPC 2.0 envelope 结构。
//!
//! 手写实现以避免引入大型 jsonrpc crate 依赖。关联：ADR-013 §2（传输协议选型）。
//!
//! **零 IO 约束**：本文件仅 import serde / std，禁止引入任何 IO / 异步 / 运行时依赖。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 请求（通知或有 id 的调用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl Request {
    /// 构造一个有 id 的调用请求。
    pub fn call(method: impl Into<String>, params: Value, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            method: method.into(),
            params: Some(params),
            id: Some(id),
        }
    }
}

/// JSON-RPC 2.0 成功响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
    pub id: Value,
}

/// JSON-RPC 2.0 错误对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
