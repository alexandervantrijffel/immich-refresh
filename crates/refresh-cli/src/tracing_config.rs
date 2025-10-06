use crate::prelude::*;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn get_log_file_path() -> Result<PathBuf> {
    let home_dir = env::var("HOME").context("Failed to get HOME environment variable")?;

    let log_dir = Path::new(&home_dir)
        .join(".local")
        .join("state")
        .join("immich-refresh");

    let log_file_path = log_dir.join("run.log");

    Ok(log_file_path)
}

pub fn configure(dry_run: bool) -> Result<()> {
    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if dry_run {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .init();

        info!("[DRY RUN] Logging to stdout only (file logging disabled)");
    } else {
        let log_file_path = get_log_file_path()?;
        let log_dir = log_file_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid log file path"))?;

        // Create the log directory if it doesn't exist
        fs::create_dir_all(log_dir)
            .with_context(|| format!("Failed to create log directory at {}", log_dir.display()))?;

        // Test if we can write to the log file location before initializing the appender
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .with_context(|| {
                format!(
                    "Failed to initialize log file at {}. Check permissions.",
                    log_file_path.display()
                )
            })?;

        let file_appender = tracing_appender::rolling::never(log_dir, "run.log");
        let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();

        info!("Logging to stdout and {}", log_file_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_dry_run() {
        // This test verifies that configure doesn't panic in dry-run mode
        // We can't easily test the actual tracing output without complex setup
        let result = configure(true);
        // The function should succeed even if we can't write to /var/log
        assert!(result.is_ok());
    }
}
