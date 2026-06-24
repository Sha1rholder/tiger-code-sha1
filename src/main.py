import argparse
import os
import subprocess
import sys
import time
from collections.abc import Iterable
from pathlib import Path

from utils import en, py_sc, tiger
from utils.types import Code, DictHeader, FilePath, Freq, Text, Weight

ZH_DICT_HEADER: DictHeader = DictHeader("""\
---
name: tiger_sha1_zh
version: placeholder
sort: original
columns:
  - code
  - text
...
""")

PY_DICT_HEADER: DictHeader = DictHeader("""\
---
name: tiger_sha1_py
version: placeholder
sort: original
columns:
  - code
  - text
...
""")

_last_print_time = time.perf_counter()


def main(*, debug: bool = False) -> None:
	"""更新中文、拼音和英文词典并按需写出调试文件"""
	global _last_print_time

	print("Now doing loading SC2013 character set.", flush=True)
	_last_print_time = time.perf_counter()
	sc2013_set = get_sc2013(
		[
			read_lines(FilePath("upstream/SC2013/level-1.txt")),
			read_lines(FilePath("upstream/SC2013/level-2.txt")),
			read_lines(FilePath("upstream/SC2013/level-3.txt")),
			read_lines(FilePath("custom/char.unfilter.txt")),
		]
	)

	now = time.perf_counter()
	print(
		f"Completed loading SC2013 character set in {now - _last_print_time:.2f}s. ",
		flush=True,
	)
	_last_print_time = now
	py_rows = py_sc.get_py_sc(
		read_py_dict(FilePath("upstream/tiger/PY_c.dict.yaml")),
		sc2013_set,
	)
	write_rows(FilePath("tiger_sha1_py.dict.yaml"), PY_DICT_HEADER, py_rows)

	now = time.perf_counter()
	print(
		f"Completed building pinyin dictionary in {now - _last_print_time:.2f}s. ",
		flush=True,
	)
	_last_print_time = now
	zh_add_files = get_add_files(".zh.tsv")
	(
		sorted_zh_add_files_rows,
		zh_add_rows,
		debug_zh_dict_rows,
		zh_rows,
	) = tiger.build_zh_outputs(
		read_tiger_dict(FilePath("upstream/tiger/tiger.dict.yaml")),
		sc2013_set,
		read_zh_add(FilePath("custom/char.recode.tsv")),
		[read_zh_add(FilePath(str(path))) for path in zh_add_files],
	)
	for path, rows in zip(zh_add_files, sorted_zh_add_files_rows, strict=True):
		write_zh_add(FilePath(str(path)), rows)
	if debug:
		now = time.perf_counter()
		print(
			f"Completed building Chinese dictionary in {now - _last_print_time:.2f}s. ",
			flush=True,
		)
		_last_print_time = now
		write_zh_add(FilePath("temp/zh_dict.tsv"), debug_zh_dict_rows)
		write_zh_add(FilePath("temp/zh_add.tsv"), zh_add_rows)
		now = time.perf_counter()
		print(
			f"Completed writing Chinese debug files in {now - _last_print_time:.2f}s. ",
			flush=True,
		)
		_last_print_time = now
	write_rows(FilePath("tiger_sha1_zh.dict.yaml"), ZH_DICT_HEADER, zh_rows)

	now = time.perf_counter()
	completed_zh_task = (
		"writing Chinese dictionary" if debug else "building Chinese dictionary"
	)
	print(
		f"Completed {completed_zh_task} in {now - _last_print_time:.2f}s. ",
		flush=True,
	)
	_last_print_time = now
	en_add_files = get_add_files(".en.tsv")
	(
		sorted_en_add_files_entries,
		en_add_entries,
		en_dict,
		en_base_entries,
	) = en.build_en_outputs(
		[read_en_add(FilePath(str(path))) for path in en_add_files],
		read_esdb_words(FilePath("upstream/ESDB.txt")),
	)
	for path, entries in zip(en_add_files, sorted_en_add_files_entries, strict=True):
		write_en_add(FilePath(str(path)), entries)
	write_words(FilePath("lua/en_dict.txt"), en_dict)
	if debug:
		now = time.perf_counter()
		print(
			f"Completed building English dictionary in {now - _last_print_time:.2f}s. ",
			flush=True,
		)
		_last_print_time = now
		write_en_add(FilePath("temp/en_add.tsv"), en_add_entries)
		write_en_review_tsv(FilePath("temp/en_dict.tsv"), en_base_entries)
	now = time.perf_counter()
	if debug:
		print(
			f"Completed writing English debug files in {now - _last_print_time:.2f}s. ",
			flush=True,
		)
	else:
		print(
			f"Completed building English dictionary in {now - _last_print_time:.2f}s. ",
			flush=True,
		)


