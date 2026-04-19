mod data;
mod seeder;
mod users;

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use crate::seeder::SeedReport;

#[derive(Parser, Debug)]
#[command(
    name = "sdk-seed",
    about = "Seed the decentcom test environment via the SDK (REST API)",
    long_about = "Connects to the running test servers (open, private, strict) and \
                  populates them with users, channels, messages, threads, reactions, \
                  invites, allowlist entries, and bot approvals using the public \
                  decentcom-sdk. Replaces the legacy raw-SQL seeding."
)]
struct Cli {
    /// Maximum time, in seconds, to wait for each server's `/health` endpoint
    /// to respond before giving up.
    #[arg(long, default_value_t = 30, env = "DECENTCOM_SEED_HEALTH_TIMEOUT")]
    health_timeout_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    let users = users::all_users();
    let servers = data::all_servers();

    println!("Waiting for servers to become healthy...");
    for cfg in servers {
        wait_for_health(cfg.base_url, Duration::from_secs(cli.health_timeout_secs))
            .await
            .with_context(|| format!("server {} did not become healthy", cfg.display_name))?;
        println!("  ✓ {} ready ({})", cfg.display_name, cfg.base_url);
    }

    println!("\nSeeding via SDK...");
    let mut total = SeedReport::default();
    for cfg in servers {
        println!("\n→ {} ({})", cfg.display_name, cfg.base_url);
        let report = seeder::seed_server(cfg, &users)
            .await
            .with_context(|| format!("seeding {} failed", cfg.display_name))?;
        report.print_indented("  ");
        total = total.merge(&report);
    }

    println!("\n{}", "=".repeat(60));
    println!("SDK SEEDING COMPLETE");
    println!("{}", "=".repeat(60));
    total.print_indented("  ");
    Ok(())
}

async fn wait_for_health(base_url: &str, timeout: Duration) -> Result<()> {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                if std::time::Instant::now() >= deadline {
                    return Err(anyhow!(
                        "{} did not respond with 2xx within {:?}",
                        url,
                        timeout
                    ));
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    }
}
