# /// script
# requires-python = ">=3.12"
# dependencies = ["google-cloud-texttospeech>=2.38"]
# ///
import argparse
import os
import re
import sys
import time
import wave
from pathlib import Path
from xml.etree import ElementTree
from xml.sax.saxutils import escape

from google.api_core import exceptions as api_errors
from google.cloud import texttospeech

DEFAULT_VOICE = "Charon"
DEFAULT_MODEL = "gemini-3.1-flash-tts-preview"
DEFAULT_PROMPT = "Read this in a calm, warm voice, like a documentary narrator."
KEY_PATH = Path.home() / ".config" / "gcloud-keys" / "tts-video.json"
ATTEMPTS = 3
RETRYABLE = (
    api_errors.DeadlineExceeded,
    api_errors.InternalServerError,
    api_errors.ServiceUnavailable,
    api_errors.TooManyRequests,
)
ACRONYM = re.compile(r"\b[A-Z]{2,}\b")
SPELLED = r'<say-as interpret-as="characters">\g<0></say-as>'
AUTHORED_READING = {"say-as", "sub", "phoneme"}
ENGLISH_LETTERS = dict(
    zip(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "ei bi si di i ef ji eic ai jei kei el em en ou pi kyu ar es ti yu vi dabelyu eks wai zi".split(),
    )
)


def letter_names(match: re.Match[str]) -> str:
    return " ".join(ENGLISH_LETTERS[letter] for letter in match[0])


def english_letters(match: re.Match[str]) -> str:
    return f'<sub alias="{letter_names(match)}">{match[0]}</sub>'


def to_ssml(text: str, language: str) -> str:
    spell = SPELLED if language.startswith("en") else english_letters
    parts = re.split(r"(<[^>]+>)", text if "<" in text else escape(text))
    depth = 0
    for index, part in enumerate(parts):
        tag = re.match(r"<(/?)([\w:-]+)", part)
        if tag:
            if tag[2] in AUTHORED_READING and not part.endswith("/>"):
                depth += -1 if tag[1] else 1
        elif depth == 0:
            parts[index] = ACRONYM.sub(spell, part)
    return "".join(parts)


def to_plain(text: str, language: str) -> str:
    spell = (lambda match: match[0]) if language.startswith("en") else letter_names
    root = ElementTree.fromstring(
        f"<speak>{text if '<' in text else escape(text)}</speak>"
    )
    words = ACRONYM.sub(spell, root.text or "")
    for child in root:
        if child.tag != "sub":
            raise ValueError(
                f"has <{child.tag}>, which Gemini-TTS does not read: keep only <sub>, and steer intonation with voice_prompt.txt or [short pause]"
            )
        words += child.get("alias", "") + ACRONYM.sub(spell, child.tail or "")
    return words


def to_input(
    text: str, language: str, prompt: str | None
) -> texttospeech.SynthesisInput:
    if prompt is not None:
        return texttospeech.SynthesisInput(text=to_plain(text, language), prompt=prompt)
    ssml = to_ssml(text, language)
    return (
        texttospeech.SynthesisInput(ssml=f"<speak>{ssml}</speak>")
        if "<" in ssml
        else texttospeech.SynthesisInput(text=text)
    )


def narration_lines(path: Path) -> list[tuple[str, str]]:
    """Read every `vo_NN` label and the text line below it.

    Exits with a message instead of a traceback when the file is missing or a block has no text.
    """
    if not path.is_file():
        raise SystemExit(f"missing {path}")

    lines = path.read_text(encoding="utf-8").splitlines()
    blocks: list[tuple[str, str]] = []
    for index, line in enumerate(lines):
        if not line.startswith("vo_"):
            continue
        text = lines[index + 1].strip() if index + 1 < len(lines) else ""
        if not text:
            raise SystemExit(
                f"{path}: {line.split()[0]} has no text on the line below it"
            )
        if "<" in text:
            try:
                ElementTree.fromstring(f"<speak>{text}</speak>")
            except ElementTree.ParseError as error:
                raise SystemExit(
                    f"{path}: {line.split()[0]} is not valid SSML: {error}"
                )
        blocks.append((line.split()[0], text))

    if not blocks:
        raise SystemExit(f"{path} has no vo_ blocks")
    return blocks


def require_credentials() -> Path:
    """Return the key file to call the API with, so no session has to know where it is.

    `GOOGLE_APPLICATION_CREDENTIALS` wins; otherwise the path this repository documents is used.
    """
    configured = os.environ.get("GOOGLE_APPLICATION_CREDENTIALS")
    if configured:
        key = Path(configured).expanduser()
        if not key.is_file():
            raise SystemExit(
                f"GOOGLE_APPLICATION_CREDENTIALS points at a missing file: {key}"
            )
        return key

    if not KEY_PATH.is_file():
        raise SystemExit(
            f"no service account key: set GOOGLE_APPLICATION_CREDENTIALS, or place one at {KEY_PATH}"
        )
    os.environ["GOOGLE_APPLICATION_CREDENTIALS"] = str(KEY_PATH)
    return KEY_PATH


