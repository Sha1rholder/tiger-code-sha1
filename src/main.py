import argparse
import subprocess

from utils import add, en, py_sc, sc2013, tiger

LOCAL_WEIGHT_STRIDE = 100
LENGTH_WEIGHT_BASE = {
	1: 300,
	2: 200,
	3: 100,
	4: 0,
}


def replace_tsv2(filename: str, rows: list[tuple[str, str]]) -> None:
	"""替换Rime词典tsv（2 rows）"""
	with open(filename, encoding="utf-8") as f:
		lines = f.readlines()

	for index, line in enumerate(lines):
		if line.strip() == "...":
			header = lines[: index + 1]
			break
	else:
		raise SystemExit(f"{filename}中找不到词典正文分隔符：...")

	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.writelines(header)
		for code, text in rows:
			f.write(f"{code}\t{text}\n")


def replace_tsv3(filename: str, rows: list[tuple[str, str, int]]) -> None:
	"""替换Rime词典tsv（3 rows）"""
	with open(filename, encoding="utf-8") as f:
		lines = f.readlines()

	for index, line in enumerate(lines):
		if line.strip() == "...":
			header = lines[: index + 1]
			break
	else:
		raise SystemExit(f"{filename}中找不到词典正文分隔符：...")

	with open(filename, "w", encoding="utf-8", newline="") as f:
		f.writelines(header)
		for code, text, weight in rows:
			f.write(f"{code}\t{weight}\t{text}\n")


def code_len_group(code: str) -> int:
	return len(code) if len(code) < 4 else 4


def add_prefix_local_weights(rows: list[tuple[str, str]]) -> list[tuple[str, str, int]]:
	prefix_counts_by_len: dict[int, dict[str, int]] = {}
	weighted_rows: list[tuple[str, str, int]] = []

	for code, text in rows:
		code_len = code_len_group(code)
		if code_len == 1:
			weighted_rows.append((code, text, LENGTH_WEIGHT_BASE[code_len]))
			continue

		prefix = code[:-1]
		prefix_counts = prefix_counts_by_len.setdefault(code_len, {})
		prefix_count = prefix_counts.get(prefix, 0)
		if prefix_count >= LOCAL_WEIGHT_STRIDE:
			raise SystemExit(
				f"prefix-local weight overflow: code length group {code_len}, "
				f"prefix {prefix!r} has more than {LOCAL_WEIGHT_STRIDE} entries"
			)

		prefix_counts[prefix] = prefix_count + 1
		local_weight = LOCAL_WEIGHT_STRIDE - prefix_count - 1
		weight = LENGTH_WEIGHT_BASE[code_len] + local_weight
		weighted_rows.append((code, text, weight))

	return weighted_rows


def main() -> None:
	sc2013_set = sc2013.get_result()

	py_rows = py_sc.get_result(sc2013_set)
	replace_tsv2("tiger_sha1_py.dict.yaml", py_rows)

	tiger_rows = tiger.get_result(sc2013_set)
	add_rows = add.get_result()
	tiger_add = tiger_rows + add_rows
	tiger_add.sort(key=lambda item: len(item[0]))
	tiger_add_weight = add_prefix_local_weights(tiger_add)

	en_rows = [(word, word) for word in en.get_result()]

	replace_tsv3("tiger_sha1.dict.yaml", tiger_add_weight)
	replace_tsv2("tiger_sha1_en.dict.yaml", en_rows)

	seen: set[tuple[str, str]] = set()
	for code, text in tiger_add + en_rows:
		if (code, text) in seen:
			print(f"Warning: duplicate entry found — code: {code}, text: {text}")
		else:
			seen.add((code, text))


def git_sync() -> None:
	"""Stage all changes, commit with user input, and push if on main."""
	print("Running git add .")
	subprocess.run(["git", "add", "."], check=True)

	# 检查是否有 staged changes
	result = subprocess.run(["git", "diff", "--cached", "--quiet"])
	if result.returncode != 0:
		msg = input('Commit message (default: "update"): ').strip()
		if not msg:
			msg = "update"
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
	parser = argparse.ArgumentParser(description="Update Rime dictionaries")
	parser.add_argument(
		"--deploy",
		action="store_true",
		help="Run WeaselDeployer.exe after updating dictionaries",
	)
	parser.add_argument(
		"--sync",
		action="store_true",
		help="Sync changes: git add, commit, and push (only on main)",
	)
	return parser.parse_args()


def deploy() -> None:
	deployer = r"C:\Program Files\Rime\weasel-0.17.4\WeaselDeployer.exe"
	print(f"Running {deployer} ...")
	subprocess.run([deployer, "/deploy"], check=True)
	print("Deploy complete.")


if __name__ == "__main__":
	import os
	from pathlib import Path

	os.chdir(Path(__file__).resolve().parent.parent)

	args = parse_args()
	main()
	if args.deploy:
		deploy()
	if args.sync:
		git_sync()
