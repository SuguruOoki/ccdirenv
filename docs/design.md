# ccdirenv — Design Document

- **Status**: Draft (2026-04-13)
- **Audience**: Contributors and reviewers
- **License**: MIT OR Apache-2.0

## 1. Purpose

`ccdirenv` transparently selects the correct Claude Code account based on the current working directory. Think *direnv for Claude Code accounts*: no manual switch command — just `cd` into a project and `claude` runs under the right identity.

### Problem

Claude Code stores a single active OAuth identity in `~/.claude.json` and the OS credential store (Keychain on macOS, `~/.claude/.credentials.json` on Linux). Users juggling multiple accounts (personal / work / client) must either:

1. Log out and re-log in every time (lossy, slow).
2. Run manual switchers (`ccswitch`, `ClaudeCTX`, etc.) before launching `claude`.
3. Maintain parallel `$HOME` hacks.

None bind the account to the working directory automatically, so the wrong account is a constant footgun.

### Solution

A Rust CLI + transparent PATH shim that:

1. Stores each account as an isolated profile directory (`~/.ccdirenv/profiles/<name>/`).
2. Resolves the correct profile from the current working directory (marker file or central config).
3. Launches Claude Code with `CLAUDE_CONFIG_DIR` pointed at that profile.
4. Leaves Claude Code's own installation and auto-update path untouched.

## 2. Non-goals

- Managing non-Claude-Code tools (Codex, Gemini CLI, etc.). Explicitly out of scope to avoid scope creep seen in `cc-switch`.
- Web UI / desktop app. CLI only.
- Synchronizing profiles across machines. Users handle this via dotfiles if desired.
- Supporting Windows in v1 (can be added later; architecture does not preclude it).

## 3. Architecture overview

```
┌──────────────────────────────────────────────────────────────┐
│ User shell (zsh/bash/fish)                                   │
│   $PATH = ~/.ccdirenv/bin : ~/.local/bin : ...               │
└──────────────────────────────────────────────────────────────┘
             │ invokes `claude ...`
             ▼
┌──────────────────────────────────────────────────────────────┐
│ ~/.ccdirenv/bin/claude   (Rust shim, ≤ 2MB static binary)    │
│   1. Fast-path exit for --version / --help / migrate-*       │
│   2. Strip own dir from PATH, locate real `claude` via       │
│      `which claude` walk (follows symlinks)                  │
│   3. Resolve profile:                                        │
│        a. Walk up CWD → .ccdirenv marker                     │
│        b. Fallback to ~/.ccdirenv/config.toml [directories]  │
│        c. Fallback to default_profile                        │
│   4. Set CLAUDE_CONFIG_DIR=<resolved profile dir>            │
│   5. execvp(real_claude, argv, modified_env)                 │
└──────────────────────────────────────────────────────────────┘
             │ execvp (same PID, TTY, stdio)
             ▼
┌──────────────────────────────────────────────────────────────┐
│ Real claude binary (Anthropic-managed, auto-updates)         │
│   Reads CLAUDE_CONFIG_DIR, loads per-profile state           │
└──────────────────────────────────────────────────────────────┘
```

### Key invariants

- **Claude's install is untouched.** We only add a dir to `PATH`; we never modify `~/.local/bin/claude`, `~/.local/share/claude/**`, or the user's Keychain outside our own namespace.
- **Transparent pass-through.** Any flag/subcommand/stdin/TTY behavior matches `claude` exactly. `execvp` preserves PID so shell job control works.
- **Fail-open on resolution errors.** If profile resolution fails for any reason, the shim logs to stderr and launches Claude with the default profile rather than blocking the user.

## 4. Storage layout

```
~/.ccdirenv/
├── bin/
│   └── claude                   # the shim (Rust binary)
├── profiles/
│   ├── default/                 # seeded on `ccdirenv init`
│   │   ├── .claude.json
│   │   ├── .credentials.json    # plaintext, chmod 0400 (v1)
│   │   ├── projects/
│   │   ├── plugins/
│   │   └── ...                  # anything Claude writes to CLAUDE_CONFIG_DIR
│   ├── work/
│   └── personal/
├── config.toml                  # central directory→profile patterns
└── state.json                   # {last_resolved: {cwd, profile, at}}
```

Profile directories mirror what Claude Code itself creates under `CLAUDE_CONFIG_DIR`. We do not touch their internals.

### Profile isolation

