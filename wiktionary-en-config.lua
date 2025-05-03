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
	for k, v in pairs(entry) do
		--print(string.format('found key "%s" with value "%s"', k, v))
		if type(v) == "table" then
			--intercept(v)
		end
	end
--]]

local function is_empty(t)
	return next(t) == nil
end

local function format_etymology(etymology_text)
	local list = {}
	table.insert(list, apply_style("Etymology:", "bold"))
	table.insert(list, wrap_text_at(etymology_text, 80))
	return table.concat(list, "\n")
end

local function translations_to_strings(translations)
	local result = {}

	table.sort(translations, function(t1, t2)
		return t1.lang < t2.lang
	end)

	for _, v in pairs(translations) do
		local word = v.word and v.word or ""
		local lang = apply_style(v.lang, "dimmed")
		lang = apply_style(lang, "italic")
		local formatted = string.format(" %s) %s", lang, word)
		table.insert(result, formatted)
	end
	return result
end

local function format_translations(translations)
	if is_empty(translations) then
		return nil
	end
	local list = {}
	table.insert(list, apply_style("Translations:", "bold"))
	table.insert(list, table.concat(translations_to_strings(translations), "\n"))
	return table.concat(list, "\n")
end

local function format_entry(entry)
	local content = {}
	if entry.etymology then
		table.insert(content, format_etymology(entry.etymology))
	end
	if entry.translations then
		table.insert(content, format_translations(entry.translations))
	end

	local horizontal_line_str = horizontal_line()
	print(
		string.format(
			[[
%s
%s (%s)
%s
%s
	]],
			horizontal_line_str,
			entry.word,
			entry.pos,
			horizontal_line_str,
			table.concat(content, string.format("\n%s\n", horizontal_line_str))
		)
	)
	print(horizontal_line())

	-- standard formatter
	--return to_pretty_string(entry)
	return ""
end

intercept = function(entry)
	entry.word = apply_color(entry.word, "cyan")
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
	print(format_entry(entry))

	return entry
end
