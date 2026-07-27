# forehead-cli

CLI tool for [forehead](https://github.com/afsall-inc/forehead) — create and maintain file headers for code license.

## Installation

```bash
cargo install forehead-cli
```

## Usage

```bash
forehead apply        # Apply headers to all source files
forehead check        # CI mode — exit 1 on missing/wrong headers
forehead list         # List files with header status
forehead remove       # Remove headers from files
forehead init         # Scaffold forehead.toml
```

## License

Apache-2.0 OR MIT