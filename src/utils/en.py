from wordfreq import get_frequency_dict


def get_result(esdb_filename: str) -> list[str]:
	"""返回按自定义顺序排列的英文单词列表，保留ESDB原始大小写"""
	esdb: list[str] = []
	with open(esdb_filename, encoding="utf-8") as f:
		after_sep = False
		for line in f:
			if line.strip() == "---":
				after_sep = True
				continue
			if after_sep:
				word = line.strip()
				if word:
					esdb.append(word)

	esdb = dedupe_case_variants(esdb)

	en_freq = get_frequency_dict("en")

	# 去掉含非标准英文字母字符、无法匹配wordfreq的词，并按词频降序排列
	en_words: list[str] = sorted(
		(
			word
			for word in esdb
			if word.isascii() and word.isalpha() and word.casefold() in en_freq
		),
		key=lambda word: en_freq[word.casefold()],
		reverse=True,
	)

	return en_words


def dedupe_case_variants(words: list[str]) -> list[str]:
	"""同一单词有多种大小写形式时，逐位优先保留小写形式"""
	groups: dict[str, list[tuple[int, str]]] = {}
	for index, word in enumerate(words):
		groups.setdefault(word.casefold(), []).append((index, word))

	keep_indexes: set[int] = set()
	for entries in groups.values():
		candidates = entries
		max_pos = min(len(word) for _, word in candidates)
		for pos in range(max_pos):
			if len(candidates) == 1:
				break
			if any(word[pos].isupper() for _, word in candidates) and any(
				word[pos].islower() for _, word in candidates
			):
				candidates = [
					(index, word)
					for index, word in candidates
					if not word[pos].isupper()
				]
		keep_indexes.add(candidates[0][0])

	return [word for index, word in enumerate(words) if index in keep_indexes]


def add_case_variants(en_words: list[str]) -> list[str]:
	"""为首字母小写词生成首字母大写版本，为非全小写词生成全大写版本"""
	initial_caps: list[str] = []
	all_caps: list[str] = []
	seen = set(en_words)
	for word in en_words:
		if word[0].islower():
			initial_cap = word[0].upper() + word[1:]
			if initial_cap not in seen:
				initial_caps.append(initial_cap)
				seen.add(initial_cap)
		if not word.islower():
			all_cap = word.upper()
			if all_cap not in seen:
				all_caps.append(all_cap)
				seen.add(all_cap)

	return en_words + initial_caps + all_caps


def write_result(filename: str, words: list[str]) -> None:
	"""将词表写为一行一词的纯文本文件"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


if __name__ == "__main__":
	write_result("lua/en_dict.txt", get_result("upstream/ESDB.txt"))
