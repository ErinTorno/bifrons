local g_vars = {}
local g_material_handle = {}
local g_variant = "blank"
local g_all_variants = {
    "book_guy",
    "glasses_hat",
}

function all_colors()
    return {
        Color.hex("#5e292f"),
        Color.hex("#e09b4d"),
        Color.hex("#9cabb1"),
        Color.hex("#785c3b"),
        Color.hex("#4b4158"),
        Color.hex("#63602e"),
        Color.hex("#deceb4"),
    }
end

function on_init()
    g_vars       = Vars.all(entity)
    g_variant    = g_vars.painting_variant or Random.value(g_all_variants)
    local handles = Material.handle_table(entity)
    local color  = g_vars.painting_color or Random.value(all_colors())
    
    local mat    = handles.painting:get()
    mat.texture  = format("props/furniture/paintings/{}.png", g_variant)
    mat.color    = color
    mat:apply(handles.painting)

    local frame_color = g_vars.painting_frame_color or Random.value(all_colors())
    local mat    = handles.frame:get()
    mat.color    = frame_color
    mat:apply(handles.frame)
end