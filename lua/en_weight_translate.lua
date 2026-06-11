---@diagnostic disable: undefined-global

---按 tiger_sha1_en.dict.yaml 懒加载英文候选
---当输入不是任何主码表编码前缀时，先产出原始输入，方便直接上屏未知英文词
---英文候选整体排在主码表候选之后，但在本translator内保持英文词表顺序
---Lua产出的英文候选默认带尾随空格，便于连续输入英文

local MAX_PREFIX_LEN = 4
local EN_DICT_NAME = "tiger_sha1_en.dict.yaml"
local MAIN_DICT_NAME = "tiger_sha1_weasel.dict.yaml"
local LOG_PREFIX = "en_weight_translate"
local PERF_LOG_CONFIG = "en_weight_translate/enable_perf_log"
local APPEND_SPACE_CONFIG = "en_weight_translate/append_space_to_candidates"
local DEFAULT_APPEND_SPACE_TO_EN_CANDIDATES = true
local EN_CANDIDATE_SUFFIX = " "

local en_loaded = false
local entries_by_prefix = {}
local main_loaded = false
local main_prefixes = {}
local loading_main_dicts = {}
local append_space_to_en_candidates = DEFAULT_APPEND_SPACE_TO_EN_CANDIDATES

local function data_dir()
	if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
		return rime_api.get_user_data_dir()
	end
	return "."
end

local function open_file(filename)
	local file = io.open(data_dir() .. "/" .. filename, "r")
	if file ~= nil then
		return file
	end
	return io.open(filename, "r")
end

local function starts_with(text, prefix)
	return text:sub(1, #prefix) == prefix
end

local function add_entry(entry)
	-- 同一词条挂到1~4码前缀桶，长输入再在桶内过滤，避免每次全表扫描
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

local function add_main_code(code)
	-- 记录主码表所有编码前缀，用于判断raw input是否会干扰中文候选
	for len = 1, #code do
		main_prefixes[code:sub(1, len)] = true
	end
end

local function split_tabs(line)
	local fields = {}
	local start = 1
	while true do
		local tab_start, tab_end = line:find("\t", start, true)
		if tab_start == nil then
			fields[#fields + 1] = line:sub(start)
			break
		end
		fields[#fields + 1] = line:sub(start, tab_start - 1)
		start = tab_end + 1
	end
	return fields
end

local function dict_filename(dict_name)
	if dict_name:match("%.dict%.yaml$") then
		return dict_name
	end
	return dict_name .. ".dict.yaml"
end

local load_main_dict

local function load_imported_tables(import_tables)
	for _, dict_name in ipairs(import_tables) do
		load_main_dict(dict_filename(dict_name))
	end
end

load_main_dict = function(filename)
	-- 递归加载主码表避免环形引用
	if loading_main_dicts[filename] == true then
		return
	end
	loading_main_dicts[filename] = true

	local file = open_file(filename)
	if file == nil then
		if log ~= nil and log.warning ~= nil then
			log.warning(LOG_PREFIX .. ": failed to open " .. filename)
		end
		return
	end

	local in_body = false
	local in_columns = false
	local in_import_tables = false
	local columns = {}
	local import_tables = {}
	local code_index = 1

	for line in file:lines() do
		if in_body then
			local fields = split_tabs(line)
			local code = fields[code_index]
			if code ~= nil and code ~= "" then
				add_main_code(code)
			end
		elseif line == "..." then
			for index, column in ipairs(columns) do
				if column == "code" then
					code_index = index
					break
				end
			end
			in_body = true
		elseif line:match("^columns:%s*$") then
			in_columns = true
			in_import_tables = false
		elseif line:match("^import_tables:%s*$") then
			in_import_tables = true
			in_columns = false
		elseif line:match("^%S") then
			in_columns = false
			in_import_tables = false
		else
			if in_columns then
				local column = line:match("^%s*%-%s*(%S+)%s*$")
				if column ~= nil then
					columns[#columns + 1] = column
				end
			elseif in_import_tables then
				local dict_name = line:match("^%s*%-%s*(%S+)%s*$")
				if dict_name ~= nil then
					import_tables[#import_tables + 1] = dict_name
				end
			end
		end
	end

	file:close()
	load_imported_tables(import_tables)
end

local function load_main_prefixes()
	if main_loaded then
		return
	end

	main_prefixes = {}
	loading_main_dicts = {}
	main_loaded = true
	load_main_dict(MAIN_DICT_NAME)
end

local function load_entries()
	if en_loaded then
		return
	end

	entries_by_prefix = {}
	en_loaded = true

	local file = open_file(EN_DICT_NAME)
	if file == nil then
		if log ~= nil and log.warning ~= nil then
			log.warning(LOG_PREFIX .. ": failed to open " .. EN_DICT_NAME)
		end
		return
	end

	local in_body = false
	local count = 0
	local rank = 0
	for line in file:lines() do
		if in_body then
			-- 英文词典正文格式：编码<TAB>文本，文件顺序即补全权重顺序
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
		log.info(LOG_PREFIX .. ": loaded " .. count .. " entries from " .. EN_DICT_NAME)
	end
end

local function get_config_bool(env, path, default)
	if default == nil then
		default = false
	end

	local engine = env ~= nil and env.engine or nil
	local schema = engine ~= nil and engine.schema or nil
	local config = schema ~= nil and schema.config or nil
	if config == nil or config.get_bool == nil then
		return default
	end

	local ok, value = pcall(function()
		return config:get_bool(path)
	end)
	if not ok or value == nil then
		return default
	end
	return ok and value == true
end

local function segment_start(segment)
	return segment.start or segment._start or 0
end

local function segment_end(segment, input)
	return segment._end or segment["end"] or (segment_start(segment) + #input)
end

local function en_candidate_text(text)
	if append_space_to_en_candidates then
		return text .. EN_CANDIDATE_SUFFIX
	end
	return text
end

local function make_candidate(segment, input, entry)
	local cand = Candidate("english", segment_start(segment), segment_end(segment, input), en_candidate_text(entry.text),
		" ")
	-- 使英文候选词排在码表候选词之后，同时在translator内部保持词典顺序
	cand.quality = -entry.rank
	return cand
end

local function make_raw_candidate(segment, input)
	local cand = Candidate("raw_input", segment_start(segment), segment_end(segment, input), en_candidate_text(input),
		" ")
	cand.quality = 0
	return cand
end

local function translator(input, segment, env)
	load_entries()
	load_main_prefixes()

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
	-- 4码以内直接用对应前缀桶；大于等于4码为一桶
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

	if not main_prefixes[input] then
		-- 输入不再可能命中中文码表时，先给原始英文，避免未知词只能选补全项
		yield(make_raw_candidate(segment, input))
	end

	for _, entry in ipairs(bucket) do
		if not needs_filter or starts_with(entry.code, lookup_input) then
			yield(make_candidate(segment, input, entry))
		end
	end
end

return {
	init = function(env)
		append_space_to_en_candidates = get_config_bool(
			env,
			APPEND_SPACE_CONFIG,
			DEFAULT_APPEND_SPACE_TO_EN_CANDIDATES
		)
		load_entries()
		load_main_prefixes()
	end,
	func = translator,
}
