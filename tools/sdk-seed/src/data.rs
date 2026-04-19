//! Static seed data for the test environment. The SDK seeder reads this and
//! drives the REST API to recreate the same state that the legacy raw-SQL
//! seeder produced.

use crate::users::perm;

pub struct UserConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub roles: &'static [&'static str],
}

pub struct CustomRole {
    pub name: &'static str,
    pub color: Option<&'static str>,
    pub permissions: i64,
    pub position: i32,
}

pub struct Channel {
    pub name: &'static str,
    pub topic: Option<&'static str>,
    pub category: Option<&'static str>,
}

pub struct Message {
    pub channel_name: &'static str,
    pub author: &'static str,
    pub content: &'static str,
}

pub struct ThreadReply {
    pub author: &'static str,
    pub content: &'static str,
}

/// References a parent message by `(channel_name, message_index)` where the
/// index is the position of the message inside that channel's seed list.
pub struct ThreadSeed {
    pub channel_name: &'static str,
    pub message_index: usize,
    pub replies: &'static [ThreadReply],
}

pub struct EmojiReaction {
    pub emoji: &'static str,
    pub users: &'static [&'static str],
}

pub struct MessageReactions {
    pub channel_name: &'static str,
    pub message_index: usize,
    pub reactions: &'static [EmojiReaction],
}

pub struct InviteSeed {
    pub created_by: &'static str,
    pub max_uses: i64,
    pub joiners: &'static [&'static str],
}

pub struct AllowlistEntry {
    pub pubkey_user: &'static str,
}

pub struct BotSeed {
    pub name: &'static str,
    /// When `true`, an admin auto-approves the bot after it registers.
    pub approve: bool,
}

pub struct ServerConfig {
    pub display_name: &'static str,
    pub base_url: &'static str,
    /// The user listed first owns the server (first to authenticate, becoming
    /// the auto-admin). All other entries are joined in order.
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

static ALL_SERVERS: [&ServerConfig; 3] = [&OPEN, &PRIVATE, &STRICT];

pub fn all_servers() -> &'static [&'static ServerConfig] {
    &ALL_SERVERS
}

