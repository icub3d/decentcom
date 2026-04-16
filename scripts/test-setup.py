#!/usr/bin/env python3
"""Bootstrap test databases with users, roles, channels, and invites.

Run this AFTER 'make clean' and BEFORE starting the servers.
The servers run their own migrations on startup, so this script
starts each server briefly to run migrations, then inserts seed data.

Usage:
    python3 scripts/test-setup.py          # full setup
    python3 scripts/test-setup.py --clean  # clean only (DBs + keychain)
"""

import json
import os
import secrets
import sqlite3
import string
import subprocess
import sys
import time
import signal
from datetime import datetime, timezone, timedelta
from pathlib import Path

# ---------------------------------------------------------------------------
# Test user definitions (deterministic seeds for reproducibility)
# ---------------------------------------------------------------------------
# These are NOT real accounts — they use trivial seeds for testing only.

B58_ALPHABET = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

def b58encode(data: bytes) -> str:
    n = int.from_bytes(data, "big")
    result = b""
    while n > 0:
        n, r = divmod(n, 58)
        result = bytes([B58_ALPHABET[r]]) + result
    for b in data:
        if b == 0:
            result = bytes([B58_ALPHABET[0]]) + result
        else:
            break
    return result.decode()

def make_user(name: str, seed_byte: int):
    """Create a test user dict from a repeated single byte seed."""
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    seed = bytes([seed_byte] * 32)
    key = Ed25519PrivateKey.from_private_bytes(seed)
    pub_bytes = key.public_key().public_bytes_raw()
    return {
        "name": name,
        "seed": seed,
        "hex_seed": seed.hex(),
        "pubkey": b58encode(pub_bytes),
        "user_id": f"user-{name}",
    }

USERS = {
    "alice":   make_user("alice", 1),
    "bob":     make_user("bob", 2),
    "charlie": make_user("charlie", 3),
    "dave":    make_user("dave", 4),
}

# Permissions (must match server/src/permissions.rs)
SEND_MESSAGES   = 1 << 0
READ_MESSAGES   = 1 << 1
MANAGE_MESSAGES = 1 << 2
MANAGE_CHANNELS = 1 << 3
MANAGE_ROLES    = 1 << 4
KICK_MEMBERS    = 1 << 5
BAN_MEMBERS     = 1 << 6
MANAGE_INVITES  = 1 << 7
MANAGE_SERVER   = 1 << 8
ATTACH_FILES    = 1 << 9
ADD_REACTIONS   = 1 << 10
MENTION_EVERYONE = 1 << 11
VIEW_AUDIT_LOG  = 1 << 12
ADMINISTRATOR   = 1 << 13
ALL_PERMISSIONS = (1 << 14) - 1

ROOT = Path(__file__).resolve().parent.parent
CONFIGS = ROOT / "test-configs"

DB_FILES = ["open.db", "private.db", "strict.db"]
MEDIA_DIRS = ["media-open", "media-private", "media-strict"]
KEYCHAIN_SERVICE = "decentcom"

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------

def clean_databases():
    """Remove all test database files and media directories."""
    for db in DB_FILES:
        for suffix in ["", "-shm", "-wal"]:
            p = CONFIGS / f"{db}{suffix}"
            if p.exists():
                p.unlink()
                print(f"  Removed {p.relative_to(ROOT)}")
    for d in MEDIA_DIRS:
        p = CONFIGS / d
        if p.exists():
            import shutil
            shutil.rmtree(p)
            print(f"  Removed {p.relative_to(ROOT)}/")
        p.mkdir(exist_ok=True)

def clean_webview_data():
    """Remove the Tauri WebView data directory (localStorage, cache, etc.)."""
    import shutil
    data_dir = Path.home() / ".local" / "share" / "com.jmarsh.client"
    if data_dir.exists():
        shutil.rmtree(data_dir)
        print(f"  Removed Tauri WebView data ({data_dir})")
    else:
        print(f"  No WebView data to clean")

