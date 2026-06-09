# Tiger Code SHA1

基于虎码改良的中英混合简体输入方案

目前仅支持Weasel小狼毫，未来会支持全平台

## 特性（对比虎码秃版）

- **拼音反查**：通过`|`前缀触发拼音反查功能
- **中英混输**：支持中文和英文混合输入，无需切换输入法
- **英文词库**：集成英文词库，支持首字母大写
- **规范汉字**：默认仅收录《通用规范汉字表（2013）》中的标准简体字
- **中文补全**：使用前缀局部权重保持补全排序，并减少加词时的词典diff
- **英文候选**：使用Lua translator按词表顺序惰性产出英文候选，避免短前缀全量排序
- **自定义加词**：自定义加词可通过`src/main.py`自动sort并同步到输入方案中
- **特殊符号**：见`symbols.yaml`，切换到全角可输入一些常用特殊符号
- **唯一编码**：所有字符只保留唯一编码以降低心智负担
- **永不自动上屏**：即使唯一候选也要按空格上屏
- **永不自动清除buffer**：可以输入超过4码以实现更流畅的中英混输

## 文件结构

```text
├── tiger_sha1_weasel.schema.yaml	# 主输入方案
├── tiger_sha1.dict.yaml			# 主词典
├── tiger_sha1_en.dict.yaml			# 英文词典
├── alphabet.dict.yaml				# 大写字母表
├── tiger_sha1_py.schema.yaml		# 拼音反查伪方案
├── tiger_sha1_py.dict.yaml			# 拼音反查词典
├── symbols.yaml					# 符号表
├── weasel.custom.yaml				# 小狼毫界面定制
├── lua/
│	├── en_weight_translate.lua		# 英文候选按词表顺序惰性产出
│	└── hide_en_comment.lua			# 隐藏英文补全建议
├── src/
│	├── main.py						# 更新dicts
│	└── utils/
│		├── tiger.py				# 虎码处理
│		├── en.py					# 英文处理
│		├── py_sc.py				# 拼音处理
│		├── sc2013.py				# 规范汉字处理
│		└── add.py					# 附加词条处理
└── upstream/
	├── tiger/						# 虎码原始数据
	├── SC2013/						# 通用规范汉字表
	├── ESDB.txt					# 英文拼写数据库
	└── add.tsv						# 附加词条
```

## 使用方法

1. 安装Weasel小狼毫和Noto Sans SC字体
2. 将本仓库内容复制或`git clone`（建议浅clone）到Rime用户数据目录`%APPDATA%\Rime`
3. 重新部署Rime

若要加减词，请编辑`upstream/add.tsv`，然后执行`uv run src/main.py`以更新词典。不需要手动整理`upstream/add.tsv`，脚本会自动处理

可用参数：
- `--deploy`：更新词典后自动重新部署Weasel
- `--sync`：更新词典后自动执行`git add .`、`git commit`、`git push`以同步到上游（仅在main分支时触发push，其他分支仅commit）

词典生成工具会：
1. 从虎码原始数据提取规范汉字编码
2. 生成拼音反查词典
3. 集成高频英文单词
4. 为中文词条生成前缀局部权重，英文词条保持词表顺序

## 致谢

- [虎码输入法](https://www.tiger-code.com) - 原始编码方案
- [Rime Weasel](https://rime.im) - 输入引擎
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table) - 数据集
- [English Speller Database](https://wordlist.aspell.net) - 英语单词数据库
- [wordfreq](https://github.com/rspeer/wordfreq) - 英语词频数据库
