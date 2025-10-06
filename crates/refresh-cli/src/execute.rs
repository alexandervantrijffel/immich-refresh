use crate::prelude::*;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

const IMMICH_CLI_PATH: &str = "/usr/src/app/cli/bin/immich";

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Immich CLI not found at {0}")]
    ImmichCliNotFound(String),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct ExecuteArgs {
    pub path: Box<str>,
    pub album_name: Box<str>,
    pub dry_run: bool,
}

pub trait Execute {
    fn execute(&self, args: &ExecuteArgs) -> Result<(), ExecuteError>;
}

pub struct Executer {
    signal_received: Arc<AtomicBool>,
}

impl Executer {
    pub fn new() -> Self {
        let signal_received = Arc::new(AtomicBool::new(false));

        // Set up signal handlers for SIGINT and SIGTERM
        let signal_flag = signal_received.clone();
        if let Err(e) = flag::register(SIGINT, signal_flag.clone()) {
            error!("Failed to register SIGINT handler: {}", e);
        }

        let signal_flag = signal_received.clone();
        if let Err(e) = flag::register(SIGTERM, signal_flag) {
            error!("Failed to register SIGTERM handler: {}", e);
        }

        Self { signal_received }
    }

    fn format_command(&self, args: &ExecuteArgs) -> String {
        format!(
            "{} upload -H -r -c 24 -A {} {}/*",
            IMMICH_CLI_PATH, args.album_name, args.path
        )
    }

    fn check_immich_cli_exists() -> Result<(), ExecuteError> {
        let cli_path = Path::new(IMMICH_CLI_PATH);
        if !cli_path.exists() {
            return Err(ExecuteError::ImmichCliNotFound(IMMICH_CLI_PATH.to_string()));
        }
        Ok(())
    }

    fn execute_command(&self, command_str: &str) -> Result<(), ExecuteError> {
        // Parse command into program and arguments
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ExecuteError::Other(anyhow::anyhow!("Empty command")));
        }

        let program = parts[0];
        let args = &parts[1..];

        info!("Executing: {}", command_str);

        // Spawn the process with piped stdout and stderr
        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ExecuteError::Other(anyhow::anyhow!(
                    "Failed to spawn command '{}': {}",
                    command_str,
                    e
                ))
            })?;

        // Get stdout and stderr handles
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ExecuteError::Other(anyhow::anyhow!("Failed to capture stdout")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ExecuteError::Other(anyhow::anyhow!("Failed to capture stderr")))?;

        // Create buffered readers
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Read and stream stdout
        let stdout_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stdout_reader.lines().map_while(Result::ok) {
                println!("{}", line);
                lines.push(line);
            }
            lines
        });

        // Read and stream stderr
        let stderr_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stderr_reader.lines().map_while(Result::ok) {
                eprintln!("{}", line);
                lines.push(line);
            }
            lines
        });

        // Check for signals while waiting for the process
        loop {
            if self.signal_received.load(Ordering::Relaxed) {
                // Signal received, kill the child process
                info!("Signal received, terminating command");
                if let Err(e) = child.kill() {
                    error!("Failed to kill child process: {}", e);
                }
                std::process::exit(130);
            }

            // Check if process has exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process has exited
                    let output_lines = stdout_handle.join().unwrap_or_default();
                    let error_lines = stderr_handle.join().unwrap_or_default();

                    // Log output to logfile
                    for line in &output_lines {
                        info!("stdout: {}", line);
                    }
                    for line in &error_lines {
                        error!("stderr: {}", line);
                    }

                    if !status.success() {
                        let exit_code = status.code().unwrap_or(-1);

                        // Check for authentication error
                        let auth_error = error_lines.iter().any(|line| {
                            line.to_lowercase().contains("auth")
                                || line.to_lowercase().contains("login")
                                || line.to_lowercase().contains("credential")
                                || line.to_lowercase().contains("unauthorized")
                        });

                        if auth_error {
                            let error_message = format!(
                                "Immich CLI authentication required. Exit code: {}. Please authenticate first.",
                                exit_code
                            );
                            error!("{}", error_message);
                            return Err(ExecuteError::AuthFailed(error_message));
                        } else {
                            let error_message = format!(
                                "Command '{}' failed with exit code {}",
                                command_str, exit_code
                            );
                            error!("{}", error_message);
                            return Err(ExecuteError::Other(anyhow::anyhow!(error_message)));
                        }
                    }

                    info!("Command completed successfully");
                    return Ok(());
                }
                Ok(None) => {
                    // Process still running, sleep briefly
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(ExecuteError::Other(anyhow::anyhow!(
                        "Error waiting for process: {}",
                        e
                    )));
                }
            }
        }
    }
}

impl Execute for Executer {
    fn execute(&self, args: &ExecuteArgs) -> Result<(), ExecuteError> {
        // Check if Immich CLI exists before proceeding
        Self::check_immich_cli_exists()?;

        let command = self.format_command(args);

        if args.dry_run {
            info!("[DRY RUN] Would execute: {}", command);
            Ok(())
        } else {
            self.execute_command(&command)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[test]
    fn test_format_command() {
        let executer = Executer::new();
        let args = ExecuteArgs {
            path: "/base/child1/grandchildA".into(),
            album_name: "grandchildA".into(),
            dry_run: false,
        };

        let command = executer.format_command(&args);
        assert_eq!(
            command,
            "/usr/src/app/cli/bin/immich upload -H -r -c 24 -A grandchildA /base/child1/grandchildA/*"
        );
    }

    #[rstest]
    #[case("/base/child1/grandchildA", "grandchildA", true)]
    #[case("/base/child2/other", "child2", true)]
    #[case("/photos/2024/vacation", "vacation", true)]
    fn test_executer_execute_dry_run(
        #[case] path: &str,
        #[case] album_name: &str,
        #[case] dry_run: bool,
    ) {
        let executer = Executer::new();
        let args = ExecuteArgs {
            path: path.into(),
            album_name: album_name.into(),
            dry_run,
        };

        let result = executer.execute(&args);
        // Should fail with ImmichCliNotFound since CLI doesn't exist in test environment
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecuteError::ImmichCliNotFound(_)
        ));
    }

    #[test]
    fn test_execute_trait() {
        let executer: Box<dyn Execute> = Box::new(Executer::new());
        let args = ExecuteArgs {
            path: "/test/path".into(),
            album_name: "test_album".into(),
            dry_run: true,
        };

        let result = executer.execute(&args);
        // Should fail with ImmichCliNotFound since CLI doesn't exist in test environment
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecuteError::ImmichCliNotFound(_)
        ));
    }

    #[test]
    fn test_check_immich_cli_not_found() {
        let result = Executer::check_immich_cli_exists();
        // Should fail since CLI doesn't exist in test environment
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecuteError::ImmichCliNotFound(_)
        ));
    }
}
