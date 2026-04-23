use crate::topology::grid_topology::GridTopology;
use crate::topology::ragged_topology_array_2d::{RaggedTopoArray2D, RaggedTopoArray2DError};
use crate::topology::topo_array::{TopoArray, TopoArray2D};
use crate::topology::topology::{Topology, TopologyError};

#[derive(Clone)]
pub struct TopoArray3D<T: Clone + 'static> {
    values: Vec<Vec<Vec<T>>>,
    topology: GridTopology,
}

impl<T: Clone> TopoArray3D<T> {
    pub fn new(values: Vec<Vec<Vec<T>>>, periodic: bool) -> Self {
        let topology = GridTopology::new_3d(values.len(), values[0].len(), values[0][0].len(), periodic);
        Self {
            values,
            topology,
        }
    }

    pub fn with_topology(values: Vec<Vec<Vec<T>>>, topology: GridTopology) -> Self {
        Self {
            values,
            topology,
        }
    }

    /// Gets a copy of the value at the specified location.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<T> {
        if x > self.values.len() - 1 || y > self.values[0].len() - 1 || z > self.values[0][0].len() - 1 {
            return None;
        }
        Some(self.values[x][y][z].clone())
    }

    /// Gets a copy of the value at the specified index.
    pub fn get_index_internal(&self, index: usize) -> Option<T> {
        let res = self.topology.get_coord(index);
        if res.is_err() {
            return None;
        }
        let (x, y, z) = res.unwrap();
        self.get(x, y, z)
    }
}

impl<T: Clone> TopoArray<T, GridTopology> for TopoArray3D<T> {
    fn topology(&self) -> Option<&GridTopology> {
        Some(&self.topology)
    }

    fn get_coord(&self, x: usize, y: usize, z: usize) -> Result<&T, TopologyError> {
        if x >= self.values.len() || y >= self.values[0].len() || z >= self.values[0][0].len() {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z });
        }

        Ok(self.values.get(x).unwrap().get(y).unwrap().get(z).unwrap())
    }

    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        let result = self.topology.get_coord(index);
        if result.is_err() {
            return Err(TopologyError::TraitError(result.unwrap_err().into()));
        }
        let (x, y, z) = result.unwrap();
        self.get_coord(x, y, z)
    }

    /// The index here is from the flattened array, not the original
    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        self.get_index(index).ok()
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.get_coord(x, y, z).ok()
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        unimplemented!("is this ever used?");
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        unimplemented!("is this ever used?");
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, GridTopology>>> {
        Some(Box::new(self.clone()))
    }
}