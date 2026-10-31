//! 输出YAML产物

pub mod en_dict;
pub mod pinyin;
mod prelude;
mod tiger_dict;
mod zh_chars;
pub mod zh_dict;
pub mod zh_en_dict;

pub use prelude::{CodeChar, CodeNode, Freq, NodeId, Text, Weight};

use crate::pinyin::PINYIN_DICT;
use crate::zh_en_dict::ZH_EN_DICT;
use std::cmp::Reverse;
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

/// 按编码字典序遍历节点，保留节点内候选顺序，写为带YAML头的TSV
fn write_tsv(writer: &mut impl Write, head: &str, nodes: &[CodeNode]) -> io::Result<()> {
	writeln!(writer, "{head}")?;
	// 字符串路径仅在写文件时生成，深度优先遍历复用同一个缓冲区
	let mut code = String::new();
	let mut pending = vec![(0, 0)];
	while let Some((node_id, parent_len)) = pending.pop() {
		let node = &nodes[node_id];
		code.truncate(parent_len);
		if let Some(parent) = node.parent {
			code.push(char::from(nodes[parent].children[&node_id]));
		}
		for text in node.texts() {
			writeln!(writer, "{code}\t{}", text.as_str())?;
		}

		// 子节点按添加顺序存储，输出时临时按字符倒序入栈，保证弹出顺序为字典序
		let mut children = node.children.iter().collect::<Vec<_>>();
		children.sort_unstable_by_key(|(_, character)| Reverse(**character));
		pending.extend(children.into_iter().map(|(&child, _)| (child, code.len())));
	}
	Ok(())
}

/// 输出拼音反查和中英混合词典到Rime用户目录
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("dicts must be inside the Rime user directory");
	// 先完成数据加载和校验，再打开输出文件
	let pinyin = &*PINYIN_DICT;
	let dictionary = &*ZH_EN_DICT;
	{
		// 将拼音节点写入文件并刷新缓冲区
		let mut writer = BufWriter::new(File::create(output_dir.join(PINYIN_LOOKUP_FILE))?);
		write_tsv(&mut writer, PINYIN_LOOKUP_HEAD, pinyin)?;
		writer.flush()?;
	}

	{
		// 将中英混合节点写入文件并刷新缓冲区
		let mut writer = BufWriter::new(File::create(output_dir.join(TIGER_CODE_SHA1_FILE))?);
		write_tsv(&mut writer, TIGER_CODE_SHA1_HEAD, dictionary)?;
		writer.flush()?;
	}
	Ok(())
}
