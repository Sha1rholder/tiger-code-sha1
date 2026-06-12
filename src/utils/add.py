def get_result(add_words: str) -> list[tuple[str, str]]:
	"""返回按编码长度和字母顺序稳定排序后的附加词条(code, text)列表。"""
	# 从add_words提取(code, text)列表add
	add: list[tuple[str, str]] = []
	with open(add_words, encoding="utf-8") as f:
		for line_number, line in enumerate(f, 1):
			line = line.rstrip("\n")
			if not line:
				continue

			parts = line.split("\t")
			if line_number == 1 and parts == ["code", "text"]:
				continue
			if len(parts) != 2:
				raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
			code, text = parts

			add.append((code, text))

	# 检查重复的text并警告
	seen_text: set[str] = set()
	for _, text in add:
		if text in seen_text:
			print(f"警告：text '{text}' 重复")
		else:
			seen_text.add(text)

	# 先按code长度升序排列；后对于相同的code长度，按字母顺序（先a后z）排列；再对于相同的code，按原先后顺序排列
	add.sort(key=lambda item: (len(item[0]), item[0].casefold()))

	# 写回add_words
	with open(add_words, "w", encoding="utf-8", newline="") as f:
		f.write("code\ttext\n")
		for code, text in add:
			f.write(f"{code}\t{text}\n")

	return add


if __name__ == "__main__":
	rows = get_result("tiger_sha1_add.tsv")
	for code, text in rows:
		print(f"{code}\t{text}")
