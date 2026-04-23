mod config;
mod sink;

use std::sync::Arc;

use chrono::Utc;
use decentcom_bot::{
    events::{MemberEvent, MemberModEvent, MessageDeleteEvent},
    Bot, Config as BotConfig, Context,
};
use serde_json::json;
use tracing::info;

use crate::config::Config;
use crate::sink::Sink;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load("auditbot.toml")?;
    let bot_cfg = BotConfig {
        server_url: cfg.server_url.clone(),
        mnemonic: cfg.mnemonic.clone(),
        seed_hex: None,
        display_name: cfg.display_name.clone(),
    };

    let sink = Arc::new(Sink::from_config(&cfg)?);
    let cfg = Arc::new(cfg);

    let mut bot = Bot::with_config(bot_cfg)?;

    if cfg.filter.member_kick {
        let sink = sink.clone();
        bot = bot.on_member_kick(move |_ctx: Context, evt: MemberModEvent| {
            let sink = sink.clone();
            async move {
                info!(user_id = %evt.user_id, "audit: member kicked");
                let record = json!({
                    "event": "MEMBER_KICK",
                    "timestamp": Utc::now(),
                    "user_id": evt.user_id,
                    "pubkey": evt.pubkey,
                    "reason": evt.reason,
                });
                sink.write(&record).await;
                Ok(())
            }
        });
    }

    if cfg.filter.member_ban {
        let sink = sink.clone();
        bot = bot.on_member_ban(move |_ctx: Context, evt: MemberModEvent| {
            let sink = sink.clone();
            async move {
                info!(user_id = %evt.user_id, "audit: member banned");
                let record = json!({
                    "event": "MEMBER_BAN",
                    "timestamp": Utc::now(),
                    "user_id": evt.user_id,
                    "pubkey": evt.pubkey,
                    "reason": evt.reason,
                });
                sink.write(&record).await;
                Ok(())
            }
        });
    }

    if cfg.filter.message_delete {
        let sink = sink.clone();
        bot = bot.on_message_delete(move |_ctx: Context, evt: MessageDeleteEvent| {
            let sink = sink.clone();
            async move {
                info!(message_id = %evt.id, "audit: message deleted");
                let record = json!({
                    "event": "MESSAGE_DELETE",
                    "timestamp": Utc::now(),
                    "message_id": evt.id,
                    "channel_id": evt.channel_id,
                });
                sink.write(&record).await;
                Ok(())
            }
        });
    }

    if cfg.filter.member_join {
        let sink = sink.clone();
        bot = bot.on_member_join(move |_ctx: Context, evt: MemberEvent| {
            let sink = sink.clone();
            async move {
                info!(user_id = %evt.user_id, "audit: member joined");
                let record = json!({
                    "event": "MEMBER_JOIN",
                    "timestamp": Utc::now(),
                    "user_id": evt.user_id,
                    "pubkey": evt.pubkey,
                    "is_bot": evt.is_bot,
                });
                sink.write(&record).await;
                Ok(())
            }
        });
    }

    if cfg.filter.member_leave {
        let sink = sink.clone();
        bot = bot.on_member_leave(move |_ctx: Context, evt: MemberEvent| {
            let sink = sink.clone();
            async move {
                info!(user_id = %evt.user_id, "audit: member left");
                let record = json!({
                    "event": "MEMBER_LEAVE",
                    "timestamp": Utc::now(),
                    "user_id": evt.user_id,
                    "pubkey": evt.pubkey,
                });
                sink.write(&record).await;
                Ok(())
            }
        });
    }

    bot.run().await?;
    Ok(())
}
