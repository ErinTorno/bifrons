-- Globals

local g_def_lobby    = {}
local g_lobby_themes = {}

-- Messages

--  -- Available to be called by other level's preview scripts via
--  Message.new("add_lobby_theme")
--      :to("levels/main_menu/main.lua")
--      :add_arg("my_theme_name")
--      :add_arg({
--          piece      = "levels/my_level/my_lobby.piece.ron", -- extension optional; if nil, then default lobby piece is used
--      })
--      :send()
function add_lobby_theme(name, config)
    g_lobby_themes[name] = config
end

-- Events

function on_init()
    g_def_lobby = {
        piece = "levels/main_menu/lobby.piece.ron",
    }
    g_lobby_themes.default = g_def_lobby
end

-- Utils