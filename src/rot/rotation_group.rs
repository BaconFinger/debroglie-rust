use crate::rot::rotations::Rotation;

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
}