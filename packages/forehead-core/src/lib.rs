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

use crate::{
    config::Config,
    error::ForeheadError,
    header::{
        apply_header_to_file, check_header_on_file, remove_header_from_file,
        replace_header_on_file, FileStatus,
    },
};
use std::{fs, path::Path};
use walkdir::WalkDir;

pub mod comment;
pub mod config;
pub mod error;
pub mod header;
pub mod template;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".github"];
const SKIP_FILES: &[&str] = &["Cargo.lock", "forehead.toml"];

/// Files with the `.toml` extension are skipped by default. `Cargo.toml` is
/// exempt because its `license` field is maintained by the apply command.
fn is_skipped_toml(entry: &walkdir::DirEntry) -> bool {
    let fname = entry.file_name().to_str().unwrap_or("");
    fname != "Cargo.toml" && entry.path().extension().and_then(|s| s.to_str()) == Some("toml")
}

/// Check whether an entry matches the configured `ignore` list. An entry is
/// ignored when its file/dir name equals an entry, or when its path relative
/// to the project root ends with an entry (path suffix matching).
/// Path separators are normalised so configs written with `/` work on Windows.
fn is_ignored(rel: &Path, fname: &str, ignore: &[String]) -> bool {
    if ignore.is_empty() {
        return false;
    }
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let rel_str = rel_str.trim_start_matches("./");
    ignore.iter().any(|pattern| {
        let pattern = pattern.replace('\\', "/");
        if pattern.is_empty() {
            return false;
        }
        fname == pattern || rel_str.ends_with(&pattern)
    })
}

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

    fn walk_entries(&self) -> impl Iterator<Item = walkdir::DirEntry> + use<'_> {
        let ignore = &self.config.ignore;
        WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(move |e| {
                let fname = e.file_name().to_str().unwrap_or("");
                if SKIP_DIRS.contains(&fname) {
                    return false;
                }
                if is_skipped_toml(e) {
                    return false;
                }
                let rel = e.path().strip_prefix(&self.root).unwrap_or(e.path());
                !is_ignored(rel, fname, ignore)
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let fname = e.file_name().to_str().unwrap_or("");
                if SKIP_FILES.contains(&fname) {
                    return false;
                }
                let rel = e.path().strip_prefix(&self.root).unwrap_or(e.path());
                !is_ignored(rel, fname, ignore)
            })
    }

    pub fn apply(&self, dry_run: bool) -> Result<ApplyReport, ForeheadError> {
        let mut report = ApplyReport::new();
        let indicators = self.config.header.all_indicators();
        let greetings = &self.config.header.greetings;

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

            match apply_header_to_file(
                &path,
                &template,
                &comment_style,
                &subst,
                dry_run,
                &indicators,
                greetings,
            ) {
                Ok(true) => report.applied.push(rel),
                Ok(false) => {}
                Err(e) => report.errors.push((rel, e.to_string())),
            }
        }

        Ok(report)
    }

    pub fn check(&self) -> Result<CheckReport, ForeheadError> {
        let mut report = CheckReport::new();
        let indicators = self.config.header.all_indicators();
        let greetings = &self.config.header.greetings;

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

            match check_header_on_file(
                &path,
                &template,
                &comment_style,
                &subst,
                &indicators,
                greetings,
            ) {
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
        let indicators = self.config.header.all_indicators();
        let greetings = &self.config.header.greetings;

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

            match check_header_on_file(
                &path,
                &template,
                &comment_style,
                &subst,
                &indicators,
                greetings,
            ) {
                Ok(status) => results.push((rel, status)),
                Err(_) => results.push((rel, FileStatus::Wrong)),
            }
        }

        Ok(results)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn remove(&self, dry_run: bool) -> Result<ApplyReport, ForeheadError> {
        let mut report = ApplyReport::new();
        let indicators = self.config.header.all_indicators();

        for entry in self.walk_entries() {
            let path = entry.path().to_path_buf();
            let fname = entry.file_name().to_str().unwrap_or("").to_string();
            let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();

            if fname == "Cargo.toml" || fname == "Cargo.lock" {
                continue;
            }

            let comment_style = match comment::comment_style_for(&path) {
                Some(s) => s,
                None => continue,
            };

            match remove_header_from_file(&path, &comment_style, &indicators, dry_run) {
                Ok(true) => report.applied.push(rel),
                Ok(false) => {}
                Err(e) => report.errors.push((rel, e.to_string())),
            }
        }

        Ok(report)
    }

    pub fn replace(&self, dry_run: bool) -> Result<ApplyReport, ForeheadError> {
        let mut report = ApplyReport::new();
        let indicators = self.config.header.all_indicators();
        let greetings = &self.config.header.greetings;

        for entry in self.walk_entries() {
            let path = entry.path().to_path_buf();
            let fname = entry.file_name().to_str().unwrap_or("").to_string();
            let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();

            if fname == "Cargo.toml" || fname == "Cargo.lock" {
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

            match replace_header_on_file(
                &path,
                &template,
                &comment_style,
                &subst,
                &indicators,
                greetings,
                dry_run,
            ) {
                Ok(true) => report.applied.push(rel),
                Ok(false) => {}
                Err(e) => report.errors.push((rel, e.to_string())),
            }
        }

        Ok(report)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct TestProject {
        dir: PathBuf,
    }

    impl TestProject {
        fn new(files: &[(&str, &str)], ignore: &[&str]) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir =
                std::env::temp_dir().join(format!("forehead-it-{}-{nanos}", std::process::id()));
            fs::create_dir_all(dir.join("docs")).unwrap();
            fs::write(
                dir.join("docs").join("HEADER"),
                "This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.",
            )
            .unwrap();

            let mut config = String::new();
            if !ignore.is_empty() {
                config.push_str("ignore = [");
                config.push_str(
                    &ignore
                        .iter()
                        .map(|s| format!("\"{s}\""))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                config.push_str("]\n\n");
            }
            config.push_str(
                "[project]\nname = \"TestProj\"\ndefault_license = \"Apache-2.0 OR MIT\"\n\n[templates]\nmit-apache = \"docs/HEADER\"\n\n[[mapping]]\npaths = [\".\"]\ntemplate = \"mit-apache\"\n",
            );
            fs::write(dir.join("forehead.toml"), config).unwrap();

            for (rel, content) in files {
                let p = dir.join(rel);
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(p, content).unwrap();
            }

            TestProject { dir }
        }

        fn forehead(&self) -> Forehead {
            let config = Config::from_path(&self.dir.join("forehead.toml")).unwrap();
            Forehead::new(config)
        }

        fn applied_rel(&self) -> Vec<String> {
            self.forehead()
                .apply(false)
                .unwrap()
                .applied
                .iter()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .collect()
        }

        fn read(&self, rel: &str) -> String {
            fs::read_to_string(self.dir.join(rel)).unwrap()
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn toml_files_ignored_by_default() {
        let tp = TestProject::new(
            &[
                ("main.rs", "fn main() {}\n"),
                ("settings.toml", "[foo]\nbar = 1\n"),
                ("rustfmt.toml", "max_width = 100\n"),
            ],
            &[],
        );
        assert_eq!(tp.applied_rel(), vec!["main.rs".to_string()]);
        assert_eq!(tp.read("settings.toml"), "[foo]\nbar = 1\n");
        assert_eq!(tp.read("rustfmt.toml"), "max_width = 100\n");
    }

    #[test]
    fn cargo_toml_license_field_still_synced() {
        let tp = TestProject::new(
            &[(
                "Cargo.toml",
                "[package]\nname = \"foo\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n",
            )],
            &[],
        );
        let applied = tp.applied_rel();
        assert!(applied.contains(&"Cargo.toml".to_string()));
        let content = tp.read("Cargo.toml");
        assert!(content.contains("license = \"Apache-2.0 OR MIT\""));
        assert!(!content.contains("SPDX"), "no comment header added to toml");
    }

    #[test]
    fn ignore_config_excludes_matching_dirs() {
        let tp = TestProject::new(
            &[
                ("src/main.rs", "fn main() {}\n"),
                ("vendor/third_party.rs", "// generated\n"),
                ("nested/vendor/other.rs", "// generated\n"),
            ],
            &["vendor"],
        );
        assert_eq!(tp.applied_rel(), vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn ignore_config_excludes_matching_files() {
        let tp = TestProject::new(
            &[
                ("main.rs", "fn main() {}\n"),
                ("generated.rs", "// generated\n"),
            ],
            &["generated.rs"],
        );
        assert_eq!(tp.applied_rel(), vec!["main.rs".to_string()]);
    }

    #[test]
    fn ignore_path_suffix_matches_subdirs() {
        let tp = TestProject::new(
            &[
                ("src/main.rs", "fn main() {}\n"),
                ("src/gen.rs", "// generated\n"),
                ("gen.rs", "// generated too\n"),
            ],
            &["gen.rs"],
        );
        assert_eq!(tp.applied_rel(), vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn hardcoded_skips_still_applied_with_no_ignore() {
        let tp = TestProject::new(
            &[
                ("main.rs", "fn main() {}\n"),
                (".github/workflows/ci.yml", "# workflow\n"),
                ("target/debug/out.rs", "// built\n"),
                (".git/hooks/pre-commit.rs", "// git\n"),
            ],
            &[],
        );
        assert_eq!(tp.applied_rel(), vec!["main.rs".to_string()]);
    }

    #[test]
    fn legacy_skip_alias_still_parses() {
        let tp = TestProject::new(
            &[("main.rs", "fn main() {}\n"), ("skipme.rs", "// x\n")],
            &["skipme.rs"],
        );
        // Rewrite the config to use the legacy `skip` key instead of `ignore`.
        let cfg_path = tp.dir.join("forehead.toml");
        let cfg = fs::read_to_string(&cfg_path)
            .unwrap()
            .replace("ignore = [\"skipme.rs\"]", "skip = [\"skipme.rs\"]");
        fs::write(&cfg_path, cfg).unwrap();

        let config = Config::from_path(&cfg_path).unwrap();
        assert_eq!(config.ignore, vec!["skipme.rs".to_string()]);
        let applied = Forehead::new(config)
            .apply(false)
            .unwrap()
            .applied
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();
        assert_eq!(applied, vec!["main.rs".to_string()]);
    }

    #[test]
    fn stale_header_replaced_not_duplicated() {
        let tp = TestProject::new(
            &[(
                "main.rs",
                "use std::io;\n// Copyright (C) 2020 Old Author.\n// SPDX-License-Identifier: MIT\n\nfn main() {}\n",
            )],
            &[],
        );

        let report1 = tp.forehead().apply(false).unwrap();
        let contents1 = tp.read("main.rs");
        assert!(
            contents1.starts_with("// This file is part of TestProj."),
            "new header at top: {contents1:?}"
        );
        assert!(!contents1.contains("Old Author"));
        assert!(contents1.contains("use std::io;"));
        assert!(report1
            .applied
            .iter()
            .any(|p| p.to_string_lossy().ends_with("main.rs")));

        // Second apply is idempotent — header is not duplicated.
        let report2 = tp.forehead().apply(false).unwrap();
        assert!(!report2
            .applied
            .iter()
            .any(|p| p.to_string_lossy().ends_with("main.rs")));
        assert_eq!(tp.read("main.rs"), contents1);
    }
}
