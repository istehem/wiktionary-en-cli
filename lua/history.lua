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
  local total_width = constants.max_line_width

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
  local view = {
    document_name = "analytics",
    view_name = "word_count",
  }
  local view_content = db_client:get_view_in_collection(features.history.name, view)
  if not view_content.exists then
    return {
      result = "a count view doesn't exist yet, please create it by using the history extension 'create_count_view' option",
    }
  end
  local rows = view_content.rows
  if utils.is_empty(rows) then
    return { result = 0 }
  end
  return { result = view_content.rows[1].value.count }
end

local function create_count_view()
  local definition = {
    document_name = "analytics",
    view_name = "word_count",
    map = "function(doc) { doc.word && emit(doc.word, 1); }",
    reduce = "_stats",
  }
  local result = db_client:create_view_in_collection(features.history.name, definition)
  if result.created then
    return { result = "view created" }
  end
  return { result = result.message or "unkwon error" }
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
      elseif option == "create_count_view" then
        return create_count_view()
      else
        local error_msg = string.format("unknown option '%s'", option)
        return { result = error_msg, error = "unknown_option" }
      end
    end
    error("should be unreachable")
  else
    return format_all_entries()
  end
end

return history
