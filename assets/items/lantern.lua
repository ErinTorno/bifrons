local g_light
local g_islit
local g_vars = {}

function new_light()
    local light = Light.new(LightKind.point(7000.0, 20., 0.1))
    light.label = "lantern_light"
    light.color = g_vars.lantern_color or Color.hex("#e09b4d")
    light.anim  = LightAnim.sin(0.25, 0.75)
    light.pos   = Vec3.new(0, 0.1846, 0)
    return light
end

function set_lit(is_lit)
    if is_lit then
        g_islit = true

        local handles = Material.handle_table(entity)
        local outline = handles.out:config()
        outline.color = g_vars.lantern_color or Color.hex("#e09b4d")
        outline.emissive_color = g_vars.lantern_emissive or Color.hex("#a17d37")
        outline:apply(handles.out)

        local fire = handles.fire:config()
        local fire_color = g_vars.lantern_color or Color.hex("#e09b4d")
        fire_color.a        = 0.5
        fire.color          = fire_color
        fire.emissive_color = fire_color
        fire:apply(handles.fire)

        if g_light then
            Entity.show(g_light)
        end
    else
        g_islit = false
        if g_light then
            Entity.hide(g_light)
        end

        local handles = Material.handle_table(entity)
        local outline = handles.out:config()
        outline.emissive_color = Color.black
        outline:apply(handles.out)

        local fire = handles.fire:config()
        local fire_color    = g_vars.lantern_color or Color.hex("#e09b4d")
        fire_color.a        = 0.0
        fire.color          = fire_color
        fire.emissive_color = fire_color
        
        fire:apply(handles.fire)
    end
end

-- Events

function on_init()
    g_vars = Var.all(entity)
    g_light = new_light():spawn()
    world:push_child(entity, g_light)
    set_lit(true)
end

function on_use()
    set_lit(not g_islit)
end

function on_equip()
end

function on_unequip()
end