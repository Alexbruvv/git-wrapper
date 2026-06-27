use std::env;
use std::io::IsTerminal;

// All wrapper chatter goes to stderr so it never pollutes the stdout of the
// git command being wrapped.

fn use_color() -> bool {
    std::io::stderr().is_terminal() && env::var_os("NO_COLOR").is_none()
}

fn paint(code: &str, text: &str) -> String {
    if use_color() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

/// Low-key informational line (e.g. "switched to account X").
pub fn notice(message: &str) {
    eprintln!("{}", paint("2", &format!("gw: {message}")));
}

pub fn warn(message: &str) {
    eprintln!("{}", paint("33", &format!("gw: warning: {message}")));
}

pub fn error(message: &str) {
    eprintln!("{}", paint("31", &format!("gw: error: {message}")));
}
