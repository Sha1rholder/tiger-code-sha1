//! 生成中英混合词典[`ZH_EN_DICT`]
//!
//! 合并[`ZH_WORDS_DICT`]与[`EN_DICT_LONG`]，英文独有编码以编码本身为首选候选

use crate::en_dict::EN_DICT_LONG;
use crate::zh_dict::ZH_WORDS_DICT;
use crate::{Code, Text};
use std::collections::{HashMap, hash_map::Entry};
use std::sync::LazyLock;

/// 按编码分组的中英混合词典
pub static ZH_EN_DICT: LazyLock<HashMap<Code, Vec<Text>>> = LazyLock::new(|| {
	let mut dictionary = (*ZH_WORDS_DICT).clone();

	for (code, texts) in EN_DICT_LONG.iter() {
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
});
