local g_damage_frames = nil
local damage_mat = nil
local g_segment = 1
local g_unbreakable = false

function on_init()
    local vars = Vars.all(entity)
    
    g_unbreakable = Vars.unbreakable or false
    if not g_unbreakable then
        local health = Pool.cap(entity, "health")
        if health == nil or health <= 0 then
            Log.warn('props/damageable.lua applied to entity without "health" Pool; this script will do nothing')
        end

        local damage_frames = vars.damage_frames
        if damage_frames then
            if #damage_frames <= 1 then
                Log.warn('props/damageable.lua applied to prefab with one or less damage_frames; there will be nothing to change into')
            end
            g_segment = 1 / #damage_frames
            g_damage_frames = damage_frames
        else
            Log.warn('props/damageable.lua applied to prefab with no damage_frames')
        end
        
        local damage_mat = vars.damage_mat
        if damage_mat then
            local materials = Material.handle_table(entity)
            if materials then
                local handle = materials[damage_mat]
                if not handle then
                    Log.warn('props/damageable.lua damage_mat "{}" not found in table {}', damage_mat, materials)
                end
            end
            g_damage_mat = damage_mat
        else
            Log.warn('props/damageable.lua applied to prefab with no damage_mat to replace')
        end
        on_take_damage()
    end
end

local g_last_tex = nil
function on_take_damage()
    if not g_unbreakable and g_damage_frames then
        local health = Pool.ratio(entity, "health")
        for i = 1, #g_damage_frames do 
            if health <= (i * g_segment) then
                if g_last_tex ~= i then
                    g_last_tex = i
                    local asset  = g_damage_frames[i]
                    local handle = Material.handle_table(entity)[g_damage_mat]
                    if handle then
                        local mat   = handle:get()
                        mat.texture = string(g_damage_frames[i])
                        mat:apply(handle)
                    end
                end
                break
            end
        end
    end
end