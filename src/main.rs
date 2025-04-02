use std::{
    fs::{self, File},
    io,
};

use ::time::OffsetDateTime;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use rmatrix::{app::App, term::CrosstermTerminal};
use tracing::error;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    enable_logging().context("Failed to enable logging")?;

    if let Err(err) = run() {
        eprintln!("{:?}", err);
        error!("{}", err);
    }

    Ok(())
}

fn run() -> Result<()> {
    App::builder()
        .rng(rand::rng())
        .terminal(
            CrosstermTerminal::builder()
                .stdout(io::stdout().lock())
                .build()
                .context("Failed to setup the terminal")?,
        )
        .sequence_height_bounds(5..=14)
        .build()
        .context("Failed to create the app")?
        .run()
        .context("Failed to run the app")
}

fn enable_logging() -> Result<()> {
    let dirs = ProjectDirs::from("app.augustyniak", "magmast", "rmatrix")
        .context("Failed to get project directories")?;

    let logs_path = dirs.data_local_dir().join("logs");
    fs::create_dir_all(&logs_path).context("Failed to create logs directory")?;

    let now = OffsetDateTime::now_local().context("Failed to get local time")?;
    let log_path = logs_path.join(format!("{}.log", now));
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
