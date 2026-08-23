use crate::data::parse_target;
use crate::{Code, Freq, Text};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 原始词频
type FreqSelf = Freq;
/// 提权词频
type FreqBoost = Freq;
/// 图节点编号
type NodeId = usize;
/// 降权次数
type DemotionCount = usize;

/// 作为基本形式参与构图的最短码长
const INFLECT_START: usize = 4;
/// 允许进入英文词典的最低词频
const MIN_FREQ: Freq = Freq::from(1e-7);
/// 允许双写的小写辅音
const DOUBLE_LOWER: &str = "bdgklmnprstz";
/// 允许双写的大写辅音
const DOUBLE_UPPER: &str = "BDGKLMNPRSTZ";
/// EN_WORDS_LONG中最短码长
const MIN_LONG: usize = 5;

/// 英文文本的大小写分类
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum WordCase {
	/// 全小写，如apple
	Lower,
	/// 首字母小写的混合形式，如applE
	LowerMixed,
	/// 标准首字母大写，如Apple
	Capitalized,
	/// 首字母大写的混合形式，如ApplE
	CapitalizedMixed,
	/// 全大写，如APPLE
	Upper,
}

impl WordCase {
	/// 最终结果中的固定分类顺序
	const OUTPUT_ORDER: [Self; 5] = [
		Self::Lower,
		Self::LowerMixed,
		Self::Capitalized,
		Self::CapitalizedMixed,
		Self::Upper,
	];
}

/// 英文文本
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TextEn(Text);

impl TextEn {
	/// 从字符串创建英文文本
	fn from_string(value: String) -> Self {
		assert!(
			is_valid_english_text(&value),
			"English text must contain only ASCII letters and apostrophes with a nonempty case stem: {value}"
		);
		Self(Text::from(value))
	}

	/// 返回底层字符串切片
	fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// 返回英文文本的字节长度
	fn len(&self) -> usize {
		self.as_str().len()
	}

	/// 返回英文文本的大小写分类
	fn word_case(&self) -> WordCase {
		// 删除所有格后缀并忽略其余撇号
		let stem = self
			.as_str()
			.strip_suffix("'s")
			.unwrap_or_else(|| self.as_str());
		let letters = stem
			.bytes()
			.filter(|byte| *byte != b'\'')
			.collect::<Vec<_>>();
		let first = letters
			.first()
			.expect("validated English text must have a case stem");

		// 首字母大写时区分全大写、标准形式和混合形式
		if first.is_ascii_uppercase() {
			if letters.iter().all(u8::is_ascii_uppercase) {
				WordCase::Upper
			} else if letters[1..].iter().all(u8::is_ascii_lowercase) {
				WordCase::Capitalized
			} else {
				WordCase::CapitalizedMixed
			}
		} else if letters.iter().all(u8::is_ascii_lowercase) {
			WordCase::Lower
		} else {
			WordCase::LowerMixed
		}
	}

	/// 返回英文文本的全小写形式
	fn lower(&self) -> TextLower {
		TextLower::from_lowercase(self.as_str().to_ascii_lowercase())
	}

	/// 转换为通用文本
	fn into_text(self) -> Text {
		self.0
	}
}

/// 英文文本的全小写形式
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TextLower(TextEn);

impl TextLower {
	/// 从全小写字符串创建英文小写文本
	fn from_lowercase(value: String) -> Self {
		assert!(
			value.bytes().all(|byte| !byte.is_ascii_uppercase()),
			"lowercase English text must not contain uppercase ASCII letters: {value}"
		);
		Self(TextEn::from_string(value))
	}

	/// 返回底层字符串切片
	fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// 返回英文小写文本的字节长度
	fn len(&self) -> usize {
		self.0.len()
	}

	/// 返回标准小写词形
	fn lower_form(&self) -> TextEn {
		self.0.clone()
	}

	/// 返回标准首字母大写词形
	fn capitalized_form(&self) -> TextEn {
		// 只转换首个字母并保留撇号位置
		let mut bytes = self.as_str().as_bytes().to_vec();
		let first = bytes
			.iter_mut()
			.find(|byte| byte.is_ascii_alphabetic())
			.expect("validated English text must contain a letter");
		first.make_ascii_uppercase();
		TextEn::from_string(String::from_utf8(bytes).expect("validated English text must be ASCII"))
	}

	/// 返回标准全大写词形
	fn upper_form(&self) -> TextEn {
		TextEn::from_string(self.as_str().to_ascii_uppercase())
	}
}

/// 返回字符串能否构成具有非空大小写词干的英文文本
fn is_valid_english_text(value: &str) -> bool {
	// 字符只能是ASCII字母或撇号
	if !value
		.bytes()
		.all(|byte| byte.is_ascii_alphabetic() || byte == b'\'')
	{
		return false;
	}

	// 去掉所有格后缀后必须仍含字母
	value
		.strip_suffix("'s")
		.unwrap_or(value)
		.bytes()
		.any(|byte| byte.is_ascii_alphabetic())
}

