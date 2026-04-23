# decentcom-auditbot

Streams moderation events to a configurable external sink — either a local
newline-delimited JSON (NDJSON) file or an HTTP webhook.

## Quick start

```
cargo build -p decentcom-auditbot
```

### File sink example (`auditbot.toml`)

```toml
server_url   = "https://your-server.example.com"
mnemonic     = "word1 word2 ... word24"
display_name = "AuditBot"
sink_type    = "file"
sink_path    = "/var/log/decentcom/audit.ndjson"

[filter]
member_kick    = true
member_ban     = true
message_delete = true
member_join    = false
member_leave   = false
```

### Webhook sink example

```toml
server_url = "https://your-server.example.com"
mnemonic   = "word1 word2 ... word24"
sink_type  = "webhook"
sink_url   = "https://hooks.example.com/decentcom-audit"
```

Run:

```
./target/debug/auditbot
```

## Event record format

Each event is a JSON object. Example records:

```json
{"event":"MEMBER_KICK","timestamp":"2024-01-15T12:00:00Z","user_id":"u1","pubkey":"abc…","reason":"spam"}
{"event":"MEMBER_BAN","timestamp":"2024-01-15T12:01:00Z","user_id":"u2","pubkey":"def…","reason":null}
{"event":"MESSAGE_DELETE","timestamp":"2024-01-15T12:02:00Z","message_id":"m3","channel_id":"c1"}
{"event":"MEMBER_JOIN","timestamp":"2024-01-15T12:03:00Z","user_id":"u4","pubkey":"ghi…","is_bot":false}
{"event":"MEMBER_LEAVE","timestamp":"2024-01-15T12:04:00Z","user_id":"u5","pubkey":"jkl…"}
```

## Configuration reference

| Key           | Required | Description                                         |
|---------------|----------|-----------------------------------------------------|
| `server_url`  | yes      | Base URL of the decentcom server                    |
| `mnemonic`    | yes      | 24-word BIP39 mnemonic                              |
| `display_name`| no       | Name shown in the member list                       |
| `sink_type`   | yes      | `"file"` or `"webhook"`                             |
| `sink_path`   | if file  | Path to the NDJSON log file (created if missing)    |
| `sink_url`    | if webhook | URL to POST events to                              |

### `[filter]`

| Key             | Default | Description                         |
|-----------------|---------|-------------------------------------|
| `member_kick`   | `true`  | Log member kick events              |
| `member_ban`    | `true`  | Log member ban events               |
| `message_delete`| `true`  | Log message delete events           |
| `member_join`   | `false` | Log member join events              |
| `member_leave`  | `false` | Log member leave events             |

### Environment variable overrides

| Variable          | Overrides     |
|-------------------|---------------|
| `BOT_SERVER_URL`  | `server_url`  |
| `BOT_MNEMONIC`    | `mnemonic`    |
| `BOT_DISPLAY_NAME`| `display_name`|
| `AUDIT_SINK_PATH` | `sink_path`   |
| `AUDIT_SINK_URL`  | `sink_url`    |

## Notes

- Webhook failures are logged but do not crash the bot.
- File writes are serialised with an async lock; concurrent events are safe.
