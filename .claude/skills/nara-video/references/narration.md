# Narration

Read this when a video has a voice-over.
A video without narration skips this file.

## Pipeline

| Step                                 | Output                                          | Tool                                |
| ------------------------------------ | ----------------------------------------------- | ----------------------------------- |
| 1. Write the script                  | `videos/<name>/narration.txt`                   | Text editor                         |
| 2. Synthesize the voice              | `videos/<name>/audio/<model>-<voice>/vo_NN.wav` | `uv run tools/tts.py videos/<name>` |
| 3. Trim, retime, and level the voice | `videos/<name>/media/vo_NN.wav`                 | FFmpeg                              |
| 4. Measure durations and pauses      | Timing constants in `src/lib.rs`                | `ffprobe`, `silencedetect`          |
| 5. Place the tracks and check levels | Audio map, LUFS report                          | fframes `audio analyze`             |

## Script Rules

- One `vo_NN` block per scene. The line under the label is what TTS reads and, with its tags removed, what the subtitle shows, word for word.
- The language of the video: formal Indonesian with "Anda" for an Indonesian video, plain English for an English one. Complete short sentences, at most about 22 words per block.
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
3. Create a service account under IAM & Admin, then Service Accounts. Chirp 3: HD needs no IAM role, but Gemini-TTS needs `aiplatform.endpoints.predict`, so grant the account Agent Platform User (`roles/aiplatform.user`) under IAM.
4. Open the account, then Keys, then Add key, then Create new key, then JSON.
5. Store the key outside every repository, run `chmod 600` on it, and delete the key in the console when the audio is done.

An organization created on or after 2024-05-03 enforces `iam.disableServiceAccountKeyCreation` by default, so key creation fails in a campus Workspace project unless an admin lifts it.
A project under a personal Google account has no organization policy.

Google renamed Vertex AI to Gemini Enterprise Agent Platform, so the Console lists the role as Agent Platform User; searching for `roles/aiplatform.user` finds it.
The first request after the grant on 2026-10-25 failed with `403 Permission 'aiplatform.endpoints.predict' denied`, and a retry moments later succeeded, so give IAM a minute to propagate.
Gemini-TTS runs on the same service account key and client library as Chirp 3: HD; it needs no Gemini API key.

