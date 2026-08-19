use crate::data::parse_target;
use crate::sc2013::{SC2013, contains_non_sc2013_char};
use crate::{Code, Text, Weight};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 按权重降序排列的拼音和汉字
pub static ZH_PY: LazyLock<Vec<(Code, Text)>> = LazyLock::new(|| {
	let path = "src/data/py/py.tsv";
	let mut weighted_entries = HashMap::new();

	for line in parse_target(path).into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let code = fields.next().expect("missing code field");
		let weight = fields
			.next()
			.expect("missing weight field")
			.parse()
			.expect("invalid weight field");

		if contains_non_sc2013_char(text, &SC2013) {
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
});
