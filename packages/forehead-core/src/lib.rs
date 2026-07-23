// This file is part of forehead.
//
// Copyright (C) 2026-Present Afsall Labs.
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

use crate::{
    config::Config,
    error::ForeheadError,
    header::{apply_header_to_file, check_header_on_file, FileStatus},
};
use std::{fs, path::Path};
use walkdir::WalkDir;

pub mod comment;
pub mod config;
pub mod error;
pub mod header;
pub mod template;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules"];
const SKIP_FILES: &[&str] = &["Cargo.lock", "forehead.toml"];

pub struct Forehead {
    config: Config,
    root: std::path::PathBuf,
}

impl Forehead {
    pub fn new(config: Config) -> Self {
        let root = config.project_dir.clone();
        Forehead { config, root }
    }

    pub fn from_config_path(path: &Path) -> Result<Self, ForeheadError> {
        let config = Config::from_path(path)?;
        Ok(Forehead::new(config))
    }

    fn walk_entries(&self) -> impl Iterator<Item = walkdir::DirEntry> {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| {
                let fname = e.file_name().to_str().unwrap_or("");
                !SKIP_DIRS.contains(&fname)
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let fname = e.file_name().to_str().unwrap_or("");
                !SKIP_FILES.contains(&fname)
            })
    }

    pub fn apply(&self, dry_run: bool) -> Result<ApplyReport, ForeheadError> {
        let mut report = ApplyReport::new();

        for entry in self.walk_entries() {
            let path = entry.path().to_path_buf();
            let fname = entry.file_name().to_str().unwrap_or("").to_string();
            let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();

            if fname == "Cargo.toml" {
                self.apply_cargo_toml_license(&path, &rel, dry_run, &mut report);
                continue;
            }

            let comment_style = match comment::comment_style_for(&path) {
                Some(s) => s,
                None => continue,
            };

            let template = match self.config.template_for(&path, &self.root) {
                Some(t) => t,
                None => continue,
            };

            let subst = self.config.substitution_for(&path, &self.root);

            match apply_header_to_file(&path, &template, &comment_style, &subst, dry_run) {
                Ok(true) => report.applied.push(rel),
                Ok(false) => {}
                Err(e) => report.errors.push((rel, e.to_string())),
            }
        }

        Ok(report)
    }

    pub fn check(&self) -> Result<CheckReport, ForeheadError> {
        let mut report = CheckReport::new();

        for entry in self.walk_entries() {
            let path = entry.path().to_path_buf();
            let fname = entry.file_name().to_str().unwrap_or("").to_string();
            let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();

            if fname == "Cargo.toml" {
                if !self.check_cargo_toml_license(&path, &rel) {
                    report.missing.push(rel);
                }
                continue;
            }

            let comment_style = match comment::comment_style_for(&path) {
                Some(s) => s,
                None => continue,
            };

            let template = match self.config.template_for(&path, &self.root) {
                Some(t) => t,
                None => continue,
            };

            let subst = self.config.substitution_for(&path, &self.root);

            match check_header_on_file(&path, &template, &comment_style, &subst) {
                Ok(FileStatus::Correct) => {}
                Ok(FileStatus::Missing | FileStatus::Wrong) => {
                    report.missing.push(rel);
                }
                Err(e) => report.errors.push((rel, e.to_string())),
            }
        }

        Ok(report)
    }

    pub fn list(&self) -> Result<Vec<(std::path::PathBuf, FileStatus)>, ForeheadError> {
        let mut results = Vec::new();

        for entry in self.walk_entries() {
            let path = entry.path().to_path_buf();
            let fname = entry.file_name().to_str().unwrap_or("").to_string();
            let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();

            if fname == "Cargo.toml" {
                let status = if self.check_cargo_toml_license(&path, &rel) {
                    FileStatus::Correct
                } else {
                    FileStatus::Missing
                };
                results.push((rel, status));
                continue;
            }

            let comment_style = match comment::comment_style_for(&path) {
                Some(s) => s,
                None => continue,
            };

            let template = match self.config.template_for(&path, &self.root) {
                Some(t) => t,
                None => continue,
            };

            let subst = self.config.substitution_for(&path, &self.root);

            match check_header_on_file(&path, &template, &comment_style, &subst) {
                Ok(status) => results.push((rel, status)),
                Err(_) => results.push((rel, FileStatus::Wrong)),
            }
        }

        Ok(results)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    fn apply_cargo_toml_license(
        &self,
        path: &Path,
        rel: &Path,
        _dry_run: bool,
        report: &mut ApplyReport,
    ) {
        let license_str = self.config.license_for(path, &self.root);

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                report.errors.push((rel.to_path_buf(), e.to_string()));
                return;
            }
        };

        let new_content = update_cargo_toml_license(&content, &license_str);

        if new_content != content {
            if let Err(e) = fs::write(path, &new_content) {
                report.errors.push((rel.to_path_buf(), e.to_string()));
            } else {
                report.applied.push(rel.to_path_buf());
            }
        }
    }

    fn check_cargo_toml_license(&self, path: &Path, _rel: &Path) -> bool {
        let license_str = self.config.license_for(path, &self.root);

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let new_content = update_cargo_toml_license(&content, &license_str);
        new_content == content
    }
}

fn update_cargo_toml_license(content: &str, license_str: &str) -> String {
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_package = false;
    let mut found_license = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            new_lines.push(line.to_string());
        } else if in_package
            && !found_license
            && (trimmed.starts_with("license") || trimmed.starts_with("license.workspace"))
        {
            new_lines.push(format!("license = \"{license_str}\""));
            found_license = true;
            in_package = false;
        } else if in_package && !found_license && trimmed.starts_with('[') {
            new_lines.push(format!("license = \"{license_str}\""));
            new_lines.push(String::new());
            new_lines.push(line.to_string());
            found_license = true;
            in_package = false;
        } else {
            new_lines.push(line.to_string());
        }
    }

    new_lines.join("\n")
}

#[derive(Debug, Default)]
pub struct ApplyReport {
    pub applied: Vec<std::path::PathBuf>,
    pub errors: Vec<(std::path::PathBuf, String)>,
}

impl ApplyReport {
    pub fn new() -> Self {
        ApplyReport::default()
    }
    pub fn is_empty(&self) -> bool {
        self.applied.is_empty() && self.errors.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct CheckReport {
    pub missing: Vec<std::path::PathBuf>,
    pub errors: Vec<(std::path::PathBuf, String)>,
}

impl CheckReport {
    pub fn new() -> Self {
        CheckReport::default()
    }
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.errors.is_empty()
    }
}
