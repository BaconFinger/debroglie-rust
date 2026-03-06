use crate::models::tile_model::TileModel;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::tile::{Tile, TileId};
use crate::topology::direction::{Direction, DirectionSet, DirectionSetType};
use crate::topology::grid_topology::{GridTopology, GridTopologyError};
use crate::topology::topo_array::{DefaultTopoArray, TopoArray};
use crate::topology::topology::{Topology, TopologyError};
use crate::trait_error::TraitError;
use crate::wfc::pattern_model::PatternModel;
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdjacentModelError {
    #[error("Directions must be set before calling this method")]
    DirectionsNotSet,

    #[error("Failed to get topology")]
    NoTopology,

    #[error("Failed to get tile at ({x}, {y}, {z})")]
    CannotGetAtCoord { x: usize, y: usize, z: usize },

    #[error("Failed to get direction from index {index}")]
    CannotGetDirectionFromIndex { index: usize },

    #[error("Failed to get adjacent tile at ({x}, {y}, {z})")]
    CannotGetAdjacentTile { x: usize, y: usize, z: usize },

    #[error("Sample is incompatible because {reason}")]
    IncompatibleSample { reason: String },

    #[error("No tiles have assigned frequencies")]
    NoFrequencies,

    #[error("Cannot set directions to {target} because it has already been set to {current}")]
    CannotSetDirections { target: String, current: String },

    // Error wrappers
    #[error(transparent)]
    Topology(#[from] TopologyError),
    #[error(transparent)]
    GridTopology(#[from] GridTopologyError),
}

/// AdjacentModel constrains which tiles can be placed adjacent to which other ones.
/// It does so by maintaining for each tile, a list of tiles that can be placed next to it in each direction.
/// The list is always symmetric, i.e. if it is legal to place tile B directly above tile A, then it is legal to place A directly below B.
pub struct AdjacentModel {
    directions: Option<DirectionSet>,

    /// Maps each unique tile to its pattern.
    /// Note that unique here is referring to the tile's value, not its key/id/reference.
    /// The pattern is an index.
    tiles_to_patterns: HashMap<TileId, usize>,

    /// The frequency of each pattern from the provided sample (Tiles).
    /// Each index is a pattern and the stored value is the number of occurrences of that pattern.
    frequencies: Vec<f64>,

    /// Don't quite know what this does yet
    propagator: Vec<Vec<HashSet<usize>>>,

    sample: Box<dyn TopoArray<Tile, GridTopology>>,
}

impl AdjacentModel {
    /// Constructs an AdjacentModel.
    pub fn new() -> Self {
        Self {
            directions: None,
            tiles_to_patterns: HashMap::new(),
            frequencies: Vec::new(),
            propagator: Vec::new(),
            sample: Box::new(DefaultTopoArray::new()),
        }
    }

    /// Sets the directions of the Adjacent model, if it has not been set at construction.
    /// This specifies how many neighbours each tile has.
    /// Once set, it cannot be changed.
    pub fn set_directions(&mut self, directions: DirectionSet) -> Result<(), AdjacentModelError> {
        if let Some(ref current_directions) = self.directions {
            if current_directions.direction_type() != DirectionSetType::Unknown
                && current_directions.direction_type() != directions.direction_type()
            {
                return Err(AdjacentModelError::CannotSetDirections {
                    target: format!("{:?}", directions.direction_type()),
                    current: format!("{:?}", current_directions.direction_type()),
                });
            }
        }

        self.directions = Some(directions);
        Ok(())
    }

    fn require_directions(&self) -> Result<&DirectionSet, AdjacentModelError> {
        match &self.directions {
            Some(directions) => {
                if directions.direction_type() == DirectionSetType::Unknown {
                    Err(AdjacentModelError::DirectionsNotSet)
                } else {
                    Ok(directions)
                }
            }
            None => Err(AdjacentModelError::DirectionsNotSet),
        }
    }

    /// Adds the sample to the model. This will create a copy of the sample that is owned by the model.
    pub fn add_sample_simple<T: Topology>(
        &mut self,
        sample: &dyn TopoArray<Tile, GridTopology>,
    ) -> Result<(), AdjacentModelError> {
        let (width, height, depth, direction_count) = {
            let topology = sample.topology().ok_or(AdjacentModelError::NoTopology)?;
            self.set_directions(topology.directions().clone())?;

            let width = topology.width();
            let height = topology.height();
            let depth = topology.depth();
            let direction_count = topology.directions().count();
            (width, height, depth, direction_count)
        };

        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    let index = sample
                        .topology()
                        .ok_or(AdjacentModelError::NoTopology)?
                        .get_index(x, y, z)?;

                    if !sample
                        .topology()
                        .ok_or(AdjacentModelError::NoTopology)?
                        .contains_index(index)
                    {
                        continue;
                    }

                    // Need the tile to get the pattern.
                    let tile = sample.get_id_from_coord(x, y, z);
                    if tile.is_none() {
                        return Err(AdjacentModelError::CannotGetAtCoord { x, y, z });
                    }
                    let tile = TileId(tile.unwrap());

                    // Find the pattern and update the frequency
                    let pattern = self.get_pattern(tile, sample);
                    if pattern < self.frequencies.len() {
                        self.frequencies[pattern] += 1.0;
                    }

                    // Update propagator - collect adjacent tiles first
                    // Much more verbose than the C# to avoid borrowing conflicts
                    let mut adjacent_tiles = Vec::new();
                    for d in 0..direction_count {
                        let direction = Direction::from_index(d)
                            .ok_or(AdjacentModelError::CannotGetDirectionFromIndex { index: d })?;
                        let result = sample
                            .topology()
                            .ok_or(AdjacentModelError::NoTopology)?
                            .try_move_coord_to_coord(x, y, z, direction)?;
                        if let Some((x2, y2, z2)) = result {
                            let tile2 = sample.get_id_from_coord(x2, y2, z2).ok_or(
                                AdjacentModelError::CannotGetAdjacentTile {
                                    x: x2,
                                    y: y2,
                                    z: z2,
                                },
                            )?;
                            adjacent_tiles.push((d, TileId(tile2)));
                        }
                    }

                    // Now process adjacent tiles
                    for (d, tile2) in adjacent_tiles {
                        let pattern2 = self.get_pattern(tile2, sample);

                        // Ensure propagator is large enough
                        while self.propagator.len() <= pattern.max(pattern2) {
                            self.propagator.push(vec![HashSet::new(); direction_count]);
                        }

                        self.propagator[pattern][d].insert(pattern2);
                    }
                }
            }
        }

        self.sample = sample
            .clone_box()
            .ok_or(AdjacentModelError::IncompatibleSample {
                reason: "Cannot be cloned to Box".to_string(),
            })?;

        Ok(())
    }

    fn get_pattern(&mut self, tile: TileId, sample: &dyn TopoArray<Tile, GridTopology>) -> usize {
        let direction_count = self.directions.as_ref().map(|d| d.count()).unwrap_or(4); // Default fallback

        let matching_tile = self.in_tiles_to_patterns(tile, sample);
        if let Some(matching_tile) = matching_tile {
            if let Some(&pattern) = self.tiles_to_patterns.get(&matching_tile) {
                return pattern;
            }
        }

        let pattern = self.tiles_to_patterns.len();
        self.tiles_to_patterns.insert(tile, pattern);
        self.frequencies.push(0.0);
        self.propagator.push(vec![HashSet::new(); direction_count]);

        pattern
    }

    /// Checks if the given tile is in the tiles_to_patterns mapping. If so, returns the TileId.
    /// In the C# package a Tile could be used as a Key in a HashMap, and it would compare by the
    /// value of the tile (e.g., 2 different tiles with the contents of '_' would be treated as the same
    /// Key). So this function replicates this behavior by comparing the value of the tiles.
    fn in_tiles_to_patterns(
        &self,
        tile: TileId,
        sample: &dyn TopoArray<Tile, GridTopology>,
    ) -> Option<TileId> {
        let new_tile = sample.get_value_from_index(tile.0)?;
        let matching_tile = {
            let mut result = None;
            for (t_key, &_pattern) in &self.tiles_to_patterns {
                if *t_key == tile {
                    result = Some(t_key.clone());
                    break;
                }
                let existing_tile = sample.get_value_from_index(t_key.0)?;

                if existing_tile.get_value() == new_tile.get_value() {
                    result = Some(t_key.clone());
                    break;
                }
            }
            result
        };
        matching_tile
    }

    /*

    // /// Constructs an AdjacentModel and initializes it with a given sample.
    // pub fn create<T>(sample: Vec<Vec<T>>, periodic: bool) -> Result<Self, String>
    // where
    //     T: Clone + Into<Tile> + std::fmt::Debug + std::any::Any + Send + Sync + PartialEq + std::hash::Hash + 'static,
    // {
    //     let topo_array = TopoArray2D::new(sample, periodic);
    //     let tile_array = topo_array.to_tiles()
    //         .map_err(|e| format!("Failed to convert to tiles: {}", e))?;
    //     Self::create_from_topo_array(&tile_array)
    // }
    //
    // /// Constructs an AdjacentModel and initializes it with a given sample.
    // pub fn create_from_topo_array<T: TopoArray<Tile, GridTopology>>(
    //     sample: &T
    // ) -> Result<Self, String> {
    //     let mut model = Self::new();
    //     model.add_sample_simple(sample)?;
    //     Ok(model)
    // }

    // /// Constructs an AdjacentModel with specified directions.
    // pub fn with_directions(directions: DirectionSet) -> Self {
    //     let mut model = Self::new();
    //     model.set_directions(directions).unwrap(); // Safe since directions is None initially
    //     model
    // }

    // /// Constructs an AdjacentModel and initializes it with a given sample.
    // pub fn from_sample<T: TopoArray<Tile, GridTopology>>(sample: &T) -> Result<Self, String> {
    //     let mut model = Self::new();
    //     model.add_sample_simple(sample)?;
    //     Ok(model)
    // }

    //
    // /// Finds a tile and all its rotations, and sets their total frequency.
    // pub fn set_frequency_with_rotation(
    //     &mut self,
    //     tile: &Tile,
    //     frequency: f64,
    //     tile_rotation: &TileRotation
    // ) -> Result<(), String> {
    //     let rotated_tiles = tile_rotation.rotate_all(tile);
    //
    //     // Get all patterns first to avoid borrowing issues
    //     let patterns: Vec<usize> = rotated_tiles.iter()
    //         .map(|rt| self.get_pattern(rt.clone()))
    //         .collect();
    //
    //     // Clear frequencies for all rotated tiles
    //     for &pattern in &patterns {
    //         if pattern < self.frequencies.len() {
    //             self.frequencies[pattern] = 0.0;
    //         }
    //     }
    //
    //     // Set incremental frequency
    //     let incremental_frequency = frequency / rotated_tiles.len() as f64;
    //     for &pattern in &patterns {
    //         if pattern < self.frequencies.len() {
    //             self.frequencies[pattern] += incremental_frequency;
    //         }
    //     }
    //
    //     Ok(())
    // }
    //
    // /// Sets the frequency of a given tile.
    // pub fn set_frequency(&mut self, tile: &Tile, frequency: f64) {
    //     let pattern = self.get_pattern(tile.clone());
    //     if pattern < self.frequencies.len() {
    //         self.frequencies[pattern] = frequency;
    //     }
    // }
    //
    // /// Sets all tiles as equally likely to be picked
    // pub fn set_uniform_frequency(&mut self) {
    //     let tiles: Vec<Tile> = self.tiles_to_patterns.keys().cloned().collect();
    //     for tile in tiles {
    //         self.set_frequency(&tile, 1.0);
    //     }
    // }
    //
    // /// Declares that the tiles in dest can be placed adjacent to the tiles in src, in the direction specified.
    // /// Then it adds similar declarations for other rotations and reflections, as specified by rotations.
    // pub fn add_adjacency_with_rotation(
    //     &mut self,
    //     src: &[Tile],
    //     dest: &[Tile],
    //     dir: Direction,
    //     tile_rotation: Option<&TileRotation>
    // ) -> Result<(), String> {
    //     let directions = self.require_directions()?;
    //     let d = dir as usize;
    //     let direction = match directions.get_direction_by_index(d) {
    //         Some(dir) => dir,
    //         None => return Err("Directions must be set before calling this method".to_string()),
    //     };
    //     let (x, y, z) = directions.get_deltas(direction);
    //     self.add_adjacency_xyz_with_rotation(src, dest, x, y, z, tile_rotation)
    // }
    //
    // /// Declares that the tiles in dest can be placed adjacent to the tiles in src, in the direction specified by (x, y, z).
    // /// Then it adds similar declarations for other rotations and reflections, as specified by rotations.
    // pub fn add_adjacency_xyz_with_rotation(
    //     &mut self,
    //     src: &[Tile],
    //     dest: &[Tile],
    //     x: i32,
    //     y: i32,
    //     z: i32,
    //     tile_rotation: Option<&TileRotation>
    // ) -> Result<(), String> {
    //     let directions = self.require_directions()?;
    //
    //     if let Some(tile_rotation) = tile_rotation {
    //         // Collect all the rotated tile pairs first to avoid borrowing conflicts
    //         let mut rotated_pairs = Vec::new();
    //
    //         for rotation in tile_rotation.rotation_group().rotations() {
    //             let (x2, y2) = TopoArrayUtils::rotate_vector(
    //                 directions.direction_type(),
    //                 x,
    //                 y,
    //                 rotation
    //             );
    //
    //             let rotated_src = tile_rotation.rotate_tiles(src.to_vec(), rotation);
    //             let rotated_dest = tile_rotation.rotate_tiles(dest.to_vec(), rotation);
    //
    //             rotated_pairs.push((rotated_src, rotated_dest, x2, y2, z));
    //         }
    //
    //         // Now process all the collected pairs
    //         for (rotated_src, rotated_dest, x2, y2, z2) in rotated_pairs {
    //             self.add_adjacency_xyz(&rotated_src, &rotated_dest, x2, y2, z2)?;
    //         }
    //     } else {
    //         self.add_adjacency_xyz(src, dest, x, y, z)?;
    //     }
    //
    //     Ok(())
    // }
    //
    // /// Declares that the tiles in dest can be placed adjacent to the tiles in src, in the direction specified by (x, y, z).
    // /// (x, y, z) must be a valid direction, which usually means a unit vector.
    // pub fn add_adjacency_xyz(
    //     &mut self,
    //     src: &[Tile],
    //     dest: &[Tile],
    //     x: i32,
    //     y: i32,
    //     z: i32
    // ) -> Result<(), String> {
    //     let directions = self.require_directions()?;
    //     let result = directions.get_direction(x, y, z);
    //     if result.is_err() {
    //         return Err(result.unwrap_err().to_string())
    //     }
    //     let dir = result.unwrap();
    //     self.add_adjacency_list(src, dest, dir)
    // }
    //
    // /// Declares that the tiles in dest can be placed adjacent to the tiles in src, in the direction specified.
    // pub fn add_adjacency_list(
    //     &mut self,
    //     src: &[Tile],
    //     dest: &[Tile],
    //     dir: Direction
    // ) -> Result<(), String> {
    //     self.require_directions()?;
    //
    //     for s in src {
    //         for d in dest {
    //             self.add_adjacency_single(s, d, dir)?;
    //         }
    //     }
    //
    //     Ok(())
    // }
    //
    // /// Declares that dest can be placed adjacent to src, in the direction specified by (x, y, z).
    // /// (x, y, z) must be a valid direction, which usually means a unit vector.
    // pub fn add_adjacency_xyz_single(
    //     &mut self,
    //     src: &Tile,
    //     dest: &Tile,
    //     x: i32,
    //     y: i32,
    //     z: i32
    // ) -> Result<(), String> {
    //     let directions = self.require_directions()?;
    //     let result = directions.get_direction(x, y, z);
    //     if result.is_err() {
    //         return Err(result.unwrap_err().to_string())
    //     }
    //     let d = result.unwrap();
    //     self.add_adjacency_single(src, dest, d)
    // }
    //
    // /// Declares that dest can be placed adjacent to src, in the direction specified.
    // pub fn add_adjacency_single(&mut self, src: &Tile, dest: &Tile, d: Direction) -> Result<(), String> {
    //     let directions = self.require_directions()?;
    //     let directions_count = directions.count();
    //     let id = directions.inverse(d);
    //
    //     // Get patterns first to avoid borrowing conflicts
    //     let src_pattern = self.get_pattern(src.clone());
    //     let dest_pattern = self.get_pattern(dest.clone());
    //
    //     // Ensure propagator is large enough
    //     while self.propagator.len() <= src_pattern.max(dest_pattern) {
    //         self.propagator.push(vec![HashSet::new(); directions_count]);
    //     }
    //
    //     self.propagator[src_pattern][d as usize].insert(dest_pattern);
    //     self.propagator[dest_pattern][id as usize].insert(src_pattern);
    //
    //     Ok(())
    // }
    //
    // pub fn add_adjacency(&mut self, adjacency: &Adjacency) -> Result<(), String> {
    //     for src in &adjacency.src {
    //         for dest in &adjacency.dest {
    //             self.add_adjacency_single(src, dest, adjacency.direction)?;
    //         }
    //     }
    //     Ok(())
    // }
    //
    // pub fn is_adjacent(&self, src: &Tile, dest: &Tile, d: Direction) -> bool {
    //     if let (Some(src_pattern), Some(dest_pattern)) = (
    //         self.tiles_to_patterns.get(src),
    //         self.tiles_to_patterns.get(dest)
    //     ) {
    //         if *src_pattern < self.propagator.len() {
    //             return self.propagator[*src_pattern][d as usize].contains(dest_pattern);
    //         }
    //     }
    //     false
    // }
    //
    // pub fn add_sample_with_rotation(
    //     &mut self,
    //     sample: &dyn TopoArray<Tile, GridTopology>,
    //     tile_rotation: Option<&TileRotation>
    // ) -> Result<(), String> {
    //     let rotated_samples = OverlappingAnalysis::get_rotated_samples(sample, tile_rotation)?;
    //
    //     for rotated_sample in rotated_samples {
    //         self.add_sample_simple(&rotated_sample)?;
    //     }
    //
    //     Ok(())
    // }
    */
}

