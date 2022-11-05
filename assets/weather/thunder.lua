local g_last_thunder = 0
local g_thunder_delay = 10

function on_init()
    g_last_thunder = Time.elapsed()
    math.randomseed(os.time())
end

function on_update()
    local now = Time.elapsed()
    if (now - g_last_thunder) >= g_thunder_delay then
        g_thunder_delay = 8 + math.random(1, 6)
    end
end