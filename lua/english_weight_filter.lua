---@diagnostic disable: undefined-global
-- rime_api, log, yield are provided by Rime runtime


local ranks
local weights

local function data_dir()
	if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
		return rime_api.get_user_data_dir()
	end
	return "."
end

local function is_english_word(text)
	return text:match("^[a-z][a-z]+$") ~= nil and #text > 3
end

local function load_weights()
	if weights ~= nil then
		return weights, ranks
	end

	ranks = {}
	weights = {}

	local file = io.open(data_dir() .. "/tiger.dict.yaml", "r")
	if file == nil then
		file = io.open("tiger.dict.yaml", "r")
	end
	if file == nil then
		return weights, ranks
	end

	local in_body = false
	local rank = 0
	for line in file:lines() do
		if in_body then
			local code, weight_text, text = line:match("^([^\t]+)\t([^\t]+)\t(.+)$")
			local weight = tonumber(weight_text)
			if code ~= nil and weight ~= nil and code == text and is_english_word(text) then
				rank = rank + 1
				weights[text] = weight
				ranks[text] = rank
			end
		elseif line == "..." then
			in_body = true
		end
	end

	file:close()

	if log ~= nil and log.info ~= nil then
		local count = 0
		for _ in pairs(weights) do
			count = count + 1
		end
		log.info("english_weight_filter loaded " .. count .. " words")
	end

	return weights, ranks
end

local function filter(input, env)
	local context_input = env.engine.context.input
	if not context_input:match("^[a-z]+$") then
		for cand in input:iter() do
			yield(cand)
		end
		return
	end

	local english_weights, english_ranks = load_weights()
	if english_weights == nil or english_ranks == nil then
		for cand in input:iter() do
			yield(cand)
		end
		return
	end

	local before_english = {}
	local english = {}
	local after_english = {}
	local seen_english = false

	for cand in input:iter() do
		local weight = english_weights[cand.text]
		if weight ~= nil then
			seen_english = true
			english[#english + 1] = { cand = cand, weight = weight }
		elseif seen_english then
			after_english[#after_english + 1] = cand
		else
			before_english[#before_english + 1] = cand
		end
	end

	table.sort(english, function(a, b)
		if a.weight == b.weight then
			local a_rank = english_ranks[a.cand.text] or math.huge
			local b_rank = english_ranks[b.cand.text] or math.huge
			return a_rank < b_rank
		end
		return a.weight > b.weight
	end)

	for _, cand in ipairs(before_english) do
		yield(cand)
	end
	for _, item in ipairs(english) do
		yield(item.cand)
	end
	for _, cand in ipairs(after_english) do
		yield(cand)
	end
end

return filter
