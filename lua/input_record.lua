---@diagnostic disable: undefined-global

---记录Rime已上屏文本，以及无候选时的少量编辑按键

local kNoop = 2

local OPTION_NAME = "input_record_enabled"
local LOG_PREFIX = "input_record"
local LOG_DIR_CONFIG = "input_record/log_dir"
local FLUSH_SIZE_CONFIG = "input_record/flush_size"
local FLUSH_INTERVAL_CONFIG = "input_record/flush_interval_sec"
local RECORD_IDLE_EDIT_KEYS_CONFIG = "input_record/record_idle_edit_keys"

local DEFAULT_LOG_DIR = "input_records"
local DEFAULT_FLUSH_SIZE = 20
local DEFAULT_FLUSH_INTERVAL_SEC = 5
local MAX_BUFFER_SIZE = 1000

local function warn(message)
	if log ~= nil and log.warning ~= nil then
		log.warning(LOG_PREFIX .. ": " .. message)
	end
end

local function user_data_dir()
	if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
		return rime_api.get_user_data_dir()
	end
	return "."
end

local function is_absolute_path(path)
	return path:match("^%a:[/\\]") ~= nil or path:sub(1, 1) == "/" or path:sub(1, 1) == "\\"
end

local function join_path(base, child)
	if child == "" or is_absolute_path(child) then
		return child
	end
	if base:match("[/\\]$") then
		return base .. child
	end
	return base .. "/" .. child
end

local function ensure_dir(path)
	if path == nil or path == "" then
		return false
	end

	local command
	if package.config:sub(1, 1) == "\\" then
		command = 'mkdir "' .. path .. '" >nul 2>nul'
	else
		command = 'mkdir -p "' .. path .. '" >/dev/null 2>&1'
	end

	os.execute(command)
	return true
end

local function get_config(env)
	local engine = env ~= nil and env.engine or nil
	local schema = engine ~= nil and engine.schema or nil
	return schema ~= nil and schema.config or nil
end

local function get_config_bool(env, path, default)
	local config = get_config(env)
	if config == nil or config.get_bool == nil then
		return default
	end

	local ok, value = pcall(function()
		return config:get_bool(path)
	end)
	if not ok or value == nil then
		return default
	end
	return value == true
end

local function get_config_int(env, path, default)
	local config = get_config(env)
	if config == nil or config.get_int == nil then
		return default
	end

	local ok, value = pcall(function()
		return config:get_int(path)
	end)
	if not ok or type(value) ~= "number" or value <= 0 then
		return default
	end
	return math.floor(value)
end

local function get_config_string(env, path, default)
	local config = get_config(env)
	if config == nil or config.get_string == nil then
		return default
	end

	local ok, value = pcall(function()
		return config:get_string(path)
	end)
	if not ok or value == nil or value == "" then
		return default
	end
	return value
end

local json_escapes = {
	["\\"] = "\\\\",
	['"'] = '\\"',
	["\b"] = "\\b",
	["\f"] = "\\f",
	["\n"] = "\\n",
	["\r"] = "\\r",
	["\t"] = "\\t",
}

local function json_escape(value)
	return tostring(value or ""):gsub('[%z\1-\31\\"]', function(char)
		return json_escapes[char] or string.format("\\u%04x", char:byte())
	end)
end

local function schema_id(env)
	local engine = env ~= nil and env.engine or nil
	local schema = engine ~= nil and engine.schema or nil
	if schema == nil then
		return ""
	end
	return schema.schema_id or schema.schema_name or ""
end

local function is_enabled(env)
	local context = env ~= nil and env.engine ~= nil and env.engine.context or nil
	if context == nil or context.get_option == nil then
		return true
	end

	local ok, value = pcall(function()
		return context:get_option(OPTION_NAME)
	end)
	if not ok or value == nil then
		return true
	end
	return value == true
end

