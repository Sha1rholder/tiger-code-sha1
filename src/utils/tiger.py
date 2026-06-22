from utils.types import Code, Text


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
	tiger_rows = filter_tiger(upstream_tiger_dict, sc2013, zh_recodes)
	sorted_zh_add_files_rows = [sort_zh_add(rows) for rows in zh_add_files_rows]
	zh_add_rows = sort_zh_add(
		[row for file_rows in sorted_zh_add_files_rows for row in file_rows]
	)
	debug_zh_dict_rows = get_debug_zh_dict_rows(tiger_rows, sc2013)
	zh_rows = combine_tiger_add(tiger_rows, zh_add_rows)

	return sorted_zh_add_files_rows, zh_add_rows, debug_zh_dict_rows, zh_rows


def code_len_group(code: str) -> int:
	"""返回码长分组，4码及以上归为4"""
	return min(len(code), 4)


def filter_tiger(
	upstream_tiger_dict: list[tuple[Code, Text]],
	sc2013: set[Text],
	zh_recodes: list[tuple[Code, Text]],
) -> list[tuple[Code, Text]]:
	"""返回过滤并单一化编码后的虎码单字(code, text)列表"""
	recode_by_text = validate_zh_recodes(zh_recodes, sc2013)
	reserved_codes = set(recode_by_text.values())
	recoded_texts = set(recode_by_text)
	seen_recoded_texts: set[str] = set()
	selected: list[tuple[Code, Text] | None] = []
	index_by_text: dict[str, int] = {}
	code_counts: dict[str, int] = {}

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

	return [row for row in selected if row is not None]


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


def get_debug_zh_dict_rows(
	rows: list[tuple[Code, Text]],
	sc2013: set[Text],
) -> list[tuple[Code, Text]]:
	"""返回只包含简体中文单字的调试词条"""
	return [row for row in rows if len(row[1]) == 1 and row[1] in sc2013]


def combine_tiger_add(
	tiger_rows: list[tuple[Code, Text]],
	zh_add_rows: list[tuple[Code, Text]],
) -> list[tuple[Code, Text]]:
	"""按码长分层合并虎码基础词和中文附加词"""
	rows: list[tuple[Code, Text]] = []
	for group in (1, 2, 3, 4):
		rows.extend(row for row in tiger_rows if code_len_group(row[0]) == group)
		rows.extend(row for row in zh_add_rows if code_len_group(row[0]) == group)
	return rows


def sort_zh_add(rows: list[tuple[Code, Text]]) -> list[tuple[Code, Text]]:
	"""返回按编码长度和字母顺序稳定排序后的附加词条(code, text)列表"""
	seen_text: set[str] = set()
	for row in rows:
		if row[1] in seen_text:
			print(f"警告：中文附加词text='{row[1]}'重复")
		else:
			seen_text.add(row[1])

	return sorted(rows, key=lambda item: (len(item[0]), item[0].casefold()))
