local formatter = require("formatter")
local interceptor = require("interceptor")
local history = require("history")

Extensions = {}
-- format Wiktionary results
Extensions.format_entry = formatter.format_entry
-- format Wiktionary did-you-mean banner
Extensions.format_did_you_mean_banner = formatter.format_did_you_mean_banner
-- intercept Wiktionary results
Extensions.intercept = interceptor.intercept
-- handle history
Extensions.history = history.main
