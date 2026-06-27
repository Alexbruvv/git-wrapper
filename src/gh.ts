import { realRunner } from "./runner.js";
import type { Account, Runner } from "./types.js";

/** Raised when a `gh` invocation fails in a way the user must act on. */
export class GhError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "GhError";
    }
}

interface GhJsonHost {
    login: string;
    host: string;
    active: boolean;
}

/** Thin, mockable wrapper around the `gh` CLI. */
export class Gh {
    constructor(private readonly runner: Runner = realRunner) {}

    /** Whether the `gh` binary is available on PATH. */
    async isInstalled(): Promise<boolean> {
        try {
            const { code } = await this.runner.capture("gh", ["--version"]);
            return code === 0;
        } catch {
            return false;
        }
    }

    /**
     * Logged-in accounts across all hosts, with which one is active per host.
     * Prefers the structured `--json hosts` output, falling back to parsing the
     * human-readable form on older gh versions.
     */
    async status(): Promise<Account[]> {
        const json = await this.runner.capture("gh", [
            "auth",
            "status",
            "--json",
            "hosts",
        ]);
        if (json.stdout.trim().startsWith("{")) {
            return parseJsonStatus(json.stdout);
        }
        // Older gh: no --json support. Fall back to the text format.
        const text = await this.runner.capture("gh", ["auth", "status"]);
        const combined = `${text.stdout}\n${text.stderr}`;
        if (combined.trim() === "") {
            throw new GhError(
                "could not read `gh auth status`; run `gh auth login` to sign in",
            );
        }
        return parseTextStatus(combined);
    }

    /** Switch the active account for a host. Throws GhError on failure. */
    async switch(host: string, user: string): Promise<void> {
        const { code, stderr } = await this.runner.capture("gh", [
            "auth",
            "switch",
            "--hostname",
            host,
            "--user",
            user,
        ]);
        if (code !== 0) {
            throw new GhError(
                `gh auth switch failed: ${stderr.trim() || `exit ${code}`}`,
            );
        }
    }

    /** Ensure gh is configured as git's credential helper for a host. */
    async setupGit(host: string): Promise<void> {
        const { code, stderr } = await this.runner.capture("gh", [
            "auth",
            "setup-git",
            "--hostname",
            host,
        ]);
        if (code !== 0) {
            throw new GhError(
                `gh auth setup-git failed: ${stderr.trim() || `exit ${code}`}`,
            );
        }
    }
}

export function parseJsonStatus(stdout: string): Account[] {
    let data: { hosts?: Record<string, GhJsonHost[]> };
    try {
        data = JSON.parse(stdout);
    } catch {
        return [];
    }
    const accounts: Account[] = [];
    for (const entries of Object.values(data.hosts ?? {})) {
        for (const e of entries) {
            accounts.push({
                user: e.login,
                host: e.host,
                active: Boolean(e.active),
            });
        }
    }
    return accounts;
}

/** Parse the human-readable `gh auth status` output (fallback path). */
export function parseTextStatus(text: string): Account[] {
    const accounts: Account[] = [];
    const lines = text.split("\n");
    let current: Account | null = null;
    // Lines look like: "  ✓ Logged in to github.com account NAME (keyring)"
    const loginRe = /Logged in to (\S+) account (\S+)/;
    const activeRe = /Active account:\s*(true|false)/i;
    for (const line of lines) {
        const m = loginRe.exec(line);
        if (m) {
            current = { host: m[1]!, user: m[2]!, active: false };
            accounts.push(current);
            continue;
        }
        const a = activeRe.exec(line);
        if (a && current) current.active = a[1]!.toLowerCase() === "true";
    }
    return accounts;
}
