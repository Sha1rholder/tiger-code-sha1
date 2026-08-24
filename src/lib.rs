pub mod en;
mod prelude;
pub mod py;
mod sc2013;
mod yaml;
pub mod zh;
pub mod zh_raw;

pub use prelude::{Code, Freq, Text, Weight};

/// 是否输出拼音反查词典
pub const OUTPUT_PY_DICT: bool = true;
/// 是否输出中英混合词典
pub const OUTPUT_ZH_DICT: bool = true;
/// 是否输出英文词典
pub const OUTPUT_EN_DICT: bool = false;

/// 输出全部词典文件
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	yaml::write_dicts(OUTPUT_PY_DICT, OUTPUT_ZH_DICT, OUTPUT_EN_DICT)?;
	Ok(())
}
