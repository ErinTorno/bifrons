# ⏰ time

Marks a certain time instant over the application's run length.

## Time.elapsed
```lua
Time.elapsed = function() -> number
```
Returns the second elapsed since the start of the game application.
```lua
if Time.elapsed() > 60.0 then
    Log.info("It is been longer than 1 minute since launch.")
end
```

## time.delta
```lua
time.delta: <const> number
```
The number of seconds since from the previous on_update to the one that created this `time` instance.

## time.elapsed
```lua
time.elapsed: <const> number
```
The number of seconds since startup and when this was created.

## time:__eq
```lua
function time:__eq(that: time) -> bool
```

## time:__tostring
```lua
function time:__tostring() -> string
```