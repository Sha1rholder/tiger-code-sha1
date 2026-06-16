import argparse
import os
import subprocess
from collections.abc import Iterable
from pathlib import Path

from utils import en, py_sc, tiger

ZH_DICT_HEADER = """---
name: tiger_sha1_zh
version: placeholder
sort: original
columns:
  - code
  - text
...
"""

PY_DICT_HEADER = """---
name: tiger_sha1_py
version: placeholder
sort: original
columns:
  - code
  - text
...
"""


def read_lines(filename: str) -> list[str]:
	"""读取UTF-8文本并按行返回"""
	return Path(filename).read_text(encoding="utf-8").splitlines()


def get_sc2013(levels: Iterable[Iterable[str]]) -> set[str]:
	"""合并《通用规范汉字表》多个级别为汉字集合"""
	sc2013: set[str] = set()
	for lines in levels:
		for line in lines:
			text = line.strip()
			if text:
				sc2013.add(text)

	return sc2013


def read_tiger_dict(filename: str) -> list[tuple[str, str]]:
	"""读取虎码上游词典正文，返回(code, text)，舍弃weight列"""
	rows: list[tuple[str, str]] = []
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
		rows.append((code, text))

	return rows


def read_py_dict(filename: str) -> list[tuple[str, int, str]]:
	"""读取拼音上游词典正文，返回(code, weight, text)"""
	rows: list[tuple[str, int, str]] = []
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
		rows.append((code, weight, text))

	if pending_text_parts:
		pending_text = "".join(pending_text_parts)
		raise SystemExit(f"第{pending_start_line}行不是有效的TSV行：{pending_text}")

	return rows


def read_zh_add(filename: str) -> list[tuple[str, str]]:
	"""读取中文附加词TSV，返回(code, text)"""
	rows: list[tuple[str, str]] = []
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
		rows.append((parts[0], parts[1]))

	return rows


def read_words(filename: str) -> list[str]:
	"""读取一行一词的纯文本词表"""
	words: list[str] = []
	for line in Path(filename).read_text(encoding="utf-8").splitlines():
		word = line.strip()
		if word:
			words.append(word)
	return words


def read_esdb_words(filename: str) -> set[str]:
	"""读取ESDB正文为拼写集合"""
	words: set[str] = set()
	after_sep = False
	for line in Path(filename).read_text(encoding="utf-8").splitlines():
		if line.strip() == "---":
			after_sep = True
			continue
		if not after_sep:
			continue

		word = line.strip()
		if word:
			words.add(word)
	return words


def write_rows(filename: str, dict_header: str, rows: list[tuple[str, str]]) -> None:
	"""写出带词典头的(code, text)词典"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.write(dict_header)
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def write_zh_add(filename: str, rows: list[tuple[str, str]]) -> None:
	"""写出中文附加词TSV"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.write("code\ttext\n")
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def write_words(filename: str, words: list[str]) -> None:
	"""写出一行一词的纯文本词表"""
	with open(filename, "w", encoding="utf-8", newline="") as f:
		for word in words:
			f.write(f"{word}\n")


def write_en_review_tsv(filename: str, entries: list[en.RankedWord]) -> None:
	"""写出英文词表审查TSV"""
	path = Path(filename)
	if path.parent != Path("."):
		path.parent.mkdir(parents=True, exist_ok=True)
	with path.open("w", encoding="utf-8", newline="") as f:
		f.write("word\tfrequency\tboosted_frequency\tdemotion_count\n")
		for entry in entries:
			f.write(
				f"{entry.word}\t"
				f"{entry.frequency:.17g}\t"
				f"{entry.boosted_frequency:.17g}\t"
				f"{entry.demotion_count}\n"
			)


def main(*, write_en_dict_review: bool = False) -> None:
	"""更新中文、拼音和英文词典并按需写出审查文件"""
	sc2013_set = get_sc2013(
		[
			read_lines("upstream/SC2013/level-1.txt"),
			read_lines("upstream/SC2013/level-2.txt"),
			read_lines("upstream/SC2013/level-3.txt"),
		]
	)

	py_rows = py_sc.get_py_sc(
		read_py_dict("upstream/tiger/PY_c.dict.yaml"),
		sc2013_set,
	)
	write_rows("tiger_sha1_py.dict.yaml", PY_DICT_HEADER, py_rows)

	tiger_rows = tiger.filter_tiger(
		read_tiger_dict("upstream/tiger/tiger.dict.yaml"),
		sc2013_set,
	)
	zh_add_rows = tiger.sort_zh_add(read_zh_add("add/0.Sha1rholder.zh.tsv"))
	write_zh_add("add/0.Sha1rholder.zh.tsv", zh_add_rows)

	zh_rows = tiger.combine_tiger_add(tiger_rows, zh_add_rows)
	write_rows("tiger_sha1_zh.dict.yaml", ZH_DICT_HEADER, zh_rows)

	en_add_words = en.sort_add_words(read_words("add/0.Sha1rholder.en.txt"))
	write_words("add/0.Sha1rholder.en.txt", en_add_words)

	en_base_entries = en.get_base_ranked_entries(read_esdb_words("upstream/ESDB.txt"))
	en_add_seen = set(en_add_words)
	en_base_words = [
		entry.word
		for entry in en_base_entries
		if len(entry.word) >= en.MIN_WORD_LEN and entry.word not in en_add_seen
	]
	en_dict = en_add_words + en.add_case_variants(en_base_words)
	write_words("lua/en_dict.txt", en_dict)
	if write_en_dict_review:
		write_en_review_tsv("temp/en_dict.tsv", en_base_entries)


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
		"--en_dict",
		action="store_true",
		help="Also write temp/en_dict.tsv for English dictionary review",
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
	main(write_en_dict_review=args.en_dict)
	if args.deploy:
		deploy()
	if args.sync:
		git_sync()