/// 构造按全小写形式分组的ESDB
fn build_esdb(
	lines: impl IntoIterator<Item = String>,
	path: &str,
) -> HashMap<TextLower, HashSet<TextEn>> {
	let mut lines = lines.into_iter();
	let mut found_marker = false;
	// 跳过说明文本并定位词表起点
	for line in lines.by_ref() {
		if line == "---" {
			found_marker = true;
			break;
		}
	}
	assert!(found_marker, "missing ESDB data marker in {path}");

	let mut seen = HashSet::new();
	let mut grouped = HashMap::<TextLower, HashSet<TextEn>>::new();
	// 校验唯一性并按全小写形式分组
	for value in lines {
		assert!(
			!value.ends_with('\'') && !value.ends_with("'S"),
			"ESDB text must not end with an apostrophe or uppercase possessive suffix: {value}"
		);
		let text = TextEn::from_string(value);
		assert!(
			seen.insert(text.clone()),
			"duplicate text in {path}: {}",
			text.as_str()
		);
		let lower = text.lower();
		assert!(
			grouped.entry(lower).or_default().insert(text),
			"ESDB group must not contain duplicate text"
		);
	}

	grouped
}

/// ESDB
static ESDB: LazyLock<HashMap<TextLower, HashSet<TextEn>>> = LazyLock::new(|| {
	let path = "src/data/ESDB/ESDB.txt";
	// 读取并分组ESDB词条
	build_esdb(parse_target(path), path)
});

/// 构造合法英文wordfreq
fn build_en_freq(
	lines: impl IntoIterator<Item = String>,
	path: &str,
) -> HashMap<TextLower, FreqSelf> {
	let mut lines = lines.into_iter();
	// 校验TSV标题
	assert_eq!(
		lines.next().as_deref(),
		Some("text\tfrequency"),
		"invalid English wordfreq header in {path}"
	);

	let mut seen = HashSet::new();
	let mut frequencies = HashMap::new();
	// 读取并过滤英文词频
	for line in lines {
		let mut fields = line.split('\t');
		let text = fields.next().expect("missing text field");
		let raw_freq = fields.next().expect("missing freq field");
		assert!(fields.next().is_none(), "extra fields in {path}: {line}");
		// 重复和大写ASCII属于数据错误
		assert!(
			seen.insert(text.to_owned()),
			"duplicate text in {path}: {text}"
		);
		assert!(
			text.bytes().all(|byte| !byte.is_ascii_uppercase()),
			"English wordfreq results must not contain uppercase ASCII letters: {line}"
		);

		let raw_freq: f64 = raw_freq.parse().expect("invalid freq field");
		assert!(
			raw_freq.is_finite() && raw_freq > 0.0,
			"English wordfreq must contain positive finite frequencies: {line}"
		);
		// 非英文词形不进入英文词频表
		if !is_valid_english_text(text) {
			continue;
		}

		// 按全小写形式保存词频
		let lower = TextLower::from_lowercase(text.to_owned());
		assert!(
			frequencies
				.insert(lower, FreqSelf::from(raw_freq))
				.is_none(),
			"duplicate lowercase text in {path}: {text}"
		);
	}

	frequencies
}

/// 英文wordfreq
static EN_FREQ: LazyLock<HashMap<TextLower, FreqSelf>> = LazyLock::new(|| {
	let path = "src/data/wordfreq/en.tsv";
	// 读取并过滤英文词频
	build_en_freq(parse_target(path), path)
});

/// 按大小写分类并保持分类内的输入顺序
fn build_custom_words(
	lines: impl IntoIterator<Item = String>,
	path: &str,
) -> HashMap<WordCase, Vec<TextEn>> {
	let mut seen = HashSet::new();
	let mut grouped = HashMap::<WordCase, Vec<TextEn>>::new();

	// 校验唯一性并按大小写分类
	for value in lines {
		let text = TextEn::from_string(value);
		assert!(
			seen.insert(text.clone()),
			"duplicate text in {path}: {}",
			text.as_str()
		);
		grouped.entry(text.word_case()).or_default().push(text);
	}

	grouped
}

/// 英文前置加词
static EN_FIRST: LazyLock<HashMap<WordCase, Vec<TextEn>>> = LazyLock::new(|| {
	let path = "src/data/en/first.txt";
	// 读取并分类英文前置加词
	build_custom_words(parse_target(path), path)
});

/// 英文后置加词
static EN_LAST: LazyLock<HashMap<WordCase, Vec<TextEn>>> = LazyLock::new(|| {
	let path = "src/data/en/last.txt";
	// 读取并分类英文后置加词
	build_custom_words(parse_target(path), path)
});

