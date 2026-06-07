from utils import add, en, py_sc, sc2013, tiger


def replace_tsv2(filename: str, rows: list[tuple[str, str]]) -> None:
	"""替换Rime词典tsv（2 rows）"""
	with open(filename, encoding="utf-8") as f:
		lines = f.readlines()

	for index, line in enumerate(lines):
		if line.strip() == "...":
			header = lines[: index + 1]
			break
	else:
		raise SystemExit(f"{filename}中找不到词典正文分隔符：...")

	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.writelines(header)
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def replace_tsv3(filename: str, rows: list[tuple[str, str, str]]) -> None:
	"""替换Rime词典tsv（3 rows）"""
	with open(filename, encoding="utf-8") as f:
		lines = f.readlines()

	for index, line in enumerate(lines):
		if line.strip() == "...":
			header = lines[: index + 1]
			break
	else:
		raise SystemExit(f"{filename}中找不到词典正文分隔符：...")

	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.writelines(header)
		for code, weight, text in rows:
			f.write(f"{code}\t{weight}\t{text}\n")


def main() -> None:
	MAX_WEIGHT_1 = 900000
	MAX_WEIGHT_2 = 800000
	MAX_WEIGHT_3 = 700000
	MAX_WEIGHT_4 = 600000
	MAX_WEIGHT_more = 500000
	MAX_WEIGHT_EN = 400000

	sc2013_set = sc2013.get_result()

	py_rows = py_sc.get_result(sc2013_set)
	replace_tsv2("tiger_sha1_py.dict.yaml", py_rows)

	tiger_rows = tiger.get_result(sc2013_set)
	add_rows = add.get_result()
	tiger_add = tiger_rows + add_rows
	tiger_add.sort(key=lambda item: len(item[0]))

	max_weight_by_code_len = {
		1: MAX_WEIGHT_1,
		2: MAX_WEIGHT_2,
		3: MAX_WEIGHT_3,
		4: MAX_WEIGHT_4,
	}
	code_len_counts: dict[int, int] = {}
	tiger_add_weight = []
	for code, text in tiger_add:
		code_len = len(code) if len(code) < 5 else 5
		index = code_len_counts.get(code_len, 0)
		max_weight = max_weight_by_code_len.get(code_len, MAX_WEIGHT_more)
		tiger_add_weight.append((code, str(max(0, max_weight - index)), text))
		code_len_counts[code_len] = index + 1

	en_rows = [(word, word) for word in en.get_result()]
	en_rows_weight = [
		(code, str(max(0, MAX_WEIGHT_EN - index)), text)
		for index, (code, text) in enumerate(en_rows)
	]

	replace_tsv3("tiger_sha1.dict.yaml", tiger_add_weight)
	replace_tsv3("tiger_sha1_en.dict.yaml", en_rows_weight)

	seen: set[tuple[str, str]] = set()
	for code, _, text in tiger_add_weight + en_rows_weight:
		if (code, text) in seen:
			print(f"Warning: duplicate entry found — code: {code}, text: {text}")
		else:
			seen.add((code, text))


if __name__ == "__main__":
	main()
