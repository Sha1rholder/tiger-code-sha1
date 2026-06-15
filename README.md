# Tiger Code SHA1

基于虎码、新手友好、面向程序员和技术写作者的中英混合输入方案

*目前仅支持 Weasel 小狼毫*

## 特性

- **中英混输**：没有自动上屏，支持中文和英文混合输入，无需切换 ASCII 模式
- **拼音反查**：通过 `|` 前缀触发拼音反查功能
- **虎码补全**：基于词频的虎码补全开关（默认关闭）
- **简化字表**：默认仅收录《通用规范汉字表（2013）》中的标准简体字，每个字只保留一个兼顾上游权重和短码占用的编码
- **英文词表**：以 ESDB 作为拼写集合，结合 `wordfreq` 词频、英文后缀变体关系和附加词生成候选顺序
- **更多符号**：全角模式自带一些常用特殊符号和 emoji
- **自动整理**：脚本化更新词表、整理附加词、部署和同步

## 文件结构

```text
├ tiger_sha1_weasel.schema.yaml		# 主输入方案
├ tiger_sha1_weasel.dict.yaml		# 主方案词典壳，导入alphabet和tiger_sha1_zh
├ tiger_sha1_zh.dict.yaml			# 中文基础词典（机器生成）
├ tiger_sha1_py.schema.yaml			# 拼音反查伪方案
├ tiger_sha1_py.dict.yaml			# 拼音反查词典（机器生成）
├ tiger_sha1_add_zh.tsv				# 中文附加词条，生成时自动排序并写回
├ tiger_sha1_add_en.txt				# 英文附加词条，生成时自动排序并写回
├ alphabet.dict.yaml				# 大写字母表
├ symbols.yaml						# 符号表
├ weasel.custom.yaml				# 小狼毫界面定制
├ lua/
│	├ clear_buffer_on_ctrl.lua		# 按 Ctrl 清空 buffer
│	├ commit_raw_before_symbol.lua	# 符号键提交 buffer
│	├ en_dict.txt					# 英文词表（机器生成）
│	├ en_weight_translate.lua		# 英文候选按词表顺序惰性产出
│	└ hide_en_comment.lua			# 隐藏英文补全建议
├ src/
│	├ main.py						# 读取源数据、解析格式、合并 SC2013、调用 utils、写出词典、部署、同步
│	├ README.md						# 开发文档
│	└ utils/
│		├ en.py						# 英文排序和大小写变体
│		├ py_sc.py					# 拼音反查过滤和排序
│		└ tiger.py					# 虎码过滤和中文附加词整理
└ upstream/
	├ tiger/						# 虎码原始数据
	├ SC2013/						# 通用规范汉字表
	└ ESDB.txt						# English Speller Database
```

## 使用方法

1. 安装依赖
	- [Weasel 小狼毫](https://rime.im/)
	- [Git](https://git-scm.com/)
	- [Astral uv](https://docs.astral.sh/uv/)
	- [Noto Sans SC 字体](https://fonts.google.com/noto/specimen/Noto+Sans+SC)
2. 中止 Weasel 程序并清空用户文件夹
3. 执行 `git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA\Rime"; uv run "$env:APPDATA\Rime\src\main.py" --deploy`
4. 在 Weasel 控制面板中选择 `tiger_sha1_weasel`

若要加减词，请编辑 `tiger_sha1_add_zh.tsv` 或 `tiger_sha1_add_en.txt`，然后执行 `uv run src/main.py --deploy`。不需要手动整理附加词表，脚本会自动排序并写回

`src/main.py` 可用参数：

- `--deploy`：更新词典后自动重新部署 Weasel
- `--en_dict`：更新词典时额外输出 `temp/en_dict.tsv` 供审查英文词表
- `--sync`：更新词典后自动执行 `git add .`、`git commit`、`git push` 以同步到上游（仅在 main 分支时触发 push，需要先配置 git repo）

若要使英文候选默认带尾随空格，可将 `tiger_sha1_weasel.schema.yaml > en_weight_translate > append_space_to_candidates` 的值改为 `true` 后重新部署

## 开发

实现细节见 `src/README.md`。设计约束是：`src/main.py` 负责所有项目文件读写、格式解析和 SC2013 合并，`src/utils/` 中的模块只接收结构化数据并返回结构化数据

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime Weasel](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
