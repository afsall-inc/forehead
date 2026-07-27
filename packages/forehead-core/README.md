# forehead-core

Core library for [forehead](https://github.com/afsall-inc/forehead) — a tool for creating and maintaining file headers for code license.

## Usage

```rust
use forehead_core::{Forehead, Config};

let config = Config::from_path("forehead.toml")?;
let forehead = Forehead::new(config);

// Apply headers
let report = forehead.apply(false)?;

// Check headers (CI mode)
let report = forehead.check()?;
if !report.is_clean() {
    std::process::exit(1);
}

// Remove headers
let report = forehead.remove(false)?;
```

## Features

- Configurable header detection with built-in and custom indicators
- Support for 30+ file types with correct comment style detection
- Template placeholders: `{project}`, `{author}`, `{year}`, `{year_span}`, `{license}`, `{repository}`, `{description}`, `{file}`
- Optional greetings line prepended to every header
- Headless detection, apply, check, and remove operations

## License

Apache-2.0 OR MIT