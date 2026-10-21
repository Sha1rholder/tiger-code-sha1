//! 生成应用重编码后的虎码字符表[`TIGER_CHARS_DICT`]
//!
//! 读取虎码原始数据，按字符集过滤并选择编码节点，再应用重编码与按权重填充候选

use crate::prelude::{insert_code, parse_target};
use crate::zh_chars::ZH_CHARS;
use crate::{CodeNode, NodeId, Text, Weight};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 单个字符对应的多个编码节点及其权重
type TigerNodes = HashMap<NodeId, Weight>;
/// 虎码原始数据
type TigerEntries = HashMap<Text, TigerNodes>;
/// 已选编码节点及其优先级
type TigerChar = (NodeId, Weight);
/// 已筛选虎码
type TigerCharsRaw = HashMap<Text, TigerChar>;

/// 虎码字符节点，0号节点固定为根
pub(crate) static TIGER_CHARS_DICT: LazyLock<Vec<CodeNode>> = LazyLock::new(|| {
	build_tiger_chars(
		parse_target("data/zh/tiger-code/tiger.dict.yaml.tsv"),
		parse_target("data/zh/recode.tsv"),
		&ZH_CHARS,
	)
});

/// 在同一组节点中完成单字选码、重编码和候选排序
fn build_tiger_chars(
	lines: impl IntoIterator<Item = String>,
	recodes: impl IntoIterator<Item = String>,
	zh_chars: &HashSet<Text>,
) -> Vec<CodeNode> {
	let path = "data/zh/tiger-code/tiger.dict.yaml.tsv";
	let mut lines = lines.into_iter();
	// 校验并跳过TSV标题
	assert_eq!(
		lines.next().as_deref(),
		Some("text\tcode\tweight"),
		"invalid tiger character header in {path}"
	);
	let mut nodes = vec![CodeNode::root()];
	// 读取原始虎码数据，去掉非法字符
	let mut tiger_raw = TigerEntries::new();

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
		let node = insert_code(&mut nodes, code);

		let previous = tiger_raw
			.entry(Text::from(text.to_owned()))
			.or_default()
			.insert(node, Weight::from(weight));
		assert!(
			previous.is_none(),
			"duplicate tiger code and text in {path}: {code}\t{text}"
		);
	}

	// 丢弃ZH_CHARS外的字符
	tiger_raw.retain(|text, _| zh_chars.contains(text));

	// 生成有序的单字编码表
	// 显式按Weight降序处理，消除对HashMap迭代顺序的依赖
	let mut weighted_entries = tiger_raw
		.into_iter()
		.flat_map(|(text, candidates)| {
			candidates
				.into_iter()
				.map(move |(node, weight)| (text.clone(), node, weight))
		})
		.collect::<Vec<_>>();
	weighted_entries.sort_unstable_by(
		|(left_text, left_node, left_weight), (right_text, right_node, right_weight)| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
				.then_with(|| nodes[*left_node].cmp_path(&nodes[*right_node], &nodes))
		},
	);

	let mut tiger_chars = TigerCharsRaw::new();
	// 仅统计已选单字的占用，路径节点存在不代表该编码已被占用
	let mut current_nodes = HashMap::<NodeId, usize>::new();

	for (text, node, weight) in weighted_entries {
		// 新Text按遍历顺序直接插入
		if !tiger_chars.contains_key(&text) {
			*current_nodes.entry(node).or_default() += 1;
			let _ = tiger_chars.insert(text, (node, weight));
			continue;
		}

		let current_node = tiger_chars
			.get(&text)
			.expect("existing text must have a node")
			.0;
		// 仅接受更短且当前未占用的编码节点
		if nodes[node].depth(&nodes) >= nodes[current_node].depth(&nodes)
			|| current_nodes.contains_key(&node)
		{
			continue;
		}

		// 释放旧节点的当前使用次数，不删除节点或改变父子关系
		let remove_old_node = {
			let count = current_nodes
				.get_mut(&current_node)
				.expect("current node must have a count");
			*count -= 1;
			*count == 0
		};
		if remove_old_node {
			let _ = current_nodes.remove(&current_node);
		}
		// 登记新节点，保留该字最初选定的优先级
		let previous = current_nodes.insert(node, 1);
		debug_assert!(previous.is_none());
		tiger_chars
			.get_mut(&text)
			.expect("existing text must have a node")
			.0 = node;
	}

	{
		// 断言字符集合完整覆盖ZH_CHARS
		let tiger_keys = tiger_chars.keys().collect::<HashSet<_>>();
		let zh_chars_keys = zh_chars.iter().collect::<HashSet<_>>();
		assert_eq!(
			tiger_keys, zh_chars_keys,
			"tiger raw keys must equal ZH_CHARS"
		);
	}
	// 将重编码解析到同一组节点
	let tiger_recode = recodes
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Text::from(text.to_owned()), insert_code(&mut nodes, code))
		})
		// 已从字符表移除的字不再应用重编码
		.filter(|(text, _)| zh_chars.contains(text))
		.collect::<HashMap<_, _>>();
	// 应用重编码
	// 重编码键必须来自原始字符表
	assert!(
		tiger_recode
			.keys()
			.all(|text| tiger_chars.contains_key(text)),
		"tiger recode keys must be a subset of tiger raw keys"
	);

	// 只替换字与节点的关联，保留原始优先级与所有父子关系
	for (text, node) in tiger_recode {
		tiger_chars
			.get_mut(&text)
			.expect("validated recode text must exist")
			.0 = node;
	}

	// 按原始Weight降序填充节点候选，同权重时按Text排序
	let mut entries = tiger_chars.into_iter().collect::<Vec<_>>();
	entries.sort_unstable_by(
		|(left_text, (_, left_weight)), (right_text, (_, right_weight))| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
		},
	);

	for (text, (node, _)) in entries {
		nodes[node].texts_mut().push(text);
	}
	nodes
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::prelude::code_nodes_to_groups;
	use std::collections::BTreeMap;

	fn build(
		entries: &[(&str, &str, usize)],
		recodes: &[(&str, &str)],
		characters: &str,
	) -> BTreeMap<String, Vec<String>> {
		let lines = std::iter::once("text\tcode\tweight".to_owned()).chain(
			entries
				.iter()
				.map(|(text, code, weight)| format!("{text}\t{code}\t{weight}")),
		);
		let recodes = std::iter::once("code\ttext".to_owned())
			.chain(recodes.iter().map(|(code, text)| format!("{code}\t{text}")));
		let characters = characters
			.chars()
			.map(|character| Text::from(character.to_string()))
			.collect();
		let nodes = build_tiger_chars(lines, recodes, &characters);
		code_nodes_to_groups(&nodes)
			.into_iter()
			.map(|(code, texts)| {
				(
					code.as_str().to_owned(),
					texts
						.into_iter()
						.map(|text| text.as_str().to_owned())
						.collect(),
				)
			})
			.collect()
	}

	#[test]
	fn shorter_codes_release_only_the_last_occupant() {
		let dictionary = build(
			&[
				("甲", "ab", 100),
				("乙", "ab", 90),
				("丙", "abc", 80),
				("丁", "abcd", 70),
				("甲", "a", 60),
				("丙", "ab", 50),
				("乙", "b", 40),
				("丁", "ab", 30),
			],
			&[],
			"甲乙丙丁",
		);
		assert_eq!(dictionary.len(), 4);
		assert_eq!(dictionary["a"], ["甲"]);
		assert_eq!(dictionary["b"], ["乙"]);
		assert_eq!(dictionary["abc"], ["丙"]);
		assert_eq!(dictionary["ab"], ["丁"]);
	}

	#[test]
	fn recoding_preserves_descendants_and_original_priority() {
		let dictionary = build(
			&[
				("甲", "ab", 100),
				("甲", "a", 10),
				("乙", "abc", 90),
				("丙", "z", 80),
				("丁", "z", 80),
				("戊", "zzz", 70),
				("外", "q", 110),
			],
			&[("z", "甲"), ("I'n", "戊"), ("q", "外")],
			"甲乙丙丁戊",
		);
		assert_eq!(dictionary.len(), 3);
		assert_eq!(dictionary["z"], ["甲", "丁", "丙"]);
		assert_eq!(dictionary["abc"], ["乙"]);
		assert_eq!(dictionary["I'n"], ["戊"]);
	}

	#[test]
	fn equal_weights_choose_by_path_instead_of_allocation_order() {
		let mut entries = [("甲", "zzz", 100), ("甲", "abd", 100), ("甲", "abc", 100)];
		let dictionary = build(&entries, &[], "甲");
		assert_eq!(dictionary.len(), 1);
		assert_eq!(dictionary["abc"], ["甲"]);

		entries.reverse();
		assert_eq!(build(&entries, &[], "甲"), dictionary);
	}

	#[test]
	#[should_panic(expected = "duplicate tiger code and text")]
	fn duplicate_text_and_path_are_rejected() {
		let _ = build(&[("甲", "ab", 100), ("甲", "ab", 90)], &[], "甲");
	}
}
