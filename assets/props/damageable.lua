function on_init()
    local health = Pool.cap(entity, "health")
    if health == nil or health <= 0 then
        Log.warn('props/damageable.lua applied to entity without "health" Pool; this script will do nothing')
    end
end

function on_take_damage()
    local health = Pool.current(entity, "health")
    if health and health <= Pool.cap(entity, "health") / 2 then
        Log.info("Setting damage material")
    end
end