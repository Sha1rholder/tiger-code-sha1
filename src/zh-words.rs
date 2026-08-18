use crate::data::{TIGER_CHARS, ZH_CUSTOM, ZH_FREQ};
use crate::{Code, Freq, Text};
use std::collections::HashMap;
use std::sync::LazyLock;

type CompletionWeight = Freq; // 补全加权词频
type CompletionText = Text; // 补全部
type FirstText = Text; // 首字
type OriginalText = Text; // 原词

const CANDIDATES: usize = 5;
const MAX_CANDIDATES: usize = 9;

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
			words.sort_unstable_by(|(_, left), (_, right)| right.total_cmp(left));
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
pub static ZH_WORDS: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 生成按补全加权词频排序的自动词
	let automatic_words = build_automatic_words(&ZH_FREQ, &TIGER_CHARS);
	// 按优先级合并全部中文候选
	merge_zh_words(&TIGER_CHARS, &ZH_CUSTOM, automatic_words)
});

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;

	/// 构造测试文本
	fn text(value: &str) -> Text {
		Text::from(value.to_owned())
	}

	/// 构造测试编码
	fn code(value: &str) -> Code {
		Code::from(value.to_owned())
	}

	/// 构造测试词频
	fn freq(value: f64) -> Freq {
		Freq::from(value)
	}

	/// 验证设计文档中的补全码长示例
	#[test]
	fn completion_code_lengths_match_examples() {
		let char_codes = build_char_codes(&TIGER_CHARS);

		for (original, expected) in [("我们", 3), ("习近平", 6), ("为了", 2)] {
			let (_, completion) = split_original_text(&text(original));
			assert_eq!(completion_code_len(&completion, &char_codes), expected);
		}
	}

	/// 验证补全加权词频由词频和补全码长共同决定
	#[test]
	fn automatic_words_sort_by_completion_weight() {
		let tiger_chars = HashMap::from([
			(code("x"), vec![text("甲")]),
			(code("abcd"), vec![text("乙")]),
			(code("c"), vec![text("丙")]),
		]);
		let zh_freq = HashMap::from([(text("甲乙"), freq(1.0)), (text("甲丙"), freq(2.0))]);

		let automatic_words = build_automatic_words(&zh_freq, &tiger_chars);

		assert_eq!(
			automatic_words.get(&code("x")),
			Some(&vec![text("甲乙"), text("甲丙")])
		);
	}

	/// 验证基础候选优先并按剩余数量补充自动词
	#[test]
	fn merged_words_preserve_priority_and_duplicates() {
		let tiger_chars = HashMap::from([
			(code("x"), vec![text("甲"), text("乙")]),
			(code("y"), vec![text("一"), text("二"), text("三")]),
		]);
		let zh_custom = HashMap::from([
			(code("x"), vec![text("乙"), text("丙")]),
			(code("y"), vec![text("四"), text("五"), text("六")]),
		]);
		let automatic_words = HashMap::from([
			(code("x"), vec![text("丁"), text("戊"), text("己")]),
			(code("y"), vec![text("自动")]),
		]);

		let zh_words = merge_zh_words(&tiger_chars, &zh_custom, automatic_words);

		assert_eq!(
			zh_words.get(&code("x")),
			Some(&vec![
				text("甲"),
				text("乙"),
				text("乙"),
				text("丙"),
				text("丁"),
			])
		);
		assert_eq!(
			zh_words.get(&code("y")),
			Some(&vec![
				text("一"),
				text("二"),
				text("三"),
				text("四"),
				text("五"),
				text("六"),
			])
		);
	}

	/// 验证基础候选不能超过候选页容量
	#[test]
	#[should_panic(expected = "base Chinese candidates must not exceed MAX_CANDIDATES")]
	fn excessive_base_candidates_are_rejected() {
		let tiger_chars = HashMap::from([(
			code("x"),
			vec![text("一"), text("二"), text("三"), text("四"), text("五")],
		)]);
		let zh_custom = HashMap::from([(
			code("x"),
			vec![text("六"), text("七"), text("八"), text("九"), text("十")],
		)]);

		let _ = merge_zh_words(&tiger_chars, &zh_custom, HashMap::new());
	}

	/// 验证虎码字符不能映射到多个编码
	#[test]
	#[should_panic(expected = "tiger text must map to one code")]
	fn duplicate_tiger_text_is_rejected() {
		let tiger_chars =
			HashMap::from([(code("a"), vec![text("甲")]), (code("b"), vec![text("甲")])]);

		let _ = build_char_codes(&tiger_chars);
	}

	/// 验证补全部字符必须存在虎码
	#[test]
	#[should_panic(expected = "completion text must have a code")]
	fn missing_completion_code_is_rejected() {
		let tiger_chars = HashMap::from([(code("x"), vec![text("甲")])]);
		let zh_freq = HashMap::from([(text("甲乙"), freq(1.0))]);

		let _ = build_automatic_words(&zh_freq, &tiger_chars);
	}

	/// 验证真实中文候选表的键集合和候选上限
	#[test]
	fn real_zh_words_cover_all_source_codes() {
		let expected_codes = TIGER_CHARS
			.keys()
			.chain(ZH_CUSTOM.keys())
			.cloned()
			.collect::<HashSet<_>>();
		let actual_codes = ZH_WORDS.keys().cloned().collect::<HashSet<_>>();

		assert_eq!(actual_codes, expected_codes);
		assert!(ZH_WORDS.values().all(|texts| !texts.is_empty()));
		assert!(ZH_WORDS.values().all(|texts| texts.len() <= MAX_CANDIDATES));
	}
}
