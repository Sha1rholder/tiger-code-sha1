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


def main() -> None:
	sc2013_set = sc2013.get_result()

	py_rows = py_sc.get_result(sc2013_set)
	replace_tsv2("tiger_sha1_py.dict.yaml", py_rows)

	tiger_rows = tiger.get_result(sc2013_set)
	add_rows = add.get_result()
	tiger_add = tiger_rows + add_rows
	tiger_add.sort(key=lambda item: len(item[0]))

	en_rows = [(word, word) for word in en.get_result()]

	replace_tsv2("tiger_sha1.dict.yaml", tiger_add)
	replace_tsv2("tiger_sha1_en.dict.yaml", en_rows)

	seen: set[tuple[str, str]] = set()
	for code, text in tiger_add + en_rows:
		if (code, text) in seen:
			print(f"Warning: duplicate entry found — code: {code}, text: {text}")
		else:
			seen.add((code, text))


if __name__ == "__main__":
	main()
