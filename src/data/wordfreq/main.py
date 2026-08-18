"""导出wordfreq中英文词频数据"""

from pathlib import Path

from wordfreq import get_frequency_dict

LANGUAGES = ("zh", "en")
SCRIPT_DIR = Path(__file__).resolve().parent

SCRIPT_DIR.mkdir(parents=True, exist_ok=True)
for language in LANGUAGES:
	# 按原始顺序写出指定语言的词频TSV并返回词条数
	frequencies = get_frequency_dict(language)
	with (SCRIPT_DIR / f"{language}.tsv").open(
		"w",
		encoding="utf-8",
		newline="",
	) as file:
		file.write("text\tfrequency\n")
		for word, frequency in frequencies.items():
			file.write(f"{word}\t{frequency:.17g}\n")
print("Success!")
