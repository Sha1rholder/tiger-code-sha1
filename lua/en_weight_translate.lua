---@diagnostic disable: undefined-global

---按 tiger_sha1_en.dict.yaml 顺序懒加载产出英文候选词

local MAX_PREFIX_LEN = 4
local DICT_NAME = "tiger_sha1_en.dict.yaml"
local LOG_PREFIX = "en_weight_translate"
local PERF_LOG_CONFIG = "en_weight_translate/enable_perf_log"

local loaded = false
local entries_by_prefix = {}

local function data_dir()
	if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
		return rime_api.get_user_data_dir()
	end
	return "."
end

local function open_dict()
	local file = io.open(data_dir() .. "/" .. DICT_NAME, "r")
	if file ~= nil then
		return file
	end
	return io.open(DICT_NAME, "r")
end

local function starts_with(text, prefix)
	return text:sub(1, #prefix) == prefix
end

local function add_entry(entry)
	local max_len = math.min(MAX_PREFIX_LEN, #entry.code)
	for len = 1, max_len do
		local prefix = entry.code:sub(1, len)
		local bucket = entries_by_prefix[prefix]
		if bucket == nil then
			bucket = {}
			entries_by_prefix[prefix] = bucket
		end
		bucket[#bucket + 1] = entry
	end
end

local function load_entries()
	if loaded then
		return
	end

	entries_by_prefix = {}
	loaded = true

	local file = open_dict()
	if file == nil then
		if log ~= nil and log.warning ~= nil then
			log.warning(LOG_PREFIX .. ": failed to open " .. DICT_NAME)
		end
		return
	end

	local in_body = false
	local count = 0
	local rank = 0
	for line in file:lines() do
		if in_body then
			-- 词典正文格式：编码<TAB>文本
			local code, text = line:match("^([^\t]+)\t(.+)$")
			if code ~= nil and text ~= nil then
				rank = rank + 1
				count = count + 1
				add_entry({
					code = code,
					text = text,
					rank = rank,
				})
			end
		elseif line == "..." then
			in_body = true
		end
	end

	file:close()

	if log ~= nil and log.info ~= nil then
		log.info(LOG_PREFIX .. ": loaded " .. count .. " entries from " .. DICT_NAME)
	end
end

local function get_config_bool(env, path)
	local engine = env ~= nil and env.engine or nil
	local schema = engine ~= nil and engine.schema or nil
	local config = schema ~= nil and schema.config or nil
	if config == nil or config.get_bool == nil then
		return false
	end

	local ok, value = pcall(function()
		return config:get_bool(path)
	end)
	return ok and value == true
end

local function segment_start(segment)
	return segment.start or segment._start or 0
end

local function segment_end(segment, input)
	return segment._end or segment["end"] or (segment_start(segment) + #input)
end

local function make_candidate(segment, input, entry)
	local cand = Candidate("english", segment_start(segment), segment_end(segment, input), entry.text, " ")
	-- 使英文候选词排在码表候选词之后，同时在translator内部保持词典顺序
	cand.quality = -entry.rank
	return cand
end

local function translator(input, segment, env)
	load_entries()

	if input == nil or not input:match("^[A-Za-z]+$") then
		return
	end

	local start_time = nil
	local perf_log_enabled = get_config_bool(env, PERF_LOG_CONFIG)
	if perf_log_enabled and os ~= nil and os.clock ~= nil then
		start_time = os.clock()
	end

	local lookup_input = input
	local bucket_key = lookup_input:sub(1, math.min(MAX_PREFIX_LEN, #lookup_input))
	local bucket = entries_by_prefix[bucket_key] or {}
	local needs_filter = #lookup_input > MAX_PREFIX_LEN

	if perf_log_enabled and log ~= nil and log.info ~= nil and start_time ~= nil then
		local elapsed_ms = (os.clock() - start_time) * 1000
		log.info(
			LOG_PREFIX
			.. ": input="
			.. input
			.. " bucket="
			.. tostring(#bucket)
			.. " prepare_ms="
			.. string.format("%.3f", elapsed_ms)
		)
	end

	for _, entry in ipairs(bucket) do
		if not needs_filter or starts_with(entry.code, lookup_input) then
			yield(make_candidate(segment, input, entry))
		end
	end
end

return {
	init = function(_env)
		load_entries()
	end,
	func = translator,
}
