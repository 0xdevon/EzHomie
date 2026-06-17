mod config;
mod planner;
mod platform;

use std::{
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::{
    config::{Config, DEFAULT_CONFIG},
    planner::{build_off_plan, build_on_plan, Action},
    platform::{MacOsSystem, System},
};

#[derive(Debug, Parser)]
#[command(name = "ezhomie", version, about = "Start or end your macOS workday.")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Open the configured work apps, URLs, and commands in order.
    On(RunArgs),
    /// Quit configured work apps and optionally shut down the Mac.
    Off(OffArgs),
    /// Show the resolved config path and configured target counts.
    Status,
    /// Manage EzHomie configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Args)]
struct RunArgs {
    /// Print the action plan without launching anything.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct OffArgs {
    /// Print the action plan without quitting apps or shutting down.
    #[arg(long)]
    dry_run: bool,

    /// Skip the confirmation prompt and countdown.
    #[arg(long, short = 'y')]
    yes: bool,

    /// Quit apps but do not shut down.
    #[arg(long)]
    no_shutdown: bool,
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    /// Create a default TOML config file.
    Init {
        /// Overwrite an existing config file.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = config::resolve_config_path(cli.config)?;

    match cli.command {
        Commands::On(args) => {
            let config = Config::load(&config_path)?;
            let plan = build_on_plan(&config);
            run_plan(&plan, args.dry_run, &MacOsSystem)
        }
        Commands::Off(args) => {
            let mut config = Config::load(&config_path)?;
            if args.no_shutdown {
                config.off.shutdown = false;
            }

            let plan = build_off_plan(&config);
            run_plan(&plan, args.dry_run, &MacOsSystem)?;

            if args.dry_run {
                if config.off.shutdown {
                    println!(
                        "shutdown: enabled after {}s confirmation",
                        config.off.confirm_seconds
                    );
                } else {
                    println!("shutdown: disabled");
                }
                return Ok(());
            }

            if config.off.shutdown {
                confirm_shutdown(config.off.confirm_seconds, args.yes)?;
                MacOsSystem.shutdown()?;
            }

            Ok(())
        }
        Commands::Status => {
            let config = Config::load(&config_path)?;
            println!("Config: {}", config_path.display());
            println!("Apps: {}", config.apps.len());
            println!("URLs: {}", config.urls.len());
            println!("Commands: {}", config.commands.len());
            println!("Shutdown on off: {}", config.off.shutdown);
            println!("Confirm seconds: {}", config.off.confirm_seconds);
            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Init { force } => init_config(&config_path, force),
        },
    }
}

fn init_config(path: &PathBuf, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "config already exists at {}. Use --force to overwrite it.",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    std::fs::write(path, DEFAULT_CONFIG)
        .with_context(|| format!("failed to write config {}", path.display()))?;
    println!("Created config: {}", path.display());
    Ok(())
}

fn run_plan<S: System>(plan: &[Action], dry_run: bool, system: &S) -> Result<()> {
    if plan.is_empty() {
        println!("Nothing to do.");
        return Ok(());
    }

    for action in plan {
        println!("{}", action.describe());
        if dry_run {
            continue;
        }

        match action {
            Action::OpenApp { name, .. } => system.open_app(name)?,
            Action::OpenUrl { url, .. } => system.open_url(url)?,
            Action::RunCommand { cmd, .. } => system.run_command(cmd)?,
            Action::QuitApp { name, timeout_secs } => system.quit_app(name, *timeout_secs)?,
        }

        if action.delay_ms() > 0 {
            thread::sleep(Duration::from_millis(action.delay_ms()));
        }
    }

    Ok(())
}

fn confirm_shutdown(confirm_seconds: u64, yes: bool) -> Result<()> {
    if yes {
        return Ok(());
    }

    println!(
        "EzHomie will shut down this Mac in {confirm_seconds} seconds. Press Ctrl+C to cancel."
    );
    for remaining in (1..=confirm_seconds).rev() {
        print!("\rShutting down in {remaining}s ");
        io::stdout().flush()?;
        thread::sleep(Duration::from_secs(1));
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RecordingSystem;

    impl System for RecordingSystem {
        fn open_app(&self, _name: &str) -> Result<()> {
            panic!("dry-run should not open apps");
        }

        fn open_url(&self, _url: &str) -> Result<()> {
            panic!("dry-run should not open URLs");
        }

        fn run_command(&self, _cmd: &str) -> Result<()> {
            panic!("dry-run should not run commands");
        }

        fn quit_app(&self, _name: &str, _timeout_secs: u64) -> Result<()> {
            panic!("dry-run should not quit apps");
        }

        fn shutdown(&self) -> Result<()> {
            panic!("dry-run should not shut down");
        }
    }

    #[test]
    fn dry_run_does_not_call_system() {
        let config = Config::default();
        let on_plan = build_on_plan(&config);
        let off_plan = build_off_plan(&config);

        run_plan(&on_plan, true, &RecordingSystem).unwrap();
        run_plan(&off_plan, true, &RecordingSystem).unwrap();
    }
}
