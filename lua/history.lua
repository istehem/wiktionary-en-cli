local constants = require("constants")
local api = require("wiktionary_api")
local db_client = require("wiktionary_db_client")
local utils = require("utils")
local features = require("features")

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

local function format_all_entries()
  local formatted_entries = {}
  local documents = db_client:find_in_collection(features.history.name, {})

  for _, entry in ipairs(documents) do
    table.insert(formatted_entries, format_history_entry(entry))
  end

  return { result = table.concat(formatted_entries, "\n") }
end

local function delete()
  local query = {}
  local count = db_client:delete_in_collection(features.history.name, query)
  return { result = string.format("deleted %s history entries", count) }
end

local function index()
  local keys = { word = 1 }
  db_client:create_index_for_collection(features.history.name, keys)
  return { result = "index created on key 'word'" }
end

local function count()
  return { result = db_client:count_documents_in_collection(features.history.name) }
end

history.main = function(options)
  if not utils.is_empty(options) then
    for _, option in ipairs(options) do
      if option == "delete" then
        return delete()
      elseif option == "index" then
        return index()
      elseif option == "count" then
        return count()
      else
        error_msg = string.format("unknown option '%s'", option)
        return { result = error_msg, error = "unknown_option" }
      end
    end
    error("should be unreachable")
  else
    return format_all_entries()
  end
end

return history
