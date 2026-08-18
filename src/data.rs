use crate::prelude::*;
use indexmap::IndexMap;
use std::cmp::Reverse;

use std::collections::HashMap;
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

/// 过滤并选择每个字符使用的虎码，同时保留字符首次插入的顺序
fn build_tiger_chars_raw(
	mut tiger: Vec<(Code, Text, Weight)>,
	sc2013: &HashSet<Text>,
) -> IndexMap<Text, Code> {
	// 丢弃SC2013外的字符
	tiger.retain(|(_, text, _)| sc2013.contains(text));
	// 按Weight稳定降序，保持同Weight的原顺序
	tiger.sort_by_key(|(_, _, weight)| Reverse(*weight));
	// 丢弃Weight
	let tiger: Vec<(Code, Text)> = tiger
		.into_iter()
		.map(|(code, text, _)| (code, text))
		.collect();
	let mut tiger_chars_raw = IndexMap::new();
	// 记录当前IndexMap中每个Code的使用次数
	let mut current_codes = HashMap::<Code, usize>::new();

	for (code, text) in tiger {
		// 新Text按遍历顺序直接插入
		if !tiger_chars_raw.contains_key(&text) {
			*current_codes.entry(code.clone()).or_default() += 1;
			let _ = tiger_chars_raw.insert(text, code);
			continue;
		}

		let current_code = tiger_chars_raw
			.get(&text)
			.expect("existing text must have a code");
		// 仅接受更短且当前未占用的Code
		if code.len() >= current_code.len() || current_codes.contains_key(&code) {
			continue;
		}

		// 释放旧Code的当前使用次数
		let old_code = current_code.clone();
		let remove_old_code = {
			let count = current_codes
				.get_mut(&old_code)
				.expect("current code must have a count");
			*count -= 1;
			*count == 0
		};
		if remove_old_code {
			let _ = current_codes.remove(&old_code);
		}
		// 登记并写入新Code
		let previous = current_codes.insert(code.clone(), 1);
		debug_assert!(previous.is_none());
		*tiger_chars_raw
			.get_mut(&text)
			.expect("existing text must have a code") = code;
	}

	tiger_chars_raw
}

/// 断言重编码键有效并替换字符编码
fn apply_tiger_recode(
	tiger_chars_raw: &mut IndexMap<Text, Code>,
	tiger_recode: HashMap<Text, Code>,
) {
	// 重编码键必须来自原始字符表
	assert!(
		tiger_recode
			.keys()
			.all(|text| tiger_chars_raw.contains_key(text)),
		"tiger recode keys must be a subset of tiger raw keys"
	);

	// 原位替换以保持IndexMap顺序
	for (text, code) in tiger_recode {
		*tiger_chars_raw
			.get_mut(&text)
			.expect("validated recode text must exist") = code;
	}
}

/// 按编码分组文本并保留输入中的相对顺序
fn group_texts_by_code(
	entries: impl IntoIterator<Item = (Code, Text)>,
) -> HashMap<Code, Vec<Text>> {
	let mut grouped = HashMap::<Code, Vec<Text>>::new();

	// 同Code的Text按输入顺序追加
	for (code, text) in entries {
		grouped.entry(code).or_default().push(text);
	}

	grouped
}

/// 返回文本是否包含键集合中的其它文本
fn contains_other_text(text: &str, texts: &HashSet<&str>) -> bool {
	// 逐字符选择子串起点
	for (start, _) in text.char_indices() {
		let suffix = &text[start..];

		// 枚举该起点后的所有字符边界
		for (relative_end, _) in suffix
			.char_indices()
			.skip(1)
			.chain(std::iter::once((suffix.len(), '\0')))
		{
			let end = start + relative_end;
			// 跳过文本自身
			if start == 0 && end == text.len() {
				continue;
			}
			// 命中任意其它键后立即返回
			if texts.contains(&text[start..end]) {
				return true;
			}
		}
	}

	false
}

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

