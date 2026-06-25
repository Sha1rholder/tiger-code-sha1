from wordfreq import get_frequency_dict

from utils.types import Code, Freq, Text

AUTO_ZH_ADD_LIMIT = 5
MIN_ZH_FREQUENCY = Freq(
	0.000001
)  # 值改为0.000001后进程有概率崩溃，性能也极大下降，目前不知道怎么修（越低越容易崩）


def build_zh_outputs(
	upstream_tiger_dict: list[tuple[Code, Text]],
	sc2013: set[Text],
	zh_recodes: list[tuple[Code, Text]],
	zh_add_files_rows: list[list[tuple[Code, Text]]],
) -> tuple[
	list[list[tuple[Code, Text]]],
	list[tuple[Code, Text]],
	list[tuple[Code, Text]],
	list[tuple[Code, Text]],
]:
	"""生成中文附加词、调试词条和最终虎码词典数据"""
	tiger_dict = filter_tiger(upstream_tiger_dict, sc2013, zh_recodes)
	tiger_rows = flatten_tiger(tiger_dict)
	wordfreq_zh = get_wordfreq_zh(sc2013)
	sorted_zh_add_files_rows = [sort_zh_add(rows) for rows in zh_add_files_rows]
	manual_zh_add_dict = get_manual_zh_add_dict(
		[row for file_rows in zh_add_files_rows for row in file_rows]
	)
	auto_zh_add_dict = get_auto_zh_add_dict(tiger_dict, wordfreq_zh)
	zh_add_dict = merge_zh_dicts(manual_zh_add_dict, auto_zh_add_dict)
	zh_dict = merge_zh_dicts(tiger_dict, manual_zh_add_dict, auto_zh_add_dict)
	zh_add_rows = flatten_tiger(zh_add_dict)
	zh_rows = flatten_tiger(zh_dict)

	return sorted_zh_add_files_rows, zh_add_rows, tiger_rows, zh_rows


def filter_tiger(
	upstream_tiger_dict: list[tuple[Code, Text]],
	sc2013: set[Text],
	zh_recodes: list[tuple[Code, Text]],
) -> dict[Code, list[Text]]:
	"""返回过滤并单一化编码后的虎码编码到文本列表映射"""
	recode_by_text = validate_zh_recodes(zh_recodes, sc2013)
	reserved_codes = set(recode_by_text.values())
	recoded_texts = set(recode_by_text)
	seen_recoded_texts: set[Text] = set()
	selected: list[tuple[Code, Text] | None] = []
	index_by_text: dict[Text, int] = {}
	code_counts: dict[Code, int] = {}

	for upstream_code, text in upstream_tiger_dict:
		if text not in sc2013:
			continue

		code = recode_by_text.get(text)
		if code is None:
			if upstream_code in reserved_codes:
				continue
			code = upstream_code
		else:
			seen_recoded_texts.add(text)

		entry = (Code(code), Text(text))
		current_index = index_by_text.get(text)
		if current_index is None:
			index_by_text[text] = len(selected)
			selected.append(entry)
			code_counts[code] = code_counts.get(code, 0) + 1
			continue

		current = selected[current_index]
		if current is None:
			raise AssertionError("selected text index points to an empty row")
		if len(code) >= len(current[0]):
			continue

		# 后续短码只有在未被已选中的更高权重条目占用时才替换
		if code_counts.get(code, 0) > 0:
			continue

		code_counts[current[0]] -= 1
		if code_counts[current[0]] == 0:
			del code_counts[current[0]]
		selected[current_index] = None
		index_by_text[text] = len(selected)
		selected.append(entry)
		code_counts[code] = 1

	missing_texts = sorted(recoded_texts - seen_recoded_texts)
	if missing_texts:
		raise SystemExit(f"单字改码text未在上游虎码中出现：{', '.join(missing_texts)}")

	tiger_dict: dict[Code, list[Text]] = {}
	for row in selected:
		if row is None:
			continue
		code, text = row
		tiger_dict.setdefault(code, []).append(text)

	return tiger_dict


def flatten_tiger(tiger_dict: dict[Code, list[Text]]) -> list[tuple[Code, Text]]:
	"""展开虎码编码到文本列表映射为(code, text)列表"""
	return [(code, text) for code, texts in tiger_dict.items() for text in texts]


def get_wordfreq_zh(sc2013: set[Text]) -> list[Text]:
	"""返回按词频降序过滤后的中文自动加词候选词"""
	wordfreq_zh_freq = get_wordfreq_zh_freq(sc2013)
	return [entry[0] for entry in wordfreq_zh_freq]


