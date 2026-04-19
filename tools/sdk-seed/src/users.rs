use decentcom_sdk::Identity;

/// A test identity with a deterministic Ed25519 seed.
pub struct User {
    pub name: &'static str,
    pub identity: Identity,
}

impl User {
    fn new(name: &'static str, seed: [u8; 32]) -> Self {
        Self {
            name,
            identity: Identity::from_seed(&seed),
        }
    }

    pub fn pubkey(&self) -> &str {
        self.identity.pubkey()
    }

    pub fn find<'a>(users: &'a [Self], name: &str) -> &'a Self {
        users
            .iter()
            .find(|u| u.name == name)
            .unwrap_or_else(|| panic!("unknown seed user: {name}"))
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
}
