use crate::prelude::*;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// 跳过空行和`#`开头的行，读取单个文本文件为列表
fn parse_target(path: impl AsRef<Path>) -> Vec<String> {
	let path = path.as_ref();
	let content = fs::read_to_string(path)
		.unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

	content
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(str::to_owned)
		.collect()
}

/// 英文前置加词
pub static EN_FIRST: LazyLock<BTreeSet<Text>> = LazyLock::new(|| {
	parse_target("src/data/en/first.txt")
		.into_iter()
		.map(Text::from)
		.collect()
});

/// 英文后置加词
pub static EN_LAST: LazyLock<BTreeSet<Text>> = LazyLock::new(|| {
	parse_target("src/data/en/last.txt")
		.into_iter()
		.map(Text::from)
		.collect()
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

/// 允许的汉字表
pub static SC2013: LazyLock<HashSet<Text>> = LazyLock::new(|| {
	parse_target("src/data/SC2013/level-1.txt")
		.into_iter()
		.chain(parse_target("src/data/SC2013/level-2.txt"))
		.chain(parse_target("src/data/SC2013/level-3.txt"))
		.chain(parse_target("src/data/SC2013/custom.txt"))
		.map(Text::from)
		.collect()
});

/// 拼音、汉字和权重
pub static ZH_PY: LazyLock<HashSet<(Code, Text, Weight)>> = LazyLock::new(|| {
	parse_target("src/data/py/py.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let code = fields.next().expect("missing code field");
			let weight = fields
				.next()
				.expect("missing weight field")
				.parse()
				.expect("invalid weight field");

			(
				Code::from(code.to_owned()),
				Text::from(text.to_owned()),
				Weight::from(weight),
			)
		})
		.collect()
});

/// 虎码
pub static TIGER: LazyLock<HashSet<(Code, Text, Weight)>> = LazyLock::new(|| {
	parse_target("src/data/zh/tiger.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let code = fields.next().expect("missing code field");
			let weight = fields
				.next()
				.expect("missing weight field")
				.parse()
				.expect("invalid weight field");

			(
				Code::from(code.to_owned()),
				Text::from(text.to_owned()),
				Weight::from(weight),
			)
		})
		.collect()
});

/// 自定义编码
pub static ZH_CUSTOM: LazyLock<BTreeSet<(Code, Text)>> = LazyLock::new(|| {
	parse_target("src/data/zh/custom.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let code = fields.next().expect("missing code field");

			(Code::from(code.to_owned()), Text::from(text.to_owned()))
		})
		.collect()
});

/// 重编码
pub static ZH_RECODE: LazyLock<HashSet<(Code, Text)>> = LazyLock::new(|| {
	parse_target("src/data/zh/tiger.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let code = fields.next().expect("missing code field");

			(Code::from(code.to_owned()), Text::from(text.to_owned()))
		})
		.collect()
});

/// 中文wordfreq
pub static ZH_FREQ: LazyLock<HashSet<(Text, Freq)>> = LazyLock::new(|| {
	parse_target("src/data/wordfreq/zh.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let freq = fields.next().expect("missing freq field");

			(
				Text::from(text.to_owned()),
				Freq::from(freq.parse().expect("invalid freq field")),
			)
		})
		.collect()
});

/// 英文wordfreq
pub static EN_FREQ: LazyLock<HashSet<(Text, Freq)>> = LazyLock::new(|| {
	parse_target("src/data/wordfreq/en.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let text = fields.next().expect("missing text field");
			let freq = fields.next().expect("missing freq field");

			(
				Text::from(text.to_owned()),
				Freq::from(freq.parse().expect("invalid freq field")),
			)
		})
		.collect()
});

#[cfg(test)]
mod tests {}
