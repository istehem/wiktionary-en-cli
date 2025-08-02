local db_client = require("wiktionary_db_client")
local features = require("features")
local utils = require("utils")

local interceptor = {}

--[[
pub fn tick(&mut self) {
        self.last_seen_at = self.now_seen_at;
        self.now_seen_at = Utc::now();
        self.count += 1;
}
--]]

local function tick(entry)
  entry.last_seen_at = entry.now_seen_at
  entry.now_seen_at = os.time()
  entry.count = entry.count + 1
end

interceptor.intercept = function(entry)
  local query = { word = entry.word }
  local existing = db_client:find_one_in_collection(features.history.name, query)

  if existing then
    -- update
    tick(existing)
    print(utils.format_date(existing.last_seen_at))
    print(utils.format_date(existing.now_seen_at))
    print(existing.count)
  else
    -- insert
    print("not found")
  end
  return entry
end

return interceptor