/// 虎码字符表
pub static TIGER_CHARS: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 合并SC2013字符表
	let sc2013 = parse_target("src/data/SC2013/level-1.txt")
		.into_iter()
		.chain(parse_target("src/data/SC2013/level-2.txt"))
		.chain(parse_target("src/data/SC2013/level-3.txt"))
		.chain(parse_target("src/data/SC2013/custom.txt"))
		.map(Text::from)
		.collect::<HashSet<_>>();
	// 读取原始虎码元组
	let tiger = parse_target("src/data/zh/tiger.tsv")
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
		.collect();
	// 生成有序的单字编码表
	let mut tiger_chars_raw = build_tiger_chars_raw(tiger, &sc2013);
	// 断言字符集合完整覆盖SC2013
	let tiger_keys = tiger_chars_raw.keys().collect::<HashSet<_>>();
	let sc2013_keys = sc2013.iter().collect::<HashSet<_>>();
	assert_eq!(tiger_keys, sc2013_keys, "tiger raw keys must equal SC2013");
	// 断言后释放SC2013
	drop(sc2013);

	// 读取Text到Code的重编码表
	let tiger_recode = parse_target("src/data/zh/recode.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Text::from(text.to_owned()), Code::from(code.to_owned()))
		})
		.collect::<HashMap<_, _>>();
	// 应用重编码
	apply_tiger_recode(&mut tiger_chars_raw, tiger_recode);

	// 按Code分组并保留Text相对顺序
	group_texts_by_code(tiger_chars_raw.into_iter().map(|(text, code)| (code, text)))
});

/// 自定义编码
pub static ZH_CUSTOM: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	// 读取自定义Code和Text
	let entries = parse_target("src/data/zh/custom.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Code::from(code.to_owned()), Text::from(text.to_owned()))
		});

	// 按Code分组并保留Text原顺序
	group_texts_by_code(entries)
});

