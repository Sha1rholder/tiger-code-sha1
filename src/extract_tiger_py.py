"""从上游拼音字典裁出 SC2013 单字。

`PY_c.dict.yaml` 里有字也有词；这里只收一个字、且属于 SC2013 的行。输出沿用 `tiger_py.dict.yaml` 的头部与字段顺序，正文重新排序后写回。
"""

from rime_dict import (
	ROOT,
	format_rows,
	iter_tsv_fields,
	load_sc2013,
	parse_columns,
	require_columns,
	row_from_fields,
	sort_by_code_and_weight,
	split_header_and_tsv,
	write_dict,
)

SOURCE = ROOT / "upstream" / "PY_c.dict.yaml"
TARGET = ROOT / "tiger_py.dict.yaml"


def extract_rows(
	source_header: list[str], source_tsv: list[str], sc2013: set[str]
) -> list[dict[str, str]]:
	"""留下 SC2013 单字；完全相同的上游行只取一次。"""
	source_columns = parse_columns(source_header)
	indexes = require_columns(source_columns, "text")
	text_index = indexes["text"]

	rows: list[dict[str, str]] = []
	seen_rows: set[tuple[str, ...]] = set()
	for fields in iter_tsv_fields(
		source_tsv,
		min_field_count=len(source_columns),
		exact_field_count=len(source_columns),
	):
		text = fields[text_index]
		if len(text) != 1 or text not in sc2013:
			continue

		row_key = tuple(fields)
		if row_key in seen_rows:
			continue
		seen_rows.add(row_key)
		rows.append(row_from_fields(source_columns, fields))

	return rows


def main() -> None:
	"""读上游、筛单字、按目标格式写回拼音字典。"""
	sc2013 = load_sc2013()
	source_header, source_tsv = split_header_and_tsv(SOURCE)
	target_header, _ = split_header_and_tsv(TARGET)
	target_columns = parse_columns(target_header)

	rows = sort_by_code_and_weight(extract_rows(source_header, source_tsv, sc2013))
	target_tsv = format_rows(target_columns, rows)

	write_dict(TARGET, target_header, target_tsv)
	print(f"loaded {len(sc2013)} SC2013 chars")
	print(f"wrote {len(target_tsv)} rows to {TARGET}")


if __name__ == "__main__":
	main()
