import { realRunner } from "./runner.js";
import type { Runner } from "./types.js";

/** Run `git <args>` with inherited stdio, returning git's exit code. */
export function runGit(
  args: string[],
  runner: Runner = realRunner,
): Promise<number> {
  return runner.passthrough("git", args);
}

// TODO (phase 3): isGitInstalled(), setLocalIdentity(), detectSshRemote().
