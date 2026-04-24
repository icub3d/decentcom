# decentcom-welcomebot

Greets new members in a configured channel and optionally posts server rules.

## Quick start

```
cargo build -p decentcom-welcomebot
```

Create `welcomebot.toml`:

```toml
server_url         = "https://your-server.example.com"
mnemonic           = "word1 word2 ... word24"
display_name       = "WelcomeBot"        # optional
welcome_channel_id = "general"
welcome_template   = "Welcome, **{display_name}**! Glad to have you here."
rules_message      = "Please keep it friendly — see #rules for the full list."  # optional
```

Run:

```
./target/debug/welcomebot
```

Or with env vars (override any TOML field):

```
BOT_SERVER_URL=https://... BOT_MNEMONIC="..." ./welcomebot
```

## Configuration reference

| Key                  | Required | Default                                           | Description                                               |
|----------------------|----------|---------------------------------------------------|-----------------------------------------------------------|
| `server_url`         | yes      | —                                                 | Base URL of the decentcom server                          |
| `mnemonic`           | yes      | —                                                 | 24-word BIP39 mnemonic for the bot's identity             |
| `display_name`       | no       | —                                                 | Name shown in the member list                             |
| `welcome_channel_id` | yes      | —                                                 | Channel where welcome messages are posted                 |
| `welcome_template`   | no       | `"Welcome, **{display_name}**! Glad to have you here."` | Message template (`{display_name}`, `{pubkey}` supported) |
| `rules_message`      | no       | —                                                 | Extra message posted after the welcome                    |

### Environment variable overrides

| Variable              | Overrides              |
|-----------------------|------------------------|
| `BOT_SERVER_URL`      | `server_url`           |
| `BOT_MNEMONIC`        | `mnemonic`             |
| `BOT_DISPLAY_NAME`    | `display_name`         |
| `WELCOME_CHANNEL_ID`  | `welcome_channel_id`   |
| `WELCOME_TEMPLATE`    | `welcome_template`     |

## Authentication

The bot needs to be approved by a server admin before it can send messages.
After first run it will appear in the admin panel's pending-bot queue.
