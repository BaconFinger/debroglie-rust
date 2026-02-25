
use crate::refactor::topology::grid_topology::GridTopology;
use crate::refactor::topology::ragged_topology_array_2d::{RaggedTopoArray2D, RaggedTopoArray2DGeneric};
use crate::refactor::topology::topology::{Topology, TopologyError};

/// A read-only array coupled with a specific Topology
pub trait TopoArray<T, Topo: Topology + Clone> {

    /// Gets the value at a particular location.
    fn get_coord(&self, topology: &Topo, x: usize, y: usize, z: usize) -> Result<&T, TopologyError> {
        let index = topology.get_index(x, y, z)?;
        self.get_index(topology, index)
    }

    /// Gets the value at a particular location (2D convenience method)
    fn get_coord_2d(&self, topology: &Topo, x: usize, y: usize) -> Result<&T, TopologyError> {
        self.get_coord(topology, x, y, 0)
    }

    /// Gets the value at a particular location.
    /// See Topology to see how location indices work.
    fn get_index(&self, topology: &Topo, index: usize) -> Result<&T, TopologyError>;

    /// Gets the total number of elements
    fn len(&self, topology: &Topo) -> Option<usize> {
        Some(topology.index_count())
    }

    /// Returns true if the array is empty
    fn is_empty(&self, topology: &Topo) -> bool {
        self.len(topology).unwrap_or(0) == 0
    }
}

/// A mutable array coupled with a specific Topology
pub trait TopoArrayMut<T, Topo: Topology + Clone>: TopoArray<T, Topo> {
    /// Sets the value at a particular location.
    fn set_coord(&mut self, topology: &Topo, x: usize, y: usize, z: usize, value: T) -> Result<(), TopologyError> {
        let index = topology.get_index(x, y, z)?;
        self.set_index(index, value)
    }

    /// Sets the value at a particular location (2D convenience method)
    fn set_coord_2d(&mut self, topology: &Topo, x: usize, y: usize, value: T) -> Result<(), TopologyError> {
        self.set_coord(topology, x, y, 0, value)
    }

    /// Sets the value at a particular location.
    fn set_index(&mut self, index: usize, value: T) -> Result<(), TopologyError>;

    /// Gets a mutable reference to the value at a particular location.
    fn get_mut_coord(&mut self, topology: &Topo, x: usize, y: usize, z: usize) -> Result<&mut T, TopologyError> {
        let index = topology.get_index(x, y, z)?;
        self.get_mut_index(index)
    }

    /// Gets a mutable reference to the value at a particular location (2D convenience method)
    fn get_mut_coord_2d(&mut self, topology: &Topo, x: usize, y: usize) -> Result<&mut T, TopologyError> {
        self.get_mut_coord(topology, x, y, 0)
    }

    /// Gets a mutable reference to the value at a particular location.
    fn get_mut_index(&mut self, index: usize) -> Result<&mut T, TopologyError>;
}

/// Utility struct containing methods for constructing TopoArray objects.
/// This is the Rust equivalent of the C# TopoArray static class.
pub struct TopologyArray;

impl TopologyArray {
    // /// Constructs a TopoArray from a vector. `result.get(i) == values[i]`
    // pub fn create<T, Topo: Topology>(values: Vec<T>, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_1d::TopoArray1D<T, Topo> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_1d::TopoArray1D::new(values, topology)
    // }
    // 
    // /// Constructs a TopoArray from a 2D vector. `result.get(x, y) == values[x][y]`
    // pub fn create_2d<T>(values: Vec<Vec<T>>, periodic: bool) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_2d::TopoArray2D<T, crate::procedural_generation::debroglie_scuffed::topology::grid_topology::GridTopology> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_2d::TopoArray2D::new(values, periodic)
    // }
    // 
    // /// Constructs a TopoArray from a 2D vector with explicit topology. `result.get(x, y) == values[x][y]`
    // pub fn create_2d_with_topology<T, Topo: Topology>(values: Vec<Vec<T>>, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_2d::TopoArray2D<T, Topo> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_2d::TopoArray2D::with_topology(values, topology)
    // }

