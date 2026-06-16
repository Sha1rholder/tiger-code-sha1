def get_py_sc(
	upstream_py_dict: list[tuple[str, int, str]],
	sc2013: set[str],
) -> list[tuple[str, str]]:
	"""返回按词频降序排列并过滤后的拼音(code, text)列表"""
	rows = [
		(code, weight, text)
		for code, weight, text in upstream_py_dict
		if all(char in sc2013 for char in text)
	]
	rows.sort(key=lambda row: row[1], reverse=True)
	return [(code, text) for code, _weight, text in rows]
