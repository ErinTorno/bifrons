require "editor/tools/common"
require "editor/tools/move"
require "editor/tools/grid_toggle"
require "editor/tools/point_placer"
require "editor/level"

g_editor_handle = nil

function on_init()
    local tool_buttons = {}

    for _, name in ipairs(Tools.ordered) do
        table.insert(tool_buttons, Tools.ui_button(name))
    end

    local git_url = "https://github.com/ErinTorno/bifrons"
    local lua_url = git_url .. "/blob/master/docs/Lua Api Directory.md"
    local is_about_window_open = Atom.create(false)

    g_editor_handle = UI.compile(
        UI.verticalpanel {
            UI.menu {
                UI.button {
                    menubar  = "File", -- File -- File -- File -- File -- File -- File -- File -- F
                    text     = Text.new("Open"),
                    on_click = function()
                        local file = File.dialog({
                            directory = "assets",
                            filters   = { level = {"level.ron"}, },
                        })
                        Log.info(file)
                    end,
                },
                UI.button {
                    text     = Text.new("Save level"),
                },
                UI.button {
                    text     = Text.new("Save level as..."),
                },
                UI.button {
                    text     = Text.new("Exit"),
                    on_click = UI.queue_app_exit,
                },
                UI.hyperlink {
                    menubar  = "Help", -- Help -- Help -- Help -- Help -- Help -- Help -- Help -- H
                    text     = Text.new("Lua API"),
                    url      = lua_url,
                },
                UI.hyperlink {
                    text     = Text.new("Git repository"),
                    url      = git_url,
                },
                UI.button {
                    text     = Text.new("About"),
                    on_click = function()
                        is_about_window_open:map(function(b) return not b end)
                    end,
                },
            },
            -- UI.label "testing..."
        },
        UI.sidepanel {
            anchor = "left",
            UI.vertical(tool_buttons),
        },
        UI.sidepanel {
            anchor = "right",
            UI.label "todo :)",
        },
        UI.window {
            is_open      = is_about_window_open,
            title        = Text.new("About Lemegeton Editor Tabularum"),
            is_closeable = true,
            UI.label(Text.new("The")
                :style(TextStyle.from { italics = true, color = Color "wine" })
                :push(" Glorious Lemegeton Editor Tabularum,")
                :style(TextStyle.from { font = "monospace" })
                :push(" version ")
                :style(TextStyle.from { font = "monospace", color = Color "lime" })
                :push("0.1.0\n")),
            UI.hyperlink {
                text = Text.new("Git Repository @ " .. git_url),
                url  = git_url,
            },
        }
    )
    UI.show(g_editor_handle)
end