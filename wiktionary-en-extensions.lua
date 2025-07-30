local formatter = require("formatter")
local interceptor = require("interceptor")

extensions = {}
-- format Wiktionary results
extensions.format_entry = formatter.format_entry
-- format Wiktionary did-you-mean banner
extensions.format_did_you_mean_banner = formatter.format_did_you_mean_banner
-- format an history entry
extensions.format_history_entry = formatter.format_history_entry
-- intercept Wiktionary results
extensions.intercept = interceptor.intercept
-- dummy extension
extensions.dummy = function()
  return { result = "Hello World!" }
end
