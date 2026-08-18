/// 汉字去重编码表处理模块
pub mod dedup {
	use crate::data::{SC2013, TIGER, ZH_RECODE};
	use crate::prelude::{Code, Text, Weight};
	use std::collections::{HashMap, HashSet};

	/// 返回单字权重映射和过滤并单一化编码后的虎码条目
	pub fn tiger_chars() -> (HashMap<Text, Weight>, Vec<(Code, Text)>) {
		filter_tiger(&TIGER, &SC2013, &ZH_RECODE)
	}

	/// 按权重和上游顺序生成单字权重映射与虎码条目
	fn filter_tiger(
		upstream_tiger: &[(Code, Text, Weight)],
		sc2013: &HashSet<Text>,
		zh_recodes: &[(Code, Text)],
	) -> (HashMap<Text, Weight>, Vec<(Code, Text)>) {
		let recode_by_text = validate_zh_recodes(zh_recodes, sc2013);
		let reserved_codes: HashSet<Code> = recode_by_text.values().cloned().collect();
		let recoded_texts: HashSet<Text> = recode_by_text.keys().cloned().collect();
		let mut seen_recoded_texts = HashSet::new();
		let mut selected: Vec<Option<(Code, Text, Weight)>> = Vec::new();
		let mut index_by_text = HashMap::new();
		let mut code_counts: HashMap<Code, usize> = HashMap::new();
		let mut sorted_tiger = upstream_tiger.to_vec();
		sorted_tiger.sort_by(|left, right| right.2.cmp(&left.2));

		for (upstream_code, text, weight) in sorted_tiger {
			if !text.is_single_char() || !sc2013.contains(&text) {
				continue;
			}

			let code = if let Some(recode) = recode_by_text.get(&text) {
				seen_recoded_texts.insert(text.clone());
				recode.clone()
			} else {
				if reserved_codes.contains(&upstream_code) {
					continue;
				}
				upstream_code
			};

			let Some(&current_index) = index_by_text.get(&text) else {
				index_by_text.insert(text.clone(), selected.len());
				selected.push(Some((code.clone(), text, weight)));
				*code_counts.entry(code).or_default() += 1;
				continue;
			};

			let current_code = selected[current_index]
				.as_ref()
				.expect("selected text index points to an empty row")
				.0
				.clone();
			if code.len() >= current_code.len() {
				continue;
			}
			if code_counts.get(&code).copied().unwrap_or_default() > 0 {
				continue;
			}

			let remove_current_code = {
				let count = code_counts
					.get_mut(&current_code)
					.expect("selected code is missing from code counts");
				*count -= 1;
				*count == 0
			};
			if remove_current_code {
				code_counts.remove(&current_code);
			}
			selected[current_index] = None;
			index_by_text.insert(text.clone(), selected.len());
			selected.push(Some((code.clone(), text, weight)));
			code_counts.insert(code, 1);
		}

		assert!(
			recoded_texts.is_subset(&seen_recoded_texts),
			"recode text is missing from upstream tiger data"
		);

		let mut weight_by_text = HashMap::new();
		let mut tiger_chars = Vec::new();
		for (code, text, weight) in selected.into_iter().flatten() {
			weight_by_text.insert(text.clone(), weight);
			tiger_chars.push((code, text));
		}

		(weight_by_text, tiger_chars)
	}

