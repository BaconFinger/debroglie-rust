use std::any::Any;
use std::fmt;
use crate::topology::direction::{Direction, EdgeLabel};
use crate::topology::grid_topology::GridTopology;

/// Error types for topology operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyError {
    IndexOutOfBounds { index: usize, max: usize },
    CoordinateOutOfBounds { x: usize, y: usize, z: usize },
    InvalidMaskLength { expected: usize, actual: usize },

    Other(String),
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopologyError::IndexOutOfBounds { index, max } => {
                write!(f, "Index {} is out of bounds (max: {})", index, max)
            }
            TopologyError::CoordinateOutOfBounds { x, y, z } => {
                write!(f, "Coordinate ({}, {}, {}) is out of bounds", x, y, z)
            }
            TopologyError::InvalidMaskLength { expected, actual } => {
                write!(f, "Invalid mask length: expected {}, got {}", expected, actual)
            }

            TopologyError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl std::error::Error for TopologyError {}

/// A Topology specifies a discrete area, volume or graph, and provides generic navigation methods.
/// Topologies are used to support generation in a wide variety of shapes.
/// Topologies do not actually store data, they just specify the dimensions.
/// Actual data is stored in a TopoArray.
/// Further information can be found in the documentation.
pub trait Topology {
    /// Number of unique indices (distinct locations) in the topology
    fn index_count(&self) -> usize;

    /// Number of unique directions
    fn directions_count(&self) -> usize;

    /// The extent along the x-axis.
    fn width(&self) -> usize;

    /// The extent along the y-axis.
    fn height(&self) -> usize;

    /// The extent along the z-axis.
    fn depth(&self) -> usize;

    /// Given an index and a direction, gives the index that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    /// Additionally returns information about the edge traversed.
    fn try_move_full(
        &self,
        index: usize,
        direction: Direction,
    ) -> Result<Option<(usize, Direction, EdgeLabel)>, TopologyError>;

    /// Given an index and a direction, gives the index that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    fn try_move(&self, index: usize, direction: Direction) -> Result<Option<usize>, TopologyError> {
        match self.try_move_full(index, direction)? {
            Some((dest, _, _)) => Ok(Some(dest)),
            None => Ok(None),
        }
    }

    /// Given a co-ordinate and a direction, gives the index that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    /// Additionally returns information about the edge traversed.
    fn try_move_coord_full(
        &self,
        x: usize,
        y: usize,
        z: usize,
        direction: Direction,
    ) -> Result<Option<(usize, Direction, EdgeLabel)>, TopologyError> {
        let index = self.get_index(x, y, z)?;
        self.try_move_full(index, direction)
    }

    /// Given a co-ordinate and a direction, gives the index that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    fn try_move_coord(
        &self,
        x: usize,
        y: usize,
        z: usize,
        direction: Direction,
    ) -> Result<Option<usize>, TopologyError> {
        let index = self.get_index(x, y, z)?;
        self.try_move(index, direction)
    }

    /// Given a co-ordinate and a direction, gives the co-ordinate that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    fn try_move_coord_to_coord(
        &self,
        x: usize,
        y: usize,
        z: usize,
        direction: Direction,
    ) -> Result<Option<(usize, usize, usize)>, TopologyError> {
        match self.try_move_coord(x, y, z, direction)? {
            Some(dest_index) => {
                let (dest_x, dest_y, dest_z) = self.get_coord(dest_index)?;
                Ok(Some((dest_x, dest_y, dest_z)))
            }
            None => Ok(None),
        }
    }

    /// A array with one value per index indicating if the value is missing.
    /// Not all uses of Topology support masks.
    fn mask(&self) -> Option<Vec<bool>>;

    /// Reduces a three dimensional co-ordinate to a single integer. This is mostly used internally.
    fn get_index(&self, x: usize, y: usize, z: usize) -> Result<usize, TopologyError>;

    /// Inverts get_index
    fn get_coord(&self, index: usize) -> Result<(usize, usize, usize), TopologyError>;

    /// Returns a topology with the same structure as this one,
    /// but with a different mask.
    fn with_mask(&self, mask: Vec<bool>) -> Result<Self, TopologyError>
    where
        Self: Sized;

    /// Check if a coordinate is valid (within bounds)
    fn is_valid_coord(&self, x: usize, y: usize, z: usize) -> bool {
        x < self.width() && y < self.height() && z < self.depth()
    }

    /// Check if an index is valid (within bounds)
    fn is_valid_index(&self, index: usize) -> bool {
        index < self.index_count()
    }

    /// Check if a position is masked out
    fn is_masked(&self, index: usize) -> Result<bool, TopologyError> {
        if !self.is_valid_index(index) {
            return Err(TopologyError::IndexOutOfBounds {
                index,
                max: self.index_count(),
            });
        }

        if let Some(mask) = self.mask() {
            Ok(!mask[index])
        } else {
            Ok(false) // No mask means not masked
        }
    }

    /// 2D convenience methods
    fn try_move_2d(&self, x: usize, y: usize, direction: Direction) -> Result<Option<usize>, TopologyError> {
        self.try_move_coord(x, y, 0, direction)
    }

    fn try_move_2d_to_coord(&self, x: usize, y: usize, direction: Direction) -> Result<Option<(usize, usize)>, TopologyError> {
        match self.try_move_coord_to_coord(x, y, 0, direction)? {
            Some((dest_x, dest_y, _)) => Ok(Some((dest_x, dest_y))),
            None => Ok(None),
        }
    }

    fn get_index_2d(&self, x: usize, y: usize) -> Result<usize, TopologyError> {
        self.get_index(x, y, 0)
    }

    fn get_coord_2d(&self, index: usize) -> Result<(usize, usize), TopologyError> {
        let (x, y, _) = self.get_coord(index)?;
        Ok((x, y))
    }

    /// Returns true if a given index has not been masked out
    fn contains_index(&self, index: usize) -> bool {
        match self.mask() {
            None => index < self.index_count(), // No mask, just check bounds
            Some(mask) => index < mask.len() && mask[index],
        }
    }

    /// Attempts to downcast this topology to a GridTopology
    fn as_grid_topology(&self) -> Result<&GridTopology, TopologyError> {
        self
            .as_any()
            .downcast_ref::<GridTopology>()
            .ok_or_else(|| TopologyError::Other("Expected a grid-based topology".to_string()))
    }

    fn as_any(&self) -> &dyn Any;

    // fn get_indices
}

