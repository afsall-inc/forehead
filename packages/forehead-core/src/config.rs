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

use crate::{error::ForeheadError, template::HeaderTemplate};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub project_dir: PathBuf,

    pub project: ProjectConfig,

    pub templates: HashMap<String, String>,

    #[serde(default)]
    pub mapping: Vec<Mapping>,

    #[serde(default)]
    pub header: HeaderConfig,

    #[serde(default)]
    pub file_types: HashMap<String, FileTypeConfig>,

    #[serde(default)]
    pub skip: Vec<String>,

    #[serde(default)]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HeaderConfig {
    #[serde(default)]
    pub indicators: Vec<String>,
    #[serde(default)]
    pub greetings: String,
}

const BUILTIN_INDICATORS: &[&str] = &["Copyright", "SPDX", "License"];

impl HeaderConfig {
    pub fn all_indicators(&self) -> Vec<String> {
        if self.indicators.len() == 1 && self.indicators[0].eq_ignore_ascii_case("none") {
            return vec![];
        }
        let mut all: Vec<String> = BUILTIN_INDICATORS.iter().map(|s| s.to_string()).collect();
        all.extend(self.indicators.iter().cloned());
        all
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub default_license: String,
    #[serde(default = "default_author")]
    pub default_author: String,
    #[serde(default = "default_year")]
    pub default_year: u32,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub year_span: String,
}

fn default_author() -> String {
    "Afsall Inc".to_string()
}

fn default_year() -> u32 {
    2026
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mapping {
    pub paths: Vec<String>,
    pub template: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub year_span: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileTypeConfig {
    pub line: Option<String>,
    pub block: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Substitution {
    pub project: String,
    pub author: String,
    pub year: String,
    pub year_span: String,
    pub license: String,
    pub repository: String,
    pub description: String,
    pub file: String,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, ForeheadError> {
        let contents = fs::read_to_string(path).map_err(|e| {
            ForeheadError::Config(format!("failed to read {}: {e}", path.display()))
        })?;
        let mut config: Config = toml::from_str(&contents).map_err(|e| {
            ForeheadError::Config(format!("failed to parse {}: {e}", path.display()))
        })?;
        config.project_dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| {
                // If parent is empty (config in current dir), use "."
                std::path::Path::new(".")
            })
            .to_path_buf();

        // Normalize to absolute if possible
        if let Ok(cwd) = std::env::current_dir() {
            if config.project_dir.is_relative() {
                config.project_dir = cwd.join(&config.project_dir);
            }
        }

        // Resolve template paths relative to config file
        let base = config.project_dir.clone();
        for (_, template_path) in config.templates.iter_mut() {
            let p = Path::new(&template_path);
            if p.is_relative() {
                *template_path = base.join(p).to_string_lossy().to_string();
            }
        }

        Ok(config)
    }

    pub fn template_for(&self, file_path: &Path, root: &Path) -> Option<HeaderTemplate> {
        let rel = file_path.strip_prefix(root).unwrap_or(file_path);
        let rel_str = rel.to_string_lossy();

        let template_key = self
            .mapping
            .iter()
            .find(|m| {
                m.paths
                    .iter()
                    .any(|p| p.as_str() == "." || p.is_empty() || rel_str.starts_with(p.as_str()))
            })
            .map(|m| &m.template);

        let template_key = template_key.unwrap_or_else(|| {
            self.templates
                .keys()
                .next()
                .expect("at least one template required")
        });

        let tpath = self.templates.get(template_key)?;
        HeaderTemplate::from_file(Path::new(tpath)).ok()
    }

    pub fn substitution_for(&self, file_path: &Path, root: &Path) -> Substitution {
        let rel = file_path.strip_prefix(root).unwrap_or(file_path);
        let rel_str = rel.to_string_lossy();
        let fname = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        let mapping = self.mapping.iter().find(|m| {
            m.paths
                .iter()
                .any(|p| p.as_str() == "." || p.is_empty() || rel_str.starts_with(p.as_str()))
        });

        let project = mapping
            .and_then(|m| m.project.clone())
            .unwrap_or_else(|| self.project.name.clone());

        let author = mapping
            .and_then(|m| m.author.clone())
            .unwrap_or_else(|| self.project.default_author.clone());

        let year = mapping
            .and_then(|m| m.year)
            .unwrap_or(self.project.default_year)
            .to_string();

        let year_span = mapping
            .and_then(|m| m.year_span.clone())
            .unwrap_or_else(|| {
                if self.project.year_span.is_empty() {
                    format!("{}-Present", self.project.default_year)
                } else {
                    self.project.year_span.clone()
                }
            });

        let license = mapping
            .and_then(|m| m.license.clone())
            .unwrap_or_else(|| self.project.default_license.clone());

        let repository = mapping
            .and_then(|m| m.repository.clone())
            .unwrap_or_else(|| self.project.repository.clone());

        let description = mapping
            .and_then(|m| m.description.clone())
            .unwrap_or_else(|| self.project.description.clone());

        Substitution {
            project,
            author,
            year,
            year_span,
            license,
            repository,
            description,
            file: fname.to_string(),
        }
    }

    pub fn license_for(&self, file_path: &Path, root: &Path) -> String {
        let rel = file_path.strip_prefix(root).unwrap_or(file_path);
        let rel_str = rel.to_string_lossy();

        let mapping = self.mapping.iter().find(|m| {
            m.paths
                .iter()
                .any(|p| p.as_str() == "." || p.is_empty() || rel_str.starts_with(p.as_str()))
        });

        mapping
            .and_then(|m| m.license.clone())
            .unwrap_or_else(|| self.project.default_license.clone())
    }
}
