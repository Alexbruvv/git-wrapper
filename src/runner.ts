import { spawn } from "node:child_process";
import type { Runner, RunResult } from "./types.js";

/** Default runner backed by `child_process.spawn`. */
export const realRunner: Runner = {
    capture(cmd: string, args: string[]): Promise<RunResult> {
        return new Promise((resolve, reject) => {
            const child = spawn(cmd, args, {
                stdio: ["ignore", "pipe", "pipe"],
            });
            let stdout = "";
            let stderr = "";
            child.stdout.on("data", (d) => (stdout += d.toString()));
            child.stderr.on("data", (d) => (stderr += d.toString()));
            child.on("error", reject);
            child.on("close", (code) =>
                resolve({ code: code ?? 1, stdout, stderr }),
            );
        });
    },

    passthrough(cmd: string, args: string[]): Promise<number> {
        return new Promise((resolve, reject) => {
            const child = spawn(cmd, args, { stdio: "inherit" });
            // Forward termination signals so Ctrl-C reaches the wrapped process.
            const forward = (sig: NodeJS.Signals) => child.kill(sig);
            process.on("SIGINT", forward);
            process.on("SIGTERM", forward);
            child.on("error", reject);
            child.on("close", (code, signal) => {
                process.off("SIGINT", forward);
                process.off("SIGTERM", forward);
                if (signal) {
                    // Mirror shell convention: 128 + signal number.
                    resolve(128 + (osSignalNumber(signal) ?? 0));
                } else {
                    resolve(code ?? 1);
                }
            });
        });
    },
};

function osSignalNumber(signal: NodeJS.Signals): number | undefined {
    const map: Partial<Record<NodeJS.Signals, number>> = {
        SIGINT: 2,
        SIGTERM: 15,
        SIGKILL: 9,
    };
    return map[signal];
}
