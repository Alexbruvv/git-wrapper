import { realRunner } from "./runner.js";
import type { Runner } from "./types.js";

/** Run `git <args>` with inherited stdio, returning git's exit code. */
export function runGit(
    args: string[],
    runner: Runner = realRunner,
): Promise<number> {
    return runner.passthrough("git", args);
}

/** Whether the `git` binary is available on PATH. */
export async function isGitInstalled(
    runner: Runner = realRunner,
): Promise<boolean> {
    try {
        const { code } = await runner.capture("git", ["--version"]);
        return code === 0;
    } catch {
        return false;
    }
}

/** Whether the current directory is inside a git work tree. */
export async function isInsideWorkTree(
    runner: Runner = realRunner,
): Promise<boolean> {
    const { code, stdout } = await runner.capture("git", [
        "rev-parse",
        "--is-inside-work-tree",
    ]);
    return code === 0 && stdout.trim() === "true";
}

/** Apply repo-local git identity. Best-effort; ignores failures. */
export async function setLocalIdentity(
    name: string | undefined,
    email: string | undefined,
    runner: Runner = realRunner,
): Promise<void> {
    if (name !== undefined) {
        await runner.capture("git", ["config", "--local", "user.name", name]);
    }
    if (email !== undefined) {
        await runner.capture("git", ["config", "--local", "user.email", email]);
    }
}

/** URL of the `origin` remote, or null if there is none. */
export async function getOriginUrl(
    runner: Runner = realRunner,
): Promise<string | null> {
    const { code, stdout } = await runner.capture("git", [
        "remote",
        "get-url",
        "origin",
    ]);
    if (code !== 0) return null;
    const url = stdout.trim();
    return url === "" ? null : url;
}

/** Heuristic: does this remote URL use SSH rather than HTTPS? */
export function isSshRemote(url: string): boolean {
    return url.startsWith("git@") || url.startsWith("ssh://");
}
