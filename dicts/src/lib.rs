//! `tiger_sha1`词典生成库
//!
//! 声明全部模块，提供词典生成主入口`run()`

pub mod en_dict;
pub mod pinyin;
mod prelude;
mod yaml;
mod zh_chars;
pub mod zh_dict;
pub mod zh_en_dict;

pub use prelude::{Code, Freq, Text, Weight};

/// 输出全部词典文件
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	yaml::write_dicts(OUTPUT_PY_DICT, OUTPUT_ZH_DICT)?;
	Ok(())
}
