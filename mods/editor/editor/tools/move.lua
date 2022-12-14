require "editor/tools/common"

do
    local name      = "move"
    local ac_color  = Color "blue"
    local in_color  = Color "purple"
    local color     = Atom.create(in_color)
    local icon      = Image.load("ui/icons/move.png")
    local ac_size   = Vec2.new(48, 48)
    local in_size   = Vec2.new(44, 44)
    local size      = Atom.create(in_size)

    Tools.table[name] = {
        icon    = { color = color, image = icon, size = size },
        tooltip = Tools.easy_tooltip(ac_color, name, ": Click on things to move them or edit their properties"),
        on_button_click = function()
            Tools.set_active(name)
        end,
        on_tool_change = function(tool_name)
            if tool_name == name then
                color:set(ac_color)
                size:set(ac_size)
            else
                color:set(in_color)
                size:set(in_size)
            end
        end,
    }
    table.insert(Tools.ordered, name)
end