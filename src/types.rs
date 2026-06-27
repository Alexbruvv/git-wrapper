use std::io;
use std::path::PathBuf;

use serde::Deserialize;

/// Parsed and validated contents of a project's `.gitwrapper` file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// gh username to switch to before running git.
    pub account: String,
    /// Hostname the account lives on. Defaults to "github.com".
    #[serde(default = "default_host")]
    pub host: String,
    /// When set, applied as repo-local git `user.name`.
    pub user_name: Option<String>,
    /// When set, applied as repo-local git `user.email`.
    pub user_email: Option<String>,
    /// Restore the previously active gh account after the command runs.
    #[serde(default = "default_true")]
    pub restore_previous: bool,
    /// Absolute path the config was loaded from (filled in after parsing).
    #[serde(skip)]
    pub source_path: PathBuf,
}

fn default_host() -> String {
    "github.com".to_string()
}

fn default_true() -> bool {
    true
}

/// A GitHub account known to the local `gh` CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub user: String,
    pub host: String,
    pub active: bool,
}

/// Result of running a child process to completion.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Injectable command runner so gh/git calls can be mocked in tests.
pub trait Runner {
    /// Run `cmd` to completion, capturing stdout/stderr.
    fn capture(&self, cmd: &str, args: &[&str]) -> io::Result<RunResult>;
    /// Run `cmd` with inherited stdio, returning its exit code.
    fn passthrough(&self, cmd: &str, args: &[&str]) -> io::Result<i32>;
}
