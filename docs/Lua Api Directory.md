# ๐ Lua Doc Directory

#### A note on `require`
All paths using [require](https://www.lua.org/pil/8.1.html) are relative to the root directory of a union of ./assets and all mods directories, with conflicting file names being overriden depending on load order.
```lua
-- my_mod/scripts/utils.lua
MyUtils.fix_everything = function() --[[...]] end

-- my_mod/main.lua
require "my_mod/scripts/utils" -- this is not relative to this script's path

MyUtils.fix_everything()
```

### ๐ [Globally defined values](lua_api/Globals.md)

## ๐ Modules ๐ Modules ๐ Modules ๐

### ๐ [Log](lua_api/Log.md)
Printing and logging functions.

### ๐งฎ [Math](lua_api/Math.md)
Functions for working with numbers and math.

### ๐ฒ [Random](lua_api/Random.md)
Random value generation functions.

### ๐ฑ [UI](lua_api/UI.md)
For creating UI/GUI elements.

### ๐ช [Var](lua_api/Var.md)
For sharing values between scripts and serialized formats.

## ๐ด Types ๐ด Types ๐ด Types ๐ด Types ๐ด

### โ๏ธ [atom](lua_api/types/Atom.md)
Wrappers around values for handling on change behavior.

### ๐ [color](lua_api/types/Color.md)
Dynamic colors that can change with the game's palette.

### โ๏ธ [entity](lua_api/types/Entity.md)
An instance of a thing in Bevy's ECS.

### โ๏ธ [font](lua_api/types/Font.md)
Font families that may be displayed by [textstyles](lua_api/types/TextStyle.md).

### ๐ [handle](lua_api/types/Handle.md)
References to assets managed by Bevy.

### ๐ผ๏ธ [image](lua_api/types/Image.md)
Image file loading.

### ๐๏ธ [level](lua_api/types/Level.md)
Pieces of map portions that can be applied to the game world.

### ๐งต [material](lua_api/types/Material.md)
Objects defining how images are presented over a mesh.

### ๐ [message](lua_api/types/Message.md)
Sending parameters to receiving functions on one or more other script instances.

### ๐จ [palette](lua_api/types/Palette.md)
Sets of colors that can be applied to the world.

### ๐ต๐ฝ [query](lua_api/types/Query.md)
Search and filter entities according to various predicates.

### ๐ [rgba](lua_api/types/Rgba.md)
sRGBA and linear RGBA concrete colors with mathematical operations.

### ๐ฌ [text](lua_api/types/Text.md)
Formatted text for displaying in UI elements.

### ๐๐พ [textstyle](lua_api/types/TextStyle.md)
Define how sections of text are styled.

### โฐ [time](lua_api/types/Time.md)
Timestamp instances and time-related functions.

### ๐ข [vec2](lua_api/types/Vec2.md)
x, y vectors.

### ๐ข [vec3](lua_api/types/Vec3.md)
x, y, z vectors.