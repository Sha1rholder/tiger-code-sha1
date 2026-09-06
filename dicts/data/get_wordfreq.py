# /// script
# requires-python = ">=3.11"
# dependencies = ["wordfreq"]
# ///
"""导出wordfreq中英文词频数据"""

from pathlib import Path

from wordfreq import get_frequency_dict

LANGUAGES = ("zh", "en")
SCRIPT_DIR = Path(__file__).resolve().parent # CWD不敏感

for language in LANGUAGES:
	lang_dir = SCRIPT_DIR / language
	lang_dir.mkdir(parents=True, exist_ok=True)
	# 按原始顺序写出指定语言的词频TSV
	frequencies = get_frequency_dict(language)
	with (lang_dir / "wordfreq.tsv").open(
		"w",
		encoding="utf-8",
		newline="",
	) as file:
		_ = file.write("text\tfrequency\n")
		for word, frequency in frequencies.items():
			_ = file.write(f"{word}\t{frequency:.17g}\n")
print("Success!")
