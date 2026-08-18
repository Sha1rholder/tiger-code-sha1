/// 生成中文加词

// 根据tiger单字表`Vec<(Code, Text)>`生成tiger单字表的无序字典形式`HashMap<Code, Vec<Text>>`
// debug断言tiger单字表中每个Code对应的Text数量不溢出i8
// 根据tiger单字表和tiger加字/词表的两个`Vec<(Code, Text)>`生成`HashMap<Code, i8（每个Code对应几个Text）>`
// CANDIDATES = 5
// `HashMap<Code, max(CANDIDATES - i8, 0)（每个Code最多应该补多少个词）>`（暂称为A）

// debug断言ZH_FREQ不含单字Text
// 用type关键字给Weight（补全权重）、Text（补全部）、Text（首字）、Text（原词）起个别名
// 对ZH_FREQ，丢弃包含过滤后候选集合中任意其它词的词，如`之所以`⊃`所以`，就去掉`之所以`项
// 对ZH_FREQ，生成`HashMap<首字, HashMap<补全部, 补全权重>>`
// 对该变量的每个首字，利用tiger单字表的无序字典形式找到对应的Code，然后生成`HashMap<Code, HashMap<原词, 补全权重>>`
// 对该变量用补全权重降序排列，形成`HashMap<Code, Vec<原词>>`（暂称为B）

// debug断言B的所有键名必在A的键名中
// 对B的每个Code，只保留A指定的前几个候选原词
// 把B展开为`Vec<(Code, Text)>`
