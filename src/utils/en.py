from collections.abc import Callable, Iterator

from wordfreq import get_frequency_dict

from utils.types import Text

MIN_WORD_LEN = 4
CONSONANTS: set[str] = set("bcdfghjklmnpqrstvwxyz")


def sort_add_words(words: list[Text]) -> list[Text]:
	"""返回按单词长度和字母顺序稳定排序后的英文附加词"""
	clean_words: list[Text] = [word for word in words if word]

	seen: set[Text] = set()
	for word in clean_words:
		if word in seen:
			print(f"警告：英文附加词'{word}'重复")
		else:
			seen.add(word)

	return sorted(clean_words, key=lambda word: (len(word), word.lower()))


def get_base_ranked_entries(
	esdb_words: set[Text],
) -> list[tuple[Text, float, float, int]]:
	"""返回未过滤码长的英文词条排序指标"""
	esdb = expand_esdb_case_variants(esdb_words)
	en_freq: dict[str, float] = get_frequency_dict("en")

	infos: list[tuple[Text, Text, float]] = [
		(Text(word), Text(word), en_freq[word.lower()])
		for word in sorted(esdb, key=lambda value: (value.lower(), value))
		if (
			word.isascii()
			and word.isalpha()
			and len(word) >= 3
			and word.lower() in en_freq
		)
	]

	return rank_base_entries(infos)


def expand_esdb_case_variants(words: set[Text]) -> set[Text]:
	"""按ESDB词面扩增大小写变体"""
	expanded = set(words)
	for word in words:
		if not word:
			continue
		if word.islower():
			expanded.add(Text(word.upper()))
			expanded.add(Text(word[0].upper() + word[1:]))
		else:
			expanded.add(Text(word.upper()))
			expanded.add(Text(word.lower()))

	return expanded


def rank_base_entries(
	infos: list[tuple[Text, Text, float]],
) -> list[tuple[Text, float, float, int]]:
	"""按降权次数、提权词频、原词频和词面排序"""
	infos_by_key = {info[1]: info for info in infos}
	parent_by_key = build_parent_map(infos_by_key)

	boosted_frequency = {info[1]: info[2] for info in infos}
	demotion_count = {info[1]: 0 for info in infos}

	for info in infos:
		ancestor_key = parent_by_key.get(info[1])
		while ancestor_key is not None:
			ancestor = infos_by_key[ancestor_key]
			if ancestor[2] > info[2]:
				boosted_frequency[ancestor_key] += info[2]
				demotion_count[info[1]] += 1
			ancestor_key = parent_by_key.get(ancestor_key)

	entries = [
		(Text(info[0]), info[2], boosted_frequency[info[1]], demotion_count[info[1]])
		for info in infos
	]

	entries.sort(
		key=lambda entry: (
			entry[3],
			-entry[2],
			-entry[1],
			entry[0].lower(),
			entry[0],
		),
	)
	return entries


def build_parent_map(
	infos_by_key: dict[Text, tuple[Text, Text, float]],
) -> dict[Text, Text]:
	"""为每个词选择唯一直接基本形式"""
	parent_by_key: dict[Text, Text] = {}
	for key in infos_by_key:
		candidates = [
			(priority, Text(base_key))
			for priority, base_key in iter_base_candidates(key)
			if base_key != key and base_key in infos_by_key
		]
		if not candidates:
			continue

		_priority, parent_key = min(
			candidates,
			key=lambda candidate: (
				candidate[0],
				-infos_by_key[candidate[1]][2],
				candidate[1],
			),
		)
		parent_by_key[key] = parent_key

	return parent_by_key


