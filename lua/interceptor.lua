local db_client = require("wiktionary_db_client")
local features = require("features")
local utils = require("utils")

local interceptor = {}

local function tick(entry)
  entry.last_seen_at = entry.now_seen_at
  entry.now_seen_at = os.time()
  entry.count = entry.count + 1
end

interceptor.intercept = function(entry)
  local query = { word = entry.word }
  local existing = db_client:find_one_in_collection(features.history.name, query)

  if existing then
    tick(existing)
    local update = {
      last_seen_at = existing.last_seen_at,
      now_seen_at = existing.now_seen_at,
      count = existing.count,
    }
    db_client:update_one_in_collection(features.history.name, query, update)
  else
    local now = os.time()
    local history_entry = {
      word = entry.word,
      last_seen_at = now,
      now_seen_at = now,
      count = 1,
    }
    db_client:insert_one_into_collection(features.history.name, history_entry)
  end
  return entry
end

return interceptor
