from utils.types import CodeText, CodeWeightText


def get_py_sc(
	upstream_py_dict: list[CodeWeightText],
	sc2013: set[str],
) -> list[CodeText]:
	"""返回按词频降序排列并过滤后的拼音CodeText列表"""
	rows = [
		entry
		for entry in upstream_py_dict
		if all(char in sc2013 for char in entry.text)
	]
	rows.sort(key=lambda row: row.weight, reverse=True)
	return [CodeText(code=entry.code, text=entry.text) for entry in rows]
