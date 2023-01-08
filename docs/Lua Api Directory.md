# 🌙 Lua Doc Directory

#### A note on `require`
All paths using [require](https://www.lua.org/pil/8.1.html) are relative to the root directory of a union of ./assets and all mods directories, with conflicting file names being overriden depending on load order.
```lua
-- my_mod/scripts/utils.lua
MyUtils.fix_everything = function() --[[...]] end

-- my_mod/main.lua
require "my_mod/scripts/utils" -- this is not relative to this script's path

MyUtils.fix_everything()
```

### 🌏 [Globally defined values](lua_api/Globals.md)

## 📚 Modules 📚 Modules 📚 Modules 📚

### 📝 [Log](lua_api/Log.md)
Printing and logging functions.

### 🧮 [Math](lua_api/Math.md)
Functions for working with numbers and math.

### 🎲 [Random](lua_api/Random.md)
Random value generation functions.

### 📱 [UI](lua_api/UI.md)
For creating UI/GUI elements.

### 🪅 [Var](lua_api/Var.md)
For sharing values between scripts and serialized formats.

## 🎴 Types 🎴 Types 🎴 Types 🎴 Types 🎴

### ⚛️ [atom](lua_api/types/Atom.md)
Wrappers around values for handling on change behavior.

### 🌈 [color](lua_api/types/Color.md)
Dynamic colors that can change with the game's palette.

### ♟️ [entity](lua_api/types/Entity.md)
An instance of a thing in Bevy's ECS.

### ✒️ [font](lua_api/types/Font.md)
Font families that may be displayed by [textstyles](lua_api/types/TextStyle.md).

### 👏 [handle](lua_api/types/Handle.md)
References to assets managed by Bevy.

### 🖼️ [image](lua_api/types/Image.md)
Image file loading.

### 🏚️ [level](lua_api/types/Level.md)
Pieces of map portions that can be applied to the game world.

### 🧵 [material](lua_api/types/Material.md)
Objects defining how images are presented over a mesh.

### 💌 [message](lua_api/types/Message.md)
Sending parameters to receiving functions on one or more other script instances.

### 🎨 [palette](lua_api/types/Palette.md)
Sets of colors that can be applied to the world.

### 🕵🏽 [query](lua_api/types/Query.md)
Search and filter entities according to various predicates.

### 🌈 [rgba](lua_api/types/Rgba.md)
sRGBA and linear RGBA concrete colors with mathematical operations.

### 💬 [text](lua_api/types/Text.md)
Formatted text for displaying in UI elements.

### 💃🏾 [textstyle](lua_api/types/TextStyle.md)
Define how sections of text are styled.

### ⏰ [time](lua_api/types/Time.md)
Timestamp instances and time-related functions.

### 🔢 [vec2](lua_api/types/Vec2.md)
x, y vectors.

### 🔢 [vec3](lua_api/types/Vec3.md)
x, y, z vectors.