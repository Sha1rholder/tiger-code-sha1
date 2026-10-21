//! 生成原始中文候选表[`ZH_WORDS_DICT`]
//!
//! 基于虎码字符表生成自动词候选，读取自定义条目并按优先级合并

use crate::prelude::{code_nodes_to_groups, group_texts_by_code, parse_target};
use crate::tiger_dict::TIGER_CHARS_DICT;
use crate::zh_chars::{ZH_CHARS, contains_illegal_zh_char};
use crate::{Code, Freq, Text};
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

/// 补全候选项数量
const CANDIDATES: usize = 3;

/// 中文候选词
pub static ZH_WORDS_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 临时兼容边界，词模块迁移到节点后删除
	let tiger_chars = code_nodes_to_groups(&TIGER_CHARS_DICT);
	// 单字到最终虎码的映射，已应用重编码
	let mut char_codes = HashMap::new();

	// 反转虎码字符表并断言每个字符只有一个编码
	for (code, texts) in &tiger_chars {
		for text in texts {
			let previous = char_codes.insert(text.clone(), code.clone());
			assert!(previous.is_none(), "tiger text must map to one code");
		}
	}

	// 首字到补全部及其加权词频的映射
	let mut completions = HashMap::<FirstText, HashMap<CompletionText, CompletionWeight>>::new();

	let frequencies: HashMap<Text, Freq> = {
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
			.filter(|text| {
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
			})
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
	};

	// 按首字收集补全部和补全加权词频
	for (original, freq) in frequencies {
		// 将原词拆为首字和补全部
		let mut chars = original.as_str().chars();
		let first = chars.next().expect("original text must not be empty");
		let completion = chars.collect::<String>();
		assert!(
			!completion.is_empty(),
			"original text must contain multiple characters"
		);

		let first = FirstText::from(first.to_string());
		let completion = CompletionText::from(completion);
		// 计算补全部对应的补全码长
		// 每个字符编码后的空格也占一码
		let code_len = completion
			.as_str()
			.chars()
			.map(|character| {
				let text = Text::from(character.to_string());
				char_codes
					.get(&text)
					.expect("completion text must have a code")
					.len() + 1
			})
			.sum();
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
			// 拼接首字和补全部
			let mut original =
				String::with_capacity(first.as_str().len() + completion.as_str().len());
			original.push_str(first.as_str());
			original.push_str(completion.as_str());
			let original = OriginalText::from(original);
			let previous = weighted_words
				.entry(code.clone())
				.or_default()
				.insert(original, weight);
			assert!(previous.is_none(), "original text must be unique per code");
		}
	}

	// 按补全加权词频不稳定降序
	let automatic_words = weighted_words
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
		.collect::<HashMap<Code, Vec<Text>>>();

	// 按优先级合并全部中文候选
	let mut zh_words = HashMap::<Code, Vec<Text>>::new();

	// 先加入虎码字符
	for (code, texts) in &tiger_chars {
		zh_words
			.entry(code.clone())
			.or_default()
			.extend(texts.iter().cloned());
	}
	let custom_words = {
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
	};

	// 再加入自定义词并保留重复
	for (code, texts) in custom_words {
		zh_words.entry(code).or_default().extend(texts);
	}

	// 最后用自动词补足候选
	for (code, texts) in automatic_words {
		let words = zh_words.entry(code).or_default();
		let available = CANDIDATES.saturating_sub(words.len());
		words.extend(texts.into_iter().take(available));
	}

	zh_words
});
