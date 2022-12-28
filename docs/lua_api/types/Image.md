# 🖼️ image

Images are the base visual asset for textures, icons, and UI images.

## Image.load
```lua
Image.load = function(path: string) -> handle<image>
```
Loads an image if it isn't already, and returns a [handle](types/Handle.md) to it.

```lua
local icon_handle = Image.load("images/example.png")
```