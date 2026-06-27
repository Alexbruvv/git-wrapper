import { beforeAll, describe, expect, it } from "bun:test";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

// Exercises the actual compiled standalone binary. Requires `bun run build`, so
// it self-skips when the binary is absent.
const BIN = resolve(import.meta.dir, "..", "gw");
const built = existsSync(BIN);

function run(
    args: string[],
    cwd: string,
): Promise<{ code: number; stdout: string; stderr: string }> {
    return new Promise((res, rej) => {
        // Run the standalone binary directly — no Node or Bun runtime needed.
        const child = spawn(BIN, args, { cwd });
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
