use crate::topology::direction::{Direction, DirectionSet, EdgeLabel};
use crate::topology::topology::{Topology, TopologyError};
use std::any::Any;

/// A grid topology is a topology with a regular repeating pattern.
/// It supports more operations than a generic topology.
#[derive(Debug, Clone)]
pub struct GridTopology {
    directions: DirectionSet,
    width: usize,
    height: usize,
    depth: usize,
    periodic_x: bool,
    periodic_y: bool,
    periodic_z: bool,
    mask: Option<Vec<bool>>,
}

impl GridTopology {
    /// Constructs a 2d square grid topology of given dimensions and periodicity.
    pub fn new_2d(width: usize, height: usize, periodic: bool) -> Self {
        Self::new(
            DirectionSet::CARTESIAN_2D,
            width,
            height,
            1,
            periodic,
            periodic,
            periodic,
            None,
        )
    }

    /// Constructs a 3d cube grid topology of given dimensions and periodicity.
    pub fn new_3d(width: usize, height: usize, depth: usize, periodic: bool) -> Self {
        Self::new(
            DirectionSet::CARTESIAN_3D,
            width,
            height,
            depth,
            periodic,
            periodic,
            periodic,
            None,
        )
    }

    /// Constructs a 2d topology.
    pub fn new_2d_with_periodicity(
        directions: DirectionSet,
        width: usize,
        height: usize,
        periodic_x: bool,
        periodic_y: bool,
        mask: Option<Vec<bool>>,
    ) -> Result<Self, GridTopologyError> {
        Self::new_checked(
            directions, width, height, 1, periodic_x, periodic_y, false, mask,
        )
    }

    /// Constructs a topology.
    pub fn new(
        directions: DirectionSet,
        width: usize,
        height: usize,
        depth: usize,
        periodic_x: bool,
        periodic_y: bool,
        periodic_z: bool,
        mask: Option<Vec<bool>>,
    ) -> Self {
        // Unsafe version that doesn't validate mask length for backwards compatibility
        Self {
            directions,
            width,
            height,
            depth,
            periodic_x,
            periodic_y,
            periodic_z,
            mask,
        }
    }

    /// Constructs a topology with validation.
    pub fn new_checked(
        directions: DirectionSet,
        width: usize,
        height: usize,
        depth: usize,
        periodic_x: bool,
        periodic_y: bool,
        periodic_z: bool,
        mask: Option<Vec<bool>>,
    ) -> Result<Self, GridTopologyError> {
        let expected_size = width * height * depth;

        if let Some(ref mask) = mask {
            if mask.len() != expected_size {
                return Err(GridTopologyError::InvalidMaskLength {
                    expected: expected_size,
                    actual: mask.len(),
                });
            }
        }

        Ok(Self {
            directions,
            width,
            height,
            depth,
            periodic_x,
            periodic_y,
            periodic_z,
            mask,
        })
    }

    /// Returns a GridTopology with the same parameters, but with the specified mask
    pub fn with_mask(&self, mask: Vec<bool>) -> Result<Self, GridTopologyError> {
        let expected_size = self.width * self.height * self.depth;
        if mask.len() != expected_size {
            return Err(GridTopologyError::InvalidMaskLength {
                expected: expected_size,
                actual: mask.len(),
            });
        }

        Ok(Self {
            directions: self.directions.clone(),
            width: self.width,
            height: self.height,
            depth: self.depth,
            periodic_x: self.periodic_x,
            periodic_y: self.periodic_y,
            periodic_z: self.periodic_z,
            mask: Some(mask),
        })
    }

    /// Returns a GridTopology with the same parameters, with the dimensions overridden. Any mask is reset.
    pub fn with_size(&self, width: usize, height: usize, depth: usize) -> Self {
        Self {
            directions: self.directions.clone(),
            width,
            height,
            depth,
            periodic_x: self.periodic_x,
            periodic_y: self.periodic_y,
            periodic_z: self.periodic_z,
            mask: None, // Reset mask when changing size
        }
    }

    /// Returns a GridTopology with the same parameters, with the periodicity overridden.
    pub fn with_periodic(&self, periodic_x: bool, periodic_y: bool, periodic_z: bool) -> Self {
        Self {
            directions: self.directions.clone(),
            width: self.width,
            height: self.height,
            depth: self.depth,
            periodic_x,
            periodic_y,
            periodic_z,
            mask: self.mask.clone(),
        }
    }

    /// Characterizes the adjacency relationship between locations.
    pub fn directions(&self) -> &DirectionSet {
        &self.directions
    }

    /// Number of unique directions
    pub fn directions_count(&self) -> usize {
        self.directions.count()
    }

    /// The extent along the x-axis.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The extent along the y-axis.
    pub fn height(&self) -> usize {
        self.height
    }

