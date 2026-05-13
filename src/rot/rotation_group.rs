use std::borrow::BorrowMut;
use crate::rot::rotations::Rotation;

#[derive(Clone, Debug)]
pub struct RotationGroup {
    rotations: Vec<Rotation>,
    rotational_symmetry: i32,
    reflectional_symmetry: bool,
    smallest_angle: i32,
}

impl RotationGroup {
    pub fn default() -> Self {
        Self {
            rotations: Vec::new(),
            rotational_symmetry: 0,
            reflectional_symmetry: false,
            smallest_angle: 0,
        }
    }

    pub fn new(rotational_symmetry: i32, reflectional_symmetry: bool) -> Self {
        let smallest_angle = 360 / rotational_symmetry;
        let mut rotations = Vec::new();
        let max = if reflectional_symmetry { 2 } else { 1 };
        for refl in 0..max {
            for rot in (0..360).step_by(smallest_angle as usize) {
                rotations.push(Rotation::new(rot, refl > 0));
            }
        }

        Self {
            rotations,
            rotational_symmetry,
            reflectional_symmetry,
            smallest_angle,
        }
    }

    pub fn get_rotations(&self) -> &Vec<Rotation> {
        &self.rotations
    }

    pub fn get_rotations_mut(&mut self) -> &mut Vec<Rotation> {
        self.rotations.borrow_mut()
    }

    pub fn is_reflectionally_symmetric(&self) -> bool {
        self.reflectional_symmetry
    }

    pub fn get_smallest_angle(&self) -> i32 {
        self.smallest_angle
    }

    /// Checks if rotation is not a member of the group.
    pub fn check_contains(&self, rotation: &Rotation) -> bool {
        if rotation.get_rotate_cw() / self.smallest_angle * self.smallest_angle != rotation.get_rotate_cw() {
            return false;
        }
        if rotation.get_reflect_x() && !self.reflectional_symmetry {
            return false;
        }
        true
    }
}