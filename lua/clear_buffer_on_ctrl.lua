---@diagnostic disable: undefined-global

---按下Ctrl键清空候选缓冲区且不切换ascii_mode

local kAccepted = 1
local kNoop = 2

local function is_ctrl_key(key)
	local repr = key:repr()
	return key.keycode == 0xffe3
		or key.keycode == 0xffe4
		or repr == "Control_L"
		or repr == "Control_R"
		or repr == "Control+Control_L"
		or repr == "Control+Control_R"
end

local function processor(key, env)
	local context = env.engine.context

	if is_ctrl_key(key) then
		if key:release() then
			if env.ctrl_clear_candidate and (context.input or "") ~= "" then
				context:clear()
				env.ctrl_clear_candidate = false
				return kAccepted
			end
			env.ctrl_clear_candidate = false
			return kNoop
		end

		env.ctrl_clear_candidate = true
		return kNoop
	end

	if env.ctrl_clear_candidate then
		env.ctrl_clear_candidate = false
	end

	return kNoop
end

return processor