def clean_keychain():
    """Remove all decentcom test entries from the OS keychain.

    The Rust `keyring` crate (v3+) stores entries with attributes:
      application=rust-keyring, service=<svc>, target=default, username=<key>
    We must match these attributes when looking up and clearing entries.
    We also clean any legacy entries that used the simpler schema.
    """
    KR_ATTRS = ["application", "rust-keyring", "service", KEYCHAIN_SERVICE, "target", "default"]

    # First read the accounts index to find per-account seeds.
    try:
        result = subprocess.run(
            ["secret-tool", "lookup"] + KR_ATTRS + ["username", "accounts_index"],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode == 0 and result.stdout.strip():
            pubkeys = json.loads(result.stdout.strip())
            for pk in pubkeys:
                for prefix in ["seed_", "label_"]:
                    subprocess.run(
                        ["secret-tool", "clear"] + KR_ATTRS + ["username", f"{prefix}{pk}"],
                        capture_output=True, timeout=5,
                    )
                print(f"  Removed keychain entries for {pk[:12]}…")
    except Exception:
        pass

    # Clear the index itself and legacy entries.
    for name in ["accounts_index", "master_privkey_seed", "master_privkey"]:
        subprocess.run(
            ["secret-tool", "clear"] + KR_ATTRS + ["username", name],
            capture_output=True, timeout=5,
        )
        # Also clear legacy (non-keyring) format if present.
        subprocess.run(
            ["secret-tool", "clear", "service", KEYCHAIN_SERVICE, "username", name],
            capture_output=True, timeout=5,
        )
    print("  Cleared keychain entries")

def seed_localstorage():
    """Pre-populate the Tauri WebView localStorage with server connections for each account.

    WebKit stores localStorage as a SQLite database with keys and values encoded as
    UTF-16LE bytes. The file is named after the origin: http_localhost_1420 for dev
    (Vite's default port).
    """
    ls_dir = Path.home() / ".local" / "share" / "com.jmarsh.client" / "localstorage"
    ls_dir.mkdir(parents=True, exist_ok=True)
    ls_db = ls_dir / "http_localhost_1420.localstorage"

    conn = sqlite3.connect(str(ls_db))
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable "
        "(key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB NOT NULL ON CONFLICT FAIL)"
    )

    def enc(s: str) -> bytes:
        """Encode a value as UTF-16LE blob (matching WebKitGTK localStorage format).
        Keys are stored as plain text strings; values as UTF-16LE blobs."""
        return s.encode("utf-16-le")

    OPEN    = {"id": "http://localhost:8081", "address": "http://localhost:8081", "name": "Open Server"}
    PRIVATE = {"id": "http://localhost:8082", "address": "http://localhost:8082", "name": "Private Server"}
    STRICT  = {"id": "http://localhost:8083", "address": "http://localhost:8083", "name": "Strict Server"}

    # Map each account to the servers they belong to (matches DB seed above).
    account_servers = {
        "alice":   [OPEN, PRIVATE, STRICT],
        "bob":     [OPEN, PRIVATE, STRICT],
        "charlie": [OPEN],
        "dave":    [OPEN],
    }

    for name, servers in account_servers.items():
        pk = USERS[name]["pubkey"]
        state = json.dumps({
            "state": {
                "currentServerId": OPEN["id"],
                "servers": {s["id"]: s for s in servers},
                "theme": "mocha",
            },
            "version": 0,
        })
        key = f"decentcom-app-storage-{pk}"
        conn.execute(
            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
            (key, enc(state)),
        )

    # Set alice as the default active account.
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        ("decentcom-active-pubkey", enc(USERS["alice"]["pubkey"])),
    )

    conn.commit()
    conn.close()
    print(f"  Seeded localStorage for {len(account_servers)} accounts (active: alice)")


def store_test_accounts_in_keychain():
    """Store the test user seeds in the OS keychain so the Tauri client can use them.

    The Rust `keyring` crate (v3+) looks up entries with:
      application=rust-keyring, service=decentcom, target=default, username=<key>
    We must create entries with these exact attributes.
    """
    KR_ATTRS = ["application", "rust-keyring", "service", KEYCHAIN_SERVICE, "target", "default"]
    pubkeys = [u["pubkey"] for u in USERS.values()]
    index_json = json.dumps(pubkeys)

    subprocess.run(
        ["secret-tool", "store", "--label", "decentcom accounts_index"]
        + KR_ATTRS + ["username", "accounts_index"],
        input=index_json, text=True, capture_output=True, timeout=5,
    )
    print(f"  Stored accounts_index with {len(pubkeys)} accounts")

    for user in USERS.values():
        subprocess.run(
            ["secret-tool", "store", "--label", f"decentcom seed_{user['name']}"]
            + KR_ATTRS + ["username", f"seed_{user['pubkey']}"],
            input=user["hex_seed"], text=True, capture_output=True, timeout=5,
        )
        # Store a friendly label for the account.
        subprocess.run(
            ["secret-tool", "store", "--label", f"decentcom label_{user['name']}"]
            + KR_ATTRS + ["username", f"label_{user['pubkey']}"],
            input=user["name"].capitalize(), text=True, capture_output=True, timeout=5,
        )
        print(f"  Stored seed + label for {user['name']} ({user['pubkey'][:12]}…)")

