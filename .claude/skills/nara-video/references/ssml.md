# SSML on Chirp 3: HD

Read this when a video uses a full Cloud voice name, such as `id-ID-Chirp3-HD-Charon`, instead of the Gemini-TTS default in `narration.md`.
Gemini-TTS reads no SSML; Chirp 3: HD reads it, and keeps 1 million free characters per month and often returns byte-identical audio for identical input.

## Voices

Chirp 3: HD carries the same 30 voice names in every locale it supports, so `--list-voices id-ID` and `--list-voices en-US` list the same 16 male and 14 female Chirp 3: HD voices, and only the prefix changes: `id-ID-Chirp3-HD-Puck` and `en-US-Chirp3-HD-Puck` are the same voice. Verified for `id-ID`, `en-US`, `de-DE`, and `ja-JP` on 2026-10-16.

Other families answer `--voice` too, tested with `en-US-Standard-F` and `en-US-Neural2-J`: `--list-voices en-US` returns 99 voices, of which 30 are Chirp 3: HD and the rest are Standard, WaveNet, Neural2, News, Studio, Casual, Polyglot, and three older Chirp-HD ones. `--list-voices id-ID` returns only Chirp 3: HD, Standard, and WaveNet.

## Intonation

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

## Acronyms

For a Chirp 3: HD voice outside English, the script sends the letter names as `<sub alias="ei pi ai">API</sub>`.
For an `en-*` voice it sends `<say-as interpret-as="characters">API</say-as>`, because an English voice already says English letter names and would read an alias such as `ei pi ai` as English words.
Text inside `<say-as>`, `<sub>`, and `<phoneme>` stays as written.

Five ways to say "Model AI ini memanggil API dan berjalan di GPU." were compared by ear on 2026-10-17: plain text, `<say-as interpret-as="characters">`, a switch to `<voice name="en-US-Chirp3-HD-Charon">`, the `<sub>` alias, and `<phoneme>` with English IPA.
The alias was chosen, and the voice switch stretched the line from about 4 s to 7.44 s.
The test covered the letter names for A, G, I, P, and U; the other 21 follow the same scheme but have not been heard.
