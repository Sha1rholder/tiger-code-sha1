//! 生成中英混合词典[`ZH_EN_DICT`]
//!
//! 合并[`ZH_WORDS_DICT`]与[`EN_DICT`]的节点，按9格窗口编排中英文候选

use crate::en_dict::{EN_DICT, EnglishDictionary};
use crate::prelude::insert_child;
use crate::zh_dict::ZH_WORDS_DICT;
use crate::{CodeNode, Text};
use std::sync::LazyLock;

/// 与Rime的menu/page_size保持一致
const PAGE_SIZE: usize = 9;
/// 每个窗口前3格优先取中文自动词
const CHINESE_SLOTS: usize = 3;

/// 中英混合词典节点，0号节点固定为根
pub static ZH_EN_DICT: LazyLock<Vec<CodeNode>> = LazyLock::new(|| {
	let mut nodes = ZH_WORDS_DICT.nodes.clone();
	merge_english(&mut nodes, &ZH_WORDS_DICT.fixed_counts, &EN_DICT);

	nodes
});

/// 按英文节点的添加顺序合并路径并编排候选，已有节点id保持稳定
fn merge_english(nodes: &mut Vec<CodeNode>, fixed_counts: &[usize], english: &EnglishDictionary) {
	// 两棵树的id互相独立，记录英文节点在合并树中的对应位置
	let mut targets = vec![0; english.nodes.len()];
	for (source, entry) in english.nodes.iter().enumerate().skip(1) {
		let parent = entry.parent.expect("nonroot node must have a parent");
		let character = english.nodes[parent].children[&source];
		let target = insert_child(nodes, targets[parent], character);
		targets[source] = target;
		if entry.texts().is_empty() {
			continue;
		}

		let texts = nodes[target].texts_mut();
		if texts.is_empty() {
			// 没有中文候选时才添加字面首选，已有的空心路径也属于这种情况
			let literal = Text::from(
				entry
					.path(&english.nodes)
					.into_iter()
					.map(char::from)
					.collect::<String>(),
			);
			texts.push(literal.clone());
			texts.extend(
				entry
					.texts()
					.iter()
					.filter(|text| **text != literal)
					.cloned(),
			);
		} else {
			let (fixed, automatic) = texts.split_at(fixed_counts[target]);
			let (prepend, remaining) = entry.texts().split_at(english.prepend_counts[source]);
			*texts = arrange_candidates(fixed, automatic, prepend, remaining);
		}
	}
}

