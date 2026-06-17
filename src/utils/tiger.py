from utils.types import CodeText


def code_len_group(code: str) -> int:
	"""返回码长分组，4码及以上归为4"""
	return min(len(code), 4)


def filter_tiger(
	upstream_tiger_dict: list[CodeText],
	sc2013: set[str],
) -> list[CodeText]:
	"""返回过滤并单一化编码后的虎码单字CodeText列表"""
	selected: list[CodeText | None] = []
	index_by_text: dict[str, int] = {}
	code_counts: dict[str, int] = {}

	for entry in upstream_tiger_dict:
		if entry.text not in sc2013:
			continue

		current_index = index_by_text.get(entry.text)
		if current_index is None:
			index_by_text[entry.text] = len(selected)
			selected.append(entry)
			code_counts[entry.code] = code_counts.get(entry.code, 0) + 1
			continue

		current = selected[current_index]
		if current is None:
			raise AssertionError("selected text index points to an empty row")
		if len(entry.code) >= len(current.code):
			continue

		# 后续短码只有在未被已选中的更高权重条目占用时才替换
		if code_counts.get(entry.code, 0) > 0:
			continue

		code_counts[current.code] -= 1
		if code_counts[current.code] == 0:
			del code_counts[current.code]
		selected[current_index] = None
		index_by_text[entry.text] = len(selected)
		selected.append(CodeText(code=entry.code, text=entry.text))
		code_counts[entry.code] = 1

	return [row for row in selected if row is not None]


def combine_tiger_add(
	tiger_rows: list[CodeText],
	zh_add_rows: list[CodeText],
) -> list[CodeText]:
	"""按码长分层合并虎码基础词和中文附加词"""
	rows: list[CodeText] = []
	for group in (1, 2, 3, 4):
		rows.extend(row for row in tiger_rows if code_len_group(row.code) == group)
		rows.extend(row for row in zh_add_rows if code_len_group(row.code) == group)
	return rows


def sort_zh_add(rows: list[CodeText]) -> list[CodeText]:
	"""返回按编码长度和字母顺序稳定排序后的附加词条CodeText列表"""
	seen_text: set[str] = set()
	for row in rows:
		if row.text in seen_text:
			print(f"警告：中文附加词text='{row.text}'重复")
		else:
			seen_text.add(row.text)

	return sorted(rows, key=lambda item: (len(item.code), item.code.casefold()))
