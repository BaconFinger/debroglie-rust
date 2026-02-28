use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::refactor::topology::grid_topology::GridTopology;
use crate::refactor::topology::topo_array::{TopoArray, TopoArray2D};
use crate::refactor::topology::topology::{Topology, TopologyError};

/// A 2D array with potentially different row lengths, coupled with a topology
#[derive(Debug, Clone)]
pub struct RaggedTopoArray2D<T> {
    original_values: Vec<Vec<T>>,
    values: Vec<Vec<usize>>,
    flattened_values: Vec<T>,
    topology: GridTopology,
}

impl<T: Clone> RaggedTopoArray2D<T> {
    /// Create a new RaggedTopoArray2D from a jagged array and periodicity.
    /// This also creates its own output topology and stores that in the Context.
    pub fn new(values: Vec<Vec<T>>, periodic: bool) -> Self {
        let height = values.len();
        let width = values.iter().map(|row| row.len()).max().unwrap_or(0);

        // Create the output topology
        let topology = GridTopology::new_2d(width, height, periodic);
        let (flattened_values, mapped) = flatten_and_map_values(&values);

        Self { values: mapped, topology, flattened_values, original_values: values }
    }

    /// Create a new RaggedTopoArray2D from a jagged array and explicit topology
    pub fn with_topology(values: Vec<Vec<T>>, topology: GridTopology) -> RaggedTopoArray2DGeneric<T, GridTopology>
    {
        RaggedTopoArray2DGeneric::new(values, topology)
    }

    /// Get the width of the widest row
    pub fn max_width(&self) -> usize {
        self.values.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    /// Get the number of rows
    pub fn height(&self) -> usize {
        self.values.len()
    }

    /// Get the width of a specific row
    pub fn row_width(&self, y: usize) -> Option<usize> {
        self.values.get(y).map(|row| row.len())
    }

    /// Check if coordinates are valid for the ragged array
    pub fn is_valid_coord(&self, x: usize, y: usize) -> bool {
        if y >= self.values.len() {
            return false;
        }
        x < self.values[y].len()
    }
}

fn flatten_and_map_values<T: Clone>(values: &Vec<Vec<T>>) -> (Vec<T>, Vec<Vec<usize>>) {
    let mut flattened_values: Vec<T> = Vec::new();
    let mut mapped = Vec::new();
    let mut flattened_index: usize = 0;
    for (y, row) in values.iter().enumerate() {
        mapped.push(Vec::new());
        for (x, value) in row.iter().enumerate() {
            mapped[y].push(flattened_index);
            flattened_values.push(value.clone());
            flattened_index += 1;
        }
    }
    (flattened_values, mapped)
}

impl<T: Clone + 'static> TopoArray<T, GridTopology> for RaggedTopoArray2D<T> {
    fn topology(&self) -> Option<&GridTopology> {
        Some(&self.topology)
    }

    fn get_coord(&self, x: usize, y: usize, _z: usize) -> Result<&T, TopologyError> {
        if y >= self.values.len() {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z: 0 });
        }

        let index = self.values[y].get(x).ok_or(TopologyError::CoordinateOutOfBounds { x, y, z: 0 })?;
        self.get_value_from_index(index.clone()).ok_or(TopologyError::Other(format!("Unable to get value for coordinate ({}, {})", x, y)))
    }

    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        let result = self.topology.get_coord(index);
        if result.is_err() {
            return Err(TopologyError::Other(format!("GridTopologyError: {}", result.unwrap_err().to_string())));
        }
        let (x, y, z) = result.unwrap();
        self.get_coord(x, y, z)
    }

    /// The index here is from the flattened array, not the original
    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        if index >= self.flattened_values.len() {
            println!("Index out of bounds: {}", index);
            return None;
        }

        self.flattened_values.get(index)
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        if z > 1 {
            println!("Called for a depth of {} on a 2D array, ignoring", z);
        }
        if y >= self.values.len() {
            return None;
        }
        if x >= self.values[y].len() {
            return None;
        }
        let index = self.values[y].get(x)?;
        self.get_value_from_index(index.clone())
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        let result = self.topology.get_coord(index);
        if result.is_err() {
            return None;
        }
        let (x, y, z) = result.unwrap();
        self.get_id_from_coord(x, y, z)
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if y >= self.values.len() {
            return None;
        }

        let idx = self.values[y].get(x);
        if idx.is_none() {
            return None;
        }
        Some(idx.unwrap().clone())
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, GridTopology>>> {
        Some(Box::new(self.clone()))
    }
}

