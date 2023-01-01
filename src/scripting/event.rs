/// Called once when this script is being removed from an Entity or its Entity is removed
/// *params:* ()
/// todo! implement
pub const _ON_DROP:        &str = "on_drop";

/// Called after the script fully loads and is processed
/// *params:* (time: Time)
/// Library mods and Entity information might not be available when ran at the root/global level,
/// which makes this useful to make use of that during script startup
pub const ON_INIT:        &str = "on_init";

/// Called when an entity interacts with this script's entity
/// *params:* (context: {prompts: Entity})
/// todo! implement
pub const _ON_INTERACT:    &str = "on_interact";

/// Called when the game's localization language changes
/// *params:* (context: {prev: String, new: String})
/// todo! implement
pub const _ON_LANG_CHANGE: &str = "on_lang_change";

/// Called when a save state is being made while this script is active
/// *params:* (reader: SaveReader)
/// todo! implement
pub const _ON_LOAD:        &str = "on_load";

/// Called when a room is revealed on the map (including at setup for (reveal_before_entry: true))
/// *params:* (context: {
///     name:   String, -- the room's name
///     entity: Entity, -- the room's entity
/// })
pub const ON_ROOM_REVEAL: &str = "on_room_reveal";

/// Called when a save state is being made while this script is active
/// *params:* (writer: SaveWriter)
/// todo! implement
pub const _ON_SAVE:        &str = "on_save";

/// Called when this script's entity takes damage from a source
/// *params:* (context: {
///     attacker:    Entity, -- the entity that attacked me
///     damage:      Number, -- the damage I'm taking
///     base_damage: Number, -- the unmodified damage dealt by the attacker
///     tags:        Table<String, Boolean> -- the attack's damage tags 
/// })
/// todo! implement, and a way to cancel the damage?
pub const _ON_TAKE_DAMAGE: &str = "on_take_damage";

/// Called repeatedly at a fixed timestamp
/// *params:* ()
pub const ON_UPDATE:      &str = "on_update";

pub mod constants {
    /// How many seconds it takes until the next on_update call
    /// 
    /// With 1/15, on_update is called 15 times a second
    pub const ON_UPDATE_DELAY: f32 = 1. / 15.;
}