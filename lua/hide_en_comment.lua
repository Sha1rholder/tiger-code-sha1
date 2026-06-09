---@diagnostic disable: undefined-global

---隐藏包含ASCII候选词的补全注释
---使用单个空格作为替代注释，因为空注释在 Weasel 0.17/librime-lua 中会回退显示原始补全提示

local function has_latin_letter(text)
	return text ~= nil and text:find("[A-Za-z]") ~= nil
end

local function candidate_start(cand)
	return cand.start or cand._start or 0
end

local function candidate_end(cand)
	return cand._end or cand.start or cand._start or 0
end

local function filter(input, _env)
	for cand in input:iter() do
		if has_latin_letter(cand.text) then
			-- 重新构建候选词以防码表补全条目的继承注释泄露
			local replacement = Candidate(cand.type, candidate_start(cand), candidate_end(cand), cand.text, " ")
			replacement.quality = cand.quality
			yield(replacement)
		else
			yield(cand)
		end
	end
end

return {
	func = filter,
}
