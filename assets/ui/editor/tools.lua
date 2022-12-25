Tools = Tools or {}

Tools.table   = Tools.table or {}
Tools.ordered = Tools.ordered or {}

Tools.set_active = function(name)
    Tools.active = name
    for k, tool in pairs(Tools.table) do
        if tool.on_tool_change then
            tool.on_tool_change(name)
        end
    end
end

Tools.ui_button = function(name)
    local tool = Tools.table[name]
    return UI.imagebutton {
        image    = tool.icon.image,
        color    = tool.icon.color,
        size     = tool.icon.size,
        on_click = tool.on_button_click,
    }
end

-- Tool behavior impl

do
    local name      = "select"
    local ac_color  = Color "gold"
    local in_color  = Color "coffee"
    local color     = Atom.create(in_color)
    local icon      = Image.load("ui/icons/select.png")
    local ac_size   = Vec2.new(48, 48)
    local in_size   = Vec2.new(44, 44)
    local size      = Atom.create(in_size)

    Tools.table[name] = {
        icon = { color = color, image = icon, size = size },
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
    Tools.active = name -- default tool
    Tools.table[name].on_tool_change(name)
end

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
        icon = { color = color, image = icon, size = size },
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

do
    local name       = "grid_toggle"
    local ac_color   = Color "lime"
    local in_color   = Color "teal"
    local color      = Atom.create(in_color)
    local icon       = Image.load("ui/icons/grid_toggle.png")
    local is_toggled = false

    Tools.table[name] = {
        icon = { color = color, image = icon, size = Vec2.new(44, 44) },
        on_button_click = function()
            is_toggled = not is_toggled
            if is_toggled then
                color:set(ac_color)
            else
                color:set(in_color)
            end
        end,
    }
    table.insert(Tools.ordered, name)
end