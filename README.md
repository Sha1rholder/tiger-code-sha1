# Tiger Code SHA1

基于虎码改编、面向程序员和技术写作者的中英混合魔改[虎码](https://www.tiger-code.com/)输入方案

## 特性

- **中英混输**：没有自动上屏，支持中英文混输
- **拼音反查**：通过`|`前缀触发拼音反查功能
- **简化字表**：默认仅收录国标简体字，每个字仅保留一个编码
- **中文词组**：基于词频自动补充常用词组，并支持手动加词
- **英文词典**：基于词频和标准拼写库自动生成英文单词候选
- **更多符号**：全角模式自带常用特殊符号和emoji（按F8切换）

## 文件结构

```text
Rime/
├ tiger_sha1_zh.schema.yaml		# 主输入方案
├ tiger_sha1_zh.dict.yaml		# 中文词典（机器生成）
├ tiger_sha1_py.schema.yaml		# 拼音反查伪方案
├ tiger_sha1_py.dict.yaml		# 拼音反查词典（机器生成）
├ symbols.yaml					# 符号表
├ default.custom.yaml			# Rime默认配置
├ weasel.custom.yaml			# 小狼毫界面定制（其他框架可忽略）
├ lua/commit_raw_symbol.lua		# 符号输入体验优化
└ src/							# 词典生成代码
```

## 使用方法

1. 安装依赖
2. 清空Rime用户文件夹
3. 执行`git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA/Rime"; cd "$env:APPDATA/Rime/src/data/wordfreq/"; uv run main.py; cd "$env:APPDATA/Rime"; cargo run --release`
4. 在Weasel控制面板中选择`tiger_sha1_zh`

fcitx5目录为`~/.local/share/fcitx5/rime`

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime Weasel](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
