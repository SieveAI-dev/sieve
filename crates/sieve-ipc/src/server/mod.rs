//! IPC server 实现（JSON-RPC 2.0 over Unix socket）。
//!
//! 供 sieve-cli daemon 进程调用，管理客户端连接、路由控制面请求、
//! fan-out 广播通知。关联：ADR-013、ADR-028。

pub mod socket_server;

pub use socket_server::{
    BroadcastPlan, ControlError, ControlPlaneRequest, HelloBuilder, IpcServer, OversizeCallback,
    OversizeKind,
};
