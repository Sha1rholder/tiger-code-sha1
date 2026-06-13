---@diagnostic disable: undefined-global

---记录Rime已上屏文本；可选记录无候选时的少量编辑按键。

local kNoop = 2

local OPTION_NAME = "input_record_enabled"
local LOG_PREFIX = "input_record"
local LOG_DIR_CONFIG = "input_record/log_dir"
local FLUSH_SIZE_CONFIG = "input_record/flush_size"
local FLUSH_INTERVAL_CONFIG = "input_record/flush_interval_sec"
local SYNC_INTERVAL_CONFIG = "input_record/sync_interval_sec"
local STDIO_BUFFER_BYTES_CONFIG = "input_record/stdio_buffer_bytes"
local RECORD_IDLE_EDIT_KEYS_CONFIG = "input_record/record_idle_edit_keys"

local DEFAULT_LOG_DIR = "input_records"
local DEFAULT_FLUSH_SIZE = 100
local DEFAULT_FLUSH_INTERVAL_SEC = 30
local DEFAULT_SYNC_INTERVAL_SEC = 120
local DEFAULT_STDIO_BUFFER_BYTES = 65536
local DEFAULT_RECORD_IDLE_EDIT_KEYS = false
local MAX_BUFFER_SIZE = 5000

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
		command = 'if not exist "' .. path .. '" mkdir "' .. path .. '" >nul 2>nul'
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

local function read_enabled(context)
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

local function cached_time(state, now)
	if state.time_cache_epoch ~= now then
		state.time_cache_epoch = now
		state.time_cache_text = os.date("%Y-%m-%d %H:%M:%S", now)
	end
	return state.time_cache_text
end

