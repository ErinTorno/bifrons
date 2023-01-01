# ⚛️ atom\<t>

A wrapper around mutable values with on change detection capabilities. Used frequently with [UI](../UI.md) schema properties to rerender UI elements with new properties.

## Atom.create
```lua
Atom.create = function(a: t) -> atom<t>
```
Creates an atom with the given starting value.

## atom.index
```lua
atom.index: <const> int
```
The internal index used by the atom.

## atom:get
```lua
function atom:get() -> t
```
Returns the value stored within this atom.

## atom:map
```lua
function atom:map(f: function(t) -> any)
```
Retrieves the atom's value, applies it to `f`, and stores `f`'s return as the new atom value.

## atom:set
```lua
function atom:set(a: any) -> t
```
Sets the atom's value, returning the previous one.

## atom:__call
```lua
function atom:__call() -> t
```
Alias for [atom:get](#atom:get).
```lua
local i = my_int_atom()
```

## atom:__eq
```lua
function atom:__eq(that: atom) -> bool
```
Compares atom references, not inner values.

## atom:__tostring
```lua
function atom:__tostring() -> string
```
In the form "AtomRef#{index}".