pub mod servers;

use ed25519_dalek::SigningKey;

pub struct User {
    pub name: &'static str,
    #[allow(dead_code)]
    pub seed: [u8; 32],
    pub pubkey: String,
    pub hex_seed: String,
    pub user_id: String,
}

impl User {
    pub fn new(name: &'static str, seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let pub_bytes = signing_key.verifying_key().to_bytes();
        let pubkey = bs58::encode(&pub_bytes).into_string();
        let hex_seed = hex::encode(seed);
        let user_id = format!("user-{name}");
        Self {
            name,
            seed,
            pubkey,
            hex_seed,
            user_id,
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

pub mod perm {
    pub const SEND_MESSAGES: i64 = 1 << 0;
    pub const READ_MESSAGES: i64 = 1 << 1;
    #[allow(dead_code)]
    pub const MANAGE_MESSAGES: i64 = 1 << 2;
    #[allow(dead_code)]
    pub const MANAGE_CHANNELS: i64 = 1 << 3;
    #[allow(dead_code)]
    pub const MANAGE_ROLES: i64 = 1 << 4;
    #[allow(dead_code)]
    pub const KICK_MEMBERS: i64 = 1 << 5;
    #[allow(dead_code)]
    pub const BAN_MEMBERS: i64 = 1 << 6;
    #[allow(dead_code)]
    pub const MANAGE_INVITES: i64 = 1 << 7;
    #[allow(dead_code)]
    pub const MANAGE_SERVER: i64 = 1 << 8;
    #[allow(dead_code)]
    pub const ATTACH_FILES: i64 = 1 << 9;
    #[allow(dead_code)]
    pub const ADD_REACTIONS: i64 = 1 << 10;
    #[allow(dead_code)]
    pub const MENTION_EVERYONE: i64 = 1 << 11;
    #[allow(dead_code)]
    pub const VIEW_AUDIT_LOG: i64 = 1 << 12;
    #[allow(dead_code)]
    pub const ADMINISTRATOR: i64 = 1 << 13;
}

pub struct UserConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub roles: &'static [&'static str],
}

pub struct CustomRole {
    pub id: &'static str,
    pub name: &'static str,
    pub color: Option<&'static str>,
    pub permissions: i64,
    pub position: i64,
}

pub struct Channel {
    pub id: &'static str,
    pub name: &'static str,
    pub topic: Option<&'static str>,
    pub category: Option<&'static str>,
}

pub struct Message {
    pub channel_id: &'static str,
    pub author: &'static str,
    pub content: &'static str,
    pub offset_min: i64,
}

pub struct ThreadReply {
    pub author: &'static str,
    pub content: &'static str,
    pub offset_min: i64,
}

pub struct ThreadSeed {
    pub parent_message_id: &'static str,
    pub replies: &'static [ThreadReply],
}

pub struct EmojiReaction {
    pub emoji: &'static str,
    pub users: &'static [&'static str],
}

pub struct MessageReactions {
    pub message_id: &'static str,
    pub reactions: &'static [EmojiReaction],
}

pub struct InviteSeed {
    pub code: &'static str,
    pub created_by: &'static str,
    pub max_uses: i64,
}

pub struct AllowlistEntry {
    pub pubkey_user: &'static str,
    pub added_by: &'static str,
}

pub struct BotSeed {
    pub name: &'static str,
}

pub struct ServerConfig {
    pub db_name: &'static str,
    pub display_name: &'static str,
    pub users: &'static [UserConfig],
    pub custom_roles: &'static [CustomRole],
    pub channels: &'static [Channel],
    pub messages: &'static [Message],
    pub threads: &'static [ThreadSeed],
    pub reactions: &'static [MessageReactions],
    pub invites: &'static [InviteSeed],
    pub allowlist: &'static [AllowlistEntry],
    pub bots: &'static [BotSeed],
}

static ALL_SERVERS: [&ServerConfig; 3] = [&servers::OPEN, &servers::PRIVATE, &servers::STRICT];

pub fn all_servers() -> &'static [&'static ServerConfig] {
    &ALL_SERVERS
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
        println!("  {:<12}  id: {}  pubkey: {}…", bot.name, bot.user_id, &bot.pubkey[..20]);
    }
    println!("  bot-alpha — approved on Open Server (localhost:8081)");
    println!("  bot-beta  — not registered with any server");
    println!();
    println!("Server Layout:");
    println!("{}", "-".repeat(60));
    println!("  Open Server     (localhost:8081) — open membership");
    println!("    Users: alice (admin), bob, charlie (restricted), dave");
    println!("    Text Channels: #general, #random, #introductions");
    println!("    Admin:         #announcements, #mod-log");
    println!("    Projects:      #frontend, #backend, #design");
    println!("    Messages: seeded across all channels");
    println!("    Sample Threads (test the threads feature):");
    println!("      #general → Alice's welcome (3 replies)");
    println!("      #frontend → Dave's drag-and-drop question (2 replies)");
    println!("      #backend → Bob's SFU question (3 replies)");
    println!("    Sample Reactions (test emoji reactions):");
    println!("      5+ messages with emoji reactions (multiple emojis per message)");
    println!();
    println!("  Private Server  (localhost:8082) — invite only");
    println!("    Users: alice (admin), bob");
    println!("    Invite code: TESTinv1");
    println!("    charlie & dave can test joining via invite");
    println!("    General:  #lobby, #members-only");
    println!("    Planning: #roadmap, #feedback");
    println!("    Messages: seeded across all channels");
    println!("    Sample Threads:");
    println!("      #lobby → Alice's welcome (2 replies)");
    println!("      #feedback → Alice's feedback question (1 reply)");
    println!("    Sample Reactions:");
    println!("      3 messages with emoji reactions");
    println!();
    println!("  Strict Server   (localhost:8083) — allowlist only");
    println!("    Users: alice (admin), bob");
    println!("    Allowlisted: alice, bob");
    println!("    charlie & dave are NOT allowlisted");
    println!("    Main:       #verified, #security");
    println!("    Operations: #incidents");
    println!("    Messages: seeded across all channels");
    println!();
    println!("Permissions Notes:");
    println!("{}", "-".repeat(60));
    println!("  alice   — Admin on all servers (full permissions)");
    println!("  bob     — Regular member (default @everyone perms)");
    println!("  charlie — 'Restricted' role on Open: send+read only");
    println!("            (no file attach, no reactions, no manage)");
    println!("            NOT on Private or Strict servers");
    println!("  dave    — Regular on Open only; not on Private/Strict");
    println!();
    println!("To start servers: make dev");
    println!("  or: overmind start -f Procfile (just servers)");
    println!();
    println!("Client localStorage pre-configured:");
    println!("  alice & bob — Open + Private + Strict servers added");
    println!("  charlie & dave — Open server added");
    println!("  Active account: alice (switch via account switcher in client)");
    println!("{}", "=".repeat(60));
}
