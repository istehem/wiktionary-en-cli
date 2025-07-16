--local api = require("wiktionary_api")
--local utils = require("utils")
local db_client = require("wiktionary_db_client")

local interceptor = {}

interceptor.intercept = function(entry)
	db_client:write_to_history(entry.word)
	--entry.word = api.apply_color("★", "green") .. entry.word
	return entry
end

return interceptor
