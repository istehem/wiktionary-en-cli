vim.g.rustaceanvim = {
  server = {
    default_settings = {
      ["rust-analyzer"] = {
        cargo = {
          extraEnv = {
            DICTIONARY_DB_PATH_PLACEHOLDER = "{}",
            LUA_DIR = "",
            DICTIONARY_EXTENSIONS = "",
            DICTIONARY_CONFIG = "",
            SONIC_HOST = "",
            SONIC_PASSWORD = "",
            COUCH_DB_HOST = "http://localhost:5984",
            COUCH_DB_PASSWORD = "",
            COUCH_DB_USER = "",
          },
          allFeatures = true,
        },
      },
    },
  },
}
