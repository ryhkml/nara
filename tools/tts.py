# /// script
# requires-python = ">=3.12"
# dependencies = ["google-cloud-texttospeech>=2.38"]
# ///
import argparse
from pathlib import Path

from google.cloud import texttospeech


def narration_lines(path: Path) -> list[tuple[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    return [
        (line.split()[0], lines[i + 1].strip())
        for i, line in enumerate(lines)
        if line.startswith("vo_")
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("video_dir", type=Path)
    parser.add_argument("--voice", default="id-ID-Chirp3-HD-Charon")
    args = parser.parse_args()

    items = narration_lines(args.video_dir / "narration.txt")
    assert items and all(
        text for _, text in items
    ), "narration.txt has no vo_ blocks with text"

    out_dir = args.video_dir / "audio" / args.voice
    out_dir.mkdir(parents=True, exist_ok=True)

    client = texttospeech.TextToSpeechClient()
    voice = texttospeech.VoiceSelectionParams(
        language_code="-".join(args.voice.split("-")[:2]), name=args.voice
    )
    audio_config = texttospeech.AudioConfig(
        audio_encoding=texttospeech.AudioEncoding.LINEAR16,
        sample_rate_hertz=48000,
    )

    for name, text in items:
        response = client.synthesize_speech(
            input=texttospeech.SynthesisInput(text=text),
            voice=voice,
            audio_config=audio_config,
        )
        (out_dir / f"{name}.wav").write_bytes(response.audio_content)
        print(out_dir / f"{name}.wav")


if __name__ == "__main__":
    main()
