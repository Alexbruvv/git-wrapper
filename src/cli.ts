import { runWrapped } from "./core.js";
import { doctor } from "./commands/doctor.js";
import { which } from "./commands/which.js";
import { init } from "./commands/init.js";
import * as log from "./log.js";
// Embedded at build time by `bun build --compile`, so the standalone binary
// carries its own version without a package.json on disk.
import pkg from "../package.json";

function readVersion(): string {
  return pkg.version;
}

const HELP = `gw — wraps git, switching the active GitHub CLI account per project.

Usage:
  gw <git args...>     Switch to the project's account, then run git
  gw doctor            Diagnose git/gh install and auth state
  gw which             Show the account/host resolved for this directory
  gw init [account]    Scaffold a .gitwrapper file
  gw --gw-version      Print gw's version
  gw --gw-help         Show this help

Anything that is not a gw meta-command is passed straight through to git.
`;

export async function main(argv: string[]): Promise<number> {
  const [first] = argv;

  switch (first) {
    case "--gw-version":
      process.stdout.write(readVersion() + "\n");
      return 0;
    case "--gw-help":
      process.stdout.write(HELP);
      return 0;
    case "doctor":
      return doctor();
    case "which":
      return which();
    case "init":
      return init(argv.slice(1));
  }

  // Switch to the project's gh account, run git, then restore.
  return runWrapped(argv);
}

main(process.argv.slice(2))
  .then((code) => {
    process.exitCode = code;
  })
  .catch((err) => {
    log.error(err instanceof Error ? err.message : String(err));
    process.exitCode = 1;
  });
