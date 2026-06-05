def get_result(sc2013: set[str]) -> list[tuple[str, str]]:
	"""返回按原顺序过滤并去重后的虎码单字(code, text)列表。"""
	# 从`upstream/tiger.dict.yaml`的tsv部分提取(code, text)列表tiger，保留原顺序
	tiger: list[tuple[str, str]] = []
	seen_texts: set[str] = set()

	with open("upstream/tiger.dict.yaml", encoding="utf-8") as f:
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

			# 对于text相同的元组，只保留更靠前的
			if text in seen_texts:
				continue

			seen_texts.add(text)
			tiger.append((code, text))

	return tiger


if __name__ == "__main__":
	sc2013: set[str] = set()
	for filename in ("level-1.txt", "level-2.txt", "level-3.txt"):
		with open(f"upstream/SC2013/{filename}", encoding="utf-8") as f:
			sc2013.update(line.strip() for line in f if line.strip())
	for code, text in get_result(sc2013):
		print(f"{code}\t{text}")