# ---------------------------------------------------------------------------
# Database bootstrap (run migrations via server, then insert seed data)
# ---------------------------------------------------------------------------

def now_iso():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")

def run_migrations(config_name: str):
    """Start the server briefly to run migrations, then kill it."""
    config_path = CONFIGS / f"{config_name}.toml"
    print(f"  Running migrations for {config_name}...")
    proc = subprocess.Popen(
        ["cargo", "run", "-q", "-p", "server", "--", "--config", str(config_path)],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=ROOT,
    )
    # Wait for server to start and run migrations (watch for bind message).
    time.sleep(3)
    proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()

def make_invite_code():
    alphabet = string.ascii_letters + string.digits
    return "".join(secrets.choice(alphabet) for _ in range(8))

def make_msg_id(channel_id: str, idx: int) -> str:
    return f"msg-{channel_id}-{idx:03d}"

def seed_database(db_name: str, config: dict):
    """Insert seed data into a server's SQLite database."""
    db_path = CONFIGS / db_name
    if not db_path.exists():
        print(f"  WARNING: {db_path} does not exist, skipping seed")
        return

    conn = sqlite3.connect(str(db_path))
    c = conn.cursor()
    ts = now_iso()

    # Insert users.
    for user_def in config["users"]:
        user = USERS[user_def["name"]]
        c.execute(
            "INSERT OR IGNORE INTO users (id, pubkey, display_name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            (user["user_id"], user["pubkey"], user_def.get("display_name", user["name"].capitalize()), ts, ts),
        )

    # Insert members and roles.
    for user_def in config["users"]:
        user = USERS[user_def["name"]]
        c.execute("INSERT OR IGNORE INTO members (user_id, joined_at) VALUES (?, ?)", (user["user_id"], ts))
        for role_id in user_def.get("roles", ["everyone"]):
            c.execute("INSERT OR IGNORE INTO member_roles (user_id, role_id) VALUES (?, ?)", (user["user_id"], role_id))

    # Insert custom roles (non-builtin).
    for role in config.get("custom_roles", []):
        c.execute(
            "INSERT OR IGNORE INTO roles (id, name, color, permissions, position, is_builtin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
            (role["id"], role["name"], role.get("color"), role["permissions"], role["position"], ts, ts),
        )

    # Remove any auto-created channels (e.g. the server seeds "general" on first
    # run) that aren't in our seed config.
    seeded_names = {ch["name"] for ch in config.get("channels", [])}
    c.execute("SELECT id, name FROM channels")
    for row in c.fetchall():
        if row[1] not in seeded_names:
            c.execute("DELETE FROM messages WHERE channel_id = ?", (row[0],))
            c.execute("DELETE FROM channels WHERE id = ?", (row[0],))

    # Insert channels. If a channel with the same name already exists (e.g. the
    # server auto-creates "general" on first run), update it to match our config.
    for i, ch in enumerate(config.get("channels", [])):
        # Try to claim an existing channel with the same name.
        c.execute("SELECT id FROM channels WHERE name = ?", (ch["name"],))
        existing = c.fetchone()
        if existing:
            c.execute(
                "UPDATE channels SET id=?, topic=?, category=?, position=?, updated_at=? WHERE name=?",
                (ch["id"], ch.get("topic"), ch.get("category"), i, ts, ch["name"]),
            )
        else:
            c.execute(
                "INSERT INTO channels (id, name, topic, category, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                (ch["id"], ch["name"], ch.get("topic"), ch.get("category"), i, ts, ts),
            )

    # Insert messages.
    base_time = datetime(2026, 4, 15, 10, 0, 0, tzinfo=timezone.utc)
    user_names = [u["name"] for u in config["users"]]
    msg_count = 0
    for msg in config.get("messages", []):
        author = USERS[msg["author"]]
        msg_time = base_time + timedelta(minutes=msg.get("offset_min", msg_count))
        msg_ts = msg_time.strftime("%Y-%m-%dT%H:%M:%S.000Z")
        msg_id = make_msg_id(msg["channel_id"], msg_count)
        c.execute(
            "INSERT OR IGNORE INTO messages (id, channel_id, author_id, content, created_at) VALUES (?, ?, ?, ?, ?)",
            (msg_id, msg["channel_id"], author["user_id"], msg["content"], msg_ts),
        )
        msg_count += 1

    # Insert invites.
    for inv in config.get("invites", []):
        creator = USERS[inv["created_by"]]
        c.execute(
            "INSERT OR IGNORE INTO invites (code, created_by, grant_role_id, max_uses, use_count, expires_at, created_at) VALUES (?, ?, ?, ?, 0, ?, ?)",
            (inv["code"], creator["user_id"], inv.get("grant_role_id"), inv.get("max_uses", 0), inv.get("expires_at"), ts),
        )

    # Insert allowlist entries.
    for entry in config.get("allowlist", []):
        adder = USERS[entry["added_by"]]
        target = USERS[entry["pubkey_user"]]
        c.execute(
            "INSERT OR IGNORE INTO allowlist (pubkey, added_by, added_at) VALUES (?, ?, ?)",
            (target["pubkey"], adder["user_id"], ts),
        )

    conn.commit()
    conn.close()

