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

    # Insert channels.
    for i, ch in enumerate(config.get("channels", [])):
        c.execute(
            "INSERT OR IGNORE INTO channels (id, name, topic, category, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (ch["id"], ch["name"], ch.get("topic"), ch.get("category"), i, ts, ts),
        )

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
        {"id": "ch-general", "name": "general", "topic": "General discussion", "category": "Text Channels"},
        {"id": "ch-random", "name": "random", "topic": "Off-topic", "category": "Text Channels"},
        {"id": "ch-announcements", "name": "announcements", "topic": "Important updates", "category": "Text Channels"},
    ],
}

PRIVATE_CONFIG = {
    "users": [
        {"name": "alice", "display_name": "Alice (Admin)", "roles": ["everyone", "admin"]},
        {"name": "bob", "display_name": "Bob", "roles": ["everyone"]},
        # charlie and dave are NOT members — they can test joining via invite.
    ],
    "channels": [
        {"id": "ch-lobby", "name": "lobby", "topic": "Welcome!", "category": "General"},
        {"id": "ch-members-only", "name": "members-only", "topic": "For members", "category": "General"},
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
        {"id": "ch-verified", "name": "verified", "topic": "Verified users only", "category": "Main"},
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
    print("    Channels: #general, #random, #announcements")
    print()
    print("  Private Server  (localhost:8082) — invite only")
    print("    Users: alice (admin), bob")
    print(f"    Invite code: {PRIVATE_INVITE}")
    print("    charlie & dave can test joining via invite")
    print("    Channels: #lobby, #members-only")
    print()
    print("  Strict Server   (localhost:8083) — allowlist only")
    print("    Users: alice (admin), bob")
    print("    Allowlisted: alice, bob")
    print("    charlie & dave are NOT allowlisted")
    print("    Channels: #verified")
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
