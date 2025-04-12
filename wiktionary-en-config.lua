function one_plus_one()
    return 1 + 1
end

config = function()
    return {
        language = "sv",
        message = "Hello World!"
    }
end

--[[
config = {
    language = "sv",
    message = "Hello World!"
}
--]]

intercept = function(entry)
    for k, v in pairs(entry) do
	print(string.format('found key "%s" with value "%s"', k, v))
	if type(v) == 'table' then
	    intercept(v)
	end
    end
end
