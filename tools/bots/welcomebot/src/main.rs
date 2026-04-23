mod config;

use std::sync::Arc;

use decentcom_bot::{events::MemberEvent, Bot, Config as BotConfig, Context};
use tracing::warn;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load("welcomebot.toml")?;
    let bot_cfg = BotConfig {
        server_url: cfg.server_url.clone(),
        mnemonic: cfg.mnemonic.clone(),
        display_name: cfg.display_name.clone(),
    };

    let cfg = Arc::new(cfg);

    Bot::with_config(bot_cfg)?
        .on_member_join(move |ctx: Context, evt: MemberEvent| {
            let cfg = cfg.clone();
            async move { handle_join(ctx, evt, &cfg).await }
        })
        .run()
        .await?;

    Ok(())
}

async fn handle_join(ctx: Context, evt: MemberEvent, cfg: &Config) -> anyhow::Result<()> {
    if evt.is_bot {
        return Ok(());
    }

    let display_name = evt
        .display_name
        .as_deref()
        .unwrap_or("new member")
        .to_string();

    let msg = cfg
        .welcome_template
        .replace("{display_name}", &display_name)
        .replace("{pubkey}", &evt.pubkey);

    if let Err(e) = ctx.send(&cfg.welcome_channel_id, &msg).await {
        warn!("failed to send welcome message: {e}");
    }

    if let Some(rules) = &cfg.rules_message {
        if let Err(e) = ctx.send(&cfg.welcome_channel_id, rules).await {
            warn!("failed to send rules message: {e}");
        }
    }

    Ok(())
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
