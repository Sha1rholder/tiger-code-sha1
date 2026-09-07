//! 中文字符表与集外字符判定，输出[`ZH_CHARS`]

use crate::Text;
use crate::prelude::parse_target;
use std::collections::HashSet;
use std::sync::LazyLock;

/// 合法中文字符表
pub(crate) static ZH_CHARS: LazyLock<HashSet<Text>> = LazyLock::new(|| {
	parse_target("data/zh/chars/level-1.txt")
		.into_iter()
		.chain(parse_target("data/zh/chars/level-2.txt"))
		.chain(parse_target("data/zh/chars/level-3.txt"))
		.chain(parse_target("data/zh/chars/extra.txt"))
		.map(Text::from)
		.collect()
});

/// 判断文本是否含有不在中文字符表中的字符
pub(crate) fn contains_illegal_zh_char(text: &str, zh_chars: &HashSet<Text>) -> bool {
	text.chars()
		.any(|character| !zh_chars.contains(&Text::from(character.to_string())))
}
