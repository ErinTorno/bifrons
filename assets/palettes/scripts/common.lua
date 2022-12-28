--!shared

-- `ratio` the percentage of color left remaining.
--   1 is all color, 0.5 half, and 0 is no color.
function desaturate(rgba, handle, ratio)
    local r = ratio or 0
    rgba.saturation = rgba.saturation * Math.clamp(r, 0, 1)
    return rgba
end