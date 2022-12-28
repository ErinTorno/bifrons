# 🔢 vec2

A vector of `x` and `y` numbers. For 3-dim vectors, see [vec3](Vec3.md).

## Vec2.new
```lua
Vec2.new = function(x: number, y: number) -> vec2
```

## Vec2.one
```lua
Vec2.one = function() -> vec2
```
Equivalent to `Vec2.new(1, 1)`.

## Vec2.splat
```lua
Vec2.splat = function(n: number) -> vec2
```
Equivalent to `Vec2.new(n, n)`.

## Vec2.zero
```lua
Vec2.zero = function() -> vec2
```
Equivalent to `Vec2.new(0, 0)`.

## vec2.x
```lua
vec2.x: number
```

## vec2.y
```lua
vec2.y: number
```

## vec2:clone
```lua
function vec2:clone() -> vec2
```

## vec2:extend
```lua
function vec2:extend(z: number) -> vec3
```
Adds a `z` dimension to the vec, becoming a [vec3](Vec3.md).

## vec2:__add
```lua
function vec2:__add(v: vec2 or vec3) -> vec2 or vec3
```

## vec2:__div
```lua
function vec2:__div(v: number or vec2) -> vec2
```

## vec2:__len
```lua
function vec2:__len() -> 2
```

## vec2:__mul
```lua
function vec2:__mul(v: number or vec2) -> vec2
```

## vec2:__pairs
```lua
function vec2:__pairs() -> iterator, vec2, string
```
Allows use of `pairs(vec2)`.
```lua
for k, v in pairs(Vec2.new(0.3, 2.4)) do
    Log.info("{} = {}", k, v)
end
-- Prints:
-- x = 0.3
-- y = 2.4
```

## vec2:__sub
```lua
function vec2:__sub(v: vec2 or vec3) -> vec2 or vec3
```

## vec2:__tostring
```lua
function vec2:__tostring() -> string
```