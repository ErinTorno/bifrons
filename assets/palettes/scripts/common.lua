--!shared

function desaturate(rgba, ratio)
    local r = ratio or 1
    rgba.saturation = rgba.saturation * Math.clamp(1 - r, 0, 1)
    return rgba
end