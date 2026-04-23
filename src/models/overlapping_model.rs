use std::collections::HashMap;
use std::rc::Rc;
use crate::models::overlapping_analysis::{Coord3d, OverlappingAnalysis, OverlappingAnalysisError, Periodics};
use crate::models::pattern_array::PatternArray;
use crate::rot::tile_rotation::TileRotation;
use crate::tile::Tile;
use crate::topology::direction::DirectionSet;
use crate::topology::grid_topology::GridTopology;
use crate::topology::topo_array::TopoArray;

#[derive(Debug, thiserror::Error)]
pub enum OverlappingModelError {
    #[error("{msg}")]
    Other { msg: String }, // TODO: Remove

    #[error(transparent)]
    OverlappingAnalysisError(#[from] OverlappingAnalysisError),
}

/// OverlappingModel constrains that every n by n rectangle in the output is a copy of a rectangle taken from the sample.
pub struct OverlappingModel<'a> {
    nx: i32,
    ny: i32,
    nz: i32,

    pattern_indices: HashMap<Rc<PatternArray>, i32>,
    pattern_arrays: Vec<Rc<PatternArray>>,
    frequencies: Vec<f64>,
    sample_topology_directions: DirectionSet,
    propagator: Vec<Vec<Vec<i32>>>,

    patterns_to_tiles: HashMap<i32, &'a Tile>,
    tiles_to_patterns: HashMap<&'a Tile, Vec<i32>>,
}

impl<'a> OverlappingModel<'a> {
    pub fn new() -> Self {
        Self {
            nx: 0,
            ny: 0,
            nz: 0,

            pattern_indices: HashMap::new(),
            pattern_arrays: Vec::new(),
            frequencies: Vec::new(),
            sample_topology_directions: DirectionSet::CARTESIAN_2D,
            propagator: Vec::new(),

            patterns_to_tiles: HashMap::new(),
            tiles_to_patterns: HashMap::new(),
        }
    }

    pub fn with_sample(sample: &dyn TopoArray<Tile, GridTopology>, n: i32, rotational_symmetry: i32, reflectional_symmetry: bool) -> Result<Self, OverlappingModelError> {
        let mut model = Self::new();
        model.nx = n;
        model.ny = n;
        model.nz = n;
        let rot = TileRotation::new(rotational_symmetry, reflectional_symmetry);
        model.add_sample(sample, rot);

        Ok(model)
    }

    pub fn add_sample(&mut self, sample: &dyn TopoArray<Tile, GridTopology>, rotation: TileRotation) -> Result<(), OverlappingModelError> {
        if sample.topology().is_none() {
            return Err(OverlappingModelError::Other { msg: "Sample has no topology".to_string() });
        }
        if sample.topology().unwrap().depth() == 1 {
            self.nz = 1;
        }

        let periodic_x = sample.topology().unwrap().periodic_x();
        let periodic_y = sample.topology().unwrap().periodic_y();
        let periodic_z = sample.topology().unwrap().periodic_z();

        let rotated_samples = OverlappingAnalysis::get_rotated_samples(sample, Some(rotation))?;
        for rotated_sample in rotated_samples.iter() {
            OverlappingAnalysis::get_patterns(
                rotated_sample.as_ref(),
                Coord3d { x: self.nx as usize, y: self.ny as usize, z: self.nz as usize },
                Periodics { x: periodic_x, y: periodic_y, z: periodic_z },
                &mut self.pattern_indices,
                &mut self.pattern_arrays,
                &mut self.frequencies,
            )?
        }

        self.sample_topology_directions = sample.topology().unwrap().directions().clone();
        self.propagator = Vec::new(); // Mark as dirty

        Ok(())
    }
}