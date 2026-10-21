//! 生成中文候选节点表[`ZH_WORDS_DICT`]
//!
//! 在虎码字符节点上生成自动词候选，读取自定义条目并按优先级追加

use crate::prelude::{insert_code, parse_target};
use crate::tiger_dict::TIGER_CHARS_DICT;
use crate::zh_chars::{ZH_CHARS, contains_illegal_zh_char};
use crate::{CodeNode, Freq, NodeId, Text};
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

/// 中文候选节点，保留虎码字符节点及其id，0号节点固定为根
pub static ZH_WORDS_DICT: LazyLock<Vec<CodeNode>> = LazyLock::new(|| {
	let mut nodes = (*TIGER_CHARS_DICT).clone();
	// 单字到最终节点的映射，已应用重编码
	let mut char_nodes = HashMap::new();

	// 反转虎码字符表并断言每个字符只有一个节点
	for (node_id, node) in nodes.iter().enumerate() {
		for text in node.texts() {
			let previous = char_nodes.insert(text.clone(), node_id);
			assert!(previous.is_none(), "tiger text must map to one node");
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
				let node = *char_nodes
					.get(&text)
					.expect("completion text must have a node");
				nodes[node].depth(&nodes) + 1
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

	let mut weighted_words = HashMap::<NodeId, HashMap<OriginalText, CompletionWeight>>::new();

	// 按首字所在节点合并自动词，同码单字共用候选名额
	for (first, completion_weights) in completions {
		let node = *char_nodes.get(&first).expect("first text must have a node");

		for (completion, weight) in completion_weights {
			// 拼接首字和补全部
			let mut original =
				String::with_capacity(first.as_str().len() + completion.as_str().len());
			original.push_str(first.as_str());
			original.push_str(completion.as_str());
			let original = OriginalText::from(original);
			let previous = weighted_words
				.entry(node)
				.or_default()
				.insert(original, weight);
			assert!(previous.is_none(), "original text must be unique per node");
		}
	}

	// 单字已在节点中，按文件顺序追加自定义词并保留重复
	for line in parse_target("data/zh/custom.tsv").into_iter().skip(1) {
		let mut fields = line.split('\t');
		let code = fields.next().expect("missing code field");
		let text = fields.next().expect("missing text field");
		let node = insert_code(&mut nodes, code);
		nodes[node].texts_mut().push(Text::from(text.to_owned()));
	}

	// 最后按补全加权词频降序补足候选，同权重时按Text排序
	for (node, words) in weighted_words {
		let mut words = words.into_iter().collect::<Vec<_>>();
		words.sort_unstable_by(|(left_text, left), (right_text, right)| {
			right
				.total_cmp(left)
				.then_with(|| left_text.cmp(right_text))
		});
		let texts = nodes[node].texts_mut();
		let available = CANDIDATES.saturating_sub(texts.len());
		texts.extend(words.into_iter().take(available).map(|(text, _)| text));
	}

	nodes
});

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn output_preserves_character_nodes_and_candidate_priority() {
		let mut nodes = (*ZH_WORDS_DICT).clone();
		assert!(nodes[0].texts().is_empty());
		assert_eq!(nodes[0].depth(&nodes), 0);
		assert!(nodes.len() >= TIGER_CHARS_DICT.len());

		// 单字节点的id、深度和候选前缀保持不变
		let mut prefixes = vec![Vec::new(); nodes.len()];
		for (id, original) in TIGER_CHARS_DICT.iter().enumerate() {
			assert_eq!(nodes[id].depth(&nodes), original.depth(&TIGER_CHARS_DICT));
			prefixes[id].extend_from_slice(original.texts());
		}

		// 自定义路径必须已经存在，候选紧接在单字后且保持文件顺序
		let node_count = nodes.len();
		for line in parse_target("data/zh/custom.tsv").into_iter().skip(1) {
			let (code, text) = line.split_once('\t').unwrap();
			let id = insert_code(&mut nodes, code);
			assert_eq!(nodes.len(), node_count);
			prefixes[id].push(Text::from(text.to_owned()));
		}

		for (id, (node, prefix)) in nodes.iter().zip(prefixes).enumerate() {
			assert!(node.texts().starts_with(&prefix), "candidate order at {id}");
			assert!(node.texts().len() <= CANDIDATES.max(prefix.len()));
			let characters = TIGER_CHARS_DICT
				.get(id)
				.map(CodeNode::texts)
				.unwrap_or_default();
			// 自动词只能追加在首字最终所在的节点，空心节点不会被提升或补字
			for word in &node.texts()[prefix.len()..] {
				assert!(word.as_str().chars().count() > 1);
				assert!(
					characters
						.iter()
						.any(|first| word.as_str().starts_with(first.as_str())),
					"automatic word at the wrong node: {word:?}"
				);
			}
		}
	}
}
