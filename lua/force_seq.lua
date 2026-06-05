local index = nil
local disabled = false
local MAX_SORTED_CANDIDATES = 1000

local function warn(message)
	if log and log.warning then
		log.warning("force_seq: " .. message)
	end
end

local function path_join(dir, filename)
	local sep = "\\"
	if dir:find("/", 1, true) then
		sep = "/"
	end
	if dir:sub(-1) == "/" or dir:sub(-1) == "\\" then
		return dir .. filename
	end
	return dir .. sep .. filename
end

local function get_user_data_dir()
	if not rime_api or not rime_api.get_user_data_dir then
		return nil
	end

	local ok, dir = pcall(rime_api.get_user_data_dir)
	if ok then
		return dir
	end
	return nil
end

local function read_dict(path)
	local file = io.open(path, "r")
	if not file then
		return nil, "cannot open " .. path
	end

	local by_key = {}
	local by_text = {}
	local after_sep = false
	local order = 0

	for line in file:lines() do
		line = line:gsub("\r$", "")
		if after_sep then
			if line ~= "" and not line:match("^%s*#") then
				local code, text = line:match("^([^\t]+)\t([^\t]+)")
				if code and text then
					order = order + 1
					local key = code .. "\t" .. text
					if not by_key[key] then
						by_key[key] = order
					end
					if not by_text[text] then
						by_text[text] = order
					end
				end
			end
		elseif line:match("^%.%.%.%s*$") then
			after_sep = true
		end
	end

	file:close()

	if order == 0 then
		return nil, "no dictionary entries parsed from " .. path
	end
	return { by_key = by_key, by_text = by_text }, nil
end

local function ensure_index()
	if disabled or index then
		return index
	end

	local dir = get_user_data_dir()
	if not dir or dir == "" then
		disabled = true
		warn("failed to get user data dir")
		return nil
	end

	local loaded, err = read_dict(path_join(dir, "tiger.dict.yaml"))
	if not loaded then
		disabled = true
		warn(err)
		return nil
	end

	index = loaded
	return index
end

local function candidate_text(cand)
	if cand and cand.text then
		return cand.text
	end
	return ""
end

local function lookup_order(dict_index, cand, input_code)
	local text = candidate_text(cand)
	return dict_index.by_key[input_code .. "\t" .. text] or dict_index.by_text[text]
end

function force_seq(input, env)
	local context_input = env.engine.context.input or ""
	local dict_index = ensure_index()

	if not dict_index or not context_input:match("^[A-Za-z]+$") then
		for cand in input:iter() do
			yield(cand)
		end
		return
	end

	local cands = {}
	local count = 0
	local iter = input:iter()

	for cand in iter do
		count = count + 1
		table.insert(cands, {
			cand = cand,
			seq = count,
			order = lookup_order(dict_index, cand, context_input),
		})
		if count >= MAX_SORTED_CANDIDATES then
			break
		end
	end

	table.sort(cands, function(a, b)
		if a.order and b.order then
			if a.order ~= b.order then
				return a.order < b.order
			end
			return a.seq < b.seq
		end
		if a.order then
			return true
		end
		if b.order then
			return false
		end
		return a.seq < b.seq
	end)

	for _, item in ipairs(cands) do
		yield(item.cand)
	end

	for cand in iter do
		yield(cand)
	end
end

return force_seq
