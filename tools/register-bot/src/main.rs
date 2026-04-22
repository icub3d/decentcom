use anyhow::{Context, Result};
use decentcom_sdk::{Client, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    let server_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://localhost:8081".to_string());

    let (identity, mnemonic) = Identity::generate().context("failed to generate identity")?;

    println!("Bot pubkey:  {}", identity.pubkey());
    println!("Bot mnemonic (save this to re-use the identity):");
    println!("  {}", mnemonic);
    println!();
    println!("Registering with {}...", server_url);

    let client = Client::new(&server_url).context("failed to build client")?;
    client
        .authenticate_as_bot(&identity)
        .await
        .context("bot authentication failed")?;

    println!("Done. The bot is now pending approval on the server.");
    println!("Open Server Settings → Pending Bots to approve or reject it.");

    Ok(())
}
