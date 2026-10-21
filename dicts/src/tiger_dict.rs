//! 生成应用重编码后的虎码字符表[`TIGER_CHARS_DICT`]
//!
//! 读取虎码原始数据，按字符集过滤并选码，排列单字、消除空心节点，最后应用重编码

use crate::prelude::{eliminate_hollow_nodes, insert_code, parse_target};
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
