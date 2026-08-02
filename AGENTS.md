# Forehead — Agent Guide

## Commands

```bash
forehead apply        # Apply headers to all source files
forehead apply --dry-run  # Preview changes without modifying
forehead check        # CI mode — exit 1 on missing/wrong headers
forehead list         # List files with header status
forehead remove       # Remove headers from all source files
forehead remove --dry-run  # Preview removals without modifying
forehead init         # Scaffold forehead.toml
```

## Config

The `forehead.toml` file defines project metadata, header templates, path-based mapping rules, and header detection settings.

### `[header]` section

- `indicators` — extra keywords on top of built-in defaults (`Copyright`, `SPDX`, `License`) for header detection. Set to `["none"]` to disable all built-in defaults.
- `greetings` — optional line prepended to every header. Supports template placeholders.

## Invariants

- All source files must have a correct license header
- `forehead check` must pass in CI
- Config path is `forehead.toml` by default
- Template paths are relative to the config file

## PRDoc

Each PR requires a `prdoc/pr_<number>.prdoc` file. Use changelogger to manage them:

```bash
changelogger prdoc generate --pr <NUMBER>   # Auto-generate prdoc skeleton
changelogger prdoc validate prdoc/pr_*.prdoc # Validate all prdocs
changelogger changelog verify --from <TAG>   # Verify all commits have prdocs
changelogger changelog generate --from <TAG> # Generate CHANGELOG.md
changelogger changelog bump --current <VER>  # Compute next version
```

Config: `changelogger.toml` — defines schema, output dir, template, and audiences.

## CI Order

```bash
mise run ci        # fmt → clippy → test → build → headers
```

## Toolchain

- Rust: `nightly-2026-02-18` (pinned in `rust-toolchain.toml`)
- Edition: 2021
- License: Apache-2.0 OR MIT