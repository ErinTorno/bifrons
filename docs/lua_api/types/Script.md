# 🎭 script

Scripting functions related to scripts (in a meta way).

## Script.load
```lua
Script.load = function(path: string) -> handle<script>
```
Loads a .lua file as an asset if it isn't already, and returns a [handle](types/Handle.md) to it.

This will not execute any lua code until something making use of it is spawned (like a level or prefab). To include a script in the current lua file, use `require`.