use std::collections::HashMap;
use crate::rot::rotation_group::RotationGroup;
use crate::rot::rotations::Rotation;
use crate::rot::tile_rotation::{TileRotation, TileRotationTreatment};
use crate::rot::tile_symmetry::TileSymmetry;
use crate::tile::{Tile, TileId};

#[derive(Debug, Clone, thiserror::Error)]
pub enum TileRotationBuilderError {
    #[error("Tile {tile_id} is not in the rotation group")]
    InvalidRotation { tile_id: usize},

    #[error("Missing tile {tile_id} in subgroup {subgroup}")]
    NoTileInSubgroup { tile_id: usize, subgroup: String },

    #[error("Failed to ensure tile exists in subgroup")]
    FailedToEnsureTile,

    #[error("Cannot {action}: conflict between {first} and {second}")]
    SetTileConflict {action: String, first: usize, second: usize},

    #[error("{msg}")]
    Todo {msg: String},
}

/// Builds a TileRotation.
/// This class lets you specify some transformations between tiles via rotation and reflection.
/// It then infers the full set of rotations possible and informs you if there are contradictions.
///
/// As an example of inference, if a square tile 1 transforms to tile 2 when rotated clockwise,
/// and tile 2 transforms to itself when reflected in the x-axis,
/// then we can infer that tile 1 must transform to tile 1 when reflected in the y-axis.
pub struct TileRotationBuilder {
    tile_to_subgroup: HashMap<Tile, SubGroup>,
    rotation_group: RotationGroup,
    default_treatment: TileRotationTreatment,
}

impl TileRotationBuilder {

    /// default default_treatment is Unchanged
    pub fn new(rotational_symmetry: i32, reflectional_symmetry: bool, default_treatment: TileRotationTreatment) -> Self {
        let rotation_group = RotationGroup::new(rotational_symmetry, reflectional_symmetry);
        Self {
            tile_to_subgroup: HashMap::new(),
            rotation_group,
            default_treatment,
        }
    }

    /// Indicates that if you reflect then rotate clockwise the src tile as indicated, then you get the dest tile.
    pub fn add(&mut self, src: Tile, rotation: Rotation, dest: Tile) -> Result<(), TileRotationBuilderError> {
        if !self.rotation_group.check_contains(&rotation) {
            return Err(TileRotationBuilderError::InvalidRotation { tile_id: src.get_id().0 });
        }

        // TODO? Consider Rc<SubGroup> for performance, or use arena allocation with refs.
        // Refs should be fine since SubGroups don't escape this class.

        let mut src_subgroup_destinations: Vec<Tile> = Vec::new();

        // These copies will replace the originals later
        let mut src_subgroup = self.get_group(src.clone()).clone();
        let mut dest_subgroup = self.get_group(dest.clone()).clone();

        // Groups need merging
        if src_subgroup != dest_subgroup {
            let src_rotations = src_subgroup.get_rotations(src.clone());
            if src_rotations.first().is_none() {
                return Err(TileRotationBuilderError::NoTileInSubgroup { tile_id: src.get_id().0, subgroup: "src_subgroup".to_string() });
            }
            let src_rotation = src_rotations.first().unwrap();

            let dest_rotations = dest_subgroup.get_rotations(dest.clone());
            if dest_rotations.first().is_none() {
                return Err(TileRotationBuilderError::NoTileInSubgroup { tile_id: dest.get_id().0, subgroup: "dest_subgroup".to_string() });
            }
            let dest_rotation = dest_rotations.first().unwrap();

            // Arrange dest_rotation so that it is relatively rotated
            // to src_rotation as specified by r.
            dest_subgroup.permute(|rot| dest_rotation.inverse() * src_rotation.clone() * rotation * rot);

            // Attempt to copy over tiles
            src_subgroup.entries.extend(dest_subgroup.entries.clone());

            for (k, v) in &dest_subgroup.tiles {
                Self::set(&mut src_subgroup, k.clone(), v.clone(), "record rotation from src to dest by rotation")?;
                src_subgroup_destinations.push(v.clone());
                // self.tile_to_subgroup.insert(v.clone(), src_subgroup.clone());
            }
        }

        src_subgroup.entries.push(Entry { src: src.clone(), rotation: rotation.clone(), dest: dest.clone() });
        Self::expand(&mut src_subgroup)?;

        // Now update self with the modified subgroups
        for dest_tile in &src_subgroup_destinations {
            self.tile_to_subgroup.insert(dest_tile.clone(), src_subgroup.clone());
        }
        self.tile_to_subgroup.insert(src.clone(), src_subgroup);
        Ok(())
    }

