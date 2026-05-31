"""从上游虎码字典裁出 SC2013 简体单字。

同一个字在上游可能出现多次；这里只保留权重最高的一行。目标文件的 YAML 头不动，只把 TSV 正文换成筛好的结果。
"""

from rime_dict import (
	ROOT,
	format_rows,
	iter_tsv_fields,
	load_sc2013,
	parse_columns,
	require_columns,
	row_from_fields,
	split_header_and_tsv,
	write_dict,
)

SOURCE = ROOT / "upstream" / "tiger.dict.yaml"
TARGET = ROOT / "tiger_sc.dict.yaml"


def extract_rows(
	source_header: list[str], source_tsv: list[str], sc2013: set[str]
) -> list[dict[str, str]]:
	"""只取 SC2013 中的字；一字多行时取最高权重。"""
	source_columns = parse_columns(source_header)
	indexes = require_columns(source_columns, "text", "weight")
	text_index = indexes["text"]
	weight_index = indexes["weight"]

	rows_by_text: dict[str, dict[str, str]] = {}
	weights_by_text: dict[str, int] = {}
	for fields in iter_tsv_fields(
		source_tsv,
		min_field_count=len(source_columns),
		exact_field_count=len(source_columns),
	):
		text = fields[text_index]
		if text not in sc2013:
			continue

		weight = int(fields[weight_index])
		if text not in weights_by_text or weight > weights_by_text[text]:
			weights_by_text[text] = weight
			rows_by_text[text] = row_from_fields(source_columns, fields)

	return list(rows_by_text.values())


def sort_by_weight_and_code(rows: list[dict[str, str]]) -> list[dict[str, str]]:
	"""先权重降序，再短码，最后按字母序。"""
	return sorted(
		rows, key=lambda row: (-int(row["weight"]), len(row["code"]), row["code"])
	)


def main() -> None:
	"""读上游、按字去重取高权重、按权重与编码排序后写回简体虎码字典。"""
	sc2013 = load_sc2013()
	source_header, source_tsv = split_header_and_tsv(SOURCE)
	target_header, _ = split_header_and_tsv(TARGET)
	target_columns = parse_columns(target_header)

	rows = sort_by_weight_and_code(extract_rows(source_header, source_tsv, sc2013))
	target_tsv = format_rows(target_columns, rows)

	write_dict(TARGET, target_header, target_tsv)
	print(f"loaded {len(sc2013)} SC2013 chars")
	print(f"wrote {len(target_tsv)} rows to {TARGET}")


if __name__ == "__main__":
	main()