/// 按现有大小写形式扩增标准大小写词形
fn expand_case_forms(groups: &mut HashMap<TextLower, HashSet<TextEn>>) {
	for (lower, forms) in groups {
		debug_assert!(forms.iter().all(|text| text.len() == lower.len()));
		// 记录扩增前已有的大小写分类
		let cases = forms.iter().map(TextEn::word_case).collect::<HashSet<_>>();

		// 全大写分类补充标准全大写形式
		if cases.contains(&WordCase::Upper) {
			let _ = forms.insert(lower.upper_form());
		}
		// 常规形式补齐小写、首字母大写和全大写
		if cases.contains(&WordCase::Lower) || cases.contains(&WordCase::Capitalized) {
			let _ = forms.insert(lower.lower_form());
			let _ = forms.insert(lower.capitalized_form());
			let _ = forms.insert(lower.upper_form());
		}
		// 混合大小写形式只补充标准小写
		if cases.contains(&WordCase::LowerMixed) || cases.contains(&WordCase::CapitalizedMixed) {
			let _ = forms.insert(lower.lower_form());
		}
	}
}

/// 英文词图节点
#[derive(Clone, Debug)]
struct WordNode {
	text: TextEn,
	freq_self: FreqSelf,
	direct_bases: Vec<NodeId>,
}

/// 英文词评分
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WordScore {
	demotion_count: DemotionCount,
	freq_boost: FreqBoost,
}

/// 扩增ESDB、关联词频并创建稳定节点
fn build_nodes(
	esdb: &HashMap<TextLower, HashSet<TextEn>>,
	frequencies: &HashMap<TextLower, FreqSelf>,
) -> Vec<WordNode> {
	// 克隆并扩增原始大小写组
	let mut groups = esdb.clone();
	expand_case_forms(&mut groups);

	let mut nodes = Vec::new();
	// 丢弃无词频组并展开所有词形
	for (lower, forms) in groups {
		let Some(freq_self) = frequencies.get(&lower).copied() else {
			continue;
		};
		for text in forms {
			nodes.push(WordNode {
				text,
				freq_self,
				direct_bases: Vec::new(),
			});
		}
	}

	// 按码长和字面顺序建立稳定拓扑顺序
	nodes.sort_unstable_by(|left, right| {
		left.text
			.len()
			.cmp(&right.text.len())
			.then_with(|| left.text.cmp(&right.text))
	});
	assert!(
		nodes.windows(2).all(|pair| pair[0].text != pair[1].text),
		"expanded English texts must be unique"
	);

	nodes
}

/// 生成一次变体可能得到的全部字面量
fn direct_variant_literals(base: &TextEn) -> HashSet<String> {
	let value = base.as_str();
	let Some(last) = value.bytes().next_back() else {
		return HashSet::new();
	};
	let mut variants = HashSet::new();

	if last.is_ascii_lowercase() {
		// 追加通用小写后缀
		for suffix in [
			"s", "es", "d", "ed", "ing", "ly", "er", "est", "ment", "ness", "able", "'", "'s",
			"'t", "'re",
		] {
			let _ = variants.insert(format!("{value}{suffix}"));
		}

		// 处理小写末字母替换
		let stem = &value[..value.len() - 1];
		match last {
			b'y' => {
				for suffix in ["ies", "ied", "ily", "ier", "iest", "iness"] {
					let _ = variants.insert(format!("{stem}{suffix}"));
				}
			}
			b'f' => {
				let _ = variants.insert(format!("{stem}ves"));
			}
			b'e' => {
				let _ = variants.insert(format!("{stem}ing"));
				let _ = variants.insert(format!("{stem}able"));
			}
			_ => {}
		}

		// 处理小写辅音双写
		if DOUBLE_LOWER.as_bytes().contains(&last) {
			for suffix in ["ed", "ing", "er", "est"] {
				let _ = variants.insert(format!("{value}{}{suffix}", last as char));
			}
		}
	} else if last.is_ascii_uppercase() {
		// 追加通用大写后缀
		for suffix in [
			"s", "S", "ES", "D", "ED", "ING", "LY", "ER", "EST", "MENT", "NESS", "ABLE", "'", "'s",
			"'S", "'m", "'M", "'T", "'RE",
		] {
			let _ = variants.insert(format!("{value}{suffix}"));
		}

		// 处理大写末字母替换
		let stem = &value[..value.len() - 1];
		match last {
			b'Y' => {
				for suffix in ["IES", "IED", "ILY", "IER", "IEST", "INESS"] {
					let _ = variants.insert(format!("{stem}{suffix}"));
				}
			}
			b'F' => {
				let _ = variants.insert(format!("{stem}VES"));
			}
			b'E' => {
				let _ = variants.insert(format!("{stem}ING"));
				let _ = variants.insert(format!("{stem}ABLE"));
			}
			_ => {}
		}

		// 处理大写辅音双写
		if DOUBLE_UPPER.as_bytes().contains(&last) {
			for suffix in ["ED", "ING", "ER", "EST"] {
				let _ = variants.insert(format!("{value}{}{suffix}", last as char));
			}
		}
	}

	variants
}

