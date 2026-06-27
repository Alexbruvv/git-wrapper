use std::io;
use std::process::{Command, ExitStatus};

use crate::types::{RunResult, Runner};

/// Default runner backed by `std::process::Command`.
pub struct RealRunner;

impl Runner for RealRunner {
    fn capture(&self, cmd: &str, args: &[&str]) -> io::Result<RunResult> {
        let output = Command::new(cmd).args(args).output()?;
        Ok(RunResult {
            code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    fn passthrough(&self, cmd: &str, args: &[&str]) -> io::Result<i32> {
        // Inherited stdio: the child shares the terminal, so the OS delivers
        // signals (Ctrl-C) to it and interactive editors work unbuffered.
        let status = Command::new(cmd).args(args).status()?;
        Ok(exit_code(status))
    }
}

/// Map a child's exit status to a shell-style code (128 + signal on Unix).
fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        128 + status.signal().unwrap_or(0)
    }
    #[cfg(not(unix))]
    {
        1
    }
}
