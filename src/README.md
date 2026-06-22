# 词典生成实现说明

`src/main.py`负责所有项目文件读写和Rime/TSV/TXT格式解析，`src/utils/`中的模块只处理已经读入的结构化数据。所有输入读取都按行分割处理，兼容`CRLF`和`LF`

主流程执行顺序：

1. 读取`upstream/SC2013/level-1.txt`、`level-2.txt`、`level-3.txt`，交给`main.py`合并为`set[str]`
2. 读取`upstream/tiger/PY_c.dict.yaml`正文为`list[tuple[code, weight, text]]`，交给`utils/py_sc.py`过滤并按词频降序生成拼音反查词典
3. 读取`upstream/tiger/tiger.dict.yaml`正文为`list[tuple[code, text]]`，交给`utils/tiger.py`过滤、单一化编码并合并中文附加词
4. 读取`add/`中参与生成的`.zh.tsv`中文附加词典，逐文件交给`utils/tiger.py`检查重复`text`并排序，然后由`main.py`写回
5. 读取`upstream/ESDB.txt`为`set[Text]`，读取`add/`中参与生成的`.en.tsv`英文附加词典为`list[tuple[text, demotion_count]]`，逐文件交给`utils/en.py`排序并写回，再生成英文排序和ESDB大小写扩增词

## 模块职责

- `src/main.py`：读取源文件、解析格式、调用纯函数、写出生成文件、按需部署Weasel、按需同步git
- `src/utils/py_sc.py`：过滤拼音反查行并按`weight`降序输出`(code, text)`
- `src/utils/tiger.py`：过滤虎码单字、处理同字多码、整理中文附加词、按码长合并附加词
- `src/utils/en.py`：基于ESDB拼写集合和`wordfreq`生成英文基础词排序，计算ESDB大小写扩增、变体关系、提权词频和降权次数

## 中文处理

`main.py`按上游文件顺序读取`upstream/tiger/tiger.dict.yaml`正文，并丢弃第三列`weight`。该源文件必须保持上游`weight`自上而下递减，因为`utils/tiger.py`会把输入顺序视为权重顺序

`main.py`读取`upstream/SC2013/level-1.txt`、`level-2.txt`、`level-3.txt`并直接合并为规范汉字集合，随后传给`utils/py_sc.py`和`utils/tiger.py`过滤词条

`upstream/tiger/PY_c.dict.yaml`如果出现被折成两行的记录，`main.py`会把没有制表符的连续行拼回上一条拼音词条，再继续解析

`utils/tiger.py`只保留`text`在《通用规范汉字表（2013）》集合中的单字。同一个字出现多个编码时：

- 先接受上游更靠前的编码
- 后续若出现更短编码，且该短码未被已选中的更高权重条目占用，则替换为短码，并把该字移动到当前短码所在位置
- 若短码已经被已选中的更高权重条目占用，继续保留原编码
- 码长相同或后续编码更长时，继续保留原编码

中文附加词来自`add/`中参与生成的`.zh.tsv`文件，第一行固定为`code<TAB>text`。空行会被跳过，其余行必须严格为两列TSV。文件名以`-`或`.-`开头的词典会被生成逻辑忽略；以`.`开头的词典会被`.gitignore`忽略。`utils/tiger.py`会按以下规则稳定排序后由`main.py`逐文件写回：

1. `code`长度升序
2. `code.casefold()`升序
3. 相同排序键保留原始先后顺序

逐文件排序写回后，`main.py`会按文件名字符顺序拼接全部中文附加词，再按同一规则整体排序。若执行`uv run src/main.py --debug`，脚本会额外输出`temp/zh_dict.tsv`用于审查只包含简体中文单字的基础虎码词典，并输出`temp/add.tsv`用于审查合并后的中文附加词中间态

基础虎码和中文附加词按码长分层合并：每个码长组中先放基础虎码，再追加同码长附加词；1码、2码、3码各自成组，4码及以上归为4码组

`tiger_sha1_zh.dict.yaml`写出为`code<TAB>text`两列，排序由生成顺序决定，不再写入weight列

## 英文处理

英文附加词来自`add/`中参与生成的`.en.tsv`文件，第一行固定为`text<TAB>demotion_count`。空行会被跳过，其余行必须严格为两列TSV。`demotion_count`必须是非负整数，不限制最大值。文件名以`-`或`.-`开头的词典会被生成逻辑忽略。完全相同的重复词会输出警告。排序规则为：