/// 为存在于词表中的直接变体记录基本形式入边
fn attach_direct_bases(nodes: &mut [WordNode]) {
	// 建立字面量到稳定节点编号的临时索引
	let lookup = nodes
		.iter()
		.enumerate()
		.map(|(node_id, node)| (node.text.as_str(), node_id))
		.collect::<HashMap<_, _>>();
	assert_eq!(
		lookup.len(),
		nodes.len(),
		"English graph texts must be unique"
	);

	let mut direct_bases = vec![Vec::new(); nodes.len()];
	// 只为词表中实际存在的变体记录入边
	for (base_id, node) in nodes.iter().enumerate() {
		if node.text.len() < INFLECT_START {
			continue;
		}
		for variant in direct_variant_literals(&node.text) {
			let Some(&variant_id) = lookup.get(variant.as_str()) else {
				continue;
			};
			assert!(
				nodes[variant_id].text.len() > node.text.len() && variant_id > base_id,
				"English variants must follow their shorter base forms"
			);
			direct_bases[variant_id].push(base_id);
		}
	}
	drop(lookup);

	// 排序去重后写回节点
	for (node, mut bases) in nodes.iter_mut().zip(direct_bases) {
		bases.sort_unstable();
		bases.dedup();
		node.direct_bases = bases;
	}
}

/// 计算每个节点的全部直接和间接基本形式
fn collect_all_bases(nodes: &[WordNode]) -> Vec<HashSet<NodeId>> {
	let mut all_bases = vec![HashSet::new(); nodes.len()];
	// 稳定节点顺序保证基本形式闭包已经就绪
	for node_id in 0..nodes.len() {
		let mut bases = HashSet::new();
		for &base_id in &nodes[node_id].direct_bases {
			assert!(base_id < node_id, "English graph must follow node order");
			let _ = bases.insert(base_id);
			bases.extend(all_bases[base_id].iter().copied());
		}
		all_bases[node_id] = bases;
	}

	all_bases
}

/// 根据全部基本形式计算节点评分
fn score_nodes(nodes: &[WordNode]) -> Vec<WordScore> {
	let all_bases = collect_all_bases(nodes);
	// 每个节点先保留自身原始词频
	let mut scores = nodes
		.iter()
		.map(|node| WordScore {
			demotion_count: 0,
			freq_boost: node.freq_self,
		})
		.collect::<Vec<_>>();

	// 每个唯一低频变体只为对应基本形式提权一次
	for (variant_id, bases) in all_bases.into_iter().enumerate() {
		let mut bases = bases.into_iter().collect::<Vec<_>>();
		bases.sort_unstable();
		for base_id in bases {
			if nodes[variant_id]
				.freq_self
				.total_cmp(&nodes[base_id].freq_self)
				== Ordering::Less
			{
				scores[base_id].freq_boost += nodes[variant_id].freq_self;
				scores[variant_id].demotion_count += 1;
			}
		}
	}

	scores
}

/// 比较两个带评分英文词的最终自动排序顺序
fn compare_scored_words(
	(left_text, left_score): &(TextEn, WordScore),
	(right_text, right_score): &(TextEn, WordScore),
) -> Ordering {
	// 依次比较降权次数、提权词频、码长和字面量
	left_score
		.demotion_count
		.cmp(&right_score.demotion_count)
		.then_with(|| right_score.freq_boost.total_cmp(&left_score.freq_boost))
		.then_with(|| left_text.len().cmp(&right_text.len()))
		.then_with(|| left_text.cmp(right_text))
}

/// 构造按大小写分类和评分排序的自动英文词
fn build_automatic_words(
	esdb: &HashMap<TextLower, HashSet<TextEn>>,
	frequencies: &HashMap<TextLower, FreqSelf>,
) -> HashMap<WordCase, Vec<TextEn>> {
	// 构图并计算全部节点评分
	let mut nodes = build_nodes(esdb, frequencies);
	attach_direct_bases(&mut nodes);
	let scores = score_nodes(&nodes);
	let mut grouped = HashMap::<WordCase, Vec<(TextEn, WordScore)>>::new();

	// 过滤低频节点并按大小写分类
	for (node, score) in nodes.into_iter().zip(scores) {
		if score.freq_boost.total_cmp(&MIN_FREQ) == Ordering::Less {
			continue;
		}
		grouped
			.entry(node.text.word_case())
			.or_default()
			.push((node.text, score));
	}

	// 分类内排序并丢弃评分
	grouped
		.into_iter()
		.map(|(word_case, mut words)| {
			words.sort_unstable_by(compare_scored_words);
			(word_case, words.into_iter().map(|(text, _)| text).collect())
		})
		.collect()
}

