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

-- Helper functions
-------------------------------------------------------------------------------
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
			local formatted = string.format(" %s. enPr: %s %s", api.apply_style(i, "italic"), v.enpr, format_tags(v.tags))
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

function translate_to(t)
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

function format_glosses(glosses)
	return table.concat(glosses, "\n")
end

function examples_to_strings(examples)
	result = {}
	for i, v in ipairs(examples) do
		if v.text then
			local formatted = string.format(
				"%s. %s",
				api.apply_style(api.apply_style(i, "italic"), "dimmed"),
				api.wrap_text_at(v.text, 80 - 1)
			)
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

-- Enable a configuration of standard values by defining a config function.
-- It may also be defined as a simple table
-------------------------------------------------------------------------------
config = function()
	return {
		language = "en",
		message = "Hello World!",
	}
end

-- Enable custom formatting of results by defining a format function.
-------------------------------------------------------------------------------
function format(entry)
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
