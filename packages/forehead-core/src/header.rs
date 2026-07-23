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
    comment::{format_as_comment, CommentStyle},
    config::Substitution,
    error::ForeheadError,
    template::HeaderTemplate,
};
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Correct,
    Missing,
    Wrong,
}

/// Check if a file has the correct header
pub fn check_header_on_file(
    path: &Path,
    template: &HeaderTemplate,
    style: &CommentStyle,
    subst: &Substitution,
    indicators: &[String],
    greetings: &str,
) -> Result<FileStatus, ForeheadError> {
    let content = fs::read_to_string(path)?;
    let expected = build_expected_header(template, style, subst, greetings);

    let header_block = extract_header_block(&content, style, indicators);

    match header_block {
        Some(existing) => {
            if normalize_header(&existing) == normalize_header(&expected) {
                Ok(FileStatus::Correct)
            } else {
                Ok(FileStatus::Wrong)
            }
        }
        None => Ok(FileStatus::Missing),
    }
}

/// Apply a header to a file. Returns true if the file was modified.
pub fn apply_header_to_file(
    path: &Path,
    template: &HeaderTemplate,
    style: &CommentStyle,
    subst: &Substitution,
    dry_run: bool,
    indicators: &[String],
    greetings: &str,
) -> Result<bool, ForeheadError> {
    let content = fs::read_to_string(path)?;
    let expected = build_expected_header(template, style, subst, greetings);

    let header_block = extract_header_block(&content, style, indicators);

    let new_content = match header_block {
        Some(existing) => {
            if normalize_header(&existing) == normalize_header(&expected) {
                return Ok(false);
            }
            replace_header(&content, &existing, &expected, style)
        }
        None => prepend_header(&content, &expected, style),
    };

    if new_content == content {
        return Ok(false);
    }

    if !dry_run {
        fs::write(path, &new_content)?;
    }

    Ok(true)
}

fn build_expected_header(
    template: &HeaderTemplate,
    style: &CommentStyle,
    subst: &Substitution,
    greetings: &str,
) -> String {
    let mut all_lines = template.lines.clone();
    if !greetings.is_empty() {
        let substituted = substitute_header_text(greetings, subst);
        all_lines.insert(0, substituted);
    }
    let formatted = format_as_comment(&all_lines, style);
    substitute_header(&formatted, subst)
}

