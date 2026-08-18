/// 汉字去重编码表处理模块
pub mod dedup {
	use crate::data::{SC2013, TIGER, ZH_CUSTOM, ZH_PY, ZH_RECODE};
	use crate::prelude::{Code, Text, Weight};
	use std::collections::HashMap;

	pub fn tiger_dd() -> Vec<(Code, Text, Weight)> {
		// 获取有效虎码条目并排序
		let tiger = TIGER
			.iter()
			.filter(|(_, text, _)| SC2013.contains(text))
			.sort_by(|a, b| b.2.cmp(&a.2))
			.cloned()
			.collect();
		let mut tiger_0: HashMap<Text, (Code, Weight)> = HashMap::new();
		for tiger_item in tiger {
			if tiger_0.contains_key(&tiger_item.1) {
				if tiger_item.len() < tiger_0.get(&tiger_item.1).unwrap().0.len() {
					// xxx
				}
			} else {
				tiger_0.insert(tiger_item.1.clone(), (tiger_item.0.clone(), tiger_item.2));
			}
		}
		tiger
	}
}
