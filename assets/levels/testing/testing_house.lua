TH = TH or {}
TH.levels = TH.levels or {}

function on_init()
    Log.info("testing_house.lua on_init ran on behalf of {}; loaded at {} seconds", "testing_house.level.ron", Time.elapsed())
    -- local light = Light.new(LightKind.point(1600, 20, 0))
    -- local light = Light.new(LightKind.directional(28000.0, 10000.0))
    -- local light = Light.new(LightKind.default_spotlight(Vec3.new(0, 0, 0)))
    -- light.color = Rgba.hex("#f3d79b")
    -- light.pos   = Vec3.new(0, 4, 0)
    -- light.anim  = LightAnim.sin(0.15, 1.25) -- 0.1 * intensity over 0.5 seconds
    -- light:spawn()
    -- for _, light_ety in ipairs(Query.named("foyer_lights"):entities(world)) do
    --     light_ety:hide()
    -- end
    -- Level.reveal()

    -- let's cause an infinite loop!
    -- Level.change("levels/testing/testing_house")

    -- Palette.load("palettes/black_and_white"):on_load(function(handle)
    --     Palette.swap(handle)
    -- end)
    -- Palette.load("palettes/black_and_white"):on_load(Palette.swap)
    Palette.load("palettes/default"):on_load(function(handle)
        local palette = handle:get()
        -- for name, rgba in pairs(palette) do
        --     Log.info("{} = {}", name, rgba)
        -- end
        Log.info("default has {} colors", #palette)
        -- local prev_red = palette:set("red", Rgba.new(1, 0, 0))
        -- Log.info("prev red was {}", prev_red)
    end)
    -- Palette.load("palettes/woodblock"):on_load(Palette.swap)

    TH.levels.kitchen = Level.load("levels/pieces/kitchen")
    TH.levels.kitchen:on_load(function(handle)
        local level = handle:get()
        level:spawn {
            parent   = entity,
            name     = "kitchen",
            position = Vec3.new(0, 0, -10),
        }
    end)
    -- Palette.load("palettes/aom"):on_load(Palette.swap)
    -- Palette.load("palettes/default"):on_load(function(handle)
    --     local palette = handle:get()
    --     palette.background = "red"
    --     palette:apply(handle)
    -- end)
end

function on_update(time)
    -- g_lantern_islit = not g_lantern_islit
    -- Message.new("set_lit")
    --     :add_arg(g_lantern_islit)
    --     :to_script("items/lantern.lua")
    --     :send()
end

function on_room_reveal(name, entity)
    Log.info("testing_house.lua on_room_reveal {} = ", name, entity)
    -- Message.new("on_whatever")
    --     :add_arg(ctx.name)
    --     :to_script("levels/testing/testing_house.lua")
    --     :send()
end

function on_whatever(room_name)
    Log.info("testing_house.lua on_whatever for {}", room_name)
end