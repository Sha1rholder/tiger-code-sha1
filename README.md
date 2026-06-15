# Tiger Code SHA1

基于虎码、新手友好、面向程序员和技术写作者的极致实用主义中英混合输入方案

*目前仅支持Weasel小狼毫*

## 特性

- **中英混输**：没有自动上屏，支持中文和英文混合输入，无需切换ASCII模式
- **拼音反查**：通过`|`前缀触发拼音反查功能
- **虎码补全**：基于词频的虎码补全开关（默认关闭）
- **简化字表**：默认仅收录《通用规范汉字表（2013）》中的标准简体字且每个字只保留一个编码
- **更多符号**：全角模式自带一些常用特殊符号和emoji
- **自动加词**：脚本化更新词表、部署、同步

## 文件结构

```text
├ tiger_sha1_weasel.schema.yaml		# 主输入方案
├ tiger_sha1.dict.yaml				# 主词典（机器生成）
├ tiger_sha1_py.schema.yaml			# 拼音反查伪方案
├ tiger_sha1_py.dict.yaml			# 拼音反查词典（机器生成）
├ tiger_sha1_add_zh.tsv				# 附加词条（中文词表后）
├ tiger_sha1_add_en.txt				# 附加词条（英文词表前）
├ alphabet.dict.yaml				# 大写字母表
├ symbols.yaml						# 符号表
├ weasel.custom.yaml				# 小狼毫界面定制
├ lua/
│	├ clear_buffer_on_ctrl.lua		# 按Ctrl清空buffer
│	├ commit_raw_before_symbol.lua	# 符号键提交buffer
│	├ en_dict.txt					# 英文词表（机器生成）
│	├ en_weight_translate.lua		# 英文候选按词表顺序惰性产出
│	└ hide_en_comment.lua			# 隐藏英文补全建议
├ src/
│	├ main.py						# 更新dicts、重新部署、同步git
│	├ README.md						# 开发文档
│	└ utils/
│		├ tiger.py					# 虎码处理
│		├ en.py						# 英文处理
│		├ py_sc.py					# 拼音处理
│		├ sc2013.py					# 规范汉字处理
│		└ add.py					# 附加词条处理
└ upstream/
	├ tiger/						# 虎码原始数据
	├ SC2013/						# 通用规范汉字表
	└ ESDB.txt						# 英文拼写数据库
```

## 使用方法

1. 安装依赖
	- [Weasel小狼毫](https://rime.im/)
	- [Git](https://git-scm.com/)
	- [Astral.uv](https://docs.astral.sh/uv/)
	- [Noto Sans SC字体](https://fonts.google.com/noto/specimen/Noto+Sans+SC)
2. 中止Weasel程序并清空用户文件夹
3. 执行`git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA\Rime"; uv run "$env:APPDATA\Rime\src\main.py" --deploy`
4. 在Weasel控制面板中选择`tiger_sha1_weasel`

若要加减词，请编辑`tiger_sha1_add_zh.tsv`或`tiger_sha1_add_en.txt`，然后执行`uv run "$env:APPDATA\Rime\src\main.py --deploy"`。不需要手动整理附加词表，脚本会自动处理

`src/main.py`可用参数：
- `--deploy`：更新词典后自动重新部署Weasel
- `--en_dict`：更新词典时额外输出`temp/en_dict.tsv`供审查英文词表
- `--sync`：更新词典后自动执行`git add .`、`git commit`、`git push`以同步到上游（仅在main分支时触发push，需要先配置git repo）

若要使英文候选默认带尾随空格，可将`tiger_sha1_weasel.default.yaml > en_weight_translate > append_space_to_candidates`的值改为true后重新部署

## 开发

实现细节见`src/README.md`

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime Weasel](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
