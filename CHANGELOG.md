# Changelog

## 0.4.0 - 2026-09-10

### Added

- Codex CLI account switching through a native `codex` shim using `CODEX_HOME`.
- `--tool codex` for `login`, `import`, `list`, `which`, and `doctor`.
- Login argument forwarding after `--`, including device authentication and API keys from stdin.
- `import --from` for custom configuration sources.
- English and Japanese setup instructions, tool support tables, and migration guidance.

### Fixed

- Keep existing discovery settings when rerunning `init` without an explicit `--mode`.
- Resolve real CLI binaries installed next to ccdirenv without excluding their installation directory.
- Do not mistake prompt text for help, version, or maintenance commands.
- Diagnose shadowed or broken shims and stop on malformed configuration or unusable profile directories.
- Allow Claude configuration imports after Codex has been configured for the same profile.
- Reject invalid profile paths, recursive import destinations, and symlinked credential files; create new profile directories with private permissions.

### Compatibility

- Existing Claude profile directories and default commands remain unchanged. Codex uses `profiles/<name>/codex`.
- Both tools share owner mappings, directory globs, markers, and `CCDIRENV_PROFILE`.
- File imports do not migrate OS keyring credentials. Log in again where required.
- Automatic switching applies to CLI commands through the shims on macOS and Linux. The launch directory determines the profile; tool-specific directory options do not change that selection.
