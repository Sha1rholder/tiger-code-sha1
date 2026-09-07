//! `tiger_code_sha1`词典生成库
//!
//! 将拼音反查与中英混合词典写为带YAML头的TSV，提供生成入口[`run()`]

pub mod en_dict;
pub mod pinyin;
mod prelude;
mod zh_chars;
pub mod zh_dict;
pub mod zh_en_dict;

pub use prelude::{Code, Freq, Text, Weight};

use crate::pinyin::ZH_PY;
use crate::zh_en_dict::ZH_EN_DICT;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// 拼音反查词典文件
const PINYIN_LOOKUP_FILE: &str = "pinyin_lookup.dict.yaml";
/// 主词典文件
const TIGER_CODE_SHA1_FILE: &str = "tiger_code_sha1.dict.yaml";
/// 拼音反查词典文件头
const PINYIN_LOOKUP_HEAD: &str = concat!(
	"---
name: pinyin_lookup
version: \"",
	env!("CARGO_PKG_VERSION"),
	"\"
sort: original
columns:
  - code
  - text
..."
);
/// 中英混合词典文件头
const TIGER_CODE_SHA1_HEAD: &str = concat!(
	"---
name: tiger_code_sha1
version: \"",
	env!("CARGO_PKG_VERSION"),
	"\"
sort: original
columns:
  - code
  - text
..."
);

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
	groups.sort_unstable_by_key(|(code, _)| *code);
	write_tsv(
		writer,
		head,
		groups
			.into_iter()
			.flat_map(|(code, texts)| texts.iter().map(move |text| (code, text))),
	)
}

/// 将有序条目写入文件并刷新缓冲区
fn write_entries_file(path: &Path, head: &str, entries: &[(Code, Text)]) -> io::Result<()> {
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
	path: &Path,
	head: &str,
	dictionary: &HashMap<Code, Vec<Text>>,
) -> io::Result<()> {
	let mut writer = BufWriter::new(File::create(path)?);
	write_dictionary_tsv(&mut writer, head, dictionary)?;
	writer.flush()
}

/// 输出拼音反查和中英混合词典到Rime用户目录
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("dicts must be inside the Rime user directory");
	// 先完成数据加载和校验，再打开输出文件
	let pinyin = &*ZH_PY;
	let dictionary = &*ZH_EN_DICT;
	write_entries_file(
		&output_dir.join(PINYIN_LOOKUP_FILE),
		PINYIN_LOOKUP_HEAD,
		pinyin,
	)?;
	write_dictionary_file(
		&output_dir.join(TIGER_CODE_SHA1_FILE),
		TIGER_CODE_SHA1_HEAD,
		dictionary,
	)?;
	Ok(())
}
