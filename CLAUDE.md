# Workspace

Best practice for making videos as code with an AI agent, applied on top of fframes.
The `nara-video` skill holds the method, and fframes itself is documented by its primary source, <https://github.com/dmtrKovalenko/fframes>.

- `videos/<name>/`: one Cargo crate per video, all in one workspace; `videos/*` is gitignored, so those crates stay local.
- `tools/tts.py`: narration synthesis, used by the skill's narration step. It finds the key at `~/.config/gcloud-keys/tts-video.json` when `GOOGLE_APPLICATION_CREDENTIALS` is unset; run `uv run tools/tts.py --help`.
- Versions: a release opens a new `## [x.y.z]` section in `CHANGELOG.md` and adds a signed `X.Y.Z` tag on that same commit; the section lists the `feat`, `fix`, and `perf` commits since the previous tag; tag names carry no `v` prefix.
- Publishing that release reuses the same section as the GitHub release notes, so the changelog stays the single source.

The Bash tool runs bash while the login shell is fish, so set variables with `export VAR=value`.
