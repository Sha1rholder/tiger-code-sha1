"""
1. 创建`temp/`文件夹（若本就存在则静默跳过）
2. 合并`SC2013/`的3个字表为set `sc2013`
3. 筛拼音生成字表`py` (code, weight, text)
	- 从`upstream/PY_c.dict.yaml`提取字表`py`并去掉所有text不在`sc2013`中的行（text必须完全匹配，因此只会保留单字）
	- 以weight降序排列输出到`tiger_py.dict.yaml`
4. 筛虎码单字生成字表`tiger` (code, weight, text)
	- 从`upstream/tiger.dict.yaml`提取字表`tiger`并去掉所有text不在`sc2013`中的行
	- 去重。对于text相同的行，只保留靠上的（不要依赖weight）
	- 输出`temp/py.tsv`
5. 对`tiger`“分频”
	- 每个单字母code最靠上行的weight设为90000，然后对于每个code重码把weight设为89999、89998……以此类推
	- 每个双字母code最靠上行的weight设为9000，然后对于每个code重码把weight设为8999、8998……以此类推
	- 每个三字母code最靠上行的weight设为900，然后对于每个code重码把weight设为899、898……以此类推
	- 每个四字母code最靠上行的weight设为90，然后对于每个code重码把weight设为89、88……以此类推
	- 输出`temp/tiger.tsv`
6. format `custom/add.tsv`并生成字表`add`
	- 从`custom/add.tsv`提取字表`add`
	- 把`add`按code长度升序排列
	- 对于相同的code长度，按字母顺序（先a后z）排列
	- 对于相同的code，按weight降序排列
	- 对于相同的code和weight，打印错误告诉用户哪重了然后非零退出
	- 对于每组相同的code，使weight成为以9为最大值-1为公差的等差数列
	- 输出`temp/add.tsv`
	- 对于每个单字母code，weight加10000
	- 对于每个双字母code，weight加1000
	- 对于每个三字母code，weight加100
	- 对于每个四字母code，weight加10
	- 对于五字母及以上的code，weight不变
7. 合并`tiger`和`add`
	- 合并`tiger`和`add`生成字表`tiger_add`
	- 把`tiger_add`以weight降序排列
	- 输出`temp/tiger_add.tsv`
8. 制作字表`en`
	- `from wordfreq import get_frequency_dict`
	- 生成英语字表`en` (code, weight, text)
	- 输出`temp/en_raw.tsv`
	- ……
……
"""
