use crate::seed::User;
use anyhow::Result;
use keyring::Entry;

const SERVICE: &str = "decentcom";

fn entry(username: &str) -> Result<Entry> {
    Entry::new(SERVICE, username).map_err(|e| anyhow::anyhow!("keyring entry error: {e}"))
}

pub fn clean(users: &[User]) -> Result<()> {
    // Read accounts_index to find per-account entries to delete.
    if let Ok(e) = entry("accounts_index") {
        if let Ok(index_json) = e.get_password() {
            let pubkeys: Vec<String> = serde_json::from_str(&index_json).unwrap_or_default();
            for pk in &pubkeys {
                for prefix in &["seed_", "label_"] {
                    let key = format!("{prefix}{pk}");
                    if let Ok(e2) = entry(&key) {
                        let _ = e2.delete_credential();
                    }
                }
                println!("  Removed keychain entries for {}…", &pk[..pk.len().min(12)]);
            }
        }
    }

    // Also attempt to clean entries by known user pubkeys (belt-and-suspenders).
    for user in users {
        for prefix in &["seed_", "label_"] {
            let key = format!("{prefix}{}", user.pubkey);
            if let Ok(e2) = entry(&key) {
                let _ = e2.delete_credential();
            }
        }
    }

    // Delete the index and legacy entries.
    for name in &["accounts_index", "master_privkey_seed", "master_privkey"] {
        if let Ok(e) = entry(name) {
            let _ = e.delete_credential();
        }
    }
    println!("  Cleared keychain entries");
    Ok(())
}

pub fn store(users: &[User]) -> Result<()> {
    let pubkeys: Vec<&str> = users.iter().map(|u| u.pubkey.as_str()).collect();
    let index_json = serde_json::to_string(&pubkeys)?;

    entry("accounts_index")?.set_password(&index_json)?;
    println!("  Stored accounts_index with {} accounts", pubkeys.len());

    for user in users {
        entry(&format!("seed_{}", user.pubkey))?.set_password(&user.hex_seed)?;
        entry(&format!("label_{}", user.pubkey))?.set_password(&capitalize(user.name))?;
        println!(
            "  Stored seed + label for {} ({}…)",
            user.name,
            &user.pubkey[..user.pubkey.len().min(12)]
        );
    }
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
