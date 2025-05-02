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
	--let mut res: Vec<ColoredString> = Vec::new();
	--res.push("Etymology:".bold());
	--res.push(etymology.normal());
	--return Some(NEWLINE.normal().joinwrap(res, LINE_WRAP_AT));
	local list = {}
	table.insert(list, apply_style("Etymology:", "bold"))
	table.insert(list, wrap_text_at(etymology_text, 80))
	return table.concat(list, "\n")
end

local function format_entry(entry)
	local content = {}
	if entry.etymology then
		table.insert(content, format_etymology(entry.etymology))
	end

	--    format!("{}{}{}", NEWLINE, horizontal_line, NEWLINE)
	--        .normal()
	--        .join(entries)

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
	print(format_entry(entry))

	return entry
end
