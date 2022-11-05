local colors = {}

function on_init()
    print("testing_house.lua on_init ran for " .. Level.name() .. " loaded at " .. string(Time.elapsed() .. " seconds"))
    table.insert(colors, Color.hex("#100f14"))
    -- table.insert(colors, Color.hex("#921b4b"))
    -- table.insert(colors, Color.hex("#442434"))
    -- table.insert(colors, Color.hex("#494129"))
    -- table.insert(colors, Color.hex("#55231a"))
    -- table.insert(colors, Color.hex("#001522"))
    -- table.insert(colors, Color.hex("#750687"))
    -- table.insert(colors, Color.hex("#067587"))
    Log.info("testing format {}, {}, {}, {}, and {}, and {}, and entity {}", 1, 3.14, 'str', true, {x = 5, y = 10}, Color.hex("921b4b"), entity)
    -- Event.register("o")
    -- Color.set_filter(function(color)
    --     color.hue = color.hue + 180
    --     return color
    -- end)
    
    local light = Light.new(LightKind.point(1600, 20, 0))
    -- local light = Light.new(LightKind.directional(28000.0, 10000.0))
    -- local light = Light.new(LightKind.default_spotlight(Vec3.new(0, 0, 0)))
    light.color = Color.hex("#f3d79b")
    light.pos   = Vec3.new(0, 4, 0)
    light.anim  = LightAnim.sin(0.15, 1.25) -- 0.1 * intensity over 0.5 seconds
    -- light:spawn()
end

local g_timeofday       = 1
local g_bkg_switch_secs = 45.0
function on_update()
    local secs = finite_or(Time.elapsed() % (g_bkg_switch_secs * #colors), 0)

    local next_timeofday = math.floor(secs / g_bkg_switch_secs) + 1
    if g_timeofday ~= next_timeofday then
        g_timeofday = next_timeofday
        Color.set_background(colors[g_timeofday])
    end
end