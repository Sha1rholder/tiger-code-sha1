//! 生成应用重编码后的虎码字符表[`TIGER_CHARS_DICT`]
//!
//! 读取虎码原始数据，按字符集过滤并选码，排列单字、消除空心节点，最后应用重编码

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
	let path = "data/zh/tiger-code/tiger.dict.yaml.tsv";
	let mut lines = parse_target(path).into_iter();
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
	tiger_raw.retain(|text, _| ZH_CHARS.contains(text));

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
		let zh_chars_keys = ZH_CHARS.iter().collect::<HashSet<_>>();
		assert_eq!(
			tiger_keys, zh_chars_keys,
			"tiger raw keys must equal ZH_CHARS"
		);
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
	// 清理会重新编号，后续只保留与节点id无关的单字权重
	let weights = entries
		.iter()
		.map(|(text, (_, weight))| (text.clone(), *weight))
		.collect::<HashMap<_, _>>();

	for (text, (node, _)) in entries {
		nodes[node].texts_mut().push(text);
	}
	eliminate_hollow_nodes(&mut nodes);
	// 重编码最后执行，允许刻意留下新的空心节点
	// 从提升后的实际位置移动单字，保留优先级，不再清理重编码产生的空节点
	let locations = nodes
		.iter()
		.enumerate()
		.flat_map(|(node, entry)| entry.texts().iter().cloned().map(move |text| (text, node)))
		.collect::<HashMap<_, _>>();
	let recodes = parse_target("data/zh/recode.tsv")
		.into_iter()
		.skip(1)
		.filter_map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = Text::from(fields.next().expect("missing text field").to_owned());
			// 已从字符表移除的字不再应用重编码，也不为其新建路径
			if !weights.contains_key(&text) {
				return None;
			}
			Some((text, insert_code(&mut nodes, code)))
		})
		.collect::<HashMap<_, _>>();

	for (text, target) in recodes {
		let source = locations[&text];
		if source == target {
			continue;
		}
		let position = nodes[source]
			.texts()
			.iter()
			.position(|candidate| candidate == &text)
			.expect("recode text must exist at its current node");
		let text = nodes[source].texts_mut().remove(position);
		nodes[target].texts_mut().push(text);
	}

	// 重编码可能合并候选，继续沿用单字最初选定的权重和同权重排序
	for node in nodes.iter_mut().skip(1) {
		node.texts_mut().sort_unstable_by(|left, right| {
			weights[right]
				.cmp(&weights[left])
				.then_with(|| left.cmp(right))
		});
	}
	nodes
});

/// 按节点添加顺序消除空心节点，根节点不参与提升
///
/// 先删除未采用编码留下的空枝，再从最早子节点的路径上取首选
/// 清理结束后统一压缩Vec并更新父子id，调用方不得继续使用旧id
fn eliminate_hollow_nodes(nodes: &mut Vec<CodeNode>) {
	assert!(!nodes.is_empty(), "nodes must contain the root at index 0");
	let mut removed = vec![false; nodes.len()];

	// 父节点总是先创建，倒序清理可排除所有没有候选的子树
	for node in (1..nodes.len()).rev() {
		prune_empty_branch(nodes, node, &mut removed);
	}

	for node in 1..nodes.len() {
		if removed[node] || !nodes[node].texts().is_empty() {
			continue;
		}

		let mut highest = node;
		while let Some(parent) = nodes[highest].parent {
			if parent == 0 || !nodes[parent].texts().is_empty() {
				break;
			}
			highest = parent;
		}

		// 沿最早加入的子节点向下，遇到候选就取其首选
		let mut source = highest;
		while nodes[source].texts().is_empty() {
			source = *nodes[source]
				.children
				.first_key_value()
				.expect("live empty node must have a child with descendants")
				.0;
		}
		let text = nodes[source].texts_mut().remove(0);
		nodes[highest].texts_mut().push(text);

		// 仍有候选或子节点时保留，轮到该节点时继续补字
		prune_empty_branch(nodes, source, &mut removed);
	}

	compact_nodes(nodes, &removed);
}

/// 只删除无候选且无子节点的空枝，遇到根、候选或其他分支就停止
fn prune_empty_branch(nodes: &mut [CodeNode], mut node: NodeId, removed: &mut [bool]) {
	while node != 0
		&& !removed[node]
		&& nodes[node].texts().is_empty()
		&& nodes[node].children.is_empty()
	{
		let parent = nodes[node].parent.expect("nonroot node must have a parent");
		let _ = nodes[parent]
			.children
			.remove(&node)
			.expect("parent must contain its child");
		removed[node] = true;
		node = parent;
	}
}

