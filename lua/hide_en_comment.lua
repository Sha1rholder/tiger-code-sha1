---@diagnostic disable: undefined-global

---Hide completion comments for candidates containing ASCII letters.
---A single space is used as the replacement comment because an empty comment can fall back to the original completion hint in Weasel 0.17/librime-lua.

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
			-- Rebuild the candidate instead of wrapping it, so inherited comments from table completion entries do not leak through.
			yield(Candidate(cand.type, candidate_start(cand), candidate_end(cand), cand.text, " "))
		else
			yield(cand)
		end
	end
end

return {
	func = filter,
}
