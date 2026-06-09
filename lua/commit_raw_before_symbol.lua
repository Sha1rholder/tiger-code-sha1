---@diagnostic disable: undefined-global

---在处理后续符号前直接提交当前的纯字母编码

---重新处理同一个按键一次，以便标点处理器和express_editor仍能决定如何处理该符号本身

local kAccepted = 1
local kNoop = 2

local function is_ascii_symbol_code(keycode)
	if type(keycode) ~= "number" then
		return false
	end

	if keycode >= 0x21 and keycode <= 0x7e then
		local char = string.char(keycode)
		return not char:match("[%w]")
	end

	return false
end

local function is_plain_symbol_key(key)
	if key:release() or key:ctrl() or key:alt() or key:super() then
		return false
	end

	if is_ascii_symbol_code(key.keycode) then
		return true
	end

	local repr = key:repr()
	return repr ~= nil and repr ~= "" and repr:match("^[!-/%:-@%[-`{-~]$") ~= nil
end

local function processor(key, env)
	local engine = env.engine
	local context = engine.context
	local input = context.input or ""

	if input:match("^[A-Za-z]+$") and is_plain_symbol_key(key) then
		engine:commit_text(input)
		context:clear()
		engine:process_key(key)
		return kAccepted
	end

	return kNoop
end

return processor
