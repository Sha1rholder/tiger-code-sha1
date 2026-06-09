from wordfreq import get_frequency_dict


def get_base_forms(word: str) -> list[str]:
	"""返回英文单词可能的基础词形。"""
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

	return base_forms


def get_result() -> list[str]:
	"""返回按词频降序排列的英文单词列表（含首字母大写版本）。"""
	# 把`upstream/ESDB.txt`制成set esdb
	esdb: set[str] = set()
	with open("upstream/ESDB.txt", encoding="utf-8") as f:
		after_sep = False
		for line in f:
			if line.strip() == "---":
				after_sep = True
				continue
			if after_sep:
				esdb.add(line.strip())

	en_freq = get_frequency_dict("en")

	# 去掉含非标准英文字母字符、不在esdb中的词并按词频降序排列输出一维列表en_words
	en_words: list[str] = sorted(
		(w for w in en_freq if w.isascii() and w.isalpha() and w in esdb),
		key=lambda w: en_freq[w],
		reverse=True,
	)

	en_word_set = set(en_words)

	def is_relative_low_freq_variant(word: str) -> bool:
		return any(
			base in en_word_set and en_freq[word] < en_freq[base]
			for base in get_base_forms(word)
		)

	en_words = [w for w in en_words if not is_relative_low_freq_variant(w)]

	# 去掉码长小于4的词
	en_words = [w for w in en_words if len(w) >= 4]

	# 对于每个纯小写的词，生成一个首字母大写的版本加在原词后面
	result: list[str] = []
	for w in en_words:
		result.append(w)
		if w.islower():
			result.append(w.capitalize())

	return result


if __name__ == "__main__":
	for word in get_result():
		print(word)
