import { realRunner } from "./runner.js";
import type { Account, Runner } from "./types.js";

/**
 * Thin wrapper around the `gh` CLI. All methods are stubs for phase 1 and are
 * filled in during phase 3, where the `runner` seam lets tests mock gh.
 */
export class Gh {
  constructor(private readonly runner: Runner = realRunner) {}

  /** Whether the `gh` binary is available. TODO (phase 3). */
  async isInstalled(): Promise<boolean> {
    throw new Error("not implemented (phase 3)");
  }

  /** Logged-in accounts and which one is active. TODO (phase 3). */
  async status(): Promise<Account[]> {
    throw new Error("not implemented (phase 3)");
  }

  /** Switch the active account for a host. TODO (phase 3). */
  async switch(_host: string, _user: string): Promise<void> {
    throw new Error("not implemented (phase 3)");
  }

  /** Ensure gh is wired as git's credential helper. TODO (phase 3). */
  async setupGit(_host: string): Promise<void> {
    throw new Error("not implemented (phase 3)");
  }
}
