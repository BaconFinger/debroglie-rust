use crate::tile::TileId;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// A set of tiles, specific to a particular TilePropagator.
/// This struct internally caches some computations, making it faster
/// if you have lots of operations using the same set of tiles.
#[derive(Debug, Clone)]
pub struct TilePropagatorTileSet {
    tiles: Vec<TileId>,
    offset_to_patterns: HashMap<i32, HashSet<i32>>,
}

impl TilePropagatorTileSet {
    pub(crate) fn new(tiles: Vec<TileId>) -> Self {
        TilePropagatorTileSet {
            tiles,
            offset_to_patterns: HashMap::new(),
        }
    }

    pub fn tiles(&self) -> Vec<TileId> {
        self.tiles.clone()
    }

    pub(crate) fn offset_to_patterns(&mut self) -> &mut HashMap<i32, HashSet<i32>> {
        &mut self.offset_to_patterns
    }
}

impl fmt::Display for TilePropagatorTileSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tile_strings: Vec<String> = self
            .tiles
            .iter()
            .map(|tile| format!("{:?}", tile))
            .collect();
        write!(f, "{}", tile_strings.join(","))
    }
}
