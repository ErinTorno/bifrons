# 🌈 color

Dynamic colors with values that can change according to the current palette.

See [rgba](Rgba.md) for fully-evaluated colors.

## Color.background
```lua
Color.background: <const> color
```
The background color.

## Color.custom
```lua
Color.custom = function(rgba: rgba) -> color
```
Creates a color associated with `rgba`. This color may be [eval](#coloreval)uated as a different rgba value depending on the palettes `on_miss` behavior.

## Color.const
```lua
Color.const = function(rgba: rgba) -> color
```
Creates a color that always evaluates to the given `rgba`.

## Color.named
```lua
Color.named = function(name: string) -> color
```
Creates a color that corresponds to the `name`.

## Color.transparent
```lua
Color.transparent = function() -> color
```
Returns a completely transparent color.

## Color.__call
```lua
Color.__call = function(s: string) -> color
```
Creates a color by parsing the given `s`, following the same logic as .ron DynColor deserialization. As the only param is a string, this is frequently called without parentheses.
```
local r = Color("red")
local g = Color "green" -- no () needed!
```

## color:eval
```lua
function color:eval(palette: handle<palette> or nil) -> rgba
```
Returns the given [rgba](Rgba.md) value for this color using either the [`palette`](Palette.md), or the current one if [`palette`](Palette.md) is nil.

```lua
local cur_red  = red:eval()
local cool_red = red:eval(cool_palette_handle)
```

## color:__call
```lua
function color:__call(palette: handle<palette> or nil) -> rgba
```
Alias for [eval](#color:eval).
```lua
local rgba = my_color()
```

## color:__eq
```lua
function color:__eq(that: color) -> bool
```

## color:__tostring
```lua
function color:__tostring() -> string
```