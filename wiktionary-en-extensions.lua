local formatter = require("formatter")
local interceptor = require("interceptor")
local history = require("history")

extensions = {}
-- format Wiktionary results
extensions.format_entry = formatter.format_entry
-- format Wiktionary did-you-mean banner
extensions.format_did_you_mean_banner = formatter.format_did_you_mean_banner
-- intercept Wiktionary results
extensions.intercept = interceptor.intercept
-- show history
extensions.history = history.format
