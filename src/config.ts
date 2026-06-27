import type { GitWrapperConfig } from "./types.js";

export const CONFIG_FILENAME = ".gitwrapper";

/**
 * Walk up from `startDir` to the filesystem root looking for a `.gitwrapper`
 * file, parse and validate it. Returns null when none is found.
 *
 * TODO (phase 2): implement walk-up discovery, JSON parse, schema validation
 * and defaults (host="github.com", restorePrevious=true).
 */
export async function loadConfig(
  _startDir: string = process.cwd(),
): Promise<GitWrapperConfig | null> {
  throw new Error("not implemented (phase 2)");
}
