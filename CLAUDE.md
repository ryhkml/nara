# Workspace

Best practice for making videos as code with an AI agent, applied on top of fframes.
The `nara-video` skill holds the method, and fframes itself is documented by its primary source, <https://github.com/dmtrKovalenko/fframes>.

- `videos/<name>/`: one Cargo crate per video, all in one workspace; `videos/*` is gitignored, so those crates stay local.
- `tools/tts.py`: narration synthesis, used by the skill's narration step. It finds the key at `~/.config/gcloud-keys/tts-video.json` when `GOOGLE_APPLICATION_CREDENTIALS` is unset; run `uv run tools/tts.py --help`.
- Versions: a release opens a new `## [x.y.z]` section in `CHANGELOG.md` and adds a signed `X.Y.Z` tag on that same commit; the section lists the `feat`, `fix`, and `perf` commits since the previous tag; tag names carry no `v` prefix.
- Publishing that release reuses the same section as the GitHub release notes, so the changelog stays the single source.
- Git: run no `git commit`, `git tag`, or `git push` unless the user asks for that step in the message that triggers the work.

Narration pitch is set per request, not per voice name. Chirp 3: HD resamples the register on every call, so one request per `vo_NN` block lets the same speaker land up to about 6 st apart; one sentence measured 121 Hz in one request and 178 Hz in another.

- Synthesize the whole script as one SSML request with `<break time="2000ms"/>` between blocks, then cut the take at those silences. The video then holds one performance and no request boundary falls inside it.
- Never lift part of the narration with `<prosody pitch>`. A `+3st` on one question was audible as a different speaker; the engine's own `?` contour already rises, measured 78 to 242 Hz over the last 0.25 s.
- Judge uniformity by the same sentence across requests only. Median F0 differs with the text, so a table of per-line medians cannot separate register drift from intonation.

Editing a file is not permission to commit it, and finishing a task is not permission to publish it.
A request to improve a video, fix a bug, or update a document never covers the commit, the tag, or the push.
When the work looks ready to publish, say so and wait for the instruction.

The Bash tool runs bash while the login shell is fish, so set variables with `export VAR=value`.
