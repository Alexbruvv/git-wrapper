use crate::config;
use crate::log;

/// Resolve the nearest `.gitwrapper` and report the account/host that would be
/// used for the current directory, plus where the config was found.
pub fn which() -> i32 {
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            log::error(&format!("cannot determine current directory: {err}"));
            return 1;
        }
    };

    match config::load_config(&cwd) {
        Ok(None) => {
            println!("No .gitwrapper found; git commands run unchanged.");
            0
        }
        Ok(Some(cfg)) => {
            let mut lines = vec![
                format!("account: {}", cfg.account),
                format!("host:    {}", cfg.host),
            ];
            if let Some(name) = &cfg.user_name {
                lines.push(format!("name:    {name}"));
            }
            if let Some(email) = &cfg.user_email {
                lines.push(format!("email:   {email}"));
            }
            lines.push(format!("restore: {}", cfg.restore_previous));
            lines.push(format!("config:  {}", cfg.source_path.display()));
            println!("{}", lines.join("\n"));
            0
        }
        Err(err) => {
            log::error(&format!("{}: {}", err.path.display(), err.message));
            1
        }
    }
}
