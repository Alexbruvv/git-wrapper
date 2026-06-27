import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  parseConfig,
  findConfigPath,
  loadConfig,
  ConfigError,
  DEFAULT_HOST,
} from "../src/config.js";

describe("parseConfig", () => {
  const p = "/tmp/.gitwrapper";

  it("requires account", () => {
    expect(() => parseConfig("{}", p)).toThrow(ConfigError);
    expect(() => parseConfig('{"account":""}', p)).toThrow(/non-empty/);
  });

  it("applies defaults for host and restorePrevious", () => {
    const c = parseConfig('{"account":"alice"}', p);
    expect(c.account).toBe("alice");
    expect(c.host).toBe(DEFAULT_HOST);
    expect(c.restorePrevious).toBe(true);
    expect(c.userName).toBeUndefined();
    expect(c.sourcePath).toBe(p);
  });

  it("reads optional fields", () => {
    const c = parseConfig(
      JSON.stringify({
        account: "bob",
        host: "ghe.corp",
        userName: "Bob",
        userEmail: "bob@corp.test",
        restorePrevious: false,
      }),
      p,
    );
    expect(c).toMatchObject({
      account: "bob",
      host: "ghe.corp",
      userName: "Bob",
      userEmail: "bob@corp.test",
      restorePrevious: false,
    });
  });

  it("rejects malformed JSON", () => {
    expect(() => parseConfig("{not json", p)).toThrow(/invalid JSON/);
  });

  it("rejects wrong types", () => {
    expect(() => parseConfig("[]", p)).toThrow(/JSON object/);
    expect(() => parseConfig('{"account":"a","host":1}', p)).toThrow(/"host" must be a string/);
    expect(() => parseConfig('{"account":"a","restorePrevious":"yes"}', p)).toThrow(
      /"restorePrevious" must be a boolean/,
    );
  });
});

describe("findConfigPath / loadConfig (filesystem)", () => {
  let dir: string;

  beforeEach(async () => {
    dir = await mkdtemp(join(tmpdir(), "gw-test-"));
  });
  afterEach(async () => {
    await rm(dir, { recursive: true, force: true });
  });

  it("returns null when no config exists", async () => {
    const nested = join(dir, "a", "b");
    await mkdir(nested, { recursive: true });
    expect(await findConfigPath(nested)).toBeNull();
    expect(await loadConfig(nested)).toBeNull();
  });

  it("finds a config in a parent directory", async () => {
    await writeFile(join(dir, ".gitwrapper"), '{"account":"alice"}');
    const nested = join(dir, "a", "b");
    await mkdir(nested, { recursive: true });
    const found = await findConfigPath(nested);
    expect(found).toBe(join(dir, ".gitwrapper"));
    const cfg = await loadConfig(nested);
    expect(cfg?.account).toBe("alice");
  });

  it("prefers the nearest config", async () => {
    await writeFile(join(dir, ".gitwrapper"), '{"account":"root"}');
    const nested = join(dir, "a");
    await mkdir(nested, { recursive: true });
    await writeFile(join(nested, ".gitwrapper"), '{"account":"inner"}');
    const cfg = await loadConfig(nested);
    expect(cfg?.account).toBe("inner");
  });
});
