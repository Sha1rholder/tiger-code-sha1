mod data;
pub mod prelude;
pub mod zh;
#[path = "zh-custom.rs"]
mod zh_custom;

use crate::prelude::{Code, Text, Weight};
use std::collections::HashMap;

/// 生成单字权重映射、虎码单字表和虎码加字加词表
fn build_zh() -> (HashMap<Text, Weight>, Vec<(Code, Text)>, Vec<(Code, Text)>) {
	let (weight_by_text, tiger_chars) = zh::dedup::tiger_chars();
	let tiger_custom = zh_custom::tiger_custom();
	(weight_by_text, tiger_chars, tiger_custom)
}

/// 执行词表生成流程
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	let _ = build_zh();
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	/// 验证中文编排结果独立保留虎码单字表和加字加词表
	#[test]
	fn builds_separate_tiger_and_custom_rows() {
		let (expected_weights, tiger_chars) = zh::dedup::tiger_chars();
		let (actual_weights, actual_tiger_chars, tiger_custom) = build_zh();

		assert_eq!(actual_weights, expected_weights);
		assert_eq!(actual_tiger_chars, tiger_chars);
		assert_eq!(tiger_custom.as_slice(), data::ZH_CUSTOM.as_slice());
	}
}