pub static OPEN: ServerConfig = ServerConfig {
    display_name: "Open Server",
    base_url: "http://localhost:8081",
    users: &[
        UserConfig {
            name: "alice",
            display_name: "Alice (Admin)",
            roles: &[],
        },
        UserConfig {
            name: "bob",
            display_name: "Bob",
            roles: &[],
        },
        UserConfig {
            name: "charlie",
            display_name: "Charlie (Restricted)",
            roles: &["restricted"],
        },
        UserConfig {
            name: "dave",
            display_name: "Dave",
            roles: &[],
        },
    ],
    custom_roles: &[CustomRole {
        name: "restricted",
        color: Some("#ef4444"),
        permissions: perm::READ_MESSAGES | perm::SEND_MESSAGES,
        position: 500,
    }],
    channels: &[
        Channel {
            name: "general",
            topic: Some("General discussion"),
            category: Some("Text Channels"),
        },
        Channel {
            name: "random",
            topic: Some("Off-topic chat"),
            category: Some("Text Channels"),
        },
        Channel {
            name: "introductions",
            topic: Some("Say hello!"),
            category: Some("Text Channels"),
        },
        Channel {
            name: "announcements",
            topic: Some("Important updates from admins"),
            category: Some("Admin"),
        },
        Channel {
            name: "mod-log",
            topic: Some("Moderation activity log"),
            category: Some("Admin"),
        },
        Channel {
            name: "frontend",
            topic: Some("UI/UX discussion"),
            category: Some("Projects"),
        },
        Channel {
            name: "backend",
            topic: Some("Server-side work"),
            category: Some("Projects"),
        },
        Channel {
            name: "design",
            topic: Some("Design assets and feedback"),
            category: Some("Projects"),
        },
    ],
    messages: &[
        // #general (0..=9)
        Message { channel_name: "general", author: "alice", content: "Welcome everyone to the Open Server! Feel free to chat here." },
        Message { channel_name: "general", author: "bob", content: "Thanks Alice! Glad to be here." },
        Message { channel_name: "general", author: "charlie", content: "Hey all 👋" },
        Message { channel_name: "general", author: "dave", content: "What's everyone working on today?" },
        Message { channel_name: "general", author: "alice", content: "Setting up the test environment and making sure everything works." },
        Message { channel_name: "general", author: "bob", content: "I've been looking at the permissions system. Pretty slick." },
        Message { channel_name: "general", author: "dave", content: "Same here. The role-based access is nice." },
        Message { channel_name: "general", author: "charlie", content: "I seem to have limited access compared to you all though." },
        Message { channel_name: "general", author: "alice", content: "That's by design, Charlie — you're on the restricted role for testing purposes." },
        Message { channel_name: "general", author: "charlie", content: "Ah that makes sense. Good to know!" },
        // #random (0..=5)
        Message { channel_name: "random", author: "bob", content: "Anyone else think decentralized chat is the future?" },
        Message { channel_name: "random", author: "dave", content: "Absolutely. No single point of failure." },
        Message { channel_name: "random", author: "alice", content: "That's the whole idea behind decentcom 🎉" },
        Message { channel_name: "random", author: "bob", content: "Plus Ed25519 keys instead of passwords. No more credential stuffing." },
        Message { channel_name: "random", author: "charlie", content: "Wait, so my identity IS my key pair?" },
        Message { channel_name: "random", author: "alice", content: "Exactly. Your public key is your identity. No email, no username/password." },
        // #introductions (0..=3)
        Message { channel_name: "introductions", author: "alice", content: "I'm Alice, admin of this server. I manage the infrastructure and keep things running." },
        Message { channel_name: "introductions", author: "bob", content: "Bob here. I'm interested in backend development and distributed systems." },
        Message { channel_name: "introductions", author: "charlie", content: "Charlie. Just exploring the platform, happy to help test things!" },
        Message { channel_name: "introductions", author: "dave", content: "Dave. Frontend dev. Excited about the Tauri + React stack." },
        // #announcements (0..=2)
        Message { channel_name: "announcements", author: "alice", content: "📢 Server rules: Be respectful, no spam, keep discussions on-topic." },
        Message { channel_name: "announcements", author: "alice", content: "📢 New channels added under Projects category for dev discussions." },
        Message { channel_name: "announcements", author: "alice", content: "📢 Reminder: Charlie is on the restricted role for testing — this is intentional." },
        // #frontend (0..=4)
        Message { channel_name: "frontend", author: "dave", content: "The account switcher is looking great. Nice work on the labels." },
        Message { channel_name: "frontend", author: "bob", content: "Agreed. The double-click-to-rename UX is intuitive." },
        Message { channel_name: "frontend", author: "alice", content: "Thanks! Next up is polishing the channel management UI." },
        Message { channel_name: "frontend", author: "dave", content: "Should we add drag-and-drop for reordering channels?" },
        Message { channel_name: "frontend", author: "alice", content: "That's on the roadmap. Let's get the basics solid first." },
        // #backend (0..=3)
        Message { channel_name: "backend", author: "bob", content: "The SQLx migrations are clean. Love the trait-based storage abstraction." },
        Message { channel_name: "backend", author: "alice", content: "That'll make it easy to swap in PostgreSQL later for the scale-out path." },
        Message { channel_name: "backend", author: "bob", content: "Have we decided on the SFU strategy for WebRTC yet?" },
        Message { channel_name: "backend", author: "alice", content: "Not yet — that's an open question in the architecture doc. Milestone 3 territory." },
        // #design (0..=3)
        Message { channel_name: "design", author: "dave", content: "I've been working with the Catppuccin Mocha palette. It looks fantastic in dark mode." },
        Message { channel_name: "design", author: "alice", content: "Mocha is the default but we support all Catppuccin flavors." },
        Message { channel_name: "design", author: "bob", content: "Can we add a light theme too? Not everyone likes dark mode." },
        Message { channel_name: "design", author: "dave", content: "Catppuccin Latte is the light variant. We could default to it based on OS preference." },
    ],
    threads: &[
        ThreadSeed {
            channel_name: "general",
            message_index: 0,
            replies: &[
                ThreadReply { author: "bob", content: "Thanks Alice! Excited to get started." },
                ThreadReply { author: "charlie", content: "This platform is really slick! 🚀" },
                ThreadReply { author: "dave", content: "Looking forward to collaborating with everyone here." },
            ],
        },
        ThreadSeed {
            channel_name: "frontend",
            message_index: 3,
            replies: &[
                ThreadReply { author: "alice", content: "Definitely! Let's add that after we ship the basics." },
                ThreadReply { author: "bob", content: "I can help with the implementation if needed." },
            ],
        },
        ThreadSeed {
            channel_name: "backend",
            message_index: 2,
            replies: &[
                ThreadReply { author: "alice", content: "Great question. We're evaluating mediasoup vs livekit right now." },
                ThreadReply { author: "bob", content: "Both solid options. mediasoup would keep us more self-contained." },
                ThreadReply { author: "dave", content: "What about resource requirements? That might be the deciding factor." },
            ],
        },
    ],
    reactions: &[
        MessageReactions {
            channel_name: "general",
            message_index: 0,
            reactions: &[
                EmojiReaction { emoji: "👋", users: &["bob", "charlie", "dave"] },
                EmojiReaction { emoji: "🎉", users: &["alice", "bob"] },
            ],
        },
        MessageReactions {
            channel_name: "general",
            message_index: 4,
            reactions: &[
                EmojiReaction { emoji: "👍", users: &["bob", "dave"] },
                EmojiReaction { emoji: "💯", users: &["charlie"] },
            ],
        },
        MessageReactions {
            channel_name: "random",
            message_index: 2,
            reactions: &[
                EmojiReaction { emoji: "🚀", users: &["alice", "dave", "charlie"] },
                EmojiReaction { emoji: "🙌", users: &["bob"] },
            ],
        },
        MessageReactions {
            channel_name: "random",
            message_index: 0,
            reactions: &[EmojiReaction { emoji: "💯", users: &["dave", "alice"] }],
        },
        MessageReactions {
            channel_name: "frontend",
            message_index: 0,
            reactions: &[
                EmojiReaction { emoji: "👏", users: &["alice"] },
                EmojiReaction { emoji: "❤️", users: &["bob", "dave"] },
            ],
        },
    ],
    invites: &[],
    allowlist: &[],
    bots: &[BotSeed { name: "bot-alpha", approve: true }],
};

