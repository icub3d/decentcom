use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use decentcom_sdk::identity::Identity;
use decentcom_sdk::rest::Client;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::context::Context;
use crate::error::{Error, Result};
use shared::gateway::{ReactionEventData, ThreadEventData, ThreadUpdateData};

use crate::events::{
    ChannelDeleteEvent, ChannelEvent, Event, MemberEvent, MemberModEvent, MemberRoleEvent,
    MessageDeleteEvent, MessageEvent, RoleDeleteEvent, RoleEvent,
};

// ── Handler type aliases ───────────────────────────────────────────────────

macro_rules! handler_type {
    ($name:ident, $evt:ty) => {
        type $name = Arc<
            dyn Fn(Context, $evt) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
                + Send
                + Sync,
        >;
    };
}

handler_type!(MessageHandler, MessageEvent);
handler_type!(MessageDeleteHandler, MessageDeleteEvent);
handler_type!(ChannelHandler, ChannelEvent);
handler_type!(ChannelDeleteHandler, ChannelDeleteEvent);
handler_type!(RoleHandler, RoleEvent);
handler_type!(RoleDeleteHandler, RoleDeleteEvent);
handler_type!(MemberRoleHandler, MemberRoleEvent);
handler_type!(MemberHandler, MemberEvent);
handler_type!(MemberModHandler, MemberModEvent);
handler_type!(ReactionHandler, ReactionEventData);
handler_type!(ThreadHandler, ThreadEventData);
handler_type!(ThreadUpdateHandler, ThreadUpdateData);