    /// Declares that a tile is symetric, and therefore transforms to iteself.
    /// This is a shorthand for calling Add(tile,..., tile) for specific rotations.
    pub fn add_symmetry(&mut self, tile: Tile, ts: TileSymmetry) -> Result<(), TileRotationBuilderError> {
        // The subgroups in the order are found here:
        // https://groupprops.subwiki.org/wiki/Subgroup_structure_of_dihedral_group:D8

        match ts
        {
            TileSymmetry::F => {
                self.get_group(tile.clone());
            }
            TileSymmetry::N => {
                self.add(tile.clone(), Rotation::new(0 * 90, false), tile.clone())?;
            }
            TileSymmetry::T => {
                self.add(tile.clone(), Rotation::new(0 * 90, true), tile.clone())?;
            }
            TileSymmetry::L => {
                self.add(tile.clone(), Rotation::new(1 * 90, true), tile.clone())?;
            }
            TileSymmetry::E => {
                self.add(tile.clone(), Rotation::new(2 * 90, true), tile.clone())?;
            }
            TileSymmetry::Q => {
                self.add(tile.clone(), Rotation::new(3 * 90, true), tile.clone())?;
            }
            TileSymmetry::I => {
                self.add(tile.clone(), Rotation::new(0 * 90, true), tile.clone())?;
                self.add(tile.clone(), Rotation::new(2 * 90, false), tile.clone())?;
            }
            TileSymmetry::Slash => {
                self.add(tile.clone(), Rotation::new(1 * 90, true), tile.clone())?;
                self.add(tile.clone(), Rotation::new(2 * 90, false), tile.clone())?;
            }
            TileSymmetry::Cyclic => {
                self.add(tile.clone(), Rotation::new(1 * 90, false), tile.clone())?;
            }
            TileSymmetry::X => {

                self.add(tile.clone(), Rotation::new(0 * 90, true), tile.clone())?;
                self.add(tile.clone(), Rotation::new(1 * 90, false), tile.clone())?;
            }
        }
        Ok(())
    }

    /// Extracts the full set of rotations
    pub fn build(&mut self) -> Result<TileRotation, TileRotationBuilderError> {
        let mut rotations: HashMap<Tile, HashMap<Rotation, Tile>> = HashMap::new();

        // TODO: Make safe. This will require comprehensive test coverage to ensure the safe version still works.
        // Should be able to make it safe by swapping out tile_to_subgroup then iterating on that
        unsafe {
            let self_ptr = self as *mut Self;
            for (tile, mut sg) in &mut (*self_ptr).tile_to_subgroup {
                let dict = (*self_ptr).get_dict(tile.clone(), sg)?;
                rotations.insert(tile.clone(), dict);
            }
        }
        // for (tile, mut sg) in &mut self.tile_to_subgroup {
        //     let dict = self.get_dict(tile.clone(), sg)?;
        //     rotations.insert(tile.clone(), dict);
        // }

        let mut treatments: HashMap<Tile, TileRotationTreatment> = HashMap::new();
        for (tile, sg) in &mut self.tile_to_subgroup {
            if sg.treatment.is_some() {
                treatments.insert(tile.clone(), sg.treatment.clone().unwrap());
            }
        }
        Ok(TileRotation::build(
            rotations,
            treatments,
            self.default_treatment.clone(),
            self.rotation_group.clone(),
        ))
    }

    // For a given tile (found in a given rotation group)
    // Find the full set of tiles it rotates to.
    // Returns a new subgroup if one was generated.
    // TODO: Should be called get_hash?
    fn get_dict(&self, tile: Tile, sg: &mut SubGroup) -> Result<HashMap<Rotation, Tile>, TileRotationBuilderError> {
        let maybe_treatment = sg.treatment.as_ref();
        let treatment = if maybe_treatment.is_some() {maybe_treatment.unwrap().clone()} else {self.default_treatment.clone()};
        if treatment == TileRotationTreatment::Generated {
            self.generate(sg)?;
        }
        let rots = sg.get_rotations(tile.clone());
        let r1 = rots.first();
        if r1.is_none() {
            return Err(TileRotationBuilderError::Todo {msg: format!("No rotations for tile {}", tile.get_id().0)});
        }
        let r1 = r1.unwrap();
        let mut result = HashMap::new();

        for r2 in self.rotation_group.get_rotations() {
            let dest = sg.tiles.get(&r2);
            if dest.is_none() {
                continue;
            }
            result.insert(r1.inverse() * r2.clone(), dest.unwrap().clone());
        }

        Ok(result)
    }