def wav_seconds(path: Path) -> float:
    """Length of a written take, so the timing constants in lib.rs come from the file itself."""
    with wave.open(str(path), "rb") as handle:
        return handle.getnframes() / handle.getframerate()


def synthesize_with_retry(
    client: texttospeech.TextToSpeechClient,
    voice: texttospeech.VoiceSelectionParams,
    audio_config: texttospeech.AudioConfig,
    synthesis_input: texttospeech.SynthesisInput,
) -> bytes:
    """Synthesize one line, retrying while the service reports a transient failure.

    A half-finished run otherwise loses every take, and each take is a fresh recording.
    """
    for attempt in range(ATTEMPTS):
        try:
            response = client.synthesize_speech(
                input=synthesis_input,
                voice=voice,
                audio_config=audio_config,
            )
            return response.audio_content
        except RETRYABLE as error:
            if attempt == ATTEMPTS - 1:
                raise
            print(f"{error}: retrying {attempt + 1}/{ATTEMPTS - 1}", file=sys.stderr)
            time.sleep(2.0 * (attempt + 1))
    raise RuntimeError("unreachable")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("video_dir", type=Path, nargs="?")
    parser.add_argument(
        "--voice",
        default=DEFAULT_VOICE,
        help="Gemini-TTS voice such as Puck, or a full Cloud voice name such as id-ID-Chirp3-HD-Puck (default: %(default)s)",
    )
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL,
        help="Gemini-TTS model for a voice without a locale prefix (default: %(default)s)",
    )
    parser.add_argument(
        "--language",
        default="id-ID",
        help="language code for a voice without a locale prefix (default: %(default)s)",
    )
    parser.add_argument(
        "--rate",
        type=float,
        default=1.15,
        help="speaking rate from 0.25 to 2.0 (default: %(default)s)",
    )
    parser.add_argument(
        "--list-voices",
        metavar="LANGUAGE",
        help="print the voices this account can use for a language code, then exit",
    )
    args = parser.parse_args()

    key = require_credentials()
    print(f"key: {key}", file=sys.stderr)
    client = texttospeech.TextToSpeechClient()

    if args.list_voices:
        voices = client.list_voices(language_code=args.list_voices).voices
        for voice in sorted(voices, key=lambda voice: voice.name):
            gender = voice.ssml_gender.name.removeprefix("SSML_VOICE_GENDER_")
            label = "" if gender == "UNSPECIFIED" else f" {gender.lower()}"
            rate = voice.natural_sample_rate_hertz / 1000
            print(f"{voice.name}{label} {rate:.0f} kHz")
        print(f"{len(voices)} voices for {args.list_voices}")
        return

    if args.video_dir is None:
        parser.error("video_dir is required unless --list-voices is used")

    narration = args.video_dir / "narration.txt"
    blocks = narration_lines(narration)
    if "-" in args.voice:
        language = "-".join(args.voice.split("-")[:2])
        voice = texttospeech.VoiceSelectionParams(
            language_code=language, name=args.voice
        )
        prompt = None
        out_dir = args.video_dir / "audio" / args.voice
    else:
        language = args.language
        voice = texttospeech.VoiceSelectionParams(
            language_code=language, name=args.voice, model_name=args.model
        )
        prompt_file = args.video_dir / "voice_prompt.txt"
        prompt = (
            prompt_file.read_text(encoding="utf-8").strip()
            if prompt_file.is_file()
            else DEFAULT_PROMPT
        )
        print(f"prompt: {prompt}", file=sys.stderr)
        out_dir = args.video_dir / "audio" / f"{args.model}-{args.voice}"

    inputs = []
    for name, text in blocks:
        try:
            inputs.append((name, to_input(text, language, prompt)))
        except ValueError as error:
            raise SystemExit(f"{narration}: {name} {error}")
    out_dir.mkdir(parents=True, exist_ok=True)

    audio_config = texttospeech.AudioConfig(
        audio_encoding=texttospeech.AudioEncoding.LINEAR16,
        sample_rate_hertz=48000,
        speaking_rate=args.rate,
    )

    total = 0.0
    for name, synthesis_input in inputs:
        audio = synthesize_with_retry(client, voice, audio_config, synthesis_input)
        out_file = out_dir / f"{name}.wav"
        out_file.write_bytes(audio)
        seconds = wav_seconds(out_file)
        total += seconds
        print(f"{out_file} {seconds:.2f}s")
    print(f"{len(blocks)} blocks, {total:.2f}s of speech")


if __name__ == "__main__":
    main()
