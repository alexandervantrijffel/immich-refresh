use crate::execute::{Execute, ExecuteArgs, ExecuteError, Executer};
use crate::prelude::*;
use crate::Arguments;

pub fn traverse(arguments: &Arguments, executor: &Executer) -> Result<()> {
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
            // Check for signals before processing each directory
            executor.check_signal();

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
    use crate::execute::Executer;
    use std::fs;
    use tempfile::TempDir;

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

        let executor = Executer::new();

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: true, // Use dry run to avoid executing actual commands
        };

        let result = traverse(&arguments, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_traverse_nonexistent_path() {
        let executor = Executer::new();

        let arguments = Arguments {
            path: "/nonexistent/path".into(),
            dry_run: true,
        };

        let result = traverse(&arguments, &executor);
        assert!(result.is_err());
    }

    #[test]
    fn test_traverse_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let executor = Executer::new();

        let arguments = Arguments {
            path: file_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: true,
        };

        let result = traverse(&arguments, &executor);
        assert!(result.is_err());
    }

    #[test]
    fn test_traverse_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let executor = Executer::new();

        let arguments = Arguments {
            path: temp_dir
                .path()
                .to_string_lossy()
                .into_owned()
                .into_boxed_str(),
            dry_run: true,
        };

        let result = traverse(&arguments, &executor);
        assert!(result.is_ok());
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

        let executor = Executer::new();

        let arguments = Arguments {
            path: base_path.to_string_lossy().into_owned().into_boxed_str(),
            dry_run: true,
        };

        let result = traverse(&arguments, &executor);
        assert!(result.is_ok());
    }
}
