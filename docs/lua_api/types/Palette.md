# 🎨 palette

Palettes are sets of named colors and behaviors to do in case of a missed color lookup. These are used to evaluate [color](Color.md)s into [rgba](Rgba.md) values.

## Palette.add
```lua
Palette.load = function(p: palette) -> handle<palette>
```
Adds a palette as a new asset, and returns the [handle](types/Handle.md) to it. This handle can then be used to [swap](#paletteswap) to the current palette `p`.

```lua
local handle = Palette.add(my_palette)
```

## Palette.current
```lua
Palette.current = function() -> handle<palette>
```
Returns a [handle](types/Handle.md) to the current palette.

## Palette.load
```lua
Palette.load = function(path: string) -> handle<palette>
```
Loads a palette if it isn't already, and returns a [handle](types/Handle.md) to it.

## Palette.new
```lua
Palette.new = function() -> palette
```
Creates a new empty palette with no colors and all fields at their default values.

## Palette.swap
```lua
Palette.swap = function(h: handle<palette>) -> handle<palette> or nil
```
Switches the current palette to the one associated with `h`, and returns the previous handle, or nil if the palette could not be swapped.

## palette.background
```lua
palette.background: color
```
The background [color](Color.md). Setter also accepts `string` color names and `rgba` custom colors.

Default is `Color.const(Rgba.black())`.

## palette.on_miss
```lua
palette.on_miss: "identity" or "clamp" or rgba or table
```
This defines the behavior when a [color](Color.md) not named in this palette is encountered (either [custom](Color.md#colorcustom) or a missing [named](Color.md#colornamed)).

Default is `"identity"`.

If a [named](Color.md#colornamed) color is encountered that is defined in the default palette, but not this, then the default one is used as the [rgba](Rgba.md) value for all of these operations.

The following values are valid definitions for on_miss:
- `"clamp"`: [rgba](Rgba.md) values will be [clamp](#paletteclamp)ed
- `"identity"`: no changes will be made to [rgba](Rgba.md) values.
- `rgba`: will use this [rgba](Rgba.md) value for all missing colors.
- `{file = string, fn = string, params = array<any> or nil}`
  
  A lua script and function to use to evaluate each color.
  - `file`: the lua asset path
  - `fn`: the name of the function in that lua file to call
  - `params`: additional parameters to that function, if any. Each item in params will be accessable by that function as a parameter.

  The referenced function takes parameters in the following form:
  ```lua
  function on_miss_example_fn(input: rgba, palette_handle: handle<palette>, params: any...) -> rgba
  ```
  For safety reasons, this reference is expected to be to a separate .lua file, and that file is expected to exclusively contain on_miss callback functions and their utilities.

  **Warning:** Any `palette` or [`color`](Color.md) related operations run by these on_miss callback functions risk an infinite loop or failed RwLock operation. Similarly, using such operations even outside of on_miss functions in this lua instance might risk the same issue.

  #### Example
  ```lua
  palette.on_miss = {
      file   = "palettes/my_palette.lua",
      fn     = "adjust_color",
      params = { 0.5, true, Rgba.new(0.56, 0.34, 0.64) },
  }
  ```
  Could refer to the following function in `palettes/my_palette.lua` file.
  ```lua
  function adjust_color(rgba, palette_handle, ratio, is_invert, fuchsia)
      -- ...
      return rgba
  end
  ```

## palette.missing_rgba
```lua
palette.missing_rgba: rgba
```
The [rgba](Rgba.md) color used when a named color is evaluated that isn't supplied by this palette or the default one.

Default is `Rgba.fuchsia()`.

## palette:apply
```lua
function palette:apply(h: handle<palette>) -> handle<palette>
```
Sets the palette associated with the [handle](Handle.md) `h`, and returns a strong handle to that palette.

## palette:clamp
```lua
function palette:clamp(c: rgba) -> rgba
```
Returns the closest [rgba](Rgba.md) color in this palette to `c`, using the [difference_from](Rgba.md#rgbadifference_from) function to determine color distances. If this has no colors, then `c` is returned.

## palette:clone
```lua
function palette:clone() -> palette
```

## palette:get
```lua
function palette:get(name: string) -> rgba or nil
```
Gets the [rgba](Rgba.md) color for this name.

## palette:remove
```lua
function palette:remove(name: string) -> rgba or nil
```
Removes the [rgba](Rgba.md) color for this name, returning the previous color if there was one.

## palette:set
```lua
function palette:set(name: string, c: rgba) -> rgba or nil
```
Sets the [rgba](Rgba.md) color for this name, returning the previous color if there was one.

## palette:__eq
```lua
function palette:__eq() -> bool
```

## palette:__index
```lua
function palette:__index(name: string) -> rgba or nil
```
Alias for [get](#paletteset).
```lua
Log.info("rgba red is {}", palette.red)
```

## palette:__len
```lua
function palette:__len() -> int
```
Returns the number of colors in the palette.

## palette:__newindex
```lua
function palette:__newindex(name: string, c: rgba)
```
Alias for [set](#paletteset).
```lua
palette.super_green = Rgba.new_linear(0., 3., 0.)
```

## palette:__pairs
```lua
function palette:__pairs() -> iterator, palette, string
```
Allows use of `pairs(palette)` to iterate through name-rgba pairs.
```lua
for name, rgba in pairs(my_palette) do
    -- ...
end
```

## palette:__tostring
```lua
function palette:__tostring() -> string
```