/// 生成中文加词

// 用type关键字给Weight（补全权重）、Text（补全部）、Text（首字）、Text（原词）起个别名
// 对ZH_FREQ，生成`HashMap<首字, HashMap<补全部, 补全权重>>`
// 对该变量的每个首字，利用TIGER_CHARS的无序字典形式找到对应的Code，然后生成`HashMap<Code, HashMap<原词, 补全权重>>`
// 对该变量用补全权重降序排列，形成zh_words: `HashMap<Code, Vec<原词>>`

// TIGER_CHARS + ZH_CUSTOM + zh_words -> `HashMap<Code, Vec<Text>>`保留顺序
// 只保留每个`Vec<Text>`的前`const CANDIDATES = 5`项
