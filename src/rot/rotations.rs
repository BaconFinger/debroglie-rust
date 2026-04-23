use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Mul;

#[derive(Copy, Clone, Debug, Eq)]
pub struct Rotation {
    /// Rotation in degrees, clockwise (assuming a y-down co-ordinate system, typically used
    /// for 2d graphics).
    rotate_cw: i32,


    /// If true, this "rotation" also includes a reflection along the x-axis.
    /// The reflection is applied before doing any rotation by RotateCw.
    reflect_x: bool,
}

impl Rotation {
    pub fn new(rotate_cw: i32, reflect_x: bool) -> Self {
        Self {
            rotate_cw,
            reflect_x,
        }
    }

    pub fn default() -> Self {
        Self::new(0, false)
    }

    /// True for the default constructed rotation, that doesn't do anything.
    pub fn is_identity(&self) -> bool {
        self.rotate_cw == 0 && !self.reflect_x
    }

    pub fn get_rotate_cw(&self) -> i32 {
        self.rotate_cw
    }

    pub fn get_reflect_x(&self) -> bool {
        self.reflect_x
    }

    /// Returns the rotation that rotates back from this one.
    /// i.e. r.inverse() * r gives the identity rotation for all Rotation objects.
    pub fn inverse(&self) -> Self {
        let rotate_cw = if self.reflect_x { self.rotate_cw } else { 360 - self.rotate_cw % 360 };
        Self {
            rotate_cw,
            reflect_x: self.reflect_x,
        }
    }

    pub fn get_hash_code(&self) -> i32 {
        let suffix = if self.reflect_x { 1 } else { 0 };
        let hash_val = self.rotate_cw * 2 + suffix;
        hash_val
    }
}

impl Mul for Rotation {
    type Output = Rotation;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut inner = self.rotate_cw;
        if rhs.reflect_x {
            inner = -self.rotate_cw;
        }
        let outer = (inner + rhs.rotate_cw + 360) % 360;
        Self::new(outer, self.reflect_x ^ rhs.reflect_x)
    }
}

impl PartialEq for Rotation {
    fn eq(&self, other: &Self) -> bool {
        self.rotate_cw == other.rotate_cw && self.reflect_x == other.reflect_x
    }
}

impl Hash for Rotation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash_val = self.get_hash_code();
        hash_val.hash(state);
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = if self.reflect_x { "x" } else { "" };
        write!(f, "!{}{}", inner, self.rotate_cw)
    }
}