import { writeFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { createInterface } from "node:readline/promises";
import { Gh } from "../gh.js";
import { CONFIG_FILENAME, DEFAULT_HOST } from "../config.js";
import { realRunner } from "../runner.js";
import * as log from "../log.js";
import type { Account, Runner } from "../types.js";

/**
 * Scaffold a `.gitwrapper` in the current directory. Accepts an optional
 * account argument; otherwise prompts (when interactive) or infers a sensible
 * default from the logged-in gh accounts.
 *
 * Usage: gw init [account]
 */
export async function init(
  args: string[] = [],
  cwd: string = process.cwd(),
  runner: Runner = realRunner,
): Promise<number> {
  const target = join(cwd, CONFIG_FILENAME);
  if (await exists(target)) {
    log.error(`${CONFIG_FILENAME} already exists in this directory`);
    return 1;
  }

  const gh = new Gh(runner);
  const accounts = (await gh.isInstalled()) ? await safeStatus(gh) : [];

  const chosen = await chooseAccount(args[0], accounts);
  if (chosen === null) return 1;

  const config = { account: chosen.user, host: chosen.host };
  await writeFile(target, JSON.stringify(config, null, 2) + "\n");
  process.stdout.write(`Created ${target}\n`);
  process.stdout.write(`  account: ${chosen.user} @ ${chosen.host}\n`);
  return 0;
}

interface Chosen {
  user: string;
  host: string;
}

async function chooseAccount(
  argAccount: string | undefined,
  accounts: Account[],
): Promise<Chosen | null> {
  // Explicit argument wins.
  if (argAccount) {
    const match = accounts.find((a) => a.user.toLowerCase() === argAccount.toLowerCase());
    if (match) return { user: match.user, host: match.host };
    log.warn(`"${argAccount}" is not a logged-in gh account; writing it anyway`);
    return { user: argAccount, host: DEFAULT_HOST };
  }

  if (accounts.length === 0) {
    log.error("no gh accounts found — run `gh auth login`, or `gw init <account>`");
    return null;
  }

  // Interactive selection when attached to a terminal.
  if (process.stdin.isTTY && accounts.length > 1) {
    return promptForAccount(accounts);
  }

  // Non-interactive: prefer the active account, else the only one.
  const active = accounts.find((a) => a.active) ?? accounts[0]!;
  return { user: active.user, host: active.host };
}

async function promptForAccount(accounts: Account[]): Promise<Chosen | null> {
  const rl = createInterface({ input: process.stdin, output: process.stderr });
  try {
    process.stderr.write("Select an account for this project:\n");
    accounts.forEach((a, i) => {
      process.stderr.write(
        `  ${i + 1}) ${a.user} @ ${a.host}${a.active ? " (active)" : ""}\n`,
      );
    });
    const answer = (await rl.question("Number: ")).trim();
    const idx = Number.parseInt(answer, 10) - 1;
    const pick = accounts[idx];
    if (!pick) {
      log.error("invalid selection");
      return null;
    }
    return { user: pick.user, host: pick.host };
  } finally {
    rl.close();
  }
}

async function safeStatus(gh: Gh): Promise<Account[]> {
  try {
    return await gh.status();
  } catch {
    return [];
  }
}

async function exists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch {
    return false;
  }
}
