use std::collections::HashMap;

use serde::Deserialize;

use crate::types::{Account, Runner};

/// Raised when a `gh` invocation fails in a way the user must act on.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct GhError(pub String);

/// Thin, mockable wrapper around the `gh` CLI.
pub struct Gh<'a> {
    runner: &'a dyn Runner,
}

impl<'a> Gh<'a> {
    pub fn new(runner: &'a dyn Runner) -> Self {
        Self { runner }
    }

    /// Whether the `gh` binary is available on PATH.
    pub fn is_installed(&self) -> bool {
        matches!(self.runner.capture("gh", &["--version"]), Ok(r) if r.code == 0)
    }

    /// Logged-in accounts across all hosts, with which one is active per host.
    /// Prefers the structured `--json hosts` output, falling back to parsing the
    /// human-readable form on older gh versions.
    pub fn status(&self) -> Result<Vec<Account>, GhError> {
        let json = self
            .runner
            .capture("gh", &["auth", "status", "--json", "hosts"])
            .map_err(|e| GhError(format!("could not run gh: {e}")))?;
        if json.stdout.trim_start().starts_with('{') {
            return Ok(parse_json_status(&json.stdout));
        }

        // Older gh: no --json support. Fall back to the text format.
        let text = self
            .runner
            .capture("gh", &["auth", "status"])
            .map_err(|e| GhError(format!("could not run gh: {e}")))?;
        let combined = format!("{}\n{}", text.stdout, text.stderr);
        if combined.trim().is_empty() {
            return Err(GhError(
                "could not read `gh auth status`; run `gh auth login` to sign in".to_string(),
            ));
        }
        Ok(parse_text_status(&combined))
    }

    /// Switch the active account for a host.
    pub fn switch(&self, host: &str, user: &str) -> Result<(), GhError> {
        let r = self
            .runner
            .capture(
                "gh",
                &["auth", "switch", "--hostname", host, "--user", user],
            )
            .map_err(|e| GhError(format!("gh auth switch failed: {e}")))?;
        if r.code != 0 {
            return Err(GhError(format!(
                "gh auth switch failed: {}",
                detail(&r.stderr, r.code)
            )));
        }
        Ok(())
    }

    /// Ensure gh is configured as git's credential helper for a host.
    pub fn setup_git(&self, host: &str) -> Result<(), GhError> {
        let r = self
            .runner
            .capture("gh", &["auth", "setup-git", "--hostname", host])
            .map_err(|e| GhError(format!("gh auth setup-git failed: {e}")))?;
        if r.code != 0 {
            return Err(GhError(format!(
                "gh auth setup-git failed: {}",
                detail(&r.stderr, r.code)
            )));
        }
        Ok(())
    }
}

fn detail(stderr: &str, code: i32) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        format!("exit {code}")
    } else {
        trimmed.to_string()
    }
}

#[derive(Deserialize)]
struct GhStatusJson {
    hosts: Option<HashMap<String, Vec<GhJsonHost>>>,
}

#[derive(Deserialize)]
struct GhJsonHost {
    login: String,
    host: String,
    active: bool,
}

/// Parse `gh auth status --json hosts` output into accounts.
pub fn parse_json_status(stdout: &str) -> Vec<Account> {
    let Ok(data) = serde_json::from_str::<GhStatusJson>(stdout) else {
        return Vec::new();
    };
    let mut accounts = Vec::new();
    for entries in data.hosts.into_iter().flatten().flat_map(|(_, v)| v) {
        accounts.push(Account {
            user: entries.login,
            host: entries.host,
            active: entries.active,
        });
    }
    accounts
}

/// Parse the human-readable `gh auth status` output (fallback path).
pub fn parse_text_status(text: &str) -> Vec<Account> {
    let mut accounts: Vec<Account> = Vec::new();
    for line in text.lines() {
        if let Some((host, user)) = parse_login_line(line) {
            accounts.push(Account {
                user,
                host,
                active: false,
            });
        } else if let Some(active) = parse_active_line(line) {
            if let Some(last) = accounts.last_mut() {
                last.active = active;
            }
        }
    }
    accounts
}

// Lines look like: "  ✓ Logged in to github.com account NAME (keyring)".
fn parse_login_line(line: &str) -> Option<(String, String)> {
    let rest = line.split_once("Logged in to ")?.1;
    let (host, rest) = rest.split_once(" account ")?;
    let user = rest.split_whitespace().next()?;
    Some((host.to_string(), user.to_string()))
}

fn parse_active_line(line: &str) -> Option<bool> {
    let lower = line.to_lowercase();
    let rest = lower.split_once("active account:")?.1.trim().to_string();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON_OUT: &str = r#"{"hosts":{"github.com":[{"login":"akc-barton","host":"github.com","active":true},{"login":"Alexbruvv","host":"github.com","active":false}]}}"#;

    const TEXT_OUT: &str = "github.com\n  \u{2713} Logged in to github.com account akc-barton (keyring)\n  - Active account: true\n  - Git operations protocol: https\n\n  \u{2713} Logged in to github.com account Alexbruvv (keyring)\n  - Active account: false\n  - Git operations protocol: https\n";

    fn expected() -> Vec<Account> {
        vec![
            Account {
                user: "akc-barton".to_string(),
                host: "github.com".to_string(),
                active: true,
            },
            Account {
                user: "Alexbruvv".to_string(),
                host: "github.com".to_string(),
                active: false,
            },
        ]
    }

    #[test]
    fn parses_json_status() {
        assert_eq!(parse_json_status(JSON_OUT), expected());
    }

    #[test]
    fn parses_text_status_fallback() {
        assert_eq!(parse_text_status(TEXT_OUT), expected());
    }

    #[test]
    fn json_status_empty_on_garbage() {
        assert!(parse_json_status("not json").is_empty());
    }
}
