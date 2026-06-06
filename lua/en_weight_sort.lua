---@diagnostic disable: undefined-global

---按`tiger_sha1.dict.yaml`中的weight重排虎码候选，绕过Rime默认的码长优先排序

local MAX_WEIGHT_EN = 900000 -- from src/main.py

local entries = {}
local loaded = false

local function data_dir()
	if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
		return rime_api.get_user_data_dir()
	end
	return "."
end

local function open_dict()
	local file = io.open(data_dir() .. "/tiger_sha1.dict.yaml", "r")
	if file ~= nil then
		return file
	end
	return io.open("tiger_sha1.dict.yaml", "r")
end

---读取`tiger_sha1.dict.yaml`正文，按text缓存需要Lua重排的尾段词条
local function load_weights()
	if loaded then
		return
	end

	entries = {}
	loaded = true

	local file = open_dict()
	if file == nil then
		if log ~= nil and log.warning ~= nil then
			log.warning("tiger_weight_sort: failed to open tiger_sha1.dict.yaml")
		end
		return
	end

	local in_body = false
	local rank = 0
	local count = 0
	for line in file:lines() do
		if in_body then
			rank = rank + 1
			-- 词典正文格式：code<TAB>weight<TAB>text
			local code, weight_text, text = line:match("^([^\t]+)\t([^\t]+)\t(.+)$")
			local weight = tonumber(weight_text)
			if code ~= nil and text ~= nil and weight ~= nil and weight <= MAX_WEIGHT_EN then
				if entries[text] == nil then
					entries[text] = {}
					count = count + 1
				end
				entries[text][#entries[text] + 1] = {
					code = code:lower(),
					weight = weight,
					rank = rank,
				}
			end
		elseif line == "..." then
			in_body = true
		end
	end

	file:close()

	if log ~= nil and log.info ~= nil then
		log.info("tiger_weight_sort: loaded " .. count .. " weights from weight " .. MAX_WEIGHT_EN)
	end
end

local function starts_with(text, prefix)
	return text:sub(1, #prefix) == prefix
end

---返回当前输入前缀下某个候选text对应的最高weight和词典行序
local function lookup_weight(text, input)
	local text_entries = entries[text]
	if text_entries == nil then
		return nil
	end

	local weight = nil
	local rank = nil
	for _, entry in ipairs(text_entries) do
		if starts_with(entry.code, input) then
			if weight == nil or entry.weight > weight then
				weight = entry.weight
				rank = entry.rank
			elseif entry.weight == weight and entry.rank < rank then
				rank = entry.rank
			end
		end
	end

	return weight, rank
end

---weight降序；同weight按词典行序；仍相同则保持Rime原始相对顺序
local function compare(a, b)
	if a.weight ~= b.weight then
		return a.weight > b.weight
	end
	if a.rank ~= b.rank then
		return a.rank < b.rank
	end
	return a.index < b.index
end

---只重排普通字母输入的虎码候选；反查、标点等候选原样透传
local function filter(input, env)
	load_weights()

	local context_input = env.engine.context.input
	if context_input == nil or not context_input:match("^[A-Za-z]+$") then
		for cand in input:iter() do
			yield(cand)
		end
		return
	end

	local sorted = {}
	local unknown = {}
	local index = 0
	local lookup_input = context_input:lower()
	for cand in input:iter() do
		-- 只使用code以当前输入开头的词条，避免重复text拿到别的码的weight
		index = index + 1
		local weight, rank = lookup_weight(cand.text, lookup_input)
		if weight ~= nil then
			sorted[#sorted + 1] = {
				cand = cand,
				weight = weight,
				rank = rank or math.huge,
				index = index,
			}
		else
			unknown[#unknown + 1] = cand
		end
	end

	table.sort(sorted, compare)

	-- 未缓存候选来自更高权重的虎码和短码add，保持Rime原始相对顺序并位于重排尾段之前
	for _, cand in ipairs(unknown) do
		yield(cand)
	end
	for _, item in ipairs(sorted) do
		yield(item.cand)
	end
end

return {
	init = function(_env)
		load_weights()
	end,
	func = filter,
}
