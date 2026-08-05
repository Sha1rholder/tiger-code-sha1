"""导出wordfreq词频数据"""

from pathlib import Path

from wordfreq import get_frequency_dict

LANGUAGES = ("zh", "en")
OUTPUT_DIRECTORY = Path(__file__).resolve().parent / "words"


def main() -> None:
	"""导出中英文词频数据"""
	OUTPUT_DIRECTORY.mkdir(parents=True, exist_ok=True)
	for language in LANGUAGES:
		entry_count = write_words(language)
		print(
			f"Wrote {entry_count} {language} entries to "
			f"{OUTPUT_DIRECTORY / f'{language}.tsv'}"
		)

def write_words(language: str) -> int:
	"""按原始顺序写出指定语言的词频TSV并返回词条数"""
	frequencies = get_frequency_dict(language)
	with (OUTPUT_DIRECTORY / f"{language}.tsv").open(
		"w",
		encoding="utf-8",
		newline="",
	) as file:
		file.write("word\tfrequency\n")
		for word, frequency in frequencies.items():
			file.write(f"{word}\t{frequency:.17g}\n")
	return len(frequencies)


if __name__ == "__main__":
	main()