impl TileModel<GridTopology> for AdjacentModel {
    fn get_tile_model_mapping(
        &mut self,
        grid_topology: &GridTopology,
    ) -> Result<TileModelMapping<GridTopology>, TraitError> {
        self.require_directions()?;
        self.set_directions(grid_topology.directions().clone())?;

        let total_frequency: f64 = self.frequencies.iter().sum();
        if total_frequency == 0.0 {
            return Err(Box::new(AdjacentModelError::NoFrequencies));
        }

        // Convert propagator to the required format
        let propagator_converted: Vec<Vec<Vec<usize>>> = self
            .propagator
            .iter()
            .map(|pattern_propagator| {
                pattern_propagator
                    .iter()
                    .map(|direction_set| direction_set.iter().cloned().collect())
                    .collect()
            })
            .collect();

        let pattern_model = PatternModel::new(propagator_converted, self.frequencies.clone());

        // Build mappings
        let mut tiles_to_patterns_by_offset = HashMap::new();
        let mut offset_map = HashMap::new();
        for (tile, &pattern) in &self.tiles_to_patterns {
            let mut pattern_set = HashSet::new();
            pattern_set.insert(pattern);
            offset_map.insert(tile.clone(), pattern_set);
        }
        tiles_to_patterns_by_offset.insert(0, offset_map);

        let mut patterns_to_tiles_by_offset = HashMap::new();
        let patterns_to_tiles: HashMap<usize, TileId> = self
            .tiles_to_patterns
            .iter()
            .map(|(tile, &pattern)| (pattern, tile.clone()))
            .collect();
        patterns_to_tiles_by_offset.insert(0, patterns_to_tiles);

        Ok(TileModelMapping::new(
            // &grid_topology,
            pattern_model,
            tiles_to_patterns_by_offset,
            patterns_to_tiles_by_offset,
        ))
    }

