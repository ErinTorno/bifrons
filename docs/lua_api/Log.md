# 📝 Log

This module contains logging functions.

## info
```lua
Log.info = function(...: any...)
```
Prints to the log at the info level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.info("some math: {} + {} = {}", 2, 4, 2 + 4)
Log.info({ x = 5, y = 3, z = 13})
```

## error
```lua
Log.error = function(...: any...)
```
Prints to the log at the error level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.error("Things went {}", false and "right" or "wrong")
```

## warn
```lua
Log.warn = function(...: any...)
```
Prints to the log at the warn level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.warn("Param is missing {}, defaulting to {}", "foo", 123)
```