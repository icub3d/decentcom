//! Drives the SDK to populate a single server with the seed data declared in
//! [`crate::data`]. This is the SDK-based replacement for the legacy raw-SQL
//! seeder that used to live in `tools/test-setup/src/db.rs`.

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use decentcom_sdk::rest::models::{
    ApproveBotRequest, CreateChannelRequest, CreateInviteRequest, CreateRoleRequest,
    UpdateChannelRequest, UpdateProfileRequest,
};
use decentcom_sdk::Client;

use crate::data::{ServerConfig, ThreadSeed};
use crate::users::User;

#[derive(Debug, Default, Clone, Copy)]
pub struct SeedReport {
    pub channels: usize,
    pub roles: usize,
    pub members: usize,
    pub messages: usize,
    pub threads: usize,
    pub reactions: usize,
    pub invites: usize,
    pub allowlist: usize,
    pub bots: usize,
}

impl SeedReport {
    pub fn merge(&self, other: &SeedReport) -> SeedReport {
        SeedReport {
            channels: self.channels + other.channels,
            roles: self.roles + other.roles,
            members: self.members + other.members,
            messages: self.messages + other.messages,
            threads: self.threads + other.threads,
            reactions: self.reactions + other.reactions,
            invites: self.invites + other.invites,
            allowlist: self.allowlist + other.allowlist,
            bots: self.bots + other.bots,
        }
    }

    pub fn print_indented(&self, prefix: &str) {
        println!("{prefix}members:    {}", self.members);
        println!("{prefix}roles:      {}", self.roles);
        println!("{prefix}channels:   {}", self.channels);
        println!("{prefix}messages:   {}", self.messages);
        println!("{prefix}threads:    {}", self.threads);
        println!("{prefix}reactions:  {}", self.reactions);
        println!("{prefix}invites:    {}", self.invites);
        println!("{prefix}allowlist:  {}", self.allowlist);
        println!("{prefix}bots:       {}", self.bots);
    }
}

/// Per-server lookup tables built up while seeding.
struct SeedCtx<'a> {
    cfg: &'a ServerConfig,
    users: &'a [User],
    /// One authenticated, member-joined SDK client per seed user (alice, bob, …).
    clients: HashMap<&'a str, Client>,
    /// `name` (e.g. "general") → server-assigned channel id.
    channel_ids: HashMap<&'static str, String>,
    /// `(channel_name, message_index)` → server-assigned message id.
    message_ids: HashMap<(&'static str, usize), String>,
    /// Custom role name → server-assigned role id.
    role_ids: HashMap<&'static str, String>,
}

impl<'a> SeedCtx<'a> {
    fn new(cfg: &'a ServerConfig, users: &'a [User]) -> Self {
        Self {
            cfg,
            users,
            clients: HashMap::new(),
            channel_ids: HashMap::new(),
            message_ids: HashMap::new(),
            role_ids: HashMap::new(),
        }
    }

    fn user(&self, name: &str) -> &'a User {
        User::find(self.users, name)
    }

    fn client(&self, user_name: &str) -> Result<Client> {
        self.clients
            .get(user_name)
            .cloned()
            .ok_or_else(|| anyhow!("no authenticated client for user {user_name}"))
    }

    fn channel_id(&self, name: &str) -> Result<&str> {
        self.channel_ids
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("no channel id for name {name}"))
    }

    fn message_id(&self, channel_name: &str, idx: usize) -> Result<&str> {
        self.message_ids
            .get(&(
                self.canonical_channel_name(channel_name),
                idx,
            ))
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("no message id for ({channel_name}, {idx})"))
    }

    /// Resolve a borrowed channel name back to the `'static` literal that owns
    /// it inside `cfg.channels`. Required because hash keys in `message_ids`
    /// use the static lifetime supplied by the seed data.
    fn canonical_channel_name(&self, name: &str) -> &'static str {
        for ch in self.cfg.channels {
            if ch.name == name {
                return ch.name;
            }
        }
        panic!("unknown channel name in seed data: {name}");
    }
}