Each profile is independent for `.claude.json`, credentials, MCP config, per-project history, and plugin cache. Users who want shared MCP config across profiles can symlink `profiles/personal/mcp.json` → `profiles/work/mcp.json` (documented recipe; not automated in v1).

## 5. Profile resolution

Resolution runs on every shim invocation (sub-millisecond target).

### Order

1. **Marker file**: walk from `$PWD` upward to `/`. First directory containing a `.ccdirenv` file wins. File format is a single line containing the profile name.
   ```
   $ cat ~/work/acme/.ccdirenv
   work
   ```
2. **Central config**: `~/.ccdirenv/config.toml`, `[directories]` table. Keys are shell-style glob patterns; values are profile names. Tilde (`~`) and env vars are expanded. First match in file order wins.
3. **Default**: `default_profile` from `config.toml`, or literal `default` if unset.

### Config schema

```toml
# ~/.ccdirenv/config.toml
default_profile = "personal"

[directories]
"~/work/**"     = "work"
"~/clients/a/**" = "client-a"
"~/oss/**"      = "personal"
```

Pattern matching uses the `globset` crate. Case-sensitive on Linux, case-insensitive on macOS (match filesystem semantics).

### Edge cases

| Case | Behavior |
|---|---|
| Profile named in marker does not exist | Warn to stderr, fall back to default_profile. |
| Marker file is a directory | Ignore, continue walking up. |
| Marker file empty or whitespace-only | Treat as "no marker" and continue. |
| Symlinked CWD | Resolve real path first, then walk. |
| CWD under `/` (no parents) | Proceed to step 2. |
| No `config.toml` file | Skip step 2. |

## 6. Shim behavior

### Invocation flow (happy path)

```rust
fn main() -> ! {
    let args: Vec<OsString> = env::args_os().collect();

    // Fast path: no profile resolution needed for pure meta commands.
    if is_fast_path(&args) {
        exec_real_claude_unchanged(&args);
    }

    let real = locate_real_claude()?;        // errors: exec pass-through with warning
    let profile = resolve_profile(cwd())?;   // errors: use default, log
    let env = build_env(profile);
    execvp(&real, &args, &env)               // never returns
}
```

### Fast-path triggers

These skip profile resolution to keep latency minimal and avoid touching profile state for meta operations:

