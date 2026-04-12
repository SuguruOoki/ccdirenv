# ccdirenv

> direnv-style automatic Claude Code account switching. The right account, just by `cd`-ing in.

`ccdirenv` transparently selects the correct Claude Code account based on the current working directory. No manual switch command, no shell gymnastics — just `cd` into a project and `claude` launches under the right identity.

## Status

Pre-release. Design document: [`docs/design.md`](./docs/design.md).

## How it works

1. Each account lives in an isolated profile directory under `~/.ccdirenv/profiles/<name>/`.
2. A tiny Rust shim placed earlier in `PATH` resolves the profile for the current directory and sets `CLAUDE_CONFIG_DIR` before `exec`-ing the real `claude` binary.
3. Resolution order: `.ccdirenv` marker file in any parent directory → `~/.ccdirenv/config.toml` glob patterns → default profile.

Claude Code's own installation and auto-update path are never touched.

## Install (planned)

```sh
# crates.io
cargo install ccdirenv

# Homebrew
brew tap SuguruOoki/tap && brew install ccdirenv
```

## Quick start (planned)

```sh
ccdirenv init            # prints PATH snippet to add to your shell rc
ccdirenv import default  # migrate existing login into 'default' profile
ccdirenv login work      # create and log into a new profile
ccdirenv use work        # bind the current directory to the 'work' profile
ccdirenv which           # show which profile resolves here
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
- MIT license ([LICENSE-MIT](./LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