/// Extract the header comment block from file content
fn extract_header_block(
    content: &str,
    style: &CommentStyle,
    indicators: &[String],
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }

    match style {
        CommentStyle::Line(prefix) => {
            let mut header_lines = Vec::new();
            let mut has_indicator = indicators.is_empty(); // if no indicators, any block qualifies

            for line in &lines {
                let trimmed = line.trim();
                if trimmed.starts_with(prefix) {
                    header_lines.push(*line);
                    if !has_indicator {
                        has_indicator = is_header_line(trimmed, indicators);
                    }
                } else if trimmed.is_empty() {
                    if !header_lines.is_empty() {
                        header_lines.push(*line);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if header_lines.is_empty() || !has_indicator {
                None
            } else {
                Some(header_lines.join("\n"))
            }
        }
        CommentStyle::Block(open, close) => {
            let mut in_block = false;
            let mut block_lines = Vec::new();
            let mut found = false;
            let mut has_indicator = indicators.is_empty();

            for line in &lines {
                let trimmed = line.trim();
                if trimmed.starts_with(open) && !in_block {
                    let rest = trimmed.strip_prefix(open).unwrap_or("").trim();
                    if !has_indicator {
                        has_indicator = is_header_line(rest, indicators);
                    }
                    if has_indicator || rest.is_empty() {
                        in_block = true;
                        block_lines.push(*line);
                    } else {
                        break;
                    }
                } else if in_block {
                    block_lines.push(*line);
                    if !has_indicator {
                        has_indicator = is_header_line(trimmed, indicators);
                    }
                    if trimmed.ends_with(close) {
                        found = true;
                        break;
                    }
                } else {
                    break;
                }
            }

            if found && has_indicator {
                Some(block_lines.join("\n"))
            } else {
                None
            }
        }
        CommentStyle::BlockTriple(_delim) => {
            let mut in_block = false;
            let mut block_lines = Vec::new();
            let mut found = false;
            let mut has_indicator = indicators.is_empty();

            for line in &lines {
                let trimmed = line.trim();
                if (trimmed == "\"\"\"" || trimmed == "'''") && !in_block {
                    in_block = true;
                    block_lines.push(*line);
                } else if in_block {
                    block_lines.push(*line);
                    if !has_indicator {
                        has_indicator = is_header_line(trimmed, indicators);
                    }
                    if trimmed == "\"\"\"" || trimmed == "'''" {
                        found = true;
                        break;
                    }
                } else {
                    break;
                }
            }

            if found && has_indicator {
                Some(block_lines.join("\n"))
            } else {
                None
            }
        }
    }
}

fn is_header_line(line: &str, indicators: &[String]) -> bool {
    if indicators.is_empty() {
        return true;
    }
    let lower = line.to_lowercase();
    indicators.iter().any(|i| lower.contains(&i.to_lowercase()))
}

fn normalize_header(header: &str) -> String {
    header
        .lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn substitute_header(header: &str, subst: &Substitution) -> String {
    substitute_header_text(header, subst)
}

fn substitute_header_text(text: &str, subst: &Substitution) -> String {
    text.replace("{project}", &subst.project)
        .replace("{author}", &subst.author)
        .replace("{year}", &subst.year)
        .replace("{year_span}", &subst.year_span)
        .replace("{license}", &subst.license)
        .replace("{repository}", &subst.repository)
        .replace("{description}", &subst.description)
        .replace("{file}", &subst.file)
}

fn replace_header(
    content: &str,
    old_header: &str,
    new_header: &str,
    _style: &CommentStyle,
) -> String {
    if let Some(pos) = content.find(old_header) {
        let _before = &content[..pos];
        let after = &content[pos + old_header.len()..];
        let after = after.trim_start();
        format!("{}\n\n{}", new_header, after)
    } else {
        prepend_header(content, new_header, _style)
    }
}

fn prepend_header(content: &str, header: &str, _style: &CommentStyle) -> String {
    let trimmed = content.trim_start();
    format!("{}\n\n{}", header, trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{comment::CommentStyle, config::Substitution, template::HeaderTemplate};

    fn test_subst() -> Substitution {
        Substitution {
            project: "TestProject".into(),
            author: "Test Author".into(),
            year: "2026".into(),
            year_span: "2026-Present".into(),
            license: "Apache-2.0 OR MIT".into(),
            repository: "https://github.com/test/test".into(),
            description: "Test description".into(),
            file: "test.rs".into(),
        }
    }

    #[test]
    fn test_prepend_header_to_rust_file() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "pub fn hello() {}\n";
        let expected = "// This file is part of TestProject.\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let result = apply_header_to_file_str(content, &template, &style, &subst);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_existing_header() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content =
            "// Old copyright notice.\n// SPDX-License-Identifier: MIT\n\npub fn hello() {}\n";
        let expected = "// This file is part of TestProject.\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let result = apply_header_to_file_str(content, &template, &style, &subst);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_header_already_correct() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "// This file is part of TestProject.\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let result = apply_header_to_file_str(content, &template, &style, &subst);
        // Should be unchanged (returns same content)
        assert_eq!(result, content);
    }

    #[test]
    fn test_python_hash_comment() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("#".into());
        let subst = test_subst();

        let content = "def hello():\n    pass\n";
        let expected = "# This file is part of TestProject.\n# Copyright (C) 2026-Present Test Author.\n# SPDX-License-Identifier: Apache-2.0 OR MIT.\n\ndef hello():\n    pass\n";

        let result = apply_header_to_file_str(content, &template, &style, &subst);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_html_block_comment() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Block("<!--".into(), "-->".into());
        let subst = test_subst();

        let content = "<html><body>Hello</body></html>\n";
        let result = apply_header_to_file_str(content, &template, &style, &subst);

        assert!(result.contains("<!--"));
        assert!(result.contains("-->"));
        assert!(result.contains("This file is part of TestProject."));
        assert!(result.contains("<html><body>Hello</body></html>"));
    }

    #[test]
    fn test_check_correct_header() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "// This file is part of TestProject.\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let status = check_header_on_file_str(content, &template, &style, &subst);
        assert_eq!(status, FileStatus::Correct);
    }

    #[test]
    fn test_check_missing_header() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "pub fn hello() {}\n";

        let status = check_header_on_file_str(content, &template, &style, &subst);
        assert_eq!(status, FileStatus::Missing);
    }

    #[test]
    fn test_check_wrong_header() {
        let template = HeaderTemplate::new("This file is part of {project}.\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "// Copyright (C) 2020 Wrong Author.\n// SPDX-License-Identifier: MIT\n\npub fn hello() {}\n";

        let status = check_header_on_file_str(content, &template, &style, &subst);
        assert_eq!(status, FileStatus::Wrong);
    }

    // Helper functions that work on strings directly
    fn default_indicators() -> Vec<String> {
        vec![
            "Copyright".to_string(),
            "SPDX".to_string(),
            "License".to_string(),
        ]
    }

    fn apply_header_to_file_str(
        content: &str,
        template: &HeaderTemplate,
        style: &CommentStyle,
        subst: &Substitution,
    ) -> String {
        apply_header_to_file_str_with(content, template, style, subst, &default_indicators(), "")
    }

    fn apply_header_to_file_str_with(
        content: &str,
        template: &HeaderTemplate,
        style: &CommentStyle,
        subst: &Substitution,
        indicators: &[String],
        greetings: &str,
    ) -> String {
        let expected = build_expected_header(template, style, subst, greetings);

        let header_block = extract_header_block(content, style, indicators);

        match header_block {
            Some(existing) => {
                if normalize_header(&existing) == normalize_header(&expected) {
                    return content.to_string();
                }
                replace_header(content, &existing, &expected, style)
            }
            None => prepend_header(content, &expected, style),
        }
    }

    fn check_header_on_file_str(
        content: &str,
        template: &HeaderTemplate,
        style: &CommentStyle,
        subst: &Substitution,
    ) -> FileStatus {
        check_header_on_file_str_with(content, template, style, subst, &default_indicators(), "")
    }

    fn check_header_on_file_str_with(
        content: &str,
        template: &HeaderTemplate,
        style: &CommentStyle,
        subst: &Substitution,
        indicators: &[String],
        greetings: &str,
    ) -> FileStatus {
        let expected = build_expected_header(template, style, subst, greetings);

        let header_block = extract_header_block(content, style, indicators);

        match header_block {
            Some(existing) => {
                if normalize_header(&existing) == normalize_header(&expected) {
                    FileStatus::Correct
                } else {
                    FileStatus::Wrong
                }
            }
            None => FileStatus::Missing,
        }
    }

    #[test]
    fn test_greetings_in_header() {
        let template = HeaderTemplate::new(
            "Copyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.",
        );
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();

        let content = "// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let expected = "// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let result = apply_header_to_file_str_with(
            content,
            &template,
            &style,
            &subst,
            &default_indicators(),
            "بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم",
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_custom_indicator() {
        let template = HeaderTemplate::new("CustomTag: Active\nCopyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.");
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();
        let mut indicators = default_indicators();
        indicators.push("CustomTag".to_string());

        let content = "// CustomTag: Active\n// Copyright (C) 2026-Present Test Author.\n// SPDX-License-Identifier: Apache-2.0 OR MIT.\n\npub fn hello() {}\n";

        let status =
            check_header_on_file_str_with(content, &template, &style, &subst, &indicators, "");
        assert_eq!(status, FileStatus::Correct);
    }

    #[test]
    fn test_none_indicator() {
        let template = HeaderTemplate::new(
            "Copyright (C) {year_span} {author}.\nSPDX-License-Identifier: {license}.",
        );
        let style = CommentStyle::Line("//".into());
        let subst = test_subst();
        let indicators: Vec<String> = vec![]; // "none" sentinel results in empty vec

        let content = "// Some random comment.\n// Another random line.\n\npub fn hello() {}\n";

        let status =
            check_header_on_file_str_with(content, &template, &style, &subst, &indicators, "");
        assert_eq!(status, FileStatus::Wrong);
    }
}
