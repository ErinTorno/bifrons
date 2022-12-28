require "ui/editor/tools/common"

do
    local name      = "point_placer"
    local ac_color  = Color "red"
    local in_color  = Color "wine"
    local color     = Atom.create(in_color)
    local icon      = Image.load("ui/icons/point.png")
    local ac_size   = Vec2.new(48, 48)
    local in_size   = Vec2.new(44, 44)
    local size      = Atom.create(in_size)

    Tools.table[name] = {
        icon    = { color = color, image = icon, size = size },
        tooltip = Tools.easy_tooltip(ac_color, name, ": Place point entities in the world\n\n")
            :style(Tools.tt_color("yellow")):push("💡")
            :style(Tools.tt_color("grey"  )):push("Point entities are tagged points in space without any size"),
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