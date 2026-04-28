# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**English | [日本語](./README.ja.md)**

> direnv-style automatic Claude Code account switching, built around [ghq](https://github.com/x-motemen/ghq). The right account, just by `cd`-ing in.

`ccdirenv` selects the correct Claude Code account based on the current directory. The primary workflow is **ghq-aware**: every repository you `ghq get` under a configured GitHub owner (or any host/owner pair) automatically resolves to the matching profile — no per-repo setup, no shell wrappers.

## Why

If you use ghq, your repositories already live under a predictable layout: `~/ghq/<host>/<owner>/<repo>`. ccdirenv hooks into that layout. Map an owner to a profile once, and every present-and-future repository under that owner uses the right Claude Code account automatically. Personal repos go to your personal account, work repos go to your work account, client repos go to the client account.

ghq isn't required — directory globs and per-repo markers still work — but the project assumes ghq as the default mental model and is optimized for it.

## How it works

1. Each Claude Code account lives in an isolated profile directory under `~/.ccdirenv/profiles/<name>/`.
2. A small Rust shim placed earlier in `PATH` resolves the profile for the current directory and sets `CLAUDE_CONFIG_DIR` before handing off to the real `claude` binary.
3. Resolution order (first match wins):
   1. `CCDIRENV_PROFILE` environment variable (force override)
   2. `.ccdirenv` marker file in the current directory or any parent
   3. `~/.ccdirenv/config.toml` glob patterns under `[directories]`
   4. **`[ghq.owners]` mapping (`<host>/<owner>` lookup against the ghq root)**
   5. `default_profile` from `config.toml`, falling back to `default`

Claude Code's own installation and auto-update path are never touched — the shim only adds an entry earlier in `PATH` and execs whatever `claude` it finds after stripping itself.

## Install

```sh
# crates.io (recommended)
cargo install ccdirenv

# Or grab a prebuilt binary from a GitHub Release (macOS / Linux):
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/SuguruOoki/ccdirenv/releases/latest/download/ccdirenv-installer.sh | sh

# Homebrew (after the tap is published)
brew tap SuguruOoki/tap
brew install ccdirenv
```

This project assumes you have [ghq](https://github.com/x-motemen/ghq) installed. **`ccdirenv init` will install ghq for you** if it's not already on `PATH` — it tries Homebrew first, then `go install`, and falls back to printing the manual instructions if neither is available. You can re-run the check at any time with `ccdirenv ghq install`.

To skip the auto-install (useful in CI), set `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1`.

## Setup

```sh
# 1. Create the profile directory layout and install the shim at ~/.ccdirenv/bin/claude.
ccdirenv init

# 2. Add this line to your shell rc (~/.zshrc, ~/.bashrc, etc.) — ccdirenv init prints the exact instruction:
#    export PATH="$HOME/.ccdirenv/bin:$PATH"

# 3. (Optional) Migrate your existing ~/.claude/ login into the 'default' profile.
ccdirenv import default

# 4. Create and log into additional profiles.
ccdirenv login work
ccdirenv login personal

# 5. Map ghq owners to profiles.
ccdirenv ghq map github.com/your-personal-handle  default
ccdirenv ghq map github.com/your-employer         work
ccdirenv ghq map github.com/a-client-org          client-acme
```

That's it. Every repository under those owners now uses the right Claude Code account the moment you `cd` into it.

## ghq integration (the primary workflow)

```toml
# ~/.ccdirenv/config.toml
default_profile = "default"

[ghq.owners]
"github.com/SuguruOoki" = "default"
"github.com/TheMoshInc" = "mosh"
"github.com/AcmeCorp"   = "work"
```

The ghq root is auto-detected: `$GHQ_ROOT` if set, otherwise `~/ghq` (the ghq default). Override it with `[ghq] root = "/custom/path"` if you keep ghq elsewhere.

CLI:

```sh
ccdirenv ghq list                                # show current mappings + ghq root
ccdirenv ghq map github.com/Acme work            # add or update a mapping
ccdirenv ghq unmap github.com/Acme               # remove a mapping
ccdirenv ghq root /custom/ghq                    # override ghq root
ccdirenv ghq root ""                             # clear the override (back to $GHQ_ROOT / ~/ghq)
```

Subdirectories of a matched repo also resolve to the same profile, so `cd ~/ghq/github.com/Acme/widget/src/lib` still picks `work`.

### Per-repo overrides

Sometimes one repo under a mapped owner needs a different profile (a fork, a sandbox, etc.). Two ways:

```sh
# Option A: pin a single repo via marker file (highest priority).
cd ~/ghq/github.com/Acme/sandbox && ccdirenv use personal

# Option B: write an explicit glob in config.toml that beats the ghq mapping.
```

```toml
[directories]
"~/ghq/github.com/Acme/sandbox/**" = "personal"
```

## Other useful commands

```sh
ccdirenv which            # which profile applies to the current directory
ccdirenv list             # list profiles and their logged-in email
ccdirenv use <profile>    # bind cwd to a profile via .ccdirenv marker
ccdirenv unuse            # remove the marker
ccdirenv config           # open ~/.ccdirenv/config.toml in $EDITOR
ccdirenv doctor           # diagnostics (PATH order, real claude resolvability, etc.)
```

Once the shim is on `PATH`, running `claude` in any directory automatically launches against the resolved profile.

## Environment variables

| Variable | Effect |
|---|---|
| `CCDIRENV_PROFILE=<name>` | Force a specific profile for this invocation, ignoring marker / config / ghq. |
| `CCDIRENV_DISABLE=1` | Skip resolution entirely — the shim transparently execs the real `claude` with the inherited `CLAUDE_CONFIG_DIR`. Useful for debugging. |
| `CCDIRENV_DEBUG=1` | Print the chosen profile, profile dir, and real claude path to stderr before exec. |
| `CCDIRENV_HOME=<path>` | Override `~/.ccdirenv/` as the data root (useful for testing). |
| `GHQ_ROOT=<path>` | Honoured for ghq root detection when `[ghq] root` is not set. |

## Troubleshooting

Run `ccdirenv doctor`. It reports:

- whether the shim is installed at `~/.ccdirenv/bin/claude`
- whether `PATH` includes `~/.ccdirenv/bin`
- whether a non-shim `claude` binary is reachable on `PATH`
- whether `config.toml` exists

If the shim is installed but `claude` keeps picking up the wrong account, make sure `~/.ccdirenv/bin` comes *before* `~/.local/bin` (or wherever Claude Code installed itself) in your `PATH`.

If a ghq-rooted directory unexpectedly resolves to `default`, run `ccdirenv ghq list` and confirm the owner mapping is registered, and that your ghq root matches what ccdirenv detects.

## Uninstall

```sh
cargo uninstall ccdirenv
rm -rf ~/.ccdirenv/bin
# Remove the PATH line from your shell rc.
# Your profiles under ~/.ccdirenv/profiles/ are left alone — delete them manually if you want.
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
