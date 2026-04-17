use crate::backup;
use anyhow::Result;
use bip39::{Language, Mnemonic};
use ed25519_dalek::SigningKey;
use serde_json::json;
use std::path::Path;

// Fixed 32-byte entropy values per dev account. These are distinct from the
// test-DB trivial seeds ([0x01; 32] etc.) — BIP39 PBKDF2 derivation means the
// resulting Ed25519 keys are different. These accounts are for testing the
// backup / restore UI flow, not for the pre-seeded server databases.
const DEV_ACCOUNTS: &[([u8; 32], &str)] = &[
    ([0x01; 32], "alice"),
    ([0x02; 32], "bob"),
    ([0x03; 32], "charlie"),
    ([0x04; 32], "dave"),
];

struct DevAccount {
    #[allow(dead_code)]
    name: &'static str,
    mnemonic: Mnemonic,
    seed: [u8; 32],
    pubkey: String,
}

impl DevAccount {
    fn from_entropy(entropy: [u8; 32], name: &'static str) -> Result<Self> {
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| anyhow::anyhow!("bip39 error for {name}: {e:?}"))?;
        let bip39_seed = mnemonic.to_seed("");
        let seed: [u8; 32] = bip39_seed[..32]
            .try_into()
            .map_err(|_| anyhow::anyhow!("seed slice length mismatch"))?;
        let signing_key = SigningKey::from_bytes(&seed);
        let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        Ok(Self { name, mnemonic, seed, pubkey })
    }
}

/// Servers that Alice/Bob join — matches what `localstorage::seed` writes.
fn server_list_metadata(accounts: &[&str]) -> String {
    let all_servers = [
        ("http://localhost:8081", "Open Server"),
        ("http://localhost:8082", "Private Server"),
        ("http://localhost:8083", "Strict Server"),
    ];

    // alice & bob get all three; charlie & dave only get Open.
    let servers: Vec<_> = if accounts.contains(&"alice") || accounts.contains(&"bob") {
        all_servers.to_vec()
    } else {
        all_servers[..1].to_vec()
    };

    let servers_map: serde_json::Map<String, serde_json::Value> = servers
        .iter()
        .map(|(addr, name)| {
            (
                addr.to_string(),
                json!({ "id": addr, "address": addr, "name": name }),
            )
        })
        .collect();

    json!({
        "version": 1,
        "servers": servers_map,
        "theme": "mocha"
    })
    .to_string()
}

pub fn generate(out_dir: &Path, passphrase: &str) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;

    println!("Generating dev assets in {}…", out_dir.display());
    println!("  Passphrase for .dckb files: {passphrase}");
    println!("  NOTE: these accounts differ from the test-DB accounts.");
    println!("  They use BIP39 derivation and are for testing the backup/restore UI.\n");

    for &(entropy, name) in DEV_ACCOUNTS {
        let acct = DevAccount::from_entropy(entropy, name)?;

        // Write mnemonic text file.
        let mnemonic_path = out_dir.join(format!("{name}-mnemonic.txt"));
        std::fs::write(&mnemonic_path, acct.mnemonic.to_string())?;

        // Build server-list metadata.
        let metadata = server_list_metadata(&[name]);

        // Encrypt backup.
        let backup_bytes =
            backup::encrypt_key(&acct.seed, passphrase, Some(&metadata)).map_err(|e| {
                anyhow::anyhow!("backup encryption failed for {name}: {e}")
            })?;
        let backup_path = out_dir.join(format!("{name}.dckb"));
        std::fs::write(&backup_path, &backup_bytes)?;

        println!("  {} (pubkey: {}…)", name, &acct.pubkey[..16]);
        println!("    mnemonic → {}", mnemonic_path.display());
        println!("    backup   → {}", backup_path.display());
    }

    println!("\nDone. Import a .dckb file via the client's \"Restore from Backup\" flow.");
    println!("Use passphrase: {passphrase}");
    Ok(())
}
