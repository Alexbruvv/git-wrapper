import { ConfigError, loadConfig } from "../config.js";
import * as log from "../log.js";

/**
 * Resolve the nearest `.gitwrapper` and report the account/host that would be
 * used for the current directory, plus where the config was found.
 */
export async function which(cwd: string = process.cwd()): Promise<number> {
    let config: Awaited<ReturnType<typeof loadConfig>>;
    try {
        config = await loadConfig(cwd);
    } catch (err) {
        if (err instanceof ConfigError) {
            log.error(`${err.path}: ${err.message}`);
            return 1;
        }
        throw err;
    }

    if (config === null) {
        process.stdout.write(
            "No .gitwrapper found; git commands run unchanged.\n",
        );
        return 0;
    }

    const lines = [`account: ${config.account}`, `host:    ${config.host}`];
    if (config.userName) lines.push(`name:    ${config.userName}`);
    if (config.userEmail) lines.push(`email:   ${config.userEmail}`);
    lines.push(`restore: ${config.restorePrevious}`);
    lines.push(`config:  ${config.sourcePath}`);
    process.stdout.write(`${lines.join("\n")}\n`);
    return 0;
}
