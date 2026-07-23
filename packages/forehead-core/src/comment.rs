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

use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum CommentStyle {
    Line(String),
    Block(String, String),
    /// For Python docstrings / multiline string comments
    BlockTriple(String),
}

impl CommentStyle {
    pub fn prefix(&self) -> &str {
        match self {
            CommentStyle::Line(p) => p,
            CommentStyle::Block(o, _) => o,
            CommentStyle::BlockTriple(d) => d,
        }
    }
}

pub fn comment_style_for(path: &Path) -> Option<CommentStyle> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Check by exact filename
    match fname {
        "Makefile" | "makefile" | "Dockerfile" | "Containerfile" => {
            return Some(CommentStyle::Line("#".to_string()));
        }
        "CMakeLists.txt" => return Some(CommentStyle::Line("#".to_string())),
        "Rakefile" | "Gemfile" => return Some(CommentStyle::Line("#".to_string())),
        _ => {}
    }

    // Check by extension
    match ext {
        // // line comments
        "rs" | "go" | "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "hh" | "java" | "js" | "ts"
        | "jsx" | "tsx" | "kt" | "kts" | "swift" | "zig" | "dart" | "php" | "m" | "mm" | "cs"
        | "scala" | "groovy" | "vala" | "v" | "nu" | "svelte" | "vue" => {
            Some(CommentStyle::Line("//".to_string()))
        }

        // # line comments
        "py" | "rb" | "pl" | "pm" | "sh" | "bash" | "zsh" | "fish" | "yaml" | "yml" | "toml"
        | "r" | "R" | "rake" | "gemspec" | "erb" | "el" | "ex" | "exs" | "jl" | "coffee"
        | "csh" | "awk" | "tcl" | "mk" | "ninja" | "bzl" | "BUILD" | "WORKSPACE" | "dhall"
        | "nix" | "conf" | "cfg" | "ini" | "desktop" | "service" | "timer" | "diff" | "patch"
        | "ziggy" | "ziggy-schema" => Some(CommentStyle::Line("#".to_string())),

        // -- line comments
        "sql" | "hs" | "lhs" | "lua" | "ada" | "adb" | "ads" | "sqlite" | "sqlite3" | "vhd"
        | "vhdl" => Some(CommentStyle::Line("--".to_string())),

        // % line comments
        "tex" | "sty" | "cls" | "ltx" | "bib" | "matlab" | "maxima" | "prolog" => {
            Some(CommentStyle::Line("%".to_string()))
        }

        // ; line comments
        "lisp" | "lsp" | "clj" | "cljs" | "cljc" | "edn" | "scm" | "ss" => {
            Some(CommentStyle::Line(";".to_string()))
        }

        // <!-- --> block comments
        "html" | "htm" | "xhtml" | "xml" | "xsl" | "xslt" | "xsd" | "svg" | "md" | "markdown"
        | "rmd" | "mdown" | "mkdn" | "mdx" => {
            Some(CommentStyle::Block("<!--".to_string(), "-->".to_string()))
        }

        // /* */ block comments
        "css" | "scss" | "sass" | "less" | "graphql" | "gql" | "proto" | "flatbuffers" | "fbs"
        | "solidity" | "sol" => Some(CommentStyle::Block("/*".to_string(), "*/".to_string())),

        _ => None,
    }
}

pub fn format_as_comment(lines: &[String], style: &CommentStyle) -> String {
    match style {
        CommentStyle::Line(prefix) => lines
            .iter()
            .map(|l| {
                if l.trim().is_empty() {
                    prefix.clone()
                } else {
                    format!("{} {}", prefix, l.trim())
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommentStyle::Block(open, close) => {
            let mut result = open.clone();
            result.push('\n');
            for line in lines {
                if line.trim().is_empty() {
                    result.push_str(open);
                    result.push('\n');
                } else {
                    result.push_str(&format!("{} {}", open, line.trim()));
                    result.push('\n');
                }
            }
            result.push_str(close);
            result.push('\n');
            result
        }
        CommentStyle::BlockTriple(delim) => {
            let mut result = delim.clone();
            result.push('\n');
            for line in lines {
                result.push_str(line.trim());
                result.push('\n');
            }
            result.push_str(delim);
            result.push('\n');
            result
        }
    }
}
