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

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "forehead",
    version,
    about = "Create and maintain file headers for code license"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Apply license headers to all source files
    Apply {
        /// Path to forehead.toml config
        #[arg(short, long, default_value = "forehead.toml")]
        config: String,

        /// Dry run — show what would be changed without modifying
        #[arg(short, long)]
        dry_run: bool,

        /// Directory to process (defaults to config file's directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Check that all files have correct headers (CI mode, exit 1 on failure)
    Check {
        /// Path to forehead.toml config
        #[arg(short, long, default_value = "forehead.toml")]
        config: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Directory to process
        #[arg(short, long)]
        path: Option<String>,
    },
    /// List files and their header status
    List {
        /// Path to forehead.toml config
        #[arg(short, long, default_value = "forehead.toml")]
        config: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Directory to process
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Scaffold a forehead.toml config file in the current directory
    Init {
        /// Path to write the config file
        #[arg(short, long, default_value = "forehead.toml")]
        path: String,
    },
    /// Remove license headers from all source files
    Remove {
        /// Path to forehead.toml config
        #[arg(short, long, default_value = "forehead.toml")]
        config: String,

        /// Dry run — show what would be changed without modifying
        #[arg(short, long)]
        dry_run: bool,

        /// Directory to process
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Replace invalid or old headers with the correct one
    Replace {
        /// Path to forehead.toml config
        #[arg(short, long, default_value = "forehead.toml")]
        config: String,

        /// Dry run — show what would be changed without modifying
        #[arg(short, long)]
        dry_run: bool,

        /// Directory to process
        #[arg(short, long)]
        path: Option<String>,
    },
}
