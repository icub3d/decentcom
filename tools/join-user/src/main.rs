use anyhow::{Context, Result};
use decentcom_sdk::{rest::models::UpdateProfileRequest, Client, Identity};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let server_url = args
        .next()
        .unwrap_or_else(|| "http://localhost:8081".to_string());
    let display_name = args.next();

    let (identity, mnemonic) = Identity::generate().context("failed to generate identity")?;

    println!("New user pubkey:  {}", identity.pubkey());
    println!("Mnemonic (save to reuse this identity):");
    println!("  {mnemonic}");
    println!();
    println!("Joining {server_url}...");

    let client = Client::new(&server_url).context("failed to build client")?;
    client
        .authenticate(&identity)
        .await
        .context("authentication failed")?;

    client
        .members()
        .join()
        .await
        .context("join failed")?;

    if let Some(name) = &display_name {
        client
            .profiles()
            .update(&UpdateProfileRequest {
                display_name: Some(name.clone()),
            })
            .await
            .context("setting display name failed")?;
        println!("Joined as \"{name}\" (pubkey: {})", identity.pubkey());
    } else {
        println!("Joined (pubkey: {})", identity.pubkey());
    }

    Ok(())
}