- `claude --version`, `claude -V`
- `claude --help`, `claude -h`
- `claude doctor`
- `claude migrate-installer` (operates on Claude's own install, not user data)
- Empty argv beyond arg0 is NOT fast-path (interactive session needs correct profile).

Fast-path detection scans only the first positional arg and top-level flags to avoid mis-matching inner subcommands.

### Real binary resolution

1. Canonicalize our own exe path → derive `SHIM_DIR`.
2. Build a PATH with `SHIM_DIR` removed.
3. `which::which_in("claude", &path_clean, cwd)` → first hit.
4. If nothing found, print a friendly error and `exit(127)`.

This ensures that even if Claude moves its install location on update, the shim finds the new binary without reinstall.

### Escape hatch

- `CCDIRENV_DISABLE=1` → skip resolution entirely, exec real claude with current env.
- `CCDIRENV_PROFILE=<name>` → force a specific profile for this invocation.
- `CCDIRENV_DEBUG=1` → trace resolution steps to stderr.

## 7. CLI surface (`ccdirenv` management command)

```
ccdirenv init                 Install shim, print PATH setup snippet for shell rc
ccdirenv login [<profile>]    Create profile dir, run `claude /login` inside it
ccdirenv list                 Show all profiles with active account email
ccdirenv which                Print profile that resolves for current CWD
ccdirenv use <profile>        Create/overwrite .ccdirenv marker in CWD
ccdirenv unuse                Remove .ccdirenv marker in CWD
ccdirenv config               Open ~/.ccdirenv/config.toml in $EDITOR
ccdirenv doctor               Diagnose PATH order, real claude resolvability, permissions
ccdirenv import <name>        One-time migration: copy existing ~/.claude/* into profile
ccdirenv --version            Version + commit hash
```

### Exit codes

- `0` success
- `1` generic error (with message)
- `2` config/usage error
- `127` real `claude` not found

## 8. Initial setup flow

```
$ cargo install ccdirenv        # or brew install, or download release
$ ccdirenv init
→ creates ~/.ccdirenv/{bin,profiles/default,config.toml}
→ installs shim at ~/.ccdirenv/bin/claude
→ prints (does not auto-edit):
    Add this line to your shell rc (~/.zshrc / ~/.bashrc):
      export PATH="$HOME/.ccdirenv/bin:$PATH"

$ ccdirenv import default     # optional: migrate existing login into 'default' profile
$ ccdirenv login work         # opens browser, saves to work profile
```

### Why not auto-edit shell rc?

- Respects user's dotfile management (chezmoi, home-manager, etc.)
- Avoids the `direnv` footgun of silently modifying rc files
- `init` prints the exact line; user decides

## 9. Credential storage (v1)

**Plaintext per-profile `.credentials.json`**, permissions `0400`, inside the profile directory.

### Rationale

- Matches Claude Code's existing `plaintext` fallback implementation, verified in binary disassembly at `/Users/suguru_oki_mosh/.local/share/claude/versions/<v>`.
- `CLAUDE_CONFIG_DIR` redirects credential reads/writes to the profile dir automatically — no custom handling needed on our side.
- Threat model: if an attacker can read `~/.ccdirenv/profiles/*/`, they already have enough access to compromise the user regardless. Disk encryption (FileVault/LUKS) is the real defense at this layer.

### v2 roadmap (not shipped in v1)

macOS Keychain per-profile: use service names `Claude Code-credentials-<profile>`. Requires a thin interposer that rewrites Keychain service names before exec. Deferred until user demand materializes.

## 10. OS support

| OS | Arch | v1 | Notes |
|---|---|---|---|
| macOS | arm64 | ✅ | Primary dev target |
| macOS | x86_64 | ✅ | Cross-compile from arm64 |
| Linux | x86_64 | ✅ | glibc 2.31+ (ubuntu 20.04 baseline) |
| Linux | arm64 | ✅ | Release tarball |
| Windows | any | ❌ v2+ | Needs `CreateProcess` path + different PATH semantics |

## 11. Distribution

1. **crates.io**: `cargo install ccdirenv`. Source of truth.
2. **GitHub Releases**: prebuilt tarballs per target triple. SHA256 + minisign signature.
3. **Homebrew tap**: `brew tap SuguruOoki/ccdirenv && brew install ccdirenv`. Tap repo auto-bumped from release workflow.
4. **Shell one-liner**: `curl -fsSL https://.../install.sh | sh` (optional, later).

## 12. Testing strategy

### Unit (Rust `cargo test`)

- `resolve::test_marker_walks_up_directories`
- `resolve::test_marker_empty_falls_through`
- `resolve::test_config_glob_matches_tilde`
- `path::test_shim_dir_removed_from_path`
- `env::test_fast_path_detection`
- `config::test_toml_parses_and_round_trips`

### Integration (tempfile-based)

- Create tempdir, seed fake `claude` binary (shell script that prints `$CLAUDE_CONFIG_DIR`), seed profile tree, invoke shim, assert output.
- Cover: marker resolution, config resolution, default fallback, fast-path skip, `CCDIRENV_DISABLE`.

### Shell E2E (`bats-core`)

- `init` → PATH hint printed.
- `cd ~/work && claude --print-config-dir` (with fake claude) → prints work profile path.
- `cd /tmp && claude ...` → default profile.

### CI matrix

GitHub Actions: `{macos-14, macos-13, ubuntu-22.04, ubuntu-22.04-arm}` × `{cargo test, cargo clippy -- -D warnings, cargo fmt --check}`.

## 13. Observability

- `CCDIRENV_DEBUG=1` prints resolution trace to stderr: real claude path, CWD, marker hits, config hits, final profile.
- `~/.ccdirenv/state.json` records the last resolution for `ccdirenv which` without a filesystem walk.
- No telemetry, no network calls, ever.

## 14. Security considerations

- Shim does **not** `setuid`, does **not** write outside `~/.ccdirenv/`.
- `exec` uses explicit path (`execvp` with absolute path from `which`), not shell interpretation.
- Marker files read with 64KB limit to avoid runaway memory on hostile inputs.
- Glob patterns compiled with `globset` (no shell execution).
- No arbitrary code execution from `config.toml` (strict parse, no `include`/`exec` directives).

## 15. Open questions

None blocking v1. Deferred to v2:

1. Keychain integration on macOS.
2. Windows support.
3. Optional `direnv`-style automatic `cd`-hook for shells that want environment vars set too.
4. Profile sync/backup tooling.

## 16. Naming rationale

- `ccdirenv` = **cc** (Claude Code) + **direnv** (the widely-known directory-based env switcher).
- One-line pitch: *"direnv-style automatic Claude Code account switching."*
- Differentiates from crowded `cc-switch` / `ccswitch` / `ClaudeCTX` space where all competitors require manual invocation.
