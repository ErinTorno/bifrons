# 💌 font

Defines the font families [TextStyle](TextStyle.md)s can use.

## Font.load
```lua
Font.load = function(path: string) -> handle<font>
```
Loads a font if it isn't already, and returns a [handle](types/Handle.md) to it.

```lua
local font_handle = Font.load("fonts/UbuntuMono-R.ttf")
```