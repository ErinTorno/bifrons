local g_light_owners = {}

function new_light()
    local light = Light.new(LightKind.point(7000.0, 20., 0.1))
    light.label = "lantern_light"
    light.color = Color.hex("#e09b4d")
    light.anim  = LightAnim.sin(0.25, 0.75)
    return light
end

-- Events

function on_equip()
    local light = new_light():spawn()
    g_light_owners[entity] = light
    world:push_child(entity, light)
end

function on_unequip()
    local light_entity = g_light_owners[entity]
    if light_entity then
        world:remove_child(entity, light_entity)
        world:despawn_recursive(light_entity)
        g_light_owners[entity] = nil
    end
end