def get_wordfreq_zh_freq(sc2013: set[Text]) -> list[tuple[Text, Freq]]:
	"""返回按词频降序过滤后的中文词频数据"""
	wordfreq_zh_freq: list[tuple[Text, Freq]] = [
		(Text(word), Freq(frequency))
		for word, frequency in sorted(
			get_frequency_dict("zh").items(),
			key=lambda entry: -entry[1],
		)
		if (
			frequency >= MIN_ZH_FREQUENCY
			and len(word) > 1
			and all(Text(char) in sc2013 for char in word)
		)
	]

	return drop_containing_words(wordfreq_zh_freq)


def drop_containing_words(
	entries: list[tuple[Text, Freq]],
) -> list[tuple[Text, Freq]]:
	"""丢弃包含其它候选词的中文词频条目"""
	words = [entry[0] for entry in entries]
	return [
		entry
		for entry in entries
		if not any(len(other) < len(entry[0]) and other in entry[0] for other in words)
	]


def get_auto_zh_add_dict(
	tiger_dict: dict[Code, list[Text]], wordfreq_zh: list[Text]
) -> dict[Code, list[Text]]:
	"""按单字编码和词频候选生成自动中文加词"""
	zh_add_dict: dict[Code, list[Text]] = {}
	for code, texts in tiger_dict.items():
		# 下面三行换成 candidate_limit = AUTO_ZH_ADD_LIMIT 对结果无影响，纯是加点速度
		candidate_limit = AUTO_ZH_ADD_LIMIT - len(texts)
		if candidate_limit <= 0:
			continue

		prefixes = tuple(texts)
		for word in wordfreq_zh:
			if word.startswith(prefixes):
				zh_add_dict.setdefault(code, []).append(word)
				candidate_limit -= 1
				if candidate_limit == 0:
					break

	return zh_add_dict


def get_manual_zh_add_dict(rows: list[tuple[Code, Text]]) -> dict[Code, list[Text]]:
	"""按编码分组手动中文附加词并报警同码重复文本"""
	zh_add_dict = rows_to_zh_add_dict(rows)
	for code, texts in zh_add_dict.items():
		seen_text: set[Text] = set()
		for text in texts:
			if text in seen_text:
				print(f"警告：中文附加词code='{code}'text='{text}'重复")
			else:
				seen_text.add(text)

	return zh_add_dict


def rows_to_zh_add_dict(rows: list[tuple[Code, Text]]) -> dict[Code, list[Text]]:
	"""把(code, text)列表转换为编码到文本列表映射"""
	zh_add_dict: dict[Code, list[Text]] = {}
	for code, text in rows:
		zh_add_dict.setdefault(code, []).append(text)

	return zh_add_dict


def merge_zh_dicts(*dicts: dict[Code, list[Text]]) -> dict[Code, list[Text]]:
	"""合并多个编码到文本列表映射并去重限制同码候选数"""
	merged: dict[Code, list[Text]] = {}
	for zh_dict in dicts:
		for code, texts in zh_dict.items():
			merged.setdefault(code, []).extend(texts)

	return {code: dedupe_limit_texts(texts) for code, texts in merged.items()}


def dedupe_limit_texts(texts: list[Text]) -> list[Text]:
	"""按首次出现顺序去重并限制文本列表长度"""
	deduped: list[Text] = []
	seen_texts: set[Text] = set()
	for text in texts:
		if text in seen_texts:
			continue
		seen_texts.add(text)
		deduped.append(text)
		if len(deduped) == AUTO_ZH_ADD_LIMIT:
			break

	return deduped


def validate_zh_recodes(
	zh_recodes: list[tuple[Code, Text]],
	sc2013: set[Text],
) -> dict[Text, Code]:
	"""校验中文单字改码并返回text到code的映射"""
	recode_by_text: dict[Text, Code] = {}
	text_by_code: dict[Code, Text] = {}
	for code, text in zh_recodes:
		if not code:
			raise SystemExit(f"单字改码code不能为空：{text}")
		if not text:
			raise SystemExit(f"单字改码text不能为空：{code}")
		if len(text) != 1:
			raise SystemExit(f"单字改码text必须是单字：{text}")
		if text not in sc2013:
			raise SystemExit(f"单字改码text不在放行字表中：{text}")
		if text in recode_by_text:
			raise SystemExit(f"单字改码text重复：{text}")
		if code in text_by_code:
			raise SystemExit(f"单字改码code重复：{code}")
		recode_by_text[text] = code
		text_by_code[code] = text

	return recode_by_text


def sort_zh_add(rows: list[tuple[Code, Text]]) -> list[tuple[Code, Text]]:
	"""返回按code字母顺序稳定排序后的附加词条(code, text)列表"""
	return sorted(rows, key=lambda item: item[0].lower())
