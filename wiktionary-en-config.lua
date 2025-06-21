local formatter = require("formatter")
-- local interceptor = require("interceptor")

config = {}
-- set a default language
config.language = "en"
-- format Wiktionary results
config.format = formatter.format
-- format Wiktionary did-you-mean banner
config.format_did_you_mean_banner = formatter.format_banner
-- intercept Wiktionary results
config.intercept = nil -- interceptor.intercept