/// 校验并合并前置词、自动词和后置词
fn merge_en_words(
	mut automatic: HashMap<WordCase, Vec<TextEn>>,
	first: &HashMap<WordCase, Vec<TextEn>>,
	last: &HashMap<WordCase, Vec<TextEn>>,
) -> Vec<Text> {
	// 收集自定义词之间及其与自动词的精确冲突
	let automatic_texts = automatic.values().flatten().collect::<HashSet<_>>();
	let mut custom_seen = HashSet::new();
	let mut conflicts = HashSet::new();

	for text in first.values().flatten().chain(last.values().flatten()) {
		if !custom_seen.insert(text.clone()) || automatic_texts.contains(text) {
			let _ = conflicts.insert(text.clone());
		}
	}
	drop(automatic_texts);

	// 按字面顺序生成稳定错误信息
	if !conflicts.is_empty() {
		let mut conflicts = conflicts.into_iter().collect::<Vec<_>>();
		conflicts.sort_unstable();
		let conflicts = conflicts
			.iter()
			.map(TextEn::as_str)
			.collect::<Vec<_>>()
			.join(", ");
		panic!("custom English words conflict with generated words or each other: {conflicts}");
	}

	let mut words = Vec::new();
	// 按固定大小写分类顺序合并三层优先级
	for word_case in WordCase::OUTPUT_ORDER {
		if let Some(values) = first.get(&word_case) {
			words.extend(values.iter().cloned());
		}
		if let Some(values) = automatic.remove(&word_case) {
			words.extend(values);
		}
		if let Some(values) = last.get(&word_case) {
			words.extend(values.iter().cloned());
		}
	}

	words.into_iter().map(TextEn::into_text).collect()
}

/// 英文候选词
static EN_WORDS: LazyLock<Vec<Text>> = LazyLock::new(|| {
	// 先生成自动词再合并自定义词
	let automatic = build_automatic_words(&ESDB, &EN_FREQ);
	merge_en_words(automatic, &EN_FIRST, &EN_LAST)
});

/// 用英文词的所有非空前缀构造词典
fn build_en_dict(words: &[Text]) -> HashMap<Code, Vec<Text>> {
	let mut dictionary = HashMap::<Code, Vec<Text>>::new();

	for text in words {
		for end in 1..=text.as_str().len() {
			let code = Code::from(text.as_str()[..end].to_owned());
			dictionary.entry(code).or_default().push(text.clone());
		}
	}

	dictionary
}

/// 按所有非空前缀码分组的英文词典
pub static EN_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| build_en_dict(&EN_WORDS));

#[cfg(test)]
mod tests {
	use super::*;

	/// 构造测试英文文本
	fn english(value: &str) -> TextEn {
		TextEn::from_string(value.to_owned())
	}

	/// 构造测试英文小写文本
	fn lower(value: &str) -> TextLower {
		TextLower::from_lowercase(value.to_owned())
	}

	/// 构造测试词频
	fn freq(value: f64) -> Freq {
		Freq::from(value)
	}

	/// 构造测试图节点
	fn node(value: &str, freq_self: f64, direct_bases: Vec<NodeId>) -> WordNode {
		WordNode {
			text: english(value),
			freq_self: freq(freq_self),
			direct_bases,
		}
	}

	/// 返回英文文本字符串切片
	fn english_strings(values: &[TextEn]) -> Vec<&str> {
		values.iter().map(TextEn::as_str).collect()
	}

	/// 返回通用文本字符串切片
	fn text_strings(values: &[Text]) -> Vec<&str> {
		values.iter().map(Text::as_str).collect()
	}

	/// 构造带ESDB分隔行的测试输入
	fn esdb_lines(values: &[&str]) -> Vec<String> {
		["metadata", "---"]
			.into_iter()
			.chain(values.iter().copied())
			.map(str::to_owned)
			.collect()
	}

	/// 验证英文文本约束、大小写分类和标准词形
	#[test]
	fn english_text_classification_and_forms_follow_invariants() {
		for (value, expected) in [
			("apple", WordCase::Lower),
			("applE", WordCase::LowerMixed),
			("Apple", WordCase::Capitalized),
			("ApplE", WordCase::CapitalizedMixed),
			("APPLE", WordCase::Upper),
			("can't", WordCase::Lower),
			("Apple's", WordCase::Capitalized),
			("A's", WordCase::Upper),
			("I'M", WordCase::Upper),
		] {
			assert_eq!(english(value).word_case(), expected);
		}

		let chatgpt = english("ChatGPT").lower();
		assert_eq!(chatgpt.as_str(), "chatgpt");
		assert_eq!(chatgpt.lower_form().as_str(), "chatgpt");
		assert_eq!(chatgpt.capitalized_form().as_str(), "Chatgpt");
		assert_eq!(chatgpt.upper_form().as_str(), "CHATGPT");
	}

