import { describe, it, expect, beforeAll } from "vitest";
import { spawn } from "node:child_process";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Exercises the actual compiled bin via a child process. Requires a build, so
// it self-skips when dist is absent (e.g. running tests before `npm run build`).
const BIN = resolve(fileURLToPath(import.meta.url), "..", "..", "bin", "gw.js");
const built = existsSync(resolve(fileURLToPath(import.meta.url), "..", "..", "dist", "cli.js"));

function run(
  args: string[],
  cwd: string,
): Promise<{ code: number; stdout: string; stderr: string }> {
  return new Promise((res, rej) => {
    const child = spawn(process.execPath, [BIN, ...args], { cwd });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => (stdout += d));
    child.stderr.on("data", (d) => (stderr += d));
    child.on("error", rej);
    child.on("close", (code) => res({ code: code ?? 1, stdout, stderr }));
  });
}

describe.skipIf(!built)("compiled binary", () => {
  let dir: string;
  beforeAll(async () => {
    dir = await mkdtemp(join(tmpdir(), "gw-int-"));
  });

  it("prints its version", async () => {
    const { code, stdout } = await run(["--gw-version"], dir);
    expect(code).toBe(0);
    expect(stdout.trim()).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it("which reports no config in an empty dir", async () => {
    const { code, stdout } = await run(["which"], dir);
    expect(code).toBe(0);
    expect(stdout).toMatch(/No \.gitwrapper found/);
  });

  it("reports a malformed config and exits non-zero", async () => {
    const bad = await mkdtemp(join(tmpdir(), "gw-int-bad-"));
    await writeFile(join(bad, ".gitwrapper"), "{ not json");
    const { code, stderr } = await run(["which"], bad);
    expect(code).toBe(1);
    expect(stderr).toMatch(/invalid JSON/);
    await rm(bad, { recursive: true, force: true });
  });
});
