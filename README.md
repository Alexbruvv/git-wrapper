# `gw` — GitWrapper

Wraps `git` so the right **GitHub CLI account** is active before every command.
Declare the account once per project in a `.gitwrapper` file; run `gw` instead
of `git` and stop pushing with the wrong identity.

```bash
gw push            # switch to this project's gh account, then `git push`
gw commit -m "..."   # same, then `git commit`
```

`gw` is a small (~1–2 MB) self-contained binary written in Rust — no runtime
needed.

## Install

Download the archive for your OS/arch from the
[Releases](https://github.com/alexbruvv/git-wrapper/releases) page
(`.tar.gz`, or `.zip` on Windows), extract it, and put `gw` on your `PATH`:

```bash
# example: macOS arm64
curl -L https://github.com/alexbruvv/git-wrapper/releases/latest/download/gw-darwin-arm64.tar.gz | tar xz
chmod +x gw && mv gw /usr/local/bin/gw
```

Requires [`git`](https://git-scm.com) and the
[GitHub CLI (`gh`)](https://cli.github.com) on your `PATH`, with each account
logged in via `gh auth login`.

### Mise

You can use [Mise](https://mise.jdx.dev/) to install git-wrapper easily.
Run `mise use -g github:alexbruvv/git-wrapper`

## `.gitwrapper`

Place a `.gitwrapper` file at the root of a project (it's discovered by walking
up from your working directory):

```jsonc
{
    "account": "alexbruvv", // required: gh username to switch to
    "host": "github.com", // optional, default "github.com"
    "userName": "Alex Colville", // optional: repo-local git user.name
    "userEmail": "me@acolville.co.uk", // optional: repo-local git user.email
    "restorePrevious": true, // optional: restore prior account after (default true)
}
```

When no `.gitwrapper` is found, `gw` is a transparent passthrough to `git`.

## Commands

| Command                    | Description                                              |
| -------------------------- | -------------------------------------------------------- |
| `gw <git args…>`           | Switch account, then run git                             |
| `gw doctor`                | Diagnose git/gh install and auth state                   |
| `gw which`                 | Show the account/host resolved for the current directory |
| `gw init [account]`        | Scaffold a `.gitwrapper` file and add it to `.gitignore` |
| `gw init … --no-gitignore` | Scaffold without touching `.gitignore`                   |
| `gw --gw-version`          | Print version                                            |
| `gw --gw-help`             | Show help                                                |

## Caveats

- Switching changes **global** `gh` state; `gw` restores the previous account
  afterward by default.
- Account switching covers **HTTPS** remotes (via gh's git credential helper).
  SSH key switching is planned for a later release.

## Develop

Built and tested with [Rust](https://www.rust-lang.org) (stable):

```bash
cargo build              # debug binary at target/debug/gw
cargo test               # unit + integration tests
cargo clippy -- -D warnings
cargo fmt
cargo run -- which       # run from source
cargo build --release    # ~1–2 MB release binary
```

CI runs `cargo fmt --check`, `clippy -D warnings`, and `cargo test`. Releases are
automatic: bump the `version` in `Cargo.toml` and merge to `main`, and the
release workflow tags `vX.Y.Z`, cross-compiles the per-platform binaries, and
attaches the archives to a GitHub Release.

## License

GPL

