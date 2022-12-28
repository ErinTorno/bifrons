require "ui/editor/tools/common"

do
    local name       = "grid_toggle"
    local ac_color   = Color "lime"
    local in_color   = Color "teal"
    local color      = Atom.create(in_color)
    local icon       = Image.load("ui/icons/grid_toggle.png")

    local tooltip_intro = Tools.easy_tooltip(ac_color, name, ": Toggles a grid for precise movement (including snap options)\n\n")
    local tooltip       = Atom.create(nil)
    function enable()
        color:set(ac_color)
        
        local new_tooltip = tooltip_intro:clone()
            :style(Tools.tt_color("blue"))
            :push("Grid is currently enabled")
        tooltip:set(new_tooltip)
    end
    function disable()
        color:set(in_color)

        local new_tooltip = tooltip_intro:clone()
            :style(Tools.tt_color("red"))
            :push("Grid is currently disabled")
        tooltip:set(new_tooltip)
    end
    disable()

    Tools.table[name] = {
        icon    = { color = color, image = icon, size = Vec2.new(44, 44) },
        tooltip = tooltip,
        on_button_click = function()
            Tools.is_grid_toggled = not Tools.is_grid_toggled
            if Tools.is_grid_toggled then
                enable()
            else
                disable()
            end
        end,
    }
    table.insert(Tools.ordered, name)
end