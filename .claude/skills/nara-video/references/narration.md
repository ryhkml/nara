# Narration

Read this when a video has a voice-over.
A video without narration skips this file.

## Pipeline

| Step                                 | Output                                  | Tool                                                |
| ------------------------------------ | --------------------------------------- | --------------------------------------------------- |
| 1. Write the script                  | `videos/<name>/narration.txt`           | Text editor                                         |
| 2. Synthesize the voice              | `videos/<name>/audio/<voice>/vo_NN.wav` | `uv run tools/tts.py videos/<name> --voice <voice>` |
| 3. Trim, retime, and level the voice | `videos/<name>/media/vo_NN.wav`         | FFmpeg                                              |
| 4. Measure durations and pauses      | Timing constants in `src/lib.rs`        | `ffprobe`, `silencedetect`                          |
| 5. Place the tracks and check levels | Audio map, LUFS report                  | fframes `audio analyze`                             |

## Script Rules

- One `vo_NN` block per scene. The line under the label is what TTS reads and what the subtitle shows, word for word.
- Formal Indonesian with "Anda", complete short sentences, at most about 22 words per block.
- UI labels stay in the UI's own language, because the viewer has to find them on screen.
- Every fact in the narration comes from an authoritative source, and anything the video shows on a live system is checked against that system.
- `tools/tts.py` reads only a `vo_NN` line and the line after it, so other lines can carry notes for a human voice actor, such as how to pronounce a term.

```text
vo_01
<what the voice says in scene 1>

vo_02
<what the voice says in scene 2>
```

## Google Cloud Setup

1. Create or pick a project and enable billing. Google requires billing for Text-to-Speech even when usage stays inside the free tier.
2. Enable Cloud Text-to-Speech API under APIs & Services, then Library.
3. Create a service account under IAM & Admin, then Service Accounts. Google's docs name no IAM role for calling synthesize.
4. Open the account, then Keys, then Add key, then Create new key, then JSON.
5. Store the key outside every repository, run `chmod 600` on it, and delete the key in the console when the audio is done.

An organization created on or after 2024-05-03 enforces `iam.disableServiceAccountKeyCreation` by default, so key creation fails in a campus Workspace project unless an admin lifts it.
A project under a personal Google account has no organization policy.

Pricing checked on 2026-10-12: Chirp 3: HD gives 1 million free characters per month, then USD 30 per million.
A one-minute narration is under 1,000 characters, so comparing every voice stays inside the free tier.

## Synthesize

```bash
export GOOGLE_APPLICATION_CREDENTIALS=~/.config/gcloud-keys/tts-video.json
uv run tools/tts.py videos/<name>
uv run tools/tts.py videos/<name> --voice id-ID-Chirp3-HD-Puck
```

- Each voice writes into its own folder, `audio/<voice>/`, so takes can be compared side by side. `audio/` is gitignored.
- The output is 48 kHz `LINEAR16`, which already carries a WAV header.
- The language code comes from the voice name, and the default voice is `id-ID-Chirp3-HD-Charon`.

Google lists 30 Chirp 3: HD voices for `id-ID`.
Male: Achird, Algenib, Algieba, Alnilam, Charon, Enceladus, Fenrir, Iapetus, Orus, Puck, Rasalgethi, Sadachbia, Sadaltager, Schedar, Umbriel, Zubenelgenubi.
Female: Achernar, Aoede, Autonoe, Callirrhoe, Despina, Erinome, Gacrux, Kore, Laomedeia, Leda, Pulcherrima, Sulafat, Vindemiatrix, Zephyr.

Chirp 3: HD also accepts SSML `<sub alias="...">` and `<phoneme>` for a mispronounced word, and `speaking_rate` from 0.25 to 2.0 for pace; Google labels the SSML support Preview.
Untested improvement: set `speaking_rate` in `AudioConfig` instead of the FFmpeg `atempo` step below, so the model paces the speech itself.

## Audio Preparation

Two FFmpeg passes per file turn raw TTS into the media the video embeds.
Run them, and the measurements below, from `videos/<name>/`.

```bash
ffmpeg -i audio/<voice>/vo_01.wav \
  -af "silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.04,areverse,silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.08,areverse,atempo=1.09" \
  -ac 1 -ar 48000 trimmed/vo_01.wav

ffmpeg -i trimmed/vo_01.wav \
  -af "volume=2.8dB,alimiter=limit=0.84:attack=2:release=40:level=false" \
  -ac 1 -ar 48000 -c:a pcm_s16le media/vo_01.wav
```

- The first pass trims leading and trailing silence, keeping 0.04 s of head and 0.08 s of tail, and speeds the voice up to fit the duration cap.
- The second pass raises the level and limits peaks to about -1.5 dBFS.
- Duration budget: `total = LEAD + sum(lengths) + (n - 1) * GAP + TAIL`, with `LEAD = 0.2`, `GAP = 0.25`, and `TAIL = 0.8` as tested values. For example, 52.3 s of raw narration under a 50 s cap needs `atempo=1.09`.

Measure each processed file, then copy the numbers into `lib.rs`:

```bash
ffprobe -v error -show_entries format=duration -of csv=p=0 media/vo_01.wav
ffmpeg -i media/vo_01.wav -af silencedetect=noise=-40dB:d=0.12 -f null -
```

`silencedetect` gives the pauses inside a line, and those become the start times of the subtitle chunks.

## Timeline from Narration

Each scene is a beat whose length is the measured voice duration.
Start times are derived, so a new voice take only changes the `length` values.

```rust
struct Beat {
    voice: &'static str,
    length: f32,
    subtitles: &'static [(f32, &'static str)],
}

let mut cursor = LEAD;
for (start, beat) in starts.iter_mut().zip(BEATS.iter()) {
    *start = cursor;
    cursor += beat.length + GAP;
}
let total = cursor - GAP + TAIL;
```

- A beat also carries its visual keys, such as a camera, pointer targets, and click times.
- A beat's visual boundary is its voice start minus 0.15 s, so motion leads the voice slightly.
- Times inside a beat (subtitle chunks, clicks, focus changes) are offsets from the voice start, taken from `silencedetect`.
- The audio map places each `vo_NN.wav` at its start with `AudioTrack::new(file, Second(start)..Eof).gain_db(-2.2).voice()`.

## Subtitles

- Subtitles repeat the narration exactly, because many viewers watch muted.
- Line breaks are written by hand as `\n`, balanced, at most about 36 characters per line for 44 px Open Sans Semibold on a 1080 px wide frame.
- A chunk appears at its voice offset with a 0.15 s fade and stays until the next chunk.

## Loudness

- Concatenated stems measured -14.8 LUFS with FFmpeg `ebur128`, but the fframes mix of the same stems measured -11.8 LUFS with `audio analyze`.
- `.gain_db(-2.2)` on every voice track brought the mix to -14.0 LUFS integrated with a true peak of -3.6 dBTP, and the final MP4 measured -14.1 LUFS.
- Judge loudness on the final mix with `audio analyze`, never on the stems.
- No background music is mixed in. Music for TikTok or Reels is picked inside the app, so its license stays with the platform.

## Open Decisions

- Choose TTS or a recorded human voice per video; a recorded voice needs the speaker's consent before it goes on a public account.
- Move the Python and FFmpeg audio steps into one script once a second video confirms the steps are stable.