Pricing checked on 2026-10-12: Chirp 3: HD gives 1 million free characters per month, then USD 30 per million.
A one-minute narration is under 1,000 characters, so comparing every voice stays inside the free tier.
Gemini-TTS has no free tier. A search snapshot of the [pricing page](https://cloud.google.com/text-to-speech/pricing) on 2026-10-25 (the page itself did not load) listed USD 1 per million text tokens in and USD 20 per million audio tokens out for `gemini-3.1-flash-tts-preview` and `gemini-2.5-pro-tts`, and USD 0.50 and USD 10 for `gemini-2.5-flash-tts`.
At 25 audio tokens per second, one minute of `gemini-3.1-flash-tts-preview` narration costs about USD 0.03.

## Synthesize

```bash
uv run tools/tts.py videos/<name>                                  # gemini-3.1-flash-tts-preview, Charon, rate 1.15
uv run tools/tts.py videos/<name> --voice Puck --rate 1.0
uv run tools/tts.py videos/<name> --model gemini-2.5-pro-tts
uv run tools/tts.py videos/<name> --voice id-ID-Chirp3-HD-Charon   # Chirp 3: HD, reads SSML
uv run tools/tts.py --list-voices id-ID   # names, gender and sample rate, straight from the API
```

- A voice without a locale prefix, such as `Charon`, goes to Gemini-TTS with `--model` and `--language`, whose default is `id-ID`. A full Cloud voice name, such as `id-ID-Chirp3-HD-Charon`, goes to that voice, and the language comes from the name.
- Gemini-TTS takes its delivery from `videos/<name>/voice_prompt.txt`, the whole file as one prompt. Without that file the script falls back to an English default prompt that has not been heard. Each run prints the prompt it sent on stderr.
- `--rate` sets `speaking_rate`, from 0.25 to 2.0, on both paths, and defaults to 1.15.
- Takes land in `audio/<model>-<voice>/` for Gemini-TTS and in `audio/<voice>/` for a Cloud voice, so models and voices compare side by side. The rate is not in the folder name, so a run at another rate overwrites the takes of the last one. `audio/` is gitignored.
- The output is 48 kHz `LINEAR16`, which already carries a WAV header, on both paths.
- With `GOOGLE_APPLICATION_CREDENTIALS` unset, the script uses `~/.config/gcloud-keys/tts-video.json` itself.
- Each run prints the key path on stderr and every take's length, so the timing constants in `lib.rs` come from measured files.

The defaults were chosen by ear on 2026-10-25 for the transformer video.
Chirp 3: HD with a `<prosody>` per sentence sounded uneven in intonation, and of `gemini-2.5-flash-lite-preview-tts`, `gemini-2.5-flash-tts`, `gemini-2.5-pro-tts`, and `gemini-3.1-flash-tts-preview`, the last one sounded best.
At rate 1.0 it sounded slow, about 1.4 times the Chirp 3: HD length. On one line, rate 1.15 gave 0.85 times and rate 1.3 gave 0.71 times the length at 1.0, and 1.15 was kept.
`gemini-3.1-flash-tts-preview` is a Preview model, so Google can change or retire it; `gemini-2.5-flash-tts` and `gemini-2.5-pro-tts` carry no Preview label.

Gemini-TTS uses the same 30 voice names as Chirp 3: HD.
Male: Achird, Algenib, Algieba, Alnilam, Charon, Enceladus, Fenrir, Iapetus, Orus, Puck, Rasalgethi, Sadachbia, Sadaltager, Schedar, Umbriel, Zubenelgenubi.
Female: Achernar, Aoede, Autonoe, Callirrhoe, Despina, Erinome, Gacrux, Kore, Laomedeia, Leda, Pulcherrima, Sulafat, Vindemiatrix, Zephyr.

The `--list-voices en-US` listing ends with the 30 locale-free Gemini-TTS names (`Achernar`, `Achird`, …); without the `model_name` that `--model` sends, the service answers `This voice requires a model name to be specified` for them.

## Intonation with Gemini-TTS

Gemini-TTS reads plain text and takes its delivery from the prompt, so write the tone once in `voice_prompt.txt`, in the language of the video:

```text
Bacakan dengan tenang dan hangat, seperti narator video dokumenter.
```

- Mark a pause inside a line with `[short pause]` (about 250 ms), `[medium pause]` (about 500 ms), or `[long pause]` (1 s or more). [Google's Gemini-TTS page](https://docs.cloud.google.com/text-to-speech/docs/gemini-tts) also lists Preview tags such as `[sigh]`, `[laughing]`, and `[whispering]`; only `[short pause]` has been used in a video.
- `<sub alias="...">` still works: the script sends the alias, and the subtitle keeps the written word. Acronyms are spelled as described in Acronyms below.
- Gemini-TTS reads no SSML, so every other tag, such as `<prosody>` or `<break>`, stops the run before the first request and names its block.
- The subtitle drops the bracket tags as it drops `<sub>`.

```text
vo_07
<sub alias="atensi">Attention</sub>, feed-forward, dan residual menyusun satu blok. Blok itu diulang enam kali. [short pause] Positional encoding menjaga urutan kata.
```

A full Cloud voice name, such as `id-ID-Chirp3-HD-Charon`, reads SSML instead of a prompt; read [`ssml.md`](ssml.md) before writing a line for one.

## Acronyms

`tools/tts.py` spells every word of two or more capital letters, such as AI, API, and GPU, with English letter names, also in a line without tags.
Outside English it writes each letter name the way an Indonesian reader says it, from `ei` for A to `zi` for Z, so API becomes `ei pi ai`; an English Gemini-TTS voice gets the acronym as written.
An acronym said as a word, such as JSON or NASA, or a Roman numeral takes a manual alias, because text inside `<sub>` stays as written: `<sub alias="jeison">JSON</sub>`.
The letter names were compared by ear only on Chirp 3: HD, in [`ssml.md`](ssml.md).
`uv run tools/test_tts.py` checks these rules.

## Audio Preparation

Two FFmpeg passes per file turn raw TTS into the media the video embeds.
Run them, and the measurements below, from `videos/<name>/`.

```bash
ffmpeg -i audio/<model>-<voice>/vo_01.wav \
  -af "silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.04,areverse,silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.08,areverse" \
  -ac 1 -ar 48000 trimmed/vo_01.wav

ffmpeg -i trimmed/vo_01.wav -af ebur128 -f null - 2>&1 | grep -A1 "Integrated loudness"

ffmpeg -i trimmed/vo_01.wav \
  -af "volume=<gain>dB,alimiter=limit=0.84:attack=2:release=40:level=false" \
  -ac 1 -ar 48000 -c:a pcm_s16le media/vo_01.wav
```

- The first pass trims leading and trailing silence, keeping 0.04 s of head and 0.08 s of tail. Set the pace with `--rate`; append `atempo=<factor>` to this pass only for a last small fit, as the first video's Chirp 3: HD takes did with `atempo=1.09`.
- The second pass sets the level and limits peaks to about -1.5 dBFS. Gemini-TTS takes differ by up to 3 dB from line to line, so set `<gain>` per file to `-14.5 - I`, where `I` is the integrated loudness of the trimmed take; the transformer takes then measured -14.6 to -15.4 LUFS. A fixed `volume=2.8dB` suited the Chirp 3: HD takes.
- Duration budget: `total = LEAD + sum(lengths) + (n - 1) * GAP + TAIL`, with `LEAD = 0.2`, `GAP = 0.25`, and `TAIL = 0.8` as tested values. For example, 52.3 s of raw narration under a 50 s cap needs `atempo=1.09`.
- Re-voicing a video changes its lengths: the eight transformer lines measured 48.8 s after trimming with Chirp 3: HD and 54.7 s with Gemini-TTS at rate 1.15, so each scene grew by its voice's extra length, the video went from 57.2 s to 63.2 s, and a cue tied to a spoken word moved to that word's new `silencedetect` time.

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
- Background music and the UI sound effects come from `tools/sound.rs`, which writes them from code, so no third-party license applies. The same chords, tempo, and instruments return in every video, and only the length follows the video.
- The Helpdesk video placed the music at `.gain_db(-8.)` with `Ducking { depth_db: -10., ..Default::default() }`, clicks at -12 dB, keystrokes at -15 to -17 dB, the chime at -11 dB, the notification at -12 dB, and whooshes at -14 dB. The mix measured -14.3 LUFS integrated with a true peak of -1.9 dBTP.

## Open Decisions

- Choose TTS or a recorded human voice per video; a recorded voice needs the speaker's consent before it goes on a public account.
- Move the Python and FFmpeg audio steps into one script once a second video confirms the steps are stable.
