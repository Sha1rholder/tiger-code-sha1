def code_len_group(code: str) -> int:
	"""返回码长分组，4码及以上归为4"""
	return min(len(code), 4)


def filter_tiger(
	upstream_tiger_dict: list[tuple[str, str]],
	sc2013: set[str],
) -> list[tuple[str, str]]:
	"""返回过滤并单一化编码后的虎码单字(code, text)列表"""
	selected: list[tuple[str, str] | None] = []
	index_by_text: dict[str, int] = {}
	code_counts: dict[str, int] = {}

	for code, text in upstream_tiger_dict:
		if text not in sc2013:
			continue

		current_index = index_by_text.get(text)
		if current_index is None:
			index_by_text[text] = len(selected)
			selected.append((code, text))
			code_counts[code] = code_counts.get(code, 0) + 1
			continue

		current = selected[current_index]
		if current is None:
			raise AssertionError("selected text index points to an empty row")
		current_code, _current_text = current
		if len(code) >= len(current_code):
			continue

		# 后续短码只有在未被已选中的更高权重条目占用时才替换
		if code_counts.get(code, 0) > 0:
			continue

		code_counts[current_code] -= 1
		if code_counts[current_code] == 0:
			del code_counts[current_code]
		selected[current_index] = None
		index_by_text[text] = len(selected)
		selected.append((code, text))
		code_counts[code] = 1

	return [row for row in selected if row is not None]


def combine_tiger_add(
	tiger_rows: list[tuple[str, str]],
	zh_add_rows: list[tuple[str, str]],
) -> list[tuple[str, str]]:
	"""按码长分层合并虎码基础词和中文附加词"""
	rows: list[tuple[str, str]] = []
	for group in (1, 2, 3, 4):
		rows.extend(row for row in tiger_rows if code_len_group(row[0]) == group)
		rows.extend(row for row in zh_add_rows if code_len_group(row[0]) == group)
	return rows


def sort_zh_add(rows: list[tuple[str, str]]) -> list[tuple[str, str]]:
	"""返回按编码长度和字母顺序稳定排序后的附加词条(code, text)列表"""
	seen_text: set[str] = set()
	for _code, text in rows:
		if text in seen_text:
			print(f"警告：中文附加词text='{text}'重复")
		else:
			seen_text.add(text)

	return sorted(rows, key=lambda item: (len(item[0]), item[0].casefold()))
