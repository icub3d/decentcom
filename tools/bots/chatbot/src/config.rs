use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub mnemonic: String,
    pub seed_hex: Option<String>,
    pub display_name: Option<String>,
    pub provider: ProviderConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderConfig {
    Ollama {
        url: String,
        model: String,
        system_prompt: Option<String>,
        temperature: Option<f32>,
    },
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_path() -> PathBuf {
        std::env::var("BOT_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("chatbot.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_config() {
        let toml_str = r#"
        server_url = "ws://localhost:3000"
        mnemonic = "test mnemonic"
        display_name = "testbot"

        [provider]
        type = "ollama"
        url = "http://localhost:11434"
        model = "llama3"
        temperature = 0.8
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server_url, "ws://localhost:3000");
        assert_eq!(config.mnemonic, "test mnemonic");
        assert_eq!(config.display_name.unwrap(), "testbot");
        
        let ProviderConfig::Ollama { url, model, system_prompt, temperature } = config.provider;
        assert_eq!(url, "http://localhost:11434");
        assert_eq!(model, "llama3");
        assert_eq!(system_prompt, None);
        assert_eq!(temperature, Some(0.8));
    }
}
