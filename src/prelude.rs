use std::hash::{Hash, Hasher};

/// 文本
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Text(String);
impl From<String> for Text {
	fn from(value: String) -> Self {
		Self(value)
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
	/// 返回编码的字节长度
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

/// 权重
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Weight(u32);
impl Weight {
	pub fn from(value: u32) -> Self {
		Self(value)
	}
}

/// wordfreq
#[derive(Clone, Copy, Debug)]
pub struct Freq(f64);
impl Freq {
	pub fn from(value: f64) -> Self {
		Self(value)
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
