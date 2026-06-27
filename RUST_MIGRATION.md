# v1: Migrate `gw` from Bun/TypeScript to Rust

## Why

- **Binary size.** The Bun `--compile` artifact is ~61 MB (the embedded
  runtime). A statically-linked Rust release binary for this tool will be
  **~1–2 MB** — a >30× reduction, and the original motivation here.
- **Startup + footprint.** No runtime to boot; native process spawn.
- **Distribution.** A small native binary is trivial to ship and install
  (`cargo-dist` generates installers); no runtime assumptions.
- **Fit.** `gw` is almost entirely "parse a small JSON file, shell out to
  `git`/`gh`, move bytes around." That is squarely in Rust's comfort zone and
  uses no Bun/Node-specific APIs that would be hard to replace.

This is a **clean reimplementation**, not incremental interop — TS and Rust
don't share a runtime. The existing Bun binary stays the behavioural oracle
until parity is proven, then the Bun toolchain is removed.

## Hard requirement: behavioural parity

v1 must be a drop-in replacement. The contract to preserve:

- **CLI surface:** `gw <git args…>`, `gw doctor`, `gw which`,
  `gw init [account] [--no-gitignore]`, `gw --gw-version`, `gw --gw-help`.
- **Transparent passthrough:** anything not a meta-command is forwarded to
  `git` verbatim, with inherited stdio, forwarded signals, and git's exit code.
- **`.gitwrapper` format** (unchanged, camelCase keys): `account` (required),
  `host` (default `github.com`), `userName`, `userEmail`, `restorePrevious`
  (default `true`). Discovered by walking up from cwd.
- **Semantics:** case-insensitive account match; switch → run → restore
  previous account; skip switch/restore when already active; repo-local
  identity only inside a work tree; SSH-remote warning; no-config →
  transparent passthrough; all wrapper chatter on **stderr**, git's stdout
  untouched.

The current TS unit + integration tests are the conformance spec; each is
ported to Rust so the same cases gate the rewrite.

## Module mapping (TS → Rust)

| TypeScript | Rust | Notes |
|---|---|---|
| `src/cli.ts` | `src/main.rs` + `src/cli.rs` | arg routing + dispatch; `main` returns `ExitCode` |
| `src/types.ts` | `src/types.rs` | `Config`, `Account`, `RunResult`; `Runner` trait |
| `src/runner.ts` | `src/runner.rs` | `Runner` trait + real impl over `std::process::Command` |
| `src/config.ts` | `src/config.rs` | serde structs + walk-up discovery + `ConfigError` |
| `src/gh.ts` | `src/gh.rs` | `Gh` over a `Runner`; serde for `--json hosts`; text fallback |
| `src/git.ts` | `src/git.rs` | passthrough spawn, install/work-tree checks, identity, SSH detect |
| `src/core.ts` | `src/core.rs` | `run_wrapped` orchestration |
| `src/commands/{doctor,which,init}.ts` | `src/commands/{doctor,which,init}.rs` | one module each |
| `src/log.ts` | `src/log.rs` | stderr `notice`/`warn`/`error`, `NO_COLOR`-aware |
| `test/*.test.ts` | `tests/*.rs` + `#[cfg(test)]` | integration via `assert_cmd`; units inline |

## Crate choices

- **serde** + **serde_json** — config + `gh --json` parsing.
  `#[serde(rename_all = "camelCase")]`; `#[serde(default = …)]` for `host` and
  `restorePrevious` defaults.
- **anyhow** (app-level errors, ergonomic `?`) + **thiserror** (typed
  `ConfigError`/`GhError` to preserve the existing match-on-variant flow).
