# Tiger Code SHA1

基于虎码改编、面向程序员和技术写作者的中英混合形码输入方案

>[虎码](https://www.tiger-code.com/)是一种较新的汉字形码输入法，它通过将汉字拆分为更细致的字根，并用一套系统化的编码规则（通常基于字形结构与笔画特征）来对应键盘输入，从而实现高效率、低重码的中文输入。相比五笔等传统形码，虎码在字根设计和编码规则上更强调系统性和扩展性，力求在保持输入速度优势的同时提高对生僻字和复杂字形的覆盖能力，但学习门槛也相对较高，需要专门记忆字根和拆字规则

*目前仅支持Weasel小狼毫*

## 特性

- **中英混输**：没有自动上屏，支持中文和英文混合输入，一般无需切换ASCII模式
- **拼音反查**：通过`|`前缀触发拼音反查功能
- **简化字表**：默认仅收录国标简体字，每个字仅保留一个编码
- **中文词组**：基于词频自动补充常用词组，并支持手动加词
- **英文词典**：基于词频和标准拼写库自动生成稳定英文词典，并支持英文附加词按降权次数插入
- **更多符号**：全角模式自带一些常用特殊符号和emoji
- **自动整理**：脚本化更新词典、整理附加词、部署和同步

## 文件结构

```text
Rime/
├ tiger_sha1_weasel.schema.yaml	# 主输入方案
├ tiger_sha1_weasel.dict.yaml	# 主方案词典壳，导入alphabet和tiger_sha1_zh
├ tiger_sha1_zh.dict.yaml		# 中文基础词典（机器生成）
├ tiger_sha1_py.schema.yaml		# 拼音反查伪方案
├ tiger_sha1_py.dict.yaml		# 拼音反查词典（机器生成）
├ alphabet.dict.yaml			# 大写字母表
├ symbols.yaml					# 符号表
├ weasel.custom.yaml			# 小狼毫界面定制
├ custom/
│	├ char.unfilter.txt			# 额外放行单字
│	├ char.recode.tsv			# 基础单字改码表
│	├ *.zh.tsv					# 自定义中文词典
│	└ *.en.tsv					# 自定义英文词典
├ lua/
│	├ commit_raw_symbol.lua		# 有buffer时符号键直接提交ASCII
│	├ en_weight_translate.lua	# 英文候选按词典顺序惰性产出
│	└ en_dict.txt				# 英文词典（机器生成）
├ src/
│	├ main.py					# 读取源数据、解析格式、调用utils入口、写出词典、部署、同步
│	├ README.md					# 开发文档
│	└ utils/
│		├ en.py					# 英文词典生成器和英文数据编排入口
│		├ py_sc.py				# 拼音反查生成器
│		└ tiger.py				# 中文词典生成器和中文数据编排入口
└ upstream/
	├ ESDB.txt					# English Speller Database
	├ tiger/					# 虎码原始数据
	│	├ tiger.dict.yaml		# 秃版虎码字表
	│	└ PY_c.dict.yaml		# 秃版拼音表
	└ SC2013/					# 通用规范汉字表
```

## 使用方法

1. 安装依赖
	- [Weasel小狼毫](https://rime.im/)（请使用默认路径）
	- [Git](https://git-scm.com/)
	- [Astral uv](https://docs.astral.sh/uv/)
	- [Noto Sans SC字体](https://fonts.google.com/noto/specimen/Noto+Sans+SC)
2. 终止Weasel程序并清空用户文件夹
3. 执行`git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA\Rime"; uv run "$env:APPDATA\Rime\src\main.py" --deploy`
4. 在Weasel控制面板中选择`tiger_sha1_weasel`

若要加减词，请编辑`custom/`中的`.zh.tsv`中文词典或`.en.tsv`英文词典，然后执行`uv run src/main.py --deploy`。不需要手动整理附加词典，脚本会先按单文件排序并写回，这个排序只用于让加词文件便于阅读，不决定同码候选顺序。基础虎码单字、手动中文附加词和自动中文加词会按编码分组合并，同码候选依次拼接，再按首次出现顺序去重，且每个编码最多保留前5个候选。自动中文加词来自`wordfreq`中文词频。英文附加词文件按`demotion_count`和`aAbBcC...zZ`逐位词面顺序排序；`demotion_count`用于插入基础英文词对应降权分组的首部。`custom/`中以`-`或`.-`开头的词典会被生成逻辑忽略；以`.`开头的词典会被`.gitignore`忽略

若要让不在《通用规范汉字表（2013）》中的单字参与中文和拼音词典生成，请把单字逐行写入`custom/char.unfilter.txt`。若要覆盖基础虎码单字编码，请编辑`custom/char.recode.tsv`，表头固定为`code<TAB>text`，且`code`和`text`都不能重复。改码表只处理基础单字，不用于添加词组

## 开发

实现细节见`src/README.md`

`src/main.py`可用参数：
- `--deploy`：自动重新部署Weasel
- `--debug`：额外在`temp/`中输出`zh_dict.tsv`、`add.tsv`、`add.txt`、`en_dict.tsv`供审查中间词典
- `--sync`：自动执行`git add .`、`git commit`、`git push`以同步到上游（仅在main分支触发push）

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime Weasel](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