def iter_base_candidates(word: Text) -> Iterator[tuple[int, Text]]:
	"""按规则顺序产出直接基本形式候选"""
	rules = [
		lambda value: strip_suffix(value, Text("s")),
		lambda value: strip_suffix(value, Text("S")),
		lambda value: strip_suffix(value, Text("es")),
		lambda value: strip_suffix(value, Text("ES")),
		lambda value: replace_suffix(value, Text("ies"), Text("y")),
		lambda value: replace_suffix(value, Text("IES"), Text("Y")),
		lambda value: strip_suffix(value, Text("d")),
		lambda value: strip_suffix(value, Text("D")),
		lambda value: strip_suffix(value, Text("ed")),
		lambda value: strip_suffix(value, Text("ED")),
		lambda value: replace_suffix(value, Text("ied"), Text("y")),
		lambda value: replace_suffix(value, Text("IED"), Text("Y")),
		lambda value: strip_doubled_consonant_suffix(value, Text("ed")),
		lambda value: strip_doubled_consonant_suffix(value, Text("ED")),
		lambda value: strip_suffix(value, Text("ing")),
		lambda value: strip_suffix(value, Text("ING")),
		lambda value: replace_suffix(value, Text("ing"), Text("e")),
		lambda value: replace_suffix(value, Text("ING"), Text("E")),
		lambda value: strip_doubled_consonant_suffix(value, Text("ing")),
		lambda value: strip_doubled_consonant_suffix(value, Text("ING")),
		lambda value: strip_suffix(value, Text("ly")),
		lambda value: strip_suffix(value, Text("LY")),
		lambda value: replace_suffix(value, Text("ily"), Text("y")),
		lambda value: replace_suffix(value, Text("ILY"), Text("Y")),
		lambda value: strip_suffix(value, Text("er")),
		lambda value: strip_suffix(value, Text("ER")),
		lambda value: strip_suffix(value, Text("est")),
		lambda value: strip_suffix(value, Text("EST")),
		lambda value: replace_suffix(value, Text("ier"), Text("y")),
		lambda value: replace_suffix(value, Text("IER"), Text("Y")),
		lambda value: replace_suffix(value, Text("iest"), Text("y")),
		lambda value: replace_suffix(value, Text("IEST"), Text("Y")),
		lambda value: strip_doubled_consonant_suffix(value, Text("er")),
		lambda value: strip_doubled_consonant_suffix(value, Text("ER")),
		lambda value: strip_doubled_consonant_suffix(value, Text("est")),
		lambda value: strip_doubled_consonant_suffix(value, Text("EST")),
		lambda value: strip_suffix(value, Text("ment")),
		lambda value: strip_suffix(value, Text("MENT")),
		lambda value: strip_suffix(value, Text("ness")),
		lambda value: strip_suffix(value, Text("NESS")),
		lambda value: replace_suffix(value, Text("iness"), Text("y")),
		lambda value: replace_suffix(value, Text("INESS"), Text("Y")),
		lambda value: strip_suffix(value, Text("able")),
		lambda value: strip_suffix(value, Text("ABLE")),
		lambda value: replace_suffix(value, Text("able"), Text("e")),
		lambda value: replace_suffix(value, Text("ABLE"), Text("E")),
	]

	for priority, rule in enumerate(rules):
		base = rule(word)
		if base:
			yield priority, base


def strip_suffix(word: Text, suffix: Text) -> Text | None:
	"""去掉指定后缀，无法去掉时返回None"""
	if len(word) <= len(suffix) or not word.endswith(suffix):
		return None
	return Text(word[: -len(suffix)])


def replace_suffix(word: Text, suffix: Text, replacement: Text) -> Text | None:
	"""将指定后缀替换为另一段文本，无法替换时返回None"""
	base = strip_suffix(word, suffix)
	if base is None:
		return None
	return Text(base + replacement)


def strip_doubled_consonant_suffix(word: Text, suffix: Text) -> Text | None:
	"""去掉后缀和词尾双写辅音，无法匹配时返回None"""
	base = strip_suffix(word, suffix)
	if base is None or len(base) < 2:
		return None
	if base[-1] != base[-2] or base[-1].lower() not in CONSONANTS:
		return None
	return Text(base[:-1])


def reorder_case_variants(words: list[Text]) -> list[Text]:
	"""把首字母大写词稳定移到末尾，再把前两字母大写词稳定移到末尾"""
	return move_matching_to_end(
		move_matching_to_end(words, is_initial_upper),
		is_second_initial_upper,
	)


def move_matching_to_end(
	words: list[Text], predicate: Callable[[Text], bool]
) -> list[Text]:
	"""把满足条件的词稳定移动到列表末尾"""
	unmatched: list[Text] = []
	matched: list[Text] = []
	for word in words:
		if predicate(word):
			matched.append(word)
		else:
			unmatched.append(word)

	return unmatched + matched


def is_initial_upper(word: Text) -> bool:
	"""判断首字母是否为大写"""
	return bool(word) and word[0].isupper()


def is_second_initial_upper(word: Text) -> bool:
	"""判断首字母和第二个字母是否都为大写"""
	return len(word) > 1 and word[0].isupper() and word[1].isupper()
