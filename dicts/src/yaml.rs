//! 词典文件序列化
//!
//! 将拼音反查、中英混合与英文词典写为带YAML头的TSV文件

use crate::pinyin::ZH_PY;
use crate::zh_en_dict::ZH_DICT;
use crate::{Code, Text};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};

/// 拼音反查词典文件
const PINYIN_LOOKUP_FILE: &str = "pinyin_lookup.dict.yaml";
/// 主词典文件
const TIGER_CODE_SHA1_FILE: &str = "tiger_code_sha1.dict.yaml";
/// 拼音反查词典文件头
const PINYIN_LOOKUP_HEAD: &str = "---
name: pinyin_lookup
version: placeholder
sort: original
columns:
  - code
  - text
...";
/// 中英混合词典文件头
const TIGER_CODE_SHA1_HEAD: &str = "---
name: tiger_code_sha1
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
	Ok(())
}
