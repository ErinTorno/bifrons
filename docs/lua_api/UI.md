# 📱 UI

This module contains functions for creating UI schema tables and compiling them into usable containers.

## compile
```lua
UI.compile = function(schemas: table...) -> handle<uicontainer>
```
Compiles each given schema into TopLevel UI elements, saves the resulting uicontainer as an asset, and returns a handle to it.

```lua
local handle = UI.compile(
    UI.verticalpanel {
        UI.label("Hello, world!"),
    }
)
UI.show(handle)
```

## hide
```lua
UI.hide = function(handle: handle<uicontainer>)
```
Hides the uicontainer of the `handle`, if it's visible.

## queue_app_exit
```lua
UI.queue_app_exit = function()
```
Game will close on next system update.

## show
```lua
UI.show = function(handle: handle<uicontainer>)
```
Shows the uicontainer of the `handle`, if it's not already.

## 📜 Schema creation utilities

The following functions are helpers for creating UI element schemas. Most of these functions take a single table to be used as the base schema, and are intended to be called by omitting the parentheses.

### Child elements

Child elements are included in schema in the array portion. In effect, all values with int or number keys are considered child elements in a schema.

```lua
UI.vertical {
    prop     = "some value",
    on_click = function() end
    -- where key = ... ends usually denotes start of child elements
    UI.label("I'm a child label"),
    UI.label("Me too!"),
    UI.button {
        text = Text.new("I'm a child button"),
    },
}
```

### [Atom](types/Atom.md) properties

Schema key-value properties with values of type `t` can instead take `atom<t>`, allowing those properties to be changed after UI compilation. Changing the atom's value can be costly in some cases, including for bigger types, like large tables or [Text](types/Text.md) values.

### Universal element properties

The following key-value pairs in schema define properties that all elements can use:

#### is_visible: bool
If false, element is hidden and not accessable. Defaults to true.
#### on_click: function() or nil
If not nil, when this element is clicked, this function is called.
#### size: vec2 or nil
Tries to configure this element to this size as appropriate.
#### tooltip: text or nil
If not nil, then this text will appear when the mouse hovers over this element.

## button
```lua
UI.button = function { text = text, ... }
```
Creates a button with the given text.

## horizontal
```lua
UI.horizontal = function { ... }
```
Aligns child elements horizontally.

## imagebutton
```lua
UI.imagebutton = function {
    image     = handle<image>,
    color     = color,
    is_framed = bool,
    ...
}
```
Creates an imagebutton.

## label
```lua
UI.label = function(s: text or string or table)
```
Creates a text label from the input.

If `s` is a `text` or `string`, then a simple label containing `s` is created.

If `s` is a `table`, then the table schema is marked as a label. Schema expects a `text = text` property defined.

## menu
```lua
UI.menu = function { ... }
```
Creates a menu bar (like up top with File, Edit, Help, etc.).

Element schemas expect either `menubar = "Menu Name"` as a property, or they inherit the menubar of the previous one. Will error if the first elem doesn't have any menubar defined.

```lua
UI.menu {
    UI.button {
        menubar = "File",
        text    = Text.new("Open"),
    },
    UI.button {
        text    = Text.new("Save"),
    },
    UI.button {
        menubar = "Help",
        text    = Text.new("About"),
    },
}
```

## vertical
```lua
UI.vertical = function { ... }
```
Aligns child elements vertical.