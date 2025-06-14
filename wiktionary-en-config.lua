-- Generic helper functions
-------------------------------------------------------------------------------
local function is_empty(t)
	return next(t) == nil
end

local function filter(arr, func)
	local result = {}
	for _, v in ipairs(arr) do
		if func(v) then
			table.insert(result, v)
		end
	end
	return result
end

local function has_value(tab, val)
	for _, value in ipairs(tab) do
		if value == val then
			return true
		end
	end
	return false
end

local function leftpad(text, length, pad_char)
	local assure_text = tostring(text)
	return length and string.rep(pad_char or " ", length - #assure_text) .. assure_text or assure_text
end

-- Helper functions
-------------------------------------------------------------------------------
local function calculate_pad_size(xs)
	return #tostring(#xs)
end

local function style_italic_dimmed(text, length)
	return api.apply_style(api.apply_style(leftpad(text, length), "italic"), "dimmed")
end

local function format_etymology(etymology_text)
	if not etymology_text then
		return nil
	end
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

local function sounds_to_strings(sounds, padding)
	local result = {}
	for _, v in pairs(sounds) do
		-- some sounds may not contain a pronunciation
		local index = #result + 1
		if v.ipa then
			local formatted =
				string.format(" %s. IPA: %s %s", style_italic_dimmed(index, padding), v.ipa, format_tags(v.tags))
			table.insert(result, formatted)
		end
		if v.enpr then
			local formatted =
				string.format(" %s. enPr: %s %s", style_italic_dimmed(index, padding), v.enpr, format_tags(v.tags))
			table.insert(result, formatted)
		end
		if v.other then
			local formatted =
				string.format(" %s. other: %s %s", style_italic_dimmed(index, padding), v.other, format_tags(v.tags))
			table.insert(result, formatted)
		end
	end
	return result
end

local function format_sounds(sounds)
	local result = {}
	local as_strings = sounds_to_strings(sounds, calculate_pad_size(sounds))
	if is_empty(as_strings) then
		return nil
	end

	table.insert(result, api.apply_style("Pronunciation", "bold"))
	table.insert(result, table.concat(as_strings, "\n"))
	return table.concat(result, "\n")
end

local function translations_to_strings(translations)
	local result = {}

	table.sort(translations, function(t1, t2)
		return t1.lang < t2.lang
	end)

	for _, v in pairs(translations) do
		local word = v.word and v.word or ""
		local lang = style_italic_dimmed(v.lang)
		local formatted = string.format(" %s) %s", lang, word)
		table.insert(result, formatted)
	end
	return result
end

local function translate_to(t)
	return has_value({ "en", "sv", "de", "fr", "es", "it" }, t.code)
end

local function format_translations(translations)
	local filtered_translations = filter(translations, translate_to)
	if is_empty(filtered_translations) then
		return nil
	end
	local list = {}
	table.insert(list, api.apply_style("Translations:", "bold"))
	table.insert(list, table.concat(translations_to_strings(filtered_translations), "\n"))
	return table.concat(list, "\n")
end

local function format_glosses(glosses)
	return table.concat(glosses, "\n")
end

local function examples_to_strings(examples)
	result = {}
	for i, v in ipairs(examples) do
		if v.text then
			local formatted = string.format("%s. %s", style_italic_dimmed(i), api.wrap_text_at(v.text, 80 - 1))
			table.insert(result, api.indent(formatted))
		end
	end

	return result
end

local function format_examples(examples)
	return table.concat(examples_to_strings(examples), "\n")
end

local function format_sense(sense, i)
	local result = {}
	local title = string.format("%s. %s", api.apply_style(i, "bold"), api.apply_style(format_tags(sense.tags), "bold"))
	table.insert(result, title)
	table.insert(result, api.wrap_text_at(format_glosses(sense.glosses), 80))
	if not is_empty(sense.examples) then
		table.insert(result, format_examples(sense.examples))
	end
	return table.concat(result, "\n")
end

local function senses_to_strings(senses)
	local result = {}
	for i, v in ipairs(senses) do
		table.insert(result, format_sense(v, i))
	end
	return result
end

local function format_senses(senses)
	if is_empty(senses) then
		return nil
	end
	return table.concat(senses_to_strings(senses), "\n")
end

local function format_related_word_sense(sense)
	if not sense then
		return ""
	end
	return string.format("(%s) ", sense)
end

local function related_words_to_strings(related_words, padding)
	local result = {}
	for i, v in ipairs(related_words) do
		local clarifications = {}
		if not is_empty(v.tags) then
			table.insert(clarifications, format_tags(v.tags))
		end
		if v.sense then
			table.insert(clarifications, format_related_word_sense(v.sense))
		end
		local formatted =
			string.format(" %s. %s %s", style_italic_dimmed(i, padding), v.word, table.concat(clarifications, " "))
		table.insert(result, formatted)
	end
	return result
end

local function format_related_words(related_words, category_title)
	if is_empty(related_words) then
		return nil
	end
	local list = {}
	table.insert(list, api.apply_style(category_title, "bold"))
	table.insert(list, table.concat(related_words_to_strings(related_words, calculate_pad_size(related_words)), "\n"))
	return table.concat(list, "\n")
end

-- Enable a configuration of standard values by defining a config function.
-- It may also be defined as a simple table
-------------------------------------------------------------------------------
config = function()
	return {
		language = "en",
	}
end

-- Enable custom formatting of results by defining a format function.
-------------------------------------------------------------------------------
function format(entry)
	entry.word = api.apply_color(entry.word, "cyan")

	local content = {}

	local formatted_etymology = format_etymology(entry.etymology)
	if formatted_etymology then
		table.insert(content, formatted_etymology)
	end
	local formatted_sounds = format_sounds(entry.sounds)
	if formatted_sounds then
		table.insert(content, formatted_sounds)
	end
	local formatted_senses = format_senses(entry.senses)
	if formatted_senses then
		table.insert(content, formatted_senses)
	end
	local formatted_synonyms = format_related_words(entry.synonyms, "Synonyms:")
	if formatted_synonyms then
		table.insert(content, formatted_synonyms)
	end
	local formatted_antonyms = format_related_words(entry.antonyms, "Antonyms:")
	if formatted_antonyms then
		table.insert(content, formatted_antonyms)
	end
	local formatted_translations = format_translations(entry.translations)
	if formatted_translations then
		table.insert(content, formatted_translations)
	end
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

-- Enable interception by defining an intercept function.
-------------------------------------------------------------------------------
--[[
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
