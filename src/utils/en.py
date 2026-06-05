from wordfreq import get_frequency_dict


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
	# 去掉码长小于4、含非标准英文字母字符、不在esdb中的词并按词频降序排列输出一维列表en_words
	en_words: list[str] = sorted(
		(
			w
			for w in en_freq
			if len(w) >= 4 and w.isascii() and w.isalpha() and w in esdb
		),
		key=lambda w: en_freq[w],
		reverse=True,
	)
	# 对于en_words中每个纯小写的词，生成一个首字母大写的版本加在原词后面
	result: list[str] = []
	for w in en_words:
		result.append(w)
		if w.islower():
			result.append(w.capitalize())

	return result


if __name__ == "__main__":
	for word in get_result():
		print(word)