/// 优先项也占窗口位置，队列耗尽后由另一队列补满余下格子
fn arrange_candidates(
	fixed: &[Text],
	automatic: &[Text],
	prepend: &[Text],
	english: &[Text],
) -> Vec<Text> {
	let mut texts =
		Vec::with_capacity(fixed.len() + automatic.len() + prepend.len() + english.len());
	texts.extend_from_slice(fixed);
	let mut automatic = automatic.iter();
	let mut english = english.iter();
	// 首屏先用自动词将中文补至3格，再连续输出全部prepend
	texts.extend(
		automatic
			.by_ref()
			.take(CHINESE_SLOTS.saturating_sub(texts.len()))
			.cloned(),
	);
	texts.extend_from_slice(prepend);
	loop {
		let next = if texts.len() % PAGE_SIZE < CHINESE_SLOTS {
			automatic.next().or_else(|| english.next())
		} else {
			english.next().or_else(|| automatic.next())
		};
		let Some(text) = next else { break };
		texts.push(text.clone());
	}
	texts
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::prelude::insert_code;

	fn candidates(prefix: char, count: usize) -> Vec<Text> {
		(0..count)
			.map(|index| Text::from(format!("{prefix}{index}")))
			.collect()
	}

	#[test]
	fn windows_match_examples_and_priority_overflow() {
		for (fixed, automatic, prepend, english, pages) in [
			(
				4,
				10,
				2,
				100,
				vec![
					"FFFFPPEEE",
					"ZZZEEEEEE",
					"ZZZEEEEEE",
					"ZZZEEEEEE",
					"ZEEEEEEEE",
					"EEEEEEEEE",
				],
			),
			(1, 3, 1, 100, vec!["FZZPEEEEE", "ZEEEEEEEE", "EEEEEEEEE"]),
			(
				1,
				30,
				1,
				3,
				vec!["FZZPEEEZZ", "ZZZZZZZZZ", "ZZZZZZZZZ", "ZZZZZZZZ"],
			),
			(
				10,
				6,
				2,
				10,
				vec!["FFFFFFFFF", "FPPEEEEEE", "ZZZEEEEZZ", "Z"],
			),
			(1, 6, 10, 10, vec!["FZZPPPPPP", "PPPPEEEEE", "ZZZEEEEEZ"]),
			(0, 0, 2, 10, vec!["PPEEEEEEE", "EEE"]),
			(1, 12, 0, 0, vec!["FZZZZZZZZ", "ZZZZ"]),
			(1, 0, 1, 10, vec!["FPEEEEEEE", "EEE"]),
			(1, 1, 1, 10, vec!["FZPEEEEEE", "EEEE"]),
			(0, 0, 0, 0, vec![]),
		] {
			let result = arrange_candidates(
				&candidates('F', fixed),
				&candidates('Z', automatic),
				&candidates('P', prepend),
				&candidates('E', english),
			);
			assert_eq!(result.len(), fixed + automatic + prepend + english);
			let actual = result
				.chunks(PAGE_SIZE)
				.take(pages.len())
				.map(|page| {
					page.iter()
						.map(|text| text.as_str().chars().next().unwrap())
						.collect::<String>()
				})
				.collect::<Vec<_>>();
			assert_eq!(
				actual, pages,
				"counts: {fixed}, {automatic}, {prepend}, {english}"
			);
		}
	}

	#[test]
	fn windows_preserve_every_queue_and_use_the_current_page_position() {
		// 遍历空队列、配额边界和跨页边界，检查完整输出而非只检查首屏
		for fixed_count in 0..=12 {
			for prepend_count in 0..=12 {
				for automatic_count in [0, 1, 2, 3, 4, 8, 9, 10, 20] {
					for english_count in [0, 1, 2, 3, 4, 8, 9, 10, 20] {
						let queues = [
							candidates('F', fixed_count),
							candidates('Z', automatic_count),
							candidates('P', prepend_count),
							candidates('E', english_count),
						];
						let result =
							arrange_candidates(&queues[0], &queues[1], &queues[2], &queues[3]);
						assert_eq!(result.len(), queues.iter().map(Vec::len).sum::<usize>());
						// 每个唯一候选恰好出现一次，各队列顺序保持不变
						for (prefix, expected) in ['F', 'Z', 'P', 'E'].into_iter().zip(&queues) {
							let actual = result
								.iter()
								.filter(|text| text.as_str().starts_with(prefix))
								.collect::<Vec<_>>();
							assert_eq!(actual, expected.iter().collect::<Vec<_>>());
						}
						let initial_automatic =
							automatic_count.min(CHINESE_SLOTS.saturating_sub(fixed_count));
						let prepend_start = fixed_count + initial_automatic;
						let mixed_start = prepend_start + prepend_count;
						assert_eq!(&result[..fixed_count], queues[0]);
						assert_eq!(
							&result[fixed_count..prepend_start],
							&queues[1][..initial_automatic]
						);
						assert_eq!(&result[prepend_start..mixed_start], queues[2]);
						let last_automatic = result
							.iter()
							.rposition(|text| text.as_str().starts_with('Z'));
						let last_english = result
							.iter()
							.rposition(|text| text.as_str().starts_with('E'));
						if let (Some(last_automatic), Some(last_english)) =
							(last_automatic, last_english)
						{
							// 两个队列均有剩余时，类别必须与最终输出位置对应的配额一致
							for (index, text) in
								result.iter().enumerate().skip(mixed_start).take_while(
									|(index, _)| *index <= last_automatic.min(last_english),
								) {
								let prefix = if index % PAGE_SIZE < CHINESE_SLOTS {
									'Z'
								} else {
									'E'
								};
								assert!(text.as_str().starts_with(prefix));
							}
						}
					}
				}
			}
		}
	}

	#[test]
	fn page_size_matches_rime_schema() {
		assert!(
			include_str!("../../tiger_code_sha1.schema.yaml")
				.lines()
				.any(|line| line.trim() == format!("page_size: {PAGE_SIZE}"))
		);
	}

	fn add_texts(nodes: &mut Vec<CodeNode>, code: &str, texts: &[&str]) -> usize {
		let node = insert_code(nodes, code);
		nodes[node]
			.texts_mut()
			.extend(texts.iter().map(|text| Text::from((*text).to_owned())));
		node
	}

	#[test]
	fn merging_uses_source_boundaries_and_preserves_chinese_only_nodes() {
		let mut nodes = vec![CodeNode::root()];
		let a = add_texts(
			&mut nodes,
			"a",
			&["甲", "手工", "自动一", "自动二", "自动三"],
		);
		let q = add_texts(
			&mut nodes,
			"q",
			&["乙", "乙词一", "乙词二", "乙词三", "乙词四"],
		);
		let fixed_counts = vec![0, 2, 1];
		let mut english = vec![CodeNode::root()];
		let source = add_texts(&mut english, "a", &["apre", "auto", "aappend"]);
		let z = add_texts(&mut english, "z", &["z", "zpre", "zoo"]);
		let mut prepend_counts = vec![0; english.len()];
		prepend_counts[source] = 1;
		prepend_counts[z] = 2;
		let english = EnglishDictionary {
			nodes: english,
			prepend_counts,
		};
		merge_english(&mut nodes, &fixed_counts, &english);
		assert_eq!(
			nodes[a]
				.texts()
				.iter()
				.map(Text::as_str)
				.collect::<Vec<_>>(),
			[
				"甲",
				"手工",
				"自动一",
				"apre",
				"auto",
				"aappend",
				"自动二",
				"自动三"
			]
		);
		assert_eq!(
			nodes[q]
				.texts()
				.iter()
				.map(Text::as_str)
				.collect::<Vec<_>>(),
			["乙", "乙词一", "乙词二", "乙词三", "乙词四"]
		);
		let z = insert_code(&mut nodes, "z");
		assert_eq!(
			nodes[z]
				.texts()
				.iter()
				.map(Text::as_str)
				.collect::<Vec<_>>(),
			["z", "zpre", "zoo"]
		);
	}

	#[test]
	fn merging_preserves_chinese_priority_and_handles_hollow_prefixes() {
		let mut nodes = vec![CodeNode::root()];
		let q = add_texts(&mut nodes, "q", &["中文独有"]);
		let a = add_texts(&mut nodes, "a", &["甲", "a"]);
		let ab = add_texts(&mut nodes, "a'b", &["乙"]);
		// a'只有路径没有候选，合并时应按英文独有编码处理
		let mut english = vec![CodeNode::root()];
		add_texts(&mut english, "a'b", &["a'bc", "a'b"]);
		add_texts(&mut english, "a'", &["a'b", "a'", "a'"]);
		add_texts(&mut english, "a", &["after", "a", "after"]);
		add_texts(&mut english, "z", &["zebra", "z", "z", "zebra"]);
		add_texts(&mut english, "A", &["Apple"]);
		add_texts(&mut english, "xyz", &["xyz"]);
		let fixed_counts = nodes
			.iter()
			.map(|node| node.texts().len())
			.collect::<Vec<_>>();
		let english = EnglishDictionary {
			prepend_counts: vec![0; english.len()],
			nodes: english,
		};
		merge_english(&mut nodes, &fixed_counts, &english);

		let count = nodes.len();
		assert_eq!(insert_code(&mut nodes, "q"), q);
		assert_eq!(insert_code(&mut nodes, "a"), a);
		assert_eq!(insert_code(&mut nodes, "a'b"), ab);
		for (code, expected) in [
			("q", vec!["中文独有"]),
			("a", vec!["甲", "a", "after", "a", "after"]),
			("a'", vec!["a'", "a'b"]),
			("a'b", vec!["乙", "a'bc", "a'b"]),
			("z", vec!["z", "zebra", "zebra"]),
			("A", vec!["A", "Apple"]),
			("x", vec![]),
			("xy", vec![]),
			("xyz", vec!["xyz"]),
		] {
			let node = insert_code(&mut nodes, code);
			assert_eq!(
				nodes[node]
					.texts()
					.iter()
					.map(Text::as_str)
					.collect::<Vec<_>>(),
				expected,
				"candidates at {code}"
			);
		}
		assert_eq!(nodes.len(), count);
		assert!(nodes[0].texts().is_empty());
		assert_eq!(nodes[0].parent, None);
	}

	#[test]
	fn merging_appends_nodes_in_source_order_and_remaps_parent_ids() {
		let mut nodes = vec![CodeNode::root()];
		add_texts(&mut nodes, "q", &["甲"]);
		let original_len = nodes.len();
		let mut english = vec![CodeNode::root()];
		add_texts(&mut english, "z'a", &["z'a"]);
		add_texts(&mut english, "A", &["A"]);
		let fixed_counts = nodes
			.iter()
			.map(|node| node.texts().len())
			.collect::<Vec<_>>();
		let english = EnglishDictionary {
			prepend_counts: vec![0; english.len()],
			nodes: english,
		};
		merge_english(&mut nodes, &fixed_counts, &english);

		let paths = nodes[original_len..]
			.iter()
			.map(|node| {
				node.path(&nodes)
					.into_iter()
					.map(char::from)
					.collect::<String>()
			})
			.collect::<Vec<_>>();
		assert_eq!(paths, ["z", "z'", "z'a", "A"]);
		let root_characters = nodes[0]
			.children
			.values()
			.copied()
			.map(char::from)
			.collect::<String>();
		assert_eq!(root_characters, "qzA");
		let mut incoming = vec![0; nodes.len()];
		for (parent, node) in nodes.iter().enumerate() {
			for (&child, &character) in &node.children {
				assert!(child > parent);
				assert_eq!(nodes[child].parent, Some(parent));
				assert_eq!(nodes[child].path(&nodes).last(), Some(&character));
				incoming[child] += 1;
			}
		}
		assert_eq!(incoming[0], 0);
		assert!(incoming[1..].iter().all(|&count| count == 1));
	}
}