pub static PRIVATE: ServerConfig = ServerConfig {
    display_name: "Private Server",
    base_url: "http://localhost:8082",
    users: &[
        UserConfig { name: "alice", display_name: "Alice (Admin)", roles: &[] },
        UserConfig { name: "bob", display_name: "Bob", roles: &[] },
    ],
    custom_roles: &[],
    channels: &[
        Channel { name: "lobby", topic: Some("Welcome! Start here."), category: Some("General") },
        Channel { name: "members-only", topic: Some("For members"), category: Some("General") },
        Channel { name: "roadmap", topic: Some("Feature planning and priorities"), category: Some("Planning") },
        Channel { name: "feedback", topic: Some("Share your thoughts"), category: Some("Planning") },
    ],
    messages: &[
        // #lobby (0..=3)
        Message { channel_name: "lobby", author: "alice", content: "Welcome to the Private Server! This is invite-only." },
        Message { channel_name: "lobby", author: "bob", content: "Thanks for the invite, Alice!" },
        Message { channel_name: "lobby", author: "alice", content: "Feel free to share invite codes with people you trust." },
        Message { channel_name: "lobby", author: "bob", content: "Will do. The invite system seems straightforward." },
        // #members-only (0..=2)
        Message { channel_name: "members-only", author: "alice", content: "This channel is for discussions among verified members." },
        Message { channel_name: "members-only", author: "bob", content: "Good to have a space separate from the lobby." },
        Message { channel_name: "members-only", author: "alice", content: "Exactly. New members start in lobby until they get oriented." },
        // #roadmap (0..=2)
        Message { channel_name: "roadmap", author: "alice", content: "Current priorities: 1) Multi-account support ✅  2) File uploads ✅  3) Channel management" },
        Message { channel_name: "roadmap", author: "bob", content: "What about voice channels? That's the killer feature for me." },
        Message { channel_name: "roadmap", author: "alice", content: "Voice is Milestone 3. We need to resolve the SFU strategy first." },
        // #feedback (0..=3)
        Message { channel_name: "feedback", author: "bob", content: "The UI feels snappy. The Tauri approach is paying off." },
        Message { channel_name: "feedback", author: "alice", content: "Glad to hear it. Any rough edges you've noticed?" },
        Message { channel_name: "feedback", author: "bob", content: "Maybe some loading states when switching servers? It can feel abrupt." },
        Message { channel_name: "feedback", author: "alice", content: "Good point. I'll add that to the backlog." },
    ],
    threads: &[
        ThreadSeed {
            channel_name: "lobby",
            message_index: 0,
            replies: &[
                ThreadReply { author: "bob", content: "Thanks for creating this! It feels great to have a members-only space." },
                ThreadReply { author: "alice", content: "Happy to have you here. Let's keep it a quality discussion space." },
            ],
        },
        ThreadSeed {
            channel_name: "feedback",
            message_index: 1,
            replies: &[ThreadReply { author: "bob", content: "Actually, now that you mention it, there's one more thing..." }],
        },
    ],
    reactions: &[
        MessageReactions {
            channel_name: "lobby",
            message_index: 0,
            reactions: &[
                EmojiReaction { emoji: "👋", users: &["bob"] },
                EmojiReaction { emoji: "🤝", users: &["alice", "bob"] },
            ],
        },
        MessageReactions {
            channel_name: "roadmap",
            message_index: 0,
            reactions: &[EmojiReaction { emoji: "✅", users: &["bob"] }],
        },
        MessageReactions {
            channel_name: "feedback",
            message_index: 0,
            reactions: &[EmojiReaction { emoji: "🎯", users: &["alice"] }],
        },
    ],
    invites: &[InviteSeed { created_by: "alice", max_uses: 0, joiners: &["bob"] }],
    allowlist: &[],
    bots: &[],
};

