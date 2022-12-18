# Globals

The following values are defined as a global, and are accessed without any prefix.

## finite_or
```lua
function finite_or(num: number, or_else: number) -> number
```

Returns the given number `num`, or a default number `or_else` if `num` is infinite or NaN.

## format
```lua
function format(...: any...) -> string
```

Stringifies the first param using [`string`](#string), and replaces the `{}` in that string with each subsequent param, returning the resulting string.
```lua
format("Hello, {} docs!", "Bifrons")
format(123)
format("Number {}, bool {}, and table {}", 123, true, {x = 5, y = 10})
```

## PI
```lua
PI = 3.14159265358979323846264338327950288
```
The π constant at a f64 (double) precision.

## string
```lua
function string(value: any) -> string
```
Stringifies the given value. For tables and userdata types, `__tostring` is called if it is defined.

```lua
string(123) == "123"
string(3.13) == "3.14"
string("hello") == "hello"
string({x = 5, y = 10}) == "{x = 5, y = 10}"
string(Rgba.white) == "#ffffff"
```
