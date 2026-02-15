use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
    /// The "third" axis used for DirectionSet.Hexagonal2d
    /// it's redundant with X and Y, but still useful to refer to.
    W,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    XPlus = 0,
    XMinus = 1,
    YPlus = 2,
    YMinus = 3,
    ZPlus = 4,
    ZMinus = 5,
    // Note: WPlus and WMinus are aliases that map to the same values as ZPlus/ZMinus
    // They're handled separately in the DirectionSet implementations
}

impl Direction {
    /// Convert direction to its index value
    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Create direction from index, returns None if invalid
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Direction::XPlus),
            1 => Some(Direction::XMinus),
            2 => Some(Direction::YPlus),
            3 => Some(Direction::YMinus),
            4 => Some(Direction::ZPlus),
            5 => Some(Direction::ZMinus),
            _ => None,
        }
    }

    /// Check if this direction represents a W axis direction (for hexagonal 3D)
    pub fn is_w_direction(self) -> bool {
        matches!(self, Direction::ZPlus | Direction::ZMinus)
    }

    /// Get the W direction equivalent (WPlus = ZPlus, WMinus = ZMinus)
    pub fn as_w_direction(self) -> Option<Self> {
        match self {
            Direction::ZPlus => Some(Direction::ZPlus),  // This represents WPlus
            Direction::ZMinus => Some(Direction::ZMinus), // This represents WMinus
            _ => None,
        }
    }
}

/// DirectionSetType indicates what neighbors are considered adjacent to each tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionSetType {
    Unknown,
    Cartesian2d,
    Hexagonal2d,
    Cartesian3d,
    Hexagonal3d,
}

// EdgeLabel should mirror the Direction values for C# compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeLabel {
    XPlus = 0,
    XMinus = 1,
    YPlus = 2,
    YMinus = 3,
    ZPlus = 4,
    ZMinus = 5,
}

impl Default for EdgeLabel {
    fn default() -> Self {
        EdgeLabel::XPlus
    }
}

// Direct cast from Direction to EdgeLabel (matching C# behavior)
impl From<Direction> for EdgeLabel {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::XPlus => EdgeLabel::XPlus,
            Direction::XMinus => EdgeLabel::XMinus,
            Direction::YPlus => EdgeLabel::YPlus,
            Direction::YMinus => EdgeLabel::YMinus,
            Direction::ZPlus => EdgeLabel::ZPlus,
            Direction::ZMinus => EdgeLabel::ZMinus,
        }
    }
}

/// Error type for direction operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectionError {
    NoDirectionFound { x: i32, y: i32, z: i32 },
}

impl fmt::Display for DirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectionError::NoDirectionFound { x, y, z } => {
                write!(f, "No direction corresponds to ({}, {}, {})", x, y, z)
            }
        }
    }
}

impl std::error::Error for DirectionError {}

/// Wrapper around DirectionSetType supplying some convenience data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectionSet {
    dx: &'static [i32],
    dy: &'static [i32],
    dz: &'static [i32],
    count: usize,
    direction_type: DirectionSetType,
}

impl DirectionSet {
    /// The Directions associated with square grids.
    pub const CARTESIAN_2D: DirectionSet = DirectionSet {
        dx: &[1, -1, 0, 0],
        dy: &[0, 0, 1, -1],
        dz: &[0, 0, 0, 0],
        count: 4,
        direction_type: DirectionSetType::Cartesian2d,
    };

    /// The Directions associated with hexagonal grids.
    /// Conventionally, x is treated as moving right, and y as moving down and left,
    /// But the same Directions object will work just as well with several other conventions
    /// as long as you are consistent.
    pub const HEXAGONAL_2D: DirectionSet = DirectionSet {
        dx: &[1, -1, 0, 0, 1, -1],
        dy: &[0, 0, 1, -1, 1, -1],
        dz: &[0, 0, 0, 0, 0, 0],
        count: 6,
        direction_type: DirectionSetType::Hexagonal2d,
    };

    /// The Directions associated with grids of hexagon prisms.
    /// x is right, and z as moving down and left, y is up (prism axis), and w is the same as one unity of x and z.
    /// Note: Due to enum limitations, WPlus/WMinus are represented by indices 6/7 in this set,
    /// but map to the same Direction enum values as ZPlus/ZMinus.
    pub const HEXAGONAL_3D: DirectionSet = DirectionSet {
        //            X+  X-  Y+  Y-  Z+  Z-  W+  W-
        dx: &[1, -1, 0, 0, 0, 0, 1, -1],
        dy: &[0, 0, 1, -1, 0, 0, 0, 0],
        dz: &[0, 0, 0, 0, 1, -1, 1, -1],
        count: 8,
        direction_type: DirectionSetType::Hexagonal3d,
    };