	/// 校验中文单字改码并返回文本到编码的映射
	fn validate_zh_recodes(
		zh_recodes: &[(Code, Text)],
		sc2013: &HashSet<Text>,
	) -> HashMap<Text, Code> {
		let mut recode_by_text = HashMap::new();
		let mut recoded_codes = HashSet::new();

		for (code, text) in zh_recodes {
			assert!(code.len() > 0, "recode code must not be empty");
			assert!(text.is_single_char(), "recode text must be one character");
			assert!(sc2013.contains(text), "recode text is not allowed");
			assert!(
				!recode_by_text.contains_key(text),
				"recode text must be unique"
			);
			assert!(
				recoded_codes.insert(code.clone()),
				"recode code must be unique"
			);
			recode_by_text.insert(text.clone(), code.clone());
		}

		recode_by_text
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		/// 构造虎码测试条目
		fn row(code: &str, text: &str, weight: u32) -> (Code, Text, Weight) {
			(
				Code::from(code.to_owned()),
				Text::from(text.to_owned()),
				Weight::from(weight),
			)
		}

		/// 构造改码测试条目
		fn recode(code: &str, text: &str) -> (Code, Text) {
			(Code::from(code.to_owned()), Text::from(text.to_owned()))
		}

		/// 构造放行文本集合
		fn allowed(texts: &[&str]) -> HashSet<Text> {
			texts
				.iter()
				.map(|text| Text::from((*text).to_owned()))
				.collect()
		}

		/// 把带权重条目拆成预期返回结构
		fn split_rows(
			rows: Vec<(Code, Text, Weight)>,
		) -> (HashMap<Text, Weight>, Vec<(Code, Text)>) {
			let mut weight_by_text = HashMap::new();
			let mut tiger_chars = Vec::new();
			for (code, text, weight) in rows {
				weight_by_text.insert(text.clone(), weight);
				tiger_chars.push((code, text));
			}

			(weight_by_text, tiger_chars)
		}

		/// 验证仅保留放行集合中的单字
		#[test]
		fn filters_non_single_and_disallowed_text() {
			let tiger = vec![
				row("a", "甲乙", 100),
				row("b", "乙", 90),
				row("c", "甲", 80),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲乙", "甲"]), &[]);

			assert_eq!(result, split_rows(vec![row("c", "甲", 80)]));
		}

		/// 验证同长或更长的后续编码不会替换首次编码
		#[test]
		fn keeps_first_code_when_later_codes_are_not_shorter() {
			let tiger = vec![
				row("ab", "甲", 100),
				row("cd", "甲", 90),
				row("abc", "甲", 80),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲"]), &[]);

			assert_eq!(result, split_rows(vec![row("ab", "甲", 100)]));
		}

		/// 验证未占用短码会替换原条目并保留短码权重和位置
		#[test]
		fn replaces_with_unoccupied_shorter_code_at_current_position() {
			let tiger = vec![
				row("abc", "甲", 100),
				row("x", "乙", 90),
				row("a", "甲", 80),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲", "乙"]), &[]);

			assert_eq!(
				result,
				split_rows(vec![row("x", "乙", 90), row("a", "甲", 80)])
			);
		}

		/// 验证已占用短码会阻止替换
		#[test]
		fn keeps_original_code_when_shorter_code_is_occupied() {
			let tiger = vec![
				row("abc", "甲", 100),
				row("a", "乙", 90),
				row("a", "甲", 80),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲", "乙"]), &[]);

			assert_eq!(
				result,
				split_rows(vec![row("abc", "甲", 100), row("a", "乙", 90)])
			);
		}

		/// 验证替换后释放的旧编码可供后续短码使用
		#[test]
		fn releases_old_code_after_replacement() {
			let tiger = vec![
				row("abc", "甲", 100),
				row("long", "乙", 90),
				row("a", "甲", 80),
				row("abc", "乙", 70),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲", "乙"]), &[]);

			assert_eq!(
				result,
				split_rows(vec![row("a", "甲", 80), row("abc", "乙", 70)])
			);
		}

		/// 验证同权重条目保持上游顺序并拥有占码优先权
		#[test]
		fn preserves_upstream_order_for_equal_weights() {
			let tiger = vec![
				row("abc", "甲", 100),
				row("a", "乙", 100),
				row("a", "甲", 100),
			];
			let result = filter_tiger(&tiger, &allowed(&["甲", "乙"]), &[]);

			assert_eq!(
				result,
				split_rows(vec![row("abc", "甲", 100), row("a", "乙", 100)])
			);
		}

		/// 验证改码覆盖上游编码并独占目标码
		#[test]
		fn applies_recodes_and_reserves_their_codes() {
			let tiger = vec![row("x", "甲", 100), row("z", "乙", 90), row("y", "乙", 80)];
			let result = filter_tiger(&tiger, &allowed(&["甲", "乙"]), &[recode("z", "甲")]);

			assert_eq!(
				result,
				split_rows(vec![row("z", "甲", 100), row("y", "乙", 80)])
			);
		}

		/// 验证空改码编码会失败
		#[test]
		#[should_panic(expected = "recode code must not be empty")]
		fn rejects_empty_recode_code() {
			filter_tiger(&[], &allowed(&["甲"]), &[recode("", "甲")]);
		}

		/// 验证多字改码文本会失败
		#[test]
		#[should_panic(expected = "recode text must be one character")]
		fn rejects_non_single_recode_text() {
			filter_tiger(&[], &allowed(&["甲乙"]), &[recode("a", "甲乙")]);
		}

		/// 验证未放行改码文本会失败
		#[test]
		#[should_panic(expected = "recode text is not allowed")]
		fn rejects_disallowed_recode_text() {
			filter_tiger(&[], &allowed(&["乙"]), &[recode("a", "甲")]);
		}

		/// 验证重复改码文本会失败
		#[test]
		#[should_panic(expected = "recode text must be unique")]
		fn rejects_duplicate_recode_text() {
			filter_tiger(
				&[],
				&allowed(&["甲"]),
				&[recode("a", "甲"), recode("b", "甲")],
			);
		}

		/// 验证重复改码编码会失败
		#[test]
		#[should_panic(expected = "recode code must be unique")]
		fn rejects_duplicate_recode_code() {
			filter_tiger(
				&[],
				&allowed(&["甲", "乙"]),
				&[recode("a", "甲"), recode("a", "乙")],
			);
		}

		/// 验证未在上游出现的改码文本会失败
		#[test]
		#[should_panic(expected = "recode text is missing from upstream tiger data")]
		fn rejects_recode_text_missing_from_upstream() {
			filter_tiger(
				&[row("a", "乙", 100)],
				&allowed(&["甲", "乙"]),
				&[recode("x", "甲")],
			);
		}

		/// 验证真实虎码数据的输出约束
		#[test]
		fn processes_real_tiger_data() {
			let (weight_by_text, tiger_chars) = tiger_chars();
			let mut seen_texts = HashSet::new();

			assert!(!tiger_chars.is_empty());
			assert_eq!(weight_by_text.len(), tiger_chars.len());
			assert!(tiger_chars.iter().all(|(_, text)| {
				text.is_single_char() && SC2013.contains(text) && seen_texts.insert(text)
			}));
			assert!(tiger_chars.windows(2).all(|rows| {
				weight_by_text
					.get(&rows[0].1)
					.expect("first text is missing its weight")
					>= weight_by_text
						.get(&rows[1].1)
						.expect("second text is missing its weight")
			}));
		}
	}
}
