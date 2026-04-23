//! `decentcom-bot` — Bot SDK for the decentcom protocol.
//!
//! Wraps the gateway WebSocket and REST API with typed event handlers and
//! action helpers so bot authors can build bots without reimplementing the
//! protocol.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use decentcom_bot::{Bot, Context, events::MessageEvent};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     Bot::from_env()?
//!         .on_message(handle_message)
//!         .run()
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handle_message(ctx: Context, evt: MessageEvent) -> anyhow::Result<()> {
//!     if evt.content.starts_with("!ping") {
//!         ctx.reply(&evt, "pong").await?;
//!     }
//!     Ok(())
//! }
//! ```

pub mod bot;
pub mod config;
pub mod context;
pub mod error;
pub mod events;

pub use bot::Bot;
pub use config::Config;
pub use context::Context;
pub use error::{Error, Result};
pub use events::{Event, MessageEvent};
