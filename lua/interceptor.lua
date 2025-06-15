require("utils")

interceptor = {}

interceptor.intercept = function(entry)
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
	if utils.is_empty(entry.translations) then
		entry.translations = { translation_1, translation_2 }
	end
	return entry
end
