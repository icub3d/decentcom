use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn run_migrations(root: &Path, config_name: &str) -> Result<()> {
    let config_path = root.join("test-configs").join(format!("{config_name}.toml"));
    println!("  Running migrations for {config_name}...");

    let mut child = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "server",
            "--",
            "--config",
            config_path.to_str().unwrap_or_default(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .current_dir(root)
        .spawn()?;

    std::thread::sleep(Duration::from_secs(3));

    // Try to terminate gracefully, then force kill on timeout.
    terminate_with_timeout(&mut child, Duration::from_secs(5))?;
    Ok(())
}

fn terminate_with_timeout(child: &mut std::process::Child, timeout: Duration) -> Result<()> {
    // Signal the process to terminate.
    send_term(child)?;

    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait()? {
            Some(_) => return Ok(()),
            None if std::time::Instant::now() >= deadline => {
                child.kill()?;
                child.wait()?;
                return Ok(());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

#[cfg(unix)]
fn send_term(child: &mut std::process::Child) -> Result<()> {
    use std::io;
    let ret = libc_sigterm(child.id() as i32);
    if ret == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "kill(SIGTERM) failed: {}",
            io::Error::last_os_error()
        ))
    }
}

#[cfg(not(unix))]
fn send_term(child: &mut std::process::Child) -> Result<()> {
    child.kill()?;
    Ok(())
}

#[cfg(unix)]
fn libc_sigterm(pid: i32) -> i32 {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    // SIGTERM = 15 on all Unix-like systems.
    unsafe { kill(pid, 15) }
}
