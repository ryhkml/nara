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

- One `vo_NN` block per scene. The line under the label is what TTS reads and, with its SSML tags removed, what the subtitle shows, word for word.
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
uv run tools/tts.py videos/<name> --voice en-US-Chirp3-HD-Puck   # any locale, same 30 names
uv run tools/tts.py --list-voices id-ID   # names, gender and sample rate, straight from the API
```

- Each voice writes into its own folder, `audio/<voice>/`, so takes can be compared side by side. `audio/` is gitignored.
- The output is 48 kHz `LINEAR16`, which already carries a WAV header.
- The language code comes from the voice name, and the default voice is `id-ID-Chirp3-HD-Charon`.
- With `GOOGLE_APPLICATION_CREDENTIALS` unset, the script uses `~/.config/gcloud-keys/tts-video.json` itself, so the export above is only needed for tools that require it.
- Each run prints the key path on stderr and every take's length, so the timing constants in `lib.rs` come from measured files.

Chirp 3: HD carries the same 30 voice names in every locale it supports, so `--list-voices id-ID` and `--list-voices en-US` list the same 16 male and 14 female Chirp 3: HD voices, and only the prefix changes: `id-ID-Chirp3-HD-Puck` and `en-US-Chirp3-HD-Puck` are the same voice. Verified for `id-ID`, `en-US`, `de-DE`, and `ja-JP` on 2026-10-16.
Male: Achird, Algenib, Algieba, Alnilam, Charon, Enceladus, Fenrir, Iapetus, Orus, Puck, Rasalgethi, Sadachbia, Sadaltager, Schedar, Umbriel, Zubenelgenubi.
Female: Achernar, Aoede, Autonoe, Callirrhoe, Despina, Erinome, Gacrux, Kore, Laomedeia, Leda, Pulcherrima, Sulafat, Vindemiatrix, Zephyr.

Other families answer `--voice` too, tested with `en-US-Standard-F` and `en-US-Neural2-J`: `--list-voices en-US` returns 99 voices, of which 30 are Chirp 3: HD and the rest are Standard, WaveNet, Neural2, News, Studio, Casual, Polyglot, and three older Chirp-HD ones. `--list-voices id-ID` returns only Chirp 3: HD, Standard, and WaveNet.
The same listing ends with 30 locale-free names for `en-US` (`Achernar`, `Achird`, …). The service answers `This voice requires a model name to be specified` for them, and no model name is documented, so use the `<locale>-Chirp3-HD-<name>` form instead.

Chirp 3: HD also accepts `speaking_rate` from 0.25 to 2.0 for pace.
Untested improvement: set `speaking_rate` in `AudioConfig` instead of the FFmpeg `atempo` step below, so the model paces the speech itself.

## Intonation with SSML

A narration line that contains a tag goes to the API as SSML: `tools/tts.py` wraps it in `<speak>` and checks that it parses as XML before the first request, so a broken line stops the run before any take is paid for.
A line without a tag goes as plain text, unless it holds an acronym, which turns it into SSML too (see Acronyms below).
Inside an SSML line, write `&`, `<`, and `>` in the spoken words as `&amp;`, `&lt;`, and `&gt;`, and leave out `<speak>`.

[Google's SSML reference](https://docs.cloud.google.com/text-to-speech/docs/ssml) covers only part of the [W3C SSML 1.1 specification](https://www.w3.org/TR/speech-synthesis11/), and its [Chirp 3: HD page](https://docs.cloud.google.com/text-to-speech/docs/chirp3-hd) narrows it further to `<speak>`, `<say-as>`, `<p>`, `<s>`, `<phoneme>`, `<sub>`, `<break>`, `<audio>`, `<prosody>`, and `<voice>`, and says it ignores every other tag.
Google labels this support Preview, offers it only on synchronous requests (the kind `tools/tts.py` sends), and counts the tags toward the character quota.
The `[pause]`, `[pause short]`, and `[pause long]` tags from the Chirp 3: HD page work only in the `markup` input, which one request cannot combine with SSML, so use `<break>` instead.

Measured on 2026-10-17 with `id-ID-Chirp3-HD-Charon`, two or three takes per variant, against the same words without the tag; pitch is the median F0 of an autocorrelation estimate:

| Markup                                             | Measured effect                                       | Verdict                                        |
| -------------------------------------------------- | ----------------------------------------------------- | ---------------------------------------------- |
| `<prosody pitch="+4st">`, `pitch="-4st"`           | median +1.8 to +3.1 st, and -2.4 to -3.9 st           | Works, a bit weaker than asked                 |
| `<prosody pitch="x-high">`                         | about +10 to +12 st                                   | Works, too strong for narration                |
| `<prosody rate="80%">`, `rate="120%"`              | 1.34 to 1.36 times, and 0.89 to 0.92 times the length | Works                                          |
| `<prosody rate="x-slow">`                          | 2.0 times the length                                  | Works, too slow for narration                  |
| `<prosody volume="-6dB">`                          | -5.2 to -6.4 dB                                       | Works                                          |
| `<break time="800ms"/>`                            | 1.1 to 1.3 s of extra length per break                | Works, longer than asked                       |
| `<break strength="none"/>`, `"x-strong"`           | both added 1.1 to 1.5 s                               | `strength` is ignored, use `time`              |
| `<p>`, `<s>`                                       | audio byte-identical to the plain text                | No effect                                      |
| `<prosody>` with `range`, `contour`, or `duration` | audio byte-identical to the line without the tag      | Ignored                                        |
| `<emphasis level="strong">`                        | +4.7 to +5.2 dB, 1.14 to 1.19 times the length        | Not on Google's list, so the effect can vanish |

Identical input often returns byte-identical audio, so a re-take of an unchanged line can return the same take.
The median pitch of one line still drifts by about 2 st between takes, so a single sentence at `+2st` may not stand out.
Contrast between sentences of the same take holds better: with `+3st` on a question and `-1st` on the sentence after it, the question measured 1.27 to 1.46 times the F0 of the next sentence in three takes, where 4 st predicts 1.26.

Rules for a clear, varied line:

- Put each `<prosody>` around one full sentence, as Google advises, and give neighboring sentences different settings: lift a hook or a question with `pitch="+3st"`, keep the body at the default, and drop a closing sentence with `pitch="-1st"` or `pitch="-2st"`.
- Slow the sentence that carries the key fact with `rate="90%"`, and keep every rate between `85%` and `110%`.
- Separate two ideas with `<break time="300ms"/>` to `<break time="600ms"/>`, and add the extra length to the duration budget, because a break runs longer than its `time`.
- Write an acronym as plain capitals and let the script spell it, replace a term with `<sub alias="...">`, and fix one word with `<phoneme alphabet="ipa" ph="...">`.
- The subtitle keeps the written words: drop every tag, and show the text inside `<sub>`, not its alias.

```text
vo_01
<prosody pitch="+3st">Pernahkah Anda bertanya bagaimana mesin memahami bahasa?</prosody> <break time="400ms"/> <prosody rate="90%">Jawabannya ada pada model AI bernama transformer.</prosody>
```

The subtitle of that block reads: Pernahkah Anda bertanya bagaimana mesin memahami bahasa? Jawabannya ada pada model AI bernama transformer.

### Acronyms

`tools/tts.py` spells every word of two or more capital letters, such as AI, API, and GPU, with English letter names, also in a line without tags.
For a voice outside English it sends `<sub alias="ei pi ai">API</sub>`, where the alias writes each English letter name the way an Indonesian reader says it, from `ei` for A to `zi` for Z.
For an `en-*` voice it sends `<say-as interpret-as="characters">API</say-as>`, because an English voice already says English letter names and would read an alias such as `ei pi ai` as English words.
Text inside `<say-as>`, `<sub>`, and `<phoneme>` stays as written, so an acronym said as a word, such as JSON or NASA, or a Roman numeral takes a manual alias: `<sub alias="jeison">JSON</sub>`.
`uv run tools/test_tts.py` checks these rules.

Five ways to say "Model AI ini memanggil API dan berjalan di GPU." were compared by ear on 2026-10-17: plain text, `<say-as interpret-as="characters">`, a switch to `<voice name="en-US-Chirp3-HD-Charon">`, the `<sub>` alias, and `<phoneme>` with English IPA.
The alias was chosen, and the voice switch stretched the line from about 4 s to 7.44 s.
The test covered the letter names for A, G, I, P, and U; the other 21 follow the same scheme but have not been heard.

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
