# decentcom-bot

Bot SDK for the [decentcom](https://github.com/icub3d/decentcom) protocol.

Wraps the gateway WebSocket and REST API with typed event handlers and action
helpers so you can build bots in Rust without reimplementing the protocol.

## Quick start

```rust
use decentcom_bot::{Bot, Context, events::MessageEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    Bot::from_env()?
        .on_message(handle_message)
        .run()
        .await?;

    Ok(())
}

async fn handle_message(ctx: Context, evt: MessageEvent) -> anyhow::Result<()> {
    if evt.content.starts_with("!ping") {
        ctx.reply(&evt, "pong").await?;
    }
    Ok(())
}
```

Set `BOT_SERVER_URL` and `BOT_MNEMONIC` in your environment, then run:

```
BOT_SERVER_URL=https://your-server.example.com \
BOT_MNEMONIC="word1 word2 ... word24" \
cargo run
```

## Configuration

Config can come from environment variables, a TOML file, or both (env vars win).

### Environment variables

| Variable          | Required | Description                                     |
|-------------------|----------|-------------------------------------------------|
| `BOT_SERVER_URL`  | yes      | Base URL of the decentcom server                |
| `BOT_MNEMONIC`    | yes      | 24-word BIP39 mnemonic for the bot's identity   |
| `BOT_DISPLAY_NAME`| no       | Display name shown in the member list           |

### TOML file

```toml
server_url   = "https://your-server.example.com"
mnemonic     = "word1 word2 ... word24"
display_name = "My Bot"      # optional
```

Load it with `Bot::from_file("bot.toml")`.

## Event handlers

Register handlers via the builder methods. Every handler receives a cloned
[`Context`] and the typed event payload.

| Method                    | Event payload                              |
|---------------------------|--------------------------------------------|
| `on_message`              | New message posted                         |
| `on_message_update`       | Message edited                             |
| `on_message_delete`       | Message deleted                            |
| `on_channel_create`       | Channel created                            |
| `on_channel_update`       | Channel updated                            |
| `on_channel_delete`       | Channel deleted                            |
| `on_role_create`          | Role created                               |
| `on_role_update`          | Role updated                               |
| `on_role_delete`          | Role deleted                               |
| `on_member_role_add`      | Role added to a member                     |
| `on_member_role_remove`   | Role removed from a member                 |
| `on_member_join`          | Member joined the server                   |
| `on_member_leave`         | Member left the server                     |
| `on_member_kick`          | Member was kicked                          |
| `on_member_ban`           | Member was banned                          |
| `on_member_update`        | Member profile updated                     |
| `on_reaction_add`         | Reaction added to a message                |
| `on_reaction_remove`      | Reaction removed from a message            |
| `on_thread_create`        | Thread created                             |
| `on_thread_message_create`| Reply posted to a thread                   |
| `on_thread_update`        | Thread updated (reply count etc.)          |

## Context — REST actions

`Context` is the handle your handlers use to interact with the server.

```rust
// Send a message
ctx.send(channel_id, "Hello!").await?;

// Reply in the same channel as the incoming message
ctx.reply(&evt, "pong").await?;

// Edit a message (bot must be the author)
ctx.edit_message(channel_id, message_id, "updated text").await?;

// Delete a message
ctx.delete_message(channel_id, message_id).await?;

// Kick a member (by pubkey)
ctx.kick(pubkey).await?;

// Ban a member
ctx.ban(pubkey, Some("reason".to_string())).await?;

// Access the full SDK client for anything else
let members = ctx.sdk().members().list().await?;
```

## Authentication

The bot uses the same Ed25519 challenge-response flow as human users, with an
`is_bot: true` claim bound into the signed challenge. The server will queue the
bot as pending until an admin approves it via the admin panel or API.

The bot's Ed25519 keypair is derived deterministically from its BIP39 mnemonic.
Keep the mnemonic secret — it is the bot's identity.

## Reconnect behavior

`Bot::run()` loops forever. On any gateway disconnect it waits with exponential
back-off (1 s → 2 s → 4 s → … → 64 s cap) then reconnects and re-authenticates.

## Logging

`decentcom-bot` uses the [`tracing`](https://docs.rs/tracing) crate. Add a
subscriber in `main()`:

```rust
tracing_subscriber::fmt::init();
```

Or configure it however suits your deployment (JSON output, log levels, etc.).
