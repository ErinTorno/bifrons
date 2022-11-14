use std::collections::{HashMap, HashSet};

use bevy::prelude::{Entity, Vec3};
use serde::{Deserialize, Serialize};

// Config

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum GridShape {
    BitGrid(Vec<Vec<u8>>),
    Rect {
        rows: usize,
        columns: usize,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GridConfig {
    pub shape: GridShape,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default)]
    pub actor_rotation: Vec3,
}

// Actual implementation

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub rotation: Vec3,
    pub actor_rotation: Vec3,
    pub cells: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CellID {
    Local  { x: usize, y: usize },
    Global { x: usize, y: usize, room: String, grid: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    pub id:       CellID,
    pub distance: f32,
    pub angle:    f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub connections: Vec<Connection>, // expected to be ordered by distance asc
    pub entities:    HashSet<Entity>,
}

pub struct EntityGrid(HashMap<String, HashMap<String, Grid>>);

impl EntityGrid {
    pub fn get<SR, SG>(&self, room_name: SR, grid_name: SG, x: usize, y: usize) -> Option<&Cell> where SR: AsRef<str>, SG: AsRef<str> {
        if let Some(room_map) = self.0.get(room_name.as_ref()) {
            if let Some(grid) = room_map.get(grid_name.as_ref()) {
                return grid.cells.get(y).and_then(|v| v.get(x));
            }
        }
        None
    }

    pub fn get_mut<SR, SG>(&mut self, room_name: SR, grid_name: SG, x: usize, y: usize) -> Option<&mut Cell> where SR: AsRef<str>, SG: AsRef<str> {
        if let Some(room_map) = self.0.get_mut(room_name.as_ref()) {
            if let Some(grid) = room_map.get_mut(grid_name.as_ref()) {
                return grid.cells.get_mut(y).and_then(|v| v.get_mut(x))
            }
        }
        None
    }
}