pub async fn seed_server(cfg: &'static ServerConfig, users: &[User]) -> Result<SeedReport> {
    let mut ctx = SeedCtx::new(cfg, users);
    let mut report = SeedReport::default();

    // 1. Owner (first listed user) authenticates and becomes the admin.
    let owner_name = cfg
        .users
        .first()
        .ok_or_else(|| anyhow!("server {} has no users configured", cfg.display_name))?
        .name;

    let owner_client = authenticate(cfg.base_url, ctx.user(owner_name)).await?;
    set_display_name(&owner_client, cfg.users[0].display_name).await?;
    // Owner is auto-joined as admin on first user creation regardless of mode.
    ctx.clients.insert(owner_name, owner_client);
    report.members += 1;

    // 2. Custom roles must exist before we hand them out.
    seed_roles(&mut ctx, &mut report).await?;

    // 3. Channels (creates new + patches the auto-seeded "general").
    seed_channels(&mut ctx, &mut report).await?;

    // 4. Allowlist entries (so allowlist members can join).
    seed_allowlist(&mut ctx, &mut report).await?;

    // 5. Invites (so invite-only members can join).
    let invite_codes = seed_invites(&mut ctx, &mut report).await?;

    // 6. Authenticate and join the rest of the seed users.
    seed_other_members(&mut ctx, &mut report, &invite_codes).await?;

    // 7. Assign custom roles to users that need them.
    assign_member_roles(&mut ctx).await?;

    // 8. Post messages (each user posts as themselves).
    seed_messages(&mut ctx, &mut report).await?;

    // 9. Threads.
    seed_threads(&mut ctx, &mut report).await?;

    // 10. Reactions.
    seed_reactions(&mut ctx, &mut report).await?;

    // 11. Bot users.
    seed_bots(&mut ctx, &mut report).await?;

    Ok(report)
}

// ── Step implementations ─────────────────────────────────────────────────────

async fn authenticate(base_url: &str, user: &User) -> Result<Client> {
    let client = Client::new(base_url)
        .with_context(|| format!("building SDK client for {base_url}"))?;
    client
        .authenticate(&user.identity)
        .await
        .with_context(|| format!("authenticating {} ({})", user.name, base_url))?;
    Ok(client)
}

async fn set_display_name(client: &Client, display_name: &str) -> Result<()> {
    client
        .profiles()
        .update(&UpdateProfileRequest {
            display_name: Some(display_name.to_string()),
        })
        .await
        .context("updating profile display name")?;
    Ok(())
}

async fn seed_roles(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    if ctx.cfg.custom_roles.is_empty() {
        return Ok(());
    }
    let owner = ctx.client(ctx.cfg.users[0].name)?;
    let existing = owner.roles().list().await.context("listing roles")?;
    for role_def in ctx.cfg.custom_roles {
        if let Some(found) = existing.iter().find(|r| r.name == role_def.name) {
            ctx.role_ids.insert(role_def.name, found.id.clone());
            continue;
        }
        let created = owner
            .roles()
            .create(&CreateRoleRequest {
                name: role_def.name.to_string(),
                color: role_def.color.map(|s| s.to_string()),
                permissions: role_def.permissions,
                position: Some(role_def.position),
            })
            .await
            .with_context(|| format!("creating role {}", role_def.name))?;
        ctx.role_ids.insert(role_def.name, created.id);
        report.roles += 1;
    }
    Ok(())
}

async fn seed_channels(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    let owner = ctx.client(ctx.cfg.users[0].name)?;
    let existing = owner.channels().list().await.context("listing channels")?;
    let by_name: HashMap<&str, &decentcom_sdk::rest::models::Channel> =
        existing.iter().map(|c| (c.name.as_str(), c)).collect();

    for (i, ch_def) in ctx.cfg.channels.iter().enumerate() {
        let position = i as i32;
        if let Some(existing_ch) = by_name.get(ch_def.name) {
            owner
                .channels()
                .update(
                    &existing_ch.id,
                    &UpdateChannelRequest {
                        name: Some(ch_def.name.to_string()),
                        topic: ch_def.topic.map(|s| s.to_string()),
                        category: Some(ch_def.category.map(|s| s.to_string())),
                        position: Some(position),
                    },
                )
                .await
                .with_context(|| format!("updating channel {}", ch_def.name))?;
            ctx.channel_ids.insert(ch_def.name, existing_ch.id.clone());
        } else {
            let created = owner
                .channels()
                .create(&CreateChannelRequest {
                    name: ch_def.name.to_string(),
                    category: ch_def.category.map(|s| s.to_string()),
                    position: Some(position),
                })
                .await
                .with_context(|| format!("creating channel {}", ch_def.name))?;
            // Topic is set via update, since CreateChannelRequest does not carry it.
            if ch_def.topic.is_some() {
                owner
                    .channels()
                    .update(
                        &created.id,
                        &UpdateChannelRequest {
                            topic: ch_def.topic.map(|s| s.to_string()),
                            ..Default::default()
                        },
                    )
                    .await
                    .with_context(|| format!("setting topic for {}", ch_def.name))?;
            }
            ctx.channel_ids.insert(ch_def.name, created.id);
            report.channels += 1;
        }
    }
    Ok(())
}