# ---------------------------------------------------------------------------
# Server configurations
# ---------------------------------------------------------------------------

# Shared invite code so it's printed and usable.
PRIVATE_INVITE = "TESTinv1"

OPEN_CONFIG = {
    "users": [
        {"name": "alice", "display_name": "Alice (Admin)", "roles": ["everyone", "admin"]},
        {"name": "bob", "display_name": "Bob", "roles": ["everyone"]},
        {"name": "charlie", "display_name": "Charlie (Restricted)", "roles": ["everyone", "restricted"]},
        {"name": "dave", "display_name": "Dave", "roles": ["everyone"]},
    ],
    "custom_roles": [
        {
            "id": "restricted",
            "name": "Restricted",
            "color": "#ef4444",
            "permissions": READ_MESSAGES | SEND_MESSAGES,  # no attach, no reactions, no manage
            "position": 500,
        },
    ],
    "channels": [
        # Text Channels
        {"id": "ch-general", "name": "general", "topic": "General discussion", "category": "Text Channels"},
        {"id": "ch-random", "name": "random", "topic": "Off-topic chat", "category": "Text Channels"},
        {"id": "ch-introductions", "name": "introductions", "topic": "Say hello!", "category": "Text Channels"},
        # Admin
        {"id": "ch-announcements", "name": "announcements", "topic": "Important updates from admins", "category": "Admin"},
        {"id": "ch-mod-log", "name": "mod-log", "topic": "Moderation activity log", "category": "Admin"},
        # Projects
        {"id": "ch-frontend", "name": "frontend", "topic": "UI/UX discussion", "category": "Projects"},
        {"id": "ch-backend", "name": "backend", "topic": "Server-side work", "category": "Projects"},
        {"id": "ch-design", "name": "design", "topic": "Design assets and feedback", "category": "Projects"},
    ],
    "messages": [
        # #general — a realistic conversation
        {"channel_id": "ch-general", "author": "alice", "content": "Welcome everyone to the Open Server! Feel free to chat here.", "offset_min": 0},
        {"channel_id": "ch-general", "author": "bob", "content": "Thanks Alice! Glad to be here.", "offset_min": 2},
        {"channel_id": "ch-general", "author": "charlie", "content": "Hey all 👋", "offset_min": 3},
        {"channel_id": "ch-general", "author": "dave", "content": "What's everyone working on today?", "offset_min": 5},
        {"channel_id": "ch-general", "author": "alice", "content": "Setting up the test environment and making sure everything works.", "offset_min": 7},
        {"channel_id": "ch-general", "author": "bob", "content": "I've been looking at the permissions system. Pretty slick.", "offset_min": 10},
        {"channel_id": "ch-general", "author": "dave", "content": "Same here. The role-based access is nice.", "offset_min": 12},
        {"channel_id": "ch-general", "author": "charlie", "content": "I seem to have limited access compared to you all though.", "offset_min": 14},
        {"channel_id": "ch-general", "author": "alice", "content": "That's by design, Charlie — you're on the restricted role for testing purposes.", "offset_min": 16},
        {"channel_id": "ch-general", "author": "charlie", "content": "Ah that makes sense. Good to know!", "offset_min": 17},
        # #random — lighter chat
        {"channel_id": "ch-random", "author": "bob", "content": "Anyone else think decentralized chat is the future?", "offset_min": 20},
        {"channel_id": "ch-random", "author": "dave", "content": "Absolutely. No single point of failure.", "offset_min": 22},
        {"channel_id": "ch-random", "author": "alice", "content": "That's the whole idea behind decentcom 🎉", "offset_min": 24},
        {"channel_id": "ch-random", "author": "bob", "content": "Plus Ed25519 keys instead of passwords. No more credential stuffing.", "offset_min": 26},
        {"channel_id": "ch-random", "author": "charlie", "content": "Wait, so my identity IS my key pair?", "offset_min": 28},
        {"channel_id": "ch-random", "author": "alice", "content": "Exactly. Your public key is your identity. No email, no username/password.", "offset_min": 30},
        # #introductions
        {"channel_id": "ch-introductions", "author": "alice", "content": "I'm Alice, admin of this server. I manage the infrastructure and keep things running.", "offset_min": 0},
        {"channel_id": "ch-introductions", "author": "bob", "content": "Bob here. I'm interested in backend development and distributed systems.", "offset_min": 5},
        {"channel_id": "ch-introductions", "author": "charlie", "content": "Charlie. Just exploring the platform, happy to help test things!", "offset_min": 10},
        {"channel_id": "ch-introductions", "author": "dave", "content": "Dave. Frontend dev. Excited about the Tauri + React stack.", "offset_min": 15},
        # #announcements — admin posts
        {"channel_id": "ch-announcements", "author": "alice", "content": "📢 Server rules: Be respectful, no spam, keep discussions on-topic.", "offset_min": 0},
        {"channel_id": "ch-announcements", "author": "alice", "content": "📢 New channels added under Projects category for dev discussions.", "offset_min": 60},
        {"channel_id": "ch-announcements", "author": "alice", "content": "📢 Reminder: Charlie is on the restricted role for testing — this is intentional.", "offset_min": 120},
        # #frontend — project chat
        {"channel_id": "ch-frontend", "author": "dave", "content": "The account switcher is looking great. Nice work on the labels.", "offset_min": 0},
        {"channel_id": "ch-frontend", "author": "bob", "content": "Agreed. The double-click-to-rename UX is intuitive.", "offset_min": 3},
        {"channel_id": "ch-frontend", "author": "alice", "content": "Thanks! Next up is polishing the channel management UI.", "offset_min": 6},
        {"channel_id": "ch-frontend", "author": "dave", "content": "Should we add drag-and-drop for reordering channels?", "offset_min": 10},
        {"channel_id": "ch-frontend", "author": "alice", "content": "That's on the roadmap. Let's get the basics solid first.", "offset_min": 12},
        # #backend
        {"channel_id": "ch-backend", "author": "bob", "content": "The SQLx migrations are clean. Love the trait-based storage abstraction.", "offset_min": 0},
        {"channel_id": "ch-backend", "author": "alice", "content": "That'll make it easy to swap in PostgreSQL later for the scale-out path.", "offset_min": 5},
        {"channel_id": "ch-backend", "author": "bob", "content": "Have we decided on the SFU strategy for WebRTC yet?", "offset_min": 10},
        {"channel_id": "ch-backend", "author": "alice", "content": "Not yet — that's an open question in the architecture doc. Milestone 3 territory.", "offset_min": 12},
        # #design
        {"channel_id": "ch-design", "author": "dave", "content": "I've been working with the Catppuccin Mocha palette. It looks fantastic in dark mode.", "offset_min": 0},
        {"channel_id": "ch-design", "author": "alice", "content": "Mocha is the default but we support all Catppuccin flavors.", "offset_min": 5},
        {"channel_id": "ch-design", "author": "bob", "content": "Can we add a light theme too? Not everyone likes dark mode.", "offset_min": 8},
        {"channel_id": "ch-design", "author": "dave", "content": "Catppuccin Latte is the light variant. We could default to it based on OS preference.", "offset_min": 11},
    ],
}

