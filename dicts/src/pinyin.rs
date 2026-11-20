//! 生成拼音反查候选表[`PINYIN_DICT`]
//!
//! 读取`data/zh/tiger-code/PY_c.dict.yaml.tsv`中的拼音文本对，按字符集过滤并建树，节点内按权重降序排列

use crate::prelude::{insert_code, parse_target};
use crate::zh_chars::{ZH_CHARS, contains_illegal_zh_char};
use crate::{CodeNode, Text, Weight};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 拼音编码节点，候选按权重降序排列，同权重时保留源文件顺序，0号节点固定为根
pub static PINYIN_DICT: LazyLock<Vec<CodeNode>> = LazyLock::new(|| {
	let path = "data/zh/tiger-code/PY_c.dict.yaml.tsv";
	let mut lines = parse_target(path).into_iter();
	// 校验并跳过TSV标题
	assert_eq!(
		lines.next().as_deref(),
		Some("text\tcode\tweight"),
		"invalid pinyin header in {path}"
	);
	let mut nodes = vec![CodeNode::root()];
	// 读取拼音数据，同权重时保留源文件顺序
	let mut weighted_entries = HashMap::new();

	for (order, line) in lines.enumerate() {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		// 先过滤文本，非法字符所在的记录无需解析编码和权重
		if contains_illegal_zh_char(text, &ZH_CHARS) {
			continue;
		}
		let code = fields.next().expect("missing code field");
		let weight = fields
			.next()
			.expect("missing weight field")
			.parse()
			.expect("invalid weight field");

		let node = insert_code(&mut nodes, code);
		let entry = (node, Text::from(text.to_owned()));
		let previous = weighted_entries.insert(entry, (Weight::from(weight), order));
		assert!(
			previous.is_none(),
			"duplicate code and text in {path}: {code}\t{text}"
		);
	}

	let mut weighted_entries = weighted_entries.into_iter().collect::<Vec<_>>();
	weighted_entries.sort_unstable_by_key(|(_, (weight, order))| (Reverse(*weight), *order));

	for ((node, text), _) in weighted_entries {
		nodes[node].texts_mut().push(text);
	}
	nodes
});
