//! 生成原始中文候选表[`ZH_WORDS_DICT`]
//!
//! 读取虎码字符表、自定义条目与中文词频，经补全与优先级合并后按编码分组

use crate::prelude::parse_target;
use crate::zh_chars::{ZH_CHARS, contains_illegal_zh_char};
use crate::{Code, Freq, Text, Weight};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 原词
type OriginalText = Text;
/// 首字
type FirstText = Text;
/// 补全部
type CompletionText = Text;
/// 补全加权词频
type CompletionWeight = Freq;
/// 单个字符的多个虎码及其权重
type TigerCodes = HashMap<Code, Weight>;
/// 虎码原始数据
type TigerEntries = HashMap<Text, TigerCodes>;
/// 已选虎码及其优先级
type TigerChar = (Code, Weight);
/// 已筛选虎码
type TigerCharsRaw = HashMap<Text, TigerChar>;

/// 补全候选项数量
const CANDIDATES: usize = 3;
/// 最大补全候选项数量
const MAX_CANDIDATES: usize = 9;

/// 读取虎码原始数据，以Text和Code为键收集并拒绝重复组合，丢弃非法字符
fn build_tiger_entries(
	lines: impl IntoIterator<Item = String>,
	path: &str,
	zh_chars: &HashSet<Text>,
) -> TigerEntries {
	let mut entries = TigerEntries::new();

	for line in lines {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let code = fields.next().expect("missing code field");
		let weight = fields
			.next()
			.expect("missing weight field")
			.parse()
			.expect("invalid weight field");
		assert!(fields.next().is_none(), "extra fields in {path}: {line}");

		let previous = entries
			.entry(Text::from(text.to_owned()))
			.or_default()
			.insert(Code::from(code.to_owned()), Weight::from(weight));
		assert!(
			previous.is_none(),
			"duplicate tiger code and text in {path}: {code}\t{text}"
		);
	}

	// 丢弃ZH_CHARS外的字符
	entries.retain(|text, _| zh_chars.contains(text));

	entries
}

/// 按Weight选择每个字符使用的虎码
fn build_tiger_chars(tiger: TigerEntries) -> TigerCharsRaw {
	// 显式按Weight降序处理，消除对HashMap迭代顺序的依赖
	let mut weighted_entries = tiger
		.into_iter()
		.flat_map(|(text, codes)| {
			codes
				.into_iter()
				.map(move |(code, weight)| (text.clone(), code, weight))
		})
		.collect::<Vec<_>>();
	weighted_entries.sort_unstable_by(
		|(left_text, left_code, left_weight), (right_text, right_code, right_weight)| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
				.then_with(|| left_code.cmp(right_code))
		},
	);

	let mut tiger_chars_raw = TigerCharsRaw::new();
	// 记录当前HashMap中每个Code的使用次数
	let mut current_codes = HashMap::<Code, usize>::new();

	for (text, code, weight) in weighted_entries {
		// 新Text按遍历顺序直接插入
		if !tiger_chars_raw.contains_key(&text) {
			*current_codes.entry(code.clone()).or_default() += 1;
			let _ = tiger_chars_raw.insert(text, (code, weight));
			continue;
		}

		let current_code = tiger_chars_raw
			.get(&text)
			.expect("existing text must have a code")
			.0
			.clone();
		// 仅接受更短且当前未占用的Code
		if code.len() >= current_code.len() || current_codes.contains_key(&code) {
			continue;
		}

		// 释放旧Code的当前使用次数
		let old_code = current_code;
		let remove_old_code = {
			let count = current_codes
				.get_mut(&old_code)
				.expect("current code must have a count");
			*count -= 1;
			*count == 0
		};
		if remove_old_code {
			let _ = current_codes.remove(&old_code);
		}
		// 登记并写入新Code
		let previous = current_codes.insert(code.clone(), 1);
		debug_assert!(previous.is_none());
		tiger_chars_raw
			.get_mut(&text)
			.expect("existing text must have a code")
			.0 = code;
	}

	tiger_chars_raw
}

/// 断言重编码键有效并替换字符编码
fn apply_tiger_recode(tiger_chars_raw: &mut TigerCharsRaw, tiger_recode: HashMap<Text, Code>) {
	// 重编码键必须来自原始字符表
	assert!(
		tiger_recode
			.keys()
			.all(|text| tiger_chars_raw.contains_key(text)),
		"tiger recode keys must be a subset of tiger raw keys"
	);

	// 原位替换字符编码，保留原始优先级
	for (text, code) in tiger_recode {
		tiger_chars_raw
			.get_mut(&text)
			.expect("validated recode text must exist")
			.0 = code;
	}
}

/// 按编码分组文本并保留输入中的相对顺序
fn group_texts_by_code(
	entries: impl IntoIterator<Item = (Code, Text)>,
) -> HashMap<Code, Vec<Text>> {
	let mut grouped = HashMap::<Code, Vec<Text>>::new();

	// 同Code的Text按输入顺序追加
	for (code, text) in entries {
		grouped.entry(code).or_default().push(text);
	}

	grouped
}

