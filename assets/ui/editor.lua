require "ui/editor/tools"

g_editor_handle = nil

function on_init()
    local tool_buttons = {}
    for _, name in ipairs(Tools.ordered) do
        table.insert(tool_buttons, Tools.ui_button(name))
    end

    g_editor_handle = UI.add {
        UI.sidepanel {
            anchor = "left",
            UI.vertical(tool_buttons),
        },
    }
    UI.show(g_editor_handle)
end