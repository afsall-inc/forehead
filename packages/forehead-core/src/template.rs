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

use crate::{config::Substitution, error::ForeheadError};
use std::{fs, path::Path};

#[derive(Debug, Clone)]
pub struct HeaderTemplate {
    pub content: String,
    pub lines: Vec<String>,
}

impl HeaderTemplate {
    pub fn from_file(path: &Path) -> Result<Self, ForeheadError> {
        let content = fs::read_to_string(path).map_err(|e| {
            ForeheadError::Template(format!("failed to read {}: {e}", path.display()))
        })?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        Ok(HeaderTemplate { content, lines })
    }

    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        HeaderTemplate {
            content: content.to_string(),
            lines,
        }
    }

    pub fn substitute(&self, subst: &Substitution) -> String {
        let mut result = String::new();
        for line in &self.lines {
            let rendered = line
                .replace("{project}", &subst.project)
                .replace("{author}", &subst.author)
                .replace("{year}", &subst.year)
                .replace("{year_span}", &subst.year_span)
                .replace("{license}", &subst.license)
                .replace("{repository}", &subst.repository)
                .replace("{description}", &subst.description)
                .replace("{file}", &subst.file);
            result.push_str(&rendered);
            result.push('\n');
        }
        result
    }

    /// Returns the template as a list of comment lines with the given comment style.
    /// Each line has the comment prefix applied.
    pub fn as_comment_lines(&self, comment_prefix: &str, subst: &Substitution) -> Vec<String> {
        let mut result = Vec::new();
        for line in &self.lines {
            let rendered = line
                .replace("{project}", &subst.project)
                .replace("{author}", &subst.author)
                .replace("{year}", &subst.year)
                .replace("{year_span}", &subst.year_span)
                .replace("{license}", &subst.license)
                .replace("{repository}", &subst.repository)
                .replace("{description}", &subst.description)
                .replace("{file}", &subst.file);
            if rendered.trim().is_empty() {
                result.push(comment_prefix.to_string());
            } else {
                result.push(format!("{} {}", comment_prefix, rendered.trim()));
            }
        }
        result
    }
}
