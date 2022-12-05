function desaturate(ctx)
    local ratio = ctx.config.ratio or 1
    local color = ctx.color
    color.saturation = color.saturation * math.clamp(1 - ratio, 0, 1)
    ctx(color)
end