PRIVATE_CONFIG = {
    "users": [
        {"name": "alice", "display_name": "Alice (Admin)", "roles": ["everyone", "admin"]},
        {"name": "bob", "display_name": "Bob", "roles": ["everyone"]},
        # charlie and dave are NOT members — they can test joining via invite.
    ],
    "channels": [
        # General
        {"id": "ch-lobby", "name": "lobby", "topic": "Welcome! Start here.", "category": "General"},
        {"id": "ch-members-only", "name": "members-only", "topic": "For members", "category": "General"},
        # Planning
        {"id": "ch-roadmap", "name": "roadmap", "topic": "Feature planning and priorities", "category": "Planning"},
        {"id": "ch-feedback", "name": "feedback", "topic": "Share your thoughts", "category": "Planning"},
    ],
    "messages": [
        # #lobby
        {"channel_id": "ch-lobby", "author": "alice", "content": "Welcome to the Private Server! This is invite-only.", "offset_min": 0},
        {"channel_id": "ch-lobby", "author": "bob", "content": "Thanks for the invite, Alice!", "offset_min": 3},
        {"channel_id": "ch-lobby", "author": "alice", "content": "Feel free to share invite codes with people you trust.", "offset_min": 5},
        {"channel_id": "ch-lobby", "author": "bob", "content": "Will do. The invite system seems straightforward.", "offset_min": 8},
        # #members-only
        {"channel_id": "ch-members-only", "author": "alice", "content": "This channel is for discussions among verified members.", "offset_min": 0},
        {"channel_id": "ch-members-only", "author": "bob", "content": "Good to have a space separate from the lobby.", "offset_min": 5},
        {"channel_id": "ch-members-only", "author": "alice", "content": "Exactly. New members start in lobby until they get oriented.", "offset_min": 8},
        # #roadmap
        {"channel_id": "ch-roadmap", "author": "alice", "content": "Current priorities: 1) Multi-account support ✅  2) File uploads ✅  3) Channel management", "offset_min": 0},
        {"channel_id": "ch-roadmap", "author": "bob", "content": "What about voice channels? That's the killer feature for me.", "offset_min": 10},
        {"channel_id": "ch-roadmap", "author": "alice", "content": "Voice is Milestone 3. We need to resolve the SFU strategy first.", "offset_min": 12},
        # #feedback
        {"channel_id": "ch-feedback", "author": "bob", "content": "The UI feels snappy. The Tauri approach is paying off.", "offset_min": 0},
        {"channel_id": "ch-feedback", "author": "alice", "content": "Glad to hear it. Any rough edges you've noticed?", "offset_min": 3},
        {"channel_id": "ch-feedback", "author": "bob", "content": "Maybe some loading states when switching servers? It can feel abrupt.", "offset_min": 6},
        {"channel_id": "ch-feedback", "author": "alice", "content": "Good point. I'll add that to the backlog.", "offset_min": 8},
    ],
    "invites": [
        {"code": PRIVATE_INVITE, "created_by": "alice", "max_uses": 0},
    ],
}