    /// The Directions associated with cubic grids.
    pub const CARTESIAN_3D: DirectionSet = DirectionSet {
        dx: &[1, -1, 0, 0, 0, 0],
        dy: &[0, 0, 1, -1, 0, 0],
        dz: &[0, 0, 0, 0, 1, -1],
        count: 6,
        direction_type: DirectionSetType::Cartesian3d,
    };

    /// Create a new DirectionSet with the given parameters
    pub fn new(
        dx: &'static [i32],
        dy: &'static [i32],
        dz: &'static [i32],
        direction_type: DirectionSetType,
    ) -> Self {
        let count = dx.len();
        assert_eq!(count, dy.len(), "dx and dy must have the same length");
        assert_eq!(count, dz.len(), "dx and dz must have the same length");

        Self {
            dx,
            dy,
            dz,
            count,
            direction_type,
        }
    }

    /// Get the delta-x values
    pub fn dx(&self) -> &[i32] {
        self.dx
    }

    /// Get the delta-y values
    pub fn dy(&self) -> &[i32] {
        self.dy
    }

    /// Get the delta-z values
    pub fn dz(&self) -> &[i32] {
        self.dz
    }

    /// Get the number of directions
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get the direction set type
    pub fn direction_type(&self) -> DirectionSetType {
        self.direction_type
    }

    /// Given a direction, returns the direction that makes the reverse movement.
    pub fn inverse(&self, d: Direction) -> Direction {
        let index = d.as_index();
        let inverse_index = index ^ 1;
        Direction::from_index(inverse_index).expect("Invalid direction for inverse")
    }

    /// Find the direction corresponding to the given coordinate deltas
    pub fn get_direction(&self, x: i32, y: i32, z: i32) -> Result<Direction, DirectionError> {
        for d in 0..self.count {
            if x == self.dx[d] && y == self.dy[d] && z == self.dz[d] {
                // Handle the special case for hexagonal 3D where indices 6 and 7
                // represent WPlus and WMinus but map to ZPlus and ZMinus enum values
                if self.direction_type == DirectionSetType::Hexagonal3d && d >= 6 {
                    return Ok(if d == 6 { Direction::ZPlus } else { Direction::ZMinus });
                }
                return Direction::from_index(d).ok_or(DirectionError::NoDirectionFound { x, y, z });
            }
        }
        Err(DirectionError::NoDirectionFound { x, y, z })
    }

    /// Find the direction corresponding to 2D coordinate deltas (z=0)
    pub fn get_direction_2d(&self, x: i32, y: i32) -> Result<Direction, DirectionError> {
        self.get_direction(x, y, 0)
    }

    /// Get the coordinate deltas for a given direction
    pub fn get_deltas(&self, d: Direction) -> (i32, i32, i32) {
        let index = d.as_index();
        if index < self.count {
            (self.dx[index], self.dy[index], self.dz[index])
        } else {
            panic!("Direction index {} out of bounds for DirectionSet with {} directions", index, self.count);
        }
    }

    /// Get 2D coordinate deltas for a given direction
    pub fn get_deltas_2d(&self, d: Direction) -> (i32, i32) {
        let (dx, dy, _) = self.get_deltas(d);
        (dx, dy)
    }

    /// Returns an iterator over all valid directions for this set
    pub fn iter(&self) -> DirectionSetIterator {
        DirectionSetIterator {
            direction_set: self,
            current: 0,
        }
    }

    /// Check if a direction is valid for this direction set
    pub fn is_valid_direction(&self, d: Direction) -> bool {
        let index = d.as_index();
        if index < self.count {
            true
        } else if self.direction_type == DirectionSetType::Hexagonal3d {
            // For hexagonal 3D, ZPlus and ZMinus can also represent WPlus and WMinus
            matches!(d, Direction::ZPlus | Direction::ZMinus)
        } else {
            false
        }
    }

    pub fn get_direction_type(&self) -> DirectionSetType {
        self.direction_type
    }

    /// Get direction by index, handling the special hexagonal 3D case
    pub fn get_direction_by_index(&self, index: usize) -> Option<Direction> {
        if index < 6 {
            Direction::from_index(index)
        } else if self.direction_type == DirectionSetType::Hexagonal3d && index < self.count {
            // Indices 6 and 7 in hexagonal 3D represent WPlus and WMinus
            if index == 6 {
                Some(Direction::ZPlus)  // Represents WPlus
            } else {
                Some(Direction::ZMinus) // Represents WMinus
            }
        } else {
            None
        }
    }
}

