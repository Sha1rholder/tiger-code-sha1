---@diagnostic disable: undefined-global

---@class KeyEvent Rime按键事件
---@field keycode number 键码
---@field release fun(self: KeyEvent): boolean 是否为释放事件
---@field ctrl fun(self: KeyEvent): boolean 是否按下Ctrl
---@field alt fun(self: KeyEvent): boolean 是否按下Alt
---@field super fun(self: KeyEvent): boolean 是否按下Super
---@field repr fun(self: KeyEvent): string? 按键的字符串表示

---@class Environment Rime环境对象
---@field engine Engine

---@class Engine Rime引擎
---@field context Context
---@field commit_text fun(self: Engine, text: string) 上屏文本

---@class Context Rime上下文
---@field input string 编码串
---@field clear fun(self: Context) 清空编码串
---@field get_selected_candidate fun(self: Context): Candidate? 获取当前候选
---@field composition Composition

---@class Composition Rime分段集合
---@field back fun(self: Composition): Segment? 获取末尾分段

---@class Segment Rime分段
---@field selected_index integer? 当前候选索引
---@field menu Menu?

---@class Menu Rime候选菜单
---@field get_candidate_at fun(self: Menu, index: integer): Candidate? 获取指定候选

---@class Candidate Rime候选
---@field text string 候选文本

---有输入buffer且末尾是英文字母时直接提交当前编码和后续ASCII符号
---符号候选为half_shape数组首选时，连按同一符号直接提交ASCII符号

local kAccepted = 1
local kNoop = 2

local repeat_symbol_rules = {
	["'"] = { candidate = "‘", commit = "’" },
	["<"] = { candidate = "《", commit = "<" },
	[">"] = { candidate = "》", commit = ">" },
}

---安全调用对象方法
---@param object any 对象
---@param method string 方法名
---@param ... any 参数
---@return any
local function call_method(object, method, ...)
	if object == nil then
		return nil
	end

	local ok_get, func = pcall(function()
		return object[method]
	end)
	if not ok_get or type(func) ~= "function" then
		return nil
	end

	local ok_call, result = pcall(func, object, ...)
	if ok_call then
		return result
	end

	return nil
end

---获取候选文本
---@param candidate Candidate|nil 候选对象
---@return string|nil 候选文本
local function candidate_text(candidate)
	if candidate == nil then
		return nil
	end

	local ok, text = pcall(function()
		return candidate.text
	end)
	if ok and type(text) == "string" then
		return text
	end

	return nil
end

---从分段菜单中获取当前候选
---@param context Context Rime上下文
---@return Candidate|nil 当前候选
local function selected_candidate_from_menu(context)
	local composition = context.composition
	local segment = call_method(composition, "back")
	if segment == nil or segment.menu == nil then
		return nil
	end

	local selected_index = segment.selected_index
	if type(selected_index) ~= "number" then
		selected_index = 0
	end

	return call_method(segment.menu, "get_candidate_at", selected_index)
end

---获取当前候选文本
---@param context Context Rime上下文
---@return string|nil 当前候选文本
local function selected_candidate_text(context)
	return candidate_text(call_method(context, "get_selected_candidate"))
	    or candidate_text(selected_candidate_from_menu(context))
end

---从键码获取ASCII符号字符（排除字母和数字）
---@param keycode number 键码
---@return string|nil 是ASCII符号时返回字符，否则返回nil
local function ascii_symbol_from_keycode(keycode)
	if type(keycode) ~= "number" then
		return nil
	end

	if keycode >= 0x21 and keycode <= 0x7e then
		local char = string.char(keycode)
		if not char:match("[%w]") then
			return char
		end
	end

	return nil
end

---获取不带修饰键的ASCII符号字符
---@param key KeyEvent 按键事件
---@return string|nil 是纯符号键时返回ASCII字符，否则返回nil
local function plain_ascii_symbol_from_key(key)
	if key:release() or key:ctrl() or key:alt() or key:super() then
		return nil
	end

	local symbol = ascii_symbol_from_keycode(key.keycode)
	if symbol ~= nil then
		return symbol
	end

	local repr = key:repr()
	if repr ~= nil and repr ~= "" and repr:match("^[!-/%:-@%[-`{-~]$") ~= nil then
		return repr
	end

	return nil
end

---处理重复符号输入
---@param engine Engine Rime引擎
---@param context Context Rime上下文
---@param input string 编码串
---@param symbol string ASCII符号
---@return boolean 是否已处理
local function handle_repeated_symbol(engine, context, input, symbol)
	local rule = repeat_symbol_rules[symbol]
	if rule == nil or input ~= symbol then
		return false
	end

	if selected_candidate_text(context) ~= rule.candidate then
		return false
	end

	engine:commit_text(rule.commit)
	context:clear()
	return true
end

---Rime处理器入口：处理符号连按和英文buffer后的ASCII符号
---@param key KeyEvent 按键事件
---@param env Environment Rime环境对象
---@return integer kAccepted表示按键已被处理，kNoop表示未处理
local function processor(key, env)
	local engine = env.engine
	local context = engine.context
	local input = context.input or ""
	local symbol = plain_ascii_symbol_from_key(key)

	if symbol == nil then
		return kNoop
	end

	if handle_repeated_symbol(engine, context, input, symbol) then
		return kAccepted
	end

	if input ~= "" and input:sub(-1):match("[a-zA-Z]") then
		engine:commit_text(input .. symbol)
		context:clear()
		return kAccepted
	end

	return kNoop
end

return processor
