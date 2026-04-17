use crate::seed::{self, User};
use anyhow::Result;
use chrono::{DateTime, Duration, TimeZone, Utc};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string()
}

fn make_msg_id(channel_id: &str, idx: usize) -> String {
    format!("msg-{channel_id}-{idx:03}")
}

fn make_thread_id(parent: &str) -> String {
    format!("thread-{parent}-00")
}

fn base_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 4, 15, 10, 0, 0).unwrap()
}

pub fn seed_all(root: &Path, users: &[User]) -> Result<()> {
    for cfg in seed::all_servers() {
        seed_one(root, cfg, users)?;
        println!("  ✓ {} seeded", cfg.display_name);
    }
    Ok(())
}

fn seed_one(root: &Path, cfg: &seed::ServerConfig, users: &[User]) -> Result<()> {
    let db_path = root.join("test-configs").join(cfg.db_name);
    if !db_path.exists() {
        println!(
            "  WARNING: {} does not exist, skipping seed",
            db_path.display()
        );
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;
    let ts = now_iso();

    // Insert users.
    for user_def in cfg.users {
        let user = User::find(users, user_def.name);
        conn.execute(
            "INSERT OR IGNORE INTO users (id, pubkey, display_name, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![user.user_id, user.pubkey, user_def.display_name, ts, ts],
        )?;
    }

    // Insert members and roles.
    for user_def in cfg.users {
        let user = User::find(users, user_def.name);
        conn.execute(
            "INSERT OR IGNORE INTO members (user_id, joined_at) VALUES (?1, ?2)",
            rusqlite::params![user.user_id, ts],
        )?;
        for role_id in user_def.roles {
            conn.execute(
                "INSERT OR IGNORE INTO member_roles (user_id, role_id) VALUES (?1, ?2)",
                rusqlite::params![user.user_id, role_id],
            )?;
        }
    }

    // Insert custom roles.
    for role in cfg.custom_roles {
        conn.execute(
            "INSERT OR IGNORE INTO roles \
             (id, name, color, permissions, position, is_builtin, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
            rusqlite::params![
                role.id,
                role.name,
                role.color,
                role.permissions,
                role.position,
                ts,
                ts
            ],
        )?;
    }

    // Remove auto-created channels not in our seed config.
    let seeded_names: std::collections::HashSet<&str> =
        cfg.channels.iter().map(|c| c.name).collect();
    let mut stmt = conn.prepare("SELECT id, name FROM channels")?;
    let existing: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);
    for (ch_id, ch_name) in existing {
        if !seeded_names.contains(ch_name.as_str()) {
            conn.execute("DELETE FROM messages WHERE channel_id = ?1", [&ch_id])?;
            conn.execute("DELETE FROM channels WHERE id = ?1", [&ch_id])?;
        }
    }

    // Upsert channels.
    for (i, ch) in cfg.channels.iter().enumerate() {
        let existing_id: Option<String> = conn
            .query_row(
                "SELECT id FROM channels WHERE name = ?1",
                [ch.name],
                |row| row.get(0),
            )
            .optional()?;
        if existing_id.is_some() {
            conn.execute(
                "UPDATE channels SET id=?1, topic=?2, category=?3, position=?4, updated_at=?5 \
                 WHERE name=?6",
                rusqlite::params![ch.id, ch.topic, ch.category, i as i64, ts, ch.name],
            )?;
        } else {
            conn.execute(
                "INSERT INTO channels (id, name, topic, category, position, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![ch.id, ch.name, ch.topic, ch.category, i as i64, ts, ts],
            )?;
        }
    }

    // Insert messages with a global counter per DB.
    let bt = base_time();
    for (msg_count, msg) in cfg.messages.iter().enumerate() {
        let user = User::find(users, msg.author);
        let msg_time = bt + Duration::minutes(msg.offset_min);
        let msg_ts = msg_time.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let msg_id = make_msg_id(msg.channel_id, msg_count);
        conn.execute(
            "INSERT OR IGNORE INTO messages \
             (id, channel_id, author_id, content, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![msg_id, msg.channel_id, user.user_id, msg.content, msg_ts],
        )?;
    }

    // Insert invites.
    for inv in cfg.invites {
        let creator = User::find(users, inv.created_by);
        conn.execute(
            "INSERT OR IGNORE INTO invites \
             (code, created_by, grant_role_id, max_uses, use_count, expires_at, created_at) \
             VALUES (?1, ?2, NULL, ?3, 0, NULL, ?4)",
            rusqlite::params![inv.code, creator.user_id, inv.max_uses, ts],
        )?;
    }

    // Insert allowlist entries.
    for entry in cfg.allowlist {
        let adder = User::find(users, entry.added_by);
        let target = User::find(users, entry.pubkey_user);
        conn.execute(
            "INSERT OR IGNORE INTO allowlist (pubkey, added_by, added_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![target.pubkey, adder.user_id, ts],
        )?;
    }

    // Threads (after main data).
    let bt = base_time();
    for thread_cfg in cfg.threads {
        let parent_msg_id = thread_cfg.parent_message_id;
        let thread_id = make_thread_id(parent_msg_id);

        let parent_row: Option<(String, String)> = conn
            .query_row(
                "SELECT author_id, channel_id FROM messages WHERE id = ?1",
                [parent_msg_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (creator_id, channel_id) = match parent_row {
            Some(r) => r,
            None => {
                println!(
                    "  WARNING: Parent message {parent_msg_id} not found, skipping thread"
                );
                continue;
            }
        };

        conn.execute(
            "INSERT OR IGNORE INTO threads \
             (id, channel_id, parent_message_id, creator_id, created_at, reply_count) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            rusqlite::params![thread_id, channel_id, parent_msg_id, creator_id, ts],
        )?;

        for (idx, reply) in thread_cfg.replies.iter().enumerate() {
            let user = User::find(users, reply.author);
            let reply_time = bt + Duration::minutes(reply.offset_min);
            let reply_ts = reply_time.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
            let reply_msg_id = format!("msg-{thread_id}-reply-{idx:02}");

            conn.execute(
                "INSERT OR IGNORE INTO messages \
                 (id, channel_id, author_id, content, created_at, thread_id) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    reply_msg_id,
                    channel_id,
                    user.user_id,
                    reply.content,
                    reply_ts,
                    thread_id
                ],
            )?;
            conn.execute(
                "UPDATE threads SET reply_count = reply_count + 1, last_reply_at = ?1 \
                 WHERE id = ?2",
                rusqlite::params![reply_ts, thread_id],
            )?;
        }
    }

    // Reactions (after main data).
    for reaction_cfg in cfg.reactions {
        let msg_id = reaction_cfg.message_id;
        let exists: Option<String> = conn
            .query_row(
                "SELECT id FROM messages WHERE id = ?1",
                [msg_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            println!("  WARNING: Message {msg_id} not found, skipping reactions");
            continue;
        }
        for reaction in reaction_cfg.reactions {
            for user_name in reaction.users {
                let user = User::find(users, user_name);
                conn.execute(
                    "INSERT OR IGNORE INTO reactions \
                     (message_id, user_id, emoji, created_at) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![msg_id, user.user_id, reaction.emoji, ts],
                )?;
            }
        }
    }

    Ok(())
}