def read_lines(filename: FilePath) -> list[str]:
	"""读取UTF-8文本并按行返回"""
	return Path(filename).read_text(encoding="utf-8").splitlines()


def get_sc2013(levels: Iterable[Iterable[str]]) -> set[Text]:
	"""合并《通用规范汉字表》多个级别为汉字集合"""
	sc2013: set[Text] = set()
	for lines in levels:
		for line in lines:
			text = line.strip()
			if text:
				sc2013.add(Text(text))

	return sc2013


def read_tiger_dict(filename: FilePath) -> list[tuple[Code, Text]]:
	"""读取虎码上游词典正文，返回(code, text)列表，舍弃weight列"""
	rows: list[tuple[Code, Text]] = []
	after_sep = False
	for line_number, line in enumerate(
		Path(filename).read_text(encoding="utf-8").splitlines(), 1
	):
		if line.strip() == "...":
			after_sep = True
			continue
		if not after_sep or not line:
			continue

		parts = line.split("\t")
		if len(parts) < 2:
			raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
		text, code = parts[:2]
		rows.append((Code(code), Text(text)))

	return rows


def read_py_dict(filename: FilePath) -> list[tuple[Code, Weight, Text]]:
	"""读取拼音上游词典正文，返回(code, weight, text)列表"""
	rows: list[tuple[Code, Weight, Text]] = []
	after_sep = False
	pending_text_parts: list[str] = []
	pending_start_line: int | None = None
	for line_number, line in enumerate(
		Path(filename).read_text(encoding="utf-8").splitlines(), 1
	):
		if line.strip() == "...":
			after_sep = True
			continue
		if not after_sep or not line:
			continue

		if "\t" not in line:
			if pending_start_line is None:
				pending_start_line = line_number
			pending_text_parts.append(line)
			continue

		parts = line.split("\t")
		if len(parts) < 3:
			raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
		text, code, weight_text = parts[:3]
		if pending_text_parts:
			text = "".join(pending_text_parts) + text
			pending_text_parts.clear()
			pending_start_line = None
		try:
			weight = int(weight_text)
		except ValueError as error:
			raise SystemExit(f"第{line_number}行weight不是整数：{line}") from error
		rows.append((Code(code), Weight(weight), Text(text)))

	if pending_text_parts:
		pending_text = "".join(pending_text_parts)
		raise SystemExit(f"第{pending_start_line}行不是有效的TSV行：{pending_text}")

	return rows


def read_zh_add(filename: FilePath) -> list[tuple[Code, Text]]:
	"""读取中文附加词TSV，返回(code, text)列表"""
	rows: list[tuple[Code, Text]] = []
	for line_number, line in enumerate(
		Path(filename).read_text(encoding="utf-8").splitlines(), 1
	):
		if not line:
			continue

		parts = line.split("\t")
		if line_number == 1 and parts == ["code", "text"]:
			continue
		if len(parts) != 2:
			raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")
		rows.append((Code(parts[0]), Text(parts[1])))

	return rows


def read_en_add(filename: FilePath) -> list[tuple[Text, int]]:
	"""读取英文附加词TSV，返回(text, demotion_count)列表"""
	rows: list[tuple[Text, int]] = []
	header_seen = False
	for line_number, line in enumerate(
		Path(filename).read_text(encoding="utf-8").splitlines(), 1
	):
		if not line:
			continue

		parts = line.split("\t")
		if not header_seen:
			if parts != ["text", "demotion_count"]:
				raise SystemExit(f"第{line_number}行不是英文附加词TSV表头：{line}")
			header_seen = True
			continue
		if len(parts) != 2:
			raise SystemExit(f"第{line_number}行不是有效的TSV行：{line}")

		text = parts[0].strip()
		if not text:
			continue
		try:
			demotion_count = int(parts[1])
		except ValueError as error:
			raise SystemExit(
				f"第{line_number}行demotion_count不是整数：{line}"
			) from error
		if demotion_count < 0:
			raise SystemExit(f"第{line_number}行demotion_count不能为负数：{line}")
		rows.append((Text(text), demotion_count))

	if not header_seen:
		raise SystemExit(f"{filename}缺少英文附加词TSV表头")

	return rows


