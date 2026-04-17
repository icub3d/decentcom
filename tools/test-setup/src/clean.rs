use anyhow::Result;
use std::path::Path;

const DB_FILES: &[&str] = &["open.db", "private.db", "strict.db"];
const MEDIA_DIRS: &[&str] = &["media-open", "media-private", "media-strict"];

pub fn clean_databases(root: &Path) -> Result<()> {
    let configs = root.join("test-configs");
    for db in DB_FILES {
        for suffix in &["", "-shm", "-wal"] {
            let p = configs.join(format!("{db}{suffix}"));
            if p.exists() {
                std::fs::remove_file(&p)?;
                println!("  Removed {}", p.strip_prefix(root).unwrap_or(&p).display());
            }
        }
    }
    for dir in MEDIA_DIRS {
        let p = configs.join(dir);
        if p.exists() {
            std::fs::remove_dir_all(&p)?;
            println!("  Removed {}/", p.strip_prefix(root).unwrap_or(&p).display());
        }
        std::fs::create_dir_all(&p)?;
    }
    Ok(())
}

pub fn clean_webview_data() -> Result<()> {
    let data_dir = webview_data_dir()?;
    if data_dir.exists() {
        std::fs::remove_dir_all(&data_dir)?;
        println!("  Removed Tauri WebView data ({})", data_dir.display());
    } else {
        println!("  No WebView data to clean");
    }
    Ok(())
}

pub fn webview_data_dir() -> Result<std::path::PathBuf> {
    let base = xdg_data_home()?;
    Ok(base.join("com.jmarsh.client"))
}

pub fn xdg_data_home() -> Result<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        if !dir.is_empty() {
            return Ok(std::path::PathBuf::from(dir));
        }
    }
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(std::path::PathBuf::from(home).join(".local").join("share"))
}
