# immich refresh CLI vibe coding tool

We are going to build a CLI tool that can run on a Linux AMD64. The CLI tool accepts one path argument and one optional --dry-run argument. The path argument is the starting path, it should work on. For that starting path, it should iterate other child folders. And we are going to search for folders for which we are going to run an external command. It should search for all directories below the child directory of the path that was given. And for those directories, it should get the name of the directory and use that to perform a shell execute for an external command in which the name of the directory is passed. If the name of the directory that is two levels below the path is other, then it should use the name of the parent directory instead of the directory that is two levels below the path argument. 

Example:

- Path argument: /base
- Child directories: /base/child1/grandchildA, /base/child2/grandchildB
- if grandchildA or grandchildB is other (compare case insensitive), then use the name of the parent directory of grand child as the album name instead of the grand child directory. Otherwise, the album name is the name of the grand child directory.

The external command that we're going to call is the immich photomanagement-cli-upload command. And we will pass a path to it and the name of the album to which the uploaded photos should be added. 

The execute method of the upload, command, struct, and trade has to do a shell execute with the CLI tool of immich. And then it has to check whether the results, the exit code of that command was 0, and whether there was no timeout. Stream the output of the command to stdout and stderr while it is running.

Our CLI is started as:

immich-refresh <path> --dry-run

Example command:

`/usr/src/app/cli/bin/immich upload -H -r -c 24 -A <album name> <path>/*`

This command fails if the immich CLI is not yet authenticated to the server. Make sure that the error is shown to the user.

The directories are processed sequentially. If one of the commands fails, the CLI should continue. On sigint and sigterm, the CLI should stop processing new directories immediately by killing the current immich upload command and then exit with code 130.

In dry run mode, the CLI should only log to stdout what it would do, but not actually execute the command.

Don't fall into the trap of premature performance optimization.  Add a single crate crates/refresh-cli to the workspace. Change the executable name to immich-refresh. Define all crate dependencies in the root Cargo.toml file. Add the dependencies as dependency.workspace = true to the crate dependencies. Use the latest version of all crates. Use the latest stable edition of Rust. Use concise names for functions, methods, arguments, variable names that clearly describe what it is intended to do. Don't shorten the names. Write idiomatic Rust code which is aware of the latest trends in rust up to version 1.90.

We're going to use the tracing crate and tracing subscribers to log to sdout and to a log file as well. Write the log to /var/log/immich-refresh.log, append only. Log this location at startup to stdout. If the command failed or there is any kind of error, we're going to log it with an error severity. We're not going to use Tokyo or Async in this project and prefer to use immutable variables where possible. Don't use clap, just use use std::env::args(). Use anyhow::Result for every fallible function. Use std::process::Command to execute the external command. Use pretty assertions for tests. Use rstest for tests. Use mockall for mocking the trait in the tests. Write unit tests for all functions and methods. Write integration tests that test the whole flow with a mock implementation of the trait that simulates success and failure of the external command. Use cargo make to define tasks for building, testing, and running the CLI tool. Write a Makefile.toml in the root of the workspace that defines these tasks. Write a README.md file that explains how to build, test, and run the CLI tool.

The technical requirements are that we have to build this in Rust, in a Rust workspace, which will be cross-compiled from an ARM64 development station to an AMD64 architecture. Target: x86_64-unknown-linux-gnu. We are going to add a new struct that represents the external command that will be called. That struct has one property, which is the directory name that is used in the external command.

The struct has one method called UploadCommand. And for the struct we're gonna also introduce a trait that has the execute method, which by passing the trait everywhere we can make tests that have a mock implementation for the trait so that we can test whether the execute function was called and with which arguments.