local function encode_event(env, event)
	local now = os.time()
	local fields = {
		'"t":' .. tostring(now),
		'"time":"' .. json_escape(os.date("%Y-%m-%d %H:%M:%S", now)) .. '"',
		'"type":"' .. json_escape(event.type) .. '"',
		'"schema":"' .. json_escape(schema_id(env)) .. '"',
	}

	if event.text ~= nil then
		fields[#fields + 1] = '"text":"' .. json_escape(event.text) .. '"'
	end
	if event.key ~= nil then
		fields[#fields + 1] = '"key":"' .. json_escape(event.key) .. '"'
	end

	return "{" .. table.concat(fields, ",") .. "}"
end

local function log_filename(state)
	return join_path(state.log_dir, "rime_input_" .. os.date("%Y-%m-%d") .. ".jsonl")
end

local function flush(env, force)
	local state = env.input_record
	if state == nil or #state.buffer == 0 then
		return
	end

	local now = os.time()
	if not force and #state.buffer < state.flush_size and now - state.last_flush_at < state.flush_interval_sec then
		return
	end

	if not state.dir_ready then
		state.dir_ready = ensure_dir(state.log_dir)
	end
	if not state.dir_ready then
		warn("failed to prepare log dir " .. tostring(state.log_dir))
		return
	end

	local filename = log_filename(state)
	local file, err = io.open(filename, "a")
	if file == nil then
		warn("failed to open " .. filename .. ": " .. tostring(err))
		return
	end

	for _, line in ipairs(state.buffer) do
		file:write(line)
		file:write("\n")
	end
	file:close()

	state.buffer = {}
	state.last_flush_at = now
end

local function append_event(env, event)
	if not is_enabled(env) then
		return
	end

	local state = env.input_record
	if state == nil then
		return
	end

	state.buffer[#state.buffer + 1] = encode_event(env, event)
	while #state.buffer > MAX_BUFFER_SIZE do
		table.remove(state.buffer, 1)
	end

	flush(env, false)
end

local function commit_text(context)
	if context == nil or context.get_commit_text == nil then
		return nil
	end

	local ok, text = pcall(function()
		return context:get_commit_text()
	end)
	if not ok then
		return nil
	end
	return text
end

local function has_menu(context)
	if context == nil or context.has_menu == nil then
		return false
	end

	local ok, value = pcall(function()
		return context:has_menu()
	end)
	return ok and value == true
end

local function is_plain_key(key)
	return not key:release() and not key:ctrl() and not key:alt() and not key:shift() and not key:super()
end

local function edit_key_name(key)
	local repr = key:repr()
	if key.keycode == 0x20 or repr == "space" or repr == "Space" then
		return "space"
	end
	if key.keycode == 0xff08 or repr == "BackSpace" then
		return "BackSpace"
	end
	return nil
end

local function init(env)
	local log_dir = get_config_string(env, LOG_DIR_CONFIG, DEFAULT_LOG_DIR)
	env.input_record = {
		buffer = {},
		commit_connection = nil,
		context = env.engine.context,
		dir_ready = false,
		flush_interval_sec = get_config_int(env, FLUSH_INTERVAL_CONFIG, DEFAULT_FLUSH_INTERVAL_SEC),
		flush_size = get_config_int(env, FLUSH_SIZE_CONFIG, DEFAULT_FLUSH_SIZE),
		last_flush_at = os.time(),
		log_dir = join_path(user_data_dir(), log_dir),
		record_idle_edit_keys = get_config_bool(env, RECORD_IDLE_EDIT_KEYS_CONFIG, true),
	}

	local context = env.engine.context
	if context == nil or context.commit_notifier == nil then
		warn("commit_notifier is unavailable")
		return
	end

	local ok, connection_or_err = pcall(function()
		return context.commit_notifier:connect(function(notified_context)
			local source_context = notified_context or context
			local text = commit_text(source_context) or commit_text(context)
			if text ~= nil and text ~= "" then
				append_event(env, {
					type = "commit",
					text = text,
				})
			end
		end)
	end)
	if ok then
		env.input_record.commit_connection = connection_or_err
	else
		warn("failed to connect commit_notifier: " .. tostring(connection_or_err))
	end
end

local function fini(env)
	if env.input_record ~= nil then
		flush(env, true)

		local connection = env.input_record.commit_connection
		local context = env.input_record.context
		if connection ~= nil then
			pcall(function()
				if connection.disconnect ~= nil then
					connection:disconnect()
				elseif connection.Disconnect ~= nil then
					connection:Disconnect()
				elseif context ~= nil and context.commit_notifier ~= nil and context.commit_notifier.disconnect ~= nil then
					context.commit_notifier:disconnect(connection)
				end
			end)
		end
	end
end

local function processor(key, env)
	local state = env.input_record
	if state == nil or not state.record_idle_edit_keys or not is_plain_key(key) then
		return kNoop
	end

	local name = edit_key_name(key)
	if name == nil then
		return kNoop
	end

	local context = env.engine.context
	if not has_menu(context) then
		append_event(env, {
			type = "edit",
			key = name,
		})
	end

	return kNoop
end

return {
	init = init,
	func = processor,
	fini = fini,
}