// TODO: Uncomment
// #[cfg(test)]
// mod tests {
//     use crate::procedural_generation::debroglie_scuffed::topology::topology_array::{TopoArray, TopoArrayMut};
//     use super::*;
//
//     // Example implementation for testing
//     #[derive(Debug, Clone)]
//     struct SimpleGridTopology {
//         width: usize,
//         height: usize,
//         depth: usize,
//         mask: Option<Vec<bool>>,
//     }
//
//     impl SimpleGridTopology {
//         fn new(width: usize, height: usize, depth: usize) -> Self {
//             Self {
//                 width,
//                 height,
//                 depth,
//                 mask: None,
//             }
//         }
//
//         fn with_mask(width: usize, height: usize, depth: usize, mask: Vec<bool>) -> Result<Self, TopologyError> {
//             let expected = width * height * depth;
//             if mask.len() != expected {
//                 return Err(TopologyError::InvalidMaskLength {
//                     expected,
//                     actual: mask.len(),
//                 });
//             }
//             Ok(Self {
//                 width,
//                 height,
//                 depth,
//                 mask: Some(mask),
//             })
//         }
//     }
//
//     impl Topology for SimpleGridTopology {
//         fn index_count(&self) -> usize {
//             self.width * self.height * self.depth
//         }
//
//         fn directions_count(&self) -> usize {
//             4 // Simple 2D grid
//         }
//
//         fn width(&self) -> usize {
//             self.width
//         }
//
//         fn height(&self) -> usize {
//             self.height
//         }
//
//         fn depth(&self) -> usize {
//             self.depth
//         }
//
//         fn try_move_full(
//             &self,
//             index: usize,
//             direction: Direction,
//         ) -> Result<Option<(usize, Direction, EdgeLabel)>, TopologyError> {
//             if index >= self.index_count() {
//                 return Err(TopologyError::IndexOutOfBounds {
//                     index,
//                     max: self.index_count(),
//                 });
//             }
//
//             let (x, y, z) = self.get_coord(index)?;
//
//             // Simple movement logic (no wrapping)
//             let (new_x, new_y, new_z) = match direction {
//                 Direction::XPlus => (x + 1, y, z),
//                 Direction::XMinus => (x.saturating_sub(1), y, z),
//                 Direction::YPlus => (x, y + 1, z),
//                 Direction::YMinus => (x, y.saturating_sub(1), z),
//                 Direction::ZPlus => (x, y, z + 1),
//                 Direction::ZMinus => (x, y, z.saturating_sub(1)),
//             };
//
//             // Check bounds
//             if new_x >= self.width || new_y >= self.height || new_z >= self.depth {
//                 return Ok(None);
//             }
//
//             // Check if we actually moved (handles saturating_sub edge case)
//             if (new_x, new_y, new_z) == (x, y, z) {
//                 return Ok(None);
//             }
//
//             let dest_index = self.get_index(new_x, new_y, new_z)?;
//
//             // Check mask
//             if let Some(ref mask) = self.mask {
//                 if !mask[dest_index] {
//                     return Ok(None);
//                 }
//             }
//
//             // Simple inverse direction calculation
//             let inverse_direction = match direction {
//                 Direction::XPlus => Direction::XMinus,
//                 Direction::XMinus => Direction::XPlus,
//                 Direction::YPlus => Direction::YMinus,
//                 Direction::YMinus => Direction::YPlus,
//                 Direction::ZPlus => Direction::ZMinus,
//                 Direction::ZMinus => Direction::ZPlus,
//             };
//
//             let edge_label = match direction {
//                 Direction::XPlus => EdgeLabel::XPlus,
//                 Direction::XMinus => EdgeLabel::XMinus,
//                 Direction::YPlus => EdgeLabel::YPlus,
//                 Direction::YMinus => EdgeLabel::YMinus,
//                 Direction::ZPlus => EdgeLabel::ZPlus,
//                 Direction::ZMinus => EdgeLabel::ZMinus,
//             };
//
//             Ok(Some((dest_index, inverse_direction, edge_label)))
//         }
//
//         fn mask(&self) -> Option<Vec<bool>> {
//             self.mask.clone()
//         }
//
//         fn get_index(&self, x: usize, y: usize, z: usize) -> Result<usize, TopologyError> {
//             if x >= self.width || y >= self.height || z >= self.depth {
//                 return Err(TopologyError::CoordinateOutOfBounds { x, y, z });
//             }
//             Ok(x + y * self.width + z * self.width * self.height)
//         }
//
//         fn get_coord(&self, index: usize) -> Result<(usize, usize, usize), TopologyError> {
//             if index >= self.index_count() {
//                 return Err(TopologyError::IndexOutOfBounds {
//                     index,
//                     max: self.index_count(),
//                 });
//             }
//             let x = index % self.width;
//             let i = index / self.width;
//             let y = i % self.height;
//             let z = i / self.height;
//             Ok((x, y, z))
//         }
//
//         fn with_mask(&self, mask: Vec<bool>) -> Result<Self, TopologyError> {
//             SimpleGridTopology::with_mask(self.width, self.height, self.depth, mask)
//         }
//
//         fn as_any(&self) -> &dyn Any {
//             self
//         }
//     }
//
//     // Example TopoArray implementation
//     #[derive(Debug)]
//     struct SimpleTopoArray<T> {
//         topology: SimpleGridTopology,
//         data: Vec<T>,
//     }
//
//     impl<T> SimpleTopoArray<T> {
//         fn new(topology: SimpleGridTopology, data: Vec<T>) -> Self {
//             assert_eq!(topology.index_count(), data.len());
//             Self { topology, data }
//         }
//     }
//
//     impl<T> TopoArray<T, SimpleGridTopology> for SimpleTopoArray<T> {
//         fn topology(&self) -> &SimpleGridTopology {
//             &self.topology
//         }
//
//         fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
//             self.data.get(index).ok_or(TopologyError::IndexOutOfBounds {
//                 index,
//                 max: self.data.len(),
//             })
//         }
//     }
//
//     impl<T> TopoArrayMut<T, SimpleGridTopology> for SimpleTopoArray<T> {
//         fn set_index(&mut self, index: usize, value: T) -> Result<(), TopologyError> {
//             if index >= self.data.len() {
//                 return Err(TopologyError::IndexOutOfBounds {
//                     index,
//                     max: self.data.len(),
//                 });
//             }
//             self.data[index] = value;
//             Ok(())
//         }
//
//         fn get_mut_index(&mut self, index: usize) -> Result<&mut T, TopologyError> {
//             if index >= self.data.len() {
//                 return Err(TopologyError::IndexOutOfBounds {
//                     index,
//                     max: self.data.len(),
//                 });
//             }
//             Ok(&mut self.data[index])
//         }
//     }
//
//     #[test]
//     fn test_simple_topology() {
//         let topology = SimpleGridTopology::new(3, 3, 1);
//         assert_eq!(topology.index_count(), 9);
//         assert_eq!(topology.width(), 3);
//         assert_eq!(topology.height(), 3);
//         assert_eq!(topology.depth(), 1);
//
//         // Test coordinate conversion
//         assert_eq!(topology.get_index(1, 1, 0).unwrap(), 4); // Center of 3x3 grid
//         assert_eq!(topology.get_coord(4).unwrap(), (1, 1, 0));
//     }
//
//     #[test]
//     fn test_topology_movement() {
//         let topology = SimpleGridTopology::new(3, 3, 1);
//
//         // Test successful movement
//         let result = topology.try_move_full(4, Direction::XPlus).unwrap(); // Center right
//         assert!(result.is_some());
//         let (dest, inv_dir, edge_label) = result.unwrap();
//         assert_eq!(dest, 5);
//         assert_eq!(inv_dir, Direction::XMinus);
//         assert_eq!(edge_label, EdgeLabel::XPlus);
//
//         // Test boundary (should fail)
//         let result = topology.try_move(2, Direction::XPlus).unwrap(); // Right edge
//         assert!(result.is_none());
//     }
//
//     #[test]
//     fn test_topology_with_mask() {
//         let mask = vec![true, false, true, true, true, true, true, true, true];
//         let topology = SimpleGridTopology::with_mask(3, 3, 1, mask).unwrap();
//
//         // Movement to unmasked cell should succeed
//         let result = topology.try_move(0, Direction::XPlus).unwrap(); // (0,0) to (1,0)
//         assert!(result.is_none()); // Should fail because (1,0) is masked (index 1)
//
//         // Movement to unmasked cell should succeed
//         let result = topology.try_move(0, Direction::YPlus).unwrap(); // (0,0) to (0,1)
//         assert_eq!(result, Some(3)); // Should succeed because (0,1) is unmasked (index 3)
//     }
//
//     #[test]
//     fn test_topo_array() {
//         let topology = SimpleGridTopology::new(2, 2, 1);
//         let data = vec![1, 2, 3, 4];
//         let array = SimpleTopoArray::new(topology, data);
//
//         assert_eq!(array.len(), 4);
//         assert!(!array.is_empty());
//
//         // Test coordinate access
//         assert_eq!(*array.get_coord_2d(0, 0).unwrap(), 1);
//         assert_eq!(*array.get_coord_2d(1, 0).unwrap(), 2);
//         assert_eq!(*array.get_coord_2d(0, 1).unwrap(), 3);
//         assert_eq!(*array.get_coord_2d(1, 1).unwrap(), 4);
//
//         // Test index access
//         assert_eq!(*array.get_index(0).unwrap(), 1);
//         assert_eq!(*array.get_index(3).unwrap(), 4);
//     }
//
//     #[test]
//     fn test_mutable_topo_array() {
//         let topology = SimpleGridTopology::new(2, 2, 1);
//         let data = vec![1, 2, 3, 4];
//         let mut array = SimpleTopoArray::new(topology, data);
//
//         // Test mutable access
//         array.set_coord_2d(1, 1, 42).unwrap();
//         assert_eq!(*array.get_coord_2d(1, 1).unwrap(), 42);
//
//         // Test mutable reference
//         *array.get_mut_coord_2d(0, 0).unwrap() = 99;
//         assert_eq!(*array.get_coord_2d(0, 0).unwrap(), 99);
//     }
//
//     #[test]
//     fn test_convenience_methods() {
//         let topology = SimpleGridTopology::new(3, 3, 1);
//
//         // Test 2D convenience methods
//         assert_eq!(topology.get_index_2d(1, 1).unwrap(), 4);
//         assert_eq!(topology.get_coord_2d(4).unwrap(), (1, 1));
//
//         let result = topology.try_move_2d(1, 1, Direction::XPlus).unwrap();
//         assert_eq!(result, Some(5));
//
//         let result = topology.try_move_2d_to_coord(1, 1, Direction::YPlus).unwrap();
//         assert_eq!(result, Some((1, 2)));
//     }
// }