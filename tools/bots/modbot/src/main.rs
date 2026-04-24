mod config;
mod detector;

use std::sync::Arc;

use decentcom_bot::{events::MessageEvent, Bot, Config as BotConfig, Context};
use detector::{Detector, Violation};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::config::{Action, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load("modbot.toml")?;
    let bot_cfg = BotConfig {
        server_url: cfg.server_url.clone(),
        mnemonic: cfg.mnemonic.clone(),
        seed_hex: None,
        display_name: cfg.display_name.clone(),
    };

    let cfg = Arc::new(cfg);
    let detector = Arc::new(Mutex::new(Detector::new(
        cfg.detection.duplicate_threshold,
        cfg.detection.duplicate_window_secs,
        cfg.detection.link_threshold,
        cfg.detection.link_window_secs,
    )));

    Bot::with_config(bot_cfg)?
        .on_message({
            let cfg = cfg.clone();
            let detector = detector.clone();
            move |ctx, evt| {
                let cfg = cfg.clone();
                let detector = detector.clone();
                async move { handle_message(ctx, evt, &cfg, detector).await }
            }
        })
        .run()
        .await?;

    Ok(())
}

async fn handle_message(
    ctx: Context,
    evt: MessageEvent,
    cfg: &Config,
    detector: Arc<Mutex<Detector>>,
) -> anyhow::Result<()> {
    // Ignore bot messages.
    if evt.author_id == ctx.bot_user_id() {
        return Ok(());
    }

    let violations = {
        let mut d = detector.lock().await;
        d.record_message(&evt.author_id, &evt.content)
    };

    for violation in violations {
        let (action, reason) = match violation {
            Violation::DuplicateMessages => (
                &cfg.action.on_duplicate,
                format!(
                    "sending duplicate messages ({} times in {}s)",
                    cfg.detection.duplicate_threshold,
                    cfg.detection.duplicate_window_secs
                ),
            ),
            Violation::LinkFlood => (
                &cfg.action.on_link_flood,
                format!(
                    "link flood ({} links in {}s)",
                    cfg.detection.link_threshold,
                    cfg.detection.link_window_secs
                ),
            ),
        };

        info!(
            user_id = %evt.author_id,
            action = action.as_str(),
            reason = %reason,
            "modbot action triggered"
        );

        match action {
            Action::Warn => {
                if let Some(ch) = &cfg.action.warn_channel_id {
                    let msg = format!(
                        "Warning: <{}> — {}",
                        evt.author_id, reason
                    );
                    if let Err(e) = ctx.send(ch, &msg).await {
                        warn!("failed to post warning: {e}");
                    }
                }
            }
            Action::Kick => {
                // Clear state so a re-join starts fresh.
                detector.lock().await.clear_user(&evt.author_id);
                if let Err(e) = ctx.kick(&evt.author_id).await {
                    warn!("failed to kick {}: {e}", evt.author_id);
                }
            }
            Action::Ban => {
                detector.lock().await.clear_user(&evt.author_id);
                if let Err(e) = ctx.ban(&evt.author_id, Some(reason)).await {
                    warn!("failed to ban {}: {e}", evt.author_id);
                }
            }
        }
    }

    Ok(())
}
