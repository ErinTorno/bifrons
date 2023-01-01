require "editor/tools/common"
require "editor/tools/move"
require "editor/tools/grid_toggle"
require "editor/tools/point_placer"

g_editor_handle = nil

function on_init()
    local tool_buttons = {}
    for _, name in ipairs(Tools.ordered) do
        table.insert(tool_buttons, Tools.ui_button(name))
    end

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
                UI.button {
                    menubar  = "Help", -- Help -- Help -- Help -- Help -- Help -- Help -- Help -- H
                    text     = Text.new("Lua API"),
                },
                UI.button {
                    text     = Text.new("About"),
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
        }
    )
    UI.show(g_editor_handle)
end