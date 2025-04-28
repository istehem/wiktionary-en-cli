function one_plus_one()
	return 1 + 1
end

config = function()
	return {
		language = "sv",
		message = "Hello World!",
	}
end

--[[
config = {
    language = "sv",
    message = "Hello World!"
}
--]]
local function is_empty(t)
	return next(t) == nil
end

intercept = function(entry)
	entry.word = apply_color(entry.word, "cyan")
	translation_1 = {
		lang = apply_style("en", "dimmed"),
		code = "en",
		word = "Hello",
	}
	translation_2 = {
		lang = apply_style("en", "dimmed"),
		code = "en",
		word = "Word!",
	}
	if is_empty(entry.translations) then
		entry.translations = { translation_1, translation_2 }
	end
	for k, v in pairs(entry) do
		--print(string.format('found key "%s" with value "%s"', k, v))
		if type(v) == "table" then
			--intercept(v)
		end
	end
	return entry
end