    /// The extent along the z-axis.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Does the topology wrap on the x-axis.
    pub fn periodic_x(&self) -> bool {
        self.periodic_x
    }

    /// Does the topology wrap on the y-axis.
    pub fn periodic_y(&self) -> bool {
        self.periodic_y
    }

    /// Does the topology wrap on the z-axis.
    pub fn periodic_z(&self) -> bool {
        self.periodic_z
    }

    /// A array with one value per index indicating if the value is missing.
    /// Not all uses of Topology support masks.
    pub fn mask(&self) -> Option<&[bool]> {
        self.mask.as_deref()
    }

    /// Number of unique indices (distinct locations) in the topology
    pub fn index_count(&self) -> usize {
        self.width * self.height * self.depth
    }

    /// Checks if two grids are the same size without regard for masks or periodicity.
    pub fn is_same_size(&self, other: &GridTopology) -> bool {
        self.width == other.width && self.height == other.height && self.depth == other.depth
    }

    /// Reduces a three dimensional co-ordinate to a single integer. This is mostly used internally.
    pub fn get_index(&self, x: usize, y: usize, z: usize) -> Result<usize, GridTopologyError> {
        if x >= self.width || y >= self.height || z >= self.depth {
            return Err(GridTopologyError::CoordinateOutOfBounds {
                x,
                y,
                z,
                width: self.width,
                height: self.height,
                depth: self.depth,
            });
        }
        Ok(x + y * self.width + z * self.width * self.height)
    }

    /// Inverts get_index
    pub fn get_coord(&self, index: usize) -> Result<(usize, usize, usize), GridTopologyError> {
        let max_index = self.width * self.height * self.depth;
        if index >= max_index {
            return Err(GridTopologyError::IndexOutOfBounds {
                index,
                max: max_index,
            });
        }

        let x = index % self.width;
        let i = index / self.width;
        let y = i % self.height;
        let z = i / self.height;
        Ok((x, y, z))
    }

    /// Try to move from an index in a direction, returning destination, inverse direction, and edge label
    pub fn try_move_full(
        &self,
        index: usize,
        direction: Direction,
    ) -> Result<Option<(usize, Direction, EdgeLabel)>, GridTopologyError> {
        let (x, y, z) = self.get_coord(index)?;
        let inverse_direction = self.directions.inverse(direction);
        let edge_label = EdgeLabel::from(direction);

        if let Some(dest) = self.try_move_coord(x, y, z, direction)? {
            Ok(Some((dest, inverse_direction, edge_label)))
        } else {
            Ok(None)
        }
    }

    /// Try to move from an index in a direction, returning just the destination
    pub fn try_move(
        &self,
        index: usize,
        direction: Direction,
    ) -> Result<Option<usize>, GridTopologyError> {
        let (x, y, z) = self.get_coord(index)?;
        self.try_move_coord(x, y, z, direction)
    }

    /// Try to move from coordinates in a direction, returning destination index
    pub fn try_move_coord(
        &self,
        x: usize,
        y: usize,
        z: usize,
        direction: Direction,
    ) -> Result<Option<usize>, GridTopologyError> {
        if let Some((dest_x, dest_y, dest_z)) = self.try_move_coord_to_coord(x, y, z, direction)? {
            Ok(Some(self.get_index(dest_x, dest_y, dest_z)?))
        } else {
            Ok(None)
        }
    }

    /// Given a co-ordinate and a direction, gives the co-ordinate that is one step in that direction,
    /// if it exists and is not masked out. Otherwise, it returns None.
    pub fn try_move_coord_to_coord(
        &self,
        x: usize,
        y: usize,
        z: usize,
        direction: Direction,
    ) -> Result<Option<(usize, usize, usize)>, GridTopologyError> {
        let d = direction.as_index();
        if d >= self.directions.count() {
            return Err(GridTopologyError::DirectionOutOfBounds {
                direction: d,
                max: self.directions.count(),
            });
        }

        // Calculate new coordinates
        let mut new_x = x as i32 + self.directions.dx()[d];
        let mut new_y = y as i32 + self.directions.dy()[d];
        let mut new_z = z as i32 + self.directions.dz()[d];

        // Handle x-axis wrapping/bounds
        if self.periodic_x {
            if new_x < 0 {
                new_x += self.width as i32;
            }
            if new_x >= self.width as i32 {
                new_x -= self.width as i32;
            }
        } else if new_x < 0 || new_x >= self.width as i32 {
            return Ok(None);
        }

        // Handle y-axis wrapping/bounds
        if self.periodic_y {
            if new_y < 0 {
                new_y += self.height as i32;
            }
            if new_y >= self.height as i32 {
                new_y -= self.height as i32;
            }
        } else if new_y < 0 || new_y >= self.height as i32 {
            return Ok(None);
        }

        // Handle z-axis wrapping/bounds
        if self.periodic_z {
            if new_z < 0 {
                new_z += self.depth as i32;
            }
            if new_z >= self.depth as i32 {
                new_z -= self.depth as i32;
            }
        } else if new_z < 0 || new_z >= self.depth as i32 {
            return Ok(None);
        }

        let dest_x = new_x as usize;
        let dest_y = new_y as usize;
        let dest_z = new_z as usize;

        // Check mask if present
        if let Some(ref mask) = self.mask {
            let index = self.get_index(dest_x, dest_y, dest_z)?;
            if !mask[index] {
                return Ok(None);
            }
        }

        Ok(Some((dest_x, dest_y, dest_z)))
    }

