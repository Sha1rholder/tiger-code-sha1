from dataclasses import dataclass
from pathlib import Path

from wordfreq import get_frequency_dict

MIN_WORD_LEN = 4
CONSONANTS = set("bcdfghjklmnpqrstvwxyz")


@dataclass(frozen=True)
class RankedWord:
	word: str
	frequency: float
	boosted_frequency: float
	demotion_count: int


@dataclass(frozen=True)
class WordInfo:
	word: str
	key: str
	frequency: float
	input_order: int


def get_result(esdb_filename: str) -> list[str]:
	"""返回按自定义顺序排列的英文单词列表，保留ESDB原始大小写"""
	return add_case_variants(
		[
			entry.word
			for entry in get_base_ranked_entries(esdb_filename)
			if len(entry.word) >= MIN_WORD_LEN
		]
	)


def get_base_ranked_entries(esdb_filename: str) -> list[RankedWord]:
	"""返回未过滤码长且不含派生大小写词条的英文词条排序指标"""
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

	# 去掉含非标准英文字母字符、无法匹配wordfreq的词
	infos = [
		WordInfo(
			word=word,
			key=word.casefold(),
			frequency=en_freq[word.casefold()],
			input_order=input_order,
		)
		for input_order, word in enumerate(esdb)
		if (
			word.isascii()
			and word.isalpha()
			and len(word) >= 3
			and word.casefold() in en_freq
		)
	]

	return rank_base_entries(infos)


def rank_base_entries(infos: list[WordInfo]) -> list[RankedWord]:
	"""按提权词频和降权次数排序"""
	infos_by_key = {info.key: info for info in infos}
	parent_by_key = build_parent_map(infos_by_key)

	boosted_frequency = {info.key: info.frequency for info in infos}
	demotion_count = {info.key: 0 for info in infos}

	for info in infos:
		ancestor_key = parent_by_key.get(info.key)
		while ancestor_key is not None:
			ancestor = infos_by_key[ancestor_key]
			if ancestor.frequency > info.frequency:
				boosted_frequency[ancestor_key] += info.frequency
				demotion_count[info.key] += 1
			ancestor_key = parent_by_key.get(ancestor_key)

	entries = [
		RankedWord(
			word=info.word,
			frequency=info.frequency,
			boosted_frequency=boosted_frequency[info.key],
			demotion_count=demotion_count[info.key],
		)
		for info in infos
	]

	entries.sort(
		key=lambda entry: (
			entry.demotion_count,
			-entry.boosted_frequency,
			infos_by_key[entry.word.casefold()].input_order,
		),
	)
	return entries


def build_parent_map(infos_by_key: dict[str, WordInfo]) -> dict[str, str]:
	"""为每个词选择唯一直接基本形式"""
	parent_by_key: dict[str, str] = {}
	for key in infos_by_key:
		candidates = [
			(priority, base_key)
			for priority, base_key in iter_base_candidates(key)
			if base_key != key and base_key in infos_by_key
		]
		if not candidates:
			continue

		_, parent_key = min(
			candidates,
			key=lambda candidate: (
				candidate[0],
				-infos_by_key[candidate[1]].frequency,
				infos_by_key[candidate[1]].input_order,
			),
		)
		parent_by_key[key] = parent_key

	return parent_by_key


def iter_base_candidates(word: str):
	"""按README中的规则顺序产出直接基本形式候选"""
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


def add_case_variants(en_dict: list[str]) -> list[str]:
	"""为首字母小写词生成首字母大写版本，为非全小写词生成全大写版本"""
	initial_caps: list[str] = []
	all_caps: list[str] = []
	seen = set(en_dict)
	for word in en_dict:
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

	return en_dict + initial_caps + all_caps


def write_result(filename: str, words: list[str]) -> None:
	"""将词表写为一行一词的纯文本文件"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


def write_review_tsv(filename: str, entries: list[RankedWord]) -> None:
	"""将词表和排序指标写为便于审查的TSV文件"""
	path = Path(filename)
	if path.parent != Path("."):
		path.parent.mkdir(parents=True, exist_ok=True)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("word\tfrequency\tboosted_frequency\tdemotion_count\n")
		for entry in entries:
			f.write(
				f"{entry.word}\t"
				f"{entry.frequency:.17g}\t"
				f"{entry.boosted_frequency:.17g}\t"
				f"{entry.demotion_count}\n"
			)


if __name__ == "__main__":
	write_result("lua/en_dict.txt", get_result("upstream/ESDB.txt"))
