local constants = setmetatable({}, {
	__index = {
		max_line_width = 80,
	},
	__newindex = function()
		error("read only value")
	end,
})

return constants
