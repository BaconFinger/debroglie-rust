use std::ops::Add;

/// Represents a location in a topology.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point<T> where T: Add<Output = T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point<T> where T: Add<Output = T> {
    pub fn new(x: T, y: T, z: T) -> Point<T> {
        Self{ x, y, z}
    }
}