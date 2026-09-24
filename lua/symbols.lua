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
---@field get_selected_candidate fun(self: Context): Candidate? 获取当前选中的候选

---@class Candidate Rime候选
---@field text string 候选文本

---存在buffer时输入symbol直接上屏当前buffer和raw symbol
---双击`'`、`"`、`<`、`>`、`_`直接上屏对应次选

local kAccepted = 1
local kNoop = 2

local second_symbols = {
	['"'] = { ["“"] = "”" },
	["'"] = { ["‘"] = "’" },
	["<"] = { ["《"] = "<" },
	[">"] = { ["》"] = ">" },
	["_"] = { ["——"] = "_" },
}

---获取不带修饰的symbol
---@param key KeyEvent 按键事件
---@return string|nil 是纯符号键时返回ASCII字符，否则返回nil
local function get_raw_symbol(key)
	if key:release() or key:ctrl() or key:alt() or key:super() then
		return nil
	end

	if type(key.keycode) == "number" and key.keycode >= 0x21 and key.keycode <= 0x7e then
		local char = string.char(key.keycode)
		if not char:match("[%w]") then
			return char
		end
	end

	local repr = key:repr()
	if repr ~= nil and repr ~= "" and repr:match("^[!-/%:-@%[-`{-~]$") ~= nil then
		return repr
	end

	return nil
end

---Rime处理器入口：处理双击符号和buffer后的ASCII符号
---@param key KeyEvent 按键事件
---@param env Environment Rime环境对象
---@return integer kAccepted表示按键已被处理，kNoop表示未处理
local function processor(key, env)
	local engine = env.engine
	local context = engine.context
	local input = context.input or ""
	local symbol = get_raw_symbol(key)

	if symbol == nil then
		return kNoop
	end

	if second_symbols[symbol] ~= nil then
		local candidate = context:get_selected_candidate()
		if candidate ~= nil then
			local second_symbol = second_symbols[symbol][candidate.text]
			if second_symbol ~= nil then
				engine:commit_text(second_symbol)
				context:clear()
				return kAccepted
			end
		end
	end

	if input ~= "" then
		engine:commit_text(input .. symbol)
		context:clear()
		return kAccepted
	end

	return kNoop
end

return processor
