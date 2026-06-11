//! IPC protocol for ff.
//!
//! This crate implements the JSON-RPC 2.0 protocol over Unix domain sockets
//! for communication between the ff CLI and the ff daemon.

pub mod codec;
pub mod messages;
pub mod protocol;
pub mod transport;

pub use codec::JsonRpcCodec;
pub use messages::*;
pub use protocol::{
    ErrorCode, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    ProtocolError,
};
pub use transport::IpcTransport;