    // Gets the rotation group containing Tile, creating it if it doesn't exist
    fn get_group(&mut self, tile: Tile) -> &mut SubGroup {
        self.tile_to_subgroup.entry(tile.clone()).or_insert_with(|| {
            let mut new_subgroup = SubGroup::new();
        new_subgroup.tiles.insert(Rotation::default(), tile.clone());
            new_subgroup
        })
    }

    // Gets the rotation group containing Tile, creating it if it doesn't exist
    fn ensure_exists(&mut self, tile: Tile) {
        self.tile_to_subgroup.entry(tile.clone()).or_insert_with(|| {
            let mut new_subgroup = SubGroup::new();
            new_subgroup.tiles.insert(Rotation::default(), tile.clone());
            new_subgroup
        });
    }

    fn set(sg: &mut SubGroup, rotation: Rotation, tile: Tile, action: &str) -> Result<bool, TileRotationBuilderError> {
        if sg.tiles.contains_key(&rotation) {
            if sg.tiles[&rotation] != tile {
                return Err(TileRotationBuilderError::SetTileConflict {action: action.to_string(), first: sg.tiles[&rotation].get_id().0, second: tile.get_id().0});
            }
            return Ok(false);
        }
        sg.tiles.insert(rotation, tile);
        Ok(true)
    }

    fn expand(sg: &mut SubGroup) -> Result<(), TileRotationBuilderError> {
        loop {
            let mut expanded = false;
            for entry in sg.entries.clone() {
                for (k, v) in sg.tiles.clone() {
                    if v == entry.src {
                        expanded = expanded || Self::set(sg, k.clone() * entry.rotation, entry.dest.clone(), "resolve conflicting rotations")?;
                    }
                    if v == entry.dest {
                        expanded = expanded || Self::set(sg, k.clone() * entry.rotation.inverse(), entry.src.clone(), "resolve conflicting rotations")?;
                    }
                }
            }
            if !expanded {
                break;
            }
        }
        Ok(())
    }


