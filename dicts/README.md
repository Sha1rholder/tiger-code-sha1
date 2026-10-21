# YAML词典生成工具

```text
./
├ data/					# 原始数据
│	├ get_wordfreq.py	# 导出wordfreq中英文词频
│	├ zh/				# 中文数据
│	└ en/				# 英文数据
└ src/
	├ main.rs
	├ prelude.rs		# 自定义类型、编码节点、通用读取与临时编码转换函数
	├ zh_chars.rs		# 处理现行国标通用规范汉字表和自定义加字
	├ pinyin.rs			# 生成拼音反查候选表
	├ tiger_dict.rs		# 单字选码、候选排序、消除空心节点，最后重编码
	├ zh_dict.rs		# 在单字节点上追加自定义条目和自动词候选，输出中文节点表
	├ en_dict.rs		# 生成英文编码表
	├ zh_en_dict.rs		# 合并中英文编码表
	└ lib.rs			# 输出YAML产物
```
