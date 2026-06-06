def get_result() -> tuple[list[tuple[str, str]], int]:
	"""返回按编码长度和字母顺序稳定排序后的附加词条(code, text)列表。"""
	# 从`upstream/add.tsv`提取(code, text)列表add
	add: list[tuple[str, str]] = []
	with open("upstream/add.tsv", encoding="utf-8") as f:
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

	# 先按code长度升序排列；后对于相同的code长度，按字母顺序（先a后z）排列；再对于相同的code，按原先后顺序排列
	add.sort(key=lambda item: (len(item[0]), item[0].casefold()))

	# 写回`upstream/add.tsv`
	with open("upstream/add.tsv", "w", encoding="utf-8", newline="") as f:
		f.write("code\ttext\n")
		for code, text in add:
			f.write(f"{code}\t{text}\n")

	# 第一个码长大于等于5的字符的index，需要lua sort
	sort_start = next(
		(index for index, (code, _) in enumerate(add) if len(code) >= 5), len(add)
	)

	return add, sort_start


if __name__ == "__main__":
	rows, sort_start = get_result()
	for code, text in rows:
		print(f"{code}\t{text}")
	print(sort_start)
