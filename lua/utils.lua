-- Generic helper functions
-------------------------------------------------------------------------------
local utils = {}

utils.is_empty = function(t)
	return next(t) == nil
end

utils.filter = function(arr, func)
	local result = {}
	for _, v in ipairs(arr) do
		if func(v) then
			table.insert(result, v)
		end
	end
	return result
end

utils.has_value = function(tab, val)
	for _, value in ipairs(tab) do
		if value == val then
			return true
		end
	end
	return false
end

utils.leftpad = function(text, length, pad_char)
	local assure_text = tostring(text)
	return length and string.rep(pad_char or " ", length - #assure_text) .. assure_text or assure_text
end

return utils