pub static STRICT: ServerConfig = ServerConfig {
    display_name: "Strict Server",
    base_url: "http://localhost:8083",
    users: &[
        UserConfig { name: "alice", display_name: "Alice (Admin)", roles: &[] },
        UserConfig { name: "bob", display_name: "Bob", roles: &[] },
    ],
    custom_roles: &[],
    channels: &[
        Channel { name: "verified", topic: Some("Verified users only"), category: Some("Main") },
        Channel { name: "security", topic: Some("Security discussions and advisories"), category: Some("Main") },
        Channel { name: "incidents", topic: Some("Incident reports and post-mortems"), category: Some("Operations") },
    ],
    messages: &[
        // #verified (0..=4)
        Message { channel_name: "verified", author: "alice", content: "This server uses allowlist-only mode. Only pre-approved pubkeys can join." },
        Message { channel_name: "verified", author: "bob", content: "Makes sense for high-trust environments." },
        Message { channel_name: "verified", author: "alice", content: "Exactly. Think corporate teams, security groups, etc." },
        Message { channel_name: "verified", author: "bob", content: "How do new users get added to the allowlist?" },
        Message { channel_name: "verified", author: "alice", content: "An admin adds their pubkey manually. No invite codes here." },
        // #security (0..=3)
        Message { channel_name: "security", author: "alice", content: "Reminder: never share your private key seed. Your identity depends on it." },
        Message { channel_name: "security", author: "bob", content: "What's the recovery story if someone loses their key?" },
        Message { channel_name: "security", author: "alice", content: "That's still an open design question. See docs/design/identity.md for options." },
        Message { channel_name: "security", author: "bob", content: "Device sub-keys look promising. You could revoke a lost device without losing your main identity." },
        // #incidents (0..=1)
        Message { channel_name: "incidents", author: "alice", content: "No incidents to report. Let's keep it that way! 🎉" },
        Message { channel_name: "incidents", author: "bob", content: "🪵 All systems nominal." },
    ],
    threads: &[],
    reactions: &[],
    invites: &[],
    allowlist: &[AllowlistEntry { pubkey_user: "bob" }],
    bots: &[],
};
