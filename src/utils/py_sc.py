from utils.types import Code, Text, Weight


def get_py_sc(
	upstream_py_dict: list[tuple[Code, Weight, Text]],
	sc2013: set[Text],
) -> list[tuple[Code, Text]]:
	"""返回按词频降序排列并过滤后的拼音(code, text)列表"""
	rows: list[tuple[Code, Weight, Text]] = [
		entry for entry in upstream_py_dict if all(char in sc2013 for char in entry[2])
	]
	rows.sort(key=lambda row: row[1], reverse=True)
	return [(Code(entry[0]), Text(entry[2])) for entry in rows]
