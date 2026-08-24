use crate::en::EN_DICT;
use crate::py::ZH_PY;
use crate::zh::ZH_DICT;
use crate::{Code, Text};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};

/// 拼音反查词典文件
const PY_FILE: &str = "tiger_sha1_py.dict.yaml";
/// 中英混合词典文件
const ZH_FILE: &str = "tiger_sha1_zh.dict.yaml";
/// 英文词典文件
const EN_FILE: &str = "tiger_sha1_en.dict.yaml";
/// 拼音反查词典文件头
const PY_HEAD: &str = "---
name: tiger_sha1_py
version: placeholder
sort: original
columns:
  - code
  - text
...";
/// 中英混合词典文件头
const ZH_HEAD: &str = "---
name: tiger_sha1_zh
version: placeholder
sort: original
columns:
  - code
  - text
...";
/// 英文词典文件头
const EN_HEAD: &str = "---
name: tiger_sha1_en
version: placeholder
sort: original
columns:
  - code
  - text
...";

/// 将有序条目写为带YAML头的TSV
fn write_tsv<'a>(
	writer: &mut impl Write,
	head: &str,
	entries: impl IntoIterator<Item = (&'a Code, &'a Text)>,
) -> io::Result<()> {
	writeln!(writer, "{head}")?;
	for (code, text) in entries {
		writeln!(writer, "{}\t{}", code.as_str(), text.as_str())?;
	}
	Ok(())
}

/// 将分组词典按编码排序后写为TSV
fn write_dictionary_tsv(
	writer: &mut impl Write,
	head: &str,
	dictionary: &HashMap<Code, Vec<Text>>,
) -> io::Result<()> {
	let mut groups = dictionary.iter().collect::<Vec<_>>();
	groups.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
	write_tsv(
		writer,
		head,
		groups
			.into_iter()
			.flat_map(|(code, texts)| texts.iter().map(move |text| (code, text))),
	)
}

/// 将有序条目写入文件并刷新缓冲区
fn write_entries_file(path: &str, head: &str, entries: &[(Code, Text)]) -> io::Result<()> {
	let mut writer = BufWriter::new(File::create(path)?);
	write_tsv(
		&mut writer,
		head,
		entries.iter().map(|(code, text)| (code, text)),
	)?;
	writer.flush()
}

/// 将分组词典写入文件并刷新缓冲区
fn write_dictionary_file(
	path: &str,
	head: &str,
	dictionary: &HashMap<Code, Vec<Text>>,
) -> io::Result<()> {
	let mut writer = BufWriter::new(File::create(path)?);
	write_dictionary_tsv(&mut writer, head, dictionary)?;
	writer.flush()
}

/// 输出全部词典文件
pub(crate) fn write_dicts(
	output_py_dict: bool,
	output_zh_dict: bool,
	output_en_dict: bool,
) -> io::Result<()> {
	if output_py_dict {
		write_entries_file(PY_FILE, PY_HEAD, &ZH_PY)?;
	}
	if output_zh_dict {
		write_dictionary_file(ZH_FILE, ZH_HEAD, &ZH_DICT)?;
	}
	if output_en_dict {
		write_dictionary_file(EN_FILE, EN_HEAD, &EN_DICT)?;
	}
	Ok(())
}

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

	/// 验证YAML头、制表符和输入顺序
	#[test]
	fn tsv_preserves_entry_order_and_format() {
		let entries = [(code("b"), text("乙")), (code("a"), text("甲"))];
		let mut output = Vec::new();

		write_tsv(
			&mut output,
			"head",
			entries.iter().map(|(code, text)| (code, text)),
		)
		.expect("TSV writing must succeed");

		assert_eq!(String::from_utf8(output).unwrap(), "head\nb\t乙\na\t甲\n");
	}

	/// 验证分组词典按编码排序并保留候选顺序和重复项
	#[test]
	fn dictionary_tsv_sorts_codes_and_preserves_candidates() {
		let dictionary = HashMap::from([
			(code("b"), vec![text("乙")]),
			(code("a"), vec![text("甲"), text("甲"), text("啊")]),
		]);
		let mut output = Vec::new();

		write_dictionary_tsv(&mut output, "head", &dictionary).expect("TSV writing must succeed");

		assert_eq!(
			String::from_utf8(output).unwrap(),
			"head\na\t甲\na\t甲\na\t啊\nb\t乙\n"
		);
	}
}
