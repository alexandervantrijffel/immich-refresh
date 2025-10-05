use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ExecuteArgs {
    pub path: Box<str>,
    pub album_name: Box<str>,
    pub dry_run: bool,
}

pub trait Execute {
    fn execute(&self, args: &ExecuteArgs) -> Result<()>;
}

pub struct Executer;

impl Executer {
    pub fn new() -> Self {
        Self
    }

    fn format_command(&self, args: &ExecuteArgs) -> String {
        format!(
            "/usr/src/app/cli/bin/immich upload -H -r -c 24 -A {} {}/*",
            args.album_name, args.path
        )
    }
}

impl Execute for Executer {
    fn execute(&self, args: &ExecuteArgs) -> Result<()> {
        let command = self.format_command(args);

        if args.dry_run {
            info!("[DRY RUN] Would execute: {}", command);
        } else {
            info!("Executing: {}", command);
        }

        Ok(())
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
    #[case("/base/child1/grandchildA", "grandchildA", false)]
    #[case("/base/child2/other", "child2", true)]
    #[case("/photos/2024/vacation", "vacation", false)]
    fn test_executer_execute_success(
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
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_trait() {
        let executer: Box<dyn Execute> = Box::new(Executer::new());
        let args = ExecuteArgs {
            path: "/test/path".into(),
            album_name: "test_album".into(),
            dry_run: false,
        };

        let result = executer.execute(&args);
        assert!(result.is_ok());
    }
}
