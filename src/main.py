"""
1. 创建`temp/`文件夹（若本就存在则静默跳过）
2. 合并`SC2013/`的3个字表为set `sc2013`
3. 筛拼音生成字表`py` (code, weight, text)
	- 从`upstream/PY_c.dict.yaml`提取字表`py`并去掉所有text不在`sc2013`中的行（text必须完全匹配，因此只会保留单字）
	- 以weight降序排列输出`tiger_py.dict.yaml`
4. 筛虎码单字生成字表`tiger` (code, weight, text)
	- 从`upstream/tiger.dict.yaml`提取字表`tiger`并去掉所有text不在`sc2013`中的行
	- 去重。对于text相同的行，只保留靠上的（不要依赖weight）
	- 输出`temp/py.tsv`
5. 对`tiger`“分频”
	- 每个单字母code最靠上行的weight设为90000000，然后对于每个code重码把weight设为89999999、89999998……以此类推
	- 每个双字母code最靠上行的weight设为9000000，然后对于每个code重码把weight设为8999999、8999998……以此类推
	- 每个三字母code最靠上行的weight设为900000，然后对于每个code重码把weight设为899999、899998……以此类推
	- 每个四字母code最靠上行的weight设为90000，然后对于每个code重码把weight设为89999、89998……以此类推
	- 输出`temp/tiger.tsv`
6. format `custom/add.tsv`并生成字表`add`
	- 从`custom/add.tsv`提取字表`add`
	- 把`add`按code长度升序排列
	- 对于相同的code长度，按字母顺序（先a后z）排列
	- 对于相同的code，按weight降序排列
	- 对于相同的code和weight，打印错误告诉用户哪重了然后非零退出
	- 对于每组相同的code，使weight成为以9为最大值-1为公差的等差数列
	- 写回`custom/add.tsv`
	- 对于每个单字母code，weight加10000000
	- 对于每个双字母code，weight加1000000
	- 对于每个三字母code，weight加100000
	- 对于每个四字母code，weight加10000
	- 对于五及以上字母的code，weight加1000
7. 合并`tiger`和`add`
	- 合并`tiger`和`add`生成字表`tiger_add`
	- 把`tiger_add`以weight降序排列
	- 输出`temp/tiger_add.tsv`
8. 制作字表`en`
	- `from wordfreq import get_frequency_dict`
	- 生成英语字表`en` (code, weight, text)
	- 对`en`去掉
		- code长度小于等于4的行
		- text不全是标准英文小写字母的行
		- 不在`upstream/ESDB.txt`中的行
	- 输出`temp/en_raw.tsv`
	- 使`en`中weight最小值为1，然后每遇到一个比上一个weight值更高的weight，就使其+1
	- 输出`temp/en.tsv`
9. 合并`en`和`tiger_add`
	- 把`en` append到`tiger_add`的末尾，生成`tiger_add_en`
	- 用`tiger_add_en`替换`tiger.dict.yaml`的tsv部分
"""

from __future__ import annotations

import csv
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

from wordfreq import get_frequency_dict

ROOT = Path(__file__).resolve().parents[1]
TEMP_DIR = ROOT / "temp"
SC2013_DIR = ROOT / "SC2013"
UPSTREAM_DIR = ROOT / "upstream"
CUSTOM_DIR = ROOT / "custom"


@dataclass(frozen=True)
class Entry:
	code: str
	weight: int
	text: str


def read_sc2013() -> set[str]:
	chars: set[str] = set()
	for path in sorted(SC2013_DIR.glob("level-*.txt")):
		for line in path.read_text(encoding="utf-8-sig").splitlines():
			text = line.strip()
			if text:
				chars.add(text)
	return chars


def iter_rime_dict(path: Path) -> list[Entry]:
	entries: list[Entry] = []
	in_body = False

	for line_no, raw_line in enumerate(
		path.read_text(encoding="utf-8-sig").splitlines(), 1
	):
		line = raw_line.rstrip("\n")
		if not in_body:
			if line.strip() == "...":
				in_body = True
			continue
		if not line or line.lstrip().startswith("#"):
			continue

		parts = line.split("\t")
		if len(parts) < 3:
			continue

		text, code, weight_text = parts[:3]
		try:
			weight = int(weight_text)
		except ValueError as exc:
			raise ValueError(
				f"{path}:{line_no}: invalid weight {weight_text!r}"
			) from exc
		entries.append(Entry(code=code, weight=weight, text=text))

	return entries


def write_tsv(path: Path, entries: list[Entry]) -> None:
	with path.open("w", encoding="utf-8", newline="") as file:
		writer = csv.writer(file, delimiter="\t", lineterminator="\n")
		writer.writerow(["code", "weight", "text"])
		for entry in entries:
			writer.writerow([entry.code, entry.weight, entry.text])


def read_tsv(path: Path) -> list[Entry]:
	with path.open("r", encoding="utf-8-sig", newline="") as file:
		reader = csv.DictReader(file, delimiter="\t")
		required = {"code", "weight", "text"}
		if reader.fieldnames is None or set(reader.fieldnames) < required:
			raise ValueError(f"{path}: expected TSV header with code, weight, text")

		entries: list[Entry] = []
		for line_no, row in enumerate(reader, 2):
			code = (row["code"] or "").strip()
			text = row["text"] or ""
			weight_text = (row["weight"] or "").strip()
			if not code or not text or not weight_text:
				raise ValueError(
					f"{path}:{line_no}: code, weight, and text must be non-empty"
				)
			try:
				weight = int(weight_text)
			except ValueError as exc:
				raise ValueError(
					f"{path}:{line_no}: invalid weight {weight_text!r}"
				) from exc
			entries.append(Entry(code=code, weight=weight, text=text))
	return entries


