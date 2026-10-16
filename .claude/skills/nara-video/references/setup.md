# Setup

Read this when preparing a machine or starting a new video.

## Stack

| Layer                | Tool                                                                                                    | Version used                                                      | Role                                                                                                                |
| -------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Rendering            | [fframes](https://github.com/dmtrKovalenko/fframes)                                                     | 1.2.0, commit `dca7076` (2026-10-11), local clone at `../fframes` | Rust library: `render_frame` returns an SVG tree for every frame, and a statically linked FFmpeg encodes the result |
| Agent skill          | [`fframes-video`](https://github.com/dmtrKovalenko/fframes/tree/main/skills/fframes-video)              | same commit                                                       | Primary source for the fframes API, CLI, design, and audio                                                          |
| Language             | Rust through rustup, stable channel                                                                     | rustc and cargo 1.97.1                                            | Each video compiles to a binary with its own CLI                                                                    |
| Final encode         | FFmpeg on the system                                                                                    | 7.1.5 (`ffmpeg-free`, has `libx264`)                              | H.264 and AAC encode of the lossless master                                                                         |
| Narration (optional) | [Google Cloud Text-to-Speech, Chirp 3: HD](https://docs.cloud.google.com/text-to-speech/docs/chirp3-hd) | `google-cloud-texttospeech` 2.38                                  | Voice-over, see `narration.md`                                                                                      |
| Script runner        | uv                                                                                                      | 0.12.17                                                           | Runs `tools/tts.py` with PEP 723 inline dependencies, no virtualenv to manage                                       |

## System Packages (Fedora 43)

| Package               | Version                                                         | Why                                                                        |
| --------------------- | --------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `clang`, `clang-libs` | 21.1.8                                                          | bindgen in `ffmpeg-sys-fframes` needs `libclang.so`                        |
| `nasm`, `pkgconf`     | 2.16.03, 2.3.0                                                  | FFmpeg build tooling                                                       |
| `x264-devel`          | 0.165 (RPM Fusion)                                              | Only needed when fframes links x264, see Known Issues                      |
| `ffmpeg-free`         | 7.1.5                                                           | Final encode; confirm `ffmpeg -encoders \| grep libx264` lists the encoder |
| Fonts                 | TTF files, such as Open Sans from `/usr/share/fonts/open-sans/` | Copied into each video's `media/`                                          |

The [fframes README](https://github.com/dmtrKovalenko/fframes/blob/main/README.md) lists packages for Debian, Arch, and macOS, but not Fedora.
Nix users can use `shell.nix` in the fframes repository instead.

## Downloads and Disk

- The first build downloads a prebuilt static FFmpeg archive of about 78 MB per codec feature set and caches it in `~/.cache/ffmpeg-sys-fframes/binaries-9.0.0/`.
- A standalone video project's `target/` grew to 2.6 GB. This repository is one Cargo workspace, so all videos share the root `target/`.
- A lossless FFV1 master takes about 350 MB for 49 s of 1080x1920 at 60 fps.

## Workspace

- Every folder in `videos/` that holds a `Cargo.toml` is a workspace member.
- `videos/.gitkeep` keeps the folder in git: when `videos/*` matches nothing, Cargo reads it as a literal path and every command fails.
- The root `Cargo.toml` declares `fframes` under `[workspace.dependencies]` and holds the `[profile.*]` sections, because Cargo ignores profiles in a member.
- The build uses the local clone at `../fframes`, commit `dca7076`, which upstream `main` contains. A git dependency would fetch about 787 MB of history over a slow connection, and the crates.io 1.2.0 release is not verified against that commit. Moving to another commit also moves the commit in the primary source URL in `SKILL.md`.

## New Video

The generator binary is built inside the fframes clone.

```bash
../fframes/target/release/cargo-fframes new <package> \
  --title "<title>" --template single-scene \
  --backend cpu --format portrait --fps 60 --dir videos/<name> \
  --fframes-path ../fframes --yes
```

The first video ran this command outside a workspace; `--dir videos/<name>` inside this workspace is untested.

The primary source explains every flag, template, and backend; these are the choices made here:

- `--backend cpu`, because Skia on Vulkan, the generator's default on Linux, was not tried on this workstation.
- `--template single-scene`, because `multi-scene` rejects the portrait format.
- Inside a Cargo workspace, the generator writes `fframes = { workspace = true, ... }`, which resolves through the root `Cargo.toml`.

### Changes to the Template

| File          | Change                                                                                                           | Reason                                       |
| ------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| `Cargo.toml`  | Remove the `[target.'cfg(not(windows))'.dependencies]` block that adds the `h264` and `libav-agree-gpl` features | x264 ABI mismatch, see Known Issues          |
| `src/main.rs` | `preferred_encoder: Some("ffv1")`, `.default_output("master.mkv")`, remove the `--title` flag                    | Lossless master for the system FFmpeg encode |
| `media/`      | Remove `DMSans-Medium.ttf`, add the video's fonts, images, and audio                                             | Fonts and assets compile into the binary     |

## Known Issues

| Issue             | Symptom                                                                                                                  | Workaround used                                                     | Untested alternative                                                                                                                                                                                              |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| x264 ABI mismatch | Link fails with `undefined symbol: x264_encoder_open_163`; the prebuilt FFmpeg targets x264 ABI 163 and Fedora ships 165 | Drop the `h264` feature, render FFV1, encode with the system FFmpeg | `FFMPEG_FORCE_BUILD=1` compiles FFmpeg from source, but the build script skips the build while `target/release/build/ffmpeg-sys-fframes-*/out/dist/lib/libavutil.a` exists, so that stale `dist/` has to go first |

## Open Decisions

- Try the Skia Vulkan backend for the real-time `preview` window and faster renders.
