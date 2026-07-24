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

# Check headers (CI mode — exits 1 on failure)
forehead check

# List files with their header status
forehead list

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
| `#` line | Python, Ruby, Shell, YAML, TOML, R, Julia, Perl, Nix, Makefile, CMake, Dockerfile, config files |
| `--` line | SQL, Haskell, Lua, Ada, VHDL |
| `%` line | TeX/LaTeX, MATLAB, Prolog |
| `;` line | Lisp, Clojure, Scheme |
| `<!-- -->` block | HTML, XML, SVG, Markdown |
| `/* */` block | CSS, SCSS, Less, GraphQL, Protocol Buffers, Solidity |

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
```

## License

Apache-2.0 OR MIT