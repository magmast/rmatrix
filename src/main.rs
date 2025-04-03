use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use directories::ProjectDirs;
use rmatrix::{Cli, app::App, term::CrosstermTerminal};
use time::OffsetDateTime;
use tracing::error;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let cli = Cli::parse();

    enable_logging(cli.log_file.as_ref()).context("Failed to enable logging")?;

    if let Err(err) = run(cli) {
        eprintln!("{:?}", err);
        error!("{}", err);
    }

    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    App::builder()
        .rng(rand::rng())
        .terminal(
            CrosstermTerminal::builder()
                .stdout(io::stdout().lock())
                .maybe_head_color(cli.head_color)
                .maybe_tail_color(cli.tail_color)
                .build()
                .context("Failed to setup the terminal")?,
        )
        .sequence_height_bounds(10..=32)
        .maybe_speed(cli.speed)
        .build()
        .context("Failed to create the app")?
        .run()
        .context("Failed to run the app")
}

fn enable_logging(log_file_path: Option<impl Into<PathBuf>>) -> Result<()> {
    let log_path = if let Some(path) = log_file_path {
        path.into()
    } else {
        default_log_file_path().context("Failed to get default log file path")?
    };

    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).context("Failed to create logs directory")?;
    }

    let log_file = File::options()
        .append(true)
        .create(true)
        .open(log_path)
        .context("Failed to open log file")?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .with_writer(log_file)
        .init();

    Ok(())
}

fn default_log_file_path() -> Result<PathBuf> {
    let now = OffsetDateTime::now_local().context("Failed to get local time")?;
    let path = ProjectDirs::from("app.augustyniak", "magmast", "rmatrix")
        .context("Failed to get project directories")?
        .data_local_dir()
        .join("logs")
        .join(format!("{}.log", now));
    Ok(path)
}
