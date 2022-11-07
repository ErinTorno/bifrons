use bevy::{reflect::TypeUuid};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(default)]
    pub equip_slots: HashSet<String>,
}