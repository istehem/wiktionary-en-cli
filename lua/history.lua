local constants = require("constants")
local api = require("wiktionary_api")
local db_client = require("wiktionary_db_client")
local utils = require("utils")

local history = {}

local function format_history_entry(history_entry)
  local last_seen = utils.format_date(history_entry.last_seen_at)
  local word = history_entry.word
  local colored_word = api.apply_color(word, "cyan")
  local left_width = #word
  local right = string.format("Last seen: %s count: %s", last_seen, history_entry.count)
  total_width = constants.max_line_width

  local padding_length = math.max(total_width / 2 - left_width, 1)
  return colored_word .. string.rep(" ", padding_length) .. right
end

history.format = function()
  local formatted_entries = {}
  local document = db_client:find_all_in_history_as_doc()

  for _, entry in ipairs(document.find_result) do
    table.insert(formatted_entries, format_history_entry(entry))
  end

  return { result = table.concat(formatted_entries, "\n") }
end

return history
