import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { doctor } from "../src/commands/doctor.js";
import type { Account, RunResult, Runner } from "../src/types.js";

/** Runner that reports git/gh as installed and returns a fixed account list. */
function envRunner(accounts: Account[], ghInstalled = true): Runner {
  const hosts: Record<string, unknown[]> = {};
  for (const a of accounts) {
    (hosts[a.host] ??= []).push({ login: a.user, host: a.host, active: a.active });
  }
  const json = JSON.stringify({ hosts });
  return {
    async capture(cmd: string, args: string[]): Promise<RunResult> {
      if (cmd === "git" && args[0] === "--version") return ok("git version 2.0");
      if (cmd === "gh" && args[0] === "--version") {
        return ghInstalled ? ok("gh version 2.0") : { code: 1, stdout: "", stderr: "" };
      }
      if (cmd === "gh" && args.join(" ").startsWith("auth status --json")) return ok(json);
      return ok("");
    },
    async passthrough() {
      return 0;
    },
  };
}
const ok = (stdout: string): RunResult => ({ code: 0, stdout, stderr: "" });

describe("gw doctor", () => {
  let dir: string;
  let writeSpy: ReturnType<typeof vi.spyOn>;
  beforeEach(async () => {
    dir = await mkdtemp(join(tmpdir(), "gw-doctor-"));
    writeSpy = vi.spyOn(process.stdout, "write").mockReturnValue(true);
  });
  afterEach(async () => {
    writeSpy.mockRestore();
    await rm(dir, { recursive: true, force: true });
  });

  const output = () => writeSpy.mock.calls.map((c) => c[0]).join("");

  it("passes when the configured account is logged in", async () => {
    await writeFile(join(dir, ".gitwrapper"), '{"account":"alice"}');
    const code = await doctor(
      dir,
      envRunner([{ user: "alice", host: "github.com", active: true }]),
    );
    expect(code).toBe(0);
    expect(output()).toMatch(/account is logged in/);
  });

  it("fails when the configured account is not logged in", async () => {
    await writeFile(join(dir, ".gitwrapper"), '{"account":"carol"}');
    const code = await doctor(
      dir,
      envRunner([{ user: "alice", host: "github.com", active: true }]),
    );
    expect(code).toBe(1);
    expect(output()).toMatch(/account not logged in/);
  });

  it("fails when gh is not installed", async () => {
    const code = await doctor(dir, envRunner([], false));
    expect(code).toBe(1);
    expect(output()).toMatch(/gh installed.*cli\.github\.com/s);
  });

  it("reports no config gracefully", async () => {
    const code = await doctor(
      dir,
      envRunner([{ user: "alice", host: "github.com", active: true }]),
    );
    expect(code).toBe(0);
    expect(output()).toMatch(/no \.gitwrapper found/);
  });
});
