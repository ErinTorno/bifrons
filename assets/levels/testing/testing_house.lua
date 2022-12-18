function on_init()
    Log.info("testing_house.lua on_init ran on behalf of {}; loaded at {} seconds", Level.name(), Time.elapsed())
    Log.info("testing format {}, {}, {}, {}, and {}, and {}, and entity {}", 1, 3.14, 'str', true, {x = 5, y = 10}, Rgba.hex("921b4b"), entity)
    
    -- local light = Light.new(LightKind.point(1600, 20, 0))
    -- local light = Light.new(LightKind.directional(28000.0, 10000.0))
    -- local light = Light.new(LightKind.default_spotlight(Vec3.new(0, 0, 0)))
    -- light.color = Rgba.hex("#f3d79b")
    -- light.pos   = Vec3.new(0, 4, 0)
    -- light.anim  = LightAnim.sin(0.15, 1.25) -- 0.1 * intensity over 0.5 seconds
    -- light:spawn()
    -- for _, light_ety in ipairs(Query.named("foyer_lights"):entities(world)) do
    --     Entity.hide(light_ety)
    -- end
    -- Level.reveal()
    -- local wrc = "world_reset_count"
    -- -- if not already in registry, this is skipped
    -- local exists = Registry.contains(wrc)
    -- Registry.update(wrc, function(i) return i + 1 end)
    -- Registry.alloc_if_new(wrc, function() return 0 end)
    -- local count = Registry.get(wrc)

    -- let's cause an infinite loop!
    -- Level.change("levels/testing/testing_house")

    -- Palette.load("palettes/black_and_white"):on_load(function(handle)
    --     Palette.change(handle)
    -- end)
    -- Palette.load("palettes/black_and_white"):on_load(Palette.change)
    -- Palette.load("palettes/default"):on_load(Palette.change)

    Level.spawn_piece("pieces/kitchen", {
        parent = entity,
        name   = "kitchen",
        reveal = true,
        pos    = Vec3.new(0, 0, -10),
    })
end

function on_update(time)
    -- g_lantern_islit = not g_lantern_islit
    -- Message.new("set_lit")
    --     :add_arg(g_lantern_islit)
    --     :to_script("items/lantern.lua")
    --     :send()
end

function on_room_reveal(ctx)
    Log.info("testing_house.lua on_room_reveal ctx = {}", ctx)
    -- Message.new("on_whatever")
    --     :add_arg(ctx.name)
    --     :to_script("levels/testing/testing_house.lua")
    --     :send()
end

function on_whatever(room_name)
    Log.info("testing_house.lua on_whatever for {}", room_name)
end