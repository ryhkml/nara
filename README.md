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
- uv, for narration only

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

```bash
export GOOGLE_APPLICATION_CREDENTIALS=<path to a service account key>
uv run tools/tts.py videos/<name> --voice id-ID-Chirp3-HD-Charon
```

The script reads `videos/<name>/narration.txt`.
It writes one WAV file for each `vo_NN` block into `videos/<name>/audio/<voice>/`.
[`narration.md`](.claude/skills/nara-video/references/narration.md) describes the format of `narration.txt` and the audio preparation steps.

## License

[MIT](LICENSE)
