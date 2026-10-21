//! 生成应用重编码后的虎码字符表[`TIGER_CHARS_DICT`]
//!
//! 读取虎码原始数据，按字符集过滤并选择编码，再应用重编码与按权重分组

use crate::prelude::{group_texts_by_code, parse_target};
use crate::zh_chars::ZH_CHARS;
use crate::{Code, Text, Weight};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// 单个字符的多个虎码及其权重
type TigerCodes = HashMap<Code, Weight>;
/// 虎码原始数据
type TigerEntries = HashMap<Text, TigerCodes>;
/// 已选虎码及其优先级
type TigerChar = (Code, Weight);
/// 已筛选虎码
type TigerCharsRaw = HashMap<Text, TigerChar>;

/// 虎码字符表
pub(crate) static TIGER_CHARS_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	let path = "data/zh/tiger-code/tiger.dict.yaml.tsv";
	let mut lines = parse_target(path).into_iter();
	// 校验并跳过TSV标题
	assert_eq!(
		lines.next().as_deref(),
		Some("text\tcode\tweight"),
		"invalid tiger character header in {path}"
	);
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

		let previous = tiger_raw
			.entry(Text::from(text.to_owned()))
			.or_default()
			.insert(Code::from(code.to_owned()), Weight::from(weight));
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
		.flat_map(|(text, codes)| {
			codes
				.into_iter()
				.map(move |(code, weight)| (text.clone(), code, weight))
		})
		.collect::<Vec<_>>();
	weighted_entries.sort_unstable_by(
		|(left_text, left_code, left_weight), (right_text, right_code, right_weight)| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
				.then_with(|| left_code.cmp(right_code))
		},
	);

	let mut tiger_chars = TigerCharsRaw::new();
	// 记录当前HashMap中每个Code的使用次数
	let mut current_codes = HashMap::<Code, usize>::new();

	for (text, code, weight) in weighted_entries {
		// 新Text按遍历顺序直接插入
		if !tiger_chars.contains_key(&text) {
			*current_codes.entry(code.clone()).or_default() += 1;
			let _ = tiger_chars.insert(text, (code, weight));
			continue;
		}

		let current_code = tiger_chars
			.get(&text)
			.expect("existing text must have a code")
			.0
			.clone();
		// 仅接受更短且当前未占用的Code
		if code.len() >= current_code.len() || current_codes.contains_key(&code) {
			continue;
		}

		// 释放旧Code的当前使用次数
		let old_code = current_code;
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
		tiger_chars
			.get_mut(&text)
			.expect("existing text must have a code")
			.0 = code;
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
	// 读取Text到Code的重编码表
	let tiger_recode = parse_target("data/zh/recode.tsv")
		.into_iter()
		.skip(1)
		.map(|line| {
			let mut fields = line.split('\t');
			let code = fields.next().expect("missing code field");
			let text = fields.next().expect("missing text field");

			(Text::from(text.to_owned()), Code::from(code.to_owned()))
		})
		// 已从字符表移除的字不再应用重编码
		.filter(|(text, _)| ZH_CHARS.contains(text))
		.collect::<HashMap<_, _>>();
	// 应用重编码
	// 重编码键必须来自原始字符表
	assert!(
		tiger_recode
			.keys()
			.all(|text| tiger_chars.contains_key(text)),
		"tiger recode keys must be a subset of tiger raw keys"
	);

	// 原位替换字符编码，保留原始优先级
	for (text, code) in tiger_recode {
		tiger_chars
			.get_mut(&text)
			.expect("validated recode text must exist")
			.0 = code;
	}

	// 按Code分组，组内按原始Weight降序排列
	let mut entries = tiger_chars.into_iter().collect::<Vec<_>>();
	entries.sort_unstable_by(
		|(left_text, (_, left_weight)), (right_text, (_, right_weight))| {
			right_weight
				.cmp(left_weight)
				.then_with(|| left_text.cmp(right_text))
		},
	);

	group_texts_by_code(entries.into_iter().map(|(text, (code, _))| (code, text)))
});
