use crate::prelude::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const LOG_FILE_PATH: &str = "/var/log/immich-refresh.log";

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
        let file_appender = tracing_appender::rolling::never("/var/log", "immich-refresh.log");
        let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();

        info!("Logging to stdout and {}", LOG_FILE_PATH);
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
