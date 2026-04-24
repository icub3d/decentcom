//! Test identity definitions used by the test-setup tool.
//!
//! Server seeding (channels, roles, messages, etc.) was moved out of this
//! crate as part of #101 — it now lives in `tools/sdk-seed`, which exercises
//! the same data through the public REST API via `decentcom-sdk`. What stays
//! here is the canonical list of test identities, since the keychain and
//! localStorage helpers still need to know who the test users are.

use ed25519_dalek::SigningKey;

pub struct User {
    pub name: &'static str,
    pub pubkey: String,
    pub hex_seed: String,
}

impl User {
    pub fn new(name: &'static str, seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let pub_bytes = signing_key.verifying_key().to_bytes();
        let pubkey = bs58::encode(&pub_bytes).into_string();
        let hex_seed = hex::encode(seed);
        Self {
            name,
            pubkey,
            hex_seed,
        }
    }

    pub fn find<'a>(users: &'a [Self], name: &str) -> &'a Self {
        users
            .iter()
            .find(|u| u.name == name)
            .unwrap_or_else(|| panic!("unknown user: {name}"))
    }
}

pub fn all_users() -> Vec<User> {
    vec![
        User::new("alice", [0x01; 32]),
        User::new("bob", [0x02; 32]),
        User::new("charlie", [0x03; 32]),
        User::new("dave", [0x04; 32]),
        User::new("bot-alpha", [0x10; 32]),
        User::new("bot-beta", [0x11; 32]),
    ]
}

pub fn print_summary(users: &[User]) {
    println!("\n{}", "=".repeat(60));
    println!("TEST ENVIRONMENT READY");
    println!("{}", "=".repeat(60));
    println!("\nTest Accounts (all stored in OS keychain):");
    println!("{}", "-".repeat(60));
    let (bots, humans): (Vec<_>, Vec<_>) = users.iter().partition(|u| u.name.starts_with("bot-"));
    for user in &humans {
        println!("  {:<10}  {}…", user.name, &user.pubkey[..20]);
    }
    println!();
    println!("Bot Accounts:");
    println!("{}", "-".repeat(60));
    for bot in &bots {
        println!("  {:<12}  pubkey: {}…", bot.name, &bot.pubkey[..20]);
    }
    println!("  bot-alpha — approved on Open Server (localhost:8081) by sdk-seed");
    println!("              runs as welcomebot (Procfile) using test-configs/welcomebot.toml");
    println!("  bot-beta  — not registered with any server");
    println!();
    println!("Server Layout (seeded by tools/sdk-seed via the REST API):");
    println!("{}", "-".repeat(60));
    println!("  Open Server     (localhost:8081) — open membership");
    println!("    Users: alice (admin), bob, charlie (restricted), dave");
    println!("  Private Server  (localhost:8082) — invite only");
    println!("    Users: alice (admin), bob (joined via invite)");
    println!("  Strict Server   (localhost:8083) — allowlist only");
    println!("    Users: alice (admin), bob (allowlisted)");
    println!();
    println!("Client localStorage pre-configured:");
    println!("  alice & bob — Open + Private + Strict servers added");
    println!("  charlie & dave — Open server added");
    println!("  Active account: alice (switch via account switcher in client)");
    println!("{}", "=".repeat(60));
}
