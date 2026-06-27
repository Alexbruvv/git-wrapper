import { readFile, stat } from "node:fs/promises";
import { dirname, join, parse as parsePath } from "node:path";
import type { GitWrapperConfig } from "./types.js";

export const CONFIG_FILENAME = ".gitwrapper";
export const DEFAULT_HOST = "github.com";

/** Raised when a `.gitwrapper` file exists but is malformed. */
export class ConfigError extends Error {
  constructor(
    message: string,
    readonly path: string,
  ) {
    super(message);
    this.name = "ConfigError";
  }
}

/**
 * Walk up from `startDir` to the filesystem root, returning the path of the
 * first `.gitwrapper` file found, or null if none exists.
 */
export async function findConfigPath(
  startDir: string = process.cwd(),
): Promise<string | null> {
  let dir = startDir;
  const { root } = parsePath(dir);
  // Loop until we step past the root (dirname of root is root itself).
  for (;;) {
    const candidate = join(dir, CONFIG_FILENAME);
    if (await isFile(candidate)) return candidate;
    if (dir === root) return null;
    const parent = dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
}

/**
 * Discover, parse and validate the nearest `.gitwrapper`. Returns null when no
 * config is found (callers treat that as transparent passthrough). Throws
 * ConfigError when a file is present but invalid.
 */
export async function loadConfig(
  startDir: string = process.cwd(),
): Promise<GitWrapperConfig | null> {
  const path = await findConfigPath(startDir);
  if (path === null) return null;
  return parseConfig(await readFile(path, "utf8"), path);
}

/** Parse and validate raw `.gitwrapper` JSON into a normalised config. */
export function parseConfig(raw: string, path: string): GitWrapperConfig {
  let data: unknown;
  try {
    data = JSON.parse(raw);
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err);
    throw new ConfigError(`invalid JSON: ${detail}`, path);
  }

  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new ConfigError("expected a JSON object", path);
  }
  const obj = data as Record<string, unknown>;

  const account = obj.account;
  if (typeof account !== "string" || account.trim() === "") {
    throw new ConfigError('"account" is required and must be a non-empty string', path);
  }

  const host = optionalString(obj, "host", path) ?? DEFAULT_HOST;
  const userName = optionalString(obj, "userName", path);
  const userEmail = optionalString(obj, "userEmail", path);

  let restorePrevious = true;
  if ("restorePrevious" in obj && obj.restorePrevious !== undefined) {
    if (typeof obj.restorePrevious !== "boolean") {
      throw new ConfigError('"restorePrevious" must be a boolean', path);
    }
    restorePrevious = obj.restorePrevious;
  }

  return {
    account: account.trim(),
    host,
    ...(userName !== undefined ? { userName } : {}),
    ...(userEmail !== undefined ? { userEmail } : {}),
    restorePrevious,
    sourcePath: path,
  };
}

function optionalString(
  obj: Record<string, unknown>,
  key: string,
  path: string,
): string | undefined {
  const value = obj[key];
  if (value === undefined) return undefined;
  if (typeof value !== "string") {
    throw new ConfigError(`"${key}" must be a string`, path);
  }
  return value;
}

async function isFile(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isFile();
  } catch {
    return false;
  }
}
