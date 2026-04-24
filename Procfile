open: cargo watch -i client -p server -x "run -p server -- --config test-configs/open.toml"
private: cargo watch -i client -p server -x "run -p server -- --config test-configs/private.toml"
strict: cargo watch -i client -p server -x "run -p server -- --config test-configs/strict.toml"
seed: cargo run -q --bin sdk-seed
welcomebot: WELCOMEBOT_CONFIG=test-configs/welcomebot.toml cargo run -q --bin welcomebot
client: cd client && pnpm tauri dev
