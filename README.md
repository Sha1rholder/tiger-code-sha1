个人Rime虎码输入方案，仅保留规范简体字

`uv run src/update_tiger.py` to update and format dicts after modification.

- `SC2013/`：国务院2013规范简体字表
- `src/`：字表生成脚本
- `upstream/`：虎码秃版字表和拼音表，用于派生本方案，不要改
- `symbols.yaml`：符号表，切换成全角模式可输出一些自定义符号
- `tiger_add.dict.yaml`：手写补充词组，合入主方案，可含非简体字
- `tiger_py.dict.yaml`：从上游拼音字表筛选的单字拼音反查方案
- `tiger_py.schema.yaml`：拼音伪方案
- `tiger_sc.dict.yaml`：从上游字表派生的简体单字
- `tiger_weasel.schema.yaml`：Weasel端输入方案
