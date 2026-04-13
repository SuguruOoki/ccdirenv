# ccdirenv

[![CI](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml/badge.svg)](https://github.com/SuguruOoki/ccdirenv/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccdirenv.svg)](https://crates.io/crates/ccdirenv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**English | [日本語](./README.ja.md)**

> direnv-style automatic Claude Code account switching. The right account, just by `cd`-ing in.

`ccdirenv` transparently selects the correct Claude Code account based on the current working directory. No manual switch command, no shell gymnastics — just `cd` into a project and `claude` launches under the right identity.

## Why

Juggling personal, work, and client Claude Code accounts today means either logging out and back in, or running an explicit switch command before every `claude` invocation. `ccdirenv` binds accounts to directories so the right account is used automatically — the same pattern [direnv](https://direnv.net/) popularised for environment variables.

## How it works

1. Each account lives in an isolated profile directory under `~/.ccdirenv/profiles/<name>/`.
2. A small Rust shim placed earlier in `PATH` resolves the profile for the current directory and sets `CLAUDE_CONFIG_DIR` before handing off to the real `claude` binary.
3. Resolution order:
   1. `CCDIRENV_PROFILE` environment variable (force override for one invocation)
   2. `.ccdirenv` marker file in the current directory or any parent
   3. `~/.ccdirenv/config.toml` glob patterns (first match wins)
   4. `default_profile` from `config.toml`, falling back to `default`

Claude Code's own installation and auto-update path are never touched — the shim only adds an entry earlier in `PATH` and execs whatever `claude` it finds after stripping itself.

## Install

```sh
# crates.io (recommended for now)
cargo install ccdirenv

# Homebrew (macOS and Linux, coming once the tap is published)
brew tap SuguruOoki/tap
brew install ccdirenv
```

## Setup

```sh
# 1. Create the profile directory layout and install the shim at ~/.ccdirenv/bin/claude.
ccdirenv init

# 2. Follow the printed instruction: add this line to your shell rc (~/.zshrc, ~/.bashrc, etc.):
#    export PATH="$HOME/.ccdirenv/bin:$PATH"
#    Then reload your shell.

# 3. (Optional) Migrate your existing ~/.claude/ login into the 'default' profile.
ccdirenv import default

# 4. Create and log into additional profiles.
ccdirenv login work
ccdirenv login personal
```

## Daily use

```sh
# Bind the current directory (and its children) to the 'work' profile.
cd ~/clients/acme && ccdirenv use work

# Check which profile resolves here.
ccdirenv which

# Remove the binding.
ccdirenv unuse

# List all profiles with their logged-in email.
ccdirenv list

# Run diagnostics if something looks off.
ccdirenv doctor
```

Once a directory is bound, running `claude` from inside it automatically launches against the right account. Running `claude` from anywhere else uses the directory's configured profile, or `default`.

## Global patterns via `~/.ccdirenv/config.toml`

Instead of sprinkling `.ccdirenv` markers everywhere, you can centrally declare glob-to-profile rules:

```toml
default_profile = "personal"

[directories]
"~/clients/acme/**"  = "acme"
"~/clients/globex/**" = "globex"
"~/work/**"           = "work"
"~/oss/**"            = "personal"
```

Per-directory `.ccdirenv` markers always win over central patterns. Central patterns are matched in file order — the first hit wins.

Edit the file from any directory with `ccdirenv config`.

## Environment variables

| Variable | Effect |
|---|---|
| `CCDIRENV_PROFILE=<name>` | Force a specific profile for this invocation, ignoring marker and config. |
| `CCDIRENV_DISABLE=1` | Skip resolution entirely — the shim transparently execs the real `claude` with the inherited `CLAUDE_CONFIG_DIR`. Useful for debugging. |
| `CCDIRENV_DEBUG=1` | Print the chosen profile, profile dir, and real claude path to stderr before exec. |
| `CCDIRENV_HOME=<path>` | Override `~/.ccdirenv/` as the data root (useful for testing). |

## Troubleshooting

Run `ccdirenv doctor`. It reports:

- whether the shim is installed at `~/.ccdirenv/bin/claude`
- whether `PATH` includes `~/.ccdirenv/bin`
- whether a non-shim `claude` binary is reachable on `PATH`
- whether `config.toml` exists

If the shim is installed but `claude` keeps picking up the wrong account, make sure `~/.ccdirenv/bin` comes *before* `~/.local/bin` (or wherever Claude Code installed itself) in your `PATH`.

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
