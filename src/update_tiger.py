"""一键更新所有虎码派生词典。

脚本路径按自身位置计算，所以无论从哪个目录调用，都并行运行同目录下的生成脚本；任何一步失败，整次更新立即停下。
"""

import subprocess
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
SCRIPTS = (
	"extract_tiger_sc.py",
	"format_tiger_add.py",
	"extract_tiger_py.py",
	"make_tiger_en.py",
)


def run_script(script_name: str) -> None:
	"""执行单个生成器脚本。"""
	script_path = SCRIPT_DIR / script_name
	print(f"==> {script_path}", flush=True)
	subprocess.run([sys.executable, str(script_path)], check=True)


def main() -> None:
	"""并行执行各生成器，任何失败立即停止。"""
	with ProcessPoolExecutor() as executor:
		futures = {executor.submit(run_script, name): name for name in SCRIPTS}
		for future in as_completed(futures):
			try:
				future.result()
			except subprocess.CalledProcessError as e:
				print(
					f"==> {futures[future]} 失败，退出码 {e.returncode}",
					file=sys.stderr,
				)
				sys.exit(1)


if __name__ == "__main__":
	main()
