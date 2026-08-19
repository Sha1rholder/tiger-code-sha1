use crate::data::parse_target;
use crate::{Freq, Text};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

type TextLowcase = Text;
type TextESDB = Text;

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
static ESDB: LazyLock<HashSet<TextESDB>> = LazyLock::new(|| {
	let values: HashSet<TextESDB> = parse_target("src/data/ESDB/ESDB.txt")
		.into_iter()
		.skip_while(|line| line != "---")
		.skip(1)
		.map(TextESDB::from)
		.collect();
	assert!(
		values.iter().all(|text| text
			.as_str()
			.chars()
			.all(|character| character.is_ascii_alphabetic() || character == '\'')),
		"ESDB must contain only ASCII letters and apostrophes"
	);
	values
});

#[cfg(test)]
mod tests {
	use super::*;

	/// 构造测试文本
	fn text(value: &str) -> Text {
		Text::from(value.to_owned())
	}

	/// 验证ESDB只包含ASCII字母和撇号
	#[test]
	fn esdb_contains_only_ascii_letters_and_apostrophes() {
		assert!(ESDB.iter().all(|text| {
			text.as_str()
				.chars()
				.all(|character| character.is_ascii_alphabetic() || character == '\'')
		}));
	}

	/// 验证英文词频键只包含ASCII小写字母和撇号
	#[test]
	fn en_freq_contains_only_lowercase_ascii_letters_and_apostrophes() {
		assert!(EN_FREQ.keys().all(|text| {
			text.as_str()
				.chars()
				.all(|character| character.is_ascii_lowercase() || character == '\'')
		}));
	}

	/// 验证非ESDB词条被丢弃
	#[test]
	fn en_freq_discards_non_esdb_entries() {
		assert!(!EN_FREQ.contains_key(&text("00")));
		assert!(!EN_FREQ.contains_key(&text("u.s")));
	}

	/// 验证英文词频保留ESDB中的全部原形
	#[test]
	fn en_freq_preserves_all_esdb_forms() {
		let (_, forms) = EN_FREQ.get(&text("a")).expect("missing English frequency");

		assert_eq!(forms, &vec![text("A"), text("a")]);
	}
}

/// 合法英文wordfreq
static EN_FREQ: LazyLock<HashMap<TextLowcase, (Freq, Vec<TextESDB>)>> = LazyLock::new(|| {
	let mut esdb_forms: HashMap<TextLowcase, Vec<TextESDB>> = HashMap::new();

	// 按小写形式分组ESDB原形
	for text in ESDB.iter() {
		let text_lowcase = TextLowcase::from(text.as_str().to_ascii_lowercase());
		esdb_forms
			.entry(text_lowcase)
			.or_default()
			.push(text.clone());
	}
	// 固定同一小写形式的原形顺序
	for forms in esdb_forms.values_mut() {
		forms.sort();
	}

	let mut frequencies = HashMap::new();

	// 只保留大小写不敏感匹配ESDB的词频
	for line in parse_target("src/data/wordfreq/en.tsv").into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let text_lowcase = TextLowcase::from(text.to_ascii_lowercase());
		let Some(forms) = esdb_forms.get(&text_lowcase) else {
			continue;
		};

		let freq = fields.next().expect("missing freq field");
		let freq = Freq::from(freq.parse().expect("invalid freq field"));

		let _ = frequencies.insert(text_lowcase, (freq, forms.clone()));
	}

	assert!(
		frequencies.keys().all(|text| {
			text.as_str()
				.chars()
				.all(|character| character.is_ascii_lowercase() || character == '\'')
		}),
		"English word frequency results must contain only lowercase ASCII letters and apostrophes"
	);
	frequencies
});
