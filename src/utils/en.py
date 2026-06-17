from collections.abc import Iterable

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

	return sorted(clean_words, key=lambda word: (len(word), word.casefold()))


def get_base_ranked_entries(
	esdb_words: set[str],
) -> list[tuple[Text, float, float, int]]:
	"""返回未过滤码长且不含派生大小写词条的英文词条排序指标"""
	esdb: list[str] = dedupe_case_variants(esdb_words)
	en_freq: dict[str, float] = get_frequency_dict("en")

	infos: list[tuple[Text, Text, float]] = [
		(Text(word), Text(word.casefold()), en_freq[word.casefold()])
		for word in esdb
		if (
			word.isascii()
			and word.isalpha()
			and len(word) >= 3
			and word.casefold() in en_freq
		)
	]

	return rank_base_entries(infos)


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
			entry[0].casefold(),
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


def iter_base_candidates(word: str):
	"""按规则顺序产出直接基本形式候选"""
	rules = [
		lambda value: strip_suffix(value, "s"),
		lambda value: strip_suffix(value, "es"),
		lambda value: replace_suffix(value, "ies", "y"),
		lambda value: strip_suffix(value, "d"),
		lambda value: strip_suffix(value, "ed"),
		lambda value: replace_suffix(value, "ied", "y"),
		lambda value: strip_doubled_consonant_suffix(value, "ed"),
		lambda value: strip_suffix(value, "ing"),
		lambda value: replace_suffix(value, "ing", "e"),
		lambda value: strip_doubled_consonant_suffix(value, "ing"),
		lambda value: strip_suffix(value, "ly"),
		lambda value: replace_suffix(value, "ily", "y"),
		lambda value: strip_suffix(value, "er"),
		lambda value: strip_suffix(value, "est"),
		lambda value: replace_suffix(value, "ier", "y"),
		lambda value: replace_suffix(value, "iest", "y"),
		lambda value: strip_doubled_consonant_suffix(value, "er"),
		lambda value: strip_doubled_consonant_suffix(value, "est"),
		lambda value: strip_suffix(value, "ment"),
		lambda value: strip_suffix(value, "ness"),
		lambda value: replace_suffix(value, "iness", "y"),
		lambda value: strip_suffix(value, "able"),
		lambda value: replace_suffix(value, "able", "e"),
	]

	for priority, rule in enumerate(rules):
		base = rule(word)
		if base:
			yield priority, base


def strip_suffix(word: str, suffix: str) -> str | None:
	"""去掉指定后缀，无法去掉时返回None"""
	if len(word) <= len(suffix) or not word.endswith(suffix):
		return None
	return word[: -len(suffix)]


def replace_suffix(word: str, suffix: str, replacement: str) -> str | None:
	"""将指定后缀替换为另一段文本，无法替换时返回None"""
	base = strip_suffix(word, suffix)
	if base is None:
		return None
	return base + replacement


def strip_doubled_consonant_suffix(word: str, suffix: str) -> str | None:
	"""去掉后缀和词尾双写辅音，无法匹配时返回None"""
	base = strip_suffix(word, suffix)
	if base is None or len(base) < 2:
		return None
	if base[-1] != base[-2] or base[-1] not in CONSONANTS:
		return None
	return base[:-1]


def dedupe_case_variants(words: Iterable[str]) -> list[str]:
	"""同一单词有多种大小写形式时，逐位优先保留更偏小写的形式"""
	groups: dict[str, list[str]] = {}
	for word in words:
		groups.setdefault(word.casefold(), []).append(word)

	return sorted(
		(min(group, key=case_variant_sort_key) for group in groups.values()),
		key=lambda word: (word.casefold(), word),
	)


def case_variant_sort_key(word: str) -> tuple[tuple[int, ...], str]:
	"""小写字符优先，其次用词面保证确定性"""
	char_key = tuple(
		0 if char.islower() else 1 if char.isupper() else 2 for char in word
	)
	return char_key, word


def add_case_variants(en_dict: list[Text]) -> list[Text]:
	"""为全小写词生成首字母大写版本，为非全小写词生成全大写版本"""
	initial_caps: list[Text] = []
	all_caps: list[Text] = []
	seen = set(en_dict)
	for word in en_dict:
		if word.islower():
			initial_cap = Text(word[0].upper() + word[1:])
			if initial_cap not in seen:
				initial_caps.append(initial_cap)
				seen.add(initial_cap)
		if not word.isupper():
			all_cap = Text(word.upper())
			if all_cap not in seen:
				all_caps.append(all_cap)
				seen.add(all_cap)

	return en_dict + initial_caps + all_caps