- **Arg parsing:** keep it minimal to guarantee passthrough fidelity. Two
  options:
  - **`clap`** (derive) with `allow_external_subcommands` + `trailing_var_arg`
    so unknown subcommands/flags flow to git untouched; or
  - **`lexopt`/`pico-args`** + a hand-rolled `match` on the first token
    (closest to today's `cli.ts`).
  Recommendation: **hand-rolled match** — `gw`'s routing is tiny and clap's
  passthrough edge cases (e.g. `gw --version` vs `gw --gw-version`, `gw push -f`)
  are exactly where a parser fights us.
- **TTY check:** `std::io::IsTerminal` (stable, no crate).
- **Colours:** `anstream` + `owo-colors`, or hand-rolled ANSI with a
  `NO_COLOR`/`isatty` guard (matches current `log.ts`).
- **Interactive `init` picker:** `dialoguer`, or a plain stdin read to stay
  dependency-light.
- **Dev/test:** `assert_cmd` + `predicates` (CLI integration), `tempfile`
  (temp dirs, replacing `mkdtemp`).

## Key technical risks & how they're handled

1. **Passthrough fidelity.** Spawn `git` with `Stdio::inherit()` for all three
   streams; propagate `status.code()`. The bulk of the parity tests target this.
2. **Signals (Ctrl-C).** With inherited stdio on a terminal, SIGINT is
   delivered by the kernel to the whole foreground process group, so the child
   already receives it — simpler than Node, which intercepts signals and forced
   the manual forwarding in `runner.ts`. The parent just `wait`s. On Windows,
   `Command` inheritance covers Ctrl-C similarly. Verify with an interactive
   `git rebase -i`/commit-editor test.
3. **Exit-code/signal-death mapping.** Mirror the shell `128 + signo`
   convention on Unix via `ExitStatusExt::signal()`.
4. **gh JSON drift.** Port both the `--json hosts` path and the human-readable
   fallback; reuse the exact account fixtures from `gh.test.ts`.
5. **serde defaults vs. strict errors.** `.gitwrapper` must still hard-error on
   malformed JSON / missing `account`, but apply defaults for the optionals —
   covered by porting `config.test.ts`.
6. **Global `gh` state + restore.** Identical ordering: record active → switch →
   run → restore unless already-active / `restorePrevious:false`.

## Build, size, and release

- **Release profile** (in `Cargo.toml`) to hit the size goal:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  ```
- **Cross-compilation:** the same five targets via `cross` (or
  `cargo build --target …`): linux x64/arm64 (musl for static), darwin
  x64/arm64, windows x64.
- **Release automation:** **`cargo-dist`** generates the GitHub Actions release
  workflow, builds per-target archives, and produces shell/PowerShell
  installers + a Homebrew tap — superseding the current `build:all` +
  `action-gh-release` setup, and keeping the "version bump → release" trigger.
- **CI:** replace Biome/tsc/`bun test` with `cargo fmt --check`,
  `cargo clippy -- -D warnings`, and `cargo test` (matrix over the OSes).

## Phased plan (each phase ends green: builds, clippy-clean, tests pass)

1. **Scaffold** — `cargo init --name gw`, `Cargo.toml` (deps + release
   profile), `rustfmt.toml`, `clippy` in CI skeleton, module stubs, a `main`
   that routes `--gw-version`/`--gw-help` and passes everything else to `git`.
2. **Config** — serde structs, walk-up discovery, `ConfigError`; port
   `config.test.ts`. Implement `gw which`.
3. **Runner + git + gh** — `Runner` trait (mockable like today), real impl;
   git helpers and the `Gh` wrapper with JSON + text parsing; port `gh.test.ts`.
4. **Core orchestration** — `run_wrapped` (preflight → resolve → switch →
   identity → SSH warn → passthrough → restore); port `core.test.ts` with a
   fake `Runner`.
5. **Meta-commands** — `doctor`, `init` (incl. `--no-gitignore` + `.gitignore`
   handling), help; port `doctor.test.ts` / `init.test.ts`.
6. **Integration + parity** — `assert_cmd` tests mirroring
   `integration.test.ts`; differential check: run the same arg sets through the
   retained Bun binary and the Rust binary, diff stdout/stderr/exit code.
7. **Release tooling + docs** — `cargo-dist` init, CI swap, README install
   rewrite (now a ~1–2 MB binary), measure actual size.
8. **Cutover** — delete Bun/TS sources, `biome.json`, `tsconfig.json`,
   `bun.lock`, `package.json`, `scripts/`; bump to `1.0.0`; merge `v1` → `main`.

## Open questions

1. **Arg routing:** hand-rolled match (recommended) vs. `clap` with external
   subcommands?
2. **MSRV / toolchain pin** and whether to commit `Cargo.lock` (yes for a
   binary).
3. **musl static Linux builds** (portable, recommended) vs. glibc?
4. **`cargo-dist`** for releases vs. a hand-written Actions matrix?
5. Keep the package name/scope identity (`@acolville/gw` was npm-only; the Rust
   crate/binary is just `gw`) — any crates.io publish, or GitHub Releases only?

## Prerequisites

- Rust toolchain (`rustup`) — **not currently installed** on this machine;
  needed before Phase 1.
- `cross` (or Docker) for Linux cross-builds from macOS.
