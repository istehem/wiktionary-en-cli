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
	if type(v) == 'table' then
	    for k2, v2 in pairs(entry) do
	        print(string.format('found sub key "%s" with sub value "%s"', k2, v2))
	    end
	end
	print(string.format('found key "%s" with value "%s"', k, v))
    end
end
