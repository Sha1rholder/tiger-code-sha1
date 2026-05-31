"""用 https://github.com/rspeer/wordfreq 生成英文词表 tiger_en.dict.yaml"""

import re

from wordfreq import get_frequency_dict

from rime_dict import ROOT, format_rows, parse_columns, split_header_and_tsv, write_dict

TARGET = ROOT / "tiger_en.dict.yaml"
LANGUAGE = "en"
WORDLIST = "best"
MIN_LETTER_COUNT = 5
WEIGHT_SCALE = 1_000_000_000
WORD_RE = re.compile(r"[a-z]+")


def iter_rows() -> list[dict[str, str]]:
	"""把 wordfreq 的英文频率表转换成 Rime 词典行。"""
	rows: list[dict[str, str]] = []

	for word, frequency in get_frequency_dict(LANGUAGE, wordlist=WORDLIST).items():
		if len(word) < MIN_LETTER_COUNT or not WORD_RE.fullmatch(word):
			continue

		rows.append(
			{
				"code": word,
				"weight": str(max(1, round(frequency * WEIGHT_SCALE))),
				"text": word,
			}
		)

	return sorted(rows, key=lambda row: (-int(row["weight"]), row["code"]))


def main() -> None:
	"""按目标词典头部字段顺序，重新生成英文词表正文。"""
	header, _ = split_header_and_tsv(TARGET)
	columns = parse_columns(header)
	rows = iter_rows()

	write_dict(TARGET, header, format_rows(columns, rows))
	print(f"wrote {len(rows)} rows to {TARGET}")


if __name__ == "__main__":
	main()
