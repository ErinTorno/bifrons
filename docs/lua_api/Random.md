# 🎲 Random

This module provides functions for getting random values.

Values are generated using the [ChaCha algorithm with 8 rounds](https://rust-random.github.io/rand/rand_chacha/struct.ChaCha8Rng.html). The seed is reset on level change, and with the same seed and Random calls the same results will always be generated.

## bool
```lua
Random.bool = function() -> bool
```
Returns `true` or `false`.

```lua
if Random.bool() then
    -- ...
end
```

## int
```lua
Random.int = function(min: int, max: int) -> int
```
Returns a whole integer between `min` and `max` (inclusive)

```lua
local a = Random.int(0, 20)
local b = Random.int(100, 999)
```

## key
```lua
Random.key = function(table: table) -> any
```
Returns a random key from the table, or `nil` if it's empty.
```lua
local a = Random.key({a = 1, b = 2, c = 3})
local b = Random.key({})
```

## kv
```lua
Random.kv = function(table: table) -> any, any
```
Returns a random key-value pair from the table, or `nil` if it's empty.
```lua
local key, val = Random.kv({a = 1, b = 2, c = 3})
```

## number
```lua
Random.number = function(min: number, max: number) -> number
```
Returns a number between `min` and `max` (inclusive). If `min` is `nil`, then 0.0 to 1.0 (exclusive) is used instead.

```lua
local a = Random.number()
local b = Random.number(0, 123)
```

## value
```lua
Random.value = function(table: table) -> any
```
Returns a random value from the table, or `nil` if it's empty.
```lua
local a = Random.kv({a = 1, b = 2, c = 3})
local b = Random.kv({})
```