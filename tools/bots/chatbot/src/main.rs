mod config;

use std::sync::Arc;

use decentcom_bot::{events::MessageEvent, Bot, Config as BotConfig, Context};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::config::{Config, ProviderConfig};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: Option<String>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

struct AppState {
    config: Config,
    http: reqwest::Client,
}

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

    let state = Arc::new(AppState {
        config: cfg,
        http: reqwest::Client::new(),
    });

    Bot::with_config(bot_cfg)?
        .on_message(move |ctx: Context, evt: MessageEvent| {
            let state = state.clone();
            async move { handle_message(ctx, evt, state).await }
        })
        .run()
        .await?;

    Ok(())
}

async fn handle_message(
    ctx: Context,
    evt: MessageEvent,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    // Ignore our own messages
    if evt.author_id == ctx.bot_user_id() {
        return Ok(());
    }

    let bot_name = state
        .config
        .display_name
        .as_deref()
        .unwrap_or("chatbot");
    
    let mention1 = format!("@{}", bot_name);
    let mention2 = format!("<@{}>", ctx.bot_user_id());

    let mut prompt = evt.content.clone();
    let is_mentioned = prompt.contains(&mention1) || prompt.contains(&mention2);

    if !is_mentioned {
        return Ok(());
    }

    // Strip mentions from the prompt
    prompt = prompt.replace(&mention1, "").replace(&mention2, "").trim().to_string();

    match &state.config.provider {
        ProviderConfig::Ollama {
            url,
            model,
            system_prompt,
            temperature,
        } => {
            let req = OllamaRequest {
                model: model.clone(),
                prompt,
                system: system_prompt.clone(),
                stream: false,
                options: Some(OllamaOptions {
                    temperature: *temperature,
                }),
            };

            let endpoint = format!("{}/api/generate", url.trim_end_matches('/'));
            
            let res = state.http.post(&endpoint).json(&req).send().await;
            
            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(ollama_resp) = resp.json::<OllamaResponse>().await {
                            if let Err(e) = ctx.reply(&evt, &ollama_resp.response).await {
                                error!("Failed to send reply: {e}");
                            }
                        } else {
                            error!("Failed to parse Ollama response");
                        }
                    } else {
                        error!("Ollama returned status: {}", resp.status());
                    }
                }
                Err(e) => {
                    error!("Failed to contact Ollama: {e}");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_mentions() {
        let bot_name = "chatbot";
        let bot_id = "u123";
        let mention1 = format!("@{}", bot_name);
        let mention2 = format!("<@{}>", bot_id);
        
        let content = "Hello @chatbot, how are you?";
        let cleaned = content.replace(&mention1, "").replace(&mention2, "").trim().to_string();
        assert_eq!(cleaned, "Hello , how are you?");
    }

    #[test]
    fn ollama_response_parsing() {
        let json = r#"{"model":"llama3","created_at":"2023-08-04T19:22:45.499127Z","response":"I am doing well, thank you!","done":true}"#;
        let resp: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response, "I am doing well, thank you!");
    }

    #[test]
    fn ollama_request_serialization() {
        let req = OllamaRequest {
            model: "llama3".to_string(),
            prompt: "Hello".to_string(),
            system: None,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""model":"llama3""#));
        assert!(json.contains(r#""prompt":"Hello""#));
        assert!(json.contains(r#""stream":false"#));
        assert!(json.contains(r#""temperature":0.7"#));
    }
}
