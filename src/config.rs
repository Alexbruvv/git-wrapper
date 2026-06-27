use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::types::Config;

pub const CONFIG_FILENAME: &str = ".gitwrapper";
pub const DEFAULT_HOST: &str = "github.com";

/// Raised when a `.gitwrapper` file exists but is malformed.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ConfigError {
    pub path: PathBuf,
    pub message: String,
}

fn cfg_err(path: &Path, message: impl Into<String>) -> ConfigError {
    ConfigError {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

/// Walk up from `start_dir` to the filesystem root, returning the path of the
/// first `.gitwrapper` file found, or `None` if none exists.
pub fn find_config_path(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = Some(start_dir);
    while let Some(d) = dir {
        let candidate = d.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

/// Discover, read and parse the nearest `.gitwrapper`. Returns `Ok(None)` when
/// no config is found (callers treat that as transparent passthrough).
pub fn load_config(start_dir: &Path) -> Result<Option<Config>, ConfigError> {
    let Some(path) = find_config_path(start_dir) else {
        return Ok(None);
    };
    let raw =
        fs::read_to_string(&path).map_err(|e| cfg_err(&path, format!("could not read: {e}")))?;
    Ok(Some(parse_config(&raw, &path)?))
}

/// Parse and validate raw `.gitwrapper` JSON into a normalised [`Config`].
pub fn parse_config(raw: &str, path: &Path) -> Result<Config, ConfigError> {
    let data: Value =
        serde_json::from_str(raw).map_err(|e| cfg_err(path, format!("invalid JSON: {e}")))?;
    let obj = data
        .as_object()
        .ok_or_else(|| cfg_err(path, "expected a JSON object"))?;

    let account = match obj.get("account") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            return Err(cfg_err(
                path,
                "\"account\" is required and must be a non-empty string",
            ))
        }
    };

    let host = optional_string(obj, "host", path)?.unwrap_or_else(|| DEFAULT_HOST.to_string());
    let user_name = optional_string(obj, "userName", path)?;
    let user_email = optional_string(obj, "userEmail", path)?;

    let restore_previous = match obj.get("restorePrevious") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(cfg_err(path, "\"restorePrevious\" must be a boolean")),
    };

    Ok(Config {
        account,
        host,
        user_name,
        user_email,
        restore_previous,
        source_path: path.to_path_buf(),
    })
}

fn optional_string(
    obj: &Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<Option<String>, ConfigError> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(cfg_err(path, format!("\"{key}\" must be a string"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn p() -> PathBuf {
        PathBuf::from("/tmp/.gitwrapper")
    }

    #[test]
    fn requires_account() {
        assert!(parse_config("{}", &p()).is_err());
        assert!(parse_config(r#"{"account":""}"#, &p()).is_err());
    }

    #[test]
    fn applies_defaults() {
        let c = parse_config(r#"{"account":"alice"}"#, &p()).unwrap();
        assert_eq!(c.account, "alice");
        assert_eq!(c.host, DEFAULT_HOST);
        assert!(c.restore_previous);
        assert!(c.user_name.is_none());
        assert_eq!(c.source_path, p());
    }

    #[test]
    fn reads_optional_fields() {
        let c = parse_config(
            r#"{"account":"bob","host":"ghe.corp","userName":"Bob","userEmail":"bob@corp.test","restorePrevious":false}"#,
            &p(),
        )
        .unwrap();
        assert_eq!(c.host, "ghe.corp");
        assert_eq!(c.user_name.as_deref(), Some("Bob"));
        assert_eq!(c.user_email.as_deref(), Some("bob@corp.test"));
        assert!(!c.restore_previous);
    }

    #[test]
    fn rejects_malformed_json() {
        let err = parse_config("{not json", &p()).unwrap_err();
        assert!(err.message.contains("invalid JSON"));
    }

    #[test]
    fn rejects_wrong_types() {
        assert!(parse_config("[]", &p())
            .unwrap_err()
            .message
            .contains("JSON object"));
        assert!(parse_config(r#"{"account":"a","host":1}"#, &p())
            .unwrap_err()
            .message
            .contains("\"host\" must be a string"));
        assert!(
            parse_config(r#"{"account":"a","restorePrevious":"yes"}"#, &p())
                .unwrap_err()
                .message
                .contains("\"restorePrevious\" must be a boolean")
        );
    }

    #[test]
    fn finds_nearest_then_parent() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join(".gitwrapper"), r#"{"account":"root"}"#).unwrap();
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();

        // Found in a parent.
        let cfg = load_config(&nested).unwrap().unwrap();
        assert_eq!(cfg.account, "root");

        // Nearer config wins.
        fs::write(root.join("a").join(".gitwrapper"), r#"{"account":"inner"}"#).unwrap();
        let cfg = load_config(&nested).unwrap().unwrap();
        assert_eq!(cfg.account, "inner");
    }

    #[test]
    fn returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("x").join("y");
        fs::create_dir_all(&nested).unwrap();
        assert!(find_config_path(&nested).is_none());
        assert!(load_config(&nested).unwrap().is_none());
    }
}
