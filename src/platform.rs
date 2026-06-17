use std::{process::Command, thread, time::Duration};

use anyhow::{bail, Context, Result};

pub trait System {
    fn open_app(&self, name: &str) -> Result<()>;
    fn open_url(&self, url: &str) -> Result<()>;
    fn run_command(&self, cmd: &str) -> Result<()>;
    fn quit_app(&self, name: &str, timeout_secs: u64) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
}

pub struct MacOsSystem;

impl System for MacOsSystem {
    fn open_app(&self, name: &str) -> Result<()> {
        run_checked(Command::new("/usr/bin/open").arg("-a").arg(name))
            .with_context(|| format!("failed to open app {name}"))
    }

    fn open_url(&self, url: &str) -> Result<()> {
        run_checked(Command::new("/usr/bin/open").arg(url))
            .with_context(|| format!("failed to open URL {url}"))
    }

    fn run_command(&self, cmd: &str) -> Result<()> {
        run_checked(Command::new("/bin/zsh").arg("-lc").arg(cmd))
            .with_context(|| format!("failed to run command {cmd}"))
    }

    fn quit_app(&self, name: &str, timeout_secs: u64) -> Result<()> {
        let script = format!("tell application {} to quit", apple_script_string(name));
        run_checked(Command::new("/usr/bin/osascript").arg("-e").arg(script))
            .with_context(|| format!("failed to ask {name} to quit"))?;

        if timeout_secs > 0 {
            thread::sleep(Duration::from_secs(timeout_secs));
        }

        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        run_checked(
            Command::new("/usr/bin/osascript")
                .arg("-e")
                .arg("tell application \"System Events\" to shut down"),
        )
        .context("failed to shut down through System Events")
    }
}

fn run_checked(command: &mut Command) -> Result<()> {
    let output = command.output().context("failed to spawn process")?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    bail!(
        "command exited with status {}. stdout: {} stderr: {}",
        output.status,
        stdout.trim(),
        stderr.trim()
    );
}

fn apple_script_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_apple_script_strings() {
        assert_eq!(apple_script_string("A \"B\""), "\"A \\\"B\\\"\"");
        assert_eq!(apple_script_string("A\\B"), "\"A\\\\B\"");
    }
}