#[allow(clippy::type_complexity)]
fn wrap<F, Fut, E>(f: F) -> Arc<dyn Fn(Context, E) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync>
where
    F: Fn(Context, E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    E: 'static,
{
    Arc::new(move |ctx, evt| Box::pin(f(ctx, evt)))
}

/// Reconnect back-off: starts at 1 s, doubles each attempt, caps at 64 s.
fn backoff(attempt: u32) -> Duration {
    let secs = 2u64.pow(attempt).min(64);
    Duration::from_secs(secs)
}

/// The main bot builder and runner.
///
/// Register event handlers via the `on_*` methods then call [`Bot::run`].
pub struct Bot {
    config: Config,
    on_message: Option<MessageHandler>,
    on_message_update: Option<MessageHandler>,
    on_message_delete: Option<MessageDeleteHandler>,
    on_channel_create: Option<ChannelHandler>,
    on_channel_update: Option<ChannelHandler>,
    on_channel_delete: Option<ChannelDeleteHandler>,
    on_role_create: Option<RoleHandler>,
    on_role_update: Option<RoleHandler>,
    on_role_delete: Option<RoleDeleteHandler>,
    on_member_role_add: Option<MemberRoleHandler>,
    on_member_role_remove: Option<MemberRoleHandler>,
    on_member_join: Option<MemberHandler>,
    on_member_leave: Option<MemberHandler>,
    on_member_kick: Option<MemberModHandler>,
    on_member_ban: Option<MemberModHandler>,
    on_member_update: Option<MemberHandler>,
    on_reaction_add: Option<ReactionHandler>,
    on_reaction_remove: Option<ReactionHandler>,
    on_thread_create: Option<ThreadHandler>,
    on_thread_message_create: Option<ThreadHandler>,
    on_thread_update: Option<ThreadUpdateHandler>,
}

impl Bot {
    /// Create a bot from environment variables (`BOT_SERVER_URL`, `BOT_MNEMONIC`).
    pub fn from_env() -> Result<Self> {
        Self::with_config(Config::from_env()?)
    }

    /// Create a bot from a TOML config file, with env var overrides.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::with_config(Config::from_file(path)?)
    }

    /// Create a bot with an explicit [`Config`].
    pub fn with_config(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            on_message: None,
            on_message_update: None,
            on_message_delete: None,
            on_channel_create: None,
            on_channel_update: None,
            on_channel_delete: None,
            on_role_create: None,
            on_role_update: None,
            on_role_delete: None,
            on_member_role_add: None,
            on_member_role_remove: None,
            on_member_join: None,
            on_member_leave: None,
            on_member_kick: None,
            on_member_ban: None,
            on_member_update: None,
            on_reaction_add: None,
            on_reaction_remove: None,
            on_thread_create: None,
            on_thread_message_create: None,
            on_thread_update: None,
        })
    }

    // ── Handler registration ───────────────────────────────────────────────

    pub fn on_message<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MessageEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_message = Some(wrap(f)); self
    }

    pub fn on_message_update<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MessageEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_message_update = Some(wrap(f)); self
    }

    pub fn on_message_delete<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MessageDeleteEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_message_delete = Some(wrap(f)); self
    }

    pub fn on_channel_create<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ChannelEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_channel_create = Some(wrap(f)); self
    }

    pub fn on_channel_update<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ChannelEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_channel_update = Some(wrap(f)); self
    }

    pub fn on_channel_delete<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ChannelDeleteEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_channel_delete = Some(wrap(f)); self
    }

    pub fn on_role_create<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, RoleEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_role_create = Some(wrap(f)); self
    }

    pub fn on_role_update<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, RoleEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_role_update = Some(wrap(f)); self
    }

    pub fn on_role_delete<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, RoleDeleteEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_role_delete = Some(wrap(f)); self
    }

    pub fn on_member_role_add<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberRoleEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_role_add = Some(wrap(f)); self
    }

    pub fn on_member_role_remove<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberRoleEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_role_remove = Some(wrap(f)); self
    }

    pub fn on_member_join<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_join = Some(wrap(f)); self
    }

    pub fn on_member_leave<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_leave = Some(wrap(f)); self
    }

    pub fn on_member_kick<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberModEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_kick = Some(wrap(f)); self
    }

    pub fn on_member_ban<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberModEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_ban = Some(wrap(f)); self
    }

    pub fn on_member_update<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, MemberEvent) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_member_update = Some(wrap(f)); self
    }

    pub fn on_reaction_add<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ReactionEventData) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_reaction_add = Some(wrap(f)); self
    }

    pub fn on_reaction_remove<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ReactionEventData) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_reaction_remove = Some(wrap(f)); self
    }

    pub fn on_thread_create<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ThreadEventData) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_thread_create = Some(wrap(f)); self
    }

    pub fn on_thread_message_create<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ThreadEventData) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_thread_message_create = Some(wrap(f)); self
    }

    pub fn on_thread_update<F, Fut>(mut self, f: F) -> Self
    where F: Fn(Context, ThreadUpdateData) -> Fut + Send + Sync + 'static, Fut: Future<Output = anyhow::Result<()>> + Send + 'static {
        self.on_thread_update = Some(wrap(f)); self
    }

    // ── Event loop ─────────────────────────────────────────────────────────

    /// Connect, authenticate, and run the bot's event loop.
    ///
    /// Automatically reconnects with exponential back-off on disconnection.
    /// Returns only on an unrecoverable error (e.g. config error, auth failure
    /// on the first attempt).
    pub async fn run(self) -> Result<()> {
        let config = self.config;
        let identity = Identity::from_phrase(&config.mnemonic)
            .map_err(|e| Error::Config(format!("invalid mnemonic: {e}")))?;

        let handlers = Arc::new(Handlers {
            on_message: self.on_message,
            on_message_update: self.on_message_update,
            on_message_delete: self.on_message_delete,
            on_channel_create: self.on_channel_create,
            on_channel_update: self.on_channel_update,
            on_channel_delete: self.on_channel_delete,
            on_role_create: self.on_role_create,
            on_role_update: self.on_role_update,
            on_role_delete: self.on_role_delete,
            on_member_role_add: self.on_member_role_add,
            on_member_role_remove: self.on_member_role_remove,
            on_member_join: self.on_member_join,
            on_member_leave: self.on_member_leave,
            on_member_kick: self.on_member_kick,
            on_member_ban: self.on_member_ban,
            on_member_update: self.on_member_update,
            on_reaction_add: self.on_reaction_add,
            on_reaction_remove: self.on_reaction_remove,
            on_thread_create: self.on_thread_create,
            on_thread_message_create: self.on_thread_message_create,
            on_thread_update: self.on_thread_update,
        });

        let mut attempt: u32 = 0;
        loop {
            match connect_and_run(&config, &identity, handlers.clone()).await {
                Ok(()) => {
                    info!("gateway connection closed cleanly");
                    attempt = 0;
                }
                Err(e) => {
                    warn!("gateway error: {e}");
                }
            }

            let delay = backoff(attempt);
            info!("reconnecting in {}s (attempt {})", delay.as_secs(), attempt + 1);
            tokio::time::sleep(delay).await;
            attempt = attempt.saturating_add(1);
        }
    }
}

// ── Connection helper ──────────────────────────────────────────────────────

async fn connect_and_run(
    config: &Config,
    identity: &Identity,
    handlers: Arc<Handlers>,
) -> Result<()> {
    let client = Client::new(&config.server_url)?;
    let resp = client.authenticate_as_bot(identity).await?;
    info!("authenticated as bot user_id={}", resp.user_id);

    if let Some(name) = &config.display_name {
        if let Err(e) = client
            .profiles()
            .update(&decentcom_sdk::rest::models::UpdateProfileRequest {
                display_name: Some(name.clone()),
            })
            .await
        {
            warn!("failed to set display name: {e}");
        }
    }

    let ctx = Context::new(client.clone());
    let mut gw = client.gateway().connect().await?;
    let hello = gw.wait_for_hello().await?;
    info!(
        "connected to server '{}' with {} channels",
        hello.server_name,
        hello.channels.len()
    );

    for ch in &hello.channels {
        if let Err(e) = gw.subscribe(&ch.id).await {
            warn!("failed to subscribe to channel {}: {e}", ch.id);
        }
    }

    run_event_loop(gw, ctx, handlers).await
}

