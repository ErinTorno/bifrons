# ♟️ entity

A specific instance of a thing stored in the ECS. See [Bevy's documentation on Entities](https://docs.rs/bevy/latest/bevy/ecs/entity/index.html) for more information.

## Entity.spawn
```lua
Entity.spawn = function() -> entity
```
Spawns a new empty entity.

## entity:add_child
```lua
function entity:add_child(child: entity)
```
Adds the `child` to this entity's children.

```lua
for _, child in pairs(children) do
    my_parent_entity:add_child(child)
end
```

## entity:despawn
```lua
function entity:despawn()
```
Despawns an entity, removing it (and all its children, if any) from the world.

## entity:hide
```lua
function entity:hide()
```
Makes an entity no longer visible.

## entity:show
```lua
function entity:show()
```
Makes an entity visible, if it has any graphical elements.