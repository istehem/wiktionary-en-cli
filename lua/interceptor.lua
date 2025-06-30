local api = require("wiktionary_api")
local utils = require("utils")

local interceptor = {}

interceptor.intercept = function(entry)
	local history_filename = api.project_folder() .. "/history.txt"
	local history = io.open(history_filename, "a")
	history:write(entry.word .. "\n")
	history:close()
	return entry
end

return interceptor
