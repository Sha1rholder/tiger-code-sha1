use crate::data::ZH_CUSTOM;
use crate::prelude::{Code, Text};

/// 返回独立的虎码加字和加词表
pub(crate) fn tiger_custom() -> Vec<(Code, Text)> {
	ZH_CUSTOM.clone()
}

#[cfg(test)]
mod tests {
	use super::*;

	/// 构造编码文本测试条目
	fn row(code: &str, text: &str) -> (Code, Text) {
		(Code::from(code.to_owned()), Text::from(text.to_owned()))
	}

	/// 验证真实自定义表按编码文本字段独立返回
	#[test]
	fn returns_real_custom_rows() {
		let result = tiger_custom();

		assert_eq!(result.as_slice(), ZH_CUSTOM.as_slice());
		assert_eq!(ZH_CUSTOM.first(), Some(&row("o", "〇")));
	}
}
