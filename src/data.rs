use std::fs;
use std::path::Path;

/// 跳过空行和`#`开头的行，读取单个文本文件为列表
pub(crate) fn parse_target(path: impl AsRef<Path>) -> Vec<String> {
	let path = path.as_ref();
	let content = fs::read_to_string(path)
		.unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

	content
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(str::to_owned)
		.collect()
}