impl<T> TopoArray2D<T, GridTopology> for RaggedTopoArray2D<T> {
    fn get_contents(&self) -> &Vec<Vec<T>> {
        &self.original_values
    }
}


/// Generic version that works with any topology type
#[derive(Debug, Clone)]
pub struct RaggedTopoArray2DGeneric<T, TopologyT>
where
    TopologyT: Topology + Clone,
{
    original_values: Vec<Vec<T>>,
    values: Vec<Vec<usize>>,
    flattened_values: Vec<T>,
    topology: TopologyT,
}

impl<T, TopologyT> RaggedTopoArray2DGeneric<T, TopologyT>
where
    T: Clone,
    TopologyT: Topology + Clone,
{
    /// Create a new RaggedTopoArray2DGeneric from a jagged array and topology
    pub fn new(values: Vec<Vec<T>>, topology: TopologyT) -> Self {
        let (flattened_values, mapped) = flatten_and_map_values(&values);

        Self { values: mapped, topology, flattened_values, original_values: values }
    }

    /// Get the width of the widest row
    pub fn max_width(&self) -> usize {
        self.values.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    /// Get the number of rows
    pub fn height(&self) -> usize {
        self.values.len()
    }

    /// Get the width of a specific row
    pub fn row_width(&self, y: usize) -> Option<usize> {
        self.values.get(y).map(|row| row.len())
    }

    /// Check if coordinates are valid for the ragged array
    pub fn is_valid_coord(&self, x: usize, y: usize) -> bool {
        if y >= self.values.len() {
            return false;
        }
        x < self.values[y].len()
    }
}

impl<T, TopologyT> TopoArray<T, TopologyT> for RaggedTopoArray2DGeneric<T, TopologyT>
where
    T: Clone + 'static,
    TopologyT: Topology + Clone + 'static,
{
    fn topology(&self) -> Option<&TopologyT> {
        Some(&self.topology)
    }

    fn get_coord(&self, x: usize, y: usize, _z: usize) -> Result<&T, TopologyError> {
        if y >= self.values.len() {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z: 0 });
        }

        let index = self.values[y].get(x).ok_or(TopologyError::CoordinateOutOfBounds { x, y, z: 0 })?;
        self.get_value_from_index(index.clone()).ok_or(TopologyError::Other(format!("Unable to get value for coordinate ({}, {})", x, y)))
    }

    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        let (x, y, z) = self.topology.get_coord(index)?;
        self.get_coord(x, y, z)
    }

    /// The index here is from the flattened array, not the original
    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        if index >= self.flattened_values.len() {
            println!("Index out of bounds: {}", index);
            return None;
        }

        self.flattened_values.get(index)
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        if z > 1 {
            println!("Called for a depth of {} on a 2D array, ignoring", z);
        }
        if y >= self.values.len() {
            return None;
        }
        if x >= self.values[y].len() {
            return None;
        }
        let index = self.values[y].get(x)?;
        self.get_value_from_index(index.clone())
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        let result = self.topology.get_coord(index);
        if result.is_err() {
            return None;
        }
        let (x, y, z) = result.unwrap();
        self.get_id_from_coord(x, y, z)
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if y >= self.values.len() {
            return None;
        }

        let idx = self.values[y].get(x);
        if idx.is_none() {
            return None;
        }
        Some(idx.unwrap().clone())
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, TopologyT>>> {
        Some(Box::new(self.clone()))   
    }
}

impl<T, TopologyT> TopoArray2D<T, TopologyT> for RaggedTopoArray2DGeneric<T, TopologyT>
where
    TopologyT: Topology + Clone,
{
    fn get_contents(&self) -> &Vec<Vec<T>> {
        &self.original_values
    }
}