# Forehead

A tool for creating and maintaining file headers for code license. Supports CLI and CI/CD workflows.

## Installation

```bash
cargo install forehead-cli
```

Or build from source:

```bash
git clone https://github.com/afsall-inc/forehead
cd forehead
cargo build --release
```

## Usage

```bash
# Apply headers to all source files
forehead apply

# Preview changes without modifying
forehead apply --dry-run

# Check headers (CI mode — exits 1 on failure)
forehead check

# List files with their header status
forehead list

# Remove headers from all source files
forehead remove

# Preview removals without modifying
forehead remove --dry-run

# Scaffold a forehead.toml config
forehead init
```

## Configuration

Create a `forehead.toml` in your project root:

```toml
[project]
name = "my-project"
default_license = "Apache-2.0 OR MIT"
default_author = "Your Name"
default_year = 2026
repository = "https://github.com/your-org/my-project"
description = "My project description"

[templates]
mit-apache = "docs/LICENSES/headers/HEADER-MIT-APACHE"
gpl3 = "docs/LICENSES/headers/HEADER-GPL3"

[[mapping]]
paths = ["."]
template = "mit-apache"

[[mapping]]
paths = ["repos/enterprise/"]
template = "gpl3"

[header]
# Extra keywords on top of built-in defaults (Copyright, SPDX, License)
# that identify a comment block as a file header.
# Set to ["none"] to disable all built-in defaults.
# indicators = []
# Optional line prepended to every header. Supports template placeholders.
# greetings = "بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم"

# Directories or files to exclude by name or relative path suffix.
# TOML files are always ignored by default (Cargo.toml is still synced).
# ignore = ["vendor", "generated.rs"]
```

## Header Configuration

### `[header]` section

The `[header]` section controls how forehead detects and builds file headers.

**`indicators`** (optional, default: empty)

Extra keywords on top of the built-in defaults (`Copyright`, `SPDX`, `License`) that identify a consecutive comment block as a file header.

| Scenario | Behavior |
|----------|----------|
| `indicators = []` (unset) | Built-in defaults apply |
| `indicators = ["CustomTag"]` | Built-in defaults + `CustomTag` |
| `indicators = ["none"]` | No keyword detection — any consecutive comment block at the top of a file is treated as a header |

**`greetings`** (optional, default: empty)

An optional line prepended to the very top of every header. Supports template placeholders. For example, a Basmala:

```toml
[header]
greetings = "بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم"
```

This will produce:

```rust
// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// This file is part of my-project.
// Copyright (C) 2026-Present Your Name.
// SPDX-License-Identifier: Apache-2.0 OR MIT
```

## File Selection

The `ignore` field at the top level of `forehead.toml` controls which files and directories are excluded from processing.

```toml
ignore = ["vendor", "generated.rs", "build/out.rs"]
```

**Matching rules:**

- **Name match** — an entry matches if it equals the file or directory name. `ignore = ["vendor"]` excludes any directory named `vendor` and its entire subtree.
- **Path suffix match** — an entry matches if the file's relative path ends with it. `ignore = ["gen.rs"]` excludes `src/gen.rs`, `build/gen.rs`, etc., without needing to specify the full path.

**Default exclusions:**

| Entry | Reason |
|-------|--------|
| `.git/` | Git internals |
| `target/` | Rust build output |
| `node_modules/` | JavaScript dependencies |
| `.github/` | GitHub Actions workflows and CI config |
| `Cargo.lock` | Rust lockfile (auto-generated) |
| `forehead.toml` | The tool's own config |
| `*.toml` (except `Cargo.toml`) | Config files; `Cargo.toml` is still visited for its `license` field |

The default exclusions are always applied — they don't need to be listed in `ignore`. The `ignore` list is additive on top of them.

**Legacy key:** The old key name `skip` is still accepted as an alias for backward compatibility.

**Idempotent application:** When applying headers, forehead detects stale license headers anywhere in a file (not just at the top) and replaces them instead of stacking a new header on top. This means headers are never duplicated, even if the file was previously modified by another tool or an older version of forehead.

## Template Placeholders

| Placeholder | Description | Example |
|-------------|-------------|---------|
| `{project}` | Project name | `my-project` |
| `{author}` | Copyright holder | `Your Name` |
| `{year}` | Single year | `2026` |
| `{year_span}` | Year range | `2026-Present` |
| `{license}` | License identifier | `Apache-2.0 OR MIT` |
| `{repository}` | Project URL | `https://github.com/...` |
| `{description}` | Project description | `My project` |
| `{file}` | Current filename | `main.rs` |

## Supported File Types

| Comment Style | Languages |
|--------------|-----------|
| `//` line | Rust, Go, C/C++, Java, JS/TS, Swift, Kotlin, Dart, Zig, PHP, C#, Scala, Svelte, Vue |
| `#` line | Python, Ruby, Shell, YAML, R, Julia, Perl, Nix, Makefile, CMake, Dockerfile, config files |
| `--` line | SQL, Haskell, Lua, Ada, VHDL |
| `%` line | TeX/LaTeX, MATLAB, Prolog |
| `;` line | Lisp, Clojure, Scheme |
| `<!-- -->` block | HTML, XML, SVG, Markdown |
| `/* */` block | CSS, SCSS, Less, GraphQL, Protocol Buffers, Solidity |

> **Note:** TOML files are skipped by default and are not processed for headers. `Cargo.toml` is the only exception — it is still visited so forehead can maintain its `license` field, but no comment header is ever added to it.

## Library Usage

```rust
use forehead_core::{Forehead, Config};

let config = Config::from_path("forehead.toml")?;
let forehead = Forehead::new(config);

// Apply headers
let report = forehead.apply(false)?;

// Check headers (CI mode)
let report = forehead.check()?;
if !report.is_clean() {
    eprintln!("Headers missing on: {:?}", report.missing);
    std::process::exit(1);
}

// Remove headers
let report = forehead.remove(false)?;
```

## License

Apache-2.0 OR MIT