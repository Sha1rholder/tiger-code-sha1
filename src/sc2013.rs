use crate::Text;
use crate::prelude::parse_target;
use std::collections::HashSet;
use std::sync::LazyLock;

/// 合法中文字符表
pub(crate) static SC2013: LazyLock<HashSet<Text>> = LazyLock::new(|| {
	parse_target("src/data/SC2013/level-1.txt")
		.into_iter()
		.chain(parse_target("src/data/SC2013/level-2.txt"))
		.chain(parse_target("src/data/SC2013/level-3.txt"))
		.chain(parse_target("src/data/SC2013/custom.txt"))
		.map(Text::from)
		.collect()
});

/// 判断文本是否含有SC2013之外的字符
pub(crate) fn contains_non_sc2013_char(text: &str, sc2013: &HashSet<Text>) -> bool {
	text.chars()
		.any(|character| !sc2013.contains(&Text::from(character.to_string())))
}