    fn get_tile(&self, index: usize) -> Option<&Tile> {
        self.sample.get_value_from_index(index)
    }

    //
    // fn tiles(&self) -> Vec<Tile> {
    //     self.tiles_to_patterns.keys().cloned().collect()
    // }
    //
    // fn multiply_frequency(&mut self, tile: &Tile, multiplier: f64) {
    //     if let Some(&pattern) = self.tiles_to_patterns.get(tile) {
    //         if pattern < self.frequencies.len() {
    //             self.frequencies[pattern] *= multiplier;
    //         }
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct Adjacency {
    pub src: Vec<TileId>,
    pub dest: Vec<TileId>,
    pub direction: Direction,
}

impl Adjacency {
    pub fn new(src: Vec<TileId>, dest: Vec<TileId>, direction: Direction) -> Self {
        Self {
            src,
            dest,
            direction,
        }
    }
}

// Just some debug print functions
impl AdjacentModel {
    pub fn debug_print_propagator(&self) {
        for (i, row) in self.propagator.iter().enumerate() {
            println!("Pattern {}:", i);
            for (j, col) in row.iter().enumerate() {
                println!("  {}: {:?}", j, col);
            }
        }
    }

    pub fn debug_print_frequencies(&self) {
        for (i, freq) in self.frequencies.iter().enumerate() {
            println!("Pattern {}: {}", i, freq);
        }
    }

