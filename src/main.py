from utils import add, en, py_sc, sc2013, tiger


def replace_tsv2(filename: str, rows: list[tuple[str, str]]) -> None:
	"""用2 rows替换Rime词典文件中`...`之后的tsv部分"""
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
	"""用3 rows替换Rime词典文件中`...`之后的tsv部分"""
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
	# 用sc2013.py生成sc2013
	sc2013_set = sc2013.get_result()

	# 调用py_sc.py，用py替换tiger_py.dict.yaml的tsv部分
	py_rows = py_sc.get_result(sc2013_set)
	replace_tsv2("tiger_py.dict.yaml", py_rows)

	# 用tiger.py获取tiger，用add.py获取add，用en.py获取en并处理
	tiger_rows = tiger.get_result(sc2013_set)
	add_rows = add.get_result()
	en_rows = [(word, word) for word in en.get_result()]

	# 合并tiger+add+en
	tiger_add_en = tiger_rows + add_rows + en_rows

	# 替换tiger.dict.yaml的tsv部分
	replace_tsv2("tiger.dict.yaml", tiger_add_en)
	# from math import ceil, log10
	# start_weight = 10 ** ceil(log10(len(tiger_add_en))) if tiger_add_en else 0
	# tiger_add_en_weight = [
	# 	(code, str(start_weight - index), text)
	# 	for index, (code, text) in enumerate(tiger_add_en)
	# ]
	# replace_tsv3("tiger.dict.yaml", tiger_add_en_weight)


if __name__ == "__main__":
	main()
