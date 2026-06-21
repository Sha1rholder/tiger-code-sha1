import argparse
import os
import subprocess
from collections.abc import Iterable
from pathlib import Path

from utils import en, py_sc, tiger
from utils.types import Code, DictHeader, FilePath, Text, Weight

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


def read_words(filename: FilePath) -> list[Text]:
	"""读取一行一词的纯文本词典"""
	words: list[Text] = []
	for line in Path(filename).read_text(encoding="utf-8").splitlines():
		word = line.strip()
		if word:
			words.append(Text(word))
	return words


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
			for path in Path("add").iterdir()
			if path.is_file()
			and path.suffix == suffix
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


def write_words(filename: FilePath, words: list[Text]) -> None:
	"""写出一行一词的纯文本词典"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


def write_en_review_tsv(
	filename: FilePath, entries: list[tuple[Text, float, float, int]]
) -> None:
	"""写出英文词典审查TSV"""
	path = Path(filename)
	ensure_parent_dir(path)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("word\tfrequency\tboosted_frequency\tdemotion_count\n")
		for entry in entries:
			f.write(f"{entry[0]}\t{entry[1]:.17g}\t{entry[2]:.17g}\t{entry[3]}\n")


def sort_zh_add_files() -> list[tuple[Code, Text]]:
	"""排序写回所有中文附加词文件并返回合并排序后的词条"""
	rows: list[tuple[Code, Text]] = []
	for path in get_add_files(".tsv"):
		file_rows = tiger.sort_zh_add(read_zh_add(FilePath(str(path))))
		write_zh_add(FilePath(str(path)), file_rows)
		rows.extend(file_rows)

	return tiger.sort_zh_add(rows)


def sort_en_add_files() -> list[Text]:
	"""排序写回所有英文附加词文件并返回合并排序后的词典"""
	words: list[Text] = []
	for path in get_add_files(".txt"):
		file_words = en.sort_add_words(read_words(FilePath(str(path))))
		write_words(FilePath(str(path)), file_words)
		words.extend(file_words)

	return words


def main(*, debug: bool = False) -> None:
	"""更新中文、拼音和英文词典并按需写出调试文件"""
	sc2013_set = get_sc2013(
		[
			read_lines(FilePath("upstream/SC2013/level-1.txt")),
			read_lines(FilePath("upstream/SC2013/level-2.txt")),
			read_lines(FilePath("upstream/SC2013/level-3.txt")),
		]
	)

	py_rows = py_sc.get_py_sc(
		read_py_dict(FilePath("upstream/tiger/PY_c.dict.yaml")),
		sc2013_set,
	)
	write_rows(FilePath("tiger_sha1_py.dict.yaml"), PY_DICT_HEADER, py_rows)

	tiger_rows = tiger.filter_tiger(
		read_tiger_dict(FilePath("upstream/tiger/tiger.dict.yaml")),
		sc2013_set,
	)
	zh_add_rows = sort_zh_add_files()
	if debug:
		write_zh_add(
			FilePath("temp/zh_dict.tsv"),
			tiger.get_debug_zh_dict_rows(tiger_rows, sc2013_set),
		)
		write_zh_add(FilePath("temp/add.tsv"), zh_add_rows)

	zh_rows = tiger.combine_tiger_add(tiger_rows, zh_add_rows)
	write_rows(FilePath("tiger_sha1_zh.dict.yaml"), ZH_DICT_HEADER, zh_rows)

	en_add_words = sort_en_add_files()

	en_base_entries = en.get_base_ranked_entries(
		read_esdb_words(FilePath("upstream/ESDB.txt"))
	)
	en_add_seen = set(en_add_words)
	en_base_words = [
		entry[0]
		for entry in en_base_entries
		if len(entry[0]) >= en.MIN_WORD_LEN and entry[0] not in en_add_seen
	]
	(
		en_regular_words,
		en_initial_upper_words,
		en_second_initial_upper_words,
	) = en.reorder_case_variants(en_base_words)
	en_dict = (
		en_add_words
		+ en_regular_words
		+ en_initial_upper_words
		+ en_second_initial_upper_words
	)
	write_words(FilePath("lua/en_dict.txt"), en_dict)
	if debug:
		write_words(FilePath("temp/add.txt"), en_add_words)
		write_en_review_tsv(FilePath("temp/en_freq.tsv"), en_base_entries)


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
		"--deploy",
		action="store_true",
		help="Run WeaselDeployer.exe after updating dictionaries",
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
	main(debug=args.debug)
	if args.deploy:
		deploy()
	if args.sync:
		git_sync()
