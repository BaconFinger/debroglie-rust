use crate::topology::topo_array::TopoArray;
use crate::topology::topology::{Topology, TopologyError};

/// A 1D topological array implementation
pub struct TopoArray1D<T, TopologyT: Topology + Clone> {
    values: Vec<T>,
    topology: TopologyT,
}

impl<T, TopologyT: Topology + Clone> TopoArray1D<T, TopologyT> {
    /// Creates a new TopoArray1D with the given values and topology
    pub fn new(values: Vec<T>, topology: TopologyT) -> Self {
        Self { values, topology }
    }
}

impl<T: Clone + 'static, TopologyT: Topology + Clone> TopoArray<T, TopologyT> for TopoArray1D<T, TopologyT> {
    /// Gets the topology associated with this array
    fn topology(&self) -> Option<&TopologyT> {
        Some(&self.topology)
    }

    /// Gets the value at the specified index
    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        self.values.get(index)
            .ok_or(TopologyError::IndexOutOfBounds {
                index,
                max: self.values.len()
            })
    }

    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        unimplemented!();
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        unimplemented!();
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        unimplemented!();
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        unimplemented!();
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, TopologyT>>> {
        unimplemented!();
    }
}