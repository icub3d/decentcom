//! Rust SDK for the decentcom protocol.
//!
//! Provides identity management, REST API wrappers, and a low-level gateway
//! (WebSocket) client. Wire types live in the `shared` crate; this crate
//! implements the logic for speaking the protocol.

pub mod error;
pub mod identity;
pub mod rest;
pub mod gateway;

pub use error::{Error, Result};
pub use identity::Identity;
pub use rest::{Client, Config};

pub use shared;