	/// 验证非法字符和空大小写词干会被拒绝
	#[test]
	fn english_text_validation_rejects_invalid_values() {
		for value in ["", "'", "'s", "''s", "well-known", "abc1", "café"] {
			assert!(!is_valid_english_text(value), "{value}");
		}
		for value in ["a", "A", "can't", "'apple", "apple's"] {
			assert!(is_valid_english_text(value), "{value}");
		}
	}

	/// 验证ESDB分组和标准大小写扩增
	#[test]
	fn esdb_groups_and_expands_case_forms() {
		let mut groups = build_esdb(
			esdb_lines(&["apple", "Apple", "iPhone", "NASA", "APPLE's"]),
			"test",
		);
		expand_case_forms(&mut groups);

		let apple = groups.get(&lower("apple")).expect("missing apple group");
		assert_eq!(
			apple.iter().map(TextEn::as_str).collect::<HashSet<_>>(),
			HashSet::from(["apple", "Apple", "APPLE"])
		);
		let iphone = groups.get(&lower("iphone")).expect("missing iphone group");
		assert_eq!(
			iphone.iter().map(TextEn::as_str).collect::<HashSet<_>>(),
			HashSet::from(["iphone", "iPhone"])
		);
		let nasa = groups.get(&lower("nasa")).expect("missing NASA group");
		assert_eq!(
			nasa.iter().map(TextEn::as_str).collect::<HashSet<_>>(),
			HashSet::from(["NASA"])
		);
		let apple_possessive = groups
			.get(&lower("apple's"))
			.expect("missing apple's group");
		assert_eq!(
			apple_possessive
				.iter()
				.map(TextEn::as_str)
				.collect::<HashSet<_>>(),
			HashSet::from(["APPLE's", "APPLE'S"])
		);
	}

	/// 验证ESDB必须包含数据分隔行
	#[test]
	#[should_panic(expected = "missing ESDB data marker")]
	fn esdb_rejects_missing_marker() {
		let _ = build_esdb(["apple".to_owned()], "test");
	}

	/// 验证ESDB拒绝禁用的末尾形式
	#[test]
	#[should_panic(expected = "must not end with an apostrophe")]
	fn esdb_rejects_forbidden_endings() {
		let _ = build_esdb(esdb_lines(&["word'"]), "test");
	}

	/// 验证英文词频过滤非法词形并保留合法词形
	#[test]
	fn english_frequencies_filter_non_english_text() {
		let frequencies = build_en_freq(
			[
				"text\tfrequency",
				"apple\t0.5",
				"can't\t0.25",
				"well-known\t0.2",
				"abc1\t0.1",
			]
			.into_iter()
			.map(str::to_owned),
			"test",
		);

		assert_eq!(frequencies.len(), 2);
		assert_eq!(frequencies.get(&lower("apple")), Some(&freq(0.5)));
		assert_eq!(frequencies.get(&lower("can't")), Some(&freq(0.25)));
	}

	/// 验证英文词频拒绝大写ASCII词条
	#[test]
	#[should_panic(expected = "must not contain uppercase ASCII letters")]
	fn english_frequencies_reject_uppercase_text() {
		let _ = build_en_freq(
			["text\tfrequency", "Apple\t0.5"]
				.into_iter()
				.map(str::to_owned),
			"test",
		);
	}

	/// 验证英文词频拒绝非正或非有限数值
	#[test]
	#[should_panic(expected = "positive finite frequencies")]
	fn english_frequencies_reject_invalid_frequency() {
		let _ = build_en_freq(
			["text\tfrequency", "apple\t0"]
				.into_iter()
				.map(str::to_owned),
			"test",
		);
	}

	/// 验证自定义词按分类保留源顺序
	#[test]
	fn custom_words_preserve_order_within_each_case() {
		let grouped = build_custom_words(
			["beta", "alpha", "Beta", "AlPhA"]
				.into_iter()
				.map(str::to_owned),
			"test",
		);

		assert_eq!(
			english_strings(grouped.get(&WordCase::Lower).expect("missing lower words")),
			vec!["beta", "alpha"]
		);
		assert_eq!(
			english_strings(
				grouped
					.get(&WordCase::Capitalized)
					.expect("missing capitalized words")
			),
			vec!["Beta"]
		);
		assert_eq!(
			english_strings(
				grouped
					.get(&WordCase::CapitalizedMixed)
					.expect("missing mixed words")
			),
			vec!["AlPhA"]
		);
	}

	/// 验证自定义词拒绝精确重复
	#[test]
	#[should_panic(expected = "duplicate text in test: alpha")]
	fn custom_words_reject_exact_duplicates() {
		let _ = build_custom_words(["alpha", "alpha"].into_iter().map(str::to_owned), "test");
	}

