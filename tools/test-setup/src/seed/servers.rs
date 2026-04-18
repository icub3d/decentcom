use super::{
    AllowlistEntry, BotSeed, Channel, CustomRole, EmojiReaction, InviteSeed, Message,
    MessageReactions, ServerConfig, ThreadReply, ThreadSeed, UserConfig,
};
use super::perm;

pub static OPEN: ServerConfig = ServerConfig {
    db_name: "open.db",
    display_name: "Open Server",
    users: &[
        UserConfig {
            name: "alice",
            display_name: "Alice (Admin)",
            roles: &["everyone", "admin"],
        },
        UserConfig {
            name: "bob",
            display_name: "Bob",
            roles: &["everyone"],
        },
        UserConfig {
            name: "charlie",
            display_name: "Charlie (Restricted)",
            roles: &["everyone", "restricted"],
        },
        UserConfig {
            name: "dave",
            display_name: "Dave",
            roles: &["everyone"],
        },
    ],
    custom_roles: &[CustomRole {
        id: "restricted",
        name: "Restricted",
        color: Some("#ef4444"),
        permissions: perm::READ_MESSAGES | perm::SEND_MESSAGES,
        position: 500,
    }],
    channels: &[
        // Text Channels
        Channel {
            id: "ch-general",
            name: "general",
            topic: Some("General discussion"),
            category: Some("Text Channels"),
        },
        Channel {
            id: "ch-random",
            name: "random",
            topic: Some("Off-topic chat"),
            category: Some("Text Channels"),
        },
        Channel {
            id: "ch-introductions",
            name: "introductions",
            topic: Some("Say hello!"),
            category: Some("Text Channels"),
        },
        // Admin
        Channel {
            id: "ch-announcements",
            name: "announcements",
            topic: Some("Important updates from admins"),
            category: Some("Admin"),
        },
        Channel {
            id: "ch-mod-log",
            name: "mod-log",
            topic: Some("Moderation activity log"),
            category: Some("Admin"),
        },
        // Projects
        Channel {
            id: "ch-frontend",
            name: "frontend",
            topic: Some("UI/UX discussion"),
            category: Some("Projects"),
        },
        Channel {
            id: "ch-backend",
            name: "backend",
            topic: Some("Server-side work"),
            category: Some("Projects"),
        },
        Channel {
            id: "ch-design",
            name: "design",
            topic: Some("Design assets and feedback"),
            category: Some("Projects"),
        },
    ],
    messages: &[
        // #general — a realistic conversation (000-009)
        Message {
            channel_id: "ch-general",
            author: "alice",
            content: "Welcome everyone to the Open Server! Feel free to chat here.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-general",
            author: "bob",
            content: "Thanks Alice! Glad to be here.",
            offset_min: 2,
        },
        Message {
            channel_id: "ch-general",
            author: "charlie",
            content: "Hey all 👋",
            offset_min: 3,
        },
        Message {
            channel_id: "ch-general",
            author: "dave",
            content: "What's everyone working on today?",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-general",
            author: "alice",
            content: "Setting up the test environment and making sure everything works.",
            offset_min: 7,
        },
        Message {
            channel_id: "ch-general",
            author: "bob",
            content: "I've been looking at the permissions system. Pretty slick.",
            offset_min: 10,
        },
        Message {
            channel_id: "ch-general",
            author: "dave",
            content: "Same here. The role-based access is nice.",
            offset_min: 12,
        },
        Message {
            channel_id: "ch-general",
            author: "charlie",
            content: "I seem to have limited access compared to you all though.",
            offset_min: 14,
        },
        Message {
            channel_id: "ch-general",
            author: "alice",
            content: "That's by design, Charlie — you're on the restricted role for testing purposes.",
            offset_min: 16,
        },
        Message {
            channel_id: "ch-general",
            author: "charlie",
            content: "Ah that makes sense. Good to know!",
            offset_min: 17,
        },
        // #random — lighter chat (010-015)
        Message {
            channel_id: "ch-random",
            author: "bob",
            content: "Anyone else think decentralized chat is the future?",
            offset_min: 20,
        },
        Message {
            channel_id: "ch-random",
            author: "dave",
            content: "Absolutely. No single point of failure.",
            offset_min: 22,
        },
        Message {
            channel_id: "ch-random",
            author: "alice",
            content: "That's the whole idea behind decentcom 🎉",
            offset_min: 24,
        },
        Message {
            channel_id: "ch-random",
            author: "bob",
            content: "Plus Ed25519 keys instead of passwords. No more credential stuffing.",
            offset_min: 26,
        },
        Message {
            channel_id: "ch-random",
            author: "charlie",
            content: "Wait, so my identity IS my key pair?",
            offset_min: 28,
        },
        Message {
            channel_id: "ch-random",
            author: "alice",
            content: "Exactly. Your public key is your identity. No email, no username/password.",
            offset_min: 30,
        },
        // #introductions (016-019)
        Message {
            channel_id: "ch-introductions",
            author: "alice",
            content: "I'm Alice, admin of this server. I manage the infrastructure and keep things running.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-introductions",
            author: "bob",
            content: "Bob here. I'm interested in backend development and distributed systems.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-introductions",
            author: "charlie",
            content: "Charlie. Just exploring the platform, happy to help test things!",
            offset_min: 10,
        },
        Message {
            channel_id: "ch-introductions",
            author: "dave",
            content: "Dave. Frontend dev. Excited about the Tauri + React stack.",
            offset_min: 15,
        },
        // #announcements — admin posts (020-022)
        Message {
            channel_id: "ch-announcements",
            author: "alice",
            content: "📢 Server rules: Be respectful, no spam, keep discussions on-topic.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-announcements",
            author: "alice",
            content: "📢 New channels added under Projects category for dev discussions.",
            offset_min: 60,
        },
        Message {
            channel_id: "ch-announcements",
            author: "alice",
            content: "📢 Reminder: Charlie is on the restricted role for testing — this is intentional.",
            offset_min: 120,
        },
        // #frontend — project chat (023-027)
        Message {
            channel_id: "ch-frontend",
            author: "dave",
            content: "The account switcher is looking great. Nice work on the labels.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-frontend",
            author: "bob",
            content: "Agreed. The double-click-to-rename UX is intuitive.",
            offset_min: 3,
        },
        Message {
            channel_id: "ch-frontend",
            author: "alice",
            content: "Thanks! Next up is polishing the channel management UI.",
            offset_min: 6,
        },
        Message {
            channel_id: "ch-frontend",
            author: "dave",
            content: "Should we add drag-and-drop for reordering channels?",
            offset_min: 10,
        },
        Message {
            channel_id: "ch-frontend",
            author: "alice",
            content: "That's on the roadmap. Let's get the basics solid first.",
            offset_min: 12,
        },
        // #backend (028-031)
        Message {
            channel_id: "ch-backend",
            author: "bob",
            content: "The SQLx migrations are clean. Love the trait-based storage abstraction.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-backend",
            author: "alice",
            content: "That'll make it easy to swap in PostgreSQL later for the scale-out path.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-backend",
            author: "bob",
            content: "Have we decided on the SFU strategy for WebRTC yet?",
            offset_min: 10,
        },
        Message {
            channel_id: "ch-backend",
            author: "alice",
            content: "Not yet — that's an open question in the architecture doc. Milestone 3 territory.",
            offset_min: 12,
        },
        // #design (032-035)
        Message {
            channel_id: "ch-design",
            author: "dave",
            content: "I've been working with the Catppuccin Mocha palette. It looks fantastic in dark mode.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-design",
            author: "alice",
            content: "Mocha is the default but we support all Catppuccin flavors.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-design",
            author: "bob",
            content: "Can we add a light theme too? Not everyone likes dark mode.",
            offset_min: 8,
        },
        Message {
            channel_id: "ch-design",
            author: "dave",
            content: "Catppuccin Latte is the light variant. We could default to it based on OS preference.",
            offset_min: 11,
        },
    ],
    threads: &[
        ThreadSeed {
            parent_message_id: "msg-ch-general-000",
            replies: &[
                ThreadReply {
                    author: "bob",
                    content: "Thanks Alice! Excited to get started.",
                    offset_min: 2,
                },
                ThreadReply {
                    author: "charlie",
                    content: "This platform is really slick! 🚀",
                    offset_min: 4,
                },
                ThreadReply {
                    author: "dave",
                    content: "Looking forward to collaborating with everyone here.",
                    offset_min: 6,
                },
            ],
        },
        ThreadSeed {
            parent_message_id: "msg-ch-frontend-026",
            replies: &[
                ThreadReply {
                    author: "alice",
                    content: "Definitely! Let's add that after we ship the basics.",
                    offset_min: 3,
                },
                ThreadReply {
                    author: "bob",
                    content: "I can help with the implementation if needed.",
                    offset_min: 5,
                },
            ],
        },
        ThreadSeed {
            parent_message_id: "msg-ch-backend-030",
            replies: &[
                ThreadReply {
                    author: "alice",
                    content: "Great question. We're evaluating mediasoup vs livekit right now.",
                    offset_min: 2,
                },
                ThreadReply {
                    author: "bob",
                    content: "Both solid options. mediasoup would keep us more self-contained.",
                    offset_min: 4,
                },
                ThreadReply {
                    author: "dave",
                    content: "What about resource requirements? That might be the deciding factor.",
                    offset_min: 8,
                },
            ],
        },
    ],
    reactions: &[
        MessageReactions {
            message_id: "msg-ch-general-000",
            reactions: &[
                EmojiReaction {
                    emoji: "👋",
                    users: &["bob", "charlie", "dave"],
                },
                EmojiReaction {
                    emoji: "🎉",
                    users: &["alice", "bob"],
                },
            ],
        },
        MessageReactions {
            message_id: "msg-ch-general-004",
            reactions: &[
                EmojiReaction {
                    emoji: "👍",
                    users: &["bob", "dave"],
                },
                EmojiReaction {
                    emoji: "💯",
                    users: &["charlie"],
                },
            ],
        },
        MessageReactions {
            message_id: "msg-ch-random-012",
            reactions: &[
                EmojiReaction {
                    emoji: "🚀",
                    users: &["alice", "dave", "charlie"],
                },
                EmojiReaction {
                    emoji: "🙌",
                    users: &["bob"],
                },
            ],
        },
        MessageReactions {
            message_id: "msg-ch-random-010",
            reactions: &[EmojiReaction {
                emoji: "💯",
                users: &["dave", "alice"],
            }],
        },
        MessageReactions {
            message_id: "msg-ch-frontend-023",
            reactions: &[
                EmojiReaction {
                    emoji: "👏",
                    users: &["alice"],
                },
                EmojiReaction {
                    emoji: "❤️",
                    users: &["bob", "dave"],
                },
            ],
        },
    ],
    invites: &[],
    allowlist: &[],
    bots: &[BotSeed { name: "bot-alpha" }],
};

