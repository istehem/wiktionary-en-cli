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
--[[
                doc! {
                    "word": &entry.word,
                },
                doc! {
                   "$set": doc!{
                     "last_seen_at": entry.last_seen_at.timestamp(),
                     "now_seen_at": entry.now_seen_at.timestamp(),
                     "count": entry.count as u32,
                }},
        Self {
            word,
            now_seen_at: now,
            last_seen_at: now,
            count: 1,
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