	/// 验证无词频ESDB组在节点创建前被丢弃
	#[test]
	fn nodes_drop_esdb_groups_without_frequency() {
		let esdb = build_esdb(esdb_lines(&["apple", "beta"]), "test");
		let frequencies = HashMap::from([(lower("apple"), freq(0.5))]);
		let nodes = build_nodes(&esdb, &frequencies);

		assert_eq!(
			nodes
				.iter()
				.map(|node| node.text.as_str())
				.collect::<Vec<_>>(),
			vec!["APPLE", "Apple", "apple"]
		);
	}

	/// 验证小写末字母的直接变体规则
	#[test]
	fn lowercase_direct_variants_cover_documented_rules() {
		let happy = direct_variant_literals(&english("happy"));
		for value in [
			"happys",
			"happyes",
			"happies",
			"happyd",
			"happyed",
			"happied",
			"happying",
			"happyly",
			"happily",
			"happyer",
			"happier",
			"happyest",
			"happiest",
			"happyment",
			"happyness",
			"happiness",
			"happyable",
			"happy'",
			"happy's",
			"happy't",
			"happy're",
		] {
			assert!(happy.contains(value), "{value}");
		}

		let plan = direct_variant_literals(&english("plan"));
		for value in ["planned", "planning", "planner", "plannest"] {
			assert!(plan.contains(value), "{value}");
		}
		let safe = direct_variant_literals(&english("safe"));
		assert!(safe.contains("safing"));
		assert!(safe.contains("safable"));
		assert!(direct_variant_literals(&english("wolf")).contains("wolves"));
	}

	/// 验证大写末字母的直接变体规则
	#[test]
	fn uppercase_direct_variants_cover_documented_rules() {
		let variants = direct_variant_literals(&english("TRY"));
		for value in [
			"TRYs", "TRYS", "TRYES", "TRIES", "TRYD", "TRYED", "TRIED", "TRYING", "TRYLY", "TRILY",
			"TRYER", "TRIER", "TRYEST", "TRIEST", "TRYMENT", "TRYNESS", "TRINESS", "TRYABLE",
			"TRY'", "TRY's", "TRY'S", "TRY'm", "TRY'M", "TRY'T", "TRY'RE",
		] {
			assert!(variants.contains(value), "{value}");
		}
	}

	/// 验证不足4码的词不能成为基本形式
	#[test]
	fn graph_respects_inflection_start_length() {
		let esdb = build_esdb(esdb_lines(&["cat", "cats", "plan", "planned"]), "test");
		let frequencies = ["cat", "cats", "plan", "planned"]
			.into_iter()
			.map(|value| (lower(value), freq(0.5)))
			.collect::<HashMap<_, _>>();
		let mut nodes = build_nodes(&esdb, &frequencies);
		attach_direct_bases(&mut nodes);
		let ids = nodes
			.iter()
			.enumerate()
			.map(|(node_id, node)| (node.text.as_str(), node_id))
			.collect::<HashMap<_, _>>();

		let cats_id = ids["cats"];
		assert!(nodes[cats_id].direct_bases.is_empty());
		let plan_id = ids["plan"];
		let planned_id = ids["planned"];
		assert!(nodes[planned_id].direct_bases.contains(&plan_id));
	}

	/// 验证菱形路径只计算一次唯一基本形式
	#[test]
	fn scoring_deduplicates_diamond_paths() {
		let nodes = vec![
			node("PRES", 10.0, vec![]),
			node("PRESS", 8.0, vec![0]),
			node("PRESSED", 1.0, vec![0, 1]),
		];
		let all_bases = collect_all_bases(&nodes);
		assert_eq!(all_bases[2], HashSet::from([0, 1]));

		let scores = score_nodes(&nodes);
		assert_eq!(scores[0].freq_boost, freq(19.0));
		assert_eq!(scores[1].freq_boost, freq(9.0));
		assert_eq!(scores[2].freq_boost, freq(1.0));
		assert_eq!(scores[0].demotion_count, 0);
		assert_eq!(scores[1].demotion_count, 1);
		assert_eq!(scores[2].demotion_count, 2);
	}

	/// 验证评分不会按中间节点频率剪枝
	#[test]
	fn scoring_compares_each_reachable_endpoint() {
		let nodes = vec![
			node("base", 10.0, vec![]),
			node("bases", 20.0, vec![0]),
			node("basess", 5.0, vec![1]),
		];
		let scores = score_nodes(&nodes);

		assert_eq!(scores[0].freq_boost, freq(15.0));
		assert_eq!(scores[1].freq_boost, freq(25.0));
		assert_eq!(scores[1].demotion_count, 0);
		assert_eq!(scores[2].demotion_count, 2);
	}

