// Minimal logging helpers. All wrapper chatter goes to stderr so it never
// pollutes the stdout of the git command being wrapped.

const useColor = process.stderr.isTTY && process.env.NO_COLOR === undefined;

const dim = (s: string) => (useColor ? `\x1b[2m${s}\x1b[0m` : s);
const yellow = (s: string) => (useColor ? `\x1b[33m${s}\x1b[0m` : s);
const red = (s: string) => (useColor ? `\x1b[31m${s}\x1b[0m` : s);

/** Low-key informational line (e.g. "switched to account X"). */
export function notice(message: string): void {
  process.stderr.write(dim(`gw: ${message}`) + "\n");
}

export function warn(message: string): void {
  process.stderr.write(yellow(`gw: warning: ${message}`) + "\n");
}

export function error(message: string): void {
  process.stderr.write(red(`gw: error: ${message}`) + "\n");
}