1. `demotion_count`升序
2. 单词长度升序
3. `text.lower()`升序
4. 相同排序键保留原始先后顺序

逐文件排序写回后，`main.py`会按文件名字符顺序拼接全部英文附加词，再按同一规则整体排序。英文附加词会按`demotion_count`插入基础英文词对应降权分组的首部；若某个附加词降权次数在基础词中不存在，也按数值位置插入。若基础英文词典包含完全相同的词，主流程会跳过基础词典里的重复项，避免最终英文词典重复

基础英文候选来源是`upstream/ESDB.txt`拼写集合与`wordfreq`英语词频库的大小写不敏感交集。ESDB只作为无序拼写白名单，不参与排序。生成时会过滤掉：

- 含非ASCII英文字母字符的词
- 长度小于3的词
- 无法在`wordfreq`中匹配到词频的词

ESDB读入为`set[Text]`后扩增大小写形式：若词面全小写，则加入全大写和首字母大写形式；否则加入全大写和全小写形式。附加英文词不会参与扩增；基础排序完成后，主流程只保留长度至少为`MIN_WORD_LEN = 4`且未与附加词完全重复的词，再与附加词按降权次数合并

## 英文变体排序

当词A可以通过一次变体得到词B，则称A为B的直接基本形式，B为A的直接变体；当词A可以通过多次变体得到词B，则称A为B的间接基本形式，B为A的间接变体

每个词最多选择一个直接基本形式。多个候选同时存在时，按规则优先级、基本形式词频降序、基本形式词面升序确定唯一结果

目前考虑的后缀变体规则（基于文本字面量，不做语义判断，大小写敏感）：

- 复数/第三人称：`s`、`S`、`es`、`ES`、`-y+ies`、`-Y+IES`
- 过去式：`d`、`D`、`ed`、`ED`、`-y+ied`、`-Y+IED`、双写辅音形式
- 进行时：`ing`、`ING`、`-e+ing`、`-E+ING`、双写辅音形式
- 副词：`ly`、`LY`、`-y+ily`、`-Y+ILY`
- 施事名词/工具名词/比较级/最高级：`er`、`ER`、`est`、`EST`、`-y+ier`、`-Y+IER`、`-y+iest`、`-Y+IEST`、双写辅音形式
- 名词化：`ment`、`MENT`、`ness`、`NESS`、`-y+iness`、`-Y+INESS`
- 形容词：`able`、`ABLE`、`-e+able`、`-E+ABLE`

大小写不同的词不会互相视为基本形式或变体，例如`cat`、`Cat`、`CAT`是三个独立词；`cats`、`Cats`、`CATS`只有在各自的基本形式也存在时才分别关联到`cat`、`Cat`、`CAT`

一个词及其所有低频变体的词频之和为其提权词频。若某个祖先基本形式的词频高于当前词，当前词的降权次数加1

基础英文排序键为：

1. 降权次数升序
2. 提权词频降序
3. 原始`wordfreq`词频降序
4. `word.lower()`升序
5. `word`升序

## 英文基础词大小写重排

英文附加词和基础英文词按降权次数合并后，会按最终输出顺序拆成基础词、首字母大写词、前两字母大写词三组，再拼接写入`lua/en_dict.txt`：

- 基础词保留在最前
- 首字母大写词排在基础词之后
- 首字母和第二个字母都大写的词排在最后
- 每组内部保持合并后的原始相对顺序

`lua/en_dict.txt`是一行一词的纯文本文件，Lua translator按该文件顺序惰性产出英文候选。若执行`uv run src/main.py --debug`，脚本会额外输出`temp/en_dict.tsv`用于审查英文排序指标

## 输出文件

- `tiger_sha1_weasel.dict.yaml`：主方案词典壳，导入`alphabet`和`tiger_sha1_zh`
- `tiger_sha1_zh.dict.yaml`：中文基础词典
- `tiger_sha1_py.dict.yaml`：拼音反查词典
- `lua/en_dict.txt`：英文词典
- `temp/zh_dict.tsv`：只包含简体中文单字的基础虎码词典TSV
- `temp/add.tsv`：合并排序后的中文附加词TSV
- `temp/add.txt`：合并排序后的英文附加词
- `temp/en_dict.tsv`：保留完整排序指标的英文词典
