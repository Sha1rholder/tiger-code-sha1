//! 生成拼音反查候选表[`ZH_PY`]
//!
//! 读取`data/zh/tiger-code/PY_c.dict.yaml.tsv`中的拼音汉字对，按字符集过滤后按权重降序排列

use crate::prelude::parse_target;
use crate::zh_chars::{ZH_CHARS, contains_illegal_zh_char};
use crate::{Code, Text, Weight};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::LazyLock;

/// 读取无表头的拼音数据，同权重时保留源文件顺序
fn build_pinyin(
	lines: impl IntoIterator<Item = String>,
	path: &str,
	zh_chars: &HashSet<Text>,
) -> Vec<(Code, Text)> {
	let mut weighted_entries = HashMap::new();

	for line in lines {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let code = fields.next().expect("missing code field");
		let weight = fields
			.next()
			.expect("missing weight field")
			.parse()
			.expect("invalid weight field");

		if contains_illegal_zh_char(text, zh_chars) {
			continue;
		}

		let entry = (Code::from(code.to_owned()), Text::from(text.to_owned()));
		let previous = weighted_entries.insert(entry, Weight::from(weight));
		assert!(
			previous.is_none(),
			"duplicate code and text in {path}: {code}\t{text}"
		);
	}

	let mut weighted_entries = weighted_entries
		.into_iter()
		.map(|((code, text), weight)| (code, text, weight))
		.collect::<Vec<_>>();
	weighted_entries.sort_by_key(|(_, _, weight)| Reverse(*weight));

	weighted_entries
		.into_iter()
		.map(|(code, text, _)| (code, text))
		.collect()
}

/// 按权重降序排列的拼音和汉字
pub static PINYIN_DICT: LazyLock<Vec<(Code, Text)>> = LazyLock::new(|| {
	let path = "data/zh/tiger-code/PY_c.dict.yaml.tsv";
	build_pinyin(parse_target(path), path, &ZH_CHARS)
});
