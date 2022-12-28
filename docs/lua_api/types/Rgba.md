# 🌈 rgba

Fully evaluated RGB colors, with accessable color components. Color are expected to be between 0 and 1, with the exception of linear RGB colors, in which components above 1 can be useful in situations such as bloom.

See [color](Color.md) for colors with dynamic values.

## Rgba.hex
```lua
Rgba.hex = function(hex: string) -> rgba
```
Converts the given hex string into a sRGB value. Alpha hex byte is optional.
```lua
local blue        = Rgba.hex("#8490d5")
local mostly_rose = Rgba.hex("#de4e7680")
```

## Rgba.new
```lua
Rgba.new = function(r: number, g: number, b: number, a: number or nil) -> rgba
```

## Rgba.new_linear
```lua
Rgba.new_linear = function(r: number, g: number, b: number, a: number or nil) -> rgba
```

## Constants

## 🔳 Rgba.black
```lua
Rgba.black = function() -> rgba
```
Equivalent to `Rgba.new`

## 🟪 Rgba.fuchsia
```lua
Rgba.fuchsia = function() -> rgba
```

## 🔲 Rgba.white
```lua
Rgba.black = function() -> rgba
```

## RGBA components

## 🟥 rgba.r
```lua
rgba.r: number
```
The red component, usually between 0 and 1.

## 🟩 rgba.g
```lua
rgba.g: number
```
The green component, usually between 0 and 1.

## 🟦 rgba.b
```lua
rgba.b: number
```
The blue component, usually between 0 and 1.

## ⬛ rgba.a
```lua
rgba.a: number
```
The alpha component, usually between 0 (fully transparent) and 1 (fully opaque).

## HSL components

## rgba.hue
```lua
rgba.hue: number
```
The color's hue as expressed in the HSL colorspace.

## rgba.saturation
```lua
rgba.saturation: number
```
The color's saturation as expressed in the HSL colorspace.

## rgba.lightness
```lua
rgba.lightness: number
```
The color's lightness as expressed in the HSL colorspace.

## Misc

## rgba.is_linear
```lua
rgba.is_linear: <const> bool
```
True if this is linear RGB. (and not sRGB). See the [linear](#rgba:linear) and [srgb](#rgba:srgb) methods for converting between the two.

## rgba:difference_from
```lua
function rgba:difference_from(that: rgba) -> number
```
Gets the color difference between this rgba and that by converting them into the CIE L\*C\*h° colorspace. See [palette::Lch](https://docs.rs/palette/latest/palette/struct.Lch.html).

This method is associative, so the following is true:
```lua
a:difference_from(b) == b:difference_from(a)
```

## rgba:linear
```lua
function rgba:linear() -> rgba
```
Returns this color converted to linear RGB.

## rgba:srgb
```lua
function rgba:srgb() -> rgba
```
Returns this color converted to sRGB.

## rgba:__add
```lua
function rgba:__add(that: rgba) -> rgba
```
Adds `that` rgba to this in the linear rgba colorspace. Result will be converted back to sRGB if `is_linear` is false.

## rgba:__div
```lua
function rgba:__div(that: rgba or number) -> rgba
```
Divides this by `that` rgba or this's RGB components by `that` number in the linear rgba colorspace. Result will be converted back to sRGB if `is_linear` is false.

## rgba:__eq
```lua
function rgba:__eq(that: rgba) -> boo
```
Equality function. Components are compared up to 6 decimal places, and linear/srgb types are converted to the same colorspace before comparison.

## rgba:__mul
```lua
function rgba:__mul(that: rgba or number) -> rgba
```
Multiplies this by `that` rgba or this's RGB components by `that` number in the linear rgba colorspace. Result will be converted back to sRGB if `is_linear` is false.

## rgba:__sub
```lua
function rgba:__sub(that: rgba) -> rgba
```
Subtracts `that` rgba from this in the linear rgba colorspace. Result will be converted back to sRGB if `is_linear` is false.

## rgba:__tostring
```lua
function rgba:__tostring() -> string
```