    /// Check if a coordinate is within bounds
    pub fn is_valid_coord(&self, x: usize, y: usize, z: usize) -> bool {
        x < self.width && y < self.height && z < self.depth
    }

    /// Check if an index is within bounds
    pub fn is_valid_index(&self, index: usize) -> bool {
        index < self.index_count()
    }

    /// Check if a position is masked out
    pub fn is_masked(&self, index: usize) -> Result<bool, GridTopologyError> {
        if !self.is_valid_index(index) {
            return Err(GridTopologyError::IndexOutOfBounds {
                index,
                max: self.index_count(),
            });
        }

        if let Some(ref mask) = self.mask {
            Ok(!mask[index])
        } else {
            Ok(false) // No mask means not masked
        }
    }
}

impl Topology for GridTopology {
    fn index_count(&self) -> usize {
        self.width * self.height * self.depth
    }

    fn directions_count(&self) -> usize {
        4 // For 2D grid
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn depth(&self) -> usize {
        self.depth
    }

    fn get_coord(&self, index: usize) -> Result<(usize, usize, usize), TopologyError> {
        if index >= self.index_count() {
            return Err(TopologyError::IndexOutOfBounds {
                index,
                max: self.index_count(),
            });
        }

        let x = index % self.width;
        let i = index / self.width;
        let y = i % self.height;
        let z = i / self.height;
        Ok((x, y, z))
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> Result<usize, TopologyError> {
        if x >= self.width || y >= self.height || z >= self.depth {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z });
        }
        Ok(x + y * self.width + z * self.width * self.height)
    }

    fn try_move_full(
        &self,
        index: usize,
        direction: Direction,
    ) -> Result<Option<(usize, Direction, EdgeLabel)>, TopologyError> {
        let result = self.get_coord(index);
        if result.is_err() {
            return Err(TopologyError::TraitError(result.unwrap_err().into()));
        }
        let (x, y, z) = result.unwrap(); // OK because already checked for error
        let inverse_direction = self.directions.inverse(direction);
        let edge_label = EdgeLabel::from(direction);

        let result = self.try_move_coord(x, y, z, direction);
        if result.is_err() {
            return Err(TopologyError::TraitError(result.unwrap_err().into()));
        }
        if let Some(dest) = result.unwrap() {
            // OK because already checked for error
            Ok(Some((dest, inverse_direction, edge_label)))
        } else {
            Ok(None)
        }
    }

    fn mask(&self) -> Option<Vec<bool>> {
        return self.mask.clone();
    }

