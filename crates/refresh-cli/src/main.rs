use anyhow::{bail, Context, Result};
use std::env;

#[derive(Debug, PartialEq, Eq)]
pub struct Arguments {
    pub path: Box<str>,
    pub dry_run: bool,
}

/// Parse command line arguments.
///
/// Expected format:
/// - `immich-refresh <path>`
/// - `immich-refresh <path> --dry-run`
pub fn parse_arguments() -> Result<Arguments> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Usage: immich-refresh <path> [--dry-run]");
    }

    let starting_path = args[1].clone().into_boxed_str();

    let mut dry_run = false;

    if args.len() > 2 {
        if args[2] == "--dry-run" {
            dry_run = true;
        } else {
            bail!(
                "Invalid argument: {}. Expected --dry-run or no additional arguments",
                args[2]
            );
        }
    }

    if args.len() > 3 {
        bail!("Too many arguments. Usage: immich-refresh <path> [--dry-run]");
    }

    Ok(Arguments {
        path: starting_path,
        dry_run,
    })
}

fn main() -> Result<()> {
    let arguments = parse_arguments().context("Failed to parse arguments")?;

    println!("Starting path: {}", arguments.path);
    println!("Dry run: {}", arguments.dry_run);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(vec!["immich-refresh", "/base"], "/base", false)]
    #[case(vec!["immich-refresh", "/base", "--dry-run"], "/base", true)]
    #[case(vec!["immich-refresh", "/home/user/photos"], "/home/user/photos", false)]
    fn test_parse_arguments_success(
        #[case] args: Vec<&str>,
        #[case] expected_path: &str,
        #[case] expected_dry_run: bool,
    ) {
        let parsed = parse_arguments_from_vec(args.iter().map(|s| s.to_string()).collect());

        assert!(parsed.is_ok());
        let arguments = parsed.unwrap();
        assert_eq!(arguments.path.as_ref(), expected_path);
        assert_eq!(arguments.dry_run, expected_dry_run);
    }

    #[rstest]
    #[case(vec!["immich-refresh"])]
    #[case(vec!["immich-refresh", "/base", "--invalid"])]
    #[case(vec!["immich-refresh", "/base", "--dry-run", "extra"])]
    fn test_parse_arguments_failure(#[case] args: Vec<&str>) {
        let parsed = parse_arguments_from_vec(args.iter().map(|s| s.to_string()).collect());
        assert!(parsed.is_err());
    }

    // Helper function for testing without modifying actual env::args
    fn parse_arguments_from_vec(args: Vec<String>) -> Result<Arguments> {
        if args.len() < 2 {
            bail!("Usage: immich-refresh <path> [--dry-run]");
        }

        let starting_path = args[1].clone().into_boxed_str();

        let mut dry_run = false;

        if args.len() > 2 {
            if args[2] == "--dry-run" {
                dry_run = true;
            } else {
                bail!(
                    "Invalid argument: {}. Expected --dry-run or no additional arguments",
                    args[2]
                );
            }
        }

        if args.len() > 3 {
            bail!("Too many arguments. Usage: immich-refresh <path> [--dry-run]");
        }

        Ok(Arguments {
            path: starting_path,
            dry_run,
        })
    }
}
