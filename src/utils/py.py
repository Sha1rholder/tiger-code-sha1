def get_result(sc2013: set[str]) -> list[tuple[str, str]]:
	"""返回按原顺序过滤后的拼音单字(code, text)列表。"""
	# 从`upstream/PY_c.dict.yaml`的tsv部分提取(code, text)列表py
	py: list[tuple[str, str]] = []

	with open("upstream/PY_c.dict.yaml", encoding="utf-8") as f:
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

			# 去掉所有text不在sc2013中的行（text必须完全严格匹配，因此只会保留单字）
			if text not in sc2013:
				continue

			py.append((code, text))

	return py


if __name__ == "__main__":
	sc2013: set[str] = set()
	for filename in ("level-1.txt", "level-2.txt", "level-3.txt"):
		with open(f"upstream/SC2013/{filename}", encoding="utf-8") as f:
			sc2013.update(line.strip() for line in f if line.strip())
	for code, text in get_result(sc2013):
		print(f"{code}\t{text}")