/// 按优先级降序分组已筛选虎码
fn group_tiger_chars_by_code(tiger_chars: TigerCharsRaw) -> HashMap<Code, Vec<Text>> {
	let mut entries = tiger_chars.into_iter().collect::<Vec<_>>();
	entries.sort_unstable_by(
		|(left_text, (_, left_weight)), (right_text, (_, right_weight))| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
		},
	);

	group_texts_by_code(entries.into_iter().map(|(text, (code, _))| (code, text)))
}

/// 虎码字符表
static TIGER_CHARS_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	let path = "data/zh/tiger-code/tiger.dict.yaml.tsv";
	let mut lines = parse_target(path).into_iter();
	// 校验并跳过TSV标题
	assert_eq!(
		lines.next().as_deref(),
		Some("text\tcode\tweight"),
		"invalid tiger character header in {path}"
	);
	// 读取原始虎码数据，去掉非法字符
	let tiger_raw = build_tiger_entries(lines, path, &ZH_CHARS);
	// 生成有序的单字编码表
	let mut tiger_chars = build_tiger_chars(tiger_raw);
	{
		// 断言字符集合完整覆盖ZH_CHARS
		let tiger_keys = tiger_chars.keys().collect::<HashSet<_>>();
		let zh_chars_keys = ZH_CHARS.iter().collect::<HashSet<_>>();
		assert_eq!(
			tiger_keys, zh_chars_keys,
			"tiger raw keys must equal ZH_CHARS"
		);
	}
	// 读取Text到Code的重编码表
	let tiger_recode = parse_target("data/zh/recode.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Text::from(text.to_owned()), Code::from(code.to_owned()))
		})
		// 已从字符表移除的字不再应用重编码
		.filter(|(text, _)| ZH_CHARS.contains(text))
		.collect::<HashMap<_, _>>();
	// 应用重编码
	apply_tiger_recode(&mut tiger_chars, tiger_recode);

	// 按Code分组，组内按原始Weight降序排列
	group_tiger_chars_by_code(tiger_chars)
});

/// 自定义编码
static ZH_CUSTOM_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 读取自定义Code和Text
	let entries = parse_target("data/zh/custom.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Code::from(code.to_owned()), Text::from(text.to_owned()))
		});

	// 按Code分组并保留Text原顺序
	group_texts_by_code(entries)
});

/// 返回文本是否包含键集合中的其它文本
fn contains_other_text(text: &str, texts: &HashSet<&str>) -> bool {
	// 逐字符选择子串起点
	for (start, _) in text.char_indices() {
		let suffix = &text[start..];

		// 枚举该起点后的所有字符边界
		for (relative_end, _) in suffix
			.char_indices()
			.skip(1)
			.chain(std::iter::once((suffix.len(), '\0')))
		{
			let end = start + relative_end;
			// 跳过文本自身
			if start == 0 && end == text.len() {
				continue;
			}
			// 命中任意其它键后立即返回
			if texts.contains(&text[start..end]) {
				return true;
			}
		}
	}

	false
}

/// 合法中文wordfreq
static ZH_FREQ: LazyLock<HashMap<Text, Freq>> = LazyLock::new(|| {
	let path = "data/zh/wordfreq.tsv";
	let mut seen = HashSet::new();
	let mut frequencies = HashMap::new();

	// 读取并过滤中文词频
	for line in parse_target(path).into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let freq = fields.next().expect("missing freq field");
		let freq = Freq::from(freq.parse().expect("invalid freq field"));
		let parsed_text = text.to_owned();

		// 拒绝重复Text
		if !seen.insert(parsed_text.clone()) {
			panic!("duplicate text in {path}: {text}");
		}
		// 丢弃单字和包含非法字符的词
		if text.chars().count() == 1 || contains_illegal_zh_char(text, &ZH_CHARS) {
			continue;
		}

		let _ = frequencies.insert(parsed_text, freq);
	}

	// 临时借用全部合法键
	let texts = frequencies
		.keys()
		.map(String::as_str)
		.collect::<HashSet<_>>();
	// 找出包含任意其它键的键
	let contained_texts = texts
		.iter()
		.copied()
		.filter(|text| contains_other_text(text, &texts))
		.map(str::to_owned)
		.collect::<Vec<_>>();
	// 释放键集合借用
	drop(texts);

	// 删除包含其它键的键值对
	for text in contained_texts {
		let _ = frequencies.remove(&text);
	}

	// 转换为公开的Text键
	frequencies
		.into_iter()
		.map(|(text, freq)| (Text::from(text), freq))
		.collect()
});