pub static PRIVATE: ServerConfig = ServerConfig {
    db_name: "private.db",
    display_name: "Private Server",
    users: &[
        UserConfig {
            name: "alice",
            display_name: "Alice (Admin)",
            roles: &["everyone", "admin"],
        },
        UserConfig {
            name: "bob",
            display_name: "Bob",
            roles: &["everyone"],
        },
    ],
    custom_roles: &[],
    channels: &[
        // General
        Channel {
            id: "ch-lobby",
            name: "lobby",
            topic: Some("Welcome! Start here."),
            category: Some("General"),
        },
        Channel {
            id: "ch-members-only",
            name: "members-only",
            topic: Some("For members"),
            category: Some("General"),
        },
        // Planning
        Channel {
            id: "ch-roadmap",
            name: "roadmap",
            topic: Some("Feature planning and priorities"),
            category: Some("Planning"),
        },
        Channel {
            id: "ch-feedback",
            name: "feedback",
            topic: Some("Share your thoughts"),
            category: Some("Planning"),
        },
    ],
    messages: &[
        // #lobby (000-003)
        Message {
            channel_id: "ch-lobby",
            author: "alice",
            content: "Welcome to the Private Server! This is invite-only.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-lobby",
            author: "bob",
            content: "Thanks for the invite, Alice!",
            offset_min: 3,
        },
        Message {
            channel_id: "ch-lobby",
            author: "alice",
            content: "Feel free to share invite codes with people you trust.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-lobby",
            author: "bob",
            content: "Will do. The invite system seems straightforward.",
            offset_min: 8,
        },
        // #members-only (004-006)
        Message {
            channel_id: "ch-members-only",
            author: "alice",
            content: "This channel is for discussions among verified members.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-members-only",
            author: "bob",
            content: "Good to have a space separate from the lobby.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-members-only",
            author: "alice",
            content: "Exactly. New members start in lobby until they get oriented.",
            offset_min: 8,
        },
        // #roadmap (007-009)
        Message {
            channel_id: "ch-roadmap",
            author: "alice",
            content: "Current priorities: 1) Multi-account support ✅  2) File uploads ✅  3) Channel management",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-roadmap",
            author: "bob",
            content: "What about voice channels? That's the killer feature for me.",
            offset_min: 10,
        },
        Message {
            channel_id: "ch-roadmap",
            author: "alice",
            content: "Voice is Milestone 3. We need to resolve the SFU strategy first.",
            offset_min: 12,
        },
        // #feedback (010-013)
        Message {
            channel_id: "ch-feedback",
            author: "bob",
            content: "The UI feels snappy. The Tauri approach is paying off.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-feedback",
            author: "alice",
            content: "Glad to hear it. Any rough edges you've noticed?",
            offset_min: 3,
        },
        Message {
            channel_id: "ch-feedback",
            author: "bob",
            content: "Maybe some loading states when switching servers? It can feel abrupt.",
            offset_min: 6,
        },
        Message {
            channel_id: "ch-feedback",
            author: "alice",
            content: "Good point. I'll add that to the backlog.",
            offset_min: 8,
        },
    ],
    threads: &[
        ThreadSeed {
            parent_message_id: "msg-ch-lobby-000",
            replies: &[
                ThreadReply {
                    author: "bob",
                    content: "Thanks for creating this! It feels great to have a members-only space.",
                    offset_min: 3,
                },
                ThreadReply {
                    author: "alice",
                    content: "Happy to have you here. Let's keep it a quality discussion space.",
                    offset_min: 5,
                },
            ],
        },
        ThreadSeed {
            parent_message_id: "msg-ch-feedback-011",
            replies: &[ThreadReply {
                author: "bob",
                content: "Actually, now that you mention it, there's one more thing...",
                offset_min: 2,
            }],
        },
    ],
    reactions: &[
        MessageReactions {
            message_id: "msg-ch-lobby-000",
            reactions: &[
                EmojiReaction {
                    emoji: "👋",
                    users: &["bob"],
                },
                EmojiReaction {
                    emoji: "🤝",
                    users: &["alice", "bob"],
                },
            ],
        },
        MessageReactions {
            message_id: "msg-ch-roadmap-007",
            reactions: &[EmojiReaction {
                emoji: "✅",
                users: &["bob"],
            }],
        },
        MessageReactions {
            message_id: "msg-ch-feedback-010",
            reactions: &[EmojiReaction {
                emoji: "🎯",
                users: &["alice"],
            }],
        },
    ],
    invites: &[InviteSeed {
        code: "TESTinv1",
        created_by: "alice",
        max_uses: 0,
    }],
    allowlist: &[],
    bots: &[],
};

