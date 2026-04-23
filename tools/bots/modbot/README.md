# decentcom-modbot

Detects spam patterns and takes configurable action (warn / kick / ban).

## Detection rules

| Rule               | Description                                                |
|--------------------|------------------------------------------------------------|
| Duplicate messages | Same content sent N times within a sliding window          |
| Link flood         | More than N `http://`/`https://` links within a window     |

## Quick start

```
cargo build -p decentcom-modbot
```

Create `modbot.toml`:

```toml
server_url   = "https://your-server.example.com"
mnemonic     = "word1 word2 ... word24"
display_name = "ModBot"

[detection]
duplicate_threshold   = 5    # trigger after 5 identical messages …
duplicate_window_secs = 30   # … within 30 seconds
link_threshold        = 5    # trigger after 5 links …
link_window_secs      = 60   # … within 60 seconds

[action]
on_duplicate    = "warn"    # warn | kick | ban
on_link_flood   = "kick"
warn_channel_id = "general" # required if any action is "warn"
```

Run:

```
./target/debug/modbot
```

## Configuration reference

### `[detection]`

| Key                    | Default | Description                                  |
|------------------------|---------|----------------------------------------------|
| `duplicate_threshold`  | 5       | Identical-message count to trigger           |
| `duplicate_window_secs`| 30      | Sliding window for duplicate detection       |
| `link_threshold`       | 5       | Link count to trigger                        |
| `link_window_secs`     | 60      | Sliding window for link flood detection      |

### `[action]`

| Key               | Default  | Values              | Description                          |
|-------------------|----------|---------------------|--------------------------------------|
| `on_duplicate`    | `"warn"` | warn / kick / ban   | Action on duplicate message rule     |
| `on_link_flood`   | `"kick"` | warn / kick / ban   | Action on link flood rule            |
| `warn_channel_id` | —        | channel ID          | Channel for warning messages         |

### Environment variable overrides

| Variable          | Overrides     |
|-------------------|---------------|
| `BOT_SERVER_URL`  | `server_url`  |
| `BOT_MNEMONIC`    | `mnemonic`    |
| `BOT_DISPLAY_NAME`| `display_name`|

## Notes

- State is in-memory only; restarting the bot resets all counters.
- The `timeout` action is not available in this release (no server-side endpoint yet).
- Bot's own messages are never flagged.
