use std::fs;
use std::io::IsTerminal;
use std::path::Path;

use crate::config::{CONFIG_FILENAME, DEFAULT_HOST};
use crate::gh::Gh;
use crate::log;
use crate::runner::RealRunner;
use crate::types::{Account, Runner};

/// Scaffold a `.gitwrapper` in the current directory. Accepts an optional
/// account argument; otherwise prompts (when interactive) or infers a sensible
/// default from the logged-in gh accounts. By default the file is added to
/// `.gitignore`; pass `--no-gitignore` to leave `.gitignore` untouched.
///
/// Usage: gw init [account] [--no-gitignore]
pub fn init(args: &[String]) -> i32 {
    let cwd = std::env::current_dir().unwrap_or_default();
    run_init(args, &cwd, &RealRunner)
}

fn run_init(args: &[String], cwd: &Path, runner: &dyn Runner) -> i32 {
    let no_gitignore = args.iter().any(|a| a == "--no-gitignore");
    let arg_account = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(String::as_str);

    let target = cwd.join(CONFIG_FILENAME);
    if target.exists() {
        log::error(&format!(
            "{CONFIG_FILENAME} already exists in this directory"
        ));
        return 1;
    }

    let gh = Gh::new(runner);
    let accounts = if gh.is_installed() {
        gh.status().unwrap_or_default()
    } else {
        Vec::new()
    };

    let Some(chosen) = choose_account(arg_account, &accounts) else {
        return 1;
    };

    let config = serde_json::json!({ "account": chosen.user, "host": chosen.host });
    let text = format!("{}\n", serde_json::to_string_pretty(&config).unwrap());
    if let Err(e) = fs::write(&target, text) {
        log::error(&format!("could not write {}: {e}", target.display()));
        return 1;
    }
    println!("Created {}", target.display());
    println!("  account: {} @ {}", chosen.user, chosen.host);

    if !no_gitignore {
        add_to_gitignore(cwd, CONFIG_FILENAME);
    }
    0
}

struct Chosen {
    user: String,
    host: String,
}

fn choose_account(arg_account: Option<&str>, accounts: &[Account]) -> Option<Chosen> {
    // Explicit argument wins.
    if let Some(arg) = arg_account {
        if let Some(m) = accounts.iter().find(|a| a.user.eq_ignore_ascii_case(arg)) {
            return Some(Chosen {
                user: m.user.clone(),
                host: m.host.clone(),
            });
        }
        log::warn(&format!(
            "\"{arg}\" is not a logged-in gh account; writing it anyway"
        ));
        return Some(Chosen {
            user: arg.to_string(),
            host: DEFAULT_HOST.to_string(),
        });
    }

    if accounts.is_empty() {
        log::error("no gh accounts found — run `gh auth login`, or `gw init <account>`");
        return None;
    }

    // Interactive selection when attached to a terminal.
    if std::io::stdin().is_terminal() && accounts.len() > 1 {
        return prompt_for_account(accounts);
    }

    // Non-interactive: prefer the active account, else the only one.
    let active = accounts.iter().find(|a| a.active).unwrap_or(&accounts[0]);
    Some(Chosen {
        user: active.user.clone(),
        host: active.host.clone(),
    })
}

fn prompt_for_account(accounts: &[Account]) -> Option<Chosen> {
    use std::io::Write;
    eprintln!("Select an account for this project:");
    for (i, a) in accounts.iter().enumerate() {
        let active = if a.active { " (active)" } else { "" };
        eprintln!("  {}) {} @ {}{active}", i + 1, a.user, a.host);
    }
    eprint!("Number: ");
    let _ = std::io::stderr().flush();

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        log::error("could not read selection");
        return None;
    }
    let idx = match answer.trim().parse::<usize>() {
        Ok(n) if (1..=accounts.len()).contains(&n) => n - 1,
        _ => {
            log::error("invalid selection");
            return None;
        }
    };
    let pick = &accounts[idx];
    Some(Chosen {
        user: pick.user.clone(),
        host: pick.host.clone(),
    })
}

/// Ensure `entry` is listed in the directory's `.gitignore`, creating the file
/// if needed and never duplicating an existing entry.
fn add_to_gitignore(cwd: &Path, entry: &str) {
    let path = cwd.join(".gitignore");
    let content = fs::read_to_string(&path).unwrap_or_default();
    let slash = format!("/{entry}");
    let already = content
        .lines()
        .map(str::trim)
        .any(|l| l == entry || l == slash.as_str());
    if already {
        println!("  {entry} already in .gitignore");
        return;
    }

    let prefix = if !content.is_empty() && !content.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    let updated = format!("{content}{prefix}{entry}\n");
    if fs::write(&path, updated).is_ok() {
        println!("  Added {entry} to .gitignore");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::FakeRunner;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn writes_config_for_explicit_account() {
        let dir = tempdir().unwrap();
        let code = run_init(&["alice".to_string()], dir.path(), &FakeRunner::new(&[]));
        assert_eq!(code, 0);
        let written = fs::read_to_string(dir.path().join(".gitwrapper")).unwrap();
        assert!(written.contains("\"account\": \"alice\""));
        assert!(written.contains("\"host\": \"github.com\""));
    }

    #[test]
    fn uses_active_account_non_interactively() {
        let dir = tempdir().unwrap();
        let runner = FakeRunner::new(&[("bob", false), ("alice", true)]);
        let code = run_init(&[], dir.path(), &runner);
        assert_eq!(code, 0);
        let written = fs::read_to_string(dir.path().join(".gitwrapper")).unwrap();
        assert!(written.contains("\"account\": \"alice\""));
    }

    #[test]
    fn refuses_to_overwrite() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitwrapper"), "{}").unwrap();
        let code = run_init(&["alice".to_string()], dir.path(), &FakeRunner::new(&[]));
        assert_eq!(code, 1);
    }

    #[test]
    fn errors_with_no_accounts_and_no_arg() {
        let dir = tempdir().unwrap();
        let code = run_init(&[], dir.path(), &FakeRunner::new(&[]));
        assert_eq!(code, 1);
    }

    #[test]
    fn adds_to_gitignore_by_default() {
        let dir = tempdir().unwrap();
        run_init(&["alice".to_string()], dir.path(), &FakeRunner::new(&[]));
        let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(gitignore, ".gitwrapper\n");
    }

    #[test]
    fn appends_to_existing_gitignore() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "node_modules/").unwrap();
        run_init(&["alice".to_string()], dir.path(), &FakeRunner::new(&[]));
        let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(gitignore, "node_modules/\n.gitwrapper\n");
    }

    #[test]
    fn does_not_duplicate_gitignore_entry() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), ".gitwrapper\n").unwrap();
        run_init(&["alice".to_string()], dir.path(), &FakeRunner::new(&[]));
        let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(gitignore, ".gitwrapper\n");
    }

    #[test]
    fn skips_gitignore_with_flag() {
        let dir = tempdir().unwrap();
        let code = run_init(
            &["alice".to_string(), "--no-gitignore".to_string()],
            dir.path(),
            &FakeRunner::new(&[]),
        );
        assert_eq!(code, 0);
        assert!(dir.path().join(".gitwrapper").exists());
        assert!(!dir.path().join(".gitignore").exists());
    }
}
