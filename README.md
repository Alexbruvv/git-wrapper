# `gw` — GitWrapper

Wraps `git` so the right **GitHub CLI account** is active before every command.
Declare the account once per project in a `.gitwrapper` file; run `gw` instead
of `git` and stop pushing with the wrong identity.

```bash
gw push            # switch to this project's gh account, then `git push`
gw commit -m "…"   # same, then `git commit`
```

> Status: **functional, pre-publish.** Account switching, restore, the
> `.gitwrapper` format, and the `doctor`/`which`/`init` commands all work
> (phases 1–4). Remaining: packaging polish and the first npm publish (phases
> 5–6). See [PLAN.md](PLAN.md) for the roadmap.

## Install

```bash
npm install -g @acolville/gw
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

```bash
npm install
npm run build
npm test
```

## License

MIT
