def get_result() -> set[str]:
	"""返回《通用规范汉字表》三个级别合并后的汉字集合。"""
	# 合并`upstream/SC2013/`中的3个字表为set sc2013
	sc2013: set[str] = set()
	for filename in ("level-1.txt", "level-2.txt", "level-3.txt"):
		with open(f"upstream/SC2013/{filename}", encoding="utf-8") as f:
			for line in f:
				text = line.strip()
				if text:
					sc2013.add(text)

	return sc2013


if __name__ == "__main__":
	for text in get_result():
		print(text)
