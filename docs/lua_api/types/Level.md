# 🏚️ level

Levels are collections of prefabs, geometry, lighting, and scripts. Multiple levels can be spawned at once around the game world, and by each other.

## Level.load
```lua
Font.load = function(path: string) -> handle<level>
```
Loads a level into memory if it isn't already, and returns a [handle](types/Handle.md) to it.

```lua
local handle_a = Level.load("levels/my_levels/haunted_house.prefab.ron")
local handle_b = Level.load("levels/my_levels/haunted_house") -- extension is optional
assert(handle_a == handle_b) 
```

## level:is_defined_in_file
```lua
function level:is_defined_in_file() -> bool
```
Returns true if this level was backed (and likely loaded) by a prefab.ron file. All levels loaded via [Level.load](#levelload) are expected to return true, and those by [Level.new](#levelnew) are unlikely too, unless something else saves them after creation.

## level:spawn
```lua
function level:spawn(config: table or nil) -> entity
```
Places an instance of this level in the world, returning the [entity](Entity.md) created.

If `config` isn't null, it supports the following properties:
```lua
level:spawn {
    -- A debug name used to refer to this symbol.
    -- If not given, one will be generated based on file path or hash.
    debug_name  = string or nil,
    -- Is this visible to the players yet?
    is_revealed = bool or true,
    -- EulerRot::XYZ radian rotation for the spawned level
    rotation    = vec3 or Vec3.zero(),
    -- An entity that will be the parent of this level, if any
    parent      = entity or nil,
    -- The offset in the world this is spawned
    position    = vec3 or Vec3.zero(),
}
```
See [EulerRot::XYZ in the Bevy api](https://docs.rs/bevy/latest/bevy/math/enum.EulerRot.html) for [vec3](Vec3.md) rotation logic. All units are in radians.

### 🧵 [materials](Material.md)
These store [material](Material.md)s used by this level's geometry.

## level:get_material
```lua
function level:get_material(name: string) -> material or nil
```
Gets the [material](Material.md) with that `name`, if one exists.

## level:remove_material
```lua
function level:remove_material(name: string) -> material or nil
```
Removes the [material](Material.md) with that `name`, and returns the previous one, if any.

## level:set_material
```lua
function level:set_material(name: string, mat: material) -> material or nil
```
Sets the [material](Material.md) with that `name`, and returns the previous one, if any.

### 🎭 [scripts](Script.md)
These [script](Script.md)s are Lua files that are each loaded as Lua instances upon [spawn](#levelspawn)ing the level.

## level:add_script
```lua
function level:add_script(script: handle<script>)
```
Adds a [handle](Handle.md) to that [`script`](Script.md) to the level's scripts.

## level:remove_script
```lua
function level:remove_script(script: handle<script>) -> int
```
Removes all instances of that [`script`](Script.md) from this level, and returns the number of times it got removed.

### 🪅 [vars](../Var.md)
The [var](../Var.md)s that a level contains are available to its scripts via [Var.all()](../Var.md#varall).

## level:get_var
```lua
function level:get_var(key: string) -> any or nil
```
Gets the script var associated with that string key, if one exists.

## level:set_var
```lua
function level:set_var(key: string, var: any)
```
Sets the script var associated with that string key. See [the Var module](../Var.md) for what lua types are usable here, and which ones support serialization.