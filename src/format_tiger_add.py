"""整理手写补充词典 `tiger_add.dict.yaml`。

它不从上游生成，只把已有条目排齐：先码长、再码、再按权重降序。同一个 `code + weight` 不允许出现两次，避免候选顺序变得暧昧。
"""

import sys

from rime_dict import ROOT, split_header_and_tsv, write_dict

TARGET = ROOT / "tiger_add.dict.yaml"


def sort_key(row: tuple[str, str, str]) -> tuple[int, str, int]:
	"""补充词按短码优先；同码时高权重在前。"""
	code, weight, _text = row
	return (len(code), code, -int(weight))


def extract_rows(tsv: list[str]) -> list[tuple[str, str, str]]:
	"""读出三列表格；坏行只警告，不中断整理。"""
	rows: list[tuple[str, str, str]] = []
	for line in tsv:
		stripped = line.strip()
		if not stripped:
			continue

		fields = stripped.split("\t")
		if len(fields) != 3:
			print(f"Warning: skipping malformed line: {stripped!r}", file=sys.stderr)
			continue

		rows.append((fields[0], fields[1], fields[2]))
	return rows


def check_duplicate_code_weight(rows: list[tuple[str, str, str]]) -> None:
	"""同码同权重会让候选相互争位，因此直接报错。"""
	groups: dict[tuple[str, str], list[str]] = {}
	for code, weight, text in rows:
		groups.setdefault((code, weight), []).append(text)

	duplicates = {k: v for k, v in groups.items() if len(v) > 1}
	if not duplicates:
		return

	print("错误：以下编码+权重组合重复，会导致候选顺序不确定：", file=sys.stderr)
	for (code, weight), texts in duplicates.items():
		print(
			f"  编码={code!r}  权重={weight!r}  文本: {', '.join(texts)}",
			file=sys.stderr,
		)
	sys.exit(1)


def reassign_weights(rows: list[tuple[str, str, str]]) -> list[tuple[str, str, str]]:
	"""同 code 的条目，权重改为 9, 8, 7, … 等差数列（公差 1，最大值 9）。

	调用前须已按 sort_key 排序，确保同码内高权重在前。
	"""
	result: list[tuple[str, str, str]] = []
	i = 0
	while i < len(rows):
		code = rows[i][0]
		j = i
		while j < len(rows) and rows[j][0] == code:
			j += 1
		for k in range(i, j):
			result.append((rows[k][0], str(9 - (k - i)), rows[k][2]))
		i = j
	return result


def format_rows(rows: list[tuple[str, str, str]]) -> list[str]:
	"""写回 `code weight text` 三列。"""
	return [f"{code}\t{weight}\t{text}" for code, weight, text in rows]


def main() -> None:
	"""检查、排序并原地写回补充词典。"""
	header, tsv = split_header_and_tsv(TARGET)
	rows = extract_rows(tsv)
	check_duplicate_code_weight(rows)
	rows.sort(key=sort_key)
	rows = reassign_weights(rows)

	write_dict(TARGET, header, format_rows(rows))
	print(f"Formatted {len(rows)} entries in {TARGET}")


if __name__ == "__main__":
	main()
