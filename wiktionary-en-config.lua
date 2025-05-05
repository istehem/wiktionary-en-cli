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
	table.insert(list, api.apply_style("Etymology:", "bold"))
	table.insert(list, api.wrap_text_at(etymology_text, 80))
	return table.concat(list, "\n")
end

local function format_tags(tags)
	if is_empty(tags) then
		return ""
	end
	local result = {}
	for _, v in pairs(tags) do
		table.insert(result, string.format("(%s)", v))
	end
	return table.concat(result, ", ")
end

local function sounds_to_strings(sounds)
	local result = {}
	for i, v in ipairs(sounds) do
		if v.ipa then
			local formatted = string.format(" %s. IPA: %s %s", api.apply_style(i, "italic"), v.ipa, format_tags(v.tags))
			table.insert(result, formatted)
		end
		if v.enpr then
			local formatted =
				string.format(" %s. enPr: %s %s", api.apply_style(i, "italic"), v.enpr, format_tags(v.tags))
			table.insert(result, formatted)
		end
	end
	return result
end

local function format_sounds(sounds)
	local result = {}
	if is_empty(sounds) then
		return nil
	end

	table.insert(result, api.apply_style("Pronunciation", "bold"))
	table.insert(result, table.concat(sounds_to_strings(sounds), "\n"))
	return table.concat(result, "\n")
	--return result
end

local function translations_to_strings(translations)
	local result = {}

	table.sort(translations, function(t1, t2)
		return t1.lang < t2.lang
	end)

	for _, v in pairs(translations) do
		local word = v.word and v.word or ""
		local lang = api.apply_style(v.lang, "dimmed")
		lang = api.apply_style(lang, "italic")
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
	table.insert(list, api.apply_style("Translations:", "bold"))
	table.insert(list, table.concat(translations_to_strings(translations), "\n"))
	return table.concat(list, "\n")
end

function format_glosses(glosses)
	return table.concat(glosses, "\n")
end

function examples_to_strings(examples)
	result = {}
	for i, v in ipairs(examples) do
		if v.text then
			local formatted = string.format("%s. %s", api.apply_style(i, "italic"), api.wrap_text_at(v.text, 80 - 1))
			table.insert(result, api.indent(formatted))
		end
	end

	return result
end

function format_examples(examples)
	return table.concat(examples_to_strings(examples), "\n")
end

function format_sense(sense, i)
	local result = {}
	local title = string.format("%s. %s", api.apply_style(i, "bold"), api.apply_style(format_tags(sense.tags), "bold"))
	table.insert(result, title)
	table.insert(result, api.wrap_text_at(format_glosses(sense.glosses), 80))
	if not is_empty(sense.examples) then
		table.insert(result, format_examples(sense.examples))
	end
	return table.concat(result, "\n")
end

function senses_to_strings(senses)
	local result = {}
	for i, v in ipairs(senses) do
		table.insert(result, format_sense(v, i))
	end
	return result
end

function format_senses(senses)
	if is_empty(senses) then
		return nil
	end
	return table.concat(senses_to_strings(senses), "\n")
end

function format_entry(entry)
	entry.word = api.apply_color(entry.word, "cyan")
	
	local content = {}
	if entry.etymology then
		table.insert(content, format_etymology(entry.etymology))
	end
	if entry.sounds then
		table.insert(content, format_sounds(entry.sounds))
	end
	if entry.senses then
		table.insert(content, format_senses(entry.senses))
	end
	if entry.translations then
		table.insert(content, format_translations(entry.translations))
	end

	-- standard formatter
	--return api.to_pretty_string(entry)

	local horizontal_line = api.horizontal_line()
	return string.format(
		[[
%s
%s (%s)
%s
%s
	]],
		horizontal_line,
		entry.word,
		entry.pos,
		horizontal_line,
		table.concat(content, string.format("\n%s\n", horizontal_line))
	)
end

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
	print(format_entry(entry))

	return entry
end