    /// Constructs a TopoArray from a jagged vector. `result.get(x, y) == values[y][x]`
    /// Note: This follows the C# convention where jagged arrays are accessed as values[y][x]
    pub fn create_jagged<T>(ctx: &Context<GridTopology>, values: Vec<Vec<T>>, periodic: bool) -> RaggedTopoArray2D<T> {
        RaggedTopoArray2D::new(ctx, values, periodic)
    }

    /// Constructs a TopoArray from a jagged vector with explicit topology. `result.get(x, y) == values[y][x]`
    pub fn create_jagged_with_topology<T, Topo: Topology + Clone>(values: Vec<Vec<T>>, topology: TopologyId) -> RaggedTopoArray2DGeneric<T, Topo> {
        RaggedTopoArray2D::with_topology(values, topology)
    }

    // /// Constructs a TopoArray from a 3D vector. `result.get(x, y, z) == values[x][y][z]`
    // pub fn create_3d<T>(values: Vec<Vec<Vec<T>>>, periodic: bool) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D<T, crate::procedural_generation::debroglie_scuffed::topology::grid_topology::GridTopology> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D::new(values, periodic)
    // }
    // 
    // /// Constructs a TopoArray from a 3D vector with explicit topology. `result.get(x, y, z) == values[x][y][z]`
    // pub fn create_3d_with_topology<T, Topo: Topology>(values: Vec<Vec<Vec<T>>>, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D<T, Topo> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D::with_topology(values, topology)
    // }
    // 
    // /// Constructs a TopoArray with a constant value for all positions
    // pub fn from_constant<T, Topo: Topology>(value: T, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_constant::TopoArrayConstant<T, Topo> {
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_constant::TopoArrayConstant::new(value, topology)
    // }
    // 
    // /// Constructs a TopoArray by invoking f at each location in the topology.
    // pub fn create_by_point<T, Topo>(f: fn(crate::procedural_generation::debroglie_scuffed::point::Point<i32>) -> T, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D<T, Topo>
    // where
    //     T: Default + Clone,
    //     Topo: Topology + crate::procedural_generation::debroglie_scuffed::topology::topology_extensions::TopologyExtensions,
    // {
    //     let width = topology.width();
    //     let height = topology.height();
    //     let depth = topology.depth();
    // 
    //     let mut values = vec![vec![vec![T::default(); depth]; height]; width];
    // 
    //     for z in 0..depth {
    //         for y in 0..height {
    //             for x in 0..width {
    //                 if let Ok(index) = topology.get_index(x, y, z) {
    //                     if topology.contains_index(index) {
    //                         values[x][y][z] = f(crate::procedural_generation::debroglie_scuffed::point::Point::new(x as i32, y as i32, z as i32));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // 
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_3d::TopoArray3D::with_topology(values, topology)
    // }
    // 
    // /// Constructs a TopoArray by invoking f at each index in the topology.
    // pub fn create_by_index<T, Topo>(f: fn(usize) -> T, topology: Topo) -> crate::procedural_generation::debroglie_scuffed::topology::topo_array_1d::TopoArray1D<T, Topo>
    // where
    //     Topo: Topology + crate::procedural_generation::debroglie_scuffed::topology::topology_extensions::TopologyExtensions,
    // {
    //     let mut values = Vec::with_capacity(topology.index_count());
    // 
    //     // Get all valid indices from the topology
    //     for index in topology.get_indices() {
    //         values.push(f(index));
    //     }
    // 
    //     // If the topology has fewer valid indices than the total count,
    //     // we need to fill the vector to match the topology's index_count
    //     while values.len() < topology.index_count() {
    //         values.push(f(values.len()));
    //     }
    // 
    //     crate::procedural_generation::debroglie_scuffed::topology::topo_array_1d::TopoArray1D::new(values, topology)
    // }
}