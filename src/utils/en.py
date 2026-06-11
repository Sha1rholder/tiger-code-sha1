from wordfreq import get_frequency_dict


def get_base_forms(word: str) -> list[str]:
	"""返回单词可能的基础词形"""
	base_forms: list[str] = []

	def add(form: str) -> None:
		if len(form) >= 2 and form not in base_forms:
			base_forms.append(form)

	# 复数和第三人称形式
	if len(word) >= 4 and word.endswith("ies"):
		add(word[:-3] + "y")
	if len(word) >= 4 and word.endswith("ves"):
		add(word[:-3] + "f")
		add(word[:-3] + "fe")
		add(word[:-1])
	if len(word) >= 4 and word.endswith("es"):
		add(word[:-2])
		add(word[:-1])
	elif len(word) >= 4 and word.endswith("s") and not word.endswith("ss"):
		add(word[:-1])

	# 过去式
	if len(word) >= 4 and word.endswith("ied"):
		add(word[:-3] + "y")
	if len(word) >= 4 and word.endswith("ed"):
		stem = word[:-2]
		add(word[:-1])
		add(stem)
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])

	# 进行时
	if len(word) >= 5 and word.endswith("ing"):
		stem = word[:-3]
		if len(stem) >= 3 or stem in {"be", "do", "go"}:
			add(stem)
		if len(stem) >= 3 or stem in {"dy", "ly", "ty", "us"}:
			add(stem + "e")
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])

	# 副词后缀
	if len(word) >= 4 and word.endswith("ly"):
		stem = word[:-2]
		if len(stem) >= 3:
			add(stem)
			add(stem + "e")
		if len(word) >= 5 and word.endswith("ily"):
			add(word[:-3] + "y")
		if len(word) >= 7 and word.endswith("ically"):
			add(word[:-4])
		if len(word) >= 5 and word.endswith("bly"):
			add(word[:-1] + "e")

	# 比较级和最高级
	if len(word) >= 5 and word.endswith("iest"):
		add(word[:-4] + "y")
	elif len(word) >= 5 and word.endswith("est"):
		stem = word[:-3]
		add(stem)
		add(stem + "e")
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])

	if len(word) >= 4 and word.endswith("ier"):
		add(word[:-3] + "y")

	# 名词化后缀
	if len(word) >= 7 and word.endswith("ments"):
		stem = word[:-5]
		add(stem)
		add(stem + "e")
	elif len(word) >= 6 and word.endswith("ment"):
		stem = word[:-4]
		add(stem)
		add(stem + "e")

	if len(word) >= 8 and word.endswith("inesses"):
		add(word[:-7] + "y")
	elif len(word) >= 7 and word.endswith("nesses"):
		stem = word[:-6]
		add(stem)
		add(stem + "e")
	elif len(word) >= 6 and word.endswith("iness"):
		add(word[:-5] + "y")
	elif len(word) >= 5 and word.endswith("ness"):
		stem = word[:-4]
		add(stem)
		add(stem + "e")

	# 施事/工具名词后缀
	if len(word) >= 5 and word.endswith("ers"):
		stem = word[:-3]
		add(stem)
		add(stem + "e")
		if stem.endswith("i"):
			add(stem[:-1] + "y")
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])
	elif len(word) >= 4 and word.endswith("er"):
		stem = word[:-2]
		add(stem)
		add(stem + "e")
		if stem.endswith("i"):
			add(stem[:-1] + "y")
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])

	# able形容词后缀
	if len(word) >= 6 and word.endswith("able"):
		stem = word[:-4]
		if len(stem) >= 3 or stem in {"be", "do", "go"}:
			add(stem)
		add(stem + "e")
		if stem.endswith("i"):
			add(stem[:-1] + "y")
		if len(stem) > 2 and stem[-1] == stem[-2]:
			add(stem[:-1])

	return base_forms


def get_result() -> list[str]:
	"""返回按自定义顺序排列的英文单词列表，保留ESDB原始大小写"""
	esdb: list[str] = []
	with open("upstream/ESDB.txt", encoding="utf-8") as f:
		after_sep = False
		for line in f:
			if line.strip() == "---":
				after_sep = True
				continue
			if after_sep:
				word = line.strip()
				if word:
					esdb.append(word)

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

	en_word_set = {word.casefold() for word in en_words}

	def is_relative_low_freq_variant(word: str) -> bool:
		word_key = word.casefold()
		return any(
			base in en_word_set and en_freq[word_key] < en_freq[base]
			for base in get_base_forms(word_key)
		)

	normal_words: list[str] = []
	variant_words: list[str] = []
	for word in en_words:
		if is_relative_low_freq_variant(word):
			variant_words.append(word)
		else:
			normal_words.append(word)
	en_words = normal_words + variant_words

	# 去掉码长小于4的词
	en_words = [w for w in en_words if len(w) >= 4]

	initial_caps: list[str] = []
	all_caps: list[str] = []
	seen = set(en_words)
	for word in en_words:
		# 对于每个**首字母小写**的词，生成首字母大写版本
		if word[0].islower():
			initial_cap = word[0].upper() + word[1:]
			if initial_cap not in seen:
				initial_caps.append(initial_cap)
				seen.add(initial_cap)
		# 对于每个**不是全小写**的词，生成全大写版本
		if not word.islower():
			all_cap = word.upper()
			if all_cap not in seen:
				all_caps.append(all_cap)
				seen.add(all_cap)

	en_words.extend(initial_caps)
	en_words.extend(all_caps)

	return en_words


def write_result(
	filename: str = "lua/en_dict.txt", words: list[str] | None = None
) -> None:
	"""将英文词表写为一行一词的纯文本文件"""
	if words is None:
		words = get_result()

	with open(filename, "w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


if __name__ == "__main__":
	write_result()
