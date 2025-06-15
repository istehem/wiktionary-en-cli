-- Enable interception by defining an intercept function.
-------------------------------------------------------------------------------
intercept = function(entry)
	translation_1 = {
		lang = "en",
		code = "en",
		word = "Hello",
	}
	translation_2 = {
		lang = "en",
		code = "en",
		word = "Word!",
	}
	if is_empty(entry.translations) then
		entry.translations = { translation_1, translation_2 }
	end
	return entry
end
--]]

config = {}
config.intercept = intercept
config.format = format
