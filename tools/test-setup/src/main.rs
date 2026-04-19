mod backup;
mod clean;
mod dev_assets;
mod keychain;
mod localstorage;
mod seed;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "test-setup",
    about = "Bootstrap the decentcom test environment",
    long_about = "Cleans test databases and WebView storage, injects test \
                  identities into the OS keychain, and pre-populates the \
                  Tauri WebView's localStorage with the test server list. \
                  Server data (channels, messages, etc.) is seeded by \
                  tools/sdk-seed against running servers."
)]
struct Cli {
    /// Repository root. Defaults to walking up from CWD for a directory
    /// containing both Makefile and Cargo.toml.
    #[arg(long, env = "DECENTCOM_ROOT", global = true)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Clean test state only: databases, WebView data, and keychain entries.
    Clean,

    /// Pre-stage the local environment for `sdk-seed`: clean DBs, install
    /// keychain entries, and seed the WebView's localStorage. Does NOT seed
    /// any server data (use `sdk-seed` for that, after the servers boot).
    Prepare,

    /// Generate developer assets: BIP39 mnemonic files and encrypted .dckb
    /// backup files for testing the backup/restore UI flow.
    GenDevAssets {
        /// Output directory for generated assets.
        #[arg(long, default_value = "dev-assets", env = "DECENTCOM_DEV_ASSETS_DIR")]
        out_dir: PathBuf,

        /// Passphrase used to encrypt the .dckb backup files.
        #[arg(
            long,
            default_value = "decentcom-test",
            env = "DECENTCOM_DEV_ASSETS_PASSPHRASE"
        )]
        passphrase: String,
    },
}

fn repo_root(cli_root: Option<PathBuf>) -> PathBuf {
    if let Some(r) = cli_root {
        return r;
    }
    if let Ok(r) = std::env::var("DECENTCOM_ROOT") {
        if !r.is_empty() {
            return PathBuf::from(r);
        }
    }
    let mut dir = std::env::current_dir().unwrap_or_default();
    for _ in 0..10 {
        if dir.join("Makefile").exists() && dir.join("Cargo.toml").exists() {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }
    std::env::current_dir().unwrap_or_default()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = repo_root(cli.root);

    match cli.command {
        None | Some(Command::Prepare) => prepare(&root),
        Some(Command::Clean) => run_clean(&root),
        Some(Command::GenDevAssets { out_dir, passphrase }) => {
            let out_dir = if out_dir.is_absolute() {
                out_dir
            } else {
                root.join(out_dir)
            };
            dev_assets::generate(&out_dir, &passphrase)
        }
    }
}

fn run_clean(root: &Path) -> Result<()> {
    let users = seed::all_users();
    println!("Cleaning test environment...");
    clean::clean_databases(root)?;
    clean::clean_webview_data()?;
    keychain::clean(&users)?;
    println!("\nDone (clean only).");
    Ok(())
}

fn prepare(root: &Path) -> Result<()> {
    let users = seed::all_users();

    println!("Cleaning test environment...");
    clean::clean_databases(root)?;
    clean::clean_webview_data()?;
    keychain::clean(&users)?;

    println!("\nStoring test accounts in keychain...");
    keychain::store(&users)?;

    println!("\nSeeding WebView localStorage...");
    localstorage::seed(&users)?;

    seed::print_summary(&users);
    println!("\nNext: start the servers and run `cargo run -q --bin sdk-seed` to seed");
    println!("      channels, messages, and other content via the REST API.");
    Ok(())
}
