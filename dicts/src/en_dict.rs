//! 生成英文词典[`EN_DICT_LONG`]
//!
//! 读取ESDB、英文词频与自定义前后加词，构建变体图并评分后按ESDB和最终词频过滤，再按编码分组

use crate::prelude::parse_target;
use crate::{Code, Freq, Text};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 原始词频
type FreqSelf = Freq;
/// 提权并降权后的最终词频
type FreqFinal = Freq;
/// 图节点编号
type NodeId = usize;
/// 降权次数
type DemotionCount = usize;

/// 作为基本形式参与构图的最短码长
const INFLECT_START: usize = 4;
/// 允许进入英文词典的最低词频
const MIN_FREQ: Freq = Freq::from(1e-8);
/// 降权因子
///
/// 每被降权一次，最终词频就除以一次[`DEMOTION_POWER`]。设为10时相当于降一级Zipf scale
const DEMOTION_POWER: usize = 10;
/// 允许双写的小写辅音
const DOUBLE_LOWER: &str = "bdgklmnprstz";
/// 允许双写的大写辅音
const DOUBLE_UPPER: &str = "BDGKLMNPRSTZ";
/// [`EN_WORDS_LONG`]中最短码长
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

/// 构造按全小写形式分组的[`ESDB`]
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

/// English Speller Database正确英文拼写数据库
static ESDB: LazyLock<HashMap<TextLower, HashSet<TextEn>>> = LazyLock::new(|| {
	let path = "data/en/ESDB.txt";
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
	let path = "data/en/wordfreq.tsv";
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
static EN_PREPEND: LazyLock<HashMap<WordCase, Vec<TextEn>>> = LazyLock::new(|| {
	let path = "data/en/prepend.txt";
	// 读取并分类英文前置加词
	build_custom_words(parse_target(path), path)
});

/// 英文后置加词
static EN_APPEND: LazyLock<HashMap<WordCase, Vec<TextEn>>> = LazyLock::new(|| {
	let path = "data/en/append.txt";
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

/// 按wordfreq原始小写形式构造的英文词图节点
#[derive(Clone, Debug)]
struct WordNode {
	text: TextEn,
	freq_self: FreqSelf,
	direct_bases: Vec<NodeId>,
}

/// 英文词评分
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WordScore {
	freq_final: FreqFinal,
}

/// 为全部英文wordfreq创建稳定小写节点，构图不依赖[`ESDB`]及大小写扩增
fn build_nodes(frequencies: &HashMap<TextLower, FreqSelf>) -> Vec<WordNode> {
	let mut nodes = frequencies
		.iter()
		.map(|(lower, &freq_self)| WordNode {
			text: lower.lower_form(),
			freq_self,
			direct_bases: Vec::new(),
		})
		.collect::<Vec<_>>();

	// 按码长和字面顺序建立稳定拓扑顺序
	nodes.sort_unstable_by(|left, right| {
		left.text
			.len()
			.cmp(&right.text.len())
			.then_with(|| left.text.cmp(&right.text))
	});
	assert!(
		nodes.windows(2).all(|pair| pair[0].text != pair[1].text),
		"English graph texts must be unique"
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
	let mut demotion_counts: Vec<DemotionCount> = vec![0; nodes.len()];
	// 每个节点先保留自身原始词频
	let mut scores = nodes
		.iter()
		.map(|node| WordScore {
			freq_final: node.freq_self,
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
				scores[base_id].freq_final += nodes[variant_id].freq_self;
				demotion_counts[variant_id] += 1;
			}
		}
	}

	// 提权全部完成后，每次降权将整个词频除以[`DEMOTION_POWER`]
	for (score, demotion_count) in scores.iter_mut().zip(demotion_counts) {
		for _ in 0..demotion_count {
			score.freq_final = score.freq_final.divide(DEMOTION_POWER);
		}
	}

	scores
}

/// 比较两个带评分英文词的最终自动排序顺序
fn compare_scored_words(
	(left_text, left_score): &(TextEn, WordScore),
	(right_text, right_score): &(TextEn, WordScore),
) -> Ordering {
	right_score
		.freq_final
		.total_cmp(&left_score.freq_final) // 最终词频
		.then_with(|| left_text.len().cmp(&right_text.len())) // 码长
		.then_with(|| left_text.cmp(right_text)) // 字面量
}

/// 构造按大小写分类和评分排序的自动英文词
fn build_automatic_words(
	esdb: &HashMap<TextLower, HashSet<TextEn>>,
	frequencies: &HashMap<TextLower, FreqSelf>,
) -> HashMap<WordCase, Vec<TextEn>> {
	// 以原始小写词构图并评分，大小写形式共享同一最终词频
	let mut nodes = build_nodes(frequencies);
	attach_direct_bases(&mut nodes);
	let scores = score_nodes(&nodes);
	let scores = nodes
		.into_iter()
		.zip(scores)
		.map(|(node, score)| (node.text.lower(), score))
		.collect::<HashMap<_, _>>();

	// 评分后按ESDB和最终词频筛选，再扩增保留词的大小写形式
	let mut groups = esdb.clone();
	groups.retain(|lower, _| {
		scores
			.get(lower)
			.is_some_and(|score| score.freq_final.total_cmp(&MIN_FREQ) != Ordering::Less)
	});
	expand_case_forms(&mut groups);
	let mut grouped = HashMap::<WordCase, Vec<(TextEn, WordScore)>>::new();
	for (lower, forms) in groups {
		let score = scores[&lower];
		for text in forms {
			grouped
				.entry(text.word_case())
				.or_default()
				.push((text, score));
		}
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
	first: &HashMap<WordCase, Vec<TextEn>>,
	mut automatic: HashMap<WordCase, Vec<TextEn>>,
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
static EN_WORDS_LONG: LazyLock<Vec<Text>> = LazyLock::new(|| {
	// 先生成自动词再合并自定义词
	let automatic = build_automatic_words(&ESDB, &EN_FREQ);
	merge_en_words(&EN_PREPEND, automatic, &EN_APPEND)
		.iter()
		.filter(|text| text.as_str().len() > MIN_LONG)
		.cloned()
		.collect()
});

/// 用英文词的所有非空前缀构造词典
fn build_en_dicts(words: &[Text]) -> HashMap<Code, Vec<Text>> {
	let mut dictionary = HashMap::<Code, Vec<Text>>::new();

	for text in words {
		for end in 1..=text.as_str().len() {
			let code = Code::from(text.as_str()[..end].to_owned());
			dictionary.entry(code).or_default().push(text.clone());
		}
	}

	dictionary
}

/// 按所有非空前缀码分组的长英文词典
pub static EN_DICT_LONG: LazyLock<HashMap<Code, Vec<Text>>> =
	LazyLock::new(|| build_en_dicts(&EN_WORDS_LONG));
