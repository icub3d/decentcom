mod config;

use std::sync::Arc;

use decentcom_bot::{events::MemberEvent, Bot, Config as BotConfig, Context};
use tokio::sync::OnceCell;
use tracing::warn;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let path = Config::default_path();
    let cfg = Config::load(&path)?;

    let bot_cfg = BotConfig {
        server_url: cfg.server_url.clone(),
        mnemonic: cfg.mnemonic.clone(),
        seed_hex: cfg.seed_hex.clone(),
        display_name: cfg.display_name.clone(),
    };

    let cfg = Arc::new(cfg);
    // Resolved once on first member-join and reused across reconnects.
    let channel_id: Arc<OnceCell<String>> = Arc::new(OnceCell::new());

    Bot::with_config(bot_cfg)?
        .on_member_join(move |ctx: Context, evt: MemberEvent| {
            let cfg = cfg.clone();
            let channel_id = channel_id.clone();
            async move { handle_join(ctx, evt, &cfg, &channel_id).await }
        })
        .run()
        .await?;

    Ok(())
}

async fn handle_join(
    ctx: Context,
    evt: MemberEvent,
    cfg: &Config,
    channel_id: &OnceCell<String>,
) -> anyhow::Result<()> {
    if evt.is_bot {
        return Ok(());
    }

    // Resolve channel name → ID once; reuse on subsequent events.
    let cid = channel_id
        .get_or_try_init(|| async {
            resolve_channel(ctx.sdk(), &cfg.welcome_channel).await
        })
        .await?;

    let display_name = evt
        .display_name
        .as_deref()
        .unwrap_or("new member")
        .to_string();

    let msg = cfg
        .welcome_template
        .replace("{display_name}", &display_name)
        .replace("{pubkey}", &evt.pubkey);

    if let Err(e) = ctx.send(cid, &msg).await {
        warn!("failed to send welcome message: {e}");
    }

    if let Some(rules) = &cfg.rules_message {
        if let Err(e) = ctx.send(cid, rules).await {
            warn!("failed to send rules message: {e}");
        }
    }

    Ok(())
}

/// Find a channel by name or ID. Returns the channel's server-assigned ID.
async fn resolve_channel(
    client: &decentcom_sdk::rest::Client,
    name_or_id: &str,
) -> anyhow::Result<String> {
    let channels = client.channels().list().await?;
    channels
        .iter()
        .find(|c| c.name == name_or_id || c.id == name_or_id)
        .map(|c| c.id.clone())
        .ok_or_else(|| anyhow::anyhow!("channel '{}' not found on server", name_or_id))
}

#[cfg(test)]
mod tests {
    #[test]
    fn template_interpolation() {
        let tmpl = "Welcome, **{display_name}**! pubkey={pubkey}";
        let result = tmpl
            .replace("{display_name}", "Alice")
            .replace("{pubkey}", "abc123");
        assert_eq!(result, "Welcome, **Alice**! pubkey=abc123");
    }
}
