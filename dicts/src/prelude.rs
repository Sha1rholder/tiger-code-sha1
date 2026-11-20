//! 项目基础类型[`Text`]、[`CodeChar`]、[`CodeNode`]、[`Weight`]、[`Freq`]
//! 与通用数据文件解析函数[`parse_target`]

use std::collections::BTreeMap;
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

/// 编码字符，变体按ASCII字典序排列，供完整路径比较使用
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CodeChar {
	/// 多音节拼音的分隔符
	Space,
	Apostrophe,
	UpperA,
	UpperB,
	UpperC,
	UpperD,
	UpperE,
	UpperF,
	UpperG,
	UpperH,
	UpperI,
	UpperJ,
	UpperK,
	UpperL,
	UpperM,
	UpperN,
	UpperO,
	UpperP,
	UpperQ,
	UpperR,
	UpperS,
	UpperT,
	UpperU,
	UpperV,
	UpperW,
	UpperX,
	UpperY,
	UpperZ,
	LowerA,
	LowerB,
	LowerC,
	LowerD,
	LowerE,
	LowerF,
	LowerG,
	LowerH,
	LowerI,
	LowerJ,
	LowerK,
	LowerL,
	LowerM,
	LowerN,
	LowerO,
	LowerP,
	LowerQ,
	LowerR,
	LowerS,
	LowerT,
	LowerU,
	LowerV,
	LowerW,
	LowerX,
	LowerY,
	LowerZ,
}

impl TryFrom<char> for CodeChar {
	type Error = char;

	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			' ' => Ok(Self::Space),
			'\'' => Ok(Self::Apostrophe),
			'A' => Ok(Self::UpperA),
			'B' => Ok(Self::UpperB),
			'C' => Ok(Self::UpperC),
			'D' => Ok(Self::UpperD),
			'E' => Ok(Self::UpperE),
			'F' => Ok(Self::UpperF),
			'G' => Ok(Self::UpperG),
			'H' => Ok(Self::UpperH),
			'I' => Ok(Self::UpperI),
			'J' => Ok(Self::UpperJ),
			'K' => Ok(Self::UpperK),
			'L' => Ok(Self::UpperL),
			'M' => Ok(Self::UpperM),
			'N' => Ok(Self::UpperN),
			'O' => Ok(Self::UpperO),
			'P' => Ok(Self::UpperP),
			'Q' => Ok(Self::UpperQ),
			'R' => Ok(Self::UpperR),
			'S' => Ok(Self::UpperS),
			'T' => Ok(Self::UpperT),
			'U' => Ok(Self::UpperU),
			'V' => Ok(Self::UpperV),
			'W' => Ok(Self::UpperW),
			'X' => Ok(Self::UpperX),
			'Y' => Ok(Self::UpperY),
			'Z' => Ok(Self::UpperZ),
			'a' => Ok(Self::LowerA),
			'b' => Ok(Self::LowerB),
			'c' => Ok(Self::LowerC),
			'd' => Ok(Self::LowerD),
			'e' => Ok(Self::LowerE),
			'f' => Ok(Self::LowerF),
			'g' => Ok(Self::LowerG),
			'h' => Ok(Self::LowerH),
			'i' => Ok(Self::LowerI),
			'j' => Ok(Self::LowerJ),
			'k' => Ok(Self::LowerK),
			'l' => Ok(Self::LowerL),
			'm' => Ok(Self::LowerM),
			'n' => Ok(Self::LowerN),
			'o' => Ok(Self::LowerO),
			'p' => Ok(Self::LowerP),
			'q' => Ok(Self::LowerQ),
			'r' => Ok(Self::LowerR),
			's' => Ok(Self::LowerS),
			't' => Ok(Self::LowerT),
			'u' => Ok(Self::LowerU),
			'v' => Ok(Self::LowerV),
			'w' => Ok(Self::LowerW),
			'x' => Ok(Self::LowerX),
			'y' => Ok(Self::LowerY),
			'z' => Ok(Self::LowerZ),
			other => Err(other),
		}
	}
}

impl From<CodeChar> for char {
	fn from(value: CodeChar) -> Self {
		match value {
			CodeChar::Space => ' ',
			CodeChar::Apostrophe => '\'',
			CodeChar::UpperA => 'A',
			CodeChar::UpperB => 'B',
			CodeChar::UpperC => 'C',
			CodeChar::UpperD => 'D',
			CodeChar::UpperE => 'E',
			CodeChar::UpperF => 'F',
			CodeChar::UpperG => 'G',
			CodeChar::UpperH => 'H',
			CodeChar::UpperI => 'I',
			CodeChar::UpperJ => 'J',
			CodeChar::UpperK => 'K',
			CodeChar::UpperL => 'L',
			CodeChar::UpperM => 'M',
			CodeChar::UpperN => 'N',
			CodeChar::UpperO => 'O',
			CodeChar::UpperP => 'P',
			CodeChar::UpperQ => 'Q',
			CodeChar::UpperR => 'R',
			CodeChar::UpperS => 'S',
			CodeChar::UpperT => 'T',
			CodeChar::UpperU => 'U',
			CodeChar::UpperV => 'V',
			CodeChar::UpperW => 'W',
			CodeChar::UpperX => 'X',
			CodeChar::UpperY => 'Y',
			CodeChar::UpperZ => 'Z',
			CodeChar::LowerA => 'a',
			CodeChar::LowerB => 'b',
			CodeChar::LowerC => 'c',
			CodeChar::LowerD => 'd',
			CodeChar::LowerE => 'e',
			CodeChar::LowerF => 'f',
			CodeChar::LowerG => 'g',
			CodeChar::LowerH => 'h',
			CodeChar::LowerI => 'i',
			CodeChar::LowerJ => 'j',
			CodeChar::LowerK => 'k',
			CodeChar::LowerL => 'l',
			CodeChar::LowerM => 'm',
			CodeChar::LowerN => 'n',
			CodeChar::LowerO => 'o',
			CodeChar::LowerP => 'p',
			CodeChar::LowerQ => 'q',
			CodeChar::LowerR => 'r',
			CodeChar::LowerS => 's',
			CodeChar::LowerT => 't',
			CodeChar::LowerU => 'u',
			CodeChar::LowerV => 'v',
			CodeChar::LowerW => 'w',
			CodeChar::LowerX => 'x',
			CodeChar::LowerY => 'y',
			CodeChar::LowerZ => 'z',
		}
	}
}

