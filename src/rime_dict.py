"""Rime 词典脚本共用的细活。

这些脚本都在同一种文件里行走：前半段是 YAML 头，`...` 之后是 TSV 正文。本模块只处理这些朴素规则，把“怎样读写”从各脚本的“筛什么”里分出来。
"""

import re
from collections.abc import Iterable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SC2013_FILES = ("level-1.txt", "level-2.txt", "level-3.txt")

_COLUMN_RE = re.compile(r"\s*-\s*([^#\s]+)")


def split_header_and_tsv(path: Path) -> tuple[list[str], list[str]]:
	"""以 `...` 为界，分开 Rime 词典的头与正文。"""
	lines = path.read_text(encoding="utf-8").splitlines()

	for index, line in enumerate(lines):
		if line.strip() == "...":
			return lines[: index + 1], lines[index + 1 :]

	raise ValueError(f"missing YAML/TSV separator '...' in {path}")


def parse_columns(header: list[str]) -> list[str]:
	"""从 YAML 头里的 `columns` 读出 TSV 字段顺序。"""
	columns: list[str] = []
	in_columns = False

	for line in header:
		if line.startswith("columns:"):
			in_columns = True
			continue

		if in_columns:
			match = _COLUMN_RE.match(line)
			if match:
				columns.append(match.group(1))
				continue

			if line and not line.startswith((" ", "\t")):
				break

	if not columns:
		raise ValueError("missing columns definition")

	return columns


def require_columns(columns: list[str], *required: str) -> dict[str, int]:
	"""确认脚本要用的字段存在，并返回字段下标。"""
	indexes: dict[str, int] = {}

	for column in required:
		try:
			indexes[column] = columns.index(column)
		except ValueError as error:
			required_text = ", ".join(required)
			raise ValueError(f"source columns must include {required_text}") from error

	return indexes


def iter_tsv_fields(
	tsv: Iterable[str],
	*,
	min_field_count: int,
	exact_field_count: int | None = None,
) -> Iterable[list[str]]:
	"""逐行给出可用字段；空行、注释行、缺字段的行自然略过。"""
	for line in tsv:
		if not line or line.startswith("#"):
			continue

		fields = line.split("\t")

		if len(fields) < min_field_count:
			continue

		if exact_field_count is not None and len(fields) != exact_field_count:
			continue

		yield fields


def row_from_fields(columns: list[str], fields: list[str]) -> dict[str, str]:
	"""把一行 TSV 按字段名收束成字典。"""
	return dict(zip(columns, fields, strict=False))


def format_rows(columns: list[str], rows: Iterable[dict[str, str]]) -> list[str]:
	"""按目标词典的字段顺序，把行重新写回 TSV。"""
	lines: list[str] = []

	for row in rows:
		missing_columns = set(columns).difference(row)
		if missing_columns:
			missing = ", ".join(sorted(missing_columns))
			raise ValueError(f"source row missing target columns: {missing}")

		lines.append("\t".join(row[column] for column in columns))

	return lines


def write_dict(path: Path, header: list[str], tsv: Iterable[str]) -> None:
	"""保留原头部，只替换正文。"""
	path.write_text("\n".join([*header, *tsv]) + "\n", encoding="utf-8")


def sort_by_code(rows: Iterable[dict[str, str]]) -> list[dict[str, str]]:
	"""先短码、再字母序。"""
	return sorted(rows, key=lambda row: (len(row["code"]), row["code"]))


def sort_by_code_and_weight(rows: Iterable[dict[str, str]]) -> list[dict[str, str]]:
	"""先短码、再字母序；同码时权重高者在前。"""
	return sorted(
		rows, key=lambda row: (len(row["code"]), row["code"], -int(row["weight"]))
	)


def load_sc2013(root: Path = ROOT) -> set[str]:
	"""读入 SC2013 三级字表，得到允许进入简体字典的字集。"""
	chars: set[str] = set()
	sc2013_dir = root / "SC2013"

	for filename in SC2013_FILES:
		path = sc2013_dir / filename
		for line in path.read_text(encoding="utf-8").splitlines():
			text = line.strip()
			if not text or text.startswith("#"):
				continue
			chars.update(text)

	return chars