    // Fills all remaining slots with RotatedTile
    // Care is taken that as few distinct RotatedTiles are used as possible
    // If there's two possible choices, preference is given to rotations over reflections.
    fn generate(&self, sg: &mut SubGroup) -> Result<(), TileRotationBuilderError> {
        'start: loop {
            let refl_count = if self.rotation_group.is_reflectionally_symmetric() {2} else {1};
            for refl in 0..refl_count {
                for rot in (0..360).step_by(self.rotation_group.get_smallest_angle() as usize) {
                    let rotation = Rotation::new(rot, refl > 0);
                    if sg.tiles.contains_key(&rotation) {
                        continue;
                    }

                    // Found an empty spot, figure out what to rotate from
                    let refl_count_2 = if self.rotation_group.is_reflectionally_symmetric() {2} else {1};
                    for refl2 in 0..refl_count_2 {
                        for rot2 in (0..360).step_by(self.rotation_group.get_smallest_angle() as usize) {
                            let mut rotation2 = Rotation::new(rot2, (refl2 > 0) != (refl > 0));
                            let st = sg.tiles.get(&rotation2);
                            if st.is_none() {
                                continue;
                            }
                            let src_tile = st.unwrap();

                            if let Some(tile_rotation) = &src_tile.rotation {
                                rotation2 = rotation2 * tile_rotation.inverse();
                            }
                            let src_to_dest = rotation2.inverse() * rotation;
                            let dest_tile = if src_to_dest.get_reflect_x() == false && src_to_dest.get_rotate_cw() == 0 {
                                src_tile.clone()
                            } else {
                                Tile::new_rotated(src_tile.get_name().clone(), src_tile.get_value().clone(), src_to_dest)
                            };
                            Self::expand(sg)?;
                            continue 'start;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq)]
pub struct SubGroup {
    pub entries: Vec<Entry>,
    pub tiles: HashMap<Rotation, Tile>,
    pub treatment: Option<TileRotationTreatment>,
    pub treatment_set_by: Tile,
}

/// Stores a set of tiles related to each other by transformations.
/// If we have two key value pairs (k1, v1) and (k2, v2) in Tiles, then
/// we can apply rotation (k1.Inverse() * k2) to rotate v1 to v2.
impl SubGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            entries: Vec::new(),
            tiles: HashMap::new(),
            treatment: None,
            treatment_set_by: Tile::default(),
        }
    }

    // A tile may appear multiple times in a rotation group if it is symmetric in some way.
    pub fn get_rotations(&self, tile: Tile) -> Vec<Rotation> {
        self.tiles.iter()
            .filter(|(_, v)| **v == tile)
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn permute(&mut self, f: impl Fn(Rotation) -> Rotation) {
        self.tiles = self.tiles.iter()
            .map(|(k, v)| (f(k.clone()), v.clone()))
            .collect();
    }
}

impl Clone for SubGroup {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            tiles: self.tiles.clone(),
            treatment: self.treatment.clone(),
            treatment_set_by: self.treatment_set_by.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub src: Tile,
    pub rotation: Rotation,
    pub dest: Tile,
}

impl Entry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Self {
            src: Tile::default(),
            rotation: Rotation::default(),
            dest: Tile::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tile::ToTileVisual;
    use super::*;

    /// When you add a source tile, rotation, and destination tile,
    /// you cannot add the same source and rotation for another destination.
    /// Or, to put it more concisely, a source tile and rotation can only have one destination.
    #[test]
    pub fn rotation_builder_contradiction_1() {
        // Arrange
        let tile_1 = Tile::new("1".to_string(), 1.to_tile_visual());
        let tile_2 = Tile::new("2".to_string(), 2.to_tile_visual());
        let tile_3 = Tile::new("3".to_string(), 3.to_tile_visual());
        let rotation = Rotation::new(0, true);

        let mut builder = TileRotationBuilder::new(4, true, TileRotationTreatment::Unchanged);

        // Act
        let res_1 = builder.add(tile_1.clone(), rotation, tile_2);
        let res_2 = builder.add(tile_1.clone(), rotation, tile_3);

        // Assert
        assert!(res_1.is_ok());
        assert!(res_2.is_err());
    }

    /// Test for transitive conflicts.
    /// By adding tile 1 and 2, you are saying that tile 2 can go back to tile 1.
    /// Trying to add tile 2 as a source for a third tile is invalid;
    /// tile 2 cannot go back to both tile 1 and 3.
    #[test]
    pub fn rotation_builder_contradiction_2() {
        // Arrange
        let tile_1 = Tile::new("1".to_string(), 1.to_tile_visual());
        let tile_2 = Tile::new("2".to_string(), 2.to_tile_visual());
        let tile_3 = Tile::new("3".to_string(), 3.to_tile_visual());
        let rotation = Rotation::new(0, true);

        let mut builder = TileRotationBuilder::new(4, true, TileRotationTreatment::Unchanged);

        // Act
        let res_1 = builder.add(tile_1, rotation, tile_2.clone());
        let res_2 = builder.add(tile_2.clone(), rotation, tile_3);

        // Assert
        assert!(res_1.is_ok());
        assert!(res_2.is_err());
    }

    // TODO: Performance test with large, though relatively reasonable tiles.
    // Get this information by inspecting the state of TileRotationBuilder in C# when running
    // a sample.

    #[test]
    pub fn test_compounding() {
        // Arrange
        let tile_1 = Tile::new("1".to_string(), 1.to_tile_visual());
        let tile_2 = Tile::new("2".to_string(), 2.to_tile_visual());
        let rotation = Rotation::new(0, false);

        let mut builder = TileRotationBuilder::new(4, true, TileRotationTreatment::Unchanged);
        builder.add(tile_1.clone(), rotation, tile_2.clone());
        builder.add(tile_2.clone(), Rotation::new(90, false), tile_2.clone());

        // Act
        let result = builder.build();
        assert!(result.is_ok());
        let rot = result.unwrap();
        let mut rotrot = Rotation::new(1, false);
        let (b1, r1) = rot.rotate(&tile_1, &mut rotrot);

        // Assert
        assert!(r1.is_some());
        assert!(b1);
        assert_eq!(r1.unwrap(), tile_1);
    }


    // Boilerplate for log debugging infinite or hanging loops:
    /*
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {

            tx.send(()).unwrap();
        });
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .expect("Test timed out");
     */
}