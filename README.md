<!--
<!-- This file is part of forehead.
<!--
<!-- Copyright (C) 2026-Present Afsall Labs.
<!-- SPDX-License-Identifier: Apache-2.0 OR MIT
<!--
<!-- Licensed under the Apache License, Version 2.0 (the "License");
<!-- you may not use this file except in compliance with the License.
<!-- You may obtain a copy of the License at
<!--
<!-- http://www.apache.org/licenses/LICENSE-2.0
<!--
<!-- Unless required by applicable law or agreed to in writing, software
<!-- distributed under the License is distributed on an "AS IS" BASIS,
<!-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
<!-- See the License for the specific language governing permissions and
<!-- limitations under the License.
<!--
<!-- Alternatively, this file is available under the MIT License:
<!-- Permission is hereby granted, free of charge, to any person obtaining a copy
<!-- of this software and associated documentation files (the "Software"), to deal
<!-- in the Software without restriction, including without limitation the rights
<!-- to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
<!-- copies of the Software, and to permit persons to whom the Software is
<!-- furnished to do so, subject to the following conditions:
<!--
<!-- The above copyright notice and this permission notice shall be included in all
<!-- copies or substantial portions of the Software.
<!--
<!-- THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
<!-- IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
<!-- FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
<!-- AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
<!-- LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
<!-- OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
<!-- SOFTWARE.
-->


# Forehead

A tool for creating and maintaining file headers for code license. Supports CLI and CI/CD workflows.

## Installation

```bash
cargo install forehead-cli
```

Or build from source:

```bash
git clone https://github.com/afsall-labs/forehead
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