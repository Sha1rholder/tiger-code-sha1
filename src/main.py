import argparse
import os
import subprocess
from pathlib import Path

from utils import add, en, py_sc, sc2013, tiger


def main(*, write_en_dict_review: bool = False) -> None:
	sc2013_set = sc2013.get_result()

	py_rows = py_sc.get_result(sc2013_set)
	py_sc.write_result("tiger_sha1_py.dict.yaml", py_rows)

	tiger_rows = tiger.get_result(sc2013_set, "upstream/tiger/tiger.dict.yaml")
	add_rows = add.get_result("tiger_sha1_add.tsv")
	tiger_add = tiger_rows + add_rows

	en_base_entries = en.get_base_ranked_entries("upstream/ESDB.txt")
	en_dict = en.add_case_variants(
		[entry.word for entry in en_base_entries if len(entry.word) >= en.MIN_WORD_LEN]
	)
	en_rows = [(word, word) for word in en_dict]

	tiger.write_result("tiger_sha1.dict.yaml", tiger_add)
	en.write_result("lua/en_dict.txt", en_dict)
	if write_en_dict_review:
		en.write_review_tsv("temp/en_dict.tsv", en_base_entries)

	seen: set[tuple[str, str]] = set()
	duplicates = 0
	for code, text in tiger_add + en_rows:
		if (code, text) in seen:
			print(f"Warning: duplicate entry found — code: {code}, text: {text}")
			duplicates += 1
		else:
			seen.add((code, text))
	if duplicates == 0:
		print("All clear! No duplicate entries found.")


def git_sync() -> None:
	"""Stage all changes, commit with user input, and push if on main."""
	print("Running git add .")
	subprocess.run(["git", "add", "."], check=True)

	# 检查是否有 staged changes
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
