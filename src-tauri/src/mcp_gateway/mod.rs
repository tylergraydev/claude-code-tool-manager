//! MCP Gateway Module
//!
//! Provides an MCP gateway server that aggregates tools from multiple backend MCPs
//! into a single endpoint. This allows Claude to connect to one gateway instead of
//! managing multiple MCP configurations.

pub mod backend;
pub mod server;
pub mod tools;

pub use server::{GatewayServerConfig, GatewayServerState, GatewayServerStatus};
