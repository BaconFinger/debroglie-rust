use std::hash::{Hash, Hasher};
use std::num::Wrapping;
use crate::rot::rotations::Rotation;
use crate::tile::Tile;

/// Represents a tile that has been rotated and reflected in some way.
#[derive(Clone, Debug, Eq)]
pub struct RotatedTile {
    tile: Tile,
    rotation: Rotation,
}

impl RotatedTile {
    pub fn new(tile: Tile, rotation: Rotation) -> Self {
        Self { tile, rotation }
    }
}

impl Hash for RotatedTile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rotation.hash(state);
        self.tile.hash(state);
    }
}

impl PartialEq for RotatedTile {
    fn eq(&self, other: &Self) -> bool {
        self.rotation == other.rotation && self.tile == other.tile
    }
}