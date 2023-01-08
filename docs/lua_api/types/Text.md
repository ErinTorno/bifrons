# 💬 text

Sections of formatted text for use in ui displays.

Most text methods consume the textstyle and return a new text the the requested changes. These are meant to be chained
Each field also has a with_ method which consumes the textstyle and returns an updated one. These are meant to be chained, like so.

```lua
local my_text = Text.new()
    :style(TextStyle.from { color = "wine", italics = true })
    :push("The situation seems to be ever worsening...")
    :ident(32.0)
    :push("I doubt James will make it back soon either.\n")
    :push("Shutting up shop alone another night is gonna be hell.")
```

## Text.new
```lua
Text.new = function(init: string or textstyle or nil) -> text
```
Creates a new text. If a string is given as a parameter, then a section will automatically be created with the default formatting and its content being `init`. If a [textstyle](TextStyle.md) is given as a parameter, then the text is empty, but its initial style is equal to the `init`.
```lua
local empty   = Text.new()
local hw      = Text.new("Hello, world!")
local empty_y = Text.new(TextStyle.from { color = "yellow" })
```

## text:indent
```lua
function text:indent(indentation: number) -> text
```
Applies the given number of units in indentation before the next line.