# Changelog

Notable changes to nara, newest first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), the versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html), and every release carries a signed `vX.Y.Z` tag on the commit that bumps this file.

A version lists the `feat`, `fix`, and `perf` commits since the previous tag, because the commit scheme shows only those in a changelog.
`build`, `chore`, `ci`, `docs`, `ref`, `release`, `revert`, `style`, and `test` stay in `git log`.

## [Unreleased]

## [0.1.0] - 2026-10-16

### Added

- Cargo workspace in which one crate per video returns an SVG tree for every frame and fframes encodes it, with 60 fps portrait output, an FFV1 master, and the H.264 delivery steps.
- `nara-video` agent skill, with the fframes primary source pinned to a commit: setup, narration, scene design, review, render, the portrait safe zones, brand-colour conversion, the UI-walkthrough and text references, and the render and loudness checks.
- `tools/tts.py`, the narration pipeline for Google Cloud Text-to-Speech: it locates the service account key, lists the voices a language offers, retries transient failures, and prints each take's length.
- `AGENTS.md` and `.agents/skills/nara-video`, mirroring the Claude Code layout so OpenCode and other agents load the same workspace rules and skill.
- README with the layout, requirements, manual commands, and narration setup, plus the MIT license.

[Unreleased]: https://github.com/ryhkml/nara/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ryhkml/nara/releases/tag/v0.1.0
