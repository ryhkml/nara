# /// script
# requires-python = ">=3.12"
# dependencies = ["google-cloud-texttospeech>=2.38"]
# ///
from tts import to_plain, to_ssml


def alias(word: str, spoken: str) -> str:
    return f'<sub alias="{spoken}">{word}</sub>'


def spelled(word: str) -> str:
    return f'<say-as interpret-as="characters">{word}</say-as>'


assert to_ssml("Tanpa singkatan.", "id-ID") == "Tanpa singkatan."
assert to_ssml("Model AI memanggil API di GPU-nya.", "id-ID") == (
    f"Model {alias('AI', 'ei ai')} memanggil {alias('API', 'ei pi ai')}"
    f" di {alias('GPU', 'ji pi yu')}-nya."
)
assert to_ssml("Riset & AI", "id-ID") == f"Riset &amp; {alias('AI', 'ei ai')}"
assert to_ssml(spelled("CPU"), "id-ID") == spelled("CPU")
assert to_ssml('<sub alias="jeison">JSON</sub> lewat API', "id-ID") == (
    f'<sub alias="jeison">JSON</sub> lewat {alias("API", "ei pi ai")}'
)
assert to_ssml(
    '<voice name="en-US-Chirp3-HD-Charon">attention</voice> <break time="300ms"/> LLM.',
    "id-ID",
) == (
    '<voice name="en-US-Chirp3-HD-Charon">attention</voice> <break time="300ms"/>'
    f" {alias('LLM', 'el el em')}."
)
assert to_ssml("The API runs on a GPU.", "en-US") == (
    f"The {spelled('API')} runs on a {spelled('GPU')}."
)

assert to_plain("Model AI memanggil API di GPU-nya.", "id-ID") == (
    "Model ei ai memanggil ei pi ai di ji pi yu-nya."
)
assert to_plain("Riset & AI. [short pause] Selesai.", "id-ID") == (
    "Riset & ei ai. [short pause] Selesai."
)
assert (
    to_plain(
        '<sub alias="kueri">Query</sub> mencari, <sub alias="ki">key</sub> lewat API.',
        "id-ID",
    )
    == "kueri mencari, ki lewat ei pi ai."
)
assert to_plain("The API runs on a GPU.", "en-US") == "The API runs on a GPU."
try:
    to_plain('<prosody pitch="+3st">Halo.</prosody>', "id-ID")
except ValueError as error:
    assert "<prosody>" in str(error)
else:
    raise AssertionError("to_plain accepted <prosody>")
print("ok")
