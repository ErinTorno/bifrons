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
        tooltip  = tool.tooltip
    }
end

Tools.tt_style = TextStyle.new()
Tools.tt_color = function(color)
    return Tools.tt_style:clone():with_color(color)
end
Tools.easy_tooltip = function(color, name, text)
    return Text.new()
        :style(Tools.tt_style:clone():with_color(color)):push(name)
        :style(Tools.tt_style):push(text)
end