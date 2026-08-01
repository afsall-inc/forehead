// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// This file is part of forehead.
//
// Copyright (C) 2026-Present Afsall Inc.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Alternatively, this file is available under the MIT License:
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::cli::{Cli, Command};
use anyhow::{Context, Result};
use forehead_core::Forehead;
use std::path::Path;

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Apply {
            config,
            dry_run,
            path,
        } => apply(config, *dry_run, path.as_deref()),
        Command::Check { config, json, path } => check(config, *json, path.as_deref()),
        Command::List { config, json, path } => list(config, *json, path.as_deref()),
        Command::Init { path } => init(path),
        Command::Remove {
            config,
            dry_run,
            path,
        } => remove(config, *dry_run, path.as_deref()),
        Command::Replace {
            config,
            dry_run,
            path,
        } => replace(config, *dry_run, path.as_deref()),
    }
}

fn apply(config_path: &str, dry_run: bool, path: Option<&str>) -> Result<()> {
    let config_path = Path::new(config_path);
    let forehead = if let Some(p) = path {
        // Use the provided path as the project root, but still read config from config_path
        let mut config = forehead_core::config::Config::from_path(config_path)
            .context("failed to read config")?;
        config.project_dir = Path::new(p).to_path_buf();
        Forehead::new(config)
    } else {
        Forehead::from_config_path(config_path).context("failed to read config")?
    };

    let report = forehead.apply(dry_run).context("failed to apply headers")?;

    if dry_run {
        for path in &report.applied {
            println!("WOULD APPLY: {}", path.display());
        }
    } else {
        for path in &report.applied {
            println!("APPLIED: {}", path.display());
        }
    }

    for (path, err) in &report.errors {
        eprintln!("ERROR on {}: {}", path.display(), err);
    }

    if report.is_empty() {
        println!("All files up to date.");
    }

    Ok(())
}

fn check(config_path: &str, json: bool, path: Option<&str>) -> Result<()> {
    let config_path = Path::new(config_path);
    let forehead = if let Some(p) = path {
        let mut config = forehead_core::config::Config::from_path(config_path)
            .context("failed to read config")?;
        config.project_dir = Path::new(p).to_path_buf();
        Forehead::new(config)
    } else {
        Forehead::from_config_path(config_path).context("failed to read config")?
    };

    let report = forehead.check().context("failed to check headers")?;

    if json {
        let output = serde_json::json!({
            "missing": report.missing.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
            "errors": report.errors.iter().map(|(p, e)| format!("{}: {}", p.display(), e)).collect::<Vec<_>>(),
            "clean": report.is_clean(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for path in &report.missing {
            println!("MISSING: {}", path.display());
        }
        for (path, err) in &report.errors {
            eprintln!("ERROR on {}: {}", path.display(), err);
        }
    }

    if report.is_clean() {
        println!("All headers are correct.");
        Ok(())
    } else {
        anyhow::bail!(
            "{} files have missing or incorrect headers",
            report.missing.len()
        );
    }
}

fn list(config_path: &str, json: bool, path: Option<&str>) -> Result<()> {
    use forehead_core::header::FileStatus;

    let config_path = Path::new(config_path);
    let forehead = if let Some(p) = path {
        let mut config = forehead_core::config::Config::from_path(config_path)
            .context("failed to read config")?;
        config.project_dir = Path::new(p).to_path_buf();
        Forehead::new(config)
    } else {
        Forehead::from_config_path(config_path).context("failed to read config")?
    };

    let results = forehead.list().context("failed to list headers")?;

    if json {
        let entries: Vec<serde_json::Value> = results
            .iter()
            .map(|(p, s)| {
                serde_json::json!({
                    "file": p.to_string_lossy(),
                    "status": match s {
                        FileStatus::Correct => "correct",
                        FileStatus::Missing => "missing",
                        FileStatus::Wrong => "wrong",
                    }
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for (path, status) in &results {
            let label = match status {
                FileStatus::Correct => "OK",
                FileStatus::Missing => "MISSING",
                FileStatus::Wrong => "WRONG",
            };
            println!("{}  {}", label, path.display());
        }
    }

    Ok(())
}

fn init(path: &str) -> Result<()> {
    let config_content = r#"[project]
name = "my-project"
default_license = "Apache-2.0 OR MIT"
default_author = "Your Name"
default_year = 2026
repository = "https://github.com/your-org/my-project"
description = "My project description"

[templates]
mit-apache = "docs/LICENSES/headers/HEADER-MIT-APACHE"

[[mapping]]
paths = ["."]
template = "mit-apache"

[header]
# Extra keywords on top of built-in defaults (Copyright, SPDX, License)
# that identify a comment block as a file header.
# Set to ["none"] to disable all built-in defaults.
# indicators = []
# Optional line prepended to every header. Supports template placeholders.
# greetings = "بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم"

[file_types]
"#;

    std::fs::write(path, config_content).with_context(|| format!("failed to write {path}"))?;

    println!("Created {path}");
    println!("Next steps:");
    println!("  1. Create your header template files under docs/LICENSES/headers/");
    println!("  2. Run `forehead apply` to apply headers");
    Ok(())
}

fn remove(config_path: &str, dry_run: bool, path: Option<&str>) -> Result<()> {
    let config_path = Path::new(config_path);
    let forehead = if let Some(p) = path {
        let mut config = forehead_core::config::Config::from_path(config_path)
            .context("failed to read config")?;
        config.project_dir = Path::new(p).to_path_buf();
        Forehead::new(config)
    } else {
        Forehead::from_config_path(config_path).context("failed to read config")?
    };

    let report = forehead
        .remove(dry_run)
        .context("failed to remove headers")?;

    if dry_run {
        for path in &report.applied {
            println!("WOULD REMOVE: {}", path.display());
        }
    } else {
        for path in &report.applied {
            println!("REMOVED: {}", path.display());
        }
    }

    for (path, err) in &report.errors {
        eprintln!("ERROR on {}: {}", path.display(), err);
    }

    if report.is_empty() {
        println!("No headers to remove.");
    }

    Ok(())
}

fn replace(config_path: &str, dry_run: bool, path: Option<&str>) -> Result<()> {
    let config_path = Path::new(config_path);
    let forehead = if let Some(p) = path {
        let mut config = forehead_core::config::Config::from_path(config_path)
            .context("failed to read config")?;
        config.project_dir = Path::new(p).to_path_buf();
        Forehead::new(config)
    } else {
        Forehead::from_config_path(config_path).context("failed to read config")?
    };

    let report = forehead
        .replace(dry_run)
        .context("failed to replace headers")?;

    if dry_run {
        for path in &report.applied {
            println!("WOULD REPLACE: {}", path.display());
        }
    } else {
        for path in &report.applied {
            println!("REPLACED: {}", path.display());
        }
    }

    for (path, err) in &report.errors {
        eprintln!("ERROR on {}: {}", path.display(), err);
    }

    if report.is_empty() {
        println!("All headers are correct.");
    }

    Ok(())
}
