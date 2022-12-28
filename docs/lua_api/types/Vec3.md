# 🔢 vec3

A vector of `x`, `y`, and `z` numbers. For 2-dim vectors, see [vec2](Vec2.md).

## Vec3.new
```lua
Vec3.new = function(x: number, y: number) -> vec3
```

## Vec3.one
```lua
Vec3.one = function() -> vec3
```
Equivalent to `Vec3.new(1, 1, 1)`.

## Vec3.splat
```lua
Vec3.splat = function(n: number) -> vec3
```
Equivalent to `Vec3.new(n, n, n)`.

## Vec3.zero
```lua
Vec3.zero = function() -> vec3
```
Equivalent to `Vec3.new(0, 0, 0)`.


## vec3.x
```lua
vec3.x: number
```

## vec3.y
```lua
vec3.y: number
```

## vec3.z
```lua
vec3.z: number
```

## vec3:clone
```lua
function vec3:clone() -> vec3
```

## vec3:__add
```lua
function vec3:__add(v: vec3 or vec2) -> vec3
```

## vec3:__div
```lua
function vec3:__div(v: number or vec3) -> vec3
```

## vec3:__len
```lua
function vec3:__len() -> 3
```

## vec3:__mul
```lua
function vec3:__mul(v: number or vec3) -> vec3
```

## vec3:__pairs
```lua
function vec3:__pairs() -> iterator, vec3, string
```
Allows use of `pairs(vec3)`.
```lua
for k, v in pairs(Vec3.new(0.3, 2.4, 1)) do
    Log.info("{} = {}", k, v)
end
-- Prints:
-- x = 0.3
-- y = 2.4
-- z = 1
```


## vec3:__sub
```lua
function vec3:__sub(v: vec3 or vec2) -> vec3
```

## vec3:__eq
```lua
function vec3:__eq(that: vec3) -> bool
```

## vec3:__tostring
```lua
function vec3:__tostring() -> string
```