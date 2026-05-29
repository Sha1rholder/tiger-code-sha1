"""一键更新所有虎码派生词典。

脚本路径按自身位置计算，所以无论从哪个目录调用，都依次运行同目录下的生成脚本；任何一步失败，整次更新立即停下。
"""

import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
SCRIPTS = ("extract_tiger_sc.py", "format_tiger_add.py", "extract_tiger_py.py")


def main() -> None:
	"""按固定顺序执行各生成器，让输出可预期。"""
	for script_name in SCRIPTS:
		script_path = SCRIPT_DIR / script_name
		print(f"==> {script_path}", flush=True)
		subprocess.run([sys.executable, str(script_path)], check=True)


if __name__ == "__main__":
	main()
