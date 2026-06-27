# GitWrapper (`gw`) — Implementation Plan

## 1. Purpose

`gw` is a thin wrapper around `git` that, before running any git command,
switches the active GitHub CLI (`gh`) account to the one configured for the
current project. This lets a developer who juggles several GitHub identities
(personal, work, client) stop manually running `gh auth switch` and stop
pushing with the wrong account.

```
gw push          # → switch to project's account, then `git push`
gw commit -m ..  # → switch to project's account, then `git commit -m ..`
```

The project's account is declared in a per-project `.gitwrapper` file.

## 2. Resolved decisions

- **Package name:** scoped — `@acolville/gw` (bin: `gw`).
- **No `.gitwrapper` found:** transparent passthrough — print a dim notice and
  run `git` unchanged. (No strict mode in v1.)
- **Restore previous account after the command:** default **on**.
- **SSH remote key switching:** deferred to v2. v1 detects SSH remotes and warns
  that gh switching only covers HTTPS credentials.
- **Toolchain & distribution:** [Bun](https://bun.sh). `bun install` for deps,
  `bun test` (built-in runner, no vitest), `tsc --noEmit` for typecheck. Ships as
  a self-contained executable via `bun build --compile` (`bun run build` →
  `./gw`; `bun run build:all` → `dist/gw-<platform>`), distributed through
  GitHub Releases rather than npm. No Node or Bun needed at runtime. The version
  is embedded at build time via a bundled `package.json` import.

## 3. How account switching works

`gh` is the source of truth for authentication. When `gh auth setup-git` has
been run, `gh` acts as git's HTTPS credential helper, so the **active** `gh`
account determines which credentials git uses for `push`/`pull`/`fetch`.

- Switch the active account: `gh auth switch --hostname <host> --user <user>`
- List logged-in accounts / active one: `gh auth status`
- Confirm gh is a git credential helper: `gh auth setup-git` (idempotent)

`gh auth switch` mutates **global** gh state, affecting other open shells, so
`gw` records the previously active account and restores it after the git
command finishes (restore default on).

## 4. The `.gitwrapper` file

Discovered by walking up from the current directory to the filesystem root
(first match wins), so it works from any subdirectory of a project. Format is
JSON.

```jsonc
{
  "account": "alexbruvv",            // required: gh username to switch to
  "host": "github.com",              // optional, default "github.com"
  "userName": "Alex Colville",       // optional: set repo-local git user.name
  "userEmail": "me@acolville.co.uk", // optional: set repo-local user.email
  "restorePrevious": true            // optional, default true
}
```

If `userName`/`userEmail` are present, `gw` sets them via `git config --local`
so commits are attributed correctly too.

## 5. Execution flow

1. Parse argv. Meta-commands (`doctor`, `which`, `init`, `--gw-help`,
   `--gw-version`) are handled and exit. Otherwise all args pass through to git.
2. Preflight: `git` installed; `gh` installed (`gh --version`).
3. Resolve config: nearest `.gitwrapper`. Not found → transparent passthrough.
4. Validate the target account is logged into gh (`gh auth status`).
5. Record currently active account (for restore).
6. Switch (`gh auth switch …`), skipping if already active; ensure
   `gh auth setup-git` has wired the credential helper.
7. If configured, apply repo-local `user.name` / `user.email`.
8. Spawn `git <args>` with inherited stdio; forward signals.
9. Capture git's exit code.
10. If `restorePrevious` and the account was changed, switch gh back.
11. Exit with git's original exit code.

## 6. CLI surface

| Command | Description |
|---|---|
| `gw <git args…>` | Switch account, run `git <git args…>` |
| `gw doctor` | Check `git`/`gh`, auth status, list accounts, validate nearest config |
| `gw which` | Show resolved account/host + config path for the current dir |
| `gw init` | Scaffold a `.gitwrapper` (pick from logged-in gh accounts) |
| `gw --gw-help` | Wrapper help |
| `gw --gw-version` | Wrapper version |

---

# Phases

## Phase 1 — Scaffold ✅ (this phase)

Stand up the project skeleton so later phases just fill in logic. (Originally
scaffolded on a Node/npm/vitest/tsc toolchain; later migrated to Bun — see
Resolved decisions. The bullets below describe the original scaffold.)

- `package.json`: name `@acolville/gw`, `"type": "module"`, `bin.gw`,
  `engines.node >=18`, `files`, scripts (`build`, `dev`, `test`, `lint`,
  `prepublishOnly`), `publishConfig.access=public`.
- `tsconfig.json`: ESM (`NodeNext`), `outDir dist`, strict.
- `bin/gw.js`: shebang shim that imports `dist/cli.js`.
- `src/` module stubs with typed signatures and `TODO`s:
  `cli.ts`, `config.ts`, `gh.ts`, `git.ts`, `runner.ts`, `log.ts`,
  `commands/{doctor,which,init}.ts`.
- `src/types.ts`: `GitWrapperConfig`, `Account`, runner types.
- A minimal working `cli.ts`: routes `--gw-version`/`--gw-help` and otherwise
  passes through to `git` (no switching yet) — proves the binary runs.
- `test/` with one smoke test; `vitest` configured.
- `.gitignore`, `.npmignore`/`files`, `README.md` skeleton, `LICENSE` (MIT),
  `.gitwrapper` for this repo itself (dogfooding).
- `git init` and an initial commit.

**Done when:** `npm install && npm run build` succeeds and
`node bin/gw.js --gw-version` prints the version; `node bin/gw.js status`
runs `git status` via passthrough.

## Phase 2 — Config discovery & parsing ✅

- `config.ts`: walk-up search for `.gitwrapper`, JSON parse, schema validation
  with clear errors, defaults (`host`, `restorePrevious`).
- `gw which` becomes real (prints resolved account + config path).
- Unit tests for discovery (nested dirs, missing file, malformed JSON).

## Phase 3 — gh integration & git passthrough ✅

- `runner.ts`: mockable exec/spawn wrapper.
- `gh.ts`: `isInstalled`, `status()` parse (accounts + active), `switch()`,
  `setupGit()`.
- `git.ts`: passthrough spawn with inherited stdio, signal forwarding, exit code.
- Wire the full execution flow in `cli.ts` (preflight → switch → run → restore).
- Repo-local `user.name`/`user.email` application.
- SSH-remote detection + warning.

## Phase 4 — Meta-commands ✅

- `gw doctor`: environment + auth diagnostics.
- `gw init`: interactive scaffold from logged-in accounts.
- `gw --gw-help`: full help text.

## Phase 5 — Tests & docs ✅

- Unit tests across modules with mocked runner; one gated real-binary
  integration test (self-skips without a build).
- README: install, `.gitwrapper` schema, examples, caveats (global state, SSH).

## Phase 6 — Release ✅ (verified; release itself deferred)

- `bun run build` compiles a 61 MB standalone `gw` (Mach-O arm64 here); runs with
  no Node/Bun present (`--gw-version`, `--gw-help`, real switch/restore).
- `bun run build:all` cross-compiles all five targets into `dist/`
  (linux x64/arm64, darwin x64/arm64, windows x64).
- GitHub Actions: `ci.yml` (typecheck + build + `bun test` on Bun), `release.yml`
  (cross-compile + attach binaries to a GitHub Release on a `vX.Y.Z` tag).
- **Outstanding:** cutting the first release tag — intentionally not done.

## 7. Edge cases (tracked across phases)

- No `.gitwrapper` → transparent passthrough + notice.
- Account not in gh → hard error with `gh auth login` remediation.
- `gh` missing → hard error with install link.
- SSH remote → warn (v1).
- Already on the right account → skip switch + skip restore.
- git fails → still restore; propagate exit code.
- Interactive git (editors) → inherited stdio, unbuffered.
- Concurrent `gw` invocations race on global gh state → documented; lock file is
  possible v2 hardening.
