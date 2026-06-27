import { ConfigError, loadConfig } from "../config.js";
import { Gh, GhError } from "../gh.js";
import { isGitInstalled } from "../git.js";
import { realRunner } from "../runner.js";
import type { Runner } from "../types.js";

const PASS = "✓";
const FAIL = "✗";
const INFO = "•";

/**
 * Diagnose the environment: git/gh install, gh auth accounts, and the nearest
 * `.gitwrapper`. Prints a checklist; returns 1 if anything is actionable.
 */
export async function doctor(
    cwd: string = process.cwd(),
    runner: Runner = realRunner,
): Promise<number> {
    const out = (s: string) => process.stdout.write(`${s}\n`);
    let ok = true;

    // git
    const gitOk = await isGitInstalled(runner);
    out(`${gitOk ? PASS : FAIL} git installed`);
    ok &&= gitOk;

    // gh
    const gh = new Gh(runner);
    const ghOk = await gh.isInstalled();
    out(
        `${ghOk ? PASS : FAIL} gh installed${ghOk ? "" : " — see https://cli.github.com"}`,
    );
    ok &&= ghOk;

    // accounts
    let accounts = [] as Awaited<ReturnType<Gh["status"]>>;
    if (ghOk) {
        try {
            accounts = await gh.status();
            if (accounts.length === 0) {
                out(`${FAIL} no gh accounts — run \`gh auth login\``);
                ok = false;
            } else {
                out(`${INFO} gh accounts:`);
                for (const a of accounts) {
                    out(
                        `    - ${a.user} @ ${a.host}${a.active ? " (active)" : ""}`,
                    );
                }
            }
        } catch (err) {
            out(
                `${FAIL} gh auth status failed: ${err instanceof Error ? err.message : err}`,
            );
            ok = false;
        }
    }

    // config
    try {
        const config = await loadConfig(cwd);
        if (config === null) {
            out(`${INFO} no .gitwrapper found — git runs unchanged here`);
        } else {
            out(`${INFO} .gitwrapper: ${config.sourcePath}`);
            out(`    account: ${config.account} @ ${config.host}`);
            const known = accounts.some(
                (a) =>
                    a.host.toLowerCase() === config.host.toLowerCase() &&
                    a.user.toLowerCase() === config.account.toLowerCase(),
            );
            if (ghOk && accounts.length > 0) {
                out(
                    known
                        ? `    ${PASS} account is logged in`
                        : `    ${FAIL} account not logged in — run \`gh auth login --hostname ${config.host}\``,
                );
                ok &&= known;
            }
        }
    } catch (err) {
        if (err instanceof ConfigError) {
            out(`${FAIL} .gitwrapper invalid (${err.path}): ${err.message}`);
            ok = false;
        } else if (err instanceof GhError) {
            out(`${FAIL} ${err.message}`);
            ok = false;
        } else {
            throw err;
        }
    }

    return ok ? 0 : 1;
}
