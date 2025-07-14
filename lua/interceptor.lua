local api = require("wiktionary_api")
local utils = require("utils")

local interceptor = {}

interceptor.intercept = function(entry)
	db:write_to_history(entry.word)
	--entry.word = api.apply_color("★", "green") .. entry.word
	return entry
end

return interceptor
