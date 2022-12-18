--!shared

local g_vars = {}
local g_material_handle = {}
local g_variant = "blank"
local g_all_variants = {
    "book_guy",
    "glasses_hat",
}
local g_all_colors = {
    "blue",
    "brown",
    "clay",
    "cream",
    "cornflower",
    "cyan",
    "ferra",
    "fuchsia",
    "gold",
    "green",
    "icy",
    "lavender",
    "lime",
    "olive",
    "orange",
    "pink",
    "purple",
    "red",
    "rose",
    "sand",
    "sierra",
    "teal",
    "white",
    "wine",
    "yellow",
}

function on_init()
    g_vars       = Vars.all(entity)
    g_variant    = g_vars.painting_variant or Random.value(g_all_variants)
    local handles = Material.handle_table(entity)
    local color  = g_vars.painting_color or Random.value(g_all_colors)
    
    local mat    = handles.painting:get()
    mat.texture  = format("props/furniture/paintings/{}.png", g_variant)
    mat.color    = color
    mat:apply(handles.painting)

    local frame_color = g_vars.painting_frame_color or Random.value(g_all_colors)
    local mat    = handles.frame:get()
    mat.color    = frame_color
    mat:apply(handles.frame)
end