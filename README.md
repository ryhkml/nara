# nara

In nara, an AI agent makes videos as code with [fframes](https://github.com/dmtrKovalenko/fframes).
Each video is a Rust crate.
The crate returns an SVG tree for each frame.
The fframes library encodes the frames with FFmpeg.
The `nara-video` agent skill describes the method: setup, narration, scene design, review, render, and delivery.

## Layout

| Path                         | Purpose                                                                                 |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| `videos/<name>/`             | One Cargo crate for each video. All crates are in one workspace.                        |
| `tools/tts.py`               | Makes the narration audio with Google Cloud Text-to-Speech.                             |
| `tools/sound.rs`             | Makes the backsound and the UI sound effects from code.                                 |
| `.claude/skills/nara-video/` | The agent skill that describes the method, with its reference files.                    |
| `Cargo.toml`                 | The workspace root. It declares the shared `fframes` dependency and the build profiles. |

## Requirements

- Rust, stable channel, installed with rustup
- A local clone of fframes at `../fframes`, at commit `dca7076`
- clang, nasm, and pkgconf, for the fframes build
- FFmpeg with `libx264`, to encode the final video
- uv and a Google Cloud Text-to-Speech service account key, for narration only

[`setup.md`](.claude/skills/nara-video/references/setup.md) lists the exact versions and the Fedora package names.

## Usage

Open Claude Code in this repository.
Ask for a video.
The agent then follows the `nara-video` skill for each step.

To do the steps manually, run these commands from the repository root.
`<package>` is the crate name of the video.

```bash
cargo build --release -p <package>
./target/release/<package> inspect --fail-on warning
./target/release/<package> strip all -n 20 --columns 5 --width 300

mkdir -p target/fframes-tmp
TMPDIR=$PWD/target/fframes-tmp ./target/release/<package> render -o videos/<name>/master.mkv
rm -r target/fframes-tmp/fframes-*
```

To create a video, read [`setup.md`](.claude/skills/nara-video/references/setup.md).
To encode the final H.264 file and verify it, read [`render.md`](.claude/skills/nara-video/references/render.md).

### Narration

The script reads `videos/<name>/narration.txt` and writes one WAV file for each `vo_NN` block into `videos/<name>/audio/`.
By default it uses Gemini-TTS, model `gemini-3.1-flash-tts-preview` with the voice Charon at speaking rate 1.15, which takes its tone from a style prompt in `videos/<name>/voice_prompt.txt`.
A full Cloud voice name, such as `id-ID-Chirp3-HD-Charon`, uses Chirp 3: HD instead, where SSML tags such as `<break>` and `<prosody>` set the pauses, pitch, and pace.
The script spells each word in capital letters, such as AI, API, and GPU, with English letter names.

```bash
uv run tools/tts.py videos/<name>									# Gemini-TTS, Charon, rate 1.15
uv run tools/tts.py videos/<name> --voice Puck --rate 1.0			# another voice and pace
uv run tools/tts.py videos/<name> --voice id-ID-Chirp3-HD-Charon	# Chirp 3: HD with SSML
uv run tools/tts.py --list-voices id-ID								# what this account can use
```

It looks for the service account key at `~/.config/gcloud-keys/tts-video.json`, so nothing has to be exported.
Set `GOOGLE_APPLICATION_CREDENTIALS` to override that path.

Make the key under a Google Cloud project with billing and the Text-to-Speech API enabled, keep it outside every repository, and run `chmod 600` on it.
Gemini-TTS also needs the Agent Platform User role (`roles/aiplatform.user`) on the service account.
[`narration.md`](.claude/skills/nara-video/references/narration.md) walks through the setup, lists the voices, and describes the format of `narration.txt` and the FFmpeg steps that turn the takes into media.
It also describes the style prompt that sets the intonation, and [`ssml.md`](.claude/skills/nara-video/references/ssml.md) lists the SSML tags for Chirp 3: HD.

### Backsound and Sound Effects

The tool writes `music.wav`, `click.wav`, `key_1.wav` to `key_4.wav`, `chime.wav`, `ding.wav`, and `whoosh.wav` into the media folder of a video.
The music has the same chords, tempo, and instruments in every video.
Its last chord starts at the outro second, and the track ends at the total length, so each video gets the same backsound at its own length.
The output is the same for the same arguments, so the WAV files do not have to be kept.

```bash
rustc -O --edition 2021 -o target/sound tools/sound.rs
./target/sound videos/<name>/media <outro seconds> <total seconds>
```

[`narration.md`](.claude/skills/nara-video/references/narration.md) lists the gains that the first video used for each track.

## License

[MIT](LICENSE)