/// 节点在所属Vec中的下标，0固定为根节点，清理压缩后需要重新取得id
pub type NodeId = usize;

/// 编码节点，完整路径由父节点和自身字符共同确定
#[derive(Clone, Debug)]
pub struct CodeNode {
	pub(crate) parent: Option<NodeId>,
	character: Option<CodeChar>,
	texts: Vec<Text>,
	// id按创建顺序递增，BTreeMap据此保留子节点的添加顺序
	pub(crate) children: BTreeMap<NodeId, CodeChar>,
}

impl CodeNode {
	/// 创建放在0号位置的唯一根节点
	pub(crate) fn root() -> Self {
		Self {
			parent: None,
			character: None,
			texts: Vec::new(),
			children: BTreeMap::new(),
		}
	}

	/// 当前节点自身的有序候选
	pub(crate) fn texts(&self) -> &[Text] {
		&self.texts
	}

	/// 修改普通节点自身的候选
	pub(crate) fn texts_mut(&mut self) -> &mut Vec<Text> {
		assert!(self.parent.is_some(), "root must not have texts");
		&mut self.texts
	}

	/// 从父链计算编码长度，根节点的长度为0
	pub(crate) fn depth(&self, nodes: &[Self]) -> usize {
		let mut depth = 0;
		let mut current = self;
		while let Some(parent) = current.parent {
			depth += 1;
			current = &nodes[parent];
		}
		depth
	}

	/// 比较完整路径的字典序，不依赖节点的分配顺序
	pub(crate) fn cmp_path(&self, other: &Self, nodes: &[Self]) -> std::cmp::Ordering {
		self.path(nodes).cmp(&other.path(nodes))
	}

	/// 沿父链取得从根到当前节点的字符
	pub(crate) fn path(&self, nodes: &[Self]) -> Vec<CodeChar> {
		let mut path = Vec::new();
		let mut current = self;
		while let Some(parent) = current.parent {
			path.push(
				current
					.character
					.expect("nonroot node must have a character"),
			);
			current = &nodes[parent];
		}
		path.reverse();
		path
	}
}

/// 将输入编码解析到节点，只追加节点，已有id保持稳定
pub(crate) fn insert_code(nodes: &mut Vec<CodeNode>, code: &str) -> NodeId {
	assert!(!nodes.is_empty(), "nodes must contain the root at index 0");
	assert!(!code.is_empty(), "code must not be empty");
	// 先校验全部字符，避免非法输入留下部分路径
	let characters = code
		.chars()
		.map(|character| {
			CodeChar::try_from(character)
				.unwrap_or_else(|invalid| panic!("invalid code character: {invalid:?}"))
		})
		.collect::<Vec<_>>();
	let mut node_id = 0;

	for character in characters {
		node_id = insert_child(nodes, node_id, character);
	}

	node_id
}

/// 查找或追加指定字符的子节点，已有id和子节点顺序保持不变
pub(crate) fn insert_child(
	nodes: &mut Vec<CodeNode>,
	parent: NodeId,
	character: CodeChar,
) -> NodeId {
	if let Some((&child, _)) = nodes[parent]
		.children
		.iter()
		.find(|(_, existing)| **existing == character)
	{
		return child;
	}

	let child = nodes.len();
	nodes.push(CodeNode {
		parent: Some(parent),
		character: Some(character),
		texts: Vec::new(),
		children: BTreeMap::new(),
	});
	let _ = nodes[parent].children.insert(child, character);
	child
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

	/// 将词频除以非零整数
	pub(crate) fn divide(self, divisor: usize) -> Self {
		assert_ne!(divisor, 0, "frequency divisor must be nonzero");
		Self(self.0 / divisor as f64)
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

/// 相对dicts目录读取文件，跳过空行和`#`开头的行
pub(crate) fn parse_target(path: impl AsRef<Path>) -> Vec<String> {
	let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
	let content = fs::read_to_string(&path)
		.unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

	content
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(str::to_owned)
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encoding_characters_round_trip_in_ascii_order() {
		let characters = (0..=127_u8)
			.filter_map(|value| CodeChar::try_from(char::from(value)).ok())
			.collect::<Vec<_>>();
		assert_eq!(characters.len(), 54);
		assert!(characters.windows(2).all(|pair| pair[0] < pair[1]));
		assert_eq!(
			characters.into_iter().map(char::from).collect::<String>(),
			" 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
		);
	}
}