def write_rime_dict(path: Path, name: str, entries: list[Entry]) -> None:
	with path.open("w", encoding="utf-8", newline="") as file:
		file.write(
			f"""---
name: {name}
version: "1.0"
sort: by_weight
use_preset_vocabulary: false
columns:
  - code
  - weight
  - text
...
"""
		)
		for entry in entries:
			file.write(f"{entry.code}\t{entry.weight}\t{entry.text}\n")


def build_py(sc2013: set[str]) -> list[Entry]:
	entries = [
		entry
		for entry in iter_rime_dict(UPSTREAM_DIR / "PY_c.dict.yaml")
		if entry.text in sc2013
	]
	return sorted(entries, key=lambda entry: entry.weight, reverse=True)


def build_tiger(sc2013: set[str]) -> list[Entry]:
	seen_text: set[str] = set()
	entries: list[Entry] = []
	for entry in iter_rime_dict(UPSTREAM_DIR / "tiger.dict.yaml"):
		if entry.text not in sc2013 or entry.text in seen_text:
			continue
		seen_text.add(entry.text)
		entries.append(entry)
	return entries


def refrequency_tiger(entries: list[Entry]) -> list[Entry]:
	base_by_length = {1: 90000000, 2: 9000000, 3: 900000, 4: 90000}
	seen_by_code: defaultdict[str, int] = defaultdict(int)
	result: list[Entry] = []

	for entry in entries:
		base = base_by_length.get(len(entry.code), 0)
		weight = base - seen_by_code[entry.code]
		seen_by_code[entry.code] += 1
		result.append(Entry(code=entry.code, weight=weight, text=entry.text))

	return result


def format_add(entries: list[Entry]) -> tuple[list[Entry], list[Entry]]:
	sorted_entries = sorted(
		entries, key=lambda entry: (len(entry.code), entry.code, -entry.weight)
	)
	seen_code_weight: dict[tuple[str, int], Entry] = {}

	for entry in sorted_entries:
		key = (entry.code, entry.weight)
		if key in seen_code_weight:
			first = seen_code_weight[key]
			print(
				"duplicate custom/add.tsv code+weight: "
				f"code={entry.code!r} weight={entry.weight} "
				f"text={first.text!r} and text={entry.text!r}",
				file=sys.stderr,
			)
			raise SystemExit(1)
		seen_code_weight[key] = entry

	group_index: defaultdict[str, int] = defaultdict(int)
	formatted: list[Entry] = []
	for entry in sorted_entries:
		weight = 9 - group_index[entry.code]
		group_index[entry.code] += 1
		formatted.append(Entry(code=entry.code, weight=weight, text=entry.text))

	boost_by_length = {1: 10000000, 2: 1000000, 3: 100000, 4: 10000}
	boosted = [
		Entry(
			code=entry.code,
			weight=entry.weight + boost_by_length.get(len(entry.code), 1000),
			text=entry.text,
		)
		for entry in formatted
	]
	return formatted, boosted


def load_esdb() -> set[str]:
	words: set[str] = set()
	in_body = False
	for line in (UPSTREAM_DIR / "ESDB.txt").read_text(encoding="utf-8").splitlines():
		if not in_body:
			if line.strip() == "---":
				in_body = True
			continue
		word = line.strip().lower()
		if word:
			words.add(word)
	return words


def build_en_raw() -> list[Entry]:
	esdb = load_esdb()
	frequencies = get_frequency_dict("en")
	entries: list[Entry] = []
	for word, frequency in frequencies.items():
		if len(word) <= 4:
			continue
		if not word.isalpha() or not word.isascii() or not word.islower():
			continue
		if word not in esdb:
			continue
		weight = max(1, round(frequency * 1_000_000_000))
		entries.append(Entry(code=word, weight=weight, text=word))
	return sorted(entries, key=lambda entry: entry.weight, reverse=True)


def normalize_en_weights(entries: list[Entry]) -> list[Entry]:
	sorted_asc = sorted(entries, key=lambda entry: entry.weight)
	result: list[Entry] = []
	current_weight = 1
	prev_weight: int | None = None
	for entry in sorted_asc:
		if prev_weight is not None and entry.weight > prev_weight:
			current_weight += 1
		result.append(Entry(code=entry.code, weight=current_weight, text=entry.text))
		prev_weight = entry.weight
	return sorted(result, key=lambda entry: entry.weight, reverse=True)


def main() -> None:
	TEMP_DIR.mkdir(exist_ok=True)

	sc2013 = read_sc2013()

	py_entries = build_py(sc2013)
	write_rime_dict(ROOT / "tiger_py.dict.yaml", "tiger_py", py_entries)

	tiger_raw = build_tiger(sc2013)
	write_tsv(TEMP_DIR / "py.tsv", tiger_raw)

	tiger_entries = refrequency_tiger(tiger_raw)
	write_tsv(TEMP_DIR / "tiger.tsv", tiger_entries)

	add_formatted, add_boosted = format_add(read_tsv(CUSTOM_DIR / "add.tsv"))
	write_tsv(CUSTOM_DIR / "add.tsv", add_formatted)

	tiger_add = sorted(
		tiger_entries + add_boosted, key=lambda entry: entry.weight, reverse=True
	)
	write_tsv(TEMP_DIR / "tiger_add.tsv", tiger_add)

	en_raw = build_en_raw()
	write_tsv(TEMP_DIR / "en_raw.tsv", en_raw)

	en = normalize_en_weights(en_raw)
	write_tsv(TEMP_DIR / "en.tsv", en)

	tiger_add_en = tiger_add + en
	write_rime_dict(ROOT / "tiger.dict.yaml", "tiger", tiger_add_en)


if __name__ == "__main__":
	main()