async fn seed_allowlist(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    if ctx.cfg.allowlist.is_empty() {
        return Ok(());
    }
    let owner = ctx.client(ctx.cfg.users[0].name)?;
    for entry in ctx.cfg.allowlist {
        let target = ctx.user(entry.pubkey_user);
        owner
            .members()
            .add_allowlist(target.pubkey().to_string())
            .await
            .with_context(|| format!("allowlisting {}", target.name))?;
        report.allowlist += 1;
    }
    Ok(())
}

/// Returns a map of `created_by_user_name → invite code` so the join phase can
/// look up which invite to use.
async fn seed_invites(
    ctx: &mut SeedCtx<'_>,
    report: &mut SeedReport,
) -> Result<HashMap<&'static str, String>> {
    let mut codes: HashMap<&'static str, String> = HashMap::new();
    if ctx.cfg.invites.is_empty() {
        return Ok(codes);
    }
    for invite_def in ctx.cfg.invites {
        let creator_client = ctx.client(invite_def.created_by)?;
        let invite = creator_client
            .invites()
            .create(&CreateInviteRequest {
                max_uses: Some(invite_def.max_uses),
                expires_in_seconds: None,
                grant_role_id: None,
            })
            .await
            .with_context(|| format!("creating invite by {}", invite_def.created_by))?;
        for joiner in invite_def.joiners {
            codes.insert(joiner, invite.code.clone());
        }
        report.invites += 1;
    }
    Ok(codes)
}

async fn seed_other_members(
    ctx: &mut SeedCtx<'_>,
    report: &mut SeedReport,
    invite_codes: &HashMap<&'static str, String>,
) -> Result<()> {
    for (i, user_def) in ctx.cfg.users.iter().enumerate() {
        if i == 0 {
            // owner already handled
            continue;
        }
        let user = ctx.user(user_def.name);
        let client = authenticate(ctx.cfg.base_url, user).await?;
        set_display_name(&client, user_def.display_name).await?;

        // Decide how to join.
        if let Some(code) = invite_codes.get(user_def.name) {
            client
                .invites()
                .join(code)
                .await
                .with_context(|| format!("{} joining via invite", user_def.name))?;
        } else {
            // Try POST /members/join. For Open this is a no-op (auto-joined),
            // for Allowlist it succeeds because the entry was added above.
            // We tolerate "already a member" / 409 by checking membership.
            let me = client
                .members()
                .get(user.pubkey())
                .await
                .ok();
            if me.is_none() {
                client
                    .members()
                    .join()
                    .await
                    .with_context(|| format!("{} joining server", user_def.name))?;
            }
        }
        ctx.clients.insert(user_def.name, client);
        report.members += 1;
    }
    Ok(())
}

async fn assign_member_roles(ctx: &mut SeedCtx<'_>) -> Result<()> {
    let owner = ctx.client(ctx.cfg.users[0].name)?;
    for user_def in ctx.cfg.users {
        for role_name in user_def.roles {
            let role_id = ctx
                .role_ids
                .get(role_name)
                .ok_or_else(|| anyhow!("role {role_name} not seeded"))?;
            let user = ctx.user(user_def.name);
            owner
                .roles()
                .add_member_role(user.pubkey(), role_id)
                .await
                .with_context(|| {
                    format!("assigning role {role_name} to {}", user_def.name)
                })?;
        }
    }
    Ok(())
}

