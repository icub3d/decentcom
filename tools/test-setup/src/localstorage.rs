use crate::clean::xdg_data_home;
use crate::seed::User;
use anyhow::Result;
use rusqlite::Connection;
use serde_json::{json, Value};

struct ServerInfo {
    id: &'static str,
    address: &'static str,
    name: &'static str,
}

const OPEN: ServerInfo = ServerInfo {
    id: "http://localhost:8081",
    address: "http://localhost:8081",
    name: "Open Server",
};
const PRIVATE: ServerInfo = ServerInfo {
    id: "http://localhost:8082",
    address: "http://localhost:8082",
    name: "Private Server",
};
const STRICT: ServerInfo = ServerInfo {
    id: "http://localhost:8083",
    address: "http://localhost:8083",
    name: "Strict Server",
};

fn server_obj(s: &ServerInfo) -> Value {
    json!({ "id": s.id, "address": s.address, "name": s.name })
}

fn enc_utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect()
}

pub fn seed(users: &[User]) -> Result<()> {
    let ls_dir = xdg_data_home()?
        .join("com.jmarsh.client")
        .join("localstorage");
    std::fs::create_dir_all(&ls_dir)?;
    let ls_db = ls_dir.join("http_localhost_1420.localstorage");

    let conn = Connection::open(&ls_db)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ItemTable \
         (key TEXT UNIQUE ON CONFLICT REPLACE, \
          value BLOB NOT NULL ON CONFLICT FAIL)",
    )?;

    // Map each account to its server list.
    let account_servers: &[(&str, &[&ServerInfo])] = &[
        ("alice", &[&OPEN, &PRIVATE, &STRICT]),
        ("bob", &[&OPEN, &PRIVATE, &STRICT]),
        ("charlie", &[&OPEN]),
        ("dave", &[&OPEN]),
    ];

    for (name, servers) in account_servers {
        let user = User::find(users, name);
        let servers_map: serde_json::Map<String, Value> = servers
            .iter()
            .map(|s| (s.id.to_string(), server_obj(s)))
            .collect();
        let state = json!({
            "state": {
                "currentServerId": OPEN.id,
                "servers": servers_map,
                "theme": "mocha",
            },
            "version": 0
        });
        let key = format!("decentcom-app-storage-{}", user.pubkey);
        let value = enc_utf16le(&state.to_string());
        conn.execute(
            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
    }

    // Set alice as the active account.
    let alice = User::find(users, "alice");
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
        rusqlite::params![
            "decentcom-active-pubkey",
            enc_utf16le(&alice.pubkey)
        ],
    )?;

    println!(
        "  Seeded localStorage for {} accounts (active: alice)",
        account_servers.len()
    );
    Ok(())
}