/// 合法中文wordfreq
pub static ZH_FREQ: LazyLock<HashMap<Text, Freq>> = LazyLock::new(|| {
	let path = "src/data/wordfreq/zh.tsv";
	let mut seen = HashSet::new();
	let mut frequencies = HashMap::new();
	// 从TIGER_CHARS提取合法字符
	let allowed_chars = TIGER_CHARS
		.values()
		.flatten()
		.cloned()
		.collect::<HashSet<_>>();

	// 读取并过滤中文词频
	for line in parse_target(path).into_iter().skip(1) {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let freq = fields.next().expect("missing freq field");
		let freq = Freq::from(freq.parse().expect("invalid freq field"));
		let parsed_text = text.to_owned();

		// 拒绝重复Text
		if !seen.insert(parsed_text.clone()) {
			panic!("duplicate text in {path}: {text}");
		}
		// 丢弃单字和包含非法字符的词
		if text.chars().count() == 1
			|| text
				.chars()
				.any(|character| !allowed_chars.contains(&Text::from(character.to_string())))
		{
			continue;
		}

		let _ = frequencies.insert(parsed_text, freq);
	}

	// 临时借用全部合法键
	let texts = frequencies.keys().map(String::as_str).collect::<HashSet<_>>();
	// 找出包含任意其它键的键
	let contained_texts = texts
		.iter()
		.copied()
		.filter(|text| contains_other_text(text, &texts))
		.map(str::to_owned)
		.collect::<Vec<_>>();
	// 释放键集合借用
	drop(texts);

	// 删除包含其它键的键值对
	for text in contained_texts {
		let _ = frequencies.remove(&text);
	}

	// 转换为公开的Text键
	frequencies
		.into_iter()
		.map(|(text, freq)| (Text::from(text), freq))
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

#[cfg(test)]
mod tests {
	use super::*;

	/// 构造测试文本
	fn text(value: &str) -> Text {
		Text::from(value.to_owned())
	}

	/// 构造测试编码
	fn code(value: &str) -> Code {
		Code::from(value.to_owned())
	}

	/// 构造测试权重
	fn weight(value: u32) -> Weight {
		Weight::from(value)
	}

	/// 验证只匹配其它完整键且命中任意一个即可
	#[test]
	fn contained_text_detection_skips_the_text_itself() {
		let texts = ["之所以", "所以", "敏感元件", "元件", "敏感", "独立"]
			.into_iter()
			.collect::<HashSet<_>>();

		assert!(contains_other_text("之所以", &texts));
		assert!(contains_other_text("敏感元件", &texts));
		assert!(!contains_other_text("所以", &texts));
		assert!(!contains_other_text("敏感", &texts));
		assert!(!contains_other_text("独立", &texts));
	}

	/// 验证真实中文词频表丢弃包含其它键的词
	#[test]
	fn zh_freq_removes_texts_containing_other_keys() {
		assert!(ZH_FREQ.contains_key(&text("所以")));
		assert!(ZH_FREQ.contains_key(&text("敏感")));
		assert!(ZH_FREQ.contains_key(&text("元件")));
		assert!(!ZH_FREQ.contains_key(&text("之所以")));
		assert!(!ZH_FREQ.contains_key(&text("敏感元件")));
	}

	/// 验证过滤、稳定排序、当前编码占用和编码释放
	#[test]
	fn tiger_raw_uses_current_code_occupancy() {
		let sc2013 = [text("甲"), text("乙"), text("丙")]
			.into_iter()
			.collect::<HashSet<_>>();
		let tiger = vec![
			(code("unused"), text("外"), weight(200)),
			(code("zz"), text("甲"), weight(100)),
			(code("zz"), text("乙"), weight(100)),
			(code("long"), text("丙"), weight(90)),
			(code("a"), text("甲"), weight(80)),
			(code("zz"), text("丙"), weight(70)),
			(code("b"), text("乙"), weight(60)),
			(code("zz"), text("丙"), weight(50)),
			(code("c"), text("甲"), weight(40)),
			(code("bbbb"), text("乙"), weight(30)),
		];

		let tiger_chars_raw = build_tiger_chars_raw(tiger, &sc2013);

		assert_eq!(
			tiger_chars_raw.keys().cloned().collect::<Vec<_>>(),
			vec![text("甲"), text("乙"), text("丙")]
		);
		assert_eq!(tiger_chars_raw.get(&text("甲")), Some(&code("a")));
		assert_eq!(tiger_chars_raw.get(&text("乙")), Some(&code("b")));
		assert_eq!(tiger_chars_raw.get(&text("丙")), Some(&code("zz")));
	}

	/// 验证重编码不改变字符顺序且分组保持相对顺序
	#[test]
	fn recode_and_group_preserve_text_order() {
		let mut tiger_chars_raw = IndexMap::new();
		let _ = tiger_chars_raw.insert(text("甲"), code("aa"));
		let _ = tiger_chars_raw.insert(text("乙"), code("bb"));
		let _ = tiger_chars_raw.insert(text("丙"), code("aa"));
		let tiger_recode = [(text("乙"), code("aa"))].into_iter().collect();

		apply_tiger_recode(&mut tiger_chars_raw, tiger_recode);
		let tiger_chars =
			group_texts_by_code(tiger_chars_raw.into_iter().map(|(text, code)| (code, text)));

		assert_eq!(
			tiger_chars.get(&code("aa")),
			Some(&vec![text("甲"), text("乙"), text("丙")])
		);
	}

	/// 验证重编码拒绝原始字符表之外的键
	#[test]
	#[should_panic(expected = "tiger recode keys must be a subset of tiger raw keys")]
	fn recode_rejects_unknown_text() {
		let mut tiger_chars_raw = IndexMap::new();
		let _ = tiger_chars_raw.insert(text("甲"), code("a"));
		let tiger_recode = [(text("乙"), code("b"))].into_iter().collect();

		apply_tiger_recode(&mut tiger_chars_raw, tiger_recode);
	}

	/// 验证真实虎码字符表覆盖SC2013并应用全部重编码
	#[test]
	fn tiger_chars_cover_sc2013_and_apply_recode() {
		let sc2013 = parse_target("src/data/SC2013/level-1.txt")
			.into_iter()
			.chain(parse_target("src/data/SC2013/level-2.txt"))
			.chain(parse_target("src/data/SC2013/level-3.txt"))
			.chain(parse_target("src/data/SC2013/custom.txt"))
			.map(Text::from)
			.collect::<HashSet<_>>();
		let tiger_texts = TIGER_CHARS.values().flatten().collect::<HashSet<_>>();
		let tiger_text_count = TIGER_CHARS.values().map(Vec::len).sum::<usize>();

		assert_eq!(tiger_text_count, sc2013.len());
		assert_eq!(tiger_texts.len(), sc2013.len());
		assert!(sc2013.iter().all(|text| tiger_texts.contains(text)));

		for line in parse_target("src/data/zh/recode.tsv").into_iter().skip(1) {
			let mut fields = line.split('\t');
			let code = code(fields.next().expect("missing code field"));
			let text = text(fields.next().expect("missing text field"));

			assert!(
				TIGER_CHARS
					.get(&code)
					.is_some_and(|texts| texts.contains(&text))
			);
		}
	}

	/// 验证自定义编码按源文件顺序分组
	#[test]
	fn custom_codes_preserve_source_order() {
		assert_eq!(
			ZH_CUSTOM.get(&code("bl")),
			Some(&vec![text("<blockquote>"), text("</blockquote>")])
		);
		assert_eq!(
			ZH_CUSTOM.get(&code("sha")),
			Some(&vec![
				text("sha1"),
				text("sha1rholder"),
				text("sha1rholder@gmail.com"),
				text("sha1rholder@outlook.com"),
			])
		);
	}
}
