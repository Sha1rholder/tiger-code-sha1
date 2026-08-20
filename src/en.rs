use crate::data::parse_target;
use crate::{Freq, Text};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/*
todo
struct TextEn(Text);
struct TextLower(TextEn); // 单词的全小写形式
enum WordCase {
	apple,
	applE,
	Apple,
	ApplE,
	APPLE,
}
TextEn继承Text所有性质，构造时必须满足只含`abc...zABC...Z'`字符，且额外实现两个派生属性
1. wordcase: WordCase。忽略所有`'`和末尾的`'s`（如有），match
	- 首字母大写 -> match
		- 全大写 -> APPLE
		- 第二个字母后全小写 -> Apple
		- _ -> ApplE
	- _ -> match
		- 全小写 -> apple
		- _ -> applE
2. lower: TextLower
*/

type FreqSelf = Freq; // 原始词频
type FreqBoost = Freq; // 提权词频
// type 同一单词的大小写形式 = HashSet<TextEn>;
// type 码长表<T> = Vec<Option<T>>; // index = lower.len() - 1

const INFLECT_START: u8 = 4; // 作为基础词参与构图至少需要的码长
// const MIN_FREQ: float = 1e-7; // 允许进入英文词典的最低词频

// todo: 把EN_FIRST和EN_LAST改为构造`HashMap<WordCase, 同一单词的大小写形式>`
/// 英文前置加词
static EN_FIRST: LazyLock<Vec<Text>> = LazyLock::new(|| {
	let values: Vec<Text> = parse_target("src/data/en/first.txt")
		.into_iter()
		.map(Text::from)
		.collect();
	assert_eq!(
		values.len(),
		values.iter().collect::<HashSet<_>>().len(),
		"English first data must not contain duplicate entries"
	);
	values
});

/// 英文后置加词
static EN_LAST: LazyLock<Vec<Text>> = LazyLock::new(|| {
	let values: Vec<Text> = parse_target("src/data/en/last.txt")
		.into_iter()
		.map(Text::from)
		.collect();
	assert_eq!(
		values.len(),
		values.iter().collect::<HashSet<_>>().len(),
		"English last data must not contain duplicate entries"
	);
	values
});

// todo: 改为构造`HashMap<TextLower, 同一单词的大小写形式>`并断言ESDB不存在以`'`或`'S`结尾的词
/// ESDB
static ESDB: LazyLock<HashSet<Text>> = LazyLock::new(|| {
	let values: HashSet<Text> = parse_target("src/data/ESDB/ESDB.txt")
		.into_iter()
		.skip_while(|line| line != "---")
		.skip(1)
		.map(Text::from)
		.collect();
	assert!(
		values.iter().all(|text| text
			.as_str()
			.chars()
			.all(|character| character.is_ascii_alphabetic() || character == '\'')),
		"ESDB must contain only ASCII letters and apostrophes"
	);
	values
});

// todo: 改为构造`HashMap<TextLower, FreqSelf>>`
/// 英文wordfreq
static EN_FREQ: LazyLock<HashMap<Text, FreqSelf>> = LazyLock::new(|| {
	let path = "src/data/wordfreq/en.tsv";
	let mut frequencies = HashMap::new();

	// 读取英文词频并保留词条原样
	for line in parse_target(path).into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let freq = fields.next().expect("missing freq field");
		let freq = FreqSelf::from(freq.parse().expect("invalid freq field"));
		let text = Text::from(text.to_owned());

		assert!(
			text.as_str()
				.chars()
				.all(|character| !character.is_ascii_uppercase()),
			"English wordfreq results must not contain uppercase ASCII letters: {line}"
		);
		assert!(
			frequencies.insert(text, freq).is_none(),
			"duplicate text in {path}: {line}"
		);
	}

	frequencies
});

