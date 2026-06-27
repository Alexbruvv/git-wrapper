/** Parsed and validated contents of a project's `.gitwrapper` file. */
export interface GitWrapperConfig {
    /** gh username to switch to before running git. */
    account: string;
    /** Hostname the account lives on. Defaults to "github.com". */
    host: string;
    /** When set, applied as repo-local git `user.name`. */
    userName?: string;
    /** When set, applied as repo-local git `user.email`. */
    userEmail?: string;
    /** Restore the previously active gh account after the command. Default true. */
    restorePrevious: boolean;
    /** Absolute path the config was loaded from (for diagnostics). */
    sourcePath: string;
}

/** A GitHub account known to the local `gh` CLI. */
export interface Account {
    user: string;
    host: string;
    active: boolean;
}

/** Result of running a child process to completion. */
export interface RunResult {
    code: number;
    stdout: string;
    stderr: string;
}

/**
 * Injectable command runner so gh/git calls can be mocked in tests.
 * `capture` collects output; `passthrough` inherits stdio for interactive use.
 */
export interface Runner {
    capture(cmd: string, args: string[]): Promise<RunResult>;
    passthrough(cmd: string, args: string[]): Promise<number>;
}
