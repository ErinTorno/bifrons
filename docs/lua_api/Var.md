# 🪅 Var

For sharing values between scripts and serialized formats.

## Var.all
```lua
Var.all = function() -> table
```
Gets a map of all script vars assigned for the entity holding this script, including from level or prefab definitions, and those attached by other lua scripts.

### 👽 Valid types
Not all types can be serialized and transferred between scripts. The following are supported by [ron files](https://github.com/ron-rs/ron) and as variables in lua scripts.

### bool
##### Lua
```lua
this_bool = true,
```
##### RON
```rust
that_bool: Bool(false),
```

### [color](types/Color.md)
Accepts a color string. See [Color.__call](types/Color.md#color__call) for string parsing behavior.
##### Lua
```lua
highlight = Color "#d5cb57",
```
##### RON
```rust
background: Color("#de4e76"),
```

### int
##### Lua
```lua
this_int = 123,
```
##### RON
```rust
that_int: Int(456),
```

### nil
##### Lua
```lua
unit = nil,
```
##### RON
```rust
unit: Nil,
```

### number
##### Lua
```lua
this_num = 987.654,
```
##### RON
```rust
that_num: Num(123.456),
```

### [rgba](types/Rgba.md)
Accepts a color string. See [Rgba.__call](types/Rgba.md#rgba__call) for string parsing behavior.
##### Lua
```lua
mat_color = Rgba "#50405a",
```
##### RON
```rust
on_lit_rgb: Rgba("#e09b4d"),
```

### string
##### Lua
```lua
name = "The Foyer",
```
##### RON
```rust
desc: Str("A real cool place"),
```

### table
Keys and values must be valid var types too.

Note that lua format maps to various RON formats.
- `Array`s take a list of vars.
- `Table`s map string keys to vars.
- `AnyUserTable`s map any type of key to any type of object.
##### Lua
```lua
my_table = {
    normal = "serializable",
    lua    = "values",
    123    = false,
},
```
##### RON
```rust
"array": Array([Int(456), Num(3.1415)]),
"string_keys": Table({
    "vals": Num(123.),
    "are":  Bool(true),
    "just": Str("strings"),
}),
"any_keys_req": AnyUserTable([
    (Str("tuple"), Vec3(0, 1, 0)),
    (Num(3.),      Str("pairs instead")),
]),
```

### [vec2](types/Vec2.md)
##### Lua
```lua
pos = Vec2.new(10.5, 20),
```
##### RON
```rust
offset: Vec2(0, 29.34),
```

### [vec3](types/Vec3.md)
##### Lua
```lua
pos = Vec3.new(12, 34.5, 67.89),
```
##### RON
```rust
rot: Vec3(0., 1.0471, 0.),
```

## 🏳️‍⚧️ TransVars

The following types of vars are able to be transferred between lua scripts, but will fail to deserialize or serialize. They usually encompass types which are not valid between runs.

### [entity](types/Entity.md)

### [handle](types/Handle.md)

### table
[As above](#table), except keys and values may also be TransVars.

### [text](types/Text.md)
```lua
label = Text.new():push("Bottom text")
```

### [time](types/Time.md)