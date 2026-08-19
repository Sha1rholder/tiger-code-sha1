use crate::data::parse_target;
use crate::{Freq, Text};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 英文前置加词
pub static EN_FIRST: LazyLock<Vec<Text>> = LazyLock::new(|| {
	let values: Vec<Text> = parse_target("src/data/en/first.txt")
		.into_iter()
		.map(Text::from)
		.collect();
	assert_eq!(
		values.len(),
		values.iter().collect::<HashSet<_>>().len(),
		"English first data must not contain duplicate entries"
	);
	values
});

/// 英文后置加词
pub static EN_LAST: LazyLock<Vec<Text>> = LazyLock::new(|| {
	let values: Vec<Text> = parse_target("src/data/en/last.txt")
		.into_iter()
		.map(Text::from)
		.collect();
	assert_eq!(
		values.len(),
		values.iter().collect::<HashSet<_>>().len(),
		"English last data must not contain duplicate entries"
	);
	values
});

/// ESDB
pub static ESDB: LazyLock<HashSet<Text>> = LazyLock::new(|| {
	parse_target("src/data/ESDB/ESDB.txt")
		.into_iter()
		.skip_while(|line| line != "---")
		.skip(1)
		.map(Text::from)
		.collect()
});

/// 合法英文wordfreq
pub static EN_FREQ: LazyLock<HashMap<Text, Freq>> = LazyLock::new(|| {
	let path = "src/data/wordfreq/en.tsv";
	let mut seen = HashSet::new();
	let mut frequencies = HashMap::new();

	for line in parse_target(path).into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let freq = fields.next().expect("missing freq field");
		let freq = Freq::from(freq.parse().expect("invalid freq field"));
		let parsed_text = Text::from(text.to_owned());

		if !seen.insert(parsed_text.clone()) {
			panic!("duplicate text in {path}: {text}");
		}
		if text
			.chars()
			.any(|character| !character.is_ascii_alphabetic() && character != '\'')
		{
			continue;
		}

		let _ = frequencies.insert(parsed_text, freq);
	}

	frequencies
});

// 1
