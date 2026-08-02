# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- feat: add forehead replace command [forehead-core(minor), forehead-cli(minor)]
- feat: initial forehead implementation — Rust tool for license headers [forehead-core(minor), forehead-cli(minor)]
- feat: remove .md header support, add forehead remove command, enable greetings [forehead-core(minor), forehead-cli(minor)]

### Changed

- chore: align Docker tag format with release tag (v prefix) [forehead-core(none), forehead-cli(none)]
- chore: fix CI — validate existing prdoc, don't auto-generate [forehead-core(none), forehead-cli(none)]
- chore: fix release workflow permissions and push auth [forehead-core(none), forehead-cli(none)]
- chore: fix release workflow — use gh release create [forehead-core(none), forehead-cli(none)]

### Fixed

- chore: add CI/CD release workflow with auto-publish on version bump [forehead-core(patch), forehead-cli(patch)]
- docs: update AGENTS.md and README.md with remove command and dry-run [forehead-core(patch), forehead-cli(patch)]
- feat: replace prdoc with changelogger, bump v0.1.4 [forehead-core(patch), forehead-cli(patch)]

