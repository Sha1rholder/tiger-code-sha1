# YAML词典生成工具

```text
./
├ data/					# 原始数据
│	├ get_wordfreq.py	# 导出wordfreq中英文词频
│	├ zh/				# 中文数据
│	└ en/				# 英文数据
└ src/
	├ main.rs
	├ prelude.rs		# 自定义类型、编码节点与通用读取函数
	├ zh_chars.rs		# 处理现行国标通用规范汉字表和自定义加字
	├ pinyin.rs			# 生成拼音反查节点，保留多音节空格和有序候选
	├ tiger_dict.rs		# 单字选码、候选排序、消除空心节点，最后重编码
	├ zh_dict.rs		# 在单字节点上追加自定义条目和自动词候选，输出中文节点表
	├ en_dict.rs		# 生成英文编码节点，按前缀保存有序候选
	├ zh_en_dict.rs		# 合并中英文节点
	└ lib.rs			# 按编码字典序遍历节点并输出YAML产物
```
