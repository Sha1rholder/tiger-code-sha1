//! 项目基础类型[`Text`]、[`CodeChar`]、[`CodeNode`]、[`Weight`]、[`Freq`]与待迁移的[`Code`]
//! 与通用数据文件解析函数[`parse_target`]

use std::collections::{BTreeMap, HashMap};
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

/// 临时字符串编码，供尚未迁移到节点的模块使用
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Code(String);
impl From<String> for Code {
	fn from(value: String) -> Self {
		Self(value)
	}
}
impl Code {
	/// 临时兼容转换，从节点的完整路径恢复字符串编码
	pub(crate) fn from_node(nodes: &[CodeNode], node_id: NodeId) -> Self {
		Self(
			nodes[node_id]
				.path(nodes)
				.into_iter()
				.map(char::from)
				.collect(),
		)
	}

	/// 返回底层字符串切片
	pub(crate) fn as_str(&self) -> &str {
		&self.0
	}

	/// 返回编码的字节长度
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// 返回编码是否为空
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

/// 编码字符，变体按ASCII字典序排列，供完整路径比较使用
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CodeChar {
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
	parent: Option<NodeId>,
	character: Option<CodeChar>,
	texts: Vec<Text>,
	// id按创建顺序递增，BTreeMap据此保留子节点的添加顺序
	children: BTreeMap<NodeId, CodeChar>,
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
	fn path(&self, nodes: &[Self]) -> Vec<CodeChar> {
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
		if let Some((&child, _)) = nodes[node_id]
			.children
			.iter()
			.find(|(_, existing)| **existing == character)
		{
			node_id = child;
		} else {
			let child = nodes.len();
			nodes.push(CodeNode {
				parent: Some(node_id),
				character: Some(character),
				texts: Vec::new(),
				children: BTreeMap::new(),
			});
			let _ = nodes[node_id].children.insert(child, character);
			node_id = child;
		}
	}

	node_id
}

/// 按节点添加顺序消除空心节点，根节点不参与提升
///
/// 先删除未采用编码留下的空枝，再从最早子节点的路径上取首选
/// 清理结束后统一压缩Vec并更新父子id，调用方不得继续使用旧id
pub(crate) fn eliminate_hollow_nodes(nodes: &mut Vec<CodeNode>) {
	assert!(!nodes.is_empty(), "nodes must contain the root at index 0");
	let mut removed = vec![false; nodes.len()];

	// 父节点总是先创建，倒序清理可排除所有没有候选的子树
	for node in (1..nodes.len()).rev() {
		prune_empty_branch(nodes, node, &mut removed);
	}

	for node in 1..nodes.len() {
		if removed[node] || !nodes[node].texts.is_empty() {
			continue;
		}

		let mut highest = node;
		while let Some(parent) = nodes[highest].parent {
			if parent == 0 || !nodes[parent].texts.is_empty() {
				break;
			}
			highest = parent;
		}

		// 沿最早加入的子节点向下，遇到候选就取其首选
		let mut source = highest;
		while nodes[source].texts.is_empty() {
			source = *nodes[source]
				.children
				.first_key_value()
				.expect("live empty node must have a child with descendants")
				.0;
		}
		let text = nodes[source].texts.remove(0);
		nodes[highest].texts.push(text);

		// 仍有候选或子节点时保留，轮到该节点时继续补字
		prune_empty_branch(nodes, source, &mut removed);
	}

	compact_nodes(nodes, &removed);
}

/// 只删除无候选且无子节点的空枝，遇到根、候选或其他分支就停止
fn prune_empty_branch(nodes: &mut [CodeNode], mut node: NodeId, removed: &mut [bool]) {
	while node != 0
		&& !removed[node]
		&& nodes[node].texts.is_empty()
		&& nodes[node].children.is_empty()
	{
		let parent = nodes[node].parent.expect("nonroot node must have a parent");
		let _ = nodes[parent]
			.children
			.remove(&node)
			.expect("parent must contain its child");
		removed[node] = true;
		node = parent;
	}
}

/// 维持剩余节点的相对添加顺序，并同步重写全部父子引用
fn compact_nodes(nodes: &mut Vec<CodeNode>, removed: &[bool]) {
	let mut next = 0;
	let ids = removed
		.iter()
		.map(|&removed| {
			if removed {
				None
			} else {
				let id = next;
				next += 1;
				Some(id)
			}
		})
		.collect::<Vec<_>>();
	let mut previous = 0;
	nodes.retain(|_| {
		let keep = !removed[previous];
		previous += 1;
		keep
	});

	for node in nodes {
		node.parent = node
			.parent
			.map(|parent| ids[parent].expect("live node must have a live parent"));
		node.children = std::mem::take(&mut node.children)
			.into_iter()
			.map(|(child, character)| {
				(
					ids[child].expect("live child must not be removed"),
					character,
				)
			})
			.collect();
	}
}

/// 临时兼容尚未迁移的中英合并模块，跳过只有路径而没有候选的节点
pub(crate) fn code_nodes_to_groups(nodes: &[CodeNode]) -> HashMap<Code, Vec<Text>> {
	nodes
		.iter()
		.enumerate()
		.filter(|(_, node)| !node.texts().is_empty())
		.map(|(node_id, node)| (Code::from_node(nodes, node_id), node.texts().to_vec()))
		.collect()
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

	fn add_texts(nodes: &mut Vec<CodeNode>, code: &str, texts: &[&str]) {
		let node = insert_code(nodes, code);
		nodes[node]
			.texts_mut()
			.extend(texts.iter().map(|text| Text::from((*text).to_owned())));
	}

	fn groups(nodes: &[CodeNode]) -> BTreeMap<String, Vec<String>> {
		code_nodes_to_groups(nodes)
			.into_iter()
			.map(|(code, texts)| {
				(
					code.as_str().to_owned(),
					texts.iter().map(|text| text.as_str().to_owned()).collect(),
				)
			})
			.collect()
	}

	fn assert_filled_tree(nodes: &[CodeNode]) {
		assert_eq!(nodes[0].parent, None);
		assert_eq!(nodes[0].character, None);
		assert!(nodes[0].texts.is_empty());
		let mut incoming = vec![0; nodes.len()];
		for (id, node) in nodes.iter().enumerate() {
			if id != 0 {
				assert!(!node.texts.is_empty(), "hollow node: {id}");
				assert!(node.parent.unwrap() < id);
			}
			let mut characters = std::collections::HashSet::new();
			for (&child, &character) in &node.children {
				assert!(characters.insert(character));
				assert_eq!(nodes[child].parent, Some(id));
				assert_eq!(nodes[child].character, Some(character));
				incoming[child] += 1;
			}
		}
		assert_eq!(incoming[0], 0);
		assert!(incoming[1..].iter().all(|&count| count == 1));
	}

	#[test]
	fn hollow_nodes_take_the_first_text_from_the_first_added_child() {
		let mut nodes = vec![CodeNode::root()];
		add_texts(&mut nodes, "xzb", &["丁"]);
		add_texts(&mut nodes, "xz", &["甲", "乙"]);
		add_texts(&mut nodes, "xaa", &["丙"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 4);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xz"], ["乙"]);
		assert_eq!(dictionary["xzb"], ["丁"]);
		assert_eq!(dictionary["xa"], ["丙"]);
		assert_filled_tree(&nodes);

		// 压缩后最先创建的z分支仍在a分支之前
		let count = nodes.len();
		let x = insert_code(&mut nodes, "x");
		let children = nodes[x].children.values().copied().collect::<Vec<_>>();
		assert_eq!(children, [CodeChar::LowerZ, CodeChar::LowerA]);
		for code in dictionary.keys() {
			let _ = insert_code(&mut nodes, code);
		}
		assert_eq!(nodes.len(), count);
	}

	#[test]
	fn promotion_preserves_a_donor_that_still_has_descendants() {
		let mut nodes = vec![CodeNode::root()];
		add_texts(&mut nodes, "xab", &["乙"]);
		add_texts(&mut nodes, "xa", &["甲"]);
		add_texts(&mut nodes, "xac", &["丙"]);
		add_texts(&mut nodes, "xz", &["丁"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 4);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xa"], ["乙"]);
		assert_eq!(dictionary["xac"], ["丙"]);
		assert_eq!(dictionary["xz"], ["丁"]);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn promotion_prunes_empty_chains_but_keeps_sibling_branches() {
		let mut nodes = vec![CodeNode::root()];
		// 原始选码可能留下完全没有候选的路径
		let _ = insert_code(&mut nodes, "unused");
		add_texts(&mut nodes, "xab", &["甲"]);
		add_texts(&mut nodes, "xac", &["乙"]);
		add_texts(&mut nodes, "xz", &["丙"]);
		eliminate_hollow_nodes(&mut nodes);

		let dictionary = groups(&nodes);
		assert_eq!(dictionary.len(), 3);
		assert_eq!(dictionary["x"], ["甲"]);
		assert_eq!(dictionary["xa"], ["乙"]);
		assert_eq!(dictionary["xz"], ["丙"]);
		assert_eq!(nodes.len(), 4);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn an_unused_tree_keeps_only_its_root() {
		let mut nodes = vec![CodeNode::root()];
		let _ = insert_code(&mut nodes, "abc");
		let _ = insert_code(&mut nodes, "q");
		eliminate_hollow_nodes(&mut nodes);
		assert_eq!(nodes.len(), 1);
		assert_filled_tree(&nodes);
	}

	#[test]
	fn promotion_preserves_every_text_and_only_shortens_its_path() {
		// 固定种子生成不同形状的树，检查结果而不重复提升算法
		let mut seed = 7_u64;
		for _ in 0..64 {
			let mut nodes = vec![CodeNode::root()];
			let mut original = HashMap::new();
			for index in 0..40 {
				seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
				let length = 1 + (seed % 5) as usize;
				let mut code = String::new();
				for _ in 0..length {
					seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
					code.push(['z', 'a', 'b', 'A', '\''][(seed >> 32) as usize % 5]);
				}
				let text = format!("text{index}");
				add_texts(&mut nodes, &code, &[&text]);
				let _ = original.insert(text, code);
			}
			eliminate_hollow_nodes(&mut nodes);
			assert_filled_tree(&nodes);
			let mut remaining = original.clone();
			for (code, texts) in groups(&nodes) {
				for text in texts {
					let original_code = remaining.remove(&text).expect("duplicate or added text");
					assert!(original_code.starts_with(&code));
				}
			}
			assert!(remaining.is_empty());
			let once = groups(&nodes);
			eliminate_hollow_nodes(&mut nodes);
			assert_eq!(groups(&nodes), once);
			assert_filled_tree(&nodes);
		}
	}

	#[test]
	fn alphabet_has_exactly_53_distinct_characters() {
		let accepted = (0..=127)
			.filter_map(|value| CodeChar::try_from(char::from(value)).ok())
			.collect::<Vec<_>>();
		assert_eq!(accepted.len(), 53);
		assert_eq!(
			accepted
				.iter()
				.collect::<std::collections::HashSet<_>>()
				.len(),
			53
		);
		assert_eq!(
			accepted.iter().copied().map(char::from).collect::<String>(),
			"'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
		);
		assert!(accepted.windows(2).all(|pair| pair[0] < pair[1]));
		for character in ['中', 'é', 'Ａ', '’'] {
			assert_eq!(CodeChar::try_from(character), Err(character));
		}

		let mut nodes = vec![CodeNode::root()];
		for character in accepted {
			let node = insert_code(&mut nodes, &char::from(character).to_string());
			assert_eq!(nodes[node].parent, Some(0));
			assert_eq!(nodes[node].character, Some(character));
		}
		assert_eq!(nodes[0].children.len(), 53);
		assert_eq!(nodes.len(), 54);
	}

	#[test]
	fn prefixes_share_nodes_but_keep_their_own_candidates() {
		let mut nodes = vec![CodeNode::root()];
		let inf = insert_code(&mut nodes, "inf");
		let inq = insert_code(&mut nodes, "inq");
		let prefix = insert_code(&mut nodes, "in");
		let i = insert_code(&mut nodes, "i");
		assert_eq!(nodes.len(), 5);
		assert_eq!(insert_code(&mut nodes, "inf"), inf);
		assert_eq!(nodes.len(), 5);

		for (node, texts) in [
			(i, vec!["为", "为什么"]),
			(prefix, vec!["主", "主要", "主"]),
			(inf, vec!["缄"]),
			(inq, vec!["缜"]),
		] {
			nodes[node]
				.texts_mut()
				.extend(texts.into_iter().map(|text| Text::from(text.to_owned())));
		}
		let xyz = insert_code(&mut nodes, "xyz");
		nodes[xyz].texts_mut().push(Text::from("字".to_owned()));
		assert_eq!(nodes[0].parent, None);
		assert_eq!(nodes[0].character, None);
		assert!(nodes[0].texts().is_empty());
		assert_eq!(nodes[0].depth(&nodes), 0);
		assert_eq!(nodes[inf].depth(&nodes), 3);
		assert_eq!(nodes[inf].parent, Some(prefix));
		assert_eq!(nodes[inq].parent, Some(prefix));
		assert_eq!(nodes[prefix].parent, Some(i));
		assert_eq!(nodes[prefix].children[&inf], CodeChar::LowerF);
		assert_eq!(nodes[prefix].children[&inq], CodeChar::LowerQ);
		for (parent, node) in nodes.iter().enumerate() {
			for (&child, &character) in &node.children {
				assert_eq!(nodes[child].parent, Some(parent));
				assert_eq!(nodes[child].character, Some(character));
			}
		}

		let groups = code_nodes_to_groups(&nodes);
		assert_eq!(groups.len(), 5);
		assert_eq!(
			groups[&Code::from("in".to_owned())]
				.iter()
				.map(Text::as_str)
				.collect::<Vec<_>>(),
			["主", "主要", "主"]
		);
		for code in ["i", "in", "inf", "inq", "xyz"] {
			let node = insert_code(&mut nodes, code);
			assert_eq!(Code::from_node(&nodes, node).as_str(), code);
		}
		for code in ["", "x", "xy"] {
			assert!(!groups.contains_key(&Code::from(code.to_owned())));
		}
	}

	#[test]
	fn path_order_does_not_depend_on_node_ids() {
		let mut nodes = vec![CodeNode::root()];
		let mut codes = ["z", "inq", "inf", "in", "i", "a", "A", "'", "I", "i'n"];
		let mut ids = codes
			.iter()
			.map(|code| insert_code(&mut nodes, code))
			.collect::<Vec<_>>();
		ids.sort_unstable_by(|&left, &right| nodes[left].cmp_path(&nodes[right], &nodes));
		codes.sort_unstable();
		let ordered = ids
			.into_iter()
			.map(|node| Code::from_node(&nodes, node))
			.collect::<Vec<_>>();
		assert_eq!(ordered.iter().map(Code::as_str).collect::<Vec<_>>(), codes);
	}

	#[test]
	fn invalid_input_leaves_no_partial_path() {
		let mut nodes = vec![CodeNode::root()];
		for code in ["", "ab1", "abé", "a b", "a-b", "abc\n"] {
			let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
				insert_code(&mut nodes, code)
			}));
			assert!(result.is_err(), "accepted invalid code: {code:?}");
			assert_eq!(nodes.len(), 1);
			assert!(nodes[0].children.is_empty());
		}
	}
}
