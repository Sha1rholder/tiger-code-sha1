/// 生成中文加词

// 对ZH_FREQ，丢弃包含过滤后候选集合中任意其它词的词，如`之所以`⊃`所以`，就去掉`之所以`项
// 对ZH_FREQ，生成`HashMap<Text（首字）, Option<HashMap<Text（补全）, (Weight（权重/词频）, u8（补全码长）)>>>`
// 对该变量，生成`HashMap<Code, Vec<Text>>`

// 根据tiger单字表和tiger加字/词表的两个`Vec<(Code, Text)>`生成`HashMap<Code, i8>`，统计每个Code对应几个Text
// `HashMap<Code, i8>` -> `HashMap<Code, 3 - i8>` -> 去掉`3 - i8 <= 0`的键值对 -> `HashMap<Code, u8>`
//
