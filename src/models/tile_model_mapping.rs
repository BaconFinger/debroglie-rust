use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use crate::context::{Context, TileId, TopologyId};
use crate::point::Point;
use crate::tile::Tile;
use crate::tile_propagator_tile_set::TilePropagatorTileSet;
use crate::topology::grid_topology::GridTopology;
use crate::topology::topo_array::TopoArray;
use crate::topology::topology::Topology;
use crate::wfc::pattern_model::PatternModel;

static EMPTY_PATTERN_SET: LazyLock<HashSet<usize>> = LazyLock::new(|| HashSet::new());

pub struct TileModelMapping<TopologyT: Topology> {
    pub pattern_topology: Option<TopologyId>,
    pub pattern_model: PatternModel,
    pub tiles_to_patterns_by_offset: HashMap<i32, HashMap<TileId, HashSet<usize>>>,
    pub patterns_to_tiles_by_offset: HashMap<i32, HashMap<usize, TileId>>,

    // None for 1:1 mappings
    pub tile_coord_to_pattern_coord_index_and_offset: Option<Box<dyn TopoArray<(Point<i32>, usize, i32), TopologyT>>>,

    // None for 1:1 mappings
    pub pattern_coord_to_tile_coord_index_and_offset: Option<Box<dyn TopoArray<Vec<(Point<i32>, usize, i32)>, TopologyT>>>,
}

impl<TopologyT> TileModelMapping<TopologyT> where TopologyT: Topology + Clone {
    pub fn new(
        pattern_topology: TopologyId,
        pattern_model: PatternModel,
        tiles_to_patterns_by_offset: HashMap<i32, HashMap<TileId, HashSet<usize>>>,
        patterns_to_tiles_by_offset: HashMap<i32, HashMap<usize, TileId>>,
    ) -> Self {
        TileModelMapping {
            pattern_topology: Some(pattern_topology),
            pattern_model,
            tiles_to_patterns_by_offset,
            patterns_to_tiles_by_offset,
            tile_coord_to_pattern_coord_index_and_offset: None,
            pattern_coord_to_tile_coord_index_and_offset: None,
        }
    }

    pub fn get_tile_coord_to_pattern_coord(&self, ctx: &Context<TopologyT>, x: i32, y: i32, z: i32) -> (i32, i32, i32, i32) {
        if let Some(ref mapping) = self.tile_coord_to_pattern_coord_index_and_offset {
            if let Ok((point, _index, offset)) = mapping.as_ref().get_coord(ctx, x as usize, y as usize, z as usize) {
                (point.x, point.y, point.z, *offset)
            } else {
                (x, y, z, 0) // fallback on error
            }
        } else {
            (x, y, z, 0)
        }
    }

    #[inline]
    pub fn get_tile_coord_to_pattern_coord_by_index(&self, ctx: &Context<TopologyT>, index: usize) -> (usize, i32) {
        if let Some(ref mapping) = self.tile_coord_to_pattern_coord_index_and_offset {
            if let Ok((_point, pattern_index, offset)) = mapping.get_index(ctx, index) {
                (pattern_index.clone(), offset.clone())
            } else {
                (index, 0) // fallback on error
            }
        } else {
            (index, 0)
        }
    }

    /// Creates a set of tiles. This set can be used with some operations, and is marginally
    /// faster than passing in a fresh list of tiles every time.
    pub fn create_tile_set(&self, tiles: &[Tile]) -> TilePropagatorTileSet {
        panic!("Don't do this anymore! Everything has to be through the Context");
        // let mut tile_set = TilePropagatorTileSet::new(tiles.iter().cloned());
        //
        // // Quick optimization for size one sets
        // if tile_set.tiles().len() == 1 {
        //     let tile = tile_set.tiles()[0].clone(); // Clone the tile to avoid borrowing issues
        //     for &offset in self.tiles_to_patterns_by_offset.keys() {
        //         let patterns = if let Some(patterns_map) = self.tiles_to_patterns_by_offset.get(&offset) {
        //             patterns_map.get(&tile).unwrap_or(&EMPTY_PATTERN_SET)
        //         } else {
        //             &EMPTY_PATTERN_SET
        //         };
        //         tile_set.offset_to_patterns().insert(offset, patterns.iter().map(|&x| x as i32).collect());
        //     }
        // }
        //
        // tile_set
    }

    fn get_patterns_for_tile<'a>(tiles_to_patterns: &'a HashMap<TileId, HashSet<usize>>, tile: TileId) -> &'a HashSet<usize> {
        tiles_to_patterns.get(&tile).unwrap_or(&EMPTY_PATTERN_SET)
    }

    /// Gets the patterns associated with a tile at a given offset.
    pub fn get_patterns(&self, tile: TileId, offset: i32) -> &HashSet<usize> {
        if let Some(tiles_to_patterns) = self.tiles_to_patterns_by_offset.get(&offset) {
            Self::get_patterns_for_tile(tiles_to_patterns, tile)
        } else {
            &EMPTY_PATTERN_SET
        }
    }

    /// Gets the patterns associated with a set of tiles at a given offset.
    pub fn get_patterns_from_tile_set(&self, tile_set: Arc<Mutex<TilePropagatorTileSet>>, offset: i32) -> HashSet<usize> {
        todo!("get_patterns_from_tile_set");
        // if let Some(patterns) = tile_set.lock().unwrap().offset_to_patterns().get(&offset) {
        //     return patterns.iter().map(|&x| x as usize).collect();
        // }
        //
        // let mut patterns = HashSet::new();
        //
        // if let Some(tiles_to_patterns) = self.tiles_to_patterns_by_offset.get(&offset) {
        //     for tile in tile_set.lock().unwrap().tiles() {
        //         let tile_patterns = Self::get_patterns_for_tile(tiles_to_patterns, tile);
        //         patterns.extend(tile_patterns);
        //     }
        // }
        //
        // tile_set.lock().unwrap().offset_to_patterns().insert(offset, patterns.iter().map(|&x| x as i32).collect());
        // patterns
    }

    /// Checks if the given tile is in the tiles_to_patterns mapping. If so, returns the TileId.
    /// In the C# package a Tile could be used as a Key in a HashMap, and it would compare by the
    /// value of the tile (e.g., 2 different tiles with the contents of '_' would be treated as the same
    /// Key). So this function replicates this behavior by comparing the value of the tiles.
    fn in_tiles_to_patterns(&self, ctx: &Context<GridTopology>, tile: TileId, hash: &HashMap<TileId, HashSet<usize>>) -> Option<TileId> {
        let new_tile = ctx.tiles().get(tile)?.clone();
        let matching_tile = {
            let mut result = None;
            for (t_key, _pattern) in hash {
                if t_key.clone() == tile {
                    result = Some(t_key.clone());
                    break;
                }
                let existing_tile = ctx.tiles().get(t_key.clone())?.clone();

                if existing_tile.borrow().get_value() == new_tile.borrow().get_value() {
                    result = Some(t_key.clone());
                    break;
                }
            }
            result
        };
        matching_tile
    }
}

impl<T: Topology> Default for TileModelMapping<T> {
    fn default() -> Self {
        Self {
            pattern_topology: None,
            pattern_model: PatternModel::new(Vec::new(), Vec::new()),
            tiles_to_patterns_by_offset: Default::default(),
            patterns_to_tiles_by_offset: Default::default(),
            tile_coord_to_pattern_coord_index_and_offset: None,
            pattern_coord_to_tile_coord_index_and_offset: None,
        }
    }
}