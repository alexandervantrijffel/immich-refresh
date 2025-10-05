use crate::execute::{Execute, ExecuteArgs};
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

    let entries = fs::read_dir(base_path)
        .with_context(|| format!("Failed to read directory: {}", arguments.path))?;

    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", arguments.path))?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            let directory_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

            debug!("Found child directory: {}", directory_name);

            let execute_args = ExecuteArgs {
                path: entry_path.to_string_lossy().into_owned().into_boxed_str(),
                album_name: directory_name.to_string().into_boxed_str(),
                dry_run: arguments.dry_run,
            };

            if let Err(e) = executor.execute(&execute_args) {
                error!(
                    "Failed to execute for directory {}: {}",
                    directory_name, e
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::{Execute, ExecuteArgs};
    use mockall::mock;
    use mockall::predicate::*;
    use std::fs;
    use tempfile::TempDir;

    mock! {
        pub Executor {}

        impl Execute for Executor {
            fn execute(&self, args: &ExecuteArgs) -> Result<()>;
        }
    }

    #[test]
    fn test_traverse_with_child_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create child directories
        fs::create_dir(base_path.join("child1")).unwrap();
        fs::create_dir(base_path.join("child2")).unwrap();
        fs::create_dir(base_path.join(".hidden")).unwrap();

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
            path: temp_dir.path().to_string_lossy().into_owned().into_boxed_str(),
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
        fs::create_dir(base_path.join("child2")).unwrap();

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_execute()
            .times(2)
            .returning(|args| {
                if args.album_name.contains("child1") {
                    Err(anyhow::anyhow!("Simulated error"))
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
}
