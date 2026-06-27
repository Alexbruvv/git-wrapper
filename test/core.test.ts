import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { runWrapped } from "../src/core.js";
import { Gh } from "../src/gh.js";
import type { Account, RunResult, Runner } from "../src/types.js";

/** Runner that returns canned capture results and records passthroughs. */
class FakeRunner implements Runner {
  captures: string[][] = [];
  passthroughs: string[][] = [];
  originUrl = "https://github.com/acme/repo.git";
  gitCode = 0;

  async capture(cmd: string, args: string[]): Promise<RunResult> {
    this.captures.push([cmd, ...args]);
    const a = args.join(" ");
    if (cmd === "git" && a === "--version") return ok("git version 2.0");
    if (cmd === "git" && a === "rev-parse --is-inside-work-tree") return ok("true");
    if (cmd === "git" && a === "remote get-url origin") return ok(this.originUrl);
    return ok("");
  }

  async passthrough(cmd: string, args: string[]): Promise<number> {
    this.passthroughs.push([cmd, ...args]);
    return this.gitCode;
  }
}

const ok = (stdout: string): RunResult => ({ code: 0, stdout, stderr: "" });

/** Fake gh that records switches and reports a fixed account list. */
class FakeGh extends Gh {
  switches: string[] = [];
  setups = 0;
  constructor(private accounts: Account[]) {
    super();
  }
  override async isInstalled() {
    return true;
  }
  override async status() {
    return this.accounts;
  }
  override async switch(_host: string, user: string) {
    this.switches.push(user);
  }
  override async setupGit() {
    this.setups++;
  }
}

describe("runWrapped", () => {
  let dir: string;
  beforeEach(async () => {
    dir = await mkdtemp(join(tmpdir(), "gw-core-"));
  });
  afterEach(async () => {
    await rm(dir, { recursive: true, force: true });
  });

  async function writeConfig(obj: Record<string, unknown>) {
    await writeFile(join(dir, ".gitwrapper"), JSON.stringify(obj));
  }

  it("passes through unchanged when no config exists", async () => {
    const runner = new FakeRunner();
    const gh = new FakeGh([]);
    const code = await runWrapped(["status"], { runner, gh, cwd: dir });
    expect(code).toBe(0);
    expect(runner.passthroughs).toEqual([["git", "status"]]);
    expect(gh.switches).toEqual([]);
  });

  it("switches to the configured account then restores the previous one", async () => {
    await writeConfig({ account: "alice" });
    const runner = new FakeRunner();
    const gh = new FakeGh([
      { user: "bob", host: "github.com", active: true },
      { user: "alice", host: "github.com", active: false },
    ]);
    const code = await runWrapped(["push"], { runner, gh, cwd: dir });
    expect(code).toBe(0);
    expect(runner.passthroughs).toEqual([["git", "push"]]);
    // switch to alice, then restore bob.
    expect(gh.switches).toEqual(["alice", "bob"]);
  });

  it("does not switch or restore when already active", async () => {
    await writeConfig({ account: "alice" });
    const runner = new FakeRunner();
    const gh = new FakeGh([{ user: "alice", host: "github.com", active: true }]);
    await runWrapped(["fetch"], { runner, gh, cwd: dir });
    expect(gh.switches).toEqual([]);
  });

  it("matches the account case-insensitively", async () => {
    await writeConfig({ account: "ALICE" });
    const runner = new FakeRunner();
    const gh = new FakeGh([{ user: "alice", host: "github.com", active: true }]);
    await runWrapped(["fetch"], { runner, gh, cwd: dir });
    expect(gh.switches).toEqual([]); // resolved to active "alice", no switch
  });

  it("errors when the account is not logged in", async () => {
    await writeConfig({ account: "carol" });
    const runner = new FakeRunner();
    const gh = new FakeGh([{ user: "alice", host: "github.com", active: true }]);
    const code = await runWrapped(["status"], { runner, gh, cwd: dir });
    expect(code).toBe(1);
    expect(runner.passthroughs).toEqual([]); // git never ran
  });

  it("skips restore when restorePrevious is false", async () => {
    await writeConfig({ account: "alice", restorePrevious: false });
    const runner = new FakeRunner();
    const gh = new FakeGh([
      { user: "bob", host: "github.com", active: true },
      { user: "alice", host: "github.com", active: false },
    ]);
    await runWrapped(["push"], { runner, gh, cwd: dir });
    expect(gh.switches).toEqual(["alice"]); // no restore back to bob
  });

  it("propagates git's exit code", async () => {
    await writeConfig({ account: "alice" });
    const runner = new FakeRunner();
    runner.gitCode = 7;
    const gh = new FakeGh([{ user: "alice", host: "github.com", active: true }]);
    const code = await runWrapped(["push"], { runner, gh, cwd: dir });
    expect(code).toBe(7);
  });

  it("applies repo-local identity when configured", async () => {
    await writeConfig({ account: "alice", userName: "Alice", userEmail: "a@x.test" });
    const runner = new FakeRunner();
    const gh = new FakeGh([{ user: "alice", host: "github.com", active: true }]);
    await runWrapped(["commit"], { runner, gh, cwd: dir });
    const cfgCalls = runner.captures.filter(
      (c) => c[0] === "git" && c[1] === "config",
    );
    expect(cfgCalls).toEqual([
      ["git", "config", "--local", "user.name", "Alice"],
      ["git", "config", "--local", "user.email", "a@x.test"],
    ]);
  });
});