/// 维持剩余节点的相对添加顺序，并同步重写全部父子引用
fn compact_nodes(nodes: &mut Vec<CodeNode>, removed: &[bool]) {
	let mut next = 0;
	let ids = removed
		.iter()
		.map(|&removed| {
			if removed {
				None
			} else {
				let id = next;
				next += 1;
				Some(id)
			}
		})
		.collect::<Vec<_>>();
	let mut previous = 0;
	nodes.retain(|_| {
		let keep = !removed[previous];
		previous += 1;
		keep
	});

	for node in nodes {
		node.parent = node
			.parent
			.map(|parent| ids[parent].expect("live node must have a live parent"));
		node.children = std::mem::take(&mut node.children)
			.into_iter()
			.map(|(child, character)| {
				(
					ids[child].expect("live child must not be removed"),
					character,
				)
			})
			.collect();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::CodeChar;
	use std::collections::BTreeMap;

	fn add_texts(nodes: &mut Vec<CodeNode>, code: &str, texts: &[&str]) {
		let node = insert_code(nodes, code);
		nodes[node]
			.texts_mut()
			.extend(texts.iter().map(|text| Text::from((*text).to_owned())));
	}

	fn groups(nodes: &[CodeNode]) -> BTreeMap<String, Vec<String>> {
		nodes
			.iter()
			.filter(|node| !node.texts().is_empty())
			.map(|node| {
				(
					node.path(nodes).into_iter().map(char::from).collect(),
					node.texts()
						.iter()
						.map(|text| text.as_str().to_owned())
						.collect(),
				)
			})
			.collect()
	}

	fn assert_filled_tree(nodes: &[CodeNode]) {
		assert_eq!(nodes[0].parent, None);
		assert_eq!(nodes[0].depth(nodes), 0);
		assert!(nodes[0].texts().is_empty());
		let mut incoming = vec![0; nodes.len()];
		for (id, node) in nodes.iter().enumerate() {
			if id != 0 {
				assert!(!node.texts().is_empty(), "hollow node: {id}");
				assert!(node.parent.unwrap() < id);
			}
			let mut characters = std::collections::HashSet::new();
			for (&child, &character) in &node.children {
				assert!(characters.insert(character));
				assert_eq!(nodes[child].parent, Some(id));
				assert_eq!(nodes[child].path(nodes).last(), Some(&character));
				incoming[child] += 1;
			}
		}
		assert_eq!(incoming[0], 0);
		assert!(incoming[1..].iter().all(|&count| count == 1));
	}

	#[test]
	fn hollow_nodes_take_the_first_text_from_the_first_added_child() {
		let mut nodes = vec![CodeNode::root()];
		add_texts(&mut nodes, "xzb", &["丁"]);
		add_texts(&mut nodes, "xz", &["甲", "乙"]);
		add_texts(&mut nodes, "xaa", &["丙"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 4);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xz"], ["乙"]);
		assert_eq!(dictionary["xzb"], ["丁"]);
		assert_eq!(dictionary["xa"], ["丙"]);
		assert_filled_tree(&nodes);

		// 压缩后最先创建的z分支仍在a分支之前
		let count = nodes.len();
		let x = insert_code(&mut nodes, "x");
		let children = nodes[x].children.values().copied().collect::<Vec<_>>();
		assert_eq!(children, [CodeChar::LowerZ, CodeChar::LowerA]);
		for code in dictionary.keys() {
			let _ = insert_code(&mut nodes, code);
		}
		assert_eq!(nodes.len(), count);
	}

	#[test]
	fn promotion_preserves_a_donor_that_still_has_descendants() {
		let mut nodes = vec![CodeNode::root()];
		add_texts(&mut nodes, "xab", &["乙"]);
		add_texts(&mut nodes, "xa", &["甲"]);
		add_texts(&mut nodes, "xac", &["丙"]);
		add_texts(&mut nodes, "xz", &["丁"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 4);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xa"], ["乙"]);
		assert_eq!(dictionary["xac"], ["丙"]);
		assert_eq!(dictionary["xz"], ["丁"]);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn promotion_prunes_empty_chains_but_keeps_sibling_branches() {
		let mut nodes = vec![CodeNode::root()];
		// 原始选码可能留下完全没有候选的路径
		let _ = insert_code(&mut nodes, "unused");
		add_texts(&mut nodes, "xab", &["甲"]);
		add_texts(&mut nodes, "xac", &["乙"]);
		add_texts(&mut nodes, "xz", &["丙"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 3);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xa"], ["乙"]);
		assert_eq!(dictionary["xz"], ["丙"]);
		assert_eq!(nodes.len(), 4);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn an_unused_tree_keeps_only_its_root() {
		let mut nodes = vec![CodeNode::root()];
		let _ = insert_code(&mut nodes, "abc");
		let _ = insert_code(&mut nodes, "q");
		eliminate_hollow_nodes(&mut nodes);
		assert_eq!(nodes.len(), 1);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn promotion_preserves_every_text_and_only_shortens_its_path() {
		// 固定种子生成不同形状的树，检查结果而不重复提升算法
		let mut seed = 7_u64;
		for _ in 0..64 {
			let mut nodes = vec![CodeNode::root()];
			let mut original = HashMap::new();
			for index in 0..40 {
				seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
				let length = 1 + (seed % 5) as usize;
				let mut code = String::new();
				for _ in 0..length {
					seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
					code.push(['z', 'a', 'b', 'A', '\''][(seed >> 32) as usize % 5]);
				}
				let text = format!("text{index}");
				add_texts(&mut nodes, &code, &[&text]);
				let _ = original.insert(text, code);
			}
			eliminate_hollow_nodes(&mut nodes);
			assert_filled_tree(&nodes);
			let mut remaining = original.clone();
			for (code, texts) in groups(&nodes) {
				for text in texts {
					let original_code = remaining.remove(&text).expect("duplicate or added text");
					assert!(original_code.starts_with(&code));
				}
			}
			assert!(remaining.is_empty());
			let once = groups(&nodes);
			eliminate_hollow_nodes(&mut nodes);
			assert_eq!(groups(&nodes), once);
			assert_filled_tree(&nodes);
		}
	}
}