pub static STRICT: ServerConfig = ServerConfig {
    db_name: "strict.db",
    display_name: "Strict Server",
    users: &[
        UserConfig {
            name: "alice",
            display_name: "Alice (Admin)",
            roles: &["everyone", "admin"],
        },
        UserConfig {
            name: "bob",
            display_name: "Bob",
            roles: &["everyone"],
        },
    ],
    custom_roles: &[],
    channels: &[
        // Main
        Channel {
            id: "ch-verified",
            name: "verified",
            topic: Some("Verified users only"),
            category: Some("Main"),
        },
        Channel {
            id: "ch-security",
            name: "security",
            topic: Some("Security discussions and advisories"),
            category: Some("Main"),
        },
        // Operations
        Channel {
            id: "ch-incidents",
            name: "incidents",
            topic: Some("Incident reports and post-mortems"),
            category: Some("Operations"),
        },
    ],
    messages: &[
        // #verified (000-004)
        Message {
            channel_id: "ch-verified",
            author: "alice",
            content: "This server uses allowlist-only mode. Only pre-approved pubkeys can join.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-verified",
            author: "bob",
            content: "Makes sense for high-trust environments.",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-verified",
            author: "alice",
            content: "Exactly. Think corporate teams, security groups, etc.",
            offset_min: 8,
        },
        Message {
            channel_id: "ch-verified",
            author: "bob",
            content: "How do new users get added to the allowlist?",
            offset_min: 12,
        },
        Message {
            channel_id: "ch-verified",
            author: "alice",
            content: "An admin adds their pubkey manually. No invite codes here.",
            offset_min: 14,
        },
        // #security (005-008)
        Message {
            channel_id: "ch-security",
            author: "alice",
            content: "Reminder: never share your private key seed. Your identity depends on it.",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-security",
            author: "bob",
            content: "What's the recovery story if someone loses their key?",
            offset_min: 5,
        },
        Message {
            channel_id: "ch-security",
            author: "alice",
            content: "That's still an open design question. See docs/design/identity.md for options.",
            offset_min: 8,
        },
        Message {
            channel_id: "ch-security",
            author: "bob",
            content: "Device sub-keys look promising. You could revoke a lost device without losing your main identity.",
            offset_min: 12,
        },
        // #incidents (009-010)
        Message {
            channel_id: "ch-incidents",
            author: "alice",
            content: "No incidents to report. Let's keep it that way! 🎉",
            offset_min: 0,
        },
        Message {
            channel_id: "ch-incidents",
            author: "bob",
            content: "🪵 All systems nominal.",
            offset_min: 5,
        },
    ],
    threads: &[],
    reactions: &[],
    invites: &[],
    allowlist: &[
        AllowlistEntry {
            pubkey_user: "alice",
            added_by: "alice",
        },
        AllowlistEntry {
            pubkey_user: "bob",
            added_by: "alice",
        },
    ],
    bots: &[],
};
