/// 生成中英混合词典
use crate::en::EN_DICT;
use crate::zh_raw::ZH_WORDS;
use crate::{Code, Text};
use std::collections::HashMap;
use std::sync::LazyLock;

/// 合并中文候选词和英文词典
fn merge_zh_dict(
	zh_words: &HashMap<Code, Vec<Text>>,
	en_dict: &HashMap<Code, Vec<Text>>,
) -> HashMap<Code, Vec<Text>> {
	let mut dictionary = zh_words.clone();

	for (code, texts) in en_dict {
		dictionary
			.entry(code.clone())
			.or_default()
			.extend(texts.iter().cloned());
	}

	dictionary
}

/// 按编码分组的中英混合词典
pub static ZH_DICT: LazyLock<HashMap<Code, Vec<Text>>> =
	LazyLock::new(|| merge_zh_dict(&ZH_WORDS, &EN_DICT));

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

	/// 验证合并保留来源顺序、独有编码和重复候选
	#[test]
	fn merged_dictionary_preserves_order_codes_and_duplicates() {
		let zh_words = HashMap::from([
			(code("a"), vec![text("甲"), text("乙")]),
			(code("z"), vec![text("中文")]),
		]);
		let en_dict = HashMap::from([
			(code("a"), vec![text("apple"), text("甲")]),
			(code("e"), vec![text("English")]),
		]);

		let dictionary = merge_zh_dict(&zh_words, &en_dict);

		assert_eq!(
			dictionary.get(&code("a")),
			Some(&vec![text("甲"), text("乙"), text("apple"), text("甲")])
		);
		assert_eq!(dictionary.get(&code("z")), Some(&vec![text("中文")]));
		assert_eq!(dictionary.get(&code("e")), Some(&vec![text("English")]));
	}
}
