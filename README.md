# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**English | [日本語](./README.ja.md)**

> direnv-style automatic Claude Code account switching. The right account, just by `cd`-ing in. Works whether you use [ghq](https://github.com/x-motemen/ghq), plain git, or both.

`ccdirenv` selects the correct Claude Code account based on the current directory. Map a GitHub owner to a profile once, and every repository under that owner — present and future — uses the right Claude Code account automatically. Personal repos go to your personal account, work repos go to your work account, client repos go to the client account.

## Two ways to find the right repo

You pick at install time:

- **git mode (default)** — every repository with a git remote works, anywhere on disk. ccdirenv reads `.git/config` directly (no `git` subprocess), follows worktrees and submodules, and looks up `<host>/<owner>` from `origin`.
- **ghq mode** — fastest. ccdirenv extracts `<host>/<owner>` from the path layout `~/ghq/<host>/<owner>/<repo>` without touching any files. Recommended if you already use ghq.
- **both** — try one method, fall back to the other. Useful if some repos live under ghq and others don't.
- **off** — no automatic detection; only `[directories]` globs and `.ccdirenv` markers.

The same shared owner→profile map (`[owners]`) powers all modes.

## How it works

1. Each Claude Code account lives in an isolated profile directory under `~/.ccdirenv/profiles/<name>/`.
2. A small Rust shim placed earlier in `PATH` resolves the profile for the current directory and sets `CLAUDE_CONFIG_DIR` before handing off to the real `claude` binary.
3. Resolution order (first match wins):
   1. `CCDIRENV_PROFILE` environment variable (force override)
   2. `.ccdirenv` marker file in the current directory or any parent
   3. `~/.ccdirenv/config.toml` glob patterns under `[directories]`
   4. **owner discovery** (git and/or ghq, in `discovery_priority` order) → `[owners]` lookup
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

### ghq users

If you pick ghq mode (or both) at install time, `ccdirenv init` will install [ghq](https://github.com/x-motemen/ghq) automatically when it's missing from `PATH` — Homebrew first, then `go install`, falling back to printing manual instructions. You can re-run the check at any time with `ccdirenv ghq install`. Skip the auto-install (e.g. in CI) with `CCDIRENV_SKIP_GHQ_AUTOINSTALL=1`.

### git users

git mode requires nothing beyond `git` itself. Worktrees (`.git` as a file) and submodules are supported.

## Setup

```sh
# 1. Create the profile directory layout, install the shim, pick a discovery mode.
#    `ccdirenv init` is interactive; pass --mode=git|ghq|both|off to skip the prompt.
ccdirenv init

# 2. Add this line to your shell rc (~/.zshrc, ~/.bashrc, etc.) — ccdirenv init prints the exact instruction:
#    export PATH="$HOME/.ccdirenv/bin:$PATH"

# 3. (Optional) Migrate your existing ~/.claude/ login into the 'default' profile.
ccdirenv import default

# 4. Create and log into additional profiles.
ccdirenv login work
ccdirenv login personal

# 5. Map GitHub owners to profiles (used by both ghq and git modes).
ccdirenv owners map github.com/your-personal-handle  default
ccdirenv owners map github.com/your-employer         work
ccdirenv owners map github.com/a-client-org          client-acme
```

That's it. Every repository under those owners now uses the right Claude Code account the moment you `cd` into it.

The `init` prompt:

```
How do you organize Git repositories?
  1) ghq   — uses ~/ghq/<host>/<owner>/<repo> layout (ghq priority + git fallback)
  2) git   — any repository with a git remote (default)
  3) both  — both methods enabled (git priority, ghq fallback)
  4) off   — no repo-aware detection
Choose [1-4, default 2]:
```

Switch later with `ccdirenv mode set <ghq|git|both|off>`.

## Configuration

```toml
# ~/.ccdirenv/config.toml
default_profile = "default"
discovery_priority = "git"          # "git" | "ghq" — which method runs first when both are enabled

[ghq]
enabled = false                     # path-based detection (~/ghq layout)
# root = "~/ghq"                    # optional override, otherwise auto-detected

[git]
enabled = true                      # remote-based detection (.git/config)
remote = "origin"                   # remote to inspect

# Shared owner → profile map (used by both modes)
[owners]
"github.com/SuguruOoki" = "default"
"github.com/TheMoshInc" = "mosh"
"github.com/AcmeCorp"   = "work"

# Optional explicit per-directory overrides (highest priority before discovery)
[directories]
"~/sandbox/**" = "default"
```

CLI surface:

```sh
ccdirenv mode show                               # current discovery mode
ccdirenv mode set ghq|git|both|off               # switch modes

ccdirenv owners list                             # list owner → profile mappings
ccdirenv owners map github.com/Acme work         # add or update
ccdirenv owners unmap github.com/Acme            # remove

ccdirenv git show                                # git detection state
ccdirenv git enable / disable
ccdirenv git remote upstream                     # change which remote to inspect

ccdirenv ghq list                                # ghq state + owner mappings
ccdirenv ghq enable / disable
ccdirenv ghq root /custom/path                   # override ghq root (or "" to clear)
ccdirenv ghq install                             # install ghq if missing
```

For backward compatibility, `ccdirenv ghq map / unmap` still work as aliases of `ccdirenv owners map / unmap`. Old `[ghq.owners]` blocks in config.toml are migrated into `[owners]` automatically the next time the config is read.

### How git detection works

ccdirenv walks up from the current directory looking for `.git`, parses `.git/config` directly (no `git` subprocess), reads the configured remote URL, and extracts `<host>/<owner>`. Supported URL forms include:

- `git@github.com:Acme/widget.git`
- `https://github.com/Acme/widget.git`
- `ssh://git@github.com/Acme/widget.git`
- self-hosted hosts (any `<host>/<owner>` works as long as you have a matching `[owners]` entry)

When `.git` is a file rather than a directory (linked **worktrees** and **submodules**), ccdirenv follows `gitdir:` and `commondir` to reach the actual config — no special configuration needed.

### Per-repo overrides

Sometimes one repo under a mapped owner needs a different profile (a fork, a sandbox, etc.). Two ways:

```sh
# Option A: pin a single repo via marker file (highest priority).
cd ~/some/repo && ccdirenv use personal

# Option B: write an explicit glob in config.toml that beats discovery.
```

```toml
[directories]
"~/projects/sandbox/**" = "personal"
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

If a directory unexpectedly resolves to `default`, run `ccdirenv mode show` to confirm the active mode, then `ccdirenv owners list` to confirm the owner is mapped. For ghq mode, also check that your ghq root matches what `ccdirenv ghq list` reports. For git mode, ensure the repo has the configured remote (`ccdirenv git show`).

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
