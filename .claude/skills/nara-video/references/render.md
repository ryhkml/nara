# Review, Render, and Encode

Read this when checking a video, rendering it, or delivering the final file.
Run every command from the repository root; `<package>` is the video's crate name, which the template also uses as the binary name.

## Review Without Watching

```bash
cargo build --release -p <package>
./target/release/<package> timeline
./target/release/<package> inspect --fail-on warning
./target/release/<package> strip all -n 20 --columns 5 --width 300
./target/release/<package> frame <t1>,<t2>,<t3>
./target/release/<package> audio analyze
```

The primary source lists every command and how to address time; `strip.png` and `frames/` land in the current directory, and both are gitignored.

## Render the Master

```bash
mkdir -p target/fframes-tmp
TMPDIR=$PWD/target/fframes-tmp ./target/release/<package> render -o videos/<name>/master.mkv
rm -r target/fframes-tmp/fframes-*
```

- Every CPU render writes its encoder segments to `fframes-<uuid>` under `std::env::temp_dir()` and never deletes them ([`fframes/src/renderer/cpu.rs:152`](https://github.com/dmtrKovalenko/fframes/blob/dca70765c82d8d3a0b5916222ae7fc5c98cd42c9/fframes/src/renderer/cpu.rs#L152)). One render left 357-394 MB.
- `/tmp` on this workstation is a 3.8 GB RAM-backed tmpfs. Twelve renders held 2.6 GB of RAM, zram swap reached 100%, and the `/tmp` quota filled up (`EDQUOT`), which also stopped Claude Code from writing tool output in `/tmp`.
- `TMPDIR` moves the segments to disk, and the `rm` removes what fframes leaves behind.
- Untested alternative: set `RenderOptions::tmp_files_directory` in `main.rs` ([`cpu.rs:153`](https://github.com/dmtrKovalenko/fframes/blob/dca70765c82d8d3a0b5916222ae7fc5c98cd42c9/fframes/src/renderer/cpu.rs#L153) reads it).
- The CPU backend rendered 2942 frames at 1080x1920 in 122.8 s.

## Encode the Deliverable

```bash
ffmpeg -i videos/<name>/master.mkv -vf "setpts=N/(60*TB)" \
  -c:v libx264 -preset slow -crf 18 -profile:v high -pix_fmt yuv420p -r 60 -fps_mode cfr \
  -c:a aac -b:a 192k -ar 48000 -movflags +faststart videos/<name>/<name>.mp4
```

- The FFV1 master holds every frame, but Matroska's 1 ms timebase rounds 1/60 s down, so its timestamps end early (46.96 s for 2942 frames). `setpts=N/(60*TB)` with `-r 60 -fps_mode cfr` restores them. Rendering the master to `.nut` or `.mov` is an untested alternative.
- A 49 s 1080x1920 60 fps video came out at 5.7 MB.

## Verify

```bash
ffprobe -v error -show_entries stream=codec_name,width,height,r_frame_rate,nb_frames,duration -of compact videos/<name>/<name>.mp4
ffmpeg -i videos/<name>/<name>.mp4 -af ebur128=peak=true -f null -
```

- The frame count must equal the `timeline` count, and the video and audio durations must match.
- Spot-check sync by extracting a frame with `ffmpeg -ss <t> -i videos/<name>/<name>.mp4 -frames:v 1 x.png` and comparing it with `frame <t>` output through the `psnr` filter. Matching frames scored 42-43 dB.