    pub fn debug_print_tiles_to_patterns<T: Topology + Clone + 'static>(&self, tiles: &Vec<Tile>) {
        for (tile, pattern) in &self.tiles_to_patterns {
            let res = tiles.get(tile.0);
            match res {
                None => {
                    println!("Tile {}: {}", "", pattern);
                }
                Some(t) => {
                    println!("Tile {}: {}", t, pattern);
                }
            }
        }
    }
}

mod tests {
    #![allow(dead_code, unused_imports)]

    use super::*;
    use crate::tile::ToTile;
    use crate::topology::direction::DirectionSetType::Cartesian2d;
    use crate::topology::ragged_topology_array_2d::RaggedTopoArray2D;

    #[test]
    fn test_larger_sample() {
        // Arrange
        let initial_data = vec![
            vec!['_', '_', '_'],
            vec!['_', '*', '_'],
            vec!['_', '_', '_'],
        ];
        let initial_vec = initial_data
            .into_iter()
            .map(|x| x.clone().iter().map(|y| y.to_tile()).collect::<Vec<Tile>>())
            .collect::<Vec<Vec<Tile>>>();
        let sample = RaggedTopoArray2D::new(initial_vec, false);
        let mut model = AdjacentModel::new();

        // Act
        model.add_sample_simple::<GridTopology>(&sample).unwrap();

        // Assert
        let directions = model.directions.unwrap();
        assert_directions(&directions);
        model.directions = Some(directions);
        let star = TileId(4);
        let underscore = TileId(0);

        assert_eq!(model.tiles_to_patterns.len(), 2);
        assert_eq!(model.tiles_to_patterns.get(&star), Some(&1)); // '*'
        assert_eq!(model.tiles_to_patterns.get(&underscore), Some(&0)); // '_'

        assert_eq!(model.frequencies.len(), 2);
        assert_eq!(model.frequencies[0], 8.0); // '_'
        assert_eq!(model.frequencies[1], 1.0); // '*'

        assert_eq!(model.propagator.len(), 2);
        assert_eq!(model.propagator[0].len(), 4);
        assert_eq!(model.propagator[0][0].len(), 2);
        assert_eq!(model.propagator[0][0].get(&0), Some(&0));
        assert_eq!(model.propagator[0][0].get(&1), Some(&1));
        assert_eq!(model.propagator[0][1].len(), 2);
        assert_eq!(model.propagator[0][1].get(&0), Some(&0));
        assert_eq!(model.propagator[0][1].get(&1), Some(&1));
        assert_eq!(model.propagator[0][2].len(), 2);
        assert_eq!(model.propagator[0][2].get(&0), Some(&0));
        assert_eq!(model.propagator[0][2].get(&1), Some(&1));
        assert_eq!(model.propagator[0][3].len(), 2);
        assert_eq!(model.propagator[0][3].get(&0), Some(&0));
        assert_eq!(model.propagator[0][3].get(&1), Some(&1));

        assert_eq!(model.propagator[1].len(), 4);
        assert_eq!(model.propagator[1][0].len(), 1);
        assert_eq!(model.propagator[1][0].get(&0), Some(&0));
        assert_eq!(model.propagator[1][1].len(), 1);
        assert_eq!(model.propagator[1][1].get(&0), Some(&0));
        assert_eq!(model.propagator[1][2].len(), 1);
        assert_eq!(model.propagator[1][2].get(&0), Some(&0));
        assert_eq!(model.propagator[1][3].len(), 1);
        assert_eq!(model.propagator[1][3].get(&0), Some(&0));
    }

