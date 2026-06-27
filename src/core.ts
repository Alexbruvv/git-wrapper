import { loadConfig, ConfigError } from "./config.js";
import { Gh, GhError } from "./gh.js";
import {
  runGit,
  isGitInstalled,
  isInsideWorkTree,
  setLocalIdentity,
  getOriginUrl,
  isSshRemote,
} from "./git.js";
import { realRunner } from "./runner.js";
import * as log from "./log.js";
import type { Account, Runner } from "./types.js";

export interface CoreDeps {
  runner?: Runner;
  gh?: Gh;
  cwd?: string;
}

/**
 * The full wrapped-git flow: resolve the project account, switch gh to it, run
 * the git command with the user's stdio, then restore the previous account.
 * Returns the process exit code to use.
 */
export async function runWrapped(
  args: string[],
  deps: CoreDeps = {},
): Promise<number> {
  const runner = deps.runner ?? realRunner;
  const gh = deps.gh ?? new Gh(runner);
  const cwd = deps.cwd ?? process.cwd();

  // 1. Resolve config. No config → transparent passthrough.
  let config;
  try {
    config = await loadConfig(cwd);
  } catch (err) {
    if (err instanceof ConfigError) {
      log.error(`${err.path}: ${err.message}`);
      return 1;
    }
    throw err;
  }

  if (config === null) {
    return runGit(args, runner);
  }

  // 2. Preflight: required binaries.
  if (!(await isGitInstalled(runner))) {
    log.error("git is not installed or not on PATH");
    return 1;
  }
  if (!(await gh.isInstalled())) {
    log.error("the GitHub CLI (gh) is not installed — see https://cli.github.com");
    return 1;
  }

  // 3. Read gh accounts for the configured host.
  let accounts: Account[];
  try {
    accounts = await gh.status();
  } catch (err) {
    if (err instanceof GhError) {
      log.error(err.message);
      return 1;
    }
    throw err;
  }

  const onHost = accounts.filter((a) => eqHost(a.host, config.host));
  const target = onHost.find((a) => eqUser(a.user, config.account));
  if (!target) {
    log.error(
      `account "${config.account}" is not logged in to ${config.host}; ` +
        `run \`gh auth login --hostname ${config.host}\``,
    );
    return 1;
  }

  const previousActive = onHost.find((a) => a.active) ?? null;
  const alreadyActive = previousActive?.user === target.user;

  // 4. Switch to the project account (unless already active).
  if (!alreadyActive) {
    try {
      await gh.setupGit(config.host);
      await gh.switch(config.host, target.user);
    } catch (err) {
      if (err instanceof GhError) {
        log.error(err.message);
        return 1;
      }
      throw err;
    }
    log.notice(`switched to ${target.user} on ${config.host}`);
  }

  // 5. Repo-local identity + SSH warning (best-effort, only inside a work tree).
  if (await isInsideWorkTree(runner)) {
    if (config.userName !== undefined || config.userEmail !== undefined) {
      await setLocalIdentity(config.userName, config.userEmail, runner);
    }
    const origin = await getOriginUrl(runner);
    if (origin && isSshRemote(origin)) {
      log.warn(
        "origin uses SSH; gh account switching only affects HTTPS credentials",
      );
    }
  }

  // 6. Run git with the user's terminal attached.
  const code = await runGit(args, runner);

  // 7. Restore the previously active account.
  if (!alreadyActive && config.restorePrevious && previousActive) {
    try {
      await gh.switch(config.host, previousActive.user);
      log.notice(`restored ${previousActive.user} on ${config.host}`);
    } catch {
      log.warn(`could not restore previous account ${previousActive.user}`);
    }
  }

  return code;
}

function eqHost(a: string, b: string): boolean {
  return a.toLowerCase() === b.toLowerCase();
}

// GitHub usernames are case-insensitive.
function eqUser(a: string, b: string): boolean {
  return a.toLowerCase() === b.toLowerCase();
}
