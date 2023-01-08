# 👏 handle\<t>

A Handle to an asset managed by Bevy.

## handle.kind
```lua
handle.kind: <const> string
```
The name of the kind of asset this handle refers to. The following asset types are supported in Lua:
- font
- formlist
- image
- level
- material
- palette
- uicontainer

## handle:get
```lua
function handle:get() -> t or nil
```

Returns the asset that this handle is referencing if its loaded, or `nil` otherwise. 

The following assets support this operation; all others will error.
- formlist
- level
- material
- palette

## handle:is_loaded
```lua
function handle:is_loaded() -> bool
```

True if this handle's asset has been loaded successfully.

## handle:on_load
```lua
function handle:on_load(f: function(handle))
```

Runs the function `f` when this handle's asset finishes loading by passing in this handle. If it already has loaded, then `f` is called instantly.

## handle:path
```lua
function handle:path() -> string or nil
```

Returns the file path of this asset, if it has one. Asset without paths are usually defined by Lua, including many materials, palettes, and ui containers.

## handle:weak
```lua
function handle:weak() -> handle<t>
```

Returns a weak handle to this asset. A weak handle won't keep an asset loaded if there are no strong handles, so be aware that the asset might unload.

```lua
local w = my_handle:weak()
local i = Image.load("example.png"):weak() -- this asset might immediately unload if no one else uses the same image!
```

## handle:__eq
```lua
function handle:__eq(that: handle<any>) -> bool
```

## handle:__tostring
```lua
function handle:__tostring() -> string
```