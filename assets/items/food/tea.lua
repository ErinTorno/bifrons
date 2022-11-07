-- "name#bl": "Black tea",
-- "name#bs": "Blackberry sage tea",
-- "name#ch": "Chai tea",
-- "name#da": "Dandelion tea",
-- "name#eg": "Earl Grey tea",
-- "name#eg": "Expresso black tea",
-- "name#gr": "Green tea",
-- "name#lp": "Lemon pine tea",
-- "name#ma": "Macha tea",
-- "name#mn": "Mint green tea",
-- "name#mp": "Maple black tea",
-- "name#oo": "Oolong tea",
-- "name#pc": "Peach ginger black tea",
-- "name#pw": "Pomegranite white tea",
-- "name#ro": "Rooibos tea",
-- "name#sw": "Sakura white tea",

local g_teakind = nil

function update_tea()
    if g_teakind then
        local teacolors = {
            bl = Color.hex("#4b4158"),
            bs = Color.hex("#5f3c72"),
            ro = Color.hex("#8d3e29")
            sw = Color.hex("#faa1c4")
        }
        local color = teacolors[g_teakind]
        Log.info("teakind {} and color {}", g_teakind, color)
        local name = Lines.get(entity, "name#" .. g_teakind)
        Lines.set(entity, "name", name)
    end
end

function on_init()
    g_teakind = Random:key(teacolors)
    update_tea()
end

function on_lang_change()
    update_tea()
end

function on_save()
    Save.set(entity, { teakind = g_teakind })
end

function on_load()
    local data = Save.get(entity)
    if data then
        g_teakind = data.teakind
    end
end