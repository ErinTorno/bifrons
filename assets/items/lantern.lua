local g_light
local g_islit
local g_vars = {}

function new_light()
    local light = Light.new(LightKind.point(7000.0, 20., 0.1))
    light.label = "lantern_light"
    light.color = g_vars.lantern_color or Rgba.hex("#e09b4d")
    light.anim  = LightAnim.sin(0.25, 0.75)
    light.pos   = Vec3.new(0, 0.1846, 0)
    return light
end

function set_lit(is_lit)
    g_islit = is_lit
    if is_lit then
        local handles = Material.handle_table(entity)
        local outline = handles.out:get()
        outline.color = g_vars.lantern_color or Rgba.hex("#e09b4d")
        outline.emissive_color = g_vars.lantern_emissive or Rgba.hex("#a17d37")
        outline:apply(handles.out)

        local fire = handles.fire:get()
        local fire_color = g_vars.lantern_color or Rgba.hex("#e09b4d")
        fire_color.a        = 0.75
        fire.color          = fire_color
        fire.emissive_color = fire_color
        fire:apply(handles.fire)

        if g_light then
            Entity.show(g_light)
        end
    else
        if g_light then
            Entity.hide(g_light)
        end

        local handles = Material.handle_table(entity)
        local outline = handles.out:get()
        outline.color          = g_vars.lantern_frame_color or Rgba.hex("#4b4158")
        outline.emissive_color = Rgba.black
        outline:apply(handles.out)

        local fire = handles.fire:get()
        fire.color          = Rgba.clear
        fire.emissive_color = Rgba.clear
        fire:apply(handles.fire)
    end
end

-- Events

function on_init()
    g_vars = Vars.all(entity)
    g_light = new_light():spawn()
    entity:add_child(g_light)
    set_lit(true)
end

function on_interact(ctx)
    if g_islit then
        Prompt.new("extinguish", function()
            set_lit(false)
        end):add_to(ctx.prompts)
    else
        Prompt.new("light", function()
            set_lit(true)
        end):add_to(ctx.prompts)
    end
end

function on_use(ctx)
    local target = ctx.target
    local tags = Entity.tags(target)
    if tags.monster then
        Prompt.new("throw", function()
            Log.info("todo")
        end):enabled(g_islit):add_to(ctx.prompts)
    elseif tags.flammable then
        Prompt.new("ignite", function()
            Log.info("todo")
        end):enabled(g_islit):add_to(ctx.prompts)
    end
end

function on_equip()
end

function on_unequip()
end