    fn with_mask(&self, mask: Vec<bool>) -> Result<Self, TopologyError>
    where
        Self: Sized,
    {
        unimplemented!();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Error types for GridTopology operations
#[derive(Debug, thiserror::Error)]
pub enum GridTopologyError {
    #[error("Mask size doesn't fit the topology: expected {expected}, got {actual}")]
    InvalidMaskLength {
        expected: usize,
        actual: usize,
    },

    #[error("Index {index} is out of bounds (max: {max})")]
    IndexOutOfBounds {
        index: usize,
        max: usize,
    },

    #[error("Coordinate ({x}, {y}, {z}) is out of bounds for grid ({width}×{height}×{depth})")]
    CoordinateOutOfBounds {
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        depth: usize,
    },

    #[error("Direction {direction} is out of bounds (max: {max})")]
    DirectionOutOfBounds {
        direction: usize,
        max: usize,
    },

    #[error(transparent)]
    TopologyError(#[from] TopologyError),
}

// TODO: Uncomment
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_edge_label_conversion() {
//         assert_eq!(crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::from(crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XPlus), crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::XPlus);
//         assert_eq!(crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::from(crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YMinus), crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::YMinus);
//         assert_eq!(crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::from(crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::ZPlus), crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::ZPlus);
//     }
//
//     #[test]
//     fn test_2d_grid_creation() {
//         let grid = GridTopology::new_2d(5, 4, false);
//         assert_eq!(grid.width(), 5);
//         assert_eq!(grid.height(), 4);
//         assert_eq!(grid.depth(), 1);
//         assert_eq!(grid.index_count(), 20);
//         assert!(!grid.periodic_x());
//         assert!(!grid.periodic_y());
//         assert!(!grid.periodic_z());
//     }
//
//     #[test]
//     fn test_3d_grid_creation() {
//         let grid = GridTopology::new_3d(3, 4, 2, true);
//         assert_eq!(grid.width(), 3);
//         assert_eq!(grid.height(), 4);
//         assert_eq!(grid.depth(), 2);
//         assert_eq!(grid.index_count(), 24);
//         assert!(grid.periodic_x());
//         assert!(grid.periodic_y());
//         assert!(grid.periodic_z());
//     }
//
//     #[test]
//     fn test_coordinate_conversion() {
//         let grid = GridTopology::new_3d(3, 4, 2, false);
//
//         // Test get_index
//         assert_eq!(grid.get_index(0, 0, 0).unwrap(), 0);
//         assert_eq!(grid.get_index(1, 0, 0).unwrap(), 1);
//         assert_eq!(grid.get_index(0, 1, 0).unwrap(), 3);
//         assert_eq!(grid.get_index(0, 0, 1).unwrap(), 12);
//         assert_eq!(grid.get_index(2, 3, 1).unwrap(), 23);
//
//         // Test get_coord
//         assert_eq!(grid.get_coord(0).unwrap(), (0, 0, 0));
//         assert_eq!(grid.get_coord(1).unwrap(), (1, 0, 0));
//         assert_eq!(grid.get_coord(3).unwrap(), (0, 1, 0));
//         assert_eq!(grid.get_coord(12).unwrap(), (0, 0, 1));
//         assert_eq!(grid.get_coord(23).unwrap(), (2, 3, 1));
//     }
//
//     #[test]
//     fn test_movement_non_periodic() {
//         let grid = GridTopology::new_2d(3, 3, false);
//
//         // Test normal movement
//         assert_eq!(grid.try_move(4, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XPlus).unwrap(), Some(5)); // Center to right
//         assert_eq!(grid.try_move(4, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XMinus).unwrap(), Some(3)); // Center to left
//         assert_eq!(grid.try_move(4, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YPlus).unwrap(), Some(7)); // Center down
//         assert_eq!(grid.try_move(4, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YMinus).unwrap(), Some(1)); // Center up
//
//         // Test boundary conditions (should fail)
//         assert_eq!(grid.try_move(0, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XMinus).unwrap(), None); // Left edge
//         assert_eq!(grid.try_move(2, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XPlus).unwrap(), None); // Right edge
//         assert_eq!(grid.try_move(0, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YMinus).unwrap(), None); // Top edge
//         assert_eq!(grid.try_move(6, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YPlus).unwrap(), None); // Bottom edge
//     }
//
//     #[test]
//     fn test_movement_periodic() {
//         let grid = GridTopology::new_2d(3, 3, true);
//
//         // Test wrapping movement
//         assert_eq!(grid.try_move(0, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XMinus).unwrap(), Some(2)); // Left edge wraps to right
//         assert_eq!(grid.try_move(2, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XPlus).unwrap(), Some(0)); // Right edge wraps to left
//         assert_eq!(grid.try_move(0, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YMinus).unwrap(), Some(6)); // Top edge wraps to bottom
//         assert_eq!(grid.try_move(6, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::YPlus).unwrap(), Some(0)); // Bottom edge wraps to top
//     }
//
//     #[test]
//     fn test_try_move_full() {
//         let grid = GridTopology::new_2d(3, 3, false);
//
//         let result = grid.try_move_full(4, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XPlus).unwrap();
//         assert!(result.is_some());
//         let (dest, inv_dir, edge_label) = result.unwrap();
//         assert_eq!(dest, 5);
//         assert_eq!(inv_dir, crate::procedural_generation::debroglie_scuffed::topology::direction::Direction::XMinus);
//         assert_eq!(edge_label, crate::procedural_generation::debroglie_scuffed::topology::direction::EdgeLabel::XPlus); // Now this works correctly
//     }
//
//     // Additional tests remain the same...
//     #[test]
//     fn test_with_mask() {
//         let grid = GridTopology::new_2d(2, 2, false);
//         let mask = vec![true, false, true, false];
//
//         let masked_grid = grid.with_mask(mask.clone()).unwrap();
//         assert_eq!(masked_grid.mask(), Some(mask.as_slice()));
//
//         // Test wrong mask size
//         let wrong_mask = vec![true, false]; // Too short
//         assert!(grid.with_mask(wrong_mask).is_err());
//     }
// }