// ── Internal handler registry ──────────────────────────────────────────────

struct Handlers {
    on_message: Option<MessageHandler>,
    on_message_update: Option<MessageHandler>,
    on_message_delete: Option<MessageDeleteHandler>,
    on_channel_create: Option<ChannelHandler>,
    on_channel_update: Option<ChannelHandler>,
    on_channel_delete: Option<ChannelDeleteHandler>,
    on_role_create: Option<RoleHandler>,
    on_role_update: Option<RoleHandler>,
    on_role_delete: Option<RoleDeleteHandler>,
    on_member_role_add: Option<MemberRoleHandler>,
    on_member_role_remove: Option<MemberRoleHandler>,
    on_member_join: Option<MemberHandler>,
    on_member_leave: Option<MemberHandler>,
    on_member_kick: Option<MemberModHandler>,
    on_member_ban: Option<MemberModHandler>,
    on_member_update: Option<MemberHandler>,
    on_reaction_add: Option<ReactionHandler>,
    on_reaction_remove: Option<ReactionHandler>,
    on_thread_create: Option<ThreadHandler>,
    on_thread_message_create: Option<ThreadHandler>,
    on_thread_update: Option<ThreadUpdateHandler>,
}

async fn run_event_loop(
    mut gw: decentcom_sdk::gateway::GatewayClient,
    ctx: Context,
    handlers: Arc<Handlers>,
) -> Result<()> {
    loop {
        let raw = match gw.next_event().await {
            Some(e) => e,
            None => return Err(Error::Disconnected),
        };

        debug!(op = ?raw.op, "received gateway event");

        let event = match Event::from_gateway(&raw) {
            Ok(Some(e)) => e,
            Ok(None) => continue, // lifecycle op (Hello/Ready/Heartbeat)
            Err(e) => {
                warn!("failed to decode event: {e}");
                continue;
            }
        };

        let ctx = ctx.clone();
        let handlers = handlers.clone();

        tokio::spawn(async move {
            if let Err(e) = dispatch(ctx, event, &handlers).await {
                error!("handler error: {e:#}");
            }
        });
    }
}

macro_rules! dispatch_event {
    ($ctx:expr, $evt:expr, $handler:expr) => {
        if let Some(ref h) = $handler {
            h($ctx, $evt).await?;
        }
    };
}

async fn dispatch(ctx: Context, event: Event, h: &Handlers) -> anyhow::Result<()> {
    match event {
        Event::Message(e) => dispatch_event!(ctx, e, h.on_message),
        Event::MessageUpdate(e) => dispatch_event!(ctx, e, h.on_message_update),
        Event::MessageDelete(e) => dispatch_event!(ctx, e, h.on_message_delete),
        Event::ChannelCreate(e) => dispatch_event!(ctx, e, h.on_channel_create),
        Event::ChannelUpdate(e) => dispatch_event!(ctx, e, h.on_channel_update),
        Event::ChannelDelete(e) => dispatch_event!(ctx, e, h.on_channel_delete),
        Event::RoleCreate(e) => dispatch_event!(ctx, e, h.on_role_create),
        Event::RoleUpdate(e) => dispatch_event!(ctx, e, h.on_role_update),
        Event::RoleDelete(e) => dispatch_event!(ctx, e, h.on_role_delete),
        Event::MemberRoleAdd(e) => dispatch_event!(ctx, e, h.on_member_role_add),
        Event::MemberRoleRemove(e) => dispatch_event!(ctx, e, h.on_member_role_remove),
        Event::MemberJoin(e) => dispatch_event!(ctx, e, h.on_member_join),
        Event::MemberLeave(e) => dispatch_event!(ctx, e, h.on_member_leave),
        Event::MemberKick(e) => dispatch_event!(ctx, e, h.on_member_kick),
        Event::MemberBan(e) => dispatch_event!(ctx, e, h.on_member_ban),
        Event::MemberUpdate(e) => dispatch_event!(ctx, e, h.on_member_update),
        Event::ReactionAdd(e) => dispatch_event!(ctx, e, h.on_reaction_add),
        Event::ReactionRemove(e) => dispatch_event!(ctx, e, h.on_reaction_remove),
        Event::ThreadCreate(e) => dispatch_event!(ctx, e, h.on_thread_create),
        Event::ThreadMessageCreate(e) => dispatch_event!(ctx, e, h.on_thread_message_create),
        Event::ThreadUpdate(e) => dispatch_event!(ctx, e, h.on_thread_update),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_caps_at_64() {
        assert_eq!(backoff(0).as_secs(), 1);
        assert_eq!(backoff(1).as_secs(), 2);
        assert_eq!(backoff(6).as_secs(), 64);
        assert_eq!(backoff(10).as_secs(), 64);
    }
}
