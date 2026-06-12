import argparse
import os
import subprocess
from pathlib import Path

from utils import add, en, py_sc, sc2013, tiger


def main() -> None:
	sc2013_set = sc2013.get_result()

	py_rows = py_sc.get_result(sc2013_set)
	py_sc.write_result("tiger_sha1_py.dict.yaml", py_rows)

	tiger_rows = tiger.get_result(sc2013_set)
	add_rows = add.get_result("tiger_sha1_add.tsv")
	tiger_add = tiger_rows + add_rows

	en_words = en.get_result()
	en_rows = [(word, word) for word in en_words]

	tiger.write_result("tiger_sha1.dict.yaml", tiger_add)
	en.write_result("lua/en_dict.txt", en_words)

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


def git_sync(*, force: bool = False) -> None:
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
		push_command = ["git", "push"]
		if force:
			push_command.append("--force")
		print(f"Running {' '.join(push_command)}")
		subprocess.run(push_command, check=True)
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
	sync_group = parser.add_mutually_exclusive_group()
	sync_group.add_argument(
		"--sync",
		action="store_true",
		help="Sync changes: git add, commit, and push (only on main)",
	)
	sync_group.add_argument(
		"--force-sync",
		action="store_true",
		help="Sync changes with git push --force on main",
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
	main()
	if args.deploy:
		deploy()
	if args.sync or args.force_sync:
		git_sync(force=args.force_sync)
