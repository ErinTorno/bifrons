# 🧵 material

Materials are collections of [image](Image.md)s, [color](Color.md)s, and properties that determine how a texture is rendered to the world.

## Material.add_asset
```lua
Material.add_asset = function(mat: material) -> handle<material>
```
Adds the material as a new asset, and returns the [handle](Handle.md) to it.

```lua
local mat_handle = Material.add_asset(my_material)
```

## Material.handle_of
```lua
Material.handle_of = function(ety: entity) -> handle<material> or nil
```
Returns a [handle](Handle.md) to this [entity](Entity.md)'s material, if one exists.

To retrieve the material from the [handle](Handle.md), use [handle:get](Handle.md#handleget).

## Material.handle_table
```lua
Material.handle_table = function(ety: entity) -> { ["mat name"] = handle<material>, ... }
```
Returns a table to all materials in use by an entity, including those attached to child [entities](Entity.md).
```lua
for mat_name, handle in pairs(Material.handle_table(entity)) do
    local material = handle:get()
    Log.info("{} has emissive color {}", mat_name, material.emissive_color)
end
```

## material.alpha_blend
```lua
material.alpha_blend: bool
```
Whether to apply alpha blending to the material's image.
- `true` will blend transparent pixels in images with those behind them. This allows full use of all values of transparency.

  Many images using alpha blending layered on each other may render in the incorrect order. This is likely an engine bug. To avoid this, you are recommended to have your alpha blending materials backed by the `"background"` or a non-alpha blending layer.
- `false` will reduce transparent pixels to either full opaque or fully transparent. This might impact the quality for images making use of partial transparency, but is useful for performance reasons and to reduce the chance of layer rendering issues.

## material.atlas
```lua
material.atlas: atlas or nil
```
The [atlas](#🗺️-atlas) to separate the image into cells, if any.

## material.color
```lua
material.color: color
```
The [color](Color.md) applied to the surface of the material. Setter also accepts `string` color names and `rgba` custom colors.

Defaults to `Color "white"`

## material.emissive_color
```lua
material.emissive_color: color
```
The [color](Color.md) the material emits. This is added to the normal visible colors, so in darker environments this color becomes dominant in the texture.

Defaults to `Color.const(Rgba.black())`

## material.emissive_texture
```lua
material.emissive_texture: string or nil
```
The path to the material's emissive texture image.

## material.metallic
```lua
material.metallic: number
```
How metallic the material appears, usually within 0.0 (dielectric) to pure metallic (1.0).

Defaults to `0.01`

## material.mode
```lua
material.mode: matmode
```
The [matmode](#✂️-matmode) that determines how to tile or stretch these images across a surface.

Default is `MatMode.stretch`

## material.normal_texture
```lua
material.normal_texture: string or nil
```
The path to the material's normal map texture image.

## material.reflectance
```lua
material.reflectance: number
```
Specular intensity on a scale of usually `0.0` to `1.0`. Highlight is not visible at `0.0`, while at its maximum at `1.0`.

## material.texture
```lua
material.texture: string
```
The path to the material's texture image.

## material.unlit
```lua
material.unlit: bool
```
Defaults to `false`

## material:apply
```lua
function material:apply(h: handle<material>)
```
Updates the material referred to by `h` to this.

# 🗺️ atlas

Defines how a larger image might be broken up into smaller ones.

## Atlas.new
```lua
Atlas.new = function(rows: number, columns: number, width: number, height: number) -> atlas
```
Creates a new atlas for an image with `rows * columns` cells of `width * height` pixels dimensions.
```lua
local a = Atlas.new(4, 10, 256, 512)
```

## atlas.columns
```lua
atlas.rows: number
```

## atlas.width
```lua
atlas.width: number
```
Cell width in pixels.

## atlas.rows
```lua
atlas.rows: number
```

## atlas.height
```lua
atlas.height: number
```
Cell height in pixels.

## atlas:__eq
```lua
function atlas:__eq(that: atlas) -> bool
```

## atlas:__tostring
```lua
function atlas:__tostring() -> string
```

# ✂️ matmode

Defines how a material is displayed across a surface.

## MatMode.repeat
```lua
MatMode.repeat = function { step = number, mode = "identity" or "rotate180" } -> matmode
```
Material will repeat across every `step` units of distance, applying the `mode` operation each time.
- `"identity"`: repeat the same image each time.
- `"rotate180"`: rotate the image by 180° each iteration. This can help break up repetition.

Any key not assigned by the table will use its default value. (`step` as 1.0, `mode` as `"identity"`).

## MatMode.stretch
```lua
MatMode.stretch: matmode
```
Material will be stretched across the surface of the mesh.

## matmode:__eq
```lua
function matmode:__eq(that: matmode) -> bool
```

## matmode:__tostring
```lua
function matmode:__tostring() -> string
```