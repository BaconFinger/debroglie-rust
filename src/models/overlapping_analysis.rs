use std::collections::HashMap;
use std::rc::Rc;
use crate::models::pattern_array::PatternArray;
use crate::rot::tile_rotation::TileRotation;
use crate::tile::Tile;
use crate::topology::grid_topology::{GridTopology, GridTopologyError};
use crate::topology::topo_array::TopoArray;
use crate::topology::topo_array_utils::TopoArrayUtils;
use crate::topology::topology::Topology;

#[derive(Debug, thiserror::Error)]
pub enum OverlappingAnalysisError {
    #[error("{msg}")]
    Other { msg: String }, // TODO: Remove

    #[error(transparent)]
    GridTopologyError(#[from] GridTopologyError),
}

pub struct OverlappingAnalysis {}

#[derive(Debug, Clone, Copy)]
pub struct Coord3d {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Periodics {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl OverlappingAnalysis {
    pub fn get_rotated_samples(sample: &dyn TopoArray<Tile, GridTopology>, rotation: Option<TileRotation>) -> Result<Vec<Box<dyn TopoArray<Tile, GridTopology>>>, OverlappingAnalysisError> {
        let default_rotation = TileRotation::default();
        let tile_rotation = if rotation.is_some() { rotation.as_ref().unwrap()} else { &default_rotation };

        let res = tile_rotation.get_rotation_group().get_rotations().iter()
            .map(|rotation| TopoArrayUtils::rotate(sample, rotation, tile_rotation))
            .collect();
        Ok(res)
    }
    pub fn get_patterns<'a>(
        sample: &'a dyn TopoArray<Tile, GridTopology>,
        ns: Coord3d,
        periodics: Periodics,
        pattern_indices: &mut HashMap<Rc<PatternArray>, i32>,
        pattern_arrays: &mut Vec<Rc<PatternArray>>,
        frequencies: &mut Vec<f64>,
    ) -> Result<(), OverlappingAnalysisError> {
        if sample.topology().is_none() {
            return Err(OverlappingAnalysisError::Other { msg: "Sample has no topology".to_string() });
        }
        let topology = sample.topology().unwrap();
        let width = topology.width();
        let height = topology.height();
        let depth = topology.depth();
        let maxx = if periodics.x { width - 1 } else { width - ns.x };
        let maxy = if periodics.y { height - 1 } else { height - ns.y };
        let maxz = if periodics.z { depth - 1 } else { depth - ns.z };

        for x in 0..=maxx {
            for y in 0..=maxy {
                for z in 0..=maxz {
                    let mut pattern_array = PatternArray::default();
                    if !Self::try_extract(sample, ns, Coord3d { x, y, z }, &mut pattern_array)? {
                        continue;
                    }

                    let pattern = pattern_indices.get(&pattern_array);
                    if pattern.is_none() {
                        let pattern_array_ref = Rc::new(pattern_array);
                        pattern_indices.insert(pattern_array_ref.clone(), pattern_indices.len() as i32);
                        pattern_arrays.push(pattern_array_ref.clone());
                        frequencies.push(1.0);
                    } else {
                        frequencies[pattern.unwrap().clone() as usize] += 1.0;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn try_extract<'a>(
        sample: &'a dyn TopoArray<Tile, GridTopology>,
        ns: Coord3d,
        point: Coord3d,
        pattern_array: &mut PatternArray
    ) -> Result<bool, OverlappingAnalysisError> {
        let topology = sample.topology().ok_or(OverlappingAnalysisError::Other { msg: "Sample has no topology".to_string() })?;
        let (width, height, depth) = (topology.width(), topology.height(), topology.depth());
        let (x, y, z) = (point.x.clone(), point.y.clone(), point.z.clone());
        let (nx, ny, nz) = (ns.x.clone(), ns.y.clone(), ns.z.clone());
        let mut values: Vec<Vec<Vec<Tile>>> = vec![vec![vec![Tile::default(); nz]; ny]; nx];

        for tx in 0..nx {
            let sx = (x + tx) % width;
            for ty in 0..ny {
                let sy = (y + ty) % height;
                for tz in 0..nz {
                    let sz = (z + tz) % depth;
                    let index = topology.get_index(sx, sy, sz)?;
                    if !topology.contains_index(index) {
                        *pattern_array = PatternArray::default();
                        return Ok(false);
                    }
                    let val = sample.get_value_from_coord(sx, sy, sz);
                    if val.is_none() { // TODO: Remove when this is determined.
                        println!("No tile found for ({}, {}, {})", sx, sy, sz);
                    }
                    values[tx][ty][tz] = val.unwrap().clone();
                }
            }
        }
        *pattern_array = PatternArray::new(values);
        Ok(true)
    }
}