local function encode_event(state, event, now)
	local fields = {
		'"t":' .. tostring(now),
		'"time":"' .. json_escape(cached_time(state, now)) .. '"',
		'"type":"' .. json_escape(event.type) .. '"',
		'"schema":"' .. json_escape(state.schema_id) .. '"',
	}

	if event.text ~= nil then
		fields[#fields + 1] = '"text":"' .. json_escape(event.text) .. '"'
	end
	if event.key ~= nil then
		fields[#fields + 1] = '"key":"' .. json_escape(event.key) .. '"'
	end

	return "{" .. table.concat(fields, ",") .. "}"
end

local function log_date(now)
	return os.date("%Y-%m-%d", now)
end

local function log_filename(state, date)
	return join_path(state.log_dir, "rime_input_" .. date .. ".jsonl")
end

local function close_log_file(state)
	if state.file == nil then
		return
	end

	pcall(function()
		state.file:flush()
	end)
	pcall(function()
		state.file:close()
	end)
	state.file = nil
	state.current_date = nil
end

local function open_log_file(state, now)
	local date = log_date(now)
	if state.file ~= nil and state.current_date == date then
		return true
	end

	close_log_file(state)

	if not state.dir_ready then
		state.dir_ready = ensure_dir(state.log_dir)
	end
	if not state.dir_ready then
		warn("failed to prepare log dir " .. tostring(state.log_dir))
		return false
	end

	local filename = log_filename(state, date)
	local file, err = io.open(filename, "a")
	if file == nil then
		warn("failed to open " .. filename .. ": " .. tostring(err))
		return false
	end

	pcall(function()
		file:setvbuf("full", state.stdio_buffer_bytes)
	end)

	state.file = file
	state.current_date = date
	return true
end

local function flush(env, force, now)
	local state = env.input_record
	if state == nil then
		return
	end

	now = now or os.time()
	if #state.buffer == 0 then
		if force and state.file ~= nil then
			pcall(function()
				state.file:flush()
			end)
			state.last_sync_at = now
		end
		return
	end

	if not force and #state.buffer < state.flush_size and now - state.last_flush_at < state.flush_interval_sec then
		return
	end

	if not open_log_file(state, now) then
		return
	end

	local payload = table.concat(state.buffer, "\n") .. "\n"
	local ok, write_ok, write_err = pcall(function()
		local result, err = state.file:write(payload)
		return result ~= nil, err
	end)
	if not ok or not write_ok then
		warn("failed to write log: " .. tostring(write_err))
		close_log_file(state)
		return
	end

	state.buffer = {}
	state.last_flush_at = now

	if force or now - state.last_sync_at >= state.sync_interval_sec then
		local sync_ok, flush_ok, flush_err = pcall(function()
			local result, err = state.file:flush()
			return result ~= nil, err
		end)
		if not sync_ok or not flush_ok then
			warn("failed to flush log: " .. tostring(flush_err))
		else
			state.last_sync_at = now
		end
	end
end

local function append_event(env, event)
	local state = env.input_record
	if state == nil or state.enabled == false then
		return
	end

	local now = os.time()
	state.buffer[#state.buffer + 1] = encode_event(state, event, now)
	while #state.buffer > MAX_BUFFER_SIZE do
		table.remove(state.buffer, 1)
	end

	flush(env, false, now)
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
	if key.keycode == 0x20 then
		return "space"
	end
	if key.keycode == 0xff08 then
		return "BackSpace"
	end
	return nil
end

local function disconnect_connection(connection, notifier)
	if connection == nil then
		return
	end

	pcall(function()
		if connection.disconnect ~= nil then
			connection:disconnect()
		elseif connection.Disconnect ~= nil then
			connection:Disconnect()
		elseif notifier ~= nil and notifier.disconnect ~= nil then
			notifier:disconnect(connection)
		end
	end)
end

local function connect_option_notifier(env, context)
	if context == nil or context.option_update_notifier == nil then
		return
	end

	local ok, connection_or_err = pcall(function()
		return context.option_update_notifier:connect(function(first, second)
			local option_name
			local source_context = context

			if type(first) == "string" then
				option_name = first
			elseif first ~= nil then
				source_context = first
			end
			if type(second) == "string" then
				option_name = second
			end

			if option_name == nil or option_name == OPTION_NAME then
				local state = env.input_record
				if state ~= nil then
					state.enabled = read_enabled(source_context)
					if state.enabled == false then
						flush(env, true, os.time())
					end
				end
			end
		end)
	end)
	if ok then
		env.input_record.option_connection = connection_or_err
	else
		warn("failed to connect option_update_notifier: " .. tostring(connection_or_err))
	end
end

local function connect_commit_notifier(env, context)
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

local function init(env)
	local engine = env ~= nil and env.engine or nil
	local context = engine ~= nil and engine.context or nil
	local log_dir = get_config_string(env, LOG_DIR_CONFIG, DEFAULT_LOG_DIR)
	local now = os.time()

	env.input_record = {
		buffer = {},
		commit_connection = nil,
		context = context,
		current_date = nil,
		dir_ready = false,
		enabled = read_enabled(context),
		file = nil,
		flush_interval_sec = get_config_int(env, FLUSH_INTERVAL_CONFIG, DEFAULT_FLUSH_INTERVAL_SEC),
		flush_size = get_config_int(env, FLUSH_SIZE_CONFIG, DEFAULT_FLUSH_SIZE),
		last_flush_at = now,
		last_sync_at = now,
		log_dir = join_path(user_data_dir(), log_dir),
		option_connection = nil,
		record_idle_edit_keys = get_config_bool(env, RECORD_IDLE_EDIT_KEYS_CONFIG, DEFAULT_RECORD_IDLE_EDIT_KEYS),
		schema_id = schema_id(env),
		stdio_buffer_bytes = get_config_int(env, STDIO_BUFFER_BYTES_CONFIG, DEFAULT_STDIO_BUFFER_BYTES),
		sync_interval_sec = get_config_int(env, SYNC_INTERVAL_CONFIG, DEFAULT_SYNC_INTERVAL_SEC),
		time_cache_epoch = nil,
		time_cache_text = nil,
	}

	env.input_record.dir_ready = ensure_dir(env.input_record.log_dir)

	connect_option_notifier(env, context)
	connect_commit_notifier(env, context)
end

local function fini(env)
	local state = env.input_record
	if state == nil then
		return
	end

	flush(env, true, os.time())
	disconnect_connection(state.commit_connection, state.context ~= nil and state.context.commit_notifier or nil)
	disconnect_connection(state.option_connection, state.context ~= nil and state.context.option_update_notifier or nil)
	close_log_file(state)
end

local function processor(key, env)
	local state = env.input_record
	if state == nil or not state.record_idle_edit_keys then
		return kNoop
	end

	local name = edit_key_name(key)
	if name == nil or not is_plain_key(key) then
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
