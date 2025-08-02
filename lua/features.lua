local features = setmetatable({}, {
  __index = {
    history = {
      name = "history",
    },
  },
  __newindex = function()
    error("read only value")
  end,
})

return features
