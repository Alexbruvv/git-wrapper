import { describe, it, expect } from "bun:test";
import { realRunner } from "../src/runner.js";
import { runGit } from "../src/git.js";

describe("phase 1 scaffold", () => {
  it("captures output via the real runner", async () => {
    const res = await realRunner.capture("git", ["--version"]);
    expect(res.code).toBe(0);
    expect(res.stdout).toMatch(/git version/);
  });

  it("passes through to git and returns its exit code", async () => {
    const code = await runGit(["--version"]);
    expect(code).toBe(0);
  });
});