	/// 验证自动词完整排序优先级
	#[test]
	fn scored_words_follow_all_sort_keys() {
		let mut words = [
			(
				english("alpha"),
				WordScore {
					demotion_count: 1,
					freq_boost: freq(100.0),
				},
			),
			(
				english("be"),
				WordScore {
					demotion_count: 0,
					freq_boost: freq(5.0),
				},
			),
			(
				english("beta"),
				WordScore {
					demotion_count: 0,
					freq_boost: freq(5.0),
				},
			),
			(
				english("able"),
				WordScore {
					demotion_count: 0,
					freq_boost: freq(5.0),
				},
			),
			(
				english("cat"),
				WordScore {
					demotion_count: 0,
					freq_boost: freq(4.0),
				},
			),
		];
		words.sort_unstable_by(compare_scored_words);

		assert_eq!(
			words
				.iter()
				.map(|(text, _)| text.as_str())
				.collect::<Vec<_>>(),
			vec!["be", "able", "beta", "cat", "alpha"]
		);
	}

	/// 验证最低词频过滤和大小写分类
	#[test]
	fn automatic_words_filter_minimum_frequency() {
		let esdb = build_esdb(esdb_lines(&["apple", "tiny"]), "test");
		let frequencies =
			HashMap::from([(lower("apple"), freq(2e-7)), (lower("tiny"), freq(0.5e-7))]);
		let automatic = build_automatic_words(&esdb, &frequencies);

		assert_eq!(
			english_strings(
				automatic
					.get(&WordCase::Lower)
					.expect("missing lowercase automatic words")
			),
			vec!["apple"]
		);
		assert!(
			automatic
				.values()
				.flatten()
				.all(|text| text.lower() != lower("tiny"))
		);
	}

	/// 验证各大小写分类按前置、自动、后置顺序合并
	#[test]
	fn merged_words_follow_case_and_source_priority() {
		let automatic = HashMap::from([
			(WordCase::Lower, vec![english("middle")]),
			(WordCase::LowerMixed, vec![english("iPhone")]),
			(WordCase::Capitalized, vec![english("Middle")]),
			(WordCase::Upper, vec![english("MID")]),
		]);
		let first = HashMap::from([
			(WordCase::Lower, vec![english("first")]),
			(WordCase::Capitalized, vec![english("First")]),
		]);
		let last = HashMap::from([
			(WordCase::Lower, vec![english("last")]),
			(WordCase::Capitalized, vec![english("Last")]),
		]);

		let words = merge_en_words(automatic, &first, &last);
		assert_eq!(
			text_strings(&words),
			vec![
				"first", "middle", "last", "iPhone", "First", "Middle", "Last", "MID"
			]
		);
	}

	/// 验证自定义词与自动词冲突时统一报错
	#[test]
	#[should_panic(expected = "github, javascript")]
	fn merged_words_reject_generated_conflicts() {
		let automatic = HashMap::from([(
			WordCase::Lower,
			vec![english("github"), english("javascript")],
		)]);
		let first = HashMap::from([(
			WordCase::Lower,
			vec![english("javascript"), english("github")],
		)]);

		let _ = merge_en_words(automatic, &first, &HashMap::new());
	}

	/// 验证内存夹具能够完成全部英文词典处理阶段
	#[test]
	fn synthetic_pipeline_builds_complete_english_words() {
		let esdb = build_esdb(esdb_lines(&["apple", "plan", "planned"]), "test-esdb");
		let frequencies = HashMap::from([
			(lower("apple"), freq(5e-6)),
			(lower("plan"), freq(5e-6)),
			(lower("planned"), freq(1e-6)),
		]);
		let automatic = build_automatic_words(&esdb, &frequencies);
		let first = build_custom_words(["custom".to_owned()], "test-first");
		let last = build_custom_words(["tail".to_owned()], "test-last");

		let words = merge_en_words(automatic, &first, &last);
		assert_eq!(
			text_strings(&words),
			vec![
				"custom", "plan", "apple", "planned", "tail", "Plan", "Apple", "Planned", "PLAN",
				"APPLE", "PLANNED"
			]
		);
	}

	/// 验证每个英文词按全部非空前缀入表
	#[test]
	fn english_dictionary_indexes_every_nonempty_prefix() {
		let words = [Text::from("Apple".to_owned()), Text::from("App".to_owned())];
		let dictionary = build_en_dict(&words);

		for prefix in ["A", "Ap"] {
			assert_eq!(
				text_strings(&dictionary[&Code::from(prefix.to_owned())]),
				vec!["Apple", "App"]
			);
		}
		assert_eq!(
			text_strings(&dictionary[&Code::from("App".to_owned())]),
			vec!["Apple", "App"]
		);
		assert_eq!(
			text_strings(&dictionary[&Code::from("Appl".to_owned())]),
			vec!["Apple"]
		);
		assert_eq!(
			text_strings(&dictionary[&Code::from("Apple".to_owned())]),
			vec!["Apple"]
		);
		assert!(!dictionary.contains_key(&Code::from(String::new())));
	}
}
