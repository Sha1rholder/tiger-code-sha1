//! 生成中英混合词典`ZH_DICT`
//!
//! 合并中文候选表与长英文词典，英文独有编码以编码本身为首选候选

/// 生成中英混合词典
use crate::en_dict::EN_DICT_LONG;
use crate::zh_dict::ZH_WORDS;
use crate::{Code, Text};
use std::collections::{HashMap, hash_map::Entry};
use std::sync::LazyLock;

/// 合并中文候选词和英文词典
fn merge_zh_dict(
	zh_words: &HashMap<Code, Vec<Text>>,
	en_dict: &HashMap<Code, Vec<Text>>,
) -> HashMap<Code, Vec<Text>> {
	let mut dictionary = zh_words.clone();

	for (code, texts) in en_dict {
		match dictionary.entry(code.clone()) {
			Entry::Occupied(mut entry) => entry.get_mut().extend(texts.iter().cloned()),
			Entry::Vacant(entry) => {
				let mut texts = texts
					.iter()
					.filter(|text| text.as_str() != code.as_str())
					.cloned()
					.collect::<Vec<_>>();
				texts.insert(0, Text::from(code.as_str().to_owned()));
				entry.insert(texts);
			}
		}
	}

	dictionary
}

/// 按编码分组的中英混合词典
pub static ZH_DICT: LazyLock<HashMap<Code, Vec<Text>>> =
	LazyLock::new(|| merge_zh_dict(&ZH_WORDS, &EN_DICT_LONG));