/*
todo: 实现后续处理步骤

扩增ESDB，同一单词的大小写形式中
if apple or Apple exists: add apple、Apple、APPLE
if ApplE or applE exists: add apple

排序 -> `码长表<HashMap<TextLower, 同一单词的大小写形式>>`
查EN_FREQ词频 -> `超级词频表（想不到什么好名字）: 码长表<HashMap<TextLower, (FreqSelf, 同一单词的大小写形式)>>`

从不短于INFLECT_START的词开始构造变体图：`&超级词频表[INFLECT_START后]`（别的取片段方式也行，这里只是示意） -> `码长表<HashMap<TextLower, (FreqSelf, HashMap<TextEn, HashSet<TextEn（直接基本形式或直接变体的集合）>>)>>`（这些数据结构方便构图时寻址，属性的内容本就存在冗余）

type DemotionCount = u8;
`&超级词频表[前INFLECT_START]` -> `码长表<HashMap<TextLower, HashMap<TextEn, (0, FreqBoost（等于它自己的FreqSelf）)>>>`若FreqBoost小于MIN_FREQ就丢弃对应TextEn键值对
`&超级词频表[INFLECT_START后]`，利用变体图计算FreqBoost和DemotionCount -> `码长表<HashMap<TextLower, HashMap<TextEn, (DomotionCount, FreqBoost)>>>`若FreqBoost小于MIN_FREQ就丢弃对应TextEn键值对
分类；排序 -> `HashMap<WordCase, Vec<TextEn>>`每个Vec内优先满足DemotionCount升序；对于相同DemotionCount，按FreqBoost降序；对于相同FreqBoost，按码长升序；对于相同码长，逐位按`abc...zABC...Z'`排序（具体实现方式不一定像描述）
加入自定义词：EN_FIRST加each Vec前面，EN_LAST加each Vec后面 -> `HashMap<WordCase, Vec<TextEn>>`
解开WordCase合并Vec -> `Vec<TextEn>`顺序：apple, applE, Apple, ApplE, APPLE
*/

/*
todo: 实现构建有向无环变体图的办法

# 英文单词变体规范

当词A可以通过一次或多次变体得到词B，则称A为B的基本形式，B为A的变体；当词A可以通过一次变体得到词B，则称A为B的直接基本形式，B为A的直接变体；当词A是词B的基本形式，且B的FreqSelf低于A的FreqSelf，则称B为A的低频变体，A为B的高频基本形式；一个词的FreqBoost是它和它的所有低频变体的FreqSelf之和，一个词的DemotionCount是它拥有的高频基本形式的数量

任何变体的码长大于原词的码长，因此任何词的变体不可能同时是其基本形式，因此变体关系必是有向无环图

## 直接变体（基于Text字面量，不做语义判断，大小写敏感）

### 末字母小写

允许的双写辅音字母：`bdgklmnprstz`

- 复数/第三人称：`s`、`es`、`-y+ies`、`-f+ves`
- 过去式：`d`、`ed`、`-y+ied`、`双写辅音+ed`
- 进行时：`ing`、`-e+ing`、`双写辅音+ing`
- 副词：`ly`、`-y+ily`
- 施事名词/工具名词/比较级/最高级：`er`、`est`、`-y+ier`、`-y+iest`、`双写辅音+er`、`双写辅音+est`
- 名词化：`ment`、`ness`、`-y+iness`
- 形容词：`able`、`-e+able`
- 所有格：`'`、`'s`
- 缩写：`'t`、`'re`

### 末字母大写

允许的双写辅音字母：`BDGKLMNPRSTZ`

- 复数/第三人称：`s`、`S`、`ES`、`-Y+IES`、`-F+VES`
- 过去式：`D`、`ED`、`-Y+IED`、`双写辅音+ED`
- 进行时：`ING`、`-E+ING`、`双写辅音+ING`
- 副词：`LY`、`-Y+ILY`
- 施事名词/工具名词/比较级/最高级：`ER`、`EST`、`-Y+IER`、`-Y+IEST`、`双写辅音+ER`、`双写辅音+EST`
- 名词化：`MENT`、`NESS`、`-Y+INESS`
- 形容词：`ABLE`、`-E+ABLE`
- 所有格：`'`、`'s`、`'S`
- 缩写：`'m`、`'M`、`'T`、`'RE`
*/
