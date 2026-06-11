def get_result(sc2013: set[str]) -> list[tuple[str, str]]:
	"""返回按词频降序排列并过滤后的拼音单字(code, text)列表。"""
	# 从`upstream/tiger/PY_c.dict.yaml`的tsv部分提取(text, code, weight)列表py_raw
	py_raw: list[tuple[str, str, int]] = []

	with open("upstream/tiger/PY_c.dict.yaml", encoding="utf-8") as f:
		after_sep = False
		for line_number, line in enumerate(f, 1):
			line = line.rstrip("\n")
			if line.strip() == "...":
				after_sep = True
				continue
			if not after_sep or not line:
				continue

			parts = line.split("\t")
			if len(parts) < 3:
				raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
			text, code, weight_text = parts[:3]

			# 只保留text在sc2013中的元组
			if text not in sc2013:
				continue

			try:
				weight = int(weight_text)
			except ValueError as error:
				raise SystemExit(f"第{line_number}行weight不是整数：{line}") from error

			py_raw.append((code, text, weight))

	# 按weight降序排列py_raw
	py_raw.sort(key=lambda row: row[2], reverse=True)

	# py_raw去掉weight列生成(code, text)列表py_sc并返回
	return [(code, text) for code, text, _weight in py_raw]


def write_result(filename: str, rows: list[tuple[str, str]]) -> None:
	"""替换Rime拼音反查词典tsv正文。"""
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


if __name__ == "__main__":
	sc2013: set[str] = set()
	for filename in ("level-1.txt", "level-2.txt", "level-3.txt"):
		with open(f"upstream/SC2013/{filename}", encoding="utf-8") as f:
			sc2013.update(line.strip() for line in f if line.strip())
	for code, text in get_result(sc2013):
		print(f"{code}\t{text}")
