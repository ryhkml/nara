# Workspace

Best practice for making videos as code with an AI agent, applied on top of fframes.
The `nara-video` skill holds the method, and fframes itself is documented by its primary source, <https://github.com/dmtrKovalenko/fframes>.

- `videos/<name>/`: one Cargo crate per video, all in one workspace.
- `tools/tts.py`: narration synthesis, used by the skill's narration step. It finds the key at `~/.config/gcloud-keys/tts-video.json` when `GOOGLE_APPLICATION_CREDENTIALS` is unset; run `uv run tools/tts.py --help`.

The Bash tool runs bash while the login shell is fish, so set variables with `export VAR=value`.
