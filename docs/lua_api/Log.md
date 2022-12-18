# Log

This module contains logging functions.

## info
```lua
function info(...: any...)
```
Prints to the log at the info level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.info("some math: {} + {} = {}", 2, 4, 2 + 4)
```

## error
```lua
function error(...: any...)
```
Prints to the log at the error level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.error("Things went {}", false and "right" or "wrong")
```

## warn
```lua
function warn(...: any...)
```
Prints to the log at the warn level by using [`format`](Globals.md#format) to stringify the params.

```lua
Log.warn("Param is missing {}, defaulting to {}", "foo", 123)
```