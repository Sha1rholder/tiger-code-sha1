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

---有输入buffer且末尾是英文字母时直接提交当前编码和后续ASCII符号

local kAccepted = 1
local kNoop = 2

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

---Rime处理器入口：有输入buffer且输入buffer末尾是英文字母时，直接上屏当前编码和该符号
---@param key KeyEvent 按键事件
---@param env Environment Rime环境对象
---@return integer kAccepted表示按键已被处理，kNoop表示未处理
local function processor(key, env)
	local engine = env.engine
	local context = engine.context
	local input = context.input or ""
	local symbol = plain_ascii_symbol_from_key(key)

	if input ~= "" and symbol ~= nil and input:sub(-1):match("[a-zA-Z]") then
		engine:commit_text(input .. symbol)
		context:clear()
		return kAccepted
	end

	return kNoop
end

return processor