def read_esdb_words(filename: FilePath) -> set[Text]:
	"""读取ESDB正文为拼写集合"""
	words: set[Text] = set()
	after_sep = False
	for line in Path(filename).read_text(encoding="utf-8").splitlines():
		if line.strip() == "---":
			after_sep = True
			continue
		if not after_sep:
			continue

		word = line.strip()
		if word:
			words.add(Text(word))
	return words


def is_ignored_add_file(path: Path) -> bool:
	"""判断附加词文件是否应被生成逻辑忽略"""
	return path.name.startswith("-") or path.name.startswith(".-")


def get_add_files(suffix: str) -> list[Path]:
	"""按文件名字符顺序返回指定后缀的附加词文件"""
	return sorted(
		(
			path
			for path in Path("custom").iterdir()
			if path.is_file()
			and path.name.endswith(suffix)
			and not is_ignored_add_file(path)
		),
		key=lambda path: path.name,
	)


def ensure_parent_dir(path: Path) -> None:
	"""确保目标文件父目录存在"""
	if path.parent != Path("."):
		path.parent.mkdir(parents=True, exist_ok=True)


def write_rows(
	filename: FilePath, dict_header: DictHeader, rows: list[tuple[Code, Text]]
) -> None:
	"""写出带词典头的CodeText词典"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.write(dict_header)
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def write_zh_add(filename: FilePath, rows: list[tuple[Code, Text]]) -> None:
	"""写出(code, text)两列TSV"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("code\ttext\n")
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def write_en_add(filename: FilePath, rows: list[tuple[Text, int]]) -> None:
	"""写出(text, demotion_count)两列TSV"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("text\tdemotion_count\n")
		for text, demotion_count in rows:
			f.write(f"{text}\t{demotion_count}\n")


def write_words(filename: FilePath, words: list[Text]) -> None:
	"""写出一行一词的纯文本词典"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


def write_en_review_tsv(
	filename: FilePath, entries: list[tuple[Text, Freq, Freq, int]]
) -> None:
	"""写出英文词典审查TSV"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("word\tfrequency\tboosted_frequency\tdemotion_count\n")
		for entry in entries:
			f.write(f"{entry[0]}\t{entry[1]:.17g}\t{entry[2]:.17g}\t{entry[3]}\n")


def git_sync() -> None:
	"""暂存、提交并在main分支时推送"""
	print("Running git add .")
	subprocess.run(["git", "add", "."], check=True)

	result = subprocess.run(["git", "diff", "--cached", "--quiet"])
	if result.returncode != 0:
		msg = input("Commit message (press enter to discard): ").strip()
		if not msg:
			print("Skipping git commit.")
			return
		print(f'Running git commit -m "{msg}"')
		subprocess.run(["git", "commit", "-m", msg], check=True)
	else:
		print("Nothing to commit, working tree clean.")

	branch = subprocess.check_output(
		["git", "branch", "--show-current"],
		text=True,
	).strip()
	if branch == "main":
		print("Running git push")
		subprocess.run(["git", "push"], check=True)
		print("Push complete.")
	else:
		print(f"Branch is '{branch}', skipping push.")


def parse_args() -> argparse.Namespace:
	"""解析命令行参数"""
	parser = argparse.ArgumentParser(description="Update Rime dictionaries")
	parser.add_argument(
		"--compile",
		action="store_true",
		help="Update generated dictionaries",
	)
	parser.add_argument(
		"--deploy",
		action="store_true",
		help="Run WeaselDeployer.exe",
	)
	parser.add_argument(
		"--debug",
		action="store_true",
		help="Also write temp debug files for dictionary review",
	)
	parser.add_argument(
		"--sync",
		action="store_true",
		help="Sync changes: git add, commit, and push (only on main)",
	)
	if len(sys.argv) == 1:
		parser.print_help()
		raise SystemExit(0)
	return parser.parse_args()


def deploy() -> None:
	"""调用WeaselDeployer执行部署"""
	deployer = r"C:\Program Files\Rime\weasel-0.17.4\WeaselDeployer.exe"
	print(f"Running {deployer} ...")
	subprocess.run([deployer, "/deploy"], check=True)
	print("Deploy complete.")


if __name__ == "__main__":
	os.chdir(Path(__file__).resolve().parent.parent)

	args = parse_args()
	if args.compile or args.debug:
		main(debug=args.debug)
	if args.deploy:
		deploy()
	if args.sync:
		git_sync()
