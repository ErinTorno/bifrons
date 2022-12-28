# 🧮 Math

This module contains numbers and math functions.

## clamp
```lua
Math.clamp = function(num: number, min: number, max: number) -> number
```
Returns `min` if `num < min`, `max` if `num > max`, or else `num`.

## finite_or
```lua
Math.finite_or = function(num: number, or_else: number) -> number
```
Returns the given number `num`, or a default number `or_else` if `num` is infinite or NaN.

## pi
```lua
Math.pi = 3.14159265358979323846264338327950288
```
The π constant at a f64 (double) precision.