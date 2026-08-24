pub mod en;
mod prelude;
pub mod py;
mod sc2013;
mod yaml;
pub mod zh;
pub mod zh_raw;

pub use prelude::{Code, Freq, Text, Weight};

/// 输出全部词典文件
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	yaml::write_dicts()?;
	Ok(())
}
