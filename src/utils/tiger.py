LOCAL_WEIGHT_STRIDE = 100
LENGTH_WEIGHT_BASE = {
	1: 300,
	2: 200,
	3: 100,
	4: 0,
}
DICT_HEADER = """---
name: tiger_sha1
version: 2026.06.08
sort: by_weight
columns:
  - code
  - weight
  - text
...
"""


def get_result(sc2013: set[str], tiger_dict: str) -> list[tuple[str, str]]:
	"""返回过滤并单一化编码后的虎码单字(code, text)列表"""
	# 从tiger_dict的tsv部分提取(code, text)列表tiger，保留原顺序
	selected_by_text: dict[str, tuple[int, str, str]] = {}

	with open(tiger_dict, encoding="utf-8") as f:
		after_sep = False
		for line_number, line in enumerate(f, 1):
			line = line.rstrip("\n")
			if line.strip() == "...":
				after_sep = True
				continue
			if not after_sep or not line:
				continue

			parts = line.split("\t")
			if len(parts) < 2:
				raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
			text, code = parts[:2]
			# 去掉text不在sc2013中的元组
			if text not in sc2013:
				continue

			# 对于text相同的元组，保留码长更短的；码长相同时保留更靠前的
			current = selected_by_text.get(text)
			if current is None or len(code) < len(current[1]):
				selected_by_text[text] = (line_number, code, text)

	return [
		(code, text)
		for _, code, text in sorted(selected_by_text.values(), key=lambda item: item[0])
	]


def add_prefix_local_weights(rows: list[tuple[str, str]]) -> list[tuple[str, str, int]]:
	"""为每行分配权重

	权重 = 码长基础权重 + 同前缀内局部权重（同前缀越靠前权重越高）
	码长 > 4 的按 4 处理；码长 = 1 的直接赋予基础权重
	"""
	prefix_counts_by_len: dict[int, dict[str, int]] = {}
	weighted_rows: list[tuple[str, str, int]] = []

	for code, text in rows:
		code_len = len(code)
		if code_len > 4:
			code_len = 4
		elif code_len == 1:
			weighted_rows.append((code, text, LENGTH_WEIGHT_BASE[code_len]))
			continue

		prefix = code[:-1]
		prefix_counts = prefix_counts_by_len.setdefault(code_len, {})
		prefix_count = prefix_counts.get(prefix, 0)
		if prefix_count >= LOCAL_WEIGHT_STRIDE:
			raise SystemExit(
				f"prefix-local weight overflow: code length group {code_len}, "
				f"prefix {prefix!r} has more than {LOCAL_WEIGHT_STRIDE} entries"
			)

		prefix_counts[prefix] = prefix_count + 1
		local_weight = LOCAL_WEIGHT_STRIDE - prefix_count - 1
		weight = LENGTH_WEIGHT_BASE[code_len] + local_weight
		weighted_rows.append((code, text, weight))

	return weighted_rows


def write_result(
	filename: str,
	rows: list[tuple[str, str]],
) -> list[tuple[str, str, int]]:
	"""写入完整Rime虎码主词典，并返回写入的加权行。"""
	sorted_rows = sorted(rows, key=lambda item: len(item[0]))
	weighted_rows = add_prefix_local_weights(sorted_rows)

	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.write(DICT_HEADER)
		for code, text, weight in weighted_rows:
			f.write(f"{code}\t{weight}\t{text}\n")

	return weighted_rows


if __name__ == "__main__":
	sc2013: set[str] = set()
	for filename in ("level-1.txt", "level-2.txt", "level-3.txt"):
		with open(f"upstream/SC2013/{filename}", encoding="utf-8") as f:
			sc2013.update(line.strip() for line in f if line.strip())
	for code, text in get_result(sc2013, "upstream/tiger/tiger.dict.yaml"):
		print(f"{code}\t{text}")
