# 💃🏾 textstyle

Text styles define how a section of [text](Text.md) is displayed.

Each field also has a with_ method which consumes the textstyle and returns an updated one. This can be useful when chaining with_ calls in a parameter.

```lua
local t = Text.new()
    :style(some_other_style:clone()
        :with_color    ("red")
        :with_font     (Font.load("fonts/My Scary Font.otf"))
        :with_font_size(28))
    :push("You can't foo without making sure to bar first!")
```

## TextStyle.from
```lua
TextStyle.from = function(schema: table) -> textstyle
```
Creates a new instance by merging `schema` with the default textstyle. This is equivalent to creating a default textstyle and calling its setter for each key-value pair in the schema.

This operation can be more efficient then constructing a new textstyle and applying a lot of operations to it.
```lua
let ts = TextStyle.from {
    color     = "wine",
    font      = Font.load("fonts/My Scary Font.otf"),
    italics   = true,
    underline = { width = 0.1, color = "rose" },
}
```

## TextStyle.new
```lua
TextStyle.new = function() -> textstyle
```
Creates a new instance of the default style.

## textstyle.background
```lua
textstyle.background: color
```
The background [color](Color.md) of the text. Setter also accepts `string` color names and `rgba` custom colors.

Defaults to [Color.transparent](Color.md#colortransparent).

#### with method
```lua
function textstyle:with_background(c: color or rgba or string) -> textstyle
```

## textstyle.color
```lua
textstyle.color: color
```
The [color](Color.md) of the text. Setter also accepts `string` color names and `rgba` custom colors.

Defaults to `Color "white"`.

#### with method
```lua
function textstyle:with_color(c: color or rgba or string) -> textstyle
```

## textstyle.font
```lua
textstyle.font: "monospace" or "proportional" or handle<font>
```
Defaults to `"proportional"`.

#### with method
```lua
function textstyle:with_font(f: "monospace" or "proportional" or handle<font>) -> textstyle
```

## textstyle.font_size
```lua
textstyle.font_size: number
```
Defaults to `14.0`.

#### with method
```lua
function textstyle:with_font_size(n: number) -> textstyle
```

## textstyle.italics
```lua
textstyle.italics: bool
```
Defaults to `false`.

#### with method
```lua
function textstyle:with_italics(is_italics: bool) -> textstyle
```

## textstyle.strikethrough
```lua
textstyle.strikethrough: { width = number, color = color or rgba or string }
```
Defaults to `{ width = 0.0, color = Color.transparent() }`. Unset/nil table values will use the default instead for that key.

#### with method
```lua
function textstyle:with_strikethrough(table: { width = number, color = color or rgba or string }) -> textstyle
```

## textstyle.underline
```lua
textstyle.underline: { width = number, color = color or rgba or string }
```
Defaults to `{ width = 0.0, color = Color.transparent() }`. Unset/nil table values will use the default instead for that key.

#### with method
```lua
function textstyle:with_underline(table: { width = number, color = color or rgba or string }) -> textstyle
```

## textstyle.valign
```lua
textstyle.valign: "top" or "center" or "bottom"
```
The vertical alignment of the text. Defaults to `"center"`.

#### with method
```lua
function textstyle:with_valign(alignment: "top" or "center" or "bottom") -> textstyle
```

## textstyle:clone
```lua
function textstyle:clone() -> textstyle
```