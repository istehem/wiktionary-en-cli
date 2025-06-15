local formatter = require("formatter")
-- local interceptor = require("interceptor")

config = {}
-- set a default language
config.language = "en"
-- format Wiktionary results
config.format = formatter.format
-- intercept Wiktionary results
config.intercept = nil -- interceptor.intercept
