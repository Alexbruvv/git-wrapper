import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { init } from "../src/commands/init.js";
import type { Account, Runner, RunResult } from "../src/types.js";

/** Runner whose gh capture returns a fixed account list as `--json hosts`. */
function ghRunner(accounts: Account[]): Runner {
    const hosts: Record<string, unknown[]> = {};
    for (const a of accounts) {
        const bucket = hosts[a.host] ?? [];
        bucket.push({ login: a.user, host: a.host, active: a.active });
        hosts[a.host] = bucket;
    }
    const json = JSON.stringify({ hosts });
    return {
        async capture(cmd: string, args: string[]): Promise<RunResult> {
            if (cmd === "gh" && args[0] === "--version")
                return ok("gh version 2.0");
            if (cmd === "gh" && args.join(" ").startsWith("auth status --json"))
                return ok(json);
            return ok("");
        },
        async passthrough() {
            return 0;
        },
    };
}
const ok = (stdout: string): RunResult => ({ code: 0, stdout, stderr: "" });

describe("gw init", () => {
    let dir: string;
    beforeEach(async () => {
        dir = await mkdtemp(join(tmpdir(), "gw-init-"));
    });
    afterEach(async () => {
        await rm(dir, { recursive: true, force: true });
    });

    it("writes a config for an explicit account argument", async () => {
        const code = await init(["alice"], dir, ghRunner([]));
        expect(code).toBe(0);
        const written = JSON.parse(
            await readFile(join(dir, ".gitwrapper"), "utf8"),
        );
        expect(written).toEqual({ account: "alice", host: "github.com" });
    });

    it("uses the active account non-interactively", async () => {
        const runner = ghRunner([
            { user: "bob", host: "github.com", active: false },
            { user: "alice", host: "github.com", active: true },
        ]);
        const code = await init([], dir, runner);
        expect(code).toBe(0);
        const written = JSON.parse(
            await readFile(join(dir, ".gitwrapper"), "utf8"),
        );
        expect(written.account).toBe("alice");
    });

    it("refuses to overwrite an existing config", async () => {
        await writeFile(join(dir, ".gitwrapper"), "{}");
        const code = await init(["alice"], dir, ghRunner([]));
        expect(code).toBe(1);
    });

    it("errors when no accounts and no argument", async () => {
        const code = await init([], dir, ghRunner([]));
        expect(code).toBe(1);
    });
});
