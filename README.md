# Tiger Code SHA1

基于虎码改编、面向程序员和技术写作者的中英混合形码输入方案

>[虎码](https://www.tiger-code.com/)是一种较新的汉字形码输入法，它通过将汉字拆分为比五笔等传统形码更细致的字根并用退火算法找到较优分布，以实现高效率、低重码的中文输入。相比五笔，虎码的学习门槛较高，需要专门记忆字根和拆字规则

**目前仅支持Weasel小狼毫**

## 特性

- **中英混输**：没有自动上屏，支持中文和英文混合输入，一般无需切换ASCII模式
- **拼音反查**：通过`|`前缀触发拼音反查功能
- **简化字表**：默认仅收录国标简体字，每个字仅保留一个编码
- **中文词组**：基于词频自动补充常用词组，并支持手动加词
- **英文词典**：基于词频和标准拼写库自动生成稳定英文词典
- **更多符号**：全角模式自带常用特殊符号和emoji
- **自动整理**：脚本化更新词典、整理附加词、部署和同步

## 文件结构

```text
Rime/
├ tiger_sha1_zh.schema.yaml	# 主输入方案
├ tiger_sha1_zh.dict.yaml	# 中文词典（机器生成）
├ tiger_sha1_py.schema.yaml		# 拼音反查伪方案
├ tiger_sha1_py.dict.yaml		# 拼音反查词典（机器生成）
├ symbols.yaml					# 符号表
├ weasel.custom.yaml			# 小狼毫界面定制
├ lua/
│	└ commit_raw_symbol.lua		# buffer符号直出和数组符号连按
└ src/							# 词表编排代码
```

## 使用方法

1. 安装依赖
	- [Weasel小狼毫](https://rime.im/)（请使用默认路径）
	- [Git](https://git-scm.com/)
	- [Astral uv](https://docs.astral.sh/uv/)
	- [Rust工具链](https://rust-lang.org)
	- [Noto Sans SC字体](https://fonts.google.com/noto/specimen/Noto+Sans+SC)
2. 终止Weasel程序并清空用户文件夹
3. 执行`git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA/Rime"; cd "$env:APPDATA/Rime/src/data/wordfreq/"; uv run main.py; cd "$env:APPDATA/Rime/"; cargo run`
4. 在Weasel控制面板中选择`tiger_sha1_zh`

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime Weasel](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