/// Iterator for DirectionSet
pub struct DirectionSetIterator<'a> {
    direction_set: &'a DirectionSet,
    current: usize,
}

impl<'a> Iterator for DirectionSetIterator<'a> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.direction_set.count {
            let direction = self.direction_set.get_direction_by_index(self.current);
            self.current += 1;
            direction
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.direction_set.count - self.current;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DirectionSetIterator<'a> {}

impl<'a> IntoIterator for &'a DirectionSet {
    type Item = Direction;
    type IntoIter = DirectionSetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DirectionSetIterator {
            direction_set: self,
            current: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_index_conversion() {
        assert_eq!(Direction::XPlus.as_index(), 0);
        assert_eq!(Direction::XMinus.as_index(), 1);
        assert_eq!(Direction::YPlus.as_index(), 2);
        assert_eq!(Direction::YMinus.as_index(), 3);
        assert_eq!(Direction::ZPlus.as_index(), 4);
        assert_eq!(Direction::ZMinus.as_index(), 5);

        assert_eq!(Direction::from_index(0), Some(Direction::XPlus));
        assert_eq!(Direction::from_index(1), Some(Direction::XMinus));
        assert_eq!(Direction::from_index(6), None);
    }

    #[test]
    fn test_cartesian_2d() {
        let ds = DirectionSet::CARTESIAN_2D;
        assert_eq!(ds.count(), 4);
        assert_eq!(ds.direction_type(), DirectionSetType::Cartesian2d);

        assert_eq!(ds.dx(), &[1, -1, 0, 0]);
        assert_eq!(ds.dy(), &[0, 0, 1, -1]);
        assert_eq!(ds.dz(), &[0, 0, 0, 0]);
    }

    #[test]
    fn test_hexagonal_2d() {
        let ds = DirectionSet::HEXAGONAL_2D;
        assert_eq!(ds.count(), 6);
        assert_eq!(ds.direction_type(), DirectionSetType::Hexagonal2d);

        assert_eq!(ds.dx(), &[1, -1, 0, 0, 1, -1]);
        assert_eq!(ds.dy(), &[0, 0, 1, -1, 1, -1]);
    }

    #[test]
    fn test_cartesian_3d() {
        let ds = DirectionSet::CARTESIAN_3D;
        assert_eq!(ds.count(), 6);
        assert_eq!(ds.direction_type(), DirectionSetType::Cartesian3d);

        assert_eq!(ds.dx(), &[1, -1, 0, 0, 0, 0]);
        assert_eq!(ds.dy(), &[0, 0, 1, -1, 0, 0]);
        assert_eq!(ds.dz(), &[0, 0, 0, 0, 1, -1]);
    }

    #[test]
    fn test_hexagonal_3d() {
        let ds = DirectionSet::HEXAGONAL_3D;
        assert_eq!(ds.count(), 8);
        assert_eq!(ds.direction_type(), DirectionSetType::Hexagonal3d);

        // Test that it has the expected deltas
        assert_eq!(ds.dx(), &[1, -1, 0, 0, 0, 0, 1, -1]);
        assert_eq!(ds.dy(), &[0, 0, 1, -1, 0, 0, 0, 0]);
        assert_eq!(ds.dz(), &[0, 0, 0, 0, 1, -1, 1, -1]);
    }

    #[test]
    fn test_inverse() {
        let ds = DirectionSet::CARTESIAN_2D;
        assert_eq!(ds.inverse(Direction::XPlus), Direction::XMinus);
        assert_eq!(ds.inverse(Direction::XMinus), Direction::XPlus);
        assert_eq!(ds.inverse(Direction::YPlus), Direction::YMinus);
        assert_eq!(ds.inverse(Direction::YMinus), Direction::YPlus);
    }

    #[test]
    fn test_get_direction() {
        let ds = DirectionSet::CARTESIAN_2D;

        assert_eq!(ds.get_direction(1, 0, 0).unwrap(), Direction::XPlus);
        assert_eq!(ds.get_direction(-1, 0, 0).unwrap(), Direction::XMinus);
        assert_eq!(ds.get_direction(0, 1, 0).unwrap(), Direction::YPlus);
        assert_eq!(ds.get_direction(0, -1, 0).unwrap(), Direction::YMinus);

        // Test 2D convenience method
        assert_eq!(ds.get_direction_2d(1, 0).unwrap(), Direction::XPlus);
        assert_eq!(ds.get_direction_2d(0, 1).unwrap(), Direction::YPlus);
    }

    #[test]
    fn test_get_direction_hexagonal_3d_w_directions() {
        let ds = DirectionSet::HEXAGONAL_3D;

        // Test W directions (which map to Z directions in the enum)
        // Index 6 represents WPlus but returns ZPlus
        assert_eq!(ds.get_direction(1, 0, 1).unwrap(), Direction::ZPlus);
        // Index 7 represents WMinus but returns ZMinus
        assert_eq!(ds.get_direction(-1, 0, -1).unwrap(), Direction::ZMinus);
    }

    #[test]
    fn test_get_direction_not_found() {
        let ds = DirectionSet::CARTESIAN_2D;
        let result = ds.get_direction(2, 2, 0);
        assert!(result.is_err());

        match result {
            Err(DirectionError::NoDirectionFound { x, y, z }) => {
                assert_eq!(x, 2);
                assert_eq!(y, 2);
                assert_eq!(z, 0);
            }
            _ => panic!("Expected NoDirectionFound error"),
        }
    }

    #[test]
    fn test_get_deltas() {
        let ds = DirectionSet::CARTESIAN_2D;

        assert_eq!(ds.get_deltas(Direction::XPlus), (1, 0, 0));
        assert_eq!(ds.get_deltas(Direction::XMinus), (-1, 0, 0));
        assert_eq!(ds.get_deltas(Direction::YPlus), (0, 1, 0));
        assert_eq!(ds.get_deltas(Direction::YMinus), (0, -1, 0));

        assert_eq!(ds.get_deltas_2d(Direction::XPlus), (1, 0));
        assert_eq!(ds.get_deltas_2d(Direction::YPlus), (0, 1));
    }

    #[test]
    fn test_iterator() {
        let ds = DirectionSet::CARTESIAN_2D;
        let directions: Vec<_> = ds.iter().collect();

        assert_eq!(directions.len(), 4);
        assert_eq!(directions[0], Direction::XPlus);
        assert_eq!(directions[1], Direction::XMinus);
        assert_eq!(directions[2], Direction::YPlus);
        assert_eq!(directions[3], Direction::YMinus);
    }

    #[test]
    fn test_hexagonal_3d_iterator() {
        let ds = DirectionSet::HEXAGONAL_3D;
        let directions: Vec<_> = ds.iter().collect();

        assert_eq!(directions.len(), 8);
        // First 6 should be standard directions
        assert_eq!(directions[0], Direction::XPlus);
        assert_eq!(directions[1], Direction::XMinus);
        assert_eq!(directions[2], Direction::YPlus);
        assert_eq!(directions[3], Direction::YMinus);
        assert_eq!(directions[4], Direction::ZPlus);
        assert_eq!(directions[5], Direction::ZMinus);
        // Last 2 represent W directions but use Z enum values
        assert_eq!(directions[6], Direction::ZPlus);  // WPlus
        assert_eq!(directions[7], Direction::ZMinus); // WMinus
    }

    #[test]
    fn test_into_iterator() {
        let ds = DirectionSet::CARTESIAN_2D;
        let mut count = 0;

        for direction in &ds {
            count += 1;
            assert!(ds.is_valid_direction(direction));
        }

        assert_eq!(count, 4);
    }

    #[test]
    fn test_is_valid_direction() {
        let ds = DirectionSet::CARTESIAN_2D;

        assert!(ds.is_valid_direction(Direction::XPlus));
        assert!(ds.is_valid_direction(Direction::XMinus));
        assert!(ds.is_valid_direction(Direction::YPlus));
        assert!(ds.is_valid_direction(Direction::YMinus));
        assert!(!ds.is_valid_direction(Direction::ZPlus));
        assert!(!ds.is_valid_direction(Direction::ZMinus));
    }

    #[test]
    fn test_is_valid_direction_hexagonal_3d() {
        let ds = DirectionSet::HEXAGONAL_3D;

        // All directions should be valid in hexagonal 3D
        assert!(ds.is_valid_direction(Direction::XPlus));
        assert!(ds.is_valid_direction(Direction::XMinus));
        assert!(ds.is_valid_direction(Direction::YPlus));
        assert!(ds.is_valid_direction(Direction::YMinus));
        assert!(ds.is_valid_direction(Direction::ZPlus));  // Also represents WPlus
        assert!(ds.is_valid_direction(Direction::ZMinus)); // Also represents WMinus
    }

    #[test]
    fn test_w_direction_helpers() {
        assert!(Direction::ZPlus.is_w_direction());
        assert!(Direction::ZMinus.is_w_direction());
        assert!(!Direction::XPlus.is_w_direction());

        assert_eq!(Direction::ZPlus.as_w_direction(), Some(Direction::ZPlus));
        assert_eq!(Direction::ZMinus.as_w_direction(), Some(Direction::ZMinus));
        assert_eq!(Direction::XPlus.as_w_direction(), None);
    }
}