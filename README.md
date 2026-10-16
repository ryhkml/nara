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

The script reads `videos/<name>/narration.txt` and writes one WAV file for each `vo_NN` block into `videos/<name>/audio/<voice>/`.
A line with SSML tags, such as `<break>` and `<prosody>`, goes to the API as SSML.
These tags set the pauses, pitch, and pace of each sentence.
The script spells each word in capital letters, such as AI, API, and GPU, with English letter names.

```bash
uv run tools/tts.py videos/<name>									# id-ID-Chirp3-HD-Charon
uv run tools/tts.py videos/<name> --voice en-US-Chirp3-HD-Puck		# any locale, same 30 voices
uv run tools/tts.py --list-voices id-ID								# what this account can use
```

It looks for the service account key at `~/.config/gcloud-keys/tts-video.json`, so nothing has to be exported.
Set `GOOGLE_APPLICATION_CREDENTIALS` to override that path.

Make the key under a Google Cloud project with billing and the Text-to-Speech API enabled, keep it outside every repository, and run `chmod 600` on it.
[`narration.md`](.claude/skills/nara-video/references/narration.md) walks through the setup, lists the voices, and describes the format of `narration.txt` and the FFmpeg steps that turn the takes into media.
It also lists the SSML tags that change the intonation.

## License

[MIT](LICENSE)