/// 反转虎码字符表并断言每个字符只有一个编码
fn build_char_codes(tiger_chars: &HashMap<Code, Vec<Text>>) -> HashMap<Text, Code> {
	let mut char_codes = HashMap::new();

	// 展开Code到Text的分组
	for (code, texts) in tiger_chars {
		for text in texts {
			let previous = char_codes.insert(text.clone(), code.clone());
			assert!(previous.is_none(), "tiger text must map to one code");
		}
	}

	char_codes
}

/// 将原词拆为首字和补全部
fn split_original_text(original: &OriginalText) -> (FirstText, CompletionText) {
	let mut chars = original.as_str().chars();
	let first = chars.next().expect("original text must not be empty");
	let completion = chars.collect::<String>();
	assert!(
		!completion.is_empty(),
		"original text must contain multiple characters"
	);

	(
		FirstText::from(first.to_string()),
		CompletionText::from(completion),
	)
}

/// 计算补全部对应的补全码长
fn completion_code_len(completion: &CompletionText, char_codes: &HashMap<Text, Code>) -> usize {
	// 每个字符编码后的空格也占一码
	completion
		.as_str()
		.chars()
		.map(|character| {
			let text = Text::from(character.to_string());
			char_codes
				.get(&text)
				.expect("completion text must have a code")
				.len() + 1
		})
		.sum()
}

/// 拼接首字和补全部
fn join_text(first: &FirstText, completion: &CompletionText) -> OriginalText {
	let mut original = String::with_capacity(first.as_str().len() + completion.as_str().len());
	original.push_str(first.as_str());
	original.push_str(completion.as_str());
	OriginalText::from(original)
}

/// 按补全加权词频生成每个编码的自动词
fn build_automatic_words(
	zh_freq: &HashMap<Text, Freq>,
	tiger_chars: &HashMap<Code, Vec<Text>>,
) -> HashMap<Code, Vec<Text>> {
	let char_codes = build_char_codes(tiger_chars);
	let mut completions = HashMap::<FirstText, HashMap<CompletionText, CompletionWeight>>::new();

	// 按首字收集补全部和补全加权词频
	for (original, freq) in zh_freq {
		let (first, completion) = split_original_text(original);
		let code_len = completion_code_len(&completion, &char_codes);
		let weight = freq.scale(code_len);
		let previous = completions
			.entry(first)
			.or_default()
			.insert(completion, weight);
		assert!(
			previous.is_none(),
			"completion text must be unique per first text"
		);
	}

	let mut weighted_words = HashMap::<Code, HashMap<OriginalText, CompletionWeight>>::new();

	// 将首字分组转换为Code分组
	for (first, completion_weights) in completions {
		let code = char_codes
			.get(&first)
			.expect("first text must have a code")
			.clone();

		for (completion, weight) in completion_weights {
			let original = join_text(&first, &completion);
			let previous = weighted_words
				.entry(code.clone())
				.or_default()
				.insert(original, weight);
			assert!(previous.is_none(), "original text must be unique per code");
		}
	}

	// 按补全加权词频不稳定降序
	weighted_words
		.into_iter()
		.map(|(code, words)| {
			let mut words = words.into_iter().collect::<Vec<_>>();
			words.sort_unstable_by(|(left_text, left), (right_text, right)| {
				right
					.total_cmp(left)
					.then_with(|| left_text.cmp(right_text))
			});
			let words = words.into_iter().map(|(text, _)| text).collect();
			(code, words)
		})
		.collect()
}

/// 合并字符和自定义词并用自动词补位
fn merge_zh_words(
	tiger_chars: &HashMap<Code, Vec<Text>>,
	zh_custom: &HashMap<Code, Vec<Text>>,
	automatic_words: HashMap<Code, Vec<Text>>,
) -> HashMap<Code, Vec<Text>> {
	let mut zh_words = HashMap::<Code, Vec<Text>>::new();

	// 先加入虎码字符
	for (code, texts) in tiger_chars {
		zh_words
			.entry(code.clone())
			.or_default()
			.extend(texts.iter().cloned());
	}
	// 再加入自定义词并保留重复
	for (code, texts) in zh_custom {
		zh_words
			.entry(code.clone())
			.or_default()
			.extend(texts.iter().cloned());
	}

	// 基础候选不能超过候选页容量
	assert!(
		zh_words.values().all(|texts| texts.len() <= MAX_CANDIDATES),
		"base Chinese candidates must not exceed MAX_CANDIDATES"
	);

	// 最后用自动词补足候选
	for (code, texts) in automatic_words {
		let words = zh_words.entry(code).or_default();
		let available = CANDIDATES.saturating_sub(words.len());
		words.extend(texts.into_iter().take(available));
	}

	zh_words
}

/// 中文候选词
pub static ZH_WORDS_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 生成按补全加权词频排序的自动词
	let automatic_words = build_automatic_words(&ZH_FREQ, &TIGER_CHARS_DICT);
	// 按优先级合并全部中文候选
	merge_zh_words(&TIGER_CHARS_DICT, &ZH_CUSTOM_DICT, automatic_words)
});
