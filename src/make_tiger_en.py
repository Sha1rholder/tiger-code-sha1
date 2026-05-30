"""用 https://github.com/rspeer/wordfreq 生成英文词表 tiger_en.dict.yaml"""

import math

from wordfreq import get_frequency_dict

from rime_dict import (
	ROOT,
	format_rows,
	parse_columns,
	split_header_and_tsv,
	write_dict,
)

TARGET = ROOT / "tiger_en.dict.yaml"
MIN_ZIPF = 2.0
WEIGHT_SCALE = 1_000_000


def main() -> None:
	target_header, _ = split_header_and_tsv(TARGET)
	target_columns = parse_columns(target_header)

	freqs = get_frequency_dict("en", "best")
	rows: list[dict[str, str]] = []

	for word, freq in freqs.items():
		if not word.isascii() or not word.isalpha() or len(word) <= 6:
			continue
		word = word.lower()
		zipf = math.log(freq, 10) + 9
		if zipf < MIN_ZIPF:
			continue
		weight = round(zipf * WEIGHT_SCALE)
		rows.append({"code": word, "weight": str(weight), "text": word})

	rows.sort(key=lambda r: (-int(r["weight"]), len(r["code"]), r["code"]))

	target_tsv = format_rows(target_columns, rows)
	write_dict(TARGET, target_header, target_tsv)
	print(f"wrote {len(target_tsv)} English words to {TARGET}")


if __name__ == "__main__":
	main()
