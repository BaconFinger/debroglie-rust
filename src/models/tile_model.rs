use std::error::Error;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::tile::Tile;
use crate::topology::topology::Topology;

/// Base trait for the models used in generation.
///
/// A TileModel is a model with a well defined mapping from
/// "tiles" (arbitrary identifiers of distinct tiles)
/// with patterns (dense integers that correspond to particular
/// arrangements of tiles).
pub trait TileModel<TopologyT: Topology + Clone> {
    type Error: Error + Send + Sync + 'static;

    /// Extracts the actual model of patterns used.
    fn get_tile_model_mapping(&mut self, topology: &TopologyT) -> Result<TileModelMapping<TopologyT>, Self::Error>;
    fn get_tile(&self, index: usize) -> Option<&Tile>;

    // fn get_tile_model_mapping(&mut self, ctx: &Context<TopologyT>, topology: TopologyId) -> Result<TileModelMapping<TopologyT>, String>;
    // TODO: Implement
    // /// Extracts the actual model of patterns used.
    // fn get_tile_model_mapping(&mut self, topology: Box<dyn Topology>) -> Result<TileModelMapping<TopologyT>, String>;
    //
    // /// Gets all tiles in this model
    // fn tiles(&self) -> Vec<Tile>;
    //
    // /// Scales the occurrence frequency of a given tile by the given multiplier.
    // fn multiply_frequency(&mut self, tile: &Tile, multiplier: f64);
    //
    // /// Scales the occurrence frequency of a given tile by the given multiplier,
    // /// including other rotations of the tile.
    // fn multiply_frequency_with_rotation(&mut self, tile: &Tile, multiplier: f64, tile_rotation: &dyn TileRotation) {
    //     let mut rotated_tiles = HashSet::new();
    //
    //     for rotation in tile_rotation.rotation_group() {
    //         if let Some(result) = tile_rotation.rotate(tile, rotation) {
    //             if rotated_tiles.insert(result.clone()) {
    //                 self.multiply_frequency(&result, multiplier);
    //             }
    //         }
    //     }
    // }
}

// You'll need to define these traits/structs based on your rotation system
pub trait TileRotation {
    // TODO: Implement
    // fn rotation_group(&self) -> &[Rotation];
    // fn rotate(&self, tile: &Tile, rotation: &Rotation) -> Option<Tile>;
}