    fn assert_directions(directions: &DirectionSet) {
        assert_eq!(directions.count(), 4);
        assert_eq!(directions.dx().len(), 4);
        assert_eq!(directions.dx()[0], 1);
        assert_eq!(directions.dx()[1], -1);
        assert_eq!(directions.dx()[2], 0);
        assert_eq!(directions.dx()[3], 0);

        assert_eq!(directions.dy().len(), 4);
        assert_eq!(directions.dy()[0], 0);
        assert_eq!(directions.dy()[1], 0);
        assert_eq!(directions.dy()[2], 1);
        assert_eq!(directions.dy()[3], -1);

        assert_eq!(directions.dz().len(), 4);
        assert_eq!(directions.dz()[0], 0);
        assert_eq!(directions.dz()[1], 0);
        assert_eq!(directions.dz()[2], 0);
        assert_eq!(directions.dz()[3], 0);

        assert_eq!(directions.direction_type(), Cartesian2d);
    }

    fn to_tiles(initial_data: &Vec<Vec<char>>) -> Vec<Vec<Tile>> {
        initial_data
            .into_iter()
            .map(|x| x.clone().iter().map(|y| y.to_tile()).collect::<Vec<Tile>>())
            .collect::<Vec<Vec<Tile>>>()
    }

    #[test]
    fn test_frequencies() {
        // Arrange
        let initial_data = vec![vec!['_', '+'], vec!['_', '*']];
        let initial_vec = to_tiles(&initial_data);
        let sample = RaggedTopoArray2D::new(initial_vec, false);
        let mut model = AdjacentModel::new();

        // Act
        model.add_sample_simple::<GridTopology>(&sample).unwrap();

        // Assert
        assert_eq!(model.frequencies.len(), 3);
        assert_eq!(model.frequencies[0], 2.0);
        assert_eq!(model.frequencies[1], 1.0);
        assert_eq!(model.frequencies[2], 1.0);
    }

    #[test]
    fn test_tiles_to_patterns() {
        // Arrange
        let initial_data = vec![vec!['_', '+'], vec!['_', '*']];
        let initial_vec = to_tiles(&initial_data);
        let sample = RaggedTopoArray2D::new(initial_vec, false);
        let mut model = AdjacentModel::new();

        // Act
        model.add_sample_simple::<GridTopology>(&sample).unwrap();

        // Assert
        assert_eq!(model.tiles_to_patterns.len(), 3);
        let underscore = TileId(0);
        let plus = TileId(1);
        let star = TileId(3);
        assert_eq!(model.tiles_to_patterns.get(&star), Some(&2)); // '*'
        assert_eq!(model.tiles_to_patterns.get(&plus), Some(&1)); // '+'
        assert_eq!(model.tiles_to_patterns.get(&underscore), Some(&0)); // '_'
    }
}
