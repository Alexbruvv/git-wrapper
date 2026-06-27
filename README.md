# `gw` — GitWrapper

Wraps `git` so the right **GitHub CLI account** is active before every command.
Declare the account once per project in a `.gitwrapper` file; run `gw` instead
of `git` and stop pushing with the wrong identity.

```bash
gw push            # switch to this project's gh account, then `git push`
gw commit -m "…"   # same, then `git commit`
```

> Status: **complete and verified, awaiting first release.** Behaviour, tests,
> and CI are in place; `gw` ships as a self-contained binary built with
> [Bun](https://bun.sh) — no Node or Bun needed at runtime. Only cutting the
> first GitHub Release is outstanding. See [PLAN.md](PLAN.md) for the roadmap.

## Install

`gw` is distributed as a standalone executable per platform. Download the one
for your OS/arch from the [Releases](https://github.com/alexbruvv/git-wrapper/releases)
page, mark it executable, and put it on your `PATH`:

```bash
# example: macOS arm64
curl -L -o gw https://github.com/alexbruvv/git-wrapper/releases/latest/download/gw-darwin-arm64
chmod +x gw && mv gw /usr/local/bin/gw
```

Requires [`git`](https://git-scm.com) and the
[GitHub CLI (`gh`)](https://cli.github.com) on your `PATH`, with each account
logged in via `gh auth login`.

## `.gitwrapper`

Place a `.gitwrapper` file at the root of a project (it's discovered by walking
up from your working directory):

```jsonc
{
  "account": "alexbruvv",            // required: gh username to switch to
  "host": "github.com",              // optional, default "github.com"
  "userName": "Alex Colville",       // optional: repo-local git user.name
  "userEmail": "me@acolville.co.uk", // optional: repo-local git user.email
  "restorePrevious": true            // optional: restore prior account after (default true)
}
```

When no `.gitwrapper` is found, `gw` is a transparent passthrough to `git`.

## Commands

| Command | Description |
|---|---|
| `gw <git args…>` | Switch account, then run git |
| `gw doctor` | Diagnose git/gh install and auth state |
| `gw which` | Show the account/host resolved for the current directory |
| `gw init` | Scaffold a `.gitwrapper` file |
| `gw --gw-version` | Print version |
| `gw --gw-help` | Show help |

## Caveats

- Switching changes **global** `gh` state; `gw` restores the previous account
  afterward by default.
- Account switching covers **HTTPS** remotes (via gh's git credential helper).
  SSH key switching is planned for a later release.

## Develop

Built and tested with [Bun](https://bun.sh):

```bash
bun install
bun run typecheck      # tsc --noEmit
bun run build          # compile the standalone ./gw binary
bun test               # 32 tests; integration runs against the compiled binary
bun run build:all      # cross-compile dist/gw-<platform> for all targets
bun run dev -- which   # run from source without compiling
```

CI runs typecheck + build + tests on Bun. Tagging `vX.Y.Z` triggers the release
workflow, which cross-compiles binaries and attaches them to a GitHub Release.

## License

MIT