STRICT_CONFIG = {
    "users": [
        {"name": "alice", "display_name": "Alice (Admin)", "roles": ["everyone", "admin"]},
        {"name": "bob", "display_name": "Bob", "roles": ["everyone"]},
    ],
    "channels": [
        # Main
        {"id": "ch-verified", "name": "verified", "topic": "Verified users only", "category": "Main"},
        {"id": "ch-security", "name": "security", "topic": "Security discussions and advisories", "category": "Main"},
        # Operations
        {"id": "ch-incidents", "name": "incidents", "topic": "Incident reports and post-mortems", "category": "Operations"},
    ],
    "messages": [
        # #verified
        {"channel_id": "ch-verified", "author": "alice", "content": "This server uses allowlist-only mode. Only pre-approved pubkeys can join.", "offset_min": 0},
        {"channel_id": "ch-verified", "author": "bob", "content": "Makes sense for high-trust environments.", "offset_min": 5},
        {"channel_id": "ch-verified", "author": "alice", "content": "Exactly. Think corporate teams, security groups, etc.", "offset_min": 8},
        {"channel_id": "ch-verified", "author": "bob", "content": "How do new users get added to the allowlist?", "offset_min": 12},
        {"channel_id": "ch-verified", "author": "alice", "content": "An admin adds their pubkey manually. No invite codes here.", "offset_min": 14},
        # #security
        {"channel_id": "ch-security", "author": "alice", "content": "Reminder: never share your private key seed. Your identity depends on it.", "offset_min": 0},
        {"channel_id": "ch-security", "author": "bob", "content": "What's the recovery story if someone loses their key?", "offset_min": 5},
        {"channel_id": "ch-security", "author": "alice", "content": "That's still an open design question. See docs/design/identity.md for options.", "offset_min": 8},
        {"channel_id": "ch-security", "author": "bob", "content": "Device sub-keys look promising. You could revoke a lost device without losing your main identity.", "offset_min": 12},
        # #incidents
        {"channel_id": "ch-incidents", "author": "alice", "content": "No incidents to report. Let's keep it that way! 🎉", "offset_min": 0},
        {"channel_id": "ch-incidents", "author": "bob", "content": "🪵 All systems nominal.", "offset_min": 5},
    ],
    "allowlist": [
        {"pubkey_user": "alice", "added_by": "alice"},
        {"pubkey_user": "bob", "added_by": "alice"},
        # charlie and dave are NOT on the allowlist.
    ],
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def print_summary():
    print("\n" + "=" * 60)
    print("TEST ENVIRONMENT READY")
    print("=" * 60)
    print("\nTest Accounts (all stored in OS keychain):")
    print("-" * 60)
    for name, user in USERS.items():
        print(f"  {name:10s}  {user['pubkey'][:20]}…")
    print()
    print("Server Layout:")
    print("-" * 60)
    print("  Open Server     (localhost:8081) — open membership")
    print("    Users: alice (admin), bob, charlie (restricted), dave")
    print("    Text Channels: #general, #random, #introductions")
    print("    Admin:         #announcements, #mod-log")
    print("    Projects:      #frontend, #backend, #design")
    print(f"    Messages: seeded across all channels")
    print()
    print("  Private Server  (localhost:8082) — invite only")
    print("    Users: alice (admin), bob")
    print(f"    Invite code: {PRIVATE_INVITE}")
    print("    charlie & dave can test joining via invite")
    print("    General:  #lobby, #members-only")
    print("    Planning: #roadmap, #feedback")
    print(f"    Messages: seeded across all channels")
    print()
    print("  Strict Server   (localhost:8083) — allowlist only")
    print("    Users: alice (admin), bob")
    print("    Allowlisted: alice, bob")
    print("    charlie & dave are NOT allowlisted")
    print("    Main:       #verified, #security")
    print("    Operations: #incidents")
    print(f"    Messages: seeded across all channels")
    print()
    print("Permissions Notes:")
    print("-" * 60)
    print("  alice   — Admin on all servers (full permissions)")
    print("  bob     — Regular member (default @everyone perms)")
    print("  charlie — 'Restricted' role on Open: send+read only")
    print("            (no file attach, no reactions, no manage)")
    print("            NOT on Private or Strict servers")
    print("  dave    — Regular on Open only; not on Private/Strict")
    print()
    print("To start servers: make dev")
    print("  or: overmind start -f Procfile (just servers)")
    print()
    print("Client localStorage pre-configured:")
    print("  alice & bob — Open + Private + Strict servers added")
    print("  charlie & dave — Open server added")
    print("  Active account: alice (switch via account switcher in client)")
    print("=" * 60)

def main():
    clean_only = "--clean" in sys.argv

    print("Cleaning test environment...")
    clean_databases()
    clean_keychain()
    clean_webview_data()

    if clean_only:
        print("\nDone (clean only).")
        return

    print("\nRunning server migrations...")
    for config in ["open", "private", "strict"]:
        run_migrations(config)

    print("\nSeeding databases...")
    seed_database("open.db", OPEN_CONFIG)
    print("  ✓ Open server seeded")
    seed_database("private.db", PRIVATE_CONFIG)
    print("  ✓ Private server seeded")
    seed_database("strict.db", STRICT_CONFIG)
    print("  ✓ Strict server seeded")

    print("\nStoring test accounts in OS keychain...")
    store_test_accounts_in_keychain()

    print("\nSeeding WebView localStorage...")
    seed_localstorage()

    print_summary()

if __name__ == "__main__":
    main()