async fn seed_messages(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    // Track per-channel message indices so we can build (channel, idx) → id.
    let mut per_channel_index: HashMap<&'static str, usize> = HashMap::new();

    for msg in ctx.cfg.messages {
        let channel_name = ctx.canonical_channel_name(msg.channel_name);
        let channel_id = ctx.channel_id(channel_name)?.to_string();
        let author_client = ctx.client(msg.author)?;
        let sent = author_client
            .messages()
            .send_text(&channel_id, msg.content)
            .await
            .with_context(|| {
                format!(
                    "{} sending message in #{}",
                    msg.author, channel_name
                )
            })?;
        let idx = *per_channel_index.entry(channel_name).or_insert(0);
        ctx.message_ids.insert((channel_name, idx), sent.id);
        per_channel_index.insert(channel_name, idx + 1);
        report.messages += 1;
    }
    Ok(())
}

async fn seed_threads(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    for ThreadSeed {
        channel_name,
        message_index,
        replies,
    } in ctx.cfg.threads
    {
        let channel_id = ctx.channel_id(channel_name)?.to_string();
        let parent_id = ctx.message_id(channel_name, *message_index)?.to_string();
        // Use the parent message's author client to create the thread (any
        // member could, but choosing the parent author is intuitive).
        let parent_author = lookup_message_author(ctx.cfg, channel_name, *message_index)?;
        let parent_client = ctx.client(parent_author)?;
        let thread = parent_client
            .threads()
            .create(&channel_id, &parent_id)
            .await
            .with_context(|| {
                format!("creating thread on ({channel_name}, {message_index})")
            })?;
        for reply in *replies {
            let reply_client = ctx.client(reply.author)?;
            reply_client
                .threads()
                .post_message(
                    &thread.thread_id,
                    &decentcom_sdk::rest::models::CreateMessageRequest {
                        content: reply.content.to_string(),
                        attachment_ids: Vec::new(),
                    },
                )
                .await
                .with_context(|| {
                    format!("posting thread reply by {} on {}", reply.author, thread.thread_id)
                })?;
        }
        report.threads += 1;
    }
    Ok(())
}

async fn seed_reactions(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    for r in ctx.cfg.reactions {
        let channel_id = ctx.channel_id(r.channel_name)?.to_string();
        let message_id = ctx.message_id(r.channel_name, r.message_index)?.to_string();
        for emoji in r.reactions {
            for user_name in emoji.users {
                let client = ctx.client(user_name)?;
                client
                    .reactions()
                    .add(&channel_id, &message_id, emoji.emoji)
                    .await
                    .with_context(|| {
                        format!(
                            "{} reacting {} on ({}, {})",
                            user_name, emoji.emoji, r.channel_name, r.message_index
                        )
                    })?;
                report.reactions += 1;
            }
        }
    }
    Ok(())
}

async fn seed_bots(ctx: &mut SeedCtx<'_>, report: &mut SeedReport) -> Result<()> {
    if ctx.cfg.bots.is_empty() {
        return Ok(());
    }
    let owner = ctx.client(ctx.cfg.users[0].name)?;
    for bot in ctx.cfg.bots {
        let identity = ctx.user(bot.name);
        let client = Client::new(ctx.cfg.base_url)
            .with_context(|| format!("building bot client {}", bot.name))?;
        client
            .authenticate_as_bot(&identity.identity)
            .await
            .with_context(|| format!("bot {} authenticating", bot.name))?;
        if bot.approve {
            owner
                .members()
                .approve_bot(identity.pubkey(), &ApproveBotRequest::default())
                .await
                .with_context(|| format!("approving bot {}", bot.name))?;
        }
        report.bots += 1;
    }
    Ok(())
}

fn lookup_message_author(
    cfg: &ServerConfig,
    channel_name: &str,
    message_index: usize,
) -> Result<&'static str> {
    let mut idx = 0;
    for msg in cfg.messages {
        if msg.channel_name == channel_name {
            if idx == message_index {
                return Ok(msg.author);
            }
            idx += 1;
        }
    }
    Err(anyhow!(
        "no seed message at ({channel_name}, {message_index})"
    ))
}
