local M = {}

local function data_dir()
  if rime_api ~= nil and rime_api.get_user_data_dir ~= nil then
    return rime_api.get_user_data_dir()
  end
  return "."
end

local function is_english_word(text)
  return text:match("^[a-z][a-z]+$") ~= nil and #text > 4
end

function M.init(env)
  env.weights = {}

  local file = io.open(data_dir() .. "/tiger.dict.yaml", "r")
  if file == nil then
    file = io.open("tiger.dict.yaml", "r")
  end
  if file == nil then
    return
  end

  local in_body = false
  for line in file:lines() do
    if in_body then
      local code, weight_text, text = line:match("^([^\t]+)\t([^\t]+)\t(.+)$")
      local weight = tonumber(weight_text)
      if code ~= nil and weight ~= nil and code == text and is_english_word(text) then
        env.weights[text] = weight
      end
    elseif line == "..." then
      in_body = true
    end
  end

  file:close()

  if log ~= nil and log.info ~= nil then
    local count = 0
    for _ in pairs(env.weights) do
      count = count + 1
    end
    log.info("english_weight_filter loaded " .. count .. " words")
  end
end

function M.func(input, env)
  local before_english = {}
  local english = {}
  local after_english = {}
  local seen_english = false

  for cand in input:iter() do
    local weight = env.weights[cand.text]
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
      return a.cand.text < b.cand.text
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

return M
