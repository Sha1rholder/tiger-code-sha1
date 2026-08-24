use std::fs;
use std::hash::{Hash, Hasher};
use std::ops::AddAssign;
use std::path::Path;

/// 文本
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Text(String);
impl From<String> for Text {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl Text {
	/// 返回底层字符串切片
	pub(crate) fn as_str(&self) -> &str {
		&self.0
	}
}

/// 编码
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Code(String);
impl From<String> for Code {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl Code {
	/// 返回底层字符串切片
	pub(crate) fn as_str(&self) -> &str {
		&self.0
	}

	/// 返回编码的字节长度
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

/// 权重
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Weight(usize);
impl Weight {
	/// 创建权重
	pub fn from(value: usize) -> Self {
		Self(value)
	}
}

/// wordfreq
#[derive(Clone, Copy, Debug)]
pub struct Freq(f64);
impl Freq {
	/// 从f64创建词频
	pub const fn from(value: f64) -> Self {
		Self(value)
	}

	/// 按整数倍缩放词频
	pub(crate) fn scale(self, factor: usize) -> Self {
		Self(self.0 * factor as f64)
	}

	/// 按浮点总序比较词频
	pub(crate) fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.total_cmp(&other.0)
	}
}
impl AddAssign for Freq {
	/// 累加词频
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
	}
}
impl PartialEq for Freq {
	fn eq(&self, other: &Self) -> bool {
		self.0.to_bits() == other.0.to_bits()
	}
}
impl Eq for Freq {}
impl Hash for Freq {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.to_bits().hash(state);
	}
}

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
