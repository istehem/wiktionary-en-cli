local formatter = require("formatter")
local interceptor = require("interceptor")

config = {}
-- set a default language
config.language = "en"
-- format Wiktionary results
config.format_entry = formatter.format_entry
-- format Wiktionary did-you-mean banner
config.format_did_you_mean_banner = formatter.format_did_you_mean_banner
-- intercept Wiktionary results
config.intercept = interceptor.intercept
