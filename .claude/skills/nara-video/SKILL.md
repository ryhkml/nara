---
name: nara-video
description: This workspace's best practice for making a video as code with fframes, applied on top of the fframes-video skill. Use when starting, narrating, designing, rendering, or delivering a video in nara.
---

# Making a Video in nara

The primary source for fframes is its repository, <https://github.com/dmtrKovalenko/fframes>, and the agent skill it ships in `skills/fframes-video/`: `SKILL.md` plus `references/design.md`, `api.md`, and `audio.md`.
This skill records only how nara applies fframes, and the primary source wins wherever the two seem to differ.
Read the primary source at the commit the workspace builds, so the API it describes matches the code; append a file path to this base URL:

```text
https://raw.githubusercontent.com/dmtrKovalenko/fframes/dca70765c82d8d3a0b5916222ae7fc5c98cd42c9/skills/fframes-video/
```

Paths under `references/` are inside this skill; every other path is relative to the repository root.

## Steps

1. **Primary source.** Fetch the `fframes-video` `SKILL.md` before writing any video code, and its `references/` files when the work reaches their topic.
2. **Setup.** For a new machine or a new video crate, follow `references/setup.md`. Done when `cargo build --release -p <package>` succeeds from the repository root.
3. **Narration**, only for a video with a voice-over. Follow `references/narration.md`. Done when every `vo_NN` block in `narration.txt` has a processed `media/vo_NN.wav` and its measured length in `src/lib.rs`.
4. **Scenes.** Design with the primary source's design guide, and with `references/techniques.md` for a UI walkthrough, a portrait layout, or brand colors. Done when `inspect --fail-on warning` passes, a strip of every scene has been viewed, and, for a TikTok or Reels upload, a shaded frame of every scene shows its text inside the safe zone from `references/techniques.md`.
5. **Render and deliver.** Follow `references/render.md`, which renders with `TMPDIR` on disk because `/tmp` is a RAM-backed tmpfs that fframes leaves full. Done when the MP4's frame count equals the `timeline` count and its video and audio durations match.
