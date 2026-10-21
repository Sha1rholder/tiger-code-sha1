//! 生成中英混合词典[`ZH_EN_DICT`]
//!
//! 合并[`ZH_WORDS_DICT`]与[`EN_DICT`]的节点，英文独有编码以编码本身为首选候选
//! 完成合并后再转换为现有输出格式

use crate::en_dict::EN_DICT;
use crate::prelude::{code_nodes_to_groups, insert_child};
use crate::zh_dict::ZH_WORDS_DICT;
use crate::{Code, CodeNode, Text};
use std::collections::HashMap;
use std::sync::LazyLock;

/// 按编码分组的中英混合词典
pub static ZH_EN_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	let mut nodes = (*ZH_WORDS_DICT).clone();
	merge_english(&mut nodes, &EN_DICT);

	code_nodes_to_groups(&nodes)
});

/// 按英文节点的添加顺序合并路径，中文候选优先，已有节点id保持稳定
fn merge_english(nodes: &mut Vec<CodeNode>, english: &[CodeNode]) {
	// 两棵树的id互相独立，记录英文节点在合并树中的对应位置
	let mut targets = vec![0; english.len()];
	for (source, entry) in english.iter().enumerate().skip(1) {
		let parent = entry.parent.expect("nonroot node must have a parent");
		let character = english[parent].children[&source];
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
					.path(english)
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
			texts.extend_from_slice(entry.texts());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::prelude::insert_code;

	fn add_texts(nodes: &mut Vec<CodeNode>, code: &str, texts: &[&str]) -> usize {
		let node = insert_code(nodes, code);
		nodes[node]
			.texts_mut()
			.extend(texts.iter().map(|text| Text::from((*text).to_owned())));
		node
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
		merge_english(&mut nodes, &english);

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
		merge_english(&mut nodes, &english);

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
