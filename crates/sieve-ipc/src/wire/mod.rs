//! wire DTO 适配层（SPEC-005 §6.0）。
//!
//! daemon 在通过 Unix socket 向 GUI 发送 JSON-RPC 消息前，必须经过本模块的 wire DTO
//! 转换，而非直接序列化内部 Rust 结构。两条路径的演化是解耦的：
//!
//! - **socket JSON-RPC（GUI 路径）** → 经过 wire DTO 适配层（本模块）
//! - **pending file（hook 路径）** → 用 daemon 内部结构直接 serialize（不受本模块约束）

pub mod request_decision_wire;

pub use request_decision_wire::{
    IssueWire, MergedRequestDecisionWire, RequestDecisionWire, RequestDecisionWireKind,
};
