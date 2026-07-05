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
          },
          allFeatures = true,
        },
      },
    },
  },
}
