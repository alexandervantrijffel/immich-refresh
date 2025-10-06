use crate::execute::{Execute, ExecuteArgs, ExecuteError};
use crate::prelude::*;
use crate::Arguments;

pub fn traverse(arguments: &Arguments, executor: &impl Execute) -> Result<()> {
    let base_path = Path::new(arguments.path.as_ref());

    if !base_path.exists() {
        bail!("Path does not exist: {}", arguments.path);
    }

    if !base_path.is_dir() {
        bail!("Path is not a directory: {}", arguments.path);
    }

    info!("Traversing directory: {}", arguments.path);

    // Iterate over child directories
    let child_entries = fs::read_dir(base_path)
        .with_context(|| format!("Failed to read directory: {}", arguments.path))?;

    for child_entry in child_entries {
        let child_entry =
            child_entry.with_context(|| format!("Failed to read entry in {}", arguments.path))?;
        let child_path = child_entry.path();

        if !child_path.is_dir() {
            continue;
        }

        let child_name = child_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid child directory name"))?;

        debug!("Found child directory: {}", child_name);

        // Iterate over grandchild directories
        let grandchild_entries = fs::read_dir(&child_path)
            .with_context(|| format!("Failed to read child directory: {}", child_path.display()))?;

        for grandchild_entry in grandchild_entries {
            let grandchild_entry = grandchild_entry
                .with_context(|| format!("Failed to read entry in {}", child_path.display()))?;
            let grandchild_path = grandchild_entry.path();

            if !grandchild_path.is_dir() {
                continue;
            }

            let grandchild_name = grandchild_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid grandchild directory name"))?;

            debug!("Found grandchild directory: {}", grandchild_name);

            // Determine album name: use grandchild name unless it's "other" (case-insensitive)
            let album_name = if grandchild_name.eq_ignore_ascii_case("other") {
                child_name.to_string()
            } else {
                grandchild_name.to_string()
            };

            let execute_args = ExecuteArgs {
                path: grandchild_path
                    .to_string_lossy()
                    .into_owned()
                    .into_boxed_str(),
                album_name: album_name.into_boxed_str(),
                dry_run: arguments.dry_run,
            };

            if let Err(e) = executor.execute(&execute_args) {
                match e {
                    ExecuteError::AuthFailed(ref msg) => {
                        error!("Authentication failed: {}", msg);
                        bail!("Aborting due to authentication failure");
                    }
                    ExecuteError::ImmichCliNotFound(ref path) => {
                        error!("Immich CLI not found at {}", path);
                        bail!("Aborting because Immich CLI is not installed");
                    }
                    ExecuteError::Other(ref err) => {
                        error!(
                            "Failed to execute for directory {}: {}",
                            grandchild_name, err
                        );
                        // Continue processing other directories
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::{Execute, ExecuteArgs, ExecuteError};
    use mockall::mock;
    use mockall::predicate::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::TempDir;

    mock! {
        pub Executor {}

        impl Execute for Executor {
            fn execute(&self, args: &ExecuteArgs) -> Result<(), ExecuteError>;
        }
    }

    #[test]
    fn test_traverse_with_grandchild_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create child/grandchild structure
        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildA")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildB")).unwrap();

        fs::create_dir(base_path.join("child2")).unwrap();
        fs::create_dir(base_path.join("child2/grandchildC")).unwrap();

        // Create a file (should be ignored)
        fs::write(base_path.join("file.txt"), "test").unwrap();

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_execute()
            .times(3)
            .returning(|_| Ok(()));

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_traverse_nonexistent_path() {
        let mock_executor = MockExecutor::new();

        let arguments = Arguments {
            path: "/nonexistent/path".into(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_err());
    }

    #[test]
    fn test_traverse_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let mock_executor = MockExecutor::new();

        let arguments = Arguments {
            path: file_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_err());
    }

    #[test]
    fn test_traverse_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let mock_executor = MockExecutor::new();

        let arguments = Arguments {
            path: temp_dir
                .path()
                .to_string_lossy()
                .into_owned()
                .into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_traverse_continues_on_executor_error() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildA")).unwrap();

        fs::create_dir(base_path.join("child2")).unwrap();
        fs::create_dir(base_path.join("child2/grandchildB")).unwrap();

        let mut mock_executor = MockExecutor::new();
        mock_executor.expect_execute().times(2).returning(|args| {
            if args.album_name.contains("grandchildA") {
                Err(ExecuteError::Other(anyhow::anyhow!("Simulated error")))
            } else {
                Ok(())
            }
        });

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_traverse_aborts_on_auth_failure() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildA")).unwrap();

        fs::create_dir(base_path.join("child2")).unwrap();
        fs::create_dir(base_path.join("child2/grandchildB")).unwrap();

        let mut mock_executor = MockExecutor::new();
        // First call should fail with auth error
        mock_executor.expect_execute().times(1).returning(|_| {
            Err(ExecuteError::AuthFailed(
                "Authentication required".to_string(),
            ))
        });

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("authentication failure"));
    }

    #[test]
    fn test_traverse_aborts_on_immich_cli_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildA")).unwrap();

        let mut mock_executor = MockExecutor::new();
        // First call should fail with CLI not found error
        mock_executor.expect_execute().times(1).returning(|_| {
            Err(ExecuteError::ImmichCliNotFound(
                "'immich' command not found in PATH".to_string(),
            ))
        });

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Immich CLI is not installed"));
    }

    #[test]
    fn test_traverse_other_directory_uses_parent_name() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create structure with "other" grandchild
        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child1/other")).unwrap();
        fs::create_dir(base_path.join("child1/Other")).unwrap();
        fs::create_dir(base_path.join("child1/grandchildA")).unwrap();

        let mut mock_executor = MockExecutor::new();

        // Both "other" and "Other" should use parent name "child1"
        mock_executor.expect_execute().times(3).returning(|args| {
            // Verify album names
            let path_str = args.path.as_ref();
            if path_str.contains("/other") || path_str.contains("/Other") {
                assert_eq!(args.album_name.as_ref(), "child1");
            } else if path_str.contains("/grandchildA") {
                assert_eq!(args.album_name.as_ref(), "grandchildA");
            }
            Ok(())
        });

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: false,
        };

        let result = traverse(&arguments, &mock_executor);
        assert!(result.is_ok());
    }
}
